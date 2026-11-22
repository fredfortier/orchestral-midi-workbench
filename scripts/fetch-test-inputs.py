"""Fetch and verify the upstream score and schema fixtures for workspace tests."""

import hashlib
import json
from pathlib import Path
from urllib.request import urlopen


ROOT = Path(__file__).resolve().parent.parent
TESTS = ROOT / "crates/convert/tests"
UPSTREAM = "https://raw.githubusercontent.com"
MUSICXML_REVISION = "799e2defb2ece0ae7bafe08dcbcac25b2c631d53"  # v4.0
SCHEMA_HASHES = {
    "musicxml.xsd": "bfe37ed25a9ec00e6f2591d53df260b84efe12aed209ba3ac0a76f9287665a99",
    "xlink.xsd": "6e601f8eeb41618b50e4c7f944dff754e57ea43b602755470dda24c9c2f6df92",
    "xml.xsd": "616a3077df5cfc954ac74a75abe9697b95eef7a85dbe09367d995a483e840eb5",
}


def fetch(destination, url, expected_hash):
    if destination.exists():
        content = destination.read_bytes()
    else:
        with urlopen(url, timeout=30) as response:
            content = response.read()
    if hashlib.sha256(content).hexdigest() != expected_hash:
        raise SystemExit(f"Unexpected SHA-256 for {destination}")
    if not destination.exists():
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(content)


def main():
    manifest = json.loads((TESTS / "corpus-manifest.json").read_text())
    sample = json.loads((TESTS / "corpus-sample.json").read_text())
    revision = manifest["music21Commit"]
    if sample["music21Commit"] != revision:
        raise SystemExit("The score sample and corpus manifest disagree on the music21 revision")

    corpus = "music21/music21/corpus"
    score_hashes = {entry["path"]: entry["sha256"] for entry in manifest["files"]}
    required = {"bach/bwv267.mxl", "bach/bwv29.8.mxl"}
    required.update(entry["path"] for entry in sample["files"])
    for score in sorted(required):
        url = f"{UPSTREAM}/cuthbertLab/music21/{revision}/music21/corpus/{score}"
        fetch(ROOT / corpus / score, url, score_hashes[score])

    schemas = ROOT / ".research/standards/musicxml/musicxml-4.0/schema"
    for name, checksum in SCHEMA_HASHES.items():
        url = f"{UPSTREAM}/w3c/musicxml/{MUSICXML_REVISION}/schema/{name}"
        fetch(schemas / name, url, checksum)
    print(f"Verified {len(required)} corpus scores and {len(SCHEMA_HASHES)} schema files.")


if __name__ == "__main__":
    main()
