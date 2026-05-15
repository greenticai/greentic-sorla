use crate::sorx_validation::{
    EndpointVisibility, SORX_VALIDATION_SCHEMA, SorxValidationManifest, SorxValidationPackageRef,
    SorxValidationSuite, SorxValidationTest,
};
use greentic_sorla_ir::{
    AgentEndpointApprovalModeIr, AgentEndpointIr, AgentEndpointRiskIr, CanonicalIr,
    ProviderRequirementIr,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct SorxValidationGenerationInput<'a> {
    pub package_name: &'a str,
    pub package_version: &'a str,
    pub ir_version: Option<&'a str>,
    pub ir_hash: Option<&'a str>,
    pub agent_endpoints: &'a [AgentEndpointIr],
    pub provider_requirements: Vec<ProviderRequirementIr>,
    pub sorx_startup_asset_names: Vec<String>,
    pub has_ontology: bool,
    pub has_retrieval_bindings: bool,
}

pub fn generate_sorx_validation_manifest_from_ir(
    ir: &CanonicalIr,
    ir_hash: Option<&str>,
    sorx_startup_asset_names: Vec<String>,
) -> SorxValidationManifest {
    let ir_version = format!("{}.{}", ir.ir_version.major, ir.ir_version.minor);
    generate_sorx_validation_manifest(SorxValidationGenerationInput {
        package_name: &ir.package.name,
        package_version: &ir.package.version,
        ir_version: Some(&ir_version),
        ir_hash,
        agent_endpoints: &ir.agent_endpoints,
        provider_requirements: aggregated_provider_requirements(ir),
        sorx_startup_asset_names,
        has_ontology: ir.ontology.is_some(),
        has_retrieval_bindings: ir.retrieval_bindings.is_some(),
    })
}

pub fn generate_sorx_validation_manifest(
    input: SorxValidationGenerationInput<'_>,
) -> SorxValidationManifest {
    let mut suites = Vec::new();
    suites.push(smoke_suite(&input.sorx_startup_asset_names));

    let exported_endpoints = exported_endpoints(input.agent_endpoints);
    if !exported_endpoints.is_empty() {
        suites.push(contract_suite(&exported_endpoints));
    }
    if input.has_ontology {
        suites.push(ontology_suite(!exported_endpoints.is_empty()));
    }
    if input.has_retrieval_bindings {
        suites.push(retrieval_suite(!exported_endpoints.is_empty()));
    }

    let provider_requirements = aggregate_requirements(&input.provider_requirements);
    if !provider_requirements.is_empty() {
        suites.push(provider_suite(&provider_requirements));
    }

    if requires_security_suite(input.agent_endpoints) {
        suites.push(security_suite(input.agent_endpoints));
    }

    let promotion_requires = suites
        .iter()
        .filter(|suite| suite.required)
        .map(|suite| suite.id.clone())
        .collect();

    SorxValidationManifest {
        schema: SORX_VALIDATION_SCHEMA.to_string(),
        suite_version: "1.0.0".to_string(),
        package: SorxValidationPackageRef {
            name: input.package_name.to_string(),
            version: input.package_version.to_string(),
            ir_version: input.ir_version.map(str::to_string),
            ir_hash: input.ir_hash.map(str::to_string),
        },
        default_visibility: EndpointVisibility::Private,
        promotion_requires,
        suites,
    }
}

fn smoke_suite(startup_asset_names: &[String]) -> SorxValidationSuite {
    let assets: BTreeSet<_> = startup_asset_names.iter().map(String::as_str).collect();
    let mut tests = vec![healthcheck("runtime-startup-assets-present")];
    if assets.contains("start.schema.json") {
        tests.push(healthcheck("start-schema-present"));
    }
    if assets.contains("provider-bindings.template.yaml") {
        tests.push(healthcheck("provider-bindings-template-present"));
    }
    if assets.contains("runtime.template.yaml") {
        tests.push(healthcheck("runtime-template-present"));
    }
    tests.sort_by(|left, right| left.id().cmp(right.id()));

    SorxValidationSuite {
        id: "smoke".to_string(),
        title: Some("Static startup handoff checks".to_string()),
        required: true,
        tests,
    }
}

