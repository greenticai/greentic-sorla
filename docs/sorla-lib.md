# `greentic-sorla-lib`

`greentic-sorla-lib` is the stable facade for tools that need SoRLa behavior
without spawning the `greentic-sorla` binary.

```rust
let model = greentic_sorla_lib::normalize_answers(
    input,
    greentic_sorla_lib::NormalizeOptions::default(),
)?;
let report = greentic_sorla_lib::validate_model(
    &model,
    greentic_sorla_lib::ValidateOptions::default(),
);
if report.has_errors() {
    return Ok(report);
}
let pack = greentic_sorla_lib::build_gtpack_bytes(
    &model,
    greentic_sorla_lib::PackBuildOptions::default(),
)?;
```

## Stable Surface

- `schema_for_answers`
- `normalize_answers`
- `validate_model`
- `generate_preview`
- `build_gtpack_entries`
- `build_gtpack_bytes`
- `build_gtpack_file`
- `inspect_gtpack_bytes`
- `doctor_gtpack_bytes`

The facade returns typed diagnostics and preview structures. It does not shell
out to the CLI. The CLI uses the same implementation and only handles argument
parsing, rendering, and exit codes.

## WASM Profile

The facade crate defines a `wasm` feature for Designer extension builds:

```bash
cargo build -p greentic-sorla-lib \
  --target wasm32-wasip2 \
  --no-default-features \
  --features wasm
```

`schema_for_answers`, `normalize_answers`, `validate_model`, `generate_preview`,
and `build_gtpack_entries` are the preferred WASM-safe subset. Native callers
can also use `build_gtpack_bytes` and `build_gtpack_file` when they need a ZIP
archive directly. If a host cannot provide filesystem-backed ZIP output,
Designer extensions should return deterministic `PackEntry` values and let the
host package them.

The Designer adapter's `generate_gtpack` tool uses `build_gtpack_entries` in
WASM mode. Native hosts that need actual `.gtpack` bytes can use
`build_gtpack_bytes` from this crate with the default `pack-zip` feature.

Designer-specific prompt fragments and knowledge examples live in
`greentic-sorla-designer-extension`, but example entries intentionally use the
same answers/model shape accepted by this facade. That keeps LLM guidance,
validation, previews, and artifact planning on one contract.

## Versioning

Breaking facade changes should be made with crate version bumps and documented
migration notes. Designer extensions and external tools should pin compatible
`greentic-sorla-lib` versions. `.gtpack` compatibility remains governed by the
pack manifest schemas and deterministic doctor/inspect checks documented in
`docs/sorla-gtpack.md`.
