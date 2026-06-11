# PR-04: Refactor Prompt Engine into Domain Extraction, Review, and Compilation Phases

## Summary

Refactor `DefaultPromptAuthoringEngine` so the LLM is responsible for business/domain extraction and semantic updates, while deterministic compiler phases produce the SorLA mechanics.

The current state machine has useful phases, but they are too closely tied to generating full answers. This PR introduces clearer phases:

```text
AwaitingBusinessPrompt
ExtractingDomainModel
ReviewingDomainModel
AskingTargetedQuestions
CompilingExpandedPlan
ReviewingExpandedPlan
GeneratingAnswers
Completed
```

## Motivation

Current phases:

```text
AwaitingBusinessPrompt
AskingQuestions
ReviewingDesignPlan
ReadyToGenerateAnswers
Completed
```

The problem is that the first LLM call and final generation both have to reason about too much SorLA detail.

We want:

- First review: business concepts, records, actors, processes.
- Later review: deterministic generated SorLA consequences.
- Final generation: `answers.json` v2, not full YAML.

## Target user experience

After the initial business prompt, the user should see something like:

```text
I found these records:
- Property
- Tenant
- Landlord
- Contractor
- Maintenance Request
- Quote
- Visit
- Invoice
- Payment

I found these processes:
- Tenant reports issue
- Contractor provides quote
- Landlord approves quote
- Contractor schedules visit
- Contractor completes work
- Invoice/payment is reconciled

Questions:
1. Can tenants see contractor quotes directly?
2. Can landlords approve automatically below a threshold?
3. Does the contractor invoice the landlord or the platform?
```

This is more useful than showing a full SorLA-oriented design plan too early.

## Proposed phase model

```rust
pub enum PromptPhase {
    AwaitingBusinessPrompt,
    ExtractingDomainModel,
    ReviewingDomainModel,
    AskingTargetedQuestions,
    CompilingExpandedPlan,
    ReviewingExpandedPlan,
    GeneratingAnswers,
    Completed,
}
```

If preserving enum compatibility is hard, add compatibility mapping from old phases to new ones.

## Phase responsibilities

### AwaitingBusinessPrompt

Collect prompt from CLI arg, file, or stdin.

### ExtractingDomainModel

LLM call:

```text
business prompt -> DomainIntent + assumptions + questions
```

The output should be valid `AnswersV2` create mode or a domain extraction schema that can be embedded into `AnswersV2`.

### ReviewingDomainModel

Show human-readable business-domain plan, not full SorLA.

Include:

- actors
- records
- fields
- relationships
- lifecycles
- processes
- assumptions
- questions

### AskingTargetedQuestions

Collect only high-value clarification answers.

Prioritize questions that affect:

- record existence
- relationships
- lifecycle transitions
- permissions
- payment/compliance/business risk

Do not ask for details that deterministic defaults can safely infer.

### CompilingExpandedPlan

Run deterministic compiler:

```text
AnswersV2 domain -> ExpandedSorlaPlan
```

### ReviewingExpandedPlan

Show generated consequences summary:

```text
Generated from 8 records:
- 40 CRUD actions
- 24 record events
- 9 lifecycle events
- 24 agent endpoints
- 16 projections
- 12 metrics
- 18 default policies
```

Also show warnings:

```text
- contractor is currently an actor but not a record
- payment was mentioned but no payment provider was specified
```

### GeneratingAnswers

Write `answers.json` v2.

Optionally include `derived_preview` if useful, but avoid requiring the LLM to author it.

### Completed

Print next command:

```bash
greentic-sorla wizard --answers answers.json --pack-out answers.gtpack
```

## Prompt changes

Update `prompt_authoring_system_prompt(...)` so it requests domain extraction only.

Update `answer_generation_system_prompt(...)` so it emits `sorla.answers.v2`.

Important instruction:

```text
Do not emit derived CRUD actions, events, projections, metrics or agent endpoints. The SorLA compiler will generate these deterministically from records, relationships and lifecycles.
```

## Repair behavior

Repair should operate on smaller JSON schemas.

Instead of one large repair prompt for the full answers document, use:

```text
repair_domain_extraction_output
repair_answers_v2_output
repair_semantic_operations_output
```

Each repair prompt should include:

- original invalid JSON
- validation errors
- target schema summary
- instruction to return only corrected JSON

## Backward compatibility

If an old session state resumes:

- map `ReviewingDesignPlan` to `ReviewingDomainModel` if no expanded plan exists.
- map `ReadyToGenerateAnswers` to `GeneratingAnswers`.
- preserve existing CLI behavior.

## CLI output changes

Add output similar to:

```text
Domain extraction complete.
Found 6 records, 4 actors, 2 processes, 5 assumptions, 3 questions.
```

Then after compilation:

```text
SorLA expansion complete.
Generated 30 actions, 21 events, 18 agent endpoints, 12 projections, 9 metrics and 14 default policies.
```

## Tests

Add tests for:

- initial prompt produces domain model.
- domain review text includes records and questions.
- answers generation emits v2.
- malformed v2 is repaired.
- compiler phase runs before final answers are written.
- old session phase resumes safely.

## Acceptance criteria

- Prompt engine no longer expects LLM to generate mechanical SorLA sections.
- New phases are implemented or compatibly simulated.
- User review focuses first on business domain.
- Final `answers.json` is v2 for new prompt sessions.
- Existing session behavior does not break.
