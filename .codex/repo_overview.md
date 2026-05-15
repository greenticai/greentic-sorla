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
| PR-07 | Agent endpoint fixtures, docs, and local checks | verified | Realistic agent endpoint golden fixture, authoring docs, and pack-level end-to-end fixture assertions are implemented. |
| PR-08 | `gtc` agent endpoint handoff contract docs | verified | Downstream handoff contract docs define artifact names, schema expectations, validation guidance, and ownership boundaries. |
| PR-09 | Landlord/tenant FoundationDB e2e | verified | Non-publishable e2e harness, fixtures, xtask command, docs, script, and manual/nightly workflow validate a landlord/tenant SoR through the sibling FoundationDB provider. |
| PR-10 | Executable SoRLa migrations and agent operations | verified | Field relationships, migration backfills/idempotence keys, agent operation emits, structured result/error contract, executable contract artifact, docs, fixtures, and e2e harness consumption are implemented. |
| PR-07a | Extension-first architecture docs | implemented | Repo docs and Codex guidance now describe `greentic-sorla` as a `gtc` extension-layer product and delegate final assembly ownership to `gtc`. |
| PR-08a | Extension handoff refactor | implemented | CLI-generated manifests now declare themselves as `gtc` launcher-handoff metadata, and `greentic-sorla-pack` now exposes handoff-first APIs while keeping legacy compatibility aliases. |
| PR-09a | Naming migration | implemented | `handoff` is now the canonical integration term, source-authoring `package` language remains stable, and canonical launcher-handoff aliases are written alongside legacy package-manifest names. |
| Ontology PR-01 | Generic ontology authoring model | verified | Optional `ontology` authoring, static validation, canonical IR lowering, wizard answers/schema support, docs, and landlord/tenant example coverage are implemented. |
| Ontology PR-02 | Deterministic ontology gtpack artifacts | verified | Ontology-enabled packs now emit graph JSON, canonical ontology IR CBOR, schema JSON, manifest extension metadata, inspect summaries, validation-inspect summaries, and doctor tamper checks. |
| Ontology PR-03 | Semantic aliases and entity linking | verified | Optional `semantic_aliases` and `entity_linking` declarations now validate against ontology concepts/relationships, lower deterministically into ontology IR/graph artifacts, and render from wizard answers. |
| Ontology PR-04 | Generic retrieval bindings | verified | Optional `retrieval_bindings` declarations now validate ontology-scoped evidence provider requirements and traversal filters, lower into canonical IR, emit JSON/CBOR pack assets, and inspect/doctor cleanly. |
| Ontology PR-05 | Production ontology validation suite | verified | SORX validation manifests now include deterministic ontology/retrieval validation suites, promotion gating for exported packs, provider compatibility checks for retrieval providers, and schema rejection of obsolete suite-level fields. |
| Ontology PR-06 | Deterministic ontology handoff scenario | verified | Added the provider-agnostic `examples/ontology-business` answers fixture, retrieval-binding answers support, deterministic pack round-trip test, smoke script, and scenario docs. |
| Ontology PR-07 | Production ontology hardening and compatibility | verified | Added ontology/retrieval schema commands, expanded local checks with schema and ontology smoke coverage, and documented production readiness, security, and compatibility rules. |
| Designer PR-08 | Split CLI from reusable libraries | implemented | Reusable authoring/facade boundaries now exist without duplicating the existing lang/IR/pack crates. |
| Designer PR-09 | Publish stable Sorla library API | implemented | `greentic-sorla-lib` exposes deterministic library APIs for Designer/tooling reuse. |
| Designer PR-10 | WASM-friendly library profile | implemented | The facade is structured for the Designer extension profile with guarded WASM compatibility checks. |
| Designer PR-11 | Sorla Designer extension crate | implemented | `greentic-sorla-designer-extension` provides the current deterministic JSON adapter boundary. |
| Designer PR-12 | Designer prompting and knowledge | implemented | Designer prompting and knowledge helpers now reuse the public facade model shape and credential hygiene checks. |
| Designer PR-13 | Designer `.gtpack` artifact output | implemented | The extension can return deterministic `.gtpack` compatibility output while documenting unsupported WASM ZIP emission boundaries. |
| Designer PR-14 | Overall Designer prompt to Sorla `.gtpack` e2e | implemented | The final e2e harness covers the SoRLa-local prompt-to-pack flow and keeps Sorx/Designer validation as documented cross-repo/manual coverage. |
| Designer PR-15 | Designer node types from SoRLa agent endpoints | implemented | Packs now emit `assets/sorla/designer-node-types.json`, expose inspect summaries, and doctor-check node types against canonical agent endpoints and contract hashes. |
| Designer PR-16 | Designer extension node type contributions | implemented | The Designer extension now lists generated endpoint node types and can produce locked generic flow-node JSON from a selected node type. |
| Designer PR-17 | SoRLa-local Designer node type e2e | implemented | Added focused e2e coverage for the SoRLa-owned node-type-to-locked-endpoint-ref path without cross-repo runtime dependencies. |
| Designer PR-18 | Designer node type security hardening | implemented | Doctor and extension tests now harden generated locked endpoint metadata, required mappings, and free-text runtime selection boundaries. |
| Designer PR-19 | Agent endpoint action catalog view | implemented | Packs now emit a deterministic design-time `agent-endpoint-action-catalog.json` view derived from canonical agent endpoints. |
| Designer PR-20 | Endpoint contract hash and lock hardening | implemented | Designer node type and action catalog endpoint refs now enforce canonical `sha256:<64 lowercase hex>` hashes and pack lock coverage. |
| Designer PR-21 | Designer node type metadata polish | implemented | Node type generation now includes stable field labels, widgets, optional aliases, and search context while preserving v1 compatibility. |
| Designer PR-22 | Designer extension locked endpoint node UX | implemented | The extension can resolve node types by exact ID, endpoint ID, or label, emits richer locked metadata, and returns selection diagnostics. |

