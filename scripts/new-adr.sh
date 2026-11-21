#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 || ! "$1" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
  echo "usage: $0 <decision-slug> (lowercase letters, digits and hyphens)" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
adr_dir="${repo_root}/docs/adrs"
target="${adr_dir}/$1.md"

if [[ -e "$target" || -L "$target" ]]; then
  echo "ADR already exists: ${target}" >&2
  exit 1
fi

(set -o noclobber; cat "${adr_dir}/template.md" > "$target")
printf '%s\n' "$target"
