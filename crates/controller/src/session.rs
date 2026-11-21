//! One bidirectional session: observed document -> proposed plan -> observed result.
use crate::control::{Error as ControlError, Response as ControlOutcome};
use crate::controller::Command;
use crate::protocol::{ClientMessage, Document, Plan, ServerMessage, Snapshot, VERSION};
use std::{
    collections::{BTreeMap, HashMap},
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
    task::JoinSet,
    time::timeout,
};

pub const MAX_MESSAGE_BYTES: usize = crate::frame::MAX_ENCODED_BYTES;
/// Exact installed host build admitted by the native adapter probes.
pub const HOST_BUILD: &str = "6.1";

#[derive(Clone)]
pub struct Sessions {
    epoch: String,
    pub(crate) operations: Arc<crate::operations::Requests>,
    pub(crate) unresolved_loader: Arc<Mutex<Option<String>>>,
    pub(crate) application_gate: Arc<tokio::sync::Mutex<()>>,
    adapters: Arc<Mutex<HashMap<String, Adapter>>>,
    command_sequence: Arc<AtomicU64>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            operations: Arc::default(),
            application_gate: Arc::default(),
            unresolved_loader: Arc::default(),
            epoch: format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            adapters: Arc::default(),
            command_sequence: Arc::default(),
        }
    }
}

#[derive(Default)]
struct Adapter {
    lease: u64,
    high_water: u64,
    operations: BTreeMap<u64, Operation>,
    commands: Option<mpsc::Sender<Work>>,
}

struct Operation {
    input: Vec<u8>,
    document: String,
    state: ReceiptState,
}
enum ReceiptState {
    Pending { plan: Plan, snapshot: Box<Snapshot> },
    Finished(ServerMessage),
}
impl Operation {
    fn response(&self, operation: &str) -> ServerMessage {
        match &self.state {
            ReceiptState::Pending { plan, .. } => ServerMessage::Plan {
                operation: operation.into(),
                plan: plan.clone(),
            },
            ReceiptState::Finished(result) => result.clone(),
        }
    }
    fn finish(&mut self, result: ServerMessage) -> ServerMessage {
        self.state = ReceiptState::Finished(result.clone());
        result
    }
}

struct Work {
    request: String,
    command: Command,
    reply: oneshot::Sender<Result<ControlOutcome, ControlError>>,
}
struct PendingControl {
    work: Work,
    operation: Option<String>,
    plan: Option<Plan>,
}

impl PendingControl {
    /// Validate a correlated response against the dispatched command's result kind.
    fn observation(&self, message: ClientMessage) -> Result<ControlOutcome, String> {
        match (&self.work.command, message) {
            (Command::Mix { request }, ClientMessage::MixObservation { result, .. }) => {
                request.verify(&result)?;
                Ok(ControlOutcome::MixResult { result })
            }
            (
                Command::ExportAudio { request },
                ClientMessage::ExportAudioObservation { result, .. },
            ) => {
                request.verify(&result)?;
                Ok(ControlOutcome::ExportAudioResult { result })
            }
            (Command::Capture { path, .. }, ClientMessage::CaptureObservation { capture, .. }) => {
                capture.validate()?;
                if path
                    .as_ref()
                    .is_some_and(|p| !p.matches(&capture.entities.path))
                {
                    return Err("Capture scope differs from request".into());
                }
                Ok(ControlOutcome::CaptureResult { capture })
            }
            (
                Command::Selection { request },
                ClientMessage::SelectionObservation { result, .. },
            ) => {
                request.verify(&result)?;
                Ok(ControlOutcome::SelectionResult { result })
            }
            (Command::Info { path, .. }, ClientMessage::EntityObservation { entity, .. }) => {
                entity.validate()?;
                if path
                    .as_ref()
                    .is_some_and(|path| !path.matches(&entity.path))
                {
                    return Err("Entity path mismatch".into());
                }
                Ok(ControlOutcome::EntityResult { entity })
            }
            (Command::Cues { request }, ClientMessage::CuesObservation { result, .. }) => {
                crate::cues::verify_scoped(request, &result)?;
                Ok(ControlOutcome::CuesResult { result })
            }
            (
                Command::InstrumentNative { request },
                ClientMessage::InstrumentObservation { result, .. },
            ) => {
                if result["status"] != "rejected" {
                    crate::instrument::validate_native(request, &result)?;
                }
                Ok(ControlOutcome::InstrumentResult { result })
            }
            (
                _,
                ClientMessage::CommandRefused { reason, .. }
                | ClientMessage::CommandError { reason, .. },
            ) => Err(reason),
            (
                Command::Midi | Command::ReadClips { .. },
                ClientMessage::MidiObservation { midi, .. },
            ) => midi
                .validate()
                .map(|()| ControlOutcome::MidiResult { midi })
                .map_err(|reason| format!("Invalid native MIDI observation: {reason}")),
            _ => Err("Observation kind does not match dispatched command".into()),
        }
    }
}

