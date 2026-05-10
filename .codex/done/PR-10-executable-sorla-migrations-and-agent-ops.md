# PR-10 — Make SoRLa Migrations And Agent Operations Executable Contracts

Status: implemented and verified with `ci/local_check.sh`.

## Goal

The landlord/tenant FoundationDB e2e in PR-09 proves realistic workflows by
using a scenario harness to interpret migrations, validate relationships, and
map deterministic agent-style operations to provider events.

That exposed a SoRLa product gap: the language and IR can describe migrations
and agent endpoints, but they do not yet define an executable contract for:

- relationship constraints
- event-to-record mutation semantics
- migration defaults/backfills
- idempotent migration application
- agent operation execution plans
- structured operation results and errors

This PR should promote those pieces from e2e harness logic into first-class
SoRLa contracts.

## Current-Code Evidence

- `greentic-sorla-lang` supports records, events, migrations, and
  `agent_endpoints`, but migrations only list `projection_updates`.
- `greentic-sorla-ir` lowers metadata deterministically, but does not encode
  executable migration steps or relationship constraints.
- `greentic-sorla-pack` emits handoff artifacts, but not an executable plan that
  maps an agent endpoint to domain events or projection updates.
- PR-09's e2e must implement relationship validation, v1-to-v2 backfill
  defaults, idempotence markers, and agent operation dispatch inside the test
  harness.

## Files To Touch

- `crates/greentic-sorla-lang/src/ast.rs`
- `crates/greentic-sorla-lang/src/parser.rs`
- `crates/greentic-sorla-ir/src/lib.rs`
- `crates/greentic-sorla-pack/src/lib.rs`
- `crates/greentic-sorla-pack/tests/golden/`
- `docs/spec/v0.2.md` or a new `docs/spec/executable-contracts.md`
- `docs/agent-endpoints.md`
- `docs/landlord-tenant-e2e.md`
- PR-09 e2e fixtures after the contract exists

## Relationship Constraints

Add a way to express relationships in SoRLa records, for example:

```yaml
records:
  - name: Property
    fields:
      - name: landlord_id
        type: string
        references:
          record: Landlord
          field: id
```

Validation should reject references to unknown records/fields and preserve the
relationship contract in canonical IR.

## Executable Migration Steps

Extend migrations beyond `projection_updates` with additive field backfills and
idempotence markers, for example:

```yaml
migrations:
  - name: landlord-tenant-v2-fields
    compatibility: additive
    backfills:
      - record: Tenant
        field: preferred_contact_method
        default: null
      - record: Unit
        field: energy_rating
        default: null
    idempotence_key: landlord-tenant-v2-fields
```

Lower these steps into IR and expose them through artifacts so a downstream
runtime or e2e harness can apply them consistently.

## Agent Operation Execution Plans

Extend `agent_endpoints` with deterministic operation plans. Example shape:

```yaml
agent_endpoints:
  - id: create_tenant
    emits:
      event: TenantCreated
      stream: "landlord-tenant-sor/{landlord_id}"
      payload:
        id: "$generated.tenant_id"
        full_name: "$input.full_name"
        email: "$input.email"
```

The first pass does not need to run an LLM. It should describe how structured
agent input maps to domain events, projection queries, and structured outputs.

## Structured Results And Errors

Define a small output contract for operation execution:

- endpoint ID
- status
- structured result payload
- validation errors with input paths
- provider errors with safe messages

This should let e2e scenarios assert semantics without custom ad hoc result
types.

## Tests

Add focused tests for:

1. Parsing relationship constraints.
2. Rejecting unknown relationship references.
3. Parsing migration backfills/defaults.
4. Lowering executable migration steps deterministically.
5. Exporting migration contracts in pack artifacts.
6. Parsing agent operation plans.
7. Rejecting operation plans that emit unknown events or refer to unknown inputs.
8. Updating the landlord/tenant e2e fixtures to use first-class relationships,
   migration backfills, and operation plans.

## Acceptance Criteria

- SoRLa can express relationship constraints without e2e-only logic.
- SoRLa can express idempotent additive migration backfills.
- Agent endpoints can describe deterministic structured operation plans.
- Canonical IR and artifacts preserve these contracts.
- The PR-09 landlord/tenant e2e removes its custom migration/relationship/agent
  plan logic where possible and uses the first-class contracts.
- Existing package compatibility remains additive.
- `ci/local_check.sh` passes.
