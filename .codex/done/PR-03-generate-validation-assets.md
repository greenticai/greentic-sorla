# PR-03 — Generate default SORX validation assets from SoRLa IR and wizard answers

## Repository

greenticai/greentic-sorla

## Objective

Generate a deterministic default SORX validation manifest for each `.gtpack` using available SoRLa package metadata, agent endpoints, provider requirements, and startup/runtime metadata.

The result should be generated from the same artifact/IR path used by pack assembly. Pack inclusion is handled in a later PR.

Current implementation alignment:

- `wizard --pack-out` writes `sorla.yaml` and helper files under the selected output directory, then builds the `.gtpack` directly from the generated `sorla.yaml`.
- `build_sorla_gtpack` currently constructs pack entries in memory from `ArtifactSet`; there is no required loose `assets/sorx/...` staging tree.
- If this PR writes a loose copy for developer inspection, put it under `.greentic-sorla/generated/assets/sorx/tests/test-manifest.json` as an authoring artifact, but keep pack generation able to produce the manifest directly from IR.

## Required changes

### 1. Add validation generation module

Create a generator module near pack generation code, for example:

```text
crates/greentic-sorla-pack/src/validation_generator.rs
```

The generator should expose:

```rust
pub fn generate_sorx_validation_manifest(input: SorxValidationGenerationInput) -> SorxValidationManifest
```

### 2. Define generation input

The input should be assembled from existing package/IR structures and include:

```rust
pub struct SorxValidationGenerationInput<'a> {
    pub package_name: &'a str,
    pub package_version: &'a str,
    pub ir_version: Option<&'a str>,
    pub ir_hash: Option<&'a str>,
    pub agent_endpoints: &'a [AgentEndpointIr],
    pub provider_requirements: Vec<ProviderRequirementIr>,
    pub sorx_startup_asset_names: Vec<String>,
}
```

Use existing IR types where possible instead of inventing duplicate provider requirement structures. Provider requirements are currently available from both `ir.provider_contract.categories` and endpoint-level `provider_requirements`; generation should aggregate and sort them the same way `agent_gateway_handoff_manifest` does.

### 3. Generate default suites

Generate these suites by default:

#### `smoke`

Required. Include tests:

- `runtime-startup-assets-present`
- `start-schema-present`
- `provider-bindings-template-present`

These are static handoff checks, not runtime HTTP checks.

#### `contract`

Required when exported agent endpoints exist.

For each endpoint with at least one export surface enabled (`openapi`, `arazzo`, `mcp`, or `llms_txt`), generate an `agent-endpoint` contract placeholder test:

```json
{
  "id": "agent-endpoint-create-customer-contact-contract",
  "kind": "agent-endpoint",
  "endpoint": "create_customer_contact",
  "required": true
}
```

#### `provider`

Required when provider requirements exist.

For each provider category/capability group, generate a provider capability test.

#### `security`

Required if any endpoint is exported, high risk, approval-gated, or side-effectful. Current SoRLa endpoint metadata has no route or public-alias concept; treat exported endpoints as downstream exposure candidates, not already-public endpoints.

Generate at minimum:

- `no-secrets-in-pack`
- `public-exposure-requires-validation`
- `high-risk-endpoints-require-approval` if risk metadata exists

### 4. Deterministic ordering

Ensure:

- suites sorted by stable order: `smoke`, `contract`, `provider`, `security`, `migration`
- tests sorted by ID within suite
- provider requirements sorted lexically
- endpoint IDs sorted lexically

### 5. Emit file to generated assets

When pack generation assembles SORX assets, create:

```text
assets/sorx/tests/test-manifest.json
```

Use stable pretty JSON or canonical JSON consistent with current generated outputs.

Do not write timestamps or environment-specific paths.

### 6. Add answer-model option

Add a wizard answer field if appropriate:

```json
{
  "sorx_validation": {
    "enabled": true,
    "default_visibility": "private",
    "promotion_requires": ["smoke", "contract", "security", "provider"]
  }
}
```

Defaults:

- `enabled: true` for pack output
- `default_visibility: private`
- required promotion suites inferred from generated content

If adding this field risks schema instability, keep it internal for now and always generate validation assets when `--pack-out` is used.

Current recommendation: keep this internal until the pack asset is implemented. The existing wizard schema is versioned as `0.4`; avoid a schema bump unless users need to opt out.

### 7. Tests

Add tests verifying:

- no endpoints generates smoke only, plus provider checks if package-level provider requirements exist
- exported endpoints generate contract tests
- provider requirements generate provider tests
- output ordering is stable
- generated manifest passes `validate_static()`

## Acceptance criteria

- Running wizard/pack generation produces `assets/sorx/tests/test-manifest.json` in the intermediate output tree.
- Manifest content is deterministic.
- Generated manifest validates statically.

## Non-goals

- Do not yet include assets in the final `.gtpack` unless this is trivial and already part of pack assembly.
- Do not execute runtime tests.