fn command_request(message: &ClientMessage) -> Option<&str> {
    match message {
        ClientMessage::CaptureObservation { request, .. }
        | ClientMessage::MixObservation { request, .. }
        | ClientMessage::ExportAudioObservation { request, .. }
        | ClientMessage::SelectionObservation { request, .. }
        | ClientMessage::EntityObservation { request, .. }
        | ClientMessage::CuesObservation { request, .. }
        | ClientMessage::InstrumentObservation { request, .. }
        | ClientMessage::MidiObservation { request, .. }
        | ClientMessage::Observation { request, .. }
        | ClientMessage::CommandRefused { request, .. }
        | ClientMessage::CommandError { request, .. } => Some(request),
        _ => None,
    }
}

impl Sessions {
    pub(crate) fn next_operation(&self) -> String {
        format!(
            "{}:operation:{}",
            self.epoch,
            self.command_sequence.fetch_add(1, Ordering::Relaxed) + 1
        )
    }
    /// Active transport identities; readiness of a musical editor is established by a read.
    /// # Errors
    /// Reports unavailable session state.
    pub fn connected_adapters(&self) -> Result<Vec<String>, ControlError> {
        let adapters = self
            .adapters
            .lock()
            .map_err(|_| ControlError::unavailable("Session unavailable"))?;
        let mut names: Vec<_> = adapters
            .iter()
            .filter(|(_, adapter)| adapter.commands.as_ref().is_some_and(|c| !c.is_closed()))
            .map(|(name, _)| name.clone())
            .collect();
        names.sort();
        Ok(names)
    }

