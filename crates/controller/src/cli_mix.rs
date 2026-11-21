//! Native balance and render command presentation.
use clap::{Args, Subcommand};
use notes_transformer::{control::Response, controller::Command, export_audio, files, mix};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

#[derive(Args)]
pub struct Mix {
    #[command(subcommand)]
    command: MixAction,
}
#[derive(Subcommand)]
enum MixAction {
    Read {
        #[arg(long)]
        output: PathBuf,
    },
    /// Input: {expected: complete mix observation, changes: [{path, volumeDb, pan}]}.
    Set {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}
#[derive(Args)]
pub struct ExportAudio {
    #[command(subcommand)]
    command: ExportAction,
}
#[derive(Subcommand)]
enum ExportAction {
    /// Input: expected mix, exact tracks, fresh directory, start/end beats,
    /// sampleRate, bitDepth (INT16/INT24/FLOAT32), dither, preFader, realTime.
    Start {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        operation: String,
        #[arg(long)]
        output: PathBuf,
    },
    Status {
        #[arg(long)]
        operation: String,
        #[arg(long)]
        output: PathBuf,
    },
}
async fn send(
    port: u16,
    adapter: Option<String>,
    command: Command,
    output: &Path,
) -> Result<(), Box<dyn Error>> {
    files::ensure_fresh_output(output)?;
    let response = notes_transformer::api::exchange(port, adapter, command).await?;
    let value = match response {
        Response::MixResult { result } => {
            let value = serde_json::to_value(&result.observation)?;
            if let Some(error) = &result.error {
                files::publish(output, &serde_json::to_value(&result)?)?;
                return Err(error.clone().into());
            }
            value
        }
        Response::ExportAudioResult { result } => result,
        _ => return Err("Unexpected native mixing/export response".into()),
    };
    files::publish(output, &value)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    if value["status"] == "failed" {
        return Err("Native export failed; inspect retained operation status".into());
    }
    Ok(())
}
pub async fn mix(args: Mix, port: u16, adapter: Option<String>) -> Result<(), Box<dyn Error>> {
    let (request, output) = match args.command {
        MixAction::Read { output } => (mix::Request::Read {}, output),
        MixAction::Set { input, output } => {
            let mut value: serde_json::Value =
                serde_json::from_slice(&files::read_bounded(&input)?)?;
            if value
                .as_object_mut()
                .ok_or("Expected mixer input object")?
                .insert("action".into(), "set".into())
                .is_some()
            {
                return Err("Input must omit action".into());
            }
            (serde_json::from_value(value)?, output)
        }
    };
    request.validate()?;
    send(port, adapter, Command::Mix { request }, &output).await
}
pub async fn export(
    args: ExportAudio,
    port: u16,
    adapter: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let (request, output) = match args.command {
        ExportAction::Start {
            input,
            operation,
            output,
        } => (
            export_audio::Request::Start {
                operation,
                options: Box::new(serde_json::from_slice(&files::read_bounded(&input)?)?),
            },
            output,
        ),
        ExportAction::Status { operation, output } => {
            (export_audio::Request::Status { operation }, output)
        }
    };
    request.validate()?;
    send(port, adapter, Command::ExportAudio { request }, &output).await
}
