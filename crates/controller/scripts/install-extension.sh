#!/usr/bin/env bash
set -euo pipefail

crate_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
harness=false
code_only=false
for arg in "$@"; do
  case "$arg" in
    --harness) harness=true ;;
    --code-only) code_only=true ;;
    *) printf 'Unknown argument: %s\n' "$arg" >&2; exit 2 ;;
  esac
done
name=NotesTransformer
if "$harness"; then name=NotesHarness; fi
if "$code_only" && ! "$harness"; then
  printf '%s\n' '--code-only requires --harness' >&2; exit 2
fi
profile=main
if "$harness"; then profile=harness; fi
source_file="$crate_dir/client-adapter/target/$profile/$name.bwextension"
destination="$HOME/Bitwig Studio/Extensions/$name.bwextension"
if ! "$code_only"; then
  wire_version="$("${JAVA_HOME:+$JAVA_HOME/bin/}java" -cp "$source_file" com.midiworkbench.notes.AdapterMetadata | jq -er '.wireVersion')"
  destination="$HOME/Bitwig Studio/Extensions/$name-wire$wire_version.bwextension"
fi
if "$code_only"; then destination=/tmp/notes-editor-harness/code.bwextension; fi
mkdir -p -- "$(dirname -- "$destination")"
pending="$(mktemp "$(dirname -- "$destination")/.$name-XXXXXX.tmp")"
service_dir="$(mktemp -d)"
trap 'rm -f -- "$pending"; rm -rf -- "$service_dir"' EXIT
cp -- "$source_file" "$pending"
if "$harness" && ! "$code_only"; then
  mkdir -p "$service_dir/META-INF/services"
  printf '%s\n' com.midiworkbench.notes.EditorHarnessDefinition > "$service_dir/META-INF/services/com.bitwig.extension.ExtensionDefinition"
  artifact_timestamp="$(sed -n 's/.*<project.build.outputTimestamp>\([^<]*\)<\/project.build.outputTimestamp>.*/\1/p' "$crate_dir/client-adapter/pom.xml")"
  "${JAVA_HOME:+$JAVA_HOME/bin/}jar" --update --date="$artifact_timestamp" --file "$pending" -C "$service_dir" META-INF/services/com.bitwig.extension.ExtensionDefinition
fi
chmod 644 "$pending"
if ! "$code_only"; then
  if [[ -e "$destination" ]]; then
    if ! cmp -s -- "$pending" "$destination"; then
      printf '%s\n' 'Refusing to overwrite a deployed wire build; bump wire and adapter version.' >&2
      exit 1
    fi
    printf '%s\n' "$destination"
    if "$harness"; then bash "$0" --harness --code-only; fi
    exit 0
  fi
  retired_root="${XDG_STATE_HOME:-$HOME/.local/state}/midi-workbench/deployments/retired"
  mkdir -p -- "$retired_root"
  retired_dir="$(mktemp -d "$retired_root/adapter-XXXXXX")"
  for prior in "$HOME/Bitwig Studio/Extensions/$name.bwextension" "$HOME/Bitwig Studio/Extensions"/"$name"-wire[0-9]*.bwextension; do
    if [[ -f "$prior" && "$prior" != "$destination" ]]; then
      mv -- "$prior" "$retired_dir/"
    fi
  done
fi
mv -f -- "$pending" "$destination"
printf '%s\n' "$destination"
if "$harness" && ! "$code_only"; then
  bash "$0" --harness --code-only
fi
