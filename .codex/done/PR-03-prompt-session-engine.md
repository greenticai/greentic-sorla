# PR-03 — Implement adaptive prompt session engine in greentic-sorla-lib

## Goal

Implement the core interactive prompt-to-answers engine in `greentic-sorla-lib`.

This engine is used by both:

- `greentic-sorla prompt`
- the Sorla Designer component

The engine should dynamically design a SoRLa `answers.json` through an adaptive conversation.

## Runtime flow

```text
User describes business
  -> prompt engine
  -> adaptive questions
  -> design draft
  -> validated answers.json
```

The prompt engine outputs only `answers.json`.

It must not generate:

```text
sorla.yaml
.gtpack
runtime config
deployment artifacts
src/
assets/
build-answers.json
component source code
component build configuration
```

Repo reality check: `greentic-sorla-lib` already exposes the deterministic downstream path through `schema_for_answers`, `normalize_answers`, `validate_model`, `generate_preview`, `build_gtpack_entries`, `build_gtpack_bytes`, and `build_gtpack_file`. This PR should add only the prompt/session layer that produces answers compatible with those APIs.

## Suggested trait

```rust
pub trait PromptAuthoringEngine {
    fn start_session(&self, config: PromptSessionConfig) -> Result<PromptSessionState, SorlaError>;

    fn next_turn(&self, input: PromptTurnInput) -> Result<PromptTurnOutput, SorlaError>;

    fn generate_answers(&self, session: PromptSessionState) -> Result<serde_json::Value, SorlaError>;
}
```

Use `serde_json::Value` until a public `AnswersDocument` facade exists.

The generated value must follow the current wizard schema shape (`schema_version` currently `0.5`, with `0.4` still accepted by the existing validator). It must include the required `flow`, `output_dir`, package, provider, and output sections expected by `normalize_answers` / `wizard --answers`.

## Adaptive questioning

Questions must not be a static questionnaire.

The question graph should adapt based on previous answers.

Examples:

- If the user says a lease can have multiple tenants, ask whether liability is joint, individual, or both.
- If payments are immutable, generate ledger-style events and avoid destructive update actions.
- If documents have expiry dates, ask whether reminders or projections are required.
- If maintenance requests involve suppliers, ask whether supplier approval, quotes, or SLA tracking are needed.
- If personally identifiable information is present, ask about retention, audit, and privacy requirements.
- If the user mentions regulated data, ask about audit, access control, retention, and approval requirements.

## Session phases

Implement:

```rust
pub enum PromptPhase {
    AwaitingBusinessPrompt,
    AskingQuestions,
    ReviewingDesignPlan,
    ReadyToGenerateAnswers,
    Completed,
}
```

The engine should support save/resume by serializing `PromptSessionState`.

## LLM usage

Use the LLM capability abstraction from PR-02.

The LLM should assist with:

- extracting likely records
- identifying relationships
- identifying missing information
- proposing follow-up questions
- building a draft model
- producing candidate answers

The deterministic validation/conversion layer should ensure output matches the current wizard answers schema.

## Generated answers

The generated `answers.json` must be compatible with:

```bash
greentic-sorla wizard --answers answers.json
```

Use the existing schema returned by:

```bash
greentic-sorla wizard --schema
```

Also validate through the library facade in tests where possible:

```rust
let model = greentic_sorla_lib::normalize_answers(answers, NormalizeOptions::default())?;
let report = greentic_sorla_lib::validate_model(&model, ValidateOptions::default());
```

## Tests

Add tests using a fake LLM provider.

Minimum test scenario:

```text
Business: landlord/tenant property management
Answers:
- lease can have multiple tenants
- liability is joint
- payments are immutable
- maintenance requests use suppliers
- supplier work requires approval
Expected:
- answers.json validates
- records include landlord, tenant, property, lease, payment, maintenance_request, supplier
- events include lease_started, payment_recorded, maintenance_request_opened
- policies or approvals include supplier approval
```

## Acceptance criteria

- Prompt sessions can start, advance, save, and resume.
- Follow-up questions adapt to previous answers.
- Generated `answers.json` validates against the existing wizard schema.
- Generated `answers.json` can be passed to `wizard --answers`.
- Prompt engine outputs only `answers.json`.
- Tests use a fake LLM capability.
