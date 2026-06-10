# PR-03: Refactor Update Mode to Use Semantic Delta Operations

## Summary

Refactor `greentic-sorla prompt --sorla-yaml <file>` so update mode produces semantic delta operations instead of attempting to regenerate a full `answers.json` or patch raw YAML directly.

The LLM should produce operations like:

```json
{ "op": "add_record", "record": { "name": "quote" } }
```

not low-level YAML changes such as:

```json
{ "op": "append_yaml", "path": "/events", "value": { "name": "quote.created" } }
```

The deterministic compiler then derives consequences.

## Motivation

The existing update mode reads the existing YAML plus lock state, wraps the user request as an update prompt, and forces the generated answers into `flow: update`.

This is directionally correct, but too fragile if the LLM must reason over the entire YAML model and output a full or near-full operational structure.

A better model is:

```text
existing sorla.yaml
  -> parse to ExistingSorlaModel
user update prompt
  -> LLM generates semantic operations
semantic operations
  -> deterministic apply
  -> deterministic expansion
  -> diff preview
  -> updated sorla.yaml
```

## Goals

- Preserve the value of `answers.json` as a delta boundary.
- Make update mode semantic rather than YAML-path-based.
- Avoid complete regeneration for small updates.
- Produce understandable diffs.
- Prevent destructive operations unless explicit.

## Non-goals

- Do not implement a full bidirectional YAML parser if one already exists.
- Do not expose internal operation IDs to end users unless useful for debugging.
- Do not allow arbitrary raw YAML patch operations in LLM output.

## Proposed flow

Current:

```text
existing sorla.yaml + user request -> LLM -> answers.json update -> wizard
```

New:

```text
existing sorla.yaml
  -> ExistingSorlaModel
existing model + user request
  -> LLM SemanticOperation[]
operations
  -> apply to model
  -> deterministic expanders
  -> diff
  -> render updated sorla.yaml
```

## New module

```text
crates/greentic-sorla-lib/src/compiler/update.rs
```

Suggested public API:

```rust
pub fn plan_update_operations(
    existing: &ExistingSorlaModel,
    user_request: &str,
    llm: &dyn PromptLlm,
) -> Result<AnswersV2, PromptError>;

pub fn apply_semantic_operations(
    existing: ExistingSorlaModel,
    operations: Vec<SemanticOperation>,
) -> Result<ExistingSorlaModel, CompileError>;
```

## ExistingSorlaModel

Add a normalized model parsed from existing YAML:

```rust
pub struct ExistingSorlaModel {
    pub package: Option<PackagePlan>,
    pub actors: Vec<ActorPlan>,
    pub records: Vec<RecordPlan>,
    pub actions: Vec<ActionPlan>,
    pub events: Vec<EventPlan>,
    pub projections: Vec<ProjectionPlan>,
    pub metrics: Vec<MetricPlan>,
    pub policies: Vec<PolicyPlan>,
    pub agent_endpoints: Vec<AgentEndpointPlan>,
}
```

If existing internal structures already cover this, reuse them.

## Semantic operation application behavior

### `add_record`

- Reject if record already exists unless `merge_if_exists` is later added.
- Add record and fields.
- Add relationships.
- Mark new record as user/LLM-provided.
- Let expanders generate actions, events, projections, metrics, etc.

### `add_field`

- Reject if record does not exist.
- Reject if field already exists unless update is explicit.
- Add migration plan.
- Regenerate affected projections/search endpoints if enabled.

### `remove_field`

- Do not hard-delete by default.
- Mark deprecated or generate a warning requiring explicit destructive flag.

### `remove_record`

- Treat as destructive.
- Require explicit confirmation marker in operation:

```json
{
  "op": "remove_record",
  "record": "quote",
  "destructive_confirmed": true
}
```

If the enum cannot include this yet, reject destructive operation by default and produce diagnostic.

### `add_state_transition`

- Ensure lifecycle exists.
- Add missing state if needed only when safe.
- Generate lifecycle event and endpoint via deterministic expanders.

## Prompt changes

Update the update prompt to instruct the LLM:

```text
Return only semantic SorLA update operations.
Do not emit YAML.
Do not emit derived CRUD actions, events, projections, metrics or agent endpoints unless explicitly requested as manual overrides.
When a record, field, relationship or lifecycle transition is added, the compiler will derive the mechanical SorLA sections.
```

## Example update prompt output

User says:

```text
Add quote approval so contractors can submit quotes and landlords approve or reject them.
```

LLM output:

```json
{
  "version": "sorla.answers.v2",
  "mode": "update",
  "intent": {
    "summary": "Add contractor quote submission and landlord approval"
  },
  "operations": [
    {
      "op": "add_record",
      "record": {
        "name": "quote",
        "description": "A contractor quote for work related to a maintenance request",
        "fields": [
          { "name": "amount", "field_type": "money", "required": true },
          { "name": "description", "field_type": "text", "required": false },
          { "name": "status", "field_type": "enum", "values": ["draft", "submitted", "approved", "rejected"] }
        ],
        "relationships": [
          { "name": "maintenance_request", "target": "maintenance_request", "cardinality": "many_to_one", "required": true },
          { "name": "contractor", "target": "contractor", "cardinality": "many_to_one", "required": true }
        ],
        "lifecycle": {
          "state_field": "status",
          "states": ["draft", "submitted", "approved", "rejected"],
          "transitions": [
            { "from": "draft", "to": "submitted", "actor": "contractor" },
            { "from": "submitted", "to": "approved", "actor": "landlord" },
            { "from": "submitted", "to": "rejected", "actor": "landlord" }
          ]
        }
      }
    }
  ]
}
```

Compiler-derived consequences include:

```text
quote.create
quote.get
quote.update
quote.delete
quote.list
quote.search
quote.created
quote.updated
quote.deleted
quote.submitted
quote.approved
quote.rejected
quote_list
quote_detail
quotes_by_status
agent.create_quote
agent.search_quote
agent.submit_quote
agent.approve_quote
agent.reject_quote
quote_approval_rate
average_time_to_quote_approved
```

## Diff preview

Add a diff model:

```rust
pub struct SorlaDiffPreview {
    pub added: Vec<DiffItem>,
    pub changed: Vec<DiffItem>,
    pub removed: Vec<DiffItem>,
    pub warnings: Vec<CompileDiagnostic>,
}
```

Example CLI output:

```text
Update plan:

Added records:
  + quote

Added fields:
  + quote.amount
  + quote.description
  + quote.status

Derived additions:
  + 5 actions
  + 6 events
  + 4 agent endpoints
  + 3 projections
  + 2 metrics

Warnings:
  ! contractor record does not exist; inferred as actor only
```

## CLI behavior

For update mode:

```bash
greentic-sorla prompt --sorla-yaml sorla.yaml "add quote approval" --answers-out update.answers.json
```

Should output `sorla.answers.v2` with mode `update`.

Then:

```bash
greentic-sorla wizard --answers update.answers.json --sorla-yaml sorla.yaml --out sorla.updated.yaml
```

Should apply update operations and render updated YAML.

## Tests

Add tests for:

- adding a new record to existing SorLA model.
- adding a field to existing record.
- adding lifecycle transition.
- rejecting remove record without destructive confirmation.
- ensuring derived CRUD/events are created after update operations.
- producing diff preview.

## Acceptance criteria

- Update mode produces semantic operations in `answers.json` v2.
- Wizard can apply operations to existing model.
- Derived sections are generated deterministically after patching.
- Destructive changes are blocked unless explicitly confirmed.
- CLI can show a human-readable diff preview.
