use serde::Serialize;

use crate::ProductDescriptor;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaFlow {
    Create,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardSchema {
    pub schema_version: &'static str,
    pub wizard_version: &'static str,
    pub package_version: &'static str,
    pub locale: String,
    pub fallback_locale: &'static str,
    pub supported_modes: Vec<SchemaFlow>,
    pub product: ProductDescriptor,
    pub update_metadata: WizardUpdateMetadata,
    pub sections: Vec<WizardSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardUpdateMetadata {
    pub supports_partial_answers: bool,
    pub generated_content_strategy: &'static str,
    pub user_content_strategy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardSection {
    pub id: String,
    pub title_key: String,
    pub description_key: String,
    pub flows: Vec<SchemaFlow>,
    pub questions: Vec<WizardQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardQuestion {
    pub id: String,
    pub label_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_key: Option<String>,
    pub kind: WizardQuestionKind,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    pub choices: Vec<WizardChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<SchemaVisibility>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WizardQuestionKind {
    Text,
    TextList,
    Boolean,
    SingleSelect,
    MultiSelect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardChoice {
    pub value: String,
    pub label_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SchemaVisibility {
    pub depends_on: String,
    pub equals: String,
}
