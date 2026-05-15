# SoRLa Library API

`greentic-sorla-lib` is the reusable implementation boundary for SoRLa
authoring, validation, preview-oriented output, inspection, and deterministic
`.gtpack` compatibility artifacts.

The `greentic-sorla` package still owns the installed binary, argument parsing,
localized help text, stdout/stderr rendering, and exit codes. Its library target
re-exports `greentic-sorla-lib` for compatibility with existing tests and
helper crates.

## Crate Boundaries

- `crates/greentic-sorla-cli`: binary wrapper and compatibility re-exports.
- `crates/greentic-sorla-lib`: public facade for reusable authoring and pack
  workflows.
- `crates/greentic-sorla-lang`: SoRLa AST, parser, and source validation.
- `crates/greentic-sorla-ir`: canonical IR, deterministic serialization, and
  hashing.
- `crates/greentic-sorla-pack`: handoff artifact generation, `.gtpack`
  compatibility output, doctor, inspect, and SORX validation metadata helpers.
- `crates/greentic-sorla-wizard`: wizard schema model adapters.

The facade intentionally reuses the existing `lang`, `ir`, and `pack` crates.
No separate `core` or `validate` crate is introduced yet because the current
crate boundaries already own those concepts.

## Reuse Rules

Library code must return typed values or diagnostics and must not require a CLI
subprocess. Callers such as Designer extensions should use the facade crate
directly for schema emission, answers handling, validation, preview generation,
pack generation, doctor, and inspect behavior.

The CLI maps library results to user-facing JSON and process exit codes. The
facade remains extension-first: it emits SoRLa source material, canonical IR,
handoff metadata, and legacy `.gtpack` compatibility artifacts; final runtime
or bundle assembly remains downstream.
