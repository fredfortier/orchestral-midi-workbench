#!/usr/bin/env bash
set -euo pipefail
probe_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p /tmp/notes-document-probe-classes
"${NOTES_PROBE_JAVAC:-javac}" --release 17 -Xlint:all -Werror \
  -d /tmp/notes-document-probe-classes "$probe_dir/NotesDocumentProbe.java"
printf 'Compiled retained research source; current runnable probes use the Java harness.\n'
