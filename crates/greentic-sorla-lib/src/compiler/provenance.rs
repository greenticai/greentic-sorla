use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Provenance {
    UserProvided,
    LlmGenerated {
        agent: String,
        reason: Option<String>,
    },
    DeterministicRule {
        rule: String,
        source: String,
    },
    ExistingYaml {
        path: Option<String>,
    },
}

impl Provenance {
    pub fn deterministic(rule: impl Into<String>, source: impl Into<String>) -> Self {
        Self::DeterministicRule {
            rule: rule.into(),
            source: source.into(),
        }
    }
}
