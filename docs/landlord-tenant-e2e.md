# Landlord/Tenant FoundationDB E2E

This scenario validates `greentic-sorla` as the place where a real system of
record is authored, lowered, packaged, migrated, and exposed through deterministic
agent endpoint operations.

The concrete provider implementation is not copied into this repository. The
e2e crate depends on the sibling `greentic-sorla-providers` workspace and uses
its FoundationDB provider to validate provider health, config validation, event
append/read, and projection persistence.

## Provider Mode

The current FoundationDB provider implementation in `greentic-sorla-providers`
uses a local/dev transactional backing that follows the intended FoundationDB
keyspace behavior. This e2e therefore validates the provider contract and SoRLa
integration path without requiring a local external FoundationDB daemon.

If a future external FoundationDB runtime mode is added, it should be gated by
an explicit environment variable and documented here.

## Commands

Full scenario:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb
```

Smoke scenario:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb --smoke
```

Wrapper script:

```bash
bash scripts/e2e/run-landlord-sor-e2e.sh
```

## What It Validates

The scenario starts from `tests/e2e/fixtures/landlord_sor_v1.yaml`, builds the
SoRLa IR and artifacts, writes realistic landlord, property, unit, tenant,
tenancy, payment, and maintenance events through the FoundationDB provider, and
persists a landlord portfolio projection.

It then evolves to `tests/e2e/fixtures/landlord_sor_v2.yaml`, applies the v2
migration twice, and asserts that the migration is idempotent and preserves the
v1 data while allowing new fields.

The migration is declared in SoRLa with an `idempotence_key` and explicit
`backfills`. The e2e harness reads those backfill declarations from canonical IR
instead of maintaining a separate hardcoded field list.

Finally it runs deterministic agent-style operations against the same state:

- create a tenant
- assign the tenant to Unit 2B
- record a rent payment
- add a maintenance request
- list active tenants
- update the tenant contact preference

The agent operations are deterministic fixtures, not LLM calls. They validate
that agent endpoint declarations can map to structured operations and provider
events without corrupting existing data.

Each mutating agent operation declares an `emits` contract that names the event,
stream template, and payload template. The e2e validates those declarations
before applying the operation, so missing or stale operation contracts fail the
scenario.

## Adding Scenarios

Add new real-world scenarios beside the landlord fixtures under
`tests/e2e/fixtures/`, keep provider dependencies inside non-publishable crates,
declare relationship fields with `references`, declare migrations with
`backfills`, declare mutating agent operations with `emits`, and expose the
scenario through `cargo xtask e2e <scenario>`.
