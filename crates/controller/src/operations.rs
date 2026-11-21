//! Bounded process-lifetime application operations. No cached DAW replica or replay on reconnect.
use crate::{control, controller::Command, session::Sessions};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Mutex};
use tokio::sync::watch;

const MAX_ENTRIES: usize = 4096;
#[derive(Default)]
pub(crate) struct Requests {
    entries: Mutex<BTreeMap<String, Entry>>,
}
struct Entry {
    intent: Vec<u8>,
    operation: String,
    stage: watch::Sender<Option<Result<control::Response, control::Error>>>,
    progress: Value,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Reply {
    pub request_id: String,
    pub operation_id: Option<String>,
    pub response: control::Response,
}
impl Requests {
    pub(crate) fn status(&self, request: &str) -> Result<Value, control::Error> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| control::Error::unavailable("Operation state unavailable"))?;
        let entry=entries.get(request).ok_or_else(||control::Error::invalid("Unknown request ID in this service lifetime; reconcile affected state before another write"))?;
        let result = entry.stage.borrow();
        let (state, response, failure) = match result.as_ref() {
            None => ("pending", None, None),
            Some(Ok(response)) => ("verified", Some(response), None),
            Some(Err(error)) => (
                if matches!(error.outcome, control::FailureOutcome::Unknown) {
                    "outcomeUnknown"
                } else {
                    "failed"
                },
                None,
                Some(error),
            ),
        };
        Ok(
            json!({"requestId":request,"operationId":entry.operation,"state":state,"response":response,"failure":failure,"progress":entry.progress}),
        )
    }

    pub(crate) fn progress(
        &self,
        operation: &str,
        stage: &str,
        child: Option<&str>,
    ) -> Result<(), control::Error> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| control::Error::unavailable("Operation state unavailable"))?;
        if let Some(entry) = entries
            .values_mut()
            .find(|entry| entry.operation == operation)
        {
            entry.progress = json!({"stage":stage,"childOperation":child});
        }
        Ok(())
    }
}
/// Claim identical requests atomically, then retain a detached execution through HTTP loss.
pub(crate) async fn submit(
    state: &Sessions,
    input: control::Request,
) -> Result<Reply, control::Error> {
    if !input.command.mutates() {
        return crate::session::control(state, input.adapter, input.command)
            .await
            .map(|response| Reply {
                request_id: input.request_id,
                operation_id: None,
                response,
            });
    }
    let intent = serde_json::to_vec(&input).map_err(|e| control::Error::invalid(e.to_string()))?;
    let (operation, mut receiver) = {
        let mut entries = state
            .operations
            .entries
            .lock()
            .map_err(|_| control::Error::unavailable("Operation state unavailable"))?;
        if let Some(entry) = entries.get(&input.request_id) {
            if entry.intent != intent {
                return Err(control::Error::invalid(
                    "Request ID reused with different intent or target",
                ));
            }
            (entry.operation.clone(), entry.stage.subscribe())
        } else {
            if entries.len() >= MAX_ENTRIES {
                return Err(control::Error::unavailable(
                    "Operation history is full; no write dispatched",
                ));
            }
            let operation = state.next_operation();
            let (tx, rx) = watch::channel(None);
            entries.insert(
                input.request_id.clone(),
                Entry {
                    intent,
                    operation: operation.clone(),
                    stage: tx.clone(),
                    progress: json!({"stage":"admitted"}),
                },
            );
            let state = state.clone();
            let execution_operation = operation.clone();
            tokio::spawn(async move {
                let mut command = input.command;
                if let Command::Instrument { request } = &mut command {
                    request.bind_operation(&execution_operation);
                }
                let result = crate::session::control(&state, input.adapter, command).await;
                tx.send_replace(Some(result));
            });
            (operation, rx)
        }
    };
    loop {
        if let Some(result) = receiver.borrow().clone() {
            return result
                .map(|response| Reply {
                    request_id: input.request_id.clone(),
                    operation_id: Some(operation.clone()),
                    response,
                })
                .map_err(|mut error| {
                    error.message = format!(
                        "{}; request ID {}; operation {}",
                        error.message, input.request_id, operation
                    );
                    error
                });
        }
        match tokio::time::timeout(std::time::Duration::from_millis(200), receiver.changed()).await
        {
            Ok(Ok(())) => {}
            Ok(Err(_)) => {
                return Err(control::Error::unknown(format!(
                    "Operation worker lost; request ID {}; operation {operation}; do not replay",
                    input.request_id
                )));
            }
            Err(_) => {
                return Ok(Reply {
                    request_id: input.request_id.clone(),
                    operation_id: Some(operation),
                    response: control::Response::OperationResult {
                        operation: state.operations.status(&input.request_id)?,
                    },
                });
            }
        }
    }
}

/// Reconcile retained instrument execution by its original native ID, without another apply.
pub(crate) async fn inspect(state: &Sessions, request_id: &str) -> Result<Value, control::Error> {
    let (intent, finished, native_operation) = {
        let entries = state
            .operations
            .entries
            .lock()
            .map_err(|_| control::Error::unavailable("Operation state unavailable"))?;
        let entry = entries.get(request_id).ok_or_else(|| {
            control::Error::invalid("Unknown request in this service lifetime; do not replay")
        })?;
        (
            entry.intent.clone(),
            entry.stage.borrow().is_some(),
            entry.operation.clone(),
        )
    };
    let input: control::Request =
        serde_json::from_slice(&intent).map_err(|e| control::Error::unknown(e.to_string()))?;
    let mut report = state.operations.status(request_id)?;
    if !finished || report["state"] == "verified" || report["state"] == "failed" {
        return Ok(report);
    }
    if let Command::Instrument { request } = input.command {
        if request.operation().is_none() {
            return Ok(report);
        }
        let operation = &native_operation;
        let native = crate::instrument::native(
            state,
            input.adapter,
            json!({"action":"status","operation":operation}),
        )
        .await;
        if let Ok(native) = native {
            if finished
                && native["status"] == "completed"
                && !matches!(request, crate::instrument::Request::Assign { .. })
            {
                let entries = state
                    .operations
                    .entries
                    .lock()
                    .map_err(|_| control::Error::unavailable("Operation state unavailable"))?;
                let entry = entries
                    .get(request_id)
                    .ok_or_else(|| control::Error::unknown("Missing retained operation"))?;
                // Only replace an unknown verdict after independent native preservation readback.
                let unknown = matches!(entry.stage.borrow().as_ref(),Some(Err(error)) if matches!(error.outcome,control::FailureOutcome::Unknown));
                if unknown {
                    entry
                        .stage
                        .send_replace(Some(Ok(control::Response::InstrumentResult {
                            result: native.clone(),
                        })));
                }
            }
            report = state.operations.status(request_id)?;
            report["native"] = native;
        }
    }
    Ok(report)
}
