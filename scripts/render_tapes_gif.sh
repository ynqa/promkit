#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAPES_DIR="${ROOT_DIR}/tapes"

if ! command -v vhs >/dev/null 2>&1; then
  echo "error: vhs command not found. install it first: https://github.com/charmbracelet/vhs" >&2
  exit 1
fi

tape_count=0

while IFS= read -r tape; do
  [[ -z "${tape}" ]] && continue

  tape_count=$((tape_count + 1))
  rel_path="${tape#${ROOT_DIR}/}"
  echo "rendering ${rel_path}"
  (
    cd "${ROOT_DIR}"
    vhs "${rel_path}"
  )
done < <(find "${TAPES_DIR}" -maxdepth 1 -type f -name '*.tape' | sort)

if [[ ${tape_count} -eq 0 ]]; then
  echo "error: no .tape files found in ${TAPES_DIR}" >&2
  exit 1
fi

echo "done: rendered ${tape_count} tape(s)"
