# PR-09 — Landlord/Tenant Sorla E2E With FoundationDB Provider

## Goal

Create a repeatable end-to-end validation scenario in `greentic-sorla` for a
realistic landlord/tenant system of record authored through the wizard/SoRLa
package surface, using the FoundationDB provider from `greentic-sorla-providers`
to store and retrieve data.

This PR belongs in `greentic-sorla` because the wizard, SoRLa authoring model,
canonical IR, packaging, migrations, and agent endpoint handoff artifacts live
here. Do not move provider implementation work into this repository.

`greentic-sorla-providers` is an integration dependency for this PR. Use its
FoundationDB provider crate to exercise provider health, config validation,
event append/read, and projection persistence/rebuild behavior.

## Corrected Current-Code Notes

- `greentic-sorla` currently does not boot a runtime server or own concrete
  provider implementations.
- Provider implementations live in sibling repo
  `/projects/ai/greentic-ng/greentic-sorla-providers`.
- `greentic-sorla-providers/providers/provider-foundationdb` already exists.
  Its current implementation is a local/dev FoundationDB-compatible provider
  with transactional in-memory backing, plus a `cluster_file` config field kept
  for the intended external FoundationDB runtime path.
- This PR should not edit `greentic-sorla-providers` unless a tiny upstream bug
  is discovered and explicitly separated. The main work happens here.
- The publishable `greentic-sorla` CLI crate must not gain dev-dependencies on
  unpublished local provider crates, because `cargo package`/`cargo publish
  --dry-run` must keep passing.
- If provider crate dependencies are needed, put them in a new `publish = false`
  e2e/xtask crate, not in `crates/greentic-sorla-cli`.
- The repo currently has no `xtask` crate. Add one only if it is the cleanest way
  to provide the requested developer command.

## Files To Touch

Prefer this structure, adjusted only if the codebase shape demands it:

- `Cargo.toml`
- New crate: `crates/greentic-sorla-e2e`
- New crate: `xtask`
- `tests/e2e/README.md` or `docs/landlord-tenant-e2e.md`
- `tests/e2e/fixtures/landlord_sor_v1.yaml`
- `tests/e2e/fixtures/landlord_sor_v2.yaml`
- `tests/e2e/fixtures/landlord_seed_data.json`
- `scripts/e2e/run-landlord-sor-e2e.sh`
- `.github/workflows/landlord-tenant-e2e.yml`
- `README.md` small cross-link
- `.codex/repo_overview.md`

Avoid adding provider path dependencies to `crates/greentic-sorla-cli`.

## Developer Command

Add one clear command:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb
```

Add a faster smoke command:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb --smoke
```

The xtask should run the e2e crate/test harness, set any required fixture paths,
and return a non-zero exit code on assertion failure.

## Provider Integration

Use the FoundationDB provider from the sibling provider repo through a path
dependency in a non-publishable crate, for example:

```toml
[dependencies]
provider-foundationdb = { path = "../../../greentic-sorla-providers/providers/provider-foundationdb" }
sorla-provider-core = { path = "../../../greentic-sorla-providers/crates/sorla-provider-core" }
```

The e2e must validate:

- provider health returns ready
- provider config validation accepts a non-empty `cluster_file` and
  `tenant_prefix`
- invalid provider config returns a useful error
- events can be appended and read back
- projection records can be persisted and fetched
- projection rebuild/checkpoint behavior is exercised if supported

If the provider still uses its local/dev in-memory backing, document that this
PR validates the FoundationDB provider contract and keyspace-compatible behavior
without requiring a running external FoundationDB cluster. Do not claim it has
validated a real external cluster unless the test actually starts and uses one.

If an external FoundationDB mode exists by implementation time, gate it behind an
explicit flag/env var rather than making every local run require a daemon:

```bash
SORLA_E2E_FOUNDATIONDB_EXTERNAL=1 cargo xtask e2e landlord-tenant --provider foundationdb
```

## Landlord / Tenant SoR Model

Create a realistic v1 SoR fixture with these entities:

- `Landlord`
- `Property`
- `Unit`
- `Tenant`
- `Tenancy`
- `Payment`
- `MaintenanceRequest`

Start with a minimal v1 schema:

```text
Tenant:
- id
- full_name
- email
- phone

Property:
- id
- landlord_id
- address

Unit:
- id
- property_id
- unit_number
- rent_amount
- status

Tenancy:
- id
- tenant_id
- unit_id
- start_date
- end_date
- status
```

Include enough records to feel like a real small landlord system:

- one landlord
- one property
- multiple units
- multiple tenants
- one active tenancy
- one previous tenancy
- at least one rent payment
- at least one maintenance request

## Data And Provider Assertions

The test should use the current SoRLa compiler/packaging path first:

1. Parse the v1 SoRLa YAML fixture.
2. Lower it into canonical IR.
3. Build deterministic artifacts.
4. Use the FoundationDB provider to append domain events derived from seed data.
5. Read event streams back and assert exact stored data.
6. Persist read-model/projection snapshots for practical queries.
7. Fetch projections and assert active tenants, units, payments, and maintenance
   requests.

Add assertions for:

- data can be written
- data can be read back
- records can be updated by appending later events
- records can be queried/listed through projections
- tenant/unit/tenancy relationships remain intact
- invalid relationship references produce useful test failures or provider
  validation errors where the current contracts support them

