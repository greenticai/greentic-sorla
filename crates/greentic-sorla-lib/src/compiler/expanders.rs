use super::{
    ActionKind, ActionPlan, AgentEndpointPlan, CompileError, EventKind, EventPlan,
    ExpandedSorlaPlan, ExpansionContext, MetricKind, MetricPlan, MigrationPlan, PolicyPlan,
    ProjectionKind, ProjectionPlan, Provenance, SorlaExpander,
};

pub struct RecordCrudExpander;
pub struct RecordEventExpander;
pub struct LifecycleEventExpander;
pub struct SearchEndpointExpander;
pub struct AgentEndpointExpander;
pub struct ProjectionExpander;
pub struct MetricExpander;
pub struct PolicyDefaultExpander;
pub struct MigrationExpander;

impl SorlaExpander for RecordCrudExpander {
    fn name(&self) -> &'static str {
        "record-crud"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_crud {
            return Ok(());
        }
        let records = plan.records.clone();
        for record in records {
            for (suffix, kind) in [
                ("create", ActionKind::Create),
                ("get", ActionKind::Get),
                ("update", ActionKind::Update),
                ("delete", ActionKind::Delete),
                ("list", ActionKind::List),
            ] {
                push_action(plan, &record.name, suffix, kind, self.name());
            }
        }
        Ok(())
    }
}

impl SorlaExpander for RecordEventExpander {
    fn name(&self) -> &'static str {
        "record-events"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_events {
            return Ok(());
        }
        let records = plan.records.clone();
        for record in records {
            for (suffix, kind) in [
                ("created", EventKind::Created),
                ("updated", EventKind::Updated),
                ("deleted", EventKind::Deleted),
            ] {
                push_event(plan, &record.name, suffix, kind, self.name());
            }
        }
        Ok(())
    }
}

impl SorlaExpander for LifecycleEventExpander {
    fn name(&self) -> &'static str {
        "lifecycle-events"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_lifecycle_events {
            return Ok(());
        }
        let records = plan.records.clone();
        for record in records {
            let Some(lifecycle) = &record.lifecycle else {
                continue;
            };
            for transition in &lifecycle.transitions {
                push_event(
                    plan,
                    &record.name,
                    &transition.to,
                    EventKind::LifecycleTransition,
                    self.name(),
                );
                push_action(
                    plan,
                    &record.name,
                    &verb_for_state(&transition.to),
                    ActionKind::LifecycleTransition,
                    self.name(),
                );
            }
        }
        Ok(())
    }
}

impl SorlaExpander for SearchEndpointExpander {
    fn name(&self) -> &'static str {
        "search-endpoints"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_search {
            return Ok(());
        }
        let records = plan.records.clone();
        for record in records {
            push_action(
                plan,
                &record.name,
                "search",
                ActionKind::Search,
                self.name(),
            );
        }
        Ok(())
    }
}

impl SorlaExpander for AgentEndpointExpander {
    fn name(&self) -> &'static str {
        "agent-endpoints"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_agent_endpoints {
            return Ok(());
        }
        for action in &plan.actions {
            let Some((record, suffix)) = action.name.split_once('.') else {
                continue;
            };
            let id = format!("{}.{}_{}", ctx.naming.agent_prefix, suffix, record);
            if plan
                .agent_endpoints
                .iter()
                .any(|endpoint| endpoint.id == id)
            {
                continue;
            }
            plan.agent_endpoints.push(AgentEndpointPlan {
                id,
                action: action.name.clone(),
                record: action.record.clone(),
                provenance: Provenance::deterministic(self.name(), action.name.clone()),
            });
        }
        Ok(())
    }
}

impl SorlaExpander for ProjectionExpander {
    fn name(&self) -> &'static str {
        "projections"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_projections {
            return Ok(());
        }
        let records = plan.records.clone();
        for record in records {
            push_projection(
                plan,
                &format!("{}_list", record.name),
                &record.name,
                ProjectionKind::List,
                self.name(),
            );
            push_projection(
                plan,
                &format!("{}_detail", record.name),
                &record.name,
                ProjectionKind::Detail,
                self.name(),
            );
            push_projection(
                plan,
                &format!("{}_search", record.name),
                &record.name,
                ProjectionKind::Search,
                self.name(),
            );
            if record.lifecycle.is_some() {
                push_projection(
                    plan,
                    &format!("{}_by_status", record.name),
                    &record.name,
                    ProjectionKind::ByStatus,
                    self.name(),
                );
            }
            for relationship in &record.relationships {
                push_projection(
                    plan,
                    &format!("{}_{}", relationship.target, pluralize(&record.name)),
                    &record.name,
                    ProjectionKind::Relationship,
                    self.name(),
                );
            }
        }
        Ok(())
    }
}

