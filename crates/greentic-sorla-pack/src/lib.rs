pub mod sorx_compatibility;
pub mod sorx_exposure;
pub mod sorx_validation;
pub mod validation_generator;

pub use sorx_compatibility::{
    ApiCompatibilityMode, SORX_COMPATIBILITY_SCHEMA, SorxCompatibilityError,
    SorxCompatibilityManifest, SorxCompatibilityPackageRef, StateCompatibilityMode,
    generate_sorx_compatibility_manifest,
};
pub use sorx_exposure::{
    SORX_EXPOSURE_POLICY_SCHEMA, SorxEndpointExposurePolicy, SorxExposurePolicy,
    SorxExposurePolicyError, generate_sorx_exposure_policy,
};
pub use sorx_validation::{
    EndpointVisibility, SORX_VALIDATION_SCHEMA, SorxValidationError, SorxValidationManifest,
    SorxValidationPackageRef, SorxValidationSuite, SorxValidationTest, sorx_validation_schema_json,
};
pub use validation_generator::{
    SorxValidationGenerationInput, generate_sorx_validation_manifest,
    generate_sorx_validation_manifest_from_ir,
};

use greentic_sorla_ir::{
    AgentEndpointApprovalModeIr, AgentEndpointInputIr, AgentEndpointIr, AgentEndpointOutputIr,
    AgentEndpointRiskIr, CanonicalIr, IrVersion, ProviderRequirementIr, agent_tools_json,
    canonical_cbor, canonical_hash_hex, inspect_ir, lower_package,
};
use greentic_sorla_lang::parser::parse_package;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const AGENT_GATEWAY_HANDOFF_SCHEMA: &str = "greentic.agent-gateway.handoff.v1";
pub const OPENAPI_AGENT_OVERLAY_SCHEMA: &str = "greentic.openapi.agent-overlay.v1";
pub const MCP_TOOLS_HANDOFF_SCHEMA: &str = "greentic.mcp.tools.handoff.v1";
pub const AGENT_GATEWAY_HANDOFF_FILENAME: &str = "agent-gateway.json";
pub const AGENT_ENDPOINTS_IR_CBOR_FILENAME: &str = "agent-endpoints.ir.cbor";
pub const AGENT_OPENAPI_OVERLAY_FILENAME: &str = "agent-endpoints.openapi.overlay.yaml";
pub const AGENT_ARAZZO_FILENAME: &str = "agent-workflows.arazzo.yaml";
pub const MCP_TOOLS_FILENAME: &str = "mcp-tools.json";
pub const LLMS_TXT_FRAGMENT_FILENAME: &str = "llms.txt.fragment";
pub const EXECUTABLE_CONTRACT_FILENAME: &str = "executable-contract.json";
pub const SORX_RUNTIME_EXTENSION_ID: &str = "greentic.sorx.runtime.v1";
pub const START_SCHEMA_FILENAME: &str = "start.schema.json";
pub const START_QUESTIONS_FILENAME: &str = "start.questions.cbor";
pub const RUNTIME_TEMPLATE_FILENAME: &str = "runtime.template.yaml";
pub const PROVIDER_BINDINGS_TEMPLATE_FILENAME: &str = "provider-bindings.template.yaml";
pub const SORX_COMPATIBILITY_PATH: &str = "assets/sorx/compatibility.json";
const SORX_COMPATIBILITY_ASSET: &str = "compatibility.json";
pub const SORX_EXPOSURE_POLICY_PATH: &str = "assets/sorx/exposure-policy.json";
const SORX_EXPOSURE_POLICY_ASSET: &str = "exposure-policy.json";
pub const SORX_VALIDATION_MANIFEST_PATH: &str = "assets/sorx/tests/test-manifest.json";
const SORX_VALIDATION_MANIFEST_ASSET: &str = "tests/test-manifest.json";

const STABLE_PACK_TIMESTAMP: &str = "1970-01-01T00:00:00Z";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HandoffManifest {
    pub package_kind: &'static str,
    pub ir_version: IrVersionView,
    pub provider_repo: &'static str,
    pub required_provider_categories: Vec<ProviderRequirementView>,
    pub artifact_references: Vec<String>,
}

pub type PackageManifest = HandoffManifest;

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
    pub agent_exports: AgentExportSet,
    pub executable_contract_json: String,
    pub canonical_hash: String,
}

