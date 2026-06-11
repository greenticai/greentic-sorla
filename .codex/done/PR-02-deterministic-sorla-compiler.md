# PR-02: Add Deterministic SorLA Compiler and Expansion Pipeline

## Summary

Introduce a deterministic compiler layer that expands semantic `answers.json` v2 into a richer SorLA plan before rendering YAML.

This PR adds an internal `ExpandedSorlaPlan` and an expander pipeline. The first set of expanders should derive standard CRUD actions, record events, search endpoints, agent endpoints, projections, metrics, policies, and migrations from records and lifecycles.

## Motivation

The LLM should not generate mechanical sections such as:

- `record.create`
- `record.updated`
- `record.search`
- `record_detail`
- `agent.create_record`
- basic count metrics
- basic migrations

These should be deterministic consequences of the domain model.

This reduces prompt fragility and improves repeatability.

## Proposed architecture

```text
AnswersV2
  -> normalize domain
  -> apply semantic operations
  -> ExpandedSorlaPlan
  -> run deterministic expanders
  -> validate plan
  -> convert to legacy ResolvedAnswers or directly render YAML
```

Add:

```text
crates/greentic-sorla-lib/src/compiler/mod.rs
crates/greentic-sorla-lib/src/compiler/plan.rs
crates/greentic-sorla-lib/src/compiler/expanders.rs
crates/greentic-sorla-lib/src/compiler/provenance.rs
```

## Core data structures

```rust
pub struct ExpansionContext {
    pub mode: CompileMode,
    pub options: CompilerOptions,
    pub naming: NamingRules,
}

pub enum CompileMode {
    Create,
    Update,
}

pub struct ExpandedSorlaPlan {
    pub package: Option<PackagePlan>,
    pub actors: Vec<ActorPlan>,
    pub records: Vec<RecordPlan>,
    pub actions: Vec<ActionPlan>,
    pub events: Vec<EventPlan>,
    pub projections: Vec<ProjectionPlan>,
    pub metrics: Vec<MetricPlan>,
    pub policies: Vec<PolicyPlan>,
    pub migrations: Vec<MigrationPlan>,
    pub agent_endpoints: Vec<AgentEndpointPlan>,
    pub diagnostics: Vec<CompileDiagnostic>,
}
```

Every generated object should carry provenance:

```rust
pub enum Provenance {
    UserProvided,
    LlmGenerated { agent: String, reason: Option<String> },
    DeterministicRule { rule: String, source: String },
    ExistingYaml { path: Option<String> },
}
```

Example:

```rust
pub struct EventPlan {
    pub name: String,
    pub record: Option<String>,
    pub description: Option<String>,
    pub provenance: Provenance,
}
```

## Expander trait

```rust
pub trait SorlaExpander {
    fn name(&self) -> &'static str;
    fn expand(&self, ctx: &ExpansionContext, plan: &mut ExpandedSorlaPlan) -> Result<(), CompileError>;
}
```

## Initial expanders

Implement these in order:

```rust
RecordCrudExpander
RecordEventExpander
LifecycleEventExpander
SearchEndpointExpander
AgentEndpointExpander
ProjectionExpander
MetricExpander
PolicyDefaultExpander
MigrationExpander
```

## Expansion rules

### RecordCrudExpander

For each record `x`, generate:

```text
x.create
x.get
x.update
x.delete
x.list
```

If soft delete is enabled later, also generate:

```text
x.archive
x.restore
```

### RecordEventExpander

For each record `x`, generate:

```text
x.created
x.updated
x.deleted
```

Optionally:

```text
x.viewed
x.searched
```

but only if these are useful in audit mode.

### LifecycleEventExpander

For each lifecycle transition:

```json
{ "from": "submitted", "to": "approved", "actor": "manager" }
```

Generate:

```text
x.approved
x.approve
agent.approve_x
```

Avoid duplicate events if the state name already maps to a generated event.

### SearchEndpointExpander

For each record `x`, generate:

```text
x.search
x.lookup
x.filter
```

or whatever the existing SorLA agent endpoint conventions require.

### AgentEndpointExpander

For each CRUD action:

```text
agent.create_x
agent.get_x
agent.update_x
agent.delete_x
agent.search_x
```

For lifecycle transitions:

```text
agent.approve_quote
agent.reject_quote
agent.submit_purchase_request
```

### ProjectionExpander

For each record `x`:

```text
x_list
x_detail
x_search
```

For each relationship:

```text
parent_children
```

Example:

```text
property_maintenance_requests
customer_invoices
```

For each lifecycle field:

```text
x_by_status
```

### MetricExpander

For each record `x`:

```text
count_x
x_created_per_day
x_updated_per_day
```

For each lifecycle:

```text
x_by_status
average_time_to_<state>
```

For each transition:

```text
<from>_to_<to>_rate
```

Example:

```text
quote_approval_rate
average_time_to_quote_approved
```

### PolicyDefaultExpander

Generate safe baseline policies:

```text
creator can view own record
creator can update own draft records
admin can manage all
transition actor can perform transition
```

These should be conservative. They can later be refined by a policy reviewer agent.

### MigrationExpander

For each added record:

```text
create table/collection
```

For each added field:

```text
add column/property
```

For each removed field:

```text
mark deprecated first, do not hard-delete by default
```

## Naming rules

Add deterministic naming normalization:

```rust
pub struct NamingRules {
    pub record_case: CaseStyle,
    pub action_separator: String,
    pub event_separator: String,
}
```

Defaults:

```text
records: snake_case
fields: snake_case
actions: record.verb
events: record.past_tense
agent_endpoints: agent.verb_record
projections: record_suffix
```

## Compiler entry point

```rust
pub fn compile_answers_v2(answers: AnswersV2, existing: Option<ExistingSorlaModel>) -> Result<ExpandedSorlaPlan, CompileError>
```

For create mode:

```text
DomainIntent -> ExpandedSorlaPlan -> expanders
```

For update mode:

```text
ExistingSorlaModel + operations -> ExpandedSorlaPlan -> expanders
```

## Tests

Add tests for:

- record produces CRUD actions.
- record produces standard events.
- lifecycle produces transition events.
- relationship produces relationship projections.
- compiler does not generate duplicates.
- generated items contain provenance.
- compiler options can disable categories.

Example test:

```rust
#[test]
fn add_quote_record_generates_crud_events_search_and_agent_endpoints() {
    // Given an AnswersV2 with a quote record
    // When compile_answers_v2 runs
    // Then quote.create, quote.created, quote.search, agent.create_quote exist
}
```

## Acceptance criteria

- `AnswersV2` can be compiled into an `ExpandedSorlaPlan`.
- Standard CRUD, events, projections, metrics and agent endpoints are generated deterministically.
- Generated items have provenance.
- Expanders are individually testable.
- Duplicate generation is prevented.
