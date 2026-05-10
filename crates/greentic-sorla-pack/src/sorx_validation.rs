use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const SORX_VALIDATION_SCHEMA: &str = "greentic.sorx.validation.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorxValidationManifest {
    pub schema: String,
    pub suite_version: String,
    pub package: SorxValidationPackageRef,
    pub default_visibility: EndpointVisibility,
    pub promotion_requires: Vec<String>,
    pub suites: Vec<SorxValidationSuite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorxValidationPackageRef {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EndpointVisibility {
    Private,
    Internal,
    PublicCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorxValidationSuite {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub required: bool,
    pub tests: Vec<SorxValidationTest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SorxValidationTest {
    Healthcheck {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    AgentEndpoint {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    OpenApiContract {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    McpToolContract {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    ArazzoWorkflow {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    ProviderCapability {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
        provider_category: String,
        capabilities: Vec<String>,
    },
    ProviderConnectivity {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
        provider_category: String,
        capabilities: Vec<String>,
    },
    AuthRequired {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    PolicyEnforced {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    TenantIsolation {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    MigrationCompatibility {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
    RollbackCompatibility {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<serde_json::Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SorxValidationError {
    message: String,
}

impl SorxValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SorxValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SorxValidationError {}

impl SorxValidationManifest {
    pub fn validate_static(&self) -> Result<(), SorxValidationError> {
        if self.schema != SORX_VALIDATION_SCHEMA {
            return Err(SorxValidationError::new(format!(
                "validation manifest schema must be `{SORX_VALIDATION_SCHEMA}`, got `{}`",
                self.schema
            )));
        }
        if self.package.name.trim().is_empty() {
            return Err(SorxValidationError::new(
                "validation manifest package.name must not be empty",
            ));
        }
        if self.package.version.trim().is_empty() {
            return Err(SorxValidationError::new(
                "validation manifest package.version must not be empty",
            ));
        }

        let mut suite_ids = BTreeSet::new();
        let mut test_ids = BTreeSet::new();
        for suite in &self.suites {
            if suite.id.trim().is_empty() {
                return Err(SorxValidationError::new(
                    "validation suite id must not be empty",
                ));
            }
            if !suite_ids.insert(suite.id.as_str()) {
                return Err(SorxValidationError::new(format!(
                    "duplicate validation suite id `{}`",
                    suite.id
                )));
            }
            for test in &suite.tests {
                let test_id = test.id();
                if test_id.trim().is_empty() {
                    return Err(SorxValidationError::new(format!(
                        "validation suite `{}` has a test with an empty id",
                        suite.id
                    )));
                }
                if !test_ids.insert(test_id) {
                    return Err(SorxValidationError::new(format!(
                        "duplicate validation test id `{test_id}`"
                    )));
                }
                for reference in test.referenced_asset_paths() {
                    validate_relative_reference(reference)?;
                }
            }
        }

        for required in &self.promotion_requires {
            if !suite_ids.contains(required.as_str()) {
                return Err(SorxValidationError::new(format!(
                    "promotion requirement `{required}` does not match a validation suite"
                )));
            }
        }

        Ok(())
    }
}

impl SorxValidationTest {
    pub fn id(&self) -> &str {
        match self {
            Self::Healthcheck { id, .. }
            | Self::AgentEndpoint { id, .. }
            | Self::OpenApiContract { id, .. }
            | Self::McpToolContract { id, .. }
            | Self::ArazzoWorkflow { id, .. }
            | Self::ProviderCapability { id, .. }
            | Self::ProviderConnectivity { id, .. }
            | Self::AuthRequired { id, .. }
            | Self::PolicyEnforced { id, .. }
            | Self::TenantIsolation { id, .. }
            | Self::MigrationCompatibility { id, .. }
            | Self::RollbackCompatibility { id, .. } => id,
        }
    }

    pub fn referenced_asset_paths(&self) -> Vec<&str> {
        match self {
            Self::Healthcheck { input_ref, .. }
            | Self::AgentEndpoint { input_ref, .. }
            | Self::OpenApiContract { input_ref, .. }
            | Self::McpToolContract { input_ref, .. }
            | Self::ArazzoWorkflow { input_ref, .. }
            | Self::ProviderCapability { input_ref, .. }
            | Self::ProviderConnectivity { input_ref, .. }
            | Self::AuthRequired { input_ref, .. }
            | Self::PolicyEnforced { input_ref, .. }
            | Self::TenantIsolation { input_ref, .. }
            | Self::MigrationCompatibility { input_ref, .. }
            | Self::RollbackCompatibility { input_ref, .. } => {
                input_ref.iter().map(String::as_str).collect()
            }
        }
    }
}

fn validate_relative_reference(reference: &str) -> Result<(), SorxValidationError> {
    if reference.starts_with('/') {
        return Err(SorxValidationError::new(format!(
            "validation asset reference `{reference}` must be relative"
        )));
    }
    if reference
        .split('/')
        .any(|component| component == ".." || component.is_empty())
    {
        return Err(SorxValidationError::new(format!(
            "validation asset reference `{reference}` must not escape its base directory"
        )));
    }
    Ok(())
}

pub fn sorx_validation_schema_json() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SORX_VALIDATION_SCHEMA,
        "title": "SORX validation manifest",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "schema",
            "suite_version",
            "package",
            "default_visibility",
            "promotion_requires",
            "suites"
        ],
        "properties": {
            "schema": { "const": SORX_VALIDATION_SCHEMA },
            "suite_version": { "type": "string", "minLength": 1 },
            "package": { "$ref": "#/$defs/package" },
            "default_visibility": { "$ref": "#/$defs/endpoint_visibility" },
            "promotion_requires": {
                "type": "array",
                "items": { "type": "string", "minLength": 1 }
            },
            "suites": {
                "type": "array",
                "items": { "$ref": "#/$defs/suite" }
            }
        },
        "$defs": {
            "endpoint_visibility": {
                "type": "string",
                "enum": ["private", "internal", "public_candidate"]
            },
            "package": {
                "type": "object",
                "additionalProperties": false,
                "required": ["name", "version"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "version": { "type": "string", "minLength": 1 },
                    "ir_version": { "type": "string", "minLength": 1 },
                    "ir_hash": { "type": "string", "minLength": 1 }
                }
            },
            "suite": {
                "type": "object",
                "additionalProperties": false,
                "required": ["id", "required", "tests"],
                "properties": {
                    "id": { "type": "string", "minLength": 1 },
                    "title": { "type": "string" },
                    "required": { "type": "boolean" },
                    "tests": {
                        "type": "array",
                        "items": { "$ref": "#/$defs/test" }
                    }
                }
            },
            "test": {
                "type": "object",
                "required": ["kind", "id"],
                "properties": {
                    "kind": {
                        "type": "string",
                        "enum": [
                            "healthcheck",
                            "agent-endpoint",
                            "openapi-contract",
                            "mcp-tool-contract",
                            "arazzo-workflow",
                            "provider-capability",
                            "provider-connectivity",
                            "auth-required",
                            "policy-enforced",
                            "tenant-isolation",
                            "migration-compatibility",
                            "rollback-compatibility"
                        ]
                    },
                    "id": { "type": "string", "minLength": 1 },
                    "title": { "type": "string" },
                    "endpoint": { "type": "string" },
                    "required": { "type": "boolean" },
                    "timeout_ms": { "type": "integer", "minimum": 1 },
                    "input_ref": { "type": "string" },
                    "expect": true,
                    "provider_category": { "type": "string", "minLength": 1 },
                    "capabilities": {
                        "type": "array",
                        "items": { "type": "string", "minLength": 1 }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_manifest() -> SorxValidationManifest {
        SorxValidationManifest {
            schema: SORX_VALIDATION_SCHEMA.to_string(),
            suite_version: "1.0.0".to_string(),
            package: SorxValidationPackageRef {
                name: "landlord-tenant-sor".to_string(),
                version: "0.1.0".to_string(),
                ir_version: Some("0.1".to_string()),
                ir_hash: Some("abc123".to_string()),
            },
            default_visibility: EndpointVisibility::Private,
            promotion_requires: vec!["smoke".to_string()],
            suites: vec![SorxValidationSuite {
                id: "smoke".to_string(),
                title: None,
                required: true,
                tests: vec![SorxValidationTest::Healthcheck {
                    id: "runtime-startup-assets-present".to_string(),
                    title: None,
                    endpoint: None,
                    required: Some(true),
                    timeout_ms: None,
                    input_ref: None,
                    expect: None,
                }],
            }],
        }
    }

    #[test]
    fn accepts_valid_minimal_manifest() {
        minimal_manifest()
            .validate_static()
            .expect("minimal manifest should validate");
    }

    #[test]
    fn rejects_bad_schema() {
        let mut manifest = minimal_manifest();
        manifest.schema = "wrong.schema".to_string();

        let err = manifest
            .validate_static()
            .expect_err("bad schema should be rejected");

        assert!(err.message().contains(SORX_VALIDATION_SCHEMA));
    }

    #[test]
    fn rejects_duplicate_suite_id() {
        let mut manifest = minimal_manifest();
        manifest.suites.push(manifest.suites[0].clone());

        let err = manifest
            .validate_static()
            .expect_err("duplicate suite should be rejected");

        assert!(err.message().contains("duplicate validation suite id"));
    }

    #[test]
    fn rejects_duplicate_test_id_globally() {
        let mut manifest = minimal_manifest();
        manifest.suites.push(SorxValidationSuite {
            id: "contract".to_string(),
            title: None,
            required: true,
            tests: manifest.suites[0].tests.clone(),
        });

        let err = manifest
            .validate_static()
            .expect_err("duplicate test should be rejected");

        assert!(err.message().contains("duplicate validation test id"));
    }

    #[test]
    fn rejects_escaping_reference() {
        let mut manifest = minimal_manifest();
        manifest.suites[0].tests[0] = SorxValidationTest::AgentEndpoint {
            id: "agent-endpoint-contract".to_string(),
            title: None,
            endpoint: Some("create_customer_contact".to_string()),
            required: Some(true),
            timeout_ms: None,
            input_ref: Some("../fixtures/input.json".to_string()),
            expect: None,
        };

        let err = manifest
            .validate_static()
            .expect_err("escaping reference should be rejected");

        assert!(err.message().contains("must not escape"));
    }

    #[test]
    fn rejects_missing_promotion_suite() {
        let mut manifest = minimal_manifest();
        manifest.promotion_requires.push("contract".to_string());

        let err = manifest
            .validate_static()
            .expect_err("missing promotion suite should be rejected");

        assert!(err.message().contains("promotion requirement"));
    }

    #[test]
    fn emits_deterministic_schema_json() {
        let first = serde_json::to_string_pretty(&sorx_validation_schema_json())
            .expect("schema should serialize");
        let second = serde_json::to_string_pretty(&sorx_validation_schema_json())
            .expect("schema should serialize again");

        assert_eq!(first, second);
        assert!(first.contains(SORX_VALIDATION_SCHEMA));
        assert!(first.contains("provider-connectivity"));
    }
}
