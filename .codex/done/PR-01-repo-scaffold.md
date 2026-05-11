# PR 01: Scaffold `greentic-sorla` as a wizard-first SoRLa product

    **Repository:** `greenticai/greentic-sorla`

    ## Objective

    Create the new `greentic-sorla` repository as the canonical home for the SoRLa language, compiler, wizard, packaging model, and runtime-facing artifact contracts. The repository must be structured from day one around a **wizard-first** user experience rather than a toolbox of unrelated commands.

    ## Why this PR exists

    The earlier design drifted toward too many explicit CLI verbs. The intended product shape is closer to `gtc`: users should mainly interact through `greentic-sorla wizard --schema` and `greentic-sorla wizard --answers ...`. This PR establishes that product shape before implementation details spread in the wrong direction.

    ## Scope

    Set up the workspace, docs, CI, release scaffolding, coding standards, and a small top-level architectural baseline. Define the product boundaries clearly:
- this repo owns the language, compiler, wizard, and package generation
- it does **not** own provider implementations
- it produces artifacts that can later be bound to provider gtpacks
Also add a short architecture note that explains SoRLa v0.2 in practical terms: event-native, Git-driven, provider-aware, external-SoR-friendly.

    ## Deliverables

    - Rust workspace scaffold
- top-level README
- `docs/architecture.md`
- `docs/product-shape.md`
- `.codex/` rules or equivalent contributor guidance
- CI for fmt, clippy, tests, release tagging
- initial crate layout for:
  - `greentic-sorla-cli`
  - `greentic-sorla-lang`
  - `greentic-sorla-ir`
  - `greentic-sorla-pack`
  - `greentic-sorla-wizard`
- placeholder command that prints a useful help screen with the wizard-first UX

    ## Implementation notes for Codex

    Use the same repo hygiene patterns used elsewhere in the Greentic ecosystem:
- workspace-level lint config
- deterministic tests
- docs that explain **why** SoRLa exists, not just what files exist
- release workflow prepared for future OCI/pack publishing
The CLI should already show `wizard` as the main subcommand, even if implementation is stubbed initially. Avoid introducing many top-level user-facing commands now; if internal commands are needed, keep them hidden or clearly secondary.

    ## Acceptance criteria

    - Repository builds cleanly in CI
- `cargo test` passes
- README clearly states the wizard-first product model
- CLI help clearly advertises `wizard --schema` and `wizard --answers`
- Architecture docs explicitly state that provider implementations live in `greentic-sorla-providers`
- No provider-specific code is added here

    ## Non-goals

    - Full parser/compiler implementation
- Full wizard engine
- Any provider implementation
- Any KAFD demo logic

    ## Suggested files / areas to touch

    - `Cargo.toml`
- `crates/greentic-sorla-cli/`
- `crates/greentic-sorla-lang/`
- `crates/greentic-sorla-ir/`
- `crates/greentic-sorla-pack/`
- `crates/greentic-sorla-wizard/`
- `docs/architecture.md`
- `.github/workflows/ci.yml`
