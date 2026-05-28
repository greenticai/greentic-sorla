use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptQuestion {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub help: Option<String>,
    pub answer_kind: PromptAnswerKind,
    #[serde(default = "default_question_required")]
    pub required: bool,
    #[serde(default)]
    pub risk: PromptQuestionRisk,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

fn default_question_required() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum PromptAnswerKind {
    FreeText,
    Boolean,
    SingleChoice { choices: Vec<String> },
    MultiChoice { choices: Vec<String> },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptQuestionRisk {
    Low,
    #[default]
    Medium,
    High,
}
