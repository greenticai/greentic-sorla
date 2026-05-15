# SORX gtpack Validation

## Contract Purpose

This document defines the validation metadata contract that `greentic-sorla`
will embed in SoRLa `.gtpack` archives for downstream `greentic-sorx`
consumption.

The validation contract makes a pack self-describing enough for downstream SORX
tooling to decide whether a deployed pack version is eligible for public
exposure. It is deterministic handoff metadata. It is not a test runner,
runtime deployment system, provider connector, GHCR webhook handler, public
router, or rollback implementation.

The first validation manifest schema is:

```text
greentic.sorx.validation.v1
```

## Ownership Boundaries

`greentic-sorla` owns:

- deterministic validation manifest generation
- validation schema definitions
- pack inclusion
- `pack doctor` and `pack inspect` checks for embedded validation assets
- static consistency checks against SoRLa IR and pack metadata

Downstream `greentic-sorx` tooling owns:

- retrieving immutable pack artifacts
- verifying source, digests, signatures, and version policy
- resolving concrete providers
- starting preview runtimes
- executing validation tests
- running migrations
- enforcing public exposure gates
- storing validation reports
- promoting or rolling back deployments

This repository should only emit the deterministic metadata needed by SORX. It
must not move SORX runtime or deployment behavior into SoRLa packaging.

## Pack Asset Layout

When validation metadata is enabled, the pack contains this required entry:

```text
assets/sorx/tests/test-manifest.json
```

The manifest may reference optional validation assets under:

```text
assets/sorx/tests/data/*.json
assets/sorx/tests/contracts/*.json
assets/sorx/tests/security/*.json
assets/sorx/tests/providers/*.json
assets/sorx/tests/migrations/*.json
assets/sorx/tests/fixtures/*.json
```

Only `test-manifest.json` is mandatory when validation is enabled. Optional
assets are included only when referenced by relative path from the manifest.
References must stay within `assets/sorx/tests/`, must not begin with `/`, and
must not contain `..`.

## Manifest Shape

The validation manifest is JSON. The minimum shape is:

```json
{
  "schema": "greentic.sorx.validation.v1",
  "suite_version": "1.0.0",
  "package": {
    "name": "landlord-tenant-sor",
    "version": "0.1.0"
  },
  "default_visibility": "private",
  "promotion_requires": [
    "smoke",
    "contract",
    "ontology",
    "retrieval",
    "security",
    "provider"
  ],
  "suites": [
    {
      "id": "smoke",
      "required": true,
      "tests": []
    }
  ]
}
```

`package.name` and `package.version` must match the SoRLa package identity in
the pack. Future implementations may add `ir_version` and `ir_hash` under
`package` when the generator has those values available.

`default_visibility` describes the validation contract's conservative exposure
posture. It does not create routes, public aliases, or deployed endpoints.
Current SoRLa endpoint metadata describes export surfaces such as OpenAPI,
Arazzo, MCP, and `llms.txt`; downstream SORX decides whether any exported
surface is exposed publicly.

`promotion_requires` lists suite IDs that downstream SORX must treat as required
before promotion unless an explicit local operator policy overrides the gate.
Ontology and retrieval suites are required for exported/public-candidate packs
when the corresponding SoRLa artifacts exist. Private-only packs may still carry
ontology suites, but those suites are emitted with `required: false` and are not
listed in `promotion_requires`.

## Suites And Tests

Each suite has:

- `id`: stable unique suite ID
- `title`: optional display title
- `required`: whether the suite is required by default
- `tests`: deterministic list of tests in the suite

Test IDs should be unique within the manifest. Suites and tests should be
emitted in stable order so repeated generation from the same source produces the
same manifest bytes.

The initial test kind vocabulary is:

- `healthcheck`
- `agent-endpoint`
- `openapi-contract`
- `mcp-tool-contract`
- `arazzo-workflow`
- `provider-capability`
- `provider-connectivity`
- `auth-required`
- `policy-enforced`
- `tenant-isolation`
- `migration-compatibility`
- `rollback-compatibility`
- `ontology-static`
- `ontology-relationship`
- `ontology-alias`
- `entity-linking`
- `retrieval-binding`

The vocabulary is intentionally broader than the first generator. Early SoRLa
implementations may scaffold only static handoff checks and contract metadata.
Downstream SORX is responsible for interpreting and executing runnable tests.

Ontology suites validate deterministic handoff structure only: concept and
relationship metadata, aliases, and entity-linking declarations. Retrieval
suites validate abstract binding metadata such as provider category/capability
requirements and ontology-scoped retrieval filters. Provider compatibility still
uses `provider-capability`; high-risk, approval-gated, side-effectful, or
exported endpoints still use `policy-enforced` security checks.

Suite objects intentionally have no `kind` or
`required_for_public_exposure` fields. Kind lives on individual tests, and
promotion gating is represented by the suite's `required` flag plus the
top-level `promotion_requires` list.

## Static Validation

`greentic-sorla pack doctor` validates embedded validation metadata without
executing tests:

- manifest JSON parses
- `schema` equals `greentic.sorx.validation.v1`
- package name and version are present
- package identity matches the pack and SoRLa IR
- suite IDs are unique
- required promotion suite IDs exist
- test IDs are unique
- relative references do not escape `assets/sorx/tests/`
- referenced optional assets exist in the pack
- validation assets are covered by deterministic lock metadata
- obsolete suite-level fields such as `kind` or
  `required_for_public_exposure` are rejected by the manifest shape

These checks are static handoff checks. HTTP calls, provider connectivity,
runtime healthchecks, tenant isolation probes, and migration execution belong to
downstream SORX tooling.

## CLI Inspection

Developers and downstream SORX implementers can inspect the validation contract
without unpacking archives manually:

```bash
greentic-sorla pack schema validation
greentic-sorla pack schema exposure-policy
greentic-sorla pack schema compatibility
greentic-sorla pack schema ontology
greentic-sorla pack schema retrieval-bindings
greentic-sorla pack validation-inspect landlord-tenant-sor.gtpack
greentic-sorla pack validation-doctor landlord-tenant-sor.gtpack
greentic-sorla pack doctor landlord-tenant-sor.gtpack
```

`validation-doctor` is an alias for the same static checks as `pack doctor`.
`pack doctor` remains the canonical validation path for CI.

## Exposure Gate

SORX must not expose public endpoints from a validation-enabled pack unless the
required validation suites pass or an explicit local operator policy overrides
that gate.

This rule is a downstream promotion requirement. `greentic-sorla` only emits the
metadata and performs static checks. It does not create public routes, start
runtimes, execute providers, or promote aliases.

See `docs/sorx-deployment-handoff.md` for the expected downstream lifecycle from
GHCR publish event through preview deployment, certification report, public
exposure gate, and alias promotion.

## Compatibility Strategy

The first schema is `greentic.sorx.validation.v1`.

Additive changes may add optional fields within `v1`. Breaking changes require
a `v2` schema. Consumers should reject unknown major versions rather than
guessing at runtime behavior.

Validation assets must remain deterministic: no generated timestamps, absolute
paths, usernames, machine-specific directories, secrets, or environment-derived
values should be embedded.
