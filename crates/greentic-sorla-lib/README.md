# greentic-sorla-lib

`greentic-sorla-lib` is the reusable facade for SoRLa authoring, validation,
inspection, and deterministic `.gtpack` compatibility artifacts.

The `greentic-sorla` CLI is intentionally a thin wrapper over this crate so
Designer extensions and tests can use the same implementation without invoking
the binary.
