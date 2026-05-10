# PR-11 — Deterministic SoRLa gtpack Packaging

## Goal

Add deterministic `.gtpack` packaging for SoRLa authoring and handoff artifacts.

The first real `.gtpack` fixture must be the landlord/tenant system of record,
so downstream `greentic-sorx` work can start from the same realistic scenario
used by the PR-09 e2e and PR-10 executable-contract work.

This PR does not add a runtime server. It packages the contract that a runtime
or bundle assembler can consume later.

## Current-Code Review

The original proposal is directionally right, but the implementation must be
adjusted to the current repo:

- `greentic-sorla-cli` currently exposes a wizard-first public UX:
  `greentic-sorla wizard --schema` and `greentic-sorla wizard --answers <file>`.
  Adding `greentic-sorla pack ...` is a deliberate public CLI expansion and
  must update `docs/product-shape.md`, CLI help, and README.
- `greentic-sorla-pack` already builds deterministic SoRLa artifact sets from
  YAML with canonical CBOR, canonical hashes, agent gateway metadata, OpenAPI
  overlay YAML, Arazzo YAML, MCP tool descriptors, `llms.txt` fragments, and
  `executable-contract.json`.
- `greentic-sorla-pack` does not currently write `.gtpack` files.
- This repo currently has no `greentic-pack` dependency and no local
  `pack.cbor` / `pack.lock.cbor` writer.
- Existing SoRLa package metadata is a local `PackageManifest` with
  `package_kind: "sorla-package"`. Do not treat it as the final
  `greentic-pack` manifest unless the `greentic-pack` API confirms that shape.
- `agent-endpoints.ir.cbor` currently stores canonical IR bytes when agent
  endpoints exist. `model.cbor` already stores canonical IR bytes too. Keep that
  behavior unless PR-11 intentionally narrows `agent-endpoints.ir.cbor` and
  updates docs/tests.
- PR-10 added `executable-contract.json`; include it in the pack as a SoRLa
  handoff artifact even though it was not in the original draft.

## Ownership Boundary

`greentic-sorla` owns:

- SoRLa language parsing
- validation
- canonical IR
- wizard flow
- deterministic handoff artifacts
- `.gtpack` packaging by reusing `greentic-pack`

`greentic-sorla` must not own:

- HTTP runtime
- MCP runtime
- provider credential resolution
- OAuth setup
- database/provider execution
- runtime policy enforcement
- final `.gtbundle` assembly

## Reuse greentic-pack First

Before writing any `.gtpack` logic, audit `greentic-pack` and document the
result in `docs/sorla-gtpack.md`:

1. Check whether a Rust library API exists for creating pack manifests, lock
   files, digest metadata, and archives.
2. If a library API exists, add the smallest publish-safe dependency needed and
   call it from `greentic-sorla-pack`.
3. If only a CLI exists, create a deterministic staging directory and invoke the
   CLI in a narrow wrapper. Keep all SoRLa asset generation in
   `greentic-sorla-pack`.
4. If neither a reusable crate nor CLI is available, stop and record the gap
   rather than hand-rolling the `greentic-pack` canonical manifest and lock
   format.

Do not duplicate `pack.cbor`, `pack.lock.cbor`, canonicalization, archive
ordering, or digest calculation logic unless the audit proves there is no
reusable interface.

## CLI

Add a public command shaped like this, adapting only where the actual
`greentic-pack` API requires it:

```bash
greentic-sorla wizard --answers landlord-tenant-pack.json \
  --pack-out landlord-tenant-sor.gtpack
```

Also support direct packaging for users who already have a generated or
hand-authored SoRLa YAML file:

```bash
greentic-sorla pack ./sorla.yaml --name landlord-tenant-sor --version 0.1.0 \
  --out landlord-tenant-sor.gtpack
```

Add inspection commands:

```bash
greentic-sorla pack doctor landlord-tenant-sor.gtpack
greentic-sorla pack inspect landlord-tenant-sor.gtpack
```

The CLI should fail clearly for missing input, invalid YAML, parse errors,
invalid output paths, unsupported manifest formats, or a missing
`greentic-pack` backend.

## Pack Contents

The landlord/tenant `.gtpack` should include these stable paths:

```text
pack.cbor
pack.lock.cbor

assets/sorla/model.cbor
assets/sorla/package-manifest.cbor
assets/sorla/executable-contract.json
assets/sorla/agent-gateway.json
assets/sorla/agent-endpoints.ir.cbor
assets/sorla/agent-endpoints.openapi.overlay.yaml
assets/sorla/agent-workflows.arazzo.yaml
assets/sorla/mcp-tools.json
assets/sorla/llms.txt.fragment

assets/sorx/start.schema.json
assets/sorx/start.questions.cbor
assets/sorx/runtime.template.yaml
assets/sorx/provider-bindings.template.yaml
```

