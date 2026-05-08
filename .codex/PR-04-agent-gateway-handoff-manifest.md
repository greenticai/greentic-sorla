# PR-04 — Add Agent Gateway Handoff Manifest

## Goal

Create a deterministic handoff manifest that describes agentic endpoints for downstream `gtc` assembly.

This PR must not implement runtime API gateway behavior. It should produce metadata that `gtc` can consume later.

## Files to touch

- `crates/greentic-sorla-pack/src/lib.rs`
- `crates/greentic-sorla-ir/src/lib.rs` only if helper methods are needed
- New module if desired: `crates/greentic-sorla-pack/src/agent_gateway.rs`
- Tests in `crates/greentic-sorla-pack`

## Current-code notes

- `greentic-sorla-pack` already exposes `ArtifactSet`, `PackageManifest`, `scaffold_manifest()`, and `build_artifacts_from_yaml()`.
- The existing generated artifact set uses `cbor_artifacts: BTreeMap<String, Vec<u8>>`, `inspect_json`, `agent_tools_json`, and `canonical_hash`.
- Existing package manifest artifact references include `model.cbor` and `agent-tools.json`. Add the new agent handoff filenames to this manifest only after PR-03 adds `ir.agent_endpoints`.
- IR version is currently `IrVersion { major, minor }`, not a raw string. When serializing handoff package refs, derive `ir_version` from `ir.ir_version`, for example `format!("{}.{}", major, minor)`.

## Manifest schema

Add a new serializable handoff document:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayHandoffManifest {
    pub schema: String,
    pub package: AgentGatewayPackageRef,
    pub endpoints: Vec<AgentGatewayEndpointRef>,
    pub provider_contract: AgentGatewayProviderContract,
    pub exports: AgentGatewayExports,
    pub notes: Vec<String>,
}
```

```rust
pub const AGENT_GATEWAY_HANDOFF_SCHEMA: &str = "greentic.agent-gateway.handoff.v1";
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayPackageRef {
    pub name: String,
    pub version: String,
    pub ir_version: String,
    pub ir_hash: String,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayEndpointRef {
    pub id: String,
    pub title: String,
    pub intent: String,
    pub risk: String,
    pub approval: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub side_effects: Vec<String>,
    pub exports: AgentGatewayEndpointExports,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayEndpointExports {
    pub openapi: bool,
    pub arazzo: bool,
    pub mcp: bool,
    pub llms_txt: bool,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayProviderContract {
    pub categories: Vec<AgentGatewayProviderRequirement>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayProviderRequirement {
    pub category: String,
    pub capabilities: Vec<String>,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayExports {
    pub agent_gateway_json: bool,
    pub openapi_overlay: bool,
    pub arazzo: bool,
    pub mcp_tools: bool,
    pub llms_txt: bool,
}
```

## Builder function

Add:

```rust
pub fn agent_gateway_handoff_manifest(ir: &CanonicalIr) -> AgentGatewayHandoffManifest
```

Rules:

- Include all `ir.agent_endpoints`.
- Aggregate provider requirements from both package-level `provider_contract` and endpoint-level provider requirements.
- Sort provider categories and capabilities deterministically.
- Compute `ir_hash` via existing canonical hash function.
- Set export booleans to true if any endpoint enables that visibility.
- Add a note that this is handoff metadata and not final runtime assembly.

## Output file name convention

Document or expose constants:

```rust
pub const AGENT_GATEWAY_HANDOFF_FILENAME: &str = "agent-gateway.json";
pub const AGENT_ENDPOINTS_IR_CBOR_FILENAME: &str = "agent-endpoints.ir.cbor";
```

Also add these filenames to `scaffold_manifest().artifact_references` once the artifacts are generated or intentionally selectable.

## Tests

Add tests that confirm:

1. Empty package generates manifest with empty endpoints and exports false, except `agent_gateway_json` true if manifest itself is generated.
2. Endpoint visibility controls aggregate export booleans.
3. Provider requirements aggregate and dedupe deterministically.
4. Manifest includes stable schema string.
5. Manifest includes current IR hash.

## Acceptance criteria

- `cargo test -p greentic-sorla-pack` passes.
- No `gtc` dependency is introduced.
- Manifest clearly says handoff metadata, not runtime assembly.
