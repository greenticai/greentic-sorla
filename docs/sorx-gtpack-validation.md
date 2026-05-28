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

## Record Scalar Types and Rules

Sorx should consume regenerated gtpacks after any `sorla.yaml` record definition
change. The updated canonical model now carries richer record field metadata in
the package IR, including `records[].fields[].type` and
`records[].fields[].rules`. It also carries stable `i18n_key` metadata so Sorx
and downstream UI surfaces can resolve labels from sidecar locale catalogs
instead of from translated schema copies. There is no separate Sorx flag for
this: rebuild the gtpack from the latest English-base `sorla.yaml`, publish or
deploy that pack, and make sure Sorx reloads the new package artifact instead
of a cached model hash.

Summary of record-definition changes Sorx should expect:

- scalar field types now include `uuid`, `email`, `url`, `date`, `time`, and
  `datetime` in addition to the existing base scalar types
- field validation rules are emitted under `records[].fields[].rules`
- schema objects may include `i18n_key`
- localization is represented by sidecar catalogs such as `i18n/en.json` and
  `i18n/es.json`; do not expect one `sorla.yaml` per language

Record fields now support these first-class scalar types:

- `string`
- `decimal`
- `integer`
- `boolean`
- `uuid`
- `email`
- `url`
- `date`
- `time`
- `datetime`

Compatibility aliases such as `timestamp`, `bool`, `int`, `number`, `float`,
`double`, `u32`, `enum`, and `array` may still appear in older packs, but new
generated definitions should prefer `datetime` for timestamp values and the
semantic scalar types above where they apply.

Record fields may also include a `rules` object with these validation keys:

- `min` and `max` for numeric lower and upper bounds.
- `min_length`, `max_length`, and `pattern` for text-like fields.
- `precision` and `scale` for decimal fields.
- `before` and `after` for `date`, `time`, `datetime`, and legacy `timestamp`
  fields.
- `unique` for scalar uniqueness expectations.

Rules are record-field metadata only. Endpoint `inputs` and `outputs` do not
carry `rules` yet, so Sorx should derive endpoint validation from the record
model, operational indexes, and execution plan rather than expecting rule
objects under `agent_endpoints.items[].inputs` or `outputs`.

Because these values are part of the canonical model, changing field types or
rules changes the generated IR and package hash. Sorx deployments should treat a
new gtpack as the source of truth for validation, generated routes, inspect
output, and downstream contract checks.

## Record Hierarchy

Sorx should use `assets/sorla/agent-gateway.json` `record_hierarchy` to decide
which record entities are top-level navigation entities and which are dependent
entities that require parent context. The hierarchy is derived from record
reference fields and enriched with ontology relationship ids when present.

Each item has:

- `record`: the canonical record name.
- `main`: true when the record has no parent record references and may be shown
  as a top-level entity.
- `parents`: parent record requirements. Each parent includes the parent
  `record`, the child-side reference `field`, and optional ontology
  `relationship`.

Sorx should use this when generating menus and forms:

- Show `main: true` records as primary menu entries.
- Do not show dependent records as unconstrained top-level create actions.
- Surface dependent create actions from a parent context and prefill or require
  the parent reference field. For example, a `Building` with parent `Landlord`
  belongs under a landlord, and a `Unit` with parent `Building` belongs under a
  building.
- If a record has multiple parents, only create it when the required parent
  references are known or explicitly selected.

Generic admin endpoints that use `execution.record_selector` are record-wide
operations. Their generated agent-gateway entity is `Record` and their
collection is `records`; Sorx should place them in admin tooling instead of
merging them into a specific entity create form.

Endpoint forms should be generated from a single endpoint's `input_schema`.
Do not merge input fields from multiple endpoints just because they share an
entity or collection. Create, update, remove, and admin tools remain separate
actions with separate schemas.

## Metrics Localization

Sorx should localize metric navigation and detail views from
`assets/sorla/metrics.json` plus the embedded locale catalogs. Each metric item
may include `i18n_key`; resolve the visible label from
`<i18n_key>.label` in the active locale and use the metric `label` only as the
English fallback. Do not show raw metric ids such as `active_tenancies` as
primary display text when a localized label exists.

Metric detail views should also prefer the localized label in headings and
captions. The raw `name` remains the stable query identifier and may be shown as
developer metadata, but it should not replace the translated label in the
normal user flow.

## Role and Authorization Enforcement

Sorx should enforce roles from the latest gtpack, not from endpoint naming
conventions or legacy `execution.authorization` payloads. The canonical model in
`assets/sorla/model.cbor` now includes:

- `roles[]`: declared role ids, labels, descriptions, i18n keys, and optional
  grant strings for operator/UI policy mapping.
- `records[].access`: CRUD access rules per record. Each rule may list `roles`
  and `policies`.
- `agent_endpoints[].authorization`: invocation requirements. `roles.any_of`
  requires at least one listed role, `roles.all_of` requires every listed role,
  `policies` delegates to the policy engine, and `conditions` are structured
  policy inputs.
- `assets/sorla/agent-gateway.json` repeats endpoint `authorization` so route
  generation can enforce endpoint access without decoding all endpoint IR first.

Recommended Sorx enforcement:

- Resolve the authenticated principal's roles to SoRLa role ids before route or
  record execution.
- Deny by default when an endpoint has `authorization` and the principal fails
  `any_of`, `all_of`, or policy checks.
- Deny record CRUD when `records[].access.<operation>` exists and the principal
  fails the listed roles or policies.
- Treat `approval` as a workflow gate after authorization succeeds, not as a
  replacement for authorization.
- Do not give `admin` implicit bypass unless the package explicitly lists
  `admin` in the relevant access or authorization rule.
- Ignore or reject new packages that still rely on
  `execution.authorization.required_roles`; regenerate them with first-class
  `authorization`.

Because no production Sorx package consumes SoRLa yet, Sorx can implement this
as the baseline behavior without a compatibility fallback.

## Compatibility Strategy

The first schema is `greentic.sorx.validation.v1`.

Additive changes may add optional fields within `v1`. Breaking changes require
a `v2` schema. Consumers should reject unknown major versions rather than
guessing at runtime behavior.

Validation assets must remain deterministic: no generated timestamps, absolute
paths, usernames, machine-specific directories, secrets, or environment-derived
values should be embedded.