impl ArtifactSet {
    pub fn handoff_manifest(&self) -> &HandoffManifest {
        &self.package_manifest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayHandoffManifest {
    pub schema: String,
    pub package: AgentGatewayPackageRef,
    pub endpoints: Vec<AgentGatewayEndpointRef>,
    pub provider_contract: AgentGatewayProviderContract,
    pub exports: AgentGatewayExports,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayPackageRef {
    pub name: String,
    pub version: String,
    pub ir_version: String,
    pub ir_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayEndpointRef {
    pub id: String,
    pub title: String,
    pub intent: String,
    pub risk: String,
    pub approval: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub side_effects: Vec<String>,
    pub exports: AgentGatewayEndpointExports,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayEndpointExports {
    pub openapi: bool,
    pub arazzo: bool,
    pub mcp: bool,
    pub llms_txt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayProviderContract {
    pub categories: Vec<AgentGatewayProviderRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayProviderRequirement {
    pub category: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayExports {
    pub agent_gateway_json: bool,
    pub openapi_overlay: bool,
    pub arazzo: bool,
    pub mcp_tools: bool,
    pub llms_txt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentExportSet {
    pub agent_gateway_json: String,
    pub openapi_overlay_yaml: Option<String>,
    pub arazzo_yaml: Option<String>,
    pub mcp_tools_json: Option<String>,
    pub llms_txt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SorlaGtpackOptions {
    pub input_path: PathBuf,
    pub name: String,
    pub version: String,
    pub out_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackBuildSummary {
    pub out_path: String,
    pub name: String,
    pub version: String,
    pub sorla_package_name: String,
    pub sorla_package_version: String,
    pub ir_hash: String,
    pub manifest_hash_sha256: String,
    pub assets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackInspection {
    pub path: String,
    pub name: String,
    pub version: String,
    pub extension: String,
    pub sorla_package_name: String,
    pub sorla_package_version: String,
    pub ir_hash: String,
    pub assets: Vec<String>,
    pub optional_artifacts: BTreeMap<String, bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<SorlaGtpackValidationInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure_policy: Option<SorlaGtpackExposurePolicyInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<SorlaGtpackCompatibilityInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackValidationInspection {
    pub schema: String,
    pub suite_count: usize,
    pub test_count: usize,
    pub promotion_requires: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackExposurePolicyInspection {
    pub default_visibility: EndpointVisibility,
    pub public_candidate_endpoints: usize,
    pub approval_required_endpoints: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackCompatibilityInspection {
    pub api_mode: ApiCompatibilityMode,
    pub state_mode: StateCompatibilityMode,
    pub provider_requirement_count: usize,
    pub migration_rule_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackDoctorReport {
    pub path: String,
    pub status: String,
    pub checked_assets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SorlaPackManifest {
    schema: String,
    pack: SorlaPackIdentity,
    created_at_utc: String,
    extension: serde_json::Value,
    assets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SorlaPackIdentity {
    name: String,
    version: String,
    kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SorlaPackLock {
    schema: String,
    entries: BTreeMap<String, SorlaPackLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SorlaPackLockEntry {
    size: u64,
    sha256: String,
}

pub fn scaffold_handoff_manifest() -> HandoffManifest {
    let version = IrVersion::scaffold();
    HandoffManifest {
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
            "launcher-handoff.cbor".to_string(),
            "agent-tools.json".to_string(),
            EXECUTABLE_CONTRACT_FILENAME.to_string(),
        ],
    }
}

pub fn scaffold_manifest() -> PackageManifest {
    scaffold_handoff_manifest()
}

pub fn build_handoff_artifacts_from_yaml(input: &str) -> Result<ArtifactSet, String> {
    let parsed = parse_package(input)?;
    let ir = lower_package(&parsed.package);
    let mut package_manifest = scaffold_handoff_manifest();
    package_manifest.required_provider_categories = ir
        .provider_contract
        .categories
        .iter()
        .map(provider_view)
        .collect();
    let agent_exports = export_agent_artifacts(&ir);
    package_manifest
        .artifact_references
        .push(AGENT_GATEWAY_HANDOFF_FILENAME.to_string());
    if !ir.agent_endpoints.is_empty() {
        package_manifest
            .artifact_references
            .push(AGENT_ENDPOINTS_IR_CBOR_FILENAME.to_string());
    }
    if agent_exports.openapi_overlay_yaml.is_some() {
        package_manifest
            .artifact_references
            .push(AGENT_OPENAPI_OVERLAY_FILENAME.to_string());
    }
    if agent_exports.arazzo_yaml.is_some() {
        package_manifest
            .artifact_references
            .push(AGENT_ARAZZO_FILENAME.to_string());
    }
    if agent_exports.mcp_tools_json.is_some() {
        package_manifest
            .artifact_references
            .push(MCP_TOOLS_FILENAME.to_string());
    }
    if agent_exports.llms_txt.is_some() {
        package_manifest
            .artifact_references
            .push(LLMS_TXT_FRAGMENT_FILENAME.to_string());
    }

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
    if !ir.agent_endpoints.is_empty() {
        cbor_artifacts.insert(
            AGENT_ENDPOINTS_IR_CBOR_FILENAME.to_string(),
            canonical_cbor(&ir),
        );
    }
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
    cbor_artifacts.insert(
        "launcher-handoff.cbor".to_string(),
        canonical_cbor(&package_manifest),
    );
    cbor_artifacts.insert("views.cbor".to_string(), canonical_cbor(&ir.views));

    let inspect_json = inspect_ir(&ir);
    let agent_tools = agent_tools_json(&ir);
    let executable_contract = executable_contract_json(&ir);
    let canonical_hash = canonical_hash_hex(&ir);

    Ok(ArtifactSet {
        ir,
        package_manifest,
        cbor_artifacts,
        inspect_json,
        agent_tools_json: agent_tools,
        agent_exports,
        executable_contract_json: executable_contract,
        canonical_hash,
    })
}

pub fn build_sorla_gtpack(options: &SorlaGtpackOptions) -> Result<SorlaGtpackBuildSummary, String> {
    let yaml = fs::read_to_string(&options.input_path).map_err(|err| {
        format!(
            "failed to read SoRLa input {}: {err}",
            options.input_path.display()
        )
    })?;
    let artifacts = build_artifacts_from_yaml(&yaml)?;
    build_sorla_gtpack_from_artifacts(options, artifacts)
}

fn build_sorla_gtpack_from_artifacts(
    options: &SorlaGtpackOptions,
    artifacts: ArtifactSet,
) -> Result<SorlaGtpackBuildSummary, String> {
    semver::Version::parse(&options.version)
        .map_err(|err| format!("invalid pack version `{}`: {err}", options.version))?;
    if options.name.trim().is_empty() {
        return Err("pack name must not be empty".to_string());
    }

    let mut sorx_assets = sorx_startup_assets(&artifacts.ir);
    let sorx_startup_asset_names = sorx_assets.keys().cloned().collect();
    let validation_manifest = generate_sorx_validation_manifest_from_ir(
        &artifacts.ir,
        Some(&artifacts.canonical_hash),
        sorx_startup_asset_names,
    );
    validation_manifest
        .validate_static()
        .map_err(|err| err.to_string())?;
    sorx_assets.insert(
        SORX_VALIDATION_MANIFEST_ASSET.to_string(),
        serde_json::to_vec_pretty(&validation_manifest).map_err(|err| err.to_string())?,
    );
    let exposure_policy = generate_sorx_exposure_policy(&artifacts.ir.agent_endpoints);
    let known_endpoint_ids = artifacts
        .ir
        .agent_endpoints
        .iter()
        .map(|endpoint| endpoint.id.as_str())
        .collect();
    exposure_policy
        .validate_static(&known_endpoint_ids)
        .map_err(|err| err.to_string())?;
    sorx_assets.insert(
        SORX_EXPOSURE_POLICY_ASSET.to_string(),
        serde_json::to_vec_pretty(&exposure_policy).map_err(|err| err.to_string())?,
    );
    let compatibility_manifest =
        generate_sorx_compatibility_manifest(&artifacts.ir, Some(&artifacts.canonical_hash));
    compatibility_manifest
        .validate_static()
        .map_err(|err| err.to_string())?;
    sorx_assets.insert(
        SORX_COMPATIBILITY_ASSET.to_string(),
        serde_json::to_vec_pretty(&compatibility_manifest).map_err(|err| err.to_string())?,
    );
    let extension = sorx_runtime_extension_value(&artifacts, &sorx_assets);

    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut asset_paths = Vec::new();
    let mut cbor_artifacts: Vec<_> = artifacts.cbor_artifacts.iter().collect();
    cbor_artifacts.sort_by_key(|(name, _)| *name);
    for (name, bytes) in cbor_artifacts {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            format!("assets/sorla/{name}"),
            bytes.clone(),
        );
    }

    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        format!("assets/sorla/{AGENT_GATEWAY_HANDOFF_FILENAME}"),
        artifacts
            .agent_exports
            .agent_gateway_json
            .as_bytes()
            .to_vec(),
    );
    if let Some(openapi) = &artifacts.agent_exports.openapi_overlay_yaml {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            format!("assets/sorla/{AGENT_OPENAPI_OVERLAY_FILENAME}"),
            openapi.as_bytes().to_vec(),
        );
    }
    if let Some(arazzo) = &artifacts.agent_exports.arazzo_yaml {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            format!("assets/sorla/{AGENT_ARAZZO_FILENAME}"),
            arazzo.as_bytes().to_vec(),
        );
    }
    if let Some(mcp_tools) = &artifacts.agent_exports.mcp_tools_json {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            format!("assets/sorla/{MCP_TOOLS_FILENAME}"),
            mcp_tools.as_bytes().to_vec(),
        );
    }
    if let Some(llms_txt) = &artifacts.agent_exports.llms_txt {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            format!("assets/sorla/{LLMS_TXT_FRAGMENT_FILENAME}"),
            llms_txt.as_bytes().to_vec(),
        );
    }
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        format!("assets/sorla/{EXECUTABLE_CONTRACT_FILENAME}"),
        artifacts.executable_contract_json.as_bytes().to_vec(),
    );

    for (name, bytes) in &sorx_assets {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            format!("assets/sorx/{name}"),
            bytes.clone(),
        );
    }

    asset_paths.sort();
    asset_paths.dedup();

    let manifest = SorlaPackManifest {
        schema: "greentic.gtpack.manifest.sorla.v1".to_string(),
        pack: SorlaPackIdentity {
            name: options.name.clone(),
            version: options.version.clone(),
            kind: "application".to_string(),
        },
        created_at_utc: STABLE_PACK_TIMESTAMP.to_string(),
        extension,
        assets: asset_paths.clone(),
    };
    let pack_cbor = canonical_cbor(&manifest);
    entries.insert("pack.cbor".to_string(), pack_cbor.clone());
    entries.insert("manifest.cbor".to_string(), pack_cbor.clone());
    entries.insert(
        "manifest.json".to_string(),
        serde_json::to_string_pretty(&manifest)
            .expect("pack manifest should serialize")
            .into_bytes(),
    );
    let lock = pack_lock_for_entries(&entries);
    let lock_bytes = canonical_cbor(&lock);
    entries.insert("pack.lock.cbor".to_string(), lock_bytes);

    write_zip_entries(&options.out_path, entries)?;

    Ok(SorlaGtpackBuildSummary {
        out_path: options.out_path.display().to_string(),
        name: options.name.clone(),
        version: options.version.clone(),
        sorla_package_name: artifacts.ir.package.name,
        sorla_package_version: artifacts.ir.package.version,
        ir_hash: artifacts.canonical_hash,
        manifest_hash_sha256: sha256_hex(&pack_cbor),
        assets: asset_paths,
    })
}

fn insert_pack_asset(
    entries: &mut BTreeMap<String, Vec<u8>>,
    asset_paths: &mut Vec<String>,
    path: String,
    bytes: Vec<u8>,
) {
    asset_paths.push(path.clone());
    entries.insert(path, bytes);
}

fn sorx_startup_assets(ir: &CanonicalIr) -> BTreeMap<String, Vec<u8>> {
    let mut assets = BTreeMap::new();
    let schema = serde_json::json!({
        "schema": "greentic.sorx.start.answers.v1",
        "title": format!("{} Sorx startup answers", ir.package.name),
        "required": [
            "tenant.tenant_id",
            "server.bind",
            "server.public_base_url",
            "providers.store.kind",
            "providers.store.config_ref",
            "policy.approvals.high",
            "audit.sink"
        ],
        "example": sorx_startup_example()
    });
    assets.insert(
        START_SCHEMA_FILENAME.to_string(),
        serde_json::to_string_pretty(&schema)
            .expect("startup schema should serialize")
            .into_bytes(),
    );

    let questions = serde_json::json!({
        "schema": "greentic.sorx.start.questions.v1",
        "questions": [
            {"id": "tenant.tenant_id", "kind": "text", "required": true},
            {"id": "tenant.environment", "kind": "text", "required": false, "default": "local"},
            {"id": "server.bind", "kind": "text", "required": true, "default": "127.0.0.1:8787"},
            {"id": "server.public_base_url", "kind": "text", "required": true, "default": "http://127.0.0.1:8787"},
            {"id": "mcp.enabled", "kind": "boolean", "required": false, "default": true},
            {"id": "mcp.bind", "kind": "text", "required": false, "default": "127.0.0.1:8790"},
            {"id": "providers.store.kind", "kind": "single-select", "required": true, "choices": ["foundationdb"]},
            {"id": "providers.store.config_ref", "kind": "text", "required": true, "default": "providers.foundationdb.local"},
            {"id": "policy.approvals.high", "kind": "single-select", "required": true, "choices": ["require_approval", "deny"]},
            {"id": "audit.sink", "kind": "single-select", "required": true, "choices": ["stdout"]}
        ]
    });
    assets.insert(
        START_QUESTIONS_FILENAME.to_string(),
        canonical_cbor(&questions),
    );

    assets.insert(
        RUNTIME_TEMPLATE_FILENAME.to_string(),
        runtime_template_yaml(ir).into_bytes(),
    );
    assets.insert(
        PROVIDER_BINDINGS_TEMPLATE_FILENAME.to_string(),
        provider_bindings_template_yaml().into_bytes(),
    );
    assets
}

fn sorx_startup_example() -> serde_json::Value {
    serde_json::json!({
        "tenant": {
            "tenant_id": "demo-landlord",
            "environment": "local"
        },
        "server": {
            "bind": "127.0.0.1:8787",
            "public_base_url": "http://127.0.0.1:8787"
        },
        "mcp": {
            "enabled": true,
            "bind": "127.0.0.1:8790"
        },
        "providers": {
            "store": {
                "kind": "foundationdb",
                "config_ref": "providers.foundationdb.local"
            }
        },
        "policy": {
            "approvals": {
                "low": "auto",
                "medium": "auto",
                "high": "require_approval",
                "critical": "deny"
            }
        },
        "audit": {
            "sink": "stdout"
        }
    })
}

fn runtime_template_yaml(ir: &CanonicalIr) -> String {
    format!(
        "schema: greentic.sorx.runtime.template.v1\npackage:\n  name: {}\n  version: {}\nruntime:\n  tenant_id: ${{tenant.tenant_id}}\n  environment: ${{tenant.environment}}\nserver:\n  bind: ${{server.bind}}\n  public_base_url: ${{server.public_base_url}}\nmcp:\n  enabled: ${{mcp.enabled}}\n  bind: ${{mcp.bind}}\nproviders:\n  store:\n    kind: ${{providers.store.kind}}\n    config_ref: ${{providers.store.config_ref}}\npolicy:\n  approvals:\n    low: ${{policy.approvals.low}}\n    medium: ${{policy.approvals.medium}}\n    high: ${{policy.approvals.high}}\n    critical: ${{policy.approvals.critical}}\naudit:\n  sink: ${{audit.sink}}\n",
        ir.package.name, ir.package.version
    )
}

fn provider_bindings_template_yaml() -> String {
    "schema: greentic.sorx.provider-bindings.template.v1\nproviders:\n  foundationdb:\n    local:\n      kind: foundationdb\n      config_ref: providers.foundationdb.local\n      tenant_prefix: ${tenant.tenant_id}\n".to_string()
}

fn sorx_runtime_extension_value(
    artifacts: &ArtifactSet,
    sorx_assets: &BTreeMap<String, Vec<u8>>,
) -> serde_json::Value {
    let mut sorla = serde_json::Map::new();
    sorla.insert(
        "model".to_string(),
        serde_json::json!("assets/sorla/model.cbor"),
    );
    sorla.insert(
        "package_manifest".to_string(),
        serde_json::json!("assets/sorla/package-manifest.cbor"),
    );
    sorla.insert(
        "executable_contract".to_string(),
        serde_json::json!(format!("assets/sorla/{EXECUTABLE_CONTRACT_FILENAME}")),
    );
    sorla.insert(
        "agent_gateway".to_string(),
        serde_json::json!(format!("assets/sorla/{AGENT_GATEWAY_HANDOFF_FILENAME}")),
    );
    if artifacts
        .cbor_artifacts
        .contains_key(AGENT_ENDPOINTS_IR_CBOR_FILENAME)
    {
        sorla.insert(
            "agent_endpoints_ir".to_string(),
            serde_json::json!(format!("assets/sorla/{AGENT_ENDPOINTS_IR_CBOR_FILENAME}")),
        );
    }
    if artifacts.agent_exports.openapi_overlay_yaml.is_some() {
        sorla.insert(
            "openapi_overlay".to_string(),
            serde_json::json!(format!("assets/sorla/{AGENT_OPENAPI_OVERLAY_FILENAME}")),
        );
    }
    if artifacts.agent_exports.arazzo_yaml.is_some() {
        sorla.insert(
            "arazzo".to_string(),
            serde_json::json!(format!("assets/sorla/{AGENT_ARAZZO_FILENAME}")),
        );
    }
    if artifacts.agent_exports.mcp_tools_json.is_some() {
        sorla.insert(
            "mcp_tools".to_string(),
            serde_json::json!(format!("assets/sorla/{MCP_TOOLS_FILENAME}")),
        );
    }
    if artifacts.agent_exports.llms_txt.is_some() {
        sorla.insert(
            "llms_fragment".to_string(),
            serde_json::json!(format!("assets/sorla/{LLMS_TXT_FRAGMENT_FILENAME}")),
        );
    }

    let mut sorx = sorx_assets
        .keys()
        .filter(|name| {
            name.as_str() != SORX_VALIDATION_MANIFEST_ASSET
                && name.as_str() != SORX_EXPOSURE_POLICY_ASSET
                && name.as_str() != SORX_COMPATIBILITY_ASSET
        })
        .map(|name| {
            let key = name
                .strip_suffix(".json")
                .or_else(|| name.strip_suffix(".cbor"))
                .or_else(|| name.strip_suffix(".yaml"))
                .unwrap_or(name)
                .replace('.', "_");
            (
                key,
                serde_json::Value::String(format!("assets/sorx/{name}")),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    if sorx_assets.contains_key(SORX_VALIDATION_MANIFEST_ASSET) {
        sorx.insert(
            "validation_manifest".to_string(),
            serde_json::Value::String(SORX_VALIDATION_MANIFEST_PATH.to_string()),
        );
    }
    if sorx_assets.contains_key(SORX_EXPOSURE_POLICY_ASSET) {
        sorx.insert(
            "exposure_policy".to_string(),
            serde_json::Value::String(SORX_EXPOSURE_POLICY_PATH.to_string()),
        );
    }
    if sorx_assets.contains_key(SORX_COMPATIBILITY_ASSET) {
        sorx.insert(
            "compatibility".to_string(),
            serde_json::Value::String(SORX_COMPATIBILITY_PATH.to_string()),
        );
    }

    serde_json::json!({
        "extension": SORX_RUNTIME_EXTENSION_ID,
        "sorla": sorla,
        "sorx": sorx
    })
}

fn pack_lock_for_entries(entries: &BTreeMap<String, Vec<u8>>) -> SorlaPackLock {
    SorlaPackLock {
        schema: "greentic.gtpack.lock.sorla.v1".to_string(),
        entries: entries
            .iter()
            .map(|(path, bytes)| {
                (
                    path.clone(),
                    SorlaPackLockEntry {
                        size: bytes.len() as u64,
                        sha256: sha256_hex(bytes),
                    },
                )
            })
            .collect(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn write_zip_entries(path: &Path, entries: BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let file = fs::File::create(path)
        .map_err(|err| format!("failed to create gtpack {}: {err}", path.display()))?;
    let mut writer = ZipWriter::new(file);
    let timestamp = zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
        .map_err(|err| format!("failed to create stable zip timestamp: {err}"))?;
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(timestamp)
        .unix_permissions(0o644)
        .large_file(false);
    for (name, bytes) in entries {
        writer
            .start_file(&name, options)
            .map_err(|err| format!("failed to add {name} to gtpack: {err}"))?;
        writer
            .write_all(&bytes)
            .map_err(|err| format!("failed to write {name} to gtpack: {err}"))?;
    }
    writer
        .finish()
        .map_err(|err| format!("failed to finish gtpack archive: {err}"))?;
    Ok(())
}

pub fn inspect_sorla_gtpack(path: &Path) -> Result<SorlaGtpackInspection, String> {
    let mut archive = open_gtpack(path)?;
    let names = zip_entry_names(&mut archive)?;
    let manifest_bytes = zip_bytes(&mut archive, "pack.cbor")?;
    let manifest: SorlaPackManifest = ciborium::de::from_reader(Cursor::new(manifest_bytes))
        .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    if manifest
        .extension
        .get("extension")
        .and_then(serde_json::Value::as_str)
        != Some(SORX_RUNTIME_EXTENSION_ID)
    {
        return Err(format!(
            "pack.cbor is missing `{SORX_RUNTIME_EXTENSION_ID}` extension"
        ));
    }
    let model_bytes = zip_bytes(&mut archive, "assets/sorla/model.cbor")?;
    let ir: CanonicalIr = ciborium::de::from_reader(Cursor::new(model_bytes))
        .map_err(|err| format!("assets/sorla/model.cbor is invalid canonical IR: {err}"))?;
    let validation = validation_manifest_summary(&mut archive, &manifest, &names)?;
    let exposure_policy = exposure_policy_summary(&mut archive, &manifest, &names)?;
    let compatibility = compatibility_summary(&mut archive, &manifest, &names)?;
    let optional_artifacts = [
        AGENT_ENDPOINTS_IR_CBOR_FILENAME,
        AGENT_OPENAPI_OVERLAY_FILENAME,
        AGENT_ARAZZO_FILENAME,
        MCP_TOOLS_FILENAME,
        LLMS_TXT_FRAGMENT_FILENAME,
    ]
    .into_iter()
    .map(|name| {
        (
            format!("assets/sorla/{name}"),
            names.contains(&format!("assets/sorla/{name}")),
        )
    })
    .collect();

    Ok(SorlaGtpackInspection {
        path: path.display().to_string(),
        name: manifest.pack.name,
        version: manifest.pack.version,
        extension: manifest
            .extension
            .get("extension")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(SORX_RUNTIME_EXTENSION_ID)
            .to_string(),
        sorla_package_name: ir.package.name.clone(),
        sorla_package_version: ir.package.version.clone(),
        ir_hash: canonical_hash_hex(&ir),
        assets: names
            .into_iter()
            .filter(|name| name.starts_with("assets/"))
            .collect(),
        optional_artifacts,
        validation,
        exposure_policy,
        compatibility,
    })
}

fn sorx_extension_path(
    manifest: &SorlaPackManifest,
    key: &str,
    expected: &str,
) -> Result<Option<String>, String> {
    let Some(path) = manifest
        .extension
        .get("sorx")
        .and_then(|sorx| sorx.get(key))
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(None);
    };

    if path != expected {
        return Err(format!(
            "pack.cbor references unsupported SORX asset path `{path}` for `{key}`"
        ));
    }
    Ok(Some(path.to_string()))
}

fn validation_manifest_path(manifest: &SorlaPackManifest) -> Result<Option<String>, String> {
    sorx_extension_path(
        manifest,
        "validation_manifest",
        SORX_VALIDATION_MANIFEST_PATH,
    )
}

fn exposure_policy_path(manifest: &SorlaPackManifest) -> Result<Option<String>, String> {
    sorx_extension_path(manifest, "exposure_policy", SORX_EXPOSURE_POLICY_PATH)
}

fn compatibility_path(manifest: &SorlaPackManifest) -> Result<Option<String>, String> {
    sorx_extension_path(manifest, "compatibility", SORX_COMPATIBILITY_PATH)
}

fn compatibility_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackCompatibilityInspection>, String> {
    let Some(path) = compatibility_path(manifest)? else {
        return Ok(None);
    };
    if !names.contains(&path) {
        return Err(format!(
            "pack.cbor references missing compatibility manifest `{path}`"
        ));
    }

    let compatibility = read_compatibility_manifest(archive, &path)?;
    compatibility
        .validate_static()
        .map_err(|err| err.to_string())?;
    Ok(Some(SorlaGtpackCompatibilityInspection {
        api_mode: compatibility.api_compatibility,
        state_mode: compatibility.state_compatibility,
        provider_requirement_count: compatibility.provider_compatibility.len(),
        migration_rule_count: compatibility.migration_compatibility.len(),
    }))
}

fn exposure_policy_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackExposurePolicyInspection>, String> {
    let Some(path) = exposure_policy_path(manifest)? else {
        return Ok(None);
    };
    if !names.contains(&path) {
        return Err(format!(
            "pack.cbor references missing exposure policy `{path}`"
        ));
    }

    let policy = read_exposure_policy(archive, &path)?;
    Ok(Some(SorlaGtpackExposurePolicyInspection {
        default_visibility: policy.default_visibility,
        public_candidate_endpoints: policy
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.visibility == EndpointVisibility::PublicCandidate)
            .count(),
        approval_required_endpoints: policy
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.requires_approval)
            .count(),
    }))
}

fn validation_manifest_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackValidationInspection>, String> {
    let Some(path) = validation_manifest_path(manifest)? else {
        return Ok(None);
    };
    if !names.contains(&path) {
        return Err(format!(
            "pack.cbor references missing validation manifest `{path}`"
        ));
    }

    let validation = read_validation_manifest(archive, &path)?;
    validation
        .validate_static()
        .map_err(|err| err.to_string())?;
    Ok(Some(SorlaGtpackValidationInspection {
        schema: validation.schema,
        suite_count: validation.suites.len(),
        test_count: validation
            .suites
            .iter()
            .map(|suite| suite.tests.len())
            .sum(),
        promotion_requires: validation.promotion_requires,
    }))
}

fn validate_embedded_sorx_validation<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let path = validation_manifest_path(&pack_manifest)?
        .ok_or_else(|| "pack.cbor is missing `sorx.validation_manifest`".to_string())?;
    if !names.contains(&path) {
        return Err(format!(
            "pack.cbor references missing validation manifest `{path}`"
        ));
    }
    if !pack_manifest.assets.iter().any(|asset| asset == &path) {
        return Err(format!("pack.cbor assets do not include `{path}`"));
    }
    validate_lock_includes_entry(archive, &path)?;

    let validation = read_validation_manifest(archive, &path)?;
    validation
        .validate_static()
        .map_err(|err| err.to_string())?;
    if validation.package.name != ir.package.name {
        return Err(format!(
            "validation manifest package.name `{}` does not match SoRLa package `{}`",
            validation.package.name, ir.package.name
        ));
    }
    if validation.package.version != ir.package.version {
        return Err(format!(
            "validation manifest package.version `{}` does not match SoRLa package `{}`",
            validation.package.version, ir.package.version
        ));
    }
    for suite in &validation.suites {
        for test in &suite.tests {
            for reference in test.referenced_asset_paths() {
                let asset_path = format!("assets/sorx/tests/{reference}");
                if !names.contains(&asset_path) {
                    return Err(format!(
                        "validation manifest references missing asset `{asset_path}`"
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_embedded_sorx_exposure_policy<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let path = exposure_policy_path(&pack_manifest)?
        .ok_or_else(|| "pack.cbor is missing `sorx.exposure_policy`".to_string())?;
    if !names.contains(&path) {
        return Err(format!(
            "pack.cbor references missing exposure policy `{path}`"
        ));
    }
    if !pack_manifest.assets.iter().any(|asset| asset == &path) {
        return Err(format!("pack.cbor assets do not include `{path}`"));
    }
    validate_lock_includes_entry(archive, &path)?;

    let known_endpoint_ids = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| endpoint.id.as_str())
        .collect();
    let policy = read_exposure_policy(archive, &path)?;
    policy
        .validate_static(&known_endpoint_ids)
        .map_err(|err| err.to_string())?;
    validate_exposure_policy_against_validation(archive, &policy)?;
    Ok(())
}

fn validate_embedded_sorx_compatibility<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let path = compatibility_path(&pack_manifest)?
        .ok_or_else(|| "pack.cbor is missing `sorx.compatibility`".to_string())?;
    if !names.contains(&path) {
        return Err(format!(
            "pack.cbor references missing compatibility manifest `{path}`"
        ));
    }
    if !pack_manifest.assets.iter().any(|asset| asset == &path) {
        return Err(format!("pack.cbor assets do not include `{path}`"));
    }
    validate_lock_includes_entry(archive, &path)?;

    let compatibility = read_compatibility_manifest(archive, &path)?;
    compatibility
        .validate_static()
        .map_err(|err| err.to_string())?;
    if compatibility.package.name != ir.package.name {
        return Err(format!(
            "compatibility manifest package.name `{}` does not match SoRLa package `{}`",
            compatibility.package.name, ir.package.name
        ));
    }
    if compatibility.package.version != ir.package.version {
        return Err(format!(
            "compatibility manifest package.version `{}` does not match SoRLa package `{}`",
            compatibility.package.version, ir.package.version
        ));
    }
    Ok(())
}

fn validate_exposure_policy_against_validation<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    policy: &SorxExposurePolicy,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let validation_path = validation_manifest_path(&pack_manifest)?
        .ok_or_else(|| "pack.cbor is missing `sorx.validation_manifest`".to_string())?;
    let validation = read_validation_manifest(archive, &validation_path)?;
    if policy
        .promotion_requires
        .iter()
        .any(|requirement| requirement == "validation_success")
        && validation.promotion_requires.is_empty()
    {
        return Err(
            "exposure policy requires validation_success but validation has no promotion suites"
                .to_string(),
        );
    }
    Ok(())
}

fn read_validation_manifest<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<SorxValidationManifest, String> {
    serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))
}

fn read_exposure_policy<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<SorxExposurePolicy, String> {
    serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))
}

fn read_compatibility_manifest<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<SorxCompatibilityManifest, String> {
    serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))
}

fn validate_lock_includes_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<(), String> {
    let lock_bytes = zip_bytes(archive, "pack.lock.cbor")?;
    let lock: SorlaPackLock = ciborium::de::from_reader(Cursor::new(lock_bytes))
        .map_err(|err| format!("pack.lock.cbor is invalid CBOR: {err}"))?;
    if !lock.entries.contains_key(path) {
        return Err(format!(
            "pack.lock.cbor is missing validation asset `{path}`"
        ));
    }
    Ok(())
}

pub fn doctor_sorla_gtpack(path: &Path) -> Result<SorlaGtpackDoctorReport, String> {
    let inspection = inspect_sorla_gtpack(path)?;
    let mut archive = open_gtpack(path)?;
    let names = zip_entry_names(&mut archive)?;
    for required in required_pack_entries() {
        if !names.contains(required) {
            return Err(format!("gtpack is missing required entry `{required}`"));
        }
    }

    validate_pack_lock_entries(&mut archive, &names)?;

    let gateway: AgentGatewayHandoffManifest =
        serde_json::from_str(&zip_text(&mut archive, "assets/sorla/agent-gateway.json")?)
            .map_err(|err| format!("agent-gateway.json is invalid JSON: {err}"))?;
    let model_bytes = zip_bytes(&mut archive, "assets/sorla/model.cbor")?;
    let ir: CanonicalIr = ciborium::de::from_reader(Cursor::new(model_bytes))
        .map_err(|err| format!("model.cbor is invalid canonical IR: {err}"))?;
    validate_embedded_sorx_validation(&mut archive, &names, &ir)?;
    validate_embedded_sorx_exposure_policy(&mut archive, &names, &ir)?;
    validate_embedded_sorx_compatibility(&mut archive, &names, &ir)?;
    let endpoint_ids: BTreeSet<_> = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| endpoint.id.as_str())
        .collect();
    for endpoint in &gateway.endpoints {
        if !endpoint_ids.contains(endpoint.id.as_str()) {
            return Err(format!(
                "agent-gateway.json references unknown endpoint `{}`",
                endpoint.id
            ));
        }
    }

    if names.contains(&format!("assets/sorla/{MCP_TOOLS_FILENAME}")) {
        let mcp: serde_json::Value = serde_json::from_str(&zip_text(
            &mut archive,
            &format!("assets/sorla/{MCP_TOOLS_FILENAME}"),
        )?)
        .map_err(|err| format!("mcp-tools.json is invalid JSON: {err}"))?;
        for tool in mcp
            .get("tools")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let name = tool
                .get("name")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "mcp-tools.json has a tool without a name".to_string())?;
            if !endpoint_ids.contains(name) {
                return Err(format!(
                    "mcp-tools.json references unknown endpoint `{name}`"
                ));
            }
        }
    }

    let startup_schema: serde_json::Value =
        serde_json::from_str(&zip_text(&mut archive, "assets/sorx/start.schema.json")?)
            .map_err(|err| format!("start.schema.json is invalid JSON: {err}"))?;
    for required in [
        "tenant.tenant_id",
        "server.bind",
        "server.public_base_url",
        "providers.store.kind",
        "providers.store.config_ref",
        "policy.approvals.high",
        "audit.sink",
    ] {
        let has_required = startup_schema
            .get("required")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(required)));
        if !has_required {
            return Err(format!(
                "start.schema.json is missing required path `{required}`"
            ));
        }
    }

    reject_secret_markers(&mut archive)?;

    Ok(SorlaGtpackDoctorReport {
        path: inspection.path,
        status: "ok".to_string(),
        checked_assets: required_pack_entries()
            .into_iter()
            .map(str::to_string)
            .collect(),
    })
}

fn required_pack_entries() -> Vec<&'static str> {
    vec![
        "pack.cbor",
        "pack.lock.cbor",
        "manifest.cbor",
        "assets/sorla/model.cbor",
        "assets/sorla/package-manifest.cbor",
        "assets/sorla/executable-contract.json",
        "assets/sorla/agent-gateway.json",
        "assets/sorx/start.schema.json",
        "assets/sorx/start.questions.cbor",
        "assets/sorx/runtime.template.yaml",
        "assets/sorx/provider-bindings.template.yaml",
        SORX_COMPATIBILITY_PATH,
        SORX_EXPOSURE_POLICY_PATH,
        SORX_VALIDATION_MANIFEST_PATH,
    ]
}

fn open_gtpack(path: &Path) -> Result<ZipArchive<fs::File>, String> {
    let file = fs::File::open(path)
        .map_err(|err| format!("failed to open gtpack {}: {err}", path.display()))?;
    ZipArchive::new(file).map_err(|err| format!("failed to read gtpack {}: {err}", path.display()))
}

fn zip_entry_names<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|err| format!("failed to inspect gtpack entry {index}: {err}"))?;
        if !entry.is_dir() {
            names.insert(entry.name().to_string());
        }
    }
    Ok(names)
}

fn zip_bytes<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Result<Vec<u8>, String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|err| format!("gtpack is missing `{name}`: {err}"))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|err| format!("failed to read `{name}`: {err}"))?;
    Ok(bytes)
}

