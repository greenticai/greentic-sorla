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

The CLI delegates pack generation, doctor, and inspect behavior through the
reusable SoRLa library/pack crates. Designer extensions and tests should use
those library APIs directly when they need deterministic artifact output without
spawning the binary.

The Sorla Designer extension exposes a `generate_gtpack` tool. In WASM builds it
returns deterministic pack-entry metadata and a diagnostic asking the host to
perform native ZIP packaging. Native tooling can still call
`greentic-sorla-lib::build_gtpack_bytes` to produce `.gtpack` bytes directly.

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
greentic-sorla pack schema ontology
greentic-sorla pack schema retrieval-bindings
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
- `assets/sorla/designer-node-types.json`
- `assets/sorla/agent-endpoint-action-catalog.json`

The landlord/tenant pack example enables the full set.

`designer-node-types.json` uses the
`greentic.sorla.designer-node-types.v1` schema. It derives node labels, input
schemas, output schemas, safety metadata, backing references, and locked
`endpoint_ref` values from canonical SoRLa agent endpoints. Pack doctor verifies
that every node type points at a known endpoint, that its contract hash matches
the canonical IR, that the component binding uses the expected operation, and
that the asset is covered by `pack.lock.cbor`.

`agent-endpoint-action-catalog.json` uses the
`greentic.sorla.agent-endpoint-action-catalog.v1` schema. It is a design-time
catalog view over canonical agent endpoints, not a second runtime action source
of truth. Pack doctor verifies the schema, package metadata, endpoint IDs,
`sha256:<64 lowercase hex chars>` contract hash format, canonical hash
consistency, required input schema coverage, and `pack.lock.cbor` coverage.

When the source package declares `ontology:
greentic.sorla.ontology.v1`, the pack also includes deterministic ontology
handoff assets:

- `assets/sorla/ontology.graph.json`
- `assets/sorla/ontology.ir.cbor`
- `assets/sorla/ontology.schema.json`

These files are source material for downstream Sorx, `gtc`, retrieval,
OpenAPI/MCP enrichment, and bundle assembly. They do not make
`greentic-sorla` responsible for graph traversal, provider binding, runtime
authorization, or final bundle ownership.

When the source package declares `retrieval_bindings:
greentic.sorla.retrieval-bindings.v1`, the pack also includes:

- `assets/sorla/retrieval-bindings.json`
- `assets/sorla/retrieval-bindings.ir.cbor`

These describe ontology-scoped evidence provider requirements and traversal
filters as handoff metadata only.

## Sorx Extension

`pack.cbor` declares the Sorx-compatible extension
`greentic.sorx.runtime.v1`. The extension references SoRLa assets and Sorx
startup assets by relative paths inside the pack.

Ontology-enabled packs declare `greentic.sorla.ontology.v1` under the Sorx
extension's SoRLa metadata and point to the ontology graph JSON, canonical
ontology IR CBOR, and JSON schema assets. Consumers should use those manifest
paths instead of guessing filenames.

Retrieval-enabled packs similarly declare `greentic.sorla.retrieval-bindings.v1`
under SoRLa extension metadata and point to the JSON and canonical CBOR
retrieval binding assets.

Packs with agent endpoints also declare
`greentic.sorla.designer-node-types.v1` under SoRLa extension metadata and point
to `assets/sorla/designer-node-types.json`. Consumers should read this manifest
path instead of guessing the filename.

The same SoRLa extension metadata declares
`greentic.sorla.agent-endpoint-action-catalog.v1` and points to
`assets/sorla/agent-endpoint-action-catalog.json` when agent endpoints are
present.

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
security policy checks, ontology metadata checks, retrieval binding checks,
tenant isolation checks, and migration compatibility checks.

When ontology assets are present, exported packs include a required `ontology`
suite in `promotion_requires`. Private-only packs may include the same ontology
suite as optional metadata for downstream tooling. Retrieval-enabled exported
packs likewise include a required `retrieval` suite, and retrieval provider
requirements are folded into the existing `provider-capability` validation
suite. The validation manifest remains metadata only: it declares checks for
downstream SORX and does not execute traversal, retrieval, provider calls, or
public exposure inside SoRLa.

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
secret markers are absent. It also validates the embedded SORX validation
manifest shape, including the ontology/retrieval test kind vocabulary and the
absence of obsolete suite-level fields such as `kind` or
`required_for_public_exposure`.

For ontology-enabled packs, doctor also checks that manifest paths are relative
archive asset paths, ontology files exist, lock metadata covers them, graph and
IR hashes match, graph concepts/relationships/constraints match the canonical
ontology IR, and referenced backing records and fields exist in `model.cbor`.

For retrieval-enabled packs, doctor checks that retrieval binding assets exist,
are covered by lock metadata, and match the canonical retrieval binding IR in
`model.cbor`.

`greentic-sorx` can later consume this pack as an application/runtime input and
perform provider resolution, server startup, MCP exposure, policy enforcement,
and bundle assembly outside this repository.

For GHCR publish events, concurrent version deployment, public exposure gates,
certification reports, and alias promotion expectations, see
`docs/sorx-deployment-handoff.md`.

For a local deterministic ontology-enabled handoff scenario, see
`docs/ontology-handoff-scenario.md`.

For production hardening notes, see `docs/ontology-production-readiness.md`,
`docs/ontology-security.md`, and `docs/ontology-compatibility.md`.
