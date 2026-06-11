# PR-01: Introduce `answers.json` v2 as Semantic Authoring Intent

## Summary

Refactor the SorLA prompt output so `answers.json` becomes a semantic authoring and delta-intent document rather than a near-complete representation of `sorla.yaml`.

The current prompt flow asks the LLM to produce a large `answers.json` that includes too many operational SorLA concepts. This causes malformed output, missing sections, inconsistent naming, fragile update behavior, and overdependence on high-capability LLMs.

This PR introduces a new versioned answers model:

```json
{
  "version": "sorla.answers.v2",
  "mode": "create | update",
  "intent": {},
  "domain": {},
  "operations": [],
  "compiler_options": {}
}
```

The LLM should produce semantic business/domain intent and high-level operations. Deterministic compiler stages will derive CRUD actions, events, projections, metrics, policies, migrations, provider requirements, and agent endpoints.

## Motivation

The existing `answers.json` format is useful because it can represent both new system creation and updates to an existing `sorla.yaml`. However, it has become too close to the full SorLA model.

We want to preserve the useful property:

```text
business prompt -> answers.json -> wizard -> sorla.yaml or sorla.yaml delta
```

But change the meaning of `answers.json`:

```text
answers.json = semantic authoring intent + delta operations
sorla.yaml = compiled/rendered operational model
```

## Goals

- Keep `answers.json` as the stable boundary between prompt and wizard.
- Add `sorla.answers.v2` with create/update modes.
- Support semantic delta operations for updating existing SorLA systems.
- Reduce LLM responsibility.
- Make downstream generated sections deterministic.
- Preserve backward compatibility with current `answers.json` where possible.

## Non-goals

- Do not remove the existing `answers.json` format yet.
- Do not rewrite the whole wizard renderer.
- Do not ask the LLM to generate final YAML.
- Do not implement all deterministic expanders in this PR.

## Proposed model

Add a new module:

```text
crates/greentic-sorla-lib/src/prompt/answers_v2.rs
```