## 1. High-Level Purpose

`greentic-sorla` is the wizard-first home for the SoRLa language, compiler-facing IR, and extension-facing authoring workflow on top of `gtc`. The repository is intended to produce provider-agnostic source artifacts and abstract metadata rather than owning final runtime assembly itself.

The current implementation now includes the verified PR-01 scaffold, PR-02 language semantics, the PR-03 canonical IR/artifact slice, the PR-04 wizard schema contract, and a working PR-05 answers execution path. The CLI can now create or update a minimal SoRLa package layout deterministically and refresh wizard-owned artifacts while preserving user-authored content outside generated regions.

PR-06 is now implemented in the active CLI path: locale selection/fallback and generated metadata are part of the wizard-owned output under `.greentic-sorla/generated/`.

PR-07 through PR-08 add agent endpoint authoring, canonical IR, handoff artifacts, exporter fragments, golden fixtures, and downstream `gtc` handoff contract documentation.

PR-09 adds a repeatable landlord/tenant FoundationDB provider e2e scenario in this repository. The e2e uses the sibling `greentic-sorla-providers` workspace as an integration dependency while keeping provider implementations out of `greentic-sorla`.

PR-10 promotes the e2e scenario assumptions into first-class SoRLa contracts: record field references, executable migration backfills, idempotence keys, mutating agent endpoint emit plans, a structured operation result/error contract, and an `executable-contract.json` pack artifact.

The wizard entrypoint now also supports an interactive local mode backed by `greentic-qa-lib` when `--answers` is not provided, while still routing the collected answers through the same deterministic answers application pipeline.

PR-07 updates the repo-level documentation and Codex guidance so `gtc` is documented as the owner of extension registry resolution, launcher/setup/start handoff, and final pack/bundle assembly. The standalone `greentic-sorla wizard` flow remains documented as a local authoring and extension-development surface.

PR-08 now carries that boundary into implementation: generated manifest JSON explicitly marks itself as `gtc` launcher-handoff metadata, and the artifact crate now presents handoff-first APIs instead of package-assembly semantics as its primary surface.

PR-09 standardizes naming around that boundary: SoRLa source authoring still uses `package`, but extension-integration outputs now use `handoff` as the canonical term, with migration aliases and documentation kept in place for compatibility.

Ontology PR-01 adds a first-class provider-agnostic ontology authoring model. SoRLa now validates ontology concepts, relationships, inheritance, record/field backing, sensitivity markers, policy hooks, and provider requirement hints, then lowers the model into deterministic canonical IR while keeping existing record-first workflows compatible.

