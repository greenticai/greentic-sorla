use super::{
    ActionKind, ActionPlan, CompileDiagnostic, CompileError, DiagnosticSeverity, ExpandedSorlaPlan,
    FieldPlan, Provenance, RecordPlan, RelationshipPlan, compile_answers_v2,
    validate_expanded_plan,
};
use crate::prompt::{AnswersV2, SemanticOperation};
use greentic_sorla_lang::parser::parse_package;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExistingSorlaModel {
    pub plan: ExpandedSorlaPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SorlaUpdatePlan {
    pub model: ExistingSorlaModel,
    pub diff: SorlaDiffPreview,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SorlaDiffPreview {
    #[serde(default)]
    pub added: Vec<DiffItem>,
    #[serde(default)]
    pub changed: Vec<DiffItem>,
    #[serde(default)]
    pub removed: Vec<DiffItem>,
    #[serde(default)]
    pub warnings: Vec<CompileDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffItem {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UpdateApplyOptions {
    pub allow_destructive: bool,
}

pub fn plan_update_operations(
    existing: &ExistingSorlaModel,
    answers: &AnswersV2,
    options: UpdateApplyOptions,
) -> Result<SorlaUpdatePlan, CompileError> {
    let mut model = existing.clone();
    apply_semantic_update_operations(&mut model, &answers.operations, options)?;
    let compiled = compile_answers_v2(answers)?;
    merge_generated_sections(&mut model.plan, compiled);
    validate_expanded_plan(&mut model.plan);
    let diff = diff_existing_to_updated(existing, &model);
    Ok(SorlaUpdatePlan { model, diff })
}

pub fn existing_sorla_model_from_yaml(
    source_yaml: &str,
) -> Result<ExistingSorlaModel, CompileError> {
    let parsed = parse_package(source_yaml).map_err(CompileError::new)?;
    let package = parsed.package;
    let mut plan = ExpandedSorlaPlan {
        package: Some(super::PackagePlan {
            name: package.package.name,
            version: package.package.version,
            provenance: Provenance::ExistingYaml { path: None },
        }),
        records: package
            .records
            .iter()
            .map(|record| RecordPlan {
                name: record.name.clone(),
                description: None,
                fields: record
                    .fields
                    .iter()
                    .map(|field| FieldPlan {
                        name: field.name.clone(),
                        field_type: field.type_name.clone(),
                        required: field.required,
                        values: field.enum_values.clone(),
                        description: None,
                        provenance: Provenance::ExistingYaml { path: None },
                    })
                    .collect(),
                relationships: record
                    .fields
                    .iter()
                    .filter_map(|field| {
                        let reference = field.references.as_ref()?;
                        Some(RelationshipPlan {
                            name: field.name.clone(),
                            target: reference.record.clone(),
                            cardinality: "many_to_one".to_string(),
                            required: field.required,
                            provenance: Provenance::ExistingYaml { path: None },
                        })
                    })
                    .collect(),
                lifecycle: None,
                provenance: Provenance::ExistingYaml { path: None },
            })
            .collect(),
        actions: package
            .actions
            .iter()
            .map(|action| ActionPlan {
                name: action.name.clone(),
                record: action
                    .name
                    .split_once('.')
                    .map(|(record, _)| record.to_string()),
                kind: ActionKind::Update,
                provenance: Provenance::ExistingYaml { path: None },
            })
            .collect(),
        ..ExpandedSorlaPlan::default()
    };
    validate_expanded_plan(&mut plan);
    Ok(ExistingSorlaModel { plan })
}

pub fn apply_semantic_update_operations(
    model: &mut ExistingSorlaModel,
    operations: &[SemanticOperation],
    options: UpdateApplyOptions,
) -> Result<(), CompileError> {
    for operation in operations {
        match operation {
            SemanticOperation::AddRecord { record } => {
                if model
                    .plan
                    .records
                    .iter()
                    .any(|existing| existing.name == record.name)
                {
                    return Err(CompileError::new(format!(
                        "record `{}` already exists",
                        record.name
                    )));
                }
                model
                    .plan
                    .records
                    .push(RecordPlan::from_update_record(record));
            }
            SemanticOperation::AddField { record, field } => {
                let target = find_record_mut(&mut model.plan, record)?;
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
                        agent: "semantic-update".to_string(),
                        reason: Some("add_field".to_string()),
                    },
                });
            }
            SemanticOperation::AddRelationship {
                record,
                relationship,
            } => {
                let target = find_record_mut(&mut model.plan, record)?;
                target.relationships.push(RelationshipPlan {
                    name: relationship
                        .name
                        .clone()
                        .unwrap_or_else(|| relationship.target.clone()),
                    target: relationship.target.clone(),
                    cardinality: relationship.cardinality.clone(),
                    required: relationship.required.unwrap_or(false),
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-update".to_string(),
                        reason: Some("add_relationship".to_string()),
                    },
                });
            }
            SemanticOperation::RemoveRecord { record } => {
                require_destructive(options, format!("remove_record `{record}`"))?;
                model
                    .plan
                    .records
                    .retain(|existing| existing.name != *record);
            }
            SemanticOperation::RemoveField { record, field } => {
                require_destructive(options, format!("remove_field `{record}.{field}`"))?;
                let target = find_record_mut(&mut model.plan, record)?;
                target.fields.retain(|existing| existing.name != *field);
            }
            SemanticOperation::RenameRecord { from, to } => {
                let target = find_record_mut(&mut model.plan, from)?;
                target.name = to.clone();
            }
            SemanticOperation::RenameField { record, from, to } => {
                let target = find_record_mut(&mut model.plan, record)?;
                let field = target
                    .fields
                    .iter_mut()
                    .find(|existing| existing.name == *from)
                    .ok_or_else(|| {
                        CompileError::new(format!("field `{record}.{from}` does not exist"))
                    })?;
                field.name = to.clone();
            }
            _ => {
                model.plan.diagnostics.push(CompileDiagnostic::warning(
                    "SORLA_OPERATION_DEFERRED",
                    "semantic update operation is parsed but not applied by the initial update module",
                ));
            }
        }
    }
    Ok(())
}

