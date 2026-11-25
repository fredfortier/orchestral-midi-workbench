//! Command-line presentation of operational document intents.
use clap::{Args, Subcommand};
use notes_transformer::{control::Response, controller::Command, point::PointEdit};
use std::{error::Error, path::PathBuf};

#[derive(Args)]
pub struct Controller {
    #[arg(long, default_value_t = notes_transformer::api::DEFAULT_PORT)]
    port: u16,
    /// Required only when more than one production adapter is connected.
    #[arg(long)]
    adapter: Option<String>,
    #[command(subcommand)]
    command: Action,
}
#[derive(Args)]
struct Observation {
    /// Document from the same fresh selection; project actions use `project.controller_generation`.
    #[arg(long)]
    document: String,
    /// Revision from that observation.
    #[arg(long)]
    revision: String,
}
#[derive(Subcommand)]
enum Action {
    /// Read the selected launcher position or create a Note Clip in that empty slot.
    Launcher {
        #[command(subcommand)]
        command: LauncherAction,
    },
    /// Read native selection or replace it absolutely using returned qualified paths.
    Selection {
        #[command(subcommand)]
        command: SelectionAction,
    },
    /// Read native balance or set bounded absolute volume/pan values.
    Mix(crate::cli_mix::Mix),
    /// Export selected native channels with explicit WAV options.
    ExportAudio(crate::cli_mix::ExportAudio),
    /// Set complete automation curves atomically.
    Automation(crate::cli_automation::Automation),
    /// Read the project hierarchy or an exact subtree, with current native selection.
    Info {
        /// JSON qualified entity path. Omit to read the project hierarchy.
        #[arg(long)]
        path: Option<PathBuf>,
        /// Include the addressed device's parameter and Remote Control catalog.
        #[arg(long, requires = "path")]
        parameters: bool,
    },
    /// Read or replace complete native notes, expression and automation.
    Content {
        #[command(subcommand)]
        command: ContentAction,
    },
    /// Inspect the original application operation without replaying it.
    Operation {
        #[arg(long)]
        request_id: String,
    },
    /// Native Instrument Layer and selected-slot resource operations.
    Instrument(crate::cli_instrument::Instrument),
    /// Named performance landmarks on the native Arranger timeline.
    Cues(crate::cli_cues::Cues),
    /// Capture, assemble or decode a MIDI 2.0 performance.
    Midi {
        #[command(subcommand)]
        command: MidiAction,
    },
    /// Preflight and apply an absolute multi-edit JSON plan as one Undo action.
    Apply {
        #[arg(long)]
        plan: PathBuf,
    },
    Note {
        #[command(subcommand)]
        command: Note,
    },
    /// Edit one explicitly owned expression curve as one native Undo action.
    Point {
        #[command(subcommand)]
        command: PointAction,
    },
}

#[derive(Subcommand)]
enum ContentAction {
    /// Read complete musical content for explicit paths or the active selection.
    Read {
        /// JSON array of qualified track, group-channel or Note Clip paths.
        #[arg(long, required_unless_present = "selection")]
        paths: Option<PathBuf>,
        /// Use the effective native musical selection.
        #[arg(long, conflicts_with = "paths")]
        selection: bool,
    },
    /// Atomically replace the same complete musical content and print native readback.
    Replace {
        /// Complete snapshot previously returned by content read.
        #[arg(long)]
        expected: PathBuf,
        /// Desired snapshot for exactly the same paths.
        #[arg(long)]
        input: PathBuf,
    },
}

#[derive(Subcommand)]
enum LauncherAction {
    Read,
    Create {
        #[arg(long)]
        expected: PathBuf,
        #[arg(long)]
        beats: u32,
    },
}
#[derive(Subcommand)]
enum SelectionAction {
    Read,
    /// JSON request: document, revision, paths, mode, range.
    Set {
        #[arg(long)]
        input: PathBuf,
    },
}
#[derive(Subcommand)]
enum MidiAction {
    /// Capture canonical entities, complete clips and timing from active tracks.
    Capture {
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        include_inactive: bool,
        #[arg(long)]
        output: PathBuf,
    },
    /// Assemble a self-contained MIDI2 file offline from an interpreted capture.
    Assemble {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        report: PathBuf,
    },
    /// Decode a MIDI 2.0 Clip plus companion offline; performs no host action.
    Decode {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        companion: PathBuf,
    },
}