    fn resolve_adapter(&self, requested: Option<&str>) -> Result<String, ControlError> {
        let names = self.connected_adapters()?;
        let mut matches = names
            .into_iter()
            .filter(|name| requested.is_none_or(|requested| name == requested));
        let selected = matches
            .next()
            .ok_or_else(|| ControlError::unavailable("No matching connected adapter"))?;
        if matches.next().is_some() {
            return Err(ControlError::invalid(
                "Multiple adapters connected; name the intended adapter",
            ));
        }
        Ok(selected)
    }
    fn admit(&self, adapter: &str) -> Result<u64, String> {
        if adapter.is_empty()
            || adapter.len() > 80
            || !adapter
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err("Invalid adapter epoch".into());
        }
        let mut adapters = self
            .adapters
            .lock()
            .map_err(|_| "Session state unavailable")?;
        if !adapters.contains_key(adapter) && adapters.len() >= 8 {
            return Err("Adapter capacity reached".into());
        }
        let state = adapters.entry(adapter.into()).or_default();
        state.lease += 1;
        Ok(state.lease)
    }

    #[allow(clippy::too_many_lines)] // Keep the complete operation reconciliation transition together.
    fn process(
        &self,
        adapter: &str,
        lease: u64,
        message: ClientMessage,
    ) -> Result<Option<ServerMessage>, String> {
        {
            let adapters = self
                .adapters
                .lock()
                .map_err(|_| "Session state unavailable")?;
            if adapters
                .get(adapter)
                .is_none_or(|state| state.lease != lease)
            {
                return Err("Superseded connection".into());
            }
        }
        let (operation, input, proposal) = match &message {
            ClientMessage::Propose {
                operation,
                snapshot,
                plan,
            } => {
                let document: Document = snapshot.clone().try_into()?;
                document.validate_plan(plan)?;
                (
                    operation,
                    serde_json::to_vec(&(snapshot, plan)).map_err(|error| error.to_string())?,
                    Some((plan.clone(), snapshot.clone())),
                )
            }
            ClientMessage::Hello { .. } => return Err("Hello already received".into()),
            ClientMessage::CaptureObservation { .. }
            | ClientMessage::MixObservation { .. }
            | ClientMessage::ExportAudioObservation { .. }
            | ClientMessage::SelectionObservation { .. }
            | ClientMessage::EntityObservation { .. }
            | ClientMessage::CuesObservation { .. }
            | ClientMessage::InstrumentObservation { .. }
            | ClientMessage::MidiObservation { .. }
            | ClientMessage::Observation { .. }
            | ClientMessage::CommandRefused { .. }
            | ClientMessage::CommandError { .. } => return Err("Unexpected control message".into()),
            ClientMessage::Status { operation }
            | ClientMessage::Applied { operation, .. }
            | ClientMessage::Rejected { operation, .. } => (operation, Vec::new(), None),
        };
        let sequence = operation
            .strip_prefix(&format!("{adapter}:"))
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n > 0)
            .ok_or("Invalid operation identity")?;
        let mut adapters = self
            .adapters
            .lock()
            .map_err(|_| "Session state unavailable")?;
        let state = adapters.get(adapter).ok_or("Unknown adapter")?;
        if lease != state.lease {
            return Err("Superseded connection".into());
        }
        if let Some((plan, snapshot)) = proposal {
            if let Some(previous) = state.operations.get(&sequence) {
                if previous.input != input {
                    return Err("Operation identity conflicts with frozen input".into());
                }
                return Ok(Some(previous.response(operation)));
            }
            if sequence <= state.high_water {
                return Err("Expired operation; observe and submit a new operation".into());
            }
            if state
                .operations
                .values()
                .any(|op| matches!(op.state, ReceiptState::Pending { .. }))
            {
                return Err("An operation still requires reconciliation".into());
            }
            let state = adapters.get_mut(adapter).ok_or("Unknown adapter")?;
            while state.operations.len() >= 16 {
                state.operations.pop_first();
            }
            state.high_water = sequence;
            state.operations.insert(
                sequence,
                Operation {
                    input,
                    document: snapshot.document.clone(),
                    state: ReceiptState::Pending {
                        plan: plan.clone(),
                        snapshot: Box::new(snapshot),
                    },
                },
            );
            return Ok(Some(ServerMessage::Plan {
                operation: operation.clone(),
                plan,
            }));
        }
        let state = adapters.get_mut(adapter).ok_or("Unknown adapter")?;
        let Some(pending) = state.operations.get_mut(&sequence) else {
            return Ok(Some(ServerMessage::Unknown {
                operation: operation.clone(),
            }));
        };
        match message {
            ClientMessage::Applied {
                operation,
                snapshot,
                identities,
            } => {
                if snapshot.document != pending.document {
                    return Err("Applied result belongs to another document".into());
                }
                if let ReceiptState::Finished(result) = &pending.state {
                    return Ok(Some(result.clone()));
                }
                let observed: Document = snapshot.try_into()?;
                let ReceiptState::Pending { plan, snapshot } = &pending.state else {
                    unreachable!()
                };
                let original: Document = snapshot.as_ref().clone().try_into()?;
                original.verify_result(plan, &observed.snapshot, &identities)?;
                Ok(Some(pending.finish(ServerMessage::Complete { operation })))
            }
            ClientMessage::Rejected { operation, reason } => {
                if let ReceiptState::Finished(result) = &pending.state {
                    return Ok(Some(result.clone()));
                }
                Ok(Some(pending.finish(ServerMessage::Error {
                    operation: Some(operation),
                    reason,
                })))
            }
            ClientMessage::Status { operation } => Ok(Some(pending.response(&operation))),
            _ => unreachable!(),
        }
    }
}

const IO_DEADLINE: Duration = Duration::from_secs(10);
const MAX_CONNECTIONS: usize = 8;

/// Read a complete bounded UTF-8 JSON frame. Idle established sessions may wait
/// indefinitely, but a peer cannot hold a partial header/payload indefinitely.
async fn receive(stream: &mut TcpStream) -> io::Result<ClientMessage> {
    let first = stream.read_u8().await?;
    receive_after_first(stream, first).await
}
async fn receive_after_first(stream: &mut TcpStream, first: u8) -> io::Result<ClientMessage> {
    timeout(IO_DEADLINE, async {
        let mut header = [first, 0, 0, 0];
        stream.read_exact(&mut header[1..]).await?;
        let length = u32::from_be_bytes(header) as usize;
        if length == 0 || length > MAX_MESSAGE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid frame length",
            ));
        }
        let mut bytes = vec![0; length];
        stream.read_exact(&mut bytes).await?;
        let bytes = crate::frame::decode(bytes)?;
        tracing::debug!(
            direction = "incoming",
            encoded_bytes = length,
            decoded_bytes = bytes.len(),
            "session frame"
        );
        serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    })
    .await
    .map_err(io::Error::other)?
}

