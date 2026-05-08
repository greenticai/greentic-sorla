use clap::{ArgAction, Args, Parser, Subcommand};
use greentic_qa_lib::{
    AnswerProvider, I18nConfig, QaLibError, ResolvedI18nMap, WizardDriver, WizardFrontend,
    WizardRunConfig,
};
use greentic_sorla_pack::{
    SorlaGtpackOptions, build_sorla_gtpack, doctor_sorla_gtpack, inspect_sorla_gtpack,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

mod embedded_i18n;

const GENERATED_BEGIN: &str = "# --- BEGIN GREENTIC-SORLA GENERATED ---";
const GENERATED_END: &str = "# --- END GREENTIC-SORLA GENERATED ---";
const LOCK_FILENAME: &str = "answers.lock.json";

#[derive(Debug, Parser)]
#[command(
    name = "greentic-sorla",
    about = "Wizard-first tooling for Greentic SoRLa packages.",
    long_about = "greentic-sorla is a wizard-first tool for authoring SoRLa packages and deterministic handoff packs.\n\nSupported product surface:\n  greentic-sorla wizard --schema\n  greentic-sorla wizard --answers <file>\n  greentic-sorla wizard --answers <file> --pack-out <file.gtpack>\n  greentic-sorla pack <file> --name <name> --version <version> --out <file.gtpack>\n",
    after_help = "Internal helper commands may exist, but the supported UX is the wizard flow plus deterministic pack handoff."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate wizard schema or apply a saved answers document.
    Wizard(WizardArgs),
    /// Build, inspect, or doctor deterministic SoRLa gtpack handoff artifacts.
    Pack(PackArgs),
    #[command(name = "__inspect-product-shape", hide = true)]
    InspectProductShape,
}