#[derive(Subcommand)]
enum Note {
    /// Insert a complete `NoteEvent` JSON object, including explicit dynamics/options.
    Create {
        #[command(flatten)]
        observed: Observation,
        #[arg(long)]
        clip: String,
        #[arg(long)]
        input: PathBuf,
    },
    /// Set typed sparse changes from a JSON array, preserving omitted fields.
    Update {
        #[command(flatten)]
        observed: Observation,
        #[arg(long)]
        clip: String,
        #[arg(long)]
        note: String,
        #[arg(long)]
        changes: PathBuf,
    },
    Delete {
        #[command(flatten)]
        observed: Observation,
        #[arg(long)]
        clip: String,
        #[arg(long)]
        note: String,
    },
    /// Replace selected notes in one clip with a JSON array of complete notes.
    Replace {
        #[command(flatten)]
        observed: Observation,
        #[arg(long)]
        clip: String,
        #[arg(long)]
        input: PathBuf,
    },
    /// Apply sparse properties/operators to every selected note in one transaction.
    Set {
        #[command(flatten)]
        observed: Observation,
        #[arg(long)]
        changes: PathBuf,
    },
    /// Randomize an exact subset as one Undo action; ranges use native units.
    Randomize {
        #[command(flatten)]
        observed: Observation,
        #[arg(long)]
        seed: u64,
        #[arg(long)]
        count: usize,
        /// Exact number of distinct variables changed per chosen note; a sampled no-op refuses the whole batch
        #[arg(long, default_value_t = 1)]
        changes_per_note: usize,
        /// Eligible field and inclusive range: channel=0:15, velocity=0.2:0.8,
        /// pressure.value=0:1, pitch.curvature=-1:1. Repeat for multiple fields.
        /// Scalars: channel, key, start, duration, muted, velocity,
        /// releaseVelocity, velocitySpread. Operators: chanceEnabled, chance,
        /// occurrenceEnabled, occurrence (0..10 in native enum order),
        /// recurrenceEnabled, recurrenceLength, recurrenceMask, repeatEnabled,
        /// repeatCount, repeatCurve, repeatVelocityEnd, repeatVelocityCurve.
        /// Curves: pressure, pitch, gain, pan, timbre; point properties: value,
        /// time, interpolation (0=Hold, 1=Linear), curvature, spread.
        /// Gain is native cube-root amplitude; pitch is semitones; time is beats.
        /// Point fields require existing curves. Times resolve to native persistent ticks.
        /// Invalid/colliding batches refuse whole.
        #[arg(long = "range", value_parser = parse_range)]
        ranges: Vec<(String, [f64; 2])>,
    },
    /// Transpose every selected note, preserving expression curves and other values.
    Transpose {
        #[command(flatten)]
        observed: Observation,
        #[arg(long, allow_hyphen_values = true)]
        semitones: i16,
    },
}
#[derive(Args)]
struct PointTarget {
    #[command(flatten)]
    observed: Observation,
    #[arg(long)]
    clip: String,
    #[arg(long)]
    note: String,
    #[arg(long, value_parser = ["pressure", "pitch", "gain", "pan", "timbre"])]
    expression: String,
}
#[derive(Subcommand)]
enum PointAction {
    /// Insert a complete Point JSON object; new equal-time points default after ties.
    Create {
        #[command(flatten)]
        target: PointTarget,
        #[arg(long)]
        input: PathBuf,
        /// Insert before an existing point at the same native time.
        #[arg(long)]
        before: Option<String>,
    },
    /// Preserve omitted fields; stable time order, or an explicit equal-time anchor.
    Update {
        #[command(flatten)]
        target: PointTarget,
        #[arg(long)]
        point: String,
        /// Typed sparse JSON array: time, value, interpolation, curvature, spread.
        #[arg(long, required_unless_present = "before")]
        changes: Option<PathBuf>,
        #[arg(long)]
        before: Option<String>,
    },
    Delete {
        #[command(flatten)]
        target: PointTarget,
        #[arg(long)]
        point: String,
    },
    /// Remove every point in this curve, preserving all other expressions.
    Clear {
        #[command(flatten)]
        target: PointTarget,
    },
}
impl PointAction {
    fn intent(self) -> Result<Command, Box<dyn Error>> {
        let (target, edit) = match self {
            Self::Create {
                target,
                input,
                before,
            } => (
                target,
                PointEdit::Create {
                    point: read(&input)?,
                    before,
                },
            ),
            Self::Update {
                target,
                point,
                changes,
                before,
            } => (
                target,
                PointEdit::Update {
                    point,
                    changes: changes
                        .as_deref()
                        .map(read)
                        .transpose()?
                        .unwrap_or_default(),
                    before,
                },
            ),
            Self::Delete { target, point } => (target, PointEdit::Delete { point }),
            Self::Clear { target } => (target, PointEdit::Clear),
        };
        Ok(Command::Point {
            document: target.observed.document,
            revision: target.observed.revision,
            clip: target.clip,
            note: target.note,
            expression: serde_json::from_value(target.expression.into())?,
            edit,
        })
    }
}
fn read<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_slice(
        &notes_transformer::files::read_bounded(path)?,
    )?)
}
impl Note {
    fn intent(self) -> Result<Command, Box<dyn Error>> {
        Ok(match self {
            Note::Create {
                observed: o,
                clip,
                input,
            } => Command::Create {
                document: o.document,
                revision: o.revision,
                clip,
                note: read(&input)?,
            },
            Note::Update {
                observed: o,
                clip,
                note,
                changes,
            } => Command::Update {
                document: o.document,
                revision: o.revision,
                clip,
                note,
                changes: read(&changes)?,
            },
            Note::Delete {
                observed: o,
                clip,
                note,
            } => Command::Delete {
                document: o.document,
                revision: o.revision,
                clip,
                note,
            },
            Note::Replace {
                observed: o,
                clip,
                input,
            } => Command::Replace {
                document: o.document,
                revision: o.revision,
                clip,
                notes: read(&input)?,
            },
            Note::Set {
                observed: o,
                changes,
            } => Command::Set {
                document: o.document,
                revision: o.revision,
                changes: read(&changes)?,
            },
            Note::Randomize {
                observed: o,
                seed,
                count,
                changes_per_note,
                ranges,
            } => {
                let mut unique = std::collections::BTreeMap::new();
                for (field, range) in ranges {
                    if unique.insert(field, range).is_some() {
                        return Err("Repeated randomization field".into());
                    }
                }
                Command::Randomize {
                    document: o.document,
                    revision: o.revision,
                    options: notes_transformer::randomize::Randomize {
                        seed,
                        count,
                        changes_per_note,
                        ranges: unique,
                    },
                }
            }
            Note::Transpose {
                observed: o,
                semitones,
            } => Command::Transpose {
                document: o.document,
                revision: o.revision,
                semitones,
            },
        })
    }
}
impl Controller {
    fn intent(self) -> Result<(u16, Option<String>, Command), Box<dyn Error>> {
        let command = match self.command {
            Action::Launcher { command } => Command::Launcher {
                request: match command {
                    LauncherAction::Read => notes_transformer::launcher::Request::Read {},
                    LauncherAction::Create { expected, beats } => {
                        notes_transformer::launcher::Request::Create {
                            expected: read(&expected)?,
                            beats,
                        }
                    }
                },
            },
            Action::Selection { command } => Command::Selection {
                request: match command {
                    SelectionAction::Read => notes_transformer::selection::Request::Read {},
                    SelectionAction::Set { input } => {
                        let mut input: serde_json::Map<String, serde_json::Value> = read(&input)?;
                        if input.insert("action".into(), "set".into()).is_some() {
                            return Err("Selection input must omit action".into());
                        }
                        let request: notes_transformer::selection::Request =
                            serde_json::from_value(input.into())?;
                        request
                    }
                },
            },
            Action::Info { path, parameters } => Command::Info {
                parameters,
                path: path.as_ref().map(|path| read(path)).transpose()?,
            },
            Action::Operation { request_id } => Command::OperationStatus { request_id },
            Action::Instrument(_) => return Err("Use the instrument dispatcher".into()),
            Action::Mix(_)
            | Action::ExportAudio(_)
            | Action::Cues(_)
            | Action::Content { .. }
            | Action::Midi { .. }
            | Action::Automation(_) => {
                return Err("Workflow command requires its dedicated dispatcher".into());
            }

            Action::Apply { plan } => Command::Apply { plan: read(&plan)? },
            Action::Point { command } => command.intent()?,
            Action::Note { command } => command.intent()?,
        };
        Ok((self.port, self.adapter, command))
    }
}
pub async fn run(args: Controller) -> Result<(), Box<dyn Error>> {
    if let Action::Info { path, parameters } = args.command {
        let response = notes_transformer::api::exchange(
            args.port,
            args.adapter,
            Command::Info {
                parameters,
                path: path.as_ref().map(|path| read(path)).transpose()?,
            },
        )
        .await?;
        let notes_transformer::control::Response::EntityResult { entity } = response else {
            return Err("Unexpected entity result".into());
        };
        serde_json::to_writer(std::io::stdout().lock(), &entity)?;
        println!();
        return Ok(());
    }
    if matches!(
        &args.command,
        Action::Midi {
            command: MidiAction::Capture { .. }
                | MidiAction::Assemble { .. }
                | MidiAction::Decode { .. }
        }
    ) {
        return run_export(args).await;
    }

    run_specialist(args).await
}
async fn run_content(
    port: u16,
    adapter: Option<String>,
    command: ContentAction,
) -> Result<(), Box<dyn Error>> {
    let command = match command {
        ContentAction::Read {
            paths: Some(paths), ..
        } => Command::ReadClips {
            paths: read(&paths)?,
        },
        ContentAction::Read { paths: None, .. } => Command::Midi,
        ContentAction::Replace { expected, input } => Command::ReplaceClips {
            before: read(&expected)?,
            after: read(&input)?,
        },
    };
    let snapshot = match notes_transformer::api::exchange(port, adapter, command).await? {
        Response::MidiResult { midi } => *midi,
        Response::ControlResult { result } => result.snapshot,
        _ => return Err("Unexpected native content response".into()),
    };
    serde_json::to_writer(std::io::stdout().lock(), &snapshot)?;
    println!();
    Ok(())
}