fn zip_text<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Result<String, String> {
    let bytes = zip_bytes(archive, name)?;
    String::from_utf8(bytes).map_err(|err| format!("`{name}` is not UTF-8: {err}"))
}

fn validate_pack_lock_entries<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
) -> Result<(), String> {
    let lock_bytes = zip_bytes(archive, "pack.lock.cbor")?;
    let lock: SorlaPackLock = ciborium::de::from_reader(Cursor::new(lock_bytes))
        .map_err(|err| format!("pack.lock.cbor is invalid CBOR: {err}"))?;
    if lock.schema != "greentic.gtpack.lock.sorla.v1" {
        return Err(format!(
            "pack.lock.cbor has unsupported schema `{}`",
            lock.schema
        ));
    }
    for (path, expected) in &lock.entries {
        if !names.contains(path) {
            return Err(format!("pack.lock.cbor references missing entry `{path}`"));
        }
        let bytes = zip_bytes(archive, path)?;
        if expected.size != bytes.len() as u64 {
            return Err(format!("pack.lock.cbor size mismatch for `{path}`"));
        }
        let actual = sha256_hex(&bytes);
        if expected.sha256 != actual {
            return Err(format!("pack.lock.cbor digest mismatch for `{path}`"));
        }
    }
    Ok(())
}

