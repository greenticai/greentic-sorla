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

run_capture() {
  local output_path="$1"
  shift
  echo "+ $* > ${output_path}"
  "$@" >"${output_path}"
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
  if grep -qE "no matching package named \`greentic-sorla-(lang|ir|pack|lib)\` found" "$log_file"; then
    echo "[publish] advisory: skipping first-publish dry-run blocked by unpublished internal crate dependency."
    echo "[publish] advisory: the release workflow publishes greentic-sorla-lang, greentic-sorla-ir, greentic-sorla-pack, greentic-sorla-lib, then greentic-sorla."
    rm -f "$log_file"
    return 0
  fi

  rm -f "$log_file"
  return 1
}

PUBLISH_WORKSPACE_DIR=""

prepare_publish_workspace() {
  if [[ -n "${PUBLISH_WORKSPACE_DIR}" ]]; then
    return 0
  fi

  local workspace_version
  workspace_version="$(
    cargo metadata --no-deps --format-version 1 \
      | jq -r '.packages[] | select(.name == "greentic-sorla") | .version'
  )"
  if [[ -z "${workspace_version}" || "${workspace_version}" == "null" ]]; then
    echo "Failed to resolve greentic-sorla workspace version." >&2
    return 1
  fi

  PUBLISH_WORKSPACE_DIR="$(mktemp -d -t greentic-sorla-publish-workspace.XXXXXX)"
  trap 'rm -rf "${PUBLISH_WORKSPACE_DIR}"' EXIT
  tar \
    --exclude='./.git' \
    --exclude='./target' \
    --exclude='./packages' \
    -cf - . | tar -C "${PUBLISH_WORKSPACE_DIR}" -xf -

  SORLA_WORKSPACE_VERSION="${workspace_version}" perl -0pi -e '
    my $v = $ENV{"SORLA_WORKSPACE_VERSION"};
    s/(greentic-sorla-cli\s*=\s*\{\s*package\s*=\s*"greentic-sorla",\s*)(path\s*=)/$1version = "$v", $2/g;
    s/(greentic-sorla-ir\s*=\s*\{\s*)(path\s*=)/$1version = "$v", $2/g;
    s/(greentic-sorla-lang\s*=\s*\{\s*)(path\s*=)/$1version = "$v", $2/g;
    s/(greentic-sorla-lib\s*=\s*\{\s*)(path\s*=)/$1version = "$v", $2/g;
    s/(greentic-sorla-designer-extension\s*=\s*\{\s*)(path\s*=)/$1version = "$v", $2/g;
    s/(greentic-sorla-pack\s*=\s*\{\s*)(path\s*=)/$1version = "$v", $2/g;
  ' "${PUBLISH_WORKSPACE_DIR}/Cargo.toml"
}

publish_patch_config_for() {
  local crate="$1"
  local patches=(
    'greentic-sorla=crates/greentic-sorla-cli'
    'greentic-sorla-ir=crates/greentic-sorla-ir'
    'greentic-sorla-lang=crates/greentic-sorla-lang'
    'greentic-sorla-lib=crates/greentic-sorla-lib'
    'greentic-sorla-pack=crates/greentic-sorla-pack'
  )
  local patch name path
  for patch in "${patches[@]}"; do
    name="${patch%%=*}"
    path="${patch#*=}"
    if [[ "$name" == "$crate" ]]; then
      continue
    fi
    printf '%s\0%s\0' --config "patch.crates-io.${name}.path=\"${path}\""
  done
}

missing_metadata() {
  local manifest_path="$1"
  local field="$2"
  if ! grep -qE "^[[:space:]]*${field}([[:space:]]*=[[:space:]]*|\\.workspace[[:space:]]*=[[:space:]]*true)" "$manifest_path"; then
    echo "Missing required field ${field} in ${manifest_path}" >&2
    return 1
  fi
}

run_validation_pack_check() {
  local tmp_dir
  local pack_path
  local inspect_path
  local validation_inspect_path
  local validation_schema_path
  local exposure_schema_path
  local compatibility_schema_path
  local ontology_schema_path
  local retrieval_schema_path
  local answers_path
  local output_dir

  tmp_dir="$(mktemp -d -t greentic-sorla-validation-pack.XXXXXX)"
  trap 'rm -rf "${tmp_dir}"' RETURN
  answers_path="${tmp_dir}/answers.json"
  output_dir="${tmp_dir}/workspace"
  pack_path="${output_dir}/landlord-tenant-sor.gtpack"
  inspect_path="${tmp_dir}/inspect.json"
  validation_inspect_path="${tmp_dir}/validation-inspect.json"
  validation_schema_path="${tmp_dir}/sorx-validation.schema.json"
  exposure_schema_path="${tmp_dir}/sorx-exposure-policy.schema.json"
  compatibility_schema_path="${tmp_dir}/sorx-compatibility.schema.json"
  ontology_schema_path="${tmp_dir}/sorla-ontology.schema.json"
  retrieval_schema_path="${tmp_dir}/sorla-retrieval-bindings.schema.json"

  run_capture "${validation_schema_path}" cargo run -p greentic-sorla -- pack schema validation
  run_capture "${exposure_schema_path}" cargo run -p greentic-sorla -- pack schema exposure-policy
  run_capture "${compatibility_schema_path}" cargo run -p greentic-sorla -- pack schema compatibility
  run_capture "${ontology_schema_path}" cargo run -p greentic-sorla -- pack schema ontology
  run_capture "${retrieval_schema_path}" cargo run -p greentic-sorla -- pack schema retrieval-bindings

  jq -e '."$id" == "greentic.sorx.validation.v1"' "${validation_schema_path}" >/dev/null \
    || { echo "ERROR: validation schema command did not emit greentic.sorx.validation.v1" >&2; return 1; }
  jq -e '."$id" == "greentic.sorx.exposure-policy.v1"' "${exposure_schema_path}" >/dev/null \
    || { echo "ERROR: exposure policy schema command did not emit greentic.sorx.exposure-policy.v1" >&2; return 1; }
  jq -e '."$id" == "greentic.sorx.compatibility.v1"' "${compatibility_schema_path}" >/dev/null \
    || { echo "ERROR: compatibility schema command did not emit greentic.sorx.compatibility.v1" >&2; return 1; }
  jq -e '."$id" == "greentic.sorla.ontology.v1"' "${ontology_schema_path}" >/dev/null \
    || { echo "ERROR: ontology schema command did not emit greentic.sorla.ontology.v1" >&2; return 1; }
  jq -e '."$id" == "greentic.sorla.retrieval-bindings.v1"' "${retrieval_schema_path}" >/dev/null \
    || { echo "ERROR: retrieval bindings schema command did not emit greentic.sorla.retrieval-bindings.v1" >&2; return 1; }

  mkdir -p "${output_dir}"
  jq --arg output_dir "${output_dir}" '.output_dir = $output_dir' \
    examples/landlord-tenant/answers.json > "${answers_path}"
  run_cmd cargo run -p greentic-sorla -- wizard \
    --answers "${answers_path}" \
    --pack-out landlord-tenant-sor.gtpack
  run_cmd cargo run -p greentic-sorla -- pack doctor "${pack_path}"
  run_capture "${inspect_path}" cargo run -p greentic-sorla -- pack inspect "${pack_path}"
  run_capture "${validation_inspect_path}" cargo run -p greentic-sorla -- pack validation-inspect "${pack_path}"

  jq -e '.assets | index("assets/sorx/tests/test-manifest.json")' "${inspect_path}" >/dev/null \
    || { echo "ERROR: generated .gtpack is missing assets/sorx/tests/test-manifest.json" >&2; return 1; }
  jq -e '.assets | index("assets/sorx/exposure-policy.json")' "${inspect_path}" >/dev/null \
    || { echo "ERROR: generated .gtpack is missing assets/sorx/exposure-policy.json" >&2; return 1; }
  jq -e '.assets | index("assets/sorx/compatibility.json")' "${inspect_path}" >/dev/null \
    || { echo "ERROR: generated .gtpack is missing assets/sorx/compatibility.json" >&2; return 1; }
  jq -e '.validation.schema == "greentic.sorx.validation.v1"' "${inspect_path}" >/dev/null \
    || { echo "ERROR: pack inspect is missing validation summary" >&2; return 1; }
  jq -e '.exposure_policy.default_visibility != "public_candidate"' "${inspect_path}" >/dev/null \
    || { echo "ERROR: exposure policy default_visibility must not be public_candidate" >&2; return 1; }
  jq -e '.compatibility.state_mode == "shared_requires_migration"' "${validation_inspect_path}" >/dev/null \
    || { echo "ERROR: validation-inspect compatibility summary is missing shared_requires_migration state mode" >&2; return 1; }

  rm -rf "${tmp_dir}"
  trap - RETURN
}

yaml_package_field() {
  local yaml_path="$1"
  local field="$2"
  FIELD="$field" perl -0ne '
    if (/^package:\s*\n((?:[ \t]+.*\n)+)/m) {
      my $field = $ENV{"FIELD"};
      if ($1 =~ /^[ \t]+\Q$field\E:[ \t]*["'"'"']?([^"'"'"'\n#]+)["'"'"']?/m) {
        my $value = $1;
        $value =~ s/[ \t]+$//;
        print "$value\n";
      }
    }
  ' "$yaml_path"
}

run_all_gtpack_fixture_check() {
  local tmp_dir
  tmp_dir="$(mktemp -d -t greentic-sorla-all-gtpacks.XXXXXX)"
  trap 'rm -rf "${tmp_dir}"' RETURN

  local sorla_files=()
  while IFS= read -r path; do
    sorla_files+=("$path")
  done < <(find examples packages -path '*/sorla.yaml' -type f | LC_ALL=C sort)

  if [[ ${#sorla_files[@]} -eq 0 ]]; then
    echo "ERROR: no example or package sorla.yaml files found" >&2
    return 1
  fi

  local sorla_path name version safe_name pack_path inspect_path
  for sorla_path in "${sorla_files[@]}"; do
    name="$(yaml_package_field "$sorla_path" name)"
    version="$(yaml_package_field "$sorla_path" version)"
    if [[ -z "$name" || -z "$version" ]]; then
      echo "ERROR: failed to resolve package name/version from ${sorla_path}" >&2
      return 1
    fi
    safe_name="${name//[^A-Za-z0-9_.-]/_}"
    pack_path="${tmp_dir}/${safe_name}.gtpack"
    inspect_path="${tmp_dir}/${safe_name}.inspect.json"
    run_cmd cargo run -p greentic-sorla -- pack "$sorla_path" \
      --name "$name" \
      --version "$version" \
      --out "$pack_path"
    run_cmd cargo run -p greentic-sorla -- pack doctor "$pack_path"
    run_capture "$inspect_path" cargo run -p greentic-sorla -- pack inspect "$pack_path"
  done

  local answers_path answers_dir output_dir answers_copy pack_name pack_version
  while IFS= read -r answers_path; do
    answers_dir="$(dirname "$answers_path")"
    if [[ -f "${answers_dir}/sorla.yaml" ]]; then
      continue
    fi
    pack_name="$(jq -r '.package.name // empty' "$answers_path")"
    pack_version="$(jq -r '.package.version // empty' "$answers_path")"
    if [[ -z "$pack_name" || -z "$pack_version" ]]; then
      echo "ERROR: failed to resolve package name/version from ${answers_path}" >&2
      return 1
    fi
    safe_name="${pack_name//[^A-Za-z0-9_.-]/_}"
    output_dir="${tmp_dir}/${safe_name}-workspace"
    answers_copy="${tmp_dir}/${safe_name}.answers.json"
    pack_path="${output_dir}/${safe_name}.gtpack"
    inspect_path="${tmp_dir}/${safe_name}.answers.inspect.json"
    mkdir -p "$output_dir"
    jq --arg output_dir "$output_dir" '.output_dir = $output_dir' \
      "$answers_path" > "$answers_copy"
    run_cmd cargo run -p greentic-sorla -- wizard \
      --answers "$answers_copy" \
      --pack-out "${safe_name}.gtpack"
    run_cmd cargo run -p greentic-sorla -- pack doctor "$pack_path"
    run_capture "$inspect_path" cargo run -p greentic-sorla -- pack inspect "$pack_path"
  done < <(find examples -path '*/answers.json' -type f | LC_ALL=C sort)

  rm -rf "${tmp_dir}"
  trap - RETURN
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
for crate in greentic-sorla-lang greentic-sorla-ir greentic-sorla-pack greentic-sorla-lib greentic-sorla; do
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
run_cmd cargo clippy --workspace --all-targets --all-features -- -D warnings

run_step "cargo test"
run_cmd cargo test --all-features

run_step "Validation-enabled gtpack checks"
run_validation_pack_check

run_step "All example and package gtpack checks"
run_all_gtpack_fixture_check

run_step "Ontology handoff smoke"
run_cmd bash scripts/e2e/ontology-handoff-smoke.sh

run_step "cargo build"
run_cmd cargo build --all-features

run_step "WASM facade build"
if rustup target list --installed | grep -qx 'wasm32-wasip2'; then
  run_cmd cargo build -p greentic-sorla-lib --target wasm32-wasip2 --no-default-features --features wasm
else
  echo "[wasm] skipping greentic-sorla-lib wasm32-wasip2 build; install with: rustup target add wasm32-wasip2"
fi

run_step "cargo doc"
run_cmd cargo doc --no-deps --all-features

run_step "Packaging and publish dry-run checks"
prepare_publish_workspace
for entry in "${PUBLISHABLE_ENTRIES[@]}"; do
  crate="${entry%%$'\t'*}"
  patch_config=()
  while IFS= read -r -d '' arg; do
    patch_config+=("$arg")
  done < <(publish_patch_config_for "$crate")
  run_step "Package checks: ${crate}"
  if [[ "${CI:-}" == "true" ]]; then
    (cd "${PUBLISH_WORKSPACE_DIR}" && run_publish_check cargo package "${patch_config[@]}" --no-verify -p "$crate")
  else
    (cd "${PUBLISH_WORKSPACE_DIR}" && run_publish_check cargo package "${patch_config[@]}" --no-verify -p "$crate" --allow-dirty)
  fi
  if [[ "${CI:-}" == "true" ]]; then
    (cd "${PUBLISH_WORKSPACE_DIR}" && run_publish_check cargo package "${patch_config[@]}" -p "$crate")
  else
    (cd "${PUBLISH_WORKSPACE_DIR}" && run_publish_check cargo package "${patch_config[@]}" -p "$crate" --allow-dirty)
  fi
  if [[ "${CI:-}" == "true" ]]; then
    (cd "${PUBLISH_WORKSPACE_DIR}" && run_publish_check cargo publish "${patch_config[@]}" -p "$crate" --dry-run)
  else
    (cd "${PUBLISH_WORKSPACE_DIR}" && run_publish_check cargo publish "${patch_config[@]}" -p "$crate" --dry-run --allow-dirty)
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
