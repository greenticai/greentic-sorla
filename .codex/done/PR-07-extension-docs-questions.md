# Review Questions For Proposed PR 1

These questions are adapted to the current `greentic-sorla` repo, which still documents and implements package-manifest and artifact emission in multiple places.

## Scope Clarification

- Should this docs-only PR also update `docs/packaging.md` and `docs/artifacts.md`? They currently still present `greentic-sorla` as producing `package-manifest*` and runtime-pack-facing artifacts, so leaving them untouched would keep the old architecture in the repo.
- Where should the new Codex-facing architecture rule live? The repo already has `.codex/global_rules.md`; decide whether the extension-first rule belongs there, in a new `.codex` doc, or in both places.
- Should `README.md` keep advertising `greentic-sorla wizard --schema` and `greentic-sorla wizard --answers ...` as the primary supported product surface, or should production usage be reframed around `gtc wizard --extensions ...` with `greentic-sorla` described as a local/dev entrypoint only?
- Is `greentic-sorla wizard` still intended to be a supported standalone workflow for local schema work, or should the docs mark it as secondary to the `gtc` extension flow?

## Architecture Contract Questions

- What is the exact `gtc` extension contract this repo should document? The current repo does not contain extension descriptor or handoff types, so the doc needs a precise upstream contract instead of a paraphrase.
- Which `gtc` concepts should be named explicitly in the new docs: descriptor resolution, launcher handoff, setup handoff, start handoff, emitted handoff documents, and registry lookup?
- Should the new docs state that `greentic-sorla` produces only source/IR/intent artifacts, or may it still emit abstract metadata files under `.greentic-sorla/generated/` as long as they are not presented as final assembly outputs?

## Repo-Specific Terminology Questions

- How should current pack-oriented crate naming be described in docs before code changes land? `crates/greentic-sorla-pack` currently exists and is described in `README.md`, `docs/product-shape.md`, and `.codex/repo_overview.md`.
- Should terms like `package-manifest`, `provider-requirements`, and `gtpack-ready` be removed immediately from docs, or only reframed as temporary legacy terminology pending a later rename/migration PR?
- Should `docs/wizard.md` stop referring to generated output as "package bootstrap", "package update", and "gtpack-ready metadata", or is that rename deferred to PR 3?

## Consistency Questions

- Should `.codex/repo_overview.md` be updated in the same PR? It currently says the repo owns the packaging model and that `greentic-sorla-pack` emits package manifests and runtime-facing artifacts.
- Are crate READMEs in scope for PR 1? `crates/greentic-sorla-pack/README.md` currently says "Package-manifest and artifact-facing scaffolding for SoRLa."
- Should the new extension doc cite specific `gtc` CLI flags and file names, or keep the explanation intentionally high-level to avoid drifting from `gtc`?