Ontology PR-02 carries that model through the existing `.gtpack` compatibility path. Ontology-enabled packs now include deterministic `assets/sorla/ontology.graph.json`, `assets/sorla/ontology.ir.cbor`, and `assets/sorla/ontology.schema.json` assets discovered through `pack.cbor` extension metadata, with doctor/inspect coverage but no new final bundle assembly ownership.

Ontology PR-03 adds optional semantic aliases and entity-linking declarations. Aliases normalize by trimming, whitespace collapsing, and lowercase conversion; duplicate aliases for the same target are de-duplicated with warnings while cross-target collisions are rejected. Entity-linking strategies validate target concepts, backed record fields, URL-safe strategy IDs, sensitivity, and confidence bounds, then flow into ontology IR and graph handoff artifacts.

Ontology PR-04 adds optional retrieval bindings for ontology-scoped evidence. Retrieval providers reuse abstract provider categories/capabilities, scopes validate ontology concepts/relationships and traversal depth, and pack output includes deterministic `retrieval-bindings.json` plus canonical `retrieval-bindings.ir.cbor` when bindings are present.

Ontology PR-05 extends the embedded SORX validation contract. Ontology-enabled exported packs now carry a required `ontology` promotion suite, retrieval-enabled exported packs carry a required `retrieval` suite, private-only ontology packs keep ontology checks optional, and retrieval provider requirements are folded into existing provider-capability validation metadata without executing providers in this repo.

Ontology PR-06 adds a deterministic business-domain handoff scenario. The example answers generate ontology, semantic aliases, entity-linking, retrieval bindings, agent endpoints, validation metadata, doctor/inspect summaries, and byte-identical `.gtpack` output across repeated temp-directory runs.

Ontology PR-07 hardens that surface for production use. The CLI can emit SoRLa-owned ontology and retrieval binding schemas, local checks now verify those schema commands and the ontology handoff smoke, and docs capture compatibility, security, determinism, and extension-first ownership boundaries.

Designer PR-08 through PR-22 added the reusable library APIs, stable facade, WASM-friendly profile, Designer extension adapter crate, prompting/knowledge helpers, deterministic `.gtpack` compatibility output, prompt-to-pack e2e coverage, generated Designer node type metadata from SoRLa agent endpoints, extension tools that turn those node types into locked generic flow-node JSON, a SoRLa-local node-type-to-locked-endpoint e2e, security hardening of the generated metadata path, a deterministic agent-endpoint action catalog view, endpoint hash/lock hardening, node metadata polish, and extension UX improvements. The line builds on existing agent endpoint, executable contract, and Designer adapter boundaries rather than assuming a separate Business Action Catalog or vendored Designer SDK/WIT, preserving current crate boundaries and the extension-first ownership rule.

## 2. Main Components and Functionality

- Component: workspace root
  - **Path:** `Cargo.toml`
  - **Role:** Virtual workspace and shared metadata definition.
  - **Key functionality:**
    - Defines the main SoRLa workspace layout.
    - Uses `default-members` so normal cargo commands stay self-contained while the provider-backed e2e crate remains opt-in.
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
    - Accepts optional ontology answers and renders them into generated SoRLa YAML while preserving old answer files.
    - Accepts optional retrieval binding answers and renders them into generated SoRLa YAML alongside ontology, semantic aliases, and entity-linking sections.
    - Exposes `pack schema ontology` and `pack schema retrieval-bindings` for SoRLa-owned schema inspection alongside the SORX validation/exposure/compatibility schema commands.
    - Generates canonical `launcher-handoff.json` plus legacy-compatible `package-manifest.json`, alongside provider and locale handoff metadata, so extension naming can migrate without breaking existing consumers.
    - Includes placeholder perf/concurrency harness files.
  - **Key dependencies / integration points:** intentionally stays self-contained for schema emission so the publishable CLI package can still pass crates.io dry-run checks; production composition is documented in terms of `gtc` extension launch and handoff.