fn reject_secret_markers<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<(), String> {
    const MARKERS: &[&str] = &[
        "BEGIN PRIVATE KEY",
        "api_key:",
        "access_token:",
        "refresh_token:",
        "client_secret:",
        "password:",
    ];
    let names = zip_entry_names(archive)?;
    for name in names {
        let bytes = zip_bytes(archive, &name)?;
        let text = String::from_utf8_lossy(&bytes).to_ascii_lowercase();
        for marker in MARKERS {
            if text.contains(&marker.to_ascii_lowercase()) {
                return Err(format!(
                    "gtpack entry `{name}` appears to contain `{marker}`"
                ));
            }
        }
    }
    Ok(())
}

pub fn executable_contract_json(ir: &CanonicalIr) -> String {
    let relationships: Vec<_> = ir
        .records
        .iter()
        .flat_map(|record| {
            record.fields.iter().filter_map(move |field| {
                field.references.as_ref().map(|reference| {
                    serde_json::json!({
                        "record": record.name,
                        "field": field.name,
                        "references": {
                            "record": reference.record,
                            "field": reference.field
                        }
                    })
                })
            })
        })
        .collect();

    let migrations: Vec<_> = ir
        .compatibility
        .iter()
        .map(|migration| {
            serde_json::json!({
                "name": migration.name,
                "compatibility": migration.compatibility,
                "projection_updates": migration.projection_updates,
                "backfills": migration.backfills,
                "idempotence_key": migration.idempotence_key
            })
        })
        .collect();

    let agent_operations: Vec<_> = ir
        .agent_endpoints
        .iter()
        .filter_map(|endpoint| {
            endpoint.emits.as_ref().map(|emit| {
                serde_json::json!({
                    "endpoint_id": endpoint.id,
                    "emits": emit
                })
            })
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "greentic.sorla.executable-contract.v1",
        "package": {
            "name": ir.package.name,
            "version": ir.package.version,
            "ir_hash": canonical_hash_hex(ir)
        },
        "relationships": relationships,
        "migrations": migrations,
        "agent_operations": agent_operations,
        "operation_result_contract": {
            "schema": "greentic.sorla.operation-result.v1",
            "fields": {
                "endpoint_id": "string",
                "status": ["ok", "validation_error", "provider_error"],
                "data": "object",
                "errors": [
                    {
                        "path": "string",
                        "code": "string",
                        "message": "string"
                    }
                ],
                "provider_message": "string"
            }
        }
    }))
    .expect("executable contract should serialize")
}

