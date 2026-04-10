# Repository Overview

## 0. Milestone Status

| Milestone | Title | Status | Notes |
| --- | --- | --- | --- |
| PR-01 | Wizard-first repo scaffold | verified | Five-crate workspace, wizard-first CLI help surface, product docs, and local checks are in place. |
| PR-02 | SoRLa v0.2 language semantics | verified | v0.2 AST, YAML parser, additive v0.1 compatibility warnings, tests, and spec notes are implemented. |
| PR-03 | Canonical IR and artifact generation | verified | Deterministic IR lowering, canonical hashing, artifact emission, docs, and golden tests are implemented. |
| PR-04 | Wizard schema generation | verified | `wizard --schema` now emits a deterministic create/update schema with i18n keys, tests, and docs. |
| PR-05 | Answers-driven wizard execution | verified | `wizard --answers` now validates, creates, updates, preserves user content outside generated blocks, and writes deterministic artifacts. |
| PR-06 | i18n-ready wizard and gtpack-ready packaging | verified | Wizard schema/answers now carry locale fallback semantics and generate gtpack-ready package, provider, and locale metadata manifests. |

## 1. High-Level Purpose

`greentic-sorla` is the wizard-first home for the SoRLa language, compiler-facing IR, packaging model, and guided authoring workflow. The repository is intended to produce provider-agnostic package metadata and runtime-facing artifacts rather than provider implementations themselves.

The current implementation now includes the verified PR-01 scaffold, PR-02 language semantics, the PR-03 canonical IR/artifact slice, the PR-04 wizard schema contract, and a working PR-05 answers execution path. The CLI can now create or update a minimal SoRLa package layout deterministically and refresh wizard-owned artifacts while preserving user-authored content outside generated regions.

PR-06 is now implemented in the active CLI path: locale selection/fallback and generated package metadata are part of the wizard-owned output under `.greentic-sorla/generated/`.

The wizard entrypoint now also supports an interactive local mode backed by `greentic-qa-lib` when `--answers` is not provided, while still routing the collected answers through the same deterministic answers application pipeline.

## 2. Main Components and Functionality

- Component: workspace root
  - **Path:** `Cargo.toml`
  - **Role:** Virtual workspace and shared metadata definition.
  - **Key functionality:**
    - Defines the five-crate SoRLa workspace layout.
    - Shares package metadata and common dependencies across crates.
  - **Key dependencies / integration points:** consumed by cargo metadata, CI, packaging, and publish workflows.

- Component: `crates/greentic-sorla-cli`
  - **Path:** `crates/greentic-sorla-cli`
  - **Role:** Public CLI entrypoint.
  - **Key functionality:**
    - Exposes a wizard-first CLI surface centered on `greentic-sorla wizard --schema` and `greentic-sorla wizard --answers <file>`.
    - Keeps internal helper commands hidden from normal help output.
    - Emits a deterministic create/update wizard schema with stable section/question IDs, defaults, visibility rules, and artifact preferences.
    - Runs an interactive `greentic-qa-lib` frontend when `wizard` is invoked without `--answers`, then converts those answers into the existing `AnswersDocument` flow.
    - Validates answers documents, resolves create/update defaults including locale fallback, writes `sorla.yaml` with generated-region ownership boundaries, and syncs wizard-owned generated artifacts under `.greentic-sorla/generated/`.
    - Generates `package-manifest.json`, `provider-requirements.json`, and `locale-manifest.json` with abstract provider category declarations and locale metadata intended for future gtpack/provider-pack binding.
    - Includes placeholder perf/concurrency harness files.
  - **Key dependencies / integration points:** intentionally stays self-contained for schema emission so the publishable CLI package can still pass crates.io dry-run checks.

- Component: `crates/greentic-sorla-lang`
  - **Path:** `crates/greentic-sorla-lang`
  - **Role:** Language-facing AST and parser crate.
  - **Key functionality:**
    - Defines v0.2 package AST nodes for records, events, projections, migrations, provider requirements, and external references.
    - Parses YAML-authored packages using parser-validated `source: native|external|hybrid`.
    - Applies additive v0.1 compatibility by defaulting omitted `source` to `native` and surfacing warnings.
    - Enforces field-level authority rules for hybrid records and requires `external_ref` for external/hybrid sources.