fn merge_generated_sections(plan: &mut ExpandedSorlaPlan, generated: ExpandedSorlaPlan) {
    for action in generated.actions {
        if !plan
            .actions
            .iter()
            .any(|existing| existing.name == action.name)
        {
            plan.actions.push(action);
        }
    }
    for event in generated.events {
        if !plan
            .events
            .iter()
            .any(|existing| existing.name == event.name)
        {
            plan.events.push(event);
        }
    }
    for projection in generated.projections {
        if !plan
            .projections
            .iter()
            .any(|existing| existing.name == projection.name)
        {
            plan.projections.push(projection);
        }
    }
    for metric in generated.metrics {
        if !plan
            .metrics
            .iter()
            .any(|existing| existing.name == metric.name)
        {
            plan.metrics.push(metric);
        }
    }
    for policy in generated.policies {
        if !plan
            .policies
            .iter()
            .any(|existing| existing.name == policy.name)
        {
            plan.policies.push(policy);
        }
    }
    for migration in generated.migrations {
        if !plan
            .migrations
            .iter()
            .any(|existing| existing.name == migration.name)
        {
            plan.migrations.push(migration);
        }
    }
    for endpoint in generated.agent_endpoints {
        if !plan
            .agent_endpoints
            .iter()
            .any(|existing| existing.id == endpoint.id)
        {
            plan.agent_endpoints.push(endpoint);
        }
    }
}

fn diff_existing_to_updated(
    existing: &ExistingSorlaModel,
    updated: &ExistingSorlaModel,
) -> SorlaDiffPreview {
    let mut preview = SorlaDiffPreview::default();
    diff_names(
        "record",
        existing
            .plan
            .records
            .iter()
            .map(|record| record.name.as_str()),
        updated
            .plan
            .records
            .iter()
            .map(|record| record.name.as_str()),
        &mut preview,
    );
    diff_names(
        "action",
        existing
            .plan
            .actions
            .iter()
            .map(|action| action.name.as_str()),
        updated
            .plan
            .actions
            .iter()
            .map(|action| action.name.as_str()),
        &mut preview,
    );
    diff_names(
        "agent_endpoint",
        existing
            .plan
            .agent_endpoints
            .iter()
            .map(|endpoint| endpoint.id.as_str()),
        updated
            .plan
            .agent_endpoints
            .iter()
            .map(|endpoint| endpoint.id.as_str()),
        &mut preview,
    );
    preview.warnings = updated
        .plan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error)
        .cloned()
        .collect();
    preview
}

