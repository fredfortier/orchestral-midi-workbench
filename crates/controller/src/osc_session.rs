//! The bridge-owned OSC socket. Feedback describes a time window, not a reply ID.
use std::{
    collections::BTreeMap,
    io,
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::post,
};
use osc::{OscPacket, OscType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    net::UdpSocket,
    sync::{OwnedSemaphorePermit, Semaphore, mpsc, oneshot, watch},
    time::{Instant, timeout_at},
};

pub const DEFAULT_FEEDBACK_PORT: u16 = 9000;
pub const DEFAULT_WAIT_MS: u16 = 400;
pub const API_PATH: &str = "/v2/osc";
const MAX_WAIT_MS: u16 = 2_000;
const MAX_OBSERVATIONS: usize = 32;
const MAX_REQUEST_BYTES: usize = 16 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub address: String,
    pub arguments: Vec<String>,
    pub observe: Vec<String>,
    pub wait_ms: u16,
}

#[derive(Serialize)]
struct Receipt {
    submitted: bool,
    feedback: BTreeMap<String, Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_error: Option<String>,
}

struct Submission {
    bytes: Vec<u8>,
    observe: Vec<String>,
    window: Duration,
    reply: oneshot::Sender<io::Result<Receipt>>,
    _permit: OwnedSemaphorePermit,
}

#[derive(Clone)]
pub struct Dispatch {
    commands: mpsc::Sender<Submission>,
    available: Arc<Semaphore>,
}

pub struct Listener {
    socket: UdpSocket,
    target: SocketAddrV4,
    commands: mpsc::Receiver<Submission>,
}

impl Listener {
    pub async fn bind(target_port: u16, feedback_port: u16) -> io::Result<(Dispatch, Self)> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, feedback_port)).await?;
        let target = SocketAddrV4::new(Ipv4Addr::LOCALHOST, target_port);
        tracing::info!(%target, feedback=%socket.local_addr()?, "OSC session ready");
        let (commands, receiver) = mpsc::channel(1);
        Ok((
            Dispatch {
                commands,
                available: Arc::new(Semaphore::new(1)),
            },
            Self {
                socket,
                target,
                commands: receiver,
            },
        ))
    }

    pub async fn run(mut self, mut stop: watch::Receiver<bool>) -> io::Result<()> {
        let mut buffer = vec![0; 65_536];
        loop {
            tokio::select! {
                _ = stop.changed() => return Ok(()),
                received = self.socket.recv_from(&mut buffer) => {
                    // Discard unsolicited traffic while no observation is requested.
                    received?;
                }
                command = self.commands.recv() => {
                    let Some(command) = command else { return Ok(()) };
                    let result = self.submit(&command, &mut buffer).await;
                    let _ = command.reply.send(result);
                }
            }
        }
    }

    async fn submit(&self, command: &Submission, buffer: &mut [u8]) -> io::Result<Receipt> {
        self.socket.send_to(&command.bytes, self.target).await?;
        let mut receipt = Receipt {
            submitted: true,
            feedback: BTreeMap::new(),
            feedback_error: None,
        };
        if command.observe.is_empty() {
            return Ok(receipt);
        }
        let deadline = Instant::now() + command.window;
        while Instant::now() < deadline {
            let (size, peer) = match timeout_at(deadline, self.socket.recv_from(buffer)).await {
                Ok(Ok(datagram)) => datagram,
                Ok(Err(error)) => {
                    receipt.feedback_error = Some(error.to_string());
                    break;
                }
                Err(_) => break,
            };
            if peer.ip().is_loopback()
                && let Ok(packet) = osc::decode_datagram(&buffer[..size])
            {
                collect(packet, &command.observe, &mut receipt.feedback);
            }
        }
        Ok(receipt)
    }
}

fn collect(packet: OscPacket, observe: &[String], output: &mut BTreeMap<String, Vec<Value>>) {
    match packet {
        OscPacket::Message(message) if observe.contains(&message.addr) => {
            let values = message
                .args
                .into_iter()
                .map(|value| match value {
                    OscType::Int(value) => Value::from(value),
                    OscType::Float(value) => Value::from(f64::from(value)),
                    OscType::String(value) => Value::from(value),
                    OscType::Bool(value) => Value::from(value),
                    other => Value::from(format!("{other:?}")),
                })
                .collect();
            output.insert(message.addr, values);
        }
        OscPacket::Bundle(bundle) => {
            for packet in bundle.content {
                collect(packet, observe, output);
            }
        }
        OscPacket::Message(_) => {}
    }
}

pub fn router(dispatch: Dispatch) -> Router {
    Router::new()
        .route(API_PATH, post(submit))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .with_state(dispatch)
}

async fn submit(
    State(dispatch): State<Dispatch>,
    Json(request): Json<Request>,
) -> Result<Json<Receipt>, (StatusCode, String)> {
    let invalid = |error| (StatusCode::BAD_REQUEST, error);
    if !(1..=MAX_WAIT_MS).contains(&request.wait_ms) || request.observe.len() > MAX_OBSERVATIONS {
        return Err(invalid(format!(
            "Use at most {MAX_OBSERVATIONS} observation addresses and a 1–{MAX_WAIT_MS} ms window"
        )));
    }
    for address in &request.observe {
        osc::verify_address(address).map_err(|error| invalid(error.to_string()))?;
    }
    let bytes = osc::encode_command(request.address, &request.arguments).map_err(invalid)?;
    let permit = dispatch.available.try_acquire_owned().map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "OSC observation already in progress; command was not submitted".into(),
        )
    })?;
    let (reply, result) = oneshot::channel();
    dispatch
        .commands
        .try_send(Submission {
            bytes,
            observe: request.observe,
            window: Duration::from_millis(request.wait_ms.into()),
            reply,
            _permit: permit,
        })
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "OSC listener unavailable; command was not submitted".into(),
            )
        })?;
    result
        .await
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "OSC listener stopped; submission outcome unknown".into(),
            )
        })?
        .map(Json)
        .map_err(|error| (StatusCode::BAD_GATEWAY, format!("OSC send failed: {error}")))
}
