# PR-11 — Add full domain answers for real landlord/tenant gtpack generation

## Repository

greenticai/greentic-sorla

## Objective

Replace the placeholder landlord/tenant answers flow with a full domain-capable
answers model that can generate a real SoRLa source package and `.gtpack` from
`answers.json`.

The current `examples/landlord-tenant/answers.json` only selects high-level
wizard knobs. It does not describe real records, fields, relationships, events,
projections, actions, policies, approvals, or agent endpoint contracts. The
resulting `.gtpack` is structurally valid, but it represents a generic
`LandlordTenantSorRecord` package rather than the landlord/tenant e2e domain.

This PR makes the answers document a genuine source-of-truth format for a
domain package, so downstream `greentic-sorx` testing can consume a meaningful
GHCR-published pack.

## Current problem

`examples/landlord-tenant/answers.json` currently contains only:

- package name and version
- record source mode
- events enabled flag
- projection mode
- migration compatibility mode
- agent endpoint IDs and default endpoint metadata
- output artifact preferences

The implementation then fabricates:

- one generic record named `<PackageName>Record`
- one generic `Changed` event
- one generic projection
- generic agent endpoints with `record_id -> status`

The real landlord/tenant e2e fixture already has the desired shape in
`tests/e2e/fixtures/landlord_sor_v1.yaml`:

- records: `Landlord`, `Property`, `Unit`, `Tenant`, `Tenancy`, `Payment`,
  `MaintenanceRequest`
- typed fields and cross-record `references`
- actions such as `CreateTenant` and `RecordRentPayment`
- domain events such as `TenantCreated`, `TenancyCreated`, `PaymentRecorded`
- projections such as `LandlordPortfolio` and `ActiveTenants`
- provider requirements
- policies and approvals
- agent endpoints with concrete inputs, outputs, side effects, emitted event
  payload templates, and backing metadata

The `.gtpack` machinery can package rich SoRLa YAML today. The gap is that
`wizard --answers` cannot author that rich YAML.

## Design principles

- `answers.json` should be deterministic and reviewable.
- The answers schema should support real domain declarations, not only wizard
  defaults.
- Generated YAML should preserve current safe generated-block behavior.
- Minimal answers should continue to work for scaffolding demos.
- Rich answers should be able to round-trip the landlord/tenant v1 fixture
  closely enough that the generated `.gtpack` is meaningful for SORX.
- Validation errors should point to answer paths such as
  `records[2].fields[1].references.record`.
- Do not put provider runtime implementation, GHCR publish, SORX deployment, or
  provider credentials into `greentic-sorla`.

## Proposed answer schema

Add a new schema version, preferably `0.5`, with backward compatibility for
existing `0.4` minimal answers.

### Package

Keep existing shape:

```json
{
  "package": {
    "name": "landlord-tenant-sor",
    "version": "0.1.0"
  }
}
```

### Records

Extend `records` from simple defaults into a section that can include concrete
declarations:

```json
{
  "records": {
    "default_source": "native",
    "items": [
      {
        "name": "Property",
        "source": "native",
        "fields": [
          { "name": "id", "type": "string" },
          {
            "name": "landlord_id",
            "type": "string",
            "references": { "record": "Landlord", "field": "id" }
          },
          { "name": "address", "type": "string" }
        ]
      }
    ]
  }
}
```

Supported field properties:

- `name`
- `type`
- `required`
- `sensitive`
- `enum_values`
- `references`
- `authority`

Validation rules:

- record names must be unique
- field names within a record must be unique
- references must point to existing record and field names
- `authority` is valid only for hybrid records unless the language already
  permits broader usage

### Actions

Add:

```json
{
  "actions": [
    { "name": "CreateTenant" },
    { "name": "AssignTenantToUnit" }
  ]
}
```

Validation rules:

- action names must be unique
- backing references from agent endpoints must point to declared actions

### Events

Extend `events` to support either the old toggle or concrete declarations:

```json
{
  "events": {
    "enabled": true,
    "items": [
      {
        "name": "TenantCreated",
        "record": "Tenant",
        "kind": "domain",
        "emits": [
          { "name": "id", "type": "string" },
          { "name": "full_name", "type": "string" },
          { "name": "email", "type": "string" }
        ]
      }
    ]
  }
}
```

