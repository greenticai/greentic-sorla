#![cfg_attr(
    any(not(feature = "cli"), not(feature = "pack-zip")),
    allow(dead_code, unused_imports)
)]

use clap::{ArgAction, Args, Parser, Subcommand};
#[cfg(feature = "cli")]
use greentic_qa_lib::{
    AnswerProvider, I18nConfig, QaLibError, ResolvedI18nMap, WizardDriver, WizardFrontend,
    WizardRunConfig,
};
pub use greentic_sorla_pack::{
    AgentEndpointActionCatalogDocument, DEFAULT_DESIGNER_COMPONENT_OPERATION,
    DEFAULT_DESIGNER_COMPONENT_REF, DesignerNodeType, DesignerNodeTypesDocument,
};
use greentic_sorla_pack::{
    DesignerNodeTypeGenerationOptions, PROVIDER_BINDINGS_TEMPLATE_FILENAME,
    RUNTIME_TEMPLATE_FILENAME, SORX_COMPATIBILITY_SCHEMA, SORX_EXPOSURE_POLICY_SCHEMA,
    SORX_VALIDATION_SCHEMA, START_SCHEMA_FILENAME, SorlaGtpackInspection, SorlaGtpackOptions,
    build_handoff_artifacts_from_yaml, generate_agent_endpoint_action_catalog_from_ir,
    generate_designer_node_types_from_ir, generate_sorx_validation_manifest_from_ir,
    ontology_schema_json, retrieval_bindings_schema_json, sorx_validation_schema_json,
};
#[cfg(feature = "pack-zip")]
use greentic_sorla_pack::{build_sorla_gtpack, doctor_sorla_gtpack, inspect_sorla_gtpack};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

mod embedded_i18n {
    include!(concat!(env!("OUT_DIR"), "/embedded_i18n.rs"));
}

