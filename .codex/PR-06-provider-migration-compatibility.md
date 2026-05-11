# PR-06 — Add provider and migration compatibility validation metadata

## Repository

greenticai/greentic-sorla

## Objective

Add pack metadata that helps SORX safely validate provider compatibility and concurrent version deployment before exposing a new pack version.

This PR is especially important for running multiple versions of the same SORLA `.gtpack` concurrently.

Current implementation alignment:

- SoRLa already has canonical `compatibility` IR entries with `compatibility`, `projection_updates`, `backfills`, optional `idempotence_key`, and optional notes.
- SoRLa does not currently model `from_version_range`, `to_version`, `rollback_safe`, `shared_state_allowed`, or concrete migration execution.
- Provider requirements are abstract categories/capabilities from package-level and endpoint-level metadata; credentials and concrete provider bindings belong downstream.
- This PR must emit compatibility metadata for SORX to consume; it must not implement concurrent deployment, migration execution, rollback, provider connectivity, or state sharing in this repo.

## Required changes

### 1. Define compatibility asset

Add:

```text
assets/sorx/compatibility.json
```

Schema:

```text
greentic.sorx.compatibility.v1
```

### 2. Rust types

Create types similar to:

```rust
pub struct SorxCompatibilityManifest {
    pub schema: String,
    pub package: SorxValidationPackageRef,
    pub api_compatibility: ApiCompatibility,
    pub state_compatibility: StateCompatibility,
    pub provider_compatibility: Vec<ProviderCompatibilityRequirement>,
    pub migration_compatibility: Vec<MigrationCompatibilityRule>,
}
```

Suggested enums:

```rust
pub enum ApiCompatibilityMode {
    Additive,
    BackwardCompatible,
    Breaking,
    Unknown,
}

pub enum StateCompatibilityMode {
    IsolatedRequired,
    SharedAllowed,
    SharedRequiresMigration,
    Unknown,
}
```

### 3. Provider compatibility requirements

Represent abstract provider requirements emitted by SORLA:

```json
{
  "category": "crm",
  "required_capabilities": ["contacts.read", "contacts.write"],
  "contract_version_range": ">=0.1.0 <1.0.0",
  "required": true
}
```

Do not embed credentials or provider-specific secrets.

### 4. Migration compatibility rules

Support minimal metadata derived from current SoRLa migration/compatibility IR:

```json
{
  "name": "landlord-tenant-compatibility",
  "mode": "additive",
  "projection_updates": ["tenant_summary"],
  "backfills": [],
  "idempotence_key": "landlord-tenant-0.2.0"
}
```

Do not require fields the current language cannot express. The following richer shape may be documented as future SORX-facing metadata, but should not be mandatory in this PR unless the language/IR is extended first:

```json
{
  "from_version_range": ">=0.1.0 <0.2.0",
  "to_version": "0.2.0",
  "strategy": "additive",
  "rollback_safe": true,
  "shared_state_allowed": true,
  "requires_operator_approval": false
}
```

If no explicit migration metadata exists, default to conservative:

```text
state_compatibility = isolated_required
api_compatibility = unknown
```

### 5. Generate validation tests from compatibility

Extend validation manifest generation to include:

- provider capability tests for every required provider compatibility entry
- migration compatibility tests when current SoRLa compatibility entries exist
- rollback compatibility tests only if an explicit rollback-safety field is added by an earlier change

### 6. Include in `.gtpack`

Add compatibility asset and extension reference:

```text
greentic.sorx.compatibility.v1 -> assets/sorx/compatibility.json
```

Use the same additive `extension.sorx` manifest strategy as PR-04 unless a backwards-compatible extension registry has already landed. In the current manifest shape, `greentic.sorx.compatibility.v1` is the asset schema, not a replacement for the runtime extension id.

### 7. Doctor checks

Doctor should verify:

- compatibility manifest schema is valid
- package name/version matches pack
- provider capability entries are deterministic and non-empty
- version ranges are parseable if a semver range crate is already available; otherwise validate non-empty strings and leave strict parsing for later
- shared state is not claimed unless explicit metadata exists to support it
- rollback-safe claims include rollback compatibility tests, if rollback-safe claims are introduced

### 8. Tests

Add tests for:

- default conservative compatibility manifest
- provider requirements copied into compatibility manifest
- current SoRLa compatibility metadata generates migration compatibility tests
- shared state claims are absent by default
- rollback_safe generates rollback test requirement only if rollback_safe metadata exists

## Acceptance criteria

- SORX receives enough metadata to decide isolated vs shared state for concurrent versions.
- Provider requirements are explicit and testable.
- Doctor rejects unsafe compatibility claims.

## Non-goals

- Do not run migrations.
- Do not connect to providers.
- Do not implement semver if not already available unless small and justified.
