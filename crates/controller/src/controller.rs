//! Operational editing intents. Fixture generation and fault injection are tests.
use crate::{
    editor::{Change, Expression, NoteEvent},
    point::PointEdit,
    protocol::{Document, Edit, Plan, Snapshot},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    Selection {
        request: crate::selection::Request,
    },
    Capture {
        path: Option<crate::project_address::Path>,
        #[serde(rename = "includeInactive")]
        include_inactive: bool,
    },
    Mix {
        request: crate::mix::Request,
    },
    ExportAudio {
        request: crate::export_audio::Request,
    },
    Info {
        path: Option<crate::project_address::Path>,
        #[serde(default)]
        parameters: bool,
    },
    ReadClips {
        paths: Vec<crate::project_address::Path>,
    },
    ReplaceClips {
        before: Box<Snapshot>,
        after: Box<Snapshot>,
    },
    Cues {
        request: crate::cues::Request,
    },
    OperationStatus {
        #[serde(rename = "requestId")]
        request_id: String,
    },
    Instrument {
        request: crate::instrument::Request,
    },
    #[serde(skip_deserializing)]
    InstrumentNative {
        request: serde_json::Value,
    },
    /// Read complete effective selected notes, expressions and owner metadata.
    Midi,

    Apply {
        plan: Plan,
    },
    Create {
        document: String,
        revision: String,
        clip: String,
        note: NoteEvent,
    },
    Update {
        document: String,
        revision: String,
        clip: String,
        note: String,
        changes: Vec<Change>,
    },
    Delete {
        document: String,
        revision: String,
        clip: String,
        note: String,
    },
    /// Replace selected notes in one owner, preserving unselected material.
    Replace {
        document: String,
        revision: String,
        clip: String,
        notes: Vec<NoteEvent>,
    },
    Set {
        document: String,
        revision: String,
        changes: Vec<Change>,
    },
    Randomize {
        document: String,
        revision: String,
        options: crate::randomize::Randomize,
    },
    Point {
        document: String,
        revision: String,
        clip: String,
        note: String,
        expression: Expression,
        edit: PointEdit,
    },
    Transpose {
        document: String,
        revision: String,
        semitones: i16,
    },
}
/// Host-side actions mirror document access, not Rust's musical processors.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum DocumentCommand {
    Selection {
        request: crate::selection::Request,
    },
    Capture {
        path: Option<crate::project_address::Path>,
        #[serde(rename = "includeInactive")]
        include_inactive: bool,
    },
    Mix {
        request: crate::mix::Request,
    },
    ExportAudio {
        request: crate::export_audio::Request,
    },
    Info {
        path: Option<crate::project_address::Path>,
        #[serde(default)]
        parameters: bool,
    },
    Cues {
        request: crate::cues::Request,
    },
    Instrument {
        request: serde_json::Value,
    },
    ReadMidi,
    ReadClips {
        paths: Vec<crate::project_address::Path>,
    },
    PrepareEdit {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paths: Option<Vec<crate::project_address::Path>>,
        document: String,
        revision: Option<String>,
    },
}

impl Command {
    #[must_use]
    pub fn mutates(&self) -> bool {
        !matches!(
            self,
            Self::Selection {
                request: crate::selection::Request::Read {}
            } | Self::Capture { .. }
                | Self::Mix {
                    request: crate::mix::Request::Read {}
                }
                | Self::ExportAudio {
                    request: crate::export_audio::Request::Status { .. }
                }
                | Self::Info { .. }
                | Self::ReadClips { .. }
                | Self::Midi
                | Self::Cues {
                    request: crate::cues::Request::Read {}
                }
                | Self::OperationStatus { .. }
                | Self::Instrument {
                    request: crate::instrument::Request::Read { .. }
                        | crate::instrument::Request::Status { .. }
                }
        )
    }

    #[must_use]
    pub const fn edits(&self) -> bool {
        !matches!(
            self,
            Self::Selection { .. }
                | Self::Capture { .. }
                | Self::Mix { .. }
                | Self::ExportAudio { .. }
                | Self::OperationStatus { .. }
                | Self::Info { .. }
                | Self::Cues { .. }
                | Self::Instrument { .. }
                | Self::InstrumentNative { .. }
                | Self::ReadClips { .. }
                | Self::Midi
        )
    }

