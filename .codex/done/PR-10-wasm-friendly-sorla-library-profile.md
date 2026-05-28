# PR 10 — Add WASM-friendly Sorla library profile for Designer extensions

## Repository

`greenticai/greentic-sorla`

## Objective

Make the reusable Sorla library APIs suitable for a future Designer DesignExtension compiled to `wasm32-wasip2`.

The extension should be able to validate, normalize, preview, and ideally generate `.gtpack` bytes without process spawning or native-only dependencies.

## Required profiles

Add features:

```toml
[features]
default = ["std", "pack-zip"]
std = []
wasm = []
pack-zip = []
cli = []
```

The exact names can vary, but the separation must be clear and must be applied to the facade crate selected in PR 09, not to every crate indiscriminately.

## WASM-friendly API subset

Must work on `wasm32-wasip2`:

```rust
schema_for_answers()
normalize_answers(...)
validate_model(...)
generate_preview(...)
build_gtpack_bytes(...) // if dependencies allow
```

If `.gtpack` ZIP writing cannot be made WASM-compatible immediately, split the API:

```rust
build_pack_plan(...)
build_pack_entries(...)
```

Then host/native packaging can turn entries into a `.gtpack`.

Prefer getting `schema_for_answers`, `normalize_answers`, `validate_model`, `generate_preview`, and deterministic pack-entry planning working on `wasm32-wasip2` first. Native ZIP emission can remain a separate feature if the ZIP dependency or filesystem requirements make WASM support risky.

## Avoid in WASM path

Do not require:

- subprocess execution
- absolute filesystem paths
- native-only compression backends
- network access
- OS-specific path normalization
- environment variables
- system clock for deterministic artifacts

## Pack byte generation

Prefer deterministic in-memory packaging:

```rust
pub fn build_gtpack_bytes(model: &NormalizedSorlaModel, options: PackBuildOptions)
    -> Result<PackBuildBytes, SorlaError>;
```

If ZIP dependency is compatible, use it. If not, create:

```rust
pub fn build_gtpack_entries(...)
    -> Result<Vec<PackEntry>, SorlaError>;
```

and keep native byte generation behind a feature.

## CI checks

Add CI/local check command for WASM target when the target is installed, and make the skip behavior explicit when it is not:

```bash
cargo build -p <facade-crate> --target wasm32-wasip2 --no-default-features --features wasm
```

Use the actual crate name. Do not require network installation during CI/local check; document `rustup target add wasm32-wasip2` as a prerequisite or add a guarded check.

## Tests

Add tests for:

- no filesystem needed for normalization/validation/preview
- stable preview output
- stable pack entries
- WASM target build
- native and WASM-compatible pack plans produce same entry list

## Documentation

Update:

```text
docs/sorla-lib.md
docs/designer-extension.md
```

Explain which APIs are safe for Designer extensions and which require native host packaging.

## Acceptance criteria

```bash
rustup target add wasm32-wasip2
cargo build -p <facade-crate> --target wasm32-wasip2 --no-default-features --features wasm
cargo test --all-features
bash ci/local_check.sh
```
