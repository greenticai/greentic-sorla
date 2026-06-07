use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;

pub const ANSWERS_V2_VERSION: &str = "sorla.answers.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswersV2 {
    pub version: String,
    pub mode: AnswersMode,
    #[serde(default)]
    pub intent: AuthoringIntent,
    #[serde(default)]
    pub domain: DomainIntent,
    #[serde(default)]
    pub operations: Vec<SemanticOperation>,
    #[serde(default)]
    pub compiler_options: CompilerOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswersMode {
    Create,
    Update,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoringIntent {
    pub summary: Option<String>,
    #[serde(default)]
    pub goals: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub open_questions: Vec<ClarificationQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub id: String,
    pub question: String,
    pub reason: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainIntent {
    #[serde(default)]
    pub actors: Vec<ActorIntent>,
    #[serde(default)]
    pub records: Vec<RecordIntent>,
    #[serde(default)]
    pub processes: Vec<ProcessIntent>,
    #[serde(default)]
    pub business_rules: Vec<BusinessRuleIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorIntent {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordIntent {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub fields: Vec<FieldIntent>,
    #[serde(default)]
    pub relationships: Vec<RelationshipIntent>,
    #[serde(default)]
    pub lifecycle: Option<LifecycleIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldIntent {
    pub name: String,
    pub field_type: String,
    pub required: Option<bool>,
    #[serde(default)]
    pub values: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipIntent {
    pub name: Option<String>,
    pub target: String,
    pub cardinality: String,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleIntent {
    pub state_field: String,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub transitions: Vec<StateTransitionIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransitionIntent {
    pub from: Option<String>,
    pub to: String,
    pub actor: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessIntent {
    pub name: String,
    pub description: Option<String>,
    pub main_record: Option<String>,
    #[serde(default)]
    pub steps: Vec<ProcessStepIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessStepIntent {
    pub name: String,
    pub actor: Option<String>,
    pub action: Option<String>,
    pub record: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessRuleIntent {
    pub name: String,
    pub description: String,
    pub applies_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SemanticOperation {
    AddRecord {
        record: RecordIntent,
    },
    UpdateRecord {
        record: String,
        changes: Value,
    },
    RemoveRecord {
        record: String,
    },
    RenameRecord {
        from: String,
        to: String,
    },
    AddField {
        record: String,
        field: FieldIntent,
    },
    UpdateField {
        record: String,
        field: String,
        changes: Value,
    },
    RemoveField {
        record: String,
        field: String,
    },
    RenameField {
        record: String,
        from: String,
        to: String,
    },
    AddRelationship {
        record: String,
        relationship: RelationshipIntent,
    },
    RemoveRelationship {
        record: String,
        relationship: String,
    },
    AddActor {
        actor: ActorIntent,
    },
    UpdateActor {
        actor: String,
        changes: Value,
    },
    RemoveActor {
        actor: String,
    },
    AddProcess {
        process: ProcessIntent,
    },
    UpdateProcess {
        process: String,
        changes: Value,
    },
    RemoveProcess {
        process: String,
    },
    AddStateTransition {
        record: String,
        transition: StateTransitionIntent,
    },
    RemoveStateTransition {
        record: String,
        from: Option<String>,
        to: String,
    },
    AddBusinessRule {
        rule: BusinessRuleIntent,
    },
    RemoveBusinessRule {
        rule: String,
    },
    AddPolicyIntent {
        name: String,
        description: String,
        applies_to: Option<String>,
    },
    AddMetricIntent {
        name: String,
        description: String,
        applies_to: Option<String>,
    },
    AddProjectionIntent {
        name: String,
        description: String,
        applies_to: Option<String>,
    },
    EnableCapability {
        capability: String,
    },
    DisableCapability {
        capability: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerOptions {
    #[serde(default = "default_true")]
    pub generate_crud: bool,
    #[serde(default = "default_true")]
    pub generate_events: bool,
    #[serde(default = "default_true")]
    pub generate_lifecycle_events: bool,
    #[serde(default = "default_true")]
    pub generate_search: bool,
    #[serde(default = "default_true")]
    pub generate_agent_endpoints: bool,
    #[serde(default = "default_true")]
    pub generate_projections: bool,
    #[serde(default = "default_true")]
    pub generate_metrics: bool,
    #[serde(default = "default_true")]
    pub generate_default_policies: bool,
    #[serde(default = "default_true")]
    pub generate_migrations: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            generate_crud: true,
            generate_events: true,
            generate_lifecycle_events: true,
            generate_search: true,
            generate_agent_endpoints: true,
            generate_projections: true,
            generate_metrics: true,
            generate_default_policies: true,
            generate_migrations: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswersV2ValidationError {
    pub diagnostics: Vec<String>,
}

impl fmt::Display for AnswersV2ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.diagnostics.join("; "))
    }
}

impl std::error::Error for AnswersV2ValidationError {}

pub fn is_answers_v2_json(value: &Value) -> bool {
    value
        .get("version")
        .and_then(Value::as_str)
        .is_some_and(|version| version == ANSWERS_V2_VERSION)
}

pub fn validate_answers_v2(answers: &AnswersV2) -> Result<(), AnswersV2ValidationError> {
    let mut diagnostics = Vec::new();
    require_exact(
        "version",
        &answers.version,
        ANSWERS_V2_VERSION,
        &mut diagnostics,
    );
    validate_intent(&answers.intent, &mut diagnostics);
    validate_domain(&answers.domain, &mut diagnostics);
    validate_operations(&answers.operations, &mut diagnostics);
    if answers.mode == AnswersMode::Update
        && answers.operations.is_empty()
        && answers.domain.records.is_empty()
        && answers.domain.actors.is_empty()
        && answers.domain.processes.is_empty()
    {
        diagnostics
            .push("update answers must include semantic operations or domain intent".to_string());
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(AnswersV2ValidationError { diagnostics })
    }
}

fn validate_intent(intent: &AuthoringIntent, diagnostics: &mut Vec<String>) {
    let mut question_ids = BTreeSet::new();
    for (index, question) in intent.open_questions.iter().enumerate() {
        require_non_empty(
            format!("intent.open_questions[{index}].id"),
            &question.id,
            diagnostics,
        );
        require_non_empty(
            format!("intent.open_questions[{index}].question"),
            &question.question,
            diagnostics,
        );
        require_unique(
            "intent.open_questions.id",
            &mut question_ids,
            &question.id,
            diagnostics,
        );
    }
}

fn validate_domain(domain: &DomainIntent, diagnostics: &mut Vec<String>) {
    let mut actor_names = BTreeSet::new();
    for (index, actor) in domain.actors.iter().enumerate() {
        require_named(
            format!("domain.actors[{index}].name"),
            &mut actor_names,
            &actor.name,
            diagnostics,
        );
    }

    let mut record_names = BTreeSet::new();
    for (index, record) in domain.records.iter().enumerate() {
        require_named(
            format!("domain.records[{index}].name"),
            &mut record_names,
            &record.name,
            diagnostics,
        );
        validate_record(record, index, diagnostics);
    }

    let mut process_names = BTreeSet::new();
    for (index, process) in domain.processes.iter().enumerate() {
        require_named(
            format!("domain.processes[{index}].name"),
            &mut process_names,
            &process.name,
            diagnostics,
        );
        for (step_index, step) in process.steps.iter().enumerate() {
            require_non_empty(
                format!("domain.processes[{index}].steps[{step_index}].name"),
                &step.name,
                diagnostics,
            );
        }
    }

    let mut rule_names = BTreeSet::new();
    for (index, rule) in domain.business_rules.iter().enumerate() {
        require_named(
            format!("domain.business_rules[{index}].name"),
            &mut rule_names,
            &rule.name,
            diagnostics,
        );
        require_non_empty(
            format!("domain.business_rules[{index}].description"),
            &rule.description,
            diagnostics,
        );
    }
}

fn validate_record(record: &RecordIntent, index: usize, diagnostics: &mut Vec<String>) {
    let mut field_names = BTreeSet::new();
    for (field_index, field) in record.fields.iter().enumerate() {
        require_named(
            format!("domain.records[{index}].fields[{field_index}].name"),
            &mut field_names,
            &field.name,
            diagnostics,
        );
        require_non_empty(
            format!("domain.records[{index}].fields[{field_index}].field_type"),
            &field.field_type,
            diagnostics,
        );
        for (value_index, value) in field.values.iter().enumerate() {
            require_non_empty(
                format!("domain.records[{index}].fields[{field_index}].values[{value_index}]"),
                value,
                diagnostics,
            );
        }
    }

    let mut relationship_names = BTreeSet::new();
    for (relationship_index, relationship) in record.relationships.iter().enumerate() {
        if let Some(name) = &relationship.name {
            require_named(
                format!("domain.records[{index}].relationships[{relationship_index}].name"),
                &mut relationship_names,
                name,
                diagnostics,
            );
        }
        require_non_empty(
            format!("domain.records[{index}].relationships[{relationship_index}].target"),
            &relationship.target,
            diagnostics,
        );
        require_non_empty(
            format!("domain.records[{index}].relationships[{relationship_index}].cardinality"),
            &relationship.cardinality,
            diagnostics,
        );
    }

    if let Some(lifecycle) = &record.lifecycle {
        validate_lifecycle(lifecycle, index, diagnostics);
    }
}

fn validate_lifecycle(
    lifecycle: &LifecycleIntent,
    record_index: usize,
    diagnostics: &mut Vec<String>,
) {
    require_non_empty(
        format!("domain.records[{record_index}].lifecycle.state_field"),
        &lifecycle.state_field,
        diagnostics,
    );
    let mut states = BTreeSet::new();
    for (state_index, state) in lifecycle.states.iter().enumerate() {
        require_named(
            format!("domain.records[{record_index}].lifecycle.states[{state_index}]"),
            &mut states,
            state,
            diagnostics,
        );
    }
    for (transition_index, transition) in lifecycle.transitions.iter().enumerate() {
        if let Some(from) = &transition.from {
            require_non_empty(
                format!(
                    "domain.records[{record_index}].lifecycle.transitions[{transition_index}].from"
                ),
                from,
                diagnostics,
            );
            if !states.is_empty() && !states.contains(from.trim()) {
                diagnostics.push(format!(
                    "domain.records[{record_index}].lifecycle.transitions[{transition_index}].from references unknown state `{from}`"
                ));
            }
        }
        require_non_empty(
            format!("domain.records[{record_index}].lifecycle.transitions[{transition_index}].to"),
            &transition.to,
            diagnostics,
        );
        if !states.is_empty() && !states.contains(transition.to.trim()) {
            diagnostics.push(format!(
                "domain.records[{record_index}].lifecycle.transitions[{transition_index}].to references unknown state `{}`",
                transition.to
            ));
        }
    }
}

fn validate_operations(operations: &[SemanticOperation], diagnostics: &mut Vec<String>) {
    for (index, operation) in operations.iter().enumerate() {
        match operation {
            SemanticOperation::AddRecord { record } => {
                require_non_empty(
                    format!("operations[{index}].record.name"),
                    &record.name,
                    diagnostics,
                );
                validate_record(record, index, diagnostics);
            }
            SemanticOperation::UpdateRecord { record, .. }
            | SemanticOperation::RemoveRecord { record }
            | SemanticOperation::AddField { record, .. }
            | SemanticOperation::UpdateField { record, .. }
            | SemanticOperation::RemoveField { record, .. }
            | SemanticOperation::RenameField { record, .. }
            | SemanticOperation::AddRelationship { record, .. }
            | SemanticOperation::RemoveRelationship { record, .. }
            | SemanticOperation::AddStateTransition { record, .. }
            | SemanticOperation::RemoveStateTransition { record, .. } => {
                require_non_empty(format!("operations[{index}].record"), record, diagnostics);
            }
            SemanticOperation::RenameRecord { from, to } => {
                require_non_empty(format!("operations[{index}].from"), from, diagnostics);
                require_non_empty(format!("operations[{index}].to"), to, diagnostics);
            }
            SemanticOperation::AddActor { actor } => {
                require_non_empty(
                    format!("operations[{index}].actor.name"),
                    &actor.name,
                    diagnostics,
                );
            }
            SemanticOperation::UpdateActor { actor, .. }
            | SemanticOperation::RemoveActor { actor } => {
                require_non_empty(format!("operations[{index}].actor"), actor, diagnostics);
            }
            SemanticOperation::AddProcess { process } => {
                require_non_empty(
                    format!("operations[{index}].process.name"),
                    &process.name,
                    diagnostics,
                );
            }
            SemanticOperation::UpdateProcess { process, .. }
            | SemanticOperation::RemoveProcess { process } => {
                require_non_empty(format!("operations[{index}].process"), process, diagnostics);
            }
            SemanticOperation::AddBusinessRule { rule } => {
                require_non_empty(
                    format!("operations[{index}].rule.name"),
                    &rule.name,
                    diagnostics,
                );
                require_non_empty(
                    format!("operations[{index}].rule.description"),
                    &rule.description,
                    diagnostics,
                );
            }
            SemanticOperation::RemoveBusinessRule { rule } => {
                require_non_empty(format!("operations[{index}].rule"), rule, diagnostics);
            }
            SemanticOperation::AddPolicyIntent {
                name, description, ..
            }
            | SemanticOperation::AddMetricIntent {
                name, description, ..
            }
            | SemanticOperation::AddProjectionIntent {
                name, description, ..
            } => {
                require_non_empty(format!("operations[{index}].name"), name, diagnostics);
                require_non_empty(
                    format!("operations[{index}].description"),
                    description,
                    diagnostics,
                );
            }
            SemanticOperation::EnableCapability { capability }
            | SemanticOperation::DisableCapability { capability } => {
                require_non_empty(
                    format!("operations[{index}].capability"),
                    capability,
                    diagnostics,
                );
            }
        }

        match operation {
            SemanticOperation::AddField { field, .. } => {
                require_non_empty(
                    format!("operations[{index}].field.name"),
                    &field.name,
                    diagnostics,
                );
                require_non_empty(
                    format!("operations[{index}].field.field_type"),
                    &field.field_type,
                    diagnostics,
                );
            }
            SemanticOperation::UpdateField { field, .. }
            | SemanticOperation::RemoveField { field, .. } => {
                require_non_empty(format!("operations[{index}].field"), field, diagnostics);
            }
            SemanticOperation::RenameField { from, to, .. } => {
                require_non_empty(format!("operations[{index}].from"), from, diagnostics);
                require_non_empty(format!("operations[{index}].to"), to, diagnostics);
            }
            SemanticOperation::AddRelationship { relationship, .. } => {
                require_non_empty(
                    format!("operations[{index}].relationship.target"),
                    &relationship.target,
                    diagnostics,
                );
                require_non_empty(
                    format!("operations[{index}].relationship.cardinality"),
                    &relationship.cardinality,
                    diagnostics,
                );
            }
            SemanticOperation::RemoveRelationship { relationship, .. } => {
                require_non_empty(
                    format!("operations[{index}].relationship"),
                    relationship,
                    diagnostics,
                );
            }
            SemanticOperation::AddStateTransition { transition, .. } => {
                require_non_empty(
                    format!("operations[{index}].transition.to"),
                    &transition.to,
                    diagnostics,
                );
            }
            SemanticOperation::RemoveStateTransition { to, .. } => {
                require_non_empty(format!("operations[{index}].to"), to, diagnostics);
            }
            _ => {}
        }
    }
}

fn require_exact(path: &str, actual: &str, expected: &str, diagnostics: &mut Vec<String>) {
    if actual != expected {
        diagnostics.push(format!("{path}: expected `{expected}`, got `{actual}`"));
    }
}

fn require_named(
    path: String,
    names: &mut BTreeSet<String>,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    require_non_empty(path.clone(), value, diagnostics);
    let normalized = value.trim().to_string();
    if !normalized.is_empty() && !names.insert(normalized.clone()) {
        diagnostics.push(format!("{path}: duplicate name `{normalized}`"));
    }
}

fn require_unique(
    path: &str,
    values: &mut BTreeSet<String>,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let normalized = value.trim().to_string();
    if !normalized.is_empty() && !values.insert(normalized.clone()) {
        diagnostics.push(format!("{path}: duplicate value `{normalized}`"));
    }
}

fn require_non_empty(path: String, value: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{path}: must not be empty"));
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_answers_v2_json_by_version() {
        assert!(is_answers_v2_json(&serde_json::json!({
            "version": "sorla.answers.v2"
        })));
        assert!(!is_answers_v2_json(&serde_json::json!({
            "schema_version": "1"
        })));
    }

    #[test]
    fn answers_v2_create_round_trips_and_defaults_compiler_options() {
        let value = serde_json::json!({
            "version": "sorla.answers.v2",
            "mode": "create",
            "intent": {
                "summary": "Create a maintenance system"
            },
            "domain": {
                "actors": [
                    { "name": "tenant", "description": "Reports issues" }
                ],
                "records": [
                    {
                        "name": "maintenance_request",
                        "description": "A maintenance issue",
                        "fields": [
                            { "name": "status", "field_type": "enum", "values": ["reported", "approved"] }
                        ],
                        "lifecycle": {
                            "state_field": "status",
                            "states": ["reported", "approved"],
                            "transitions": [
                                { "from": "reported", "to": "approved", "actor": "landlord" }
                            ]
                        }
                    }
                ]
            }
        });

        let answers: AnswersV2 = serde_json::from_value(value).expect("answers parse");
        validate_answers_v2(&answers).expect("answers validate");
        assert_eq!(answers.mode, AnswersMode::Create);
        assert!(answers.compiler_options.generate_crud);
        assert!(answers.compiler_options.generate_agent_endpoints);

        let encoded = serde_json::to_value(&answers).expect("answers serialize");
        assert_eq!(encoded["mode"], "create");
    }

    #[test]
    fn answers_v2_update_accepts_semantic_operations() {
        let value = serde_json::json!({
            "version": "sorla.answers.v2",
            "mode": "update",
            "intent": { "summary": "Add quote approval" },
            "operations": [
                {
                    "op": "add_record",
                    "record": {
                        "name": "quote",
                        "description": "Contractor quote",
                        "fields": [
                            { "name": "amount", "field_type": "money", "required": true }
                        ],
                        "relationships": [
                            {
                                "name": "maintenance_request",
                                "target": "maintenance_request",
                                "cardinality": "many_to_one",
                                "required": true
                            }
                        ]
                    }
                }
            ]
        });

        let answers: AnswersV2 = serde_json::from_value(value).expect("answers parse");
        validate_answers_v2(&answers).expect("answers validate");
        assert_eq!(answers.operations.len(), 1);
        assert!(matches!(
            answers.operations[0],
            SemanticOperation::AddRecord { .. }
        ));
    }

    #[test]
    fn answers_v2_validation_reports_domain_errors() {
        let answers = AnswersV2 {
            version: "wrong".to_string(),
            mode: AnswersMode::Create,
            intent: AuthoringIntent::default(),
            domain: DomainIntent {
                records: vec![RecordIntent {
                    name: "request".to_string(),
                    description: None,
                    fields: vec![
                        FieldIntent {
                            name: "status".to_string(),
                            field_type: "enum".to_string(),
                            required: Some(true),
                            values: vec!["open".to_string()],
                            description: None,
                        },
                        FieldIntent {
                            name: "status".to_string(),
                            field_type: String::new(),
                            required: None,
                            values: Vec::new(),
                            description: None,
                        },
                    ],
                    relationships: Vec::new(),
                    lifecycle: Some(LifecycleIntent {
                        state_field: "status".to_string(),
                        states: vec!["open".to_string()],
                        transitions: vec![StateTransitionIntent {
                            from: Some("missing".to_string()),
                            to: "closed".to_string(),
                            actor: None,
                            description: None,
                        }],
                    }),
                }],
                ..DomainIntent::default()
            },
            operations: Vec::new(),
            compiler_options: CompilerOptions::default(),
        };

        let error = validate_answers_v2(&answers).expect_err("answers should fail validation");
        let rendered = error.to_string();
        assert!(rendered.contains("version"));
        assert!(rendered.contains("duplicate name `status`"));
        assert!(rendered.contains("field_type"));
        assert!(rendered.contains("unknown state `missing`"));
        assert!(rendered.contains("unknown state `closed`"));
    }
}