#[derive(Debug, Args)]
struct WizardArgs {
    /// Emit the wizard schema as deterministic JSON.
    #[arg(long, action = ArgAction::SetTrue)]
    schema: bool,
    /// Apply a saved answers document.
    #[arg(long, value_name = "FILE")]
    answers: Option<PathBuf>,
    /// Also build a deterministic .gtpack from the generated sorla.yaml.
    #[arg(long, value_name = "FILE")]
    pack_out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct PackArgs {
    /// SoRLa YAML input to package.
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,
    /// Pack name to write into the gtpack manifest.
    #[arg(long)]
    name: Option<String>,
    /// Pack semantic version.
    #[arg(long)]
    version: Option<String>,
    /// Output .gtpack path.
    #[arg(long, value_name = "FILE")]
    out: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<PackCommand>,
}

#[derive(Debug, Subcommand)]
enum PackCommand {
    /// Validate a generated SoRLa gtpack.
    Doctor(PackPathArgs),
    /// Inspect a generated SoRLa gtpack as deterministic JSON.
    Inspect(PackPathArgs),
}

#[derive(Debug, Args)]
struct PackPathArgs {
    /// .gtpack file to inspect or validate.
    #[arg(value_name = "FILE")]
    path: PathBuf,
}

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
    pub provider_repo: &'static str,
    pub generated_content_strategy: &'static str,
    pub user_content_strategy: &'static str,
    pub artifact_references: Vec<&'static str>,
    pub sections: Vec<WizardSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardSection {
    pub id: &'static str,
    pub title_key: &'static str,
    pub description_key: &'static str,
    pub flows: Vec<SchemaFlow>,
    pub questions: Vec<WizardQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardQuestion {
    pub id: &'static str,
    pub label_key: &'static str,
    pub help_key: Option<&'static str>,
    pub kind: WizardQuestionKind,
    pub required: bool,
    pub default_value: Option<&'static str>,
    pub choices: Vec<WizardChoice>,
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
    pub value: &'static str,
    pub label_key: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SchemaVisibility {
    pub depends_on: &'static str,
    pub equals: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ExecutionSummary {
    mode: &'static str,
    output_dir: String,
    package_name: String,
    locale: String,
    written_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pack_path: Option<String>,
    preserved_user_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AnswersDocument {
    schema_version: String,
    flow: String,
    output_dir: String,
    #[serde(default)]
    locale: Option<String>,
    #[serde(default)]
    package: Option<PackageAnswers>,
    #[serde(default)]
    providers: Option<ProviderAnswers>,
    #[serde(default)]
    records: Option<RecordAnswers>,
    #[serde(default)]
    events: Option<EventAnswers>,
    #[serde(default)]
    projections: Option<ProjectionAnswers>,
    #[serde(default)]
    migrations: Option<MigrationAnswers>,
    #[serde(default)]
    agent_endpoints: Option<AgentEndpointAnswers>,
    #[serde(default)]
    output: Option<OutputAnswers>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct PackageAnswers {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ProviderAnswers {
    #[serde(default)]
    storage_category: Option<String>,
    #[serde(default)]
    external_ref_category: Option<String>,
    #[serde(default)]
    hints: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RecordAnswers {
    #[serde(default)]
    default_source: Option<String>,
    #[serde(default)]
    external_ref_system: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct EventAnswers {
    #[serde(default)]
    enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ProjectionAnswers {
    #[serde(default)]
    mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct MigrationAnswers {
    #[serde(default)]
    compatibility: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct AgentEndpointAnswers {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    ids: Option<Vec<String>>,
    #[serde(default)]
    default_risk: Option<String>,
    #[serde(default)]
    default_approval: Option<String>,
    #[serde(default)]
    exports: Option<Vec<String>>,
    #[serde(default)]
    provider_category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OutputAnswers {
    #[serde(default)]
    include_agent_tools: Option<bool>,
    #[serde(default)]
    artifacts: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ResolvedAnswers {
    schema_version: String,
    flow: String,
    output_dir: String,
    locale: String,
    package_name: String,
    package_version: String,
    storage_category: String,
    external_ref_category: Option<String>,
    provider_hints: Vec<String>,
    default_source: String,
    external_ref_system: Option<String>,
    events_enabled: bool,
    projection_mode: String,
    compatibility_mode: String,
    #[serde(default)]
    agent_endpoints_enabled: bool,
    #[serde(default)]
    agent_endpoint_ids: Vec<String>,
    #[serde(default = "default_agent_endpoint_risk")]
    agent_endpoint_default_risk: String,
    #[serde(default = "default_agent_endpoint_approval")]
    agent_endpoint_default_approval: String,
    #[serde(default)]
    agent_endpoint_exports: Vec<String>,
    #[serde(default)]
    agent_endpoint_provider_category: Option<String>,
    include_agent_tools: bool,
    artifacts: Vec<String>,
}

pub fn main() -> std::process::ExitCode {
    match run(std::env::args_os()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            std::process::ExitCode::from(2)
        }
    }
}

pub fn run<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Wizard(args) => run_wizard(args),
        Commands::Pack(args) => run_pack(args),
        Commands::InspectProductShape => {
            println!("wizard-first-plus-pack");
            Ok(())
        }
    }
}

fn run_pack(args: PackArgs) -> Result<(), String> {
    match args.command {
        Some(PackCommand::Doctor(path_args)) => {
            let report = doctor_sorla_gtpack(&path_args.path)?;
            let rendered = serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        Some(PackCommand::Inspect(path_args)) => {
            let inspection = inspect_sorla_gtpack(&path_args.path)?;
            let rendered =
                serde_json::to_string_pretty(&inspection).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        None => {
            let input = args
                .input
                .ok_or_else(|| "pack requires a SoRLa input file".to_string())?;
            let name = args
                .name
                .ok_or_else(|| "pack requires `--name <name>`".to_string())?;
            let version = args
                .version
                .ok_or_else(|| "pack requires `--version <semver>`".to_string())?;
            let out = args
                .out
                .ok_or_else(|| "pack requires `--out <file.gtpack>`".to_string())?;
            let summary = build_sorla_gtpack(&SorlaGtpackOptions {
                input_path: input,
                name,
                version,
                out_path: out,
            })?;
            let rendered = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
    }
}

fn run_wizard(args: WizardArgs) -> Result<(), String> {
    let pack_out = args.pack_out;
    match (args.schema, args.answers) {
        (true, None) => {
            if pack_out.is_some() {
                return Err("`--pack-out` can only be used when applying answers or running the interactive wizard".to_string());
            }
            let schema = default_schema();
            let rendered = serde_json::to_string_pretty(&schema).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        (false, Some(path)) => {
            let contents = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read answers file {}: {err}", path.display()))?;
            let answers: AnswersDocument = serde_json::from_str(&contents)
                .map_err(|err| format!("failed to parse answers file {}: {err}", path.display()))?;
            let summary = apply_answers(answers, pack_out)?;
            let rendered = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        (true, Some(_)) => {
            Err("choose one wizard mode: use either `--schema` or `--answers <file>`".to_string())
        }
        (false, None) => run_interactive_wizard(pack_out),
    }
}

fn run_interactive_wizard(pack_out: Option<PathBuf>) -> Result<(), String> {
    let locale = selected_locale(None, None);
    let mut provider = |question_id: &str, question: &serde_json::Value| {
        prompt_interactive_answer(question_id, question)
    };
    let summary = run_interactive_wizard_with_provider(&locale, &mut provider, pack_out)?;
    let rendered = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
    println!("{rendered}");
    Ok(())
}

fn run_interactive_wizard_with_provider(
    locale: &str,
    answer_provider: &mut AnswerProvider,
    pack_out: Option<PathBuf>,
) -> Result<ExecutionSummary, String> {
    let spec_json = serde_json::to_string_pretty(&build_interactive_qa_spec(locale))
        .map_err(|err| err.to_string())?;
    let mut driver = WizardDriver::new(WizardRunConfig {
        spec_json,
        initial_answers_json: None,
        frontend: WizardFrontend::JsonUi,
        i18n: I18nConfig {
            locale: Some(locale.to_string()),
            resolved: load_interactive_i18n(locale),
            debug: false,
        },
        verbose: false,
    })
    .map_err(format_qa_error)?;

    loop {
        let ui_raw = driver.next_payload_json().map_err(format_qa_error)?;
        let ui: serde_json::Value = serde_json::from_str(&ui_raw).map_err(|err| err.to_string())?;
        if driver.is_complete() {
            break;
        }

        let question_id = ui
            .get("next_question_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "wizard QA flow failed: missing next_question_id".to_string())?;
        let question = ui
            .get("questions")
            .and_then(serde_json::Value::as_array)
            .and_then(|questions| {
                questions.iter().find(|question| {
                    question.get("id").and_then(serde_json::Value::as_str) == Some(question_id)
                })
            })
            .ok_or_else(|| format!("wizard QA flow failed: missing question `{question_id}`"))?;

        let answer = answer_provider(question_id, question).map_err(format_qa_error)?;
        let patch = serde_json::json!({ question_id: answer }).to_string();
        let submit = driver.submit_patch_json(&patch).map_err(format_qa_error)?;
        if submit.status == "error" {
            let submit_value: serde_json::Value =
                serde_json::from_str(&submit.response_json).map_err(|err| err.to_string())?;
            if submit_value.get("next_question_id").is_none() {
                return Err(format!("wizard QA flow failed: {}", submit.response_json));
            }
        }
    }

    let result = driver.finish().map_err(format_qa_error)?;

    let answers = answers_document_from_qa_answers(result.answer_set.answers)?;
    apply_answers(answers, pack_out)
}

fn apply_answers(
    answers: AnswersDocument,
    pack_out: Option<PathBuf>,
) -> Result<ExecutionSummary, String> {
    let schema = default_schema();
    validate_answers_document(&answers, &schema)?;

    let output_dir = PathBuf::from(&answers.output_dir);
    let generated_dir = output_dir.join(".greentic-sorla").join("generated");
    let lock_path = generated_dir.join(LOCK_FILENAME);

    let resolved = match answers.flow.as_str() {
        "create" => {
            if lock_path.exists() {
                return Err(
                    "output directory already contains wizard state; use `flow: update` instead of `create`".to_string(),
                );
            }
            resolve_create_answers(&answers)
        }
        "update" => {
            let previous = read_lock_file(&lock_path)?;
            resolve_update_answers(&answers, previous)
        }
        _ => unreachable!("validated earlier"),
    }?;

    fs::create_dir_all(&generated_dir).map_err(|err| {
        format!(
            "failed to create generated directory {}: {err}",
            generated_dir.display()
        )
    })?;

    let package_path = output_dir.join("sorla.yaml");
    let generated_yaml = render_package_yaml(&resolved);
    let preserved_user_content = write_generated_block(&package_path, &generated_yaml)?;

    let mut written_files = vec![relative_to_output(&output_dir, &package_path)];
    write_lock_file(&lock_path, &resolved)?;
    written_files.push(relative_to_output(&output_dir, &lock_path));

    let manifest_path = generated_dir.join("package-manifest.json");
    let manifest_json = serde_json::to_vec_pretty(&build_manifest_payload(&resolved))
        .map_err(|err| err.to_string())?;
    fs::write(&manifest_path, manifest_json).map_err(|err| {
        format!(
            "failed to write generated file {}: {err}",
            manifest_path.display()
        )
    })?;
    written_files.push(relative_to_output(&output_dir, &manifest_path));

    let provider_requirements_path = generated_dir.join("provider-requirements.json");
    let provider_requirements_json =
        serde_json::to_vec_pretty(&build_provider_requirement_manifest(&resolved))
            .map_err(|err| err.to_string())?;
    fs::write(&provider_requirements_path, provider_requirements_json).map_err(|err| {
        format!(
            "failed to write generated file {}: {err}",
            provider_requirements_path.display()
        )
    })?;
    written_files.push(relative_to_output(&output_dir, &provider_requirements_path));

    let locale_manifest_path = generated_dir.join("locale-manifest.json");
    let locale_manifest_json = serde_json::to_vec_pretty(&build_locale_manifest(&resolved))
        .map_err(|err| err.to_string())?;
    fs::write(&locale_manifest_path, locale_manifest_json).map_err(|err| {
        format!(
            "failed to write generated file {}: {err}",
            locale_manifest_path.display()
        )
    })?;
    written_files.push(relative_to_output(&output_dir, &locale_manifest_path));

    let artifact_paths = sync_generated_artifacts(&generated_dir, &resolved)?;
    written_files.extend(
        artifact_paths
            .into_iter()
            .map(|path| relative_to_output(&output_dir, &path)),
    );

    let pack_path = if let Some(pack_out) = pack_out {
        let pack_path = if pack_out.is_relative() {
            output_dir.join(pack_out)
        } else {
            pack_out
        };
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: package_path.clone(),
            name: resolved.package_name.clone(),
            version: resolved.package_version.clone(),
            out_path: pack_path.clone(),
        })?;
        written_files.push(relative_to_output(&output_dir, &pack_path));
        Some(relative_to_output(&output_dir, &pack_path))
    } else {
        None
    };

    written_files.sort();
    written_files.dedup();

    Ok(ExecutionSummary {
        mode: if resolved.flow == "create" {
            "create"
        } else {
            "update"
        },
        output_dir: resolved.output_dir.clone(),
        package_name: resolved.package_name.clone(),
        locale: resolved.locale.clone(),
        written_files,
        pack_path,
        preserved_user_content,
    })
}

fn validate_answers_document(
    answers: &AnswersDocument,
    schema: &WizardSchema,
) -> Result<(), String> {
    if answers.schema_version != schema.schema_version {
        return Err(format!(
            "schema_version mismatch: expected {}, got {}",
            schema.schema_version, answers.schema_version
        ));
    }

    if answers.output_dir.trim().is_empty() {
        return Err("answers field `output_dir` is required".to_string());
    }

    let flow = match answers.flow.as_str() {
        "create" => SchemaFlow::Create,
        "update" => SchemaFlow::Update,
        other => {
            return Err(format!(
                "answers field `flow` must be `create` or `update`, got `{other}`"
            ));
        }
    };

    if !schema.supported_modes.contains(&flow) {
        return Err(format!("schema does not support flow `{}`", answers.flow));
    }

    if answers.flow == "create" {
        let package = answers.package.as_ref().ok_or_else(|| {
            "create flow requires the `package` section with at least `package.name` and `package.version`".to_string()
        })?;
        if package.name.as_deref().unwrap_or("").trim().is_empty() {
            return Err("create flow requires `package.name`".to_string());
        }
        if package.version.as_deref().unwrap_or("").trim().is_empty() {
            return Err("create flow requires `package.version`".to_string());
        }
    }

    validate_choice(
        answers
            .records
            .as_ref()
            .and_then(|records| records.default_source.as_deref()),
        &["native", "external", "hybrid"],
        "records.default_source",
    )?;
    validate_choice(
        answers
            .projections
            .as_ref()
            .and_then(|projections| projections.mode.as_deref()),
        &["current-state", "audit-trail"],
        "projections.mode",
    )?;
    validate_choice(
        answers
            .migrations
            .as_ref()
            .and_then(|migrations| migrations.compatibility.as_deref()),
        &["additive", "backward-compatible", "breaking"],
        "migrations.compatibility",
    )?;
    validate_choice(
        answers
            .providers
            .as_ref()
            .and_then(|providers| providers.storage_category.as_deref()),
        &["storage"],
        "providers.storage_category",
    )?;
    validate_choice(
        answers
            .providers
            .as_ref()
            .and_then(|providers| providers.external_ref_category.as_deref()),
        &["external-ref"],
        "providers.external_ref_category",
    )?;
    validate_choice(
        answers
            .agent_endpoints
            .as_ref()
            .and_then(|agent_endpoints| agent_endpoints.default_risk.as_deref()),
        &["low", "medium", "high"],
        "agent_endpoints.default_risk",
    )?;
    validate_choice(
        answers
            .agent_endpoints
            .as_ref()
            .and_then(|agent_endpoints| agent_endpoints.default_approval.as_deref()),
        &["none", "optional", "required", "policy-driven"],
        "agent_endpoints.default_approval",
    )?;
    if let Some(agent_endpoints) = &answers.agent_endpoints {
        let enabled = agent_endpoints.enabled.unwrap_or(false);
        if enabled {
            let ids = normalize_text_list(agent_endpoints.ids.clone().unwrap_or_default());
            if ids.is_empty() {
                return Err(
                    "field `agent_endpoints.ids` requires at least one endpoint id when agent endpoints are enabled"
                        .to_string(),
                );
            }

            let risk = agent_endpoints.default_risk.as_deref().unwrap_or("medium");
            let approval = agent_endpoints
                .default_approval
                .as_deref()
                .unwrap_or("policy-driven");
            if risk == "high" && !matches!(approval, "required" | "policy-driven") {
                return Err(
                    "field `agent_endpoints.default_approval` must be `required` or `policy-driven` when `agent_endpoints.default_risk` is `high`"
                        .to_string(),
                );
            }

            if let Some(exports) = &agent_endpoints.exports {
                for export in exports {
                    if !["openapi", "arazzo", "mcp", "llms_txt"].contains(&export.as_str()) {
                        return Err(format!(
                            "agent_endpoints.exports contains unsupported export `{export}`"
                        ));
                    }
                }
            }
        }
    }

    if let Some(output) = &answers.output
        && let Some(artifacts) = &output.artifacts
    {
        let allowed: BTreeSet<&str> = schema.artifact_references.iter().copied().collect();
        for artifact in artifacts {
            if !allowed.contains(artifact.as_str()) {
                return Err(format!(
                    "output.artifacts contains unsupported artifact `{artifact}`"
                ));
            }
        }
    }

    Ok(())
}

fn validate_choice(value: Option<&str>, allowed: &[&str], field: &str) -> Result<(), String> {
    if let Some(value) = value
        && !allowed.contains(&value)
    {
        return Err(format!(
            "field `{field}` must be one of {}, got `{value}`",
            allowed.join(", ")
        ));
    }
    Ok(())
}

fn normalize_text_list(values: Vec<String>) -> Vec<String> {
    let mut normalized = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn resolve_create_answers(answers: &AnswersDocument) -> Result<ResolvedAnswers, String> {
    let package = answers.package.as_ref().ok_or_else(|| {
        "create flow requires the `package` section with at least `package.name` and `package.version`".to_string()
    })?;
    let default_source = answers
        .records
        .as_ref()
        .and_then(|records| records.default_source.clone())
        .unwrap_or_else(|| "native".to_string());
    let external_ref_system = answers
        .records
        .as_ref()
        .and_then(|records| records.external_ref_system.clone());

    if matches!(default_source.as_str(), "external" | "hybrid")
        && external_ref_system
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(format!(
            "field `records.external_ref_system` is required when `records.default_source` is `{default_source}`"
        ));
    }

    let mut artifacts = normalize_artifacts(
        answers
            .output
            .as_ref()
            .and_then(|output| output.artifacts.clone())
            .unwrap_or_else(default_artifacts),
    )?;

    let include_agent_tools = answers
        .output
        .as_ref()
        .and_then(|output| output.include_agent_tools)
        .unwrap_or(true);
    if include_agent_tools {
        if !artifacts
            .iter()
            .any(|artifact| artifact == "agent-tools.json")
        {
            artifacts.push("agent-tools.json".to_string());
        }
    } else {
        artifacts.retain(|artifact| artifact != "agent-tools.json");
    }
    artifacts.sort();
    artifacts.dedup();
    let agent_endpoint_values = resolve_agent_endpoint_answers(answers.agent_endpoints.as_ref());

    let resolved = ResolvedAnswers {
        schema_version: answers.schema_version.clone(),
        flow: "create".to_string(),
        output_dir: answers.output_dir.clone(),
        locale: selected_locale(answers.locale.as_deref(), None),
        package_name: package.name.clone().unwrap(),
        package_version: package.version.clone().unwrap(),
        storage_category: answers
            .providers
            .as_ref()
            .and_then(|providers| providers.storage_category.clone())
            .unwrap_or_else(|| "storage".to_string()),
        external_ref_category: if matches!(default_source.as_str(), "external" | "hybrid") {
            Some(
                answers
                    .providers
                    .as_ref()
                    .and_then(|providers| providers.external_ref_category.clone())
                    .unwrap_or_else(|| "external-ref".to_string()),
            )
        } else {
            None
        },
        provider_hints: answers
            .providers
            .as_ref()
            .and_then(|providers| providers.hints.clone())
            .unwrap_or_default(),
        default_source,
        external_ref_system,
        events_enabled: answers
            .events
            .as_ref()
            .and_then(|events| events.enabled)
            .unwrap_or(true),
        projection_mode: answers
            .projections
            .as_ref()
            .and_then(|projections| projections.mode.clone())
            .unwrap_or_else(|| "current-state".to_string()),
        compatibility_mode: answers
            .migrations
            .as_ref()
            .and_then(|migrations| migrations.compatibility.clone())
            .unwrap_or_else(|| "additive".to_string()),
        agent_endpoints_enabled: agent_endpoint_values.enabled,
        agent_endpoint_ids: agent_endpoint_values.ids,
        agent_endpoint_default_risk: agent_endpoint_values.default_risk,
        agent_endpoint_default_approval: agent_endpoint_values.default_approval,
        agent_endpoint_exports: agent_endpoint_values.exports,
        agent_endpoint_provider_category: agent_endpoint_values.provider_category,
        include_agent_tools,
        artifacts,
    };

    Ok(resolved)
}

fn resolve_update_answers(
    answers: &AnswersDocument,
    previous: ResolvedAnswers,
) -> Result<ResolvedAnswers, String> {
    if let Some(package) = &answers.package
        && let Some(name) = &package.name
        && name != &previous.package_name
    {
        return Err(format!(
            "update flow cannot change `package.name` from `{}` to `{}`",
            previous.package_name, name
        ));
    }

    let default_source = answers
        .records
        .as_ref()
        .and_then(|records| records.default_source.clone())
        .unwrap_or_else(|| previous.default_source.clone());
    let external_ref_system = answers
        .records
        .as_ref()
        .and_then(|records| records.external_ref_system.clone())
        .or_else(|| previous.external_ref_system.clone());

    if matches!(default_source.as_str(), "external" | "hybrid")
        && external_ref_system
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(format!(
            "field `records.external_ref_system` is required when `records.default_source` is `{default_source}`"
        ));
    }

    let include_agent_tools = answers
        .output
        .as_ref()
        .and_then(|output| output.include_agent_tools)
        .unwrap_or(previous.include_agent_tools);

    let mut artifacts = normalize_artifacts(
        answers
            .output
            .as_ref()
            .and_then(|output| output.artifacts.clone())
            .unwrap_or_else(|| previous.artifacts.clone()),
    )?;
    if include_agent_tools {
        if !artifacts
            .iter()
            .any(|artifact| artifact == "agent-tools.json")
        {
            artifacts.push("agent-tools.json".to_string());
        }
    } else {
        artifacts.retain(|artifact| artifact != "agent-tools.json");
    }
    artifacts.sort();
    artifacts.dedup();
    let agent_endpoint_values =
        resolve_agent_endpoint_update_answers(answers.agent_endpoints.as_ref(), &previous);

    Ok(ResolvedAnswers {
        schema_version: previous.schema_version,
        flow: "update".to_string(),
        output_dir: previous.output_dir,
        locale: selected_locale(answers.locale.as_deref(), Some(previous.locale.as_str())),
        package_name: previous.package_name,
        package_version: answers
            .package
            .as_ref()
            .and_then(|package| package.version.clone())
            .unwrap_or(previous.package_version),
        storage_category: answers
            .providers
            .as_ref()
            .and_then(|providers| providers.storage_category.clone())
            .unwrap_or(previous.storage_category),
        external_ref_category: if matches!(default_source.as_str(), "external" | "hybrid") {
            Some(
                answers
                    .providers
                    .as_ref()
                    .and_then(|providers| providers.external_ref_category.clone())
                    .or(previous.external_ref_category)
                    .unwrap_or_else(|| "external-ref".to_string()),
            )
        } else {
            None
        },
        provider_hints: answers
            .providers
            .as_ref()
            .and_then(|providers| providers.hints.clone())
            .unwrap_or(previous.provider_hints),
        default_source,
        external_ref_system,
        events_enabled: answers
            .events
            .as_ref()
            .and_then(|events| events.enabled)
            .unwrap_or(previous.events_enabled),
        projection_mode: answers
            .projections
            .as_ref()
            .and_then(|projections| projections.mode.clone())
            .unwrap_or(previous.projection_mode),
        compatibility_mode: answers
            .migrations
            .as_ref()
            .and_then(|migrations| migrations.compatibility.clone())
            .unwrap_or(previous.compatibility_mode),
        agent_endpoints_enabled: agent_endpoint_values.enabled,
        agent_endpoint_ids: agent_endpoint_values.ids,
        agent_endpoint_default_risk: agent_endpoint_values.default_risk,
        agent_endpoint_default_approval: agent_endpoint_values.default_approval,
        agent_endpoint_exports: agent_endpoint_values.exports,
        agent_endpoint_provider_category: agent_endpoint_values.provider_category,
        include_agent_tools,
        artifacts,
    })
}

struct ResolvedAgentEndpointAnswers {
    enabled: bool,
    ids: Vec<String>,
    default_risk: String,
    default_approval: String,
    exports: Vec<String>,
    provider_category: Option<String>,
}

fn resolve_agent_endpoint_answers(
    answers: Option<&AgentEndpointAnswers>,
) -> ResolvedAgentEndpointAnswers {
    let enabled = answers.and_then(|answers| answers.enabled).unwrap_or(false);
    let ids = if enabled {
        normalize_text_list(
            answers
                .and_then(|answers| answers.ids.clone())
                .unwrap_or_default(),
        )
    } else {
        Vec::new()
    };
    let exports = if enabled {
        normalize_text_list(
            answers
                .and_then(|answers| answers.exports.clone())
                .unwrap_or_else(default_agent_endpoint_exports),
        )
    } else {
        Vec::new()
    };

    ResolvedAgentEndpointAnswers {
        enabled,
        ids,
        default_risk: answers
            .and_then(|answers| answers.default_risk.clone())
            .unwrap_or_else(|| "medium".to_string()),
        default_approval: answers
            .and_then(|answers| answers.default_approval.clone())
            .unwrap_or_else(|| "policy-driven".to_string()),
        exports,
        provider_category: answers
            .and_then(|answers| answers.provider_category.clone())
            .map(|value| value.trim().to_string())
            .filter(|value| enabled && !value.is_empty()),
    }
}

fn resolve_agent_endpoint_update_answers(
    answers: Option<&AgentEndpointAnswers>,
    previous: &ResolvedAnswers,
) -> ResolvedAgentEndpointAnswers {
    let Some(answers) = answers else {
        return ResolvedAgentEndpointAnswers {
            enabled: previous.agent_endpoints_enabled,
            ids: previous.agent_endpoint_ids.clone(),
            default_risk: previous.agent_endpoint_default_risk.clone(),
            default_approval: previous.agent_endpoint_default_approval.clone(),
            exports: previous.agent_endpoint_exports.clone(),
            provider_category: previous.agent_endpoint_provider_category.clone(),
        };
    };

    let enabled = answers.enabled.unwrap_or(previous.agent_endpoints_enabled);
    ResolvedAgentEndpointAnswers {
        enabled,
        ids: if enabled {
            answers
                .ids
                .clone()
                .map(normalize_text_list)
                .unwrap_or_else(|| previous.agent_endpoint_ids.clone())
        } else {
            Vec::new()
        },
        default_risk: answers
            .default_risk
            .clone()
            .unwrap_or_else(|| previous.agent_endpoint_default_risk.clone()),
        default_approval: answers
            .default_approval
            .clone()
            .unwrap_or_else(|| previous.agent_endpoint_default_approval.clone()),
        exports: if enabled {
            answers
                .exports
                .clone()
                .map(normalize_text_list)
                .unwrap_or_else(|| previous.agent_endpoint_exports.clone())
        } else {
            Vec::new()
        },
        provider_category: answers
            .provider_category
            .clone()
            .map(|value| value.trim().to_string())
            .filter(|value| enabled && !value.is_empty())
            .or_else(|| {
                if enabled {
                    previous.agent_endpoint_provider_category.clone()
                } else {
                    None
                }
            }),
    }
}

fn default_agent_endpoint_exports() -> Vec<String> {
    ["openapi", "arazzo", "mcp", "llms_txt"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn default_agent_endpoint_risk() -> String {
    "medium".to_string()
}

fn default_agent_endpoint_approval() -> String {
    "policy-driven".to_string()
}

fn normalize_artifacts(artifacts: Vec<String>) -> Result<Vec<String>, String> {
    let allowed: BTreeSet<&str> = default_schema().artifact_references.into_iter().collect();
    let mut normalized = Vec::new();
    for artifact in artifacts {
        if !allowed.contains(artifact.as_str()) {
            return Err(format!(
                "output.artifacts contains unsupported artifact `{artifact}`"
            ));
        }
        normalized.push(artifact);
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn default_artifacts() -> Vec<String> {
    default_schema()
        .artifact_references
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn read_lock_file(path: &Path) -> Result<ResolvedAnswers, String> {
    let contents = fs::read_to_string(path).map_err(|err| {
        format!(
            "update flow requires existing wizard state at {}: {err}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|err| {
        format!(
            "failed to parse existing wizard state {}: {err}",
            path.display()
        )
    })
}

fn write_lock_file(path: &Path, resolved: &ResolvedAnswers) -> Result<(), String> {
    let contents = serde_json::to_vec_pretty(resolved).map_err(|err| err.to_string())?;
    fs::write(path, contents)
        .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))
}

fn write_generated_block(path: &Path, generated_yaml: &str) -> Result<bool, String> {
    let block = format!("{GENERATED_BEGIN}\n{generated_yaml}{GENERATED_END}\n");
    let existing = if path.exists() {
        Some(
            fs::read_to_string(path)
                .map_err(|err| format!("failed to read package file {}: {err}", path.display()))?,
        )
    } else {
        None
    };

    let (contents, preserved_user_content) = if let Some(existing) = existing {
        if let (Some(start), Some(end)) =
            (existing.find(GENERATED_BEGIN), existing.find(GENERATED_END))
        {
            let end_index = end + GENERATED_END.len();
            let suffix = existing[end_index..]
                .strip_prefix('\n')
                .unwrap_or(&existing[end_index..]);
            let mut updated = String::new();
            updated.push_str(&existing[..start]);
            updated.push_str(&block);
            updated.push_str(suffix);
            (updated, true)
        } else {
            let mut updated = existing;
            if !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(&block);
            (updated, true)
        }
    } else {
        (block, false)
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create directory {}: {err}", parent.display()))?;
    }

    fs::write(path, contents)
        .map_err(|err| format!("failed to write package file {}: {err}", path.display()))?;
    Ok(preserved_user_content)
}

fn render_package_yaml(resolved: &ResolvedAnswers) -> String {
    let record_name = format!("{}Record", to_pascal_case(&resolved.package_name));
    let mut lines = vec![
        format!("package:"),
        format!("  name: {}", resolved.package_name),
        format!("  version: {}", resolved.package_version),
        "records:".to_string(),
        format!("  - name: {record_name}"),
        format!("    source: {}", resolved.default_source),
    ];

    if matches!(resolved.default_source.as_str(), "external" | "hybrid") {
        lines.push("    external_ref:".to_string());
        lines.push(format!(
            "      system: {}",
            resolved
                .external_ref_system
                .as_deref()
                .unwrap_or("external-system")
        ));
        lines.push("      key: record_id".to_string());
        lines.push("      authoritative: true".to_string());
    }

    lines.push("    fields:".to_string());
    match resolved.default_source.as_str() {
        "native" => {
            lines.push("      - name: record_id".to_string());
            lines.push("        type: string".to_string());
            lines.push("      - name: workflow_state".to_string());
            lines.push("        type: string".to_string());
        }
        "external" => {
            lines.push("      - name: record_id".to_string());
            lines.push("        type: string".to_string());
            lines.push("      - name: external_snapshot".to_string());
            lines.push("        type: string".to_string());
        }
        "hybrid" => {
            lines.push("      - name: record_id".to_string());
            lines.push("        type: string".to_string());
            lines.push("        authority: external".to_string());
            lines.push("      - name: workflow_state".to_string());
            lines.push("        type: string".to_string());
            lines.push("        authority: local".to_string());
        }
        _ => {}
    }

    if resolved.events_enabled {
        let event_name = format!("{}Changed", record_name);
        lines.push("events:".to_string());
        lines.push(format!("  - name: {event_name}"));
        lines.push(format!("    record: {record_name}"));
        lines.push("    kind: domain".to_string());
        lines.push("    emits:".to_string());
        lines.push("      - name: record_id".to_string());
        lines.push("        type: string".to_string());

        lines.push("projections:".to_string());
        lines.push(format!("  - name: {}Projection", record_name));
        lines.push(format!("    record: {record_name}"));
        lines.push(format!("    source_event: {event_name}"));
        lines.push(format!("    mode: {}", resolved.projection_mode));
    } else {
        lines.push("events: []".to_string());
        lines.push("projections: []".to_string());
    }

    lines.push("provider_requirements:".to_string());
    lines.push(format!("  - category: {}", resolved.storage_category));
    lines.push("    capabilities:".to_string());
    lines.push("      - event-log".to_string());
    lines.push("      - projections".to_string());
    if let Some(category) = &resolved.external_ref_category {
        lines.push(format!("  - category: {category}"));
        lines.push("    capabilities:".to_string());
        lines.push("      - lookup".to_string());
    }
    if resolved.agent_endpoints_enabled
        && let Some(category) = &resolved.agent_endpoint_provider_category
    {
        lines.push(format!("  - category: {category}"));
        lines.push("    capabilities:".to_string());
        lines.push("      - agent-endpoint-handoff".to_string());
    }

    lines.push("migrations:".to_string());
    lines.push(format!("  - name: {}-compatibility", resolved.package_name));
    lines.push(format!(
        "    compatibility: {}",
        resolved.compatibility_mode
    ));
    if resolved.events_enabled {
        lines.push("    projection_updates:".to_string());
        lines.push(format!(
            "      - {}Projection",
            to_pascal_case(&resolved.package_name) + "Record"
        ));
    } else {
        lines.push("    projection_updates: []".to_string());
    }

    if resolved.agent_endpoints_enabled {
        lines.push("agent_endpoints:".to_string());
        for id in &resolved.agent_endpoint_ids {
            let title = title_from_identifier(id);
            lines.push(format!("  - id: {id}"));
            lines.push(format!("    title: {title}"));
            lines.push(format!(
                "    intent: Request the generated `{id}` business action through downstream agent handoff metadata."
            ));
            lines.push("    inputs:".to_string());
            lines.push("      - name: record_id".to_string());
            lines.push("        type: string".to_string());
            lines.push("        required: true".to_string());
            lines.push("    outputs:".to_string());
            lines.push("      - name: status".to_string());
            lines.push("        type: string".to_string());
            lines.push("    side_effects:".to_string());
            lines.push(format!("      - agent.{id}.request"));
            lines.push(format!(
                "    risk: {}",
                resolved.agent_endpoint_default_risk
            ));
            lines.push(format!(
                "    approval: {}",
                resolved.agent_endpoint_default_approval
            ));
            if let Some(category) = &resolved.agent_endpoint_provider_category {
                lines.push("    provider_requirements:".to_string());
                lines.push(format!("      - category: {category}"));
                lines.push("        capabilities:".to_string());
                lines.push("          - agent-endpoint-handoff".to_string());
            }
            lines.push("    agent_visibility:".to_string());
            lines.push(format!(
                "      openapi: {}",
                resolved
                    .agent_endpoint_exports
                    .iter()
                    .any(|item| item == "openapi")
            ));
            lines.push(format!(
                "      arazzo: {}",
                resolved
                    .agent_endpoint_exports
                    .iter()
                    .any(|item| item == "arazzo")
            ));
            lines.push(format!(
                "      mcp: {}",
                resolved
                    .agent_endpoint_exports
                    .iter()
                    .any(|item| item == "mcp")
            ));
            lines.push(format!(
                "      llms_txt: {}",
                resolved
                    .agent_endpoint_exports
                    .iter()
                    .any(|item| item == "llms_txt")
            ));
            lines.push("    examples:".to_string());
            lines.push(format!("      - name: {id}-example"));
            lines.push(format!("        summary: Example request for {title}."));
            lines.push("        input:".to_string());
            lines.push("          record_id: record-123".to_string());
            lines.push("        expected_output:".to_string());
            lines.push("          status: accepted".to_string());
        }
    } else {
        lines.push("agent_endpoints: []".to_string());
    }

    lines.join("\n") + "\n"
}

fn build_manifest_payload(resolved: &ResolvedAnswers) -> BTreeMap<&'static str, serde_json::Value> {
    let mut map = BTreeMap::new();
    map.insert(
        "package_name",
        serde_json::Value::String(resolved.package_name.clone()),
    );
    map.insert(
        "package_version",
        serde_json::Value::String(resolved.package_version.clone()),
    );
    map.insert(
        "package_kind",
        serde_json::Value::String("greentic-sorla-package".to_string()),
    );
    map.insert(
        "ir_version",
        serde_json::Value::String("sorla-ir/v1".to_string()),
    );
    map.insert("locale", serde_json::Value::String(resolved.locale.clone()));
    map.insert("flow", serde_json::Value::String(resolved.flow.clone()));
    map.insert(
        "provider_repo",
        serde_json::Value::String("greentic-sorla-providers".to_string()),
    );
    map.insert(
        "binding_mode",
        serde_json::Value::String("abstract-category-resolution".to_string()),
    );
    map.insert(
        "locale_metadata",
        serde_json::json!({
            "default_locale": resolved.locale,
            "fallback_locale": "en",
            "schema_localized": true
        }),
    );
    map.insert(
        "compatibility_metadata",
        serde_json::json!({
            "schema_version": resolved.schema_version,
            "wizard_version": env!("CARGO_PKG_VERSION"),
            "supports_partial_answers": true,
            "generated_content_strategy": "rewrite-generated-blocks-only",
            "user_content_strategy": "preserve-outside-generated-blocks",
        }),
    );
    map.insert(
        "provider_requirement_declarations",
        serde_json::to_value(build_provider_requirement_manifest(resolved))
            .expect("provider requirement manifest is serializable"),
    );
    map.insert(
        "artifacts",
        serde_json::Value::Array(
            resolved
                .artifacts
                .iter()
                .cloned()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );
    map
}

fn build_provider_requirement_manifest(
    resolved: &ResolvedAnswers,
) -> BTreeMap<&'static str, serde_json::Value> {
    let mut map = BTreeMap::new();
    map.insert(
        "provider_repo",
        serde_json::Value::String("greentic-sorla-providers".to_string()),
    );
    map.insert(
        "binding_mode",
        serde_json::Value::String("abstract-category-resolution".to_string()),
    );
    map.insert(
        "required_capability_categories",
        serde_json::json!([resolved.storage_category]),
    );

    let mut optional_capabilities = Vec::new();
    if let Some(category) = &resolved.external_ref_category {
        optional_capabilities.push(category.clone());
    }
    if let Some(category) = &resolved.agent_endpoint_provider_category {
        optional_capabilities.push(category.clone());
    }
    optional_capabilities.push("evidence-store".to_string());
    map.insert(
        "optional_capability_categories",
        serde_json::json!(optional_capabilities),
    );
    map.insert(
        "provider_requirement_declarations",
        serde_json::json!([
            {
                "name": "storage",
                "category": resolved.storage_category,
                "required": true,
            },
            {
                "name": "external_ref",
                "category": resolved.external_ref_category,
                "required": matches!(resolved.default_source.as_str(), "external" | "hybrid"),
            },
            {
                "name": "evidence",
                "category": "evidence-store",
                "required": false,
            },
            {
                "name": "agent_endpoint_handoff",
                "category": resolved.agent_endpoint_provider_category,
                "required": resolved.agent_endpoints_enabled,
            }
        ]),
    );
    map
}

fn build_locale_manifest(resolved: &ResolvedAnswers) -> BTreeMap<&'static str, serde_json::Value> {
    let mut map = BTreeMap::new();
    map.insert(
        "default_locale",
        serde_json::Value::String(resolved.locale.clone()),
    );
    map.insert(
        "fallback_locale",
        serde_json::Value::String("en".to_string()),
    );
    map.insert(
        "schema_version",
        serde_json::Value::String(resolved.schema_version.clone()),
    );
    map.insert(
        "reserved_core_keys",
        serde_json::json!([
            "wizard.title",
            "wizard.description",
            "wizard.section.title",
            "wizard.question.label",
            "wizard.question.help",
            "wizard.validation.message",
            "wizard.action.create.label",
            "wizard.action.update.label",
        ]),
    );
    map
}

fn build_interactive_qa_spec(locale: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "greentic-sorla-wizard",
        "title": "Greentic SoRLa Wizard",
        "version": default_schema().schema_version,
        "description": "Interactive wizard for creating or updating a SoRLa package.",
        "presentation": {
            "default_locale": locale,
            "intro": "Answer the questions below to create or update a SoRLa package. Press Enter to accept defaults."
        },
        "progress_policy": {
            "skip_answered": true,
            "autofill_defaults": false,
            "treat_default_as_answered": false
        },
        "questions": [
            {
                "id": "flow",
                "type": "enum",
                "title": "Wizard flow",
                "title_i18n": { "key": "wizard.flow.label" },
                "description": "Choose whether to create a new package or update an existing generated package.",
                "required": true,
                "default_value": "create",
                "choices": ["create", "update"]
            },
            {
                "id": "output_dir",
                "type": "string",
                "title": "Output directory",
                "title_i18n": { "key": "wizard.output_dir.label" },
                "description": "Directory where sorla.yaml and generated metadata will be written.",
                "required": true,
                "default_value": "."
            },
            {
                "id": "locale",
                "type": "string",
                "title": "Locale",
                "title_i18n": { "key": "wizard.locale.label" },
                "description": "Locale used for generated metadata and interactive prompts.",
                "required": false,
                "default_value": locale
            },
            {
                "id": "package_name",
                "type": "string",
                "title": "Package name",
                "title_i18n": { "key": "wizard.questions.package_name.label" },
                "description": "Stable package identifier written into sorla.yaml.",
                "required": true,
                "visible_if": {
                    "op": "eq",
                    "left": { "op": "answer", "path": "flow" },
                    "right": { "op": "literal", "value": "create" }
                }
            },
            {
                "id": "package_version",
                "type": "string",
                "title": "Package version",
                "title_i18n": { "key": "wizard.questions.package_version.label" },
                "description": "Version for the new package.",
                "required": true,
                "default_value": "0.1.0",
                "visible_if": {
                    "op": "eq",
                    "left": { "op": "answer", "path": "flow" },
                    "right": { "op": "literal", "value": "create" }
                }
            },
            {
                "id": "storage_category",
                "type": "enum",
                "title": "Storage provider category",
                "title_i18n": { "key": "wizard.questions.storage_provider.label" },
                "description": "Provider category required for package storage and generated metadata.",
                "required": true,
                "default_value": "storage",
                "choices": ["storage"]
            },
            {
                "id": "default_source",
                "type": "enum",
                "title": "Default record source",
                "title_i18n": { "key": "wizard.questions.default_source.label" },
                "description": "Choose whether records are native, external, or hybrid.",
                "required": true,
                "default_value": "native",
                "choices": ["native", "external", "hybrid"]
            },
            {
                "id": "external_ref_system",
                "type": "string",
                "title": "External reference system",
                "title_i18n": { "key": "wizard.questions.external_system.label" },
                "description": "External system identifier used when the package references authoritative external records.",
                "required": true,
                "visible_if": {
                    "op": "or",
                    "expressions": [
                        {
                            "op": "eq",
                            "left": { "op": "answer", "path": "default_source" },
                            "right": { "op": "literal", "value": "external" }
                        },
                        {
                            "op": "eq",
                            "left": { "op": "answer", "path": "default_source" },
                            "right": { "op": "literal", "value": "hybrid" }
                        }
                    ]
                }
            },
            {
                "id": "external_ref_category",
                "type": "enum",
                "title": "External reference provider category",
                "title_i18n": { "key": "wizard.questions.external_ref_provider.label" },
                "description": "Provider category used to resolve external references.",
                "required": false,
                "default_value": "external-ref",
                "choices": ["external-ref"],
                "visible_if": {
                    "op": "or",
                    "expressions": [
                        {
                            "op": "eq",
                            "left": { "op": "answer", "path": "default_source" },
                            "right": { "op": "literal", "value": "external" }
                        },
                        {
                            "op": "eq",
                            "left": { "op": "answer", "path": "default_source" },
                            "right": { "op": "literal", "value": "hybrid" }
                        }
                    ]
                }
            },
            {
                "id": "events_enabled",
                "type": "boolean",
                "title": "Enable events",
                "title_i18n": { "key": "wizard.questions.events_enabled.label" },
                "description": "Generate event and projection placeholders for this package.",
                "required": true,
                "default_value": "true"
            },
            {
                "id": "projection_mode",
                "type": "enum",
                "title": "Projection mode",
                "title_i18n": { "key": "wizard.questions.projection_mode.label" },
                "description": "Projection strategy for generated package output.",
                "required": true,
                "default_value": "current-state",
                "choices": ["current-state", "audit-trail"]
            },
            {
                "id": "compatibility_mode",
                "type": "enum",
                "title": "Compatibility mode",
                "title_i18n": { "key": "wizard.questions.compatibility_mode.label" },
                "description": "Compatibility mode used for migration metadata.",
                "required": true,
                "default_value": "additive",
                "choices": ["additive", "backward-compatible", "breaking"]
            },
            {
                "id": "agent_endpoints_enabled",
                "type": "boolean",
                "title": "Expose agentic endpoints",
                "title_i18n": { "key": "wizard.questions.agent_endpoints_enabled.label" },
                "description": "Generate agent endpoint declarations in sorla.yaml.",
                "required": true,
                "default_value": "false"
            },
            {
                "id": "agent_endpoint_ids",
                "type": "string",
                "title": "Endpoint identifiers",
                "title_i18n": { "key": "wizard.questions.agent_endpoint_ids.label" },
                "description": "Comma-separated endpoint IDs, such as create_customer_contact.",
                "required": true,
                "visible_if": {
                    "op": "eq",
                    "left": { "op": "answer", "path": "agent_endpoints_enabled" },
                    "right": { "op": "literal", "value": true }
                }
            },
            {
                "id": "agent_endpoint_default_risk",
                "type": "enum",
                "title": "Default endpoint risk",
                "title_i18n": { "key": "wizard.questions.agent_endpoint_default_risk.label" },
                "description": "Risk level for generated agent endpoints.",
                "required": true,
                "default_value": "medium",
                "choices": ["low", "medium", "high"],
                "visible_if": {
                    "op": "eq",
                    "left": { "op": "answer", "path": "agent_endpoints_enabled" },
                    "right": { "op": "literal", "value": true }
                }
            },
            {
                "id": "agent_endpoint_default_approval",
                "type": "enum",
                "title": "Default approval behavior",
                "title_i18n": { "key": "wizard.questions.agent_endpoint_default_approval.label" },
                "description": "Approval behavior for generated agent endpoints.",
                "required": true,
                "default_value": "policy-driven",
                "choices": ["none", "optional", "required", "policy-driven"],
                "visible_if": {
                    "op": "eq",
                    "left": { "op": "answer", "path": "agent_endpoints_enabled" },
                    "right": { "op": "literal", "value": true }
                }
            },
            {
                "id": "agent_endpoint_exports",
                "type": "string",
                "title": "Agent-facing export targets",
                "title_i18n": { "key": "wizard.questions.agent_endpoint_exports.label" },
                "description": "Comma-separated export targets: openapi, arazzo, mcp, llms_txt.",
                "required": true,
                "default_value": "openapi,arazzo,mcp,llms_txt",
                "visible_if": {
                    "op": "eq",
                    "left": { "op": "answer", "path": "agent_endpoints_enabled" },
                    "right": { "op": "literal", "value": true }
                }
            },
            {
                "id": "agent_endpoint_provider_category",
                "type": "string",
                "title": "Default provider category",
                "title_i18n": { "key": "wizard.questions.agent_endpoint_provider_category.label" },
                "description": "Abstract provider category for downstream agent endpoint handoff.",
                "required": false,
                "default_value": "api-gateway",
                "visible_if": {
                    "op": "eq",
                    "left": { "op": "answer", "path": "agent_endpoints_enabled" },
                    "right": { "op": "literal", "value": true }
                }
            },
            {
                "id": "include_agent_tools",
                "type": "boolean",
                "title": "Include agent tools",
                "title_i18n": { "key": "wizard.questions.include_agent_tools.label" },
                "description": "Generate agent-tools.json as part of the artifact set.",
                "required": true,
                "default_value": "true"
            }
        ]
    })
}

fn load_interactive_i18n(locale: &str) -> Option<ResolvedI18nMap> {
    let raw = embedded_i18n::locale_json(locale).or_else(|| embedded_i18n::locale_json("en"))?;
    let map = serde_json::from_str::<BTreeMap<String, String>>(raw).ok()?;
    let mut resolved = ResolvedI18nMap::new();
    for (key, value) in map {
        resolved.insert(key, value);
    }
    Some(resolved)
}

fn prompt_interactive_answer(
    question_id: &str,
    question: &serde_json::Value,
) -> Result<serde_json::Value, QaLibError> {
    let title = question
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(question_id);
    let description = question
        .get("description")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let kind = question
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("string");
    let default = question.get("default");

    println!();
    println!("{title}");
    if !description.trim().is_empty() {
        println!("{description}");
    }
    if let Some(choices) = question
        .get("choices")
        .and_then(serde_json::Value::as_array)
    {
        let rendered = choices
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        if !rendered.is_empty() {
            println!("Choices: {rendered}");
        }
    }

    loop {
        print!("> ");
        io::stdout()
            .flush()
            .map_err(|err| QaLibError::Component(err.to_string()))?;
        let mut line = String::new();
        let read = io::stdin()
            .read_line(&mut line)
            .map_err(|err| QaLibError::Component(err.to_string()))?;
        if read == 0 {
            return Err(QaLibError::Component("stdin closed".to_string()));
        }

        let input = line.trim();
        if input.is_empty() {
            if let Some(default) = default {
                return default_value_for_kind(kind, default).ok_or_else(|| {
                    QaLibError::Component(format!(
                        "invalid default value for interactive question `{question_id}`"
                    ))
                });
            }
            if question
                .get("required")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                println!("A value is required.");
                continue;
            }
            return Ok(serde_json::Value::Null);
        }

        match kind {
            "string" => return Ok(serde_json::Value::String(input.to_string())),
            "boolean" => match input.to_ascii_lowercase().as_str() {
                "y" | "yes" | "true" | "1" => return Ok(serde_json::Value::Bool(true)),
                "n" | "no" | "false" | "0" => return Ok(serde_json::Value::Bool(false)),
                _ => {
                    println!("Enter yes or no.");
                    continue;
                }
            },
            "enum" => {
                let choices = question
                    .get("choices")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if choices
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .any(|choice| choice == input)
                {
                    return Ok(serde_json::Value::String(input.to_string()));
                }
                println!("Enter one of the listed choices.");
            }
            other => {
                return Err(QaLibError::Component(format!(
                    "unsupported interactive question type `{other}` for `{question_id}`"
                )));
            }
        }
    }
}

fn default_value_for_kind(kind: &str, value: &serde_json::Value) -> Option<serde_json::Value> {
    match kind {
        "string" | "enum" => value
            .as_str()
            .map(|text| serde_json::Value::String(text.to_string())),
        "boolean" => value
            .as_str()
            .and_then(|text| match text.to_ascii_lowercase().as_str() {
                "true" | "yes" | "y" | "1" => Some(serde_json::Value::Bool(true)),
                "false" | "no" | "n" | "0" => Some(serde_json::Value::Bool(false)),
                _ => None,
            }),
        _ => None,
    }
}

fn answers_document_from_qa_answers(answers: serde_json::Value) -> Result<AnswersDocument, String> {
    let object = answers
        .as_object()
        .ok_or_else(|| "interactive wizard did not produce an answers object".to_string())?;
    let flow = get_required_string(object, "flow")?;
    let output_dir = get_required_string(object, "output_dir")?;
    let locale = object
        .get("locale")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty());

    let package = if flow == "create" {
        Some(PackageAnswers {
            name: Some(get_required_string(object, "package_name")?),
            version: Some(get_required_string(object, "package_version")?),
        })
    } else {
        None
    };

    let default_source = get_required_string(object, "default_source")?;
    let external_ref_system = object
        .get("external_ref_system")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty());
    let external_ref_category = object
        .get("external_ref_category")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty());

    Ok(AnswersDocument {
        schema_version: default_schema().schema_version.to_string(),
        flow,
        output_dir,
        locale,
        package,
        providers: Some(ProviderAnswers {
            storage_category: Some(get_required_string(object, "storage_category")?),
            external_ref_category,
            hints: None,
        }),
        records: Some(RecordAnswers {
            default_source: Some(default_source),
            external_ref_system,
        }),
        events: Some(EventAnswers {
            enabled: Some(get_required_bool(object, "events_enabled")?),
        }),
        projections: Some(ProjectionAnswers {
            mode: Some(get_required_string(object, "projection_mode")?),
        }),
        migrations: Some(MigrationAnswers {
            compatibility: Some(get_required_string(object, "compatibility_mode")?),
        }),
        agent_endpoints: Some(AgentEndpointAnswers {
            enabled: Some(get_required_bool(object, "agent_endpoints_enabled")?),
            ids: object
                .get("agent_endpoint_ids")
                .and_then(serde_json::Value::as_str)
                .map(split_csv_answer),
            default_risk: object
                .get("agent_endpoint_default_risk")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            default_approval: object
                .get("agent_endpoint_default_approval")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            exports: object
                .get("agent_endpoint_exports")
                .and_then(serde_json::Value::as_str)
                .map(split_csv_answer),
            provider_category: object
                .get("agent_endpoint_provider_category")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .filter(|value| !value.trim().is_empty()),
        }),
        output: Some(OutputAnswers {
            include_agent_tools: Some(get_required_bool(object, "include_agent_tools")?),
            artifacts: None,
        }),
    })
}

fn split_csv_answer(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn get_required_string(
    answers: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, String> {
    answers
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("interactive wizard did not produce required answer `{key}`"))
}

fn get_required_bool(
    answers: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<bool, String> {
    answers
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| format!("interactive wizard did not produce required answer `{key}`"))
}

fn format_qa_error(err: QaLibError) -> String {
    match err {
        QaLibError::NeedsInteraction => "wizard QA flow requires interactive input".to_string(),
        other => format!("wizard QA flow failed: {other}"),
    }
}

fn selected_locale(answer_locale: Option<&str>, previous_locale: Option<&str>) -> String {
    answer_locale
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var("SORLA_LOCALE")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .or_else(|| previous_locale.map(str::to_string))
        .unwrap_or_else(|| "en".to_string())
}

fn sync_generated_artifacts(
    generated_dir: &Path,
    resolved: &ResolvedAnswers,
) -> Result<Vec<PathBuf>, String> {
    let mut desired = BTreeSet::new();
    let mut written = Vec::new();

    for artifact in &resolved.artifacts {
        let path = generated_dir.join(artifact);
        desired.insert(path.clone());
        write_artifact_file(&path, artifact, resolved)?;
        written.push(path);
    }

    if resolved.include_agent_tools
        && !resolved
            .artifacts
            .iter()
            .any(|artifact| artifact == "agent-tools.json")
    {
        let path = generated_dir.join("agent-tools.json");
        desired.insert(path.clone());
        write_artifact_file(&path, "agent-tools.json", resolved)?;
        written.push(path);
    }

    let known = default_artifacts()
        .into_iter()
        .map(|artifact| generated_dir.join(artifact))
        .collect::<Vec<_>>();
    for path in known {
        if path.exists() && !desired.contains(&path) {
            fs::remove_file(&path).map_err(|err| {
                format!(
                    "failed to remove stale generated file {}: {err}",
                    path.display()
                )
            })?;
        }
    }

    Ok(written)
}

fn write_artifact_file(
    path: &Path,
    artifact: &str,
    resolved: &ResolvedAnswers,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create directory {}: {err}", parent.display()))?;
    }

    if artifact == "agent-tools.json" {
        let provider_categories = [
            Some(resolved.storage_category.clone()),
            resolved.external_ref_category.clone(),
            resolved.agent_endpoint_provider_category.clone(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let payload = serde_json::json!({
            "package": resolved.package_name,
            "locale": resolved.locale,
            "provider_categories": provider_categories,
            "agent_endpoints": resolved.agent_endpoint_ids,
        });
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|err| err.to_string())?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))?;
        return Ok(());
    }

    if artifact == "provider-requirements.json" {
        let bytes = serde_json::to_vec_pretty(&build_provider_requirement_manifest(resolved))
            .map_err(|err| err.to_string())?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))?;
        return Ok(());
    }

    if artifact == "locale-manifest.json" {
        let bytes = serde_json::to_vec_pretty(&build_locale_manifest(resolved))
            .map_err(|err| err.to_string())?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))?;
        return Ok(());
    }

    let payload = serde_json::json!({
        "artifact": artifact,
        "package_name": resolved.package_name,
        "package_version": resolved.package_version,
        "default_source": resolved.default_source,
        "projection_mode": resolved.projection_mode,
        "compatibility_mode": resolved.compatibility_mode,
    });
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&payload, &mut bytes).map_err(|err| err.to_string())?;
    fs::write(path, bytes)
        .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))?;
    Ok(())
}

fn relative_to_output(output_dir: &Path, path: &Path) -> String {
    path.strip_prefix(output_dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn to_pascal_case(input: &str) -> String {
    input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => {
                    let mut value = String::new();
                    value.push(first.to_ascii_uppercase());
                    value.push_str(chars.as_str());
                    value
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn title_from_identifier(input: &str) -> String {
    input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn default_schema() -> WizardSchema {
    WizardSchema {
        schema_version: "0.4",
        wizard_version: "0.4",
        package_version: "0.1.0",
        locale: selected_locale(None, None),
        fallback_locale: "en",
        supported_modes: vec![SchemaFlow::Create, SchemaFlow::Update],
        provider_repo: "greentic-sorla-providers",
        generated_content_strategy: "rewrite-generated-blocks-only",
        user_content_strategy: "preserve-outside-generated-blocks",
        artifact_references: vec![
            "model.cbor",
            "actions.cbor",
            "events.cbor",
            "projections.cbor",
            "policies.cbor",
            "approvals.cbor",
            "views.cbor",
            "external-sources.cbor",
            "compatibility.cbor",
            "provider-contract.cbor",
            "package-manifest.cbor",
            "agent-tools.json",
            "provider-requirements.json",
            "locale-manifest.json",
        ],
        sections: vec![
            WizardSection {
                id: "package-bootstrap",
                title_key: "wizard.sections.package.title",
                description_key: "wizard.sections.package.description",
                flows: vec![SchemaFlow::Create],
                questions: vec![
                    WizardQuestion {
                        id: "package.name",
                        label_key: "wizard.questions.package_name.label",
                        help_key: Some("wizard.questions.package_name.help"),
                        kind: WizardQuestionKind::Text,
                        required: true,
                        default_value: None,
                        choices: vec![],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "package.version",
                        label_key: "wizard.questions.package_version.label",
                        help_key: Some("wizard.questions.package_version.help"),
                        kind: WizardQuestionKind::Text,
                        required: true,
                        default_value: Some("0.1.0"),
                        choices: vec![],
                        visibility: None,
                    },
                ],
            },
            WizardSection {
                id: "package-update",
                title_key: "wizard.sections.update.title",
                description_key: "wizard.sections.update.description",
                flows: vec![SchemaFlow::Update],
                questions: vec![
                    WizardQuestion {
                        id: "update.mode",
                        label_key: "wizard.questions.update_mode.label",
                        help_key: Some("wizard.questions.update_mode.help"),
                        kind: WizardQuestionKind::SingleSelect,
                        required: true,
                        default_value: Some("safe-update"),
                        choices: vec![
                            WizardChoice {
                                value: "safe-update",
                                label_key: "wizard.choices.update_mode.safe",
                            },
                            WizardChoice {
                                value: "refresh-generated",
                                label_key: "wizard.choices.update_mode.refresh",
                            },
                        ],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "update.partial_answers",
                        label_key: "wizard.questions.partial_answers.label",
                        help_key: Some("wizard.questions.partial_answers.help"),
                        kind: WizardQuestionKind::Boolean,
                        required: true,
                        default_value: Some("true"),
                        choices: vec![],
                        visibility: None,
                    },
                ],
            },
            WizardSection {
                id: "provider-requirements",
                title_key: "wizard.sections.providers.title",
                description_key: "wizard.sections.providers.description",
                flows: vec![SchemaFlow::Create, SchemaFlow::Update],
                questions: vec![
                    WizardQuestion {
                        id: "providers.storage.category",
                        label_key: "wizard.questions.storage_provider.label",
                        help_key: Some("wizard.questions.storage_provider.help"),
                        kind: WizardQuestionKind::SingleSelect,
                        required: true,
                        default_value: Some("storage"),
                        choices: vec![WizardChoice {
                            value: "storage",
                            label_key: "wizard.choices.provider_category.storage",
                        }],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "providers.external_ref.category",
                        label_key: "wizard.questions.external_ref_provider.label",
                        help_key: Some("wizard.questions.external_ref_provider.help"),
                        kind: WizardQuestionKind::SingleSelect,
                        required: false,
                        default_value: Some("external-ref"),
                        choices: vec![WizardChoice {
                            value: "external-ref",
                            label_key: "wizard.choices.provider_category.external_ref",
                        }],
                        visibility: Some(SchemaVisibility {
                            depends_on: "records.has_external_or_hybrid",
                            equals: "true",
                        }),
                    },
                    WizardQuestion {
                        id: "providers.hints",
                        label_key: "wizard.questions.provider_hints.label",
                        help_key: Some("wizard.questions.provider_hints.help"),
                        kind: WizardQuestionKind::TextList,
                        required: false,
                        default_value: None,
                        choices: vec![],
                        visibility: None,
                    },
                ],
            },
            WizardSection {
                id: "external-sources",
                title_key: "wizard.sections.external_sources.title",
                description_key: "wizard.sections.external_sources.description",
                flows: vec![SchemaFlow::Create, SchemaFlow::Update],
                questions: vec![
                    WizardQuestion {
                        id: "records.default_source",
                        label_key: "wizard.questions.default_source.label",
                        help_key: Some("wizard.questions.default_source.help"),
                        kind: WizardQuestionKind::SingleSelect,
                        required: true,
                        default_value: Some("native"),
                        choices: vec![
                            WizardChoice {
                                value: "native",
                                label_key: "wizard.choices.record_source.native",
                            },
                            WizardChoice {
                                value: "external",
                                label_key: "wizard.choices.record_source.external",
                            },
                            WizardChoice {
                                value: "hybrid",
                                label_key: "wizard.choices.record_source.hybrid",
                            },
                        ],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "records.external_ref.system",
                        label_key: "wizard.questions.external_system.label",
                        help_key: Some("wizard.questions.external_system.help"),
                        kind: WizardQuestionKind::Text,
                        required: false,
                        default_value: None,
                        choices: vec![],
                        visibility: Some(SchemaVisibility {
                            depends_on: "records.default_source",
                            equals: "external-or-hybrid",
                        }),
                    },
                ],
            },
            WizardSection {
                id: "events-projections",
                title_key: "wizard.sections.events.title",
                description_key: "wizard.sections.events.description",
                flows: vec![SchemaFlow::Create, SchemaFlow::Update],
                questions: vec![
                    WizardQuestion {
                        id: "events.enabled",
                        label_key: "wizard.questions.events_enabled.label",
                        help_key: Some("wizard.questions.events_enabled.help"),
                        kind: WizardQuestionKind::Boolean,
                        required: true,
                        default_value: Some("true"),
                        choices: vec![],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "projections.mode",
                        label_key: "wizard.questions.projection_mode.label",
                        help_key: Some("wizard.questions.projection_mode.help"),
                        kind: WizardQuestionKind::SingleSelect,
                        required: true,
                        default_value: Some("current-state"),
                        choices: vec![
                            WizardChoice {
                                value: "current-state",
                                label_key: "wizard.choices.projection_mode.current_state",
                            },
                            WizardChoice {
                                value: "audit-trail",
                                label_key: "wizard.choices.projection_mode.audit_trail",
                            },
                        ],
                        visibility: None,
                    },
                ],
            },
            WizardSection {
                id: "compatibility",
                title_key: "wizard.sections.compatibility.title",
                description_key: "wizard.sections.compatibility.description",
                flows: vec![SchemaFlow::Create, SchemaFlow::Update],
                questions: vec![WizardQuestion {
                    id: "migrations.compatibility",
                    label_key: "wizard.questions.compatibility_mode.label",
                    help_key: Some("wizard.questions.compatibility_mode.help"),
                    kind: WizardQuestionKind::SingleSelect,
                    required: true,
                    default_value: Some("additive"),
                    choices: vec![
                        WizardChoice {
                            value: "additive",
                            label_key: "wizard.choices.compatibility.additive",
                        },
                        WizardChoice {
                            value: "backward-compatible",
                            label_key: "wizard.choices.compatibility.backward_compatible",
                        },
                        WizardChoice {
                            value: "breaking",
                            label_key: "wizard.choices.compatibility.breaking",
                        },
                    ],
                    visibility: None,
                }],
            },
            WizardSection {
                id: "agent-endpoints",
                title_key: "wizard.sections.agent_endpoints.title",
                description_key: "wizard.sections.agent_endpoints.description",
                flows: vec![SchemaFlow::Create, SchemaFlow::Update],
                questions: vec![
                    WizardQuestion {
                        id: "agent_endpoints.enabled",
                        label_key: "wizard.questions.agent_endpoints_enabled.label",
                        help_key: Some("wizard.questions.agent_endpoints_enabled.help"),
                        kind: WizardQuestionKind::Boolean,
                        required: true,
                        default_value: Some("false"),
                        choices: vec![],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "agent_endpoints.ids",
                        label_key: "wizard.questions.agent_endpoint_ids.label",
                        help_key: Some("wizard.questions.agent_endpoint_ids.help"),
                        kind: WizardQuestionKind::TextList,
                        required: false,
                        default_value: None,
                        choices: vec![],
                        visibility: Some(SchemaVisibility {
                            depends_on: "agent_endpoints.enabled",
                            equals: "true",
                        }),
                    },
                    WizardQuestion {
                        id: "agent_endpoints.default_risk",
                        label_key: "wizard.questions.agent_endpoint_default_risk.label",
                        help_key: Some("wizard.questions.agent_endpoint_default_risk.help"),
                        kind: WizardQuestionKind::SingleSelect,
                        required: true,
                        default_value: Some("medium"),
                        choices: vec![
                            WizardChoice {
                                value: "low",
                                label_key: "wizard.choices.agent_endpoint_risk.low",
                            },
                            WizardChoice {
                                value: "medium",
                                label_key: "wizard.choices.agent_endpoint_risk.medium",
                            },
                            WizardChoice {
                                value: "high",
                                label_key: "wizard.choices.agent_endpoint_risk.high",
                            },
                        ],
                        visibility: Some(SchemaVisibility {
                            depends_on: "agent_endpoints.enabled",
                            equals: "true",
                        }),
                    },
                    WizardQuestion {
                        id: "agent_endpoints.default_approval",
                        label_key: "wizard.questions.agent_endpoint_default_approval.label",
                        help_key: Some("wizard.questions.agent_endpoint_default_approval.help"),
                        kind: WizardQuestionKind::SingleSelect,
                        required: true,
                        default_value: Some("policy-driven"),
                        choices: vec![
                            WizardChoice {
                                value: "none",
                                label_key: "wizard.choices.agent_endpoint_approval.none",
                            },
                            WizardChoice {
                                value: "optional",
                                label_key: "wizard.choices.agent_endpoint_approval.optional",
                            },
                            WizardChoice {
                                value: "required",
                                label_key: "wizard.choices.agent_endpoint_approval.required",
                            },
                            WizardChoice {
                                value: "policy-driven",
                                label_key: "wizard.choices.agent_endpoint_approval.policy_driven",
                            },
                        ],
                        visibility: Some(SchemaVisibility {
                            depends_on: "agent_endpoints.enabled",
                            equals: "true",
                        }),
                    },
                    WizardQuestion {
                        id: "agent_endpoints.exports",
                        label_key: "wizard.questions.agent_endpoint_exports.label",
                        help_key: Some("wizard.questions.agent_endpoint_exports.help"),
                        kind: WizardQuestionKind::MultiSelect,
                        required: true,
                        default_value: Some("openapi,arazzo,mcp,llms_txt"),
                        choices: vec![
                            WizardChoice {
                                value: "openapi",
                                label_key: "wizard.choices.agent_endpoint_export.openapi",
                            },
                            WizardChoice {
                                value: "arazzo",
                                label_key: "wizard.choices.agent_endpoint_export.arazzo",
                            },
                            WizardChoice {
                                value: "mcp",
                                label_key: "wizard.choices.agent_endpoint_export.mcp",
                            },
                            WizardChoice {
                                value: "llms_txt",
                                label_key: "wizard.choices.agent_endpoint_export.llms_txt",
                            },
                        ],
                        visibility: Some(SchemaVisibility {
                            depends_on: "agent_endpoints.enabled",
                            equals: "true",
                        }),
                    },
                    WizardQuestion {
                        id: "agent_endpoints.provider_category",
                        label_key: "wizard.questions.agent_endpoint_provider_category.label",
                        help_key: Some("wizard.questions.agent_endpoint_provider_category.help"),
                        kind: WizardQuestionKind::Text,
                        required: false,
                        default_value: Some("api-gateway"),
                        choices: vec![],
                        visibility: Some(SchemaVisibility {
                            depends_on: "agent_endpoints.enabled",
                            equals: "true",
                        }),
                    },
                ],
            },
            WizardSection {
                id: "output-preferences",
                title_key: "wizard.sections.output.title",
                description_key: "wizard.sections.output.description",
                flows: vec![SchemaFlow::Create, SchemaFlow::Update],
                questions: vec![
                    WizardQuestion {
                        id: "output.include_agent_tools",
                        label_key: "wizard.questions.include_agent_tools.label",
                        help_key: Some("wizard.questions.include_agent_tools.help"),
                        kind: WizardQuestionKind::Boolean,
                        required: true,
                        default_value: Some("true"),
                        choices: vec![],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "output.artifacts",
                        label_key: "wizard.questions.output_artifacts.label",
                        help_key: Some("wizard.questions.output_artifacts.help"),
                        kind: WizardQuestionKind::MultiSelect,
                        required: true,
                        default_value: Some(
                            "model.cbor,actions.cbor,events.cbor,projections.cbor,policies.cbor,approvals.cbor,views.cbor,external-sources.cbor,compatibility.cbor,provider-contract.cbor,package-manifest.cbor,agent-tools.json,provider-requirements.json,locale-manifest.json",
                        ),
                        choices: vec![
                            WizardChoice {
                                value: "model.cbor",
                                label_key: "wizard.artifacts.model.cbor",
                            },
                            WizardChoice {
                                value: "actions.cbor",
                                label_key: "wizard.artifacts.actions.cbor",
                            },
                            WizardChoice {
                                value: "events.cbor",
                                label_key: "wizard.artifacts.events.cbor",
                            },
                            WizardChoice {
                                value: "projections.cbor",
                                label_key: "wizard.artifacts.projections.cbor",
                            },
                            WizardChoice {
                                value: "policies.cbor",
                                label_key: "wizard.artifacts.policies.cbor",
                            },
                            WizardChoice {
                                value: "approvals.cbor",
                                label_key: "wizard.artifacts.approvals.cbor",
                            },
                            WizardChoice {
                                value: "views.cbor",
                                label_key: "wizard.artifacts.views.cbor",
                            },
                            WizardChoice {
                                value: "external-sources.cbor",
                                label_key: "wizard.artifacts.external-sources.cbor",
                            },
                            WizardChoice {
                                value: "compatibility.cbor",
                                label_key: "wizard.artifacts.compatibility.cbor",
                            },
                            WizardChoice {
                                value: "provider-contract.cbor",
                                label_key: "wizard.artifacts.provider-contract.cbor",
                            },
                            WizardChoice {
                                value: "package-manifest.cbor",
                                label_key: "wizard.artifacts.package-manifest.cbor",
                            },
                            WizardChoice {
                                value: "agent-tools.json",
                                label_key: "wizard.artifacts.agent-tools.json",
                            },
                            WizardChoice {
                                value: "provider-requirements.json",
                                label_key: "wizard.artifacts.provider-requirements.json",
                            },
                            WizardChoice {
                                value: "locale-manifest.json",
                                label_key: "wizard.artifacts.locale-manifest.json",
                            },
                        ],
                        visibility: None,
                    },
                ],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn public_help_includes_wizard_and_pack_surface() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("wizard"));
        assert!(help.contains("--schema"));
        assert!(help.contains("--answers"));
        assert!(help.contains("--pack-out"));
        assert!(help.contains("pack"));
        assert!(help.contains("--out"));
        assert!(!help.contains("__inspect-product-shape"));
    }

    #[test]
    fn cli_schema_includes_create_and_update_modes() {
        let schema = default_schema();
        assert!(schema.supported_modes.contains(&SchemaFlow::Create));
        assert!(schema.supported_modes.contains(&SchemaFlow::Update));
        assert_eq!(schema.fallback_locale, "en");
        assert!(
            schema
                .artifact_references
                .contains(&"provider-requirements.json")
        );
        assert!(
            schema
                .sections
                .iter()
                .any(|section| section.id == "output-preferences")
        );
        let agent_section = schema
            .sections
            .iter()
            .find(|section| section.id == "agent-endpoints")
            .expect("schema should include agent endpoints section");
        assert!(agent_section.flows.contains(&SchemaFlow::Create));
        assert!(agent_section.flows.contains(&SchemaFlow::Update));
        assert!(agent_section.questions.iter().any(|question| {
            question.id == "agent_endpoints.ids"
                && question.visibility
                    == Some(SchemaVisibility {
                        depends_on: "agent_endpoints.enabled",
                        equals: "true",
                    })
        }));
    }

    #[test]
    fn schema_references_existing_english_i18n_keys() {
        let i18n_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("i18n/en.json");
        let raw = fs::read_to_string(i18n_path).expect("English i18n file should be readable");
        let keys: BTreeMap<String, String> =
            serde_json::from_str(&raw).expect("English i18n should parse");
        let schema = default_schema();

        for section in &schema.sections {
            assert!(keys.contains_key(section.title_key));
            assert!(keys.contains_key(section.description_key));
            for question in &section.questions {
                assert!(keys.contains_key(question.label_key));
                if let Some(help_key) = question.help_key {
                    assert!(keys.contains_key(help_key));
                }
                for choice in &question.choices {
                    assert!(keys.contains_key(choice.label_key));
                }
            }
        }
    }

    #[test]
    fn schema_uses_locale_environment_with_fallback() {
        unsafe {
            std::env::set_var("SORLA_LOCALE", "fr");
        }
        let schema = default_schema();
        unsafe {
            std::env::remove_var("SORLA_LOCALE");
        }
        assert_eq!(schema.locale, "fr");
        assert_eq!(schema.fallback_locale, "en");
    }

    #[test]
    fn embedded_i18n_catalogs_are_available_without_filesystem_lookups() {
        let resolved = load_interactive_i18n("es").expect("embedded locale should load");
        assert!(
            resolved
                .get("wizard.title")
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
        );
    }

    #[test]
    fn create_flow_generates_package_and_lock_file() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("create.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();

        fs::write(
            &answers_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "create",
  "output_dir": "{}",
  "package": {{
    "name": "tenancy",
    "version": "0.2.0"
  }},
  "records": {{
    "default_source": "hybrid",
    "external_ref_system": "crm"
  }},
  "providers": {{
    "hints": ["crm"]
  }}
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_path.to_str().unwrap(),
        ])
        .unwrap();

        let package_yaml = fs::read_to_string(output_dir.join("sorla.yaml")).unwrap();
        assert!(package_yaml.contains("package:"));
        assert!(package_yaml.contains("source: hybrid"));
        assert!(package_yaml.contains(GENERATED_BEGIN));

        let lock = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join(LOCK_FILENAME),
        )
        .unwrap();
        assert!(lock.contains("\"package_name\": \"tenancy\""));
        assert!(lock.contains("\"locale\": \"en\""));
        assert!(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("model.cbor")
                .exists()
        );
        let manifest = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("package-manifest.json"),
        )
        .unwrap();
        assert!(manifest.contains("\"package_kind\": \"greentic-sorla-package\""));
        assert!(manifest.contains("\"fallback_locale\": \"en\""));

        let locale_manifest = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("locale-manifest.json"),
        )
        .unwrap();
        assert!(locale_manifest.contains("\"default_locale\": \"en\""));

        let provider_manifest = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("provider-requirements.json"),
        )
        .unwrap();
        assert!(provider_manifest.contains("\"name\": \"storage\""));
    }

    #[test]
    fn wizard_answers_can_generate_gtpack_without_repo_fixture() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("create-pack.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();

        fs::write(
            &answers_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "create",
  "output_dir": "{}",
  "package": {{
    "name": "landlord-tenant-sor",
    "version": "0.1.0"
  }},
  "records": {{
    "default_source": "native"
  }},
  "events": {{
    "enabled": true
  }},
  "agent_endpoints": {{
    "enabled": true,
    "ids": ["create_tenant"],
    "default_risk": "medium",
    "default_approval": "policy-driven",
    "exports": ["openapi", "arazzo", "mcp", "llms_txt"],
    "provider_category": "storage"
  }}
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_path.to_str().unwrap(),
            "--pack-out",
            "landlord-tenant-sor.gtpack",
        ])
        .unwrap();

        let pack_path = output_dir.join("landlord-tenant-sor.gtpack");
        assert!(pack_path.exists());
        doctor_sorla_gtpack(&pack_path).expect("wizard-generated pack should doctor");
        let inspection = inspect_sorla_gtpack(&pack_path).expect("wizard pack should inspect");
        assert_eq!(inspection.name, "landlord-tenant-sor");
        assert_eq!(inspection.extension, "greentic.sorx.runtime.v1");
    }

    #[test]
    fn landlord_tenant_pack_example_answers_generate_gtpack() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("landlord-tenant-pack.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();
        let example_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("answers")
            .join("landlord_tenant_pack.json");
        let example = fs::read_to_string(example_path).expect("example answers should read");
        let patched = example.replace(
            r#""output_dir": "target/greentic-sorla-landlord-tenant-pack-example""#,
            &format!(r#""output_dir": "{}""#, output_dir.display()),
        );
        fs::write(&answers_path, patched).unwrap();

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_path.to_str().unwrap(),
            "--pack-out",
            "landlord-tenant-sor.gtpack",
        ])
        .unwrap();

        let pack_path = output_dir.join("landlord-tenant-sor.gtpack");
        doctor_sorla_gtpack(&pack_path).expect("example pack should doctor");
        let inspection = inspect_sorla_gtpack(&pack_path).expect("example pack should inspect");
        assert_eq!(inspection.name, "landlord-tenant-sor");
        assert_eq!(
            inspection
                .optional_artifacts
                .get("assets/sorla/mcp-tools.json"),
            Some(&true)
        );
    }

    #[test]
    fn wizard_schema_rejects_pack_out() {
        let err = run([
            "greentic-sorla",
            "wizard",
            "--schema",
            "--pack-out",
            "schema.gtpack",
        ])
        .expect_err("schema mode should reject pack-out");

        assert!(err.contains("`--pack-out` can only be used"));
    }

    #[test]
    fn create_flow_generates_agent_endpoints_from_answers() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("create-agent.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();

        fs::write(
            &answers_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "create",
  "output_dir": "{}",
  "package": {{
    "name": "lead-capture",
    "version": "0.2.0"
  }},
  "agent_endpoints": {{
    "enabled": true,
    "ids": ["create_customer_contact", "request_partner_followup"],
    "default_risk": "medium",
    "default_approval": "policy-driven",
    "exports": ["openapi", "mcp"],
    "provider_category": "api-gateway"
  }}
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_path.to_str().unwrap(),
        ])
        .unwrap();

        let package_yaml = fs::read_to_string(output_dir.join("sorla.yaml")).unwrap();
        assert!(package_yaml.contains("agent_endpoints:"));
        assert!(package_yaml.contains("id: create_customer_contact"));
        assert!(package_yaml.contains("id: request_partner_followup"));
        assert!(package_yaml.contains("category: api-gateway"));
        assert!(package_yaml.contains("openapi: true"));
        assert!(package_yaml.contains("arazzo: false"));
        assert!(package_yaml.contains("mcp: true"));
        assert!(package_yaml.contains("llms_txt: false"));
        serde_yaml::from_str::<serde_yaml::Value>(&package_yaml)
            .expect("generated agent endpoint YAML should be valid YAML");

        let lock = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join(LOCK_FILENAME),
        )
        .unwrap();
        assert!(lock.contains("\"agent_endpoints_enabled\": true"));

