# PR-03 — Add semantic patch model and patch application engine for sorla.yaml

Repository: `greenticai/greentic-sorla`

## Goal

Add a semantic patch protocol for editing `sorla.yaml` safely from Designer, CLI or LLM-assisted flows.

The Designer should not rewrite raw YAML directly. It should send semantic patches that the library applies, validates and renders back to canonical YAML.

This PR depends on PR-01 and PR-02. It should patch the parsed/canonical SoRLa model and then render deterministic YAML; it should not manipulate YAML with ad hoc string edits.

## Architecture

```text
sorla.yaml
  + SorlaPatch
  -> apply_sorla_patch()
  -> updated sorla.yaml
  -> diagnostics
  -> ConceptDiff
  -> ConceptViewModel
```

## Patch schema

Use:

```text
greentic.sorla.patch.v1
```

## Required API

Add to `greentic-sorla-lib`:

```rust
pub struct ApplyPatchInput {
    pub source_yaml: String,
    pub patch: SorlaPatch,
}

pub struct ApplyPatchOutput {
    pub updated_yaml: String,
    pub old_hash: String,
    pub new_hash: String,
    pub diagnostics: Vec<SorlaDiagnostic>,
    pub diff: ConceptDiff,
    pub view: ConceptViewModel,
}

pub fn apply_sorla_patch(input: ApplyPatchInput) -> Result<ApplyPatchOutput, SorlaError>;
```

## Patch envelope

```rust
pub struct SorlaPatch {
    pub schema: String,
    pub source: SorlaPatchSource,
    pub author: Option<SorlaPatchAuthor>,
    pub intent: Option<String>,
    pub operations: Vec<SorlaPatchOperation>,
}
```

Source:

```rust
pub struct SorlaPatchSource {
    pub kind: SorlaSourceKind,
    pub path: Option<String>,
    pub base_hash: String,
}
```

If `base_hash` does not match the current source hash, return a conflict result/error.

## Operation model

Use semantic operations, not raw JSON Patch.

Minimum operations:

### Records

```text
add_record
rename_record
delete_record
add_field
update_field
remove_field
```

### Relationships

```text
add_relationship
update_relationship
remove_relationship
```

### Events

```text
add_event
update_event
remove_event
```

### Projections

```text
add_projection
update_projection
remove_projection
```

### Metrics

```text
add_metric
update_metric
remove_metric
```

Metrics are deferred unless PR-01 proves metrics are already supported by the current SoRLa language and lowering pipeline. Do not add a metric editing surface by inventing a new YAML shape here.

### Policies / approvals

```text
add_policy
update_policy
remove_policy
add_approval
update_approval
remove_approval
```

### Agent endpoints

```text
add_agent_endpoint
update_agent_endpoint
remove_agent_endpoint
```

The MVP operation set should be limited to fields that the current parser and canonical renderer can round-trip: records, fields, relationships/ontology, events, projections, policies, approvals, agent endpoints, provider requirements, retrieval/ontology entries as applicable. Add operations incrementally with tests instead of declaring unsupported language features editable.

## Conflict handling

If base hash mismatches, return:

```json
{
  "status": "conflict",
  "reason": "base_hash_mismatch",
  "base_hash": "sha256:old",
  "current_hash": "sha256:new",
  "resolution": "refresh_required"
}
```

## Concept diff

Add:

```text
greentic.sorla.concept-diff.v1
```

Types:

```rust
pub struct ConceptDiff {
    pub schema: String,
    pub changes: Vec<ConceptChange>,
}

pub struct ConceptChange {
    pub kind: ConceptChangeKind,
    pub target: String,
    pub label: String,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}
```

Kinds:

```text
added
updated
removed
renamed
conflict
warning
```

## YAML rendering

After applying patches:

- validate
- render canonical deterministic `sorla.yaml`
- preserve comments if feasible, but do not make comment preservation a blocker
- stable ordering should be documented

Existing generated-file marker logic is for generated artifacts and answer-derived outputs. Do not rely on generated blocks as the safety mechanism for YAML source-of-truth editing unless the current YAML renderer already owns those blocks.

## Undo/redo

Where practical, generate inverse operations. This may be optional for MVP, but the patch engine should be designed not to block undo/redo later.

## Acceptance criteria

- Semantic patch types exist and are serializable.
- Patch application updates YAML deterministically.
- Base hash mismatch is detected.
- Patch result includes diagnostics, concept diff and refreshed concept view.
- Tests cover add/update/remove record field.
- Tests cover metrics only if the metric model already exists.
- Tests cover invalid patch rejection.
- Tests cover conflict handling.