fn contract_suite(endpoints: &[&AgentEndpointIr]) -> SorxValidationSuite {
    let mut tests = endpoints
        .iter()
        .map(|endpoint| SorxValidationTest::AgentEndpoint {
            id: format!("agent-endpoint-{}-contract", endpoint.id.replace('_', "-")),
            title: Some(format!("{} contract", endpoint.title)),
            endpoint: Some(endpoint.id.clone()),
            required: Some(true),
            timeout_ms: None,
            input_ref: None,
            expect: None,
        })
        .collect::<Vec<_>>();
    tests.sort_by(|left, right| left.id().cmp(right.id()));

    SorxValidationSuite {
        id: "contract".to_string(),
        title: Some("Agent endpoint contract checks".to_string()),
        required: true,
        tests,
    }
}

fn provider_suite(
    provider_requirements: &BTreeMap<String, BTreeSet<String>>,
) -> SorxValidationSuite {
    let mut tests = provider_requirements
        .iter()
        .map(
            |(category, capabilities)| SorxValidationTest::ProviderCapability {
                id: format!("provider-{}-capabilities", category.replace('_', "-")),
                title: Some(format!("{category} provider capabilities")),
                endpoint: None,
                required: Some(true),
                timeout_ms: None,
                input_ref: None,
                expect: None,
                provider_category: category.clone(),
                capabilities: capabilities.iter().cloned().collect(),
            },
        )
        .collect::<Vec<_>>();
    tests.sort_by(|left, right| left.id().cmp(right.id()));

    SorxValidationSuite {
        id: "provider".to_string(),
        title: Some("Provider capability checks".to_string()),
        required: true,
        tests,
    }
}

fn security_suite(endpoints: &[AgentEndpointIr]) -> SorxValidationSuite {
    let mut tests = vec![
        SorxValidationTest::PolicyEnforced {
            id: "no-secrets-in-pack".to_string(),
            title: Some("Pack contains no embedded secrets".to_string()),
            endpoint: None,
            required: Some(true),
            timeout_ms: None,
            input_ref: None,
            expect: None,
        },
        SorxValidationTest::PolicyEnforced {
            id: "public-exposure-requires-validation".to_string(),
            title: Some("Public exposure requires validation".to_string()),
            endpoint: None,
            required: Some(true),
            timeout_ms: None,
            input_ref: None,
            expect: None,
        },
    ];

    if endpoints
        .iter()
        .any(|endpoint| matches!(endpoint.risk, AgentEndpointRiskIr::High))
    {
        tests.push(SorxValidationTest::PolicyEnforced {
            id: "high-risk-endpoints-require-approval".to_string(),
            title: Some("High-risk endpoints require approval".to_string()),
            endpoint: None,
            required: Some(true),
            timeout_ms: None,
            input_ref: None,
            expect: None,
        });
    }
    tests.sort_by(|left, right| left.id().cmp(right.id()));

    SorxValidationSuite {
        id: "security".to_string(),
        title: Some("Security policy checks".to_string()),
        required: true,
        tests,
    }
}

fn ontology_suite(required: bool) -> SorxValidationSuite {
    let tests = vec![
        SorxValidationTest::OntologyStatic {
            id: "ontology-static".to_string(),
            title: Some("Ontology static metadata is valid".to_string()),
            required: Some(required),
        },
        SorxValidationTest::OntologyRelationship {
            id: "ontology-relationships".to_string(),
            title: Some("Ontology relationships resolve".to_string()),
            required: Some(required),
        },
        SorxValidationTest::OntologyAlias {
            id: "ontology-aliases".to_string(),
            title: Some("Ontology aliases are deterministic".to_string()),
            required: Some(required),
        },
        SorxValidationTest::EntityLinking {
            id: "entity-linking".to_string(),
            title: Some("Entity linking declarations resolve".to_string()),
            required: Some(required),
        },
    ];
    SorxValidationSuite {
        id: "ontology".to_string(),
        title: Some("Ontology handoff checks".to_string()),
        required,
        tests,
    }
}

