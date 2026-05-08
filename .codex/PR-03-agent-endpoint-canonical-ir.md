# PR-03 — Lower Agent Endpoints Into Canonical IR

## Goal

Add deterministic canonical IR support for agentic endpoints.

This PR should preserve current deterministic sorting and canonical hash behavior.

## Files to touch

- `crates/greentic-sorla-ir/src/lib.rs`
- `crates/greentic-sorla-lang/src/ast.rs` if shared enums need mapping helpers
- IR tests in `crates/greentic-sorla-ir/src/lib.rs`

## Current-code notes

- `CanonicalIr` currently has no `agent_endpoints` field, and `inspect_ir()` serializes the full struct with `serde_json::to_string_pretty`.
- Existing IR field naming uses Rust field names such as `type_name` in JSON, not `type`; keep that convention unless this PR deliberately introduces serde renames for IR.
- `canonical_hash_hex()` hashes `canonical_cbor(&ir)`. Adding a non-optional `agent_endpoints: Vec<_>` field will change hashes for all packages, even when empty. Update golden fixtures and tests intentionally.
- `agent_tools_json()` currently returns a small `BTreeMap` with `package` and `storage-provider-categories`. Extending it with `agent-endpoints` is the right place for a minimal summary before PR-06 exporters.

## IR changes

Add to `CanonicalIr`:

```rust
pub agent_endpoints: Vec<AgentEndpointIr>,
```

Add types:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointIr {
    pub id: String,
    pub title: String,
    pub intent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub inputs: Vec<AgentEndpointInputIr>,
    pub outputs: Vec<AgentEndpointOutputIr>,
    pub side_effects: Vec<String>,
    pub risk: AgentEndpointRiskIr,
    pub approval: AgentEndpointApprovalModeIr,
    pub provider_requirements: Vec<ProviderRequirementIr>,
    pub backing: AgentEndpointBackingIr,
    pub agent_visibility: AgentEndpointVisibilityIr,
    pub examples: Vec<AgentEndpointExampleIr>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointInputIr {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enum_values: Vec<String>,
    pub sensitive: bool,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointOutputIr {
    pub name: String,
    pub type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointRiskIr {
    Low,
    Medium,
    High,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointApprovalModeIr {
    None,
    Optional,
    Required,
    PolicyDriven,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointBackingIr {
    pub actions: Vec<String>,
    pub events: Vec<String>,
    pub flows: Vec<String>,
    pub policies: Vec<String>,
    pub approvals: Vec<String>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointVisibilityIr {
    pub openapi: bool,
    pub arazzo: bool,
    pub mcp: bool,
    pub llms_txt: bool,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointExampleIr {
    pub name: String,
    pub summary: String,
    pub input: serde_json::Value,
    pub expected_output: serde_json::Value,
}
```

## Deterministic lowering rules

- Sort `agent_endpoints` by `id`.
- Sort endpoint inputs by `name`, unless preserving required-first behavior is already a repo convention. Prefer pure lexical sorting for deterministic IR.
- Sort outputs by `name`.
- Sort `side_effects` lexically.
- Sort provider requirements by category and capabilities lexically using existing `ProviderRequirementIr` logic.
- Sort backing arrays lexically.
- Sort examples by `name`.
- Sort enum values lexically.

## Update `lower_package`

Add lowering from `package.agent_endpoints` into `CanonicalIr.agent_endpoints`.

Also update `agent_tools_json` to include minimal agent endpoint information, for example:

```json
{
  "package": "website-lead-capture",
  "storage-provider-categories": "storage",
  "agent-endpoints": "create_customer_contact,create_partner_contact"
}
```

Do not overbuild `agent_tools_json`; dedicated exporters come later.

## Tests

Add tests that confirm:

1. Agent endpoints lower into IR.
2. Lowering is deterministic across repeated calls.
3. Canonical hash is stable for same semantic input.
4. Endpoint order is sorted by ID.
5. Nested arrays are sorted.
6. Provider requirements are preserved.
7. Existing packages without endpoints lower with `agent_endpoints: []`.

## Acceptance criteria

- `cargo test -p greentic-sorla-ir` passes.
- Pack golden inspect output is updated because `model.cbor`/`inspect_ir` shape and canonical hash change when `CanonicalIr` gains a field.
- Existing IR tests continue to pass.
- `inspect_ir` includes `agent_endpoints` in JSON output.
- No exporter files are introduced yet.