- Component: `crates/greentic-sorla-ir`
  - **Path:** `crates/greentic-sorla-ir`
  - **Role:** Canonical IR and lowering crate.
  - **Key functionality:**
    - Lowers parsed SoRLa packages into a deterministic, versioned IR.
    - Separates business records, events, projections, compatibility data, external sources, and provider contract requirements.
    - Provides canonical CBOR serialization, inspectable JSON rendering, and hash derivation from canonical serialized form.

- Component: `crates/greentic-sorla-pack`
  - **Path:** `crates/greentic-sorla-pack`
  - **Role:** Package and artifact emission crate.
  - **Key functionality:**
    - Builds deterministic artifact sets from YAML package input.
    - Emits a provider-agnostic package manifest plus split CBOR artifacts such as `model.cbor`, `events.cbor`, `projections.cbor`, `external-sources.cbor`, and `provider-contract.cbor`.
    - Produces inspectable JSON and `agent-tools.json` views for tests and downstream tooling.

- Component: `crates/greentic-sorla-wizard`
  - **Path:** `crates/greentic-sorla-wizard`
  - **Role:** Wizard schema model crate.
  - **Key functionality:**
    - Defines a richer deterministic schema model covering create/update flows, provider requirements, external source declarations, event/projection defaults, compatibility choices, and output preferences.
    - Now derives its section/question schema from the public CLI crate instead of maintaining a second handwritten schema definition.
    - Includes tests for deterministic output and create/update coverage.
    - Still is not the runtime source for the publishable CLI crate because that would currently break crates.io dry-run packaging for the CLI.

- Component: `ci/local_check.sh`
  - **Path:** `ci/local_check.sh`
  - **Role:** Deterministic local quality gate.
  - **Key functionality:**
    - Runs `cargo fmt`, `cargo clippy`, `cargo test`, `cargo build`, `cargo doc`.
    - Validates required metadata for each publishable crate discovered via `cargo metadata`.
    - Runs `cargo package` and `cargo publish --dry-run` for each publishable crate.
    - Optionally runs i18n `status`/`validate` checks when translator tool is available.
  - **Key dependencies / integration points:** used by CI/release jobs and local development.

- Component: `.github/workflows/ci.yml`
  - **Path:** `.github/workflows/ci.yml`
  - **Role:** Main CI pipeline.
  - **Key functionality:** lint/test/package jobs for PRs and pushes with concurrency cancellation.
  - **Key dependencies / integration points:** wraps local checks and reusable rust workflow.

- Component: `.github/workflows/_reusable_rust.yml`
  - **Path:** `.github/workflows/_reusable_rust.yml`
  - **Role:** Shared GitHub Actions step definitions for Rust checks.
  - **Key functionality:** installs toolchain, runs formatting, clippy, tests, build, and docs.

- Component: `.github/workflows/publish.yml`
  - **Path:** `.github/workflows/publish.yml`
  - **Role:** Release workflow.
  - **Key functionality:** verifies `v<version>` tag alignment, runs verification checks, runs publish dry-run then real publish with retries, uses `CARGO_REGISTRY_TOKEN`.

- Component: `.github/workflows/perf.yml`
  - **Path:** `.github/workflows/perf.yml`
  - **Role:** lightweight PR/perf smoke check.
  - **Key functionality:** runs tests and benchmark smoke command in CI.

- Component: `.github/workflows/nightly-coverage.yml`
  - **Path:** `.github/workflows/nightly-coverage.yml`
  - **Role:** nightly coverage gate.
  - **Key functionality:** installs needed tooling (via `cargo-binstall`), runs `greentic-dev coverage`, enforces `coverage-policy.json`.

- Component: `benches/perf.rs`
  - **Path:** `crates/greentic-sorla-cli/benches/perf.rs`
  - **Role:** criterion benchmark harness.
  - **Key functionality:** placeholder benchmark now targets wizard schema generation.

- Component: `tests/perf_scaling.rs`
  - **Path:** `crates/greentic-sorla-cli/tests/perf_scaling.rs`
  - **Role:** concurrency scaling guardrail.
  - **Key functionality:** placeholder workload targets wizard schema generation across thread counts.