fn retrieval_suite(required: bool) -> SorxValidationSuite {
    SorxValidationSuite {
        id: "retrieval".to_string(),
        title: Some("Retrieval binding checks".to_string()),
        required,
        tests: vec![SorxValidationTest::RetrievalBinding {
            id: "retrieval-bindings".to_string(),
            title: Some("Retrieval bindings resolve".to_string()),
            required: Some(required),
        }],
    }
}

fn healthcheck(id: &str) -> SorxValidationTest {
    SorxValidationTest::Healthcheck {
        id: id.to_string(),
        title: None,
        endpoint: None,
        required: Some(true),
        timeout_ms: None,
        input_ref: None,
        expect: None,
    }
}

fn exported_endpoints(endpoints: &[AgentEndpointIr]) -> Vec<&AgentEndpointIr> {
    let mut exported = endpoints
        .iter()
        .filter(|endpoint| {
            endpoint.agent_visibility.openapi
                || endpoint.agent_visibility.arazzo
                || endpoint.agent_visibility.mcp
                || endpoint.agent_visibility.llms_txt
        })
        .collect::<Vec<_>>();
    exported.sort_by(|left, right| left.id.cmp(&right.id));
    exported
}

fn requires_security_suite(endpoints: &[AgentEndpointIr]) -> bool {
    endpoints.iter().any(|endpoint| {
        endpoint.agent_visibility.openapi
            || endpoint.agent_visibility.arazzo
            || endpoint.agent_visibility.mcp
            || endpoint.agent_visibility.llms_txt
            || matches!(endpoint.risk, AgentEndpointRiskIr::High)
            || !matches!(endpoint.approval, AgentEndpointApprovalModeIr::None)
            || !endpoint.side_effects.is_empty()
    })
}

fn aggregated_provider_requirements(ir: &CanonicalIr) -> Vec<ProviderRequirementIr> {
    let mut requirements = ir.provider_contract.categories.clone();
    for endpoint in &ir.agent_endpoints {
        requirements.extend(endpoint.provider_requirements.clone());
    }
    if let Some(retrieval) = &ir.retrieval_bindings {
        requirements.extend(
            retrieval
                .providers
                .iter()
                .map(|provider| ProviderRequirementIr {
                    category: provider.category.clone(),
                    capabilities: provider.required_capabilities.clone(),
                }),
        );
    }
    requirements
}

