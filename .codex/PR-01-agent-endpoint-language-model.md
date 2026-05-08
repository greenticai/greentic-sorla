# PR-01 — Add Agent Endpoint Language Model

## Goal

Introduce first-class SoRLa authoring types for agentic endpoints without adding runtime gateway behavior to `greentic-sorla`.

This PR only extends the language-facing AST and serialization model. It does not lower to canonical IR yet and does not generate handoff artifacts.

## Rationale

`greentic-sorla` already models records, events, actions, policies, approvals, views, flows, projections, migrations, and provider requirements. Agentic endpoints should become a first-class authoring concept that ties these pieces together into an agent-understandable business capability.

The model should describe:

- intent
- required inputs
- outputs
- side effects
- risk
- approval behavior
- backing actions/events/flows
- provider requirements
- agent-facing examples

## Files to touch

- `crates/greentic-sorla-lang/src/ast.rs`
- `crates/greentic-sorla-lang/src/lib.rs`
- `crates/greentic-sorla-lang/src/parser.rs` if parser normalization currently lives there
- Add/extend tests in `crates/greentic-sorla-lang/src/lib.rs` or parser test module

## Current-code notes

- The authoring AST currently lives entirely in `crates/greentic-sorla-lang/src/ast.rs`.
- Parser normalization/validation lives in `parser.rs`, but this PR should only add serde-compatible language types and round-trip/default tests. Save semantic validation for PR-02.
- `ProviderRequirement` already exists in `ast.rs`; endpoint-level provider requirements should reuse that type.
- `Package` currently uses `#[serde(deny_unknown_fields)]`, so adding `agent_endpoints` is required before any fixture containing that key will parse.

## Proposed AST changes

Add to `Package`:

```rust
#[serde(default)]
pub agent_endpoints: Vec<AgentEndpointDecl>,
```

Add types:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointDecl {
    pub id: String,
    pub title: String,
    pub intent: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub inputs: Vec<AgentEndpointInputDecl>,

    #[serde(default)]
    pub outputs: Vec<AgentEndpointOutputDecl>,

    #[serde(default)]
    pub side_effects: Vec<String>,

    #[serde(default)]
    pub risk: AgentEndpointRisk,

    #[serde(default)]
    pub approval: AgentEndpointApprovalMode,

    #[serde(default)]
    pub provider_requirements: Vec<ProviderRequirement>,

    #[serde(default)]
    pub backing: AgentEndpointBackingDecl,

    #[serde(default)]
    pub agent_visibility: AgentEndpointVisibility,

    #[serde(default)]
    pub examples: Vec<AgentEndpointExampleDecl>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointInputDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub sensitive: bool,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointOutputDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub description: Option<String>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointRisk {
    #[default]
    Low,
    Medium,
    High,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointApprovalMode {
    #[default]
    None,
    Optional,
    Required,
    PolicyDriven,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointBackingDecl {
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub flows: Vec<String>,
    #[serde(default)]
    pub policies: Vec<String>,
    #[serde(default)]
    pub approvals: Vec<String>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointVisibility {
    #[serde(default = "default_true")]
    pub openapi: bool,
    #[serde(default = "default_true")]
    pub arazzo: bool,
    #[serde(default = "default_true")]
    pub mcp: bool,
    #[serde(default = "default_true")]
    pub llms_txt: bool,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointExampleDecl {
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub input: serde_json::Value,
    #[serde(default)]
    pub expected_output: serde_json::Value,
}
```

Add a private helper:

```rust
fn default_true() -> bool {
    true
}
```

## Example SoRLa YAML

```yaml
package:
  name: website-lead-capture
  version: 0.2.0

records:
  - name: Contact
    source: hybrid
    external_ref:
      system: hubspot
      key: email
      authoritative: true
    fields:
      - name: email
        type: string
        authority: external
      - name: contact_persona
        type: string
        authority: external
      - name: problem_to_solve
        type: string
        authority: local

actions:
  - name: UpsertContact

events:
  - name: ContactCaptured
    record: Contact
    kind: integration

approvals:
  - name: ReviewHighValuePartner

agent_endpoints:
  - id: create_customer_contact
    title: Create customer contact
    intent: Capture a customer website enquiry and create or update the CRM contact.
    description: Use this when a visitor wants to learn more as a customer.
    inputs:
      - name: email
        type: string
        required: true
        sensitive: true
      - name: company_name
        type: string
        required: true
      - name: company_size
        type: string
        required: false
      - name: problem_to_solve
        type: string
        required: true
    outputs:
      - name: contact_id
        type: string
    side_effects:
      - crm.contact.upsert
      - event.ContactCaptured
    risk: medium
    approval: optional
    provider_requirements:
      - category: crm
        capabilities:
          - contacts.read
          - contacts.write
    backing:
      actions:
        - UpsertContact
      events:
        - ContactCaptured
    agent_visibility:
      openapi: true
      arazzo: true
      mcp: true
      llms_txt: true
```

## Tests

Add tests that confirm:

1. A package without `agent_endpoints` still parses.
2. A minimal endpoint parses.
3. A full endpoint with inputs, outputs, provider requirements, backing references, and visibility flags parses.
4. Unknown fields are rejected.
5. Defaults apply:
   - `risk = low`
   - `approval = none`
   - visibility flags default to `true`

## Acceptance criteria

- `cargo test -p greentic-sorla-lang` passes.
- Existing fixtures/tests continue to pass.
- The new AST compiles under edition 2024.
- No runtime, gateway, OpenAPI, Arazzo, MCP, or `gtc` behavior is introduced in this PR.
