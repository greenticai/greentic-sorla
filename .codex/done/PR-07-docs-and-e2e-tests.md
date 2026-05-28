# PR-07 — Add docs and end-to-end tests for prompt authoring

## Goal

Document and test the full prompt-to-answers workflow across CLI, library, and Designer component boundaries.

## Docs

Add:

```text
docs/prompt-authoring.md
```

Also update the existing `docs/sorla-lib.md`, `docs/library-api.md`, and README sections that already describe the stable library facade and CLI boundary.

Cover:

- why `prompt` is interactive
- why `prompt` outputs only `answers.json`
- why CLI and Sorla Designer component are frontends
- why `greentic-sorla-lib` is the shared engine
- how LLM provider/capability resolution works
- CLI usage
- SDK usage
- Designer/WebChat/Teams integration
- session persistence
- how components can use `greentic-sorla-lib` to continue from `answers.json` to `sorla.yaml` or `.gtpack`
- deterministic boundary with the wizard pipeline
- the existing split between WASM-safe `build_gtpack_entries` and native ZIP APIs (`build_gtpack_bytes` / `build_gtpack_file`)
- the fact that provider implementations live outside this repository and the LLM capability contract here is provider-agnostic

## README update

Update README with a short section:

```text
Prompt authoring
```

Explain:

```text
greentic-sorla prompt -> answers.json
greentic-sorla wizard --answers answers.json -> sorla.yaml / optional .gtpack
```

Keep the README aligned with the current product boundary: `gtc` owns final production composition, while `greentic-sorla` owns source outputs, canonical IR, and handoff metadata.

## E2E tests

Add an e2e test using fake LLM responses.

Scenario:

```text
Business: landlord/tenant property management
Answers:
- lease can have multiple tenants
- liability is joint
- payments are immutable
- maintenance requests use suppliers
- supplier work requires approval
```

Expected:

- prompt session completes
- generated `answers.json` validates
- `wizard --answers` accepts generated answers
- generated `sorla.yaml` is deterministic
- optional pack generation passes existing doctor checks if pack feature is available
- library validation uses `normalize_answers` and `validate_model`
- native pack tests use `build_gtpack_file` / CLI `--pack-out`; WASM-facing tests use `build_gtpack_entries`

## Regression tests

Ensure:

- Prompt does not generate `sorla.yaml` directly.
- Prompt does not generate `.gtpack` directly.
- Prompt does not generate component `src/`, `assets/`, or `build-answers.json`.
- CLI and library output paths are consistent.
- The prompt docs do not imply the prompt engine replaces the existing deterministic Designer adapter tools.

## Acceptance criteria

- Docs clearly explain architecture and boundaries.
- E2E test covers prompt -> answers -> wizard apply.
- Fake LLM tests are deterministic.
- CI/local checks pass.
