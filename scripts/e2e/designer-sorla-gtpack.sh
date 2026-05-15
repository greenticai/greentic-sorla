#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-/tmp/sorla-designer-e2e}"
mkdir -p "${OUT_DIR}"

cargo test -p greentic-sorla-designer-extension designer_prompt_to_gtpack

cat >"${OUT_DIR}/README.txt" <<'EOF'
The SoRLa-local Designer e2e passed.

The current WASM-safe extension path returns deterministic pack-entry metadata.
Native host or Sorx packaging can turn those entries into .gtpack ZIP bytes.
EOF