pub fn agent_gateway_handoff_manifest(ir: &CanonicalIr) -> AgentGatewayHandoffManifest {
    let endpoints: Vec<AgentGatewayEndpointRef> = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| AgentGatewayEndpointRef {
            id: endpoint.id.clone(),
            title: endpoint.title.clone(),
            intent: endpoint.intent.clone(),
            risk: agent_endpoint_risk_label(&endpoint.risk).to_string(),
            approval: agent_endpoint_approval_label(&endpoint.approval).to_string(),
            inputs: endpoint
                .inputs
                .iter()
                .map(|input| input.name.clone())
                .collect(),
            outputs: endpoint
                .outputs
                .iter()
                .map(|output| output.name.clone())
                .collect(),
            side_effects: endpoint.side_effects.clone(),
            exports: AgentGatewayEndpointExports {
                openapi: endpoint.agent_visibility.openapi,
                arazzo: endpoint.agent_visibility.arazzo,
                mcp: endpoint.agent_visibility.mcp,
                llms_txt: endpoint.agent_visibility.llms_txt,
            },
        })
        .collect();

    let exports = AgentGatewayExports {
        agent_gateway_json: true,
        openapi_overlay: endpoints.iter().any(|endpoint| endpoint.exports.openapi),
        arazzo: endpoints.iter().any(|endpoint| endpoint.exports.arazzo),
        mcp_tools: endpoints.iter().any(|endpoint| endpoint.exports.mcp),
        llms_txt: endpoints.iter().any(|endpoint| endpoint.exports.llms_txt),
    };

    AgentGatewayHandoffManifest {
        schema: AGENT_GATEWAY_HANDOFF_SCHEMA.to_string(),
        package: AgentGatewayPackageRef {
            name: ir.package.name.clone(),
            version: ir.package.version.clone(),
            ir_version: format!("{}.{}", ir.ir_version.major, ir.ir_version.minor),
            ir_hash: canonical_hash_hex(ir),
        },
        endpoints,
        provider_contract: AgentGatewayProviderContract {
            categories: aggregated_provider_requirements(ir),
        },
        exports,
        notes: vec![
            "This is handoff metadata for downstream assembly, not final runtime gateway behavior."
                .to_string(),
        ],
    }
}

pub fn export_agent_artifacts(ir: &CanonicalIr) -> AgentExportSet {
    let manifest = agent_gateway_handoff_manifest(ir);
    let agent_gateway_json =
        serde_json::to_string_pretty(&manifest).expect("agent gateway manifest should serialize");

    AgentExportSet {
        agent_gateway_json,
        openapi_overlay_yaml: (!visible_openapi_endpoints(ir).is_empty())
            .then(|| openapi_overlay_yaml(ir)),
        arazzo_yaml: (!visible_arazzo_endpoints(ir).is_empty()).then(|| arazzo_yaml(ir)),
        mcp_tools_json: (!visible_mcp_endpoints(ir).is_empty()).then(|| mcp_tools_json(ir)),
        llms_txt: (!visible_llms_txt_endpoints(ir).is_empty()).then(|| llms_txt_fragment(ir)),
    }
}

pub fn build_artifacts_from_yaml(input: &str) -> Result<ArtifactSet, String> {
    build_handoff_artifacts_from_yaml(input)
}

fn provider_view(requirement: &ProviderRequirementIr) -> ProviderRequirementView {
    ProviderRequirementView {
        category: requirement.category.clone(),
        capabilities: requirement.capabilities.clone(),
    }
}

fn aggregated_provider_requirements(ir: &CanonicalIr) -> Vec<AgentGatewayProviderRequirement> {
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
        .map(|(category, capabilities)| AgentGatewayProviderRequirement {
            category,
            capabilities: capabilities.into_iter().collect(),
        })
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

fn agent_endpoint_risk_label(risk: &AgentEndpointRiskIr) -> &'static str {
    match risk {
        AgentEndpointRiskIr::Low => "low",
        AgentEndpointRiskIr::Medium => "medium",
        AgentEndpointRiskIr::High => "high",
    }
}

fn agent_endpoint_approval_label(approval: &AgentEndpointApprovalModeIr) -> &'static str {
    match approval {
        AgentEndpointApprovalModeIr::None => "none",
        AgentEndpointApprovalModeIr::Optional => "optional",
        AgentEndpointApprovalModeIr::Required => "required",
        AgentEndpointApprovalModeIr::PolicyDriven => "policy-driven",
    }
}

fn visible_openapi_endpoints(ir: &CanonicalIr) -> Vec<&AgentEndpointIr> {
    ir.agent_endpoints
        .iter()
        .filter(|endpoint| endpoint.agent_visibility.openapi)
        .collect()
}

fn visible_arazzo_endpoints(ir: &CanonicalIr) -> Vec<&AgentEndpointIr> {
    ir.agent_endpoints
        .iter()
        .filter(|endpoint| endpoint.agent_visibility.arazzo)
        .collect()
}

fn visible_mcp_endpoints(ir: &CanonicalIr) -> Vec<&AgentEndpointIr> {
    ir.agent_endpoints
        .iter()
        .filter(|endpoint| endpoint.agent_visibility.mcp)
        .collect()
}

fn visible_llms_txt_endpoints(ir: &CanonicalIr) -> Vec<&AgentEndpointIr> {
    ir.agent_endpoints
        .iter()
        .filter(|endpoint| endpoint.agent_visibility.llms_txt)
        .collect()
}

fn openapi_overlay_yaml(ir: &CanonicalIr) -> String {
    let operations = visible_openapi_endpoints(ir)
        .into_iter()
        .map(|endpoint| {
            serde_json::json!({
                "operationId": format!("agent_{}", endpoint.id),
                "x-greentic-agent": {
                    "endpoint_id": endpoint.id,
                    "intent": endpoint.intent,
                    "risk": agent_endpoint_risk_label(&endpoint.risk),
                    "approval": agent_endpoint_approval_label(&endpoint.approval),
                    "side_effects": endpoint.side_effects,
                    "inputs": endpoint.inputs.iter().map(openapi_input_value).collect::<Vec<_>>(),
                    "outputs": endpoint.outputs.iter().map(output_value).collect::<Vec<_>>()
                }
            })
        })
        .collect::<Vec<_>>();

    serialize_yaml(serde_json::json!({
        "schema": OPENAPI_AGENT_OVERLAY_SCHEMA,
        "package": ir.package.name,
        "operations": operations
    }))
}

fn arazzo_yaml(ir: &CanonicalIr) -> String {
    let workflows = visible_arazzo_endpoints(ir)
        .into_iter()
        .map(|endpoint| {
            serde_json::json!({
                "workflowId": endpoint.id,
                "summary": endpoint.title,
                "description": endpoint.intent,
                "inputs": object_schema_value(&endpoint.inputs),
                "steps": [
                    {
                        "stepId": format!("request_{}", endpoint.id),
                        "description": format!("Request downstream Greentic execution for {}.", endpoint.id)
                    }
                ]
            })
        })
        .collect::<Vec<_>>();

    serialize_yaml(serde_json::json!({
        "arazzo": "1.0.1",
        "info": {
            "title": format!("{} agent workflows", ir.package.name),
            "version": ir.package.version
        },
        "sourceDescriptions": [],
        "workflows": workflows
    }))
}