Suggested Rust model:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswersV2 {
    pub version: String,
    pub mode: AnswersMode,
    pub intent: AuthoringIntent,
    #[serde(default)]
    pub domain: DomainIntent,
    #[serde(default)]
    pub operations: Vec<SemanticOperation>,
    #[serde(default)]
    pub compiler_options: CompilerOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswersMode {
    Create,
    Update,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthoringIntent {
    pub summary: Option<String>,
    #[serde(default)]
    pub goals: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub open_questions: Vec<ClarificationQuestion>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub id: String,
    pub question: String,
    pub reason: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainIntent {
    #[serde(default)]
    pub actors: Vec<ActorIntent>,
    #[serde(default)]
    pub records: Vec<RecordIntent>,
    #[serde(default)]
    pub processes: Vec<ProcessIntent>,
    #[serde(default)]
    pub business_rules: Vec<BusinessRuleIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorIntent {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordIntent {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub fields: Vec<FieldIntent>,
    #[serde(default)]
    pub relationships: Vec<RelationshipIntent>,
    #[serde(default)]
    pub lifecycle: Option<LifecycleIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldIntent {
    pub name: String,
    pub field_type: String,
    pub required: Option<bool>,
    #[serde(default)]
    pub values: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipIntent {
    pub name: Option<String>,
    pub target: String,
    pub cardinality: String,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleIntent {
    pub state_field: String,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub transitions: Vec<StateTransitionIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionIntent {
    pub from: Option<String>,
    pub to: String,
    pub actor: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessIntent {
    pub name: String,
    pub description: Option<String>,
    pub main_record: Option<String>,
    #[serde(default)]
    pub steps: Vec<ProcessStepIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStepIntent {
    pub name: String,
    pub actor: Option<String>,
    pub action: Option<String>,
    pub record: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRuleIntent {
    pub name: String,
    pub description: String,
    pub applies_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SemanticOperation {
    AddRecord { record: RecordIntent },
    UpdateRecord { record: String, changes: Value },
    RemoveRecord { record: String },
    RenameRecord { from: String, to: String },

    AddField { record: String, field: FieldIntent },
    UpdateField { record: String, field: String, changes: Value },
    RemoveField { record: String, field: String },
    RenameField { record: String, from: String, to: String },

    AddRelationship { record: String, relationship: RelationshipIntent },
    RemoveRelationship { record: String, relationship: String },

    AddActor { actor: ActorIntent },
    UpdateActor { actor: String, changes: Value },
    RemoveActor { actor: String },

    AddProcess { process: ProcessIntent },
    UpdateProcess { process: String, changes: Value },
    RemoveProcess { process: String },

    AddStateTransition { record: String, transition: StateTransitionIntent },
    RemoveStateTransition { record: String, from: Option<String>, to: String },

    AddBusinessRule { rule: BusinessRuleIntent },
    RemoveBusinessRule { rule: String },

    AddPolicyIntent { name: String, description: String, applies_to: Option<String> },
    AddMetricIntent { name: String, description: String, applies_to: Option<String> },
    AddProjectionIntent { name: String, description: String, applies_to: Option<String> },

    EnableCapability { capability: String },
    DisableCapability { capability: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerOptions {
    pub generate_crud: bool,
    pub generate_events: bool,
    pub generate_lifecycle_events: bool,
    pub generate_search: bool,
    pub generate_agent_endpoints: bool,
    pub generate_projections: bool,
    pub generate_metrics: bool,
    pub generate_default_policies: bool,
    pub generate_migrations: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            generate_crud: true,
            generate_events: true,
            generate_lifecycle_events: true,
            generate_search: true,
            generate_agent_endpoints: true,
            generate_projections: true,
            generate_metrics: true,
            generate_default_policies: true,
            generate_migrations: true,
        }
    }
}
```

## Create-mode example

```json
{
  "version": "sorla.answers.v2",
  "mode": "create",
  "intent": {
    "summary": "Create a landlord, tenant and contractor maintenance system"
  },
  "domain": {
    "actors": [
      { "name": "tenant", "description": "Reports issues" },
      { "name": "landlord", "description": "Approves work and payment" },
      { "name": "contractor", "description": "Quotes and performs work" }
    ],
    "records": [
      {
        "name": "maintenance_request",
        "description": "A tenant-reported maintenance issue",
        "fields": [
          { "name": "description", "field_type": "text", "required": true },
          { "name": "status", "field_type": "enum", "values": ["reported", "quoted", "approved", "scheduled", "completed", "rejected"] }
        ],
        "lifecycle": {
          "state_field": "status",
          "states": ["reported", "quoted", "approved", "scheduled", "completed", "rejected"],
          "transitions": [
            { "from": "reported", "to": "quoted", "actor": "contractor" },
            { "from": "quoted", "to": "approved", "actor": "landlord" },
            { "from": "approved", "to": "scheduled", "actor": "contractor" },
            { "from": "scheduled", "to": "completed", "actor": "contractor" }
          ]
        }
      }
    ]
  }
}
```

## Update-mode example

```json
{
  "version": "sorla.answers.v2",
  "mode": "update",
  "intent": {
    "summary": "Add contractor quotes to the maintenance request flow"
  },
  "operations": [
    {
      "op": "add_record",
      "record": {
        "name": "quote",
        "description": "A contractor quote for a maintenance request",
        "fields": [
          { "name": "amount", "field_type": "money", "required": true },
          { "name": "status", "field_type": "enum", "values": ["draft", "submitted", "approved", "rejected"] }
        ],
        "relationships": [
          { "name": "maintenance_request", "target": "maintenance_request", "cardinality": "many_to_one", "required": true }
        ]
      }
    },
    {
      "op": "add_state_transition",
      "record": "quote",
      "transition": { "from": "submitted", "to": "approved", "actor": "landlord" }
    }
  ]
}
```

## Implementation tasks

1. Add `prompt/answers_v2.rs`.
2. Add serde model and default compiler options.
3. Add validation helper:

```rust
pub fn validate_answers_v2(answers: &AnswersV2) -> Result<(), PromptError>
```

Validation should check:

- `version == "sorla.answers.v2"`.
- names are non-empty.
- record names are unique.
- actor names are unique.
- fields within a record are unique.
- relationships reference existing or operation-added records when resolvable.
- lifecycle state field exists or is implied by a field with enum type.
- state transitions point to known states where possible.

4. Add detection helper:

```rust
pub fn is_answers_v2_json(value: &serde_json::Value) -> bool
```

5. Add conversion placeholder:

```rust
pub fn answers_v2_to_legacy_answers(v2: AnswersV2) -> Result<LegacyAnswersDocument, PromptError>
```

Initially this can map only core records, actors, and package metadata. Later PRs will fill deterministic expansions.

## Tests

Add tests for:

- create-mode answers parsing.
- update-mode operation parsing.
- default compiler options.
- duplicate record rejection.
- duplicate field rejection.
- lifecycle transition validation.
- backward compatibility detection.

Suggested file:

```text
crates/greentic-sorla-lib/src/prompt/answers_v2_tests.rs
```

## Acceptance criteria

- `sorla.answers.v2` JSON can be parsed and validated.
- Existing answers format remains supported.
- New model supports both create and update modes.
- No renderer changes are required yet.
- Tests cover create and update examples.
