//! Ergonomic facade over scoped discovery and the complete-clip transaction.
use clap::{Args, Subcommand};
use notes_transformer::{
    api,
    automation::Curve,
    control::Response,
    controller::Command,
    editor::{Interpolation, Point},
    entity::Entity,
    files,
    project_address::Path,
    protocol::Snapshot,
};
use serde::Deserialize;
use std::{collections::BTreeSet, error::Error, path::PathBuf};

#[derive(Args)]
pub struct Automation {
    #[command(subcommand)]
    command: Action,
}
#[derive(Subcommand)]
enum Action {
    /// Discover the named device's parameters and mapped Remote Controls.
    #[command(hide = true)]
    Read {
        #[arg(long)]
        target: PathBuf,
    },
    /// Set track MIDI CC curves (or explicitly requested device parameters) atomically.
    /// Points use physical values and source beats (arranger beats for new curves).
    Set {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CurveInput {
    track: Option<Path>,
    channel: Option<u8>,
    controller: Option<u8>,
    device: Option<Path>,
    parameter: Option<String>,
    source: Option<String>,
    points: Vec<PointInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointInput {
    time: f64,
    value: f64,
    #[serde(default = "linear")]
    interpolation: Interpolation,
    #[serde(default)]
    curvature: f64,
    #[serde(default)]
    spread: f64,
}
fn linear() -> Interpolation {
    Interpolation::Linear
}
async fn info(port: u16, adapter: Option<String>, path: Path) -> Result<Entity, Box<dyn Error>> {
    match api::exchange(
        port,
        adapter,
        Command::Info {
            path: Some(path),
            parameters: true,
        },
    )
    .await?
    {
        Response::EntityResult { entity } => Ok(entity),
        _ => Err("Unexpected device discovery response".into()),
    }
}
pub async fn run(
    args: Automation,
    port: u16,
    adapter: Option<String>,
) -> Result<(), Box<dyn Error>> {
    match args.command {
        Action::Read { target } => {
            let path = serde_json::from_slice(&files::read_bounded(&target)?)?;
            let entity = info(port, adapter, path).await?;
            serde_json::to_writer(std::io::stdout().lock(), &entity)?;
            println!();
            Ok(())
        }
        Action::Set { input, output } => set(&input, output, port, adapter).await,
    }
}
async fn device_target(
    port: u16,
    adapter: Option<String>,
    path: Path,
    parameter: &str,
) -> Result<(Path, serde_json::Value), Box<dyn Error>> {
    let device = info(port, adapter.clone(), path).await?;
    let matches: Vec<_> = device.metadata["parameters"]
        .as_array()
        .ok_or("Target has no device parameters")?
        .iter()
        .filter(|p| {
            p["target"]["name"] == parameter
                || p["target"]["nativeParameterId"] == parameter
                || device.metadata["remoteControls"]
                    .as_array()
                    .is_some_and(|remotes| {
                        remotes.iter().any(|r| {
                            r["name"] == parameter
                                && r["nativeParameterId"] == p["target"]["nativeParameterId"]
                        })
                    })
        })
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "Parameter {parameter:?}: expected one match, found {}",
            matches.len()
        )
        .into());
    }
    Ok((device.path.track_path()?, matches[0]["target"].clone()))
}