async fn run_specialist(args: Controller) -> Result<(), Box<dyn Error>> {
    if let Action::Content { command } = args.command {
        return run_content(args.port, args.adapter, command).await;
    }
    if let Action::Mix(command) = args.command {
        return crate::cli_mix::mix(command, args.port, args.adapter).await;
    }
    if let Action::ExportAudio(command) = args.command {
        return crate::cli_mix::export(command, args.port, args.adapter).await;
    }
    if let Action::Automation(automation) = args.command {
        return crate::cli_automation::run(automation, args.port, args.adapter).await;
    }
    if let Action::Operation { request_id } = args.command {
        let result = notes_transformer::api::exchange(
            args.port,
            args.adapter,
            Command::OperationStatus { request_id },
        )
        .await?;
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }
    if let Action::Instrument(instrument) = args.command {
        return crate::cli_instrument::run(instrument, args.port, args.adapter).await;
    }
    if let Action::Cues(cues) = args.command {
        return crate::cli_cues::run(args.port, args.adapter, cues).await;
    }
    let (port, adapter, command) = args.intent()?;
    let response = notes_transformer::api::exchange(port, adapter, command).await?;
    let output = match response {
        Response::LauncherResult { result } => serde_json::to_string(&result)?,
        Response::SelectionResult { result } => serde_json::to_string(&result)?,
        Response::ControlResult { result } => serde_json::to_string_pretty(&result)?,
        _ => return Err("Unexpected command response".into()),
    };
    println!("{output}");
    Ok(())
}
fn parse_range(value: &str) -> Result<(String, [f64; 2]), String> {
    let (field, values) = value.split_once('=').ok_or("Expected FIELD=MIN:MAX")?;
    let (min, max) = values.split_once(':').ok_or("Expected FIELD=MIN:MAX")?;
    Ok((
        field.into(),
        [
            min.parse::<f64>().map_err(|e| e.to_string())?,
            max.parse::<f64>().map_err(|e| e.to_string())?,
        ],
    ))
}

