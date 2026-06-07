use super::{PromptQuestion, SorDesignDraft};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmCapabilityConfig {
    pub provider: String,
    pub model: Option<String>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub capability_id: Option<String>,
}

impl fmt::Debug for LlmCapabilityConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LlmCapabilityConfig")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("endpoint", &self.endpoint)
            .field("capability_id", &self.capability_id)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptSessionConfig {
    pub locale: Option<String>,
    pub schema_version: Option<String>,
    pub package_name_hint: Option<String>,
    pub package_version_hint: Option<String>,
    pub llm: LlmCapabilityConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptSessionState {
    pub session_id: String,
    pub phase: PromptPhase,
    #[serde(default)]
    pub llm: Option<LlmCapabilityConfig>,
    pub business_prompt: Option<String>,
    pub answers_so_far: Vec<PromptAnswer>,
    #[serde(default)]
    pub questions: Vec<PromptQuestion>,
    pub assumptions: Vec<PromptAssumption>,
    pub draft_model: Option<SorDesignDraft>,
    #[serde(default)]
    pub staged_answers: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptPhase {
    AwaitingBusinessPrompt,
    ExtractingDomainModel,
    ReviewingDomainModel,
    AskingTargetedQuestions,
    CompilingExpandedPlan,
    ReviewingExpandedPlan,
    GeneratingAnswers,
    AskingQuestions,
    ReviewingDesignPlan,
    ReadyToGenerateAnswers,
    Completed,
}

impl PromptPhase {
    pub fn is_question_phase(self) -> bool {
        matches!(self, Self::AskingTargetedQuestions | Self::AskingQuestions)
    }

    pub fn is_review_phase(self) -> bool {
        matches!(
            self,
            Self::ReviewingDomainModel
                | Self::ReviewingExpandedPlan
                | Self::ReviewingDesignPlan
                | Self::ReadyToGenerateAnswers
                | Self::GeneratingAnswers
        )
    }

    pub fn is_generation_phase(self) -> bool {
        matches!(self, Self::ReadyToGenerateAnswers | Self::GeneratingAnswers)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptTurnInput {
    pub session: PromptSessionState,
    pub user_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptTurnOutput {
    pub session: PromptSessionState,
    pub assistant_message: String,
    pub next_questions: Vec<PromptQuestion>,
    pub design_plan: Option<SorDesignDraft>,
    pub answers_document: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptAnswer {
    pub question_id: String,
    pub value: PromptAnswerValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum PromptAnswerValue {
    FreeText(String),
    Boolean(bool),
    SingleChoice(String),
    MultiChoice(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptAssumption {
    pub id: String,
    pub text: String,
    pub confidence: PromptAssumptionConfidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptAssumptionConfidence {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{
        DraftField, DraftRecord, PromptAnswerKind, PromptQuestion, PromptQuestionRisk,
    };

    #[test]
    fn prompt_session_state_round_trips_json() {
        let state = PromptSessionState {
            session_id: "session-1".to_string(),
            phase: PromptPhase::AskingQuestions,
            llm: None,
            business_prompt: Some("Track rental properties".to_string()),
            answers_so_far: vec![PromptAnswer {
                question_id: "lease.multiple_tenants".to_string(),
                value: PromptAnswerValue::Boolean(true),
            }],
            questions: Vec::new(),
            assumptions: vec![PromptAssumption {
                id: "tenant-record".to_string(),
                text: "Tenants should be represented as records.".to_string(),
                confidence: PromptAssumptionConfidence::High,
            }],
            draft_model: Some(SorDesignDraft {
                summary: "Landlord tenant system".to_string(),
                records: vec![DraftRecord {
                    name: "tenant".to_string(),
                    description: Some("A renter on a lease.".to_string()),
                    fields: vec![DraftField {
                        name: "email".to_string(),
                        type_name: "string".to_string(),
                        required: true,
                        sensitive: true,
                        description: None,
                        rules: None,
                    }],
                }],
                ..SorDesignDraft::default()
            }),
            staged_answers: true,
        };

        let encoded = serde_json::to_string(&state).expect("state serializes");
        let decoded: PromptSessionState =
            serde_json::from_str(&encoded).expect("state deserializes");
        assert_eq!(decoded, state);
    }

    #[test]
    fn llm_config_debug_redacts_api_key() {
        let config = LlmCapabilityConfig {
            provider: "openai".to_string(),
            model: Some("test-model".to_string()),
            api_key: Some("secret-key".to_string()),
            endpoint: None,
            capability_id: None,
        };

        let rendered = format!("{config:?}");
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("secret-key"));
    }

    #[test]
    fn prompt_question_round_trips_json() {
        let question = PromptQuestion {
            id: "lease.liability".to_string(),
            text: "Should tenant liability be joint, individual, or both?".to_string(),
            help: Some("This affects generated policies and approvals.".to_string()),
            answer_kind: PromptAnswerKind::SingleChoice {
                choices: vec![
                    "joint".to_string(),
                    "individual".to_string(),
                    "both".to_string(),
                ],
            },
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["lease.multiple_tenants".to_string()],
        };

        let encoded = serde_json::to_string(&question).expect("question serializes");
        let decoded: PromptQuestion =
            serde_json::from_str(&encoded).expect("question deserializes");
        assert_eq!(decoded, question);
    }
}
