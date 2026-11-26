use std::{
    path::Path,
    process::{Command, Output},
};
fn convert(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_convert"))
        .args(args)
        .output()
        .unwrap()
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn actual_binary_converts_bach_both_ways_and_schema_validates() {
    let dir = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("../../music21/music21/corpus/bach/bwv267.mxl");
    let midi = dir.path().join("bach.midi2");
    let xml = dir.path().join("bach.musicxml");
    let mxl = dir.path().join("bach.mxl");
    success(&convert(&[
        source.to_str().unwrap(),
        "--to",
        "midi2",
        "-o",
        midi.to_str().unwrap(),
    ]));
    success(&convert(&[
        midi.to_str().unwrap(),
        "--to",
        "musicxml",
        "-o",
        xml.to_str().unwrap(),
    ]));
    success(&convert(&[
        midi.to_str().unwrap(),
        "--to",
        "mxl",
        "-o",
        mxl.to_str().unwrap(),
    ]));
    let expanded = musicxml_midi2_convert::xml::extract(&std::fs::read(mxl).unwrap()).unwrap();
    let expanded_path = dir.path().join("expanded.musicxml");
    std::fs::write(&expanded_path, expanded).unwrap();
    // Remove the companion and exercise the independent performance inverse.
    std::fs::remove_file(midi.with_extension("convert.json")).unwrap();
    let standalone = dir.path().join("standalone.musicxml");
    success(&convert(&[
        midi.to_str().unwrap(),
        "--to",
        "musicxml",
        "-o",
        standalone.to_str().unwrap(),
    ]));
    let check = Command::new("python3")
        .arg(root.join("tests/check_musicxml.py"))
        .args([&xml, &expanded_path, &standalone])
        .output()
        .expect("install python3 and python3-lxml for the independent XSD oracle");
    success(&check);
    let conflict = convert(&[
        source.to_str().unwrap(),
        "--to",
        "midi2",
        "-o",
        midi.to_str().unwrap(),
    ]);
    assert_eq!(conflict.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("E_EXISTS"));
    assert_eq!(convert(&["--to", "bogus"]).status.code(), Some(2));
}

#[test]
fn blocked_artifact_path_cannot_publish_a_partial_bundle() {
    let dir = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/violin.musicxml");
    let midi = dir.path().join("violin.midi2");
    let companion = dir.path().join("manifest");
    std::fs::create_dir(&companion).unwrap();
    let result = convert(&[
        source.to_str().unwrap(),
        "--to",
        "midi2",
        "--output",
        midi.to_str().unwrap(),
        "--companion",
        companion.to_str().unwrap(),
        "--force",
    ]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("E_OUTPUT"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn explicit_companion_must_exist_and_describe_the_clip() {
    let dir = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/violin.musicxml");
    let midi = dir.path().join("violin.midi2");
    success(&convert(&[
        source.to_str().unwrap(),
        "--to",
        "midi2",
        "--output",
        midi.to_str().unwrap(),
    ]));
    let absent = dir.path().join("absent.json");
    let output = dir.path().join("result.musicxml");
    let result = convert(&[
        midi.to_str().unwrap(),
        "--to",
        "musicxml",
        "--companion",
        absent.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(!result.status.success());
    assert!(!output.exists());

    let other_clip = dir.path().join("bach.midi2");
    let bach =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../music21/music21/corpus/bach/bwv267.mxl");
    success(&convert(&[
        bach.to_str().unwrap(),
        "--to",
        "midi2",
        "--output",
        other_clip.to_str().unwrap(),
    ]));
    let other_companion = other_clip.with_extension("convert.json");
    let result = convert(&[
        midi.to_str().unwrap(),
        "--to",
        "musicxml",
        "--companion",
        other_companion.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("performed values differ from companion")
    );
    assert!(!output.exists());

    std::fs::copy(other_companion, midi.with_extension("convert.json")).unwrap();
    success(&convert(&[
        midi.to_str().unwrap(),
        "--to",
        "musicxml",
        "--output",
        output.to_str().unwrap(),
    ]));
    assert!(output.exists());
}
