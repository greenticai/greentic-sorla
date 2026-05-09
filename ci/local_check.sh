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

run_publish_check() {
  local log_file
  log_file="$(mktemp -t greentic-sorla-publish-check.XXXXXX.log)"
  echo "+ $*"
  if "$@" >"$log_file" 2>&1; then
    cat "$log_file"
    rm -f "$log_file"
    return 0
  fi

  cat "$log_file"
  if grep -qE "no matching package named \`greentic-sorla-(lang|ir|pack)\` found" "$log_file"; then
    echo "[publish] advisory: skipping first-publish dry-run blocked by unpublished internal crate dependency."
    echo "[publish] advisory: the release workflow publishes greentic-sorla-lang, greentic-sorla-ir, greentic-sorla-pack, then greentic-sorla."
    rm -f "$log_file"
    return 0
  fi

  rm -f "$log_file"
  return 1
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

PUBLISHABLE_ENTRIES=()
while IFS= read -r entry; do
  PUBLISHABLE_ENTRIES+=("$entry")
done < <(
  cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.publish == null or (.publish | length) > 0) | [.name, .manifest_path] | @tsv'
)

if [[ ${#PUBLISHABLE_ENTRIES[@]} -eq 0 ]]; then
  echo "No publishable crates found." >&2
  exit 1
fi

declare -A PUBLISHABLE_BY_NAME=()
for entry in "${PUBLISHABLE_ENTRIES[@]}"; do
  crate="${entry%%$'\t'*}"
  PUBLISHABLE_BY_NAME["$crate"]="$entry"
done

ORDERED_PUBLISHABLE_ENTRIES=()
for crate in greentic-sorla-lang greentic-sorla-ir greentic-sorla-pack greentic-sorla; do
  if [[ -n "${PUBLISHABLE_BY_NAME[$crate]:-}" ]]; then
    ORDERED_PUBLISHABLE_ENTRIES+=("${PUBLISHABLE_BY_NAME[$crate]}")
    unset "PUBLISHABLE_BY_NAME[$crate]"
  fi
done
for entry in "${PUBLISHABLE_ENTRIES[@]}"; do
  crate="${entry%%$'\t'*}"
  if [[ -n "${PUBLISHABLE_BY_NAME[$crate]:-}" ]]; then
    ORDERED_PUBLISHABLE_ENTRIES+=("${PUBLISHABLE_BY_NAME[$crate]}")
    unset "PUBLISHABLE_BY_NAME[$crate]"
  fi
done
PUBLISHABLE_ENTRIES=("${ORDERED_PUBLISHABLE_ENTRIES[@]}")

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
    run_publish_check cargo package --no-verify -p "$crate"
  else
    run_publish_check cargo package --no-verify -p "$crate" --allow-dirty
  fi
  if [[ "${CI:-}" == "true" ]]; then
    run_publish_check cargo package -p "$crate"
  else
    run_publish_check cargo package -p "$crate" --allow-dirty
  fi
  if [[ "${CI:-}" == "true" ]]; then
    run_publish_check cargo publish -p "$crate" --dry-run
  else
    run_publish_check cargo publish -p "$crate" --dry-run --allow-dirty
  fi
done

run_step "i18n checks"
if [[ -f "i18n/en.json" ]]; then
  while IFS= read -r locale_file; do
    run_cmd jq empty "$locale_file"
  done < <(find i18n -maxdepth 1 -name '*.json' -type f | LC_ALL=C sort)

  if command -v greentic-i18n-translator >/dev/null 2>&1; then
    if [[ "${I18N_STRICT:-false}" == "true" ]]; then
      run_cmd bash tools/i18n.sh status
      run_cmd bash tools/i18n.sh validate
    else
      i18n_status_log="$(mktemp -t greentic-sorla-i18n-status.XXXXXX.log)"
      i18n_validate_log="$(mktemp -t greentic-sorla-i18n-validate.XXXXXX.log)"
      echo "+ bash tools/i18n.sh status"
      if ! bash tools/i18n.sh status >"$i18n_status_log" 2>&1; then
        echo "[i18n] advisory: translations are incomplete; details: ${i18n_status_log}"
        echo "[i18n] advisory: set I18N_STRICT=true to fail local checks on translation gaps"
      fi
      echo "+ bash tools/i18n.sh validate"
      if ! bash tools/i18n.sh validate >"$i18n_validate_log" 2>&1; then
        echo "[i18n] advisory: locale validation found translation gaps; details: ${i18n_validate_log}"
        echo "[i18n] advisory: set I18N_STRICT=true to fail local checks on translation gaps"
      fi
    fi
  else
    echo "[i18n] skipping runtime i18n checks: greentic-i18n-translator not installed"
  fi
fi

run_step "Validation complete"
echo "All checks passed."
