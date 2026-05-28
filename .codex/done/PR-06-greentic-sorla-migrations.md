# PR: Extend SoRLa migrations with typed compatibility operations

Repo: `greenticai/greentic-sorla`

## Goal

Build on the existing migration contract so packages can describe richer
forward-only compatibility operations while old views and consumers remain
active.

## Current-code assumptions

- `migrations` already exists in the v0.2 language.
- Current migration declarations use `name`, `compatibility`,
  `projection_updates`, optional `idempotence_key`, optional `notes`, and
  structured `backfills`.
- Migration backfills already lower into canonical IR and are included in
  `executable-contract.json`.
- The landlord/tenant e2e already consumes migration backfills from canonical IR
  and proves idempotence by applying them twice.
- Downstream Sorx owns runtime migration execution. This repo should emit
  deterministic metadata and static checks, not execute providers or run
  deployment migrations.

## Existing shape to preserve

```yaml
migrations:
  - name: landlord-tenant-v2-fields
    compatibility: additive
    idempotence_key: landlord-tenant-v2-fields
    backfills:
      - record: Tenant
        field: preferred_contact_method
        default: null
    projection_updates:
      - TenantCurrentState
```

## Proposed additive shape

Add typed operations as an optional extension to the existing migration object:

```yaml
migrations:
  - name: landlord-tenant-1.1.0-to-2.0.0
    compatibility: backward-compatible
    from_version: 1.1.0
    to_version: 2.0.0
    idempotence_key: landlord-tenant:1.1.0:2.0.0
    operations:
      - kind: add-record
        record: Person
      - kind: split-record
        from_record: Tenant
        into_records:
          - Person
          - Tenancy
      - kind: require-index
        index: active_tenants_by_property
```

Use `operations` rather than `steps` to avoid implying that SoRLa executes an
ordered imperative script locally. Operations are declarative compatibility
metadata. Provider-specific execution plans remain downstream.

## Implementation notes

- Keep the existing migration fields backward-compatible.
- Add parser validation for operation kind, record references, field references
  where needed, and optional version ordering if semantic versions are supplied.
- If `require-index` is included, validate it against the operational index
  declarations introduced by the index PR; until that PR lands, keep this
  operation behind parser tests that supply the new section.
- Lower operations into canonical IR and include them in `model.cbor`,
  `compatibility.cbor`, and `executable-contract.json`.
- Extend SORX validation metadata with static migration compatibility checks,
  but leave dry-run/apply execution to downstream Sorx.
- Update landlord/tenant fixtures without replacing the existing backfill-based
  idempotence coverage.

## Acceptance criteria

- Existing migration YAML continues to parse and lower unchanged.
- IR includes typed migration operations in addition to existing backfills.
- Pack validation checks stable migration names/idempotence keys and rejects
  invalid operation references.
- `executable-contract.json` exposes the richer migration metadata for
  downstream executors.
- Landlord/tenant fixture demonstrates a migration that combines existing
  backfills with at least one typed compatibility operation.
- No arbitrary scripts, provider execution, or runtime migration apply logic are
  added to this repo.