async fn run_export(args: Controller) -> Result<(), Box<dyn Error>> {
    if let Action::Midi {
        command: MidiAction::Decode { input, companion },
    } = args.command
    {
        let (performance, _) = notes_transformer::decode::read(&input, &companion)?;
        serde_json::to_writer(std::io::stdout().lock(), &performance)?;
        println!();
        return Ok(());
    }

    if let Action::Midi {
        command:
            MidiAction::Capture {
                path,
                include_inactive,
                output,
            },
    } = args.command
    {
        notes_transformer::files::ensure_fresh_output(&output)?;
        let path = path
            .map(|p| {
                notes_transformer::files::read_bounded(&p)
                    .and_then(|b| Ok(serde_json::from_slice(&b)?))
            })
            .transpose()?;
        let result = notes_transformer::api::exchange(
            args.port,
            args.adapter,
            Command::Capture {
                path,
                include_inactive,
            },
        )
        .await?;
        let notes_transformer::control::Response::CaptureResult { capture } = result else {
            return Err("Unexpected capture response".into());
        };
        notes_transformer::files::publish(&output, &serde_json::to_value(capture)?)?;
        return Ok(());
    }
    if let Action::Midi {
        command:
            MidiAction::Assemble {
                input,
                output,
                report,
            },
    } = args.command
    {
        let destination = |p: &std::path::Path| -> Result<PathBuf, Box<dyn Error>> {
            let parent = p
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| std::path::Path::new("."));
            Ok(parent
                .canonicalize()?
                .join(p.file_name().ok_or("Output requires a file name")?))
        };
        if destination(&output)? == destination(&report)? {
            return Err("MIDI2 and report outputs must differ".into());
        }
        notes_transformer::files::ensure_fresh_output(&output)?;
        notes_transformer::files::ensure_fresh_output(&report)?;
        let capture = serde_json::from_slice(&notes_transformer::files::read_bounded(&input)?)?;
        let (bytes, analysis) = notes_transformer::export_midi::assemble(&capture)?;
        notes_transformer::files::publish(&report, &analysis)?;
        notes_transformer::files::publish_bytes(&output, &bytes)?;
        return Ok(());
    }
    Err("Expected capture or assembly command".into())
}