    /// Translate a resolved intent into one native request.
    /// # Errors
    /// Refuses invalid paths or unresolved musical input.
    pub fn document_command(&self) -> Result<DocumentCommand, String> {
        let (document, revision) = match self {
            Self::Selection { request } => {
                request.validate()?;
                return Ok(DocumentCommand::Selection {
                    request: request.clone(),
                });
            }
            Self::Capture {
                path,
                include_inactive,
            } => {
                if let Some(path) = path {
                    path.validate()?;
                }
                return Ok(DocumentCommand::Capture {
                    path: path.clone(),
                    include_inactive: *include_inactive,
                });
            }
            Self::Mix { request } => {
                request.validate()?;
                return Ok(DocumentCommand::Mix {
                    request: request.clone(),
                });
            }
            Self::ExportAudio { request } => {
                request.validate()?;
                return Ok(DocumentCommand::ExportAudio {
                    request: request.clone(),
                });
            }
            Self::Info { path, parameters } => {
                if let Some(path) = path {
                    path.validate()?;
                }
                return Ok(DocumentCommand::Info {
                    path: path.clone(),
                    parameters: *parameters,
                });
            }
            Self::ReadClips { paths } => {
                for path in paths {
                    path.validate()?;
                }
                return Ok(DocumentCommand::ReadClips {
                    paths: paths.clone(),
                });
            }
            Self::Cues { request } => {
                return Ok(DocumentCommand::Cues {
                    request: request.clone(),
                });
            }
            Self::OperationStatus { .. } | Self::Instrument { .. } => {
                return Err("Command requires application service execution".into());
            }
            Self::InstrumentNative { request } => {
                return Ok(DocumentCommand::Instrument {
                    request: request.clone(),
                });
            }
            Self::Midi => return Ok(DocumentCommand::ReadMidi),
            Self::ReplaceClips { before, .. } => (&before.document, &before.revision),
            Self::Apply { plan } => (&plan.document, &plan.revision),
            Self::Create {
                document, revision, ..
            }
            | Self::Update {
                document, revision, ..
            }
            | Self::Delete {
                document, revision, ..
            }
            | Self::Replace {
                document, revision, ..
            }
            | Self::Set {
                document, revision, ..
            }
            | Self::Randomize {
                document, revision, ..
            }
            | Self::Point {
                document, revision, ..
            }
            | Self::Transpose {
                document, revision, ..
            } => (document, revision),
        };
        Ok(DocumentCommand::PrepareEdit {
            paths: if let Self::ReplaceClips { before, .. } = self {
                before.paths.clone()
            } else {
                None
            },
            document: document.clone(),
            revision: (!matches!(self, Self::ReplaceClips { .. })).then(|| revision.clone()),
        })
    }

