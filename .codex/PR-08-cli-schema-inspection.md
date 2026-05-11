# PR-08 — Add CLI commands for SORX validation schema and test manifest inspection

## Repository

greenticai/greentic-sorla

## Objective

Add developer-friendly CLI commands to inspect validation schemas and embedded validation metadata in `.gtpack` files.

## Required changes

### 1. Add schema command

Add a command under the existing CLI style:

```bash
greentic-sorla pack validation-schema
```

Output JSON schema for:

```text
greentic.sorx.validation.v1
```

Optional additional flags:

```bash
greentic-sorla pack validation-schema --exposure-policy
greentic-sorla pack validation-schema --compatibility
```

If the CLI parser prefers subcommands, use:

```bash
greentic-sorla pack schema validation
greentic-sorla pack schema exposure-policy
greentic-sorla pack schema compatibility
```

Pick the pattern that best matches existing CLI design.

Current implementation alignment: the `pack` command currently has only `doctor` and `inspect` subcommands, plus the build mode when no subcommand is supplied. Prefer adding a `schema` subcommand group (`greentic-sorla pack schema validation`) if that keeps clap parsing unambiguous.

### 2. Add validation inspect command

Add:

```bash
greentic-sorla pack validation-inspect my.gtpack
```

Output a concise JSON summary:

```json
{
  "schema": "greentic.sorx.validation.v1",
  "package": {
    "name": "landlord-tenant-sor",
    "version": "0.1.0"
  },
  "suites": [
    { "id": "smoke", "required": true, "test_count": 3 }
  ],
  "promotion_requires": ["smoke", "contract", "security", "provider"],
  "exposure": {
    "default_visibility": "private",
    "public_candidate_endpoints": 2
  },
  "compatibility": {
    "state_mode": "isolated_required",
    "provider_requirement_count": 1
  }
}
```

### 3. Add validation doctor command alias

If useful, add:

```bash
greentic-sorla pack validation-doctor my.gtpack
```

This can call the same underlying doctor checks but focus output on validation assets.

Keep this as an alias only. The primary enforcement path should remain `greentic-sorla pack doctor <file.gtpack>` so CI and users have one canonical check.

### 4. Documentation

Update README and `docs/sorx-gtpack-validation.md` with examples:

```bash
greentic-sorla pack validation-schema > sorx-validation.schema.json
greentic-sorla pack validation-inspect landlord-tenant-sor.gtpack
greentic-sorla pack doctor landlord-tenant-sor.gtpack
```

### 5. Tests

Add CLI tests for:

- schema command emits valid JSON
- validation-inspect works for fixture pack
- validation-doctor fails on corrupted fixture pack
- output is deterministic enough for snapshots

## Acceptance criteria

- Developers and SORX implementers can discover schemas from CLI.
- Pack validation metadata can be inspected without unpacking the archive manually.
- Commands fit existing CLI conventions.

## Non-goals

- Do not execute embedded tests.
- Do not perform network calls.
