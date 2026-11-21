#!/usr/bin/env bash
set -euo pipefail

if [[ ${1:-} == --help || ${1:-} == -h ]]; then
  printf '%s\n' 'Usage: bash smoke-bitwig.sh [API_PORT]'
  exit 0
fi
if [[ $# -gt 1 ]]; then
  printf '%s\n' 'Usage: bash smoke-bitwig.sh [API_PORT]' >&2
  exit 2
fi
crate_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
bridge="$crate_dir/../target/debug/midi-bridge"
if [[ ! -x "$bridge" ]]; then
  printf '%s\n' 'Build midi-bridge first: cargo build --locked -p notes-transformer' >&2
  exit 1
fi

scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT
"$bridge" controller --port "${1:-8789}" cues read --output "$scratch/cues.json" >/dev/null
jq -e '
  select(.type == "cuesResult" and .result.error == null)
  | .result.observation
  | select((.document | type == "string" and length > 0)
       and (.projectUuid | type == "string" and length > 0)
       and (.revision | type == "string" and length > 0)
       and (.markers | type == "array"))
  | {document, projectUuid, revision, cueCount: (.markers | length)}
' "$scratch/cues.json"