    /// Compile a caller's intent against the independently frozen UI scope.
    /// # Errors
    /// Refuses stale revisions, invalid values, targets outside selection and collisions.
    #[allow(clippy::too_many_lines)] // Exhaustive compilation of the small operational intent enum.
    pub fn plan(&self, snapshot: &Snapshot) -> Result<Plan, String> {
        let normalized: Document = snapshot.clone().try_into()?;
        let mut plan = Plan {
            automation_create: Vec::new(),
            automation_delete: Vec::new(),
            automation: Vec::new(),
            document: snapshot.document.clone(),
            revision: snapshot.revision.clone(),
            edits: Vec::new(),
        };
        let (document, revision) = match self {
            Self::Selection { .. } | Self::Capture { .. } => {
                return Err("Capture is read-only".into());
            }
            Self::Mix { .. }
            | Self::ExportAudio { .. }
            | Self::Info { .. }
            | Self::ReadClips { .. }
            | Self::Cues { .. }
            | Self::OperationStatus { .. }
            | Self::Instrument { .. }
            | Self::InstrumentNative { .. }
            | Self::Midi => {
                return Err("Command does not produce an edit plan".into());
            }
            Self::ReplaceClips { before, after } => {
                let mut current = snapshot.clone();
                current.revision.clone_from(&before.revision);
                if before.as_ref() != &current {
                    return Err("Clip content changed; read again before replacing".into());
                }
                plan = crate::midi::replacement(before, after)?;
                plan.revision.clone_from(&snapshot.revision);
                (&snapshot.document, &snapshot.revision)
            }
            Self::Apply { plan: supplied } => {
                plan = supplied.clone();
                (&supplied.document, &supplied.revision)
            }
            Self::Create {
                document,
                revision,
                clip,
                note,
            } => {
                plan.edits.push(Edit::Create {
                    clip: clip.clone(),
                    temporary: note.id.clone(),
                    note: note.clone(),
                });
                (document, revision)
            }
            Self::Update {
                document,
                revision,
                clip,
                note,
                changes,
            } => {
                plan.edits.push(Edit::Update {
                    clip: clip.clone(),
                    note: note.clone(),
                    changes: changes.clone(),
                });
                (document, revision)
            }
            Self::Delete {
                document,
                revision,
                clip,
                note,
            } => {
                plan.edits.push(Edit::Delete {
                    clip: clip.clone(),
                    note: note.clone(),
                });
                (document, revision)
            }
            Self::Replace {
                document,
                revision,
                clip,
                notes,
            } => {
                let owner = snapshot
                    .clips
                    .iter()
                    .find(|owner| owner.id == *clip)
                    .ok_or("Unknown replacement owner")?;
                plan.edits
                    .extend(
                        owner
                            .notes
                            .iter()
                            .filter(|n| n.selected)
                            .map(|n| Edit::Delete {
                                clip: clip.clone(),
                                note: n.note.id.clone(),
                            }),
                    );
                plan.edits.extend(notes.iter().map(|note| Edit::Create {
                    clip: clip.clone(),
                    temporary: note.id.clone(),
                    note: note.clone(),
                }));
                (document, revision)
            }
            Self::Set {
                document,
                revision,
                changes,
            } => {
                if changes
                    .iter()
                    .any(|change| matches!(change, Change::Expressions(_)))
                {
                    return Err("Batch set changes properties/operators; expression graphs require explicit per-note edits".into());
                }
                for clip in &snapshot.clips {
                    for note in clip.notes.iter().filter(|n| n.selected) {
                        plan.edits.push(Edit::Update {
                            clip: clip.id.clone(),
                            note: note.note.id.clone(),
                            changes: changes.clone(),
                        });
                    }
                }
                (document, revision)
            }
            Self::Randomize {
                document,
                revision,
                options,
            } => {
                plan.edits = options.edits(&normalized)?;
                (document, revision)
            }
            Self::Point {
                document,
                revision,
                clip,
                note,
                expression,
                edit,
            } => {
                if *expression == Expression::Formant {
                    return Err("Formant is preserved; point editing admits pressure, pitch, gain, pan and timbre".into());
                }
                let source = snapshot
                    .clips
                    .iter()
                    .find(|owner| owner.id == *clip)
                    .and_then(|owner| owner.notes.iter().find(|item| item.note.id == *note))
                    .ok_or("Unknown point owner")?;
                if !source.selected {
                    return Err("Point owner is outside UI selection".into());
                }
                let original = source
                    .note
                    .expressions
                    .get(expression)
                    .map_or(&[][..], Vec::as_slice);
                let curve = edit.curve(original)?;
                plan.edits.push(Edit::Update {
                    clip: clip.clone(),
                    note: note.clone(),
                    changes: vec![Change::Expressions(BTreeMap::from([(*expression, curve)]))],
                });
                (document, revision)
            }
            Self::Transpose {
                document,
                revision,
                semitones,
            } => {
                for clip in &snapshot.clips {
                    for note in clip.notes.iter().filter(|n| n.selected) {
                        let key = i32::from(note.note.key) + i32::from(*semitones);
                        if !(0..128).contains(&key) {
                            return Err("Transposition exceeds MIDI key range".into());
                        }
                        plan.edits.push(Edit::Update {
                            clip: clip.id.clone(),
                            note: note.note.id.clone(),
                            changes: vec![Change::Key(
                                u8::try_from(key).map_err(|e| e.to_string())?,
                            )],
                        });
                    }
                }
                (document, revision)
            }
        };
        if *document != snapshot.document || *revision != snapshot.revision {
            return Err("Stale document or revision; read selection again".into());
        }
        normalized.validate_plan(&plan)?;
        Ok(plan)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Outcome {
    pub snapshot: Snapshot,
    pub plan: Option<Plan>,
    pub identities: BTreeMap<String, String>,
}
