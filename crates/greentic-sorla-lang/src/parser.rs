use crate::ast::{
    AgentEndpointApprovalMode, AgentEndpointDecl, AgentEndpointRisk, FieldAuthority,
    OntologyBacking, OntologyProviderRequirement, Package, ParseWarning, ParsedPackage,
    ProviderRequirement, Record, RecordSource,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn parse_package(input: &str) -> Result<ParsedPackage, String> {
    let mut package: Package = serde_yaml::from_str(input)
        .map_err(|err| format!("failed to parse SoRLa package: {err}"))?;

    let mut warnings = apply_v0_1_compatibility(&mut package);
    warnings.extend(validate_package(&package)?);

    Ok(ParsedPackage { package, warnings })
}

fn apply_v0_1_compatibility(package: &mut Package) -> Vec<ParseWarning> {
    let mut warnings = Vec::new();

    for record in &mut package.records {
        if record.source.is_none() {
            record.source = Some(RecordSource::Native);
            warnings.push(ParseWarning {
                path: format!("records.{}", record.name),
                message: "missing `source`; defaulted to `native` for additive v0.1 compatibility"
                    .to_string(),
            });
        }
    }

    warnings
}

fn validate_package(package: &Package) -> Result<Vec<ParseWarning>, String> {
    for record in &package.records {
        validate_record(record)?;
    }

    validate_record_references(package)?;
    validate_event_references(package)?;
    validate_projection_references(package)?;
    validate_migrations(package)?;
    validate_provider_requirements(&package.provider_requirements, "provider_requirements")?;
    let mut warnings = validate_ontology(package)?;
    warnings.extend(validate_semantic_aliases(package)?);
    validate_entity_linking(package)?;
    validate_retrieval_bindings(package)?;
    warnings.extend(validate_agent_endpoints(package)?);
    Ok(warnings)
}

fn validate_record(record: &Record) -> Result<(), String> {
    match record
        .source
        .as_ref()
        .expect("source is normalized before validation")
    {
        RecordSource::Native => Ok(()),
        RecordSource::External => {
            if record.external_ref.is_none() {
                return Err(format!(
                    "record `{}` uses source `external` and must declare `external_ref`",
                    record.name
                ));
            }
            Ok(())
        }
        RecordSource::Hybrid => {
            let external_ref = record.external_ref.as_ref().ok_or_else(|| {
                format!(
                    "record `{}` uses source `hybrid` and must declare `external_ref`",
                    record.name
                )
            })?;

            if !external_ref.authoritative {
                return Err(format!(
                    "record `{}` uses source `hybrid` and must mark `external_ref.authoritative: true`",
                    record.name
                ));
            }

            for field in &record.fields {
                if field.authority.is_none() {
                    return Err(format!(
                        "record `{}` uses source `hybrid`; field `{}` must declare `authority: local|external`",
                        record.name, field.name
                    ));
                }
            }

            let has_local = record
                .fields
                .iter()
                .any(|field| field.authority == Some(FieldAuthority::Local));
            let has_external = record
                .fields
                .iter()
                .any(|field| field.authority == Some(FieldAuthority::External));

            if !has_local || !has_external {
                return Err(format!(
                    "record `{}` uses source `hybrid`; fields must include both `local` and `external` authority",
                    record.name
                ));
            }

            Ok(())
        }
    }
}

fn validate_record_references(package: &Package) -> Result<(), String> {
    let record_names = declared_names(package.records.iter().map(|record| record.name.as_str()));
    for (record_index, record) in package.records.iter().enumerate() {
        require_non_empty(
            &record.name,
            &format!("records[{record_index}].name"),
            "record name",
        )?;
        let field_names = declared_names(record.fields.iter().map(|field| field.name.as_str()));
        for (field_index, field) in record.fields.iter().enumerate() {
            require_non_empty(
                &field.name,
                &format!("records[{record_index}].fields[{field_index}].name"),
                "field name",
            )?;
            require_non_empty(
                &field.type_name,
                &format!("records[{record_index}].fields[{field_index}].type"),
                "field type",
            )?;

            if let Some(reference) = &field.references {
                let reference_path =
                    format!("records[{record_index}].fields[{field_index}].references");
                require_non_empty(
                    &reference.record,
                    &format!("{reference_path}.record"),
                    "reference record",
                )?;
                require_non_empty(
                    &reference.field,
                    &format!("{reference_path}.field"),
                    "reference field",
                )?;
                if !record_names.contains(&reference.record) {
                    return Err(format!(
                        "{reference_path}.record: unknown referenced record `{}`",
                        reference.record
                    ));
                }
                let referenced_record = package
                    .records
                    .iter()
                    .find(|candidate| candidate.name == reference.record)
                    .expect("record existence checked above");
                let referenced_fields = declared_names(
                    referenced_record
                        .fields
                        .iter()
                        .map(|field| field.name.as_str()),
                );
                if !referenced_fields.contains(&reference.field) {
                    return Err(format!(
                        "{reference_path}.field: unknown referenced field `{}` on record `{}`",
                        reference.field, reference.record
                    ));
                }
            }
        }

        if field_names.len() != record.fields.len() {
            return Err(format!(
                "records[{record_index}].fields: duplicate field name in record `{}`",
                record.name
            ));
        }
    }
    Ok(())
}

fn validate_event_references(package: &Package) -> Result<(), String> {
    let record_names = declared_names(package.records.iter().map(|record| record.name.as_str()));
    for (index, event) in package.events.iter().enumerate() {
        require_non_empty(&event.name, &format!("events[{index}].name"), "event name")?;
        require_non_empty(
            &event.record,
            &format!("events[{index}].record"),
            "event record",
        )?;
        if !record_names.contains(&event.record) {
            return Err(format!(
                "events[{index}].record: unknown event record `{}`",
                event.record
            ));
        }
    }
    Ok(())
}

fn validate_projection_references(package: &Package) -> Result<(), String> {
    let record_names = declared_names(package.records.iter().map(|record| record.name.as_str()));
    let event_names = declared_names(package.events.iter().map(|event| event.name.as_str()));
    for (index, projection) in package.projections.iter().enumerate() {
        require_non_empty(
            &projection.name,
            &format!("projections[{index}].name"),
            "projection name",
        )?;
        if !record_names.contains(&projection.record) {
            return Err(format!(
                "projections[{index}].record: unknown projection record `{}`",
                projection.record
            ));
        }
        if !event_names.contains(&projection.source_event) {
            return Err(format!(
                "projections[{index}].source_event: unknown projection source event `{}`",
                projection.source_event
            ));
        }
    }
    Ok(())
}

fn validate_migrations(package: &Package) -> Result<(), String> {
    let projection_names = declared_names(
        package
            .projections
            .iter()
            .map(|projection| projection.name.as_str()),
    );
    let record_names = declared_names(package.records.iter().map(|record| record.name.as_str()));

    for (index, migration) in package.migrations.iter().enumerate() {
        let path = format!("migrations[{index}]");
        require_non_empty(&migration.name, &format!("{path}.name"), "migration name")?;
        if let Some(idempotence_key) = &migration.idempotence_key {
            require_non_empty(
                idempotence_key,
                &format!("{path}.idempotence_key"),
                "migration idempotence key",
            )?;
        }
        for (update_index, projection) in migration.projection_updates.iter().enumerate() {
            require_non_empty(
                projection,
                &format!("{path}.projection_updates[{update_index}]"),
                "projection update",
            )?;
            if !projection_names.contains(projection) {
                return Err(format!(
                    "{path}.projection_updates[{update_index}]: unknown projection `{projection}`"
                ));
            }
        }
        for (backfill_index, backfill) in migration.backfills.iter().enumerate() {
            let backfill_path = format!("{path}.backfills[{backfill_index}]");
            require_non_empty(
                &backfill.record,
                &format!("{backfill_path}.record"),
                "backfill record",
            )?;
            require_non_empty(
                &backfill.field,
                &format!("{backfill_path}.field"),
                "backfill field",
            )?;
            if !record_names.contains(&backfill.record) {
                return Err(format!(
                    "{backfill_path}.record: unknown backfill record `{}`",
                    backfill.record
                ));
            }
            let record = package
                .records
                .iter()
                .find(|record| record.name == backfill.record)
                .expect("record existence checked above");
            let field_names = declared_names(record.fields.iter().map(|field| field.name.as_str()));
            if !field_names.contains(&backfill.field) {
                return Err(format!(
                    "{backfill_path}.field: unknown backfill field `{}` on record `{}`",
                    backfill.field, backfill.record
                ));
            }
        }
    }
    Ok(())
}

fn validate_agent_endpoints(package: &Package) -> Result<Vec<ParseWarning>, String> {
    let mut warnings = Vec::new();
    let mut endpoint_ids = BTreeSet::new();
    let action_names = declared_names(package.actions.iter().map(|item| item.name.as_str()));
    let event_names = declared_names(package.events.iter().map(|item| item.name.as_str()));
    let flow_names = declared_names(package.flows.iter().map(|item| item.name.as_str()));
    let policy_names = declared_names(package.policies.iter().map(|item| item.name.as_str()));
    let approval_names = declared_names(package.approvals.iter().map(|item| item.name.as_str()));

    for (index, endpoint) in package.agent_endpoints.iter().enumerate() {
        let endpoint_path = format!("agent_endpoints[{index}]");
        validate_endpoint_identity(endpoint, &endpoint_path, &mut endpoint_ids, &mut warnings)?;
        validate_endpoint_inputs(endpoint, &endpoint_path, &mut warnings)?;
        validate_endpoint_outputs(endpoint, &endpoint_path)?;
        validate_side_effects(endpoint, &endpoint_path)?;
        validate_risk_and_approval(endpoint, &endpoint_path, &mut warnings)?;
        validate_backing_references(
            endpoint,
            &endpoint_path,
            &action_names,
            &event_names,
            &flow_names,
            &policy_names,
            &approval_names,
        )?;
        validate_provider_requirements(
            &endpoint.provider_requirements,
            &format!("{endpoint_path}.provider_requirements"),
        )?;
        validate_operation_plan(endpoint, &endpoint_path, &event_names)?;
        warn_about_endpoint_shape(endpoint, &endpoint_path, &mut warnings);
    }

    Ok(warnings)
}

fn validate_operation_plan(
    endpoint: &AgentEndpointDecl,
    endpoint_path: &str,
    event_names: &BTreeSet<String>,
) -> Result<(), String> {
    let Some(emit) = &endpoint.emits else {
        return Ok(());
    };

    require_non_empty(
        &emit.event,
        &format!("{endpoint_path}.emits.event"),
        "operation event",
    )?;
    require_non_empty(
        &emit.stream,
        &format!("{endpoint_path}.emits.stream"),
        "operation stream",
    )?;
    if !event_names.contains(&emit.event) {
        return Err(format!(
            "{endpoint_path}.emits.event: unknown emitted event `{}`",
            emit.event
        ));
    }

    let input_names = declared_names(endpoint.inputs.iter().map(|input| input.name.as_str()));
    validate_template_references(
        &emit.stream,
        &input_names,
        &format!("{endpoint_path}.emits.stream"),
    )?;
    validate_payload_templates(
        &emit.payload,
        &input_names,
        &format!("{endpoint_path}.emits.payload"),
    )
}

fn validate_payload_templates(
    value: &serde_json::Value,
    input_names: &BTreeSet<String>,
    path: &str,
) -> Result<(), String> {
    match value {
        serde_json::Value::String(template) => {
            validate_template_references(template, input_names, path)
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                validate_payload_templates(item, input_names, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                validate_payload_templates(item, input_names, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_template_references(
    template: &str,
    input_names: &BTreeSet<String>,
    path: &str,
) -> Result<(), String> {
    let mut rest = template;
    while let Some(start) = rest.find("$input.") {
        let after = &rest[start + "$input.".len()..];
        let name: String = after
            .chars()
            .take_while(|char| char.is_ascii_alphanumeric() || matches!(char, '_' | '-'))
            .collect();
        if name.is_empty() || !input_names.contains(&name) {
            return Err(format!("{path}: unknown input reference `$input.{name}`"));
        }
        rest = &after[name.len()..];
    }
    Ok(())
}

fn validate_endpoint_identity(
    endpoint: &AgentEndpointDecl,
    endpoint_path: &str,
    endpoint_ids: &mut BTreeSet<String>,
    warnings: &mut Vec<ParseWarning>,
) -> Result<(), String> {
    require_non_empty(
        &endpoint.id,
        &format!("{endpoint_path}.id"),
        "agent endpoint id",
    )?;
    require_non_empty(
        &endpoint.title,
        &format!("{endpoint_path}.title"),
        "agent endpoint title",
    )?;
    require_non_empty(
        &endpoint.intent,
        &format!("{endpoint_path}.intent"),
        "agent endpoint intent",
    )?;

    if !endpoint_ids.insert(endpoint.id.clone()) {
        return Err(format!(
            "{endpoint_path}.id: duplicate agent endpoint id `{}`",
            endpoint.id
        ));
    }

    if !is_recommended_endpoint_id(&endpoint.id) {
        warnings.push(ParseWarning {
            path: format!("{endpoint_path}.id"),
            message: format!(
                "agent endpoint id `{}` should match `[a-z][a-z0-9_-]*`",
                endpoint.id
            ),
        });
    }

    Ok(())
}

fn validate_endpoint_inputs(
    endpoint: &AgentEndpointDecl,
    endpoint_path: &str,
    warnings: &mut Vec<ParseWarning>,
) -> Result<(), String> {
    let mut names = BTreeSet::new();
    let mut seen_optional = false;

    for (index, input) in endpoint.inputs.iter().enumerate() {
        let input_path = format!("{endpoint_path}.inputs[{index}]");
        require_non_empty(
            &input.name,
            &format!("{input_path}.name"),
            "agent endpoint input name",
        )?;
        require_non_empty(
            &input.type_name,
            &format!("{input_path}.type"),
            "agent endpoint input type",
        )?;

        if !names.insert(input.name.clone()) {
            return Err(format!(
                "{input_path}.name: duplicate input name `{}` in agent endpoint `{}`",
                input.name, endpoint.id
            ));
        }

        if input.required && seen_optional {
            warnings.push(ParseWarning {
                path: format!("{input_path}.required"),
                message: format!(
                    "required input `{}` appears after an optional input in agent endpoint `{}`",
                    input.name, endpoint.id
                ),
            });
        }
        seen_optional |= !input.required;

        let mut enum_values = BTreeSet::new();
        for (value_index, value) in input.enum_values.iter().enumerate() {
            require_non_empty(
                value,
                &format!("{input_path}.enum_values[{value_index}]"),
                "agent endpoint input enum value",
            )?;
            if !enum_values.insert(value.clone()) {
                return Err(format!(
                    "{input_path}.enum_values[{value_index}]: duplicate enum value `{value}` in input `{}`",
                    input.name
                ));
            }
        }
    }

    Ok(())
}

fn validate_endpoint_outputs(
    endpoint: &AgentEndpointDecl,
    endpoint_path: &str,
) -> Result<(), String> {
    let mut names = BTreeSet::new();

    for (index, output) in endpoint.outputs.iter().enumerate() {
        let output_path = format!("{endpoint_path}.outputs[{index}]");
        require_non_empty(
            &output.name,
            &format!("{output_path}.name"),
            "agent endpoint output name",
        )?;
        require_non_empty(
            &output.type_name,
            &format!("{output_path}.type"),
            "agent endpoint output type",
        )?;

        if !names.insert(output.name.clone()) {
            return Err(format!(
                "{output_path}.name: duplicate output name `{}` in agent endpoint `{}`",
                output.name, endpoint.id
            ));
        }
    }

    Ok(())
}

fn validate_side_effects(endpoint: &AgentEndpointDecl, endpoint_path: &str) -> Result<(), String> {
    for (index, side_effect) in endpoint.side_effects.iter().enumerate() {
        require_non_empty(
            side_effect,
            &format!("{endpoint_path}.side_effects[{index}]"),
            "agent endpoint side effect",
        )?;
    }

    if endpoint.risk == AgentEndpointRisk::High && endpoint.side_effects.is_empty() {
        return Err(format!(
            "{endpoint_path}.side_effects: high-risk agent endpoint `{}` must declare at least one side effect",
            endpoint.id
        ));
    }

    Ok(())
}

fn validate_risk_and_approval(
    endpoint: &AgentEndpointDecl,
    endpoint_path: &str,
    warnings: &mut Vec<ParseWarning>,
) -> Result<(), String> {
    if endpoint.risk == AgentEndpointRisk::High
        && !matches!(
            endpoint.approval,
            AgentEndpointApprovalMode::Required | AgentEndpointApprovalMode::PolicyDriven
        )
    {
        return Err(format!(
            "{endpoint_path}.approval: high-risk agent endpoint `{}` must use approval: required or approval: policy-driven",
            endpoint.id
        ));
    }

    if endpoint.approval == AgentEndpointApprovalMode::Required
        && endpoint.backing.approvals.is_empty()
    {
        warnings.push(ParseWarning {
            path: format!("{endpoint_path}.backing.approvals"),
            message: format!(
                "agent endpoint `{}` uses approval: required but references no backing approvals",
                endpoint.id
            ),
        });
    }

    Ok(())
}

fn validate_backing_references(
    endpoint: &AgentEndpointDecl,
    endpoint_path: &str,
    action_names: &BTreeSet<String>,
    event_names: &BTreeSet<String>,
    flow_names: &BTreeSet<String>,
    policy_names: &BTreeSet<String>,
    approval_names: &BTreeSet<String>,
) -> Result<(), String> {
    validate_named_refs(
        &endpoint.backing.actions,
        action_names,
        &format!("{endpoint_path}.backing.actions"),
        "action",
    )?;
    validate_named_refs(
        &endpoint.backing.events,
        event_names,
        &format!("{endpoint_path}.backing.events"),
        "event",
    )?;
    validate_named_refs(
        &endpoint.backing.flows,
        flow_names,
        &format!("{endpoint_path}.backing.flows"),
        "flow",
    )?;
    validate_named_refs(
        &endpoint.backing.policies,
        policy_names,
        &format!("{endpoint_path}.backing.policies"),
        "policy",
    )?;
    validate_named_refs(
        &endpoint.backing.approvals,
        approval_names,
        &format!("{endpoint_path}.backing.approvals"),
        "approval",
    )
}

fn validate_named_refs(
    refs: &[String],
    declared: &BTreeSet<String>,
    path: &str,
    kind: &str,
) -> Result<(), String> {
    for (index, reference) in refs.iter().enumerate() {
        require_non_empty(reference, &format!("{path}[{index}]"), kind)?;
        if !declared.contains(reference) {
            return Err(format!(
                "{path}[{index}]: unknown backing {kind} reference `{reference}`"
            ));
        }
    }

    Ok(())
}

fn validate_provider_requirements(
    requirements: &[ProviderRequirement],
    path: &str,
) -> Result<(), String> {
    for (index, requirement) in requirements.iter().enumerate() {
        let requirement_path = format!("{path}[{index}]");
        require_non_empty(
            &requirement.category,
            &format!("{requirement_path}.category"),
            "provider requirement category",
        )?;

        let mut capabilities = BTreeSet::new();
        for (capability_index, capability) in requirement.capabilities.iter().enumerate() {
            require_non_empty(
                capability,
                &format!("{requirement_path}.capabilities[{capability_index}]"),
                "provider requirement capability",
            )?;
            if !capabilities.insert(capability.clone()) {
                return Err(format!(
                    "{requirement_path}.capabilities[{capability_index}]: duplicate provider capability `{capability}` in category `{}`",
                    requirement.category
                ));
            }
        }
    }

    Ok(())
}

fn validate_ontology(package: &Package) -> Result<Vec<ParseWarning>, String> {
    let Some(ontology) = &package.ontology else {
        return Ok(Vec::new());
    };

    require_non_empty(&ontology.schema, "ontology.schema", "ontology schema")?;
    if ontology.schema != "greentic.sorla.ontology.v1" {
        return Err(format!(
            "ontology.schema: unsupported ontology schema `{}`",
            ontology.schema
        ));
    }

    let record_fields = package
        .records
        .iter()
        .map(|record| {
            (
                record.name.clone(),
                declared_names(record.fields.iter().map(|field| field.name.as_str())),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut concept_ids = BTreeSet::new();
    for (concept_index, concept) in ontology.concepts.iter().enumerate() {
        let path = format!("ontology.concepts[{concept_index}]");
        validate_stable_id(&concept.id, &format!("{path}.id"))?;
        if !concept_ids.insert(concept.id.clone()) {
            return Err(format!(
                "{path}.id: duplicate ontology concept id `{}`",
                concept.id
            ));
        }
        for (extends_index, parent) in concept.extends.iter().enumerate() {
            validate_stable_id(parent, &format!("{path}.extends[{extends_index}]"))?;
        }
        if let Some(backing) = &concept.backed_by {
            validate_ontology_backing(backing, &record_fields, &format!("{path}.backed_by"))?;
        }
        validate_ontology_provider_requirements(
            &concept.provider_requirements,
            &format!("{path}.provider_requirements"),
        )?;
        for (hook_index, hook) in concept.policy_hooks.iter().enumerate() {
            require_non_empty(
                &hook.policy,
                &format!("{path}.policy_hooks[{hook_index}].policy"),
                "ontology policy hook",
            )?;
        }
    }

    for (concept_index, concept) in ontology.concepts.iter().enumerate() {
        let path = format!("ontology.concepts[{concept_index}]");
        for (extends_index, parent) in concept.extends.iter().enumerate() {
            if !concept_ids.contains(parent) {
                return Err(format!(
                    "{path}.extends[{extends_index}]: unknown ontology concept `{parent}`"
                ));
            }
        }
    }
    validate_ontology_extends_acyclic(ontology, &concept_ids)?;

    let mut relationship_ids = BTreeSet::new();
    for (relationship_index, relationship) in ontology.relationships.iter().enumerate() {
        let path = format!("ontology.relationships[{relationship_index}]");
        validate_stable_id(&relationship.id, &format!("{path}.id"))?;
        if !relationship_ids.insert(relationship.id.clone()) {
            return Err(format!(
                "{path}.id: duplicate ontology relationship id `{}`",
                relationship.id
            ));
        }
        validate_stable_id(&relationship.from, &format!("{path}.from"))?;
        validate_stable_id(&relationship.to, &format!("{path}.to"))?;
        if !concept_ids.contains(&relationship.from) {
            return Err(format!(
                "{path}.from: unknown ontology concept `{}`",
                relationship.from
            ));
        }
        if !concept_ids.contains(&relationship.to) {
            return Err(format!(
                "{path}.to: unknown ontology concept `{}`",
                relationship.to
            ));
        }
        if let Some(backing) = &relationship.backed_by {
            validate_ontology_backing(backing, &record_fields, &format!("{path}.backed_by"))?;
        }
        validate_ontology_provider_requirements(
            &relationship.provider_requirements,
            &format!("{path}.provider_requirements"),
        )?;
        for (hook_index, hook) in relationship.policy_hooks.iter().enumerate() {
            require_non_empty(
                &hook.policy,
                &format!("{path}.policy_hooks[{hook_index}].policy"),
                "ontology policy hook",
            )?;
        }
    }

    let mut constraint_ids = BTreeSet::new();
    for (constraint_index, constraint) in ontology.constraints.iter().enumerate() {
        let path = format!("ontology.constraints[{constraint_index}]");
        validate_stable_id(&constraint.id, &format!("{path}.id"))?;
        if !constraint_ids.insert(constraint.id.clone()) {
            return Err(format!(
                "{path}.id: duplicate ontology constraint id `{}`",
                constraint.id
            ));
        }
        if !concept_ids.contains(&constraint.applies_to.concept) {
            return Err(format!(
                "{path}.applies_to.concept: unknown ontology concept `{}`",
                constraint.applies_to.concept
            ));
        }
        if let Some(policy) = &constraint.requires_policy {
            require_non_empty(
                policy,
                &format!("{path}.requires_policy"),
                "ontology required policy",
            )?;
        }
    }

    Ok(Vec::new())
}

fn validate_semantic_aliases(package: &Package) -> Result<Vec<ParseWarning>, String> {
    let Some(aliases) = &package.semantic_aliases else {
        return Ok(Vec::new());
    };
    let ontology = package
        .ontology
        .as_ref()
        .ok_or_else(|| "semantic_aliases require an ontology section".to_string())?;
    let concept_ids = declared_names(ontology.concepts.iter().map(|concept| concept.id.as_str()));
    let relationship_ids = declared_names(
        ontology
            .relationships
            .iter()
            .map(|relationship| relationship.id.as_str()),
    );
    let mut warnings = Vec::new();
    let mut normalized_targets: BTreeMap<String, String> = BTreeMap::new();

    validate_alias_map(
        &aliases.concepts,
        &concept_ids,
        "semantic_aliases.concepts",
        "concept",
        &mut normalized_targets,
        &mut warnings,
    )?;
    validate_alias_map(
        &aliases.relationships,
        &relationship_ids,
        "semantic_aliases.relationships",
        "relationship",
        &mut normalized_targets,
        &mut warnings,
    )?;
    Ok(warnings)
}

fn validate_alias_map(
    aliases: &BTreeMap<String, Vec<String>>,
    known_targets: &BTreeSet<String>,
    path: &str,
    target_kind: &str,
    normalized_targets: &mut BTreeMap<String, String>,
    warnings: &mut Vec<ParseWarning>,
) -> Result<(), String> {
    for (target, values) in aliases {
        if !known_targets.contains(target) {
            return Err(format!(
                "{path}.{target}: unknown ontology {target_kind} `{target}`"
            ));
        }
        let mut target_aliases = BTreeSet::new();
        for (alias_index, alias) in values.iter().enumerate() {
            require_non_empty(
                alias,
                &format!("{path}.{target}[{alias_index}]"),
                "semantic alias",
            )?;
            let normalized = normalize_semantic_alias(alias);
            let target_key = format!("{target_kind}:{target}");
            if let Some(existing) = normalized_targets.get(&normalized) {
                if existing != &target_key {
                    return Err(format!(
                        "{path}.{target}[{alias_index}]: semantic alias `{alias}` collides with `{existing}` after normalization"
                    ));
                }
                warnings.push(ParseWarning {
                    path: format!("{path}.{target}[{alias_index}]"),
                    message: format!(
                        "duplicate semantic alias `{alias}` was de-duplicated after normalization"
                    ),
                });
            } else {
                normalized_targets.insert(normalized.clone(), target_key);
            }
            if !target_aliases.insert(normalized) {
                warnings.push(ParseWarning {
                    path: format!("{path}.{target}[{alias_index}]"),
                    message: format!(
                        "duplicate semantic alias `{alias}` was de-duplicated after normalization"
                    ),
                });
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

fn validate_entity_linking(package: &Package) -> Result<(), String> {
    let Some(entity_linking) = &package.entity_linking else {
        return Ok(());
    };
    let ontology = package
        .ontology
        .as_ref()
        .ok_or_else(|| "entity_linking requires an ontology section".to_string())?;
    let concepts = ontology
        .concepts
        .iter()
        .map(|concept| (concept.id.as_str(), concept))
        .collect::<BTreeMap<_, _>>();
    let record_fields = package
        .records
        .iter()
        .map(|record| {
            (
                record.name.clone(),
                declared_names(record.fields.iter().map(|field| field.name.as_str())),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut strategy_ids = BTreeSet::new();

    for (strategy_index, strategy) in entity_linking.strategies.iter().enumerate() {
        let path = format!("entity_linking.strategies[{strategy_index}]");
        validate_stable_id(&strategy.id, &format!("{path}.id"))?;
        if !strategy_ids.insert(strategy.id.clone()) {
            return Err(format!(
                "{path}.id: duplicate entity-linking strategy id `{}`",
                strategy.id
            ));
        }
        let concept = concepts.get(strategy.applies_to.as_str()).ok_or_else(|| {
            format!(
                "{path}.applies_to: unknown ontology concept `{}`",
                strategy.applies_to
            )
        })?;
        require_non_empty(
            &strategy.match_fields.source_field,
            &format!("{path}.match.source_field"),
            "entity-linking source field",
        )?;
        require_non_empty(
            &strategy.match_fields.target_field,
            &format!("{path}.match.target_field"),
            "entity-linking target field",
        )?;
        match &concept.backed_by {
            Some(backing) => {
                let fields = record_fields.get(&backing.record).ok_or_else(|| {
                    format!(
                        "{path}.applies_to: concept `{}` is backed by unknown record `{}`",
                        concept.id, backing.record
                    )
                })?;
                if !fields.contains(&strategy.match_fields.target_field) {
                    return Err(format!(
                        "{path}.match.target_field: unknown field `{}` on backing record `{}`",
                        strategy.match_fields.target_field, backing.record
                    ));
                }
            }
            None => {
                let Some(source_type) = &strategy.source_type else {
                    return Err(format!(
                        "{path}.source_type: unbacked concept `{}` requires an explicit non-record source type",
                        concept.id
                    ));
                };
                require_non_empty(source_type, &format!("{path}.source_type"), "source type")?;
                if source_type == "record" {
                    return Err(format!(
                        "{path}.source_type: unbacked concept `{}` requires a non-record source type",
                        concept.id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_retrieval_bindings(package: &Package) -> Result<(), String> {
    let Some(bindings) = &package.retrieval_bindings else {
        return Ok(());
    };
    let ontology = package
        .ontology
        .as_ref()
        .ok_or_else(|| "retrieval_bindings require an ontology section".to_string())?;
    require_non_empty(
        &bindings.schema,
        "retrieval_bindings.schema",
        "retrieval bindings schema",
    )?;
    if bindings.schema != "greentic.sorla.retrieval-bindings.v1" {
        return Err(format!(
            "retrieval_bindings.schema: unsupported retrieval bindings schema `{}`",
            bindings.schema
        ));
    }
    let concept_ids = declared_names(ontology.concepts.iter().map(|concept| concept.id.as_str()));
    let relationship_ids = declared_names(
        ontology
            .relationships
            .iter()
            .map(|relationship| relationship.id.as_str()),
    );

    let mut provider_ids = BTreeSet::new();
    for (provider_index, provider) in bindings.providers.iter().enumerate() {
        let path = format!("retrieval_bindings.providers[{provider_index}]");
        validate_stable_id(&provider.id, &format!("{path}.id"))?;
        if !provider_ids.insert(provider.id.clone()) {
            return Err(format!(
                "{path}.id: duplicate retrieval provider id `{}`",
                provider.id
            ));
        }
        require_non_empty(
            &provider.category,
            &format!("{path}.category"),
            "provider category",
        )?;
        reject_secret_like_value(&provider.category, &format!("{path}.category"))?;
        let mut capabilities = BTreeSet::new();
        for (capability_index, capability) in provider.required_capabilities.iter().enumerate() {
            let capability_path = format!("{path}.required_capabilities[{capability_index}]");
            require_non_empty(capability, &capability_path, "provider capability")?;
            reject_secret_like_value(capability, &capability_path)?;
            if !capabilities.insert(capability.clone()) {
                return Err(format!(
                    "{capability_path}: duplicate retrieval provider capability `{capability}`"
                ));
            }
        }
    }

    let mut scope_ids = BTreeSet::new();
    for (scope_index, scope) in bindings.scopes.iter().enumerate() {
        let path = format!("retrieval_bindings.scopes[{scope_index}]");
        validate_stable_id(&scope.id, &format!("{path}.id"))?;
        if !scope_ids.insert(scope.id.clone()) {
            return Err(format!(
                "{path}.id: duplicate retrieval scope id `{}`",
                scope.id
            ));
        }
        if !provider_ids.contains(&scope.provider) {
            return Err(format!(
                "{path}.provider: unknown retrieval provider `{}`",
                scope.provider
            ));
        }
        validate_retrieval_scope_target(
            &scope.applies_to.concept,
            &scope.applies_to.relationship,
            &concept_ids,
            &relationship_ids,
            &format!("{path}.applies_to"),
        )?;
        if let Some(filters) = &scope.filters
            && let Some(entity_scope) = &filters.entity_scope
        {
            for (rule_index, rule) in entity_scope.include_related.iter().enumerate() {
                let rule_path =
                    format!("{path}.filters.entity_scope.include_related[{rule_index}]");
                if !relationship_ids.contains(&rule.relationship) {
                    return Err(format!(
                        "{rule_path}.relationship: unknown ontology relationship `{}`",
                        rule.relationship
                    ));
                }
                if rule.max_depth > 5 {
                    return Err(format!(
                        "{rule_path}.max_depth: retrieval traversal depth must be between 0 and 5"
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_retrieval_scope_target(
    concept: &Option<String>,
    relationship: &Option<String>,
    concept_ids: &BTreeSet<String>,
    relationship_ids: &BTreeSet<String>,
    path: &str,
) -> Result<(), String> {
    match (concept, relationship) {
        (Some(concept), None) => {
            if !concept_ids.contains(concept) {
                return Err(format!(
                    "{path}.concept: unknown ontology concept `{concept}`"
                ));
            }
            Ok(())
        }
        (None, Some(relationship)) => {
            if !relationship_ids.contains(relationship) {
                return Err(format!(
                    "{path}.relationship: unknown ontology relationship `{relationship}`"
                ));
            }
            Ok(())
        }
        (None, None) => Err(format!("{path}: must declare `concept` or `relationship`")),
        (Some(_), Some(_)) => Err(format!(
            "{path}: must declare exactly one of `concept` or `relationship`"
        )),
    }
}

fn reject_secret_like_value(value: &str, path: &str) -> Result<(), String> {
    let lower = value.to_ascii_lowercase();
    for marker in ["secret", "password", "token", "api_key", "client_secret"] {
        if lower.contains(marker) {
            return Err(format!(
                "{path}: retrieval bindings must not include `{marker}`"
            ));
        }
    }
    Ok(())
}

fn validate_ontology_backing(
    backing: &OntologyBacking,
    record_fields: &BTreeMap<String, BTreeSet<String>>,
    path: &str,
) -> Result<(), String> {
    require_non_empty(
        &backing.record,
        &format!("{path}.record"),
        "ontology backing record",
    )?;
    let Some(fields) = record_fields.get(&backing.record) else {
        return Err(format!(
            "{path}.record: unknown ontology backing record `{}`",
            backing.record
        ));
    };
    if let Some(from_field) = &backing.from_field {
        require_non_empty(
            from_field,
            &format!("{path}.from_field"),
            "ontology backing from field",
        )?;
        if !fields.contains(from_field) {
            return Err(format!(
                "{path}.from_field: unknown field `{from_field}` on record `{}`",
                backing.record
            ));
        }
    }
    if let Some(to_field) = &backing.to_field {
        require_non_empty(
            to_field,
            &format!("{path}.to_field"),
            "ontology backing to field",
        )?;
        if !fields.contains(to_field) {
            return Err(format!(
                "{path}.to_field: unknown field `{to_field}` on record `{}`",
                backing.record
            ));
        }
    }
    Ok(())
}

fn validate_ontology_provider_requirements(
    requirements: &[OntologyProviderRequirement],
    path: &str,
) -> Result<(), String> {
    for (index, requirement) in requirements.iter().enumerate() {
        let requirement_path = format!("{path}[{index}]");
        require_non_empty(
            &requirement.category,
            &format!("{requirement_path}.category"),
            "ontology provider requirement category",
        )?;
        let mut capabilities = BTreeSet::new();
        for (capability_index, capability) in requirement.capabilities.iter().enumerate() {
            require_non_empty(
                capability,
                &format!("{requirement_path}.capabilities[{capability_index}]"),
                "ontology provider requirement capability",
            )?;
            if !capabilities.insert(capability.clone()) {
                return Err(format!(
                    "{requirement_path}.capabilities[{capability_index}]: duplicate ontology provider capability `{capability}` in category `{}`",
                    requirement.category
                ));
            }
        }
    }
    Ok(())
}

fn validate_ontology_extends_acyclic(
    ontology: &crate::ast::OntologyModel,
    concept_ids: &BTreeSet<String>,
) -> Result<(), String> {
    let parents = ontology
        .concepts
        .iter()
        .map(|concept| (concept.id.as_str(), concept.extends.as_slice()))
        .collect::<BTreeMap<_, _>>();

    for concept in concept_ids {
        let mut visiting = BTreeSet::new();
        visit_ontology_parent(concept, &parents, &mut visiting, &mut BTreeSet::new())?;
    }
    Ok(())
}

fn visit_ontology_parent<'a>(
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
            "ontology.concepts: inheritance cycle includes concept `{concept}`"
        ));
    }
    if let Some(parent_ids) = parents.get(concept) {
        for parent in *parent_ids {
            visit_ontology_parent(parent, parents, visiting, visited)?;
        }
    }
    visiting.remove(concept);
    visited.insert(concept);
    Ok(())
}

fn validate_stable_id(value: &str, path: &str) -> Result<(), String> {
    require_non_empty(value, path, "stable identifier")?;
    if !is_url_safe_identifier(value) {
        return Err(format!("{path}: `{value}` must be URL-safe"));
    }
    Ok(())
}

fn is_url_safe_identifier(value: &str) -> bool {
    value
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '_' | '-'))
}

fn warn_about_endpoint_shape(
    endpoint: &AgentEndpointDecl,
    endpoint_path: &str,
    warnings: &mut Vec<ParseWarning>,
) {
    if endpoint.examples.is_empty() {
        warnings.push(ParseWarning {
            path: format!("{endpoint_path}.examples"),
            message: format!("agent endpoint `{}` has no examples", endpoint.id),
        });
    }

    if endpoint.outputs.is_empty()
        && (endpoint.agent_visibility.openapi
            || endpoint.agent_visibility.arazzo
            || endpoint.agent_visibility.mcp)
    {
        warnings.push(ParseWarning {
            path: format!("{endpoint_path}.outputs"),
            message: format!(
                "agent endpoint `{}` exposes agent-facing exports but has no outputs",
                endpoint.id
            ),
        });
    }

    let has_sensitive_inputs = endpoint.inputs.iter().any(|input| input.sensitive);
    let has_approval_or_policy = endpoint.approval != AgentEndpointApprovalMode::None
        || !endpoint.backing.approvals.is_empty()
        || !endpoint.backing.policies.is_empty();
    if has_sensitive_inputs && !has_approval_or_policy {
        warnings.push(ParseWarning {
            path: format!("{endpoint_path}.inputs"),
            message: format!(
                "agent endpoint `{}` has sensitive inputs but no approval or policy reference",
                endpoint.id
            ),
        });
    }
}

fn require_non_empty(value: &str, path: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{path}: {label} must be non-empty"));
    }
    Ok(())
}

fn is_recommended_endpoint_id(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|char| {
            char.is_ascii_lowercase() || char.is_ascii_digit() || matches!(char, '_' | '-')
        })
}

fn declared_names<'a>(names: impl Iterator<Item = &'a str>) -> BTreeSet<String> {
    names.map(str::to_string).collect()
}