impl SorlaExpander for MetricExpander {
    fn name(&self) -> &'static str {
        "metrics"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_metrics {
            return Ok(());
        }
        let records = plan.records.clone();
        for record in records {
            push_metric(
                plan,
                &format!("count_{}", record.name),
                &record.name,
                MetricKind::Count,
                self.name(),
            );
            push_metric(
                plan,
                &format!("{}_created_per_day", record.name),
                &record.name,
                MetricKind::CreatedPerDay,
                self.name(),
            );
            push_metric(
                plan,
                &format!("{}_updated_per_day", record.name),
                &record.name,
                MetricKind::UpdatedPerDay,
                self.name(),
            );
            let Some(lifecycle) = &record.lifecycle else {
                continue;
            };
            push_metric(
                plan,
                &format!("{}_by_status", record.name),
                &record.name,
                MetricKind::ByStatus,
                self.name(),
            );
            for state in &lifecycle.states {
                push_metric(
                    plan,
                    &format!("average_time_to_{}_{}", record.name, state),
                    &record.name,
                    MetricKind::AverageTimeToState,
                    self.name(),
                );
            }
        }
        Ok(())
    }
}

impl SorlaExpander for PolicyDefaultExpander {
    fn name(&self) -> &'static str {
        "default-policies"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_default_policies {
            return Ok(());
        }
        let records = plan.records.clone();
        for record in records {
            let name = format!("{}_record_access", record.name);
            if plan.policies.iter().any(|policy| policy.name == name) {
                continue;
            }
            plan.policies.push(PolicyPlan {
                name,
                applies_to: Some(record.name.clone()),
                provenance: Provenance::deterministic(self.name(), record.name.clone()),
            });
        }
        Ok(())
    }
}

impl SorlaExpander for MigrationExpander {
    fn name(&self) -> &'static str {
        "migrations"
    }

    fn expand(
        &self,
        ctx: &ExpansionContext,
        plan: &mut ExpandedSorlaPlan,
    ) -> Result<(), CompileError> {
        if !ctx.options.generate_migrations {
            return Ok(());
        }
        let name = match ctx.mode {
            super::CompileMode::Create => "initial-create",
            super::CompileMode::Update => "semantic-update",
        };
        if plan
            .migrations
            .iter()
            .any(|migration| migration.name == name)
        {
            return Ok(());
        }
        plan.migrations.push(MigrationPlan {
            name: name.to_string(),
            compatibility: "additive".to_string(),
            provenance: Provenance::deterministic(self.name(), name),
        });
        Ok(())
    }
}

fn push_action(
    plan: &mut ExpandedSorlaPlan,
    record: &str,
    suffix: &str,
    kind: ActionKind,
    rule: &str,
) {
    let name = format!("{record}.{suffix}");
    if plan.actions.iter().any(|action| action.name == name) {
        return;
    }
    plan.actions.push(ActionPlan {
        name,
        record: Some(record.to_string()),
        kind,
        provenance: Provenance::deterministic(rule, record),
    });
}

fn push_event(
    plan: &mut ExpandedSorlaPlan,
    record: &str,
    suffix: &str,
    kind: EventKind,
    rule: &str,
) {
    let name = format!("{record}.{suffix}");
    if plan.events.iter().any(|event| event.name == name) {
        return;
    }
    plan.events.push(EventPlan {
        name,
        record: Some(record.to_string()),
        kind,
        provenance: Provenance::deterministic(rule, record),
    });
}

fn push_projection(
    plan: &mut ExpandedSorlaPlan,
    name: &str,
    record: &str,
    kind: ProjectionKind,
    rule: &str,
) {
    if plan
        .projections
        .iter()
        .any(|projection| projection.name == name)
    {
        return;
    }
    plan.projections.push(ProjectionPlan {
        name: name.to_string(),
        record: Some(record.to_string()),
        kind,
        provenance: Provenance::deterministic(rule, record),
    });
}

fn push_metric(
    plan: &mut ExpandedSorlaPlan,
    name: &str,
    record: &str,
    kind: MetricKind,
    rule: &str,
) {
    if plan.metrics.iter().any(|metric| metric.name == name) {
        return;
    }
    plan.metrics.push(MetricPlan {
        name: name.to_string(),
        record: Some(record.to_string()),
        kind,
        provenance: Provenance::deterministic(rule, record),
    });
}

fn verb_for_state(state: &str) -> String {
    if let Some(stem) = state.strip_suffix("ied") {
        return format!("{stem}y");
    }
    if let Some(stem) = state.strip_suffix("ed") {
        return if stem.ends_with('v') {
            format!("{stem}e")
        } else {
            stem.to_string()
        };
    }
    state.to_string()
}

fn pluralize(name: &str) -> String {
    if name.ends_with('s') {
        name.to_string()
    } else {
        format!("{name}s")
    }
}