- Component: `crates/greentic-sorla-lang`
  - **Path:** `crates/greentic-sorla-lang`
  - **Role:** Language-facing AST and parser crate.
  - **Key functionality:**
    - Defines v0.2 package AST nodes for records, events, projections, migrations, provider requirements, and external references.
    - Defines optional ontology AST nodes for concepts, relationships, cardinality, constraints, sensitivity, policy hooks, and provider requirement hints.
    - Defines optional semantic alias and entity-linking AST nodes that map user/document/provider language to ontology concepts without provider credentials or runtime matching.
    - Defines optional retrieval binding AST nodes for abstract evidence providers, ontology scopes, entity-scope filters, traversal rules, and retrieval permission modes.
    - Defines executable AST nodes for field `references`, migration `backfills`/`idempotence_key`, and agent endpoint `emits` plans.
    - Defines agent endpoint AST nodes for inputs, outputs, risk, approval, backing references, visibility, provider requirements, examples, and operation emits.
    - Parses YAML-authored packages using parser-validated `source: native|external|hybrid`.
    - Applies additive v0.1 compatibility by defaulting omitted `source` to `native` and surfacing warnings.
    - Enforces field-level authority rules for hybrid records and requires `external_ref` for external/hybrid sources.
    - Validates record/event/projection references, migration backfills, agent endpoint identity, duplicate inputs/outputs, risk/approval constraints, provider requirements, backing references, and `$input.<name>` emit payload templates.
    - Validates ontology schema version, unique URL-safe IDs, concept references, acyclic inheritance, backing records/fields, and unknown-field rejection.
    - Validates semantic aliases, deterministic alias normalization/collision behavior, entity-linking strategy IDs, confidence bounds, and backing-record target fields.
    - Validates retrieval binding schema, provider IDs/capabilities, scope/provider references, ontology target references, traversal directions, and bounded traversal depths.

- Component: `crates/greentic-sorla-ir`
  - **Path:** `crates/greentic-sorla-ir`
  - **Role:** Canonical IR and lowering crate.
  - **Key functionality:**
    - Lowers parsed SoRLa packages into a deterministic, versioned IR.
    - Lowers optional ontology metadata into deterministic canonical IR with sorted concepts, relationships, constraints, provider requirements, and inheritance parents.
    - Lowers semantic aliases and entity-linking strategies into the ontology IR with normalized sorted aliases and sorted strategy IDs.
    - Lowers retrieval bindings into canonical IR with sorted providers, capabilities, scopes, and traversal rules.
    - Separates business records, events, projections, compatibility data, external sources, and provider contract requirements.
    - Lowers field references, migration backfills/idempotence keys, and agent endpoint emits into canonical IR.
    - Lowers agent endpoints into canonical IR and includes them in canonical hashes and agent-tool views.
    - Provides canonical CBOR serialization, inspectable JSON rendering, and hash derivation from canonical serialized form.

- Component: `crates/greentic-sorla-pack`
  - **Path:** `crates/greentic-sorla-pack`
  - **Role:** Abstract artifact emission crate with handoff-first APIs and legacy pack-oriented compatibility aliases.
  - **Key functionality:**
    - Builds deterministic artifact sets from YAML package input.
    - Exposes `HandoffManifest`, `scaffold_handoff_manifest`, and `build_handoff_artifacts_from_yaml` as the primary API while preserving `PackageManifest`, `scaffold_manifest`, and `build_artifacts_from_yaml` aliases.
    - Emits canonical `launcher-handoff.cbor` plus legacy-compatible `package-manifest.cbor`, along with split CBOR artifacts such as `model.cbor`, `events.cbor`, `projections.cbor`, `external-sources.cbor`, and `provider-contract.cbor`.
    - Produces inspectable JSON and `agent-tools.json` views for tests and downstream tooling.
    - Emits agent endpoint handoff artifacts including `agent-gateway.json`, `agent-endpoints.ir.cbor`, OpenAPI overlay YAML, Arazzo workflows, `mcp-tools.json`, and `llms.txt.fragment`.
    - Emits `executable-contract.json` with relationships, migrations, agent operation emits, and operation result/error schema keyed by the canonical IR hash.
    - Emits optional ontology handoff artifacts into `.gtpack` archives: deterministic graph JSON, canonical ontology IR CBOR, and JSON schema assets referenced by `greentic.sorla.ontology.v1` extension metadata.
    - Emits optional retrieval binding handoff artifacts into `.gtpack` archives: `retrieval-bindings.json` and `retrieval-bindings.ir.cbor`, referenced by `greentic.sorla.retrieval-bindings.v1` extension metadata.
    - Exposes deterministic JSON schema helpers for ontology and retrieval binding metadata.
    - Generates deterministic SORX validation suites for ontology static checks, ontology relationships, semantic aliases, entity-linking declarations, retrieval bindings, provider capabilities, and security policy gates.
    - Requires ontology/retrieval validation suites for exported packs via `promotion_requires`, while allowing private-only ontology suites to remain optional metadata.
    - Validates ontology pack integrity in `pack doctor`, including manifest paths, lock coverage, IR hash matching, graph/IR consistency, aliases, entity-linking strategies, backing record/field references, and secret scanning.
    - Validates retrieval binding pack integrity in `pack doctor`, including manifest paths, lock coverage, and JSON/CBOR/model consistency.
    - Rejects obsolete suite-level validation fields such as `kind` and `required_for_public_exposure`; test kind remains a per-test field.
  - **Key dependencies / integration points:** documented as producing source artifacts and handoff-oriented metadata rather than final packs or bundles.

