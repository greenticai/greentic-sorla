use greentic_sorla_lang::{PRODUCT_LINE, PRODUCT_NAME, product_boundary};
use greentic_sorla_pack::{PackageManifest, scaffold_manifest};
use serde::Serialize;
use std::path::PathBuf;

pub mod schema;

pub use schema::{
    SchemaFlow, SchemaVisibility, WizardChoice, WizardQuestion, WizardQuestionKind, WizardSchema,
    WizardSection, WizardUpdateMetadata,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductDescriptor {
    pub name: &'static str,
    pub product_line: &'static str,
    pub providers_live_in: &'static str,
    pub package_manifest: PackageManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnswersPreview {
    pub mode: &'static str,
    pub answers_path: String,
    pub next_step: &'static str,
}

pub fn default_schema() -> WizardSchema {
    let boundary = product_boundary();
    let package_manifest = scaffold_manifest();
    let cli_schema = greentic_sorla_cli::default_schema();

    WizardSchema {
        schema_version: cli_schema.schema_version,
        wizard_version: cli_schema.wizard_version,
        package_version: cli_schema.package_version,
        locale: cli_schema.locale,
        fallback_locale: cli_schema.fallback_locale,
        supported_modes: cli_schema
            .supported_modes
            .into_iter()
            .map(convert_flow)
            .collect(),
        product: ProductDescriptor {
            name: PRODUCT_NAME,
            product_line: PRODUCT_LINE,
            providers_live_in: boundary.providers_live_in,
            package_manifest: package_manifest.clone(),
        },
        update_metadata: WizardUpdateMetadata {
            supports_partial_answers: true,
            generated_content_strategy: cli_schema.generated_content_strategy,
            user_content_strategy: cli_schema.user_content_strategy,
        },
        sections: cli_schema
            .sections
            .into_iter()
            .map(convert_section)
            .collect(),
    }
}

pub fn schema_for_answers(path: PathBuf) -> AnswersPreview {
    AnswersPreview {
        mode: "answers-preview",
        answers_path: path.display().to_string(),
        next_step: "answers execution lands in PR-05; this scaffold keeps the supported surface stable.",
    }
}

fn convert_flow(flow: greentic_sorla_cli::SchemaFlow) -> SchemaFlow {
    match flow {
        greentic_sorla_cli::SchemaFlow::Create => SchemaFlow::Create,
        greentic_sorla_cli::SchemaFlow::Update => SchemaFlow::Update,
    }
}

fn convert_section(section: greentic_sorla_cli::WizardSection) -> WizardSection {
    WizardSection {
        id: section.id.to_string(),
        title_key: section.title_key.to_string(),
        description_key: section.description_key.to_string(),
        flows: section.flows.into_iter().map(convert_flow).collect(),
        questions: section
            .questions
            .into_iter()
            .map(convert_question)
            .collect(),
    }
}

fn convert_question(question: greentic_sorla_cli::WizardQuestion) -> WizardQuestion {
    WizardQuestion {
        id: question.id.to_string(),
        label_key: question.label_key.to_string(),
        help_key: question.help_key.map(str::to_string),
        kind: convert_question_kind(question.kind),
        required: question.required,
        default_value: question.default_value.map(str::to_string),
        choices: question.choices.into_iter().map(convert_choice).collect(),
        visibility: question.visibility.map(convert_visibility),
    }
}

fn convert_question_kind(kind: greentic_sorla_cli::WizardQuestionKind) -> WizardQuestionKind {
    match kind {
        greentic_sorla_cli::WizardQuestionKind::Text => WizardQuestionKind::Text,
        greentic_sorla_cli::WizardQuestionKind::TextList => WizardQuestionKind::TextList,
        greentic_sorla_cli::WizardQuestionKind::Boolean => WizardQuestionKind::Boolean,
        greentic_sorla_cli::WizardQuestionKind::SingleSelect => WizardQuestionKind::SingleSelect,
        greentic_sorla_cli::WizardQuestionKind::MultiSelect => WizardQuestionKind::MultiSelect,
    }
}

fn convert_choice(choice: greentic_sorla_cli::WizardChoice) -> WizardChoice {
    WizardChoice {
        value: choice.value.to_string(),
        label_key: choice.label_key.to_string(),
    }
}

fn convert_visibility(visibility: greentic_sorla_cli::SchemaVisibility) -> SchemaVisibility {
    SchemaVisibility {
        depends_on: visibility.depends_on.to_string(),
        equals: visibility.equals.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_deterministic() {
        let first = serde_json::to_string(&default_schema()).unwrap();
        let second = serde_json::to_string(&default_schema()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn schema_supports_create_and_update_flows() {
        let schema = default_schema();
        assert!(schema.supported_modes.contains(&SchemaFlow::Create));
        assert!(schema.supported_modes.contains(&SchemaFlow::Update));
        assert!(schema.sections.iter().any(|section| {
            section.id == "package-bootstrap" && section.flows.contains(&SchemaFlow::Create)
        }));
        assert!(schema.sections.iter().any(|section| {
            section.id == "package-update" && section.flows.contains(&SchemaFlow::Update)
        }));
    }

    #[test]
    fn schema_keeps_provider_repo_out_of_this_repo() {
        let schema = default_schema();
        assert_eq!(schema.product.providers_live_in, "greentic-sorla-providers");
        assert!(
            schema
                .sections
                .iter()
                .any(|section| section.id == "provider-requirements")
        );
    }
}
