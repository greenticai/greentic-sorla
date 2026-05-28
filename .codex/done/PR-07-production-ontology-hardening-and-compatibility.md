# PR 07 — Production ontology hardening and compatibility

## Repository

`greenticai/greentic-sorla`

## Objective

Harden the ontology-driven SoRLa authoring and handoff contracts for production use.

This PR should not add new user-facing ontology features first. It should make the ontology, retrieval, validation, and handoff metadata added in PRs 01-06 safe, versioned, deterministic, testable, and operable within this repo.

This PR is scoped to `greentic-sorla` only. Do not add Sorx runtime validation, provider catalog execution, concrete provider compatibility checks, runtime policy decisions, audit runtime output, or cross-repo CI orchestration.

## Hardening areas

### 1. Schema versioning

All SoRLa-owned artifacts must have explicit schema names and documented compatibility behavior:

```text
greentic.sorla.ontology.v1
greentic.sorla.ontology.graph.v1
greentic.sorla.retrieval-bindings.v1
greentic.sorx.validation.v1
```

Only include schema names emitted or validated by this repository. Do not introduce Sorx runtime result schemas or provider catalog schemas here.

### 2. Compatibility rules

Define and test SoRLa-side compatibility behavior:

- known-compatible schema versions are accepted
- unknown major versions are rejected with stable errors
- unknown fields in SoRLa authoring YAML are rejected to match the current `#[serde(deny_unknown_fields)]` AST contract
- additive metadata in generated JSON artifacts is either explicitly modeled or rejected by doctor with a stable error
- doctor/inspect errors remain deterministic and machine-readable enough for CI

### 3. Determinism

Verify:

- sorted JSON output where this repo emits JSON
- canonical CBOR where this repo emits CBOR
- stable legacy `.gtpack` handoff output
- stable `pack inspect` and `pack validation-inspect` JSON output
- stable wizard schema output
- stable validation manifest ordering through top-level `promotion_requires`, suite IDs, and test IDs

### 4. Security

Add SoRLa-owned checks for:

- no secrets in generated artifacts
- no absolute paths or `..` path escapes in manifest-referenced assets
- no inline credential-like values in examples or generated templates
- PII/sensitivity metadata preserved into ontology artifacts
- public-candidate or agent-exported handoff metadata includes validation gates when ontology artifacts exist

Runtime policy enforcement, audit redaction, concrete public route exposure, and provider credential validation remain downstream responsibilities.

### 5. CI

Extend repo-local checks as needed:

```bash
bash ci/local_check.sh
```

Ensure this repo checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- ontology fixture doctor/inspect/validation-inspect
- deterministic repeated generation for the ontology handoff fixture
- schema commands for validation, exposure policy, compatibility, ontology, and retrieval bindings when those commands exist

### 6. Documentation

Create or update production docs:

```text
docs/ontology-production-readiness.md
docs/ontology-security.md
docs/ontology-compatibility.md
docs/sorla-gtpack.md
docs/sorx-gtpack-validation.md
```

Docs must preserve the extension-first ownership boundary: `greentic-sorla` produces source/IR/handoff metadata; `gtc`, Sorx, and provider repositories own runtime assembly and execution.

## Tests

Add tests for:

- accepted known schema versions
- rejected unknown major schema versions
- deterministic doctor error messages for ontology/retrieval/validation failures
- no secret-like strings in generated ontology/retrieval artifacts
- path traversal rejection in manifest-referenced ontology/retrieval assets
- sensitivity metadata preserved through authoring, IR, graph JSON, and inspect output
- public-candidate/exported endpoint handoff includes required validation gates when ontology exists
- private-only handoff can keep ontology validation recommended but not promotion-gating

## Acceptance criteria

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo run -p greentic-sorla -- wizard --schema
cargo run -p greentic-sorla -- wizard --answers examples/ontology-business/answers.json --pack-out /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack validation-inspect /tmp/ontology-business.gtpack
bash ci/local_check.sh
```

All checks must pass in this repository without requiring Sorx, provider repositories, external services, or cross-repo scripts.