- Component: `crates/greentic-sorla-e2e`
  - **Path:** `crates/greentic-sorla-e2e`
  - **Role:** Opt-in, non-publishable end-to-end scenario harness.
  - **Key functionality:**
    - Validates the landlord/tenant SoR scenario from YAML fixtures through parser, IR, pack artifacts, FoundationDB provider events/projections, v1-to-v2 migration, and deterministic agent operations.
    - Reads migration backfills and agent operation emits from canonical IR instead of duplicating those assumptions in the harness.
    - Depends on the sibling `../greentic-sorla-providers` workspace for the FoundationDB provider.
    - Is included in the workspace lockfile but excluded from `default-members` so normal local checks do not require the sibling provider repo.

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
    - Validates i18n JSON syntax.
    - Runs i18n `status`/`validate` as advisory by default when translator tooling is available, and as strict failures when `I18N_STRICT=true`.
    - Runs schema command checks for validation, exposure policy, compatibility, ontology, and retrieval bindings.
    - Runs the ontology handoff smoke scenario to verify deterministic ontology-business pack generation, doctor, inspect, validation-inspect, and credential/path hygiene.
  - **Key dependencies / integration points:** used by CI/release jobs and local development.

- Component: `xtask`
  - **Path:** `xtask`
  - **Role:** Repository task runner.
  - **Key functionality:**
    - Provides `cargo xtask e2e landlord-tenant --provider foundationdb [--smoke]`.
    - Invokes the opt-in e2e crate through its manifest path so provider-backed tests are explicit.

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

- Component: `.github/workflows/landlord-tenant-e2e.yml`
  - **Path:** `.github/workflows/landlord-tenant-e2e.yml`
  - **Role:** Manual/nightly provider-backed e2e workflow.
  - **Key functionality:** checks out `greentic-sorla` and sibling `greentic-sorla-providers`, then runs `cargo xtask e2e landlord-tenant --provider foundationdb` with optional smoke mode.

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

- Component: `docs/ontology.md`
  - **Path:** `docs/ontology.md`
  - **Role:** Ontology authoring documentation.
  - **Key functionality:** documents the optional provider-agnostic ontology model, validation rules, semantic aliases, entity-linking declarations, wizard answer support, deterministic `.gtpack` ontology artifacts, SORX validation metadata, and the runtime boundary that leaves graph traversal/provider execution downstream.

- Component: `docs/entity-linking.md`
  - **Path:** `docs/entity-linking.md`
  - **Role:** Semantic alias and entity-linking documentation.
  - **Key functionality:** documents alias normalization/collision rules, entity-linking strategy validation, confidence/sensitivity handling, and the provider-agnostic downstream handoff boundary.

- Component: `docs/retrieval-bindings.md`
  - **Path:** `docs/retrieval-bindings.md`
  - **Role:** Retrieval binding documentation.
  - **Key functionality:** documents ontology-scoped evidence provider declarations, traversal filters, validation rules, pack assets, and the no-runtime-execution boundary.

- Component: `examples/ontology-business`
  - **Path:** `examples/ontology-business/answers.json`
  - **Role:** Deterministic ontology handoff scenario fixture.
  - **Key functionality:** exercises rich wizard answers for generic business ontology concepts, semantic aliases, entity-linking, retrieval bindings, actions, events, projections, provider requirements, policies, approvals, migrations, and agent endpoints without external services or concrete provider credentials.

