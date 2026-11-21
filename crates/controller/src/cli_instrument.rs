//! Parse qualified instrument intents; all host execution belongs to the service.
use clap::{Args, Subcommand};
use notes_transformer::{instrument::Request, project_address::Path};
use std::{error::Error, path::PathBuf};

#[derive(Args)]
pub struct Instrument {
    #[command(subcommand)]
    command: Action,
}
#[derive(Args)]
struct Target {
    /// JSON qualified musical path: project, groups, track, layer and optional named slot.
    #[arg(long)]
    target: PathBuf,
}
impl Target {
    fn read(self) -> Result<Path, Box<dyn Error>> {
        Ok(serde_json::from_slice(
            &notes_transformer::files::read_bounded(&self.target)?,
        )?)
    }
}
#[derive(Args)]
struct Mutation {
    #[command(flatten)]
    target: Target,
    /// Stable ID for this intent; query status after disconnection instead of replaying.
    #[arg(long)]
    operation: String,
}
#[derive(Subcommand)]
enum Action {
    Read(Target),
    Layer {
        #[command(subcommand)]
        command: Layer,
    },
    Slot {
        #[command(subcommand)]
        command: Slot,
    },
    /// Ensure Kontakt independently of slot routing; optionally replace its NKI.
    Assign {
        #[command(flatten)]
        mutation: Mutation,
        /// Replace the loaded Kontakt patch; omission preserves its configuration.
        #[arg(long, requires_all=["wine_prefix","wine_runtime","display","output"])]
        load_patch: Option<PathBuf>,
        #[arg(long, requires = "load_patch")]
        wine_prefix: Option<PathBuf>,
        #[arg(long, requires = "load_patch")]
        wine_runtime: Option<PathBuf>,
        #[arg(long, requires = "load_patch")]
        display: Option<String>,
        #[arg(long, requires = "load_patch")]
        output: Option<PathBuf>,
        /// Leave the exact loaded editor open for authorized receiver configuration.
        #[arg(long, requires = "load_patch")]
        keep_editor_open: bool,
        /// Continue only a retained pre-submission ownership failure; never reinsert.
        #[arg(long, requires = "load_patch")]
        resume_before_load: Option<PathBuf>,
    },
    Status {
        #[arg(long)]
        operation: String,
    },
}
#[derive(Subcommand)]
enum Layer {
    /// Ensure the named Layer or Selector specified by the target path.
    Ensure(Mutation),
}
#[derive(Subcommand)]
enum Slot {
    Ensure {
        #[command(flatten)]
        mutation: Mutation,
        /// Musical slot name, applied on every ensure.
        #[arg(long)]
        name: String,
        /// Complete note-channel admission set. Shared control channel 1 is added.
        #[arg(long, required=true, value_parser=clap::value_parser!(u8).range(1..=16))]
        input_channel: Vec<u8>,
    },
    Delete(Mutation),
}
pub async fn run(
    args: Instrument,
    port: u16,
    adapter: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let request = match args.command {
        Action::Read(target) => Request::Read {
            target: target.read()?,
        },
        Action::Layer {
            command: Layer::Ensure(m),
        } => Request::EnsureLayer {
            operation: m.operation,
            target: m.target.read()?,
        },
        Action::Slot {
            command:
                Slot::Ensure {
                    mutation: m,
                    name,
                    input_channel,
                },
        } => Request::EnsureSlot {
            operation: m.operation,
            target: m.target.read()?,
            name,
            input_channels: input_channel.into_iter().map(|c| c - 1).collect(),
        },
        Action::Slot {
            command: Slot::Delete(m),
        } => Request::DeleteSlot {
            operation: m.operation,
            target: m.target.read()?,
        },
        Action::Assign {
            mutation: m,
            load_patch,
            wine_prefix,
            wine_runtime,
            display,
            output,
            keep_editor_open,
            resume_before_load,
        } => Request::Assign {
            operation: m.operation,
            target: m.target.read()?,
            nki: load_patch,
            runtime: match (wine_prefix, wine_runtime, display) {
                (Some(prefix), Some(runtime), Some(display)) => Some(
                    notes_transformer::kontakt_process::Runtime::new(prefix, runtime, display),
                ),
                (None, None, None) => None,
                _ => return Err("Complete loader runtime required".into()),
            },
            output,
            keep_editor_open,
            resume_before_load,
        },
        Action::Status { operation } => Request::Status { operation },
    };
    request.validate()?;
    let result = notes_transformer::api::exchange(
        port,
        adapter,
        notes_transformer::controller::Command::Instrument { request },
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
