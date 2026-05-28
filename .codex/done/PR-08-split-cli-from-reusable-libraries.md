# PR 08 — Split `greentic-sorla` CLI from reusable libraries

## Repository

`greenticai/greentic-sorla`

## Objective

Refactor `greentic-sorla` so the CLI is only a thin shell over reusable, publishable library crates where that boundary is currently missing.

This is required because the future Sorla Designer Extension should reuse the exact same authoring, validation, IR, and `.gtpack` generation logic as the CLI. No duplicated validator, no separate schema generator, and no CLI-only code paths.

## Current direction

The repo already has separate crates such as:

- `greentic-sorla-cli`
- `greentic-sorla-lang`
- `greentic-sorla-ir`
- `greentic-sorla-pack`
- `greentic-sorla-wizard`

This PR should formalize the boundary and introduce a clearer public library API without duplicating the already-existing `lang`, `ir`, `pack`, and `wizard` crates.

## Target workspace shape

Keep existing crate names unless there is a concrete extraction need. Prefer moving CLI-only answer parsing/application code into one reusable authoring/facade crate before adding more crates.

Target responsibilities:

```text
crates/greentic-sorla-cli
  CLI only. Argument parsing, stdout/stderr, exit codes.

crates/greentic-sorla-authoring or crates/greentic-sorla-lib
  Answer model, wizard schema generation, answers normalization, generated
  YAML application, diagnostics, preview-oriented summaries, and prompt/draft
  input shapes. This crate reuses lang/ir/pack instead of redefining their
  domain types.

crates/greentic-sorla-ir
  Canonical IR, hashing, deterministic serialization.

crates/greentic-sorla-pack
  .gtpack generation, pack doctor helpers, inspect helpers, artifact manifest helpers.

crates/greentic-sorla-wizard
  Wizard-specific orchestration using authoring + validation libraries.
```

Do not add `greentic-sorla-core` or `greentic-sorla-validate` in this PR unless the implementation proves the existing `lang`/`ir`/`pack` crates cannot own the needed boundary. If those crates are deferred, document that in `docs/library-api.md`.

## Required design rules

1. `greentic-sorla-cli` must not contain business validation logic.
2. `greentic-sorla-cli` must call library APIs for schema generation, answers normalization, validation, pack building, doctor, inspect, and validation inspect.
3. Library crates must not print to stdout/stderr.
4. Library crates return typed results and diagnostics.
5. CLI maps library results to user-facing output and exit codes.
6. Existing CLI behavior should remain compatible.
7. Existing examples and docs should keep working.
8. Keep deterministic output unchanged unless a test fixture is intentionally updated.
9. Preserve the extension-first rule: this repo emits source, IR, handoff metadata, and legacy `.gtpack` compatibility artifacts; it does not own final runtime/bundle assembly.
10. Do not make the CLI depend on a future Designer extension crate.

## Public API sketch

Expose a top-level facade if useful:

```rust
pub struct SorlaBuildRequest {
    pub answers_json: serde_json::Value,
    pub output_dir: Option<PathBuf>,
    pub pack_out: Option<PathBuf>,
    pub options: SorlaBuildOptions,
}

pub struct SorlaBuildOutput {
    pub normalized_answers: serde_json::Value,
    pub diagnostics: Vec<SorlaDiagnostic>,
    pub artifacts: Vec<SorlaArtifact>,
}

pub fn emit_wizard_schema() -> Result<serde_json::Value, SorlaError>;
pub fn normalize_answers(input: serde_json::Value) -> Result<NormalizedSorlaModel, SorlaError>;
pub fn validate_model(model: &NormalizedSorlaModel) -> SorlaValidationReport;
pub fn build_gtpack(model: &NormalizedSorlaModel, options: PackBuildOptions) -> Result<PackBuildResult, SorlaError>;
pub fn inspect_gtpack(path: &Path) -> Result<PackInspectResult, SorlaError>;
pub fn doctor_gtpack(path: &Path) -> SorlaValidationReport;
```

Treat this as a design sketch. The concrete API should match current repo types and should avoid promising byte-oriented pack APIs until PR 09/10 introduces them.

## Tests

Add or update tests to prove:

- CLI `wizard --schema` calls library schema emission.
- CLI `wizard --answers` calls library normalization/validation.
- CLI `wizard --answers --pack-out` calls library pack build.
- CLI `pack doctor` calls library doctor.
- CLI `pack inspect` calls library inspect.
- Existing landlord/tenant fixtures still pass.
- Library functions can be tested without running CLI subprocesses.

## Documentation

Update:

- `README.md`
- `docs/architecture.md`
- `docs/wizard.md`
- `docs/sorla-gtpack.md`

Add a new doc:

```text
docs/library-api.md
```

Describe which crates are intended for reuse by Designer extensions and which are CLI-only.

## Acceptance criteria

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
bash ci/local_check.sh

cargo run -p greentic-sorla -- wizard --schema
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out /tmp/landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/landlord-tenant-sor.gtpack
```
