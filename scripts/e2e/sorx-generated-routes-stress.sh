#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

SORX_BIN="${SORX_BIN:-greentic-sorx}"
SORLA_PACK_CMD="${SORLA_PACK_CMD:-cargo run --quiet --bin greentic-sorla -- pack}"
PORT_BASE="${PORT_BASE:-8910}"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sorla-sorx-stress.XXXXXX")"
PIDS=()

cleanup() {
  for pid in "${PIDS[@]}"; do
    if kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  done
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

write_answers() {
  local path="$1"
  local port="$2"
  local sor_name="$3"
  cat >"$path" <<JSON
{
  "tenant": {
    "tenant_id": "e2e-stress",
    "environment": "test"
  },
  "server": {
    "bind": "127.0.0.1:${port}",
    "public_base_url": "http://127.0.0.1:${port}",
    "auth": {
      "mode": "none"
    }
  },
  "mcp": {
    "enabled": false,
    "bind": "127.0.0.1:$((port + 100))"
  },
  "providers": {
    "store": {
      "kind": "memory"
    }
  },
  "policy": {
    "approvals": {
      "low": "auto",
      "medium": "auto",
      "high": "auto",
      "critical": "auto"
    }
  },
  "audit": {
    "sink": "disabled"
  },
  "deployment": {
    "tenant_id": "e2e-stress",
    "sor_name": "${sor_name}",
    "environment": "test",
    "deployment_mode": "local_single",
    "api_version_label": "local",
    "base_path": "/"
  },
  "exposure": {
    "default_visibility": "private",
    "require_validation_suite": false,
    "auto_promote_on_validation_pass": false,
    "public_aliases_allowed": ["stable", "latest", "preview"]
  },
  "ghcr": {
    "enable_publish_webhook": false,
    "allowed_repositories": ["ghcr.io/greenticai/sorla-packages/*"],
    "require_exact_digest": true
  }
}
JSON
}

start_sorx() {
  local yaml="$1"
  local port="$2"
  local sor_name="$3"
  local pack="$TMP_DIR/${sor_name}.gtpack"
  local answers="$TMP_DIR/${sor_name}.answers.json"
  local log="$TMP_DIR/${sor_name}.log"

  # shellcheck disable=SC2086
  $SORLA_PACK_CMD "$yaml" --name "$sor_name" --version 0.1.0 --out "$pack" >/dev/null
  write_answers "$answers" "$port" "$sor_name"
  "$SORX_BIN" run "$pack" --non-interactive --answers "$answers" >"$log" 2>&1 &
  local pid="$!"
  PIDS+=("$pid")

  for _ in $(seq 1 80); do
    if curl -sS "http://127.0.0.1:${port}/readyz" >/dev/null 2>&1; then
      return 0
    fi
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      cat "$log" >&2
      return 1
    fi
    sleep 0.25
  done

  cat "$log" >&2
  echo "SORX did not become ready for ${pack}" >&2
  return 1
}

stop_sorx() {
  local pid="${PIDS[-1]}"
  kill "$pid" >/dev/null 2>&1 || true
  wait "$pid" >/dev/null 2>&1 || true
  unset "PIDS[-1]"
}

post_json() {
  local port="$1"
  local path="$2"
  local body="$3"
  local expect="${4:-\"ok\":true}"
  local response

  response="$(curl -sS -i \
    -H 'content-type: application/json' \
    -H 'x-greentic-tenant-id: e2e-stress' \
    -H 'x-greentic-team-id: qa' \
    -H 'x-greentic-caller-id: e2e-runner' \
    -X POST \
    --data "$body" \
    "http://127.0.0.1:${port}${path}")"
  if ! grep -q "$expect" <<<"$response"; then
    echo "Unexpected response from ${path}" >&2
    echo "$response" >&2
    return 1
  fi
  printf '%s\n' "$response"
}

run_complex_bulk() {
  local port="$1"
  start_sorx \
    "packages/complex_sorla_test_system/0.1.0/sorla.yaml" \
    "$port" \
    "complex-sorla-stress"

  post_json "$port" "/v1/agent/audit_trails/bulk_import_entities" \
    '{"items":[{"entity":"entity_a","collection":"entity_as","data":{"entity_a_id":"a-1","a_string_attr":"Alpha"}}]}' \
    '"imported_count":1' >/dev/null

  stop_sorx
}

run_demo_lifecycle_and_side_effects() {
  local port="$1"
  start_sorx \
    "packages/demo_sorla_full_coverage-v0.1.0-output/sorla.yaml" \
    "$port" \
    "demo-sorla-full-coverage-stress"

  local sample='{"entity_id":"sample-1","name":"Sample","created_at":"2026-05-23T00:00:00Z","is_active":true,"secret_token":"secret"}'
  post_json "$port" "/v1/agent/sample_entities/archive_sample_entity" "$sample" >/dev/null
  post_json "$port" "/v1/agent/sample_entities/restore_sample_entity" "$sample" >/dev/null
  post_json "$port" "/v1/agent/external_datas/trigger_policy_action" '{}' >/dev/null
  post_json "$port" "/v1/agent/external_datas/custom_workflow_action" '{}' >/dev/null

  stop_sorx
}

run_bug_edgecase_commands() {
  local port="$1"
  start_sorx \
    "packages/bug_sor_edgecase-pack/sorla.yaml" \
    "$port" \
    "bug-sor-edgecase-stress"

  local case_payload='{"case_id":"case-1","case_type":"logic","opened_at":"2026-05-23T00:00:00Z","is_active":true,"sensitive_info":"secret","owner_id":"owner-1","external_source_id":"src-1"}'
  local assignment_payload='{"assignment_id":"assign-1","case_id":"case-1","owner_id":"owner-1","status":"pending","approved_by":"owner-1","assigned_at":"2026-05-23T00:00:00Z"}'
  local asset_payload='{"asset_id":"asset-1","storage_url":"s3://bucket/asset-1","asset_type":"log","case_id":"case-1","is_archived":false}'

  post_json "$port" "/v1/agent/bug_cases/create" "$case_payload" >/dev/null
  post_json "$port" "/v1/agent/bug_assignments/apply_assignment" "$assignment_payload" >/dev/null
  post_json "$port" "/v1/agent/bug_assets/link_asset" "$asset_payload" >/dev/null
  post_json "$port" "/v1/agent/bug_cases/generate_case_token" "$case_payload" '"case_token"' >/dev/null
  post_json "$port" "/v1/agent/bug_assignments/approve_assignment" "$assignment_payload" '"status":"active"' >/dev/null
  post_json "$port" "/v1/agent/bug_assignments/reject_assignment" "$assignment_payload" '"status":"rejected"' >/dev/null
  post_json "$port" "/v1/agent/bug_assets/unlink_asset" "$asset_payload" '"deleted":1' >/dev/null
  post_json "$port" "/v1/agent/bug_assignments/bulk_import_assignments" \
    '{"items":[{"entity":"bug_assignment","collection":"bug_assignments","data":{"assignment_id":"assign-2","case_id":"case-1","owner_id":"owner-1","status":"pending","approved_by":"owner-1","assigned_at":"2026-05-23T00:00:00Z"}}]}' \
    '"imported_count":1' >/dev/null

  stop_sorx
}

run_complex_bulk "$PORT_BASE"
run_demo_lifecycle_and_side_effects "$((PORT_BASE + 1))"
run_bug_edgecase_commands "$((PORT_BASE + 2))"

echo "SORX generated route stress e2e passed."
