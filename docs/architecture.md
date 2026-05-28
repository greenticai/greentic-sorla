# Architecture

`greentic-sorla` is the canonical home for the SoRLa language, compiler-facing
IR, and wizard-facing authoring flow.

It is not the owner of final Greentic pack generation, bundle generation, or
runtime assembly. Those responsibilities belong to `gtc`, which also owns
extension registry resolution, launcher handoff, setup handoff, and start
handoff.

The repository is intentionally wizard-first, with deterministic pack handoff as
the second public workflow. The supported product surface is:

- `greentic-sorla wizard --schema`
- `greentic-sorla wizard --answers <file>`
- `greentic-sorla wizard --answers <file> --pack-out <file.gtpack>`
- `greentic-sorla pack <file> --name <name> --version <version> --out <file.gtpack>`
- `greentic-sorla pack doctor <file.gtpack>`
- `greentic-sorla pack inspect <file.gtpack>`

The installed CLI is intentionally thin. Reusable behavior lives behind
`greentic-sorla-lib`, which coordinates the existing language, IR, pack, and
wizard crates. This lets Designer extensions and future `gtc` integrations use
the same validation and artifact paths without shelling out to the binary.

Ontology authoring is part of the local source and IR contract. SoRLa validates
ontology concepts, relationships, inheritance, record backing, sensitivity
markers, and policy hooks as provider-agnostic handoff metadata. Runtime graph
traversal, retrieval, public exposure, and concrete provider binding remain
downstream responsibilities.

For production composition, `gtc wizard --extensions ...` is the canonical
entrypoint. The standalone `greentic-sorla wizard` flow remains useful for
local development, schema iteration, fixtures, and extension debugging.

This repository owns:

- the SoRLa authoring language
- canonical IR and artifact contracts
- wizard schema and answers execution
- reusable library APIs for authoring, validation, inspection, and pack handoff
- package and pack-ready metadata generation
- deterministic agent endpoint handoff metadata
- deterministic SoRLa `.gtpack` handoff archives
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

For agent endpoints and `.gtpack` handoff, `greentic-sorla` owns authoring,
validation, canonical IR, and deterministic contract packaging. It does not
serve agent endpoints, proxy API calls, resolve credentials, own OAuth setup,
run databases, enforce runtime policy, or assemble final `.gtbundle` artifacts.
Those responsibilities stay with `greentic-sorx`, `gtc`, provider repositories,
and runtime components. The downstream validation and compatibility contract
lives in `docs/agent-endpoint-handoff-contract.md` and `docs/sorla-gtpack.md`.

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
- agent-facing business action contracts
