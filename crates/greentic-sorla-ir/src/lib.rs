use greentic_sorla_lang::ast::{
    AgentEndpointApprovalMode, AgentEndpointRisk, CompatibilityMode, EventKind, FieldAuthority,
    Package, ProjectionMode, ProviderRequirement, RecordSource,
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
    pub records: Vec<RecordIr>,
    pub events: Vec<EventIr>,
    pub actions: Vec<NamedItemIr>,
    pub policies: Vec<NamedItemIr>,
    pub approvals: Vec<NamedItemIr>,
    pub views: Vec<NamedItemIr>,
    pub flows: Vec<NamedItemIr>,
    pub projections: Vec<ProjectionIr>,
    pub external_sources: Vec<ExternalSourceIr>,
    pub compatibility: Vec<CompatibilityIr>,
    pub provider_contract: ProviderContractIr,
    pub agent_endpoints: Vec<AgentEndpointIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageIr {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordIr {
    pub name: String,
    pub source: RecordSourceIr,
    pub fields: Vec<FieldIr>,
    pub external_source: Option<String>,
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
    pub type_name: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<FieldAuthorityIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<FieldReferenceIr>,
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
    pub record: String,
    pub source_event: String,
    pub mode: ProjectionModeIr,
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
    pub projection_updates: Vec<String>,
    pub backfills: Vec<MigrationBackfillIr>,
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
    pub agent_visibility: AgentEndpointVisibilityIr,
    pub examples: Vec<AgentEndpointExampleIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emits: Option<AgentEndpointEmitIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointInputIr {
    pub name: String,
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
    let mut records: Vec<RecordIr> = package
        .records
        .iter()
        .map(|record| RecordIr {
            name: record.name.clone(),
            source: match record.source.clone().expect("source normalized by parser") {
                RecordSource::Native => RecordSourceIr::Native,
                RecordSource::External => RecordSourceIr::External,
                RecordSource::Hybrid => RecordSourceIr::Hybrid,
            },
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
            record: projection.record.clone(),
            source_event: projection.source_event.clone(),
            mode: match projection.mode {
                ProjectionMode::CurrentState => ProjectionModeIr::CurrentState,
                ProjectionMode::AuditTrail => ProjectionModeIr::AuditTrail,
            },
        })
        .collect();
    projections.sort_by(|left, right| left.name.cmp(&right.name));

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
            CompatibilityIr {
                name: migration.name.clone(),
                compatibility: match migration.compatibility {
                    CompatibilityMode::Additive => CompatibilityModeIr::Additive,
                    CompatibilityMode::BackwardCompatible => {
                        CompatibilityModeIr::BackwardCompatible
                    }
                    CompatibilityMode::Breaking => CompatibilityModeIr::Breaking,
                },
                projection_updates,
                backfills,
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
        views: sorted_named_items(
            &package
                .views
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
        ),
        flows: sorted_named_items(
            &package
                .flows
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
        ),
        projections,
        external_sources,
        compatibility,
        provider_contract: ProviderContractIr {
            categories: provider_categories,
        },
        agent_endpoints,
    }
}

fn sorted_fields(record: &greentic_sorla_lang::ast::Record) -> Vec<FieldIr> {
    let mut fields: Vec<FieldIr> = record
        .fields
        .iter()
        .map(|field| FieldIr {
            name: field.name.clone(),
            type_name: field.type_name.clone(),
            required: field.required,
            sensitive: field.sensitive,
            enum_values: field.enum_values.clone(),
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
}
