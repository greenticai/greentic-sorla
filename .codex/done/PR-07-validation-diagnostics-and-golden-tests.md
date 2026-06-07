# PR-07: Add Validation, Diagnostics, and Golden Tests for SorLA Prompt/Wizard Pipeline

## Summary

Add robust validation, diagnostics, and golden tests for the new semantic answers and deterministic compiler pipeline.

This PR makes the prompt/wizard system testable end-to-end:

```text
business prompt fixture
  -> fake LLM domain output
  -> answers.json v2
  -> deterministic expansion
  -> sorla.yaml
  -> golden comparison
```

## Motivation

The current system relies heavily on LLM repair prompts and full-shape validation. As SorLA grows, we need deterministic tests for the mechanical parts.

Golden tests will prevent regressions in:

- naming conventions.
- generated events.
- generated actions.
- generated agent endpoints.
- generated projections.
- update-mode deltas.

## Add diagnostics model

```rust
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

pub struct CompileDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    pub suggestion: Option<String>,
}
```

Example diagnostics:

```text
SORLA_DUPLICATE_RECORD
SORLA_UNKNOWN_RELATIONSHIP_TARGET
SORLA_DESTRUCTIVE_OPERATION_BLOCKED
SORLA_LIFECYCLE_STATE_FIELD_MISSING
SORLA_POLICY_REVIEW_WARNING
SORLA_NAME_NORMALIZED
```

## Add validation stages

### 1. AnswersV2 validation

Checks JSON shape and basic domain consistency.

### 2. Semantic operation validation

Checks operations before application.

### 3. Expanded plan validation

Checks after deterministic expansion:

- no duplicate actions.
- no duplicate events.
- no duplicate projections.
- every action references an existing record where required.
- every lifecycle event references a valid transition.
- every policy references known actors/records where possible.

### 4. Render validation

Checks generated YAML can be parsed back into known structures.

## Golden test directory

Add:

```text
crates/greentic-sorla-lib/tests/golden/
  maintenance_create/
    prompt.txt
    llm_domain_output.json
    expected_answers.json
    expected_expanded_plan.json
    expected_sorla.yaml
  maintenance_add_quote_update/
    existing_sorla.yaml
    user_update.txt
    llm_update_output.json
    expected_diff.txt
    expected_sorla.yaml
```

## Fake LLM tests

Use fake provider output to avoid external model calls.

Test harness:

```rust
#[test]
fn golden_maintenance_create_pipeline() {
    // Load prompt.txt
    // Use fake LLM response llm_domain_output.json
    // Run prompt engine
    // Compile answers v2
    // Render YAML
    // Compare with expected files
}
```

## Snapshot strategy

For generated YAML, use normalized comparison:

- normalize line endings.
- ignore timestamp/comment noise if any.
- preserve ordering of deterministic sections.

## Deterministic ordering

Ensure all generated lists are stable:

```text
records sorted by insertion/domain order.
actions sorted by record order then action kind.
events sorted by record order then event kind.
projections sorted by record order.
agent endpoints sorted by record order then operation kind.
```

Avoid HashMap iteration order in rendered output.

## CLI diagnostics

Add `--diagnostics-out`:

```bash
greentic-sorla wizard --answers answers.json --diagnostics-out diagnostics.json
```

Example output:

```json
[
  {
    "severity": "warning",
    "code": "SORLA_UNKNOWN_RELATIONSHIP_TARGET",
    "message": "Relationship quote.contractor references contractor, which exists as an actor but not as a record.",
    "path": "domain.records.quote.relationships.contractor",
    "suggestion": "Create contractor as a record or mark this as actor-only."
  }
]
```

## Acceptance criteria

- Validation produces structured diagnostics.
- Golden tests cover create and update flows.
- Compiler output ordering is deterministic.
- CLI can emit diagnostics JSON.
- Generated YAML can be parsed/validated after rendering.