- Component: `tests/perf_timeout.rs`
  - **Path:** `crates/greentic-sorla-cli/tests/perf_timeout.rs`
  - **Role:** timing guardrail test.
  - **Key functionality:** placeholder timeout workload targets wizard schema generation.

- Component: `docs/architecture.md`, `docs/product-shape.md`
  - **Path:** `docs/`
  - **Role:** Architectural and product-shape documentation.
  - **Key functionality:** documents wizard-first UX, crate boundaries, and the rule that providers live in `greentic-sorla-providers`.

- Component: `docs/spec/v0.2.md`
  - **Path:** `docs/spec/v0.2.md`
  - **Role:** SoRLa v0.2 language notes.
  - **Key functionality:** documents `source`, `external_ref`, field-level authority for hybrid records, events, projections, provider requirements, and compatibility-oriented migrations.

- Component: `docs/artifacts.md`
  - **Path:** `docs/artifacts.md`
  - **Role:** Artifact contract documentation.
  - **Key functionality:** documents canonical ordering/hash rules and the current emitted artifact set.

- Component: `docs/wizard.md`
  - **Path:** `docs/wizard.md`
  - **Role:** Wizard schema documentation.
  - **Key functionality:** documents create/update flows, ownership/update rules, answer document expectations, locale selection/fallback, and the current schema/i18n contract.

- Component: `docs/packaging.md`
  - **Path:** `docs/packaging.md`
  - **Role:** Generated package metadata documentation.
  - **Key functionality:** documents the new generated package manifest, provider requirements manifest, locale manifest, and the rule that provider bindings stay abstract in `greentic-sorla`.

- Component: `crates/greentic-sorla-cli/examples/answers`
  - **Path:** `crates/greentic-sorla-cli/examples/answers`
  - **Role:** Sample answers documents.
  - **Key functionality:** provides minimal create and update examples for the deterministic wizard execution path.

- Component: `crates/greentic-sorla-pack/tests/golden`
  - **Path:** `crates/greentic-sorla-pack/tests/golden`
  - **Role:** Golden fixture coverage for PR-03.
  - **Key functionality:** fixture YAML and expected inspect JSON verify deterministic lowering and artifact generation.

- Component: `tools/i18n.sh`
  - **Path:** `tools/i18n.sh`
  - **Role:** repository i18n helper.
  - **Key functionality:** commands for `translate`, `validate`, `status`, and `all`; scans repo `i18n/en.json` sources and translates them in 200-item batches by default.
  - **Key dependencies / integration points:** reads `i18n/locales.json`, prefers `greentic0i18n-translator`, and falls back to `greentic-i18n-translator` when needed.

- Component: `i18n/en.json`, `i18n/locales.json`, `i18n/*.json`
  - **Path:** `i18n/`
  - **Role:** locale source and translations.
  - **Key functionality:** English source includes a small live key set; all listed locales currently contain fallback English copies and are structurally complete.

- Component: `coverage-policy.json`
  - **Path:** `coverage-policy.json`
  - **Role:** policy source for nightly coverage gate.
  - **Key functionality:** used by `nightly-coverage.yml`.

- Component: `.codex/global_rules.md`
  - **Path:** `.codex/global_rules.md`
  - **Role:** repository operating instructions.
  - **Key functionality:** enforces pre/post PR summary and required local check behavior.

- Component: `LICENSE`
  - **Path:** `LICENSE`
  - **Role:** repository license file.
  - **Key functionality:** establishes allowed distribution terms for crates.

## 3. Work In Progress, TODOs, and Stubs

- Location: `benches/perf.rs`
  - **Status:** TODO / partial
  - **Short description:** scaffold benchmark exists in `crates/greentic-sorla-cli/benches/perf.rs` but still needs a real hot-path workload.

- Location: `tests/perf_scaling.rs`
  - **Status:** TODO / partial
  - **Short description:** placeholder scaling workload in `crates/greentic-sorla-cli/tests/perf_scaling.rs` should be replaced with deterministic concurrency-critical operations.

- Location: `tests/perf_timeout.rs`
  - **Status:** TODO / partial
  - **Short description:** timeout workload in `crates/greentic-sorla-cli/tests/perf_timeout.rs` is still scaffold-only.

- Location: `crates/greentic-sorla-wizard`
  - **Status:** implemented
  - **Short description:** PR-04 landed a real schema model and tests, but CLI/runtime consumption is still decoupled for packaging reasons.

