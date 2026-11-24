//! Loopback HTTP boundary; the SDK adapter retains its independent framed TCP session.
use crate::{control, session::Sessions};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{io, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Semaphore, time::timeout};

pub const DEFAULT_PORT: u16 = 8789;
pub const VERSION: u8 = 2;
const API_HEADER: &str = "x-midi-workbench-api-version";
const MAX_BODY_BYTES: usize = crate::frame::MAX_ENCODED_BYTES;
// One plain JSON limit on both transports.
const MAX_RESPONSE_BYTES: usize = crate::frame::MAX_DECODED_BYTES;
const MAX_COMMANDS: usize = 8;

#[derive(Clone)]
struct Api {
    sessions: Sessions,
    capacity: Arc<Semaphore>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Failure {
    error: control::Error,
}

impl IntoResponse for control::Error {
    fn into_response(self) -> Response {
        let status = match self.code {
            control::Code::InvalidRequest => StatusCode::BAD_REQUEST,
            control::Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            control::Code::Deadline => StatusCode::GATEWAY_TIMEOUT,
            control::Code::ExecutionFailed => StatusCode::BAD_GATEWAY,
            control::Code::ResourceLimit => StatusCode::PAYLOAD_TOO_LARGE,
        };
        (status, axum::Json(Failure { error: self })).into_response()
    }
}

async fn command(State(api): State<Api>, request: Request) -> Result<Response, control::Error> {
    if let Some(version) = request.headers().get(API_HEADER) {
        if version.to_str().ok() != Some(VERSION.to_string().as_str()) {
            return Err(control::Error::unavailable(format!(
                "CLI/service API mismatch: service API {VERSION}; rebuild matching CLI/service"
            )));
        }
    }
    let permit = api
        .capacity
        .try_acquire_owned()
        .map_err(|_| control::Error::unavailable("Client command capacity reached"))?;
    // Dropping an HTTP connection must not cancel an accepted document operation.
    // The owned permit bounds detached work until reconciliation/serialization ends.
    tokio::spawn(async move {
        let _permit = permit;
        if request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.split(';').next())
            .map(str::trim)
            != Some("application/json")
        {
            return Err(control::Error::invalid(
                "Content-Type must be application/json",
            ));
        }
        let bytes = timeout(
            Duration::from_secs(5),
            to_bytes(request.into_body(), MAX_BODY_BYTES),
        )
        .await
        .map_err(|_| control::Error::invalid("Request body deadline expired"))?
        .map_err(|e| control::Error::invalid(format!("Invalid or oversized request body: {e}")))?;
        let input: control::Request =
            serde_json::from_slice(&bytes).map_err(|e| control::Error::invalid(e.to_string()))?;
        let result = crate::operations::submit(&api.sessions, input).await?;
        let bytes = encode(&result, MAX_RESPONSE_BYTES).map_err(|e| control::Error {
            code: control::Code::ResourceLimit,
            outcome: control::FailureOutcome::Unknown,
            message: e.to_string(),
        })?;
        Ok((
            [(header::CONTENT_TYPE, "application/json")],
            Body::from(bytes),
        )
            .into_response())
    })
    .await
    .map_err(|e| control::Error::unknown(e.to_string()))?
}

fn encode(value: &impl Serialize, limit: usize) -> io::Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > limit {
        return Err(io::Error::other("JSON message exceeds transport limit"));
    }
    Ok(bytes)
}

async fn health(State(api): State<Api>) -> Result<axum::Json<serde_json::Value>, control::Error> {
    Ok(axum::Json(
        serde_json::json!({"apiVersion":VERSION,"wireVersion":crate::protocol::VERSION,"adapters":api.sessions.connected_adapters()?}),
    ))
}

/// Serve the public API using the same session owner as the SDK listener.
/// # Errors
/// Refuses non-loopback listeners and reports server I/O errors.
pub async fn serve(
    listener: TcpListener,
    sessions: Sessions,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    serve_with_routes(listener, sessions, Router::new(), shutdown).await
}

/// Serve the public API alongside additional loopback routes.
/// # Errors
/// Refuses non-loopback listeners and reports server I/O errors.
pub async fn serve_with_routes(
    listener: TcpListener,
    sessions: Sessions,
    routes: Router,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    if !listener.local_addr()?.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Loopback listener required",
        ));
    }
    let app = Router::new()
        .route("/v2/commands", post(command))
        .route("/v2/health", get(health))
        .with_state(Api {
            sessions,
            capacity: Arc::new(Semaphore::new(MAX_COMMANDS)),
        })
        .merge(routes);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

