use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPackage {
    pub package: Package,
    pub warnings: Vec<ParseWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseWarning {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub package: PackageMeta,
    #[serde(default)]
    pub records: Vec<Record>,
    #[serde(default)]
    pub events: Vec<EventDecl>,
    #[serde(default)]
    pub actions: Vec<ActionDecl>,
    #[serde(default)]
    pub policies: Vec<NamedBlock>,
    #[serde(default)]
    pub approvals: Vec<NamedBlock>,
    #[serde(default)]
    pub views: Vec<NamedBlock>,
    #[serde(default)]
    pub flows: Vec<NamedBlock>,
    #[serde(default)]
    pub projections: Vec<ProjectionDecl>,
    #[serde(default)]
    pub migrations: Vec<MigrationDecl>,
    #[serde(default)]
    pub provider_requirements: Vec<ProviderRequirement>,
    #[serde(default)]
    pub agent_endpoints: Vec<AgentEndpointDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub name: String,
    #[serde(default)]
    pub source: Option<RecordSource>,
    #[serde(default)]
    pub external_ref: Option<ExternalRef>,
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordSource {
    Native,
    External,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalRef {
    pub system: String,
    pub key: String,
    #[serde(default)]
    pub authoritative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub authority: Option<FieldAuthority>,
    #[serde(default)]
    pub references: Option<FieldReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldAuthority {
    Local,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldReference {
    pub record: String,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDecl {
    pub name: String,
    pub record: String,
    #[serde(default)]
    pub kind: EventKind,
    #[serde(default)]
    pub emits: Vec<EventField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    #[default]
    Domain,
    Integration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionDecl {
    pub name: String,
    pub record: String,
    pub source_event: String,
    #[serde(default)]
    pub mode: ProjectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionMode {
    #[default]
    CurrentState,
    AuditTrail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderRequirement {
    pub category: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointDecl {
    pub id: String,
    pub title: String,
    pub intent: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub inputs: Vec<AgentEndpointInputDecl>,
    #[serde(default)]
    pub outputs: Vec<AgentEndpointOutputDecl>,
    #[serde(default)]
    pub side_effects: Vec<String>,
    #[serde(default)]
    pub risk: AgentEndpointRisk,
    #[serde(default)]
    pub approval: AgentEndpointApprovalMode,
    #[serde(default)]
    pub provider_requirements: Vec<ProviderRequirement>,
    #[serde(default)]
    pub backing: AgentEndpointBackingDecl,
    #[serde(default)]
    pub agent_visibility: AgentEndpointVisibility,
    #[serde(default)]
    pub examples: Vec<AgentEndpointExampleDecl>,
    #[serde(default)]
    pub emits: Option<AgentEndpointEmitDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointInputDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointOutputDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointRisk {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointApprovalMode {
    #[default]
    None,
    Optional,
    Required,
    PolicyDriven,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointBackingDecl {
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub flows: Vec<String>,
    #[serde(default)]
    pub policies: Vec<String>,
    #[serde(default)]
    pub approvals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointVisibility {
    #[serde(default = "default_true")]
    pub openapi: bool,
    #[serde(default = "default_true")]
    pub arazzo: bool,
    #[serde(default = "default_true")]
    pub mcp: bool,
    #[serde(default = "default_true")]
    pub llms_txt: bool,
}

impl Default for AgentEndpointVisibility {
    fn default() -> Self {
        Self {
            openapi: true,
            arazzo: true,
            mcp: true,
            llms_txt: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointExampleDecl {
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub input: serde_json::Value,
    #[serde(default)]
    pub expected_output: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointEmitDecl {
    pub event: String,
    pub stream: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationDecl {
    pub name: String,
    #[serde(default)]
    pub compatibility: CompatibilityMode,
    #[serde(default)]
    pub projection_updates: Vec<String>,
    #[serde(default)]
    pub backfills: Vec<MigrationBackfillDecl>,
    #[serde(default)]
    pub idempotence_key: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationBackfillDecl {
    pub record: String,
    pub field: String,
    #[serde(default)]
    pub default: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityMode {
    #[default]
    Additive,
    BackwardCompatible,
    Breaking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionDecl {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedBlock {
    pub name: String,
}

fn default_true() -> bool {
    true
}
