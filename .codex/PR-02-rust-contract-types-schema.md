# PR-02 — Add Rust contract types and JSON schema for `greentic.sorx.validation.v1`

## Repository

greenticai/greentic-sorla

## Objective

Add typed Rust models for the SORX validation manifest and expose a deterministic JSON schema for the validation contract.

This prepares pack generation and doctor validation without yet changing `.gtpack` output.

## Required changes

### 1. Add validation contract module

Add a module in the most appropriate existing crate. The current implementation keeps pack-specific handoff and `.gtpack` manifest code in `crates/greentic-sorla-pack/src/lib.rs`, while canonical IR data types live in `crates/greentic-sorla-ir/src/lib.rs`.

Prefer:

```text
crates/greentic-sorla-pack/src/sorx_validation.rs
```

Only move the contract to IR if it must be consumed independent of pack generation:

```text
crates/greentic-sorla-ir/src/sorx_validation.rs
```

Implementation note: adding a sibling module will require splitting/exporting from the currently monolithic `greentic-sorla-pack/src/lib.rs`; keep that refactor minimal and scoped.

### 2. Define top-level manifest type

Create types similar to:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SorxValidationManifest {
    pub schema: String,
    pub suite_version: String,
    pub package: SorxValidationPackageRef,
    pub default_visibility: EndpointVisibility,
    pub promotion_requires: Vec<String>,
    pub suites: Vec<SorxValidationSuite>,
}
```

Use constant:

```rust
pub const SORX_VALIDATION_SCHEMA: &str = "greentic.sorx.validation.v1";
```

### 3. Define package ref

```rust
pub struct SorxValidationPackageRef {
    pub name: String,
    pub version: String,
    pub ir_version: Option<String>,
    pub ir_hash: Option<String>,
}
```

### 4. Define visibility enum

```rust
pub enum EndpointVisibility {
    Private,
    Internal,
    PublicCandidate,
}
```

Serialize as kebab-case or snake_case consistently with existing repo JSON conventions. Prefer snake_case if existing generated JSON uses it.

### 5. Define suite and test models

```rust
pub struct SorxValidationSuite {
    pub id: String,
    pub title: Option<String>,
    pub required: bool,
    pub tests: Vec<SorxValidationTest>,
}
```

Use an internally-tagged enum for test kinds:

```rust
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SorxValidationTest {
    Healthcheck { ... },
    AgentEndpoint { ... },
    OpenApiContract { ... },
    McpToolContract { ... },
    ArazzoWorkflow { ... },
    ProviderCapability { ... },
    AuthRequired { ... },
    PolicyEnforced { ... },
    TenantIsolation { ... },
    MigrationCompatibility { ... },
    RollbackCompatibility { ... },
}
```

Keep the first implementation fields simple and additive:

Common fields:

```rust
id: String,
title: Option<String>,
endpoint: Option<String>,
required: Option<bool>,
timeout_ms: Option<u64>,
input_ref: Option<String>,
expect: Option<serde_json::Value>,
```

For provider tests:

```rust
provider_category: String,
capabilities: Vec<String>,
```

### 6. Add schema emission helper

Add a function:

```rust
pub fn sorx_validation_schema_json() -> serde_json::Value
```

`schemars` is not currently used in the workspace. If adding it is still the smallest stable path, add it to workspace dependencies and the owning crate. Otherwise, emit a deterministic hand-authored JSON schema from typed constants and tests. Do not introduce a runtime JSON-schema validator unless it is needed by static manifest checks.

### 7. Add validation helper

Add:

```rust
impl SorxValidationManifest {
    pub fn validate_static(&self) -> Result<(), SorxValidationError>
}
```

Static validation should check:

- schema equals `greentic.sorx.validation.v1`
- package name is non-empty
- package version is non-empty
- suite IDs are unique
- test IDs are unique within manifest or globally, choose one and document it
- required promotion suites exist
- relative references do not contain `..`
- refs do not start with `/`

Current model alignment:

- Use snake_case for generated JSON fields; existing generated JSON uses snake_case for schema-like documents.
- `EndpointVisibility` is new validation/exposure metadata. Do not confuse it with current `AgentEndpointVisibilityIr`, which only says whether an endpoint is exported to OpenAPI, Arazzo, MCP, or `llms.txt`.
- Existing risk values are `low`, `medium`, and `high`; existing approval values are `none`, `optional`, `required`, and `policy-driven`.

### 8. Unit tests

Add unit tests for:

- valid minimal manifest
- bad schema rejected
- duplicate suite ID rejected
- duplicate test ID rejected
- invalid `../` ref rejected
- missing promotion suite rejected
- JSON schema can be generated deterministically

## Acceptance criteria

- New validation contract types compile.
- JSON schema generation works.
- Static validation catches invalid manifest structure.
- No `.gtpack` output changes yet.

## Non-goals

- Do not execute tests.
- Do not call HTTP endpoints.
- Do not implement provider checks.