Do not rely on logs as proof.

## Schema Migration Test

Add a v2 fixture that evolves the schema with fields such as:

Tenant:

- `date_of_birth`
- `emergency_contact_name`
- `emergency_contact_phone`
- `preferred_contact_method`

Unit:

- `bedrooms`
- `bathrooms`
- `furnished`
- `energy_rating`

Tenancy:

- `deposit_amount`
- `deposit_scheme_reference`
- `renewal_notice_date`

The migration test should:

1. Create v1 data.
2. Build/lower the v2 schema.
3. Apply a deterministic migration event or projection update path.
4. Assert old records still load safely.
5. Assert missing new fields are represented safely.
6. Add new fields to existing records.
7. Insert a new record using the expanded schema.
8. Re-run migration at least twice.
9. Assert migration is idempotent and does not duplicate events or corrupt
   projections.

## Agentic Interface Validation

Use the agent endpoint model and exporter APIs already added in PR-01 through
PR-08. The e2e should include agent-style operations against the same landlord
system:

- "Create a new tenant called Sarah Ahmed with email sarah@example.com"
- "Assign Sarah Ahmed to Unit 2B starting 2026-06-01"
- "Record a 1250 rent payment for Sarah Ahmed"
- "Add a maintenance request for Unit 2B: heating not working"
- "Show all active tenants for this landlord"
- "Update Sarah's preferred contact method to email"

The implementation does not need an LLM. Treat these as deterministic
agent-operation fixtures that map to SoRLa agent endpoint declarations and
domain events.

Assert that agentic calls:

- map to the intended SoRLa agent endpoint IDs
- enforce input/schema constraints
- return structured results
- append the expected provider events
- update the expected projections
- do not corrupt pre-existing data
- surface useful errors for invalid requests
- behave consistently before and after migration

## Suggested E2E Crate Shape

Use a non-publishable crate so provider path dependencies do not affect the
publishable CLI package:

```text
crates/greentic-sorla-e2e/
  Cargo.toml
  src/
    lib.rs
    landlord_tenant.rs
  tests/
    landlord_tenant_foundationdb.rs

tests/e2e/
  fixtures/
    landlord_sor_v1.yaml
    landlord_sor_v2.yaml
    landlord_seed_data.json
  README.md
```

The e2e crate should depend on local `greentic-sorla-lang`,
`greentic-sorla-ir`, and `greentic-sorla-pack` crates, plus the sibling provider
repo crates by path.

## Scripts

Add a simple wrapper:

```bash
scripts/e2e/run-landlord-sor-e2e.sh
```

This script should call:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb "$@"
```

Do not add `start-foundationdb.sh` or `stop-foundationdb.sh` unless the test
actually supports a real external FoundationDB process. If external mode is
added, those scripts should be clearly optional and not required for the default
local/dev provider contract run.

## CI Integration

Add a GitHub Actions workflow or job that is manually runnable:

```yaml
workflow_dispatch:
  inputs:
    provider:
      default: foundationdb
    scenario:
      default: landlord-tenant
    smoke:
      default: "false"
```

Make the default workflow run:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb
```

If the test is too heavy for every PR, run it on:

- `workflow_dispatch`
- nightly schedule
- release/publish workflow as a required pre-release check if runtime cost is
  acceptable

Keep normal PR CI fast unless the smoke mode is cheap enough.

## Documentation

Add docs that explain:

- why the scenario lives in `greentic-sorla`
- how it uses `greentic-sorla-providers` without moving provider code here
- how to run the e2e locally
- how to run smoke mode
- what FoundationDB provider mode is actually being tested
- how the v1 to v2 schema migration is validated
- how deterministic agent operations are mapped to agent endpoints
- how to add future real-world SoR scenarios

## Tests

At minimum, add tests that prove:

1. v1 fixture parses and lowers deterministically.
2. provider health/config checks pass.
3. seed events are appended and read back.
4. projections support practical list/query assertions.
5. updates append events and preserve relationships.
6. v2 migration preserves v1 data.
7. migration is idempotent when run twice.
8. deterministic agent operations produce structured results.
9. invalid agent operation input returns a useful error.
10. smoke mode runs a smaller but meaningful subset.

## Local Checks

Update `ci/local_check.sh` only if the new e2e smoke should become part of the
standard local gate. Do not add the full e2e if it makes normal local checks too
slow or dependent on sibling repo availability.

If `ci/local_check.sh` references the e2e, it must degrade clearly when the
sibling `../greentic-sorla-providers` repo is missing.

## Acceptance Criteria

- The PR is implemented in `greentic-sorla`.
- `greentic-sorla-providers` is used as a sibling provider dependency only.
- No provider implementation code is copied into this repo.
- The wizard/SoRLa fixture path can build a landlord/tenant system of record.
- The FoundationDB provider contract is exercised for write/read/update/query
  style workflows.
- v1 data survives v2 schema evolution.
- Migration can be run twice without corrupting or duplicating data.
- Agent endpoint operations are tested against the same scenario.
- The scenario is repeatable through `cargo xtask e2e landlord-tenant --provider foundationdb`.
- CI can run the scenario manually and optionally on nightly/release.
- Docs explain the exact provider mode being tested.
- `ci/local_check.sh` passes, or any external/provider availability limitation
  is documented precisely.
