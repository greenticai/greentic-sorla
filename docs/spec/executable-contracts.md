# Executable Contracts

SoRLa executable contracts make e2e scenarios and downstream execution less
dependent on sidecar assumptions. The authoring YAML can now declare
relationships, migration backfills, and mutating agent operation emits in the
same model that produces canonical IR.

## Field Relationships

Record fields can declare a reference to another record field:

```yaml
- name: tenant_id
  type: string
  references:
    record: Tenant
    field: id
```

The parser rejects references to unknown records or unknown fields.

## Migration Backfills

Migrations can declare an idempotence key and structured backfills:

```yaml
migrations:
  - name: landlord-tenant-v2-fields
    compatibility: additive
    idempotence_key: landlord-tenant-v2-fields
    backfills:
      - record: Tenant
        field: preferred_contact_method
        default: null
```

The parser validates that each backfill references an existing record and field.
The landlord/tenant e2e applies these backfills from canonical IR and runs the
migration twice to prove the idempotence key is effective.

## Agent Operation Emits

Mutating agent endpoints can declare the event append they produce:

```yaml
agent_endpoints:
  - id: create_tenant
    title: Create tenant
    intent: Create a tenant record.
    inputs:
      - name: full_name
        type: string
    emits:
      event: TenantCreated
      stream: "landlord-tenant-sor/{landlord_id}"
      payload:
        id: "$generated.tenant_id"
        full_name: "$input.full_name"
```

Parser validation checks the event name, stream presence, and `$input.<name>`
payload references. `$generated.<name>` placeholders are permitted for executor
generated IDs and values.

## Pack Artifact

`greentic-sorla-pack` emits `executable-contract.json` with schema
`greentic.sorla.executable-contract.v1`. It contains:

- `package`: name, version, and IR hash
- `relationships`: record field references
- `migrations`: compatibility entries with backfills and idempotence keys
- `agent_operations`: endpoint IDs and emits declarations
- `operation_result_contract`: structured result/error shape for executors

Downstream consumers should verify the IR hash before using the artifact for
execution planning.

Operation results use schema `greentic.sorla.operation-result.v1` with:

- `endpoint_id`
- `status`: `ok`, `validation_error`, or `provider_error`
- `data`: structured success payload
- `errors`: validation errors with `path`, `code`, and `message`
- `provider_message`: safe provider-facing error text when applicable
