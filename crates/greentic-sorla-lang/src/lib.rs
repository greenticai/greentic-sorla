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
    use crate::ast::{FieldAuthority, ProjectionMode, RecordSource};
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
}