const GENERATED_BEGIN: &str = "# --- BEGIN GREENTIC-SORLA GENERATED ---";
const GENERATED_END: &str = "# --- END GREENTIC-SORLA GENERATED ---";
const LOCK_FILENAME: &str = "answers.lock.json";
const LEGACY_PACKAGE_MANIFEST_FILENAME: &str = "package-manifest.json";
const LAUNCHER_HANDOFF_FILENAME: &str = "launcher-handoff.json";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizeOptions;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidateOptions;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreviewOptions;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackBuildOptions {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerNodeTypeOptions {
    pub component_ref: String,
    pub operation: String,
}

impl Default for DesignerNodeTypeOptions {
    fn default() -> Self {
        Self {
            component_ref: DEFAULT_DESIGNER_COMPONENT_REF.to_string(),
            operation: DEFAULT_DESIGNER_COMPONENT_OPERATION.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedSorlaModel {
    pub package_name: String,
    pub package_version: String,
    pub locale: String,
    pub source_yaml: String,
    pub normalized_answers: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaValidationReport {
    pub diagnostics: Vec<SorlaDiagnostic>,
}

impl SorlaValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaPreview {
    pub summary: SorlaPreviewSummary,
    pub cards: Vec<SorlaPreviewCard>,
    pub graph: Option<SorlaPreviewGraph>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaPreviewSummary {
    pub package_name: String,
    pub package_version: String,
    pub records: usize,
    pub events: usize,
    pub projections: usize,
    pub agent_endpoints: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaPreviewCard {
    pub title: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaPreviewGraph {
    pub nodes: usize,
    pub edges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackBuildBytes {
    pub filename: String,
    #[serde(skip_serializing)]
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub metadata: PackBuildMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackBuildMetadata {
    pub pack_id: String,
    pub pack_version: String,
    pub sorla_package_name: String,
    pub sorla_package_version: String,
    pub ir_hash: String,
    pub assets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackEntry {
    pub path: String,
    #[serde(skip_serializing)]
    pub bytes: Vec<u8>,
    pub sha256: String,
}

pub type SorlaError = String;
pub type PackBuildResult = greentic_sorla_pack::SorlaGtpackBuildSummary;

#[cfg(not(feature = "pack-zip"))]
fn build_sorla_gtpack(_options: &SorlaGtpackOptions) -> Result<PackBuildResult, String> {
    Err("gtpack ZIP byte generation requires the `pack-zip` feature; use build_gtpack_entries for wasm-safe planning".to_string())
}

#[cfg(not(feature = "pack-zip"))]
fn inspect_sorla_gtpack(_path: &Path) -> Result<SorlaGtpackInspection, String> {
    Err("gtpack inspection from ZIP files requires the `pack-zip` feature".to_string())
}

#[cfg(not(feature = "pack-zip"))]
fn doctor_sorla_gtpack(
    _path: &Path,
) -> Result<greentic_sorla_pack::SorlaGtpackDoctorReport, String> {
    Err("gtpack doctor from ZIP files requires the `pack-zip` feature".to_string())
}

#[derive(Debug, Parser)]
#[command(
    name = "greentic-sorla",
    about = "Wizard-first tooling for Greentic SoRLa source layouts and handoff artifacts.",
    long_about = "greentic-sorla is a wizard-first tool for authoring SoRLa source layouts, extension handoff artifacts, and deterministic handoff packs.\n\nSupported product surface:\n  greentic-sorla wizard --schema\n  greentic-sorla wizard --answers <file>\n  greentic-sorla wizard --answers <file> --pack-out <file.gtpack>\n  greentic-sorla pack <file> --name <name> --version <version> --out <file.gtpack>\n",
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
    /// Locale used for wizard schema metadata and interactive prompts.
    #[arg(long)]
    locale: Option<String>,
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
    /// Emit deterministic JSON schemas for SORX handoff metadata.
    Schema(PackSchemaArgs),
    /// Inspect embedded SORX validation metadata as deterministic JSON.
    ValidationInspect(PackPathArgs),
    /// Validate embedded SORX validation metadata using pack doctor checks.
    ValidationDoctor(PackPathArgs),
}

#[derive(Debug, Args)]
struct PackSchemaArgs {
    #[command(subcommand)]
    command: PackSchemaCommand,
}

#[derive(Debug, Subcommand)]
enum PackSchemaCommand {
    /// Emit the greentic.sorx.validation.v1 schema.
    Validation,
    /// Emit the greentic.sorx.exposure-policy.v1 schema.
    ExposurePolicy,
    /// Emit the greentic.sorx.compatibility.v1 schema.
    Compatibility,
    /// Emit the greentic.sorla.ontology.v1 schema.
    Ontology,
    /// Emit the greentic.sorla.retrieval-bindings.v1 schema.
    RetrievalBindings,
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
    ontology: Option<OntologyAnswers>,
    #[serde(default)]
    semantic_aliases: Option<SemanticAliasesAnswer>,
    #[serde(default)]
    entity_linking: Option<EntityLinkingAnswer>,
    #[serde(default)]
    retrieval_bindings: Option<RetrievalBindingsAnswer>,
    #[serde(default)]
    actions: Vec<NamedAnswer>,
    #[serde(default)]
    events: Option<EventAnswers>,
    #[serde(default)]
    projections: Option<ProjectionAnswers>,
    #[serde(default)]
    provider_requirements: Vec<ProviderRequirementAnswer>,
    #[serde(default)]
    policies: Vec<NamedAnswer>,
    #[serde(default)]
    approvals: Vec<NamedAnswer>,
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
    #[serde(default)]
    items: Vec<RecordItemAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RecordItemAnswer {
    name: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    external_ref: Option<ExternalRefAnswer>,
    #[serde(default)]
    fields: Vec<FieldAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ExternalRefAnswer {
    system: String,
    key: String,
    #[serde(default)]
    authoritative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct FieldAnswer {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    sensitive: Option<bool>,
    #[serde(default)]
    enum_values: Vec<String>,
    #[serde(default)]
    references: Option<FieldReferenceAnswer>,
    #[serde(default)]
    authority: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct FieldReferenceAnswer {
    record: String,
    field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyAnswers {
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    concepts: Vec<OntologyConceptAnswer>,
    #[serde(default)]
    relationships: Vec<OntologyRelationshipAnswer>,
    #[serde(default)]
    constraints: Vec<OntologyConstraintAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyConceptAnswer {
    id: String,
    kind: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    extends: Vec<String>,
    #[serde(default)]
    backed_by: Option<OntologyBackingAnswer>,
    #[serde(default)]
    sensitivity: Option<OntologySensitivityAnswer>,
    #[serde(default)]
    policy_hooks: Vec<OntologyPolicyHookAnswer>,
    #[serde(default)]
    provider_requirements: Vec<ProviderRequirementAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyRelationshipAnswer {
    id: String,
    #[serde(default)]
    label: Option<String>,
    from: String,
    to: String,
    #[serde(default)]
    cardinality: Option<OntologyCardinalityAnswer>,
    #[serde(default)]
    backed_by: Option<OntologyBackingAnswer>,
    #[serde(default)]
    sensitivity: Option<OntologySensitivityAnswer>,
    #[serde(default)]
    policy_hooks: Vec<OntologyPolicyHookAnswer>,
    #[serde(default)]
    provider_requirements: Vec<ProviderRequirementAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyCardinalityAnswer {
    from: String,
    to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyBackingAnswer {
    record: String,
    #[serde(default)]
    from_field: Option<String>,
    #[serde(default)]
    to_field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologySensitivityAnswer {
    #[serde(default)]
    classification: Option<String>,
    #[serde(default)]
    pii: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyPolicyHookAnswer {
    policy: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyConstraintAnswer {
    id: String,
    applies_to: OntologyConstraintTargetAnswer,
    #[serde(default)]
    requires_policy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct OntologyConstraintTargetAnswer {
    concept: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct SemanticAliasesAnswer {
    #[serde(default)]
    concepts: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    relationships: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct EntityLinkingAnswer {
    #[serde(default)]
    strategies: Vec<EntityLinkingStrategyAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EntityLinkingStrategyAnswer {
    id: String,
    applies_to: String,
    #[serde(default)]
    source_type: Option<String>,
    #[serde(rename = "match")]
    match_fields: EntityLinkingMatchAnswer,
    confidence: serde_json::Number,
    #[serde(default)]
    sensitivity: Option<OntologySensitivityAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EntityLinkingMatchAnswer {
    source_field: String,
    target_field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RetrievalBindingsAnswer {
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    providers: Vec<RetrievalProviderAnswer>,
    #[serde(default)]
    scopes: Vec<RetrievalScopeAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RetrievalProviderAnswer {
    id: String,
    category: String,
    #[serde(default)]
    required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RetrievalScopeAnswer {
    id: String,
    applies_to: RetrievalScopeTargetAnswer,
    provider: String,
    #[serde(default)]
    filters: Option<RetrievalFilterAnswer>,
    #[serde(default)]
    permission: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RetrievalScopeTargetAnswer {
    #[serde(default)]
    concept: Option<String>,
    #[serde(default)]
    relationship: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RetrievalFilterAnswer {
    #[serde(default)]
    entity_scope: Option<EntityScopeAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct EntityScopeAnswer {
    #[serde(default)]
    include_self: Option<bool>,
    #[serde(default)]
    include_related: Vec<RelationshipTraversalAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct RelationshipTraversalAnswer {
    relationship: String,
    direction: String,
    max_depth: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct EventAnswers {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    items: Vec<EventItemAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct EventItemAnswer {
    name: String,
    record: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    emits: Vec<EventFieldAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct EventFieldAnswer {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ProjectionAnswers {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    items: Vec<ProjectionItemAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ProjectionItemAnswer {
    name: String,
    record: String,
    source_event: String,
    #[serde(default)]
    mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct MigrationAnswers {
    #[serde(default)]
    compatibility: Option<String>,
    #[serde(default)]
    items: Vec<MigrationItemAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct MigrationItemAnswer {
    name: String,
    #[serde(default)]
    compatibility: Option<String>,
    #[serde(default)]
    projection_updates: Vec<String>,
    #[serde(default)]
    backfills: Vec<MigrationBackfillAnswer>,
    #[serde(default)]
    idempotence_key: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct MigrationBackfillAnswer {
    record: String,
    field: String,
    #[serde(default)]
    default: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ProviderRequirementAnswer {
    category: String,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct NamedAnswer {
    name: String,
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
    #[serde(default)]
    items: Vec<AgentEndpointItemAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct AgentEndpointItemAnswer {
    id: String,
    title: String,
    intent: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    inputs: Vec<FieldAnswer>,
    #[serde(default)]
    outputs: Vec<FieldAnswer>,
    #[serde(default)]
    side_effects: Vec<String>,
    #[serde(default)]
    emits: Option<AgentEndpointEmitAnswer>,
    #[serde(default)]
    risk: Option<String>,
    #[serde(default)]
    approval: Option<String>,
    #[serde(default)]
    provider_requirements: Vec<ProviderRequirementAnswer>,
    #[serde(default)]
    backing: AgentEndpointBackingAnswer,
    #[serde(default)]
    agent_visibility: Option<AgentVisibilityAnswer>,
    #[serde(default)]
    examples: Vec<AgentEndpointExampleAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct AgentEndpointEmitAnswer {
    event: String,
    stream: String,
    #[serde(default)]
    payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct AgentEndpointBackingAnswer {
    #[serde(default)]
    actions: Vec<String>,
    #[serde(default)]
    events: Vec<String>,
    #[serde(default)]
    flows: Vec<String>,
    #[serde(default)]
    policies: Vec<String>,
    #[serde(default)]
    approvals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct AgentVisibilityAnswer {
    #[serde(default)]
    openapi: Option<bool>,
    #[serde(default)]
    arazzo: Option<bool>,
    #[serde(default)]
    mcp: Option<bool>,
    #[serde(default)]
    llms_txt: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct AgentEndpointExampleAnswer {
    name: String,
    summary: String,
    #[serde(default)]
    input: serde_json::Value,
    #[serde(default)]
    expected_output: serde_json::Value,
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
    #[serde(default)]
    record_items: Vec<RecordItemAnswer>,
    #[serde(default)]
    ontology: Option<OntologyAnswers>,
    #[serde(default)]
    semantic_aliases: Option<SemanticAliasesAnswer>,
    #[serde(default)]
    entity_linking: Option<EntityLinkingAnswer>,
    #[serde(default)]
    retrieval_bindings: Option<RetrievalBindingsAnswer>,
    #[serde(default)]
    actions: Vec<NamedAnswer>,
    #[serde(default)]
    event_items: Vec<EventItemAnswer>,
    #[serde(default)]
    projection_items: Vec<ProjectionItemAnswer>,
    #[serde(default)]
    provider_requirements: Vec<ProviderRequirementAnswer>,
    #[serde(default)]
    policies: Vec<NamedAnswer>,
    #[serde(default)]
    approvals: Vec<NamedAnswer>,
    #[serde(default)]
    migration_items: Vec<MigrationItemAnswer>,
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
    #[serde(default)]
    agent_endpoint_items: Vec<AgentEndpointItemAnswer>,
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

pub fn schema_for_answers() -> Result<serde_json::Value, SorlaError> {
    serde_json::to_value(default_schema()).map_err(|err| err.to_string())
}

pub fn normalize_answers(
    input: serde_json::Value,
    _options: NormalizeOptions,
) -> Result<NormalizedSorlaModel, SorlaError> {
    let answers: AnswersDocument = serde_json::from_value(input)
        .map_err(|err| format!("failed to parse answers document: {err}"))?;
    let schema = default_schema();
    validate_answers_document(&answers, &schema)?;
    let resolved = match answers.flow.as_str() {
        "create" => resolve_create_answers(&answers)?,
        "update" => {
            let output_dir = PathBuf::from(&answers.output_dir);
            let lock_path = output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join(LOCK_FILENAME);
            let previous = read_lock_file(&lock_path)?;
            resolve_update_answers(&answers, previous)?
        }
        _ => unreachable!("validated earlier"),
    };
    let source_yaml = render_package_yaml(&resolved);
    let normalized_answers = serde_json::to_value(&resolved).map_err(|err| err.to_string())?;
    Ok(NormalizedSorlaModel {
        package_name: resolved.package_name,
        package_version: resolved.package_version,
        locale: resolved.locale,
        source_yaml,
        normalized_answers,
    })
}

pub fn validate_model(
    model: &NormalizedSorlaModel,
    _options: ValidateOptions,
) -> SorlaValidationReport {
    match build_handoff_artifacts_from_yaml(&model.source_yaml) {
        Ok(_) => SorlaValidationReport {
            diagnostics: Vec::new(),
        },
        Err(message) => SorlaValidationReport {
            diagnostics: vec![SorlaDiagnostic {
                severity: DiagnosticSeverity::Error,
                code: "sorla.validation".to_string(),
                message,
                path: None,
                suggestion: Some("Update the SoRLa model and run validation again.".to_string()),
            }],
        },
    }
}

pub fn generate_preview(
    model: &NormalizedSorlaModel,
    _options: PreviewOptions,
) -> Result<SorlaPreview, SorlaError> {
    let artifacts = build_handoff_artifacts_from_yaml(&model.source_yaml)?;
    let ir = artifacts.ir;
    let ontology_nodes = ir
        .ontology
        .as_ref()
        .map(|ontology| ontology.concepts.len())
        .unwrap_or(0);
    let ontology_edges = ir
        .ontology
        .as_ref()
        .map(|ontology| ontology.relationships.len())
        .unwrap_or(0);
    Ok(SorlaPreview {
        summary: SorlaPreviewSummary {
            package_name: ir.package.name.clone(),
            package_version: ir.package.version.clone(),
            records: ir.records.len(),
            events: ir.events.len(),
            projections: ir.projections.len(),
            agent_endpoints: ir.agent_endpoints.len(),
        },
        cards: vec![
            SorlaPreviewCard {
                title: "Records".to_string(),
                items: ir
                    .records
                    .iter()
                    .map(|record| record.name.clone())
                    .collect(),
            },
            SorlaPreviewCard {
                title: "Agent endpoints".to_string(),
                items: ir
                    .agent_endpoints
                    .iter()
                    .map(|endpoint| endpoint.id.clone())
                    .collect(),
            },
        ],
        graph: Some(SorlaPreviewGraph {
            nodes: ir.records.len() + ontology_nodes,
            edges: ir.projections.len() + ontology_edges,
        }),
    })
}

pub fn list_designer_node_types(
    model: &NormalizedSorlaModel,
    options: DesignerNodeTypeOptions,
) -> Result<DesignerNodeTypesDocument, SorlaError> {
    let artifacts = build_handoff_artifacts_from_yaml(&model.source_yaml)?;
    generate_designer_node_types_from_ir(
        &artifacts.ir,
        &DesignerNodeTypeGenerationOptions {
            component_ref: options.component_ref,
            operation: options.operation,
        },
    )
}

pub fn agent_endpoint_action_catalog(
    model: &NormalizedSorlaModel,
) -> Result<AgentEndpointActionCatalogDocument, SorlaError> {
    let artifacts = build_handoff_artifacts_from_yaml(&model.source_yaml)?;
    generate_agent_endpoint_action_catalog_from_ir(&artifacts.ir)
}

#[cfg(feature = "pack-zip")]
pub fn build_gtpack_bytes(
    model: &NormalizedSorlaModel,
    options: PackBuildOptions,
) -> Result<PackBuildBytes, SorlaError> {
    let filename = format!(
        "{}.gtpack",
        options
            .name
            .clone()
            .unwrap_or_else(|| model.package_name.clone())
    );
    let temp_path = deterministic_temp_path(&filename);
    let summary = build_gtpack_file(model, &temp_path, options)?;
    let bytes = fs::read(&temp_path).map_err(|err| {
        format!(
            "failed to read generated gtpack {}: {err}",
            temp_path.display()
        )
    })?;
    let _ = fs::remove_file(&temp_path);
    Ok(PackBuildBytes {
        filename,
        sha256: sha256_hex_public(&bytes),
        bytes,
        metadata: PackBuildMetadata {
            pack_id: summary.name,
            pack_version: summary.version,
            sorla_package_name: summary.sorla_package_name,
            sorla_package_version: summary.sorla_package_version,
            ir_hash: summary.ir_hash,
            assets: summary.assets,
        },
    })
}

pub fn build_gtpack_entries(
    model: &NormalizedSorlaModel,
    _options: PackBuildOptions,
) -> Result<Vec<PackEntry>, SorlaError> {
    let artifacts = build_handoff_artifacts_from_yaml(&model.source_yaml)?;
    let mut entries = Vec::new();
    entries.push(PackEntry {
        path: "assets/sorla/inspect.json".to_string(),
        bytes: artifacts.inspect_json.as_bytes().to_vec(),
        sha256: sha256_hex_public(artifacts.inspect_json.as_bytes()),
    });
    entries.push(PackEntry {
        path: "assets/sorla/agent-tools.json".to_string(),
        bytes: artifacts.agent_tools_json.as_bytes().to_vec(),
        sha256: sha256_hex_public(artifacts.agent_tools_json.as_bytes()),
    });
    entries.push(PackEntry {
        path: "assets/sorla/executable-contract.json".to_string(),
        bytes: artifacts.executable_contract_json.as_bytes().to_vec(),
        sha256: sha256_hex_public(artifacts.executable_contract_json.as_bytes()),
    });
    if !artifacts.ir.agent_endpoints.is_empty() {
        entries.push(PackEntry {
            path: "assets/sorla/designer-node-types.json".to_string(),
            bytes: artifacts.designer_node_types_json.as_bytes().to_vec(),
            sha256: sha256_hex_public(artifacts.designer_node_types_json.as_bytes()),
        });
        entries.push(PackEntry {
            path: "assets/sorla/agent-endpoint-action-catalog.json".to_string(),
            bytes: artifacts
                .agent_endpoint_action_catalog_json
                .as_bytes()
                .to_vec(),
            sha256: sha256_hex_public(artifacts.agent_endpoint_action_catalog_json.as_bytes()),
        });
    }
    for (name, bytes) in artifacts.cbor_artifacts {
        entries.push(PackEntry {
            path: format!("assets/sorla/{name}"),
            sha256: sha256_hex_public(&bytes),
            bytes,
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

#[cfg(feature = "pack-zip")]
pub fn build_gtpack_file(
    model: &NormalizedSorlaModel,
    output_path: &Path,
    options: PackBuildOptions,
) -> Result<PackBuildResult, SorlaError> {
    let yaml_path = deterministic_temp_path("model.sorla.yaml");
    fs::write(&yaml_path, &model.source_yaml)
        .map_err(|err| format!("failed to write temporary SoRLa model: {err}"))?;
    let result = build_sorla_gtpack(&SorlaGtpackOptions {
        input_path: yaml_path.clone(),
        name: options.name.unwrap_or_else(|| model.package_name.clone()),
        version: options
            .version
            .unwrap_or_else(|| model.package_version.clone()),
        out_path: output_path.to_path_buf(),
    });
    let _ = fs::remove_file(&yaml_path);
    result
}

#[cfg(feature = "pack-zip")]
pub fn inspect_gtpack_bytes(bytes: &[u8]) -> Result<SorlaGtpackInspection, SorlaError> {
    let path = deterministic_temp_path("inspect.gtpack");
    fs::write(&path, bytes).map_err(|err| format!("failed to write temporary gtpack: {err}"))?;
    let result = inspect_sorla_gtpack(&path);
    let _ = fs::remove_file(&path);
    result
}

#[cfg(feature = "pack-zip")]
pub fn doctor_gtpack_bytes(bytes: &[u8]) -> SorlaValidationReport {
    let path = deterministic_temp_path("doctor.gtpack");
    if let Err(err) = fs::write(&path, bytes) {
        return SorlaValidationReport {
            diagnostics: vec![SorlaDiagnostic {
                severity: DiagnosticSeverity::Error,
                code: "sorla.gtpack.write".to_string(),
                message: format!("failed to write temporary gtpack: {err}"),
                path: None,
                suggestion: None,
            }],
        };
    }
    let result = doctor_sorla_gtpack(&path);
    let _ = fs::remove_file(&path);
    match result {
        Ok(report) if report.status == "ok" => SorlaValidationReport {
            diagnostics: Vec::new(),
        },
        Ok(report) => SorlaValidationReport {
            diagnostics: vec![SorlaDiagnostic {
                severity: DiagnosticSeverity::Error,
                code: "sorla.gtpack.doctor".to_string(),
                message: format!("gtpack doctor returned status `{}`", report.status),
                path: Some(report.path),
                suggestion: None,
            }],
        },
        Err(message) => SorlaValidationReport {
            diagnostics: vec![SorlaDiagnostic {
                severity: DiagnosticSeverity::Error,
                code: "sorla.gtpack.doctor".to_string(),
                message,
                path: None,
                suggestion: None,
            }],
        },
    }
}

fn deterministic_temp_path(filename: &str) -> PathBuf {
    let safe_name = filename.replace(['/', '\\'], "_");
    std::env::temp_dir().join(format!(
        "greentic-sorla-lib-{}-{safe_name}",
        std::process::id()
    ))
}

fn sha256_hex_public(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn run<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<OsString>>();
    if let Some(help) = localized_help_for_args(&args) {
        println!("{help}");
        return Ok(());
    }

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

fn localized_help_for_args(args: &[OsString]) -> Option<String> {
    if !args.iter().any(|arg| {
        let arg = arg.to_string_lossy();
        arg == "--help" || arg == "-h"
    }) {
        return None;
    }

    let locale = explicit_locale_arg(args).or_else(|| std::env::var("SORLA_LOCALE").ok())?;
    let locale = locale.trim();
    if locale.is_empty() {
        return None;
    }

    let localized = locale_catalog(locale)?;
    let fallback = locale_catalog("en").unwrap_or_default();

    match help_command(args) {
        HelpCommand::Wizard => Some(render_wizard_help(&localized, &fallback)),
        HelpCommand::Pack => Some(render_pack_help(&localized, &fallback)),
        HelpCommand::Root => Some(render_root_help(&localized, &fallback)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HelpCommand {
    Root,
    Wizard,
    Pack,
}

fn help_command(args: &[OsString]) -> HelpCommand {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        let arg = arg.to_string_lossy();
        match arg.as_ref() {
            "wizard" => return HelpCommand::Wizard,
            "pack" => return HelpCommand::Pack,
            "--help" | "-h" => return HelpCommand::Root,
            "--locale" => {
                let _ = iter.next();
            }
            value if value.starts_with("--locale=") => {}
            value if value.starts_with('-') => {}
            _ => return HelpCommand::Root,
        }
    }
    HelpCommand::Root
}

fn explicit_locale_arg(args: &[OsString]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let arg = arg.to_string_lossy();
        if arg == "--locale" {
            return iter.next().map(|value| value.to_string_lossy().to_string());
        }
        if let Some(value) = arg.strip_prefix("--locale=") {
            return Some(value.to_string());
        }
    }
    None
}

fn locale_catalog(locale: &str) -> Option<BTreeMap<String, String>> {
    serde_json::from_str(included_locale_json(locale)?).ok()
}

fn catalog_text<'a>(
    catalog: &'a BTreeMap<String, String>,
    fallback: &'a BTreeMap<String, String>,
    key: &str,
    default: &'static str,
) -> &'a str {
    catalog
        .get(key)
        .or_else(|| fallback.get(key))
        .map(String::as_str)
        .unwrap_or(default)
}

fn render_root_help(
    catalog: &BTreeMap<String, String>,
    fallback: &BTreeMap<String, String>,
) -> String {
    format!(
        "{title}\n\n{description}\n\n{usage}: greentic-sorla <COMMAND>\n\n{commands}:\n  wizard  {wizard_description}\n  pack    {pack_description}\n  help    {help_description}\n\n{options}:\n  -h, --help  {help_option}",
        title = catalog_text(catalog, fallback, "wizard.title", "SoRLa wizard"),
        description = catalog_text(
            catalog,
            fallback,
            "wizard.description",
            "Generate schema or apply answers documents."
        ),
        wizard_description = catalog_text(
            catalog,
            fallback,
            "wizard.description",
            "Generate schema or apply answers documents."
        ),
        pack_description = catalog_text(
            catalog,
            fallback,
            "cli.commands.pack.description",
            "Build, inspect, or doctor deterministic SoRLa gtpack handoff artifacts."
        ),
        help_description = catalog_text(
            catalog,
            fallback,
            "cli.commands.help.description",
            "Print this message or the help of the given subcommand(s)."
        ),
        usage = catalog_text(catalog, fallback, "cli.usage", "Usage"),
        commands = catalog_text(catalog, fallback, "cli.commands", "Commands"),
        options = catalog_text(catalog, fallback, "cli.options", "Options"),
        help_option = catalog_text(
            catalog,
            fallback,
            "cli.options.help.description",
            "Print help"
        )
    )
}

fn render_wizard_help(
    catalog: &BTreeMap<String, String>,
    fallback: &BTreeMap<String, String>,
) -> String {
    format!(
        "{description}\n\n{usage}: greentic-sorla wizard [{options_placeholder}]\n\n{options}:\n      --schema           {schema_description}\n      --locale <LOCALE>  {locale_description}\n      --answers <FILE>   {answers_description}\n      --pack-out <FILE>  {pack_out_description}\n  -h, --help             {help_option}\n\n{core_prompts}:\n  {flow_label}\n  {output_dir_label}\n  {package_name_label}\n  {package_version_label}\n  {storage_provider_label}\n  {default_source_label}\n  {events_enabled_label}\n  {projection_mode_label}\n  {compatibility_mode_label}\n  {include_agent_tools_label}",
        description = catalog_text(
            catalog,
            fallback,
            "wizard.description",
            "Generate schema or apply answers documents."
        ),
        usage = catalog_text(catalog, fallback, "cli.usage", "Usage"),
        options_placeholder = catalog_text(catalog, fallback, "cli.options.placeholder", "OPTIONS"),
        options = catalog_text(catalog, fallback, "cli.options", "Options"),
        schema_description = catalog_text(
            catalog,
            fallback,
            "cli.wizard.options.schema.description",
            "Emit the wizard schema as deterministic JSON"
        ),
        locale_description = catalog_text(
            catalog,
            fallback,
            "cli.wizard.options.locale.description",
            "Locale used for wizard schema metadata and interactive prompts"
        ),
        answers_description = catalog_text(
            catalog,
            fallback,
            "cli.wizard.options.answers.description",
            "Apply a saved answers document"
        ),
        pack_out_description = catalog_text(
            catalog,
            fallback,
            "cli.wizard.options.pack_out.description",
            "Also build a deterministic .gtpack from the generated sorla.yaml"
        ),
        help_option = catalog_text(
            catalog,
            fallback,
            "cli.options.help.description",
            "Print help"
        ),
        core_prompts = catalog_text(catalog, fallback, "cli.wizard.core_prompts", "Core prompts"),
        flow_label = catalog_text(catalog, fallback, "wizard.flow.label", "Wizard flow"),
        output_dir_label = catalog_text(
            catalog,
            fallback,
            "wizard.output_dir.label",
            "Output directory"
        ),
        package_name_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.package_name.label",
            "Package name"
        ),
        package_version_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.package_version.label",
            "Package version"
        ),
        storage_provider_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.storage_provider.label",
            "Storage provider category"
        ),
        default_source_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.default_source.label",
            "Default record source"
        ),
        events_enabled_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.events_enabled.label",
            "Enable event declarations"
        ),
        projection_mode_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.projection_mode.label",
            "Projection mode"
        ),
        compatibility_mode_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.compatibility_mode.label",
            "Compatibility mode"
        ),
        include_agent_tools_label = catalog_text(
            catalog,
            fallback,
            "wizard.questions.include_agent_tools.label",
            "Include agent tools output"
        )
    )
}

fn render_pack_help(
    catalog: &BTreeMap<String, String>,
    fallback: &BTreeMap<String, String>,
) -> String {
    format!(
        "{description}\n\n{usage}: greentic-sorla pack [{options_placeholder}] [FILE] [COMMAND]\n\n{commands}:\n  doctor               {doctor_description}\n  inspect              {inspect_description}\n  schema               {schema_command_description}\n  validation-inspect   {validation_inspect_description}\n  validation-doctor    {validation_doctor_description}\n  help                 {help_description}\n\n{options}:\n      --name <NAME>     {name_description}\n      --version <VER>   {version_description}\n      --out <FILE>      {out_description}\n  -h, --help            {help_option}",
        description = catalog_text(
            catalog,
            fallback,
            "cli.commands.pack.description",
            "Build, inspect, or doctor deterministic SoRLa gtpack handoff artifacts."
        ),
        usage = catalog_text(catalog, fallback, "cli.usage", "Usage"),
        options_placeholder = catalog_text(catalog, fallback, "cli.options.placeholder", "OPTIONS"),
        commands = catalog_text(catalog, fallback, "cli.commands", "Commands"),
        options = catalog_text(catalog, fallback, "cli.options", "Options"),
        doctor_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.commands.doctor.description",
            "Validate a generated SoRLa gtpack."
        ),
        inspect_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.commands.inspect.description",
            "Inspect a generated SoRLa gtpack as deterministic JSON."
        ),
        schema_command_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.commands.schema.description",
            "Emit deterministic JSON schemas for SORX handoff metadata."
        ),
        validation_inspect_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.commands.validation_inspect.description",
            "Inspect embedded SORX validation metadata as deterministic JSON."
        ),
        validation_doctor_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.commands.validation_doctor.description",
            "Validate embedded SORX validation metadata using pack doctor checks."
        ),
        help_description = catalog_text(
            catalog,
            fallback,
            "cli.commands.help.description",
            "Print this message or the help of the given subcommand(s)."
        ),
        name_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.options.name.description",
            "Pack name to write into the gtpack manifest."
        ),
        version_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.options.version.description",
            "Pack semantic version."
        ),
        out_description = catalog_text(
            catalog,
            fallback,
            "cli.pack.options.out.description",
            "Output .gtpack path."
        ),
        help_option = catalog_text(
            catalog,
            fallback,
            "cli.options.help.description",
            "Print help"
        )
    )
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
        Some(PackCommand::Schema(schema_args)) => {
            let schema = match schema_args.command {
                PackSchemaCommand::Validation => sorx_validation_schema_json(),
                PackSchemaCommand::ExposurePolicy => sorx_exposure_policy_schema_json(),
                PackSchemaCommand::Compatibility => sorx_compatibility_schema_json(),
                PackSchemaCommand::Ontology => ontology_schema_json(),
                PackSchemaCommand::RetrievalBindings => retrieval_bindings_schema_json(),
            };
            let rendered = serde_json::to_string_pretty(&schema).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        Some(PackCommand::ValidationInspect(path_args)) => {
            let inspection = inspect_sorla_gtpack(&path_args.path)?;
            let rendered = serde_json::to_string_pretty(&validation_inspection_json(&inspection))
                .map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        Some(PackCommand::ValidationDoctor(path_args)) => {
            let report = doctor_sorla_gtpack(&path_args.path)?;
            let rendered = serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?;
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

fn validation_inspection_json(inspection: &SorlaGtpackInspection) -> serde_json::Value {
    serde_json::json!({
        "schema": inspection
            .validation
            .as_ref()
            .map(|validation| validation.schema.clone())
            .unwrap_or_else(|| SORX_VALIDATION_SCHEMA.to_string()),
        "package": {
            "name": inspection.sorla_package_name,
            "version": inspection.sorla_package_version,
            "ir_hash": inspection.ir_hash
        },
        "validation": inspection.validation,
        "exposure": inspection.exposure_policy,
        "compatibility": inspection.compatibility,
        "ontology": inspection.ontology,
        "retrieval_bindings": inspection.retrieval_bindings
    })
}

fn sorx_exposure_policy_schema_json() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SORX_EXPOSURE_POLICY_SCHEMA,
        "title": "SORX exposure policy",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "schema",
            "default_visibility",
            "promotion_requires",
            "allowed_route_prefixes",
            "forbidden_route_prefixes",
            "endpoints"
        ],
        "properties": {
            "schema": { "const": SORX_EXPOSURE_POLICY_SCHEMA },
            "default_visibility": {
                "type": "string",
                "enum": ["private", "internal", "public_candidate"]
            },
            "promotion_requires": {
                "type": "array",
                "items": { "type": "string", "minLength": 1 }
            },
            "allowed_route_prefixes": {
                "type": "array",
                "items": { "type": "string" }
            },
            "forbidden_route_prefixes": {
                "type": "array",
                "items": { "type": "string" }
            },
            "endpoints": {
                "type": "array",
                "items": { "$ref": "#/$defs/endpoint" }
            }
        },
        "$defs": {
            "endpoint": {
                "type": "object",
                "additionalProperties": false,
                "required": [
                    "endpoint_id",
                    "visibility",
                    "requires_approval",
                    "export_surfaces",
                    "route_prefixes"
                ],
                "properties": {
                    "endpoint_id": { "type": "string", "minLength": 1 },
                    "visibility": {
                        "type": "string",
                        "enum": ["private", "internal", "public_candidate"]
                    },
                    "requires_approval": { "type": "boolean" },
                    "risk": { "type": "string" },
                    "export_surfaces": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["openapi", "arazzo", "mcp", "llms_txt"]
                        }
                    },
                    "route_prefixes": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                }
            }
        }
    })
}

fn sorx_compatibility_schema_json() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SORX_COMPATIBILITY_SCHEMA,
        "title": "SORX compatibility manifest",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "schema",
            "package",
            "api_compatibility",
            "state_compatibility",
            "provider_compatibility",
            "migration_compatibility"
        ],
        "properties": {
            "schema": { "const": SORX_COMPATIBILITY_SCHEMA },
            "package": {
                "type": "object",
                "additionalProperties": false,
                "required": ["name", "version"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "version": { "type": "string", "minLength": 1 },
                    "ir_hash": { "type": "string", "minLength": 1 }
                }
            },
            "api_compatibility": {
                "type": "string",
                "enum": ["additive", "backward_compatible", "breaking", "unknown"]
            },
            "state_compatibility": {
                "type": "string",
                "enum": ["isolated_required", "shared_allowed", "shared_requires_migration", "unknown"]
            },
            "provider_compatibility": {
                "type": "array",
                "items": { "$ref": "#/$defs/provider" }
            },
            "migration_compatibility": {
                "type": "array",
                "items": { "$ref": "#/$defs/migration" }
            }
        },
        "$defs": {
            "provider": {
                "type": "object",
                "additionalProperties": false,
                "required": [
                    "category",
                    "required_capabilities",
                    "contract_version_range",
                    "required"
                ],
                "properties": {
                    "category": { "type": "string", "minLength": 1 },
                    "required_capabilities": {
                        "type": "array",
                        "items": { "type": "string", "minLength": 1 }
                    },
                    "contract_version_range": { "type": "string", "minLength": 1 },
                    "required": { "type": "boolean" }
                }
            },
            "migration": {
                "type": "object",
                "additionalProperties": false,
                "required": ["name", "mode", "projection_updates", "backfill_count"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "mode": {
                        "type": "string",
                        "enum": ["additive", "backward_compatible", "breaking", "unknown"]
                    },
                    "projection_updates": {
                        "type": "array",
                        "items": { "type": "string", "minLength": 1 }
                    },
                    "backfill_count": { "type": "integer", "minimum": 0 },
                    "idempotence_key": { "type": "string", "minLength": 1 }
                }
            }
        }
    })
}

fn run_wizard(args: WizardArgs) -> Result<(), String> {
    let pack_out = args.pack_out;
    match (args.schema, args.answers) {
        (true, None) => {
            if pack_out.is_some() {
                return Err("`--pack-out` can only be used when applying answers or running the interactive wizard".to_string());
            }
            let schema = default_schema_for_locale(&selected_locale(args.locale.as_deref(), None));
            let rendered = serde_json::to_string_pretty(&schema).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        (false, Some(path)) => {
            let contents = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read answers file {}: {err}", path.display()))?;
            let mut answers: AnswersDocument = serde_json::from_str(&contents)
                .map_err(|err| format!("failed to parse answers file {}: {err}", path.display()))?;
            if args.locale.is_some() {
                answers.locale = args.locale;
            }
            let summary = apply_answers(answers, pack_out)?;
            let rendered = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
            println!("{rendered}");
            Ok(())
        }
        (true, Some(_)) => {
            Err("choose one wizard mode: use either `--schema` or `--answers <file>`".to_string())
        }
        (false, None) => run_interactive_wizard(args.locale.as_deref(), pack_out),
    }
}

#[cfg(feature = "cli")]
fn run_interactive_wizard(
    requested_locale: Option<&str>,
    pack_out: Option<PathBuf>,
) -> Result<(), String> {
    let locale = selected_locale(requested_locale, None);
    let mut provider = |question_id: &str, question: &serde_json::Value| {
        prompt_interactive_answer(question_id, question)
    };
    let summary = run_interactive_wizard_with_provider(&locale, &mut provider, pack_out)?;
    let rendered = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
    println!("{rendered}");
    Ok(())
}

#[cfg(not(feature = "cli"))]
fn run_interactive_wizard(
    _requested_locale: Option<&str>,
    _pack_out: Option<PathBuf>,
) -> Result<(), String> {
    Err("interactive wizard requires the `cli` feature".to_string())
}

#[cfg(feature = "cli")]
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

    let manifest_json = serde_json::to_vec_pretty(&build_launcher_handoff_manifest(&resolved))
        .map_err(|err| err.to_string())?;
    let manifest_paths = write_generated_json_aliases(
        &generated_dir,
        &[LAUNCHER_HANDOFF_FILENAME, LEGACY_PACKAGE_MANIFEST_FILENAME],
        &manifest_json,
    )?;
    written_files.extend(
        manifest_paths
            .iter()
            .map(|path| relative_to_output(&output_dir, path)),
    );

    let provider_requirements_path = generated_dir.join("provider-requirements.json");
    let provider_requirements_json =
        serde_json::to_vec_pretty(&build_provider_handoff_manifest(&resolved))
            .map_err(|err| err.to_string())?;
    fs::write(&provider_requirements_path, provider_requirements_json).map_err(|err| {
        format!(
            "failed to write generated file {}: {err}",
            provider_requirements_path.display()
        )
    })?;
    written_files.push(relative_to_output(&output_dir, &provider_requirements_path));

    let locale_manifest_path = generated_dir.join("locale-manifest.json");
    let locale_manifest_json = serde_json::to_vec_pretty(&build_locale_handoff_manifest(&resolved))
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
        let validation_manifest_path =
            write_generated_sorx_validation_manifest(&generated_dir, &generated_yaml)?;
        written_files.push(relative_to_output(&output_dir, &validation_manifest_path));

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

fn write_generated_sorx_validation_manifest(
    generated_dir: &Path,
    generated_yaml: &str,
) -> Result<PathBuf, String> {
    let artifacts = build_handoff_artifacts_from_yaml(generated_yaml)?;
    let manifest = generate_sorx_validation_manifest_from_ir(
        &artifacts.ir,
        Some(&artifacts.canonical_hash),
        vec![
            START_SCHEMA_FILENAME.to_string(),
            RUNTIME_TEMPLATE_FILENAME.to_string(),
            PROVIDER_BINDINGS_TEMPLATE_FILENAME.to_string(),
        ],
    );
    manifest.validate_static().map_err(|err| err.to_string())?;

    let path = generated_dir
        .join("assets")
        .join("sorx")
        .join("tests")
        .join("test-manifest.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create directory {}: {err}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|err| err.to_string())?;
    fs::write(&path, bytes)
        .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))?;
    Ok(path)
}

fn validate_answers_document(
    answers: &AnswersDocument,
    schema: &WizardSchema,
) -> Result<(), String> {
    if answers.schema_version != schema.schema_version && answers.schema_version != "0.4" {
        return Err(format!(
            "schema_version mismatch: expected {} or 0.4, got {}",
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
        for (index, endpoint) in agent_endpoints.items.iter().enumerate() {
            validate_choice(
                endpoint.risk.as_deref(),
                &["low", "medium", "high"],
                &format!("agent_endpoints.items[{index}].risk"),
            )?;
            validate_choice(
                endpoint.approval.as_deref(),
                &["none", "optional", "required", "policy-driven"],
                &format!("agent_endpoints.items[{index}].approval"),
            )?;
        }

        let enabled = agent_endpoints
            .enabled
            .unwrap_or(!agent_endpoints.items.is_empty());
        if enabled {
            let ids = normalize_text_list(agent_endpoints.ids.clone().unwrap_or_default());
            if ids.is_empty() && agent_endpoints.items.is_empty() {
                return Err(
                    "field `agent_endpoints.ids` or `agent_endpoints.items` requires at least one endpoint when agent endpoints are enabled"
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

    validate_rich_answers(answers)?;

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

fn validate_rich_answers(answers: &AnswersDocument) -> Result<(), String> {
    let mut record_fields = BTreeMap::<String, BTreeSet<String>>::new();
    if let Some(records) = &answers.records {
        let mut record_names = BTreeSet::new();
        for (record_index, record) in records.items.iter().enumerate() {
            require_non_empty(&record.name, &format!("records.items[{record_index}].name"))?;
            validate_choice(
                record.source.as_deref(),
                &["native", "external", "hybrid"],
                &format!("records.items[{record_index}].source"),
            )?;
            if !record_names.insert(record.name.clone()) {
                return Err(format!(
                    "records.items[{record_index}].name duplicates record `{}`",
                    record.name
                ));
            }

            let mut field_names = BTreeSet::new();
            for (field_index, field) in record.fields.iter().enumerate() {
                require_non_empty(
                    &field.name,
                    &format!("records.items[{record_index}].fields[{field_index}].name"),
                )?;
                require_non_empty(
                    &field.type_name,
                    &format!("records.items[{record_index}].fields[{field_index}].type"),
                )?;
                validate_choice(
                    field.authority.as_deref(),
                    &["local", "external"],
                    &format!("records.items[{record_index}].fields[{field_index}].authority"),
                )?;
                validate_enum_values(
                    &field.enum_values,
                    &format!("records.items[{record_index}].fields[{field_index}].enum_values"),
                )?;
                if !field_names.insert(field.name.clone()) {
                    return Err(format!(
                        "records.items[{record_index}].fields[{field_index}].name duplicates field `{}`",
                        field.name
                    ));
                }
            }
            record_fields.insert(record.name.clone(), field_names);
        }

        for (record_index, record) in records.items.iter().enumerate() {
            for (field_index, field) in record.fields.iter().enumerate() {
                if let Some(reference) = &field.references {
                    let Some(target_fields) = record_fields.get(&reference.record) else {
                        return Err(format!(
                            "records.items[{record_index}].fields[{field_index}].references.record points to unknown record `{}`",
                            reference.record
                        ));
                    };
                    if !target_fields.contains(&reference.field) {
                        return Err(format!(
                            "records.items[{record_index}].fields[{field_index}].references.field points to unknown field `{}` on record `{}`",
                            reference.field, reference.record
                        ));
                    }
                }
            }
        }
    }

    validate_ontology_answers(answers.ontology.as_ref(), &record_fields)?;
    validate_semantic_alias_answers(answers.semantic_aliases.as_ref(), answers.ontology.as_ref())?;
    validate_entity_linking_answers(
        answers.entity_linking.as_ref(),
        answers.ontology.as_ref(),
        &record_fields,
    )?;
    validate_retrieval_binding_answers(
        answers.retrieval_bindings.as_ref(),
        answers.ontology.as_ref(),
    )?;

    let action_names = validate_named_answers(&answers.actions, "actions")?;
    let policy_names = validate_named_answers(&answers.policies, "policies")?;
    let approval_names = validate_named_answers(&answers.approvals, "approvals")?;

    let mut event_names = BTreeSet::new();
    if let Some(events) = &answers.events {
        for (event_index, event) in events.items.iter().enumerate() {
            require_non_empty(&event.name, &format!("events.items[{event_index}].name"))?;
            require_non_empty(
                &event.record,
                &format!("events.items[{event_index}].record"),
            )?;
            validate_choice(
                event.kind.as_deref(),
                &["domain", "integration"],
                &format!("events.items[{event_index}].kind"),
            )?;
            if !record_fields.is_empty() && !record_fields.contains_key(&event.record) {
                return Err(format!(
                    "events.items[{event_index}].record points to unknown record `{}`",
                    event.record
                ));
            }
            if !event_names.insert(event.name.clone()) {
                return Err(format!(
                    "events.items[{event_index}].name duplicates event `{}`",
                    event.name
                ));
            }
            for (field_index, field) in event.emits.iter().enumerate() {
                require_non_empty(
                    &field.name,
                    &format!("events.items[{event_index}].emits[{field_index}].name"),
                )?;
                require_non_empty(
                    &field.type_name,
                    &format!("events.items[{event_index}].emits[{field_index}].type"),
                )?;
            }
        }
    }

    if let Some(projections) = &answers.projections {
        let mut projection_names = BTreeSet::new();
        for (projection_index, projection) in projections.items.iter().enumerate() {
            require_non_empty(
                &projection.name,
                &format!("projections.items[{projection_index}].name"),
            )?;
            validate_choice(
                projection.mode.as_deref(),
                &["current-state", "audit-trail"],
                &format!("projections.items[{projection_index}].mode"),
            )?;
            if !record_fields.is_empty() && !record_fields.contains_key(&projection.record) {
                return Err(format!(
                    "projections.items[{projection_index}].record points to unknown record `{}`",
                    projection.record
                ));
            }
            if !event_names.is_empty() && !event_names.contains(&projection.source_event) {
                return Err(format!(
                    "projections.items[{projection_index}].source_event points to unknown event `{}`",
                    projection.source_event
                ));
            }
            if !projection_names.insert(projection.name.clone()) {
                return Err(format!(
                    "projections.items[{projection_index}].name duplicates projection `{}`",
                    projection.name
                ));
            }
        }
    }

    for (requirement_index, requirement) in answers.provider_requirements.iter().enumerate() {
        validate_provider_requirement_answer(
            requirement,
            &format!("provider_requirements[{requirement_index}]"),
        )?;
    }

    if let Some(migrations) = &answers.migrations {
        let mut migration_names = BTreeSet::new();
        for (migration_index, migration) in migrations.items.iter().enumerate() {
            require_non_empty(
                &migration.name,
                &format!("migrations.items[{migration_index}].name"),
            )?;
            validate_choice(
                migration.compatibility.as_deref(),
                &["additive", "backward-compatible", "breaking"],
                &format!("migrations.items[{migration_index}].compatibility"),
            )?;
            if !migration_names.insert(migration.name.clone()) {
                return Err(format!(
                    "migrations.items[{migration_index}].name duplicates migration `{}`",
                    migration.name
                ));
            }
        }
    }

    if let Some(agent_endpoints) = &answers.agent_endpoints {
        let mut endpoint_ids = BTreeSet::new();
        for (endpoint_index, endpoint) in agent_endpoints.items.iter().enumerate() {
            require_non_empty(
                &endpoint.id,
                &format!("agent_endpoints.items[{endpoint_index}].id"),
            )?;
            require_non_empty(
                &endpoint.title,
                &format!("agent_endpoints.items[{endpoint_index}].title"),
            )?;
            require_non_empty(
                &endpoint.intent,
                &format!("agent_endpoints.items[{endpoint_index}].intent"),
            )?;
            if !endpoint_ids.insert(endpoint.id.clone()) {
                return Err(format!(
                    "agent_endpoints.items[{endpoint_index}].id duplicates endpoint `{}`",
                    endpoint.id
                ));
            }
            validate_endpoint_fields(
                &endpoint.inputs,
                &format!("agent_endpoints.items[{endpoint_index}].inputs"),
            )?;
            validate_endpoint_fields(
                &endpoint.outputs,
                &format!("agent_endpoints.items[{endpoint_index}].outputs"),
            )?;
            for (requirement_index, requirement) in
                endpoint.provider_requirements.iter().enumerate()
            {
                validate_provider_requirement_answer(
                    requirement,
                    &format!(
                        "agent_endpoints.items[{endpoint_index}].provider_requirements[{requirement_index}]"
                    ),
                )?;
            }
            if let Some(emits) = &endpoint.emits {
                require_non_empty(
                    &emits.event,
                    &format!("agent_endpoints.items[{endpoint_index}].emits.event"),
                )?;
                require_non_empty(
                    &emits.stream,
                    &format!("agent_endpoints.items[{endpoint_index}].emits.stream"),
                )?;
                if !event_names.is_empty() && !event_names.contains(&emits.event) {
                    return Err(format!(
                        "agent_endpoints.items[{endpoint_index}].emits.event points to unknown event `{}`",
                        emits.event
                    ));
                }
            }
            validate_declared_references(
                &endpoint.backing.actions,
                &action_names,
                &format!("agent_endpoints.items[{endpoint_index}].backing.actions"),
            )?;
            validate_declared_references(
                &endpoint.backing.events,
                &event_names,
                &format!("agent_endpoints.items[{endpoint_index}].backing.events"),
            )?;
            validate_declared_references(
                &endpoint.backing.policies,
                &policy_names,
                &format!("agent_endpoints.items[{endpoint_index}].backing.policies"),
            )?;
            validate_declared_references(
                &endpoint.backing.approvals,
                &approval_names,
                &format!("agent_endpoints.items[{endpoint_index}].backing.approvals"),
            )?;
        }
    }

    Ok(())
}

fn validate_ontology_answers(
    ontology: Option<&OntologyAnswers>,
    record_fields: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), String> {
    let Some(ontology) = ontology else {
        return Ok(());
    };

    let schema = ontology
        .schema
        .as_deref()
        .unwrap_or("greentic.sorla.ontology.v1");
    if schema != "greentic.sorla.ontology.v1" {
        return Err(format!(
            "ontology.schema must be `greentic.sorla.ontology.v1`, got `{schema}`"
        ));
    }

    let mut concept_ids = BTreeSet::new();
    for (concept_index, concept) in ontology.concepts.iter().enumerate() {
        let path = format!("ontology.concepts[{concept_index}]");
        validate_url_safe_id(&concept.id, &format!("{path}.id"))?;
        validate_choice(
            Some(concept.kind.as_str()),
            &["abstract", "entity"],
            &format!("{path}.kind"),
        )?;
        if !concept_ids.insert(concept.id.clone()) {
            return Err(format!("{path}.id duplicates concept `{}`", concept.id));
        }
        for (extends_index, parent) in concept.extends.iter().enumerate() {
            validate_url_safe_id(parent, &format!("{path}.extends[{extends_index}]"))?;
        }
        if let Some(backing) = &concept.backed_by {
            validate_ontology_backing_answer(backing, record_fields, &format!("{path}.backed_by"))?;
        }
        for (hook_index, hook) in concept.policy_hooks.iter().enumerate() {
            require_non_empty(
                &hook.policy,
                &format!("{path}.policy_hooks[{hook_index}].policy"),
            )?;
        }
        for (requirement_index, requirement) in concept.provider_requirements.iter().enumerate() {
            validate_provider_requirement_answer(
                requirement,
                &format!("{path}.provider_requirements[{requirement_index}]"),
            )?;
        }
    }

    for (concept_index, concept) in ontology.concepts.iter().enumerate() {
        let path = format!("ontology.concepts[{concept_index}]");
        for (extends_index, parent) in concept.extends.iter().enumerate() {
            if !concept_ids.contains(parent) {
                return Err(format!(
                    "{path}.extends[{extends_index}] points to unknown concept `{parent}`"
                ));
            }
        }
    }
    validate_ontology_answer_cycles(ontology)?;

    let mut relationship_ids = BTreeSet::new();
    for (relationship_index, relationship) in ontology.relationships.iter().enumerate() {
        let path = format!("ontology.relationships[{relationship_index}]");
        validate_url_safe_id(&relationship.id, &format!("{path}.id"))?;
        if !relationship_ids.insert(relationship.id.clone()) {
            return Err(format!(
                "{path}.id duplicates relationship `{}`",
                relationship.id
            ));
        }
        if !concept_ids.contains(&relationship.from) {
            return Err(format!(
                "{path}.from points to unknown concept `{}`",
                relationship.from
            ));
        }
        if !concept_ids.contains(&relationship.to) {
            return Err(format!(
                "{path}.to points to unknown concept `{}`",
                relationship.to
            ));
        }
        if let Some(cardinality) = &relationship.cardinality {
            validate_choice(
                Some(cardinality.from.as_str()),
                &["one", "many"],
                &format!("{path}.cardinality.from"),
            )?;
            validate_choice(
                Some(cardinality.to.as_str()),
                &["one", "many"],
                &format!("{path}.cardinality.to"),
            )?;
        }
        if let Some(backing) = &relationship.backed_by {
            validate_ontology_backing_answer(backing, record_fields, &format!("{path}.backed_by"))?;
        }
        for (hook_index, hook) in relationship.policy_hooks.iter().enumerate() {
            require_non_empty(
                &hook.policy,
                &format!("{path}.policy_hooks[{hook_index}].policy"),
            )?;
        }
        for (requirement_index, requirement) in
            relationship.provider_requirements.iter().enumerate()
        {
            validate_provider_requirement_answer(
                requirement,
                &format!("{path}.provider_requirements[{requirement_index}]"),
            )?;
        }
    }

    let mut constraint_ids = BTreeSet::new();
    for (constraint_index, constraint) in ontology.constraints.iter().enumerate() {
        let path = format!("ontology.constraints[{constraint_index}]");
        validate_url_safe_id(&constraint.id, &format!("{path}.id"))?;
        if !constraint_ids.insert(constraint.id.clone()) {
            return Err(format!(
                "{path}.id duplicates constraint `{}`",
                constraint.id
            ));
        }
        if !concept_ids.contains(&constraint.applies_to.concept) {
            return Err(format!(
                "{path}.applies_to.concept points to unknown concept `{}`",
                constraint.applies_to.concept
            ));
        }
        if let Some(policy) = &constraint.requires_policy {
            require_non_empty(policy, &format!("{path}.requires_policy"))?;
        }
    }

    Ok(())
}

fn validate_ontology_backing_answer(
    backing: &OntologyBackingAnswer,
    record_fields: &BTreeMap<String, BTreeSet<String>>,
    path: &str,
) -> Result<(), String> {
    require_non_empty(&backing.record, &format!("{path}.record"))?;
    let Some(fields) = record_fields.get(&backing.record) else {
        return Err(format!(
            "{path}.record points to unknown record `{}`",
            backing.record
        ));
    };
    if let Some(from_field) = &backing.from_field {
        require_non_empty(from_field, &format!("{path}.from_field"))?;
        if !fields.contains(from_field) {
            return Err(format!(
                "{path}.from_field points to unknown field `{from_field}` on record `{}`",
                backing.record
            ));
        }
    }
    if let Some(to_field) = &backing.to_field {
        require_non_empty(to_field, &format!("{path}.to_field"))?;
        if !fields.contains(to_field) {
            return Err(format!(
                "{path}.to_field points to unknown field `{to_field}` on record `{}`",
                backing.record
            ));
        }
    }
    Ok(())
}

fn validate_semantic_alias_answers(
    aliases: Option<&SemanticAliasesAnswer>,
    ontology: Option<&OntologyAnswers>,
) -> Result<(), String> {
    let Some(aliases) = aliases else {
        return Ok(());
    };
    let ontology =
        ontology.ok_or_else(|| "semantic_aliases require `ontology` answers".to_string())?;
    let concept_ids = ontology
        .concepts
        .iter()
        .map(|concept| concept.id.clone())
        .collect::<BTreeSet<_>>();
    let relationship_ids = ontology
        .relationships
        .iter()
        .map(|relationship| relationship.id.clone())
        .collect::<BTreeSet<_>>();
    validate_alias_answer_map(
        &aliases.concepts,
        &concept_ids,
        "semantic_aliases.concepts",
        "concept",
    )?;
    validate_alias_answer_map(
        &aliases.relationships,
        &relationship_ids,
        "semantic_aliases.relationships",
        "relationship",
    )
}

fn validate_alias_answer_map(
    aliases: &BTreeMap<String, Vec<String>>,
    known_targets: &BTreeSet<String>,
    path: &str,
    target_kind: &str,
) -> Result<(), String> {
    let mut normalized_targets = BTreeMap::new();
    for (target, values) in aliases {
        if !known_targets.contains(target) {
            return Err(format!(
                "{path}.{target} points to unknown {target_kind} `{target}`"
            ));
        }
        for (alias_index, alias) in values.iter().enumerate() {
            require_non_empty(alias, &format!("{path}.{target}[{alias_index}]"))?;
            let normalized = normalize_semantic_alias(alias);
            if let Some(existing) = normalized_targets.insert(normalized, target.clone())
                && existing != *target
            {
                return Err(format!(
                    "{path}.{target}[{alias_index}] collides with alias target `{existing}`"
                ));
            }
        }
    }
    Ok(())
}

fn validate_entity_linking_answers(
    entity_linking: Option<&EntityLinkingAnswer>,
    ontology: Option<&OntologyAnswers>,
    record_fields: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), String> {
    let Some(entity_linking) = entity_linking else {
        return Ok(());
    };
    let ontology =
        ontology.ok_or_else(|| "entity_linking requires `ontology` answers".to_string())?;
    let concepts = ontology
        .concepts
        .iter()
        .map(|concept| (concept.id.as_str(), concept))
        .collect::<BTreeMap<_, _>>();
    let mut strategy_ids = BTreeSet::new();
    for (strategy_index, strategy) in entity_linking.strategies.iter().enumerate() {
        let path = format!("entity_linking.strategies[{strategy_index}]");
        validate_url_safe_id(&strategy.id, &format!("{path}.id"))?;
        if !strategy_ids.insert(strategy.id.clone()) {
            return Err(format!("{path}.id duplicates strategy `{}`", strategy.id));
        }
        let confidence = strategy
            .confidence
            .as_f64()
            .ok_or_else(|| format!("{path}.confidence must be a number"))?;
        if !(0.0..=1.0).contains(&confidence) {
            return Err(format!("{path}.confidence must be between 0.0 and 1.0"));
        }
        require_non_empty(
            &strategy.match_fields.source_field,
            &format!("{path}.match.source_field"),
        )?;
        require_non_empty(
            &strategy.match_fields.target_field,
            &format!("{path}.match.target_field"),
        )?;
        let concept = concepts.get(strategy.applies_to.as_str()).ok_or_else(|| {
            format!(
                "{path}.applies_to points to unknown concept `{}`",
                strategy.applies_to
            )
        })?;
        if let Some(backing) = &concept.backed_by {
            let fields = record_fields.get(&backing.record).ok_or_else(|| {
                format!(
                    "{path}.applies_to concept `{}` points to unknown backing record `{}`",
                    concept.id, backing.record
                )
            })?;
            if !fields.contains(&strategy.match_fields.target_field) {
                return Err(format!(
                    "{path}.match.target_field points to unknown field `{}` on record `{}`",
                    strategy.match_fields.target_field, backing.record
                ));
            }
        } else {
            let Some(source_type) = &strategy.source_type else {
                return Err(format!(
                    "{path}.source_type is required for unbacked concept `{}`",
                    concept.id
                ));
            };
            require_non_empty(source_type, &format!("{path}.source_type"))?;
            if source_type == "record" {
                return Err(format!(
                    "{path}.source_type must be non-record for unbacked concept `{}`",
                    concept.id
                ));
            }
        }
    }
    Ok(())
}

fn normalize_semantic_alias(alias: &str) -> String {
    alias
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn validate_retrieval_binding_answers(
    retrieval_bindings: Option<&RetrievalBindingsAnswer>,
    ontology: Option<&OntologyAnswers>,
) -> Result<(), String> {
    let Some(retrieval_bindings) = retrieval_bindings else {
        return Ok(());
    };
    let ontology =
        ontology.ok_or_else(|| "retrieval_bindings require `ontology` answers".to_string())?;
    let schema = retrieval_bindings
        .schema
        .as_deref()
        .unwrap_or("greentic.sorla.retrieval-bindings.v1");
    if schema != "greentic.sorla.retrieval-bindings.v1" {
        return Err(format!(
            "retrieval_bindings.schema must be `greentic.sorla.retrieval-bindings.v1`, got `{schema}`"
        ));
    }

    let concept_ids = ontology
        .concepts
        .iter()
        .map(|concept| concept.id.clone())
        .collect::<BTreeSet<_>>();
    let relationship_ids = ontology
        .relationships
        .iter()
        .map(|relationship| relationship.id.clone())
        .collect::<BTreeSet<_>>();

    let mut provider_ids = BTreeSet::new();
    for (provider_index, provider) in retrieval_bindings.providers.iter().enumerate() {
        let path = format!("retrieval_bindings.providers[{provider_index}]");
        validate_url_safe_id(&provider.id, &format!("{path}.id"))?;
        if !provider_ids.insert(provider.id.clone()) {
            return Err(format!("{path}.id duplicates provider `{}`", provider.id));
        }
        require_non_empty(&provider.category, &format!("{path}.category"))?;
        for (capability_index, capability) in provider.required_capabilities.iter().enumerate() {
            require_non_empty(
                capability,
                &format!("{path}.required_capabilities[{capability_index}]"),
            )?;
        }
    }

    let mut scope_ids = BTreeSet::new();
    for (scope_index, scope) in retrieval_bindings.scopes.iter().enumerate() {
        let path = format!("retrieval_bindings.scopes[{scope_index}]");
        validate_url_safe_id(&scope.id, &format!("{path}.id"))?;
        if !scope_ids.insert(scope.id.clone()) {
            return Err(format!("{path}.id duplicates scope `{}`", scope.id));
        }
        if !provider_ids.contains(&scope.provider) {
            return Err(format!(
                "{path}.provider points to unknown provider `{}`",
                scope.provider
            ));
        }
        match (&scope.applies_to.concept, &scope.applies_to.relationship) {
            (Some(concept), None) if concept_ids.contains(concept) => {}
            (None, Some(relationship)) if relationship_ids.contains(relationship) => {}
            (Some(concept), None) => {
                return Err(format!(
                    "{path}.applies_to.concept points to unknown concept `{concept}`"
                ));
            }
            (None, Some(relationship)) => {
                return Err(format!(
                    "{path}.applies_to.relationship points to unknown relationship `{relationship}`"
                ));
            }
            (Some(_), Some(_)) | (None, None) => {
                return Err(format!(
                    "{path}.applies_to must declare exactly one of concept or relationship"
                ));
            }
        }
        validate_choice(
            scope.permission.as_deref(),
            &["inherit", "public-metadata-only", "requires-policy"],
            &format!("{path}.permission"),
        )?;
        if let Some(filters) = &scope.filters
            && let Some(entity_scope) = &filters.entity_scope
        {
            for (rule_index, rule) in entity_scope.include_related.iter().enumerate() {
                let rule_path =
                    format!("{path}.filters.entity_scope.include_related[{rule_index}]");
                if !relationship_ids.contains(&rule.relationship) {
                    return Err(format!(
                        "{rule_path}.relationship points to unknown relationship `{}`",
                        rule.relationship
                    ));
                }
                validate_choice(
                    Some(rule.direction.as_str()),
                    &["incoming", "outgoing", "both"],
                    &format!("{rule_path}.direction"),
                )?;
                if rule.max_depth > 5 {
                    return Err(format!("{rule_path}.max_depth must be between 0 and 5"));
                }
            }
        }
    }

    Ok(())
}

fn validate_ontology_answer_cycles(ontology: &OntologyAnswers) -> Result<(), String> {
    let parents = ontology
        .concepts
        .iter()
        .map(|concept| (concept.id.as_str(), concept.extends.as_slice()))
        .collect::<BTreeMap<_, _>>();
    for concept in parents.keys() {
        visit_ontology_answer_parent(
            concept,
            &parents,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        )?;
    }
    Ok(())
}

fn visit_ontology_answer_parent<'a>(
    concept: &'a str,
    parents: &BTreeMap<&'a str, &'a [String]>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<(), String> {
    if visited.contains(concept) {
        return Ok(());
    }
    if !visiting.insert(concept) {
        return Err(format!(
            "ontology.concepts contains inheritance cycle including `{concept}`"
        ));
    }
    if let Some(parent_ids) = parents.get(concept) {
        for parent in *parent_ids {
            visit_ontology_answer_parent(parent, parents, visiting, visited)?;
        }
    }
    visiting.remove(concept);
    visited.insert(concept);
    Ok(())
}

fn validate_url_safe_id(value: &str, path: &str) -> Result<(), String> {
    require_non_empty(value, path)?;
    if !value
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '_' | '-'))
    {
        return Err(format!("field `{path}` must be URL-safe"));
    }
    Ok(())
}

fn require_non_empty(value: &str, path: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("field `{path}` is required"));
    }
    Ok(())
}

fn validate_named_answers(items: &[NamedAnswer], path: &str) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        require_non_empty(&item.name, &format!("{path}[{index}].name"))?;
        if !names.insert(item.name.clone()) {
            return Err(format!("{path}[{index}].name duplicates `{}`", item.name));
        }
    }
    Ok(names)
}

fn validate_endpoint_fields(fields: &[FieldAnswer], path: &str) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for (index, field) in fields.iter().enumerate() {
        require_non_empty(&field.name, &format!("{path}[{index}].name"))?;
        require_non_empty(&field.type_name, &format!("{path}[{index}].type"))?;
        validate_enum_values(&field.enum_values, &format!("{path}[{index}].enum_values"))?;
        if !names.insert(field.name.clone()) {
            return Err(format!(
                "{path}[{index}].name duplicates field `{}`",
                field.name
            ));
        }
    }
    Ok(())
}

fn validate_enum_values(values: &[String], path: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        require_non_empty(value, &format!("{path}[{index}]"))?;
        if !seen.insert(value.clone()) {
            return Err(format!("{path}[{index}] duplicates enum value `{value}`"));
        }
    }
    Ok(())
}

fn validate_provider_requirement_answer(
    requirement: &ProviderRequirementAnswer,
    path: &str,
) -> Result<(), String> {
    require_non_empty(&requirement.category, &format!("{path}.category"))?;
    let mut capabilities = BTreeSet::new();
    for (index, capability) in requirement.capabilities.iter().enumerate() {
        require_non_empty(capability, &format!("{path}.capabilities[{index}]"))?;
        if !capabilities.insert(capability.clone()) {
            return Err(format!(
                "{path}.capabilities[{index}] duplicates capability `{capability}`"
            ));
        }
    }
    Ok(())
}

fn validate_declared_references(
    references: &[String],
    declared: &BTreeSet<String>,
    path: &str,
) -> Result<(), String> {
    if declared.is_empty() {
        return Ok(());
    }
    for (index, reference) in references.iter().enumerate() {
        if !declared.contains(reference) {
            return Err(format!(
                "{path}[{index}] points to undeclared item `{reference}`"
            ));
        }
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
        record_items: answers
            .records
            .as_ref()
            .map(|records| records.items.clone())
            .unwrap_or_default(),
        ontology: answers.ontology.clone(),
        semantic_aliases: answers.semantic_aliases.clone(),
        entity_linking: answers.entity_linking.clone(),
        retrieval_bindings: answers.retrieval_bindings.clone(),
        actions: answers.actions.clone(),
        event_items: answers
            .events
            .as_ref()
            .map(|events| events.items.clone())
            .unwrap_or_default(),
        projection_items: answers
            .projections
            .as_ref()
            .map(|projections| projections.items.clone())
            .unwrap_or_default(),
        provider_requirements: answers.provider_requirements.clone(),
        policies: answers.policies.clone(),
        approvals: answers.approvals.clone(),
        migration_items: answers
            .migrations
            .as_ref()
            .map(|migrations| migrations.items.clone())
            .unwrap_or_default(),
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
        agent_endpoint_items: agent_endpoint_values.items,
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
        schema_version: answers.schema_version.clone(),
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
        record_items: answers
            .records
            .as_ref()
            .and_then(|records| {
                if records.items.is_empty() {
                    None
                } else {
                    Some(records.items.clone())
                }
            })
            .unwrap_or(previous.record_items),
        ontology: answers.ontology.clone().or(previous.ontology),
        semantic_aliases: answers
            .semantic_aliases
            .clone()
            .or(previous.semantic_aliases),
        entity_linking: answers.entity_linking.clone().or(previous.entity_linking),
        retrieval_bindings: answers
            .retrieval_bindings
            .clone()
            .or(previous.retrieval_bindings),
        actions: if answers.actions.is_empty() {
            previous.actions
        } else {
            answers.actions.clone()
        },
        event_items: answers
            .events
            .as_ref()
            .and_then(|events| {
                if events.items.is_empty() {
                    None
                } else {
                    Some(events.items.clone())
                }
            })
            .unwrap_or(previous.event_items),
        projection_items: answers
            .projections
            .as_ref()
            .and_then(|projections| {
                if projections.items.is_empty() {
                    None
                } else {
                    Some(projections.items.clone())
                }
            })
            .unwrap_or(previous.projection_items),
        provider_requirements: if answers.provider_requirements.is_empty() {
            previous.provider_requirements
        } else {
            answers.provider_requirements.clone()
        },
        policies: if answers.policies.is_empty() {
            previous.policies
        } else {
            answers.policies.clone()
        },
        approvals: if answers.approvals.is_empty() {
            previous.approvals
        } else {
            answers.approvals.clone()
        },
        migration_items: answers
            .migrations
            .as_ref()
            .and_then(|migrations| {
                if migrations.items.is_empty() {
                    None
                } else {
                    Some(migrations.items.clone())
                }
            })
            .unwrap_or(previous.migration_items),
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
        agent_endpoint_items: agent_endpoint_values.items,
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
    items: Vec<AgentEndpointItemAnswer>,
}

fn resolve_agent_endpoint_answers(
    answers: Option<&AgentEndpointAnswers>,
) -> ResolvedAgentEndpointAnswers {
    let enabled = answers
        .and_then(|answers| answers.enabled)
        .unwrap_or_else(|| answers.is_some_and(|answers| !answers.items.is_empty()));
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
        items: if enabled {
            answers
                .map(|answers| answers.items.clone())
                .unwrap_or_default()
        } else {
            Vec::new()
        },
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
            items: previous.agent_endpoint_items.clone(),
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
        items: if enabled {
            if answers.items.is_empty() {
                previous.agent_endpoint_items.clone()
            } else {
                answers.items.clone()
            }
        } else {
            Vec::new()
        },
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

fn write_generated_json_aliases(
    generated_dir: &Path,
    file_names: &[&str],
    bytes: &[u8],
) -> Result<Vec<PathBuf>, String> {
    let mut written = Vec::new();
    for file_name in file_names {
        let path = generated_dir.join(file_name);
        fs::write(&path, bytes)
            .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))?;
        written.push(path);
    }
    Ok(written)
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
    if has_rich_domain_answers(resolved) {
        return render_rich_package_yaml(resolved);
    }

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

    render_ontology(resolved, &mut lines);

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

fn has_rich_domain_answers(resolved: &ResolvedAnswers) -> bool {
    !resolved.record_items.is_empty()
        || !resolved.actions.is_empty()
        || !resolved.event_items.is_empty()
        || !resolved.projection_items.is_empty()
        || !resolved.provider_requirements.is_empty()
        || !resolved.policies.is_empty()
        || !resolved.approvals.is_empty()
        || !resolved.migration_items.is_empty()
        || !resolved.agent_endpoint_items.is_empty()
        || resolved.ontology.is_some()
        || resolved.retrieval_bindings.is_some()
}

fn render_rich_package_yaml(resolved: &ResolvedAnswers) -> String {
    let mut lines = vec![
        "package:".to_string(),
        format!("  name: {}", yaml_scalar_string(&resolved.package_name)),
        format!(
            "  version: {}",
            yaml_scalar_string(&resolved.package_version)
        ),
    ];

    render_records(resolved, &mut lines);
    render_ontology(resolved, &mut lines);
    render_semantic_aliases(resolved, &mut lines);
    render_entity_linking(resolved, &mut lines);
    render_retrieval_bindings(resolved, &mut lines);
    render_actions(&resolved.actions, &mut lines);
    render_events(resolved, &mut lines);
    render_projections(resolved, &mut lines);
    render_provider_requirements(resolved, &mut lines);
    render_named_section("policies", &resolved.policies, &mut lines);
    render_named_section("approvals", &resolved.approvals, &mut lines);
    render_migrations(resolved, &mut lines);
    render_agent_endpoints(resolved, &mut lines);

    lines.join("\n") + "\n"
}

fn render_ontology(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    let Some(ontology) = &resolved.ontology else {
        return;
    };

    lines.push("ontology:".to_string());
    lines.push(format!(
        "  schema: {}",
        yaml_scalar_string(
            ontology
                .schema
                .as_deref()
                .unwrap_or("greentic.sorla.ontology.v1")
        )
    ));
    if ontology.concepts.is_empty() {
        lines.push("  concepts: []".to_string());
    } else {
        lines.push("  concepts:".to_string());
        for concept in &ontology.concepts {
            lines.push(format!("    - id: {}", yaml_scalar_string(&concept.id)));
            lines.push(format!("      kind: {}", yaml_scalar_string(&concept.kind)));
            if let Some(description) = &concept.description {
                lines.push(format!(
                    "      description: {}",
                    yaml_scalar_string(description)
                ));
            }
            if !concept.extends.is_empty() {
                render_string_list("      extends", &concept.extends, lines);
            }
            if let Some(backing) = &concept.backed_by {
                render_ontology_backing("      backed_by", backing, lines);
            }
            if let Some(sensitivity) = &concept.sensitivity {
                render_ontology_sensitivity("      sensitivity", sensitivity, lines);
            }
            render_ontology_policy_hooks("      policy_hooks", &concept.policy_hooks, lines);
            render_ontology_provider_requirements(
                "      provider_requirements",
                &concept.provider_requirements,
                lines,
            );
        }
    }

    if ontology.relationships.is_empty() {
        lines.push("  relationships: []".to_string());
    } else {
        lines.push("  relationships:".to_string());
        for relationship in &ontology.relationships {
            lines.push(format!(
                "    - id: {}",
                yaml_scalar_string(&relationship.id)
            ));
            if let Some(label) = &relationship.label {
                lines.push(format!("      label: {}", yaml_scalar_string(label)));
            }
            lines.push(format!(
                "      from: {}",
                yaml_scalar_string(&relationship.from)
            ));
            lines.push(format!(
                "      to: {}",
                yaml_scalar_string(&relationship.to)
            ));
            if let Some(cardinality) = &relationship.cardinality {
                lines.push("      cardinality:".to_string());
                lines.push(format!(
                    "        from: {}",
                    yaml_scalar_string(&cardinality.from)
                ));
                lines.push(format!(
                    "        to: {}",
                    yaml_scalar_string(&cardinality.to)
                ));
            }
            if let Some(backing) = &relationship.backed_by {
                render_ontology_backing("      backed_by", backing, lines);
            }
            if let Some(sensitivity) = &relationship.sensitivity {
                render_ontology_sensitivity("      sensitivity", sensitivity, lines);
            }
            render_ontology_policy_hooks("      policy_hooks", &relationship.policy_hooks, lines);
            render_ontology_provider_requirements(
                "      provider_requirements",
                &relationship.provider_requirements,
                lines,
            );
        }
    }

    if ontology.constraints.is_empty() {
        lines.push("  constraints: []".to_string());
    } else {
        lines.push("  constraints:".to_string());
        for constraint in &ontology.constraints {
            lines.push(format!("    - id: {}", yaml_scalar_string(&constraint.id)));
            lines.push("      applies_to:".to_string());
            lines.push(format!(
                "        concept: {}",
                yaml_scalar_string(&constraint.applies_to.concept)
            ));
            if let Some(policy) = &constraint.requires_policy {
                lines.push(format!(
                    "      requires_policy: {}",
                    yaml_scalar_string(policy)
                ));
            }
        }
    }
}

fn render_ontology_backing(
    section: &str,
    backing: &OntologyBackingAnswer,
    lines: &mut Vec<String>,
) {
    lines.push(format!("{section}:"));
    let nested = " ".repeat(section.len() - section.trim_start().len() + 2);
    lines.push(format!(
        "{nested}record: {}",
        yaml_scalar_string(&backing.record)
    ));
    if let Some(from_field) = &backing.from_field {
        lines.push(format!(
            "{nested}from_field: {}",
            yaml_scalar_string(from_field)
        ));
    }
    if let Some(to_field) = &backing.to_field {
        lines.push(format!(
            "{nested}to_field: {}",
            yaml_scalar_string(to_field)
        ));
    }
}

fn render_ontology_sensitivity(
    section: &str,
    sensitivity: &OntologySensitivityAnswer,
    lines: &mut Vec<String>,
) {
    lines.push(format!("{section}:"));
    let nested = " ".repeat(section.len() - section.trim_start().len() + 2);
    if let Some(classification) = &sensitivity.classification {
        lines.push(format!(
            "{nested}classification: {}",
            yaml_scalar_string(classification)
        ));
    }
    if let Some(pii) = sensitivity.pii {
        lines.push(format!("{nested}pii: {pii}"));
    }
}

fn render_ontology_policy_hooks(
    section: &str,
    hooks: &[OntologyPolicyHookAnswer],
    lines: &mut Vec<String>,
) {
    if hooks.is_empty() {
        return;
    }
    lines.push(format!("{section}:"));
    for hook in hooks {
        lines.push(format!(
            "{}- policy: {}",
            " ".repeat(8),
            yaml_scalar_string(&hook.policy)
        ));
        if let Some(reason) = &hook.reason {
            lines.push(format!(
                "{}reason: {}",
                " ".repeat(10),
                yaml_scalar_string(reason)
            ));
        }
    }
}

fn render_ontology_provider_requirements(
    section: &str,
    requirements: &[ProviderRequirementAnswer],
    lines: &mut Vec<String>,
) {
    if requirements.is_empty() {
        return;
    }
    lines.push(format!("{section}:"));
    for requirement in requirements {
        render_provider_requirement("        -", requirement, lines);
    }
}

fn render_semantic_aliases(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    let Some(aliases) = &resolved.semantic_aliases else {
        return;
    };
    lines.push("semantic_aliases:".to_string());
    render_alias_map("  concepts", &aliases.concepts, lines);
    render_alias_map("  relationships", &aliases.relationships, lines);
}

fn render_alias_map(
    section: &str,
    aliases: &BTreeMap<String, Vec<String>>,
    lines: &mut Vec<String>,
) {
    if aliases.is_empty() {
        lines.push(format!("{section}: {{}}"));
        return;
    }
    lines.push(format!("{section}:"));
    for (target, values) in aliases {
        let values = normalize_text_list(values.clone());
        if values.is_empty() {
            lines.push(format!("    {}: []", yaml_scalar_string(target)));
        } else {
            lines.push(format!("    {}:", yaml_scalar_string(target)));
            for value in values {
                lines.push(format!("      - {}", yaml_scalar_string(&value)));
            }
        }
    }
}

fn render_entity_linking(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    let Some(entity_linking) = &resolved.entity_linking else {
        return;
    };
    lines.push("entity_linking:".to_string());
    if entity_linking.strategies.is_empty() {
        lines.push("  strategies: []".to_string());
        return;
    }
    lines.push("  strategies:".to_string());
    for strategy in &entity_linking.strategies {
        lines.push(format!("    - id: {}", yaml_scalar_string(&strategy.id)));
        lines.push(format!(
            "      applies_to: {}",
            yaml_scalar_string(&strategy.applies_to)
        ));
        if let Some(source_type) = &strategy.source_type {
            lines.push(format!(
                "      source_type: {}",
                yaml_scalar_string(source_type)
            ));
        }
        lines.push("      match:".to_string());
        lines.push(format!(
            "        source_field: {}",
            yaml_scalar_string(&strategy.match_fields.source_field)
        ));
        lines.push(format!(
            "        target_field: {}",
            yaml_scalar_string(&strategy.match_fields.target_field)
        ));
        lines.push(format!("      confidence: {}", strategy.confidence));
        if let Some(sensitivity) = &strategy.sensitivity {
            render_ontology_sensitivity("      sensitivity", sensitivity, lines);
        }
    }
}

fn render_retrieval_bindings(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    let Some(retrieval_bindings) = &resolved.retrieval_bindings else {
        return;
    };
    lines.push("retrieval_bindings:".to_string());
    lines.push(format!(
        "  schema: {}",
        yaml_scalar_string(
            retrieval_bindings
                .schema
                .as_deref()
                .unwrap_or("greentic.sorla.retrieval-bindings.v1")
        )
    ));
    if retrieval_bindings.providers.is_empty() {
        lines.push("  providers: []".to_string());
    } else {
        lines.push("  providers:".to_string());
        for provider in &retrieval_bindings.providers {
            lines.push(format!("    - id: {}", yaml_scalar_string(&provider.id)));
            lines.push(format!(
                "      category: {}",
                yaml_scalar_string(&provider.category)
            ));
            render_string_list(
                "      required_capabilities",
                &provider.required_capabilities,
                lines,
            );
        }
    }

    if retrieval_bindings.scopes.is_empty() {
        lines.push("  scopes: []".to_string());
    } else {
        lines.push("  scopes:".to_string());
        for scope in &retrieval_bindings.scopes {
            lines.push(format!("    - id: {}", yaml_scalar_string(&scope.id)));
            lines.push("      applies_to:".to_string());
            if let Some(concept) = &scope.applies_to.concept {
                lines.push(format!("        concept: {}", yaml_scalar_string(concept)));
            }
            if let Some(relationship) = &scope.applies_to.relationship {
                lines.push(format!(
                    "        relationship: {}",
                    yaml_scalar_string(relationship)
                ));
            }
            lines.push(format!(
                "      provider: {}",
                yaml_scalar_string(&scope.provider)
            ));
            if let Some(filters) = &scope.filters
                && let Some(entity_scope) = &filters.entity_scope
            {
                lines.push("      filters:".to_string());
                lines.push("        entity_scope:".to_string());
                lines.push(format!(
                    "          include_self: {}",
                    entity_scope.include_self.unwrap_or(false)
                ));
                if entity_scope.include_related.is_empty() {
                    lines.push("          include_related: []".to_string());
                } else {
                    lines.push("          include_related:".to_string());
                    for rule in &entity_scope.include_related {
                        lines.push(format!(
                            "            - relationship: {}",
                            yaml_scalar_string(&rule.relationship)
                        ));
                        lines.push(format!(
                            "              direction: {}",
                            yaml_scalar_string(&rule.direction)
                        ));
                        lines.push(format!("              max_depth: {}", rule.max_depth));
                    }
                }
            }
            if let Some(permission) = &scope.permission {
                lines.push(format!(
                    "      permission: {}",
                    yaml_scalar_string(permission)
                ));
            }
        }
    }
}

fn render_records(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    if resolved.record_items.is_empty() {
        lines.push("records: []".to_string());
        return;
    }

    lines.push("records:".to_string());
    for record in &resolved.record_items {
        lines.push(format!("  - name: {}", yaml_scalar_string(&record.name)));
        let source = record
            .source
            .as_deref()
            .unwrap_or(resolved.default_source.as_str());
        lines.push(format!("    source: {}", yaml_scalar_string(source)));
        let external_ref = record.external_ref.as_ref();
        if let Some(external_ref) = external_ref {
            lines.push("    external_ref:".to_string());
            lines.push(format!(
                "      system: {}",
                yaml_scalar_string(&external_ref.system)
            ));
            lines.push(format!(
                "      key: {}",
                yaml_scalar_string(&external_ref.key)
            ));
            lines.push(format!(
                "      authoritative: {}",
                external_ref.authoritative
            ));
        } else if matches!(source, "external" | "hybrid") {
            lines.push("    external_ref:".to_string());
            lines.push(format!(
                "      system: {}",
                yaml_scalar_string(
                    resolved
                        .external_ref_system
                        .as_deref()
                        .unwrap_or("external-system")
                )
            ));
            lines.push("      key: record_id".to_string());
            lines.push("      authoritative: true".to_string());
        }
        render_schema_fields("    fields", &record.fields, lines, false);
    }
}

fn render_actions(actions: &[NamedAnswer], lines: &mut Vec<String>) {
    render_named_section("actions", actions, lines);
}

fn render_named_section(section: &str, items: &[NamedAnswer], lines: &mut Vec<String>) {
    if items.is_empty() {
        lines.push(format!("{section}: []"));
        return;
    }

    lines.push(format!("{section}:"));
    for item in items {
        lines.push(format!("  - name: {}", yaml_scalar_string(&item.name)));
    }
}

fn render_events(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    if !resolved.events_enabled || resolved.event_items.is_empty() {
        lines.push("events: []".to_string());
        return;
    }

    lines.push("events:".to_string());
    for event in &resolved.event_items {
        lines.push(format!("  - name: {}", yaml_scalar_string(&event.name)));
        lines.push(format!("    record: {}", yaml_scalar_string(&event.record)));
        lines.push(format!(
            "    kind: {}",
            yaml_scalar_string(event.kind.as_deref().unwrap_or("domain"))
        ));
        if event.emits.is_empty() {
            lines.push("    emits: []".to_string());
        } else {
            lines.push("    emits:".to_string());
            for field in &event.emits {
                lines.push(format!("      - name: {}", yaml_scalar_string(&field.name)));
                lines.push(format!(
                    "        type: {}",
                    yaml_scalar_string(&field.type_name)
                ));
            }
        }
    }
}

fn render_projections(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    if resolved.projection_items.is_empty() {
        lines.push("projections: []".to_string());
        return;
    }

    lines.push("projections:".to_string());
    for projection in &resolved.projection_items {
        lines.push(format!(
            "  - name: {}",
            yaml_scalar_string(&projection.name)
        ));
        lines.push(format!(
            "    record: {}",
            yaml_scalar_string(&projection.record)
        ));
        lines.push(format!(
            "    source_event: {}",
            yaml_scalar_string(&projection.source_event)
        ));
        lines.push(format!(
            "    mode: {}",
            yaml_scalar_string(
                projection
                    .mode
                    .as_deref()
                    .unwrap_or(resolved.projection_mode.as_str())
            )
        ));
    }
}

fn render_provider_requirements(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    if resolved.provider_requirements.is_empty() {
        lines.push("provider_requirements:".to_string());
        lines.push(format!(
            "  - category: {}",
            yaml_scalar_string(&resolved.storage_category)
        ));
        lines.push("    capabilities:".to_string());
        lines.push("      - event-log".to_string());
        lines.push("      - projections".to_string());
        return;
    }

    lines.push("provider_requirements:".to_string());
    for requirement in &resolved.provider_requirements {
        render_provider_requirement("  -", requirement, lines);
    }
}

fn render_provider_requirement(
    list_prefix: &str,
    requirement: &ProviderRequirementAnswer,
    lines: &mut Vec<String>,
) {
    lines.push(format!(
        "{list_prefix} category: {}",
        yaml_scalar_string(&requirement.category)
    ));
    if requirement.capabilities.is_empty() {
        lines.push("    capabilities: []".to_string());
    } else {
        lines.push("    capabilities:".to_string());
        for capability in &requirement.capabilities {
            lines.push(format!("      - {}", yaml_scalar_string(capability)));
        }
    }
}

fn render_migrations(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    if resolved.migration_items.is_empty() {
        lines.push("migrations:".to_string());
        lines.push(format!(
            "  - name: {}",
            yaml_scalar_string(&format!("{}-compatibility", resolved.package_name))
        ));
        lines.push(format!(
            "    compatibility: {}",
            yaml_scalar_string(&resolved.compatibility_mode)
        ));
        lines.push("    projection_updates: []".to_string());
        return;
    }

    lines.push("migrations:".to_string());
    for migration in &resolved.migration_items {
        lines.push(format!("  - name: {}", yaml_scalar_string(&migration.name)));
        lines.push(format!(
            "    compatibility: {}",
            yaml_scalar_string(
                migration
                    .compatibility
                    .as_deref()
                    .unwrap_or(resolved.compatibility_mode.as_str())
            )
        ));
        render_string_list(
            "    projection_updates",
            &migration.projection_updates,
            lines,
        );
        if migration.backfills.is_empty() {
            lines.push("    backfills: []".to_string());
        } else {
            lines.push("    backfills:".to_string());
            for backfill in &migration.backfills {
                lines.push(format!(
                    "      - record: {}",
                    yaml_scalar_string(&backfill.record)
                ));
                lines.push(format!(
                    "        field: {}",
                    yaml_scalar_string(&backfill.field)
                ));
                lines.push("        default:".to_string());
                render_json_value(&backfill.default, 10, lines);
            }
        }
        if let Some(idempotence_key) = &migration.idempotence_key {
            lines.push(format!(
                "    idempotence_key: {}",
                yaml_scalar_string(idempotence_key)
            ));
        }
        if let Some(notes) = &migration.notes {
            lines.push(format!("    notes: {}", yaml_scalar_string(notes)));
        }
    }
}

fn render_agent_endpoints(resolved: &ResolvedAnswers, lines: &mut Vec<String>) {
    if !resolved.agent_endpoints_enabled || resolved.agent_endpoint_items.is_empty() {
        lines.push("agent_endpoints: []".to_string());
        return;
    }

    lines.push("agent_endpoints:".to_string());
    for endpoint in &resolved.agent_endpoint_items {
        lines.push(format!("  - id: {}", yaml_scalar_string(&endpoint.id)));
        lines.push(format!(
            "    title: {}",
            yaml_scalar_string(&endpoint.title)
        ));
        lines.push(format!(
            "    intent: {}",
            yaml_scalar_string(&endpoint.intent)
        ));
        if let Some(description) = &endpoint.description {
            lines.push(format!(
                "    description: {}",
                yaml_scalar_string(description)
            ));
        }
        render_schema_fields("    inputs", &endpoint.inputs, lines, true);
        render_schema_fields("    outputs", &endpoint.outputs, lines, false);
        render_string_list("    side_effects", &endpoint.side_effects, lines);
        if let Some(emits) = &endpoint.emits {
            lines.push("    emits:".to_string());
            lines.push(format!("      event: {}", yaml_scalar_string(&emits.event)));
            lines.push(format!(
                "      stream: {}",
                yaml_scalar_string(&emits.stream)
            ));
            lines.push("      payload:".to_string());
            render_json_value(&emits.payload, 8, lines);
        }
        lines.push(format!(
            "    risk: {}",
            yaml_scalar_string(
                endpoint
                    .risk
                    .as_deref()
                    .unwrap_or(resolved.agent_endpoint_default_risk.as_str())
            )
        ));
        lines.push(format!(
            "    approval: {}",
            yaml_scalar_string(
                endpoint
                    .approval
                    .as_deref()
                    .unwrap_or(resolved.agent_endpoint_default_approval.as_str())
            )
        ));
        if !endpoint.provider_requirements.is_empty() {
            lines.push("    provider_requirements:".to_string());
            for requirement in &endpoint.provider_requirements {
                lines.push(format!(
                    "      - category: {}",
                    yaml_scalar_string(&requirement.category)
                ));
                if requirement.capabilities.is_empty() {
                    lines.push("        capabilities: []".to_string());
                } else {
                    lines.push("        capabilities:".to_string());
                    for capability in &requirement.capabilities {
                        lines.push(format!("          - {}", yaml_scalar_string(capability)));
                    }
                }
            }
        }
        render_endpoint_backing(&endpoint.backing, lines);
        if let Some(visibility) = &endpoint.agent_visibility {
            lines.push("    agent_visibility:".to_string());
            lines.push(format!(
                "      openapi: {}",
                visibility.openapi.unwrap_or(true)
            ));
            lines.push(format!(
                "      arazzo: {}",
                visibility.arazzo.unwrap_or(true)
            ));
            lines.push(format!("      mcp: {}", visibility.mcp.unwrap_or(true)));
            lines.push(format!(
                "      llms_txt: {}",
                visibility.llms_txt.unwrap_or(true)
            ));
        }
        if !endpoint.examples.is_empty() {
            lines.push("    examples:".to_string());
            for example in &endpoint.examples {
                lines.push(format!(
                    "      - name: {}",
                    yaml_scalar_string(&example.name)
                ));
                lines.push(format!(
                    "        summary: {}",
                    yaml_scalar_string(&example.summary)
                ));
                lines.push("        input:".to_string());
                render_json_value(&example.input, 10, lines);
                lines.push("        expected_output:".to_string());
                render_json_value(&example.expected_output, 10, lines);
            }
        }
    }
}

fn render_schema_fields(
    section: &str,
    fields: &[FieldAnswer],
    lines: &mut Vec<String>,
    include_endpoint_properties: bool,
) {
    if fields.is_empty() {
        lines.push(format!("{section}: []"));
        return;
    }

    lines.push(format!("{section}:"));
    for field in fields {
        lines.push(format!("      - name: {}", yaml_scalar_string(&field.name)));
        lines.push(format!(
            "        type: {}",
            yaml_scalar_string(&field.type_name)
        ));
        if let Some(authority) = &field.authority {
            lines.push(format!(
                "        authority: {}",
                yaml_scalar_string(authority)
            ));
        }
        if let Some(required) = field.required {
            lines.push(format!("        required: {required}"));
        }
        if let Some(sensitive) = field.sensitive {
            lines.push(format!("        sensitive: {sensitive}"));
        }
        if !field.enum_values.is_empty() {
            render_string_list("        enum_values", &field.enum_values, lines);
        }
        if let Some(reference) = &field.references {
            lines.push("        references:".to_string());
            lines.push(format!(
                "          record: {}",
                yaml_scalar_string(&reference.record)
            ));
            lines.push(format!(
                "          field: {}",
                yaml_scalar_string(&reference.field)
            ));
        }
        if include_endpoint_properties && let Some(description) = &field.description {
            lines.push(format!(
                "        description: {}",
                yaml_scalar_string(description)
            ));
        }
    }
}

fn render_endpoint_backing(backing: &AgentEndpointBackingAnswer, lines: &mut Vec<String>) {
    if backing.actions.is_empty()
        && backing.events.is_empty()
        && backing.flows.is_empty()
        && backing.policies.is_empty()
        && backing.approvals.is_empty()
    {
        return;
    }

    lines.push("    backing:".to_string());
    render_string_list("      actions", &backing.actions, lines);
    render_string_list("      events", &backing.events, lines);
    if !backing.flows.is_empty() {
        render_string_list("      flows", &backing.flows, lines);
    }
    render_string_list("      policies", &backing.policies, lines);
    render_string_list("      approvals", &backing.approvals, lines);
}

fn render_string_list(section: &str, values: &[String], lines: &mut Vec<String>) {
    if values.is_empty() {
        lines.push(format!("{section}: []"));
    } else {
        lines.push(format!("{section}:"));
        for value in values {
            lines.push(format!(
                "{}- {}",
                " ".repeat(section.len() - section.trim_start().len() + 2),
                yaml_scalar_string(value)
            ));
        }
    }
}

fn render_json_value(value: &serde_json::Value, indent: usize, lines: &mut Vec<String>) {
    let spaces = " ".repeat(indent);
    match value {
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                lines.push(format!("{spaces}{{}}"));
            } else {
                for (key, value) in map {
                    match value {
                        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                            lines.push(format!("{spaces}{}:", yaml_key(key)));
                            render_json_value(value, indent + 2, lines);
                        }
                        _ => lines.push(format!(
                            "{spaces}{}: {}",
                            yaml_key(key),
                            yaml_scalar_value(value)
                        )),
                    }
                }
            }
        }
        serde_json::Value::Array(values) => {
            if values.is_empty() {
                lines.push(format!("{spaces}[]"));
            } else {
                for value in values {
                    match value {
                        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                            lines.push(format!("{spaces}-"));
                            render_json_value(value, indent + 2, lines);
                        }
                        _ => lines.push(format!("{spaces}- {}", yaml_scalar_value(value))),
                    }
                }
            }
        }
        _ => lines.push(format!("{spaces}{}", yaml_scalar_value(value))),
    }
}

fn yaml_scalar_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => yaml_scalar_string(value),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
        }
    }
}

fn yaml_key(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
    {
        value.to_string()
    } else {
        yaml_scalar_string(value)
    }
}

fn yaml_scalar_string(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | '{' | '}'))
        && !matches!(
            value,
            "true" | "false" | "null" | "yes" | "no" | "on" | "off"
        )
    {
        value.to_string()
    } else {
        serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
    }
}

fn build_launcher_handoff_manifest(
    resolved: &ResolvedAnswers,
) -> BTreeMap<&'static str, serde_json::Value> {
    let mut map = BTreeMap::new();
    map.insert(
        "handoff_kind",
        serde_json::Value::String("launcher".to_string()),
    );
    map.insert(
        "handoff_owner",
        serde_json::Value::String("gtc".to_string()),
    );
    map.insert(
        "handoff_role",
        serde_json::Value::String("extension-metadata".to_string()),
    );
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
        serde_json::to_value(build_provider_handoff_manifest(resolved))
            .expect("provider requirement manifest is serializable"),
    );
    map.insert(
        "gtc_handoff",
        serde_json::json!({
            "stage": "launcher",
            "owner": "gtc",
            "final_assembly_owner": "gtc",
        }),
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

fn build_provider_handoff_manifest(
    resolved: &ResolvedAnswers,
) -> BTreeMap<&'static str, serde_json::Value> {
    let mut map = BTreeMap::new();
    map.insert(
        "handoff_kind",
        serde_json::Value::String("provider-requirements".to_string()),
    );
    map.insert(
        "handoff_owner",
        serde_json::Value::String("gtc".to_string()),
    );
    map.insert(
        "handoff_stage",
        serde_json::Value::String("launcher".to_string()),
    );
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

fn build_locale_handoff_manifest(
    resolved: &ResolvedAnswers,
) -> BTreeMap<&'static str, serde_json::Value> {
    let mut map = BTreeMap::new();
    map.insert(
        "handoff_kind",
        serde_json::Value::String("locale".to_string()),
    );
    map.insert(
        "handoff_owner",
        serde_json::Value::String("gtc".to_string()),
    );
    map.insert(
        "handoff_stage",
        serde_json::Value::String("launcher".to_string()),
    );
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
                "description": "Directory where sorla.yaml and generated handoff metadata will be written.",
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
                "description": "Stable source layout identifier written into sorla.yaml.",
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
                "description": "Version for the new source layout.",
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
                "description": "Provider category required for source storage and generated handoff metadata.",
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
                "description": "External system identifier used when the source layout references authoritative external records.",
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
                "description": "Generate event and projection placeholders for this source layout.",
                "required": true,
                "default_value": "true"
            },
            {
                "id": "projection_mode",
                "type": "enum",
                "title": "Projection mode",
                "title_i18n": { "key": "wizard.questions.projection_mode.label" },
                "description": "Projection strategy for generated source and handoff output.",
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

#[cfg(feature = "cli")]
fn load_interactive_i18n(locale: &str) -> Option<ResolvedI18nMap> {
    let raw = included_locale_json(locale).or_else(|| included_locale_json("en"))?;
    let map = serde_json::from_str::<BTreeMap<String, String>>(raw).ok()?;
    let mut resolved = ResolvedI18nMap::new();
    for (key, value) in map {
        resolved.insert(key, value);
    }
    Some(resolved)
}

fn included_locale_json(locale: &str) -> Option<&'static str> {
    embedded_i18n::locale_json(locale)
}

#[cfg(feature = "cli")]
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
        actions: Vec::new(),
        records: Some(RecordAnswers {
            default_source: Some(default_source),
            external_ref_system,
            items: Vec::new(),
        }),
        ontology: None,
        semantic_aliases: None,
        entity_linking: None,
        retrieval_bindings: None,
        events: Some(EventAnswers {
            enabled: Some(get_required_bool(object, "events_enabled")?),
            items: Vec::new(),
        }),
        projections: Some(ProjectionAnswers {
            mode: Some(get_required_string(object, "projection_mode")?),
            items: Vec::new(),
        }),
        provider_requirements: Vec::new(),
        policies: Vec::new(),
        approvals: Vec::new(),
        migrations: Some(MigrationAnswers {
            compatibility: Some(get_required_string(object, "compatibility_mode")?),
            items: Vec::new(),
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
            items: Vec::new(),
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

#[cfg(feature = "cli")]
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
        let bytes = serde_json::to_vec_pretty(&build_provider_handoff_manifest(resolved))
            .map_err(|err| err.to_string())?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write generated file {}: {err}", path.display()))?;
        return Ok(());
    }

    if artifact == "locale-manifest.json" {
        let bytes = serde_json::to_vec_pretty(&build_locale_handoff_manifest(resolved))
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
    default_schema_for_locale(&selected_locale(None, None))
}

fn default_schema_for_locale(locale: &str) -> WizardSchema {
    WizardSchema {
        schema_version: "0.5",
        wizard_version: "0.5",
        package_version: "0.1.0",
        locale: locale.to_string(),
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
                id: "ontology",
                title_key: "wizard.sections.ontology.title",
                description_key: "wizard.sections.ontology.description",
                flows: vec![SchemaFlow::Create, SchemaFlow::Update],
                questions: vec![
                    WizardQuestion {
                        id: "ontology.schema",
                        label_key: "wizard.questions.ontology_schema.label",
                        help_key: Some("wizard.questions.ontology_schema.help"),
                        kind: WizardQuestionKind::Text,
                        required: false,
                        default_value: Some("greentic.sorla.ontology.v1"),
                        choices: vec![],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "ontology.concepts",
                        label_key: "wizard.questions.ontology_concepts.label",
                        help_key: Some("wizard.questions.ontology_concepts.help"),
                        kind: WizardQuestionKind::TextList,
                        required: false,
                        default_value: None,
                        choices: vec![],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "ontology.relationships",
                        label_key: "wizard.questions.ontology_relationships.label",
                        help_key: Some("wizard.questions.ontology_relationships.help"),
                        kind: WizardQuestionKind::TextList,
                        required: false,
                        default_value: None,
                        choices: vec![],
                        visibility: None,
                    },
                    WizardQuestion {
                        id: "retrieval_bindings.scopes",
                        label_key: "wizard.questions.retrieval_bindings_scopes.label",
                        help_key: Some("wizard.questions.retrieval_bindings_scopes.help"),
                        kind: WizardQuestionKind::TextList,
                        required: false,
                        default_value: None,
                        choices: vec![],
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
    fn pack_metadata_schema_helpers_are_deterministic() {
        for (schema, expected_id) in [
            (sorx_validation_schema_json(), SORX_VALIDATION_SCHEMA),
            (
                sorx_exposure_policy_schema_json(),
                SORX_EXPOSURE_POLICY_SCHEMA,
            ),
            (sorx_compatibility_schema_json(), SORX_COMPATIBILITY_SCHEMA),
            (
                ontology_schema_json(),
                greentic_sorla_pack::ONTOLOGY_EXTENSION_ID,
            ),
            (
                retrieval_bindings_schema_json(),
                greentic_sorla_pack::RETRIEVAL_BINDINGS_SCHEMA,
            ),
        ] {
            let first = serde_json::to_string_pretty(&schema).expect("schema serializes");
            let second = serde_json::to_string_pretty(&schema).expect("schema serializes again");
            assert_eq!(first, second);
            assert_eq!(schema["$id"], expected_id);
        }
    }

    #[test]
    fn pack_validation_inspection_json_is_compact() {
        let dir = unique_temp_dir();
        let pack_path = dir.join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: pack_path.clone(),
        })
        .expect("pack builds");
        let inspection = inspect_sorla_gtpack(&pack_path).expect("pack inspects");
        let validation = validation_inspection_json(&inspection);

        assert_eq!(validation["schema"], SORX_VALIDATION_SCHEMA);
        assert_eq!(validation["package"]["name"], "landlord-tenant-sor");
        assert_eq!(validation["validation"]["suite_count"], 4);
        assert_eq!(
            validation["exposure"]["default_visibility"],
            serde_json::json!("private")
        );
        assert_eq!(
            validation["compatibility"]["state_mode"],
            serde_json::json!("isolated_required")
        );
        assert!(validation["ontology"].is_null());
    }

    #[test]
    fn pack_metadata_subcommands_parse() {
        Cli::try_parse_from(["greentic-sorla", "pack", "schema", "validation"])
            .expect("validation schema command parses");
        Cli::try_parse_from(["greentic-sorla", "pack", "schema", "exposure-policy"])
            .expect("exposure schema command parses");
        Cli::try_parse_from(["greentic-sorla", "pack", "schema", "compatibility"])
            .expect("compatibility schema command parses");
        Cli::try_parse_from(["greentic-sorla", "pack", "schema", "ontology"])
            .expect("ontology schema command parses");
        Cli::try_parse_from(["greentic-sorla", "pack", "schema", "retrieval-bindings"])
            .expect("retrieval bindings schema command parses");
        Cli::try_parse_from(["greentic-sorla", "pack", "validation-inspect", "x.gtpack"])
            .expect("validation-inspect command parses");
        Cli::try_parse_from(["greentic-sorla", "pack", "validation-doctor", "x.gtpack"])
            .expect("validation-doctor command parses");
    }

    #[test]
    fn public_facade_builds_preview_and_gtpack_bytes() {
        let answers_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("answers")
            .join("create_minimal.json");
        let input: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(answers_path).expect("example answers read"))
                .expect("example answers parse");

        let schema = schema_for_answers().expect("schema emits");
        assert_eq!(schema["schema_version"], "0.5");

        let model = normalize_answers(input, NormalizeOptions).expect("answers normalize");
        assert_eq!(model.package_name, "tenancy");

        let report = validate_model(&model, ValidateOptions);
        assert!(!report.has_errors(), "{report:?}");

        let preview = generate_preview(&model, PreviewOptions).expect("preview generates");
        assert_eq!(preview.summary.package_name, "tenancy");
        assert!(preview.summary.records >= 1);

        let entries =
            build_gtpack_entries(&model, PackBuildOptions::default()).expect("pack entries build");
        let entries_again = build_gtpack_entries(&model, PackBuildOptions::default())
            .expect("pack entries build again");
        assert_eq!(entries, entries_again);
        assert!(
            entries
                .iter()
                .any(|entry| entry.path == "assets/sorla/model.cbor")
        );

        let pack = build_gtpack_bytes(&model, PackBuildOptions::default()).expect("pack builds");
        assert_eq!(pack.filename, "tenancy.gtpack");
        assert_eq!(pack.sha256, sha256_hex_public(&pack.bytes));
        assert_eq!(pack.metadata.pack_id, "tenancy");

        let inspection = inspect_gtpack_bytes(&pack.bytes).expect("bytes inspect");
        assert_eq!(inspection.name, "tenancy");

        let doctor = doctor_gtpack_bytes(&pack.bytes);
        assert!(!doctor.has_errors(), "{doctor:?}");
    }

    #[test]
    fn public_facade_lists_designer_node_types() {
        let answers_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("answers")
            .join("create_agent_endpoints.json");
        let input: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(answers_path).expect("example answers read"))
                .expect("example answers parse");
        let model = normalize_answers(input, NormalizeOptions).expect("answers normalize");
        let node_types = list_designer_node_types(&model, DesignerNodeTypeOptions::default())
            .expect("node types generate");
        assert_eq!(
            node_types.schema,
            greentic_sorla_pack::DESIGNER_NODE_TYPES_SCHEMA
        );
        assert!(!node_types.node_types.is_empty());
        assert!(
            node_types.node_types[0]
                .metadata
                .endpoint
                .contract_hash
                .starts_with("sha256:")
        );
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
    fn wizard_schema_accepts_explicit_locale() {
        let cli = Cli::try_parse_from(["greentic-sorla", "wizard", "--locale", "es", "--schema"])
            .expect("wizard schema should accept explicit locale");
        let Commands::Wizard(args) = cli.command else {
            panic!("expected wizard command");
        };

        let schema = default_schema_for_locale(&selected_locale(args.locale.as_deref(), None));
        assert_eq!(schema.locale, "es");
    }

    #[test]
    fn localized_help_accepts_locale_after_help_flag() {
        let root_help = localized_help_for_args(&[
            OsString::from("greentic-sorla"),
            OsString::from("--help"),
            OsString::from("--locale"),
            OsString::from("de"),
        ])
        .expect("root help should localize");
        assert!(root_help.contains("SoRLa-Wizard"));
        assert!(root_help.contains("Schema generieren oder Antwortdokumente anwenden."));

        let wizard_help = localized_help_for_args(&[
            OsString::from("greentic-sorla"),
            OsString::from("wizard"),
            OsString::from("--help"),
            OsString::from("--locale"),
            OsString::from("de"),
        ])
        .expect("wizard help should localize");
        assert!(wizard_help.contains("Schema generieren oder Antwortdokumente anwenden."));
        assert!(wizard_help.contains("Paketname"));
        assert!(wizard_help.contains("Kategorie des Speicher-Providers"));
    }

    #[test]
    fn embedded_i18n_catalogs_are_available_without_filesystem_lookups() {
        let resolved = load_interactive_i18n("es").expect("embedded locale should load");
        assert_eq!(
            resolved.get("wizard.title").map(String::as_str),
            Some("Asistente de SoRLa")
        );
        assert_eq!(
            resolved.get("wizard.flow.label").map(String::as_str),
            Some("Flujo del asistente")
        );
    }

    #[test]
    fn bundled_i18n_catalogs_cover_supported_locales() {
        let raw_locales = included_locale_json("locales").expect("locales list is bundled");
        let locales: Vec<String> =
            serde_json::from_str(raw_locales).expect("locales list should parse");

        for locale in locales {
            let raw = included_locale_json(&locale)
                .unwrap_or_else(|| panic!("locale `{locale}` should be bundled"));
            serde_json::from_str::<BTreeMap<String, String>>(raw)
                .unwrap_or_else(|err| panic!("bundled locale `{locale}` should parse: {err}"));
        }
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
        assert!(lock.contains("\"locale\":"));
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
                .join("launcher-handoff.json"),
        )
        .unwrap();
        assert!(manifest.contains("\"package_kind\": \"greentic-sorla-package\""));
        assert!(manifest.contains("\"fallback_locale\": \"en\""));
        assert!(manifest.contains("\"handoff_owner\": \"gtc\""));
        assert!(manifest.contains("\"stage\": \"launcher\""));

        let legacy_manifest = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("package-manifest.json"),
        )
        .unwrap();
        assert_eq!(manifest, legacy_manifest);

        let locale_manifest = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("locale-manifest.json"),
        )
        .unwrap();
        assert!(locale_manifest.contains("\"default_locale\": \"en\""));
        assert!(locale_manifest.contains("\"handoff_kind\": \"locale\""));

        let provider_manifest = fs::read_to_string(
            output_dir
                .join(".greentic-sorla")
                .join("generated")
                .join("provider-requirements.json"),
        )
        .unwrap();
        assert!(provider_manifest.contains("\"name\": \"storage\""));
        assert!(provider_manifest.contains("\"handoff_kind\": \"provider-requirements\""));
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

        let validation_manifest_path = output_dir
            .join(".greentic-sorla")
            .join("generated")
            .join("assets")
            .join("sorx")
            .join("tests")
            .join("test-manifest.json");
        let validation_manifest = fs::read_to_string(validation_manifest_path)
            .expect("wizard should write generated validation manifest");
        let validation: serde_json::Value =
            serde_json::from_str(&validation_manifest).expect("validation manifest is JSON");
        assert_eq!(validation["schema"], "greentic.sorx.validation.v1");
        assert_eq!(validation["package"]["name"], "landlord-tenant-sor");
        assert!(
            validation["promotion_requires"]
                .as_array()
                .expect("promotion requirements")
                .iter()
                .any(|suite| suite == "contract")
        );
    }

    #[test]
    fn landlord_tenant_pack_example_answers_generate_gtpack() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("landlord-tenant-pack.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();
        let example_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("landlord-tenant")
            .join("answers.json");
        let example = fs::read_to_string(example_path).expect("example answers should read");
        let patched = example.replace(
            r#""output_dir": "examples/landlord-tenant""#,
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
        let package_yaml = fs::read_to_string(output_dir.join("sorla.yaml")).unwrap();
        assert!(package_yaml.contains("name: Landlord"));
        assert!(package_yaml.contains("name: MaintenanceRequest"));
        assert!(package_yaml.contains("ontology:"));
        assert!(package_yaml.contains("semantic_aliases:"));
        assert!(package_yaml.contains("entity_linking:"));
        assert!(package_yaml.contains("id: Tenant"));
        assert!(package_yaml.contains("id: occupies"));
        assert!(package_yaml.contains("id: tenant_email_match"));
        assert!(package_yaml.contains("id: create_tenant"));
        assert!(package_yaml.contains("event: TenantCreated"));
        assert!(!package_yaml.contains("LandlordTenantSorRecord"));
        assert_eq!(
            inspection
                .ontology
                .as_ref()
                .expect("ontology should inspect")
                .concept_count,
            6
        );
        assert_eq!(
            inspection
                .optional_artifacts
                .get("assets/sorla/mcp-tools.json"),
            Some(&true)
        );
    }

    #[test]
    fn ontology_business_answers_generate_deterministic_handoff_pack() {
        let dir = unique_temp_dir();
        let answers_a = dir.join("ontology-business-a.json");
        let answers_b = dir.join("ontology-business-b.json");
        let output_a = dir.join("workspace-a");
        let output_b = dir.join("workspace-b");
        fs::create_dir_all(&output_a).unwrap();
        fs::create_dir_all(&output_b).unwrap();

        let example_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("ontology-business")
            .join("answers.json");
        let example = fs::read_to_string(example_path).expect("ontology example answers read");
        let mut first: serde_json::Value =
            serde_json::from_str(&example).expect("ontology example answers parse");
        first["output_dir"] = serde_json::Value::String(output_a.display().to_string());
        let mut second = first.clone();
        second["output_dir"] = serde_json::Value::String(output_b.display().to_string());
        fs::write(&answers_a, serde_json::to_vec_pretty(&first).unwrap()).unwrap();
        fs::write(&answers_b, serde_json::to_vec_pretty(&second).unwrap()).unwrap();

        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_a.to_str().unwrap(),
            "--pack-out",
            "ontology-business.gtpack",
        ])
        .unwrap();
        run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_b.to_str().unwrap(),
            "--pack-out",
            "ontology-business.gtpack",
        ])
        .unwrap();

        let package_yaml = fs::read_to_string(output_a.join("sorla.yaml")).unwrap();
        assert!(package_yaml.contains("ontology:"));
        assert!(package_yaml.contains("semantic_aliases:"));
        assert!(package_yaml.contains("entity_linking:"));
        assert!(package_yaml.contains("retrieval_bindings:"));

        let pack_a = output_a.join("ontology-business.gtpack");
        let pack_b = output_b.join("ontology-business.gtpack");
        assert_eq!(fs::read(&pack_a).unwrap(), fs::read(&pack_b).unwrap());
        doctor_sorla_gtpack(&pack_a).expect("ontology business pack should doctor");
        let inspection = inspect_sorla_gtpack(&pack_a).expect("ontology business pack inspect");
        assert!(inspection.ontology.is_some());
        assert!(inspection.retrieval_bindings.is_some());
        let validation = inspection.validation.as_ref().expect("validation summary");
        assert!(
            validation
                .promotion_requires
                .contains(&"ontology".to_string())
        );
        assert!(
            validation
                .promotion_requires
                .contains(&"retrieval".to_string())
        );

        let validation_json = validation_inspection_json(&inspection);
        assert_eq!(
            validation_json["retrieval_bindings"]["schema"],
            "greentic.sorla.retrieval-bindings.v1"
        );

        let generated_dir = output_a.join(".greentic-sorla").join("generated");
        for file in [
            "launcher-handoff.json",
            "provider-requirements.json",
            "assets/sorx/tests/test-manifest.json",
        ] {
            let text = fs::read_to_string(generated_dir.join(file)).unwrap();
            assert!(!text.contains(output_a.to_str().unwrap()));
            assert!(!text.to_ascii_lowercase().contains("tenant_id"));
            assert!(!text.to_ascii_lowercase().contains("password"));
            assert!(!text.to_ascii_lowercase().contains("api_key"));
        }
    }

    #[test]
    fn wizard_answers_generate_ontology_section() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("ontology.json");
        let output_dir = dir.join("workspace");
        fs::create_dir_all(&output_dir).unwrap();

        fs::write(
            &answers_path,
            format!(
                r#"{{
  "schema_version": "0.5",
  "flow": "create",
  "output_dir": "{}",
  "package": {{
    "name": "ontology-demo",
    "version": "0.1.0"
  }},
  "records": {{
    "default_source": "native",
    "items": [
      {{
        "name": "Customer",
        "fields": [
          {{ "name": "id", "type": "string" }}
        ]
      }},
      {{
        "name": "Contract",
        "fields": [
          {{ "name": "id", "type": "string" }}
        ]
      }},
      {{
        "name": "CustomerContract",
        "fields": [
          {{ "name": "customer_id", "type": "string" }},
          {{ "name": "contract_id", "type": "string" }}
        ]
      }}
    ]
  }},
  "ontology": {{
    "concepts": [
      {{ "id": "Party", "kind": "abstract" }},
      {{
        "id": "Customer",
        "kind": "entity",
        "extends": ["Party"],
        "backed_by": {{ "record": "Customer" }},
        "sensitivity": {{ "classification": "confidential", "pii": true }}
      }},
      {{
        "id": "Contract",
        "kind": "entity",
        "backed_by": {{ "record": "Contract" }}
      }}
    ],
    "relationships": [
      {{
        "id": "has_contract",
        "from": "Customer",
        "to": "Contract",
        "cardinality": {{ "from": "one", "to": "many" }},
        "backed_by": {{
          "record": "CustomerContract",
          "from_field": "customer_id",
          "to_field": "contract_id"
        }}
      }}
    ]
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
        assert!(package_yaml.contains("ontology:"));
        assert!(package_yaml.contains("schema: greentic.sorla.ontology.v1"));
        assert!(package_yaml.contains("id: has_contract"));
        assert!(package_yaml.contains("from_field: customer_id"));
        serde_yaml::from_str::<serde_yaml::Value>(&package_yaml)
            .expect("generated ontology YAML should be valid YAML");
    }

    #[test]
    fn wizard_schema_includes_optional_ontology_section() {
        let schema = default_schema();
        let ontology = schema
            .sections
            .iter()
            .find(|section| section.id == "ontology")
            .expect("ontology section should be present");
        assert!(
            ontology
                .questions
                .iter()
                .any(|question| question.id == "ontology.schema" && !question.required)
        );
    }

    #[test]
    fn rich_answers_validate_cross_references_with_answer_paths() {
        let dir = unique_temp_dir();
        let answers_path = dir.join("invalid-rich.json");
        let output_dir = dir.join("workspace");
        fs::write(
            &answers_path,
            format!(
                r#"{{
  "schema_version": "0.5",
  "flow": "create",
  "output_dir": "{}",
  "package": {{
    "name": "invalid-rich",
    "version": "0.1.0"
  }},
  "records": {{
    "default_source": "native",
    "items": [
      {{
        "name": "Tenant",
        "fields": [
          {{
            "name": "landlord_id",
            "type": "string",
            "references": {{ "record": "Landlord", "field": "id" }}
          }}
        ]
      }}
    ]
  }}
}}"#,
                output_dir.display()
            ),
        )
        .unwrap();

        let error = run([
            "greentic-sorla",
            "wizard",
            "--answers",
            answers_path.to_str().unwrap(),
        ])
        .expect_err("unknown record reference should fail");

        assert!(error.contains("records.items[0].fields[0].references.record"));
        assert!(error.contains("Landlord"));
    }

    #[test]
    fn validation_pack_example_answers_generate_gtpacks() {
        for (file_name, output_dir_literal, pack_name, pack_version) in [
            (
                "minimal_validation_pack.json",
                "target/greentic-sorla-minimal-validation-pack-example",
                "minimal-validation-sor",
                "0.1.0",
            ),
            (
                "landlord_tenant_validation_pack.json",
                "target/greentic-sorla-landlord-tenant-validation-pack-example",
                "landlord-tenant-sor",
                "0.1.0",
            ),
            (
                "landlord_tenant_exported_candidate_pack.json",
                "target/greentic-sorla-landlord-tenant-exported-candidate-pack-example",
                "landlord-tenant-sor",
                "0.1.0",
            ),
        ] {
            let dir = unique_temp_dir();
            let answers_path = dir.join(file_name);
            let output_dir = dir.join("workspace");
            fs::create_dir_all(&output_dir).unwrap();
            let example_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join("answers")
                .join(file_name);
            let example = fs::read_to_string(example_path).expect("example answers should read");
            let patched = example.replace(
                &format!(r#""output_dir": "{output_dir_literal}""#),
                &format!(r#""output_dir": "{}""#, output_dir.display()),
            );
            fs::write(&answers_path, patched).unwrap();

            let pack_filename = format!("{pack_name}.gtpack");
            run([
                "greentic-sorla",
                "wizard",
                "--answers",
                answers_path.to_str().unwrap(),
                "--pack-out",
                &pack_filename,
            ])
            .unwrap();

            let pack_path = output_dir.join(pack_filename);
            doctor_sorla_gtpack(&pack_path).expect("validation example pack should doctor");
            let inspection =
                inspect_sorla_gtpack(&pack_path).expect("validation example should inspect");
            assert_eq!(inspection.name, pack_name);
            assert_eq!(inspection.version, pack_version);
            assert!(inspection.validation.is_some());
            assert!(inspection.exposure_policy.is_some());
            assert!(inspection.compatibility.is_some());
        }
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
