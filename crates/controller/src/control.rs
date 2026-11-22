//! Transport-independent application commands, results and dispatch failures.
use crate::controller::{Command, Outcome};
use serde::{Deserialize, Serialize};

/// Complete edit-result admission budget, independent of either transport codec.

#[derive(Debug, Deserialize, Serialize)]
#[serde(try_from = "RawRequest")]
pub struct Request {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub adapter: Option<String>,
    pub command: Command,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequest {
    #[serde(rename = "requestId")]
    request_id: String,
    adapter: Option<String>,
    command: serde_json::Value,
}
impl TryFrom<RawRequest> for Request {
    type Error = String;
    fn try_from(raw: RawRequest) -> Result<Self, Self::Error> {
        if raw.request_id.is_empty() || raw.request_id.len() > 128 || raw.request_id.contains('\0')
        {
            return Err("A bounded nonempty requestId is required".into());
        }
        let single_field = raw.command.as_object().is_some_and(|o| o.len() == 1);
        let command: Command = serde_json::from_value(raw.command).map_err(|e| e.to_string())?;
        // Serde's internally tagged unit variants otherwise accept extra fields.
        if matches!(command, Command::Midi) && !single_field {
            return Err("Unknown fields in read command".into());
        }
        Ok(Self {
            request_id: raw.request_id,
            adapter: raw.adapter,
            command,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Response {
    LauncherResult {
        result: crate::launcher::Observation,
    },
    SelectionResult {
        result: crate::selection::Observation,
    },
    CaptureResult {
        capture: Box<crate::export_midi::Capture>,
    },
    MixResult {
        result: crate::mix::Outcome,
    },
    ExportAudioResult {
        result: serde_json::Value,
    },
    EntityResult {
        entity: crate::entity::Entity,
    },
    CuesResult {
        result: crate::cues::Outcome,
    },
    OperationResult {
        operation: serde_json::Value,
    },
    InstrumentResult {
        result: serde_json::Value,
    },
    ControlResult {
        result: Box<Outcome>,
    },
    MidiResult {
        midi: Box<crate::midi::Observation>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Code {
    InvalidRequest,
    Unavailable,
    Deadline,
    ExecutionFailed,
    ResourceLimit,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureOutcome {
    NotDispatched,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Error {
    pub code: Code,
    pub outcome: FailureOutcome,
    pub message: String,
}

impl Error {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: Code::InvalidRequest,
            outcome: FailureOutcome::NotDispatched,
            message: message.into(),
        }
    }
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: Code::Unavailable,
            outcome: FailureOutcome::NotDispatched,
            message: message.into(),
        }
    }
    pub fn unknown(message: impl Into<String>) -> Self {
        Self {
            code: Code::ExecutionFailed,
            outcome: FailureOutcome::Unknown,
            message: message.into(),
        }
    }
    #[must_use]
    pub fn deadline() -> Self {
        Self {
            code: Code::Deadline,
            outcome: FailureOutcome::Unknown,
            message: "Command deadline expired; fresh observation required; do not retry mutations"
                .into(),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (outcome: {:?})", self.message, self.outcome)
    }
}
impl std::error::Error for Error {}
