use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

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
    pub ontology: Option<OntologyModel>,
    #[serde(default)]
    pub semantic_aliases: Option<SemanticAliases>,
    #[serde(default)]
    pub entity_linking: Option<EntityLinking>,
    #[serde(default)]
    pub retrieval_bindings: Option<RetrievalBindings>,
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
pub struct OntologyModel {
    pub schema: String,
    #[serde(default)]
    pub concepts: Vec<ConceptDefinition>,
    #[serde(default)]
    pub relationships: Vec<RelationshipDefinition>,
    #[serde(default)]
    pub constraints: Vec<OntologyConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConceptId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConceptKind {
    Abstract,
    Entity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConceptDefinition {
    pub id: String,
    pub kind: ConceptKind,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_vec")]
    pub extends: Vec<String>,
    #[serde(default)]
    pub backed_by: Option<OntologyBacking>,
    #[serde(default)]
    pub sensitivity: Option<OntologySensitivity>,
    #[serde(default)]
    pub policy_hooks: Vec<OntologyPolicyHook>,
    #[serde(default)]
    pub provider_requirements: Vec<OntologyProviderRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipDefinition {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub cardinality: Option<RelationshipCardinality>,
    #[serde(default)]
    pub backed_by: Option<OntologyBacking>,
    #[serde(default)]
    pub sensitivity: Option<OntologySensitivity>,
    #[serde(default)]
    pub policy_hooks: Vec<OntologyPolicyHook>,
    #[serde(default)]
    pub provider_requirements: Vec<OntologyProviderRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipCardinality {
    pub from: CardinalityValue,
    pub to: CardinalityValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CardinalityValue {
    One,
    Many,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyBacking {
    pub record: String,
    #[serde(default)]
    pub from_field: Option<String>,
    #[serde(default)]
    pub to_field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyConstraint {
    pub id: String,
    pub applies_to: OntologyConstraintTarget,
    #[serde(default)]
    pub requires_policy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyConstraintTarget {
    pub concept: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologySensitivity {
    #[serde(default)]
    pub classification: Option<String>,
    #[serde(default)]
    pub pii: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyPolicyHook {
    pub policy: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyProviderRequirement {
    pub category: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SemanticAliases {
    #[serde(default)]
    pub concepts: std::collections::BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub relationships: std::collections::BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EntityLinking {
    #[serde(default)]
    pub strategies: Vec<EntityLinkingStrategy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityLinkingStrategy {
    pub id: String,
    pub applies_to: String,
    #[serde(default)]
    pub source_type: Option<String>,
    #[serde(rename = "match")]
    pub match_fields: EntityLinkingMatch,
    pub confidence: ConfidenceScore,
    #[serde(default)]
    pub sensitivity: Option<OntologySensitivity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityLinkingMatch {
    pub source_field: String,
    pub target_field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalBindings {
    pub schema: String,
    #[serde(default)]
    pub providers: Vec<RetrievalProviderRequirement>,
    #[serde(default)]
    pub scopes: Vec<RetrievalScope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalProviderRequirement {
    pub id: String,
    pub category: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalScope {
    pub id: String,
    pub applies_to: RetrievalScopeTarget,
    pub provider: String,
    #[serde(default)]
    pub filters: Option<RetrievalFilter>,
    #[serde(default)]
    pub permission: Option<RetrievalPermissionMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalScopeTarget {
    #[serde(default)]
    pub concept: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalFilter {
    #[serde(default)]
    pub entity_scope: Option<EntityScope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EntityScope {
    #[serde(default)]
    pub include_self: bool,
    #[serde(default)]
    pub include_related: Vec<RelationshipTraversalRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipTraversalRule {
    pub relationship: String,
    pub direction: RelationshipTraversalDirection,
    pub max_depth: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipTraversalDirection {
    Incoming,
    Outgoing,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalPermissionMode {
    Inherit,
    PublicMetadataOnly,
    RequiresPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConfidenceScore(pub u32);

impl ConfidenceScore {
    pub const SCALE: u32 = 1_000_000;

    pub fn as_f64(self) -> f64 {
        f64::from(self.0) / f64::from(Self::SCALE)
    }
}

impl Serialize for ConfidenceScore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.as_f64())
    }
}

impl<'de> Deserialize<'de> for ConfidenceScore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = ConfidenceScore;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a confidence number between 0.0 and 1.0")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                confidence_from_f64(value).map_err(E::custom)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                confidence_from_f64(value as f64).map_err(E::custom)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                confidence_from_f64(value as f64).map_err(E::custom)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

fn confidence_from_f64(value: f64) -> Result<ConfidenceScore, String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err("confidence must be between 0.0 and 1.0".to_string());
    }
    Ok(ConfidenceScore(
        (value * f64::from(ConfidenceScore::SCALE)).round() as u32,
    ))
}

fn deserialize_optional_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    Ok(match Option::<StringOrVec>::deserialize(deserializer)? {
        Some(StringOrVec::String(value)) => vec![value],
        Some(StringOrVec::Vec(values)) => values,
        None => Vec::new(),
    })
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
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
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

fn is_false(value: &bool) -> bool {
    !*value
}
