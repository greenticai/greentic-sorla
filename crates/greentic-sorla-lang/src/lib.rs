pub mod ast;
pub mod parser;

pub const PRODUCT_NAME: &str = "SoRLa";
pub const PRODUCT_LINE: &str = "wizard-first";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductBoundary {
    pub providers_live_in: &'static str,
    pub owns_provider_implementations: bool,
}

pub fn product_boundary() -> ProductBoundary {
    ProductBoundary {
        providers_live_in: "greentic-sorla-providers",
        owns_provider_implementations: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        AgentEndpointApprovalMode, AgentEndpointRisk, FieldAuthority, ProjectionMode, RecordSource,
    };
    use crate::parser::parse_package;

    #[test]
    fn product_boundary_points_to_providers_repo() {
        let boundary = product_boundary();
        assert_eq!(boundary.providers_live_in, "greentic-sorla-providers");
        assert!(!boundary.owns_provider_implementations);
    }

    #[test]
    fn parses_native_v0_1_style_record_with_warning() {
        let parsed = parse_package(
            r#"
package:
  name: leasing
  version: 0.2.0
records:
  - name: Tenant
    fields:
      - name: tenant_id
        type: string
events:
  - name: TenantRegistered
    record: Tenant
    emits:
      - name: tenant_id
        type: string
projections:
  - name: TenantCurrentState
    record: Tenant
    source_event: TenantRegistered
"#,
        )
        .expect("native record should parse");

        assert_eq!(parsed.package.records[0].source, Some(RecordSource::Native));
        assert!(parsed.package.agent_endpoints.is_empty());
        assert_eq!(parsed.warnings.len(), 1);
        assert_eq!(
            parsed.package.projections[0].mode,
            ProjectionMode::CurrentState
        );
    }

    #[test]
    fn parses_external_record_with_authoritative_reference() {
        let parsed = parse_package(
            r#"
package:
  name: fitout
  version: 0.2.0
records:
  - name: LeaseCase
    source: external
    external_ref:
      system: sharepoint
      key: lease_case_id
      authoritative: true
    fields:
      - name: lease_case_id
        type: string
provider_requirements:
  - category: external-ref
    capabilities:
      - lookup
events:
  - name: LeaseCaseSynced
    record: LeaseCase
"#,
        )
        .expect("external record should parse");

        assert_eq!(
            parsed.package.records[0].source,
            Some(RecordSource::External)
        );
        assert!(
            parsed.package.records[0]
                .external_ref
                .as_ref()
                .expect("external ref present")
                .authoritative
        );
        assert_eq!(parsed.package.provider_requirements.len(), 1);
    }

    #[test]
    fn parses_agent_endpoint_language_model() {
        let parsed = parse_package(
            r#"
package:
  name: website-lead-capture
  version: 0.2.0
records:
  - name: Contact
    source: hybrid
    external_ref:
      system: hubspot
      key: email
      authoritative: true
    fields:
      - name: email
        type: string
        authority: external
      - name: problem_to_solve
        type: string
        authority: local
actions:
  - name: UpsertContact
events:
  - name: ContactCaptured
    record: Contact
    kind: integration
approvals:
  - name: ReviewHighValuePartner
agent_endpoints:
  - id: create_customer_contact
    title: Create customer contact
    intent: Capture a customer website enquiry and create or update the CRM contact.
    description: Use this when a visitor wants to learn more as a customer.
    inputs:
      - name: email
        type: string
        required: true
        sensitive: true
      - name: company_size
        type: string
        enum_values:
          - small
          - enterprise
    outputs:
      - name: contact_id
        type: string
        description: CRM contact identifier.
    side_effects:
      - crm.contact.upsert
      - event.ContactCaptured
    risk: medium
    approval: policy-driven
    provider_requirements:
      - category: crm
        capabilities:
          - contacts.read
          - contacts.write
    backing:
      actions:
        - UpsertContact
      events:
        - ContactCaptured
      approvals:
        - ReviewHighValuePartner
    agent_visibility:
      openapi: true
      arazzo: false
      mcp: true
      llms_txt: true
    examples:
      - name: customer-lead
        summary: Capture an inbound website lead.
        input:
          email: buyer@example.com
          company_size: enterprise
        expected_output:
          contact_id: contact-123
"#,
        )
        .expect("agent endpoint should parse");

        let endpoint = &parsed.package.agent_endpoints[0];
        assert_eq!(endpoint.id, "create_customer_contact");
        assert_eq!(endpoint.inputs[0].type_name, "string");
        assert!(endpoint.inputs[0].required);
        assert!(endpoint.inputs[0].sensitive);
        assert_eq!(endpoint.inputs[1].enum_values, ["small", "enterprise"]);
        assert_eq!(endpoint.outputs[0].name, "contact_id");
        assert_eq!(endpoint.risk, AgentEndpointRisk::Medium);
        assert_eq!(endpoint.approval, AgentEndpointApprovalMode::PolicyDriven);
        assert_eq!(endpoint.provider_requirements[0].category, "crm");
        assert_eq!(endpoint.backing.actions, ["UpsertContact"]);
        assert_eq!(endpoint.backing.events, ["ContactCaptured"]);
        assert_eq!(endpoint.backing.approvals, ["ReviewHighValuePartner"]);
        assert!(!endpoint.agent_visibility.arazzo);
        assert_eq!(
            endpoint.examples[0].input["email"].as_str(),
            Some("buyer@example.com")
        );
        assert_eq!(
            endpoint.examples[0].expected_output["contact_id"].as_str(),
            Some("contact-123")
        );
    }

    #[test]
    fn agent_endpoint_defaults_are_language_facing() {
        let parsed = parse_package(
            r#"
package:
  name: minimal-agent
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a CRM contact.
"#,
        )
        .expect("minimal endpoint should parse");

        let endpoint = &parsed.package.agent_endpoints[0];
        assert_eq!(endpoint.risk, AgentEndpointRisk::Low);
        assert_eq!(endpoint.approval, AgentEndpointApprovalMode::None);
        assert!(endpoint.inputs.is_empty());
        assert!(endpoint.outputs.is_empty());
        assert!(endpoint.backing.actions.is_empty());
        assert!(endpoint.agent_visibility.openapi);
        assert!(endpoint.agent_visibility.arazzo);
        assert!(endpoint.agent_visibility.mcp);
        assert!(endpoint.agent_visibility.llms_txt);
    }

    #[test]
    fn rejects_duplicate_agent_endpoint_ids() {
        let error = parse_package(
            r#"
package:
  name: duplicate-endpoints
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
  - id: create_contact
    title: Duplicate
    intent: Duplicate endpoint.
"#,
        )
        .expect_err("duplicate endpoint IDs should be rejected");

        assert!(error.contains("agent_endpoints[1].id"));
        assert!(error.contains("duplicate agent endpoint id `create_contact`"));
    }

    #[test]
    fn rejects_duplicate_agent_endpoint_input_and_output_names() {
        let duplicate_input = parse_package(
            r#"
package:
  name: duplicate-inputs
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    inputs:
      - name: email
        type: string
      - name: email
        type: string
"#,
        )
        .expect_err("duplicate input names should be rejected");
        assert!(duplicate_input.contains("agent_endpoints[0].inputs[1].name"));
        assert!(duplicate_input.contains("duplicate input name `email`"));

        let duplicate_output = parse_package(
            r#"
package:
  name: duplicate-outputs
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    outputs:
      - name: contact_id
        type: string
      - name: contact_id
        type: string
"#,
        )
        .expect_err("duplicate output names should be rejected");
        assert!(duplicate_output.contains("agent_endpoints[0].outputs[1].name"));
        assert!(duplicate_output.contains("duplicate output name `contact_id`"));
    }

    #[test]
    fn rejects_empty_agent_endpoint_intent() {
        let error = parse_package(
            r#"
package:
  name: empty-intent
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: ""
"#,
        )
        .expect_err("empty endpoint intent should be rejected");

        assert!(error.contains("agent_endpoints[0].intent"));
        assert!(error.contains("agent endpoint intent must be non-empty"));
    }

    #[test]
    fn rejects_high_risk_agent_endpoint_without_required_or_policy_approval() {
        let error = parse_package(
            r#"
package:
  name: risky-endpoint
  version: 0.2.0
agent_endpoints:
  - id: delete_customer
    title: Delete customer
    intent: Delete a customer record.
    risk: high
    approval: none
    side_effects:
      - crm.contact.delete
"#,
        )
        .expect_err("high-risk endpoint without approval should be rejected");

        assert!(error.contains("agent_endpoints[0].approval"));
        assert!(error.contains(
            "high-risk agent endpoint `delete_customer` must use approval: required or approval: policy-driven"
        ));
    }

    #[test]
    fn rejects_unknown_agent_endpoint_backing_references() {
        let action_error = parse_package(
            r#"
package:
  name: bad-action-ref
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    backing:
      actions:
        - MissingAction
"#,
        )
        .expect_err("unknown backing action should be rejected");
        assert!(action_error.contains("agent_endpoints[0].backing.actions[0]"));
        assert!(action_error.contains("unknown backing action reference `MissingAction`"));

        let event_error = parse_package(
            r#"
package:
  name: bad-event-ref
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    backing:
      events:
        - MissingEvent
"#,
        )
        .expect_err("unknown backing event should be rejected");
        assert!(event_error.contains("agent_endpoints[0].backing.events[0]"));
        assert!(event_error.contains("unknown backing event reference `MissingEvent`"));
    }

    #[test]
    fn validates_agent_endpoint_provider_requirements() {
        let empty_category = parse_package(
            r#"
package:
  name: empty-provider-category
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    provider_requirements:
      - category: ""
        capabilities:
          - contacts.write
"#,
        )
        .expect_err("empty provider category should be rejected");
        assert!(empty_category.contains("agent_endpoints[0].provider_requirements[0].category"));

        let duplicate_capability = parse_package(
            r#"
package:
  name: duplicate-provider-capability
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    provider_requirements:
      - category: crm
        capabilities:
          - contacts.write
          - contacts.write
"#,
        )
        .expect_err("duplicate provider capabilities should be rejected");
        assert!(
            duplicate_capability
                .contains("agent_endpoints[0].provider_requirements[0].capabilities[1]")
        );
        assert!(duplicate_capability.contains("duplicate provider capability `contacts.write`"));
    }

    #[test]
    fn warns_for_agent_endpoint_without_examples() {
        let parsed = parse_package(
            r#"
package:
  name: no-examples
  version: 0.2.0
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    outputs:
      - name: contact_id
        type: string
    agent_visibility:
      openapi: false
      arazzo: false
      mcp: false
      llms_txt: false
"#,
        )
        .expect("endpoint without examples should parse with warning");

        assert!(parsed.warnings.iter().any(|warning| {
            warning.path == "agent_endpoints[0].examples"
                && warning.message.contains("has no examples")
        }));
    }

    #[test]
    fn parses_hybrid_record_with_field_level_authority() {
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
      - name: tenant_id
        type: string
        authority: external
      - name: approval_state
        type: string
        authority: local
events:
  - name: TenantApprovalRequested
    record: Tenant
    kind: domain
projections:
  - name: TenantCurrentState
    record: Tenant
    source_event: TenantApprovalRequested
migrations:
  - name: tenant-projection-v2
    compatibility: additive
    projection_updates:
      - TenantCurrentState
"#,
        )
        .expect("hybrid record should parse");

        let fields = &parsed.package.records[0].fields;
        assert_eq!(fields[0].authority, Some(FieldAuthority::External));
        assert_eq!(fields[1].authority, Some(FieldAuthority::Local));
        assert_eq!(parsed.package.migrations[0].projection_updates.len(), 1);
    }

    #[test]
    fn rejects_hybrid_record_without_field_authority() {
        let error = parse_package(
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
      - name: tenant_id
        type: string
events: []
"#,
        )
        .expect_err("hybrid records must declare field-level authority");

        assert!(error.contains("field `tenant_id` must declare `authority: local|external`"));
    }

    #[test]
    fn parses_executable_relationships_migrations_and_agent_emits() {
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
        required: true
    outputs:
      - name: tenant_id
        type: string
    emits:
      event: TenantCreated
      stream: "tenant/{tenant_id}"
      payload:
        id: "$generated.tenant_id"
        full_name: "$input.full_name"
"#,
        )
        .expect("executable contract fields should parse");

        let tenant = &parsed.package.records[1];
        assert_eq!(
            tenant.fields[1]
                .references
                .as_ref()
                .expect("tenant landlord reference should parse")
                .record,
            "Landlord"
        );
        assert_eq!(
            parsed.package.migrations[0].idempotence_key.as_deref(),
            Some("tenant-v2-fields")
        );
        assert_eq!(
            parsed.package.migrations[0].backfills[0].field,
            "preferred_contact_method"
        );
        assert_eq!(
            parsed.package.agent_endpoints[0]
                .emits
                .as_ref()
                .expect("endpoint emit should parse")
                .event,
            "TenantCreated"
        );
    }

    #[test]
    fn rejects_invalid_executable_contract_references() {
        let unknown_record = parse_package(
            r#"
package:
  name: bad-ref
  version: 0.1.0
records:
  - name: Tenant
    fields:
      - name: id
        type: string
      - name: landlord_id
        type: string
        references:
          record: Landlord
          field: id
events: []
"#,
        )
        .expect_err("unknown relationship target should be rejected");
        assert!(
            unknown_record.contains("unknown referenced record `Landlord`"),
            "{unknown_record}"
        );

        let unknown_input = parse_package(
            r#"
package:
  name: bad-emit
  version: 0.1.0
records:
  - name: Tenant
    fields:
      - name: id
        type: string
events:
  - name: TenantCreated
    record: Tenant
agent_endpoints:
  - id: create_tenant
    title: Create tenant
    intent: Create a tenant.
    inputs:
      - name: full_name
        type: string
    emits:
      event: TenantCreated
      stream: "tenant/{tenant_id}"
      payload:
        full_name: "$input.display_name"
"#,
        )
        .expect_err("unknown input template should be rejected");
        assert!(unknown_input.contains("payload.full_name"));
        assert!(
            unknown_input.contains("unknown input reference `$input.display_name`"),
            "{unknown_input}"
        );
    }

    #[test]
    fn parses_valid_ontology_model() {
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
    - id: Party
      kind: abstract
    - id: Customer
      kind: entity
      extends: Party
      backed_by:
        record: Customer
      sensitivity:
        classification: confidential
        pii: true
    - id: Contract
      kind: entity
      backed_by:
        record: Contract
  relationships:
    - id: has_contract
      label: has contract
      from: Customer
      to: Contract
      cardinality:
        from: one
        to: many
      backed_by:
        record: CustomerContract
        from_field: customer_id
        to_field: contract_id
  constraints:
    - id: customer_policy
      applies_to:
        concept: Customer
      requires_policy: customer_data_access
"#,
        )
        .expect("valid ontology should parse");

        let ontology = parsed.package.ontology.expect("ontology should parse");
        assert_eq!(ontology.concepts.len(), 3);
        assert_eq!(ontology.concepts[1].extends, ["Party"]);
        assert!(ontology.concepts[1].sensitivity.as_ref().unwrap().pii);
        assert_eq!(ontology.relationships[0].id, "has_contract");
    }

    #[test]
    fn parses_semantic_aliases_and_entity_linking() {
        let parsed = parse_package(
            r#"
package:
  name: ontology-demo
  version: 0.1.0
records:
  - name: Customer
    source: native
    fields:
      - name: id
        type: string
      - name: email
        type: string
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
      backed_by:
        record: Customer
  relationships: []
semantic_aliases:
  concepts:
    Customer:
      - client
      - account holder
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
"#,
        )
        .expect("semantic aliases and linking should parse");

        let aliases = parsed
            .package
            .semantic_aliases
            .expect("semantic aliases should parse");
        assert_eq!(aliases.concepts["Customer"], ["client", "account holder"]);
        let linking = parsed
            .package
            .entity_linking
            .expect("entity linking should parse");
        assert_eq!(linking.strategies[0].id, "email_match");
        assert_eq!(linking.strategies[0].confidence.0, 950_000);
    }

    #[test]
    fn semantic_aliases_warn_on_duplicate_and_reject_collisions() {
        let duplicate = parse_package(
            r#"
package:
  name: duplicate-alias
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships: []
semantic_aliases:
  concepts:
    Customer:
      - client
      - " Client "
"#,
        )
        .expect("duplicate alias on the same target should parse with warning");
        assert!(
            duplicate
                .warnings
                .iter()
                .any(|warning| warning.path.contains("semantic_aliases.concepts.Customer"))
        );

        let collision = parse_package(
            r#"
package:
  name: alias-collision
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
    - id: Contract
      kind: entity
  relationships: []
semantic_aliases:
  concepts:
    Customer:
      - client
    Contract:
      - " client "
"#,
        )
        .expect_err("same normalized alias on different targets should fail");
        assert!(collision.contains("collides"));
    }

    #[test]
    fn rejects_invalid_alias_and_linking_references() {
        let unknown_alias_concept = parse_package(
            r#"
package:
  name: unknown-alias
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships: []
semantic_aliases:
  concepts:
    Missing:
      - client
"#,
        )
        .expect_err("unknown alias concept should fail");
        assert!(unknown_alias_concept.contains("unknown ontology concept"));

        let unknown_alias_relationship = parse_package(
            r#"
package:
  name: unknown-relationship-alias
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships: []
semantic_aliases:
  relationships:
    owns:
      - belongs to
"#,
        )
        .expect_err("unknown alias relationship should fail");
        assert!(unknown_alias_relationship.contains("unknown ontology relationship"));

        let unknown_field = parse_package(
            r#"
package:
  name: unknown-link-field
  version: 0.1.0
records:
  - name: Customer
    source: native
    fields:
      - name: id
        type: string
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
      backed_by:
        record: Customer
  relationships: []
entity_linking:
  strategies:
    - id: email_match
      applies_to: Customer
      match:
        source_field: email
        target_field: email
      confidence: 0.95
"#,
        )
        .expect_err("unknown target field should fail");
        assert!(unknown_field.contains("entity_linking.strategies[0].match.target_field"));
    }

    #[test]
    fn rejects_invalid_entity_linking_strategy_shape() {
        let out_of_range = parse_package(
            r#"
package:
  name: bad-confidence
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships: []
entity_linking:
  strategies:
    - id: email_match
      applies_to: Customer
      source_type: document
      match:
        source_field: email
        target_field: email
      confidence: 1.5
"#,
        )
        .expect_err("out of range confidence should fail");
        assert!(out_of_range.contains("confidence must be between"));

        let duplicate_id = parse_package(
            r#"
package:
  name: duplicate-strategy
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships: []
entity_linking:
  strategies:
    - id: email_match
      applies_to: Customer
      source_type: document
      match:
        source_field: email
        target_field: email
      confidence: 0.9
    - id: email_match
      applies_to: Customer
      source_type: document
      match:
        source_field: external_email
        target_field: email
      confidence: 0.8
"#,
        )
        .expect_err("duplicate strategy id should fail");
        assert!(duplicate_id.contains("duplicate entity-linking strategy id"));
    }

    #[test]
    fn parses_valid_retrieval_bindings() {
        let parsed = parse_package(
            r#"
package:
  name: retrieval-demo
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
    - id: Contract
      kind: entity
  relationships:
    - id: governed_by
      from: Customer
      to: Contract
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: primary_evidence
      category: evidence
      required_capabilities:
        - entity.link
        - evidence.query
  scopes:
    - id: customer_evidence
      applies_to:
        concept: Customer
      provider: primary_evidence
      filters:
        entity_scope:
          include_self: true
          include_related:
            - relationship: governed_by
              direction: outgoing
              max_depth: 1
"#,
        )
        .expect("retrieval bindings should parse");
        let bindings = parsed
            .package
            .retrieval_bindings
            .expect("retrieval bindings should be present");
        assert_eq!(bindings.providers[0].id, "primary_evidence");
        assert_eq!(bindings.scopes[0].id, "customer_evidence");
    }

    #[test]
    fn rejects_invalid_retrieval_bindings() {
        let unknown_concept = parse_package(
            r#"
package:
  name: bad-retrieval-concept
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships: []
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: evidence
      category: evidence
  scopes:
    - id: missing
      applies_to:
        concept: Missing
      provider: evidence
"#,
        )
        .expect_err("unknown concept should fail");
        assert!(unknown_concept.contains("applies_to.concept"));

        let invalid_depth = parse_package(
            r#"
package:
  name: bad-retrieval-depth
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
    - id: Contract
      kind: entity
  relationships:
    - id: governed_by
      from: Customer
      to: Contract
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: evidence
      category: evidence
  scopes:
    - id: too_deep
      applies_to:
        concept: Customer
      provider: evidence
      filters:
        entity_scope:
          include_related:
            - relationship: governed_by
              direction: both
              max_depth: 6
"#,
        )
        .expect_err("invalid max depth should fail");
        assert!(invalid_depth.contains("max_depth"));

        let unknown_provider = parse_package(
            r#"
package:
  name: bad-retrieval-provider
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships: []
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers: []
  scopes:
    - id: customer_evidence
      applies_to:
        concept: Customer
      provider: missing
"#,
        )
        .expect_err("unknown provider should fail");
        assert!(unknown_provider.contains("unknown retrieval provider"));
    }

    #[test]
    fn rejects_invalid_ontology_references() {
        let duplicate = parse_package(
            r#"
package:
  name: duplicate-concepts
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
    - id: Customer
      kind: abstract
"#,
        )
        .expect_err("duplicate concept should fail");
        assert!(duplicate.contains("ontology.concepts[1].id"));

        let unknown_concept = parse_package(
            r#"
package:
  name: unknown-relationship-concept
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
  relationships:
    - id: owns
      from: Customer
      to: Asset
"#,
        )
        .expect_err("unknown relationship target should fail");
        assert!(unknown_concept.contains("ontology.relationships[0].to"));

        let cycle = parse_package(
            r#"
package:
  name: cyclic-ontology
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
      extends: Party
    - id: Party
      kind: abstract
      extends: Customer
"#,
        )
        .expect_err("inheritance cycle should fail");
        assert!(cycle.contains("inheritance cycle"));
    }

    #[test]
    fn rejects_ontology_backing_errors() {
        let missing_record = parse_package(
            r#"
package:
  name: missing-backing-record
  version: 0.1.0
records: []
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
      backed_by:
        record: Customer
"#,
        )
        .expect_err("missing backing record should fail");
        assert!(missing_record.contains("ontology.concepts[0].backed_by.record"));

        let missing_field = parse_package(
            r#"
package:
  name: missing-backing-field
  version: 0.1.0
records:
  - name: Ownership
    fields:
      - name: id
        type: string
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Party
      kind: entity
    - id: Asset
      kind: entity
  relationships:
    - id: owns
      from: Party
      to: Asset
      backed_by:
        record: Ownership
        from_field: party_id
"#,
        )
        .expect_err("missing backing field should fail");
        assert!(missing_field.contains("ontology.relationships[0].backed_by.from_field"));
    }

    #[test]
    fn rejects_unknown_ontology_fields() {
        let error = parse_package(
            r#"
package:
  name: unknown-ontology-field
  version: 0.1.0
ontology:
  schema: greentic.sorla.ontology.v1
  unsupported: true
"#,
        )
        .expect_err("unknown ontology fields should be denied by serde");

        assert!(error.contains("unknown field"));
    }
}
