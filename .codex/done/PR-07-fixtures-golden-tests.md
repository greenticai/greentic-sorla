# PR-07 — Add fixture packs and golden tests for validation-enabled `.gtpack`

## Repository

greenticai/greentic-sorla

## Objective

Add end-to-end fixture coverage proving validation-enabled `.gtpack` output is deterministic, inspectable, and doctor-valid.

## Required changes

### 1. Add fixture answer files

Create or extend examples under:

```text
crates/greentic-sorla-cli/examples/answers/
```

Add at least:

```text
minimal_validation_pack.json
landlord_tenant_validation_pack.json
landlord_tenant_exported_candidate_pack.json
```

The existing landlord/tenant pack fixture should be extended or cloned to include endpoint export surfaces and provider requirements that exercise validation generation. Current SoRLa metadata has no route/public-alias concept, so avoid naming fixtures as if this repo publishes public endpoints.

### 2. Add expected manifest snapshots

Add golden snapshots under an appropriate test fixture directory:

```text
tests/fixtures/validation/minimal/test-manifest.json
tests/fixtures/validation/landlord-tenant/test-manifest.json
tests/fixtures/validation/landlord-tenant/exposure-policy.json
tests/fixtures/validation/landlord-tenant/compatibility.json
```

Use stable formatting.

### 3. Add deterministic output tests

Add tests that:

1. Run pack generation twice from the same fixture.
2. Compare archive bytes or compare lock metadata if archive byte comparison is already used elsewhere.
3. Assert validation, exposure, and compatibility assets are present.
4. Assert `pack doctor` passes.
5. Assert `pack inspect` includes summary fields.

### 4. Add negative fixture tests

Add tests for corrupted packs:

- validation manifest missing
- validation manifest invalid schema
- exposure policy default is public
- compatibility claims shared state even though current SoRLa metadata cannot support that claim
- test manifest references a missing fixture

### 5. Avoid brittle timestamps

Make sure fixture-generated outputs do not contain:

- timestamps
- absolute paths
- current username
- machine-specific directories
- environment variables

### 6. CI integration

Ensure tests are included in existing local check script, likely:

```bash
bash ci/local_check.sh
```

Do not add slow integration tests requiring external services.

## Acceptance criteria

- Validation-enabled packs have golden coverage.
- Existing CI/local checks cover the new behavior.
- Corrupt validation assets fail doctor checks.
- Fixture packs remain deterministic.

## Non-goals

- Do not require FoundationDB to run.
- Do not require GHCR/network access.
- Do not run SORX.
