# SoRLa gtpack

`greentic-sorla` produces deterministic `.gtpack` archives containing SoRLa
authoring and handoff artifacts for downstream `greentic-sorx` consumption.

The normal installed workflow is to let the wizard write `sorla.yaml` and the
pack in one pass:

```bash
greentic-sorla wizard --answers landlord-tenant-pack.json \
  --pack-out landlord-tenant-sor.gtpack
```

The wizard uses the generated package name and version as the pack identity. It
does not depend on repository test fixtures.

This repository includes a starter answer document at
`crates/greentic-sorla-cli/examples/answers/landlord_tenant_pack.json`. When
installed, use the same schema shape in your own answers file and set
`output_dir` to the workspace you want the wizard to write.

If you already have a SoRLa YAML file, package that file directly:

```bash
greentic-sorla pack ./sorla.yaml \
  --name my-sor \
  --version 0.1.0 \
  --out my-sor.gtpack
```

Validate and inspect the pack with:

```bash
greentic-sorla pack doctor landlord-tenant-sor.gtpack
greentic-sorla pack inspect landlord-tenant-sor.gtpack
```

## Contract Boundary

The `.gtpack` is the runtime handoff contract. Loose files under `dist/` or
`.greentic-sorla/generated/` are useful authoring artifacts, but they are not the
runtime contract.

`greentic-sorla` owns parsing, validation, canonical IR, wizard authoring,
deterministic handoff artifacts, and pack assembly. It does not own HTTP
runtime, MCP runtime, OAuth setup, provider credentials, database execution,
runtime policy enforcement, or final `.gtbundle` assembly.

## Pack Contents

Every generated pack includes:

- `pack.cbor`
- `pack.lock.cbor`
- `manifest.cbor`
- `manifest.json`
- `assets/sorla/model.cbor`
- `assets/sorla/package-manifest.cbor`
- `assets/sorla/executable-contract.json`
- `assets/sorla/agent-gateway.json`
- `assets/sorx/start.schema.json`
- `assets/sorx/start.questions.cbor`
- `assets/sorx/runtime.template.yaml`
- `assets/sorx/provider-bindings.template.yaml`

The pack also carries the existing deterministic SoRLa CBOR artifacts under
`assets/sorla/`, including actions, events, projections, policies, approvals,
views, external sources, compatibility, and provider contract.

Agent endpoint exports are included when endpoint visibility enables them:

- `assets/sorla/agent-endpoints.ir.cbor`
- `assets/sorla/agent-endpoints.openapi.overlay.yaml`
- `assets/sorla/agent-workflows.arazzo.yaml`
- `assets/sorla/mcp-tools.json`
- `assets/sorla/llms.txt.fragment`

The landlord/tenant pack example enables the full set.

## Sorx Extension

`pack.cbor` declares the Sorx-compatible extension
`greentic.sorx.runtime.v1`. The extension references SoRLa assets and Sorx
startup assets by relative paths inside the pack.

Startup assets ask for runtime/environment-specific values only, such as tenant
ID, bind addresses, provider kind/config reference, approval policy, and audit
sink. Generated templates use config references and do not embed secrets.

## Determinism

Pack output is designed to be byte-stable for the same input and options:

- zip entries are sorted
- zip timestamps are normalized
- JSON/YAML output is stable
- CBOR payloads are emitted through the existing canonical artifact path
- `pack.lock.cbor` records deterministic size and SHA-256 metadata
- absolute machine paths, generated timestamps, and environment secrets are not
  embedded

## greentic-pack Reuse

This implementation assumes `greentic-pack` remains the external pack/backend
contract owner. `greentic-sorla` emits the SoRLa-specific pack shape and keeps
`pack.cbor` / `pack.lock.cbor` deterministic without linking the full
`greentic-pack` builder dependency into this workspace. If a small stable
`greentic-pack` manifest/lock library becomes available, the local manifest and
lock writer should be replaced with that API.

## Doctor

`greentic-sorla pack doctor` checks that required entries exist, manifest
references are present, `model.cbor` parses, endpoint handoff metadata references
known endpoints, MCP tools reference known endpoints, startup schema includes the
required runtime answer paths, lock metadata matches archive contents, and common
secret markers are absent.

`greentic-sorx` can later consume this pack as an application/runtime input and
perform provider resolution, server startup, MCP exposure, policy enforcement,
and bundle assembly outside this repository.