        let provider_manifest = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("provider-requirements.json"),
        )
        .unwrap();
        assert!(provider_manifest.contains("\"agent_endpoint_handoff\""));
    }

    #[test]
    fn interactive_wizard_uses_qa_and_reuses_answers_pipeline() {
        let dir = unique_temp_dir();
        let output_dir = dir.join("interactive-workspace");
        fs::create_dir_all(&output_dir).unwrap();
        let provider_output_dir = output_dir.clone();

        let mut provider = move |question_id: &str, _question: &serde_json::Value| match question_id
        {
            "flow" => Ok(serde_json::json!("create")),
            "output_dir" => Ok(serde_json::json!(provider_output_dir.display().to_string())),
            "locale" => Ok(serde_json::json!("fr")),
            "package_name" => Ok(serde_json::json!("qa-demo")),
            "package_version" => Ok(serde_json::json!("0.3.0")),
            "storage_category" => Ok(serde_json::json!("storage")),
            "default_source" => Ok(serde_json::json!("external")),
            "external_ref_system" => Ok(serde_json::json!("crm")),
            "external_ref_category" => Ok(serde_json::json!("external-ref")),
            "events_enabled" => Ok(serde_json::json!(true)),
            "projection_mode" => Ok(serde_json::json!("current-state")),
            "compatibility_mode" => Ok(serde_json::json!("additive")),
            "agent_endpoints_enabled" => Ok(serde_json::json!(false)),
            "agent_endpoint_ids" => Ok(serde_json::json!("")),
            "agent_endpoint_default_risk" => Ok(serde_json::json!("medium")),
            "agent_endpoint_default_approval" => Ok(serde_json::json!("policy-driven")),
            "agent_endpoint_exports" => Ok(serde_json::json!("openapi,arazzo,mcp,llms_txt")),
            "agent_endpoint_provider_category" => Ok(serde_json::json!("api-gateway")),
            "include_agent_tools" => Ok(serde_json::json!(true)),
            other => panic!("unexpected interactive question: {other}"),
        };

        let summary = run_interactive_wizard_with_provider("fr", &mut provider, None).unwrap();
        assert_eq!(summary.mode, "create");
        assert_eq!(summary.package_name, "qa-demo");
        assert_eq!(summary.locale, "fr");

        let lock = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join(LOCK_FILENAME),
        )
        .unwrap();
        assert!(lock.contains("\"default_source\": \"external\""));
        assert!(lock.contains("\"locale\": \"fr\""));
    }

    #[test]
    fn update_flow_preserves_user_content_and_is_idempotent() {
        let dir = unique_temp_dir();
        let create_path = dir.join("create.json");
        let update_path = dir.join("update.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();

        fs::write(
            &create_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "create",
  "output_dir": "{}",
  "package": {{
    "name": "tenancy",
    "version": "0.2.0"
  }}
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();
        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            create_path.to_str().unwrap(),
        ])
        .unwrap();

        let package_path = output_dir.join("sorla.yaml");
        let existing = fs::read_to_string(&package_path).unwrap();
        fs::write(&package_path, format!("user-notes: keep-me\n{existing}")).unwrap();

        fs::write(
            &update_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "update",
  "output_dir": "{}",
  "projections": {{
    "mode": "audit-trail"
  }},
  "output": {{
    "include_agent_tools": false,
    "artifacts": ["model.cbor", "events.cbor"]
  }}
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            update_path.to_str().unwrap(),
        ])
        .unwrap();
        let first_updated = fs::read_to_string(&package_path).unwrap();
        assert!(first_updated.contains("user-notes: keep-me"));
        assert!(first_updated.contains("mode: audit-trail"));
        assert!(
            !output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("agent-tools.json")
                .exists()
        );

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            update_path.to_str().unwrap(),
        ])
        .unwrap();
        let second_updated = fs::read_to_string(&package_path).unwrap();
        assert_eq!(first_updated, second_updated);
    }

    #[test]
    fn validation_error_is_actionable() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("invalid.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &answers_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "create",
  "output_dir": "{}",
  "package": {{
    "version": "0.2.0"
  }}
}}"#,
                dir.display()
            ),
        )
        .unwrap();

        let error = run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_path.to_str().unwrap(),
        ])
        .expect_err("missing package.name should fail");

        assert!(error.contains("package.name"));
    }

    #[test]
    fn update_flow_uses_previous_locale_when_not_overridden() {
        let dir = unique_temp_dir();
        let create_path = dir.join("create.json");
        let update_path = dir.join("update.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();

        fs::write(
            &create_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "create",
  "output_dir": "{}",
  "locale": "nl",
  "package": {{
    "name": "tenancy",
    "version": "0.2.0"
  }}
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();
        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            create_path.to_str().unwrap(),
        ])
        .unwrap();

        fs::write(
            &update_path,
            format!(
                r#"{{
  "schema_version": "0.4",
  "flow": "update",
  "output_dir": "{}"
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            update_path.to_str().unwrap(),
        ])
        .unwrap();

        let lock = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join(LOCK_FILENAME),
        )
        .unwrap();
        assert!(lock.contains("\"locale\": \"nl\""));
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "greentic-sorla-cli-tests-{}-{}",
            std::process::id(),
            nanos
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir_all(&path).unwrap();
        path
    }
}
