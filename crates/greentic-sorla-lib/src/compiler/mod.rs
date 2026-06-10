pub mod expanders;
pub mod plan;
pub mod provenance;
pub mod update;

pub use expanders::*;
pub use plan::*;
pub use provenance::*;
pub use update::*;

use crate::prompt::{
    AnswersMode, AnswersV2, BusinessRuleIntent, DomainIntent, RecordIntent, SemanticOperation,
};

pub fn compile_answers_v2(answers: &AnswersV2) -> Result<ExpandedSorlaPlan, CompileError> {
    let mut plan = ExpandedSorlaPlan::from_domain(&answers.domain);
    apply_semantic_operations(&mut plan, &answers.operations)?;
    let ctx = ExpansionContext {
        mode: match answers.mode {
            AnswersMode::Create => CompileMode::Create,
            AnswersMode::Update => CompileMode::Update,
        },
        options: answers.compiler_options.clone(),
        naming: NamingRules::default(),
    };
    run_default_expanders(&ctx, &mut plan)?;
    validate_expanded_plan(&mut plan);
    Ok(plan)
}

pub fn apply_semantic_operations(
    plan: &mut ExpandedSorlaPlan,
    operations: &[SemanticOperation],
) -> Result<(), CompileError> {
    for operation in operations {
        match operation {
            SemanticOperation::AddRecord { record } => {
                if plan
                    .records
                    .iter()
                    .any(|existing| existing.name == record.name)
                {
                    return Err(CompileError::new(format!(
                        "record `{}` already exists",
                        record.name
                    )));
                }
                plan.records.push(RecordPlan::from_intent(
                    record,
                    Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("add_record".to_string()),
                    },
                ));
            }
            SemanticOperation::AddActor { actor } => {
                if plan
                    .actors
                    .iter()
                    .any(|existing| existing.name == actor.name)
                {
                    return Err(CompileError::new(format!(
                        "actor `{}` already exists",
                        actor.name
                    )));
                }
                plan.actors.push(ActorPlan {
                    name: actor.name.clone(),
                    description: actor.description.clone(),
                    aliases: actor.aliases.clone(),
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("add_actor".to_string()),
                    },
                });
            }
            SemanticOperation::AddField { record, field } => {
                let target = plan
                    .records
                    .iter_mut()
                    .find(|existing| existing.name == *record)
                    .ok_or_else(|| {
                        CompileError::new(format!("record `{record}` does not exist"))
                    })?;
                if target
                    .fields
                    .iter()
                    .any(|existing| existing.name == field.name)
                {
                    return Err(CompileError::new(format!(
                        "field `{record}.{}` already exists",
                        field.name
                    )));
                }
                target.fields.push(FieldPlan {
                    name: field.name.clone(),
                    field_type: field.field_type.clone(),
                    required: field.required.unwrap_or(false),
                    values: field.values.clone(),
                    description: field.description.clone(),
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("add_field".to_string()),
                    },
                });
            }
            SemanticOperation::AddRelationship {
                record,
                relationship,
            } => {
                let target = plan
                    .records
                    .iter_mut()
                    .find(|existing| existing.name == *record)
                    .ok_or_else(|| {
                        CompileError::new(format!("record `{record}` does not exist"))
                    })?;
                target.relationships.push(RelationshipPlan {
                    name: relationship
                        .name
                        .clone()
                        .unwrap_or_else(|| relationship.target.clone()),
                    target: relationship.target.clone(),
                    cardinality: relationship.cardinality.clone(),
                    required: relationship.required.unwrap_or(false),
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("add_relationship".to_string()),
                    },
                });
            }
            SemanticOperation::AddProcess { process } => {
                plan.processes.push(ProcessPlan::from_intent(
                    process,
                    Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("add_process".to_string()),
                    },
                ));
            }
            SemanticOperation::AddBusinessRule { rule } => {
                plan.business_rules.push(business_rule_plan(
                    rule,
                    Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("add_business_rule".to_string()),
                    },
                ));
            }
            SemanticOperation::EnableCapability { capability } => {
                plan.capabilities.push(CapabilityPlan {
                    name: capability.clone(),
                    enabled: true,
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("enable_capability".to_string()),
                    },
                });
            }
            SemanticOperation::DisableCapability { capability } => {
                plan.capabilities.push(CapabilityPlan {
                    name: capability.clone(),
                    enabled: false,
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-operation".to_string(),
                        reason: Some("disable_capability".to_string()),
                    },
                });
            }
            other => {
                plan.diagnostics.push(CompileDiagnostic::warning(
                    "SORLA_OPERATION_DEFERRED",
                    format!(
                        "semantic operation `{}` is parsed but will be applied by the update compiler",
                        operation_name(other)
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub fn run_default_expanders(
    ctx: &ExpansionContext,
    plan: &mut ExpandedSorlaPlan,
) -> Result<(), CompileError> {
    let expanders: Vec<Box<dyn SorlaExpander>> = vec![
        Box::new(RecordCrudExpander),
        Box::new(RecordEventExpander),
        Box::new(LifecycleEventExpander),
        Box::new(SearchEndpointExpander),
        Box::new(AgentEndpointExpander),
        Box::new(ProjectionExpander),
        Box::new(MetricExpander),
        Box::new(PolicyDefaultExpander),
        Box::new(MigrationExpander),
    ];
    for expander in expanders {
        expander.expand(ctx, plan)?;
    }
    Ok(())
}

pub fn validate_expanded_plan(plan: &mut ExpandedSorlaPlan) {
    validate_unique(
        &mut plan.diagnostics,
        "SORLA_DUPLICATE_ACTION",
        "action",
        plan.actions.iter().map(|action| action.name.as_str()),
    );
    validate_unique(
        &mut plan.diagnostics,
        "SORLA_DUPLICATE_EVENT",
        "event",
        plan.events.iter().map(|event| event.name.as_str()),
    );
    validate_unique(
        &mut plan.diagnostics,
        "SORLA_DUPLICATE_PROJECTION",
        "projection",
        plan.projections
            .iter()
            .map(|projection| projection.name.as_str()),
    );
    validate_unique(
        &mut plan.diagnostics,
        "SORLA_DUPLICATE_AGENT_ENDPOINT",
        "agent endpoint",
        plan.agent_endpoints
            .iter()
            .map(|endpoint| endpoint.id.as_str()),
    );

    let record_names = plan
        .records
        .iter()
        .map(|record| record.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for record in &plan.records {
        for relationship in &record.relationships {
            if !record_names.contains(relationship.target.as_str()) {
                plan.diagnostics.push(CompileDiagnostic::warning(
                    "SORLA_UNKNOWN_RELATIONSHIP_TARGET",
                    format!(
                        "relationship `{}.{}` references missing record `{}`",
                        record.name, relationship.name, relationship.target
                    ),
                ));
            }
        }
    }
}

fn validate_unique<'a>(
    diagnostics: &mut Vec<CompileDiagnostic>,
    code: &'static str,
    kind: &str,
    values: impl Iterator<Item = &'a str>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            diagnostics.push(CompileDiagnostic::error(
                code,
                format!("duplicate {kind} `{value}`"),
            ));
        }
    }
}