fn diff_names<'a>(
    kind: &str,
    before: impl Iterator<Item = &'a str>,
    after: impl Iterator<Item = &'a str>,
    preview: &mut SorlaDiffPreview,
) {
    let before = before.collect::<std::collections::BTreeSet<_>>();
    let after = after.collect::<std::collections::BTreeSet<_>>();
    for name in after.difference(&before) {
        preview.added.push(DiffItem {
            kind: kind.to_string(),
            name: (*name).to_string(),
        });
    }
    for name in before.difference(&after) {
        preview.removed.push(DiffItem {
            kind: kind.to_string(),
            name: (*name).to_string(),
        });
    }
}

fn find_record_mut<'a>(
    plan: &'a mut ExpandedSorlaPlan,
    record: &str,
) -> Result<&'a mut RecordPlan, CompileError> {
    plan.records
        .iter_mut()
        .find(|existing| existing.name == record)
        .ok_or_else(|| CompileError::new(format!("record `{record}` does not exist")))
}

fn require_destructive(options: UpdateApplyOptions, operation: String) -> Result<(), CompileError> {
    if options.allow_destructive {
        Ok(())
    } else {
        Err(CompileError::new(format!(
            "destructive semantic operation blocked: {operation}"
        )))
    }
}

impl RecordPlan {
    fn from_update_record(record: &crate::prompt::RecordIntent) -> Self {
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
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-update".to_string(),
                        reason: Some("add_record".to_string()),
                    },
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
                    provenance: Provenance::LlmGenerated {
                        agent: "semantic-update".to_string(),
                        reason: Some("add_record".to_string()),
                    },
                })
                .collect(),
            lifecycle: None,
            provenance: Provenance::LlmGenerated {
                agent: "semantic-update".to_string(),
                reason: Some("add_record".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{
        ANSWERS_V2_VERSION, AnswersMode, AuthoringIntent, CompilerOptions, DomainIntent,
        FieldIntent, RecordIntent,
    };

    #[test]
    fn parses_existing_yaml_and_applies_add_record_update() {
        let existing = existing_sorla_model_from_yaml(
            r#"
package:
  name: maintenance
  version: 0.1.0
records:
  - name: request
    fields:
      - name: id
        type: uuid
        required: true
"#,
        )
        .expect("existing YAML parses");

        let answers = AnswersV2 {
            version: ANSWERS_V2_VERSION.to_string(),
            mode: AnswersMode::Update,
            intent: AuthoringIntent::default(),
            domain: DomainIntent::default(),
            operations: vec![SemanticOperation::AddRecord {
                record: RecordIntent {
                    name: "quote".to_string(),
                    description: None,
                    fields: vec![FieldIntent {
                        name: "amount".to_string(),
                        field_type: "money".to_string(),
                        required: Some(true),
                        values: Vec::new(),
                        description: None,
                    }],
                    relationships: Vec::new(),
                    lifecycle: None,
                },
            }],
            compiler_options: CompilerOptions::default(),
        };

        let update = plan_update_operations(&existing, &answers, UpdateApplyOptions::default())
            .expect("update plans");
        assert!(
            update
                .model
                .plan
                .records
                .iter()
                .any(|record| record.name == "quote")
        );
        assert!(
            update
                .diff
                .added
                .iter()
                .any(|item| item.kind == "record" && item.name == "quote")
        );
        assert!(
            update
                .diff
                .added
                .iter()
                .any(|item| item.kind == "action" && item.name == "quote.create")
        );
    }

    #[test]
    fn destructive_remove_record_is_blocked_by_default() {
        let mut existing = ExistingSorlaModel {
            plan: ExpandedSorlaPlan {
                records: vec![RecordPlan {
                    name: "request".to_string(),
                    description: None,
                    fields: Vec::new(),
                    relationships: Vec::new(),
                    lifecycle: None,
                    provenance: Provenance::ExistingYaml { path: None },
                }],
                ..ExpandedSorlaPlan::default()
            },
        };

        let error = apply_semantic_update_operations(
            &mut existing,
            &[SemanticOperation::RemoveRecord {
                record: "request".to_string(),
            }],
            UpdateApplyOptions::default(),
        )
        .expect_err("remove should be blocked");
        assert!(error.to_string().contains("destructive"));
    }
}