async fn send(stream: &mut TcpStream, message: &ServerMessage) -> io::Result<()> {
    let bytes = serde_json::to_vec(message).map_err(io::Error::other)?;
    let decoded_bytes = bytes.len();
    let bytes = crate::frame::encode(bytes)?;
    tracing::debug!(
        direction = "service_to_adapter",
        decoded_bytes,
        encoded_bytes = bytes.len(),
        "session wire"
    );
    let length = u32::try_from(bytes.len()).map_err(io::Error::other)?;
    timeout(IO_DEADLINE, async {
        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&bytes).await
    })
    .await
    .map_err(io::Error::other)?
}

/// Dispatch typed intent through the existing adapter owner; no network forwarding.
/// # Errors
/// Reports dispatch refusal or an uncertain outcome after dispatch.
pub async fn control(
    state: &Sessions,
    adapter: Option<String>,
    command: Command,
) -> Result<ControlOutcome, ControlError> {
    if let Command::OperationStatus { request_id } = command {
        return crate::operations::inspect(state, &request_id)
            .await
            .map(|operation| ControlOutcome::OperationResult { operation });
    }
    if let Command::Instrument {
        request: crate::instrument::Request::Read { .. } | crate::instrument::Request::Status { .. },
    } = &command
    {
        if let Command::Instrument { request } = command {
            return crate::instrument::execute(state, adapter, request)
                .await
                .map(|result| ControlOutcome::InstrumentResult { result });
        }
    }
    if !command.mutates() {
        return dispatch(state, adapter, command).await;
    }
    let _guard = state.application_gate.lock().await;
    if let Some(operation) = state
        .unresolved_loader
        .lock()
        .map_err(|_| ControlError::unavailable("Loader state unavailable"))?
        .as_ref()
    {
        let resumes = matches!(&command, Command::Instrument { request }
            if crate::instrument::resumes_blocked_loader(request, operation));
        if !resumes {
            return Err(ControlError::unknown(format!(
                "External load {operation} requires reconciliation; no further write dispatched"
            )));
        }
    }
    let adapter = Some(state.resolve_adapter(adapter.as_deref())?);
    if let Command::ReplaceClips { before, after } = &command {
        if before == after {
            let read = if before.selection.editor == "paths" {
                Command::ReadClips {
                    paths: before
                        .paths
                        .clone()
                        .ok_or_else(|| ControlError::invalid("Missing explicit content scope"))?,
                }
            } else {
                Command::Midi
            };
            let ControlOutcome::MidiResult { midi } = dispatch(state, adapter, read).await? else {
                return Err(ControlError::unknown("Unexpected clip read"));
            };
            let mut current = midi.as_ref().clone();
            current.revision.clone_from(&before.revision);
            if &current != before.as_ref() {
                return Err(ControlError::invalid(
                    "Clip content changed before no-op replacement",
                ));
            }
            return Ok(ControlOutcome::ControlResult {
                result: Box::new(crate::controller::Outcome {
                    snapshot: *midi,
                    plan: None,
                    identities: BTreeMap::new(),
                }),
            });
        }
    }
    if let Command::Instrument { request } = command {
        return crate::instrument::execute(state, adapter, request)
            .await
            .map(|result| ControlOutcome::InstrumentResult { result });
    }
    dispatch(state, adapter, command).await
}