fn business_rule_plan(rule: &BusinessRuleIntent, provenance: Provenance) -> BusinessRulePlan {
    BusinessRulePlan {
        name: rule.name.clone(),
        description: rule.description.clone(),
        applies_to: rule.applies_to.clone(),
        provenance,
    }
}

fn operation_name(operation: &SemanticOperation) -> &'static str {
    match operation {
        SemanticOperation::AddRecord { .. } => "add_record",
        SemanticOperation::UpdateRecord { .. } => "update_record",
        SemanticOperation::RemoveRecord { .. } => "remove_record",
        SemanticOperation::RenameRecord { .. } => "rename_record",
        SemanticOperation::AddField { .. } => "add_field",
        SemanticOperation::UpdateField { .. } => "update_field",
        SemanticOperation::RemoveField { .. } => "remove_field",
        SemanticOperation::RenameField { .. } => "rename_field",
        SemanticOperation::AddRelationship { .. } => "add_relationship",
        SemanticOperation::RemoveRelationship { .. } => "remove_relationship",
        SemanticOperation::AddActor { .. } => "add_actor",
        SemanticOperation::UpdateActor { .. } => "update_actor",
        SemanticOperation::RemoveActor { .. } => "remove_actor",
        SemanticOperation::AddProcess { .. } => "add_process",
        SemanticOperation::UpdateProcess { .. } => "update_process",
        SemanticOperation::RemoveProcess { .. } => "remove_process",
        SemanticOperation::AddStateTransition { .. } => "add_state_transition",
        SemanticOperation::RemoveStateTransition { .. } => "remove_state_transition",
        SemanticOperation::AddBusinessRule { .. } => "add_business_rule",
        SemanticOperation::RemoveBusinessRule { .. } => "remove_business_rule",
        SemanticOperation::AddPolicyIntent { .. } => "add_policy_intent",
        SemanticOperation::AddMetricIntent { .. } => "add_metric_intent",
        SemanticOperation::AddProjectionIntent { .. } => "add_projection_intent",
        SemanticOperation::EnableCapability { .. } => "enable_capability",
        SemanticOperation::DisableCapability { .. } => "disable_capability",
    }
}

