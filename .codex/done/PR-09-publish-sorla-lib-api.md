# PR 09 — Publish stable `sorla-lib` API for Designer and tooling reuse

## Repository

`greenticai/greentic-sorla`

## Objective

Create a stable, documented, publishable Sorla library API that can be consumed by:

- `greentic-sorla` CLI
- Sorla Designer Extension
- tests
- future `gtc` integration
- future bundle/render pipelines

The goal is for Designer to generate validated Sorla models and eventually `.gtpack` artifacts without duplicating CLI logic.

## Deliverables

Create a public facade crate or clearly documented facade module on top of the refactor from PR 08.

Recommended crate name:

```text
greentic-sorla-lib
```

Alternative:

```text
greentic-sorla-authoring
```

The facade should re-export stable types from internal crates while hiding unstable internals. If PR 08 used `greentic-sorla-authoring` as the reusable boundary, either keep that as the facade crate or add `greentic-sorla-lib` as a thin facade that depends on it.

## Public API

Add stable public functions:

```rust
pub fn schema_for_answers() -> Result<serde_json::Value, SorlaError>;

pub fn normalize_answers(
    input: serde_json::Value,
    options: NormalizeOptions,
) -> Result<NormalizedSorlaModel, SorlaError>;

pub fn validate_model(
    model: &NormalizedSorlaModel,
    options: ValidateOptions,
) -> SorlaValidationReport;

pub fn generate_preview(
    model: &NormalizedSorlaModel,
    options: PreviewOptions,
) -> Result<SorlaPreview, SorlaError>;

pub fn build_gtpack_bytes(
    model: &NormalizedSorlaModel,
    options: PackBuildOptions,
) -> Result<PackBuildBytes, SorlaError>;

pub fn build_gtpack_file(
    model: &NormalizedSorlaModel,
    output_path: &Path,
    options: PackBuildOptions,
) -> Result<PackBuildResult, SorlaError>;

pub fn inspect_gtpack_bytes(bytes: &[u8]) -> Result<PackInspectResult, SorlaError>;

pub fn doctor_gtpack_bytes(bytes: &[u8]) -> SorlaValidationReport;
```

If bytes-based packaging is not yet available, this PR should introduce deterministic `build_gtpack_entries` / `PackEntry` APIs first and keep native ZIP byte generation behind a feature. Do not fake byte APIs by shelling out to the CLI.

## Important design

`build_gtpack_bytes` is important for Designer because a design extension may need to return an artifact directly without relying on a filesystem path. The implementation must stay deterministic and must not embed temp paths, clocks, usernames, or environment-derived values.

## Typed output structures

Add:

```rust
pub struct SorlaDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    pub suggestion: Option<String>,
}

pub struct SorlaPreview {
    pub summary: SorlaPreviewSummary,
    pub cards: Vec<SorlaPreviewCard>,
    pub graph: Option<SorlaPreviewGraph>,
}

pub struct SorlaPreviewCard {
    pub title: String,
    pub items: Vec<String>,
}

pub struct PackBuildBytes {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub metadata: PackBuildMetadata,
}
```

## Determinism

All public APIs must be deterministic for the same input/options.

## Versioning

Add docs explaining:

- which API is stable
- how breaking changes are versioned
- how CLI and Designer extension should pin dependency versions
- how `.gtpack` schema compatibility is handled

## Tests

Add tests for:

- schema emission
- answer normalization
- model validation
- preview generation
- gtpack byte generation
- gtpack byte generation stable hash
- inspect from bytes
- doctor from bytes
- CLI and library output equivalence for the same fixture

## Documentation

Add:

```text
docs/sorla-lib.md
```

Include example code:

```rust
let model = normalize_answers(input, NormalizeOptions::default())?;
let report = validate_model(&model, ValidateOptions::default());
if report.has_errors() {
    return Ok(report);
}
let pack = build_gtpack_bytes(&model, PackBuildOptions::default())?;
```

## Acceptance criteria

```bash
cargo test --all-features
cargo doc --no-deps --all-features
cargo package --allow-dirty
bash ci/local_check.sh
```

Use the actual facade crate name in package/build commands once selected. Do not package the opt-in provider-backed e2e crate.
