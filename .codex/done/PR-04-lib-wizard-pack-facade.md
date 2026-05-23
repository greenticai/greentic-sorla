# PR-04 — Add missing apply facade over existing greentic-sorla-lib APIs

## Goal

Extend `greentic-sorla-lib` so downstream components can continue from `answers.json` to `sorla.yaml` and optionally `.gtpack` without shelling out to the CLI.

This is needed so the Sorla Designer component can call the library directly.

Repo reality check: most of this facade already exists. `greentic-sorla-lib` currently exposes:

- `schema_for_answers`
- `normalize_answers`
- `validate_model`
- `generate_preview`
- `build_gtpack_entries`
- `build_gtpack_bytes`
- `build_gtpack_file`
- `inspect_gtpack_bytes`
- `doctor_gtpack_bytes`

The missing public piece is the filesystem-oriented wizard apply facade. Today `apply_answers` and `ExecutionSummary` exist but are private inside `greentic-sorla-lib/src/lib.rs`.

## Important boundary

This is separate from the prompt engine.

```text
Prompt engine:
  outputs answers.json only

Library wizard/apply facade:
  answers.json -> sorla.yaml

Library pack facade:
  NormalizedSorlaModel/sorla.yaml -> .gtpack
```

## Suggested APIs

```rust
pub struct ApplyAnswersInput {
    pub answers: serde_json::Value,
    pub pack_out: Option<PathBuf>,
}

pub struct ApplyAnswersOutput {
    pub mode: String,
    pub output_dir: String,
    pub package_name: String,
    pub locale: String,
    pub written_files: Vec<String>,
    pub pack_path: Option<String>,
    pub preserved_user_content: bool,
}

pub struct PackFromAnswersInput {
    pub answers: serde_json::Value,
    pub output_path: PathBuf,
}

pub struct PackFromAnswersOutput {
    pub gtpack_path: PathBuf,
    pub summary: PackBuildResult,
}
```

Do not add a second answers-to-model or pack pipeline. Expose a stable wrapper around the existing private `apply_answers` path, and keep `normalize_answers` / `build_gtpack_*` as the in-memory and native packaging APIs. If a convenience `PackFromAnswersInput` is added, implement it by calling `normalize_answers` and the existing pack builder.

## Required behavior

- `apply_answers` validates and applies the same pipeline used by `greentic-sorla wizard --answers`.
- Pack generation uses the same deterministic path as `wizard --answers --pack-out`.
- Outputs are deterministic.
- Errors are structured enough for component UX.
- Preserve the existing generated-block behavior in `sorla.yaml` and the current `.greentic-sorla/generated` metadata layout.
- Keep the WASM-safe split: Designer/WASM callers should use `normalize_answers` plus `build_gtpack_entries`; native callers can use `build_gtpack_bytes` or `build_gtpack_file`.

## Acceptance criteria

- A consumer can call `greentic-sorla-lib` with `answers.json` and receive `sorla.yaml`.
- A consumer can call `greentic-sorla-lib` with `answers.json` and receive a `.gtpack`.
- CLI behavior remains unchanged.
- Existing deterministic generation remains unchanged.
- Unit/integration tests compare CLI and library outputs for the same fixture answers.
- Existing `docs/sorla-lib.md` stable-surface docs are updated rather than contradicted.
