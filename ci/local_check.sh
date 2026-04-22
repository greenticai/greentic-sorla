#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

run_step() {
  echo
  echo "=== $1 ==="
}

run_cmd() {
  echo "+ $*"
  "$@"
}

missing_metadata() {
  local manifest_path="$1"
  local field="$2"
  if ! grep -qE "^[[:space:]]*${field}([[:space:]]*=[[:space:]]*|\\.workspace[[:space:]]*=[[:space:]]*true)" "$manifest_path"; then
    echo "Missing required field ${field} in ${manifest_path}" >&2
    return 1
  fi
}

run_step "Environment and metadata pre-checks"
if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but not installed" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for package discovery in this script." >&2
  exit 1
fi

mapfile -t PUBLISHABLE_ENTRIES < <(
  cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.publish == null or (.publish | length) > 0) | [.name, .manifest_path] | @tsv'
)

if [[ ${#PUBLISHABLE_ENTRIES[@]} -eq 0 ]]; then
  echo "No publishable crates found." >&2
  exit 1
fi

for entry in "${PUBLISHABLE_ENTRIES[@]}"; do
  crate="${entry%%$'\t'*}"
  manifest_path="${entry#*$'\t'}"
  missing_metadata "$manifest_path" "license"
  missing_metadata "$manifest_path" "repository"
  missing_metadata "$manifest_path" "description"
  missing_metadata "$manifest_path" "readme"
  missing_metadata "$manifest_path" "categories"
  missing_metadata "$manifest_path" "keywords"
done

run_step "cargo fmt"
run_cmd cargo fmt --all -- --check

run_step "cargo clippy"
run_cmd cargo clippy --all-targets --all-features -- -D warnings

run_step "cargo test"
run_cmd cargo test --all-features

run_step "cargo build"
run_cmd cargo build --all-features

run_step "cargo doc"
run_cmd cargo doc --no-deps --all-features

run_step "Packaging and publish dry-run checks"
for entry in "${PUBLISHABLE_ENTRIES[@]}"; do
  crate="${entry%%$'\t'*}"
  run_step "Package checks: ${crate}"
  if [[ "${CI:-}" == "true" ]]; then
    run_cmd cargo package --no-verify -p "$crate"
  else
    run_cmd cargo package --no-verify -p "$crate" --allow-dirty
  fi
  if [[ "${CI:-}" == "true" ]]; then
    run_cmd cargo package -p "$crate"
  else
    run_cmd cargo package -p "$crate" --allow-dirty
  fi
  if [[ "${CI:-}" == "true" ]]; then
    run_cmd cargo publish -p "$crate" --dry-run
  else
    run_cmd cargo publish -p "$crate" --dry-run --allow-dirty
  fi
done

run_step "i18n checks"
if [[ -f "i18n/en.json" ]]; then
  if command -v greentic-i18n-translator >/dev/null 2>&1; then
    run_cmd bash tools/i18n.sh status
    run_cmd bash tools/i18n.sh validate
  else
    echo "[i18n] skipping runtime i18n checks: greentic-i18n-translator not installed"
  fi
fi

run_step "Validation complete"
echo "All checks passed."
