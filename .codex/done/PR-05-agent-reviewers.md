# PR-05: Add Bounded Reviewer Agents for Completeness, Policy, and Metrics

## Summary

Add optional bounded LLM reviewer passes after domain extraction and before final answers generation.

These reviewer agents do not generate SorLA YAML and do not generate mechanical CRUD/events/projections. They only produce structured suggestions, warnings, or semantic additions.

Initial reviewers:

```text
CompletenessReviewerAgent
PolicyReviewerAgent
MetricReviewerAgent
UpdateImpactReviewerAgent
```

## Motivation

A single prompt trying to produce everything is unreliable. However, LLMs are still valuable for judgment-based tasks, such as:

- spotting missing business records.
- identifying unclear relationships.
- warning about risky permissions.
- suggesting useful business metrics.
- checking impact of updates.

These tasks should be bounded by small schemas.

## Non-goals

- Do not create autonomous free-roaming agents.
- Do not let reviewers mutate the plan directly without structured output.
- Do not use reviewers to generate CRUD/events/search endpoints.

## Agent interface

Add:

```text
crates/greentic-sorla-lib/src/prompt/review_agents.rs
```

Suggested trait:

```rust
pub trait PromptReviewAgent {
    type Input;
    type Output;

    fn name(&self) -> &'static str;
    fn build_messages(&self, input: &Self::Input) -> Vec<PromptMessage>;
    fn parse_output(&self, raw: &str) -> Result<Self::Output, PromptError>;
    fn validate_output(&self, output: &Self::Output) -> Result<(), PromptError>;
}
```

Or keep this simpler if current prompt engine patterns make generics awkward.

## CompletenessReviewerAgent

### Input

```rust
pub struct CompletenessReviewInput {
    pub original_prompt: String,
    pub domain: DomainIntent,
}
```

### Output

```rust
pub struct CompletenessReviewOutput {
    pub missing_records: Vec<ReviewSuggestion>,
    pub missing_actors: Vec<ReviewSuggestion>,
    pub missing_relationships: Vec<ReviewSuggestion>,
    pub missing_processes: Vec<ReviewSuggestion>,
    pub questions: Vec<ClarificationQuestion>,
    pub warnings: Vec<ReviewWarning>,
}
```

Example:

```json
{
  "missing_records": [
    {
      "name": "quote",
      "reason": "The prompt mentions contractor quote approval but no quote record exists",
      "confidence": "high"
    }
  ],
  "questions": [
    {
      "id": "quote_visibility",
      "question": "Can tenants see contractor quotes directly?",
      "reason": "This affects policies and projections",
      "required": false
    }
  ]
}
```

## PolicyReviewerAgent

### Input

```rust
pub struct PolicyReviewInput {
    pub domain: DomainIntent,
    pub generated_policies: Vec<PolicyPlan>,
}
```

### Output

```rust
pub struct PolicyReviewOutput {
    pub warnings: Vec<PolicyWarning>,
    pub suggested_policy_intents: Vec<PolicyIntentSuggestion>,
}
```

Examples:

```text
Tenants should only see their own maintenance requests.
Contractors should not see all tenant contact details.
Landlords should only approve quotes for properties they own.
```

## MetricReviewerAgent

### Input

```rust
pub struct MetricReviewInput {
    pub original_prompt: String,
    pub domain: DomainIntent,
    pub generated_metrics: Vec<MetricPlan>,
}
```

### Output

```rust
pub struct MetricReviewOutput {
    pub suggested_metric_intents: Vec<MetricIntentSuggestion>,
}
```

Examples:

```text
average contractor response time
quote approval rate
repeat issue rate per property
maintenance cost per property
```

## UpdateImpactReviewerAgent

Used in update mode.

### Input

```rust
pub struct UpdateImpactReviewInput {
    pub existing_model_summary: ExistingSorlaModelSummary,
    pub requested_operations: Vec<SemanticOperation>,
}
```

### Output

```rust
pub struct UpdateImpactReviewOutput {
    pub warnings: Vec<ReviewWarning>,
    pub additional_questions: Vec<ClarificationQuestion>,
    pub potential_conflicts: Vec<ReviewWarning>,
}
```

Example warning:

```text
Adding quote approval requires either a contractor record or an actor-only relationship strategy.
```

## Configuration

Add config flags:

```text
--enable-review-agents
--disable-review-agents
--review-agent completeness
--review-agent policy
--review-agent metrics
```

Default recommendation:

- enabled for cloud/high-capability LLM mode.
- disabled or limited for deterministic/local mode.

## Integration points

Pipeline:

```text
DomainExtractorAgent
  -> CompletenessReviewerAgent
  -> ask targeted questions
  -> DeterministicCompiler
  -> PolicyReviewerAgent
  -> MetricReviewerAgent
  -> final answers.json
```

Update mode:

```text
UpdatePlannerAgent
  -> UpdateImpactReviewerAgent
  -> DeterministicCompiler
```

## Tests

Use fake LLM responses.

Test cases:

- completeness reviewer suggests missing quote record.
- policy reviewer warns about cross-tenant visibility.
- metric reviewer adds business-specific metric intents.
- malformed reviewer JSON is repaired.
- reviewer suggestions can be accepted into domain or operations.
- reviewer suggestions can remain warnings only.

## Acceptance criteria

- Review agents have bounded schemas.
- Reviewers do not emit YAML.
- Reviewers do not emit mechanical derived SorLA sections.
- Reviewers can be enabled/disabled.
- Fake LLM tests cover normal and malformed outputs.