fn mcp_tools_json(ir: &CanonicalIr) -> String {
    let tools = visible_mcp_endpoints(ir)
        .into_iter()
        .map(|endpoint| {
            serde_json::json!({
                "name": endpoint.id,
                "title": endpoint.title,
                "description": endpoint.intent,
                "inputSchema": object_schema_value(&endpoint.inputs),
                "annotations": {
                    "risk": agent_endpoint_risk_label(&endpoint.risk),
                    "approval": agent_endpoint_approval_label(&endpoint.approval),
                    "side_effects": endpoint.side_effects
                }
            })
        })
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&serde_json::json!({
        "schema": MCP_TOOLS_HANDOFF_SCHEMA,
        "tools": tools
    }))
    .expect("MCP tools handoff should serialize")
}

fn llms_txt_fragment(ir: &CanonicalIr) -> String {
    let mut lines = vec![
        format!("# {} agent endpoints", ir.package.name),
        String::new(),
        "This package exposes handoff metadata for business-safe agent endpoints.".to_string(),
    ];

    for endpoint in visible_llms_txt_endpoints(ir) {
        lines.push(String::new());
        lines.push(format!("## {}", endpoint.id));
        lines.push(String::new());
        lines.push(format!("Intent: {}", endpoint.intent));
        lines.push(format!(
            "Risk: {}",
            agent_endpoint_risk_label(&endpoint.risk)
        ));
        lines.push(format!(
            "Approval: {}",
            agent_endpoint_approval_label(&endpoint.approval)
        ));
        lines.push(format!(
            "Side effects: {}",
            join_or_none(&endpoint.side_effects)
        ));
        lines.push(format!(
            "Required inputs: {}",
            join_or_none(
                &endpoint
                    .inputs
                    .iter()
                    .filter(|input| input.required)
                    .map(|input| input.name.clone())
                    .collect::<Vec<_>>()
            )
        ));
        lines.push(format!(
            "Outputs: {}",
            join_or_none(
                &endpoint
                    .outputs
                    .iter()
                    .map(|output| output.name.clone())
                    .collect::<Vec<_>>()
            )
        ));
    }

    lines.join("\n") + "\n"
}

fn openapi_input_value(input: &AgentEndpointInputIr) -> serde_json::Value {
    serde_json::json!({
        "name": input.name,
        "type": input.type_name,
        "required": input.required,
        "sensitive": input.sensitive
    })
}

fn output_value(output: &AgentEndpointOutputIr) -> serde_json::Value {
    serde_json::json!({
        "name": output.name,
        "type": output.type_name
    })
}

fn object_schema_value(inputs: &[AgentEndpointInputIr]) -> serde_json::Value {
    let required = inputs
        .iter()
        .filter(|input| input.required)
        .map(|input| input.name.clone())
        .collect::<Vec<_>>();
    let properties = inputs
        .iter()
        .map(|input| {
            let description = input
                .sensitive
                .then_some("Sensitive input")
                .or(input.description.as_deref());
            let mut property = serde_json::Map::new();
            property.insert(
                "type".to_string(),
                serde_json::Value::String(input.type_name.clone()),
            );
            if let Some(description) = description {
                property.insert(
                    "description".to_string(),
                    serde_json::Value::String(description.to_string()),
                );
            }
            if !input.enum_values.is_empty() {
                property.insert("enum".to_string(), serde_json::json!(input.enum_values));
            }
            (input.name.clone(), serde_json::Value::Object(property))
        })
        .collect::<serde_json::Map<_, _>>();

    serde_json::json!({
        "type": "object",
        "required": required,
        "properties": properties
    })
}