pub(crate) async fn dispatch(
    state: &Sessions,
    adapter: Option<String>,
    command: Command,
) -> Result<ControlOutcome, ControlError> {
    let sender = {
        let adapters = state
            .adapters
            .lock()
            .map_err(|_| ControlError::unavailable("Session unavailable"))?;
        let mut active = adapters
            .iter()
            .filter(|(name, _)| adapter.as_ref().is_none_or(|id| id == *name))
            .filter_map(|(_, state)| state.commands.as_ref().filter(|sender| !sender.is_closed()));
        let sender = active
            .next()
            .ok_or_else(|| ControlError::unavailable(format!("No connected adapter matches service wire {VERSION}; require the loaded production adapter at that version and admitted host build {HOST_BUILD}. Deployment alone does not reload running extension classes")))?;
        if active.next().is_some() {
            return Err(ControlError::invalid(
                "Multiple adapters connected; supply --adapter",
            ));
        }
        sender.clone()
    };
    let (reply, result) = oneshot::channel();
    let sequence = state.command_sequence.fetch_add(1, Ordering::Relaxed) + 1;
    let request = format!("{}:command:{sequence}", state.epoch);
    sender
        .try_send(Work {
            request,
            command,
            reply,
        })
        .map_err(|_| ControlError::unavailable("Adapter busy or disconnected"))?;
    timeout(Duration::from_secs(40), result)
        .await
        .map_err(|_| ControlError::deadline())?
        .map_err(|_| ControlError::unknown("Adapter disconnected; fresh observation required"))?
}