- Component: `scripts/e2e/ontology-handoff-smoke.sh`
  - **Path:** `scripts/e2e/ontology-handoff-smoke.sh`
  - **Role:** Local deterministic ontology handoff smoke test.
  - **Key functionality:** generates the ontology business pack twice in temporary directories, checks byte-identical output, runs doctor/inspect/validation-inspect, verifies ontology/retrieval summaries, and scans generated handoff metadata for path/credential leakage.

- Component: `docs/ontology-handoff-scenario.md`
  - **Path:** `docs/ontology-handoff-scenario.md`
  - **Role:** Scenario documentation.
  - **Key functionality:** documents the SoRLa-owned ontology handoff scenario, one-command smoke path, and downstream Sorx/`gtc` runtime boundary.

- Component: `docs/ontology-production-readiness.md`, `docs/ontology-security.md`, `docs/ontology-compatibility.md`
  - **Path:** `docs/`
  - **Role:** Production ontology hardening documentation.
  - **Key functionality:** documents schema versioning, determinism gates, static security checks, compatibility behavior, and the boundary that keeps runtime execution in downstream Sorx/`gtc`/provider systems.

- Component: `docs/architecture.md`, `docs/product-shape.md`, `docs/extensions-with-gtc.md`
  - **Path:** `docs/`
  - **Role:** Architectural and product-shape documentation.
  - **Key functionality:** documents wizard-first UX, crate boundaries, the `gtc` extension ownership boundary, and the rule that providers live in `greentic-sorla-providers`.

- Component: `docs/spec/v0.2.md`
  - **Path:** `docs/spec/v0.2.md`
  - **Role:** SoRLa v0.2 language notes.
  - **Key functionality:** documents `source`, `external_ref`, field-level authority for hybrid records, events, projections, provider requirements, and compatibility-oriented migrations with executable backfills.

- Component: `docs/spec/executable-contracts.md`
  - **Path:** `docs/spec/executable-contracts.md`
  - **Role:** Executable SoRLa contract documentation.
  - **Key functionality:** documents field `references`, migration `backfills`/`idempotence_key`, agent endpoint `emits`, and the `greentic.sorla.executable-contract.v1` pack artifact.

- Component: `docs/artifacts.md`
  - **Path:** `docs/artifacts.md`
  - **Role:** Artifact contract documentation.
  - **Key functionality:** documents canonical ordering/hash rules, the current emitted artifact set, the executable contract artifact, and frames the current emitted artifact set as extension-friendly source artifacts rather than final packs/bundles.

- Component: `docs/agent-endpoints.md`, `docs/agent-endpoint-handoff-contract.md`
  - **Path:** `docs/`
  - **Role:** Agent endpoint authoring and downstream handoff documentation.
  - **Key functionality:** documents agent endpoint fields, safety metadata, exporter artifacts, and the `greentic-sorla`/`gtc`/provider ownership boundary.

- Component: `docs/landlord-tenant-e2e.md`
  - **Path:** `docs/landlord-tenant-e2e.md`
  - **Role:** Provider-backed e2e scenario documentation.
  - **Key functionality:** explains how to run the landlord/tenant FoundationDB scenario, what provider mode is tested, how schema migration is validated, and how deterministic agent operations are mapped.

- Component: `docs/wizard.md`
  - **Path:** `docs/wizard.md`
  - **Role:** Wizard schema documentation.
  - **Key functionality:** documents create/update flows, ownership/update rules, answer document expectations, locale selection/fallback, and the current schema/i18n contract.

- Component: `docs/packaging.md`
  - **Path:** `docs/packaging.md`
  - **Role:** Generated abstract metadata documentation.
  - **Key functionality:** documents the canonical launcher handoff document, the legacy package-manifest alias, provider requirements metadata, locale metadata, and the rule that these remain abstract handoff metadata inside `greentic-sorla`.

- Component: `docs/naming-migration.md`
  - **Path:** `docs/naming-migration.md`
  - **Role:** Compatibility and terminology guide.
  - **Key functionality:** documents the canonical `handoff` terminology, the retained `package` source-authoring terminology, and the old-to-new filename/API mapping used during migration.

- Component: `crates/greentic-sorla-cli/examples/answers`
  - **Path:** `crates/greentic-sorla-cli/examples/answers`
  - **Role:** Sample answers documents.
  - **Key functionality:** provides minimal create and update examples for the deterministic wizard execution path.

