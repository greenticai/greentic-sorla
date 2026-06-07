use super::Provenance;
use crate::prompt::CompilerOptions;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpansionContext {
    pub mode: CompileMode,
    pub options: CompilerOptions,
    pub naming: NamingRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompileMode {
    Create,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamingRules {
    pub action_separator: String,
    pub agent_prefix: String,
}

impl Default for NamingRules {
    fn default() -> Self {
        Self {
            action_separator: ".".to_string(),
            agent_prefix: "agent".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpandedSorlaPlan {
    pub package: Option<PackagePlan>,
    #[serde(default)]
    pub actors: Vec<ActorPlan>,
    #[serde(default)]
    pub records: Vec<RecordPlan>,
    #[serde(default)]
    pub processes: Vec<ProcessPlan>,
    #[serde(default)]
    pub business_rules: Vec<BusinessRulePlan>,
    #[serde(default)]
    pub capabilities: Vec<CapabilityPlan>,
    #[serde(default)]
    pub actions: Vec<ActionPlan>,
    #[serde(default)]
    pub events: Vec<EventPlan>,
    #[serde(default)]
    pub projections: Vec<ProjectionPlan>,
    #[serde(default)]
    pub metrics: Vec<MetricPlan>,
    #[serde(default)]
    pub policies: Vec<PolicyPlan>,
    #[serde(default)]
    pub migrations: Vec<MigrationPlan>,
    #[serde(default)]
    pub agent_endpoints: Vec<AgentEndpointPlan>,
    #[serde(default)]
    pub diagnostics: Vec<CompileDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePlan {
    pub name: String,
    pub version: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorPlan {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordPlan {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub fields: Vec<FieldPlan>,
    #[serde(default)]
    pub relationships: Vec<RelationshipPlan>,
    pub lifecycle: Option<LifecyclePlan>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldPlan {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    #[serde(default)]
    pub values: Vec<String>,
    pub description: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipPlan {
    pub name: String,
    pub target: String,
    pub cardinality: String,
    pub required: bool,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecyclePlan {
    pub state_field: String,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub transitions: Vec<StateTransitionPlan>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransitionPlan {
    pub from: Option<String>,
    pub to: String,
    pub actor: Option<String>,
    pub description: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessPlan {
    pub name: String,
    pub description: Option<String>,
    pub main_record: Option<String>,
    #[serde(default)]
    pub steps: Vec<ProcessStepPlan>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessStepPlan {
    pub name: String,
    pub actor: Option<String>,
    pub action: Option<String>,
    pub record: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessRulePlan {
    pub name: String,
    pub description: String,
    pub applies_to: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPlan {
    pub name: String,
    pub enabled: bool,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionPlan {
    pub name: String,
    pub record: Option<String>,
    pub kind: ActionKind,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Create,
    Get,
    Update,
    Delete,
    List,
    Search,
    LifecycleTransition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventPlan {
    pub name: String,
    pub record: Option<String>,
    pub kind: EventKind,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Created,
    Updated,
    Deleted,
    LifecycleTransition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionPlan {
    pub name: String,
    pub record: Option<String>,
    pub kind: ProjectionKind,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionKind {
    List,
    Detail,
    Search,
    Relationship,
    ByStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricPlan {
    pub name: String,
    pub record: Option<String>,
    pub kind: MetricKind,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    Count,
    CreatedPerDay,
    UpdatedPerDay,
    ByStatus,
    AverageTimeToState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyPlan {
    pub name: String,
    pub applies_to: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub name: String,
    pub compatibility: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointPlan {
    pub id: String,
    pub action: String,
    pub record: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    pub suggestion: Option<String>,
}

impl CompileDiagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            path: None,
            suggestion: None,
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            path: None,
            suggestion: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub message: String,
}

impl CompileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for CompileError {}

pub trait SorlaExpander {
    fn name(&self) -> &'static str;
    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError>;
}
