# Landlord/Tenant Pack Example

This directory contains the answers document for the landlord/tenant SoRLa
example pack used by downstream SORX testing.

Generate the pack from the repository root with:

```bash
cargo run -p greentic-sorla -- wizard \
  --answers examples/landlord-tenant/answers.json \
  --pack-out landlord-tenant-sor.gtpack
```

The command writes:

- `examples/landlord-tenant/sorla.yaml`
- `examples/landlord-tenant/.greentic-sorla/generated/`
- `examples/landlord-tenant/landlord-tenant-sor.gtpack`

Validate the generated pack with:

```bash
cargo run -p greentic-sorla -- pack doctor examples/landlord-tenant/landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack inspect examples/landlord-tenant/landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack validation-inspect examples/landlord-tenant/landlord-tenant-sor.gtpack
```
