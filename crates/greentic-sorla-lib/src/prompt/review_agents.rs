use crate::compiler::{CompileDiagnostic, ExpandedSorlaPlan};
use crate::prompt::{ClarificationQuestion, DomainIntent, LlmMessage, LlmRole, SemanticOperation};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptReviewAgentConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub agents: Vec<PromptReviewAgentKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptReviewAgentKind {
    Completeness,
    Policy,
    Metrics,
    UpdateImpact,
}

pub trait PromptReviewAgent {
    type Input;
    type Output;

    fn name(&self) -> &'static str;
    fn build_messages(&self, input: &Self::Input) -> Vec<LlmMessage>;
    fn parse_output(&self, raw: &str) -> Result<Self::Output, ReviewAgentError>;
    fn validate_output(&self, output: &Self::Output) -> Result<(), ReviewAgentError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewAgentError {
    pub message: String,
}

impl ReviewAgentError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ReviewAgentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for ReviewAgentError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSuggestion {
    pub name: String,
    pub reason: String,
    pub confidence: ReviewConfidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewWarning {
    pub message: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletenessReviewInput {
    pub original_prompt: String,
    pub domain: DomainIntent,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletenessReviewOutput {
    #[serde(default)]
    pub missing_records: Vec<ReviewSuggestion>,
    #[serde(default)]
    pub missing_actors: Vec<ReviewSuggestion>,
    #[serde(default)]
    pub missing_relationships: Vec<ReviewSuggestion>,
    #[serde(default)]
    pub missing_processes: Vec<ReviewSuggestion>,
    #[serde(default)]
    pub questions: Vec<ClarificationQuestion>,
    #[serde(default)]
    pub warnings: Vec<ReviewWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyReviewInput {
    pub domain: DomainIntent,
    pub generated_policies: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyReviewOutput {
    #[serde(default)]
    pub warnings: Vec<ReviewWarning>,
    #[serde(default)]
    pub suggested_policy_intents: Vec<PolicyIntentSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyIntentSuggestion {
    pub name: String,
    pub description: String,
    pub applies_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricReviewInput {
    pub original_prompt: String,
    pub domain: DomainIntent,
    pub generated_metrics: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricReviewOutput {
    #[serde(default)]
    pub suggested_metric_intents: Vec<MetricIntentSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricIntentSuggestion {
    pub name: String,
    pub description: String,
    pub applies_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateImpactReviewInput {
    pub existing_model_summary: String,
    pub requested_operations: Vec<SemanticOperation>,
    pub expanded_plan: Option<ExpandedSorlaPlan>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateImpactReviewOutput {
    #[serde(default)]
    pub warnings: Vec<ReviewWarning>,
    #[serde(default)]
    pub additional_questions: Vec<ClarificationQuestion>,
    #[serde(default)]
    pub potential_conflicts: Vec<ReviewWarning>,
}

pub struct CompletenessReviewerAgent;
pub struct PolicyReviewerAgent;
pub struct MetricReviewerAgent;
pub struct UpdateImpactReviewerAgent;

impl PromptReviewAgent for CompletenessReviewerAgent {
    type Input = CompletenessReviewInput;
    type Output = CompletenessReviewOutput;

    fn name(&self) -> &'static str {
        "completeness"
    }

    fn build_messages(&self, input: &Self::Input) -> Vec<LlmMessage> {
        review_messages(
            "Review this extracted SorLA domain for missing business concepts. Return only bounded JSON with suggestions, warnings, and clarification questions.",
            &serde_json::json!({
                "original_prompt": input.original_prompt,
                "domain": input.domain
            }),
        )
    }

    fn parse_output(&self, raw: &str) -> Result<Self::Output, ReviewAgentError> {
        parse_review_output(raw)
    }

    fn validate_output(&self, output: &Self::Output) -> Result<(), ReviewAgentError> {
        validate_suggestions("missing_records", &output.missing_records)?;
        validate_questions(&output.questions)
    }
}

impl PromptReviewAgent for PolicyReviewerAgent {
    type Input = PolicyReviewInput;
    type Output = PolicyReviewOutput;

    fn name(&self) -> &'static str {
        "policy"
    }

    fn build_messages(&self, input: &Self::Input) -> Vec<LlmMessage> {
        review_messages(
            "Review policy risks in this SorLA domain. Do not emit YAML or CRUD mechanics. Return only bounded JSON.",
            &serde_json::json!({
                "domain": input.domain,
                "generated_policies": input.generated_policies
            }),
        )
    }

    fn parse_output(&self, raw: &str) -> Result<Self::Output, ReviewAgentError> {
        parse_review_output(raw)
    }

    fn validate_output(&self, output: &Self::Output) -> Result<(), ReviewAgentError> {
        for (index, warning) in output.warnings.iter().enumerate() {
            require_non_empty(format!("warnings[{index}].message"), &warning.message)?;
        }
        for (index, suggestion) in output.suggested_policy_intents.iter().enumerate() {
            require_non_empty(
                format!("suggested_policy_intents[{index}].name"),
                &suggestion.name,
            )?;
            require_non_empty(
                format!("suggested_policy_intents[{index}].description"),
                &suggestion.description,
            )?;
        }
        Ok(())
    }
}

impl PromptReviewAgent for MetricReviewerAgent {
    type Input = MetricReviewInput;
    type Output = MetricReviewOutput;

    fn name(&self) -> &'static str {
        "metrics"
    }

    fn build_messages(&self, input: &Self::Input) -> Vec<LlmMessage> {
        review_messages(
            "Suggest business-specific metric intents for this SorLA domain. Do not emit derived count metrics or YAML. Return only bounded JSON.",
            &serde_json::json!({
                "original_prompt": input.original_prompt,
                "domain": input.domain,
                "generated_metrics": input.generated_metrics
            }),
        )
    }

    fn parse_output(&self, raw: &str) -> Result<Self::Output, ReviewAgentError> {
        parse_review_output(raw)
    }

    fn validate_output(&self, output: &Self::Output) -> Result<(), ReviewAgentError> {
        for (index, suggestion) in output.suggested_metric_intents.iter().enumerate() {
            require_non_empty(
                format!("suggested_metric_intents[{index}].name"),
                &suggestion.name,
            )?;
            require_non_empty(
                format!("suggested_metric_intents[{index}].description"),
                &suggestion.description,
            )?;
        }
        Ok(())
    }
}

impl PromptReviewAgent for UpdateImpactReviewerAgent {
    type Input = UpdateImpactReviewInput;
    type Output = UpdateImpactReviewOutput;

    fn name(&self) -> &'static str {
        "update-impact"
    }

    fn build_messages(&self, input: &Self::Input) -> Vec<LlmMessage> {
        review_messages(
            "Review the impact of semantic SorLA update operations. Do not mutate the plan. Return only bounded JSON warnings, conflicts, and questions.",
            &serde_json::json!({
                "existing_model_summary": input.existing_model_summary,
                "requested_operations": input.requested_operations,
                "expanded_plan": input.expanded_plan
            }),
        )
    }

    fn parse_output(&self, raw: &str) -> Result<Self::Output, ReviewAgentError> {
        parse_review_output(raw)
    }

    fn validate_output(&self, output: &Self::Output) -> Result<(), ReviewAgentError> {
        for (index, warning) in output.warnings.iter().enumerate() {
            require_non_empty(format!("warnings[{index}].message"), &warning.message)?;
        }
        validate_questions(&output.additional_questions)
    }
}

pub fn review_warnings_to_compile_diagnostics(
    code: &'static str,
    warnings: &[ReviewWarning],
) -> Vec<CompileDiagnostic> {
    warnings
        .iter()
        .map(|warning| CompileDiagnostic::warning(code, warning.message.clone()))
        .collect()
}

fn review_messages(system_prompt: &str, payload: &serde_json::Value) -> Vec<LlmMessage> {
    vec![
        LlmMessage {
            role: LlmRole::System,
            content: system_prompt.to_string(),
        },
        LlmMessage {
            role: LlmRole::User,
            content: serde_json::to_string_pretty(payload).unwrap_or_else(|_| payload.to_string()),
        },
    ]
}

fn parse_review_output<T>(raw: &str) -> Result<T, ReviewAgentError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(raw).map_err(|error| ReviewAgentError::new(error.to_string()))
}

fn validate_suggestions(
    path: &str,
    suggestions: &[ReviewSuggestion],
) -> Result<(), ReviewAgentError> {
    for (index, suggestion) in suggestions.iter().enumerate() {
        require_non_empty(format!("{path}[{index}].name"), &suggestion.name)?;
        require_non_empty(format!("{path}[{index}].reason"), &suggestion.reason)?;
    }
    Ok(())
}

fn validate_questions(questions: &[ClarificationQuestion]) -> Result<(), ReviewAgentError> {
    for (index, question) in questions.iter().enumerate() {
        require_non_empty(format!("questions[{index}].id"), &question.id)?;
        require_non_empty(format!("questions[{index}].question"), &question.question)?;
    }
    Ok(())
}

fn require_non_empty(path: String, value: &str) -> Result<(), ReviewAgentError> {
    if value.trim().is_empty() {
        Err(ReviewAgentError::new(format!("{path}: must not be empty")))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completeness_reviewer_parses_and_validates_bounded_output() {
        let agent = CompletenessReviewerAgent;
        let output = agent
            .parse_output(
                r#"{
                  "missing_records": [
                    { "name": "quote", "reason": "Quote approval is mentioned", "confidence": "high" }
                  ],
                  "questions": [
                    { "id": "quote_visibility", "question": "Can tenants see quotes?", "required": false }
                  ]
                }"#,
            )
            .expect("review parses");
        agent.validate_output(&output).expect("review validates");
        assert_eq!(output.missing_records[0].name, "quote");
    }

    #[test]
    fn policy_reviewer_rejects_empty_policy_suggestions() {
        let agent = PolicyReviewerAgent;
        let output = agent
            .parse_output(
                r#"{
                  "suggested_policy_intents": [
                    { "name": "", "description": "Limit access", "applies_to": "request" }
                  ]
                }"#,
            )
            .expect("review parses");
        let error = agent
            .validate_output(&output)
            .expect_err("review should fail");
        assert!(
            error
                .to_string()
                .contains("suggested_policy_intents[0].name")
        );
    }

    #[test]
    fn metric_reviewer_builds_bounded_messages_without_yaml_instruction() {
        let agent = MetricReviewerAgent;
        let messages = agent.build_messages(&MetricReviewInput {
            original_prompt: "Track contractor response time".to_string(),
            domain: DomainIntent::default(),
            generated_metrics: vec!["count_request".to_string()],
        });
        assert_eq!(messages[0].role, LlmRole::System);
        assert!(messages[0].content.contains("Do not emit"));
        assert!(messages[1].content.contains("count_request"));
    }

    #[test]
    fn update_impact_warnings_convert_to_compile_diagnostics() {
        let diagnostics = review_warnings_to_compile_diagnostics(
            "SORLA_UPDATE_REVIEW_WARNING",
            &[ReviewWarning {
                message: "Adding quote approval requires a contractor concept.".to_string(),
                reason: None,
            }],
        );
        assert_eq!(
            diagnostics[0].severity,
            crate::compiler::DiagnosticSeverity::Warning
        );
        assert_eq!(diagnostics[0].code, "SORLA_UPDATE_REVIEW_WARNING");
    }
}
