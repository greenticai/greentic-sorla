use greentic_sorla_lang::ast::{
    AccessRule, AgentEndpointApprovalMode, AgentEndpointRisk, CardinalityValue, CompatibilityMode,
    ConceptKind, ConfidenceScore, EndpointAuthorization, EventKind, FieldAuthority,
    MigrationOperationDecl, OperationalIndexKind, Package, ProjectionMode, ProviderRequirement,
    RecordAccess, RecordSource, ViewMode,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrVersion {
    pub major: u16,
    pub minor: u16,
}

impl IrVersion {
    pub const fn current() -> Self {
        Self { major: 0, minor: 1 }
    }

    pub const fn scaffold() -> Self {
        Self::current()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalIr {
    pub ir_version: IrVersion,
    pub package: PackageIr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ontology: Option<OntologyModelIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_bindings: Option<RetrievalBindingsIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operational_indexes: Option<OperationalIndexesIr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<RoleIr>,
    pub records: Vec<RecordIr>,
    pub events: Vec<EventIr>,
    pub actions: Vec<NamedItemIr>,
    pub policies: Vec<NamedItemIr>,
    pub approvals: Vec<NamedItemIr>,
    pub views: Vec<ViewIr>,
    pub flows: Vec<NamedItemIr>,
    pub projections: Vec<ProjectionIr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<MetricIr>,
    pub external_sources: Vec<ExternalSourceIr>,
    pub compatibility: Vec<CompatibilityIr>,
    pub provider_contract: ProviderContractIr,
    pub agent_endpoints: Vec<AgentEndpointIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyModelIr {
    pub schema: String,
    pub concepts: Vec<ConceptDefinitionIr>,
    pub relationships: Vec<RelationshipDefinitionIr>,
    pub constraints: Vec<OntologyConstraintIr>,
    pub semantic_aliases: SemanticAliasesIr,
    pub entity_linking: EntityLinkingIr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConceptKindIr {
    Abstract,
    Entity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptDefinitionIr {
    pub id: String,
    pub kind: ConceptKindIr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub extends: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backing: Option<OntologyBackingIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<OntologySensitivityIr>,
    pub policy_hooks: Vec<OntologyPolicyHookIr>,
    pub provider_requirements: Vec<ProviderRequirementIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipDefinitionIr {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardinality: Option<RelationshipCardinalityIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backing: Option<OntologyBackingIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<OntologySensitivityIr>,
    pub policy_hooks: Vec<OntologyPolicyHookIr>,
    pub provider_requirements: Vec<ProviderRequirementIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipCardinalityIr {
    pub from: CardinalityValueIr,
    pub to: CardinalityValueIr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CardinalityValueIr {
    One,
    Many,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyBackingIr {
    pub record: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyConstraintIr {
    pub id: String,
    pub applies_to: OntologyConstraintTargetIr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_policy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyConstraintTargetIr {
    pub concept: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologySensitivityIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<String>,
    pub pii: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyPolicyHookIr {
    pub policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SemanticAliasesIr {
    pub concepts: BTreeMap<String, Vec<String>>,
    pub relationships: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EntityLinkingIr {
    pub strategies: Vec<EntityLinkingStrategyIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityLinkingStrategyIr {
    pub id: String,
    pub applies_to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(rename = "match")]
    pub match_fields: EntityLinkingMatchIr,
    pub confidence: ConfidenceScore,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<OntologySensitivityIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityLinkingMatchIr {
    pub source_field: String,
    pub target_field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalBindingsIr {
    pub schema: String,
    pub providers: Vec<RetrievalProviderRequirementIr>,
    pub scopes: Vec<RetrievalScopeIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalIndexesIr {
    pub schema: String,
    pub indexes: Vec<OperationalIndexIr>,
    pub query_requirements: Vec<QueryRequirementIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalIndexIr {
    pub id: String,
    pub record: String,
    pub kind: OperationalIndexKindIr,
    pub fields: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unique: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationalIndexKindIr {
    Exact,
    Composite,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryRequirementIr {
    pub id: String,
    pub used_by: QueryRequirementTargetIr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_index: Option<String>,
    pub scan_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryRequirementTargetIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_endpoint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalProviderRequirementIr {
    pub id: String,
    pub category: String,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalScopeIr {
    pub id: String,
    pub applies_to: RetrievalScopeTargetIr,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<RetrievalFilterIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<RetrievalPermissionModeIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalScopeTargetIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concept: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalFilterIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_scope: Option<EntityScopeIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityScopeIr {
    pub include_self: bool,
    pub include_related: Vec<RelationshipTraversalRuleIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipTraversalRuleIr {
    pub relationship: String,
    pub direction: RelationshipTraversalDirectionIr,
    pub max_depth: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipTraversalDirectionIr {
    Incoming,
    Outgoing,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalPermissionModeIr {
    Inherit,
    PublicMetadataOnly,
    RequiresPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageIr {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleIr {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    pub source: RecordSourceIr,
    #[serde(default, skip_serializing_if = "RecordAccessIr::is_empty")]
    pub access: RecordAccessIr,
    pub fields: Vec<FieldIr>,
    pub external_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RecordAccessIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<AccessRuleIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create: Option<AccessRuleIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<AccessRuleIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<AccessRuleIr>,
}

impl RecordAccessIr {
    pub fn is_empty(&self) -> bool {
        self.read.is_none()
            && self.create.is_none()
            && self.update.is_none()
            && self.delete.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccessRuleIr {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecordSourceIr {
    Native,
    External,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    pub type_name: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    #[serde(default, skip_serializing_if = "FieldValidationRulesIr::is_empty")]
    pub rules: FieldValidationRulesIr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<FieldAuthorityIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<FieldReferenceIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FieldValidationRulesIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unique: bool,
}

impl FieldValidationRulesIr {
    pub fn is_empty(&self) -> bool {
        self.min.is_none()
            && self.max.is_none()
            && self.min_length.is_none()
            && self.max_length.is_none()
            && self.pattern.is_none()
            && self.precision.is_none()
            && self.scale.is_none()
            && self.before.is_none()
            && self.after.is_none()
            && !self.unique
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldReferenceIr {
    pub record: String,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldAuthorityIr {
    Local,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    pub record: String,
    pub kind: EventKindIr,
    pub emits: Vec<EventFieldIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKindIr {
    Domain,
    Integration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventFieldIr {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    pub record: String,
    pub source_event: String,
    pub mode: ProjectionModeIr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<MetricSourceIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measure: Option<MetricMeasureIr>,
    pub filters: Vec<MetricFilterIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<MetricTimeIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<MetricWindowIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub dimensions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<MetricTargetIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricSourceIr {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricMeasureIr {
    pub aggregate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricFilterIr {
    pub field: String,
    pub operator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricTimeIr {
    pub field: String,
    pub grain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricWindowIr {
    pub mode: String,
    pub size: u32,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricTargetIr {
    pub operator: String,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionModeIr {
    CurrentState,
    AuditTrail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalSourceIr {
    pub record: String,
    pub system: String,
    pub key: String,
    pub authoritative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityIr {
    pub name: String,
    pub compatibility: CompatibilityModeIr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_version: Option<String>,
    pub projection_updates: Vec<String>,
    pub backfills: Vec<MigrationBackfillIr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<MigrationOperationIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotence_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationBackfillIr {
    pub record: String,
    pub field: String,
    pub default: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MigrationOperationIr {
    AddRecord {
        record: String,
    },
    SplitRecord {
        from_record: String,
        into_records: Vec<String>,
    },
    RequireIndex {
        index: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityModeIr {
    Additive,
    BackwardCompatible,
    Breaking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedItemIr {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub mode: ViewModeIr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maps_from: Option<ViewMappingIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writes: Option<ViewWriteIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ViewModeIr {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewMappingIr {
    pub record: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewWriteIr {
    pub agent_endpoint: String,
    pub input_mapping: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderContractIr {
    pub categories: Vec<ProviderRequirementIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRequirementIr {
    pub category: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointIr {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    pub title: String,
    pub intent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub inputs: Vec<AgentEndpointInputIr>,
    pub outputs: Vec<AgentEndpointOutputIr>,
    pub side_effects: Vec<String>,
    pub risk: AgentEndpointRiskIr,
    pub approval: AgentEndpointApprovalModeIr,
    pub provider_requirements: Vec<ProviderRequirementIr>,
    pub backing: AgentEndpointBackingIr,
    #[serde(default, skip_serializing_if = "EndpointAuthorizationIr::is_empty")]
    pub authorization: EndpointAuthorizationIr,
    pub agent_visibility: AgentEndpointVisibilityIr,
    pub examples: Vec<AgentEndpointExampleIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emits: Option<AgentEndpointEmitIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EndpointAuthorizationIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<EndpointRoleRequirementIr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<serde_json::Value>,
}

impl EndpointAuthorizationIr {
    pub fn is_empty(&self) -> bool {
        self.roles.is_none() && self.policies.is_empty() && self.conditions.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EndpointRoleRequirementIr {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointInputIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    pub type_name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enum_values: Vec<String>,
    pub sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointOutputIr {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    pub type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointRiskIr {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEndpointApprovalModeIr {
    None,
    Optional,
    Required,
    PolicyDriven,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointBackingIr {
    pub actions: Vec<String>,
    pub events: Vec<String>,
    pub flows: Vec<String>,
    pub policies: Vec<String>,
    pub approvals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointVisibilityIr {
    pub openapi: bool,
    pub arazzo: bool,
    pub mcp: bool,
    pub llms_txt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointExampleIr {
    pub name: String,
    pub summary: String,
    pub input: serde_json::Value,
    pub expected_output: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointEmitIr {
    pub event: String,
    pub stream: String,
    pub payload: serde_json::Value,
}

pub fn lower_package(package: &Package) -> CanonicalIr {
    let mut roles: Vec<RoleIr> = package
        .roles
        .iter()
        .map(|role| {
            let mut grants = role.grants.clone();
            grants.sort();
            RoleIr {
                id: role.id.clone(),
                i18n_key: role.i18n_key.clone(),
                label: role.label.clone(),
                description: role.description.clone(),
                grants,
            }
        })
        .collect();
    roles.sort_by(|left, right| left.id.cmp(&right.id));

    let mut records: Vec<RecordIr> = package
        .records
        .iter()
        .map(|record| RecordIr {
            name: record.name.clone(),
            i18n_key: record.i18n_key.clone(),
            source: match record.source.clone().expect("source normalized by parser") {
                RecordSource::Native => RecordSourceIr::Native,
                RecordSource::External => RecordSourceIr::External,
                RecordSource::Hybrid => RecordSourceIr::Hybrid,
            },
            access: access_ir(&record.access),
            fields: sorted_fields(record),
            external_source: record
                .external_ref
                .as_ref()
                .map(|external_ref| external_ref.system.clone()),
        })
        .collect();
    records.sort_by(|left, right| left.name.cmp(&right.name));

    let mut events: Vec<EventIr> = package
        .events
        .iter()
        .map(|event| {
            let mut emits: Vec<EventFieldIr> = event
                .emits
                .iter()
                .map(|field| EventFieldIr {
                    name: field.name.clone(),
                    type_name: field.type_name.clone(),
                })
                .collect();
            emits.sort_by(|left, right| left.name.cmp(&right.name));

            EventIr {
                name: event.name.clone(),
                i18n_key: event.i18n_key.clone(),
                record: event.record.clone(),
                kind: match event.kind {
                    EventKind::Domain => EventKindIr::Domain,
                    EventKind::Integration => EventKindIr::Integration,
                },
                emits,
            }
        })
        .collect();
    events.sort_by(|left, right| left.name.cmp(&right.name));

    let mut projections: Vec<ProjectionIr> = package
        .projections
        .iter()
        .map(|projection| ProjectionIr {
            name: projection.name.clone(),
            i18n_key: projection.i18n_key.clone(),
            record: projection.record.clone(),
            source_event: projection.source_event.clone(),
            mode: match projection.mode {
                ProjectionMode::CurrentState => ProjectionModeIr::CurrentState,
                ProjectionMode::AuditTrail => ProjectionModeIr::AuditTrail,
            },
        })
        .collect();
    projections.sort_by(|left, right| left.name.cmp(&right.name));

    let mut metrics: Vec<MetricIr> = package
        .metrics
        .iter()
        .map(|metric| MetricIr {
            name: metric.name.clone(),
            i18n_key: metric.i18n_key.clone(),
            label: metric.label.clone(),
            description: metric.description.clone(),
            source: metric.source.as_ref().map(|source| MetricSourceIr {
                kind: source.kind.clone(),
                name: source.name.clone(),
            }),
            measure: metric.measure.as_ref().map(|measure| MetricMeasureIr {
                aggregate: measure.aggregate.clone(),
                field: measure.field.clone(),
            }),
            filters: metric
                .filters
                .iter()
                .map(|filter| MetricFilterIr {
                    field: filter.field.clone(),
                    operator: filter.operator.clone(),
                    value: filter.value.clone(),
                })
                .collect(),
            time: metric.time.as_ref().map(|time| MetricTimeIr {
                field: time.field.clone(),
                grain: time.grain.clone(),
            }),
            window: metric.window.as_ref().map(|window| MetricWindowIr {
                mode: window.mode.clone(),
                size: window.size,
                unit: window.unit.clone(),
            }),
            unit: metric.unit.clone(),
            dimensions: metric.dimensions.clone(),
            formula: metric.formula.clone(),
            depends_on: metric.depends_on.clone(),
            target: metric.target.as_ref().map(|target| MetricTargetIr {
                operator: target.operator.clone(),
                value: target.value.clone(),
                unit: target.unit.clone(),
            }),
        })
        .collect();
    metrics.sort_by(|left, right| left.name.cmp(&right.name));

    let mut external_sources: Vec<ExternalSourceIr> = package
        .records
        .iter()
        .filter_map(|record| {
            record
                .external_ref
                .as_ref()
                .map(|external_ref| ExternalSourceIr {
                    record: record.name.clone(),
                    system: external_ref.system.clone(),
                    key: external_ref.key.clone(),
                    authoritative: external_ref.authoritative,
                })
        })
        .collect();
    external_sources.sort_by(|left, right| left.record.cmp(&right.record));

    let mut compatibility: Vec<CompatibilityIr> = package
        .migrations
        .iter()
        .map(|migration| {
            let mut projection_updates = migration.projection_updates.clone();
            projection_updates.sort();
            let mut backfills: Vec<MigrationBackfillIr> = migration
                .backfills
                .iter()
                .map(|backfill| MigrationBackfillIr {
                    record: backfill.record.clone(),
                    field: backfill.field.clone(),
                    default: backfill.default.clone(),
                })
                .collect();
            backfills.sort_by(|left, right| {
                left.record
                    .cmp(&right.record)
                    .then(left.field.cmp(&right.field))
            });
            let mut operations = migration
                .operations
                .iter()
                .map(|operation| match operation {
                    MigrationOperationDecl::AddRecord { record } => {
                        MigrationOperationIr::AddRecord {
                            record: record.clone(),
                        }
                    }
                    MigrationOperationDecl::SplitRecord {
                        from_record,
                        into_records,
                    } => {
                        let mut into_records = into_records.clone();
                        into_records.sort();
                        MigrationOperationIr::SplitRecord {
                            from_record: from_record.clone(),
                            into_records,
                        }
                    }
                    MigrationOperationDecl::RequireIndex { index } => {
                        MigrationOperationIr::RequireIndex {
                            index: index.clone(),
                        }
                    }
                })
                .collect::<Vec<_>>();
            operations.sort_by(|left, right| {
                serde_json::to_string(left)
                    .expect("migration operation should serialize")
                    .cmp(
                        &serde_json::to_string(right)
                            .expect("migration operation should serialize"),
                    )
            });
            CompatibilityIr {
                name: migration.name.clone(),
                compatibility: match migration.compatibility {
                    CompatibilityMode::Additive => CompatibilityModeIr::Additive,
                    CompatibilityMode::BackwardCompatible => {
                        CompatibilityModeIr::BackwardCompatible
                    }
                    CompatibilityMode::Breaking => CompatibilityModeIr::Breaking,
                },
                from_version: migration.from_version.clone(),
                to_version: migration.to_version.clone(),
                projection_updates,
                backfills,
                operations,
                idempotence_key: migration.idempotence_key.clone(),
                notes: migration.notes.clone(),
            }
        })
        .collect();
    compatibility.sort_by(|left, right| left.name.cmp(&right.name));

    let provider_categories = sorted_provider_requirements(&package.provider_requirements);
    let agent_endpoints = sorted_agent_endpoints(package);

    CanonicalIr {
        ir_version: IrVersion::current(),
        package: PackageIr {
            name: package.package.name.clone(),
            version: package.package.version.clone(),
        },
        ontology: lower_ontology(package),
        retrieval_bindings: lower_retrieval_bindings(package),
        operational_indexes: lower_operational_indexes(package),
        roles,
        records,
        events,
        actions: sorted_named_items(
            &package
                .actions
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
        ),
        policies: sorted_named_items(
            &package
                .policies
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
        ),
        approvals: sorted_named_items(
            &package
                .approvals
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
        ),
        views: sorted_views(package),
        flows: sorted_named_items(
            &package
                .flows
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
        ),
        projections,
        metrics,
        external_sources,
        compatibility,
        provider_contract: ProviderContractIr {
            categories: provider_categories,
        },
        agent_endpoints,
    }
}

fn lower_retrieval_bindings(package: &Package) -> Option<RetrievalBindingsIr> {
    let bindings = package.retrieval_bindings.as_ref()?;
    let mut providers = bindings
        .providers
        .iter()
        .map(|provider| {
            let mut required_capabilities = provider.required_capabilities.clone();
            required_capabilities.sort();
            required_capabilities.dedup();
            RetrievalProviderRequirementIr {
                id: provider.id.clone(),
                category: provider.category.clone(),
                required_capabilities,
            }
        })
        .collect::<Vec<_>>();
    providers.sort_by(|left, right| left.id.cmp(&right.id));

    let mut scopes = bindings
        .scopes
        .iter()
        .map(|scope| RetrievalScopeIr {
            id: scope.id.clone(),
            applies_to: RetrievalScopeTargetIr {
                concept: scope.applies_to.concept.clone(),
                relationship: scope.applies_to.relationship.clone(),
            },
            provider: scope.provider.clone(),
            filters: scope.filters.as_ref().map(|filters| RetrievalFilterIr {
                entity_scope: filters.entity_scope.as_ref().map(|entity_scope| {
                    let mut include_related = entity_scope
                        .include_related
                        .iter()
                        .map(|rule| RelationshipTraversalRuleIr {
                            relationship: rule.relationship.clone(),
                            direction: match rule.direction {
                                greentic_sorla_lang::ast::RelationshipTraversalDirection::Incoming => {
                                    RelationshipTraversalDirectionIr::Incoming
                                }
                                greentic_sorla_lang::ast::RelationshipTraversalDirection::Outgoing => {
                                    RelationshipTraversalDirectionIr::Outgoing
                                }
                                greentic_sorla_lang::ast::RelationshipTraversalDirection::Both => {
                                    RelationshipTraversalDirectionIr::Both
                                }
                            },
                            max_depth: rule.max_depth,
                        })
                        .collect::<Vec<_>>();
                    include_related.sort_by(|left, right| {
                        left.relationship
                            .cmp(&right.relationship)
                            .then_with(|| left.max_depth.cmp(&right.max_depth))
                    });
                    EntityScopeIr {
                        include_self: entity_scope.include_self,
                        include_related,
                    }
                }),
            }),
            permission: scope.permission.as_ref().map(|permission| match permission {
                greentic_sorla_lang::ast::RetrievalPermissionMode::Inherit => {
                    RetrievalPermissionModeIr::Inherit
                }
                greentic_sorla_lang::ast::RetrievalPermissionMode::PublicMetadataOnly => {
                    RetrievalPermissionModeIr::PublicMetadataOnly
                }
                greentic_sorla_lang::ast::RetrievalPermissionMode::RequiresPolicy => {
                    RetrievalPermissionModeIr::RequiresPolicy
                }
            }),
        })
        .collect::<Vec<_>>();
    scopes.sort_by(|left, right| left.id.cmp(&right.id));

    Some(RetrievalBindingsIr {
        schema: bindings.schema.clone(),
        providers,
        scopes,
    })
}

fn sorted_views(package: &Package) -> Vec<ViewIr> {
    let mut views = package
        .views
        .iter()
        .map(|view| ViewIr {
            name: view.name.clone(),
            version: view.version.clone(),
            mode: match view.mode.as_ref().unwrap_or(&ViewMode::ReadOnly) {
                ViewMode::ReadOnly => ViewModeIr::ReadOnly,
                ViewMode::ReadWrite => ViewModeIr::ReadWrite,
            },
            maps_from: view.maps_from.as_ref().map(|mapping| ViewMappingIr {
                record: mapping.record.clone(),
                fields: mapping.fields.clone(),
            }),
            writes: view.writes.as_ref().map(|writes| ViewWriteIr {
                agent_endpoint: writes.agent_endpoint.clone(),
                input_mapping: writes.input_mapping.clone(),
            }),
        })
        .collect::<Vec<_>>();
    views.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.version.cmp(&right.version))
    });
    views
}

fn lower_operational_indexes(package: &Package) -> Option<OperationalIndexesIr> {
    let indexes = package.operational_indexes.as_ref()?;
    let mut lowered_indexes = indexes
        .indexes
        .iter()
        .map(|index| {
            let mut fields = index.fields.clone();
            fields.dedup();
            OperationalIndexIr {
                id: index.id.clone(),
                record: index.record.clone(),
                kind: match index.kind {
                    OperationalIndexKind::Exact => OperationalIndexKindIr::Exact,
                    OperationalIndexKind::Composite => OperationalIndexKindIr::Composite,
                    OperationalIndexKind::Text => OperationalIndexKindIr::Text,
                },
                fields,
                unique: index.unique,
            }
        })
        .collect::<Vec<_>>();
    lowered_indexes.sort_by(|left, right| left.id.cmp(&right.id));

    let mut query_requirements = indexes
        .query_requirements
        .iter()
        .map(|requirement| QueryRequirementIr {
            id: requirement.id.clone(),
            used_by: QueryRequirementTargetIr {
                projection: requirement.used_by.projection.clone(),
                view: requirement.used_by.view.clone(),
                agent_endpoint: requirement.used_by.agent_endpoint.clone(),
            },
            requires_index: requirement.requires_index.clone(),
            scan_ok: requirement.scan_ok,
        })
        .collect::<Vec<_>>();
    query_requirements.sort_by(|left, right| left.id.cmp(&right.id));

    Some(OperationalIndexesIr {
        schema: indexes.schema.clone(),
        indexes: lowered_indexes,
        query_requirements,
    })
}

fn lower_ontology(package: &Package) -> Option<OntologyModelIr> {
    let ontology = package.ontology.as_ref()?;

    let mut concepts: Vec<ConceptDefinitionIr> = ontology
        .concepts
        .iter()
        .map(|concept| {
            let mut extends = concept.extends.clone();
            extends.sort();
            extends.dedup();

            ConceptDefinitionIr {
                id: concept.id.clone(),
                kind: match concept.kind {
                    ConceptKind::Abstract => ConceptKindIr::Abstract,
                    ConceptKind::Entity => ConceptKindIr::Entity,
                },
                description: concept.description.clone(),
                extends,
                backing: concept.backed_by.as_ref().map(lower_ontology_backing),
                sensitivity: concept.sensitivity.as_ref().map(|sensitivity| {
                    OntologySensitivityIr {
                        classification: sensitivity.classification.clone(),
                        pii: sensitivity.pii,
                    }
                }),
                policy_hooks: sorted_ontology_policy_hooks(&concept.policy_hooks),
                provider_requirements: sorted_ontology_provider_requirements(
                    &concept.provider_requirements,
                ),
            }
        })
        .collect();
    concepts.sort_by(|left, right| left.id.cmp(&right.id));

    let mut relationships: Vec<RelationshipDefinitionIr> = ontology
        .relationships
        .iter()
        .map(|relationship| RelationshipDefinitionIr {
            id: relationship.id.clone(),
            label: relationship.label.clone(),
            from: relationship.from.clone(),
            to: relationship.to.clone(),
            cardinality: relationship.cardinality.as_ref().map(|cardinality| {
                RelationshipCardinalityIr {
                    from: lower_cardinality_value(&cardinality.from),
                    to: lower_cardinality_value(&cardinality.to),
                }
            }),
            backing: relationship.backed_by.as_ref().map(lower_ontology_backing),
            sensitivity: relationship.sensitivity.as_ref().map(|sensitivity| {
                OntologySensitivityIr {
                    classification: sensitivity.classification.clone(),
                    pii: sensitivity.pii,
                }
            }),
            policy_hooks: sorted_ontology_policy_hooks(&relationship.policy_hooks),
            provider_requirements: sorted_ontology_provider_requirements(
                &relationship.provider_requirements,
            ),
        })
        .collect();
    relationships.sort_by(|left, right| left.id.cmp(&right.id));

    let mut constraints: Vec<OntologyConstraintIr> = ontology
        .constraints
        .iter()
        .map(|constraint| OntologyConstraintIr {
            id: constraint.id.clone(),
            applies_to: OntologyConstraintTargetIr {
                concept: constraint.applies_to.concept.clone(),
            },
            requires_policy: constraint.requires_policy.clone(),
        })
        .collect();
    constraints.sort_by(|left, right| left.id.cmp(&right.id));

    Some(OntologyModelIr {
        schema: ontology.schema.clone(),
        concepts,
        relationships,
        constraints,
        semantic_aliases: lower_semantic_aliases(package),
        entity_linking: lower_entity_linking(package),
    })
}

fn lower_semantic_aliases(package: &Package) -> SemanticAliasesIr {
    let Some(aliases) = &package.semantic_aliases else {
        return SemanticAliasesIr::default();
    };
    SemanticAliasesIr {
        concepts: sorted_alias_map(&aliases.concepts),
        relationships: sorted_alias_map(&aliases.relationships),
    }
}

fn sorted_alias_map(aliases: &BTreeMap<String, Vec<String>>) -> BTreeMap<String, Vec<String>> {
    aliases
        .iter()
        .map(|(target, values)| {
            let mut values = values
                .iter()
                .map(|alias| normalize_semantic_alias(alias))
                .collect::<Vec<_>>();
            values.sort();
            values.dedup();
            (target.clone(), values)
        })
        .collect()
}

fn normalize_semantic_alias(alias: &str) -> String {
    alias
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn lower_entity_linking(package: &Package) -> EntityLinkingIr {
    let Some(entity_linking) = &package.entity_linking else {
        return EntityLinkingIr::default();
    };
    let mut strategies = entity_linking
        .strategies
        .iter()
        .map(|strategy| EntityLinkingStrategyIr {
            id: strategy.id.clone(),
            applies_to: strategy.applies_to.clone(),
            source_type: strategy.source_type.clone(),
            match_fields: EntityLinkingMatchIr {
                source_field: strategy.match_fields.source_field.clone(),
                target_field: strategy.match_fields.target_field.clone(),
            },
            confidence: strategy.confidence,
            sensitivity: strategy
                .sensitivity
                .as_ref()
                .map(|sensitivity| OntologySensitivityIr {
                    classification: sensitivity.classification.clone(),
                    pii: sensitivity.pii,
                }),
        })
        .collect::<Vec<_>>();
    strategies.sort_by(|left, right| left.id.cmp(&right.id));
    EntityLinkingIr { strategies }
}

fn lower_ontology_backing(
    backing: &greentic_sorla_lang::ast::OntologyBacking,
) -> OntologyBackingIr {
    OntologyBackingIr {
        record: backing.record.clone(),
        from_field: backing.from_field.clone(),
        to_field: backing.to_field.clone(),
    }
}

fn lower_cardinality_value(value: &CardinalityValue) -> CardinalityValueIr {
    match value {
        CardinalityValue::One => CardinalityValueIr::One,
        CardinalityValue::Many => CardinalityValueIr::Many,
    }
}

fn sorted_ontology_policy_hooks(
    hooks: &[greentic_sorla_lang::ast::OntologyPolicyHook],
) -> Vec<OntologyPolicyHookIr> {
    let mut hooks: Vec<OntologyPolicyHookIr> = hooks
        .iter()
        .map(|hook| OntologyPolicyHookIr {
            policy: hook.policy.clone(),
            reason: hook.reason.clone(),
        })
        .collect();
    hooks.sort_by(|left, right| left.policy.cmp(&right.policy));
    hooks
}

fn sorted_ontology_provider_requirements(
    requirements: &[greentic_sorla_lang::ast::OntologyProviderRequirement],
) -> Vec<ProviderRequirementIr> {
    let mut requirements: Vec<ProviderRequirementIr> = requirements
        .iter()
        .map(|requirement| {
            let mut capabilities = requirement.capabilities.clone();
            capabilities.sort();
            capabilities.dedup();
            ProviderRequirementIr {
                category: requirement.category.clone(),
                capabilities,
            }
        })
        .collect();
    requirements.sort_by(|left, right| left.category.cmp(&right.category));
    requirements
}

fn sorted_fields(record: &greentic_sorla_lang::ast::Record) -> Vec<FieldIr> {
    let mut fields: Vec<FieldIr> = record
        .fields
        .iter()
        .map(|field| FieldIr {
            name: field.name.clone(),
            i18n_key: field.i18n_key.clone(),
            type_name: field.type_name.clone(),
            required: field.required,
            sensitive: field.sensitive,
            enum_values: field.enum_values.clone(),
            rules: FieldValidationRulesIr {
                min: field.rules.min.clone(),
                max: field.rules.max.clone(),
                min_length: field.rules.min_length,
                max_length: field.rules.max_length,
                pattern: field.rules.pattern.clone(),
                precision: field.rules.precision,
                scale: field.rules.scale,
                before: field.rules.before.clone(),
                after: field.rules.after.clone(),
                unique: field.rules.unique,
            },
            authority: field.authority.as_ref().map(|authority| match authority {
                FieldAuthority::Local => FieldAuthorityIr::Local,
                FieldAuthority::External => FieldAuthorityIr::External,
            }),
            references: field.references.as_ref().map(|reference| FieldReferenceIr {
                record: reference.record.clone(),
                field: reference.field.clone(),
            }),
        })
        .collect();
    fields.sort_by(|left, right| left.name.cmp(&right.name));
    fields
}

fn access_ir(access: &RecordAccess) -> RecordAccessIr {
    RecordAccessIr {
        read: access.read.as_ref().map(access_rule_ir),
        create: access.create.as_ref().map(access_rule_ir),
        update: access.update.as_ref().map(access_rule_ir),
        delete: access.delete.as_ref().map(access_rule_ir),
    }
}

fn access_rule_ir(rule: &AccessRule) -> AccessRuleIr {
    AccessRuleIr {
        roles: sorted_strings(&rule.roles),
        policies: sorted_strings(&rule.policies),
    }
}

fn authorization_ir(authorization: &EndpointAuthorization) -> EndpointAuthorizationIr {
    EndpointAuthorizationIr {
        roles: authorization
            .roles
            .as_ref()
            .map(|roles| EndpointRoleRequirementIr {
                any_of: sorted_strings(&roles.any_of),
                all_of: sorted_strings(&roles.all_of),
            }),
        policies: sorted_strings(&authorization.policies),
        conditions: authorization.conditions.clone(),
    }
}

fn sorted_named_items(names: &[&str]) -> Vec<NamedItemIr> {
    let mut items: Vec<NamedItemIr> = names
        .iter()
        .map(|name| NamedItemIr {
            name: (*name).to_string(),
        })
        .collect();
    items.sort_by(|left, right| left.name.cmp(&right.name));
    items
}

fn sorted_provider_requirements(
    requirements: &[ProviderRequirement],
) -> Vec<ProviderRequirementIr> {
    let mut provider_categories: Vec<ProviderRequirementIr> = requirements
        .iter()
        .map(|requirement| {
            let mut capabilities = requirement.capabilities.clone();
            capabilities.sort();
            ProviderRequirementIr {
                category: requirement.category.clone(),
                capabilities,
            }
        })
        .collect();
    provider_categories.sort_by(|left, right| left.category.cmp(&right.category));
    provider_categories
}

fn sorted_agent_endpoints(package: &Package) -> Vec<AgentEndpointIr> {
    let mut endpoints: Vec<AgentEndpointIr> = package
        .agent_endpoints
        .iter()
        .map(|endpoint| {
            let mut inputs: Vec<AgentEndpointInputIr> = endpoint
                .inputs
                .iter()
                .map(|input| {
                    let mut enum_values = input.enum_values.clone();
                    enum_values.sort();
                    AgentEndpointInputIr {
                        name: input.name.clone(),
                        i18n_key: input.i18n_key.clone(),
                        type_name: input.type_name.clone(),
                        required: input.required,
                        description: input.description.clone(),
                        enum_values,
                        sensitive: input.sensitive,
                    }
                })
                .collect();
            inputs.sort_by(|left, right| left.name.cmp(&right.name));

            let mut outputs: Vec<AgentEndpointOutputIr> = endpoint
                .outputs
                .iter()
                .map(|output| AgentEndpointOutputIr {
                    name: output.name.clone(),
                    i18n_key: output.i18n_key.clone(),
                    type_name: output.type_name.clone(),
                    description: output.description.clone(),
                })
                .collect();
            outputs.sort_by(|left, right| left.name.cmp(&right.name));

            let mut side_effects = endpoint.side_effects.clone();
            side_effects.sort();

            let mut examples: Vec<AgentEndpointExampleIr> = endpoint
                .examples
                .iter()
                .map(|example| AgentEndpointExampleIr {
                    name: example.name.clone(),
                    summary: example.summary.clone(),
                    input: example.input.clone(),
                    expected_output: example.expected_output.clone(),
                })
                .collect();
            examples.sort_by(|left, right| left.name.cmp(&right.name));

            AgentEndpointIr {
                id: endpoint.id.clone(),
                i18n_key: endpoint.i18n_key.clone(),
                title: endpoint.title.clone(),
                intent: endpoint.intent.clone(),
                description: endpoint.description.clone(),
                inputs,
                outputs,
                side_effects,
                risk: match endpoint.risk {
                    AgentEndpointRisk::Low => AgentEndpointRiskIr::Low,
                    AgentEndpointRisk::Medium => AgentEndpointRiskIr::Medium,
                    AgentEndpointRisk::High => AgentEndpointRiskIr::High,
                },
                approval: match endpoint.approval {
                    AgentEndpointApprovalMode::None => AgentEndpointApprovalModeIr::None,
                    AgentEndpointApprovalMode::Optional => AgentEndpointApprovalModeIr::Optional,
                    AgentEndpointApprovalMode::Required => AgentEndpointApprovalModeIr::Required,
                    AgentEndpointApprovalMode::PolicyDriven => {
                        AgentEndpointApprovalModeIr::PolicyDriven
                    }
                },
                provider_requirements: sorted_provider_requirements(
                    &endpoint.provider_requirements,
                ),
                backing: AgentEndpointBackingIr {
                    actions: sorted_strings(&endpoint.backing.actions),
                    events: sorted_strings(&endpoint.backing.events),
                    flows: sorted_strings(&endpoint.backing.flows),
                    policies: sorted_strings(&endpoint.backing.policies),
                    approvals: sorted_strings(&endpoint.backing.approvals),
                },
                authorization: authorization_ir(&endpoint.authorization),
                agent_visibility: AgentEndpointVisibilityIr {
                    openapi: endpoint.agent_visibility.openapi,
                    arazzo: endpoint.agent_visibility.arazzo,
                    mcp: endpoint.agent_visibility.mcp,
                    llms_txt: endpoint.agent_visibility.llms_txt,
                },
                examples,
                emits: endpoint.emits.as_ref().map(|emit| AgentEndpointEmitIr {
                    event: emit.event.clone(),
                    stream: emit.stream.clone(),
                    payload: emit.payload.clone(),
                }),
                execution: endpoint.execution.clone(),
            }
        })
        .collect();
    endpoints.sort_by(|left, right| left.id.cmp(&right.id));
    endpoints
}

fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub fn inspect_ir(ir: &CanonicalIr) -> String {
    serde_json::to_string_pretty(ir).expect("IR inspect representation should serialize")
}

pub fn canonical_cbor<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(value, &mut bytes)
        .expect("canonical cbor serialization should work");
    bytes
}

pub fn canonical_hash_hex<T: Serialize>(value: &T) -> String {
    let bytes = canonical_cbor(value);
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn agent_tools_json(ir: &CanonicalIr) -> String {
    let mut tools = BTreeMap::new();
    tools.insert("package".to_string(), ir.package.name.clone());
    tools.insert(
        "storage-provider-categories".to_string(),
        ir.provider_contract
            .categories
            .iter()
            .map(|requirement| requirement.category.clone())
            .collect::<Vec<_>>()
            .join(","),
    );
    tools.insert(
        "agent-endpoints".to_string(),
        ir.agent_endpoints
            .iter()
            .map(|endpoint| endpoint.id.clone())
            .collect::<Vec<_>>()
            .join(","),
    );
    serde_json::to_string_pretty(&tools).expect("agent tools json should serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use greentic_sorla_lang::parser::parse_package;

    #[test]
    fn scaffold_ir_version_is_stable() {
        assert_eq!(IrVersion::current(), IrVersion { major: 0, minor: 1 });
    }

    #[test]
    fn lowering_is_deterministic_and_keeps_external_refs() {
        let parsed = parse_package(
            r#"
package:
  name: tenancy
  version: 0.2.0
records:
  - name: Tenant
    source: hybrid
    external_ref:
      system: crm
      key: tenant_id
      authoritative: true
    fields:
      - name: approval_state
        type: string
        authority: local
      - name: tenant_id
        type: string
        authority: external
events:
  - name: TenantApprovalRequested
    record: Tenant
    emits:
      - name: approval_state
        type: string
projections:
  - name: TenantCurrentState
    record: Tenant
    source_event: TenantApprovalRequested
provider_requirements:
  - category: storage
    capabilities:
      - projections
      - event-log
migrations:
  - name: tenant-projection-v2
    compatibility: additive
    projection_updates:
      - TenantCurrentState
"#,
        )
        .expect("fixture should parse");

        let first = lower_package(&parsed.package);
        let second = lower_package(&parsed.package);
        assert_eq!(first, second);
        assert_eq!(first.external_sources.len(), 1);
        assert_eq!(first.projections.len(), 1);
        assert_eq!(first.provider_contract.categories.len(), 1);
        assert!(first.agent_endpoints.is_empty());
        assert_eq!(canonical_hash_hex(&first), canonical_hash_hex(&second));
    }

    #[test]
    fn lowers_agent_endpoints_deterministically() {
        let parsed = parse_package(
            r#"
package:
  name: website-lead-capture
  version: 0.2.0
records:
  - name: Contact
    fields:
      - name: id
        type: string
actions:
  - name: SendConfirmation
  - name: UpsertContact
events:
  - name: ContactCaptured
    record: Contact
policies:
  - name: CustomerContactPolicy
approvals:
  - name: ReviewHighRiskContact
agent_endpoints:
  - id: create_partner_contact
    title: Create partner contact
    intent: Capture a partner enquiry.
    outputs:
      - name: contact_id
        type: string
    examples:
      - name: partner
        summary: Partner lead.
  - id: create_customer_contact
    title: Create customer contact
    intent: Capture a customer enquiry and create or update the CRM contact.
    inputs:
      - name: problem_to_solve
        type: string
        required: true
      - name: email
        type: string
        required: true
        sensitive: true
        enum_values:
          - z@example.com
          - a@example.com
    outputs:
      - name: status
        type: string
      - name: contact_id
        type: string
    side_effects:
      - event.ContactCaptured
      - crm.contact.upsert
    risk: medium
    approval: policy-driven
    provider_requirements:
      - category: crm
        capabilities:
          - contacts.write
          - contacts.read
    backing:
      actions:
        - UpsertContact
        - SendConfirmation
      events:
        - ContactCaptured
      policies:
        - CustomerContactPolicy
      approvals:
        - ReviewHighRiskContact
    examples:
      - name: z-example
        summary: Later example.
      - name: a-example
        summary: Earlier example.
"#,
        )
        .expect("agent endpoint fixture should parse");

        let first = lower_package(&parsed.package);
        let second = lower_package(&parsed.package);
        assert_eq!(first, second);
        assert_eq!(canonical_hash_hex(&first), canonical_hash_hex(&second));

        assert_eq!(first.agent_endpoints.len(), 2);
        assert_eq!(first.agent_endpoints[0].id, "create_customer_contact");
        assert_eq!(first.agent_endpoints[1].id, "create_partner_contact");

        let customer = &first.agent_endpoints[0];
        assert_eq!(customer.inputs[0].name, "email");
        assert_eq!(
            customer.inputs[0].enum_values,
            ["a@example.com", "z@example.com"]
        );
        assert_eq!(customer.inputs[1].name, "problem_to_solve");
        assert_eq!(customer.outputs[0].name, "contact_id");
        assert_eq!(customer.outputs[1].name, "status");
        assert_eq!(
            customer.side_effects,
            ["crm.contact.upsert", "event.ContactCaptured"]
        );
        assert_eq!(
            customer.provider_requirements[0].capabilities,
            ["contacts.read", "contacts.write"]
        );
        assert_eq!(
            customer.backing.actions,
            ["SendConfirmation", "UpsertContact"]
        );
        assert_eq!(customer.examples[0].name, "a-example");
        assert_eq!(customer.examples[1].name, "z-example");

        let tools = agent_tools_json(&first);
        assert!(
            tools.contains(
                "\"agent-endpoints\": \"create_customer_contact,create_partner_contact\""
            )
        );
    }

    #[test]
    fn lowers_executable_contract_fields() {
        let parsed = parse_package(
            r#"
package:
  name: leasing
  version: 0.3.0
records:
  - name: Landlord
    fields:
      - name: id
        type: string
  - name: Tenant
    fields:
      - name: id
        type: string
      - name: landlord_id
        type: string
        references:
          record: Landlord
          field: id
      - name: preferred_contact_method
        type: string
events:
  - name: TenantCreated
    record: Tenant
projections:
  - name: TenantCurrentState
    record: Tenant
    source_event: TenantCreated
migrations:
  - name: tenant-v2
    compatibility: additive
    idempotence_key: tenant-v2-fields
    backfills:
      - record: Tenant
        field: preferred_contact_method
        default: email
    projection_updates:
      - TenantCurrentState
agent_endpoints:
  - id: create_tenant
    title: Create tenant
    intent: Create a tenant and emit the event contract.
    inputs:
      - name: full_name
        type: string
    emits:
      event: TenantCreated
      stream: "tenant/{tenant_id}"
      payload:
        full_name: "$input.full_name"
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let tenant = ir
            .records
            .iter()
            .find(|record| record.name == "Tenant")
            .expect("tenant record should lower");
        let reference = tenant.fields[1]
            .references
            .as_ref()
            .expect("relationship should lower");
        assert_eq!(reference.record, "Landlord");
        assert_eq!(reference.field, "id");

        let migration = &ir.compatibility[0];
        assert_eq!(
            migration.idempotence_key.as_deref(),
            Some("tenant-v2-fields")
        );
        assert_eq!(migration.backfills[0].default, serde_json::json!("email"));

        let emit = ir.agent_endpoints[0]
            .emits
            .as_ref()
            .expect("operation emit should lower");
        assert_eq!(emit.event, "TenantCreated");
        assert_eq!(emit.payload["full_name"], "$input.full_name");
    }

    #[test]
    fn lowers_versioned_views_deterministically() {
        let parsed = parse_package(
            r#"
package:
  name: leasing
  version: 0.2.0
records:
  - name: Tenant
    fields:
      - name: id
        type: string
      - name: full_name
        type: string
events:
  - name: TenantUpdated
    record: Tenant
agent_endpoints:
  - id: update_tenant
    title: Update tenant
    intent: Update a tenant from view input.
    inputs:
      - name: tenant_id
        type: string
      - name: full_name
        type: string
    emits:
      event: TenantUpdated
      stream: "tenant/{tenant_id}"
views:
  - name: TenantWrite
    version: 2.0.0
    mode: read-write
    maps_from:
      record: Tenant
      fields:
        display_name: full_name
        tenant_id: id
    writes:
      agent_endpoint: update_tenant
      input_mapping:
        full_name: display_name
        tenant_id: tenant_id
  - name: LegacyTenant
"#,
        )
        .expect("versioned view fixture should parse");

        let first = lower_package(&parsed.package);
        let second = lower_package(&parsed.package);
        assert_eq!(first.views, second.views);
        assert_eq!(first.views[0].name, "LegacyTenant");
        assert_eq!(first.views[0].mode, ViewModeIr::ReadOnly);
        assert_eq!(first.views[1].name, "TenantWrite");
        assert_eq!(first.views[1].version.as_deref(), Some("2.0.0"));
        assert_eq!(first.views[1].mode, ViewModeIr::ReadWrite);
        assert_eq!(
            first.views[1]
                .maps_from
                .as_ref()
                .expect("view mapping should lower")
                .fields["display_name"],
            "full_name"
        );
    }

    #[test]
    fn lowers_operational_indexes_deterministically() {
        let parsed = parse_package(
            r#"
package:
  name: leasing
  version: 0.2.0
records:
  - name: Tenant
    fields:
      - name: id
        type: string
      - name: email
        type: string
      - name: status
        type: string
events:
  - name: TenantCreated
    record: Tenant
projections:
  - name: ActiveTenants
    record: Tenant
    source_event: TenantCreated
operational_indexes:
  schema: greentic.sorla.operational-indexes.v1
  indexes:
    - id: tenant_status_lookup
      record: Tenant
      kind: composite
      fields:
        - status
        - id
    - id: tenant_by_email
      record: Tenant
      kind: exact
      unique: true
      fields:
        - email
  query_requirements:
    - id: active_tenant_lookup
      used_by:
        projection: ActiveTenants
      requires_index: tenant_status_lookup
"#,
        )
        .expect("operational indexes should parse");

        let first = lower_package(&parsed.package);
        let second = lower_package(&parsed.package);
        assert_eq!(first.operational_indexes, second.operational_indexes);
        let indexes = first
            .operational_indexes
            .expect("operational indexes should lower");
        assert_eq!(indexes.indexes[0].id, "tenant_by_email");
        assert!(indexes.indexes[0].unique);
        assert_eq!(indexes.indexes[1].id, "tenant_status_lookup");
        assert!(!indexes.indexes[1].unique);
        assert_eq!(
            indexes.query_requirements[0].requires_index.as_deref(),
            Some("tenant_status_lookup")
        );
    }

    #[test]
    fn lowers_typed_migration_operations() {
        let parsed = parse_package(
            r#"
package:
  name: leasing
  version: 0.2.0
records:
  - name: Tenant
    fields:
      - name: id
        type: string
      - name: property_id
        type: string
      - name: status
        type: string
  - name: Person
    fields:
      - name: id
        type: string
  - name: Tenancy
    fields:
      - name: id
        type: string
operational_indexes:
  schema: greentic.sorla.operational-indexes.v1
  indexes:
    - id: active_tenants_by_property
      record: Tenant
      kind: composite
      fields:
        - property_id
        - status
migrations:
  - name: tenant-v2
    compatibility: backward-compatible
    from_version: 1.1.0
    to_version: 2.0.0
    idempotence_key: tenant:1.1.0:2.0.0
    operations:
      - kind: split-record
        from_record: Tenant
        into_records:
          - Tenancy
          - Person
      - kind: require-index
        index: active_tenants_by_property
"#,
        )
        .expect("typed migration operations should parse");

        let ir = lower_package(&parsed.package);
        let migration = &ir.compatibility[0];
        assert_eq!(migration.from_version.as_deref(), Some("1.1.0"));
        assert_eq!(migration.to_version.as_deref(), Some("2.0.0"));
        assert_eq!(migration.operations.len(), 2);
        assert!(migration.operations.iter().any(|operation| matches!(
            operation,
            MigrationOperationIr::RequireIndex { index } if index == "active_tenants_by_property"
        )));
        assert!(migration.operations.iter().any(|operation| matches!(
            operation,
            MigrationOperationIr::SplitRecord { into_records, .. } if into_records == &vec!["Person".to_string(), "Tenancy".to_string()]
        )));
    }

    #[test]
    fn lowers_ontology_deterministically() {
        let parsed = parse_package(
            r#"
package:
  name: ontology-demo
  version: 0.1.0
records:
  - name: Customer
    fields:
      - name: id
        type: string
      - name: email
        type: string
  - name: Contract
    fields:
      - name: id
        type: string
  - name: CustomerContract
    fields:
      - name: customer_id
        type: string
      - name: contract_id
        type: string
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
      extends:
        - Party
      backed_by:
        record: Customer
      sensitivity:
        classification: confidential
        pii: true
    - id: Contract
      kind: entity
      backed_by:
        record: Contract
    - id: Party
      kind: abstract
  relationships:
    - id: has_contract
      from: Customer
      to: Contract
      cardinality:
        from: one
        to: many
      backed_by:
        record: CustomerContract
        from_field: customer_id
        to_field: contract_id
    - id: governed_by
      from: Contract
      to: Customer
  constraints:
    - id: customer_policy
      applies_to:
        concept: Customer
semantic_aliases:
  concepts:
    Customer:
      - Client
      - account holder
      - client
    Contract:
      - subscription
      - agreement
  relationships:
    has_contract:
      - covered by
      - agreement link
entity_linking:
  strategies:
    - id: email_match
      applies_to: Customer
      match:
        source_field: email
        target_field: email
      confidence: 0.95
      sensitivity:
        pii: true
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: primary_evidence
      category: evidence
      required_capabilities:
        - evidence.query
        - entity.link
  scopes:
    - id: customer_evidence
      applies_to:
        concept: Customer
      provider: primary_evidence
      filters:
        entity_scope:
          include_self: true
          include_related:
            - relationship: has_contract
              direction: outgoing
              max_depth: 1
"#,
        )
        .expect("ontology fixture should parse");

        let first = lower_package(&parsed.package);
        let second = lower_package(&parsed.package);
        assert_eq!(first, second);
        assert_eq!(canonical_hash_hex(&first), canonical_hash_hex(&second));

        let ontology = first.ontology.expect("ontology should lower");
        assert_eq!(ontology.concepts[0].id, "Contract");
        assert_eq!(ontology.concepts[1].id, "Customer");
        assert_eq!(ontology.concepts[1].extends, ["Party"]);
        assert_eq!(ontology.relationships[0].id, "governed_by");
        assert_eq!(ontology.relationships[1].id, "has_contract");
        assert_eq!(
            ontology.semantic_aliases.concepts["Customer"],
            ["account holder", "client"]
        );
        assert_eq!(
            ontology.semantic_aliases.relationships["has_contract"],
            ["agreement link", "covered by"]
        );
        assert_eq!(ontology.entity_linking.strategies[0].id, "email_match");
        assert_eq!(ontology.entity_linking.strategies[0].confidence.0, 950_000);
        let retrieval = first
            .retrieval_bindings
            .expect("retrieval bindings should lower");
        assert_eq!(retrieval.providers[0].id, "primary_evidence");
        assert_eq!(
            retrieval.providers[0].required_capabilities,
            ["entity.link", "evidence.query"]
        );
        assert_eq!(retrieval.scopes[0].id, "customer_evidence");
    }
}
