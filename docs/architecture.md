# Architecture

`greentic-sorla` is the canonical home for the SoRLa language, compiler-facing IR,
wizard surface, packaging model, and runtime-facing artifact contracts.

The repository is intentionally wizard-first. The supported product surface is:

- `greentic-sorla wizard --schema`
- `greentic-sorla wizard --answers <file>`

This repository owns:

- the SoRLa authoring language
- canonical IR and artifact contracts
- wizard schema and answers execution
- package and pack-ready metadata generation

This repository does not own provider implementations. Provider-specific code
lives in `greentic-sorla-providers`, which consumes abstract provider
requirements emitted from this repo.

SoRLa v0.2 is being shaped around practical requirements:

- event-native authoring
- Git-driven, deterministic artifacts
- provider-aware package metadata
- external-system-of-record friendly modeling
