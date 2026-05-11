use greentic_sorla_ir::{CanonicalIr, CompatibilityModeIr, ProviderRequirementIr};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const SORX_COMPATIBILITY_SCHEMA: &str = "greentic.sorx.compatibility.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorxCompatibilityManifest {
    pub schema: String,
    pub package: SorxCompatibilityPackageRef,
    pub api_compatibility: ApiCompatibilityMode,
    pub state_compatibility: StateCompatibilityMode,
    pub provider_compatibility: Vec<ProviderCompatibilityRequirement>,
    pub migration_compatibility: Vec<MigrationCompatibilityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorxCompatibilityPackageRef {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiCompatibilityMode {
    Additive,
    BackwardCompatible,
    Breaking,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StateCompatibilityMode {
    IsolatedRequired,
    SharedAllowed,
    SharedRequiresMigration,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderCompatibilityRequirement {
    pub category: String,
    pub required_capabilities: Vec<String>,
    pub contract_version_range: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationCompatibilityRule {
    pub name: String,
    pub mode: ApiCompatibilityMode,
    pub projection_updates: Vec<String>,
    pub backfill_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotence_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SorxCompatibilityError {
    message: String,
}

impl SorxCompatibilityError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SorxCompatibilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SorxCompatibilityError {}

impl SorxCompatibilityManifest {
    pub fn validate_static(&self) -> Result<(), SorxCompatibilityError> {
        if self.schema != SORX_COMPATIBILITY_SCHEMA {
            return Err(SorxCompatibilityError::new(format!(
                "compatibility manifest schema must be `{SORX_COMPATIBILITY_SCHEMA}`, got `{}`",
                self.schema
            )));
        }
        if self.package.name.trim().is_empty() {
            return Err(SorxCompatibilityError::new(
                "compatibility manifest package.name must not be empty",
            ));
        }
        if self.package.version.trim().is_empty() {
            return Err(SorxCompatibilityError::new(
                "compatibility manifest package.version must not be empty",
            ));
        }
        if self.state_compatibility == StateCompatibilityMode::SharedAllowed
            && self.migration_compatibility.is_empty()
        {
            return Err(SorxCompatibilityError::new(
                "shared state compatibility requires explicit migration compatibility metadata",
            ));
        }

        let mut categories = BTreeSet::new();
        for requirement in &self.provider_compatibility {
            if requirement.category.trim().is_empty() {
                return Err(SorxCompatibilityError::new(
                    "provider compatibility category must not be empty",
                ));
            }
            if !categories.insert(requirement.category.as_str()) {
                return Err(SorxCompatibilityError::new(format!(
                    "duplicate provider compatibility category `{}`",
                    requirement.category
                )));
            }
            if requirement.required && requirement.required_capabilities.is_empty() {
                return Err(SorxCompatibilityError::new(format!(
                    "required provider compatibility `{}` must list capabilities",
                    requirement.category
                )));
            }
            if requirement.contract_version_range.trim().is_empty() {
                return Err(SorxCompatibilityError::new(format!(
                    "provider compatibility `{}` must include a non-empty contract_version_range",
                    requirement.category
                )));
            }
        }

        let mut migrations = BTreeSet::new();
        for migration in &self.migration_compatibility {
            if migration.name.trim().is_empty() {
                return Err(SorxCompatibilityError::new(
                    "migration compatibility name must not be empty",
                ));
            }
            if !migrations.insert(migration.name.as_str()) {
                return Err(SorxCompatibilityError::new(format!(
                    "duplicate migration compatibility name `{}`",
                    migration.name
                )));
            }
        }
        Ok(())
    }
}

pub fn generate_sorx_compatibility_manifest(
    ir: &CanonicalIr,
    ir_hash: Option<&str>,
) -> SorxCompatibilityManifest {
    let migrations = ir
        .compatibility
        .iter()
        .map(|migration| MigrationCompatibilityRule {
            name: migration.name.clone(),
            mode: api_mode(&migration.compatibility),
            projection_updates: migration.projection_updates.clone(),
            backfill_count: migration.backfills.len(),
            idempotence_key: migration.idempotence_key.clone(),
        })
        .collect::<Vec<_>>();

    let api_compatibility = migrations
        .iter()
        .map(|migration| migration.mode.clone())
        .max_by_key(api_mode_rank)
        .unwrap_or(ApiCompatibilityMode::Unknown);
    let state_compatibility = if migrations.is_empty() {
        StateCompatibilityMode::IsolatedRequired
    } else {
        StateCompatibilityMode::SharedRequiresMigration
    };

    SorxCompatibilityManifest {
        schema: SORX_COMPATIBILITY_SCHEMA.to_string(),
        package: SorxCompatibilityPackageRef {
            name: ir.package.name.clone(),
            version: ir.package.version.clone(),
            ir_hash: ir_hash.map(str::to_string),
        },
        api_compatibility,
        state_compatibility,
        provider_compatibility: provider_compatibility(ir),
        migration_compatibility: migrations,
    }
}

fn provider_compatibility(ir: &CanonicalIr) -> Vec<ProviderCompatibilityRequirement> {
    let mut categories: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for requirement in &ir.provider_contract.categories {
        insert_provider_requirement(&mut categories, requirement);
    }
    for endpoint in &ir.agent_endpoints {
        for requirement in &endpoint.provider_requirements {
            insert_provider_requirement(&mut categories, requirement);
        }
    }
    categories
        .into_iter()
        .map(
            |(category, capabilities)| ProviderCompatibilityRequirement {
                category,
                required_capabilities: capabilities.into_iter().collect(),
                contract_version_range: ">=0.1.0 <1.0.0".to_string(),
                required: true,
            },
        )
        .collect()
}

fn insert_provider_requirement(
    categories: &mut BTreeMap<String, BTreeSet<String>>,
    requirement: &ProviderRequirementIr,
) {
    categories
        .entry(requirement.category.clone())
        .or_default()
        .extend(requirement.capabilities.iter().cloned());
}

fn api_mode(mode: &CompatibilityModeIr) -> ApiCompatibilityMode {
    match mode {
        CompatibilityModeIr::Additive => ApiCompatibilityMode::Additive,
        CompatibilityModeIr::BackwardCompatible => ApiCompatibilityMode::BackwardCompatible,
        CompatibilityModeIr::Breaking => ApiCompatibilityMode::Breaking,
    }
}

fn api_mode_rank(mode: &ApiCompatibilityMode) -> u8 {
    match mode {
        ApiCompatibilityMode::Additive => 1,
        ApiCompatibilityMode::BackwardCompatible => 2,
        ApiCompatibilityMode::Breaking => 3,
        ApiCompatibilityMode::Unknown => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_handoff_artifacts_from_yaml;

    #[test]
    fn default_conservative_manifest_requires_isolated_state_without_migrations() {
        let artifacts = build_handoff_artifacts_from_yaml(
            r#"
package:
  name: minimal
  version: 0.1.0
records:
  - name: Account
    fields:
      - name: id
        type: string
agent_endpoints: []
"#,
        )
        .expect("fixture should build");

        let manifest = generate_sorx_compatibility_manifest(&artifacts.ir, None);
        assert_eq!(
            manifest.state_compatibility,
            StateCompatibilityMode::IsolatedRequired
        );
        manifest.validate_static().expect("manifest validates");
    }

    #[test]
    fn provider_requirements_are_copied_and_aggregated() {
        let artifacts = build_handoff_artifacts_from_yaml(
            r#"
package:
  name: provider-pack
  version: 0.1.0
records:
  - name: Contact
    fields:
      - name: id
        type: string
provider_requirements:
  - category: crm
    capabilities:
      - contacts.write
agent_endpoints:
  - id: sync_contact
    title: Sync contact
    intent: Sync.
    provider_requirements:
      - category: crm
        capabilities:
          - contacts.read
    agent_visibility:
      openapi: false
      arazzo: false
      mcp: false
      llms_txt: false
"#,
        )
        .expect("fixture should build");

        let manifest = generate_sorx_compatibility_manifest(&artifacts.ir, None);
        assert_eq!(manifest.provider_compatibility.len(), 1);
        assert_eq!(manifest.provider_compatibility[0].category, "crm");
        assert_eq!(
            manifest.provider_compatibility[0].required_capabilities,
            ["contacts.read", "contacts.write"]
        );
    }

    #[test]
    fn migration_metadata_generates_rules() {
        let artifacts = build_handoff_artifacts_from_yaml(
            r#"
package:
  name: migration-pack
  version: 0.2.0
records:
  - name: Account
    fields:
      - name: id
        type: string
events:
  - name: AccountChanged
    record: Account
    emits:
      - name: id
        type: string
projections:
  - name: AccountProjection
    record: Account
    source_event: AccountChanged
migrations:
  - name: add-account-projection
    compatibility: additive
    projection_updates:
      - AccountProjection
    idempotence_key: migration-pack-0.2.0
agent_endpoints: []
"#,
        )
        .expect("fixture should build");

        let manifest = generate_sorx_compatibility_manifest(&artifacts.ir, None);
        assert_eq!(
            manifest.state_compatibility,
            StateCompatibilityMode::SharedRequiresMigration
        );
        assert_eq!(
            manifest.migration_compatibility[0].name,
            "add-account-projection"
        );
        manifest.validate_static().expect("manifest validates");
    }

    #[test]
    fn rejects_shared_state_without_migration_metadata() {
        let mut manifest = SorxCompatibilityManifest {
            schema: SORX_COMPATIBILITY_SCHEMA.to_string(),
            package: SorxCompatibilityPackageRef {
                name: "bad".to_string(),
                version: "0.1.0".to_string(),
                ir_hash: None,
            },
            api_compatibility: ApiCompatibilityMode::Unknown,
            state_compatibility: StateCompatibilityMode::SharedAllowed,
            provider_compatibility: Vec::new(),
            migration_compatibility: Vec::new(),
        };

        let err = manifest
            .validate_static()
            .expect_err("shared state should require explicit metadata");
        assert!(err.message().contains("shared state"));
        manifest.state_compatibility = StateCompatibilityMode::IsolatedRequired;
        manifest
            .validate_static()
            .expect("fixed manifest validates");
    }
}