Validation rules:

- event names must be unique
- `record` must point to an existing record
- emitted fields should either match known record fields or be explicitly
  allowed generated fields

### Projections

Extend `projections`:

```json
{
  "projections": {
    "mode": "current-state",
    "items": [
      {
        "name": "ActiveTenants",
        "record": "Tenant",
        "source_event": "TenancyCreated",
        "mode": "current-state"
      }
    ]
  }
}
```

Validation rules:

- projection names must be unique
- `record` must point to an existing record
- `source_event` must point to an existing event

### Provider Requirements

Support concrete provider requirements while keeping the old abstract defaults:

```json
{
  "provider_requirements": [
    {
      "category": "storage",
      "capabilities": ["event-log", "projections"]
    },
    {
      "category": "event-store",
      "capabilities": ["append", "read-stream"]
    }
  ]
}
```

If absent, retain the existing generated defaults.

### Policies And Approvals

Add:

```json
{
  "policies": [
    { "name": "TenancyWritePolicy" }
  ],
  "approvals": [
    { "name": "LandlordRecordApproval" }
  ]
}
```

Validation rules:

- names must be unique per section
- agent endpoint backing references must point to declared policies and
  approvals

### Migrations

Extend `migrations`:

```json
{
  "migrations": {
    "compatibility": "additive",
    "items": [
      {
        "name": "landlord-tenant-v1-initial",
        "compatibility": "additive",
        "projection_updates": ["LandlordPortfolio", "ActiveTenants"]
      }
    ]
  }
}
```

For this PR, v1 can stay simple. Rich v2 migration/backfill support can be
deferred unless needed for SORX tests.

### Agent Endpoints

Extend `agent_endpoints` with concrete `items`. Preserve the old `ids` shortcut
for scaffolding.

```json
{
  "agent_endpoints": {
    "enabled": true,
    "items": [
      {
        "id": "create_tenant",
        "title": "Create tenant",
        "intent": "Create a tenant record from structured agent input.",
        "inputs": [
          { "name": "full_name", "type": "string", "required": true },
          { "name": "email", "type": "string", "required": true, "sensitive": true },
          { "name": "phone", "type": "string", "required": false }
        ],
        "outputs": [
          { "name": "tenant_id", "type": "string" }
        ],
        "side_effects": ["event.TenantCreated"],
        "emits": {
          "event": "TenantCreated",
          "stream": "landlord-tenant-sor/{landlord_id}",
          "payload": {
            "id": "$generated.tenant_id",
            "full_name": "$input.full_name",
            "email": "$input.email",
            "phone": "$input.phone"
          }
        },
        "risk": "medium",
        "approval": "policy-driven",
        "backing": {
          "actions": ["CreateTenant"],
          "events": ["TenantCreated"],
          "policies": ["TenancyWritePolicy"],
          "approvals": ["LandlordRecordApproval"]
        },
        "agent_visibility": {
          "openapi": true,
          "arazzo": true,
          "mcp": true,
          "llms_txt": true
        }
      }
    ]
  }
}
```

Validation rules:

- endpoint IDs must be unique
- input and output names must be unique within each endpoint
- high-risk endpoints must require `required` or `policy-driven` approval
- emitted event must exist
- backing references must exist
- `$input.<name>` payload references must point to endpoint inputs
- `$generated.<name>` payload references are allowed

## Rendering strategy

Change `render_package_yaml` from hardcoded generic scaffolding to:

1. If rich domain sections are present, render those sections exactly and
   deterministically.
2. If rich sections are absent, retain the current generic scaffold behavior for
   minimal examples and backward compatibility.
3. Sort only where existing semantics require deterministic ordering. Preserve
   explicit user order for records, fields, events, projections, and endpoints
   unless the repo has a stronger canonical ordering rule.

Recommended implementation approach:

- Introduce typed answer structs for:
  - `RecordDeclarationAnswers`
  - `FieldDeclarationAnswers`
  - `ActionDeclarationAnswers`
  - `EventDeclarationAnswers`
  - `ProjectionDeclarationAnswers`
  - `ProviderRequirementAnswers`
  - `PolicyDeclarationAnswers`
  - `ApprovalDeclarationAnswers`
  - `MigrationDeclarationAnswers`
  - `AgentEndpointDeclarationAnswers`
