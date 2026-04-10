use greentic_sorla_lang::ast::{
    CompatibilityMode, EventKind, FieldAuthority, Package, ProjectionMode, RecordSource,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<FieldAuthorityIr>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
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
                notes: migration.notes.clone(),
            }
        })
        .collect();
    compatibility.sort_by(|left, right| left.name.cmp(&right.name));

    let mut provider_categories: Vec<ProviderRequirementIr> = package
        .provider_requirements
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
    }
}

fn sorted_fields(record: &greentic_sorla_lang::ast::Record) -> Vec<FieldIr> {
    let mut fields: Vec<FieldIr> = record
        .fields
        .iter()
        .map(|field| FieldIr {
            name: field.name.clone(),
            type_name: field.type_name.clone(),
            authority: field.authority.as_ref().map(|authority| match authority {
                FieldAuthority::Local => FieldAuthorityIr::Local,
                FieldAuthority::External => FieldAuthorityIr::External,
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
        assert_eq!(canonical_hash_hex(&first), canonical_hash_hex(&second));
    }
}