impl ExpandedSorlaPlan {
    fn from_domain(domain: &DomainIntent) -> Self {
        Self {
            actors: domain
                .actors
                .iter()
                .map(|actor| ActorPlan {
                    name: actor.name.clone(),
                    description: actor.description.clone(),
                    aliases: actor.aliases.clone(),
                    provenance: Provenance::UserProvided,
                })
                .collect(),
            records: domain
                .records
                .iter()
                .map(|record| RecordPlan::from_intent(record, Provenance::UserProvided))
                .collect(),
            processes: domain
                .processes
                .iter()
                .map(|process| ProcessPlan::from_intent(process, Provenance::UserProvided))
                .collect(),
            business_rules: domain
                .business_rules
                .iter()
                .map(|rule| business_rule_plan(rule, Provenance::UserProvided))
                .collect(),
            ..Self::default()
        }
    }
}

impl RecordPlan {
    fn from_intent(record: &RecordIntent, provenance: Provenance) -> Self {
        Self {
            name: record.name.clone(),
            description: record.description.clone(),
            fields: record
                .fields
                .iter()
                .map(|field| FieldPlan {
                    name: field.name.clone(),
                    field_type: field.field_type.clone(),
                    required: field.required.unwrap_or(false),
                    values: field.values.clone(),
                    description: field.description.clone(),
                    provenance: provenance.clone(),
                })
                .collect(),
            relationships: record
                .relationships
                .iter()
                .map(|relationship| RelationshipPlan {
                    name: relationship
                        .name
                        .clone()
                        .unwrap_or_else(|| relationship.target.clone()),
                    target: relationship.target.clone(),
                    cardinality: relationship.cardinality.clone(),
                    required: relationship.required.unwrap_or(false),
                    provenance: provenance.clone(),
                })
                .collect(),
            lifecycle: record.lifecycle.as_ref().map(|lifecycle| LifecyclePlan {
                state_field: lifecycle.state_field.clone(),
                states: lifecycle.states.clone(),
                transitions: lifecycle
                    .transitions
                    .iter()
                    .map(|transition| StateTransitionPlan {
                        from: transition.from.clone(),
                        to: transition.to.clone(),
                        actor: transition.actor.clone(),
                        description: transition.description.clone(),
                        provenance: provenance.clone(),
                    })
                    .collect(),
                provenance: provenance.clone(),
            }),
            provenance,
        }
    }
}

