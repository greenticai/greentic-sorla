use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SorDesignDraft {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub records: Vec<DraftRecord>,
    #[serde(default)]
    pub relationships: Vec<DraftRelationship>,
    #[serde(default)]
    pub actions: Vec<DraftAction>,
    #[serde(default)]
    pub events: Vec<DraftEvent>,
    #[serde(default)]
    pub projections: Vec<DraftProjection>,
    #[serde(default)]
    pub metrics: Vec<DraftMetric>,
    #[serde(default)]
    pub policies: Vec<DraftPolicy>,
    #[serde(default)]
    pub approvals: Vec<DraftApproval>,
    #[serde(default)]
    pub migrations: Vec<DraftMigration>,
    #[serde(default)]
    pub provider_requirements: Vec<DraftProviderRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftRecord {
    #[serde(default = "default_record_name")]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub fields: Vec<DraftField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DraftField {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    pub sensitive: bool,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<serde_json::Value>,
}

impl<'de> Deserialize<'de> for DraftField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawDraftField {
            #[serde(default = "default_field_name")]
            name: String,
            #[serde(default)]
            type_name: Option<String>,
            #[serde(default, rename = "type")]
            field_type: Option<String>,
            #[serde(default = "default_true")]
            required: bool,
            #[serde(default)]
            sensitive: bool,
            #[serde(default)]
            description: Option<String>,
            #[serde(default)]
            rules: Option<serde_json::Value>,
        }

        let raw = RawDraftField::deserialize(deserializer)?;
        Ok(Self {
            name: raw.name,
            type_name: raw
                .type_name
                .or(raw.field_type)
                .unwrap_or_else(default_field_type),
            required: raw.required,
            sensitive: raw.sensitive,
            description: raw.description,
            rules: raw.rules,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftRelationship {
    #[serde(default = "default_relationship_name")]
    pub name: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftAction {
    #[serde(default = "default_action_name")]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub risk: DraftRisk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftEvent {
    #[serde(default = "default_event_name")]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftProjection {
    #[serde(default = "default_projection_name")]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftMetric {
    #[serde(default = "default_metric_name")]
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source_record: Option<String>,
    #[serde(default)]
    pub aggregate: Option<String>,
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub time_field: Option<String>,
    #[serde(default)]
    pub grain: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub dimensions: Vec<String>,
    #[serde(default)]
    pub formula: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub filters: Vec<DraftMetricFilter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftMetricFilter {
    #[serde(default = "default_field_name")]
    pub field: String,
    #[serde(default = "default_filter_operator")]
    pub operator: String,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftPolicy {
    #[serde(default = "default_policy_name")]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftApproval {
    #[serde(default = "default_approval_name")]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftMigration {
    #[serde(default = "default_migration_name")]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftProviderRequirement {
    #[serde(default = "default_provider_category")]
    pub category: String,
    #[serde(default)]
    pub capability: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DraftRisk {
    Low,
    #[default]
    Medium,
    High,
}

fn default_true() -> bool {
    true
}

fn default_field_type() -> String {
    "string".to_string()
}

fn default_record_name() -> String {
    "record".to_string()
}

fn default_field_name() -> String {
    "id".to_string()
}

fn default_relationship_name() -> String {
    "relationship".to_string()
}

fn default_action_name() -> String {
    "action".to_string()
}

fn default_event_name() -> String {
    "event".to_string()
}

fn default_projection_name() -> String {
    "projection".to_string()
}

fn default_metric_name() -> String {
    "metric".to_string()
}

fn default_filter_operator() -> String {
    "equals".to_string()
}

fn default_policy_name() -> String {
    "policy".to_string()
}

fn default_approval_name() -> String {
    "approval".to_string()
}

fn default_migration_name() -> String {
    "migration".to_string()
}

fn default_provider_category() -> String {
    "storage".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sor_design_draft_round_trips_json() {
        let draft = SorDesignDraft {
            summary: "Supplier approval flow".to_string(),
            records: vec![DraftRecord {
                name: "supplier".to_string(),
                description: Some("Approved maintenance supplier.".to_string()),
                fields: vec![DraftField {
                    name: "name".to_string(),
                    type_name: "string".to_string(),
                    required: true,
                    sensitive: false,
                    description: None,
                    rules: None,
                }],
            }],
            actions: vec![DraftAction {
                name: "approve_supplier_work".to_string(),
                description: None,
                risk: DraftRisk::Medium,
            }],
            approvals: vec![DraftApproval {
                name: "supplier_work_approval".to_string(),
                description: None,
                required: true,
            }],
            ..SorDesignDraft::default()
        };

        let encoded = serde_json::to_string(&draft).expect("draft serializes");
        let decoded: SorDesignDraft = serde_json::from_str(&encoded).expect("draft deserializes");
        assert_eq!(decoded, draft);
    }

    #[test]
    fn draft_deserializes_missing_llm_names_with_defaults() {
        let draft: SorDesignDraft = serde_json::from_value(serde_json::json!({
            "records": [{
                "fields": [{ "type": "string" }]
            }],
            "actions": [{}],
            "events": [{}],
            "projections": [{}],
            "policies": [{}],
            "approvals": [{}],
            "migrations": [{}],
            "provider_requirements": [{}]
        }))
        .expect("missing LLM names should default");

        assert_eq!(draft.records[0].name, "record");
        assert_eq!(draft.records[0].fields[0].name, "id");
        assert_eq!(draft.actions[0].name, "action");
        assert_eq!(draft.events[0].name, "event");
        assert_eq!(draft.projections[0].name, "projection");
        assert_eq!(draft.policies[0].name, "policy");
        assert_eq!(draft.approvals[0].name, "approval");
        assert!(draft.approvals[0].required);
        assert_eq!(draft.migrations[0].name, "migration");
        assert_eq!(draft.provider_requirements[0].category, "storage");
    }

    #[test]
    fn draft_field_accepts_both_type_and_type_name() {
        let field: DraftField = serde_json::from_value(serde_json::json!({
            "name": "bug_case_id",
            "type": "integer",
            "type_name": "string",
            "required": true,
            "sensitive": false
        }))
        .expect("duplicate LLM type aliases should normalize");

        assert_eq!(field.name, "bug_case_id");
        assert_eq!(field.type_name, "string");
    }
}
