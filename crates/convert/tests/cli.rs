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
