use greentic_sorla_designer_extension::{
    generate_flow_node_from_node_type, generate_model_from_prompt, list_designer_node_types,
};
use greentic_sorla_lib::{PackBuildOptions, build_gtpack_entries};

#[test]
fn designer_node_type_to_locked_endpoint() {
    let generated = generate_model_from_prompt(serde_json::json!({
        "prompt": "Create supplier contract risk management"
    }));
    assert_eq!(generated["status"], "valid");
    let model = generated["model"].clone();
    let model: greentic_sorla_lib::NormalizedSorlaModel =
        serde_json::from_value(model.clone()).expect("model should deserialize");

    let entries = build_gtpack_entries(&model, PackBuildOptions::default())
        .expect("pack entries should build");
    let designer_entry = entries
        .iter()
        .find(|entry| entry.path == "assets/sorla/designer-node-types.json")
        .expect("designer node types entry should be emitted");
    let pack_node_types: serde_json::Value =
        serde_json::from_slice(&designer_entry.bytes).expect("node types should parse");
    assert_eq!(
        pack_node_types["schema"],
        "greentic.sorla.designer-node-types.v1"
    );
    assert_eq!(
        pack_node_types["nodeTypes"][0]["id"],
        "sorla.agent-endpoint.add_evidence"
    );
    assert_eq!(
        pack_node_types["nodeTypes"][0]["metadata"]["endpoint"]["id"],
        "add_evidence"
    );
    let contract_hash = pack_node_types["nodeTypes"][0]["metadata"]["endpoint"]["contract_hash"]
        .as_str()
        .expect("contract hash should be present");
    assert!(contract_hash.starts_with("sha256:"));
    assert_eq!(contract_hash.len(), "sha256:".len() + 64);

    let listed = list_designer_node_types(serde_json::json!({
        "model": generated["model"].clone()
    }));
    assert_eq!(listed["diagnostics"].as_array().unwrap().len(), 0);
    assert_eq!(listed["nodeTypes"], pack_node_types["nodeTypes"]);

    let flow = generate_flow_node_from_node_type(serde_json::json!({
        "model": generated["model"].clone(),
        "node_type_id": "sorla.agent-endpoint.add_evidence",
        "step_id": "add_evidence_step",
        "value_mappings": {
            "contract_id": "$.state.contract_id",
            "uri": "$.state.uri"
        }
    }));
    assert_eq!(flow["diagnostics"].as_array().unwrap().len(), 0);
    assert_eq!(
        flow["flowNode"]["config"]["endpoint_ref"]["id"],
        "add_evidence"
    );
    assert_eq!(
        flow["flowNode"]["config"]["endpoint_ref"]["contract_hash"],
        contract_hash
    );
    assert_eq!(flow["flowNode"]["metadata"]["contract_hash"], contract_hash);

    let missing = generate_flow_node_from_node_type(serde_json::json!({
        "model": generated["model"].clone(),
        "node_type_id": "sorla.agent-endpoint.add_evidence",
        "step_id": "add_evidence_step",
        "value_mappings": {
            "contract_id": "$.state.contract_id"
        }
    }));
    assert_eq!(missing["flowNode"], serde_json::Value::Null);
    assert_eq!(
        missing["diagnostics"][0]["code"],
        "designer.mapping.missing_required_input"
    );

    let rendered = serde_json::to_string(&serde_json::json!({
        "nodeTypes": listed["nodeTypes"].clone(),
        "flow": flow["flowNode"].clone()
    }))
    .unwrap();
    for forbidden in [
        "action_label",
        "action_alias",
        "intent_query",
        "natural_language_action",
    ] {
        assert!(!rendered.contains(forbidden), "{forbidden}");
    }
}