#[allow(clippy::too_many_lines)] // Keep connection ownership, command correlation and receipt transitions together.
async fn run(mut stream: TcpStream, state: Sessions) -> io::Result<()> {
    stream.set_nodelay(true)?;
    let first = timeout(IO_DEADLINE, receive(&mut stream))
        .await
        .map_err(io::Error::other)??;
    let ClientMessage::Hello {
        version,
        adapter,
        host_build,
    } = first
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Hello or control required",
        ));
    };
    if version != VERSION || host_build != HOST_BUILD {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Unadmitted adapter: observed wire {version}, expected {VERSION}; observed host {host_build}, expected {HOST_BUILD}"
            ),
        ));
    }
    let lease = state.admit(&adapter).map_err(io::Error::other)?;
    let (commands, mut receiver) = mpsc::channel(1);
    {
        let mut adapters = state
            .adapters
            .lock()
            .map_err(|_| io::Error::other("Session unavailable"))?;
        adapters
            .get_mut(&adapter)
            .ok_or_else(|| io::Error::other("Unknown adapter"))?
            .commands = Some(commands);
    }
    send(
        &mut stream,
        &ServerMessage::Welcome {
            version: VERSION,
            epoch: state.epoch.clone(),
            max_message_bytes: MAX_MESSAGE_BYTES,
        },
    )
    .await?;
    let mut pending: Option<PendingControl> = None;
    loop {
        // The receive future must not be cancelled after consuming a partial frame.
        // Select once on the first byte, then read the remaining frame atomically.
        let mut first = [0u8; 1];
        tokio::select! {
            read=stream.read_exact(&mut first) => { read?; },
            Some(work)=receiver.recv() => {
                if work.reply.is_closed(){continue;}
                if pending.is_some(){let _=work.reply.send(Err(ControlError::unavailable("Adapter already has a CLI operation")));continue;}
                {
                    let adapters=state.adapters.lock().map_err(|_|io::Error::other("Session unavailable"))?;
                    if adapters.get(&adapter).is_none_or(|s|s.lease!=lease) {
                        let _=work.reply.send(Err(ControlError::unavailable("Superseded adapter connection")));continue;
                    }
                }
                let command = match work.command.document_command() {
                    Ok(command) => command,
                    Err(reason) => {let _=work.reply.send(Err(ControlError::invalid(reason)));continue;}
                };
                send(&mut stream,&ServerMessage::DocumentRequest{request:work.request.clone(),command}).await?;
                pending=Some(PendingControl{work,operation:None,plan:None});continue;
            },
        }
        let message = receive_after_first(&mut stream, first[0]).await?;
        tracing::debug!(direction = "adapter_to_service", "session message received");
        if let Some(request) = command_request(&message) {
            let p = pending
                .as_ref()
                .filter(|p| p.work.request == request)
                .ok_or_else(|| io::Error::other("Unsolicited command observation"))?;
            if !(p.work.command.edits() && matches!(message, ClientMessage::Observation { .. })) {
                let refused = matches!(message, ClientMessage::CommandRefused { .. });
                let command_receipt = matches!(
                    message,
                    ClientMessage::CommandRefused { .. } | ClientMessage::CommandError { .. }
                );
                let result = p.observation(message);
                // Native command failures remain queryable. Invalid read observations
                // retire this connection before any later edit authority.
                let invalid = if command_receipt {
                    None
                } else {
                    result.as_ref().err().cloned()
                };
                if let Some(p) = pending.take() {
                    let _ = p.work.reply.send(result.map_err(|reason| {
                        if refused {
                            ControlError::invalid(reason)
                        } else {
                            ControlError::unknown(reason)
                        }
                    }));
                }
                if let Some(reason) = invalid {
                    return Err(io::Error::other(reason));
                }
                continue;
            }
        }
        let message = if let ClientMessage::Observation {
            request,
            operation,
            snapshot,
        } = message
        {
            let Some(p) = pending.as_mut().filter(|p| p.work.request == request) else {
                return Err(io::Error::other("Unsolicited command observation"));
            };
            if p.operation.is_some() && p.operation != operation {
                return Err(io::Error::other("Conflicting command operation identity"));
            }
            let Some(operation) = operation else {
                return Err(io::Error::other("Missing edit operation identity"));
            };
            let plan = match p.work.command.plan(&snapshot) {
                Ok(plan) => plan,
                Err(reason) => {
                    send(
                        &mut stream,
                        &ServerMessage::Error {
                            operation: Some(operation),
                            reason: reason.clone(),
                        },
                    )
                    .await?;
                    if let Some(p) = pending.take() {
                        let _ = p.work.reply.send(Err(ControlError::invalid(reason)));
                    }
                    continue;
                }
            };
            p.operation = Some(operation.clone());
            p.plan = Some(plan.clone());
            ClientMessage::Propose {
                operation,
                snapshot,
                plan,
            }
        } else {
            message
        };
        let operation = match &message {
            ClientMessage::Propose { operation, .. }
            | ClientMessage::Applied { operation, .. }
            | ClientMessage::Rejected { operation, .. }
            | ClientMessage::Status { operation } => Some(operation.clone()),
            _ => None,
        };
        let observed = if let ClientMessage::Applied {
            snapshot,
            identities,
            ..
        } = &message
        {
            Some((snapshot.clone(), identities.clone()))
        } else {
            None
        };
        let response = match state.process(&adapter, lease, message) {
            Ok(Some(message)) => message,
            Ok(None) => continue,
            Err(reason) => ServerMessage::Error {
                operation: operation.clone(),
                reason,
            },
        };
        send(&mut stream, &response).await?;
        if pending
            .as_ref()
            .is_some_and(|p| p.operation.is_some() && p.operation == operation)
        {
            let result = match &response {
                ServerMessage::Complete { .. } => observed.map(|(snapshot, identities)| {
                    Ok(ControlOutcome::ControlResult {
                        result: Box::new(crate::controller::Outcome {
                            snapshot,
                            plan: pending.as_ref().and_then(|p| p.plan.clone()),
                            identities,
                        }),
                    })
                }),
                ServerMessage::Error { reason, .. } => Some(Err(reason.clone())),
                _ => None,
            };
            if let Some(result) = result {
                if let Some(p) = pending.take() {
                    let _ = p.work.reply.send(result.map_err(ControlError::unknown));
                }
            }
        }
    }
}

/// The listener is loopback-only. At most eight connections retain frame buffers;
/// each connection processes one frame at a time, with no unbounded send queue.
/// Shutdown drops all sockets and cancels pending I/O, preserving operation receipts
/// in `Sessions` for the lifetime of this service (never across process restart).
///
/// # Errors
/// Returns listener I/O errors or refuses a non-loopback listener.
pub async fn serve(
    listener: TcpListener,
    sessions: Sessions,
    shutdown: impl std::future::Future<Output = ()>,
) -> io::Result<()> {
    if !listener.local_addr()?.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Loopback listener required",
        ));
    }
    tokio::pin!(shutdown);
    let mut tasks = JoinSet::new();
    loop {
        tokio::select! {
            () = &mut shutdown => break,
            Some(_) = tasks.join_next(), if !tasks.is_empty() => {},
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                if tasks.len() >= MAX_CONNECTIONS {
                    drop(stream);
                    continue;
                }
                let state = sessions.clone();
                tasks.spawn(async move {
                    if let Err(error) = run(stream, state).await {
                        tracing::debug!(%error, "Editor session closed");
                    }
                });
            }
        }
    }
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    Ok(())
}