impl ProcessPlan {
    fn from_intent(process: &crate::prompt::ProcessIntent, provenance: Provenance) -> Self {
        Self {
            name: process.name.clone(),
            description: process.description.clone(),
            main_record: process.main_record.clone(),
            steps: process
                .steps
                .iter()
                .map(|step| ProcessStepPlan {
                    name: step.name.clone(),
                    actor: step.actor.clone(),
                    action: step.action.clone(),
                    record: step.record.clone(),
                    provenance: provenance.clone(),
                })
                .collect(),
            provenance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{AnswersMode, CompilerOptions};

    #[test]
    fn compiler_expands_records_deterministically() {
        let answers = AnswersV2 {
            version: crate::prompt::ANSWERS_V2_VERSION.to_string(),
            mode: AnswersMode::Create,
            intent: Default::default(),
            domain: DomainIntent {
                records: vec![crate::prompt::RecordIntent {
                    name: "quote".to_string(),
                    description: Some("Contractor quote".to_string()),
                    fields: vec![crate::prompt::FieldIntent {
                        name: "status".to_string(),
                        field_type: "enum".to_string(),
                        required: Some(true),
                        values: vec!["submitted".to_string(), "approved".to_string()],
                        description: None,
                    }],
                    relationships: Vec::new(),
                    lifecycle: Some(crate::prompt::LifecycleIntent {
                        state_field: "status".to_string(),
                        states: vec!["submitted".to_string(), "approved".to_string()],
                        transitions: vec![crate::prompt::StateTransitionIntent {
                            from: Some("submitted".to_string()),
                            to: "approved".to_string(),
                            actor: Some("landlord".to_string()),
                            description: None,
                        }],
                    }),
                }],
                ..DomainIntent::default()
            },
            operations: Vec::new(),
            compiler_options: CompilerOptions::default(),
        };

        let plan = compile_answers_v2(&answers).expect("compile answers");
        let actions = plan
            .actions
            .iter()
            .map(|action| action.name.as_str())
            .collect::<Vec<_>>();
        assert!(actions.contains(&"quote.create"));
        assert!(actions.contains(&"quote.search"));
        assert!(actions.contains(&"quote.approve"));

        let events = plan
            .events
            .iter()
            .map(|event| event.name.as_str())
            .collect::<Vec<_>>();
        assert!(events.contains(&"quote.created"));
        assert!(events.contains(&"quote.approved"));

        let endpoints = plan
            .agent_endpoints
            .iter()
            .map(|endpoint| endpoint.id.as_str())
            .collect::<Vec<_>>();
        assert!(endpoints.contains(&"agent.create_quote"));
        assert!(endpoints.contains(&"agent.approve_quote"));

        let projections = plan
            .projections
            .iter()
            .map(|projection| projection.name.as_str())
            .collect::<Vec<_>>();
        assert!(projections.contains(&"quote_list"));
        assert!(projections.contains(&"quote_by_status"));

        let metrics = plan
            .metrics
            .iter()
            .map(|metric| metric.name.as_str())
            .collect::<Vec<_>>();
        assert!(metrics.contains(&"count_quote"));
        assert!(metrics.contains(&"average_time_to_quote_approved"));
    }

    #[test]
    fn compiler_applies_add_record_operations() {
        let answers = AnswersV2 {
            version: crate::prompt::ANSWERS_V2_VERSION.to_string(),
            mode: AnswersMode::Update,
            intent: Default::default(),
            domain: DomainIntent::default(),
            operations: vec![SemanticOperation::AddRecord {
                record: crate::prompt::RecordIntent {
                    name: "invoice".to_string(),
                    description: None,
                    fields: Vec::new(),
                    relationships: Vec::new(),
                    lifecycle: None,
                },
            }],
            compiler_options: CompilerOptions::default(),
        };

        let plan = compile_answers_v2(&answers).expect("compile answers");
        assert!(plan.records.iter().any(|record| record.name == "invoice"));
        assert!(
            plan.actions
                .iter()
                .any(|action| action.name == "invoice.create")
        );
    }
}
