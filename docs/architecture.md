# Architecture

`greentic-sorla` is the canonical home for the SoRLa language, compiler-facing
IR, and wizard-facing authoring flow.

It is not the owner of final Greentic pack generation, bundle generation, or
runtime assembly. Those responsibilities belong to `gtc`, which also owns
extension registry resolution, launcher handoff, setup handoff, and start
handoff.

The repository is intentionally wizard-first. The supported product surface is:

- `greentic-sorla wizard --schema`
- `greentic-sorla wizard --answers <file>`

For production composition, `gtc wizard --extensions ...` is the canonical
entrypoint. The standalone `greentic-sorla wizard` flow remains useful for
local development, schema iteration, fixtures, and extension debugging.

This repository owns:

- the SoRLa authoring language
- canonical IR and artifact contracts
- wizard schema and answers execution
- abstract intent metadata and extension-friendly outputs

This repository may generate:

- SoRLa source outputs
- canonical IR
- abstract capability and provider requirement metadata
- asset and component declarations
- handoff-ready metadata for `gtc`

This repository does not own provider implementations. Provider-specific code
lives in `greentic-sorla-providers`, which consumes abstract provider
requirements emitted from this repo.

This repository also does not own:

- final pack assembly
- final bundle assembly
- local bundle-builder orchestration
- extension registry resolution
- extension launcher ownership
- setup/start handoff ownership

SoRLa v0.2 is being shaped around practical requirements:

- event-native authoring
- Git-driven, deterministic artifacts
- provider-aware abstract metadata
- external-system-of-record friendly modeling
