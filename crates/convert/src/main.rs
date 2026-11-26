//! The command line is an I/O adapter around the library API.
use clap::{Parser, ValueEnum};
use musicxml_midi2_convert::{
    self as convert, Diagnostic, MAX_ARTIFACT_BYTES, Result, model::RenderingPolicy,
    preserve::Companion,
};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    Midi2,
    Musicxml,
    Mxl,
}
#[derive(Parser, Debug)]
#[command(
    name = "convert",
    version,
    about = "Convert MusicXML/MXL and orchestral MIDI 2.0 Clip files"
)]
struct Args {
    input: PathBuf,
    #[arg(long, value_enum)]
    to: Option<Format>,
    /// Validate and inventory MIDI2 without reconstructing a score.
    #[arg(long, conflicts_with_all = ["to", "companion", "rendition_plan", "policy", "from", "output"])]
    validate: bool,
    /// Inspect performed notes and exact rendition-rule selectors without writing MIDI.
    #[arg(long, conflicts_with_all = ["to", "validate", "companion", "rendition_plan", "from", "report"])]
    inspect_rendition: bool,
    #[arg(long, value_enum)]
    from: Option<Format>,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long)]
    companion: Option<PathBuf>,
    /// Apply reviewed note, receiver and controller decisions (JSON; see convert README).
    #[arg(long)]
    rendition_plan: Option<PathBuf>,
    /// Rendering policy JSON, including the default tempo for scores without one.
    #[arg(long)]
    policy: Option<PathBuf>,
    #[arg(long)]
    report: Option<PathBuf>,
    #[arg(long)]
    force: bool,
}
fn io(e: impl std::fmt::Display) -> Diagnostic {
    Diagnostic::new("E_IO", e.to_string())
}
fn write_set(files: &[(PathBuf, Vec<u8>)], force: bool) -> Result<()> {
    for (path, data) in files {
        if u64::try_from(data.len()).map_err(io)? > MAX_ARTIFACT_BYTES {
            return Err(Diagnostic::new(
                "E_RESOURCE",
                format!("{} exceeds the 64 MiB artifact limit", path.display()),
            ));
        }
        match path.symlink_metadata() {
            Ok(metadata) if !metadata.is_file() => {
                return Err(Diagnostic::new(
                    "E_OUTPUT",
                    format!("{} is not a regular file", path.display()),
                ));
            }
            Ok(_) if !force => {
                return Err(Diagnostic::new(
                    "E_EXISTS",
                    format!("{} exists; use --force", path.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io(error)),
        }
    }
    let mut staging = Vec::new();
    for (path, data) in files {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(io)?;
        file.write_all(data).map_err(io)?;
        file.as_file().sync_all().map_err(io)?;
        staging.push(file);
    }
    // The companion is the completion marker for score conversion.
    for (file, (path, _)) in staging.into_iter().zip(files) {
        if force {
            file.persist(path).map_err(io)?;
        } else {
            file.persist_noclobber(path).map_err(io)?;
        }
    }
    Ok(())
}
fn read_input(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(io)?
        .take(MAX_ARTIFACT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if u64::try_from(bytes.len()).map_err(io)? > MAX_ARTIFACT_BYTES {
        return Err(Diagnostic::new("E_RESOURCE", "Input exceeds 64 MiB"));
    }
    Ok(bytes)
}
fn read_policy(path: Option<&Path>) -> Result<RenderingPolicy> {
    path.map_or_else(
        || Ok(RenderingPolicy::default()),
        |path| serde_json::from_slice(&read_input(path)?).map_err(io),
    )
}
fn inspect_rendition(args: &Args) -> Result<()> {
    let (source, score) = convert::xml::read(
        &read_input(&args.input)?,
        &read_policy(args.policy.as_deref())?,
    )?;
    let notes = convert::adapt::inspect_notes(score.notes)?;
    let directions: Vec<_> = score
        .report
        .decisions
        .iter()
        .filter(|decision| decision.rule == "written direction")
        .collect();
    let inspection = serde_json::json!({
        "duration": score.duration,
        "notes": notes,
        "sourceVoices": convert::xml::note_voices(&source)?,
        "sourceDirections": directions
    });
    let mut bytes = serde_json::to_vec_pretty(&inspection).map_err(io)?;
    bytes.push(b'\n');
    if let Some(path) = &args.output {
        let destination = path.canonicalize().ok();
        if destination.is_some()
            && std::iter::once(&args.input)
                .chain(args.policy.iter())
                .any(|input| input.canonicalize().ok() == destination)
        {
            return Err(Diagnostic::new(
                "E_OUTPUT",
                "inspection output overlaps an input",
            ));
        }
        return write_set(&[(path.clone(), bytes)], args.force);
    }
    std::io::stdout().lock().write_all(&bytes).map_err(io)
}
#[allow(
    clippy::too_many_lines,
    reason = "Keep artifact validation, staging and publication policy together."
)]
fn run(args: Args) -> Result<()> {
    if args.inspect_rendition {
        return inspect_rendition(&args);
    }
    if args.validate {
        let bytes = read_input(&args.input)?;
        let analysis = convert::validation::inspect(&bytes);
        let json = serde_json::to_vec_pretty(&analysis).map_err(io)?;
        if let Some(path) = args.report {
            if path.canonicalize().ok() == Some(args.input.canonicalize().map_err(io)?) {
                return Err(Diagnostic::new("E_OUTPUT", "Report must differ from input"));
            }
            write_set(&[(path, json.clone())], args.force)?;
        }
        println!("{}", String::from_utf8_lossy(&json));
        if !analysis.valid {
            return Err(Diagnostic::new("E_INVALID_MIDI2", "see validation report"));
        }
        if !analysis.admissible {
            return Err(Diagnostic::new("E_INADMISSIBLE", "see validation report"));
        }
        return Ok(());
    }
    let target = args.to.ok_or_else(|| {
        Diagnostic::new(
            "E_OPTIONS",
            "choose --to, --inspect-rendition or --validate",
        )
    })?;
    let bytes = read_input(&args.input)?;
    let extension = match target {
        Format::Midi2 => "midi2",
        Format::Musicxml => "musicxml",
        Format::Mxl => "mxl",
    };
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| args.input.with_extension(extension));
    if output == args.input {
        return Err(Diagnostic::new("E_OUTPUT", "input and output must differ"));
    }
    let mut files = Vec::new();
    let mut protected =
        std::collections::BTreeSet::from([args.input.canonicalize().map_err(io)?]);
    let from_midi = match args.from {
        Some(Format::Midi2) => true,
        Some(_) => false,
        None => bytes.starts_with(b"SMF2CLIP"),
    };
    if from_midi {
        if args.policy.is_some() || args.rendition_plan.is_some() {
            return Err(Diagnostic::new(
                "E_OPTIONS",
                "--policy and --rendition-plan apply to score rendering",
            ));
        }
        if matches!(target, Format::Midi2) {
            return Err(Diagnostic::new(
                "E_FORMAT",
                "choose musicxml or mxl for a MIDI clip",
            ));
        }
        let performance = convert::clip::decode(&bytes, None)?;
        let standalone = convert::arrangement::read(&performance)?.is_some();
        let explicit_companion = args.companion.is_some();
        let companion_path = args
            .companion
            .unwrap_or_else(|| args.input.with_extension("convert.json"));
        let companion: Option<Companion> =
            if explicit_companion || (!standalone && companion_path.try_exists().map_err(io)?) {
                let companion: Companion =
                    serde_json::from_slice(&read_input(&companion_path)?).map_err(io)?;
                if explicit_companion {
                    companion.validate_rendition(&performance)?;
                }
                protected.insert(companion_path.canonicalize().map_err(io)?);
                Some(companion)
            } else {
                None
            };
        let xml = convert::midi2_to_xml(&bytes, companion.as_ref(), matches!(target, Format::Mxl))?;
        if let Some(report) = args.report {
            files.push((
                report,
                serde_json::to_vec_pretty(&if standalone {
                    convert::notation::report(&performance)
                } else {
                    performance.report
                })
                .map_err(io)?,
            ));
        }
        files.push((output, xml));
    } else {
        if !matches!(target, Format::Midi2) {
            return Err(Diagnostic::new(
                "E_FORMAT",
                "choose midi2 for score conversion",
            ));
        }
        let policy = read_policy(args.policy.as_deref())?;
        if let Some(path) = &args.policy {
            protected.insert(path.canonicalize().map_err(io)?);
        }
        let converted = if let Some(path) = args.rendition_plan {
            let plan = serde_json::from_slice(&read_input(&path)?).map_err(|error| {
                Diagnostic::new("E_RENDITION_PLAN", format!("{}: {error}", path.display()))
            })?;
            protected.insert(path.canonicalize().map_err(io)?);
            convert::xml_to_rendition(&bytes, &policy, &plan)?
        } else {
            convert::xml_to_midi2(&bytes, &policy)?
        };
        let companion = args
            .companion
            .unwrap_or_else(|| output.with_extension("convert.json"));
        let report = args
            .report
            .unwrap_or_else(|| output.with_extension("report.json"));
        let source_ext = if bytes.starts_with(b"PK") {
            "source.mxl"
        } else {
            "source.musicxml"
        };
        files.push((output.with_extension(source_ext), bytes));
        files.push((output, converted.clip));
        files.push((
            report,
            serde_json::to_vec_pretty(&converted.report).map_err(io)?,
        ));
        files.push((
            companion,
            serde_json::to_vec_pretty(&converted.companion).map_err(io)?,
        ));
    }
    let mut paths = std::collections::BTreeSet::new();
    for (path, _) in &files {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let identity = path
            .canonicalize()
            .or_else(|_| {
                parent
                    .canonicalize()
                    .map(|p| p.join(path.file_name().unwrap_or_default()))
            })
            .map_err(io)?;
        if protected.contains(&identity) || !paths.insert(identity) {
            return Err(Diagnostic::new("E_OUTPUT", "artifact paths overlap"));
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            if !Path::new(parent).exists() {
                return Err(Diagnostic::new(
                    "E_OUTPUT",
                    format!("directory {} does not exist", parent.display()),
                ));
            }
        }
    }
    write_set(&files, args.force)
}
fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(if e.code == "E_INADMISSIBLE" { 2 } else { 1 });
    }
}
