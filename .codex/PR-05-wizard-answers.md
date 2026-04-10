# PR 05: Implement answers-driven wizard execution for create and update flows

    **Repository:** `greenticai/greentic-sorla`

    ## Objective

    Implement `greentic-sorla wizard --answers <file>` as the main deterministic execution path for creating and updating SoRLa packages and their packaging metadata.

    ## Why this PR exists

    The wizard is only useful if it can consume saved answers and perform real work. This PR makes the answers document the reproducible control plane for SoRLa creation and updates, which aligns with the desired Greentic-style workflow.

    ## Scope

    The answers-driven flow should:
- read an answer document
- validate it against the wizard schema
- create a new package or update an existing one
- emit/update source files and generated artifacts
- prepare provider requirement metadata
- produce stable output suitable for Git commits
It should support re-running safely on the same answers and should behave deterministically.

    ## Deliverables

    - answer document model
- validation against schema
- create flow
- update flow
- deterministic file generation
- compatibility checks during update
- useful error messages
- test fixtures for create and update

    ## Implementation notes for Codex

    Follow the spirit of `gtc`:
- answers are the source of execution truth
- no unnecessary interactivity once answers are supplied
- deterministic outputs for reproducible automation
For update flows, do not overwrite unrelated user content blindly. Prefer clear generated file boundaries and stable regions so repeated runs are safe. If a package already exists, the wizard should be able to:
- update metadata
- add provider requirements
- evolve event/projection declarations
- refresh generated artifacts
without destructive behavior.

    ## Acceptance criteria

    - `wizard --answers answers.json` can create a minimal valid SoRLa package
- the same command can update that package on a second run
- output is deterministic
- validation errors are actionable
- tests cover idempotent re-run behavior

    ## Non-goals

    - Interactive UX polish
- Full localization beyond scaffolding
- Provider pack resolution from GHCR
- KAFD demo specifics

    ## Suggested files / areas to touch

    - `crates/greentic-sorla-wizard/src/answers.rs`
- `crates/greentic-sorla-wizard/src/apply.rs`
- `tests/wizard_answers/*`
- sample `examples/answers/*.json`