- Location: `crates/greentic-sorla-cli`
  - **Status:** implemented
  - **Short description:** PR-05 landed a working answers execution path with validation, create/update flows, generated block ownership, and idempotent rerun coverage.

- Location: `crates/greentic-sorla-lang`
  - **Status:** implemented
  - **Short description:** PR-02 landed concrete AST and parser support, but IR lowering and richer fixture coverage still belong to later milestones.

- Location: `crates/greentic-sorla-ir`, `crates/greentic-sorla-pack`
  - **Status:** implemented
  - **Short description:** PR-03 landed deterministic lowering, artifact emission, canonical hashing, and golden fixture coverage.

- Location: `i18n/*.json`
  - **Status:** partial
  - **Short description:** locale files are present and keyed, but are currently English fallback copies and need real translation content beyond the reserved core namespace.

- Location: `crates/greentic-sorla-cli/src/lib.rs`
  - **Status:** implemented
  - **Short description:** interactive wizard mode now exists and is driven by `greentic-qa-lib`, but it currently asks only the core subset of questions needed to feed the stable answers pipeline.

- Location: `crates/greentic-sorla-cli/src/lib.rs`, `crates/greentic-sorla-wizard/src/lib.rs`
  - **Status:** partial / reduced duplication
  - **Short description:** the CLI crate remains the canonical schema source and the wizard crate now maps from it, but type/model duplication still exists because the publishable CLI package must stay self-contained.

## 4. Broken, Failing, or Conflicting Areas

- Location: `coverage-policy.json`
  - **Evidence:** policy baseline is currently permissive and can pass with low/placeholder coverage values.
  - **Likely cause / nature of issue:** coverage thresholds have not yet been tightened for meaningful enforcement.

- Location: `local_check` i18n validation
  - **Evidence:** i18n validate/status are skipped when `greentic-i18n-translator` is unavailable.
  - **Likely cause / nature of issue:** optional tooling dependency is not mandatory in every local environment.

- Location: `crates/greentic-sorla-cli/src/lib.rs`
  - **Evidence:** interactive wizard mode currently covers the core create/update path, but not every optional list-style schema field such as provider hints or custom artifact lists.
  - **Likely cause / nature of issue:** the initial `greentic-qa-lib` integration is intentionally narrow so it can reuse the existing deterministic answers pipeline safely.

- Location: milestone status vs implementation depth
  - **Evidence:** PR-05 through PR-06 still have decision notes in `.codex/`, while PR-01 through PR-04 are implemented and verified in code today.
  - **Likely cause / nature of issue:** roadmap intent was created before concrete code milestones were added to the repo state.

- Location: `crates/greentic-sorla-cli/src/lib.rs`, `crates/greentic-sorla-wizard/src/lib.rs`
  - **Evidence:** the wizard crate now reuses the CLI schema, but separate schema types still exist across the two crates.
  - **Likely cause / nature of issue:** the publishable CLI crate cannot currently depend on the unpublished internal wizard crate and still pass crates.io dry-run packaging.

- Location: `crates/greentic-sorla-cli/src/lib.rs`
  - **Evidence:** answers execution currently generates deterministic package metadata and artifact placeholders itself rather than delegating to the internal pack/wizard crates.
  - **Likely cause / nature of issue:** the publishable CLI crate still needs to stay self-contained for packaging, so deeper reuse is deferred.

## 5. Notes for Future Work

- Replace placeholder perf checks with real SoRLa hotspots once the compiler and wizard flows exist.
- Wire the internal `lang`, `ir`, `pack`, and `wizard` crates into the public CLI incrementally as each milestone becomes concrete enough to preserve publish/package checks.
- Add richer artifact splits such as approvals/policies/views population as those source-language sections gain real semantics.
- Revisit schema duplication once the repo has a publish strategy for internal crates or another way to keep the CLI package self-contained.
- Replace fallback-English locale catalogs with real translated content once the wizard copy stabilizes.
- Consider whether the remaining CLI/wizard type duplication should be removed later with a publish-safe shared schema crate or a different packaging strategy.
- Expand the interactive QA form to cover optional list-style fields once there is a clearer UX for provider hints and custom artifact selection.
