use std::path::Path;
pub fn assert_schema(documents: &[&[u8]], path: &Path) {
    use std::process::{Command, Stdio};
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/check_musicxml.py");
    let mut child = Command::new("python3")
        .arg(script)
        .arg("--stdin-json")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("install python3-lxml for the offline XSD oracle");
    let strings = documents
        .iter()
        .map(|d| std::str::from_utf8(d).unwrap())
        .collect::<Vec<_>>();
    serde_json::to_writer(child.stdin.take().unwrap(), &strings).unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}: {}",
        path.display(),
        String::from_utf8_lossy(&result.stderr)
    );
}
