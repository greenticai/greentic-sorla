#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

ANSWERS_A="$TMP/answers-a.json"
ANSWERS_B="$TMP/answers-b.json"
WORK_A="$TMP/work-a"
WORK_B="$TMP/work-b"
PACK_A="$WORK_A/ontology-business.gtpack"
PACK_B="$WORK_B/ontology-business.gtpack"

mkdir -p "$WORK_A" "$WORK_B"

jq --arg output_dir "$WORK_A" '.output_dir = $output_dir' \
  "$ROOT/examples/ontology-business/answers.json" > "$ANSWERS_A"
jq --arg output_dir "$WORK_B" '.output_dir = $output_dir' \
  "$ROOT/examples/ontology-business/answers.json" > "$ANSWERS_B"

cargo run -p greentic-sorla -- wizard --answers "$ANSWERS_A" --pack-out ontology-business.gtpack >/dev/null
cargo run -p greentic-sorla -- wizard --answers "$ANSWERS_B" --pack-out ontology-business.gtpack >/dev/null

cmp -s "$PACK_A" "$PACK_B"

cargo run -p greentic-sorla -- pack doctor "$PACK_A" > "$TMP/doctor.json"
cargo run -p greentic-sorla -- pack inspect "$PACK_A" > "$TMP/inspect.json"
cargo run -p greentic-sorla -- pack validation-inspect "$PACK_A" > "$TMP/validation-inspect.json"

jq -e '
  .ontology.schema == "greentic.sorla.ontology.v1"
  and .ontology.concept_count == 7
  and .retrieval_bindings.schema == "greentic.sorla.retrieval-bindings.v1"
  and (.validation.promotion_requires | index("ontology"))
' "$TMP/inspect.json" >/dev/null

jq -e '
  .validation.promotion_requires | index("retrieval")
' "$TMP/validation-inspect.json" >/dev/null

grep -q '^ontology:$' "$WORK_A/sorla.yaml"
grep -q '^semantic_aliases:$' "$WORK_A/sorla.yaml"
grep -q '^entity_linking:$' "$WORK_A/sorla.yaml"
grep -q '^retrieval_bindings:$' "$WORK_A/sorla.yaml"

for generated in \
  "$WORK_A/.greentic-sorla/generated/launcher-handoff.json" \
  "$WORK_A/.greentic-sorla/generated/provider-requirements.json" \
  "$WORK_A/.greentic-sorla/generated/assets/sorx/tests/test-manifest.json"
do
  ! grep -Eiq 'password|api_key|tenant_id' "$generated"
  ! grep -Fq "$WORK_A" "$generated"
done

sha256sum "$PACK_A" "$PACK_B"