- Component: `crates/greentic-sorla-pack/tests/golden`
  - **Path:** `crates/greentic-sorla-pack/tests/golden`
  - **Role:** Golden fixture coverage for PR-03.
  - **Key functionality:** fixture YAML and expected inspect JSON verify deterministic lowering and artifact generation.

- Component: `tests/e2e/fixtures`
  - **Path:** `tests/e2e/fixtures`
  - **Role:** Real-world e2e fixture inputs.
  - **Key functionality:** includes landlord/tenant v1 and v2 SoRLa schemas plus realistic seed data used by the FoundationDB provider e2e harness.

- Component: `tools/i18n.sh`
  - **Path:** `tools/i18n.sh`
  - **Role:** repository i18n helper.
  - **Key functionality:** commands for `translate`, `validate`, `status`, and `all`; scans repo `i18n/en.json` sources and translates them in 200-item batches by default.
  - **Key dependencies / integration points:** reads `i18n/locales.json`, prefers `greentic0i18n-translator`, and falls back to `greentic-i18n-translator` when needed.

- Component: `i18n/en.json`, `i18n/locales.json`, `i18n/*.json`
  - **Path:** `i18n/`
  - **Role:** locale source and translations.
  - **Key functionality:** English source is canonical; all listed locale files are JSON-parseable, while translation completeness is tracked by advisory/strict i18n checks.

- Component: `coverage-policy.json`
  - **Path:** `coverage-policy.json`
  - **Role:** policy source for nightly coverage gate.
  - **Key functionality:** used by `nightly-coverage.yml`.

- Component: `.codex/global_rules.md`
  - **Path:** `.codex/global_rules.md`
  - **Role:** repository operating instructions.
  - **Key functionality:** enforces pre/post PR summary and required local check behavior, and now points contributors to extension-first architecture rules.

- Component: `.codex/architecture_rules.md`
  - **Path:** `.codex/architecture_rules.md`
  - **Role:** Codex-facing architectural guardrails.
  - **Key functionality:** explicitly forbids growing local final pack/bundle generation and directs future work toward the `gtc` extension boundary.

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
  - **Short description:** PR-02 and PR-07/08 landed concrete AST, parser, and agent endpoint support.

- Location: `crates/greentic-sorla-ir`, `crates/greentic-sorla-pack`
  - **Status:** implemented
  - **Short description:** PR-03 and PR-07/08 landed deterministic lowering, artifact emission, canonical hashing, agent endpoint handoff artifacts, and golden fixture coverage.

- Location: `crates/greentic-sorla-e2e`
  - **Status:** implemented / opt-in
  - **Short description:** PR-09 landlord/tenant scenario validates SoRLa fixtures, provider-backed event/projection behavior, migration idempotence, and deterministic agent operations through `cargo xtask e2e landlord-tenant --provider foundationdb`.

- Location: `.codex/PR-10-executable-sorla-migrations-and-agent-ops.md`
  - **Status:** planned
  - **Short description:** follow-up PR needed because PR-09 still implements relationship validation, migration backfills, and agent operation dispatch in the e2e harness rather than as first-class SoRLa executable contracts.

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
  - **Evidence:** i18n validate/status are advisory unless `I18N_STRICT=true`; JSON syntax is still enforced.
  - **Likely cause / nature of issue:** translation completeness requires external translation lifecycle work and should not block ordinary code checks by default.

- Location: `crates/greentic-sorla-cli/src/lib.rs`
  - **Evidence:** interactive wizard mode currently covers the core create/update path, but not every optional list-style schema field such as provider hints or custom artifact lists.
  - **Likely cause / nature of issue:** the initial `greentic-qa-lib` integration is intentionally narrow so it can reuse the existing deterministic answers pipeline safely.

- Location: milestone status vs implementation depth
  - **Evidence:** PR-01 through PR-09 now have implementation or verification artifacts in code/docs; PR-10 is a planned follow-up.
  - **Likely cause / nature of issue:** roadmap intent is being tracked as `.codex/PR-*.md` briefs alongside implementation.

- Location: `crates/greentic-sorla-e2e`
  - **Evidence:** the scenario validates migrations and agent operations through harness logic instead of executable SoRLa migration/operation contracts.
  - **Likely cause / nature of issue:** SoRLa currently describes migrations and agent endpoints as metadata, not as an executable operation-plan language.

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
