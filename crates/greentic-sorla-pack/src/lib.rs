use greentic_sorla_ir::{
    CanonicalIr, IrVersion, ProviderRequirementIr, agent_tools_json, canonical_cbor,
    canonical_hash_hex, inspect_ir, lower_package,
};
use greentic_sorla_lang::parser::parse_package;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PackageManifest {
    pub package_kind: &'static str,
    pub ir_version: IrVersionView,
    pub provider_repo: &'static str,
    pub required_provider_categories: Vec<ProviderRequirementView>,
    pub artifact_references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct IrVersionView {
    pub major: u16,
    pub minor: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProviderRequirementView {
    pub category: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSet {
    pub ir: CanonicalIr,
    pub package_manifest: PackageManifest,
    pub cbor_artifacts: BTreeMap<String, Vec<u8>>,
    pub inspect_json: String,
    pub agent_tools_json: String,
    pub canonical_hash: String,
}

pub fn scaffold_manifest() -> PackageManifest {
    let version = IrVersion::scaffold();
    PackageManifest {
        package_kind: "sorla-package",
        ir_version: IrVersionView {
            major: version.major,
            minor: version.minor,
        },
        provider_repo: "greentic-sorla-providers",
        required_provider_categories: Vec::new(),
        artifact_references: vec![
            "model.cbor".to_string(),
            "actions.cbor".to_string(),
            "events.cbor".to_string(),
            "projections.cbor".to_string(),
            "policies.cbor".to_string(),
            "approvals.cbor".to_string(),
            "views.cbor".to_string(),
            "external-sources.cbor".to_string(),
            "compatibility.cbor".to_string(),
            "provider-contract.cbor".to_string(),
            "package-manifest.cbor".to_string(),
            "agent-tools.json".to_string(),
        ],
    }
}

pub fn build_artifacts_from_yaml(input: &str) -> Result<ArtifactSet, String> {
    let parsed = parse_package(input)?;
    let ir = lower_package(&parsed.package);
    let mut package_manifest = scaffold_manifest();
    package_manifest.required_provider_categories = ir
        .provider_contract
        .categories
        .iter()
        .map(provider_view)
        .collect();

    let mut cbor_artifacts = BTreeMap::new();
    cbor_artifacts.insert("actions.cbor".to_string(), canonical_cbor(&ir.actions));
    cbor_artifacts.insert("approvals.cbor".to_string(), canonical_cbor(&ir.approvals));
    cbor_artifacts.insert(
        "compatibility.cbor".to_string(),
        canonical_cbor(&ir.compatibility),
    );
    cbor_artifacts.insert("events.cbor".to_string(), canonical_cbor(&ir.events));
    cbor_artifacts.insert(
        "external-sources.cbor".to_string(),
        canonical_cbor(&ir.external_sources),
    );
    cbor_artifacts.insert("model.cbor".to_string(), canonical_cbor(&ir));
    cbor_artifacts.insert("policies.cbor".to_string(), canonical_cbor(&ir.policies));
    cbor_artifacts.insert(
        "projections.cbor".to_string(),
        canonical_cbor(&ir.projections),
    );
    cbor_artifacts.insert(
        "provider-contract.cbor".to_string(),
        canonical_cbor(&ir.provider_contract),
    );
    cbor_artifacts.insert(
        "package-manifest.cbor".to_string(),
        canonical_cbor(&package_manifest),
    );
    cbor_artifacts.insert("views.cbor".to_string(), canonical_cbor(&ir.views));

    let inspect_json = inspect_ir(&ir);
    let agent_tools = agent_tools_json(&ir);
    let canonical_hash = canonical_hash_hex(&ir);

    Ok(ArtifactSet {
        ir,
        package_manifest,
        cbor_artifacts,
        inspect_json,
        agent_tools_json: agent_tools,
        canonical_hash,
    })
}

fn provider_view(requirement: &ProviderRequirementIr) -> ProviderRequirementView {
    ProviderRequirementView {
        category: requirement.category.clone(),
        capabilities: requirement.capabilities.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scaffold_manifest_stays_provider_agnostic() {
        let manifest = scaffold_manifest();
        assert_eq!(manifest.package_kind, "sorla-package");
        assert_eq!(manifest.provider_repo, "greentic-sorla-providers");
        assert!(
            manifest
                .artifact_references
                .contains(&"provider-contract.cbor".to_string())
        );
    }

    #[test]
    fn builds_deterministic_artifacts_for_golden_fixture() {
        let fixture = fs::read_to_string("tests/golden/tenant_v0_2.sorla.yaml")
            .expect("fixture should be readable");
        let expected_inspect = fs::read_to_string("tests/golden/tenant_v0_2.inspect.json")
            .expect("golden should be readable");

        let first = build_artifacts_from_yaml(&fixture).expect("fixture should build");
        let second = build_artifacts_from_yaml(&fixture).expect("fixture should build");

        assert_eq!(first.inspect_json.trim_end(), expected_inspect.trim_end());
        assert_eq!(first.inspect_json, second.inspect_json);
        assert_eq!(first.canonical_hash, second.canonical_hash);
        assert!(first.cbor_artifacts.contains_key("model.cbor"));
        assert!(first.cbor_artifacts.contains_key("events.cbor"));
        assert!(first.cbor_artifacts.contains_key("projections.cbor"));
        assert!(first.cbor_artifacts.contains_key("provider-contract.cbor"));
        assert!(first.agent_tools_json.contains("storage"));
    }
}
