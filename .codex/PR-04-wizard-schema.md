# PR 04: Generate wizard schemas from SoRLa package and product model

    **Repository:** `greenticai/greentic-sorla`

    ## Objective

    Implement `greentic-sorla wizard --schema` so it emits a deterministic, i18n-ready wizard schema that can drive create and update flows for SoRLa packages end to end.

    ## Why this PR exists

    The user explicitly wants SoRLa to behave like `gtc`: the wizard is the main control surface. `--schema` must therefore be a first-class output, not an afterthought. It needs to capture both package creation and later package updates.

    ## Scope

    Create a wizard schema model that can cover:
- package bootstrap
- package update
- provider requirement selection
- external source declarations
- event/projection settings
- compatibility/evolution choices
- output/package preferences
The schema should support localized prompts and should be driven from SoRLa/product metadata, not manually duplicated in many places.

    ## Deliverables

    - wizard schema data model
- `greentic-sorla wizard --schema`
- deterministic JSON output for the schema
- localized key structure (at least English placeholders)
- tests for emitted schema shape
- docs describing how answer documents map onto the schema

    ## Implementation notes for Codex

    The wizard schema should be expressive but not overcomplicated. Follow Greentic QA/wizard conventions conceptually:
- stable IDs
- defaults
- conditional visibility
- update-safe answers
The schema must be suitable for both:
1. creating a new SoRLa package
2. updating an existing one in-place
Make sure provider selection is not hardcoded to FoundationDB. The schema should allow provider category selection and provider hints, with actual provider packs resolved later.

    ## Acceptance criteria

    - `wizard --schema` emits valid JSON
- output is stable under repeated runs
- schema contains enough fields to configure a minimal SoRLa package end to end
- schema structure supports i18n keys
- tests cover package creation and update scenarios

    ## Non-goals

    - Full `--answers` execution
- Multiple locales fully translated
- Any provider implementation

    ## Suggested files / areas to touch

    - `crates/greentic-sorla-wizard/src/schema.rs`
- `crates/greentic-sorla-cli/src/cmd_wizard.rs`
- `tests/wizard_schema/*`
- `docs/wizard.md`
