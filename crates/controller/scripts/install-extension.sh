#!/usr/bin/env bash
set -euo pipefail

crate_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
adapter_dir="$crate_dir/adapters/bitwig-ctl"
harness=false
code_only=false
for arg in "$@"; do
  case "$arg" in
    --harness) harness=true ;;
    --code-only) code_only=true ;;
    -h|--help)
      printf '%s\n' 'Usage: bash install-extension.sh [--harness [--code-only]]' \
        'Build and install a verified Bitwig extension.' \
        'Overrides: BITWIG_RUNTIME_JAR, MIDI_WORKBENCH_EXTENSIONS, NOTES_HARNESS_DIRECTORY.'
      exit 0 ;;
    *) printf 'Unknown argument: %s\n' "$arg" >&2; exit 2 ;;
  esac
done
name=NotesTransformer
if "$harness"; then name=NotesHarness; fi
if "$code_only" && ! "$harness"; then
  printf '%s\n' '--code-only requires --harness' >&2; exit 2
fi
profile=main
maven_args=()
if "$harness"; then
  profile=harness
  maven_args+=(-P harness)
fi
if [[ -n "${BITWIG_RUNTIME_JAR:-}" ]]; then
  maven_args+=("-Dbitwig.runtime.jar=$BITWIG_RUNTIME_JAR")
fi
extensions_dir="${MIDI_WORKBENCH_EXTENSIONS:-$HOME/Bitwig Studio/Extensions}"
harness_dir="${NOTES_HARNESS_DIRECTORY:-/tmp/notes-editor-harness}"

# Keep the artifact stable between the clean build, metadata read and staging.
mkdir -p -- "$adapter_dir/target"
exec {build_lock}>"$adapter_dir/target/.install.lock"
flock "$build_lock"
(cd -- "$adapter_dir" && ./mvnw "${maven_args[@]}" clean verify) >&2
source_file="$adapter_dir/target/$profile/$name.bwextension"

install_harness_code() (
  mkdir -p -- "$harness_dir"
  local staged
  staged="$(mktemp "$harness_dir/.code-XXXXXX.tmp")"
  trap 'rm -f -- "$staged"' EXIT
  install -m 644 -- "$source_file" "$staged"
  mv -f -- "$staged" "$harness_dir/code.bwextension"
  printf '%s\n' "$harness_dir/code.bwextension"
)
if "$code_only"; then
  install_harness_code
  exit 0
fi

wire_version="$("${JAVA_HOME:+$JAVA_HOME/bin/}java" -cp "$source_file" com.midiworkbench.notes.AdapterMetadata | jq -er '.wireVersion | select(type == "number" and . >= 1 and . <= 255 and floor == .)')"
destination="$extensions_dir/$name-wire$wire_version.bwextension"
mkdir -p -- "$extensions_dir"
exec {install_lock}<"$extensions_dir"
flock "$install_lock"
pending="$(mktemp "$extensions_dir/.$name-XXXXXX.tmp")"
trap 'rm -f -- "$pending"' EXIT
cp -- "$source_file" "$pending"
if "$harness"; then
  artifact_timestamp="$(sed -n 's/.*<project.build.outputTimestamp>\([^<]*\)<\/project.build.outputTimestamp>.*/\1/p' "$adapter_dir/pom.xml")"
  "${JAVA_HOME:+$JAVA_HOME/bin/}jar" --update --date="$artifact_timestamp" --file "$pending" \
    -C "$adapter_dir/src/test/resources" META-INF/services/com.bitwig.extension.ExtensionDefinition
fi
chmod 644 "$pending"
if [[ -e "$destination" ]]; then
  if ! cmp -s -- "$pending" "$destination"; then
    printf '%s\n' 'Refusing to overwrite a deployed wire build; bump wire and adapter version.' >&2
    exit 1
  fi
else
  mv -- "$pending" "$destination"
fi

retired_root="${XDG_STATE_HOME:-$HOME/.local/state}/midi-workbench/deployments/retired"
retired_dir=""
for prior in "$extensions_dir/$name.bwextension" "$extensions_dir"/"$name"-wire[0-9]*.bwextension; do
  if [[ -f "$prior" && "$prior" != "$destination" ]]; then
    if [[ -z "$retired_dir" ]]; then
      mkdir -p -- "$retired_root"
      retired_dir="$(mktemp -d "$retired_root/adapter-XXXXXX")"
    fi
    mv -- "$prior" "$retired_dir/"
  fi
done
printf '%s\n' "$destination"
if "$harness"; then
  install_harness_code
fi
