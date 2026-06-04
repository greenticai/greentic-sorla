use crate::SorlaError;
use serde::{Deserialize, Serialize};

pub const DEFAULT_LLM_CAPABILITY_ID: &str = "greentic.cap.llm";

pub trait LlmCapability {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SorlaError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmRequest {
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub system_prompt: String,
    pub messages: Vec<LlmMessage>,
    pub response_format: Option<LlmResponseFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: LlmRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LlmRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LlmResponseFormat {
    Text,
    Json,
    JsonSchema {
        name: String,
        schema: serde_json::Value,
        strict: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<LlmTokenUsage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmTokenUsage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeLlmCapability;

    impl LlmCapability for FakeLlmCapability {
        fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SorlaError> {
            let last_user_message = request
                .messages
                .iter()
                .rev()
                .find(|message| message.role == LlmRole::User)
                .map(|message| message.content.as_str())
                .unwrap_or_default();

            Ok(LlmResponse {
                content: format!("fake response for {last_user_message}"),
                usage: None,
            })
        }
    }

    #[test]
    fn fake_llm_capability_can_complete_request() {
        let capability: &dyn LlmCapability = &FakeLlmCapability;
        let response = capability
            .complete(LlmRequest {
                provider: "fake".to_string(),
                model: None,
                api_key: None,
                endpoint: None,
                system_prompt: "Extract a design draft.".to_string(),
                messages: vec![LlmMessage {
                    role: LlmRole::User,
                    content: "landlord tenant".to_string(),
                }],
                response_format: Some(LlmResponseFormat::Json),
            })
            .expect("fake completion succeeds");

        assert_eq!(response.content, "fake response for landlord tenant");
    }

    #[test]
    fn llm_request_round_trips_json() {
        let request = LlmRequest {
            provider: "fake".to_string(),
            model: Some("fixture".to_string()),
            api_key: None,
            endpoint: None,
            system_prompt: "Ask concise questions.".to_string(),
            messages: vec![
                LlmMessage {
                    role: LlmRole::System,
                    content: "system".to_string(),
                },
                LlmMessage {
                    role: LlmRole::User,
                    content: "business prompt".to_string(),
                },
            ],
            response_format: Some(LlmResponseFormat::Json),
        };

        let encoded = serde_json::to_string(&request).expect("request serializes");
        let decoded: LlmRequest = serde_json::from_str(&encoded).expect("request deserializes");
        assert_eq!(decoded, request);
    }
}
