# PR-04 — Include validation assets in `.gtpack` and extend `pack doctor`

## Repository

greenticai/greentic-sorla

## Objective

Include generated SORX validation assets inside deterministic `.gtpack` archives and extend `greentic-sorla pack doctor` to validate their structure.

## Required changes

### 1. Include validation assets in pack assembly

Update pack assembly to include:

```text
assets/sorx/tests/test-manifest.json
```

and any referenced files under:

```text
assets/sorx/tests/**
```

The pack currently includes SORX startup assets such as start schema, questions, runtime template, and provider bindings template. Add validation assets alongside those.

### 2. Update `pack.cbor` extension references

Extend the existing SORX-compatible extension metadata in `pack.cbor`.

Current implementation alignment: `pack.cbor` uses `SorlaPackManifest { extension: serde_json::Value }`, and that JSON value currently has this shape:

```json
{
  "extension": "greentic.sorx.runtime.v1",
  "sorla": {},
  "sorx": {}
}
```

Do not assume a list of extension records exists today.

Add the validation manifest as an additive key under the existing `sorx` object, for example:

```json
{
  "extension": "greentic.sorx.runtime.v1",
  "sorx": {
    "validation_manifest": "assets/sorx/tests/test-manifest.json"
  }
}
```

If a first-class extension registry is introduced later, keep this change backwards-compatible with current `greentic.sorx.runtime.v1` consumers.

Do not break existing `greentic.sorx.runtime.v1` consumers. This must be additive.

### 3. Update lock metadata

Ensure `pack.lock.cbor` includes validation assets in deterministic size/hash metadata.

Same input must produce byte-identical `.gtpack`.

### 4. Extend pack doctor

Update `greentic-sorla pack doctor <file.gtpack>` to check:

- `assets/sorx/tests/test-manifest.json` exists when the manifest references `sorx.validation_manifest`
- validation manifest parses as JSON
- schema equals `greentic.sorx.validation.v1`
- static validation passes
- all referenced input/data files exist
- no referenced path escapes `assets/sorx/tests/`
- validation assets are included in lock metadata
- validation manifest package name/version matches pack/package identity

### 5. Extend pack inspect

Update `greentic-sorla pack inspect <file.gtpack>` output to include a validation summary:

```json
{
  "validation": {
    "schema": "greentic.sorx.validation.v1",
    "suite_count": 4,
    "test_count": 12,
    "promotion_requires": ["smoke", "contract", "security", "provider"]
  }
}
```

Use existing inspect output style.

### 6. Tests

Add integration tests that:

- build a pack with validation assets
- run pack doctor successfully
- inspect shows validation summary
- corrupt validation schema causes doctor failure
- remove referenced fixture causes doctor failure
- generated `.gtpack` is deterministic across two runs

## Acceptance criteria

- Generated `.gtpack` contains validation manifest.
- `pack.cbor` references validation metadata additively without replacing `greentic.sorx.runtime.v1`.
- `pack doctor` validates embedded test manifest.
- `pack inspect` summarizes validation.
- Determinism tests pass.

## Non-goals

- Do not execute validation tests.
- Do not contact providers.
- Do not expose endpoints.
