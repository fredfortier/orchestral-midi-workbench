//! Scoped performance landmarks; the service owns execution and verification.
use clap::{Args, Subcommand};
use notes_transformer::{controller::Command, cues};
use std::{error::Error, path::PathBuf};
#[derive(Args)]
pub struct Cues {
    #[command(subcommand)]
    command: Action,
}
#[derive(Subcommand)]
enum Action {
    Read {
        #[arg(long)]
        output: PathBuf,
    },
    Create {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        document: String,
        #[arg(long)]
        revision: String,
        #[arg(long)]
        operation: String,
        #[arg(long)]
        output: PathBuf,
    },
}
pub async fn run(port: u16, adapter: Option<String>, args: Cues) -> Result<(), Box<dyn Error>> {
    let (request, output) = match args.command {
        Action::Read { output } => (cues::Request::Read {}, output),
        Action::Create {
            input,
            document,
            revision,
            operation,
            output,
        } => {
            let cues: cues::Create =
                serde_json::from_slice(&notes_transformer::files::read_bounded(&input)?)?;
            cues.validate()?;
            (
                cues::Request::Create {
                    operation,
                    document,
                    revision,
                    cues,
                },
                output,
            )
        }
    };
    notes_transformer::files::ensure_fresh_output(&output)?;
    let result = notes_transformer::api::exchange(port, adapter, Command::Cues { request }).await?;
    notes_transformer::files::publish(&output, &serde_json::to_value(&result)?)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