/// Submit one command without redirects, environment proxies or automatic retries.
/// # Errors
/// Reports a typed service failure, transport loss or invalid/oversized JSON.
/// Transport loss after sending leaves mutation outcome unknown.
pub async fn exchange(
    port: u16,
    adapter: Option<String>,
    command: crate::controller::Command,
) -> Result<control::Response, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(45))
        .build()?;
    // Older services can accept the command yet return an incompatible DTO.
    // Check before dispatch, including for mutations; never retry the command.
    let health = client
        .get(format!("http://127.0.0.1:{port}/v2/health"))
        .send()
        .await
        .map_err(|e| control::Error::unavailable(format!("Controller health check failed: {e}")))?;
    if !health.status().is_success() {
        return Err(control::Error::unavailable(format!(
            "Controller health check returned {}; command not dispatched",
            health.status()
        ))
        .into());
    }
    let bytes = read_body(health, 64 * 1024).await.map_err(|e| {
        control::Error::unavailable(format!("Invalid controller health response: {e}"))
    })?;
    let health: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        control::Error::unavailable(format!("Invalid controller health response: {e}"))
    })?;
    if health["apiVersion"] != VERSION || health["wireVersion"] != crate::protocol::VERSION {
        return Err(control::Error::unavailable(format!(
            "CLI/service version mismatch: CLI requires API 2 / wire {}, service reports API {} / wire {}; rebuild and install matching CLI, service and adapter, then restart only the service and reload the adapter; command not dispatched",
            crate::protocol::VERSION, health["apiVersion"], health["wireVersion"]
        )).into());
    }
    let request_id = match &command {
        crate::controller::Command::Cues {
            request: crate::cues::Request::Create { operation, .. },
        } => Some(operation.clone()),
        crate::controller::Command::Instrument { request } => {
            request.operation().map(str::to_owned)
        }
        _ => None,
    }
    .unwrap_or_else(|| {
        format!(
            "cli-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
    });
    let wait = command.mutates();
    let input = control::Request {
        request_id: request_id.clone(),
        adapter: adapter.clone(),
        command,
    };
    let mut response = exchange_once(&client, port, &input).await?.response;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(900);
    let mut poll = 0u64;
    loop {
        let control::Response::OperationResult { operation } = &response else {
            return Ok(response);
        };
        if !wait {
            return Ok(response);
        }
        if operation["state"] != "pending" {
            if !operation["failure"].is_null() {
                return Err(serde_json::from_value::<control::Error>(
                    operation["failure"].clone(),
                )?
                .into());
            }
            return Ok(serde_json::from_value(operation["response"].clone())?);
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(control::Error::unknown(format!("Application operation still pending; request ID {request_id}; query original status")).into());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
        poll += 1;
        let status = control::Request {
            request_id: format!("status-{}-{poll}", std::process::id()),
            adapter: adapter.clone(),
            command: crate::controller::Command::OperationStatus {
                request_id: request_id.clone(),
            },
        };
        response=exchange_once(&client,port,&status).await.map_err(|e|control::Error::unknown(format!("Status unavailable for original request {request_id}: {e}; never replay the mutation")))?.response;
    }
}
async fn exchange_once(
    client: &reqwest::Client,
    port: u16,
    input: &control::Request,
) -> Result<crate::operations::Reply, Box<dyn std::error::Error>> {
    let request_id = &input.request_id;
    let response = client
        .post(format!("http://127.0.0.1:{port}/v2/commands"))
        .header(API_HEADER, VERSION.to_string())
        .header(header::CONTENT_TYPE, "application/json")
        .body(encode(input, MAX_BODY_BYTES)?)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                control::Error::unavailable(format!("Controller HTTP connection failed: {e}"))
            } else {
                control::Error::unknown(format!(
                    "Controller HTTP exchange failed: {e}; request ID {request_id}; reconcile status, do not retry mutations"
                ))
            }
        })?;
    let status = response.status();
    let bytes = read_body(response, MAX_RESPONSE_BYTES).await.map_err(|e| {
        control::Error::unknown(format!(
            "Controller response lost: {e}; request ID {request_id}; mutation outcome may be unknown"
        ))
    })?;
    if !status.is_success() {
        let failure: Failure = serde_json::from_slice(&bytes).map_err(|e| {
            control::Error::unknown(format!("Invalid controller failure response: {e}"))
        })?;
        return Err(Box::new(failure.error));
    }
    let reply: crate::operations::Reply = serde_json::from_slice(&bytes).map_err(|e| {
        control::Error::unknown(format!(
            "Invalid controller result: {e}; request ID {request_id}"
        ))
    })?;
    if reply.request_id != *request_id {
        return Err(control::Error::unknown("Response request ID differs").into());
    }
    Ok(reply)
}

async fn read_body(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > limit.saturating_sub(bytes.len()) {
            return Err("Client response exceeds capacity".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
