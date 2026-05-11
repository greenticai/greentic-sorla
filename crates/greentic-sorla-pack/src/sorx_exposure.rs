use crate::sorx_validation::EndpointVisibility;
use greentic_sorla_ir::{AgentEndpointApprovalModeIr, AgentEndpointIr, AgentEndpointRiskIr};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const SORX_EXPOSURE_POLICY_SCHEMA: &str = "greentic.sorx.exposure-policy.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorxExposurePolicy {
    pub schema: String,
    pub default_visibility: EndpointVisibility,
    pub promotion_requires: Vec<String>,
    pub allowed_route_prefixes: Vec<String>,
    pub forbidden_route_prefixes: Vec<String>,
    pub endpoints: Vec<SorxEndpointExposurePolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorxEndpointExposurePolicy {
    pub endpoint_id: String,
    pub visibility: EndpointVisibility,
    pub requires_approval: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    pub export_surfaces: Vec<String>,
    pub route_prefixes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SorxExposurePolicyError {
    message: String,
}

impl SorxExposurePolicyError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SorxExposurePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SorxExposurePolicyError {}

impl SorxExposurePolicy {
    pub fn validate_static(
        &self,
        known_endpoint_ids: &BTreeSet<&str>,
    ) -> Result<(), SorxExposurePolicyError> {
        if self.schema != SORX_EXPOSURE_POLICY_SCHEMA {
            return Err(SorxExposurePolicyError::new(format!(
                "exposure policy schema must be `{SORX_EXPOSURE_POLICY_SCHEMA}`, got `{}`",
                self.schema
            )));
        }
        if self.default_visibility == EndpointVisibility::PublicCandidate {
            return Err(SorxExposurePolicyError::new(
                "exposure policy default_visibility must not be public_candidate",
            ));
        }
        for prefix in self
            .allowed_route_prefixes
            .iter()
            .chain(self.forbidden_route_prefixes.iter())
        {
            validate_route_prefix(prefix)?;
        }

        let mut endpoint_ids = BTreeSet::new();
        for endpoint in &self.endpoints {
            if endpoint.endpoint_id.trim().is_empty() {
                return Err(SorxExposurePolicyError::new(
                    "exposure policy endpoint_id must not be empty",
                ));
            }
            if !endpoint_ids.insert(endpoint.endpoint_id.as_str()) {
                return Err(SorxExposurePolicyError::new(format!(
                    "duplicate exposure policy endpoint_id `{}`",
                    endpoint.endpoint_id
                )));
            }
            if !known_endpoint_ids.contains(endpoint.endpoint_id.as_str()) {
                return Err(SorxExposurePolicyError::new(format!(
                    "exposure policy references unknown endpoint `{}`",
                    endpoint.endpoint_id
                )));
            }
            if endpoint.risk.as_deref() == Some("high") && !endpoint.requires_approval {
                return Err(SorxExposurePolicyError::new(format!(
                    "high-risk endpoint `{}` must require approval",
                    endpoint.endpoint_id
                )));
            }
            for prefix in &endpoint.route_prefixes {
                validate_route_prefix(prefix)?;
            }
        }
        Ok(())
    }
}

fn validate_route_prefix(prefix: &str) -> Result<(), SorxExposurePolicyError> {
    if prefix.is_empty() {
        return Ok(());
    }
    if !prefix.starts_with('/') || prefix.split('/').any(|component| component == "..") {
        return Err(SorxExposurePolicyError::new(format!(
            "route prefix `{prefix}` must be absolute and must not contain `..`"
        )));
    }
    Ok(())
}

pub fn generate_sorx_exposure_policy(endpoints: &[AgentEndpointIr]) -> SorxExposurePolicy {
    let mut endpoint_policies = endpoints
        .iter()
        .filter(|endpoint| !export_surfaces(endpoint).is_empty())
        .map(|endpoint| SorxEndpointExposurePolicy {
            endpoint_id: endpoint.id.clone(),
            visibility: EndpointVisibility::PublicCandidate,
            requires_approval: requires_approval(endpoint),
            risk: Some(risk_label(&endpoint.risk).to_string()),
            export_surfaces: export_surfaces(endpoint),
            route_prefixes: Vec::new(),
        })
        .collect::<Vec<_>>();
    endpoint_policies.sort_by(|left, right| left.endpoint_id.cmp(&right.endpoint_id));

    SorxExposurePolicy {
        schema: SORX_EXPOSURE_POLICY_SCHEMA.to_string(),
        default_visibility: EndpointVisibility::Private,
        promotion_requires: vec![
            "validation_success".to_string(),
            "security_success".to_string(),
            "provider_resolution_success".to_string(),
        ],
        allowed_route_prefixes: Vec::new(),
        forbidden_route_prefixes: vec![
            "/internal".to_string(),
            "/debug".to_string(),
            "/admin/raw".to_string(),
        ],
        endpoints: endpoint_policies,
    }
}

fn requires_approval(endpoint: &AgentEndpointIr) -> bool {
    matches!(endpoint.risk, AgentEndpointRiskIr::High)
        || !matches!(endpoint.approval, AgentEndpointApprovalModeIr::None)
        || !endpoint.side_effects.is_empty()
}

fn export_surfaces(endpoint: &AgentEndpointIr) -> Vec<String> {
    let mut surfaces = Vec::new();
    if endpoint.agent_visibility.openapi {
        surfaces.push("openapi".to_string());
    }
    if endpoint.agent_visibility.arazzo {
        surfaces.push("arazzo".to_string());
    }
    if endpoint.agent_visibility.mcp {
        surfaces.push("mcp".to_string());
    }
    if endpoint.agent_visibility.llms_txt {
        surfaces.push("llms_txt".to_string());
    }
    surfaces
}

fn risk_label(risk: &AgentEndpointRiskIr) -> &'static str {
    match risk {
        AgentEndpointRiskIr::Low => "low",
        AgentEndpointRiskIr::Medium => "medium",
        AgentEndpointRiskIr::High => "high",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use greentic_sorla_ir::{AgentEndpointBackingIr, AgentEndpointVisibilityIr};

    fn endpoint(id: &str, risk: AgentEndpointRiskIr) -> AgentEndpointIr {
        AgentEndpointIr {
            id: id.to_string(),
            title: id.to_string(),
            intent: String::new(),
            description: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            side_effects: Vec::new(),
            risk,
            approval: AgentEndpointApprovalModeIr::None,
            provider_requirements: Vec::new(),
            backing: AgentEndpointBackingIr {
                actions: Vec::new(),
                events: Vec::new(),
                flows: Vec::new(),
                policies: Vec::new(),
                approvals: Vec::new(),
            },
            agent_visibility: AgentEndpointVisibilityIr {
                openapi: true,
                arazzo: false,
                mcp: true,
                llms_txt: false,
            },
            examples: Vec::new(),
            emits: None,
        }
    }

    #[test]
    fn default_policy_is_conservative() {
        let policy = generate_sorx_exposure_policy(&[endpoint(
            "create_contact",
            AgentEndpointRiskIr::Medium,
        )]);
        assert_eq!(policy.default_visibility, EndpointVisibility::Private);
        assert_eq!(
            policy.endpoints[0].visibility,
            EndpointVisibility::PublicCandidate
        );
        assert_eq!(policy.endpoints[0].export_surfaces, ["openapi", "mcp"]);
    }

    #[test]
    fn high_risk_endpoint_requires_approval() {
        let policy =
            generate_sorx_exposure_policy(&[endpoint("delete_contact", AgentEndpointRiskIr::High)]);
        let known = BTreeSet::from(["delete_contact"]);
        assert!(policy.endpoints[0].requires_approval);
        policy.validate_static(&known).expect("policy validates");
    }

    #[test]
    fn rejects_public_candidate_default() {
        let mut policy = generate_sorx_exposure_policy(&[]);
        policy.default_visibility = EndpointVisibility::PublicCandidate;
        let err = policy
            .validate_static(&BTreeSet::new())
            .expect_err("public candidate default should fail");
        assert!(err.message().contains("default_visibility"));
    }

    #[test]
    fn rejects_unknown_endpoint() {
        let policy =
            generate_sorx_exposure_policy(&[endpoint("create_contact", AgentEndpointRiskIr::Low)]);
        let err = policy
            .validate_static(&BTreeSet::new())
            .expect_err("unknown endpoint should fail");
        assert!(err.message().contains("unknown endpoint"));
    }
}