- Convert answers into an intermediate package authoring model.
- Render YAML from that model.
- Prefer using `serde_yaml` serialization for structured sections if output
  stability is acceptable; otherwise keep explicit line rendering but drive it
  from typed data, not string templates.

## Example updates

Replace `examples/landlord-tenant/answers.json` with a rich answer document
that describes the full landlord/tenant v1 domain.

Regenerate:

```bash
cargo run -p greentic-sorla -- wizard \
  --answers examples/landlord-tenant/answers.json \
  --pack-out landlord-tenant-sor.gtpack
```

The generated `examples/landlord-tenant/sorla.yaml` should contain:

- all landlord/tenant records and relationships
- all v1 events
- all v1 projections
- provider requirements
- policies and approvals
- real agent endpoint contracts

The generated `.gtpack` should inspect as:

- package name: `landlord-tenant-sor`
- version: `0.1.0`
- extension: `greentic.sorx.runtime.v1`
- validation metadata present
- exposure metadata present
- compatibility metadata present
- agent endpoint artifacts present

## Tests

Add or update tests in `crates/greentic-sorla-cli/src/lib.rs` or integration
tests:

1. Rich records render test
   - answer document with two related records
   - generated YAML contains both records and `references`

2. Rich events/projections render test
   - answer document with an event and projection
   - generated YAML references declared record and event

3. Rich agent endpoint render test
   - endpoint with typed inputs, outputs, emits, and backing
   - generated YAML contains the exact endpoint contract

4. Validation tests
   - unknown record reference fails
   - unknown event reference fails
   - unknown backing action fails
   - `$input.missing` payload reference fails

5. Landlord/tenant example test
   - run `wizard --answers examples/landlord-tenant/answers.json --pack-out landlord-tenant-sor.gtpack`
   - doctor the generated pack
   - inspect the generated pack
   - assert the generated YAML contains `Landlord`, `Tenant`, `Tenancy`,
     `TenantCreated`, `LandlordPortfolio`, and real endpoint fields such as
     `full_name`, `tenant_id`, and `unit_id`

6. Backward compatibility test
   - existing minimal `0.4` answers still produce a valid generic package

## Documentation

Update:

- `docs/wizard.md`
- `docs/sorla-gtpack.md`
- `docs/packaging.md`
- `README.md`
- `examples/landlord-tenant/README.md`

Document:

- minimal answers mode versus rich domain answers mode
- rich answer schema examples
- how to regenerate the landlord/tenant pack
- that the checked-in landlord/tenant pack is generated from rich answers, not
  from the e2e YAML fixture

## Acceptance criteria

- `examples/landlord-tenant/answers.json` describes the real landlord/tenant v1
  domain, not a placeholder package.
- `wizard --answers examples/landlord-tenant/answers.json --pack-out landlord-tenant-sor.gtpack`
  generates a meaningful landlord/tenant `sorla.yaml`.
- The generated pack passes `pack doctor`.
- The generated pack exposes real agent endpoint handoff metadata.
- The answer schema remains backward compatible with existing minimal examples.
- Validation errors for rich answers are actionable and path-specific.
- CI/local checks exercise the answers-generated landlord/tenant pack.

## Non-goals

- Implement SORX runtime execution.
- Publish to GHCR.
- Add concrete provider credentials or runtime provider bindings.
- Replace the existing SoRLa YAML parser.
- Implement full v2 migration/backfill authoring unless needed for the initial
  landlord/tenant pack.

## Open questions

- Should rich answers be schema version `0.5`, or should `0.4` accept additive
  fields?
- Should generated YAML preserve answer ordering exactly, or should it sort for
  canonical stability?
- Should the rich answer structs live in the CLI crate initially, or should they
  move into `greentic-sorla-wizard` as a reusable library boundary?
- Should the landlord/tenant example be generated from rich answers only, or
  should the e2e YAML fixture be mechanically converted into answers in a helper
  test to prevent drift?
