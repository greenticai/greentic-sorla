# Review Questions For Proposed PR 2

These questions are adapted to the current implementation, which already emits package metadata and deterministic artifacts from both the CLI and `greentic-sorla-pack`.

## Boundary Clarification

- The repo does not currently build final pack or bundle archives, but it does generate `package-manifest.json`, `provider-requirements.json`, `package-manifest.cbor`, and a pack-oriented artifact set. Is the goal to remove only final assembly/archive behavior, or to remove these manifest-style assembly-adjacent outputs too?
- Should `crates/greentic-sorla-pack` be deleted, renamed later, or narrowed immediately to IR/intent emission only?
- Should `greentic-sorla-cli` stop writing `.greentic-sorla/generated/package-manifest.json` and `.greentic-sorla/generated/provider-requirements.json` in this PR, and if so what exact replacement files should it emit?
- Is `locale-manifest.json` still valid under the new architecture, or should it also be reframed as extension handoff metadata?

## gtc Integration Questions

- What exact `gtc` extension descriptor schema should `greentic-sorla` target? There is no local type or fixture for that contract in this repo today.
- What exact handoff artifact should `greentic-sorla` emit for `gtc`: extension answers, launcher handoff input, setup handoff input, start handoff input, or some subset of those?
- Should `greentic-sorla` own any descriptor generation at all, or only produce outputs consumable by a descriptor already owned elsewhere?
- Is the intention for `greentic-sorla` to remain invocable directly as a CLI binary while also serving as a `gtc` extension wizard binary, or should the direct path become dev-only?

## Current Code Hotspots That Need A Decision

- `crates/greentic-sorla-cli/src/lib.rs` currently writes generated files named around packages and provider requirements, and its CLI help says "authoring SoRLa packages." Should PR 2 rename that surface now or leave naming cleanup for PR 3?
- `crates/greentic-sorla-pack/src/lib.rs` still exposes `PackageManifest`, `scaffold_manifest`, `build_artifacts_from_yaml`, and emits `package-manifest.cbor`. Which of these concepts survive as abstract intent artifacts versus being removed outright?
- `docs/artifacts.md` currently says the artifact set is for "runtime packs and provider packs." Should the implementation continue emitting CBOR artifacts like `model.cbor` and `provider-contract.cbor`, or should artifact emission move behind a new extension-handoff boundary?
- Wizard schema and embedded i18n currently use package-first language throughout: "create package", "update package", "package bootstrap", "package version". Is wording churn required in this PR because it affects user-facing CLI/schema output, or is that deferred?

## Test And Migration Questions

- Which tests should enforce the new boundary? Current tests mainly assert deterministic artifact and manifest generation, not `gtc` handoff behavior.
- Do we need fixtures that mirror real `gtc` extension-launcher handoff documents before changing code, so the new integration is grounded in an actual contract?
- Should update flows continue to preserve legacy generated files if they already exist in a working directory, or may this PR stop producing them without backward-compatibility handling?
- Should examples under `crates/greentic-sorla-cli/examples/answers/` change now if the answer model stays stable but outputs change?