async fn set(
    input: &std::path::Path,
    output: Option<PathBuf>,
    port: u16,
    adapter: Option<String>,
) -> Result<(), Box<dyn Error>> {
    if let Some(path) = &output {
        files::ensure_fresh_output(path)?;
    }
    let inputs: Vec<CurveInput> = serde_json::from_slice(&files::read_bounded(input)?)?;
    if inputs.is_empty() || inputs.len() > 256 {
        return Err("Supply 1–256 automation curves".into());
    }
    let mut targets = Vec::new();
    let mut paths = BTreeSet::new();
    for input in &inputs {
        let (owner, target) = match (
            &input.track,
            input.channel,
            input.controller,
            &input.device,
            &input.parameter,
        ) {
            (Some(track), channel, Some(controller), None, None) => {
                let channel = channel.unwrap_or(0);
                track.validate()?;
                if track.track_path()? != *track || channel > 15 || controller > 119 {
                    return Err(
                        "MIDI automation requires a track, channel 0–15 and CC 0–119".into(),
                    );
                }
                (
                    track.clone(),
                    serde_json::json!({
                        "kind": "midiCc", "channel": channel, "controller": controller,
                    }),
                )
            }
            (None, None, None, Some(path), Some(parameter)) => {
                device_target(port, adapter.clone(), path.clone(), parameter).await?
            }
            _ => {
                return Err(
                    "Supply track/channel/controller OR device/parameter, never a mixture".into(),
                );
            }
        };
        paths.insert(owner.clone());
        targets.push((owner, target));
    }
    let Response::MidiResult { midi } = api::exchange(
        port,
        adapter.clone(),
        Command::ReadClips {
            paths: paths.into_iter().collect(),
        },
    )
    .await?
    else {
        return Err("Unexpected complete clip response".into());
    };
    let before = *midi;
    let after = desired(&before, &inputs, targets)?;
    let observed: Snapshot = if before == after {
        before
    } else {
        let Response::ControlResult { result } = api::exchange(
            port,
            adapter,
            Command::ReplaceClips {
                before: Box::new(before),
                after: Box::new(after),
            },
        )
        .await?
        else {
            return Err("Unexpected automation transaction response".into());
        };
        result.snapshot
    };
    if let Some(path) = output {
        files::publish_bytes(&path, &serde_json::to_vec(&observed)?)?;
    }
    serde_json::to_writer(std::io::stdout().lock(), &observed)?;
    println!();
    Ok(())
}

fn desired(
    before: &Snapshot,
    inputs: &[CurveInput],
    targets: Vec<(Path, serde_json::Value)>,
) -> Result<Snapshot, Box<dyn Error>> {
    let mut after = before.clone();
    let mut resolved = BTreeSet::new();
    for (index, (input, (owner, target))) in inputs.iter().zip(targets).enumerate() {
        let matching: Vec<_> = before
            .automation
            .iter()
            .filter(|c| c.owner == owner && same_target(&c.target, &target))
            .collect();
        if input.source.is_none() && matching.iter().any(|c| c.placement == "clip") {
            return Err("Target has committed automation; preserve it and specify an observed source explicitly".into());
        }
        let sources: BTreeSet<_> = matching.iter().map(|c| c.source.clone()).collect();
        let source = if let Some(source) = &input.source {
            if !sources.contains(source) {
                return Err("Requested automation source is absent".into());
            }
            Some(source.clone())
        } else if sources.len() > 1 {
            return Err(
                "Target has multiple automation sources; specify source from a complete clip read"
                    .into(),
            );
        } else {
            sources.into_iter().next()
        };
        let identity = if target["kind"] == "midiCc" {
            format!("midiCc:{}:{}", target["channel"], target["controller"])
        } else {
            format!("parameter:{}", target["nativeParameterId"])
        };
        if !resolved.insert((owner.clone(), identity, source.clone())) {
            return Err("Duplicate resolved automation target/source".into());
        }
        let original = before
            .automation
            .iter()
            .find(|c| Some(&c.source) == source.as_ref());
        let points: Vec<_> = input
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| Point {
                id: original
                    .and_then(|c| c.points.get(i))
                    .map_or_else(|| format!("automation:{index}:{i}"), |p| p.id.clone()),
                time: p.time,
                value: p.value,
                interpolation: p.interpolation,
                curvature: p.curvature,
                spread: p.spread,
            })
            .collect();
        if let Some(source) = &source {
            for curve in &mut after.automation {
                if &curve.source == source {
                    curve.points.clone_from(&points);
                }
            }
        } else {
            after.automation.push(Curve {
                owner,
                placement: "arranger".into(),
                note_clip: None,
                track: String::new(),
                lane: String::new(),
                clip: String::new(),
                source: String::new(),
                aliases: vec![],
                reference_count: 0,
                mapping: serde_json::json!({}),
                target,
                points,
            });
        }
    }
    Ok(after)
}

fn same_target(left: &serde_json::Value, right: &serde_json::Value) -> bool {
    left["kind"] == right["kind"]
        && if right["kind"] == "midiCc" {
            left["channel"] == right["channel"] && left["controller"] == right["controller"]
        } else {
            left["nativeParameterId"] == right["nativeParameterId"]
        }
}