Also include the existing deterministic CBOR handoff artifacts if the manifest
references them:

```text
assets/sorla/actions.cbor
assets/sorla/events.cbor
assets/sorla/projections.cbor
assets/sorla/policies.cbor
assets/sorla/approvals.cbor
assets/sorla/views.cbor
assets/sorla/external-sources.cbor
assets/sorla/compatibility.cbor
assets/sorla/provider-contract.cbor
```

Agent endpoint export artifacts remain optional for packages where endpoint
visibility disables them. The landlord/tenant fixture should enable OpenAPI,
Arazzo, MCP, and `llms.txt`, so the first `.gtpack` exercises the full handoff.

## Manifest Extension

Declare the pack as a SoRLa-authored Sorx runtime input using the actual
`greentic-pack` manifest extension/custom metadata format.

The extension payload should preserve this information:

```json
{
  "extension": "greentic.sorx.runtime.v1",
  "sorla": {
    "model": "assets/sorla/model.cbor",
    "package_manifest": "assets/sorla/package-manifest.cbor",
    "executable_contract": "assets/sorla/executable-contract.json",
    "agent_gateway": "assets/sorla/agent-gateway.json",
    "agent_endpoints_ir": "assets/sorla/agent-endpoints.ir.cbor",
    "openapi_overlay": "assets/sorla/agent-endpoints.openapi.overlay.yaml",
    "arazzo": "assets/sorla/agent-workflows.arazzo.yaml",
    "mcp_tools": "assets/sorla/mcp-tools.json",
    "llms_fragment": "assets/sorla/llms.txt.fragment"
  },
  "sorx": {
    "start_schema": "assets/sorx/start.schema.json",
    "start_questions": "assets/sorx/start.questions.cbor",
    "runtime_template": "assets/sorx/runtime.template.yaml",
    "provider_bindings_template": "assets/sorx/provider-bindings.template.yaml"
  }
}
```

If the real `greentic-pack` manifest uses different names, keep the same
semantic fields under the native extension mechanism and document the mapping.

## Sorx Startup Assets

Generate deterministic startup assets for future `greentic-sorx` use. They
should contain runtime/environment-specific questions only and no credentials.

`assets/sorx/start.schema.json` should require at least:

```json
{
  "schema": "greentic.sorx.start.answers.v1",
  "required": [
    "tenant.tenant_id",
    "server.bind",
    "server.public_base_url",
    "providers.store.kind",
    "providers.store.config_ref",
    "policy.approvals.high",
    "audit.sink"
  ]
}
```

The generated example/default structure should support:

```json
{
  "tenant": {
    "tenant_id": "demo-landlord",
    "environment": "local"
  },
  "server": {
    "bind": "127.0.0.1:8787",
    "public_base_url": "http://127.0.0.1:8787"
  },
  "mcp": {
    "enabled": true,
    "bind": "127.0.0.1:8790"
  },
  "providers": {
    "store": {
      "kind": "foundationdb",
      "config_ref": "providers.foundationdb.local"
    }
  },
  "policy": {
    "approvals": {
      "low": "auto",
      "medium": "auto",
      "high": "require_approval",
      "critical": "deny"
    }
  },
  "audit": {
    "sink": "stdout"
  }
}
```

`provider-bindings.template.yaml` may contain provider kinds and config
reference names, but not secrets, tokens, OAuth clients, cluster credentials, or
absolute machine paths.

## Determinism Requirements

The same input and options must produce byte-stable output when the reused
`greentic-pack` backend supports byte stability. If the backend embeds unavoidable
archive metadata, tests must prove digest stability through the official lock or
pack digest.

Ensure:

- sorted asset entries
- stable or normalized timestamps
- canonical CBOR for SoRLa model, manifests, questions, and lock metadata
- stable JSON/YAML formatting
- stable pack digest calculation through `greentic-pack`
- no machine-specific absolute paths
- no environment secrets
- no generated UUIDs or current time values

## Doctor And Inspect

`greentic-sorla pack doctor` should reuse `greentic-pack` validation first, then
perform SoRLa/Sorx extension checks:

- `.gtpack` exists
- `pack.cbor` exists
- `pack.lock.cbor` exists
- required SoRLa assets exist
- required Sorx startup assets exist
- manifest references resolve
- `model.cbor` parses as canonical SoRLa IR
- `agent-gateway.json` parses
- endpoint metadata references valid model operations or agent endpoints
- `mcp-tools.json`, if present, references valid endpoint IDs
- startup schema parses and includes required paths
- no obvious secrets are embedded