fn serialize_yaml(value: serde_json::Value) -> String {
    serde_yaml::to_string(&value).expect("agent YAML handoff should serialize")
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use tempfile::tempdir;
    use zip::ZipArchive;

    #[test]
    fn scaffold_handoff_manifest_stays_provider_agnostic() {
        let manifest = scaffold_handoff_manifest();
        assert_eq!(manifest.package_kind, "sorla-package");
        assert_eq!(manifest.provider_repo, "greentic-sorla-providers");
        assert!(
            manifest
                .artifact_references
                .contains(&"provider-contract.cbor".to_string())
        );
    }

    #[test]
    fn legacy_manifest_api_maps_to_handoff_manifest() {
        let manifest = scaffold_manifest();
        assert_eq!(manifest.package_kind, "sorla-package");
        assert!(
            manifest
                .artifact_references
                .contains(&"package-manifest.cbor".to_string())
        );
        assert!(
            manifest
                .artifact_references
                .contains(&"launcher-handoff.cbor".to_string())
        );
    }

    #[test]
    fn builds_deterministic_handoff_artifacts_for_golden_fixture() {
        let fixture = fs::read_to_string("tests/golden/tenant_v0_2.sorla.yaml")
            .expect("fixture should be readable");
        let expected_inspect = fs::read_to_string("tests/golden/tenant_v0_2.inspect.json")
            .expect("golden should be readable");

        let first = build_handoff_artifacts_from_yaml(&fixture).expect("fixture should build");
        let second = build_handoff_artifacts_from_yaml(&fixture).expect("fixture should build");

        assert_eq!(first.inspect_json.trim_end(), expected_inspect.trim_end());
        assert_eq!(first.inspect_json, second.inspect_json);
        assert_eq!(first.canonical_hash, second.canonical_hash);
        assert!(first.cbor_artifacts.contains_key("model.cbor"));
        assert!(first.cbor_artifacts.contains_key("events.cbor"));
        assert!(first.cbor_artifacts.contains_key("projections.cbor"));
        assert!(first.cbor_artifacts.contains_key("provider-contract.cbor"));
        assert!(
            !first
                .cbor_artifacts
                .contains_key(AGENT_ENDPOINTS_IR_CBOR_FILENAME)
        );
        assert!(first.cbor_artifacts.contains_key("launcher-handoff.cbor"));
        assert!(first.agent_exports.agent_gateway_json.contains(
            "This is handoff metadata for downstream assembly, not final runtime gateway behavior."
        ));
        assert!(first.agent_tools_json.contains("storage"));
        assert_eq!(
            first.handoff_manifest().provider_repo,
            "greentic-sorla-providers"
        );
    }

    #[test]
    fn legacy_artifact_builder_maps_to_handoff_builder() {
        let fixture = fs::read_to_string("tests/golden/tenant_v0_2.sorla.yaml")
            .expect("fixture should be readable");
        let artifacts = build_artifacts_from_yaml(&fixture).expect("fixture should build");
        assert!(
            artifacts
                .cbor_artifacts
                .contains_key("package-manifest.cbor")
        );
        assert!(
            artifacts
                .cbor_artifacts
                .contains_key("launcher-handoff.cbor")
        );
    }

    #[test]
    fn builds_agent_endpoint_fixture_end_to_end() {
        let fixture =
            fs::read_to_string("tests/golden/customer_contact_agent_endpoints.sorla.yaml")
                .expect("agent endpoint fixture should be readable");
        let expected_inspect =
            fs::read_to_string("tests/golden/customer_contact_agent_endpoints.inspect.json")
                .expect("agent endpoint inspect golden should be readable");
        let expected_gateway =
            fs::read_to_string("tests/golden/customer_contact_agent_endpoints.agent-gateway.json")
                .expect("agent gateway golden should be readable");

        let parsed = parse_package(&fixture).expect("agent endpoint fixture should parse");
        let ir = lower_package(&parsed.package);
        let first_exports = export_agent_artifacts(&ir);
        let second_exports = export_agent_artifacts(&ir);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let built =
            build_artifacts_from_yaml(&fixture).expect("agent endpoint fixture should build");

        assert_eq!(inspect_ir(&ir).trim_end(), expected_inspect.trim_end());
        assert_eq!(first_exports, second_exports);
        assert_eq!(manifest.package.ir_hash, canonical_hash_hex(&ir));
        assert!(
            built
                .cbor_artifacts
                .contains_key(AGENT_ENDPOINTS_IR_CBOR_FILENAME)
        );
        assert_eq!(
            built
                .cbor_artifacts
                .get(AGENT_ENDPOINTS_IR_CBOR_FILENAME)
                .expect("agent endpoint IR CBOR should be emitted"),
            &canonical_cbor(&ir)
        );
        for artifact in [
            AGENT_GATEWAY_HANDOFF_FILENAME,
            AGENT_ENDPOINTS_IR_CBOR_FILENAME,
            AGENT_OPENAPI_OVERLAY_FILENAME,
            AGENT_ARAZZO_FILENAME,
            MCP_TOOLS_FILENAME,
            LLMS_TXT_FRAGMENT_FILENAME,
        ] {
            assert!(
                built
                    .package_manifest
                    .artifact_references
                    .contains(&artifact.to_string()),
                "expected package manifest to reference {artifact}"
            );
        }
        assert_eq!(built.agent_exports, first_exports);

        let mut expected_gateway_value: serde_json::Value =
            serde_json::from_str(&expected_gateway).expect("agent gateway golden should parse");
        expected_gateway_value["package"]["ir_hash"] =
            serde_json::Value::String(canonical_hash_hex(&ir));
        let actual_gateway_value: serde_json::Value = serde_json::from_str(
            &serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest JSON should parse");
        assert_eq!(actual_gateway_value, expected_gateway_value);

        assert!(first_exports.openapi_overlay_yaml.is_some());
        assert!(first_exports.arazzo_yaml.is_some());
        assert!(first_exports.mcp_tools_json.is_some());
        assert!(first_exports.llms_txt.is_some());
        assert!(
            first_exports
                .llms_txt
                .as_deref()
                .expect("llms.txt fragment should be generated")
                .contains("Capture a customer enquiry")
        );
    }

    #[test]
    fn builds_deterministic_landlord_tenant_gtpack() {
        let temp = tempdir().expect("tempdir");
        let input = PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml");
        let first_out = temp.path().join("first.gtpack");
        let second_out = temp.path().join("second.gtpack");
        let options = |out_path: PathBuf| SorlaGtpackOptions {
            input_path: input.clone(),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path,
        };

        let first = build_sorla_gtpack(&options(first_out.clone())).expect("first pack builds");
        let second = build_sorla_gtpack(&options(second_out.clone())).expect("second pack builds");

        assert_eq!(first.name, "landlord-tenant-sor");
        assert_eq!(first.sorla_package_name, "landlord-tenant-sor");
        assert_eq!(
            fs::read(&first_out).unwrap(),
            fs::read(&second_out).unwrap()
        );
        assert_eq!(first.ir_hash, second.ir_hash);
        assert!(
            first
                .assets
                .contains(&"assets/sorla/model.cbor".to_string())
        );
        assert!(
            first
                .assets
                .contains(&format!("assets/sorla/{AGENT_GATEWAY_HANDOFF_FILENAME}"))
        );
        assert!(
            first
                .assets
                .contains(&format!("assets/sorla/{MCP_TOOLS_FILENAME}"))
        );

        let inspection = inspect_sorla_gtpack(&first_out).expect("inspect pack");
        assert_eq!(inspection.extension, SORX_RUNTIME_EXTENSION_ID);
        assert_eq!(inspection.sorla_package_name, "landlord-tenant-sor");
        let validation = inspection
            .validation
            .as_ref()
            .expect("inspect should summarize validation manifest");
        assert_eq!(validation.schema, SORX_VALIDATION_SCHEMA);
        assert!(validation.suite_count >= 1);
        assert!(validation.test_count >= 1);
        let exposure = inspection
            .exposure_policy
            .as_ref()
            .expect("inspect should summarize exposure policy");
        assert_eq!(exposure.default_visibility, EndpointVisibility::Private);
        assert!(exposure.public_candidate_endpoints >= 1);
        assert!(exposure.approval_required_endpoints >= 1);
        let compatibility = inspection
            .compatibility
            .as_ref()
            .expect("inspect should summarize compatibility manifest");
        assert!(compatibility.provider_requirement_count >= 1);
        assert_eq!(
            compatibility.state_mode,
            StateCompatibilityMode::IsolatedRequired
        );
        assert_eq!(
            inspection
                .optional_artifacts
                .get(&format!("assets/sorla/{AGENT_OPENAPI_OVERLAY_FILENAME}")),
            Some(&true)
        );
        doctor_sorla_gtpack(&first_out).expect("doctor accepts pack");

        let mut archive =
            ZipArchive::new(fs::File::open(&first_out).expect("open pack")).expect("read pack");
        for required in required_pack_entries() {
            archive.by_name(required).expect("required entry exists");
        }
        let pack_manifest: SorlaPackManifest = ciborium::de::from_reader(Cursor::new(
            zip_bytes(&mut archive, "pack.cbor").expect("pack.cbor"),
        ))
        .expect("pack manifest decodes");
        assert_eq!(
            pack_manifest
                .extension
                .get("sorx")
                .and_then(|sorx| sorx.get("validation_manifest"))
                .and_then(serde_json::Value::as_str),
            Some(SORX_VALIDATION_MANIFEST_PATH)
        );
        assert_eq!(
            pack_manifest
                .extension
                .get("sorx")
                .and_then(|sorx| sorx.get("exposure_policy"))
                .and_then(serde_json::Value::as_str),
            Some(SORX_EXPOSURE_POLICY_PATH)
        );
        assert_eq!(
            pack_manifest
                .extension
                .get("sorx")
                .and_then(|sorx| sorx.get("compatibility"))
                .and_then(serde_json::Value::as_str),
            Some(SORX_COMPATIBILITY_PATH)
        );
        assert!(
            pack_manifest
                .assets
                .contains(&SORX_VALIDATION_MANIFEST_PATH.to_string())
        );
        assert!(
            pack_manifest
                .assets
                .contains(&SORX_EXPOSURE_POLICY_PATH.to_string())
        );
        assert!(
            pack_manifest
                .assets
                .contains(&SORX_COMPATIBILITY_PATH.to_string())
        );
    }

    #[test]
    fn validation_pack_assets_match_golden_snapshots() {
        let temp = tempdir().expect("tempdir");
        let minimal_out = temp.path().join("minimal.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("tests/golden/tenant_v0_2.sorla.yaml"),
            name: "tenancy".to_string(),
            version: "0.2.0".to_string(),
            out_path: minimal_out.clone(),
        })
        .expect("minimal pack builds");
        assert_gtpack_json_matches_fixture(
            &minimal_out,
            SORX_VALIDATION_MANIFEST_PATH,
            "../../tests/fixtures/validation/minimal/test-manifest.json",
        );

        let landlord_out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: landlord_out.clone(),
        })
        .expect("landlord pack builds");
        assert_gtpack_json_matches_fixture(
            &landlord_out,
            SORX_VALIDATION_MANIFEST_PATH,
            "../../tests/fixtures/validation/landlord-tenant/test-manifest.json",
        );
        assert_gtpack_json_matches_fixture(
            &landlord_out,
            SORX_EXPOSURE_POLICY_PATH,
            "../../tests/fixtures/validation/landlord-tenant/exposure-policy.json",
        );
        assert_gtpack_json_matches_fixture(
            &landlord_out,
            SORX_COMPATIBILITY_PATH,
            "../../tests/fixtures/validation/landlord-tenant/compatibility.json",
        );
    }

    fn assert_gtpack_json_matches_fixture(pack_path: &Path, asset_path: &str, fixture_path: &str) {
        let mut archive =
            ZipArchive::new(fs::File::open(pack_path).expect("open pack")).expect("read pack");
        let actual = zip_text(&mut archive, asset_path).expect("asset should exist");
        let expected = fs::read_to_string(fixture_path).expect("fixture should read");
        assert_eq!(actual.trim_end(), expected.trim_end(), "{asset_path}");
    }

    #[test]
    fn gtpack_doctor_rejects_shared_state_without_migration_metadata() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        let mut archive =
            ZipArchive::new(fs::File::open(&out).expect("open pack")).expect("read pack");
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            if entry.name() == "pack.lock.cbor" {
                continue;
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            if entry.name() == SORX_COMPATIBILITY_PATH {
                let mut compatibility: serde_json::Value =
                    serde_json::from_slice(&bytes).expect("compatibility JSON");
                compatibility["state_compatibility"] =
                    serde_json::Value::String("shared_allowed".to_string());
                compatibility["migration_compatibility"] = serde_json::Value::Array(Vec::new());
                bytes =
                    serde_json::to_vec_pretty(&compatibility).expect("compatibility serializes");
            }
            entries.insert(entry.name().to_string(), bytes);
        }
        drop(archive);
        let lock = pack_lock_for_entries(&entries);
        entries.insert("pack.lock.cbor".to_string(), canonical_cbor(&lock));
        write_zip_entries(&out, entries).expect("rewrite malformed pack");

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("shared state"));
    }

    #[test]
    fn gtpack_doctor_rejects_missing_validation_fixture_reference() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        let mut archive =
            ZipArchive::new(fs::File::open(&out).expect("open pack")).expect("read pack");
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            if entry.name() == "pack.lock.cbor" {
                continue;
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            if entry.name() == SORX_VALIDATION_MANIFEST_PATH {
                let mut manifest: serde_json::Value =
                    serde_json::from_slice(&bytes).expect("validation manifest JSON");
                manifest["suites"][0]["tests"][0]["input_ref"] =
                    serde_json::Value::String("fixtures/missing.json".to_string());
                bytes = serde_json::to_vec_pretty(&manifest).expect("manifest serializes");
            }
            entries.insert(entry.name().to_string(), bytes);
        }
        drop(archive);
        let lock = pack_lock_for_entries(&entries);
        entries.insert("pack.lock.cbor".to_string(), canonical_cbor(&lock));
        write_zip_entries(&out, entries).expect("rewrite malformed pack");

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("assets/sorx/tests/fixtures/missing.json"));
    }

    #[test]
    fn gtpack_doctor_rejects_public_candidate_exposure_default() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        let mut archive =
            ZipArchive::new(fs::File::open(&out).expect("open pack")).expect("read pack");
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            if entry.name() == "pack.lock.cbor" {
                continue;
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            if entry.name() == SORX_EXPOSURE_POLICY_PATH {
                let mut policy: serde_json::Value =
                    serde_json::from_slice(&bytes).expect("exposure policy JSON");
                policy["default_visibility"] =
                    serde_json::Value::String("public_candidate".to_string());
                bytes = serde_json::to_vec_pretty(&policy).expect("policy serializes");
            }
            entries.insert(entry.name().to_string(), bytes);
        }
        drop(archive);
        let lock = pack_lock_for_entries(&entries);
        entries.insert("pack.lock.cbor".to_string(), canonical_cbor(&lock));
        write_zip_entries(&out, entries).expect("rewrite malformed pack");

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("default_visibility"));
    }

    #[test]
    fn gtpack_doctor_rejects_invalid_validation_schema() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        let mut archive =
            ZipArchive::new(fs::File::open(&out).expect("open pack")).expect("read pack");
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            if entry.name() == "pack.lock.cbor" {
                continue;
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            if entry.name() == SORX_VALIDATION_MANIFEST_PATH {
                let mut manifest: serde_json::Value =
                    serde_json::from_slice(&bytes).expect("validation manifest JSON");
                manifest["schema"] = serde_json::Value::String("wrong.schema".to_string());
                bytes = serde_json::to_vec_pretty(&manifest).expect("manifest serializes");
            }
            entries.insert(entry.name().to_string(), bytes);
        }
        drop(archive);
        let lock = pack_lock_for_entries(&entries);
        entries.insert("pack.lock.cbor".to_string(), canonical_cbor(&lock));
        write_zip_entries(&out, entries).expect("rewrite malformed pack");

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains(SORX_VALIDATION_SCHEMA));
    }

    #[test]
    fn gtpack_doctor_rejects_missing_required_asset() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        let mut archive =
            ZipArchive::new(fs::File::open(&out).expect("open pack")).expect("read pack");
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            if entry.name() == "assets/sorla/model.cbor" {
                continue;
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            entries.insert(entry.name().to_string(), bytes);
        }
        drop(archive);
        write_zip_entries(&out, entries).expect("rewrite malformed pack");

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("model.cbor"));
    }

    #[test]
    fn gtpack_build_rejects_missing_input() {
        let temp = tempdir().expect("tempdir");
        let err = build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: temp.path().join("missing.sorla.yaml"),
            name: "missing".to_string(),
            version: "0.1.0".to_string(),
            out_path: temp.path().join("missing.gtpack"),
        })
        .expect_err("missing input should fail");

        assert!(err.contains("failed to read SoRLa input"));
    }

    #[test]
    fn agent_gateway_manifest_handles_empty_packages() {
        let parsed = parse_package(
            r#"
package:
  name: empty-agent-package
  version: 0.2.0
"#,
        )
        .expect("fixture should parse");
        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);

        assert_eq!(manifest.schema, AGENT_GATEWAY_HANDOFF_SCHEMA);
        assert_eq!(manifest.package.name, "empty-agent-package");
        assert_eq!(manifest.package.ir_hash, canonical_hash_hex(&ir));
        assert!(manifest.endpoints.is_empty());
        assert!(manifest.provider_contract.categories.is_empty());
        assert!(manifest.exports.agent_gateway_json);
        assert!(!manifest.exports.openapi_overlay);
        assert!(!manifest.exports.arazzo);
        assert!(!manifest.exports.mcp_tools);
        assert!(!manifest.exports.llms_txt);
        assert!(manifest.notes[0].contains("handoff metadata"));
        assert!(manifest.notes[0].contains("not final runtime gateway behavior"));
    }

    #[test]
    fn executable_contract_exports_relationships_migrations_and_agent_operations() {
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
        let contract: serde_json::Value = serde_json::from_str(&executable_contract_json(&ir))
            .expect("executable contract should be valid JSON");

        assert_eq!(contract["schema"], "greentic.sorla.executable-contract.v1");
        assert_eq!(contract["relationships"][0]["record"], "Tenant");
        assert_eq!(
            contract["relationships"][0]["references"]["record"],
            "Landlord"
        );
        assert_eq!(
            contract["migrations"][0]["idempotence_key"],
            "tenant-v2-fields"
        );
        assert_eq!(
            contract["migrations"][0]["backfills"][0]["default"],
            serde_json::json!("email")
        );
        assert_eq!(
            contract["agent_operations"][0]["emits"]["event"],
            "TenantCreated"
        );
        assert_eq!(
            contract["operation_result_contract"]["schema"],
            "greentic.sorla.operation-result.v1"
        );
        assert_eq!(
            contract["operation_result_contract"]["fields"]["status"][1],
            "validation_error"
        );
    }

    #[test]
    fn agent_gateway_manifest_aggregates_visibility_and_provider_requirements() {
        let parsed = parse_package(
            r#"
package:
  name: website-lead-capture
  version: 0.2.0
provider_requirements:
  - category: crm
    capabilities:
      - contacts.read
  - category: storage
    capabilities:
      - event-log
actions:
  - name: UpsertContact
records:
  - name: Contact
    fields:
      - name: id
        type: string
events:
  - name: ContactCaptured
    record: Contact
agent_endpoints:
  - id: create_customer_contact
    title: Create customer contact
    intent: Capture a customer enquiry.
    inputs:
      - name: email
        type: string
        required: true
      - name: company_name
        type: string
        required: true
    outputs:
      - name: contact_id
        type: string
    side_effects:
      - crm.contact.upsert
    risk: medium
    approval: optional
    provider_requirements:
      - category: crm
        capabilities:
          - contacts.write
          - contacts.read
      - category: api-gateway
        capabilities:
          - route.publish
    backing:
      actions:
        - UpsertContact
      events:
        - ContactCaptured
    agent_visibility:
      openapi: true
      arazzo: false
      mcp: true
      llms_txt: false
    examples:
      - name: lead
        summary: Capture a lead.
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let first = agent_gateway_handoff_manifest(&ir);
        let second = agent_gateway_handoff_manifest(&ir);
        assert_eq!(first, second);
        assert_eq!(first.package.ir_hash, canonical_hash_hex(&ir));
        assert_eq!(first.endpoints.len(), 1);
        assert_eq!(first.endpoints[0].id, "create_customer_contact");
        assert_eq!(first.endpoints[0].risk, "medium");
        assert_eq!(first.endpoints[0].approval, "optional");
        assert_eq!(first.endpoints[0].inputs, ["company_name", "email"]);
        assert_eq!(first.endpoints[0].outputs, ["contact_id"]);
        assert!(first.exports.openapi_overlay);
        assert!(!first.exports.arazzo);
        assert!(first.exports.mcp_tools);
        assert!(!first.exports.llms_txt);

        assert_eq!(
            first.provider_contract.categories[0].category,
            "api-gateway"
        );
        assert_eq!(
            first.provider_contract.categories[0].capabilities,
            ["route.publish"]
        );
        assert_eq!(first.provider_contract.categories[1].category, "crm");
        assert_eq!(
            first.provider_contract.categories[1].capabilities,
            ["contacts.read", "contacts.write"]
        );
        assert_eq!(first.provider_contract.categories[2].category, "storage");
    }

    #[test]
    fn agent_exports_include_only_enabled_targets_and_are_deterministic() {
        let ir = lead_capture_ir();
        let first = export_agent_artifacts(&ir);
        let second = export_agent_artifacts(&ir);

        assert_eq!(first, second);
        assert!(
            first
                .agent_gateway_json
                .contains(AGENT_GATEWAY_HANDOFF_SCHEMA)
        );
        assert!(first.openapi_overlay_yaml.is_some());
        assert!(first.arazzo_yaml.is_none());
        assert!(first.mcp_tools_json.is_some());
        assert!(first.llms_txt.is_some());
    }

    #[test]
    fn mcp_export_includes_required_inputs_and_annotations() {
        let exports = export_agent_artifacts(&lead_capture_ir());
        let mcp: serde_json::Value = serde_json::from_str(
            exports
                .mcp_tools_json
                .as_deref()
                .expect("MCP export should be generated"),
        )
        .expect("MCP export should be valid JSON");

        assert_eq!(mcp["schema"], MCP_TOOLS_HANDOFF_SCHEMA);
        assert_eq!(mcp["tools"][0]["name"], "create_customer_contact");
        assert_eq!(
            mcp["tools"][0]["inputSchema"]["required"][0],
            "company_name"
        );
        assert_eq!(mcp["tools"][0]["inputSchema"]["required"][1], "email");
        assert_eq!(
            mcp["tools"][0]["annotations"]["side_effects"][0],
            "crm.contact.upsert"
        );
        assert_eq!(mcp["tools"][0]["annotations"]["risk"], "medium");
    }

    #[test]
    fn openapi_and_arazzo_exports_include_agent_handoff_metadata() {
        let openapi_exports = export_agent_artifacts(&lead_capture_ir());
        let openapi: serde_yaml::Value = serde_yaml::from_str(
            openapi_exports
                .openapi_overlay_yaml
                .as_deref()
                .expect("OpenAPI overlay should be generated"),
        )
        .expect("OpenAPI overlay should be valid YAML");

        assert_eq!(openapi["schema"], OPENAPI_AGENT_OVERLAY_SCHEMA);
        assert_eq!(
            openapi["operations"][0]["x-greentic-agent"]["endpoint_id"],
            "create_customer_contact"
        );
        assert_eq!(
            openapi["operations"][0]["x-greentic-agent"]["approval"],
            "optional"
        );
        assert_eq!(
            openapi["operations"][0]["x-greentic-agent"]["side_effects"][0],
            "crm.contact.upsert"
        );

        let arazzo_ir = arazzo_visible_ir();
        let arazzo_exports = export_agent_artifacts(&arazzo_ir);
        let arazzo: serde_yaml::Value = serde_yaml::from_str(
            arazzo_exports
                .arazzo_yaml
                .as_deref()
                .expect("Arazzo export should be generated"),
        )
        .expect("Arazzo export should be valid YAML");
        assert_eq!(arazzo["arazzo"], "1.0.1");
        assert_eq!(
            arazzo["workflows"][0]["workflowId"],
            "create_customer_contact"
        );
        assert_eq!(
            arazzo["workflows"][0]["steps"][0]["stepId"],
            "request_create_customer_contact"
        );
    }

    #[test]
    fn llms_txt_export_includes_safety_metadata() {
        let exports = export_agent_artifacts(&lead_capture_ir());
        let llms_txt = exports
            .llms_txt
            .as_deref()
            .expect("llms.txt fragment should be generated");

        assert!(llms_txt.contains("# website-lead-capture agent endpoints"));
        assert!(llms_txt.contains("Intent: Capture a customer enquiry."));
        assert!(llms_txt.contains("Risk: medium"));
        assert!(llms_txt.contains("Approval: optional"));
        assert!(llms_txt.contains("Side effects: crm.contact.upsert"));
        assert!(llms_txt.contains("Required inputs: company_name, email"));
        assert!(llms_txt.contains("Outputs: contact_id"));
    }

    fn lead_capture_ir() -> CanonicalIr {
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
  - name: UpsertContact
events:
  - name: ContactCaptured
    record: Contact
agent_endpoints:
  - id: create_customer_contact
    title: Create customer contact
    intent: Capture a customer enquiry.
    inputs:
      - name: email
        type: string
        required: true
        sensitive: true
      - name: company_name
        type: string
        required: true
      - name: company_size
        type: string
    outputs:
      - name: contact_id
        type: string
    side_effects:
      - crm.contact.upsert
    risk: medium
    approval: optional
    backing:
      actions:
        - UpsertContact
      events:
        - ContactCaptured
    agent_visibility:
      openapi: true
      arazzo: false
      mcp: true
      llms_txt: true
    examples:
      - name: lead
        summary: Capture a lead.
"#,
        )
        .expect("fixture should parse");
        lower_package(&parsed.package)
    }

    fn arazzo_visible_ir() -> CanonicalIr {
        let parsed = parse_package(
            r#"
package:
  name: website-lead-capture
  version: 0.2.0
agent_endpoints:
  - id: create_customer_contact
    title: Create customer contact
    intent: Capture a customer enquiry.
    inputs:
      - name: email
        type: string
        required: true
    outputs:
      - name: contact_id
        type: string
    side_effects:
      - crm.contact.upsert
    risk: medium
    approval: optional
    agent_visibility:
      openapi: false
      arazzo: true
      mcp: false
      llms_txt: false
    examples:
      - name: lead
        summary: Capture a lead.
"#,
        )
        .expect("fixture should parse");
        lower_package(&parsed.package)
    }
}
