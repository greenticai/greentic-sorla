# PR-08: Implementation Order and Codex Master Prompt

## Summary

This PR is not a code change. It defines the recommended implementation order and provides a Codex prompt to implement the SorLA prompt/wizard refactor safely.

## Recommended implementation order

Implement in this order:

1. `PR-01`: Add `answers.json` v2 semantic model.
2. `PR-02`: Add deterministic compiler and expanders.
3. `PR-06`: Integrate v2 with wizard while preserving legacy answers.
4. `PR-03`: Refactor update mode to semantic delta operations.
5. `PR-04`: Refactor prompt engine phases.
6. `PR-05`: Add optional bounded reviewer agents.
7. `PR-07`: Add diagnostics and golden tests.

Reasoning:

- The v2 model must exist before the compiler.
- The compiler must exist before wizard integration.
- Wizard integration provides immediate value before prompt phase refactor.
- Update mode becomes safer once the compiler exists.
- Reviewer agents should come after deterministic generation is stable.
- Golden tests should be added throughout, but full coverage is easiest once the pipeline exists.

## Codex master prompt

Use this prompt when asking Codex to implement these changes:

```text
You are working in the Greentic SorLA repository.

Goal:
Refactor greentic-sorla prompt/wizard so answers.json becomes a semantic authoring and delta-intent document, while SorLA mechanics such as CRUD actions, record events, search endpoints, projections, metrics, default policies, migrations and agent endpoints are generated deterministically.

Important constraints:
- Do not remove support for the current legacy answers.json format.
- Do not ask permission repeatedly for routine implementation steps.
- Complete as much as possible in one pass.
- Avoid destructive changes unless clearly required.
- Keep changes incremental and well-tested.
- Prefer small modules and explicit tests.
- Do not make the LLM generate sorla.yaml directly.
- Do not make the LLM generate derived CRUD/actions/events/projections unless explicitly marked as manual overrides.

Files mentioned by the current implementation:
- CLI prompt entry point: crates/greentic-sorla-lib/src/lib.rs around run_prompt, approx line 4927.
- Prompt state machine: crates/greentic-sorla-lib/src/prompt/engine.rs around line 122.
- Answer generation: generate_answers_with_repair in prompt/engine.rs around line 367.
- CLI LLM implementation: CliPromptLlm in crates/greentic-sorla-lib/src/lib.rs around line 5535.
- Wizard resolved model and rendering: crates/greentic-sorla-lib/src/lib.rs around lines 8048 and 9440.
- Existing update mode: crates/greentic-sorla-lib/src/lib.rs around line 5385.

Implementation stages:

1. Add `sorla.answers.v2` model.
Create `crates/greentic-sorla-lib/src/prompt/answers_v2.rs`.
Add structs for:
- AnswersV2
- AnswersMode
- AuthoringIntent
- DomainIntent
- ActorIntent
- RecordIntent
- FieldIntent
- RelationshipIntent
- LifecycleIntent
- StateTransitionIntent
- ProcessIntent
- BusinessRuleIntent
- SemanticOperation
- CompilerOptions

Include validation helpers:
- validate_answers_v2
- is_answers_v2_json

2. Add deterministic compiler.
Create:
- crates/greentic-sorla-lib/src/compiler/mod.rs
- crates/greentic-sorla-lib/src/compiler/plan.rs
- crates/greentic-sorla-lib/src/compiler/expanders.rs
- crates/greentic-sorla-lib/src/compiler/provenance.rs

Add:
- ExpandedSorlaPlan
- ExpansionContext
- Provenance
- CompileDiagnostic
- SorlaExpander trait

Initial expanders:
- RecordCrudExpander
- RecordEventExpander
- LifecycleEventExpander
- SearchEndpointExpander
- AgentEndpointExpander
- ProjectionExpander
- MetricExpander
- PolicyDefaultExpander
- MigrationExpander

3. Integrate v2 with wizard.
When `wizard --answers` reads JSON:
- if `version == "sorla.answers.v2"`, use the new compiler path.
- otherwise use the existing legacy path.

For v2 create mode:
- compile domain to ExpandedSorlaPlan.
- convert to existing resolved model/rendering path.

For v2 update mode:
- require `--sorla-yaml`.
- parse existing SorLA model.
- apply semantic operations.
- run deterministic expanders.
- render updated YAML.

4. Refactor update mode.
When `prompt --sorla-yaml` is used:
- LLM should produce `AnswersV2 { mode: update, operations: [...] }`.
- Prompt must instruct LLM not to emit derived sections.
- Destructive operations should be blocked unless explicit.

5. Refactor prompt phases.
Prefer phase responsibilities:
- AwaitingBusinessPrompt
- ExtractingDomainModel
- ReviewingDomainModel
- AskingTargetedQuestions
- CompilingExpandedPlan
- ReviewingExpandedPlan
- GeneratingAnswers
- Completed

6. Add optional reviewer agents later.
Do not block core implementation on reviewer agents.

Testing requirements:
- Unit tests for AnswersV2 parsing and validation.
- Unit tests for each expander.
- Integration tests for v2 create mode.
- Integration tests for v2 update mode adding a record.
- Ensure legacy answers still work.
- Ensure generated ordering is deterministic.

Acceptance criteria:
- Existing legacy wizard behavior still works.
- V2 answers can create a new SorLA YAML.
- V2 update answers can patch an existing SorLA YAML.
- CRUD/actions/events/projections/agent endpoints are generated deterministically.
- Prompt LLM no longer needs to generate full SorLA mechanics.
```

## Suggested branch names

```text
feature/sorla-answers-v2
feature/sorla-deterministic-compiler
feature/sorla-semantic-update-mode
feature/sorla-prompt-phase-refactor
feature/sorla-review-agents
feature/sorla-golden-tests
```

## Milestone definition

### Milestone 1

- AnswersV2 parses and validates.
- Compiler generates CRUD/events/search/agent endpoints.
- Wizard can consume v2 create answers.

### Milestone 2

- Update mode emits semantic operations.
- Wizard applies v2 update operations to existing YAML.
- Diff preview works.

### Milestone 3

- Prompt phases are refactored.
- Completeness reviewer is added.
- Golden tests cover create/update flows.