fn aggregate_requirements(
    requirements: &[ProviderRequirementIr],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut categories = BTreeMap::new();
    for requirement in requirements {
        categories
            .entry(requirement.category.clone())
            .or_insert_with(BTreeSet::new)
            .extend(requirement.capabilities.iter().cloned());
    }
    categories
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{START_SCHEMA_FILENAME, build_handoff_artifacts_from_yaml};

    fn startup_assets() -> Vec<String> {
        vec![
            START_SCHEMA_FILENAME.to_string(),
            "provider-bindings.template.yaml".to_string(),
            "runtime.template.yaml".to_string(),
        ]
    }

    #[test]
    fn no_endpoints_generates_smoke_and_provider_suites() {
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
provider_requirements:
  - category: storage
    capabilities:
      - event-log
agent_endpoints: []
"#,
        )
        .expect("fixture should build");

        let manifest = generate_sorx_validation_manifest_from_ir(
            &artifacts.ir,
            Some(&artifacts.canonical_hash),
            startup_assets(),
        );

        assert_eq!(suite_ids(&manifest), vec!["smoke", "provider"]);
        manifest
            .validate_static()
            .expect("generated manifest should validate");
    }

    #[test]
    fn exported_endpoints_generate_contract_and_security_tests() {
        let artifacts = build_handoff_artifacts_from_yaml(
            r#"
package:
  name: agent-pack
  version: 0.1.0
records:
  - name: Contact
    fields:
      - name: id
        type: string
agent_endpoints:
  - id: create_contact
    title: Create contact
    intent: Create a contact.
    risk: medium
    approval: policy-driven
    side_effects:
      - crm.contact.create
    agent_visibility:
      openapi: true
      arazzo: false
      mcp: true
      llms_txt: false
"#,
        )
        .expect("fixture should build");

        let manifest =
            generate_sorx_validation_manifest_from_ir(&artifacts.ir, None, startup_assets());

        assert_eq!(suite_ids(&manifest), vec!["smoke", "contract", "security"]);
        assert!(
            manifest
                .suites
                .iter()
                .find(|suite| suite.id == "contract")
                .expect("contract suite")
                .tests
                .iter()
                .any(|test| test.id() == "agent-endpoint-create-contact-contract")
        );
        manifest
            .validate_static()
            .expect("generated manifest should validate");
    }

    #[test]
    fn provider_requirements_are_aggregated_and_sorted() {
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
    intent: Sync a contact.
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

        let manifest =
            generate_sorx_validation_manifest_from_ir(&artifacts.ir, None, startup_assets());
        let provider_suite = manifest
            .suites
            .iter()
            .find(|suite| suite.id == "provider")
            .expect("provider suite should exist");

        match &provider_suite.tests[0] {
            SorxValidationTest::ProviderCapability {
                provider_category,
                capabilities,
                ..
            } => {
                assert_eq!(provider_category, "crm");
                assert_eq!(capabilities, &["contacts.read", "contacts.write"]);
            }
            other => panic!("unexpected provider test: {other:?}"),
        }
        manifest
            .validate_static()
            .expect("generated manifest should validate");
    }

    #[test]
    fn private_ontology_generates_optional_ontology_suite() {
        let artifacts = build_handoff_artifacts_from_yaml(ontology_fixture(false))
            .expect("ontology fixture should build");

        let manifest =
            generate_sorx_validation_manifest_from_ir(&artifacts.ir, None, startup_assets());

        assert_eq!(suite_ids(&manifest), vec!["smoke", "ontology"]);
        assert_eq!(manifest.promotion_requires, vec!["smoke"]);
        let ontology = manifest
            .suites
            .iter()
            .find(|suite| suite.id == "ontology")
            .expect("ontology suite");
        assert!(!ontology.required);
        assert_eq!(
            ontology
                .tests
                .iter()
                .map(SorxValidationTest::id)
                .collect::<Vec<_>>(),
            vec![
                "ontology-static",
                "ontology-relationships",
                "ontology-aliases",
                "entity-linking"
            ]
        );
        manifest
            .validate_static()
            .expect("generated manifest should validate");
    }

    #[test]
    fn exported_ontology_and_retrieval_generate_promotion_suites() {
        let artifacts = build_handoff_artifacts_from_yaml(ontology_fixture(true))
            .expect("ontology and retrieval fixture should build");

        let manifest =
            generate_sorx_validation_manifest_from_ir(&artifacts.ir, None, startup_assets());

        assert_eq!(
            suite_ids(&manifest),
            vec![
                "smoke",
                "contract",
                "ontology",
                "retrieval",
                "provider",
                "security"
            ]
        );
        assert_eq!(
            manifest.promotion_requires,
            vec![
                "smoke",
                "contract",
                "ontology",
                "retrieval",
                "provider",
                "security"
            ]
        );

        let retrieval = manifest
            .suites
            .iter()
            .find(|suite| suite.id == "retrieval")
            .expect("retrieval suite");
        assert!(retrieval.required);
        assert!(matches!(
            retrieval.tests.as_slice(),
            [SorxValidationTest::RetrievalBinding { id, required, .. }]
                if id == "retrieval-bindings" && required == &Some(true)
        ));

        let provider = manifest
            .suites
            .iter()
            .find(|suite| suite.id == "provider")
            .expect("provider suite");
        assert!(provider.tests.iter().any(|test| matches!(
            test,
            SorxValidationTest::ProviderCapability {
                provider_category,
                capabilities,
                ..
            } if provider_category == "evidence"
                && capabilities == &["entity.link", "evidence.query"]
        )));
        manifest
            .validate_static()
            .expect("generated manifest should validate");
    }

    #[test]
    fn suite_level_legacy_kind_fields_are_rejected() {
        let manifest = serde_json::json!({
            "schema": SORX_VALIDATION_SCHEMA,
            "suite_version": "1.0.0",
            "package": {
                "name": "legacy",
                "version": "0.1.0"
            },
            "default_visibility": "private",
            "promotion_requires": [],
            "suites": [{
                "id": "ontology",
                "kind": "ontology-static",
                "required_for_public_exposure": true,
                "required": false,
                "tests": []
            }]
        });

        let err = serde_json::from_value::<SorxValidationManifest>(manifest)
            .expect_err("legacy suite fields should not deserialize");
        assert!(
            err.to_string().contains("unknown field `kind`")
                || err
                    .to_string()
                    .contains("unknown field `required_for_public_exposure`"),
            "{err}"
        );
    }

    #[test]
    fn generation_is_stable() {
        let artifacts = build_handoff_artifacts_from_yaml(
            r#"
package:
  name: stable-pack
  version: 0.1.0
records:
  - name: Contact
    fields:
      - name: id
        type: string
agent_endpoints:
  - id: beta_endpoint
    title: Beta endpoint
    intent: Beta.
  - id: alpha_endpoint
    title: Alpha endpoint
    intent: Alpha.
"#,
        )
        .expect("fixture should build");

        let first =
            generate_sorx_validation_manifest_from_ir(&artifacts.ir, None, startup_assets());
        let second =
            generate_sorx_validation_manifest_from_ir(&artifacts.ir, None, startup_assets());

        assert_eq!(
            serde_json::to_string_pretty(&first).expect("manifest serializes"),
            serde_json::to_string_pretty(&second).expect("manifest serializes")
        );
    }

    fn suite_ids(manifest: &SorxValidationManifest) -> Vec<&str> {
        manifest
            .suites
            .iter()
            .map(|suite| suite.id.as_str())
            .collect()
    }

    fn ontology_fixture(exported: bool) -> &'static str {
        if exported {
            r#"
package:
  name: ontology-retrieval-demo
  version: 0.1.0
records:
  - name: Customer
    fields:
      - name: id
        type: string
      - name: email
        type: string
  - name: Contract
    fields:
      - name: id
        type: string
  - name: CustomerContract
    fields:
      - name: customer_id
        type: string
      - name: contract_id
        type: string
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
      backed_by:
        record: Customer
    - id: Contract
      kind: entity
      backed_by:
        record: Contract
  relationships:
    - id: has_contract
      from: Customer
      to: Contract
      backed_by:
        record: CustomerContract
        from_field: customer_id
        to_field: contract_id
semantic_aliases:
  concepts:
    Customer:
      - client
entity_linking:
  strategies:
    - id: email_match
      applies_to: Customer
      match:
        source_field: email
        target_field: email
      confidence: 0.95
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
agent_endpoints:
  - id: find_customer
    title: Find customer
    intent: Find a customer.
    risk: high
    approval: policy-driven
    side_effects:
      - customer.lookup
    agent_visibility:
      openapi: true
      arazzo: false
      mcp: true
      llms_txt: false
"#
        } else {
            r#"
package:
  name: ontology-private-demo
  version: 0.1.0
records:
  - name: Customer
    fields:
      - name: id
        type: string
      - name: email
        type: string
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Customer
      kind: entity
      backed_by:
        record: Customer
  relationships: []
semantic_aliases:
  concepts:
    Customer:
      - client
entity_linking:
  strategies:
    - id: email_match
      applies_to: Customer
      match:
        source_field: email
        target_field: email
      confidence: 0.95
agent_endpoints: []
"#
        }
    }
}
