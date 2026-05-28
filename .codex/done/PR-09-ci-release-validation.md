# PR-09 — Enforce embedded validation metadata in CI and release checks

## Repository

greenticai/greentic-sorla

## Objective

Ensure release artifacts and generated packs always include valid embedded SORX validation metadata before publishing binaries/crates.

## Required changes

### 1. Extend local check script

Update:

```text
ci/local_check.sh
```

Add steps that generate the validation-enabled fixture pack and run:

```bash
greentic-sorla pack doctor <fixture.gtpack>
greentic-sorla pack validation-inspect <fixture.gtpack>
```

Use `cargo run -p greentic-sorla -- ...` if the installed binary is not available.

Current implementation alignment: the package name is `greentic-sorla-cli`, but the binary invoked by docs/tests is `greentic-sorla`. Use whichever invocation is already used in `ci/local_check.sh` and keep it deterministic.

### 2. Extend GitHub Actions CI

Update `.github/workflows/ci.yml` to run the same validation pack generation and doctor checks.

Requirements:

- no external network
- no GHCR access
- no FoundationDB dependency
- deterministic fixture-only execution

### 3. Extend release workflow validation

Update release-binary/tag validation workflow so tag builds verify:

- validation schema command works
- fixture pack includes validation assets
- fixture pack doctor passes
- fixture pack inspect includes validation summary

This should happen before release assets are uploaded.

Current implementation alignment: this repo currently has `.github/workflows/release-binaries.yml` and no `publish.yml`. Do not recreate the deleted publish workflow for this PR.

### 4. Add failure messages

Make CI failures clear, e.g.:

```text
ERROR: generated .gtpack is missing assets/sorx/tests/test-manifest.json
ERROR: exposure policy default_visibility must not be public
ERROR: validation manifest references missing fixture assets/sorx/tests/data/foo.json
```

### 5. Tests and docs

Update README `CI and Releases` section:

- mention validation-enabled pack checks
- mention `pack doctor` now enforces SORX validation metadata

## Acceptance criteria

- `bash ci/local_check.sh` catches missing/invalid validation metadata.
- GitHub CI runs validation pack checks.
- Release workflow blocks if validation-enabled fixture pack is invalid.

## Non-goals

- Do not publish validation reports to GHCR.
- Do not call SORX.
- Do not require external services.
- Do not implement GHCR webhooks, deployment promotion, or public endpoint exposure in this repository.
