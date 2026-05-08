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
}