`greentic-sorla pack inspect` should print deterministic JSON containing at
least pack name, version, extension IDs, asset paths, lock digest, SoRLa package
name/version, IR hash, and optional-artifact presence.

## Landlord/Tenant First Pack

Use wizard-generated landlord/tenant answers as the first installed-user
`.gtpack` scenario.

Preferred input:

- `crates/greentic-sorla-cli/examples/answers/landlord_tenant_pack.json`
- `greentic-sorla wizard --answers <answers> --pack-out <pack>`

Internal e2e tests may continue to use `tests/e2e/fixtures/landlord_sor_v1.yaml`,
but public docs must not require paths that are unavailable after installation.

If PR-10 has already updated landlord/tenant fixtures with relationships,
migration backfills, and agent operation plans, use that richer fixture. If not,
extend `landlord_sor_v1.yaml` enough for packaging tests to include:

- `Landlord`
- `Property`
- `Unit`
- `Tenant`
- `Tenancy`
- `Payment`
- `MaintenanceRequest`
- agent endpoints with OpenAPI, Arazzo, MCP, and `llms.txt` visibility enabled
- provider requirements for a FoundationDB-style store as abstract requirements,
  not concrete credentials

The PR may also add a focused golden fixture under
`crates/greentic-sorla-pack/tests/golden/landlord_tenant_sorla.sorla.yaml` if
the e2e fixture is too broad, but the generated `.gtpack` smoke should still use
the landlord/tenant scenario as the first-class example.

## Files To Touch

- `Cargo.toml`
- `crates/greentic-sorla-pack/Cargo.toml`
- `crates/greentic-sorla-pack/src/lib.rs`
- `crates/greentic-sorla-pack/tests/golden/`
- `crates/greentic-sorla-cli/Cargo.toml`
- `crates/greentic-sorla-cli/src/lib.rs`
- `crates/greentic-sorla-cli/examples/answers/landlord_tenant_pack.json`
- `tests/e2e/fixtures/landlord_sor_v1.yaml` for internal e2e coverage only
- `docs/sorla-gtpack.md`
- `docs/packaging.md`
- `docs/artifacts.md`
- `docs/product-shape.md`
- `README.md`

Do not add provider path dependencies to publishable crates. If a non-publishable
test harness needs provider fixtures, keep that work in `crates/greentic-sorla-e2e`
or `xtask`.

## Tests

Add focused tests for:

1. Pack command creates a `.gtpack`.
2. Landlord/tenant is the first documented `.gtpack` example.
3. Required SoRLa assets are present under `assets/sorla/`.
4. Required Sorx startup assets are present under `assets/sorx/`.
5. Manifest extension references resolve.
6. `pack.lock.cbor` is generated by the reused `greentic-pack` path.
7. Output is byte-stable or official-digest-stable across two runs.
8. Missing input fails clearly.
9. Invalid SoRLa input fails clearly.
10. Startup schema parses and includes required runtime-only fields.
11. Generated pack does not include common secret markers.
12. Optional endpoint artifacts are omitted when endpoint visibility disables
    them and manifest metadata reflects that.
13. Doctor accepts the landlord/tenant pack.
14. Doctor rejects malformed packs with missing assets, invalid JSON, invalid
    CBOR, or unresolved manifest references.

Run at least:

```bash
cargo test --all-features
```

If `ci/local_check.sh` remains the repo's canonical local validation, run it as
the final verification.

## Documentation

Update docs to explain:

- what a SoRLa `.gtpack` is
- why loose `./dist` artifacts are not the runtime contract
- how to create the landlord/tenant pack
- how to doctor and inspect a pack
- what assets are included
- which artifacts are optional
- which pack/lock behavior is delegated to `greentic-pack`
- what remains outside `greentic-sorla`
- how `greentic-sorx` will later consume the pack

## Acceptance Criteria

- `greentic-sorla` can produce a deterministic landlord/tenant `.gtpack`.
- Packaging reuses `greentic-pack` wherever a reusable interface exists.
- The pack includes SoRLa handoff artifacts under `assets/sorla/`.
- The pack includes Sorx startup schema/templates under `assets/sorx/`.
- The pack declares a Sorx-compatible runtime extension.
- Pack doctor and inspect functionality exists.
- Tests prove byte-stable or digest-stable output.
- No runtime server logic is added to `greentic-sorla`.
- No credentials or secrets are embedded in the pack.
- Public CLI/docs are updated to reflect that `pack` is now supported alongside
  the wizard flow.
