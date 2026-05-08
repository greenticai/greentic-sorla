# Landlord/Tenant FoundationDB E2E

This scenario lives in `greentic-sorla` because it validates SoRLa authoring,
IR lowering, packaging, schema evolution, and agent endpoint mapping.

It uses the FoundationDB provider from the sibling `greentic-sorla-providers`
workspace as an integration dependency. The current provider mode is the
provider repo's local/dev FoundationDB-compatible transactional backing; it does
not require a local external FoundationDB daemon.

Run the full scenario:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb
```

Run the smoke scenario:

```bash
cargo xtask e2e landlord-tenant --provider foundationdb --smoke
```

The fixtures model landlords, properties, units, tenants, tenancies, payments,
maintenance requests, v1-to-v2 schema evolution, and deterministic agent-style
operations against the same system of record.
