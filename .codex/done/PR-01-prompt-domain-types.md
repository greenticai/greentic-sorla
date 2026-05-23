# PR-01 — Add prompt authoring domain types to greentic-sorla-lib

## Goal

Add the reusable data model for interactive prompt-to-answers authoring inside `greentic-sorla-lib`.

This PR should not call any LLM and should not add CLI behavior yet. It only introduces the shared types required by the CLI and Sorla Designer component.

## Background

The repo already contains:

- `crates/greentic-sorla-lib`
- `crates/greentic-sorla-cli`
- `crates/greentic-sorla-wizard`
- `crates/greentic-sorla-designer-extension`

The prompt engine should live in `greentic-sorla-lib` so the CLI and Designer component can share the same session engine. The current `greentic-sorla-cli` crate is a thin binary/compatibility wrapper; the parser and wizard execution already live in `greentic-sorla-lib/src/lib.rs`.

The existing `AnswersDocument` and `ExecutionSummary` types are private inside `greentic-sorla-lib`. Keep the prompt session API independent of those private structs for this PR and use `serde_json::Value` at the prompt boundary until a public answers/apply facade is introduced.

## Required changes

Add a new module, for example:

```text
crates/greentic-sorla-lib/src/prompt/
  mod.rs
  types.rs
  draft.rs
  questions.rs
```

Expose it from `lib.rs`.

## Types to add

Add serializable/deserializable types:

```rust
pub struct PromptSessionConfig {
    pub locale: Option<String>,
    pub schema_version: Option<String>,
    pub package_name_hint: Option<String>,
    pub package_version_hint: Option<String>,
    pub llm: LlmCapabilityConfig,
}

pub struct LlmCapabilityConfig {
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub capability_id: Option<String>,
}

pub struct PromptSessionState {
    pub session_id: String,
    pub phase: PromptPhase,
    pub business_prompt: Option<String>,
    pub answers_so_far: Vec<PromptAnswer>,
    pub assumptions: Vec<PromptAssumption>,
    pub draft_model: Option<SorDesignDraft>,
}

pub enum PromptPhase {
    AwaitingBusinessPrompt,
    AskingQuestions,
    ReviewingDesignPlan,
    ReadyToGenerateAnswers,
    Completed,
}

pub struct PromptTurnInput {
    pub session: PromptSessionState,
    pub user_message: String,
}

pub struct PromptTurnOutput {
    pub session: PromptSessionState,
    pub assistant_message: String,
    pub next_questions: Vec<PromptQuestion>,
    pub design_plan: Option<SorDesignDraft>,
    pub answers_document: Option<serde_json::Value>,
}
```

Do not derive `Debug` in a way that prints `api_key`. Either omit `Debug` for `LlmCapabilityConfig` or implement a redacted debug representation.

Use `serde_json::Value` for `answers_document` if the existing `AnswersDocument` type is still private. Do not make broad visibility changes unless needed. A later PR can introduce a public wrapper/facade.

If adding all draft subtypes in this PR would make it too large, add the complete container shape now but keep individual draft structs shallow and serde-compatible. The next PR can add richer validation/conversion.

## Draft model

Add an intermediate model:

```rust
pub struct SorDesignDraft {
    pub summary: String,
    pub records: Vec<DraftRecord>,
    pub relationships: Vec<DraftRelationship>,
    pub actions: Vec<DraftAction>,
    pub events: Vec<DraftEvent>,
    pub projections: Vec<DraftProjection>,
    pub policies: Vec<DraftPolicy>,
    pub approvals: Vec<DraftApproval>,
    pub migrations: Vec<DraftMigration>,
    pub provider_requirements: Vec<DraftProviderRequirement>,
}
```

Keep draft records simple and schema-facing:

```rust
pub struct DraftRecord {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<DraftField>,
}

pub struct DraftField {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    pub sensitive: bool,
    pub description: Option<String>,
}
```

## Question model

Add structured question support:

```rust
pub struct PromptQuestion {
    pub id: String,
    pub text: String,
    pub help: Option<String>,
    pub answer_kind: PromptAnswerKind,
    pub required: bool,
    pub risk: PromptQuestionRisk,
    pub depends_on: Vec<String>,
}

pub enum PromptAnswerKind {
    FreeText,
    Boolean,
    SingleChoice { choices: Vec<String> },
    MultiChoice { choices: Vec<String> },
}

pub enum PromptQuestionRisk {
    Low,
    Medium,
    High,
}
```

## Constraints

- Do not add provider-specific LLM code.
- Do not add CLI behavior.
- Do not generate `sorla.yaml`.
- Do not generate `.gtpack`.
- Do not generate component `src/`, `assets/`, or `build-answers.json`.

## Acceptance criteria

- New prompt domain types compile.
- Types are available from `greentic-sorla-lib`.
- Types are serializable/deserializable where required for session persistence.
- Unit tests cover JSON round-tripping of `PromptSessionState`, `SorDesignDraft`, and `PromptQuestion`.
