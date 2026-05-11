# SoRLa gtpack

`greentic-sorla` produces deterministic `.gtpack` archives containing SoRLa
authoring and handoff artifacts for downstream `greentic-sorx` consumption.

The normal installed workflow is to let the wizard write `sorla.yaml` and the
pack in one pass:

```bash
greentic-sorla wizard --answers examples/landlord-tenant/answers.json \
  --pack-out landlord-tenant-sor.gtpack
```

The wizard uses the generated package name and version as the pack identity. It
does not depend on repository test fixtures.

This repository includes a starter answer document at
`examples/landlord-tenant/answers.json`. It is a schema `0.5` rich answer
document with concrete landlord/tenant records, events, projections, actions,
policies, approvals, and agent endpoint contracts. From the repository root,
the command above writes `examples/landlord-tenant/landlord-tenant-sor.gtpack`.
When installed, use the same schema shape in your own answers file and set
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
greentic-sorla pack doctor examples/landlord-tenant/landlord-tenant-sor.gtpack
greentic-sorla pack inspect examples/landlord-tenant/landlord-tenant-sor.gtpack
greentic-sorla pack validation-inspect examples/landlord-tenant/landlord-tenant-sor.gtpack
greentic-sorla pack schema validation
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
- `assets/sorx/compatibility.json`
- `assets/sorx/exposure-policy.json`

Generated packs also include embedded SORX validation metadata:

- `assets/sorx/tests/test-manifest.json`
- optional referenced files under `assets/sorx/tests/`

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

## Embedded SORX Validation

Generated packs carry a deterministic exposure policy at
`assets/sorx/exposure-policy.json` using the
`greentic.sorx.exposure-policy.v1` schema.

The exposure policy keeps pack-level default visibility private, marks exported
agent endpoints as downstream public candidates, records export surfaces, and
requires approval for high-risk, approval-gated, or side-effectful endpoints. It
does not define concrete runtime routes. Route prefix lists are empty until
route metadata exists in SoRLa source.

Generated packs also carry compatibility metadata at
`assets/sorx/compatibility.json` using the `greentic.sorx.compatibility.v1`
schema. The compatibility manifest copies abstract provider
category/capability requirements and current SoRLa migration compatibility IR.
If no migration metadata exists, it defaults to isolated state for downstream
concurrent deployments.

Validation-enabled packs carry a deterministic validation manifest at
`assets/sorx/tests/test-manifest.json` using the
`greentic.sorx.validation.v1` schema.

The validation manifest describes suites that downstream SORX tooling can
execute before promoting a deployed pack version. The metadata may cover static
handoff checks, agent endpoint contracts, provider capability requirements,
security policy checks, tenant isolation checks, and migration compatibility
checks.

`greentic-sorla` owns deterministic metadata generation, pack inclusion, and
static doctor/inspect checks for validation and exposure metadata. It does not
execute validation tests, contact providers, start runtimes, expose public
routes, or promote deployments.

SORX must not expose public endpoints from a validation-enabled pack unless the
required validation suites pass or an explicit local operator policy overrides
that gate.

See `docs/sorx-gtpack-validation.md` for the validation manifest contract and
`docs/sorx-deployment-handoff.md` for the downstream SORX deployment lifecycle
that consumes validation-enabled packs.

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

For GHCR publish events, concurrent version deployment, public exposure gates,
certification reports, and alias promotion expectations, see
`docs/sorx-deployment-handoff.md`.
