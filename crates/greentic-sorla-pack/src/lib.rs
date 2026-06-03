#![cfg_attr(not(feature = "pack-zip"), allow(dead_code, unused_imports))]

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
    AgentEndpointRiskIr, CanonicalIr, EndpointAuthorizationIr, EntityLinkingIr, IrVersion,
    OntologyModelIr, OperationalIndexesIr, ProviderRequirementIr, RecordIr, RetrievalBindingsIr,
    SemanticAliasesIr, ViewIr, agent_tools_json, canonical_cbor, canonical_hash_hex, inspect_ir,
    lower_package,
};
use greentic_sorla_lang::parser::parse_package;
#[cfg(feature = "pack-zip")]
use greentic_types::{
    ExtensionInline, ExtensionRef, PackId, PackKind as GreenticPackKind,
    PackManifest as GreenticPackManifest, PackSignatures, encode_pack_manifest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
#[cfg(feature = "pack-zip")]
use zip::write::SimpleFileOptions;
#[cfg(feature = "pack-zip")]
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const AGENT_GATEWAY_HANDOFF_SCHEMA: &str = "greentic.agent-gateway.handoff.v1";
pub const SORX_AGENT_GATEWAY_SCHEMA: &str = "greentic.sorla.agent-gateway.v1";
pub const OPENAPI_AGENT_OVERLAY_SCHEMA: &str = "greentic.openapi.agent-overlay.v1";
pub const MCP_TOOLS_HANDOFF_SCHEMA: &str = "greentic.mcp.tools.handoff.v1";
pub const SORX_MCP_TOOLS_SCHEMA: &str = "greentic.sorla.mcp-tools.v1";
pub const AGENT_GATEWAY_HANDOFF_FILENAME: &str = "agent-gateway.json";
pub const AGENT_ENDPOINTS_IR_CBOR_FILENAME: &str = "agent-endpoints.ir.cbor";
pub const AGENT_OPENAPI_OVERLAY_FILENAME: &str = "agent-endpoints.openapi.overlay.yaml";
pub const AGENT_ARAZZO_FILENAME: &str = "agent-workflows.arazzo.yaml";
pub const MCP_TOOLS_FILENAME: &str = "mcp-tools.json";
pub const LLMS_TXT_FRAGMENT_FILENAME: &str = "llms.txt.fragment";
pub const EXECUTABLE_CONTRACT_FILENAME: &str = "executable-contract.json";
pub const ONTOLOGY_GRAPH_SCHEMA: &str = "greentic.sorla.ontology.graph.v1";
pub const ONTOLOGY_EXTENSION_ID: &str = "greentic.sorla.ontology.v1";
pub const ONTOLOGY_GRAPH_FILENAME: &str = "ontology.graph.json";
pub const ONTOLOGY_IR_CBOR_FILENAME: &str = "ontology.ir.cbor";
pub const ONTOLOGY_SCHEMA_FILENAME: &str = "ontology.schema.json";
pub const ONTOLOGY_GRAPH_PATH: &str = "assets/sorla/ontology.graph.json";
pub const ONTOLOGY_IR_CBOR_PATH: &str = "assets/sorla/ontology.ir.cbor";
pub const ONTOLOGY_SCHEMA_PATH: &str = "assets/sorla/ontology.schema.json";
pub const RETRIEVAL_BINDINGS_SCHEMA: &str = "greentic.sorla.retrieval-bindings.v1";
pub const RETRIEVAL_BINDINGS_FILENAME: &str = "retrieval-bindings.json";
pub const RETRIEVAL_BINDINGS_IR_CBOR_FILENAME: &str = "retrieval-bindings.ir.cbor";
pub const RETRIEVAL_BINDINGS_PATH: &str = "assets/sorla/retrieval-bindings.json";
pub const RETRIEVAL_BINDINGS_IR_CBOR_PATH: &str = "assets/sorla/retrieval-bindings.ir.cbor";
pub const OPERATIONAL_INDEXES_SCHEMA: &str = "greentic.sorla.operational-indexes.v1";
pub const OPERATIONAL_INDEXES_FILENAME: &str = "operational-indexes.json";
pub const OPERATIONAL_INDEXES_IR_CBOR_FILENAME: &str = "operational-indexes.ir.cbor";
pub const OPERATIONAL_INDEXES_PATH: &str = "assets/sorla/operational-indexes.json";
pub const OPERATIONAL_INDEXES_IR_CBOR_PATH: &str = "assets/sorla/operational-indexes.ir.cbor";
pub const METRICS_SCHEMA: &str = "greentic.sorla.metrics.v1";
pub const METRICS_FILENAME: &str = "metrics.json";
pub const METRICS_PATH: &str = "assets/sorla/metrics.json";
pub const I18N_ASSET_DIR: &str = "assets/sorla/i18n";
pub const DESIGNER_NODE_TYPES_SCHEMA: &str = "greentic.sorla.designer-node-types.v1";
pub const DESIGNER_NODE_TYPES_FILENAME: &str = "designer-node-types.json";
pub const DESIGNER_NODE_TYPES_PATH: &str = "assets/sorla/designer-node-types.json";
pub const AGENT_ENDPOINT_ACTION_CATALOG_SCHEMA: &str =
    "greentic.sorla.agent-endpoint-action-catalog.v1";
pub const AGENT_ENDPOINT_ACTION_CATALOG_FILENAME: &str = "agent-endpoint-action-catalog.json";
pub const AGENT_ENDPOINT_ACTION_CATALOG_PATH: &str =
    "assets/sorla/agent-endpoint-action-catalog.json";
pub const DEFAULT_DESIGNER_COMPONENT_REF: &str =
    "oci://ghcr.io/greenticai/components/component-sorx-business:stable";
pub const DEFAULT_DESIGNER_COMPONENT_OPERATION: &str = "invoke_locked_action";
pub const GREENTIC_STACK_PACK_SCHEMA: &str = "greentic.stack-pack.v1";
pub const GREENTIC_CAPABILITY_SECTION_SCHEMA_VERSION: u32 = 1;
pub const GREENTIC_STACK_PACK_PATH: &str = "assets/greentic/stack-pack.json";
pub const GREENTIC_CAPABILITIES_PATH: &str = "assets/greentic/capabilities.json";
pub const GREENTIC_ROUTES_PATH: &str = "assets/greentic/routes.json";
pub const GREENTIC_SETUP_SCHEMA_PATH: &str = "assets/greentic/setup.schema.json";
pub const GREENTIC_CALL_REQUEST_SCHEMA_PATH: &str = "assets/greentic/call.request.schema.json";
pub const GREENTIC_CALL_RESPONSE_SCHEMA_PATH: &str = "assets/greentic/call.response.schema.json";
pub const GREENTIC_ARTIFACTS_PATH: &str = "assets/greentic/artifacts.json";
pub const GREENTIC_ADMIN_SURFACES_PATH: &str = "assets/greentic/admin-surfaces.json";
pub const GREENTIC_SECRET_REQUIREMENTS_PATH: &str = "assets/secret-requirements.json";
pub const CAP_STACK_APPLICATION_V1: &str = "cap://greentic/stack/application/v1";
pub const CAP_RUNTIME_HOST_V1: &str = "cap://greentic/runtime/host/v1";
pub const CAP_SECRETS_V1: &str = "cap://greentic/secrets/v1";
pub const CAP_TELEMETRY_V1: &str = "cap://greentic/telemetry/v1";
pub const CAP_EXTENSION_CONTROL_V1: &str = "cap://greentic/extension/control/v1";
pub const CAP_EXTENSION_OBSERVER_V1: &str = "cap://greentic/extension/observer/v1";
pub const CAP_EXTENSION_ADMIN_V1: &str = "cap://greentic/extension/admin/v1";
pub const CONTRACT_STACK_INVOKE_V1: &str = "greentic.stack.invoke.v1";
pub const CONTRACT_STACK_ROUTES_V1: &str = "greentic.stack.routes.v1";
pub const CONTRACT_RUNTIME_INVOKE_V1: &str = "greentic.runtime.invoke.v1";
pub const CONTRACT_RUNTIME_TRAFFIC_V1: &str = "greentic.runtime.traffic.v1";
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
pub const SORX_VALIDATION_SUITE_SCHEMA: &str = "greentic.sorx.validation-suite.v1";
pub const SORX_VALIDATION_SUITE_PATH: &str = "assets/sorx/validation-suite.json";
const SORX_VALIDATION_SUITE_ASSET: &str = "validation-suite.json";
pub const SORX_VALIDATION_SUITE_CBOR_PATH: &str = "assets/sorx/validation-suite.cbor";
const SORX_VALIDATION_SUITE_CBOR_ASSET: &str = "validation-suite.cbor";

const STABLE_PACK_TIMESTAMP: &str = "1970-01-01T00:00:00Z";

fn is_false(value: &bool) -> bool {
    !*value
}

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
    pub designer_node_types_json: String,
    pub agent_endpoint_action_catalog_json: String,
    pub metrics_json: Option<String>,
    pub ontology_artifacts: Option<OntologyArtifactSet>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub record_hierarchy: Vec<AgentGatewayRecordHierarchyRef>,
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
pub struct AgentGatewayRecordHierarchyRef {
    pub record: String,
    #[serde(default)]
    pub main: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<AgentGatewayRecordParentRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayRecordParentRef {
    pub record: String,
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGatewayEndpointRef {
    pub id: String,
    pub endpoint_id: String,
    pub operation_id: String,
    pub operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<serde_json::Value>,
    pub method: String,
    pub path: String,
    pub entity: String,
    pub collection: String,
    pub provider_binding: String,
    pub title: String,
    pub intent: String,
    pub risk: String,
    pub approval: String,
    #[serde(default, skip_serializing_if = "EndpointAuthorizationIr::is_empty")]
    pub authorization: EndpointAuthorizationIr,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
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
pub struct AgentEndpointContractWarning {
    pub endpoint_id: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyArtifactSet {
    pub ir_cbor: Vec<u8>,
    pub graph_json: String,
    pub schema_json: String,
    pub ir_hash: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ontology: Option<SorlaGtpackOntologyInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_bindings: Option<SorlaGtpackRetrievalInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operational_indexes: Option<SorlaGtpackOperationalIndexesInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<SorlaGtpackMetricsInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designer_node_types: Option<SorlaGtpackDesignerNodeTypesInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_endpoint_action_catalog: Option<SorlaGtpackAgentEndpointActionCatalogInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_pack: Option<SorlaGtpackStackPackInspection>,
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
pub struct SorlaGtpackOntologyInspection {
    pub schema: String,
    pub graph_schema: String,
    pub concept_count: usize,
    pub relationship_count: usize,
    pub constraint_count: usize,
    pub ir_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackRetrievalInspection {
    pub schema: String,
    pub provider_count: usize,
    pub scope_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackOperationalIndexesInspection {
    pub schema: String,
    pub index_count: usize,
    pub query_requirement_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackMetricsInspection {
    pub schema: String,
    pub count: usize,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactDocument {
    pub schema: String,
    pub package: MetricsArtifactPackage,
    pub metrics: Vec<MetricsArtifactMetric>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactPackage {
    pub name: String,
    pub version: String,
    pub ir_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactMetric {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<MetricsArtifactSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measure: Option<MetricsArtifactMeasure>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dimensions: Vec<MetricsArtifactDimension>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<greentic_sorla_ir::MetricFilterIr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<MetricsArtifactTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula: Option<MetricsArtifactFormula>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactSource {
    pub entity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactMeasure {
    pub aggregate: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactDimension {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactTime {
    pub field: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MetricsArtifactFormula {
    pub expression: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackDesignerNodeTypesInspection {
    pub schema: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackAgentEndpointActionCatalogInspection {
    pub schema: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SorlaGtpackStackPackInspection {
    pub schema: String,
    pub stack_id: String,
    pub stack_kind: String,
    pub stack_version: String,
    pub offer_count: usize,
    pub requirement_count: usize,
    pub route_count: usize,
    pub required_secret_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticStackPackDocument {
    schema: String,
    stack: GreenticStackIdentity,
    offers: Vec<GreenticCapabilityOffer>,
    requires: Vec<GreenticCapabilityRequirement>,
    routes: Vec<GreenticStackRoute>,
    setup: GreenticStackSetup,
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticStackIdentity {
    id: String,
    kind: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticCapabilityOffer {
    id: String,
    capability: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticCapabilityRequirement {
    id: String,
    capability: String,
    #[serde(default, skip_serializing_if = "is_false")]
    optional: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticStackRoute {
    id: String,
    method: String,
    path: String,
    contract: String,
    request_schema_ref: String,
    response_schema_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticStackSetup {
    schema_ref: String,
    secret_requirements_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticPackCapabilitySection {
    schema_version: u32,
    declaration: GreenticCapabilityDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GreenticCapabilityDeclaration {
    offers: Vec<GreenticCapabilityOffer>,
    requires: Vec<GreenticCapabilityRequirement>,
    consumes: Vec<serde_json::Value>,
    profiles: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeTypesDocument {
    pub schema: String,
    pub package: DesignerNodeTypesPackageRef,
    #[serde(rename = "nodeTypes")]
    pub node_types: Vec<DesignerNodeType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeTypesPackageRef {
    pub name: String,
    pub version: String,
    pub ir_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeType {
    pub id: String,
    pub version: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub binding: DesignerNodeBinding,
    #[serde(rename = "configSchema")]
    pub config_schema: serde_json::Value,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(rename = "outputSchema")]
    pub output_schema: serde_json::Value,
    pub ui: DesignerNodeUi,
    #[serde(rename = "defaultRouting")]
    pub default_routing: DesignerNodeRouting,
    pub metadata: DesignerNodeMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeBinding {
    pub kind: String,
    pub component: DesignerComponentRef,
    pub operation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerComponentRef {
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeUi {
    pub fields: Vec<DesignerNodeUiField>,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeUiField {
    pub name: String,
    pub label: String,
    pub widget: String,
    #[serde(rename = "displayOrder")]
    pub display_order: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeRouting {
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeMetadata {
    pub endpoint: DesignerEndpointRef,
    pub risk: String,
    pub approval: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub intent: String,
    pub side_effects: Vec<String>,
    pub provider_requirements: Vec<ProviderRequirementIr>,
    pub backing: DesignerNodeBacking,
    pub exports: AgentGatewayEndpointExports,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerEndpointRef {
    pub id: String,
    pub version: String,
    pub package: String,
    pub contract_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerNodeBacking {
    pub actions: Vec<String>,
    pub events: Vec<String>,
    pub flows: Vec<String>,
    pub policies: Vec<String>,
    pub approvals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerNodeTypeGenerationOptions {
    pub component_ref: String,
    pub operation: String,
}

impl Default for DesignerNodeTypeGenerationOptions {
    fn default() -> Self {
        Self {
            component_ref: DEFAULT_DESIGNER_COMPONENT_REF.to_string(),
            operation: DEFAULT_DESIGNER_COMPONENT_OPERATION.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointActionCatalogDocument {
    pub schema: String,
    pub package: AgentEndpointActionCatalogPackageRef,
    pub actions: Vec<AgentEndpointActionCatalogAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointActionCatalogPackageRef {
    pub name: String,
    pub version: String,
    pub ir_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointActionCatalogAction {
    pub id: String,
    pub version: String,
    pub label: String,
    pub description: String,
    pub intent: String,
    pub endpoint_ref: DesignerEndpointRef,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub risk: String,
    pub approval: String,
    pub side_effects: Vec<String>,
    pub provider_requirements: Vec<ProviderRequirementIr>,
    pub backing: DesignerNodeBacking,
    pub design: AgentEndpointActionCatalogDesign,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpointActionCatalogDesign {
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
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
    let ontology_artifacts = ontology_artifacts(&ir)?;
    if ontology_artifacts.is_some() {
        package_manifest
            .artifact_references
            .push(ONTOLOGY_GRAPH_FILENAME.to_string());
        package_manifest
            .artifact_references
            .push(ONTOLOGY_IR_CBOR_FILENAME.to_string());
        package_manifest
            .artifact_references
            .push(ONTOLOGY_SCHEMA_FILENAME.to_string());
    }
    if ir.retrieval_bindings.is_some() {
        package_manifest
            .artifact_references
            .push(RETRIEVAL_BINDINGS_FILENAME.to_string());
        package_manifest
            .artifact_references
            .push(RETRIEVAL_BINDINGS_IR_CBOR_FILENAME.to_string());
    }
    if ir.operational_indexes.is_some() {
        package_manifest
            .artifact_references
            .push(OPERATIONAL_INDEXES_FILENAME.to_string());
        package_manifest
            .artifact_references
            .push(OPERATIONAL_INDEXES_IR_CBOR_FILENAME.to_string());
    }
    if !ir.metrics.is_empty() {
        package_manifest
            .artifact_references
            .push(METRICS_FILENAME.to_string());
    }
    let designer_node_types =
        generate_designer_node_types_from_ir(&ir, &DesignerNodeTypeGenerationOptions::default())?;
    let designer_node_types_json =
        serde_json::to_string_pretty(&designer_node_types).map_err(|err| err.to_string())?;
    let action_catalog = generate_agent_endpoint_action_catalog_from_ir(&ir)?;
    let agent_endpoint_action_catalog_json =
        serde_json::to_string_pretty(&action_catalog).map_err(|err| err.to_string())?;
    if !designer_node_types.node_types.is_empty() {
        package_manifest
            .artifact_references
            .push(DESIGNER_NODE_TYPES_FILENAME.to_string());
        package_manifest
            .artifact_references
            .push(AGENT_ENDPOINT_ACTION_CATALOG_FILENAME.to_string());
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
    if let Some(ontology_artifacts) = &ontology_artifacts {
        cbor_artifacts.insert(
            ONTOLOGY_IR_CBOR_FILENAME.to_string(),
            ontology_artifacts.ir_cbor.clone(),
        );
    }
    if let Some(retrieval) = &ir.retrieval_bindings {
        cbor_artifacts.insert(
            RETRIEVAL_BINDINGS_IR_CBOR_FILENAME.to_string(),
            canonical_cbor(retrieval),
        );
    }
    if let Some(indexes) = &ir.operational_indexes {
        cbor_artifacts.insert(
            OPERATIONAL_INDEXES_IR_CBOR_FILENAME.to_string(),
            canonical_cbor(indexes),
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
    let metrics_json = if ir.metrics.is_empty() {
        None
    } else {
        Some(metrics_artifact_json(&ir, &canonical_hash)?)
    };

    Ok(ArtifactSet {
        ir,
        package_manifest,
        cbor_artifacts,
        inspect_json,
        agent_tools_json: agent_tools,
        agent_exports,
        executable_contract_json: executable_contract,
        designer_node_types_json,
        agent_endpoint_action_catalog_json,
        metrics_json,
        ontology_artifacts,
        canonical_hash,
    })
}

fn metrics_artifact_json(ir: &CanonicalIr, ir_hash: &str) -> Result<String, String> {
    let document = MetricsArtifactDocument {
        schema: METRICS_SCHEMA.to_string(),
        package: MetricsArtifactPackage {
            name: ir.package.name.clone(),
            version: ir.package.version.clone(),
            ir_hash: ir_hash.to_string(),
        },
        metrics: metrics_artifact_metrics(ir),
    };
    serde_json::to_string_pretty(&document).map_err(|err| err.to_string())
}

fn metrics_artifact_metrics(ir: &CanonicalIr) -> Vec<MetricsArtifactMetric> {
    ir.metrics
        .iter()
        .map(|metric| MetricsArtifactMetric {
            name: metric.name.clone(),
            i18n_key: metric.i18n_key.clone(),
            label: metric.label.clone(),
            description: metric.description.clone(),
            source: metric.source.as_ref().map(|source| {
                let entity = source.name.clone();
                MetricsArtifactSource {
                    collection: (source.kind == "record")
                        .then(|| pluralize_snake(&snake_case_identifier(&entity))),
                    entity,
                }
            }),
            measure: metric
                .measure
                .as_ref()
                .map(|measure| MetricsArtifactMeasure {
                    aggregate: sorx_metric_aggregate(&measure.aggregate).to_string(),
                    field: measure.field.clone(),
                }),
            dimensions: metric
                .dimensions
                .iter()
                .map(|dimension| MetricsArtifactDimension {
                    name: dimension.clone(),
                    field: Some(dimension.clone()),
                    sensitive: false,
                })
                .collect(),
            filters: metric.filters.clone(),
            time: metric.time.as_ref().map(|time| MetricsArtifactTime {
                field: time.field.clone(),
                grains: vec![time.grain.clone()],
            }),
            formula: metric
                .formula
                .as_ref()
                .map(|formula| MetricsArtifactFormula {
                    expression: formula.clone(),
                    dependencies: metric.depends_on.clone(),
                }),
        })
        .collect()
}

fn sorx_metric_aggregate(aggregate: &str) -> &str {
    match aggregate {
        "average" => "avg",
        "count_distinct" => "distinct_count",
        other => other,
    }
}

pub fn generate_designer_node_types_from_ir(
    ir: &CanonicalIr,
    options: &DesignerNodeTypeGenerationOptions,
) -> Result<DesignerNodeTypesDocument, String> {
    if options.component_ref.trim().is_empty() {
        return Err("designer node type component_ref must not be empty".to_string());
    }
    if options.operation.trim().is_empty() {
        return Err("designer node type operation must not be empty".to_string());
    }
    let ir_hash = canonical_hash_hex(ir);
    let node_types = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| designer_node_type_for_endpoint(ir, endpoint, options, &ir_hash))
        .collect::<Vec<_>>();
    Ok(DesignerNodeTypesDocument {
        schema: DESIGNER_NODE_TYPES_SCHEMA.to_string(),
        package: DesignerNodeTypesPackageRef {
            name: ir.package.name.clone(),
            version: ir.package.version.clone(),
            ir_hash,
        },
        node_types,
    })
}

pub fn generate_agent_endpoint_action_catalog_from_ir(
    ir: &CanonicalIr,
) -> Result<AgentEndpointActionCatalogDocument, String> {
    let ir_hash = canonical_hash_hex(ir);
    let contract_hash = format!("sha256:{ir_hash}");
    let actions = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| {
            let description = endpoint
                .description
                .clone()
                .unwrap_or_else(|| endpoint.intent.clone());
            AgentEndpointActionCatalogAction {
                id: endpoint.id.clone(),
                version: ir.package.version.clone(),
                label: endpoint.title.clone(),
                description,
                intent: endpoint.intent.clone(),
                endpoint_ref: DesignerEndpointRef {
                    id: endpoint.id.clone(),
                    version: ir.package.version.clone(),
                    package: ir.package.name.clone(),
                    contract_hash: contract_hash.clone(),
                },
                input_schema: object_schema_value(&endpoint.inputs),
                output_schema: output_object_schema_value(&endpoint.outputs),
                risk: agent_endpoint_risk_label(&endpoint.risk).to_string(),
                approval: agent_endpoint_approval_label(&endpoint.approval).to_string(),
                side_effects: endpoint.side_effects.clone(),
                provider_requirements: endpoint.provider_requirements.clone(),
                backing: DesignerNodeBacking {
                    actions: endpoint.backing.actions.clone(),
                    events: endpoint.backing.events.clone(),
                    flows: endpoint.backing.flows.clone(),
                    policies: endpoint.backing.policies.clone(),
                    approvals: endpoint.backing.approvals.clone(),
                },
                design: AgentEndpointActionCatalogDesign {
                    aliases: design_aliases_for_endpoint(endpoint),
                    tags: vec!["sorla".to_string(), "agent-endpoint".to_string()],
                },
            }
        })
        .collect::<Vec<_>>();

    Ok(AgentEndpointActionCatalogDocument {
        schema: AGENT_ENDPOINT_ACTION_CATALOG_SCHEMA.to_string(),
        package: AgentEndpointActionCatalogPackageRef {
            name: ir.package.name.clone(),
            version: ir.package.version.clone(),
            ir_hash,
        },
        actions,
    })
}

fn designer_node_type_for_endpoint(
    ir: &CanonicalIr,
    endpoint: &AgentEndpointIr,
    options: &DesignerNodeTypeGenerationOptions,
    ir_hash: &str,
) -> DesignerNodeType {
    let endpoint_ref = serde_json::json!({
        "id": endpoint.id,
        "version": ir.package.version,
        "package": ir.package.name,
        "contract_hash": format!("sha256:{ir_hash}")
    });
    DesignerNodeType {
        id: format!("sorla.agent-endpoint.{}", endpoint.id),
        version: ir.package.version.clone(),
        label: endpoint.title.clone(),
        description: endpoint
            .description
            .clone()
            .unwrap_or_else(|| endpoint.intent.clone()),
        category: "System of Record".to_string(),
        binding: DesignerNodeBinding {
            kind: "component".to_string(),
            component: DesignerComponentRef {
                reference: options.component_ref.clone(),
            },
            operation: options.operation.clone(),
        },
        config_schema: serde_json::json!({
            "type": "object",
            "required": ["endpoint_ref"],
            "properties": {
                "endpoint_ref": {
                    "const": endpoint_ref
                }
            },
            "additionalProperties": false
        }),
        input_schema: object_schema_value(&endpoint.inputs),
        output_schema: output_object_schema_value(&endpoint.outputs),
        ui: DesignerNodeUi {
            fields: endpoint
                .inputs
                .iter()
                .enumerate()
                .map(|(index, input)| DesignerNodeUiField {
                    name: input.name.clone(),
                    label: label_from_identifier(&input.name),
                    widget: widget_for_input(input),
                    display_order: ((index + 1) * 10) as u16,
                })
                .collect(),
            tags: vec!["sorla".to_string(), "agent-endpoint".to_string()],
            aliases: design_aliases_for_endpoint(endpoint),
        },
        default_routing: DesignerNodeRouting {
            kind: "out".to_string(),
        },
        metadata: DesignerNodeMetadata {
            endpoint: DesignerEndpointRef {
                id: endpoint.id.clone(),
                version: ir.package.version.clone(),
                package: ir.package.name.clone(),
                contract_hash: format!("sha256:{ir_hash}"),
            },
            risk: agent_endpoint_risk_label(&endpoint.risk).to_string(),
            approval: agent_endpoint_approval_label(&endpoint.approval).to_string(),
            intent: endpoint.intent.clone(),
            side_effects: endpoint.side_effects.clone(),
            provider_requirements: endpoint.provider_requirements.clone(),
            backing: DesignerNodeBacking {
                actions: endpoint.backing.actions.clone(),
                events: endpoint.backing.events.clone(),
                flows: endpoint.backing.flows.clone(),
                policies: endpoint.backing.policies.clone(),
                approvals: endpoint.backing.approvals.clone(),
            },
            exports: AgentGatewayEndpointExports {
                openapi: endpoint.agent_visibility.openapi,
                arazzo: endpoint.agent_visibility.arazzo,
                mcp: endpoint.agent_visibility.mcp,
                llms_txt: endpoint.agent_visibility.llms_txt,
            },
        },
    }
}

fn output_object_schema_value(outputs: &[AgentEndpointOutputIr]) -> serde_json::Value {
    let properties = outputs
        .iter()
        .map(|output| {
            let mut property = serde_json::Map::new();
            property.insert(
                "type".to_string(),
                serde_json::Value::String(output.type_name.clone()),
            );
            if let Some(description) = &output.description {
                property.insert(
                    "description".to_string(),
                    serde_json::Value::String(description.clone()),
                );
            }
            (output.name.clone(), serde_json::Value::Object(property))
        })
        .collect::<serde_json::Map<_, _>>();

    serde_json::json!({
        "type": "object",
        "properties": properties
    })
}

fn label_from_identifier(identifier: &str) -> String {
    identifier
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn widget_for_input(input: &AgentEndpointInputIr) -> String {
    if !input.enum_values.is_empty() {
        return "select".to_string();
    }
    match input.type_name.as_str() {
        "bool" | "boolean" => "checkbox",
        "number" | "float" | "double" | "decimal" | "integer" | "int" => "number",
        _ => "text",
    }
    .to_string()
}

fn design_aliases_for_endpoint(endpoint: &AgentEndpointIr) -> Vec<String> {
    let mut aliases = BTreeSet::new();
    aliases.insert(endpoint.id.replace(['_', '-'], " "));
    aliases.insert(endpoint.title.to_ascii_lowercase());
    aliases.into_iter().collect()
}

fn ontology_artifacts(ir: &CanonicalIr) -> Result<Option<OntologyArtifactSet>, String> {
    let Some(ontology) = &ir.ontology else {
        return Ok(None);
    };

    let ir_cbor = canonical_cbor(ontology);
    let ir_hash = sha256_hex(&ir_cbor);
    let graph = ontology_graph_json(ir, ontology, &ir_hash);
    let schema = ontology_schema_json();

    Ok(Some(OntologyArtifactSet {
        ir_cbor,
        graph_json: serde_json::to_string_pretty(&graph).map_err(|err| err.to_string())?,
        schema_json: serde_json::to_string_pretty(&schema).map_err(|err| err.to_string())?,
        ir_hash,
    }))
}

fn ontology_graph_json(
    ir: &CanonicalIr,
    ontology: &OntologyModelIr,
    ir_hash: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema": ONTOLOGY_GRAPH_SCHEMA,
        "package": {
            "name": ir.package.name.clone(),
            "version": ir.package.version.clone(),
        },
        "ir_hash": ir_hash,
        "concepts": ontology.concepts.clone(),
        "relationships": ontology.relationships.clone(),
        "constraints": ontology.constraints.clone(),
        "semantic_aliases": ontology.semantic_aliases.clone(),
        "entity_linking": ontology.entity_linking.clone(),
        "indexes": {
            "concepts_by_id": true,
            "relationships_by_id": true
        }
    })
}

pub fn ontology_schema_json() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": ONTOLOGY_EXTENSION_ID,
        "title": "SoRLa ontology v1",
        "type": "object",
        "required": ["schema", "concepts", "relationships", "semantic_aliases", "entity_linking"],
        "properties": {
            "schema": { "const": ONTOLOGY_EXTENSION_ID },
            "concepts": { "type": "array" },
            "relationships": { "type": "array" },
            "constraints": { "type": "array" },
            "semantic_aliases": { "type": "object" },
            "entity_linking": { "type": "object" }
        },
        "additionalProperties": false
    })
}

pub fn retrieval_bindings_schema_json() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": RETRIEVAL_BINDINGS_SCHEMA,
        "title": "SoRLa retrieval bindings v1",
        "type": "object",
        "additionalProperties": false,
        "required": ["schema", "providers", "scopes"],
        "properties": {
            "schema": { "const": RETRIEVAL_BINDINGS_SCHEMA },
            "providers": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["id", "category", "required_capabilities"],
                    "properties": {
                        "id": { "type": "string", "minLength": 1 },
                        "category": { "type": "string", "minLength": 1 },
                        "required_capabilities": {
                            "type": "array",
                            "items": { "type": "string", "minLength": 1 }
                        }
                    }
                }
            },
            "scopes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["id", "applies_to", "provider"],
                    "properties": {
                        "id": { "type": "string", "minLength": 1 },
                        "applies_to": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "concept": { "type": "string", "minLength": 1 },
                                "relationship": { "type": "string", "minLength": 1 }
                            }
                        },
                        "provider": { "type": "string", "minLength": 1 },
                        "filters": { "type": "object" },
                        "permission": {
                            "type": "string",
                            "enum": ["inherit", "public-metadata-only", "requires-policy"]
                        }
                    }
                }
            }
        }
    })
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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
    let validation_suite = sorx_runtime_validation_suite_json(&artifacts.ir);
    sorx_assets.insert(
        SORX_VALIDATION_SUITE_ASSET.to_string(),
        serde_json::to_vec_pretty(&validation_suite).map_err(|err| err.to_string())?,
    );
    sorx_assets.insert(
        SORX_VALIDATION_SUITE_CBOR_ASSET.to_string(),
        canonical_cbor(&validation_suite),
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
    let i18n_assets = discover_adjacent_i18n_assets(&options.input_path)?;
    let i18n_asset_paths = i18n_assets
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let stack_pack = greentic_stack_pack_document(options, &artifacts);
    validate_greentic_stack_pack_document(&stack_pack)?;
    let capabilities = greentic_capability_section(&stack_pack);
    validate_greentic_capability_section(&capabilities)?;
    let routes = serde_json::json!({
        "schema": "greentic.stack.routes.v1",
        "routes": stack_pack.routes.clone(),
    });
    let setup_schema = greentic_setup_schema(&artifacts.ir);
    let call_request_schema = greentic_call_request_schema();
    let call_response_schema = greentic_call_response_schema();
    let greentic_artifacts = greentic_artifacts_document(&artifacts);
    let greentic_admin_surfaces = greentic_admin_surfaces_document(&artifacts);
    let secret_requirements = greentic_secret_requirements();
    let extension = sorx_runtime_extension_value(&artifacts, &sorx_assets, &i18n_asset_paths);

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
    if let Some(ontology) = &artifacts.ontology_artifacts {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            ONTOLOGY_GRAPH_PATH.to_string(),
            ontology.graph_json.as_bytes().to_vec(),
        );
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            ONTOLOGY_SCHEMA_PATH.to_string(),
            ontology.schema_json.as_bytes().to_vec(),
        );
    }
    if let Some(retrieval) = &artifacts.ir.retrieval_bindings {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            RETRIEVAL_BINDINGS_PATH.to_string(),
            serde_json::to_vec_pretty(retrieval).map_err(|err| err.to_string())?,
        );
    }
    if let Some(indexes) = &artifacts.ir.operational_indexes {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            OPERATIONAL_INDEXES_PATH.to_string(),
            serde_json::to_vec_pretty(indexes).map_err(|err| err.to_string())?,
        );
    }
    if let Some(metrics_json) = &artifacts.metrics_json {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            METRICS_PATH.to_string(),
            metrics_json.as_bytes().to_vec(),
        );
    }
    for (path, bytes) in i18n_assets {
        insert_pack_asset(&mut entries, &mut asset_paths, path, bytes);
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
    if !artifacts.ir.agent_endpoints.is_empty() {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            DESIGNER_NODE_TYPES_PATH.to_string(),
            artifacts.designer_node_types_json.as_bytes().to_vec(),
        );
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            AGENT_ENDPOINT_ACTION_CATALOG_PATH.to_string(),
            artifacts
                .agent_endpoint_action_catalog_json
                .as_bytes()
                .to_vec(),
        );
    }

    for (name, bytes) in &sorx_assets {
        insert_pack_asset(
            &mut entries,
            &mut asset_paths,
            format!("assets/sorx/{name}"),
            bytes.clone(),
        );
    }

    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_STACK_PACK_PATH.to_string(),
        serde_json::to_vec_pretty(&stack_pack).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_CAPABILITIES_PATH.to_string(),
        serde_json::to_vec_pretty(&capabilities).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_ROUTES_PATH.to_string(),
        serde_json::to_vec_pretty(&routes).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_SETUP_SCHEMA_PATH.to_string(),
        serde_json::to_vec_pretty(&setup_schema).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_CALL_REQUEST_SCHEMA_PATH.to_string(),
        serde_json::to_vec_pretty(&call_request_schema).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_CALL_RESPONSE_SCHEMA_PATH.to_string(),
        serde_json::to_vec_pretty(&call_response_schema).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_ARTIFACTS_PATH.to_string(),
        serde_json::to_vec_pretty(&greentic_artifacts).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_ADMIN_SURFACES_PATH.to_string(),
        serde_json::to_vec_pretty(&greentic_admin_surfaces).map_err(|err| err.to_string())?,
    );
    insert_pack_asset(
        &mut entries,
        &mut asset_paths,
        GREENTIC_SECRET_REQUIREMENTS_PATH.to_string(),
        serde_json::to_vec_pretty(&secret_requirements).map_err(|err| err.to_string())?,
    );

    asset_paths.sort();
    asset_paths.dedup();

    let mut extension = extension;
    extension["greentic"] = serde_json::json!({
        "stack_pack": GREENTIC_STACK_PACK_PATH,
        "capabilities": GREENTIC_CAPABILITIES_PATH,
        "routes": GREENTIC_ROUTES_PATH,
        "setup_schema": GREENTIC_SETUP_SCHEMA_PATH,
        "call_request_schema": GREENTIC_CALL_REQUEST_SCHEMA_PATH,
        "call_response_schema": GREENTIC_CALL_RESPONSE_SCHEMA_PATH,
        "artifacts": GREENTIC_ARTIFACTS_PATH,
        "admin_surfaces": GREENTIC_ADMIN_SURFACES_PATH,
        "secret_requirements": GREENTIC_SECRET_REQUIREMENTS_PATH,
    });

    let manifest = SorlaPackManifest {
        schema: "greentic.gtpack.manifest.sorla.v1".to_string(),
        pack: SorlaPackIdentity {
            name: options.name.clone(),
            version: options.version.clone(),
            kind: "application".to_string(),
        },
        created_at_utc: STABLE_PACK_TIMESTAMP.to_string(),
        extension,
        assets: sorx_visible_manifest_assets(&asset_paths),
    };
    let pack_cbor = canonical_cbor(&manifest);
    let greentic_manifest_cbor = greentic_pack_manifest_cbor(options, &manifest)?;
    entries.insert("pack.cbor".to_string(), pack_cbor.clone());
    entries.insert("manifest.cbor".to_string(), greentic_manifest_cbor);
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
    verify_with_greentic_pack_lib(&options.out_path)?;

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

#[cfg(feature = "pack-zip")]
fn discover_adjacent_i18n_assets(input_path: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let Some(parent) = input_path.parent() else {
        return Ok(Vec::new());
    };
    let i18n_dir = parent.join("i18n");
    if !i18n_dir.exists() {
        return Ok(Vec::new());
    }
    if !i18n_dir.is_dir() {
        return Err(format!(
            "i18n path next to SoRLa input must be a directory: {}",
            i18n_dir.display()
        ));
    }

    let mut assets = Vec::new();
    for entry in fs::read_dir(&i18n_dir).map_err(|err| {
        format!(
            "failed to read i18n directory {}: {err}",
            i18n_dir.display()
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "failed to read i18n directory {}: {err}",
                i18n_dir.display()
            )
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("i18n filename must be valid UTF-8: {}", path.display()))?;
        let bytes = fs::read(&path)
            .map_err(|err| format!("failed to read i18n catalog {}: {err}", path.display()))?;
        serde_json::from_slice::<serde_json::Value>(&bytes)
            .map_err(|err| format!("failed to parse i18n catalog {}: {err}", path.display()))?;
        assets.push((format!("{I18N_ASSET_DIR}/{filename}"), bytes));
    }
    assets.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(assets)
}

#[cfg(feature = "pack-zip")]
fn greentic_stack_pack_document(
    options: &SorlaGtpackOptions,
    artifacts: &ArtifactSet,
) -> GreenticStackPackDocument {
    let route_ids = vec!["main".to_string()];
    GreenticStackPackDocument {
        schema: GREENTIC_STACK_PACK_SCHEMA.to_string(),
        stack: GreenticStackIdentity {
            id: options.name.clone(),
            kind: "application-stack".to_string(),
            version: options.version.clone(),
        },
        offers: vec![GreenticCapabilityOffer {
            id: "offer.stack.application".to_string(),
            capability: CAP_STACK_APPLICATION_V1.to_string(),
            metadata: serde_json::json!({
                "contracts": [
                    CONTRACT_STACK_INVOKE_V1,
                    CONTRACT_STACK_ROUTES_V1,
                ],
                "routes": route_ids,
            }),
        }],
        requires: vec![
            GreenticCapabilityRequirement {
                id: "require.runtime.host".to_string(),
                capability: CAP_RUNTIME_HOST_V1.to_string(),
                optional: false,
                metadata: serde_json::json!({
                    "contracts": [
                        CONTRACT_RUNTIME_INVOKE_V1,
                        CONTRACT_RUNTIME_TRAFFIC_V1,
                    ],
                }),
            },
            GreenticCapabilityRequirement {
                id: "require.secrets".to_string(),
                capability: CAP_SECRETS_V1.to_string(),
                optional: false,
                metadata: serde_json::Value::Null,
            },
            GreenticCapabilityRequirement {
                id: "require.telemetry".to_string(),
                capability: CAP_TELEMETRY_V1.to_string(),
                optional: true,
                metadata: serde_json::Value::Null,
            },
            GreenticCapabilityRequirement {
                id: "require.extension.control".to_string(),
                capability: CAP_EXTENSION_CONTROL_V1.to_string(),
                optional: true,
                metadata: serde_json::Value::Null,
            },
            GreenticCapabilityRequirement {
                id: "require.extension.observer".to_string(),
                capability: CAP_EXTENSION_OBSERVER_V1.to_string(),
                optional: true,
                metadata: serde_json::Value::Null,
            },
            GreenticCapabilityRequirement {
                id: "require.extension.admin".to_string(),
                capability: CAP_EXTENSION_ADMIN_V1.to_string(),
                optional: true,
                metadata: serde_json::Value::Null,
            },
        ],
        routes: vec![GreenticStackRoute {
            id: "main".to_string(),
            method: "POST".to_string(),
            path: "/invoke".to_string(),
            contract: CONTRACT_STACK_INVOKE_V1.to_string(),
            request_schema_ref: GREENTIC_CALL_REQUEST_SCHEMA_PATH.to_string(),
            response_schema_ref: GREENTIC_CALL_RESPONSE_SCHEMA_PATH.to_string(),
        }],
        setup: GreenticStackSetup {
            schema_ref: GREENTIC_SETUP_SCHEMA_PATH.to_string(),
            secret_requirements_ref: GREENTIC_SECRET_REQUIREMENTS_PATH.to_string(),
        },
        metadata: serde_json::json!({
            "sorla_package": {
                "name": artifacts.ir.package.name.clone(),
                "version": artifacts.ir.package.version.clone(),
                "ir_hash": artifacts.canonical_hash.clone(),
            },
            "sorla_assets": {
                "model": "assets/sorla/model.cbor",
                "agent_gateway": format!("assets/sorla/{AGENT_GATEWAY_HANDOFF_FILENAME}"),
                "executable_contract": format!("assets/sorla/{EXECUTABLE_CONTRACT_FILENAME}"),
                "metrics": if artifacts.metrics_json.is_some() {
                    serde_json::json!(METRICS_PATH)
                } else {
                    serde_json::Value::Null
                },
            },
            "sorx_compatibility": {
                "runtime_extension": SORX_RUNTIME_EXTENSION_ID,
                "start_schema": format!("assets/sorx/{START_SCHEMA_FILENAME}"),
            },
        }),
    }
}

#[cfg(feature = "pack-zip")]
fn greentic_capability_section(
    stack_pack: &GreenticStackPackDocument,
) -> GreenticPackCapabilitySection {
    GreenticPackCapabilitySection {
        schema_version: GREENTIC_CAPABILITY_SECTION_SCHEMA_VERSION,
        declaration: GreenticCapabilityDeclaration {
            offers: stack_pack.offers.clone(),
            requires: stack_pack.requires.clone(),
            consumes: Vec::new(),
            profiles: Vec::new(),
        },
    }
}

#[cfg(feature = "pack-zip")]
fn greentic_setup_schema(ir: &CanonicalIr) -> serde_json::Value {
    serde_json::json!({
        "schema": "greentic.stack.setup.schema.v1",
        "title": format!("{} stack setup", ir.package.name),
        "type": "object",
        "properties": {
            "environment_id": {
                "type": "string",
                "description": "Deployment environment id"
            },
            "tenant_id": {
                "type": "string",
                "description": "Tenant id"
            }
        },
        "required": ["environment_id", "tenant_id"],
        "secret_requirements_ref": GREENTIC_SECRET_REQUIREMENTS_PATH
    })
}

#[cfg(feature = "pack-zip")]
fn greentic_call_request_schema() -> serde_json::Value {
    serde_json::json!({
        "$id": "greentic.stack.call.request.v1",
        "schema": "greentic.stack.call.request.v1",
        "type": "object",
        "required": [
            "schema",
            "call_id",
            "environment_id",
            "deployment_id",
            "revision_id",
            "route_id",
            "payload",
            "context"
        ],
        "properties": {
            "schema": { "const": "greentic.stack.call.request.v1" },
            "call_id": { "type": "string", "minLength": 1 },
            "environment_id": { "type": "string", "minLength": 1 },
            "deployment_id": { "type": "string", "minLength": 1 },
            "revision_id": { "type": "string", "minLength": 1 },
            "route_id": { "type": "string", "minLength": 1 },
            "payload": { "type": "object" },
            "context": { "type": "object" }
        },
        "additionalProperties": true
    })
}

#[cfg(feature = "pack-zip")]
fn greentic_call_response_schema() -> serde_json::Value {
    serde_json::json!({
        "$id": "greentic.stack.call.response.v1",
        "schema": "greentic.stack.call.response.v1",
        "type": "object",
        "required": [
            "schema",
            "call_id",
            "status",
            "payload",
            "usage",
            "metadata"
        ],
        "properties": {
            "schema": { "const": "greentic.stack.call.response.v1" },
            "call_id": { "type": "string", "minLength": 1 },
            "status": {
                "type": "string",
                "enum": ["success", "error"]
            },
            "payload": { "type": "object" },
            "usage": { "type": "object" },
            "metadata": { "type": "object" }
        },
        "additionalProperties": true
    })
}

#[cfg(feature = "pack-zip")]
fn greentic_artifacts_document(artifacts: &ArtifactSet) -> serde_json::Value {
    let mut artifacts_list = vec![
        serde_json::json!({
            "id": "canonical-ir",
            "kind": "sorla.canonical-ir",
            "path": "assets/sorla/model.cbor",
            "content_type": "application/cbor",
            "digest": format!("sha256:{}", artifacts.canonical_hash),
        }),
        serde_json::json!({
            "id": "agent-gateway",
            "kind": "sorla.agent-gateway",
            "path": format!("assets/sorla/{AGENT_GATEWAY_HANDOFF_FILENAME}"),
            "content_type": "application/json",
        }),
        serde_json::json!({
            "id": "executable-contract",
            "kind": "sorla.executable-contract",
            "path": format!("assets/sorla/{EXECUTABLE_CONTRACT_FILENAME}"),
            "content_type": "application/json",
        }),
    ];
    if !artifacts.ir.agent_endpoints.is_empty() {
        artifacts_list.push(serde_json::json!({
            "id": "designer-node-types",
            "kind": "admin.node-types",
            "path": DESIGNER_NODE_TYPES_PATH,
            "content_type": "application/json",
        }));
        artifacts_list.push(serde_json::json!({
            "id": "agent-endpoint-action-catalog",
            "kind": "admin.action-catalog",
            "path": AGENT_ENDPOINT_ACTION_CATALOG_PATH,
            "content_type": "application/json",
        }));
    }
    if artifacts.ontology_artifacts.is_some() {
        artifacts_list.push(serde_json::json!({
            "id": "ontology-graph",
            "kind": "sorla.ontology-graph",
            "path": ONTOLOGY_GRAPH_PATH,
            "content_type": "application/json",
        }));
    }
    if artifacts.ir.retrieval_bindings.is_some() {
        artifacts_list.push(serde_json::json!({
            "id": "retrieval-bindings",
            "kind": "sorla.retrieval-bindings",
            "path": RETRIEVAL_BINDINGS_PATH,
            "content_type": "application/json",
        }));
    }
    if artifacts.metrics_json.is_some() {
        artifacts_list.push(serde_json::json!({
            "id": "metrics",
            "kind": "sorla.metrics",
            "path": METRICS_PATH,
            "content_type": "application/json",
        }));
    }
    serde_json::json!({
        "schema": "greentic.stack.artifacts.v1",
        "artifacts": artifacts_list,
    })
}

#[cfg(feature = "pack-zip")]
fn greentic_admin_surfaces_document(artifacts: &ArtifactSet) -> serde_json::Value {
    let mut surfaces = Vec::new();
    if !artifacts.ir.agent_endpoints.is_empty() {
        surfaces.push(serde_json::json!({
            "id": "designer-node-types",
            "kind": "greentic.admin.page.v1",
            "title": "Designer node types",
            "asset_ref": DESIGNER_NODE_TYPES_PATH,
        }));
        surfaces.push(serde_json::json!({
            "id": "agent-endpoint-action-catalog",
            "kind": "greentic.admin.action.v1",
            "title": "Agent endpoint action catalog",
            "asset_ref": AGENT_ENDPOINT_ACTION_CATALOG_PATH,
        }));
    }
    serde_json::json!({
        "schema": "greentic.stack.admin-surfaces.v1",
        "surfaces": surfaces,
    })
}

#[cfg(feature = "pack-zip")]
fn greentic_secret_requirements() -> Vec<serde_json::Value> {
    Vec::new()
}

#[cfg(feature = "pack-zip")]
fn insert_pack_asset(
    entries: &mut BTreeMap<String, Vec<u8>>,
    asset_paths: &mut Vec<String>,
    path: String,
    bytes: Vec<u8>,
) {
    asset_paths.push(path.clone());
    entries.insert(path, bytes);
}

#[cfg(feature = "pack-zip")]
fn sorx_visible_manifest_assets(asset_paths: &[String]) -> Vec<String> {
    asset_paths
        .iter()
        .filter(|path| path.starts_with("assets/sorla/") || path.starts_with("assets/sorx/"))
        .cloned()
        .collect()
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
fn runtime_template_yaml(ir: &CanonicalIr) -> String {
    format!(
        "schema: greentic.sorx.runtime.template.v1\npackage:\n  name: {}\n  version: {}\nruntime:\n  tenant_id: ${{tenant.tenant_id}}\n  environment: ${{tenant.environment}}\nserver:\n  bind: ${{server.bind}}\n  public_base_url: ${{server.public_base_url}}\nmcp:\n  enabled: ${{mcp.enabled}}\n  bind: ${{mcp.bind}}\nproviders:\n  store:\n    kind: ${{providers.store.kind}}\n    config_ref: ${{providers.store.config_ref}}\npolicy:\n  approvals:\n    low: ${{policy.approvals.low}}\n    medium: ${{policy.approvals.medium}}\n    high: ${{policy.approvals.high}}\n    critical: ${{policy.approvals.critical}}\naudit:\n  sink: ${{audit.sink}}\n",
        ir.package.name, ir.package.version
    )
}

#[cfg(feature = "pack-zip")]
fn provider_bindings_template_yaml() -> String {
    "schema: greentic.sorx.provider-bindings.template.v1\nproviders:\n  foundationdb:\n    local:\n      kind: foundationdb\n      config_ref: providers.foundationdb.local\n      tenant_prefix: ${tenant.tenant_id}\n".to_string()
}

#[cfg(feature = "pack-zip")]
fn sorx_runtime_extension_value(
    artifacts: &ArtifactSet,
    sorx_assets: &BTreeMap<String, Vec<u8>>,
    i18n_asset_paths: &[String],
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
    if artifacts.ontology_artifacts.is_some() {
        sorla.insert(
            "ontology".to_string(),
            serde_json::json!({
                "schema": ONTOLOGY_EXTENSION_ID,
                "graph": ONTOLOGY_GRAPH_PATH,
                "ir": ONTOLOGY_IR_CBOR_PATH,
                "json_schema": ONTOLOGY_SCHEMA_PATH
            }),
        );
    }
    if artifacts.ir.retrieval_bindings.is_some() {
        sorla.insert(
            "retrieval_bindings".to_string(),
            serde_json::json!({
                "schema": RETRIEVAL_BINDINGS_SCHEMA,
                "json": RETRIEVAL_BINDINGS_PATH,
                "ir": RETRIEVAL_BINDINGS_IR_CBOR_PATH
            }),
        );
    }
    if artifacts.ir.operational_indexes.is_some() {
        sorla.insert(
            "operational_indexes".to_string(),
            serde_json::json!({
                "schema": OPERATIONAL_INDEXES_SCHEMA,
                "json": OPERATIONAL_INDEXES_PATH,
                "ir": OPERATIONAL_INDEXES_IR_CBOR_PATH
            }),
        );
    }
    if artifacts.metrics_json.is_some() {
        sorla.insert(
            "metrics".to_string(),
            serde_json::json!({
                "schema": METRICS_SCHEMA,
                "json": METRICS_PATH
            }),
        );
    }
    if !i18n_asset_paths.is_empty() {
        let locales = i18n_asset_paths
            .iter()
            .filter_map(|path| {
                Path::new(path)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|locale| {
                        serde_json::json!({
                            "locale": locale,
                            "json": path
                        })
                    })
            })
            .collect::<Vec<_>>();
        sorla.insert(
            "i18n".to_string(),
            serde_json::json!({
                "directory": I18N_ASSET_DIR,
                "locales": locales
            }),
        );
    }
    if !artifacts.ir.agent_endpoints.is_empty() {
        sorla.insert(
            "designer_node_types".to_string(),
            serde_json::json!({
                "schema": DESIGNER_NODE_TYPES_SCHEMA,
                "json": DESIGNER_NODE_TYPES_PATH
            }),
        );
        sorla.insert(
            "agent_endpoint_action_catalog".to_string(),
            serde_json::json!({
                "schema": AGENT_ENDPOINT_ACTION_CATALOG_SCHEMA,
                "json": AGENT_ENDPOINT_ACTION_CATALOG_PATH
            }),
        );
    }

    let mut sorx = sorx_assets
        .keys()
        .filter(|name| {
            name.as_str() != SORX_VALIDATION_MANIFEST_ASSET
                && name.as_str() != SORX_VALIDATION_SUITE_ASSET
                && name.as_str() != SORX_VALIDATION_SUITE_CBOR_ASSET
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
    if sorx_assets.contains_key(SORX_VALIDATION_SUITE_ASSET) {
        sorx.insert(
            "validation_suite".to_string(),
            serde_json::Value::String(SORX_VALIDATION_SUITE_PATH.to_string()),
        );
    }
    if sorx_assets.contains_key(SORX_VALIDATION_SUITE_CBOR_ASSET) {
        sorx.insert(
            "validation_suite_cbor".to_string(),
            serde_json::Value::String(SORX_VALIDATION_SUITE_CBOR_PATH.to_string()),
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
fn greentic_pack_manifest_cbor(
    options: &SorlaGtpackOptions,
    sorla_manifest: &SorlaPackManifest,
) -> Result<Vec<u8>, String> {
    let pack_id = PackId::new(&options.name)
        .map_err(|err| format!("invalid Greentic pack id `{}`: {err}", options.name))?;
    let version = semver::Version::parse(&options.version)
        .map_err(|err| format!("invalid Greentic pack version `{}`: {err}", options.version))?;
    let mut extensions = BTreeMap::new();
    extensions.insert(
        "greentic.sorla.gtpack.v1".to_string(),
        ExtensionRef {
            kind: "greentic.sorla.gtpack.v1".to_string(),
            version: "1".to_string(),
            digest: None,
            location: None,
            inline: Some(ExtensionInline::Other(serde_json::json!({
                "compat_manifest": "pack.cbor",
                "compat_manifest_schema": sorla_manifest.schema,
                "lock": "pack.lock.cbor",
                "assets": sorla_manifest.assets,
            }))),
        },
    );

    let manifest = GreenticPackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(options.name.clone()),
        version,
        kind: GreenticPackKind::Application,
        publisher: "greentic-sorla".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: Some(extensions),
        agents: BTreeMap::new(),
    };
    encode_pack_manifest(&manifest)
        .map_err(|err| format!("failed to encode Greentic pack manifest: {err}"))
}

#[cfg(feature = "pack-zip")]
fn verify_with_greentic_pack_lib(path: &Path) -> Result<(), String> {
    greentic_pack::open_pack(path, greentic_pack::SigningPolicy::DevOk)
        .map(|_| ())
        .map_err(|err| {
            format!(
                "greentic-pack-lib rejected generated gtpack {}: {}",
                path.display(),
                err.message
            )
        })
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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
    let ontology = ontology_summary(&mut archive, &manifest, &names)?;
    let retrieval_bindings = retrieval_summary(&mut archive, &manifest, &names)?;
    let operational_indexes = operational_indexes_summary(&mut archive, &manifest, &names)?;
    let metrics = metrics_summary(&mut archive, &manifest, &names)?;
    let designer_node_types = designer_node_types_summary(&mut archive, &manifest, &names)?;
    let agent_endpoint_action_catalog =
        agent_endpoint_action_catalog_summary(&mut archive, &manifest, &names)?;
    let stack_pack = greentic_stack_pack_summary(&mut archive, &manifest, &names)?;
    let optional_artifacts = [
        AGENT_ENDPOINTS_IR_CBOR_FILENAME,
        AGENT_OPENAPI_OVERLAY_FILENAME,
        AGENT_ARAZZO_FILENAME,
        MCP_TOOLS_FILENAME,
        LLMS_TXT_FRAGMENT_FILENAME,
        DESIGNER_NODE_TYPES_FILENAME,
        AGENT_ENDPOINT_ACTION_CATALOG_FILENAME,
        ONTOLOGY_GRAPH_FILENAME,
        ONTOLOGY_IR_CBOR_FILENAME,
        ONTOLOGY_SCHEMA_FILENAME,
        RETRIEVAL_BINDINGS_FILENAME,
        RETRIEVAL_BINDINGS_IR_CBOR_FILENAME,
        OPERATIONAL_INDEXES_FILENAME,
        OPERATIONAL_INDEXES_IR_CBOR_FILENAME,
        METRICS_FILENAME,
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
        ontology,
        retrieval_bindings,
        operational_indexes,
        metrics,
        designer_node_types,
        agent_endpoint_action_catalog,
        stack_pack,
    })
}

#[cfg(feature = "pack-zip")]
fn greentic_extension_path(
    manifest: &SorlaPackManifest,
    key: &str,
    expected: &str,
) -> Result<String, String> {
    let path = manifest
        .extension
        .get("greentic")
        .and_then(|greentic| greentic.get(key))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("pack.cbor is missing `greentic.{key}`"))?;
    if path != expected {
        return Err(format!(
            "pack.cbor references unsupported Greentic asset path `{path}` for `{key}`"
        ));
    }
    Ok(path.to_string())
}

#[cfg(feature = "pack-zip")]
fn greentic_stack_pack_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackStackPackInspection>, String> {
    let path = greentic_extension_path(manifest, "stack_pack", GREENTIC_STACK_PACK_PATH)?;
    if !names.contains(&path) {
        return Err(format!(
            "pack.cbor references missing stack-pack asset `{path}`"
        ));
    }
    let document = read_greentic_stack_pack(archive, &path)?;
    let secret_requirements =
        read_greentic_secret_requirements(archive, &document.setup.secret_requirements_ref)?;
    Ok(Some(SorlaGtpackStackPackInspection {
        schema: document.schema,
        stack_id: document.stack.id,
        stack_kind: document.stack.kind,
        stack_version: document.stack.version,
        offer_count: document.offers.len(),
        requirement_count: document.requires.len(),
        route_count: document.routes.len(),
        required_secret_count: secret_requirements.len(),
    }))
}

fn ontology_extension_paths(
    manifest: &SorlaPackManifest,
) -> Result<Option<(String, String, String)>, String> {
    let Some(ontology) = manifest
        .extension
        .get("sorla")
        .and_then(|sorla| sorla.get("ontology"))
    else {
        return Ok(None);
    };

    let schema = ontology
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "pack.cbor ontology extension is missing `schema`".to_string())?;
    if schema != ONTOLOGY_EXTENSION_ID {
        return Err(format!(
            "pack.cbor ontology extension has unsupported schema `{schema}`"
        ));
    }

    let graph = ontology_extension_path(ontology, "graph", ONTOLOGY_GRAPH_PATH)?;
    let ir = ontology_extension_path(ontology, "ir", ONTOLOGY_IR_CBOR_PATH)?;
    let schema_path = ontology_extension_path(ontology, "json_schema", ONTOLOGY_SCHEMA_PATH)?;
    Ok(Some((graph, ir, schema_path)))
}

fn ontology_extension_path(
    ontology: &serde_json::Value,
    key: &str,
    expected: &str,
) -> Result<String, String> {
    let path = ontology
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("pack.cbor ontology extension is missing `{key}`"))?;
    if path != expected {
        return Err(format!(
            "pack.cbor references unsupported ontology asset path `{path}` for `{key}`"
        ));
    }
    validate_relative_pack_asset_path(path)?;
    Ok(path.to_string())
}

fn validate_relative_pack_asset_path(path: &str) -> Result<(), String> {
    if path.starts_with('/') || path.contains("..") || !path.starts_with("assets/") {
        return Err(format!("pack.cbor references unsafe asset path `{path}`"));
    }
    Ok(())
}

fn retrieval_extension_paths(
    manifest: &SorlaPackManifest,
) -> Result<Option<(String, String)>, String> {
    let Some(retrieval) = manifest
        .extension
        .get("sorla")
        .and_then(|sorla| sorla.get("retrieval_bindings"))
    else {
        return Ok(None);
    };
    let schema = retrieval
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "pack.cbor retrieval_bindings extension is missing `schema`".to_string())?;
    if schema != RETRIEVAL_BINDINGS_SCHEMA {
        return Err(format!(
            "pack.cbor retrieval_bindings extension has unsupported schema `{schema}`"
        ));
    }
    let json = retrieval_extension_path(retrieval, "json", RETRIEVAL_BINDINGS_PATH)?;
    let ir = retrieval_extension_path(retrieval, "ir", RETRIEVAL_BINDINGS_IR_CBOR_PATH)?;
    Ok(Some((json, ir)))
}

fn retrieval_extension_path(
    retrieval: &serde_json::Value,
    key: &str,
    expected: &str,
) -> Result<String, String> {
    let path = retrieval
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("pack.cbor retrieval_bindings extension is missing `{key}`"))?;
    if path != expected {
        return Err(format!(
            "pack.cbor references unsupported retrieval asset path `{path}` for `{key}`"
        ));
    }
    validate_relative_pack_asset_path(path)?;
    Ok(path.to_string())
}

fn operational_indexes_extension_paths(
    manifest: &SorlaPackManifest,
) -> Result<Option<(String, String)>, String> {
    let Some(indexes) = manifest
        .extension
        .get("sorla")
        .and_then(|sorla| sorla.get("operational_indexes"))
    else {
        return Ok(None);
    };
    let schema = indexes
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "pack.cbor operational_indexes extension is missing `schema`".to_string())?;
    if schema != OPERATIONAL_INDEXES_SCHEMA {
        return Err(format!(
            "pack.cbor operational_indexes extension has unsupported schema `{schema}`"
        ));
    }
    let json = operational_indexes_extension_path(indexes, "json", OPERATIONAL_INDEXES_PATH)?;
    let ir = operational_indexes_extension_path(indexes, "ir", OPERATIONAL_INDEXES_IR_CBOR_PATH)?;
    Ok(Some((json, ir)))
}

fn operational_indexes_extension_path(
    indexes: &serde_json::Value,
    key: &str,
    expected: &str,
) -> Result<String, String> {
    let path = indexes
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("pack.cbor operational_indexes extension is missing `{key}`"))?;
    if path != expected {
        return Err(format!(
            "pack.cbor references unsupported operational indexes asset path `{path}` for `{key}`"
        ));
    }
    validate_relative_pack_asset_path(path)?;
    Ok(path.to_string())
}

fn metrics_extension_path(manifest: &SorlaPackManifest) -> Result<Option<String>, String> {
    let Some(metrics) = manifest
        .extension
        .get("sorla")
        .and_then(|sorla| sorla.get("metrics"))
    else {
        return Ok(None);
    };
    let schema = metrics
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "pack.cbor metrics extension is missing `schema`".to_string())?;
    if schema != METRICS_SCHEMA {
        return Err(format!(
            "pack.cbor metrics extension has unsupported schema `{schema}`"
        ));
    }
    let json_path = metrics
        .get("json")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "pack.cbor metrics extension is missing `json`".to_string())?;
    if json_path != METRICS_PATH {
        return Err(format!(
            "pack.cbor references unsupported metrics asset path `{json_path}`"
        ));
    }
    validate_relative_pack_asset_path(json_path)?;
    Ok(Some(json_path.to_string()))
}

fn designer_node_types_extension_path(
    manifest: &SorlaPackManifest,
) -> Result<Option<String>, String> {
    let Some(node_types) = manifest
        .extension
        .get("sorla")
        .and_then(|sorla| sorla.get("designer_node_types"))
    else {
        return Ok(None);
    };
    let schema = node_types
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "pack.cbor designer_node_types extension is missing `schema`".to_string())?;
    if schema != DESIGNER_NODE_TYPES_SCHEMA {
        return Err(format!(
            "pack.cbor designer_node_types extension has unsupported schema `{schema}`"
        ));
    }
    let json_path = node_types
        .get("json")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "pack.cbor designer_node_types extension is missing `json`".to_string())?;
    if json_path != DESIGNER_NODE_TYPES_PATH {
        return Err(format!(
            "pack.cbor references unsupported designer node types asset path `{json_path}`"
        ));
    }
    validate_relative_pack_asset_path(json_path)?;
    Ok(Some(json_path.to_string()))
}

fn agent_endpoint_action_catalog_extension_path(
    manifest: &SorlaPackManifest,
) -> Result<Option<String>, String> {
    let Some(catalog) = manifest
        .extension
        .get("sorla")
        .and_then(|sorla| sorla.get("agent_endpoint_action_catalog"))
    else {
        return Ok(None);
    };
    let schema = catalog
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            "pack.cbor agent_endpoint_action_catalog extension is missing `schema`".to_string()
        })?;
    if schema != AGENT_ENDPOINT_ACTION_CATALOG_SCHEMA {
        return Err(format!(
            "pack.cbor agent_endpoint_action_catalog extension has unsupported schema `{schema}`"
        ));
    }
    let json_path = catalog
        .get("json")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            "pack.cbor agent_endpoint_action_catalog extension is missing `json`".to_string()
        })?;
    if json_path != AGENT_ENDPOINT_ACTION_CATALOG_PATH {
        return Err(format!(
            "pack.cbor references unsupported agent endpoint action catalog asset path `{json_path}`"
        ));
    }
    validate_relative_pack_asset_path(json_path)?;
    Ok(Some(json_path.to_string()))
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
fn validation_manifest_path(manifest: &SorlaPackManifest) -> Result<Option<String>, String> {
    sorx_extension_path(
        manifest,
        "validation_manifest",
        SORX_VALIDATION_MANIFEST_PATH,
    )
}

#[cfg(feature = "pack-zip")]
fn exposure_policy_path(manifest: &SorlaPackManifest) -> Result<Option<String>, String> {
    sorx_extension_path(manifest, "exposure_policy", SORX_EXPOSURE_POLICY_PATH)
}

#[cfg(feature = "pack-zip")]
fn compatibility_path(manifest: &SorlaPackManifest) -> Result<Option<String>, String> {
    sorx_extension_path(manifest, "compatibility", SORX_COMPATIBILITY_PATH)
}

#[cfg(feature = "pack-zip")]
fn ontology_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackOntologyInspection>, String> {
    let Some((graph_path, ir_path, schema_path)) = ontology_extension_paths(manifest)? else {
        return Ok(None);
    };
    for path in [&graph_path, &ir_path, &schema_path] {
        if !names.contains(path) {
            return Err(format!(
                "pack.cbor references missing ontology asset `{path}`"
            ));
        }
    }

    let ontology_ir = read_ontology_ir(archive, &ir_path)?;
    let graph = read_ontology_graph(archive, &graph_path)?;
    let ir_bytes = zip_bytes(archive, &ir_path)?;
    let ir_hash = sha256_hex(&ir_bytes);
    if graph["ir_hash"].as_str() != Some(ir_hash.as_str()) {
        return Err("ontology.graph.json ir_hash does not match ontology.ir.cbor".to_string());
    }

    Ok(Some(SorlaGtpackOntologyInspection {
        schema: ontology_ir.schema.clone(),
        graph_schema: graph["schema"].as_str().unwrap_or("").to_string(),
        concept_count: ontology_ir.concepts.len(),
        relationship_count: ontology_ir.relationships.len(),
        constraint_count: ontology_ir.constraints.len(),
        ir_hash,
    }))
}

#[cfg(feature = "pack-zip")]
fn retrieval_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackRetrievalInspection>, String> {
    let Some((json_path, ir_path)) = retrieval_extension_paths(manifest)? else {
        return Ok(None);
    };
    for path in [&json_path, &ir_path] {
        if !names.contains(path) {
            return Err(format!(
                "pack.cbor references missing retrieval asset `{path}`"
            ));
        }
    }
    let json = read_retrieval_json(archive, &json_path)?;
    let ir = read_retrieval_ir(archive, &ir_path)?;
    if json != ir {
        return Err(
            "retrieval-bindings.json does not match retrieval-bindings.ir.cbor".to_string(),
        );
    }
    Ok(Some(SorlaGtpackRetrievalInspection {
        schema: ir.schema,
        provider_count: ir.providers.len(),
        scope_count: ir.scopes.len(),
    }))
}

#[cfg(feature = "pack-zip")]
fn operational_indexes_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackOperationalIndexesInspection>, String> {
    let Some((json_path, ir_path)) = operational_indexes_extension_paths(manifest)? else {
        return Ok(None);
    };
    for path in [&json_path, &ir_path] {
        if !names.contains(path) {
            return Err(format!(
                "pack.cbor references missing operational indexes asset `{path}`"
            ));
        }
    }
    let json = read_operational_indexes_json(archive, &json_path)?;
    let ir = read_operational_indexes_ir(archive, &ir_path)?;
    if json != ir {
        return Err(
            "operational-indexes.json does not match operational-indexes.ir.cbor".to_string(),
        );
    }
    Ok(Some(SorlaGtpackOperationalIndexesInspection {
        schema: ir.schema,
        index_count: ir.indexes.len(),
        query_requirement_count: ir.query_requirements.len(),
    }))
}

#[cfg(feature = "pack-zip")]
fn metrics_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackMetricsInspection>, String> {
    let Some(json_path) = metrics_extension_path(manifest)? else {
        return Ok(None);
    };
    if !names.contains(&json_path) {
        return Err(format!(
            "pack.cbor references missing metrics asset `{json_path}`"
        ));
    }
    let document = read_metrics_json(archive, &json_path)?;
    let names = document
        .metrics
        .iter()
        .map(|metric| metric.name.clone())
        .collect();
    Ok(Some(SorlaGtpackMetricsInspection {
        schema: document.schema,
        count: document.metrics.len(),
        names,
    }))
}

#[cfg(feature = "pack-zip")]
fn designer_node_types_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackDesignerNodeTypesInspection>, String> {
    let Some(json_path) = designer_node_types_extension_path(manifest)? else {
        return Ok(None);
    };
    if !names.contains(&json_path) {
        return Err(format!(
            "pack.cbor references missing designer node types asset `{json_path}`"
        ));
    }
    let document = read_designer_node_types(archive, &json_path)?;
    Ok(Some(SorlaGtpackDesignerNodeTypesInspection {
        schema: document.schema,
        count: document.node_types.len(),
    }))
}

#[cfg(feature = "pack-zip")]
fn agent_endpoint_action_catalog_summary<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    manifest: &SorlaPackManifest,
    names: &BTreeSet<String>,
) -> Result<Option<SorlaGtpackAgentEndpointActionCatalogInspection>, String> {
    let Some(json_path) = agent_endpoint_action_catalog_extension_path(manifest)? else {
        return Ok(None);
    };
    if !names.contains(&json_path) {
        return Err(format!(
            "pack.cbor references missing agent endpoint action catalog asset `{json_path}`"
        ));
    }
    let document = read_agent_endpoint_action_catalog(archive, &json_path)?;
    Ok(Some(SorlaGtpackAgentEndpointActionCatalogInspection {
        schema: document.schema,
        count: document.actions.len(),
    }))
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
fn validate_embedded_views<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let views_bytes = zip_bytes(archive, "assets/sorla/views.cbor")?;
    let views: Vec<ViewIr> = ciborium::de::from_reader(Cursor::new(views_bytes))
        .map_err(|err| format!("views.cbor is invalid view IR: {err}"))?;
    if views != ir.views {
        return Err("views.cbor does not match model.cbor views".to_string());
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_embedded_ontology_artifacts<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let declared_paths = ontology_extension_paths(&pack_manifest)?;
    let Some(expected_ontology) = &ir.ontology else {
        if declared_paths.is_some() {
            return Err(
                "pack.cbor declares ontology extension but model.cbor has no ontology".into(),
            );
        }
        return Ok(());
    };
    let Some((graph_path, ir_path, schema_path)) = declared_paths else {
        return Err("pack.cbor is missing sorla ontology extension".to_string());
    };

    for path in [&graph_path, &ir_path, &schema_path] {
        if !names.contains(path) {
            return Err(format!(
                "pack.cbor references missing ontology asset `{path}`"
            ));
        }
        if !pack_manifest.assets.iter().any(|asset| asset == path) {
            return Err(format!("pack.cbor assets do not include `{path}`"));
        }
        validate_lock_includes_entry(archive, path)?;
    }

    let emitted_ontology = read_ontology_ir(archive, &ir_path)?;
    if &emitted_ontology != expected_ontology {
        return Err("ontology.ir.cbor does not match model.cbor ontology IR".to_string());
    }

    let graph = read_ontology_graph(archive, &graph_path)?;
    let ir_bytes = zip_bytes(archive, &ir_path)?;
    let ir_hash = sha256_hex(&ir_bytes);
    if graph["ir_hash"].as_str() != Some(ir_hash.as_str()) {
        return Err("ontology.graph.json ir_hash does not match ontology.ir.cbor".to_string());
    }
    if graph["package"]["name"].as_str() != Some(ir.package.name.as_str()) {
        return Err("ontology.graph.json package.name does not match model.cbor".to_string());
    }
    if graph["package"]["version"].as_str() != Some(ir.package.version.as_str()) {
        return Err("ontology.graph.json package.version does not match model.cbor".to_string());
    }
    let graph_concepts = graph_array::<greentic_sorla_ir::ConceptDefinitionIr>(&graph, "concepts")?;
    if graph_concepts != emitted_ontology.concepts {
        return Err("ontology.graph.json concepts do not match ontology.ir.cbor".to_string());
    }
    let graph_relationships =
        graph_array::<greentic_sorla_ir::RelationshipDefinitionIr>(&graph, "relationships")?;
    if graph_relationships != emitted_ontology.relationships {
        return Err("ontology.graph.json relationships do not match ontology.ir.cbor".to_string());
    }
    let graph_constraints =
        graph_array::<greentic_sorla_ir::OntologyConstraintIr>(&graph, "constraints")?;
    if graph_constraints != emitted_ontology.constraints {
        return Err("ontology.graph.json constraints do not match ontology.ir.cbor".to_string());
    }
    let graph_aliases: SemanticAliasesIr =
        serde_json::from_value(graph["semantic_aliases"].clone())
            .map_err(|err| format!("ontology.graph.json `semantic_aliases` is invalid: {err}"))?;
    if graph_aliases != emitted_ontology.semantic_aliases {
        return Err(
            "ontology.graph.json semantic_aliases do not match ontology.ir.cbor".to_string(),
        );
    }
    let graph_linking: EntityLinkingIr = serde_json::from_value(graph["entity_linking"].clone())
        .map_err(|err| format!("ontology.graph.json `entity_linking` is invalid: {err}"))?;
    if graph_linking != emitted_ontology.entity_linking {
        return Err(
            "ontology.graph.json entity_linking does not match ontology.ir.cbor".to_string(),
        );
    }

    validate_ontology_backing(&emitted_ontology, ir)?;
    let schema_json: serde_json::Value = serde_json::from_str(&zip_text(archive, &schema_path)?)
        .map_err(|err| format!("{schema_path} is invalid JSON: {err}"))?;
    if schema_json["$id"].as_str() != Some(ONTOLOGY_EXTENSION_ID) {
        return Err("ontology.schema.json has unsupported $id".to_string());
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_embedded_retrieval_bindings<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let declared_paths = retrieval_extension_paths(&pack_manifest)?;
    let Some(expected) = &ir.retrieval_bindings else {
        if declared_paths.is_some() {
            return Err(
                "pack.cbor declares retrieval_bindings extension but model.cbor has no retrieval bindings"
                    .to_string(),
            );
        }
        return Ok(());
    };
    let Some((json_path, ir_path)) = declared_paths else {
        return Err("pack.cbor is missing sorla retrieval_bindings extension".to_string());
    };
    for path in [&json_path, &ir_path] {
        if !names.contains(path) {
            return Err(format!(
                "pack.cbor references missing retrieval asset `{path}`"
            ));
        }
        if !pack_manifest.assets.iter().any(|asset| asset == path) {
            return Err(format!("pack.cbor assets do not include `{path}`"));
        }
        validate_lock_includes_entry(archive, path)?;
    }
    let json = read_retrieval_json(archive, &json_path)?;
    let cbor = read_retrieval_ir(archive, &ir_path)?;
    if &json != expected || &cbor != expected {
        return Err("retrieval bindings assets do not match model.cbor".to_string());
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_embedded_operational_indexes<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let declared_paths = operational_indexes_extension_paths(&pack_manifest)?;
    let Some(expected) = &ir.operational_indexes else {
        if declared_paths.is_some() {
            return Err(
                "pack.cbor declares operational_indexes extension but model.cbor has no operational indexes"
                    .to_string(),
            );
        }
        return Ok(());
    };
    let Some((json_path, ir_path)) = declared_paths else {
        return Err("pack.cbor is missing sorla operational_indexes extension".to_string());
    };
    for path in [&json_path, &ir_path] {
        if !names.contains(path) {
            return Err(format!(
                "pack.cbor references missing operational indexes asset `{path}`"
            ));
        }
        if !pack_manifest.assets.iter().any(|asset| asset == path) {
            return Err(format!("pack.cbor assets do not include `{path}`"));
        }
        validate_lock_includes_entry(archive, path)?;
    }
    let json = read_operational_indexes_json(archive, &json_path)?;
    let cbor = read_operational_indexes_ir(archive, &ir_path)?;
    if &json != expected || &cbor != expected {
        return Err("operational indexes assets do not match model.cbor".to_string());
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_embedded_metrics<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let declared_path = metrics_extension_path(&pack_manifest)?;
    if ir.metrics.is_empty() {
        if declared_path.is_some() {
            return Err(
                "pack.cbor declares metrics extension but model.cbor has no metrics".into(),
            );
        }
        return Ok(());
    }
    let Some(json_path) = declared_path else {
        return Err("pack.cbor is missing sorla metrics extension".to_string());
    };
    if !names.contains(&json_path) {
        return Err(format!(
            "pack.cbor references missing metrics asset `{json_path}`"
        ));
    }
    if !pack_manifest.assets.iter().any(|asset| asset == &json_path) {
        return Err(format!("pack.cbor assets do not include `{json_path}`"));
    }
    validate_lock_includes_entry(archive, &json_path)?;

    let document = read_metrics_json(archive, &json_path)?;
    if document.package.name != ir.package.name
        || document.package.version != ir.package.version
        || document.package.ir_hash != canonical_hash_hex(ir)
    {
        return Err("metrics.json package metadata does not match model.cbor".to_string());
    }
    if document.metrics != metrics_artifact_metrics(ir) {
        return Err("metrics.json metrics do not match model.cbor".to_string());
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_embedded_designer_node_types<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let declared_path = designer_node_types_extension_path(&pack_manifest)?;
    if ir.agent_endpoints.is_empty() {
        if declared_path.is_some() {
            return Err(
                "pack.cbor declares designer_node_types extension but model.cbor has no agent endpoints"
                    .to_string(),
            );
        }
        return Ok(());
    }
    let Some(json_path) = declared_path else {
        return Err("pack.cbor is missing sorla designer_node_types extension".to_string());
    };
    if !names.contains(&json_path) {
        return Err(format!(
            "pack.cbor references missing designer node types asset `{json_path}`"
        ));
    }
    if !pack_manifest.assets.iter().any(|asset| asset == &json_path) {
        return Err(format!("pack.cbor assets do not include `{json_path}`"));
    }
    validate_lock_includes_entry(archive, &json_path)?;

    let document = read_designer_node_types(archive, &json_path)?;
    let expected_hash = canonical_hash_hex(ir);
    let expected_contract_hash = format!("sha256:{expected_hash}");
    if document.package.name != ir.package.name
        || document.package.version != ir.package.version
        || document.package.ir_hash != expected_hash
    {
        return Err(
            "designer-node-types.json package metadata does not match model.cbor".to_string(),
        );
    }
    if document.node_types.len() != ir.agent_endpoints.len() {
        return Err(format!(
            "designer-node-types.json has {} node types but model.cbor has {} agent endpoints",
            document.node_types.len(),
            ir.agent_endpoints.len()
        ));
    }

    let endpoints = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| (endpoint.id.as_str(), endpoint))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for node_type in &document.node_types {
        let endpoint_id = node_type.metadata.endpoint.id.as_str();
        let Some(endpoint) = endpoints.get(endpoint_id) else {
            return Err(format!(
                "designer-node-types.json references unknown endpoint `{endpoint_id}`"
            ));
        };
        if !seen.insert(endpoint_id) {
            return Err(format!(
                "designer-node-types.json contains duplicate endpoint `{endpoint_id}`"
            ));
        }
        if node_type.id != format!("sorla.agent-endpoint.{endpoint_id}") {
            return Err(format!(
                "designer-node-types.json node `{}` does not match endpoint `{endpoint_id}`",
                node_type.id
            ));
        }
        if node_type.binding.kind != "component" {
            return Err(format!(
                "designer-node-types.json node `{}` has unsupported binding kind `{}`",
                node_type.id, node_type.binding.kind
            ));
        }
        if node_type.binding.operation != DEFAULT_DESIGNER_COMPONENT_OPERATION {
            return Err(format!(
                "designer-node-types.json node `{}` has unsupported operation `{}`",
                node_type.id, node_type.binding.operation
            ));
        }
        if !is_sha256_contract_hash(&node_type.metadata.endpoint.contract_hash) {
            return Err(format!(
                "designer-node-types.json node `{}` has invalid contract_hash `{}`",
                node_type.id, node_type.metadata.endpoint.contract_hash
            ));
        }
        if node_type.metadata.endpoint.version != ir.package.version
            || node_type.metadata.endpoint.package != ir.package.name
            || node_type.metadata.endpoint.contract_hash != expected_contract_hash
        {
            return Err(format!(
                "designer-node-types.json node `{}` endpoint_ref does not match model metadata",
                node_type.id
            ));
        }
        let config_ref = node_type
            .config_schema
            .get("properties")
            .and_then(|properties| properties.get("endpoint_ref"))
            .and_then(|endpoint_ref| endpoint_ref.get("const"))
            .ok_or_else(|| {
                format!(
                    "designer-node-types.json node `{}` is missing locked endpoint_ref config",
                    node_type.id
                )
            })?;
        if config_ref.get("id").and_then(serde_json::Value::as_str) != Some(endpoint_id) {
            return Err(format!(
                "designer-node-types.json node `{}` config endpoint_ref does not match metadata",
                node_type.id
            ));
        }
        if config_ref
            .get("package")
            .and_then(serde_json::Value::as_str)
            != Some(ir.package.name.as_str())
            || config_ref
                .get("version")
                .and_then(serde_json::Value::as_str)
                != Some(ir.package.version.as_str())
            || config_ref
                .get("contract_hash")
                .and_then(serde_json::Value::as_str)
                != Some(expected_contract_hash.as_str())
        {
            return Err(format!(
                "designer-node-types.json node `{}` config endpoint_ref does not match model metadata",
                node_type.id
            ));
        }
        reject_free_text_runtime_selection(&serde_json::to_value(node_type).unwrap_or_default())?;
        for input in endpoint.inputs.iter().filter(|input| input.required) {
            let has_required = node_type
                .input_schema
                .get("required")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|required| {
                    required
                        .iter()
                        .any(|item| item.as_str() == Some(input.name.as_str()))
                });
            if !has_required {
                return Err(format!(
                    "designer-node-types.json node `{}` input schema is missing required input `{}`",
                    node_type.id, input.name
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_embedded_agent_endpoint_action_catalog<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    let declared_path = agent_endpoint_action_catalog_extension_path(&pack_manifest)?;
    if ir.agent_endpoints.is_empty() {
        if declared_path.is_some() {
            return Err(
                "pack.cbor declares agent_endpoint_action_catalog extension but model.cbor has no agent endpoints"
                    .to_string(),
            );
        }
        return Ok(());
    }
    let Some(json_path) = declared_path else {
        return Err(
            "pack.cbor is missing sorla agent_endpoint_action_catalog extension".to_string(),
        );
    };
    if !names.contains(&json_path) {
        return Err(format!(
            "pack.cbor references missing agent endpoint action catalog asset `{json_path}`"
        ));
    }
    if !pack_manifest.assets.iter().any(|asset| asset == &json_path) {
        return Err(format!("pack.cbor assets do not include `{json_path}`"));
    }
    validate_lock_includes_entry(archive, &json_path)?;

    let document = read_agent_endpoint_action_catalog(archive, &json_path)?;
    let expected_hash = canonical_hash_hex(ir);
    let expected_contract_hash = format!("sha256:{expected_hash}");
    if document.package.name != ir.package.name
        || document.package.version != ir.package.version
        || document.package.ir_hash != expected_hash
    {
        return Err(
            "agent-endpoint-action-catalog.json package metadata does not match model.cbor"
                .to_string(),
        );
    }
    if document.actions.len() != ir.agent_endpoints.len() {
        return Err(format!(
            "agent-endpoint-action-catalog.json has {} actions but model.cbor has {} agent endpoints",
            document.actions.len(),
            ir.agent_endpoints.len()
        ));
    }

    let endpoints = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| (endpoint.id.as_str(), endpoint))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for action in &document.actions {
        let Some(endpoint) = endpoints.get(action.id.as_str()) else {
            return Err(format!(
                "agent-endpoint-action-catalog.json references unknown endpoint `{}`",
                action.id
            ));
        };
        if !seen.insert(action.id.as_str()) {
            return Err(format!(
                "agent-endpoint-action-catalog.json contains duplicate endpoint `{}`",
                action.id
            ));
        }
        if action.endpoint_ref.id != action.id
            || action.endpoint_ref.version != ir.package.version
            || action.endpoint_ref.package != ir.package.name
            || action.endpoint_ref.contract_hash != expected_contract_hash
        {
            return Err(format!(
                "agent-endpoint-action-catalog.json action `{}` endpoint_ref does not match model metadata",
                action.id
            ));
        }
        if !is_sha256_contract_hash(&action.endpoint_ref.contract_hash) {
            return Err(format!(
                "agent-endpoint-action-catalog.json action `{}` has invalid contract_hash `{}`",
                action.id, action.endpoint_ref.contract_hash
            ));
        }
        for input in endpoint.inputs.iter().filter(|input| input.required) {
            let has_required = action
                .input_schema
                .get("required")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|required| {
                    required
                        .iter()
                        .any(|item| item.as_str() == Some(input.name.as_str()))
                });
            if !has_required {
                return Err(format!(
                    "agent-endpoint-action-catalog.json action `{}` input schema is missing required input `{}`",
                    action.id, input.name
                ));
            }
        }
        reject_free_text_runtime_selection(&serde_json::to_value(action).unwrap_or_default())?;
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_embedded_greentic_stack_pack<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    names: &BTreeSet<String>,
    ir: &CanonicalIr,
    inspection: &SorlaGtpackInspection,
) -> Result<(), String> {
    for path in [
        GREENTIC_STACK_PACK_PATH,
        GREENTIC_CAPABILITIES_PATH,
        GREENTIC_ROUTES_PATH,
        GREENTIC_SETUP_SCHEMA_PATH,
        GREENTIC_CALL_REQUEST_SCHEMA_PATH,
        GREENTIC_CALL_RESPONSE_SCHEMA_PATH,
        GREENTIC_ARTIFACTS_PATH,
        GREENTIC_ADMIN_SURFACES_PATH,
        GREENTIC_SECRET_REQUIREMENTS_PATH,
    ] {
        if !names.contains(path) {
            return Err(format!(
                "gtpack is missing required Greentic asset `{path}`"
            ));
        }
    }
    let manifest_bytes = zip_bytes(archive, "pack.cbor")?;
    let pack_manifest: SorlaPackManifest =
        ciborium::de::from_reader(Cursor::new(manifest_bytes))
            .map_err(|err| format!("pack.cbor is invalid SoRLa pack manifest: {err}"))?;
    for (key, expected) in [
        ("stack_pack", GREENTIC_STACK_PACK_PATH),
        ("capabilities", GREENTIC_CAPABILITIES_PATH),
        ("routes", GREENTIC_ROUTES_PATH),
        ("setup_schema", GREENTIC_SETUP_SCHEMA_PATH),
        ("call_request_schema", GREENTIC_CALL_REQUEST_SCHEMA_PATH),
        ("call_response_schema", GREENTIC_CALL_RESPONSE_SCHEMA_PATH),
        ("artifacts", GREENTIC_ARTIFACTS_PATH),
        ("admin_surfaces", GREENTIC_ADMIN_SURFACES_PATH),
        ("secret_requirements", GREENTIC_SECRET_REQUIREMENTS_PATH),
    ] {
        let path = greentic_extension_path(&pack_manifest, key, expected)?;
        if !names.contains(&path) {
            return Err(format!(
                "pack.cbor Greentic extension references missing asset `{path}`"
            ));
        }
    }

    let stack_pack = read_greentic_stack_pack(archive, GREENTIC_STACK_PACK_PATH)?;
    if stack_pack.metadata["sorla_package"]["name"].as_str() != Some(ir.package.name.as_str()) {
        return Err("stack-pack metadata does not match canonical IR package name".to_string());
    }
    if stack_pack.metadata["sorla_package"]["version"].as_str() != Some(ir.package.version.as_str())
    {
        return Err("stack-pack metadata does not match canonical IR package version".to_string());
    }
    if stack_pack.metadata["sorla_package"]["ir_hash"].as_str() != Some(inspection.ir_hash.as_str())
    {
        return Err("stack-pack metadata does not match canonical IR hash".to_string());
    }

    let capabilities = read_greentic_capabilities(archive, GREENTIC_CAPABILITIES_PATH)?;
    if capabilities.declaration.offers != stack_pack.offers {
        return Err("capability declaration offers drifted from stack-pack offers".to_string());
    }
    if capabilities.declaration.requires != stack_pack.requires {
        return Err(
            "capability declaration requirements drifted from stack-pack requirements".to_string(),
        );
    }

    let routes_doc: serde_json::Value =
        serde_json::from_str(&zip_text(archive, GREENTIC_ROUTES_PATH)?)
            .map_err(|err| format!("{GREENTIC_ROUTES_PATH} is invalid JSON: {err}"))?;
    if routes_doc.get("schema").and_then(serde_json::Value::as_str)
        != Some("greentic.stack.routes.v1")
    {
        return Err(format!(
            "{GREENTIC_ROUTES_PATH} has unsupported routes schema"
        ));
    }
    let routes_value = serde_json::to_value(&stack_pack.routes).map_err(|err| err.to_string())?;
    if routes_doc.get("routes") != Some(&routes_value) {
        return Err("routes document drifted from stack-pack routes".to_string());
    }

    let setup_schema: serde_json::Value =
        serde_json::from_str(&zip_text(archive, GREENTIC_SETUP_SCHEMA_PATH)?)
            .map_err(|err| format!("{GREENTIC_SETUP_SCHEMA_PATH} is invalid JSON: {err}"))?;
    if setup_schema
        .get("secret_requirements_ref")
        .and_then(serde_json::Value::as_str)
        != Some(GREENTIC_SECRET_REQUIREMENTS_PATH)
    {
        return Err("setup schema does not reference the secret requirements asset".to_string());
    }
    let _secret_requirements =
        read_greentic_secret_requirements(archive, GREENTIC_SECRET_REQUIREMENTS_PATH)?;
    validate_greentic_call_schema(
        archive,
        GREENTIC_CALL_REQUEST_SCHEMA_PATH,
        "greentic.stack.call.request.v1",
    )?;
    validate_greentic_call_schema(
        archive,
        GREENTIC_CALL_RESPONSE_SCHEMA_PATH,
        "greentic.stack.call.response.v1",
    )?;
    validate_greentic_artifacts_document(archive, GREENTIC_ARTIFACTS_PATH, names)?;
    validate_greentic_admin_surfaces_document(archive, GREENTIC_ADMIN_SURFACES_PATH, names)?;

    for path in [
        GREENTIC_STACK_PACK_PATH,
        GREENTIC_CAPABILITIES_PATH,
        GREENTIC_ROUTES_PATH,
        GREENTIC_SETUP_SCHEMA_PATH,
        GREENTIC_CALL_REQUEST_SCHEMA_PATH,
        GREENTIC_CALL_RESPONSE_SCHEMA_PATH,
        GREENTIC_ARTIFACTS_PATH,
        GREENTIC_ADMIN_SURFACES_PATH,
        GREENTIC_SECRET_REQUIREMENTS_PATH,
    ] {
        validate_lock_includes_entry(archive, path)?;
    }
    Ok(())
}

fn is_sha256_contract_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn reject_free_text_runtime_selection(value: &serde_json::Value) -> Result<(), String> {
    const FORBIDDEN_KEYS: [&str; 4] = [
        "action_label",
        "action_alias",
        "intent_query",
        "natural_language_action",
    ];
    match value {
        serde_json::Value::Object(object) => {
            for (key, nested) in object {
                if FORBIDDEN_KEYS.contains(&key.as_str()) {
                    return Err(format!(
                        "generated metadata contains forbidden runtime action selection field `{key}`"
                    ));
                }
                reject_free_text_runtime_selection(nested)?;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                reject_free_text_runtime_selection(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn graph_array<T>(graph: &serde_json::Value, key: &str) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(graph[key].clone())
        .map_err(|err| format!("ontology.graph.json `{key}` is invalid: {err}"))
}

#[cfg(feature = "pack-zip")]
fn validate_ontology_backing(ontology: &OntologyModelIr, ir: &CanonicalIr) -> Result<(), String> {
    let records: BTreeMap<_, _> = ir
        .records
        .iter()
        .map(|record| (record.name.as_str(), record))
        .collect();
    for concept in &ontology.concepts {
        if let Some(backing) = &concept.backing {
            validate_ontology_backing_ref(&records, backing, &format!("concept `{}`", concept.id))?;
        }
    }
    for relationship in &ontology.relationships {
        if let Some(backing) = &relationship.backing {
            validate_ontology_backing_ref(
                &records,
                backing,
                &format!("relationship `{}`", relationship.id),
            )?;
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_ontology_backing_ref(
    records: &BTreeMap<&str, &greentic_sorla_ir::RecordIr>,
    backing: &greentic_sorla_ir::OntologyBackingIr,
    label: &str,
) -> Result<(), String> {
    let record = records.get(backing.record.as_str()).ok_or_else(|| {
        format!(
            "ontology {label} backing references unknown record `{}`",
            backing.record
        )
    })?;
    for field in [&backing.from_field, &backing.to_field]
        .into_iter()
        .flatten()
    {
        if !record
            .fields
            .iter()
            .any(|candidate| candidate.name == *field)
        {
            return Err(format!(
                "ontology {label} backing references unknown field `{}` on record `{}`",
                field, backing.record
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
fn read_validation_manifest<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<SorxValidationManifest, String> {
    serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))
}

#[cfg(feature = "pack-zip")]
fn read_exposure_policy<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<SorxExposurePolicy, String> {
    serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))
}

#[cfg(feature = "pack-zip")]
fn read_compatibility_manifest<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<SorxCompatibilityManifest, String> {
    serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))
}

#[cfg(feature = "pack-zip")]
fn read_ontology_ir<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<OntologyModelIr, String> {
    let bytes = zip_bytes(archive, path)?;
    ciborium::de::from_reader(Cursor::new(bytes))
        .map_err(|err| format!("{path} is invalid ontology IR CBOR: {err}"))
}

#[cfg(feature = "pack-zip")]
fn read_ontology_graph<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<serde_json::Value, String> {
    let graph: serde_json::Value = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid ontology graph JSON: {err}"))?;
    if graph["schema"].as_str() != Some(ONTOLOGY_GRAPH_SCHEMA) {
        return Err(format!(
            "{path} has unsupported ontology graph schema `{}`",
            graph["schema"].as_str().unwrap_or("<missing>")
        ));
    }
    Ok(graph)
}

#[cfg(feature = "pack-zip")]
fn read_retrieval_ir<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<RetrievalBindingsIr, String> {
    let bytes = zip_bytes(archive, path)?;
    ciborium::de::from_reader(Cursor::new(bytes))
        .map_err(|err| format!("{path} is invalid retrieval bindings CBOR: {err}"))
}

#[cfg(feature = "pack-zip")]
fn read_retrieval_json<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<RetrievalBindingsIr, String> {
    let retrieval: RetrievalBindingsIr = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid retrieval bindings JSON: {err}"))?;
    if retrieval.schema != RETRIEVAL_BINDINGS_SCHEMA {
        return Err(format!(
            "{path} has unsupported retrieval bindings schema `{}`",
            retrieval.schema
        ));
    }
    Ok(retrieval)
}

#[cfg(feature = "pack-zip")]
fn read_operational_indexes_ir<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<OperationalIndexesIr, String> {
    let bytes = zip_bytes(archive, path)?;
    ciborium::de::from_reader(Cursor::new(bytes))
        .map_err(|err| format!("{path} is invalid operational indexes CBOR: {err}"))
}

#[cfg(feature = "pack-zip")]
fn read_operational_indexes_json<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<OperationalIndexesIr, String> {
    let indexes: OperationalIndexesIr = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid operational indexes JSON: {err}"))?;
    if indexes.schema != OPERATIONAL_INDEXES_SCHEMA {
        return Err(format!(
            "{path} has unsupported operational indexes schema `{}`",
            indexes.schema
        ));
    }
    Ok(indexes)
}

#[cfg(feature = "pack-zip")]
fn read_metrics_json<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<MetricsArtifactDocument, String> {
    let document: MetricsArtifactDocument = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid metrics JSON: {err}"))?;
    if document.schema != METRICS_SCHEMA {
        return Err(format!(
            "{path} has unsupported metrics schema `{}`",
            document.schema
        ));
    }
    Ok(document)
}

#[cfg(feature = "pack-zip")]
fn read_designer_node_types<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<DesignerNodeTypesDocument, String> {
    let document: DesignerNodeTypesDocument = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid designer node types JSON: {err}"))?;
    if document.schema != DESIGNER_NODE_TYPES_SCHEMA {
        return Err(format!(
            "{path} has unsupported designer node types schema `{}`",
            document.schema
        ));
    }
    Ok(document)
}

#[cfg(feature = "pack-zip")]
fn read_agent_endpoint_action_catalog<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<AgentEndpointActionCatalogDocument, String> {
    let document: AgentEndpointActionCatalogDocument =
        serde_json::from_str(&zip_text(archive, path)?).map_err(|err| {
            format!("{path} is invalid agent endpoint action catalog JSON: {err}")
        })?;
    if document.schema != AGENT_ENDPOINT_ACTION_CATALOG_SCHEMA {
        return Err(format!(
            "{path} has unsupported agent endpoint action catalog schema `{}`",
            document.schema
        ));
    }
    Ok(document)
}

#[cfg(feature = "pack-zip")]
fn read_greentic_stack_pack<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<GreenticStackPackDocument, String> {
    let document: GreenticStackPackDocument = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid Greentic stack-pack JSON: {err}"))?;
    validate_greentic_stack_pack_document(&document)?;
    Ok(document)
}

#[cfg(feature = "pack-zip")]
fn read_greentic_capabilities<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<GreenticPackCapabilitySection, String> {
    let document: GreenticPackCapabilitySection =
        serde_json::from_str(&zip_text(archive, path)?)
            .map_err(|err| format!("{path} is invalid Greentic capabilities JSON: {err}"))?;
    validate_greentic_capability_section(&document)?;
    Ok(document)
}

#[cfg(feature = "pack-zip")]
fn read_greentic_secret_requirements<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let document: Vec<serde_json::Value> = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid secret requirements JSON: {err}"))?;
    Ok(document)
}

#[cfg(feature = "pack-zip")]
fn validate_greentic_stack_pack_document(
    document: &GreenticStackPackDocument,
) -> Result<(), String> {
    if document.schema != GREENTIC_STACK_PACK_SCHEMA {
        return Err(format!(
            "stack-pack has unsupported schema `{}`",
            document.schema
        ));
    }
    if document.stack.id.trim().is_empty() {
        return Err("stack-pack stack id must not be empty".to_string());
    }
    if document.stack.kind != "application-stack" {
        return Err(format!(
            "stack-pack has unsupported stack kind `{}`",
            document.stack.kind
        ));
    }
    semver::Version::parse(&document.stack.version).map_err(|err| {
        format!(
            "stack-pack has invalid version `{}`: {err}",
            document.stack.version
        )
    })?;
    if document.offers.is_empty() {
        return Err("stack-pack must offer an application stack capability".to_string());
    }
    for offer in &document.offers {
        validate_named_id("stack-pack offer", &offer.id)?;
        validate_capability_id(&offer.capability)?;
    }
    for requirement in &document.requires {
        validate_named_id("stack-pack requirement", &requirement.id)?;
        validate_capability_id(&requirement.capability)?;
    }
    let stack_offer = document
        .offers
        .iter()
        .find(|offer| offer.capability == CAP_STACK_APPLICATION_V1)
        .ok_or_else(|| format!("stack-pack must offer `{CAP_STACK_APPLICATION_V1}`"))?;
    validate_contract_metadata(
        "stack-pack application offer",
        &stack_offer.metadata,
        &[CONTRACT_STACK_INVOKE_V1, CONTRACT_STACK_ROUTES_V1],
    )?;
    let runtime_req = document
        .requires
        .iter()
        .find(|requirement| requirement.capability == CAP_RUNTIME_HOST_V1)
        .ok_or_else(|| format!("stack-pack must require `{CAP_RUNTIME_HOST_V1}`"))?;
    validate_contract_metadata(
        "stack-pack runtime requirement",
        &runtime_req.metadata,
        &[CONTRACT_RUNTIME_INVOKE_V1, CONTRACT_RUNTIME_TRAFFIC_V1],
    )?;
    if !document
        .requires
        .iter()
        .any(|requirement| requirement.capability == CAP_SECRETS_V1)
    {
        return Err(format!("stack-pack must require `{CAP_SECRETS_V1}`"));
    }
    if document.routes.is_empty() {
        return Err("stack-pack must declare at least one route".to_string());
    }
    let mut route_ids = BTreeSet::new();
    for route in &document.routes {
        validate_named_id("stack-pack route", &route.id)?;
        if !route_ids.insert(route.id.as_str()) {
            return Err(format!("stack-pack route id `{}` is duplicated", route.id));
        }
        if route.method != "POST" {
            return Err(format!(
                "stack-pack route `{}` has unsupported method `{}`",
                route.id, route.method
            ));
        }
        if !route.path.starts_with('/') {
            return Err(format!(
                "stack-pack route `{}` path must start with `/`",
                route.id
            ));
        }
        if route.contract != CONTRACT_STACK_INVOKE_V1 {
            return Err(format!(
                "stack-pack route `{}` has unsupported contract `{}`",
                route.id, route.contract
            ));
        }
        if route.request_schema_ref != GREENTIC_CALL_REQUEST_SCHEMA_PATH {
            return Err(format!(
                "stack-pack route `{}` has unsupported request_schema_ref `{}`",
                route.id, route.request_schema_ref
            ));
        }
        if route.response_schema_ref != GREENTIC_CALL_RESPONSE_SCHEMA_PATH {
            return Err(format!(
                "stack-pack route `{}` has unsupported response_schema_ref `{}`",
                route.id, route.response_schema_ref
            ));
        }
    }
    if document.setup.schema_ref != GREENTIC_SETUP_SCHEMA_PATH {
        return Err(format!(
            "stack-pack setup schema_ref must be `{GREENTIC_SETUP_SCHEMA_PATH}`"
        ));
    }
    if document.setup.secret_requirements_ref != GREENTIC_SECRET_REQUIREMENTS_PATH {
        return Err(format!(
            "stack-pack secret_requirements_ref must be `{GREENTIC_SECRET_REQUIREMENTS_PATH}`"
        ));
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_greentic_call_schema<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    expected_schema: &str,
) -> Result<(), String> {
    let schema: serde_json::Value = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))?;
    if schema.get("schema").and_then(serde_json::Value::as_str) != Some(expected_schema) {
        return Err(format!("{path} has unsupported schema"));
    }
    if schema.get("$id").and_then(serde_json::Value::as_str) != Some(expected_schema) {
        return Err(format!("{path} has unsupported $id"));
    }
    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{path} is missing required fields"))?;
    for field in ["schema", "call_id"] {
        if !required.iter().any(|item| item.as_str() == Some(field)) {
            return Err(format!("{path} required fields are missing `{field}`"));
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_greentic_artifacts_document<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    names: &BTreeSet<String>,
) -> Result<(), String> {
    let document: serde_json::Value = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))?;
    if document.get("schema").and_then(serde_json::Value::as_str)
        != Some("greentic.stack.artifacts.v1")
    {
        return Err(format!("{path} has unsupported schema"));
    }
    let artifacts = document
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{path} is missing artifacts"))?;
    if artifacts.is_empty() {
        return Err(format!("{path} must list at least one artifact"));
    }
    for artifact in artifacts {
        let asset_ref = artifact
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{path} contains an artifact without path"))?;
        if !names.contains(asset_ref) {
            return Err(format!(
                "{path} references missing artifact asset `{asset_ref}`"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_greentic_admin_surfaces_document<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    names: &BTreeSet<String>,
) -> Result<(), String> {
    let document: serde_json::Value = serde_json::from_str(&zip_text(archive, path)?)
        .map_err(|err| format!("{path} is invalid JSON: {err}"))?;
    if document.get("schema").and_then(serde_json::Value::as_str)
        != Some("greentic.stack.admin-surfaces.v1")
    {
        return Err(format!("{path} has unsupported schema"));
    }
    let surfaces = document
        .get("surfaces")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{path} is missing surfaces"))?;
    for surface in surfaces {
        let asset_ref = surface
            .get("asset_ref")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{path} contains a surface without asset_ref"))?;
        if !names.contains(asset_ref) {
            return Err(format!(
                "{path} references missing admin surface asset `{asset_ref}`"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_greentic_capability_section(
    document: &GreenticPackCapabilitySection,
) -> Result<(), String> {
    if document.schema_version != GREENTIC_CAPABILITY_SECTION_SCHEMA_VERSION {
        return Err(format!(
            "capability section has unsupported schema_version `{}`",
            document.schema_version
        ));
    }
    let mut offer_ids = BTreeSet::new();
    for offer in &document.declaration.offers {
        validate_named_id("capability offer", &offer.id)?;
        validate_capability_id(&offer.capability)?;
        if !offer_ids.insert(offer.id.as_str()) {
            return Err(format!("capability offer id `{}` is duplicated", offer.id));
        }
    }
    let mut requirement_ids = BTreeSet::new();
    for requirement in &document.declaration.requires {
        validate_named_id("capability requirement", &requirement.id)?;
        validate_capability_id(&requirement.capability)?;
        if !requirement_ids.insert(requirement.id.as_str()) {
            return Err(format!(
                "capability requirement id `{}` is duplicated",
                requirement.id
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_named_id(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} id must not be empty"));
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_capability_id(value: &str) -> Result<(), String> {
    if !value.starts_with("cap://") {
        return Err(format!(
            "capability id `{value}` must use the cap:// scheme"
        ));
    }
    if value["cap://".len()..].is_empty() {
        return Err("capability id must not be empty after cap://".to_string());
    }
    for (index, ch) in value.char_indices() {
        if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '/' | '-' | '_' | '.' | '+') {
            continue;
        }
        return Err(format!(
            "capability id `{value}` contains invalid character {ch:?} at index {index}"
        ));
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
fn validate_contract_metadata(
    label: &str,
    metadata: &serde_json::Value,
    required_contracts: &[&str],
) -> Result<(), String> {
    let contracts = metadata
        .get("contracts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{label} metadata is missing contracts"))?;
    for required in required_contracts {
        if !contracts
            .iter()
            .any(|value| value.as_str() == Some(*required))
        {
            return Err(format!("{label} metadata is missing contract `{required}`"));
        }
    }
    Ok(())
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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
    validate_embedded_views(&mut archive, &ir)?;
    validate_embedded_ontology_artifacts(&mut archive, &names, &ir)?;
    validate_embedded_retrieval_bindings(&mut archive, &names, &ir)?;
    validate_embedded_operational_indexes(&mut archive, &names, &ir)?;
    validate_embedded_metrics(&mut archive, &names, &ir)?;
    validate_embedded_designer_node_types(&mut archive, &names, &ir)?;
    validate_embedded_agent_endpoint_action_catalog(&mut archive, &names, &ir)?;
    validate_embedded_greentic_stack_pack(&mut archive, &names, &ir, &inspection)?;
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
    validate_agent_gateway_runtime_contract(&gateway, &ir)?;

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

    let mut checked_assets = required_pack_entries()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if inspection.ontology.is_some() {
        checked_assets.extend(
            [
                ONTOLOGY_GRAPH_PATH,
                ONTOLOGY_IR_CBOR_PATH,
                ONTOLOGY_SCHEMA_PATH,
            ]
            .into_iter()
            .map(str::to_string),
        );
    }
    if inspection.retrieval_bindings.is_some() {
        checked_assets.extend(
            [RETRIEVAL_BINDINGS_PATH, RETRIEVAL_BINDINGS_IR_CBOR_PATH]
                .into_iter()
                .map(str::to_string),
        );
    }
    if inspection.operational_indexes.is_some() {
        checked_assets.extend(
            [OPERATIONAL_INDEXES_PATH, OPERATIONAL_INDEXES_IR_CBOR_PATH]
                .into_iter()
                .map(str::to_string),
        );
    }
    if inspection.metrics.is_some() {
        checked_assets.push(METRICS_PATH.to_string());
    }
    if inspection.designer_node_types.is_some() {
        checked_assets.push(DESIGNER_NODE_TYPES_PATH.to_string());
    }
    if inspection.agent_endpoint_action_catalog.is_some() {
        checked_assets.push(AGENT_ENDPOINT_ACTION_CATALOG_PATH.to_string());
    }
    if inspection.stack_pack.is_some() {
        checked_assets.extend(
            [
                GREENTIC_STACK_PACK_PATH,
                GREENTIC_CAPABILITIES_PATH,
                GREENTIC_ROUTES_PATH,
                GREENTIC_SETUP_SCHEMA_PATH,
                GREENTIC_CALL_REQUEST_SCHEMA_PATH,
                GREENTIC_CALL_RESPONSE_SCHEMA_PATH,
                GREENTIC_ARTIFACTS_PATH,
                GREENTIC_ADMIN_SURFACES_PATH,
                GREENTIC_SECRET_REQUIREMENTS_PATH,
            ]
            .into_iter()
            .map(str::to_string),
        );
    }

    Ok(SorlaGtpackDoctorReport {
        path: inspection.path,
        status: "ok".to_string(),
        checked_assets,
    })
}

#[cfg(feature = "pack-zip")]
fn validate_agent_gateway_runtime_contract(
    gateway: &AgentGatewayHandoffManifest,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let ir_endpoints = ir
        .agent_endpoints
        .iter()
        .map(|endpoint| (endpoint.id.as_str(), endpoint))
        .collect::<BTreeMap<_, _>>();

    for route in &gateway.endpoints {
        let endpoint = ir_endpoints.get(route.id.as_str()).ok_or_else(|| {
            format!(
                "agent-gateway.json references unknown endpoint `{}`",
                route.id
            )
        })?;
        let command = sorx_runtime_command_spec(endpoint, ir);
        let expected_method = sorx_runtime_method(endpoint, command.as_ref());
        let required_inputs = endpoint
            .inputs
            .iter()
            .filter(|input| input.required)
            .map(|input| input.name.as_str())
            .collect::<Vec<_>>();

        if route.method == "GET" && !required_inputs.is_empty() {
            return Err(format!(
                "agent-gateway.json endpoint `{}` declares GET with required body-style inputs: {}; regenerate with a body-preserving method",
                route.id,
                required_inputs.join(", ")
            ));
        }
        if route.method != expected_method {
            return Err(format!(
                "agent-gateway.json endpoint `{}` declares method `{}`, expected `{expected_method}` for its generated backing contract",
                route.id, route.method
            ));
        }
    }

    Ok(())
}

#[cfg(feature = "pack-zip")]
fn required_pack_entries() -> Vec<&'static str> {
    vec![
        "pack.cbor",
        "pack.lock.cbor",
        "manifest.cbor",
        "assets/sorla/model.cbor",
        "assets/sorla/views.cbor",
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
        GREENTIC_STACK_PACK_PATH,
        GREENTIC_CAPABILITIES_PATH,
        GREENTIC_ROUTES_PATH,
        GREENTIC_SETUP_SCHEMA_PATH,
        GREENTIC_CALL_REQUEST_SCHEMA_PATH,
        GREENTIC_CALL_RESPONSE_SCHEMA_PATH,
        GREENTIC_ARTIFACTS_PATH,
        GREENTIC_ADMIN_SURFACES_PATH,
        GREENTIC_SECRET_REQUIREMENTS_PATH,
    ]
}

#[cfg(feature = "pack-zip")]
fn open_gtpack(path: &Path) -> Result<ZipArchive<fs::File>, String> {
    let file = fs::File::open(path)
        .map_err(|err| format!("failed to open gtpack {}: {err}", path.display()))?;
    ZipArchive::new(file).map_err(|err| format!("failed to read gtpack {}: {err}", path.display()))
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
fn zip_text<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Result<String, String> {
    let bytes = zip_bytes(archive, name)?;
    String::from_utf8(bytes).map_err(|err| format!("`{name}` is not UTF-8: {err}"))
}

#[cfg(feature = "pack-zip")]
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

#[cfg(feature = "pack-zip")]
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
                "from_version": migration.from_version,
                "to_version": migration.to_version,
                "projection_updates": migration.projection_updates,
                "backfills": migration.backfills,
                "operations": migration.operations,
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
        .map(|endpoint| {
            let command = sorx_runtime_command_spec(endpoint, ir);
            let method = sorx_runtime_method(endpoint, command.as_ref()).to_string();
            AgentGatewayEndpointRef {
                endpoint_id: endpoint.id.clone(),
                operation_id: endpoint.id.clone(),
                operation: sorx_runtime_operation(endpoint, command.as_ref()).to_string(),
                command,
                method,
                path: sorx_runtime_path(endpoint, ir),
                entity: sorx_runtime_entity(endpoint, ir),
                collection: sorx_runtime_collection(endpoint, ir),
                provider_binding: "store".to_string(),
                id: endpoint.id.clone(),
                title: endpoint.title.clone(),
                intent: endpoint.intent.clone(),
                risk: agent_endpoint_risk_label(&endpoint.risk).to_string(),
                approval: agent_endpoint_approval_label(&endpoint.approval).to_string(),
                authorization: endpoint.authorization.clone(),
                input_schema: object_schema_value(&endpoint.inputs),
                output_schema: output_object_schema_value(&endpoint.outputs),
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
            }
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
        schema: SORX_AGENT_GATEWAY_SCHEMA.to_string(),
        package: AgentGatewayPackageRef {
            name: ir.package.name.clone(),
            version: ir.package.version.clone(),
            ir_version: format!("{}.{}", ir.ir_version.major, ir.ir_version.minor),
            ir_hash: canonical_hash_hex(ir),
        },
        record_hierarchy: agent_gateway_record_hierarchy(ir),
        endpoints,
        provider_contract: AgentGatewayProviderContract {
            categories: aggregated_provider_requirements(ir),
        },
        exports,
        notes: vec![
            "This manifest includes SORX runtime route metadata plus SoRLa handoff context."
                .to_string(),
        ],
    }
}

fn sorx_runtime_operation(
    endpoint: &AgentEndpointIr,
    command: Option<&serde_json::Value>,
) -> &'static str {
    if command.is_some() {
        "command"
    } else if endpoint.id.starts_with("update_") || endpoint.id.contains("_update_") {
        "update"
    } else if endpoint.id.starts_with("delete_")
        || endpoint.id.starts_with("remove_")
        || endpoint.id.contains("_refund")
    {
        "delete"
    } else if endpoint.id.starts_with("create_")
        || endpoint.id.starts_with("add_")
        || endpoint.id.starts_with("join_")
        || endpoint.id.starts_with("leave_")
        || endpoint.id.starts_with("cancel_")
        || endpoint.id.starts_with("record_")
        || endpoint.id.starts_with("assign_")
        || endpoint.id.starts_with("generate_")
        || endpoint.id.starts_with("apply_")
        || endpoint.id.starts_with("submit_")
        || endpoint.id.starts_with("approve_")
        || endpoint.id.starts_with("grant_")
        || endpoint.id.starts_with("link_")
        || endpoint.emits.is_some()
    {
        "create"
    } else {
        "query"
    }
}

fn sorx_runtime_method(
    endpoint: &AgentEndpointIr,
    command: Option<&serde_json::Value>,
) -> &'static str {
    match sorx_runtime_operation(endpoint, command) {
        "query" if endpoint.inputs.is_empty() => "GET",
        "query" => "POST",
        "update" => "PATCH",
        "delete" => "DELETE",
        _ => "POST",
    }
}

fn sorx_runtime_path(endpoint: &AgentEndpointIr, ir: &CanonicalIr) -> String {
    let collection = sorx_runtime_collection(endpoint, ir);
    match sorx_runtime_operation(endpoint, sorx_runtime_command_spec(endpoint, ir).as_ref()) {
        "query" => format!("/v1/agent/{collection}/query/{}", endpoint.id),
        "update" => match sorx_runtime_id_input(endpoint, &sorx_runtime_entity(endpoint, ir)) {
            Some(id_input) => format!("/v1/agent/{collection}/{{{id_input}}}"),
            None => format!("/v1/agent/{collection}/{}", endpoint.id),
        },
        "delete" => match sorx_runtime_id_input(endpoint, &sorx_runtime_entity(endpoint, ir)) {
            Some(id_input) => format!("/v1/agent/{collection}/{{{id_input}}}"),
            None => format!("/v1/agent/{collection}/{}", endpoint.id),
        },
        _ if endpoint.id.starts_with("create_") => format!("/v1/agent/{collection}/create"),
        _ => format!("/v1/agent/{collection}/{}", endpoint.id),
    }
}

fn sorx_runtime_command_spec(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    if let Some(execution) = &endpoint.execution {
        return Some(execution.clone());
    }

    if let Some(command) = sorx_runtime_bulk_import_command(endpoint) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_archive_restore_command(endpoint, ir) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_approval_status_command(endpoint, ir) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_show_waiting_list_command(endpoint, ir) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_join_waiting_list_command(endpoint, ir) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_leave_waiting_list_command(endpoint, ir) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_retrieve_field_command(endpoint, ir) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_generate_field_command(endpoint, ir) {
        return Some(command);
    }

    if let Some(command) = sorx_runtime_side_effect_event_command(endpoint) {
        return Some(command);
    }

    if !(endpoint.id.starts_with("leave_")
        || endpoint.id.starts_with("cancel_")
        || endpoint.id.starts_with("revoke_")
        || endpoint.id.starts_with("unlink_"))
    {
        return None;
    }

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let record_fields = record
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let filters = endpoint
        .inputs
        .iter()
        .filter(|input| record_fields.contains(input.name.as_str()))
        .map(|input| {
            (
                input.name.clone(),
                serde_json::json!(format!("$input.{}", input.name)),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    if filters.is_empty() {
        return None;
    }

    Some(serde_json::json!({
        "kind": "record_mutation",
        "action": endpoint.id,
        "target": collection,
        "steps": [
            {
                "op": "delete_where",
                "entity": entity,
                "collection": collection,
                "where": filters
            }
        ]
    }))
}

fn sorx_runtime_approval_status_command(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    let desired_status = if endpoint.id.starts_with("reject_") {
        "rejected".to_string()
    } else if endpoint.id.starts_with("approve_") {
        "approved".to_string()
    } else {
        return None;
    };

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let status_field = record
        .fields
        .iter()
        .find(|field| matches!(field.name.as_str(), "status" | "state"))
        .map(|field| field.name.clone())?;
    let desired_status = if desired_status == "approved"
        && record
            .fields
            .iter()
            .find(|field| field.name == status_field)
            .is_some_and(|field| field.enum_values.iter().any(|value| value == "active"))
    {
        "active".to_string()
    } else {
        desired_status
    };
    let filters = sorx_runtime_identity_filters(endpoint, record, &[status_field.as_str()]);

    if filters.is_empty() {
        return None;
    }

    Some(serde_json::json!({
        "kind": "record_mutation",
        "action": endpoint.id,
        "target": collection,
        "steps": [
            {
                "op": "update_where",
                "as": "update",
                "entity": entity,
                "collection": collection,
                "where": filters,
                "set": {
                    status_field.clone(): desired_status
                }
            }
        ],
        "return": {
            "updated_count": "$steps.update.updated_count",
            "records": "$steps.update.records"
        }
    }))
}

fn sorx_runtime_side_effect_event_command(endpoint: &AgentEndpointIr) -> Option<serde_json::Value> {
    if endpoint.side_effects.is_empty() {
        return None;
    }
    if !(endpoint.id.starts_with("trigger_")
        || endpoint.id.starts_with("invoke_")
        || endpoint.id.starts_with("run_")
        || endpoint.id.contains("workflow")
        || endpoint.id.ends_with("_action"))
    {
        return None;
    }

    Some(serde_json::json!({
        "kind": "side_effect",
        "action": endpoint.id,
        "steps": [
            {
                "op": "emit_event",
                "as": "event",
                "event": format!("action.{}", endpoint.id),
                "stream": endpoint.id,
                "payload": {
                    "endpoint_id": endpoint.id,
                    "operation_id": endpoint.id,
                    "input": "$input",
                    "side_effects": endpoint.side_effects
                }
            }
        ],
        "return": {
            "event": "$steps.event",
            "side_effects": endpoint.side_effects
        }
    }))
}

fn sorx_runtime_archive_restore_command(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    let active_value = if endpoint.id.starts_with("archive_") {
        false
    } else if endpoint.id.starts_with("restore_") {
        true
    } else {
        return None;
    };

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let active_field = record
        .fields
        .iter()
        .find(|field| matches!(field.name.as_str(), "is_active" | "active" | "archived"))
        .map(|field| field.name.clone())?;
    let filters = sorx_runtime_identity_filters(endpoint, record, &[active_field.as_str()]);

    if filters.is_empty() {
        return None;
    }

    Some(serde_json::json!({
        "kind": "record_mutation",
        "action": endpoint.id,
        "target": collection,
        "steps": [
            {
                "op": "update_where",
                "as": "update",
                "entity": entity,
                "collection": collection,
                "where": filters,
                "set": {
                    active_field.clone(): active_value
                }
            }
        ],
        "return": {
            "updated_count": "$steps.update.updated_count",
            "records": "$steps.update.records"
        }
    }))
}

fn sorx_runtime_identity_filters(
    endpoint: &AgentEndpointIr,
    record: &RecordIr,
    exclude: &[&str],
) -> serde_json::Map<String, serde_json::Value> {
    let record_fields = record
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let preferred_id = format!("{}_id", snake_case_identifier(&record.name));
    let identity_inputs = endpoint
        .inputs
        .iter()
        .filter(|input| !exclude.contains(&input.name.as_str()))
        .filter(|input| record_fields.contains(input.name.as_str()))
        .filter(|input| {
            input.name == "id"
                || input.name == preferred_id
                || input.name.ends_with("_id")
                || input.name.contains("external_id")
        })
        .map(|input| {
            (
                input.name.clone(),
                serde_json::json!(format!("$input.{}", input.name)),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    if !identity_inputs.is_empty() {
        return identity_inputs;
    }

    endpoint
        .inputs
        .iter()
        .filter(|input| !exclude.contains(&input.name.as_str()))
        .filter(|input| record_fields.contains(input.name.as_str()))
        .map(|input| {
            (
                input.name.clone(),
                serde_json::json!(format!("$input.{}", input.name)),
            )
        })
        .collect()
}

fn sorx_runtime_bulk_import_command(endpoint: &AgentEndpointIr) -> Option<serde_json::Value> {
    if !(endpoint.id.starts_with("bulk_") || endpoint.id.starts_with("bulk_import_")) {
        return None;
    }
    if !endpoint.inputs.iter().any(|input| input.name == "items") {
        return None;
    }

    Some(serde_json::json!({
        "kind": "bulk_mutation",
        "action": endpoint.id,
        "steps": [
            {
                "op": "foreach",
                "as": "imported",
                "items": "$input.items",
                "do": [
                    {
                        "op": "create",
                        "entity": "$item.entity",
                        "collection": "$item.collection",
                        "input": "$item.data"
                    }
                ]
            }
        ],
        "return": {
            "imported_count": "$steps.imported.count",
            "records": "$steps.imported.records"
        }
    }))
}

fn sorx_runtime_join_waiting_list_command(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    if !(endpoint.id == "join_waiting_list" || endpoint.id.starts_with("join_waiting_list_")) {
        return None;
    }

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let record_fields = record
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();

    let generated_code_field = sorx_runtime_generated_code_field(&record_fields)?;

    let input_fields = endpoint
        .inputs
        .iter()
        .map(|input| input.name.as_str())
        .collect::<BTreeSet<_>>();
    if !input_fields.contains("lab_id") {
        return None;
    }

    let mut create_input = serde_json::Map::new();
    for field in &record.fields {
        let field_name = field.name.as_str();
        let value = match field_name {
            "id" => Some(serde_json::json!("$generated.uuid")),
            "entry_id" => Some(serde_json::json!("$generated.entry_id")),
            "lab_id" if input_fields.contains("lab_id") => Some(serde_json::json!("$input.lab_id")),
            "user_id" if input_fields.contains("user_id") => {
                Some(serde_json::json!("$input.user_id"))
            }
            "user_id" => Some(serde_json::json!("$generated.uuid")),
            field if field == generated_code_field => {
                Some(serde_json::json!(format!("$generated.{field}")))
            }
            "referrer_entry_id" if input_fields.contains("invited_by_code") => {
                Some(serde_json::json!("$steps.referrer.data.entry_id"))
            }
            "referred_count" | "referral_count" => Some(serde_json::json!(0)),
            "joined_at" | "created_at" => Some(serde_json::json!("$now")),
            _ if input_fields.contains(field_name) => {
                Some(serde_json::json!(format!("$input.{field_name}")))
            }
            _ => None,
        };
        if let Some(value) = value {
            create_input.insert(field.name.clone(), value);
        }
    }

    let mut return_value = serde_json::Map::new();
    for output in &endpoint.outputs {
        let value = match output.name.as_str() {
            "entry_id" if record_fields.contains("entry_id") => {
                Some(serde_json::json!("$steps.entry.record.data.entry_id"))
            }
            "id" if record_fields.contains("id") => Some(serde_json::json!("$steps.entry.id")),
            field if field == generated_code_field => Some(serde_json::json!(format!(
                "$steps.entry.record.data.{field}"
            ))),
            "position" | "number_in_waiting_list" | "waiting_list_size" => {
                Some(serde_json::json!("$steps.waiting_list.count"))
            }
            _ if record_fields.contains(output.name.as_str()) => Some(serde_json::json!(format!(
                "$steps.entry.record.data.{}",
                output.name
            ))),
            _ => None,
        };
        if let Some(value) = value {
            return_value.insert(output.name.clone(), value);
        }
    }

    let uniqueness = sorx_runtime_uniqueness_contract(&entity, ir);
    let idempotency_fields = if input_fields.contains("user_id") {
        vec!["lab_id", "user_id"]
    } else if input_fields.contains("email") {
        vec!["lab_id", "email"]
    } else {
        vec!["lab_id"]
    };
    let idempotency = sorx_runtime_unique_index_for_fields(&entity, &idempotency_fields, ir);

    Some(serde_json::json!({
        "kind": "record_mutation",
        "action": endpoint.id,
        "idempotency": "return_existing",
        "target": collection,
        "constraints": {
            "idempotency": idempotency.map(|index| serde_json::json!({
                "mode": "return_existing",
                "index": index.id,
                "fields": index.fields
            })),
            "unique": uniqueness
        },
        "steps": [
            {
                "op": "find_one",
                "as": "referrer",
                "entity": entity,
                "collection": collection,
                "where": {
                    "lab_id": "$input.lab_id",
                    generated_code_field.clone(): "$input.invited_by_code"
                },
                "required": true,
                "when": {
                    "present": "$input.invited_by_code"
                }
            },
            {
                "op": "create",
                "as": "entry",
                "entity": entity,
                "collection": collection,
                "input": create_input
            },
            {
                "op": "increment_where",
                "as": "referrer_increment",
                "entity": entity,
                "collection": collection,
                "where": {
                    "lab_id": "$input.lab_id",
                    generated_code_field.clone(): "$input.invited_by_code"
                },
                "increments": {
                    "referred_count": 1
                },
                "when": {
                    "all": [
                        { "present": "$input.invited_by_code" },
                        { "equals": ["$steps.entry.created", true] }
                    ]
                }
            },
            {
                "op": "query",
                "as": "waiting_list",
                "entity": entity,
                "collection": collection,
                "where": {
                    "lab_id": "$input.lab_id"
                },
                "order_by": [
                    { "field": "referred_count", "direction": "desc" },
                    { "field": "joined_at", "direction": "asc" }
                ]
            }
        ],
        "return": return_value
    }))
}

fn sorx_runtime_show_waiting_list_command(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    if endpoint.id != "show_waiting_list" {
        return None;
    }

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let record_fields = record
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    if !(record_fields.contains("lab_id")
        && record_fields.contains("referred_count")
        && record_fields.contains("joined_at"))
    {
        return None;
    }
    if !endpoint.inputs.iter().any(|input| input.name == "lab_id") {
        return None;
    }

    Some(serde_json::json!({
        "kind": "record_query",
        "action": endpoint.id,
        "target": collection,
        "steps": [
            {
                "op": "query",
                "as": "waiting_list",
                "entity": entity,
                "collection": collection,
                "where": {
                    "lab_id": "$input.lab_id"
                },
                "order_by": [
                    { "field": "referred_count", "direction": "desc" },
                    { "field": "joined_at", "direction": "asc" }
                ]
            }
        ],
        "return": {
            "entries": "$steps.waiting_list.records",
            "count": "$steps.waiting_list.count"
        }
    }))
}

fn sorx_runtime_leave_waiting_list_command(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    if endpoint.id != "leave_waiting_list" {
        return None;
    }

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let record_fields = record
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    if !(record_fields.contains("lab_id")
        && record_fields.contains("entry_id")
        && record_fields.contains("referrer_entry_id")
        && record_fields.contains("referred_count"))
    {
        return None;
    }

    let filters = endpoint
        .inputs
        .iter()
        .filter(|input| record_fields.contains(input.name.as_str()))
        .map(|input| {
            (
                input.name.clone(),
                serde_json::json!(format!("$input.{}", input.name)),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    if filters.is_empty() {
        return None;
    }

    Some(serde_json::json!({
        "kind": "record_mutation",
        "action": endpoint.id,
        "target": collection,
        "steps": [
            {
                "op": "find_one",
                "as": "leaving_entry",
                "entity": entity,
                "collection": collection,
                "where": filters.clone(),
                "required": true
            },
            {
                "op": "delete_where",
                "as": "leave",
                "entity": entity,
                "collection": collection,
                "where": filters
            },
            {
                "op": "increment_where",
                "as": "referrer_decrement",
                "entity": entity,
                "collection": collection,
                "where": {
                    "lab_id": "$input.lab_id",
                    "entry_id": "$steps.leaving_entry.data.referrer_entry_id"
                },
                "increments": {
                    "referred_count": -1
                },
                "when": {
                    "all": [
                        { "present": "$steps.leaving_entry.data.referrer_entry_id" },
                        { "equals": ["$steps.leave.deleted_count", 1] }
                    ]
                }
            }
        ],
        "return": {
            "deleted_count": "$steps.leave.deleted_count"
        }
    }))
}

fn sorx_runtime_generated_code_field(record_fields: &BTreeSet<&str>) -> Option<String> {
    ["invitation_code", "referral_code", "invite_code"]
        .iter()
        .find(|field| record_fields.contains(**field))
        .map(|field| (*field).to_string())
}

fn sorx_runtime_uniqueness_contract(entity: &str, ir: &CanonicalIr) -> Vec<serde_json::Value> {
    ir.operational_indexes
        .as_ref()
        .map(|indexes| {
            indexes
                .indexes
                .iter()
                .filter(|index| index.unique && index.record == entity)
                .map(|index| {
                    serde_json::json!({
                        "index": index.id,
                        "record": index.record,
                        "kind": index.kind,
                        "fields": index.fields
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn sorx_runtime_unique_index_for_fields<'a>(
    entity: &str,
    fields: &[&str],
    ir: &'a CanonicalIr,
) -> Option<&'a greentic_sorla_ir::OperationalIndexIr> {
    ir.operational_indexes
        .as_ref()?
        .indexes
        .iter()
        .find(|index| {
            index.unique
                && index.record == entity
                && index.fields.len() == fields.len()
                && fields
                    .iter()
                    .all(|field| index.fields.iter().any(|candidate| candidate == field))
        })
}

fn sorx_runtime_retrieve_field_command(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    if !endpoint.id.starts_with("retrieve_") {
        return None;
    }

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let record_fields = record
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let output_fields = endpoint
        .outputs
        .iter()
        .filter(|output| record_fields.contains(output.name.as_str()))
        .collect::<Vec<_>>();

    if output_fields.is_empty() {
        return None;
    }

    let filters = sorx_runtime_identity_filters(endpoint, record, &[]);
    if filters.is_empty() {
        return None;
    }

    let mut return_value = serde_json::Map::new();
    for output in output_fields {
        return_value.insert(
            output.name.clone(),
            serde_json::json!(format!("$steps.entry.data.{}", output.name)),
        );
    }

    Some(serde_json::json!({
        "kind": "record_lookup",
        "action": endpoint.id,
        "target": collection,
        "steps": [
            {
                "op": "find_one",
                "as": "entry",
                "entity": entity,
                "collection": collection,
                "where": filters,
                "required": true
            }
        ],
        "return": return_value
    }))
}

fn sorx_runtime_generate_field_command(
    endpoint: &AgentEndpointIr,
    ir: &CanonicalIr,
) -> Option<serde_json::Value> {
    if !endpoint.id.starts_with("generate_") {
        return None;
    }

    let entity = sorx_runtime_entity(endpoint, ir);
    let collection = sorx_runtime_collection(endpoint, ir);
    let record = ir.records.iter().find(|record| record.name == entity)?;
    let record_fields = record
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let generated_field = endpoint
        .outputs
        .iter()
        .find(|output| {
            record_fields.contains(output.name.as_str())
                && (endpoint.id.contains(&output.name)
                    || output.name.ends_with("_code")
                    || output.name.ends_with("_token")
                    || output.name.ends_with("_key"))
        })
        .map(|output| output.name.clone())
        .or_else(|| inferred_generated_field_from_endpoint(endpoint))?;
    let input_id = sorx_runtime_record_id_input(endpoint, record)?;
    let input_path = format!("$input.{input_id}");
    let existing_value_path = format!("$steps.entry.data.{generated_field}");
    let generated_value_path = format!("$generated.{generated_field}");
    let updated_value_path = format!("$steps.update.records.0.data.{generated_field}");
    let where_field = if record_fields.contains(input_id.as_str()) {
        input_id.clone()
    } else {
        "id".to_string()
    };

    Some(serde_json::json!({
        "kind": "record_mutation",
        "action": endpoint.id,
        "target": collection,
        "steps": [
            {
                "op": "find_one",
                "as": "entry",
                "entity": entity,
                "collection": collection,
                "where": {
                    where_field.clone(): input_path
                },
                "required": true
            },
            {
                "op": "update_where",
                "as": "update",
                "entity": entity,
                "collection": collection,
                "where": {
                    where_field: input_path
                },
                "set": {
                    generated_field.clone(): {
                        "coalesce": [
                            existing_value_path,
                            generated_value_path
                        ]
                    }
                }
            }
        ],
        "return": {
            input_id: input_path,
            generated_field: updated_value_path
        }
    }))
}

fn inferred_generated_field_from_endpoint(endpoint: &AgentEndpointIr) -> Option<String> {
    let rest = endpoint.id.strip_prefix("generate_")?;
    let field = rest
        .strip_suffix("_code")
        .map(|prefix| format!("{prefix}_code"))
        .or_else(|| {
            rest.strip_suffix("_token")
                .map(|prefix| format!("{prefix}_token"))
        })
        .or_else(|| {
            rest.strip_suffix("_key")
                .map(|prefix| format!("{prefix}_key"))
        })
        .unwrap_or_else(|| format!("{rest}_value"));
    Some(field)
}

fn sorx_runtime_record_id_input(endpoint: &AgentEndpointIr, record: &RecordIr) -> Option<String> {
    let record_id = format!("{}_id", snake_case_identifier(&record.name));
    endpoint
        .inputs
        .iter()
        .find(|input| input.name == "id")
        .or_else(|| endpoint.inputs.iter().find(|input| input.name == record_id))
        .or_else(|| {
            endpoint
                .inputs
                .iter()
                .find(|input| input.name.ends_with("_id"))
        })
        .map(|input| input.name.clone())
}

fn sorx_runtime_id_input(endpoint: &AgentEndpointIr, entity: &str) -> Option<String> {
    let entity_id = format!("{}_id", snake_case_identifier(entity));
    endpoint
        .inputs
        .iter()
        .find(|input| input.name == "id")
        .or_else(|| endpoint.inputs.iter().find(|input| input.name == entity_id))
        .map(|input| input.name.clone())
}

fn sorx_runtime_entity(endpoint: &AgentEndpointIr, ir: &CanonicalIr) -> String {
    if endpoint_uses_dynamic_record_selector(endpoint) {
        return "Record".to_string();
    }

    if let Some(record) = link_endpoint_record(endpoint, ir) {
        return record.name.clone();
    }

    if let Some(emit) = &endpoint.emits
        && let Some(event) = ir.events.iter().find(|event| event.name == emit.event)
    {
        return event.record.clone();
    }

    if let Some(record) = endpoint.outputs.iter().find_map(|output| {
        let output_name = output.name.trim_end_matches('s');
        ir.records
            .iter()
            .find(|record| output_name.contains(&snake_case_identifier(&record.name)))
    }) {
        return record.name.clone();
    }

    if let Some(record) = best_scored_record_for_endpoint(endpoint, ir) {
        return record.name.clone();
    }

    ir.records
        .first()
        .map(|record| record.name.clone())
        .unwrap_or_else(|| "Record".to_string())
}

fn link_endpoint_record<'a>(
    endpoint: &AgentEndpointIr,
    ir: &'a CanonicalIr,
) -> Option<&'a RecordIr> {
    if !endpoint.id.starts_with("link_") {
        return None;
    }
    let endpoint_fields = endpoint
        .inputs
        .iter()
        .map(|input| input.name.as_str())
        .collect::<BTreeSet<_>>();
    if endpoint_fields.is_empty() {
        return None;
    }
    ir.records
        .iter()
        .filter(|record| record.fields.len() == endpoint_fields.len())
        .find(|record| {
            record
                .fields
                .iter()
                .all(|field| endpoint_fields.contains(field.name.as_str()))
        })
}

fn endpoint_entity_haystack(endpoint: &AgentEndpointIr) -> String {
    let mut parts = vec![endpoint.id.clone()];
    parts.extend(endpoint.inputs.iter().map(|input| input.name.clone()));
    parts.extend(endpoint.outputs.iter().map(|output| output.name.clone()));
    parts.join(" ")
}

fn best_scored_record_for_endpoint<'a>(
    endpoint: &AgentEndpointIr,
    ir: &'a CanonicalIr,
) -> Option<&'a RecordIr> {
    let haystack = endpoint_entity_haystack(endpoint);
    let endpoint_fields = endpoint
        .inputs
        .iter()
        .map(|input| input.name.as_str())
        .chain(endpoint.outputs.iter().map(|output| output.name.as_str()))
        .collect::<BTreeSet<_>>();
    let endpoint_id_tokens = endpoint
        .id
        .split(|ch: char| !(ch.is_ascii_alphanumeric()))
        .filter(|token| !token.is_empty())
        .collect::<BTreeSet<_>>();

    ir.records
        .iter()
        .filter_map(|record| {
            let record_name = snake_case_identifier(&record.name);
            let mut score = 0usize;
            let exact_record_match = if record_name.contains('_') {
                haystack.contains(&record_name)
            } else {
                endpoint_id_tokens.contains(record_name.as_str())
            };
            if exact_record_match {
                score += 10;
            }
            for token in record_name.split('_').filter(|token| token.len() > 2) {
                if haystack.contains(token) {
                    score += 2;
                }
            }
            for field in &record.fields {
                if endpoint_fields.contains(field.name.as_str()) {
                    score += 1;
                }
            }
            (score > 0).then_some((score, record))
        })
        .max_by_key(|(score, record)| (*score, record.name.len()))
        .map(|(_, record)| record)
}

fn sorx_runtime_collection(endpoint: &AgentEndpointIr, ir: &CanonicalIr) -> String {
    if endpoint_uses_dynamic_record_selector(endpoint) {
        return "records".to_string();
    }
    pluralize_snake(&snake_case_identifier(&sorx_runtime_entity(endpoint, ir)))
}

fn endpoint_uses_dynamic_record_selector(endpoint: &AgentEndpointIr) -> bool {
    endpoint
        .execution
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .is_some_and(|execution| execution.contains_key("record_selector"))
}

fn agent_gateway_record_hierarchy(ir: &CanonicalIr) -> Vec<AgentGatewayRecordHierarchyRef> {
    let mut relationship_by_child_field = BTreeMap::<(String, String), String>::new();
    if let Some(ontology) = &ir.ontology {
        for relationship in &ontology.relationships {
            let Some(backing) = &relationship.backing else {
                continue;
            };
            let Some(from_field) = &backing.from_field else {
                continue;
            };
            relationship_by_child_field.insert(
                (backing.record.clone(), from_field.clone()),
                relationship.id.clone(),
            );
        }
    }

    let mut hierarchy = ir
        .records
        .iter()
        .map(|record| {
            let mut parents = record
                .fields
                .iter()
                .filter_map(|field| {
                    field.references.as_ref().map(|reference| {
                        let relationship = relationship_by_child_field
                            .get(&(record.name.clone(), field.name.clone()))
                            .cloned();
                        AgentGatewayRecordParentRef {
                            record: reference.record.clone(),
                            field: field.name.clone(),
                            relationship,
                        }
                    })
                })
                .collect::<Vec<_>>();
            parents.sort_by(|left, right| {
                left.record
                    .cmp(&right.record)
                    .then_with(|| left.field.cmp(&right.field))
            });
            parents
                .dedup_by(|left, right| left.record == right.record && left.field == right.field);
            AgentGatewayRecordHierarchyRef {
                record: record.name.clone(),
                main: parents.is_empty(),
                parents,
            }
        })
        .collect::<Vec<_>>();
    hierarchy.sort_by(|left, right| left.record.cmp(&right.record));
    hierarchy
}

fn snake_case_identifier(value: &str) -> String {
    let mut out = String::new();
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == ' ' {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    out
}

fn pluralize_snake(value: &str) -> String {
    if value.ends_with('y') {
        format!("{}ies", value.trim_end_matches('y'))
    } else if value.ends_with('s') {
        value.to_string()
    } else {
        format!("{value}s")
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
            let command = sorx_runtime_command_spec(endpoint, ir);
            let operation = sorx_runtime_operation(endpoint, command.as_ref());
            let method = sorx_runtime_method(endpoint, command.as_ref());
            let path = sorx_runtime_path(endpoint, ir);
            serde_json::json!({
                "operationId": format!("agent_{}", endpoint.id),
                "x-greentic-agent": {
                    "endpoint_id": endpoint.id,
                    "intent": endpoint.intent,
                    "risk": agent_endpoint_risk_label(&endpoint.risk),
                    "approval": agent_endpoint_approval_label(&endpoint.approval),
                    "side_effects": endpoint.side_effects,
                    "inputs": endpoint.inputs.iter().map(openapi_input_value).collect::<Vec<_>>(),
                    "outputs": endpoint.outputs.iter().map(output_value).collect::<Vec<_>>(),
                    "runtime": {
                        "operation": operation,
                        "method": method,
                        "path": path,
                        "input_transport": agent_endpoint_input_transport(method)
                    }
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
            let command = sorx_runtime_command_spec(endpoint, ir);
            let method = sorx_runtime_method(endpoint, command.as_ref());
            serde_json::json!({
                "workflowId": endpoint.id,
                "summary": endpoint.title,
                "description": endpoint.intent,
                "inputs": object_schema_value(&endpoint.inputs),
                "steps": [
                    {
                        "stepId": format!("request_{}", endpoint.id),
                        "description": format!("Request downstream Greentic execution for {}.", endpoint.id),
                        "operationId": format!("agent_{}", endpoint.id),
                        "x-greentic-agent": {
                            "endpoint_id": endpoint.id,
                            "method": method,
                            "path": sorx_runtime_path(endpoint, ir),
                            "input_transport": agent_endpoint_input_transport(method)
                        }
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
            let command = sorx_runtime_command_spec(endpoint, ir);
            let method = sorx_runtime_method(endpoint, command.as_ref());
            serde_json::json!({
                "name": endpoint.id,
                "endpoint_id": endpoint.id,
                "operation_id": endpoint.id,
                "title": endpoint.title,
                "description": endpoint.intent,
                "input_schema": object_schema_value(&endpoint.inputs),
                "output_schema": output_object_schema_value(&endpoint.outputs),
                "inputSchema": object_schema_value(&endpoint.inputs),
                "annotations": {
                    "risk": agent_endpoint_risk_label(&endpoint.risk),
                    "approval": agent_endpoint_approval_label(&endpoint.approval),
                    "side_effects": endpoint.side_effects,
                    "method": method,
                    "path": sorx_runtime_path(endpoint, ir),
                    "input_transport": agent_endpoint_input_transport(method)
                }
            })
        })
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&serde_json::json!({
        "schema": SORX_MCP_TOOLS_SCHEMA,
        "tools": tools
    }))
    .expect("MCP tools handoff should serialize")
}

fn sorx_runtime_validation_suite_json(ir: &CanonicalIr) -> serde_json::Value {
    let mut tests = vec![
        serde_json::json!({
            "id": "doctor.pack.valid",
            "kind": "doctor",
            "level": "required"
        }),
        serde_json::json!({
            "id": "routes.generated",
            "kind": "route_generation",
            "level": "required"
        }),
        serde_json::json!({
            "id": "agent-gateway.present",
            "kind": "artifact_exists",
            "level": "required",
            "path": format!("assets/sorla/{AGENT_GATEWAY_HANDOFF_FILENAME}")
        }),
    ];

    if ir
        .agent_endpoints
        .iter()
        .any(|endpoint| endpoint.agent_visibility.mcp)
    {
        tests.push(serde_json::json!({
            "id": "mcp-tools.present",
            "kind": "artifact_exists",
            "level": "recommended",
            "path": format!("assets/sorla/{MCP_TOOLS_FILENAME}")
        }));
    }

    serde_json::json!({
        "schema": SORX_VALIDATION_SUITE_SCHEMA,
        "suite_id": format!("{}-runtime", ir.package.name),
        "pack_name": ir.package.name,
        "pack_version": ir.package.version,
        "gates": {
            "required_for_private_activation": true,
            "required_for_public_exposure": true,
            "minimum_pass_level": "required"
        },
        "tests": tests
    })
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

fn agent_endpoint_input_transport(method: &str) -> &'static str {
    if method == "GET" { "none" } else { "body" }
}

pub fn agent_endpoint_contract_warnings(ir: &CanonicalIr) -> Vec<AgentEndpointContractWarning> {
    let mut warnings = Vec::new();
    for endpoint in &ir.agent_endpoints {
        let command = sorx_runtime_command_spec(endpoint, ir);
        let operation = sorx_runtime_operation(endpoint, command.as_ref());

        if !endpoint.outputs.is_empty()
            && !declared_outputs_are_explicitly_returned(endpoint, command.as_ref())
        {
            let outputs = endpoint
                .outputs
                .iter()
                .map(|output| output.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            warnings.push(AgentEndpointContractWarning {
                endpoint_id: endpoint.id.clone(),
                code: "sorla.agent_endpoint.output_contract".to_string(),
                message: format!(
                    "agent endpoint `{}` declares output fields `{outputs}`, but the generated `{operation}` backing has no explicit return mapping for them",
                    endpoint.id
                ),
            });
        }

        if command.is_none() && generic_backing_underspecifies_intent(endpoint) {
            warnings.push(AgentEndpointContractWarning {
                endpoint_id: endpoint.id.clone(),
                code: "sorla.agent_endpoint.generic_backing".to_string(),
                message: format!(
                    "agent endpoint `{}` describes domain behavior that generic `{operation}` backing cannot guarantee; add explicit executable backing or simplify the claim",
                    endpoint.id
                ),
            });
        }
    }
    warnings
}

fn declared_outputs_are_explicitly_returned(
    endpoint: &AgentEndpointIr,
    command: Option<&serde_json::Value>,
) -> bool {
    let Some(command) = command else {
        return false;
    };
    let Some(return_map) = command.get("return").and_then(serde_json::Value::as_object) else {
        return endpoint.outputs.is_empty();
    };
    endpoint
        .outputs
        .iter()
        .all(|output| return_map.contains_key(&output.name))
}

fn generic_backing_underspecifies_intent(endpoint: &AgentEndpointIr) -> bool {
    let text = format!(
        "{} {} {}",
        endpoint.title,
        endpoint.intent,
        endpoint.description.as_deref().unwrap_or("")
    )
    .to_ascii_lowercase();
    [
        "ordered", "sorted", "position", "rank", "ranking", "referral", "unique", "current",
    ]
    .iter()
    .any(|term| text.contains(term))
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

    fn metrics_fixture_yaml() -> &'static str {
        r#"
package:
  name: metrics-pack-demo
  version: 0.1.0
records:
  - name: payment
    source: native
    fields:
      - name: amount
        type: decimal
      - name: paid_at
        type: datetime
events:
  - name: payment_changed
    record: payment
projections:
  - name: payment_list
    record: payment
    source_event: payment_changed
metrics:
  - name: monthly_revenue
    i18n_key: examples.metrics_pack_demo.metrics.monthly_revenue
    label: Monthly Revenue
    source:
      kind: record
      name: payment
    measure:
      aggregate: sum
      field: amount
    time:
      field: paid_at
      grain: month
    unit: GBP
"#
        .trim_start()
    }

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
            "This manifest includes SORX runtime route metadata plus SoRLa handoff context."
        ));
        assert!(first.agent_tools_json.contains("storage"));
        assert_eq!(
            first.handoff_manifest().provider_repo,
            "greentic-sorla-providers"
        );
    }

    #[test]
    fn builds_metrics_handoff_artifact_from_yaml() {
        let artifacts =
            build_handoff_artifacts_from_yaml(metrics_fixture_yaml()).expect("fixture builds");
        assert_eq!(artifacts.ir.metrics.len(), 1);
        assert!(
            artifacts
                .handoff_manifest()
                .artifact_references
                .contains(&METRICS_FILENAME.to_string())
        );
        let metrics_json = artifacts
            .metrics_json
            .as_ref()
            .expect("metrics artifact emitted");
        let document: MetricsArtifactDocument =
            serde_json::from_str(metrics_json).expect("metrics JSON decodes");
        assert_eq!(document.schema, METRICS_SCHEMA);
        assert_eq!(document.package.name, "metrics-pack-demo");
        assert_eq!(document.package.ir_hash, artifacts.canonical_hash);
        assert_eq!(document.metrics[0].name, "monthly_revenue");
        assert_eq!(
            document.metrics[0].i18n_key.as_deref(),
            Some("examples.metrics_pack_demo.metrics.monthly_revenue")
        );

        let rebuilt =
            build_handoff_artifacts_from_yaml(metrics_fixture_yaml()).expect("fixture rebuilds");
        assert_eq!(artifacts.metrics_json, rebuilt.metrics_json);
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
            DESIGNER_NODE_TYPES_FILENAME,
            AGENT_ENDPOINT_ACTION_CATALOG_FILENAME,
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
        let designer_node_types: DesignerNodeTypesDocument =
            serde_json::from_str(&built.designer_node_types_json)
                .expect("designer node types should parse");
        assert_eq!(designer_node_types.schema, DESIGNER_NODE_TYPES_SCHEMA);
        assert_eq!(
            designer_node_types.node_types.len(),
            ir.agent_endpoints.len()
        );
        assert_eq!(
            designer_node_types.node_types[0]
                .metadata
                .endpoint
                .contract_hash,
            format!("sha256:{}", canonical_hash_hex(&ir))
        );
        assert_eq!(
            designer_node_types.node_types[0].binding.operation,
            DEFAULT_DESIGNER_COMPONENT_OPERATION
        );
        let email_field = designer_node_types.node_types[0]
            .ui
            .fields
            .iter()
            .find(|field| field.name == "email")
            .expect("email field should be emitted");
        assert_eq!(email_field.label, "Email");
        assert_eq!(email_field.widget, "text");
        assert!(
            designer_node_types.node_types[0]
                .ui
                .aliases
                .contains(&"create customer contact".to_string())
        );
        let action_catalog: AgentEndpointActionCatalogDocument =
            serde_json::from_str(&built.agent_endpoint_action_catalog_json)
                .expect("action catalog should parse");
        assert_eq!(action_catalog.schema, AGENT_ENDPOINT_ACTION_CATALOG_SCHEMA);
        assert_eq!(action_catalog.actions.len(), ir.agent_endpoints.len());
        assert_eq!(
            action_catalog.actions[0].endpoint_ref.contract_hash,
            format!("sha256:{}", canonical_hash_hex(&ir))
        );
        assert!(
            built
                .package_manifest
                .artifact_references
                .contains(&DESIGNER_NODE_TYPES_FILENAME.to_string())
        );
        assert!(
            built
                .package_manifest
                .artifact_references
                .contains(&AGENT_ENDPOINT_ACTION_CATALOG_FILENAME.to_string())
        );

        let actual_gateway_value: serde_json::Value = serde_json::from_str(
            &serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest JSON should parse");
        assert_eq!(actual_gateway_value["schema"], SORX_AGENT_GATEWAY_SCHEMA);
        assert_eq!(
            actual_gateway_value["endpoints"][0]["endpoint_id"],
            "create_customer_contact"
        );
        assert_eq!(actual_gateway_value["endpoints"][0]["operation"], "create");
        assert_eq!(actual_gateway_value["endpoints"][0]["method"], "POST");
        assert_eq!(
            actual_gateway_value["endpoints"][0]["path"],
            "/v1/agent/contacts/create"
        );
        assert_eq!(actual_gateway_value["endpoints"][0]["entity"], "Contact");
        assert_eq!(
            actual_gateway_value["endpoints"][0]["collection"],
            "contacts"
        );

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
    fn dynamic_record_selector_endpoints_are_record_wide() {
        let parsed = parse_package(
            r#"
package:
  name: dynamic-record-admin
  version: 0.1.0
actions:
  - name: AdminUpdateRecord
records:
  - name: Landlord
    fields:
      - name: id
        type: uuid
      - name: full_name
        type: string
events:
  - name: RecordUpdated
    record: Landlord
agent_endpoints:
  - id: admin_update_record
    title: Admin update record
    intent: Update any selected record.
    inputs:
      - name: record_name
        type: string
        required: true
      - name: record_id
        type: uuid
        required: true
      - name: patch_json
        type: string
        required: true
    outputs:
      - name: record_id
        type: uuid
    side_effects:
      - records.update
    risk: high
    approval: required
    backing:
      actions:
        - AdminUpdateRecord
      events:
        - RecordUpdated
    execution:
      kind: record-update
      record_selector: "$input.record_name"
"#,
        )
        .expect("dynamic record endpoint package should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = manifest
            .endpoints
            .iter()
            .find(|endpoint| endpoint.id == "admin_update_record")
            .expect("admin endpoint exists");

        assert_eq!(endpoint.entity, "Record");
        assert_eq!(endpoint.collection, "records");
        assert_eq!(endpoint.path, "/v1/agent/records/admin_update_record");
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
        assert!(first.assets.contains(&DESIGNER_NODE_TYPES_PATH.to_string()));
        assert!(
            first
                .assets
                .contains(&AGENT_ENDPOINT_ACTION_CATALOG_PATH.to_string())
        );
        assert!(first.assets.contains(&GREENTIC_STACK_PACK_PATH.to_string()));
        assert!(
            first
                .assets
                .contains(&GREENTIC_CAPABILITIES_PATH.to_string())
        );
        assert!(
            first
                .assets
                .contains(&GREENTIC_SECRET_REQUIREMENTS_PATH.to_string())
        );
        assert!(
            first
                .assets
                .contains(&GREENTIC_CALL_REQUEST_SCHEMA_PATH.to_string())
        );
        assert!(
            first
                .assets
                .contains(&GREENTIC_CALL_RESPONSE_SCHEMA_PATH.to_string())
        );
        assert!(first.assets.contains(&GREENTIC_ARTIFACTS_PATH.to_string()));
        assert!(
            first
                .assets
                .contains(&GREENTIC_ADMIN_SURFACES_PATH.to_string())
        );

        let inspection = inspect_sorla_gtpack(&first_out).expect("inspect pack");
        assert_eq!(inspection.extension, SORX_RUNTIME_EXTENSION_ID);
        assert_eq!(inspection.sorla_package_name, "landlord-tenant-sor");
        let stack_pack = inspection
            .stack_pack
            .as_ref()
            .expect("inspect should summarize generic stack-pack metadata");
        assert_eq!(stack_pack.schema, GREENTIC_STACK_PACK_SCHEMA);
        assert_eq!(stack_pack.stack_id, "landlord-tenant-sor");
        assert_eq!(stack_pack.stack_kind, "application-stack");
        assert_eq!(stack_pack.offer_count, 1);
        assert_eq!(stack_pack.requirement_count, 6);
        assert_eq!(stack_pack.route_count, 1);
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
        let designer = inspection
            .designer_node_types
            .as_ref()
            .expect("inspect should summarize designer node types");
        assert_eq!(designer.schema, DESIGNER_NODE_TYPES_SCHEMA);
        assert!(designer.count >= 1);
        let catalog = inspection
            .agent_endpoint_action_catalog
            .as_ref()
            .expect("inspect should summarize action catalog");
        assert_eq!(catalog.schema, AGENT_ENDPOINT_ACTION_CATALOG_SCHEMA);
        assert!(catalog.count >= 1);
        doctor_sorla_gtpack(&first_out).expect("doctor accepts pack");

        let mut archive =
            ZipArchive::new(fs::File::open(&first_out).expect("open pack")).expect("read pack");
        let gateway: serde_json::Value = serde_json::from_str(
            &zip_text(
                &mut archive,
                &format!("assets/sorla/{AGENT_GATEWAY_HANDOFF_FILENAME}"),
            )
            .expect("agent gateway"),
        )
        .expect("agent gateway JSON decodes");
        let hierarchy = gateway["record_hierarchy"]
            .as_array()
            .expect("record hierarchy array");
        let unit_hierarchy = hierarchy
            .iter()
            .find(|entry| entry["record"] == "Unit")
            .expect("Unit hierarchy entry");
        assert_eq!(unit_hierarchy["main"], false);
        assert!(
            unit_hierarchy["parents"]
                .as_array()
                .unwrap()
                .iter()
                .any(|parent| parent["record"] == "Property" && parent["field"] == "property_id")
        );
        let landlord_hierarchy = hierarchy
            .iter()
            .find(|entry| entry["record"] == "Landlord")
            .expect("Landlord hierarchy entry");
        assert_eq!(landlord_hierarchy["main"], true);
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
                .and_then(|sorx| sorx.get("validation_suite"))
                .and_then(serde_json::Value::as_str),
            Some(SORX_VALIDATION_SUITE_PATH)
        );
        assert_eq!(
            pack_manifest
                .extension
                .get("sorx")
                .and_then(|sorx| sorx.get("validation_suite_cbor"))
                .and_then(serde_json::Value::as_str),
            Some(SORX_VALIDATION_SUITE_CBOR_PATH)
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
        assert_eq!(
            pack_manifest
                .extension
                .get("greentic")
                .and_then(|greentic| greentic.get("stack_pack"))
                .and_then(serde_json::Value::as_str),
            Some(GREENTIC_STACK_PACK_PATH)
        );
        assert_eq!(
            pack_manifest
                .extension
                .get("greentic")
                .and_then(|greentic| greentic.get("capabilities"))
                .and_then(serde_json::Value::as_str),
            Some(GREENTIC_CAPABILITIES_PATH)
        );
        let stack_pack_doc: GreenticStackPackDocument = serde_json::from_slice(
            &zip_bytes(&mut archive, GREENTIC_STACK_PACK_PATH).expect("stack-pack"),
        )
        .expect("stack-pack JSON decodes");
        assert_eq!(
            stack_pack_doc.offers[0].capability,
            CAP_STACK_APPLICATION_V1
        );
        assert_eq!(stack_pack_doc.requires[0].capability, CAP_RUNTIME_HOST_V1);
        let capabilities: GreenticPackCapabilitySection = serde_json::from_slice(
            &zip_bytes(&mut archive, GREENTIC_CAPABILITIES_PATH).expect("capabilities"),
        )
        .expect("capabilities JSON decodes");
        assert_eq!(capabilities.schema_version, 1);
        assert_eq!(capabilities.declaration.offers, stack_pack_doc.offers);
        assert_eq!(capabilities.declaration.requires, stack_pack_doc.requires);
        assert_eq!(
            stack_pack_doc.routes[0].request_schema_ref,
            GREENTIC_CALL_REQUEST_SCHEMA_PATH
        );
        let request_schema: serde_json::Value = serde_json::from_slice(
            &zip_bytes(&mut archive, GREENTIC_CALL_REQUEST_SCHEMA_PATH)
                .expect("call request schema"),
        )
        .expect("call request schema decodes");
        assert_eq!(request_schema["schema"], "greentic.stack.call.request.v1");
        let artifacts: serde_json::Value = serde_json::from_slice(
            &zip_bytes(&mut archive, GREENTIC_ARTIFACTS_PATH).expect("artifacts"),
        )
        .expect("artifacts JSON decodes");
        assert_eq!(artifacts["schema"], "greentic.stack.artifacts.v1");
        assert!(
            artifacts["artifacts"]
                .as_array()
                .expect("artifacts array")
                .iter()
                .any(|artifact| artifact["path"] == DESIGNER_NODE_TYPES_PATH)
        );
        let admin_surfaces: serde_json::Value = serde_json::from_slice(
            &zip_bytes(&mut archive, GREENTIC_ADMIN_SURFACES_PATH).expect("admin surfaces"),
        )
        .expect("admin surfaces JSON decodes");
        assert_eq!(admin_surfaces["schema"], "greentic.stack.admin-surfaces.v1");
        assert_eq!(
            admin_surfaces["surfaces"]
                .as_array()
                .expect("surfaces array")
                .len(),
            2
        );
        assert!(
            pack_manifest
                .assets
                .contains(&SORX_VALIDATION_MANIFEST_PATH.to_string())
        );
        assert!(
            pack_manifest
                .assets
                .contains(&SORX_VALIDATION_SUITE_PATH.to_string())
        );
        assert!(
            pack_manifest
                .assets
                .contains(&SORX_VALIDATION_SUITE_CBOR_PATH.to_string())
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
        assert!(
            !pack_manifest
                .assets
                .contains(&GREENTIC_STACK_PACK_PATH.to_string())
        );
        assert!(
            !pack_manifest
                .assets
                .contains(&GREENTIC_CALL_REQUEST_SCHEMA_PATH.to_string())
        );
        assert!(
            !pack_manifest
                .assets
                .contains(&GREENTIC_ADMIN_SURFACES_PATH.to_string())
        );
    }

    #[test]
    fn gtpack_embeds_adjacent_i18n_catalogs() {
        let temp = tempdir().expect("tempdir");
        let input = temp.path().join("sorla.yaml");
        fs::write(
            &input,
            r#"
package:
  name: i18n-demo
  version: 0.1.0
  i18n_key: examples.i18n_demo.package
records:
  - name: customer
    i18n_key: examples.i18n_demo.records.customer
    fields:
      - name: id
        i18n_key: examples.i18n_demo.records.customer.fields.id
        type: uuid
"#
            .trim_start(),
        )
        .expect("write input");
        let i18n_dir = temp.path().join("i18n");
        fs::create_dir(&i18n_dir).expect("create i18n dir");
        fs::write(
            i18n_dir.join("en.json"),
            r#"{"examples.i18n_demo.package.label":"I18n demo"}"#,
        )
        .expect("write English catalog");
        fs::write(
            i18n_dir.join("es.json"),
            r#"{"examples.i18n_demo.package.label":"Demo i18n"}"#,
        )
        .expect("write Spanish catalog");
        fs::write(i18n_dir.join("notes.txt"), "not packed").expect("write ignored file");

        let out = temp.path().join("i18n.gtpack");
        let summary = build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: input,
            name: "i18n-demo".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        assert!(
            summary
                .assets
                .contains(&"assets/sorla/i18n/en.json".to_string())
        );
        assert!(
            summary
                .assets
                .contains(&"assets/sorla/i18n/es.json".to_string())
        );
        assert!(
            !summary
                .assets
                .contains(&"assets/sorla/i18n/notes.txt".to_string())
        );

        let mut archive =
            ZipArchive::new(fs::File::open(&out).expect("open pack")).expect("read pack");
        let en_json =
            zip_text(&mut archive, "assets/sorla/i18n/en.json").expect("read English catalog");
        assert!(en_json.contains("I18n demo"));
        let pack_manifest: SorlaPackManifest = ciborium::de::from_reader(Cursor::new(
            zip_bytes(&mut archive, "pack.cbor").expect("pack.cbor"),
        ))
        .expect("pack manifest decodes");
        let locales = pack_manifest.extension["sorla"]["i18n"]["locales"]
            .as_array()
            .expect("i18n locales listed");
        assert!(locales.iter().any(|locale| {
            locale["locale"] == "en" && locale["json"] == "assets/sorla/i18n/en.json"
        }));
        assert!(locales.iter().any(|locale| {
            locale["locale"] == "es" && locale["json"] == "assets/sorla/i18n/es.json"
        }));
    }

    #[test]
    fn gtpack_inspection_and_doctor_validate_metrics_artifact() {
        let temp = tempdir().expect("tempdir");
        let input = temp.path().join("metrics.sorla.yaml");
        fs::write(&input, metrics_fixture_yaml()).expect("write input");
        let out = temp.path().join("metrics.gtpack");

        let summary = build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: input,
            name: "metrics-pack-demo".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");
        assert!(summary.assets.contains(&METRICS_PATH.to_string()));

        let inspection = inspect_sorla_gtpack(&out).expect("pack inspects");
        let metrics = inspection.metrics.as_ref().expect("metrics inspected");
        assert_eq!(metrics.schema, METRICS_SCHEMA);
        assert_eq!(metrics.names, vec!["monthly_revenue".to_string()]);
        assert_eq!(inspection.optional_artifacts.get(METRICS_PATH), Some(&true));
        let doctor = doctor_sorla_gtpack(&out).expect("doctor accepts metrics pack");
        assert!(doctor.checked_assets.contains(&METRICS_PATH.to_string()));

        let mut archive =
            ZipArchive::new(fs::File::open(&out).expect("open pack")).expect("read pack");
        let pack_manifest: SorlaPackManifest = ciborium::de::from_reader(Cursor::new(
            zip_bytes(&mut archive, "pack.cbor").expect("pack.cbor"),
        ))
        .expect("pack manifest decodes");
        assert_eq!(
            pack_manifest
                .extension
                .get("sorla")
                .and_then(|sorla| sorla.get("metrics"))
                .and_then(|metrics| metrics.get("json"))
                .and_then(serde_json::Value::as_str),
            Some(METRICS_PATH)
        );
        drop(archive);

        rewrite_gtpack(&out, |name, _bytes| name != METRICS_PATH);
        let err = doctor_sorla_gtpack(&out).expect_err("doctor rejects missing metrics asset");
        assert!(err.contains("metrics"));
    }

    #[test]
    fn designer_node_metadata_handles_labels_widgets_and_enums() {
        let yaml = r#"
package:
  name: designer-metadata-demo
  version: 0.1.0
records: []
agent_endpoints:
  - id: review_claim
    title: Review claim
    intent: Review a submitted claim before approval.
    description: Dispatches a locked claim review action.
    inputs:
      - name: claim_id
        type: string
        required: true
      - name: approved
        type: boolean
        required: true
      - name: priority
        type: string
        enum_values:
          - low
          - high
    outputs:
      - name: review_id
        type: string
    side_effects:
      - event.ClaimReviewed
    risk: medium
    approval: required
"#;
        let built = build_artifacts_from_yaml(yaml).expect("artifacts build");
        let document: DesignerNodeTypesDocument =
            serde_json::from_str(&built.designer_node_types_json)
                .expect("designer node types should parse");
        let node_type = document
            .node_types
            .iter()
            .find(|node_type| node_type.id == "sorla.agent-endpoint.review_claim")
            .expect("node type should be emitted");

        assert_eq!(node_type.label, "Review claim");
        assert_eq!(
            node_type.metadata.intent,
            "Review a submitted claim before approval."
        );
        assert!(node_type.ui.aliases.contains(&"review claim".to_string()));
        let fields = node_type
            .ui
            .fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    field.label.as_str(),
                    field.widget.as_str(),
                )
            })
            .collect::<Vec<_>>();
        assert!(fields.contains(&("claim_id", "Claim Id", "text")));
        assert!(fields.contains(&("approved", "Approved", "checkbox")));
        assert!(fields.contains(&("priority", "Priority", "select")));
        assert_eq!(
            node_type.input_schema["properties"]["priority"]["enum"],
            serde_json::json!(["high", "low"])
        );
    }

    #[test]
    fn gtpack_doctor_rejects_malformed_designer_node_type_metadata() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        rewrite_gtpack(&out, |path, bytes| {
            if path == DESIGNER_NODE_TYPES_PATH {
                let mut document: DesignerNodeTypesDocument =
                    serde_json::from_slice(bytes).expect("designer node types parse");
                document.node_types[0].metadata.endpoint.contract_hash = "sha256:BAD".to_string();
                *bytes = serde_json::to_vec_pretty(&document).expect("document serializes");
            }
            true
        });

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("invalid contract_hash"));
    }

    #[test]
    fn gtpack_doctor_rejects_tampered_action_catalog_metadata() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        rewrite_gtpack(&out, |path, bytes| {
            if path == AGENT_ENDPOINT_ACTION_CATALOG_PATH {
                let mut document: AgentEndpointActionCatalogDocument =
                    serde_json::from_slice(bytes).expect("action catalog parses");
                document.actions[0].endpoint_ref.contract_hash = "sha256:BAD".to_string();
                *bytes = serde_json::to_vec_pretty(&document).expect("document serializes");
            }
            true
        });

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("endpoint_ref does not match model metadata"));
    }

    #[test]
    fn gtpack_doctor_rejects_malformed_generic_stack_pack_metadata() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        rewrite_gtpack(&out, |path, bytes| {
            if path == GREENTIC_STACK_PACK_PATH {
                let mut document: GreenticStackPackDocument =
                    serde_json::from_slice(bytes).expect("stack-pack parses");
                document.requires[0].capability = "greentic.runtime.host.v1".to_string();
                *bytes = serde_json::to_vec_pretty(&document).expect("document serializes");
            }
            true
        });

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("cap://"));
    }

    #[test]
    fn gtpack_doctor_rejects_metadata_asset_without_lock_update() {
        let temp = tempdir().expect("tempdir");
        let out = temp.path().join("landlord.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: PathBuf::from("../../tests/e2e/fixtures/landlord_sor_v1.yaml"),
            name: "landlord-tenant-sor".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        rewrite_gtpack_preserving_lock(&out, |path, bytes| {
            if path == AGENT_ENDPOINT_ACTION_CATALOG_PATH {
                bytes.push(b'\n');
            }
            true
        });

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("pack.lock.cbor"));
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

    #[test]
    fn ontology_gtpack_artifacts_are_deterministic_and_discoverable() {
        let temp = tempdir().expect("tempdir");
        let input = write_ontology_fixture(temp.path());
        let first_out = temp.path().join("first.gtpack");
        let second_out = temp.path().join("second.gtpack");
        let options = |out_path: PathBuf| SorlaGtpackOptions {
            input_path: input.clone(),
            name: "ontology-demo".to_string(),
            version: "0.1.0".to_string(),
            out_path,
        };

        let first = build_sorla_gtpack(&options(first_out.clone())).expect("first pack builds");
        let second = build_sorla_gtpack(&options(second_out.clone())).expect("second pack builds");

        assert_eq!(
            fs::read(&first_out).unwrap(),
            fs::read(&second_out).unwrap()
        );
        assert_eq!(first.ir_hash, second.ir_hash);
        for path in [
            ONTOLOGY_GRAPH_PATH,
            ONTOLOGY_IR_CBOR_PATH,
            ONTOLOGY_SCHEMA_PATH,
            RETRIEVAL_BINDINGS_PATH,
            RETRIEVAL_BINDINGS_IR_CBOR_PATH,
        ] {
            assert!(first.assets.contains(&path.to_string()), "{path}");
        }

        let inspection = inspect_sorla_gtpack(&first_out).expect("inspect pack");
        let ontology = inspection
            .ontology
            .as_ref()
            .expect("inspect should summarize ontology metadata");
        assert_eq!(ontology.schema, ONTOLOGY_EXTENSION_ID);
        assert_eq!(ontology.graph_schema, ONTOLOGY_GRAPH_SCHEMA);
        assert_eq!(ontology.concept_count, 3);
        assert_eq!(ontology.relationship_count, 1);
        assert_eq!(ontology.constraint_count, 1);
        let retrieval = inspection
            .retrieval_bindings
            .as_ref()
            .expect("inspect should summarize retrieval bindings");
        assert_eq!(retrieval.schema, RETRIEVAL_BINDINGS_SCHEMA);
        assert_eq!(retrieval.provider_count, 1);
        assert_eq!(retrieval.scope_count, 1);
        assert_eq!(
            inspection.optional_artifacts.get(ONTOLOGY_GRAPH_PATH),
            Some(&true)
        );

        let mut archive =
            ZipArchive::new(fs::File::open(&first_out).expect("open pack")).expect("read pack");
        let graph: serde_json::Value =
            serde_json::from_str(&zip_text(&mut archive, ONTOLOGY_GRAPH_PATH).expect("graph"))
                .expect("graph JSON");
        assert_eq!(
            graph["semantic_aliases"]["concepts"]["Customer"],
            serde_json::json!(["client", "customer account"])
        );
        assert_eq!(
            graph["entity_linking"]["strategies"][0]["id"],
            "email_match"
        );
        let retrieval_json: serde_json::Value = serde_json::from_str(
            &zip_text(&mut archive, RETRIEVAL_BINDINGS_PATH).expect("retrieval JSON"),
        )
        .expect("retrieval JSON parses");
        assert_eq!(retrieval_json["providers"][0]["id"], "primary_evidence");
        let pack_manifest: SorlaPackManifest = ciborium::de::from_reader(Cursor::new(
            zip_bytes(&mut archive, "pack.cbor").expect("pack.cbor"),
        ))
        .expect("pack manifest decodes");
        let ontology_extension = pack_manifest
            .extension
            .get("sorla")
            .and_then(|sorla| sorla.get("ontology"))
            .expect("ontology extension is declared");
        assert_eq!(ontology_extension["schema"], ONTOLOGY_EXTENSION_ID);
        assert_eq!(ontology_extension["graph"], ONTOLOGY_GRAPH_PATH);
        assert_eq!(ontology_extension["ir"], ONTOLOGY_IR_CBOR_PATH);
        assert_eq!(ontology_extension["json_schema"], ONTOLOGY_SCHEMA_PATH);

        doctor_sorla_gtpack(&first_out).expect("doctor accepts ontology pack");
    }

    #[test]
    fn gtpack_doctor_rejects_missing_ontology_graph() {
        let temp = tempdir().expect("tempdir");
        let input = write_ontology_fixture(temp.path());
        let out = temp.path().join("ontology.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: input,
            name: "ontology-demo".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        rewrite_gtpack(&out, |path, _bytes| path != ONTOLOGY_GRAPH_PATH);

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains(ONTOLOGY_GRAPH_PATH));
    }

    #[test]
    fn gtpack_doctor_rejects_ontology_graph_that_diverges_from_ir() {
        let temp = tempdir().expect("tempdir");
        let input = write_ontology_fixture(temp.path());
        let out = temp.path().join("ontology.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: input,
            name: "ontology-demo".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        rewrite_gtpack(&out, |path, bytes| {
            if path == ONTOLOGY_GRAPH_PATH {
                let mut graph: serde_json::Value =
                    serde_json::from_slice(bytes).expect("graph JSON");
                graph["concepts"][0]["id"] =
                    serde_json::Value::String("UnknownConcept".to_string());
                *bytes = serde_json::to_vec_pretty(&graph).expect("graph serializes");
            }
            true
        });

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("concepts do not match"));
    }

    #[test]
    fn gtpack_doctor_rejects_unsafe_ontology_manifest_path() {
        let temp = tempdir().expect("tempdir");
        let input = write_ontology_fixture(temp.path());
        let out = temp.path().join("ontology.gtpack");
        build_sorla_gtpack(&SorlaGtpackOptions {
            input_path: input,
            name: "ontology-demo".to_string(),
            version: "0.1.0".to_string(),
            out_path: out.clone(),
        })
        .expect("pack builds");

        rewrite_gtpack(&out, |path, bytes| {
            if path == "pack.cbor" {
                let mut manifest: SorlaPackManifest =
                    ciborium::de::from_reader(Cursor::new(bytes.clone()))
                        .expect("manifest decodes");
                manifest.extension["sorla"]["ontology"]["graph"] =
                    serde_json::Value::String("../ontology.graph.json".to_string());
                *bytes = canonical_cbor(&manifest);
            }
            if path == "manifest.json" {
                let mut manifest: SorlaPackManifest =
                    serde_json::from_slice(bytes).expect("manifest JSON decodes");
                manifest.extension["sorla"]["ontology"]["graph"] =
                    serde_json::Value::String("../ontology.graph.json".to_string());
                *bytes = serde_json::to_vec_pretty(&manifest).expect("manifest serializes");
            }
            true
        });

        let err = doctor_sorla_gtpack(&out).expect_err("doctor should reject malformed pack");
        assert!(err.contains("ontology asset path"));
    }

    fn assert_gtpack_json_matches_fixture(pack_path: &Path, asset_path: &str, fixture_path: &str) {
        let mut archive =
            ZipArchive::new(fs::File::open(pack_path).expect("open pack")).expect("read pack");
        let actual = zip_text(&mut archive, asset_path).expect("asset should exist");
        let expected = fs::read_to_string(fixture_path).expect("fixture should read");
        assert_eq!(actual.trim_end(), expected.trim_end(), "{asset_path}");
    }

    fn write_ontology_fixture(dir: &Path) -> PathBuf {
        let path = dir.join("ontology.sorla.yaml");
        fs::write(
            &path,
            r#"
package:
  name: ontology-demo
  version: 0.1.0
records:
  - name: Customer
    fields:
      - name: id
        type: string
      - name: email
        type: string
        sensitive: true
  - name: Contract
    fields:
      - name: id
        type: string
  - name: CustomerContract
    fields:
      - name: customer_id
        type: string
        references:
          record: Customer
          field: id
      - name: contract_id
        type: string
        references:
          record: Contract
          field: id
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
    - id: customer_data_policy
      applies_to:
        concept: Customer
      requires_policy: customer_data_access
semantic_aliases:
  concepts:
    Customer:
      - customer account
      - client
  relationships:
    has_contract:
      - covered by
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
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: primary_evidence
      category: evidence
      required_capabilities:
        - evidence.query
        - entity.link
  scopes:
    - id: customer_evidence
      applies_to:
        concept: Customer
      provider: primary_evidence
      filters:
        entity_scope:
          include_self: true
          include_related:
            - relationship: has_contract
              direction: outgoing
              max_depth: 1
"#,
        )
        .expect("write ontology fixture");
        path
    }

    fn rewrite_gtpack(path: &Path, mut f: impl FnMut(&str, &mut Vec<u8>) -> bool) {
        let mut archive =
            ZipArchive::new(fs::File::open(path).expect("open pack")).expect("read pack");
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            if entry.name() == "pack.lock.cbor" {
                continue;
            }
            let name = entry.name().to_string();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            if f(&name, &mut bytes) {
                entries.insert(name, bytes);
            }
        }
        drop(archive);
        let lock = pack_lock_for_entries(&entries);
        entries.insert("pack.lock.cbor".to_string(), canonical_cbor(&lock));
        write_zip_entries(path, entries).expect("rewrite pack");
    }

    fn rewrite_gtpack_preserving_lock(path: &Path, mut f: impl FnMut(&str, &mut Vec<u8>) -> bool) {
        let mut archive =
            ZipArchive::new(fs::File::open(path).expect("open pack")).expect("read pack");
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            let name = entry.name().to_string();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            if f(&name, &mut bytes) {
                entries.insert(name, bytes);
            }
        }
        drop(archive);
        write_zip_entries(path, entries).expect("rewrite pack");
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

        assert_eq!(manifest.schema, SORX_AGENT_GATEWAY_SCHEMA);
        assert_eq!(manifest.package.name, "empty-agent-package");
        assert_eq!(manifest.package.ir_hash, canonical_hash_hex(&ir));
        assert!(manifest.endpoints.is_empty());
        assert!(manifest.provider_contract.categories.is_empty());
        assert!(manifest.exports.agent_gateway_json);
        assert!(!manifest.exports.openapi_overlay);
        assert!(!manifest.exports.arazzo);
        assert!(!manifest.exports.mcp_tools);
        assert!(!manifest.exports.llms_txt);
        assert!(manifest.notes[0].contains("SORX runtime route metadata"));
        assert!(manifest.notes[0].contains("SoRLa handoff context"));
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
    fn agent_gateway_manifest_emits_command_for_leave_endpoints() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: WaitingListEntry
    fields:
      - name: lab_id
        type: string
      - name: user_id
        type: string
      - name: joined_at
        type: timestamp
agent_endpoints:
  - id: leave_waiting_list
    title: Leave waiting list
    intent: Remove a user from a lab waiting list.
    inputs:
      - name: lab_id
        type: string
        required: true
      - name: user_id
        type: string
        required: true
      - name: reason
        type: string
        required: false
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        assert_eq!(endpoint.method, "POST");
        assert_eq!(
            endpoint.path,
            "/v1/agent/waiting_list_entries/leave_waiting_list"
        );
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["steps"][0]["op"], "delete_where");
        assert_eq!(command["steps"][0]["entity"], "WaitingListEntry");
        assert_eq!(command["steps"][0]["collection"], "waiting_list_entries");
        assert_eq!(command["steps"][0]["where"]["lab_id"], "$input.lab_id");
        assert_eq!(command["steps"][0]["where"]["user_id"], "$input.user_id");
        assert!(
            command["steps"][0]["where"].get("reason").is_none(),
            "only record fields should become delete filters"
        );
    }

    #[test]
    fn agent_gateway_manifest_emits_decrement_for_leave_waiting_list_with_referrer() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: waiting_list_entry
    fields:
      - name: entry_id
        type: string
      - name: lab_id
        type: string
      - name: email
        type: string
      - name: referrer_entry_id
        type: string
      - name: referred_count
        type: integer
agent_endpoints:
  - id: leave_waiting_list
    title: Leave waiting list
    intent: Remove a user from a lab waiting list.
    inputs:
      - name: lab_id
        type: string
        required: true
      - name: email
        type: string
        required: true
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");

        assert_eq!(command["steps"][0]["op"], "find_one");
        assert_eq!(command["steps"][0]["as"], "leaving_entry");
        assert_eq!(command["steps"][0]["where"]["lab_id"], "$input.lab_id");
        assert_eq!(command["steps"][0]["where"]["email"], "$input.email");
        assert_eq!(command["steps"][0]["required"], true);
        assert_eq!(command["steps"][1]["op"], "delete_where");
        assert_eq!(command["steps"][1]["as"], "leave");
        assert_eq!(command["steps"][2]["op"], "increment_where");
        assert_eq!(command["steps"][2]["as"], "referrer_decrement");
        assert_eq!(command["steps"][2]["where"]["lab_id"], "$input.lab_id");
        assert_eq!(
            command["steps"][2]["where"]["entry_id"],
            "$steps.leaving_entry.data.referrer_entry_id"
        );
        assert_eq!(command["steps"][2]["increments"]["referred_count"], -1);
        assert_eq!(
            command["steps"][2]["when"]["all"][0]["present"],
            "$steps.leaving_entry.data.referrer_entry_id"
        );
        assert_eq!(
            command["steps"][2]["when"]["all"][1]["equals"][0],
            "$steps.leave.deleted_count"
        );
        assert_eq!(command["steps"][2]["when"]["all"][1]["equals"][1], 1);
        assert_eq!(
            command["return"]["deleted_count"],
            "$steps.leave.deleted_count"
        );
    }

    #[test]
    fn agent_gateway_manifest_uses_post_for_query_endpoints_with_required_input() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: waiting_list_entry
    fields:
      - name: lab_id
        type: string
      - name: user_id
        type: string
      - name: joined_at
        type: timestamp
agent_endpoints:
  - id: show_waiting_list
    title: Show waiting list
    intent: Retrieve the ordered waiting list for a given lab.
    inputs:
      - name: lab_id
        type: string
        required: true
    outputs:
      - name: entries
        type: array
    agent_visibility:
      openapi: true
      arazzo: true
      mcp: true
      llms_txt: true
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "query");
        assert_eq!(endpoint.method, "POST");
        assert_eq!(
            endpoint.path,
            "/v1/agent/waiting_list_entries/query/show_waiting_list"
        );
        assert_eq!(endpoint.input_schema["required"][0], "lab_id");

        let exports = export_agent_artifacts(&ir);
        let openapi: serde_yaml::Value = serde_yaml::from_str(
            exports
                .openapi_overlay_yaml
                .as_deref()
                .expect("OpenAPI overlay should be generated"),
        )
        .expect("OpenAPI overlay should be valid YAML");
        assert_eq!(
            openapi["operations"][0]["x-greentic-agent"]["runtime"]["method"],
            "POST"
        );
        assert_eq!(
            openapi["operations"][0]["x-greentic-agent"]["runtime"]["input_transport"],
            "body"
        );

        let arazzo: serde_yaml::Value = serde_yaml::from_str(
            exports
                .arazzo_yaml
                .as_deref()
                .expect("Arazzo export should be generated"),
        )
        .expect("Arazzo export should be valid YAML");
        assert_eq!(
            arazzo["workflows"][0]["steps"][0]["x-greentic-agent"]["method"],
            "POST"
        );

        let mcp: serde_json::Value = serde_json::from_str(
            exports
                .mcp_tools_json
                .as_deref()
                .expect("MCP export should be generated"),
        )
        .expect("MCP export should be valid JSON");
        assert_eq!(mcp["tools"][0]["annotations"]["method"], "POST");
        assert_eq!(mcp["tools"][0]["annotations"]["input_transport"], "body");
    }

    #[test]
    fn agent_gateway_contract_validation_rejects_get_with_required_inputs() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: waiting_list_entry
    fields:
      - name: lab_id
        type: string
agent_endpoints:
  - id: show_waiting_list
    title: Show waiting list
    intent: Retrieve the waiting list for a lab.
    inputs:
      - name: lab_id
        type: string
        required: true
"#,
        )
        .expect("fixture should parse");
        let ir = lower_package(&parsed.package);
        let mut manifest = agent_gateway_handoff_manifest(&ir);
        manifest.endpoints[0].method = "GET".to_string();

        let err = validate_agent_gateway_runtime_contract(&manifest, &ir)
            .expect_err("GET with required inputs should be rejected");
        assert!(err.contains("declares GET with required body-style inputs: lab_id"));
    }

    #[test]
    fn agent_endpoint_contract_warnings_flag_generic_output_and_domain_claims() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: waiting_list_entry
    fields:
      - name: entry_id
        type: string
      - name: lab_id
        type: string
      - name: user_id
        type: string
agent_endpoints:
  - id: join_waiting_list
    title: Join waiting list
    intent: Add a user and return their current position.
    inputs:
      - name: lab_id
        type: string
        required: true
      - name: user_id
        type: string
        required: true
    outputs:
      - name: entry_id
        type: string
      - name: position
        type: integer
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let warnings = agent_endpoint_contract_warnings(&ir);

        assert!(warnings.iter().any(|warning| {
            warning.endpoint_id == "join_waiting_list"
                && warning.code == "sorla.agent_endpoint.output_contract"
                && warning.message.contains("entry_id, position")
        }));
        assert!(warnings.iter().any(|warning| {
            warning.endpoint_id == "join_waiting_list"
                && warning.code == "sorla.agent_endpoint.generic_backing"
        }));
    }

    #[test]
    fn agent_gateway_manifest_emits_command_for_generate_field_endpoints() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: WaitingListEntry
    fields:
      - name: id
        type: string
      - name: referral_code
        type: string
      - name: user_id
        type: string
agent_endpoints:
  - id: generate_referral_code
    title: Generate referral code
    intent: Generate or retrieve a referral code for an entry.
    inputs:
      - name: entry_id
        type: string
        required: true
    outputs:
      - name: referral_code
        type: string
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        assert_eq!(endpoint.method, "POST");
        assert_eq!(
            endpoint.path,
            "/v1/agent/waiting_list_entries/generate_referral_code"
        );
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["steps"][0]["op"], "find_one");
        assert_eq!(command["steps"][0]["as"], "entry");
        assert_eq!(command["steps"][0]["where"]["id"], "$input.entry_id");
        assert_eq!(command["steps"][1]["op"], "update_where");
        assert_eq!(command["steps"][1]["as"], "update");
        assert_eq!(
            command["steps"][1]["set"]["referral_code"]["coalesce"][0],
            "$steps.entry.data.referral_code"
        );
        assert_eq!(
            command["steps"][1]["set"]["referral_code"]["coalesce"][1],
            "$generated.referral_code"
        );
        assert_eq!(
            command["return"]["referral_code"],
            "$steps.update.records.0.data.referral_code"
        );
    }

    #[test]
    fn agent_gateway_manifest_emits_command_for_join_waiting_list() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: waiting_list_entry
    fields:
      - name: entry_id
        type: string
      - name: lab_id
        type: string
      - name: user_id
        type: string
      - name: email
        type: string
      - name: name
        type: string
      - name: invitation_code
        type: string
      - name: invited_by_code
        type: string
      - name: referrer_entry_id
        type: string
      - name: referred_count
        type: integer
      - name: joined_at
        type: timestamp
operational_indexes:
  schema: greentic.sorla.operational-indexes.v1
  indexes:
    - id: waiting_list_entry_lab_email_unique
      record: waiting_list_entry
      kind: composite
      unique: true
      fields:
        - lab_id
        - email
    - id: waiting_list_entry_lab_invitation_code_unique
      record: waiting_list_entry
      kind: composite
      unique: true
      fields:
        - lab_id
        - invitation_code
agent_endpoints:
  - id: join_waiting_list
    title: Join waiting list
    intent: Add a user to a lab waiting list and assign an invitation code.
    inputs:
      - name: lab_id
        type: string
        required: true
      - name: email
        type: string
        required: true
      - name: name
        type: string
        required: true
      - name: invited_by_code
        type: string
        required: false
    outputs:
      - name: entry_id
        type: string
      - name: invitation_code
        type: string
      - name: position
        type: integer
      - name: number_in_waiting_list
        type: integer
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        assert_eq!(endpoint.method, "POST");
        assert_eq!(
            endpoint.path,
            "/v1/agent/waiting_list_entries/join_waiting_list"
        );
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["steps"][0]["op"], "find_one");
        assert_eq!(command["steps"][0]["as"], "referrer");
        assert_eq!(command["steps"][0]["where"]["lab_id"], "$input.lab_id");
        assert_eq!(
            command["steps"][0]["where"]["invitation_code"],
            "$input.invited_by_code"
        );
        assert_eq!(command["steps"][0]["required"], true);
        assert_eq!(
            command["steps"][0]["when"]["present"],
            "$input.invited_by_code"
        );
        assert_eq!(command["steps"][1]["op"], "create");
        assert_eq!(command["steps"][1]["as"], "entry");
        assert_eq!(
            command["steps"][1]["input"]["entry_id"],
            "$generated.entry_id"
        );
        assert_eq!(command["steps"][1]["input"]["lab_id"], "$input.lab_id");
        assert_eq!(command["steps"][1]["input"]["user_id"], "$generated.uuid");
        assert_eq!(command["steps"][1]["input"]["email"], "$input.email");
        assert_eq!(command["steps"][1]["input"]["name"], "$input.name");
        assert_eq!(
            command["steps"][1]["input"]["invitation_code"],
            "$generated.invitation_code"
        );
        assert_eq!(
            command["steps"][1]["input"]["invited_by_code"],
            "$input.invited_by_code"
        );
        assert_eq!(
            command["steps"][1]["input"]["referrer_entry_id"],
            "$steps.referrer.data.entry_id"
        );
        assert_eq!(command["steps"][1]["input"]["referred_count"], 0);
        assert_eq!(command["steps"][1]["input"]["joined_at"], "$now");
        assert_eq!(command["steps"][2]["op"], "increment_where");
        assert_eq!(command["steps"][2]["as"], "referrer_increment");
        assert_eq!(command["steps"][2]["where"]["lab_id"], "$input.lab_id");
        assert_eq!(
            command["steps"][2]["where"]["invitation_code"],
            "$input.invited_by_code"
        );
        assert_eq!(command["steps"][2]["increments"]["referred_count"], 1);
        assert_eq!(
            command["steps"][2]["when"]["all"][0]["present"],
            "$input.invited_by_code"
        );
        assert_eq!(
            command["steps"][2]["when"]["all"][1]["equals"][0],
            "$steps.entry.created"
        );
        assert_eq!(command["steps"][2]["when"]["all"][1]["equals"][1], true);
        assert_eq!(command["steps"][3]["op"], "query");
        assert_eq!(command["steps"][3]["as"], "waiting_list");
        assert_eq!(command["steps"][3]["where"]["lab_id"], "$input.lab_id");
        assert_eq!(
            command["steps"][3]["order_by"][0]["field"],
            "referred_count"
        );
        assert_eq!(command["steps"][3]["order_by"][0]["direction"], "desc");
        assert_eq!(command["steps"][3]["order_by"][1]["field"], "joined_at");
        assert_eq!(command["steps"][3]["order_by"][1]["direction"], "asc");
        assert_eq!(
            command["return"]["entry_id"],
            "$steps.entry.record.data.entry_id"
        );
        assert_eq!(
            command["return"]["invitation_code"],
            "$steps.entry.record.data.invitation_code"
        );
        assert_eq!(command["return"]["position"], "$steps.waiting_list.count");
        assert_eq!(
            command["return"]["number_in_waiting_list"],
            "$steps.waiting_list.count"
        );
        assert_eq!(command["idempotency"], "return_existing");
        assert_eq!(
            command["constraints"]["idempotency"]["index"],
            "waiting_list_entry_lab_email_unique"
        );
        assert_eq!(
            command["constraints"]["unique"][0]["index"],
            "waiting_list_entry_lab_email_unique"
        );
        assert_eq!(
            command["constraints"]["unique"][1]["index"],
            "waiting_list_entry_lab_invitation_code_unique"
        );
    }

    #[test]
    fn agent_gateway_manifest_emits_ordered_command_for_show_waiting_list() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: waiting_list_entry
    fields:
      - name: lab_id
        type: string
      - name: referred_count
        type: integer
      - name: joined_at
        type: timestamp
agent_endpoints:
  - id: show_waiting_list
    title: Show waiting list
    intent: Retrieve the ordered waiting list for a lab.
    inputs:
      - name: lab_id
        type: string
        required: true
    outputs:
      - name: entries
        type: array
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        assert_eq!(endpoint.method, "POST");
        assert_eq!(
            endpoint.path,
            "/v1/agent/waiting_list_entries/show_waiting_list"
        );
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["kind"], "record_query");
        assert_eq!(command["steps"][0]["op"], "query");
        assert_eq!(command["steps"][0]["where"]["lab_id"], "$input.lab_id");
        assert_eq!(
            command["steps"][0]["order_by"][0]["field"],
            "referred_count"
        );
        assert_eq!(command["steps"][0]["order_by"][0]["direction"], "desc");
        assert_eq!(command["steps"][0]["order_by"][1]["field"], "joined_at");
        assert_eq!(command["steps"][0]["order_by"][1]["direction"], "asc");
        assert_eq!(command["return"]["entries"], "$steps.waiting_list.records");
    }

    #[test]
    fn agent_gateway_manifest_emits_command_for_retrieve_field_endpoints() {
        let parsed = parse_package(
            r#"
package:
  name: waitlist
  version: 0.1.0
records:
  - name: waiting_list_entry
    fields:
      - name: entry_id
        type: string
      - name: referral_code
        type: string
      - name: user_id
        type: string
agent_endpoints:
  - id: retrieve_referral_code
    title: Retrieve referral code
    intent: Retrieve the existing referral code for an entry.
    inputs:
      - name: entry_id
        type: string
        required: true
    outputs:
      - name: referral_code
        type: string
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        assert_eq!(endpoint.method, "POST");
        assert_eq!(
            endpoint.path,
            "/v1/agent/waiting_list_entries/retrieve_referral_code"
        );
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["kind"], "record_lookup");
        assert_eq!(command["steps"][0]["op"], "find_one");
        assert_eq!(command["steps"][0]["as"], "entry");
        assert_eq!(command["steps"][0]["where"]["entry_id"], "$input.entry_id");
        assert_eq!(
            command["return"]["referral_code"],
            "$steps.entry.data.referral_code"
        );
    }

    #[test]
    fn agent_gateway_manifest_generates_field_commands_without_referral_special_case() {
        let parsed = parse_package(
            r#"
package:
  name: invite-system
  version: 0.1.0
records:
  - name: Invite
    fields:
      - name: id
        type: string
      - name: invite_code
        type: string
      - name: email
        type: string
agent_endpoints:
  - id: generate_invite_code
    title: Generate invite code
    intent: Generate or retrieve an invite code for an invite.
    inputs:
      - name: invite_id
        type: string
        required: true
    outputs:
      - name: invite_code
        type: string
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        assert_eq!(endpoint.path, "/v1/agent/invites/generate_invite_code");
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["steps"][0]["where"]["id"], "$input.invite_id");
        assert_eq!(
            command["steps"][1]["set"]["invite_code"]["coalesce"][0],
            "$steps.entry.data.invite_code"
        );
        assert_eq!(
            command["steps"][1]["set"]["invite_code"]["coalesce"][1],
            "$generated.invite_code"
        );
        assert_eq!(command["return"]["invite_id"], "$input.invite_id");
        assert_eq!(
            command["return"]["invite_code"],
            "$steps.update.records.0.data.invite_code"
        );
    }

    #[test]
    fn agent_gateway_manifest_emits_command_for_bulk_import_endpoints() {
        let parsed = parse_package(
            r#"
package:
  name: bulk-system
  version: 0.1.0
records:
  - name: entity_a
    fields:
      - name: entity_a_id
        type: string
agent_endpoints:
  - id: bulk_import_entities
    title: Bulk import entities
    intent: Import arbitrary entity records.
    inputs:
      - name: items
        type: array
        required: true
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        assert_eq!(endpoint.method, "POST");
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["kind"], "bulk_mutation");
        assert_eq!(command["steps"][0]["op"], "foreach");
        assert_eq!(command["steps"][0]["as"], "imported");
        assert_eq!(command["steps"][0]["items"], "$input.items");
        assert_eq!(command["steps"][0]["do"][0]["op"], "create");
        assert_eq!(command["steps"][0]["do"][0]["entity"], "$item.entity");
        assert_eq!(
            command["steps"][0]["do"][0]["collection"],
            "$item.collection"
        );
        assert_eq!(command["steps"][0]["do"][0]["input"], "$item.data");
        assert_eq!(command["return"]["imported_count"], "$steps.imported.count");
        assert_eq!(command["return"]["records"], "$steps.imported.records");
    }

    #[test]
    fn agent_gateway_manifest_emits_commands_for_archive_and_restore_endpoints() {
        let parsed = parse_package(
            r#"
package:
  name: lifecycle-system
  version: 0.1.0
records:
  - name: sample_entity
    fields:
      - name: entity_id
        type: string
      - name: is_active
        type: boolean
      - name: name
        type: string
agent_endpoints:
  - id: archive_sample_entity
    title: Archive sample entity
    intent: Archive the entity.
    inputs:
      - name: entity_id
        type: string
        required: true
      - name: is_active
        type: boolean
        required: false
  - id: restore_sample_entity
    title: Restore sample entity
    intent: Restore the entity.
    inputs:
      - name: entity_id
        type: string
        required: true
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);

        let archive = &manifest.endpoints[0];
        assert_eq!(archive.operation, "command");
        assert_eq!(archive.method, "POST");
        assert_eq!(
            archive.path,
            "/v1/agent/sample_entities/archive_sample_entity"
        );
        let command = archive.command.as_ref().expect("command should be emitted");
        assert_eq!(command["steps"][0]["op"], "update_where");
        assert_eq!(
            command["steps"][0]["where"]["entity_id"],
            "$input.entity_id"
        );
        assert_eq!(command["steps"][0]["set"]["is_active"], false);

        let restore = &manifest.endpoints[1];
        assert_eq!(restore.operation, "command");
        let command = restore.command.as_ref().expect("command should be emitted");
        assert_eq!(command["steps"][0]["set"]["is_active"], true);
    }

    #[test]
    fn agent_gateway_manifest_emits_commands_for_approve_reject_and_unlink_endpoints() {
        let parsed = parse_package(
            r#"
package:
  name: assignment-system
  version: 0.1.0
records:
  - name: bug_assignment
    fields:
      - name: assignment_id
        type: string
      - name: status
        type: enum
        enum_values:
          - pending
          - active
          - rejected
  - name: bug_asset
    fields:
      - name: asset_id
        type: string
      - name: case_id
        type: string
agent_endpoints:
  - id: approve_assignment
    title: Approve assignment
    intent: Approve an assignment.
    inputs:
      - name: assignment_id
        type: string
        required: true
      - name: status
        type: string
        required: false
  - id: reject_assignment
    title: Reject assignment
    intent: Reject an assignment.
    inputs:
      - name: assignment_id
        type: string
        required: true
      - name: status
        type: string
        required: false
  - id: unlink_asset
    title: Unlink asset
    intent: Unlink an asset.
    inputs:
      - name: asset_id
        type: string
        required: true
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);

        let approve = &manifest.endpoints[0];
        assert_eq!(approve.operation, "command");
        let command = approve.command.as_ref().expect("command should be emitted");
        assert_eq!(command["steps"][0]["op"], "update_where");
        assert_eq!(
            command["steps"][0]["where"]["assignment_id"],
            "$input.assignment_id"
        );
        assert_eq!(command["steps"][0]["set"]["status"], "active");

        let reject = &manifest.endpoints[1];
        assert_eq!(reject.operation, "command");
        let command = reject.command.as_ref().expect("command should be emitted");
        assert_eq!(command["steps"][0]["set"]["status"], "rejected");

        let unlink = &manifest.endpoints[2];
        assert_eq!(unlink.operation, "command");
        let command = unlink.command.as_ref().expect("command should be emitted");
        assert_eq!(command["steps"][0]["op"], "delete_where");
        assert_eq!(command["steps"][0]["where"]["asset_id"], "$input.asset_id");
    }

    #[test]
    fn agent_gateway_manifest_generates_token_command_without_output_field() {
        let parsed = parse_package(
            r#"
package:
  name: token-system
  version: 0.1.0
records:
  - name: bug_case
    fields:
      - name: case_id
        type: string
      - name: title
        type: string
agent_endpoints:
  - id: generate_case_token
    title: Generate case token
    intent: Generate a case token.
    inputs:
      - name: case_id
        type: string
        required: true
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "command");
        let command = endpoint
            .command
            .as_ref()
            .expect("command should be emitted");
        assert_eq!(command["steps"][0]["op"], "find_one");
        assert_eq!(command["steps"][0]["where"]["case_id"], "$input.case_id");
        assert_eq!(
            command["steps"][1]["set"]["case_token"]["coalesce"][1],
            "$generated.case_token"
        );
        assert_eq!(
            command["return"]["case_token"],
            "$steps.update.records.0.data.case_token"
        );
    }

    #[test]
    fn agent_gateway_manifest_emits_command_for_generic_side_effect_actions() {
        let parsed = parse_package(
            r#"
package:
  name: workflow-system
  version: 0.1.0
records:
  - name: workflow_log
    fields:
      - name: id
        type: string
agent_endpoints:
  - id: custom_workflow_action
    title: Custom workflow action
    intent: Trigger a custom workflow.
    side_effects:
      - action.custom_workflow_action
  - id: trigger_policy_action
    title: Trigger policy action
    intent: Exercise policy hooks.
    side_effects:
      - action.trigger_policy_action
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);

        for endpoint in &manifest.endpoints {
            assert_eq!(endpoint.operation, "command");
            assert_eq!(endpoint.method, "POST");
            let command = endpoint
                .command
                .as_ref()
                .expect("command should be emitted");
            assert_eq!(command["kind"], "side_effect");
            assert_eq!(command["steps"][0]["op"], "emit_event");
            assert_eq!(command["steps"][0]["as"], "event");
            assert_eq!(
                command["steps"][0]["event"],
                format!("action.{}", endpoint.id)
            );
            assert_eq!(command["steps"][0]["payload"]["input"], "$input");
            assert_eq!(command["return"]["event"], "$steps.event");
        }
    }

    #[test]
    fn agent_gateway_manifest_routes_link_endpoints_to_join_record() {
        let parsed = parse_package(
            r#"
package:
  name: complex
  version: 0.1.0
records:
  - name: entity_b
    fields:
      - name: entity_b_id
        type: string
  - name: entity_c
    fields:
      - name: entity_c_id
        type: string
  - name: join_bc
    fields:
      - name: entity_b_id
        type: string
        references:
          record: entity_b
          field: entity_b_id
      - name: entity_c_id
        type: string
        references:
          record: entity_c
          field: entity_c_id
agent_endpoints:
  - id: link_b_to_c
    title: Link B to C
    intent: Establish a join row.
    inputs:
      - name: entity_b_id
        type: string
        required: true
      - name: entity_c_id
        type: string
        required: true
"#,
        )
        .expect("fixture should parse");

        let ir = lower_package(&parsed.package);
        let manifest = agent_gateway_handoff_manifest(&ir);
        let endpoint = &manifest.endpoints[0];

        assert_eq!(endpoint.operation, "create");
        assert_eq!(endpoint.method, "POST");
        assert_eq!(endpoint.entity, "join_bc");
        assert_eq!(endpoint.collection, "join_bcs");
        assert_eq!(endpoint.path, "/v1/agent/join_bcs/link_b_to_c");
    }

    #[test]
    fn agent_exports_include_only_enabled_targets_and_are_deterministic() {
        let ir = lead_capture_ir();
        let first = export_agent_artifacts(&ir);
        let second = export_agent_artifacts(&ir);

        assert_eq!(first, second);
        assert!(first.agent_gateway_json.contains(SORX_AGENT_GATEWAY_SCHEMA));
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

        assert_eq!(mcp["schema"], SORX_MCP_TOOLS_SCHEMA);
        assert_eq!(mcp["tools"][0]["name"], "create_customer_contact");
        assert_eq!(mcp["tools"][0]["endpoint_id"], "create_customer_contact");
        assert_eq!(
            mcp["tools"][0]["inputSchema"]["required"][0],
            "company_name"
        );
        assert_eq!(
            mcp["tools"][0]["input_schema"]["required"][0],
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
