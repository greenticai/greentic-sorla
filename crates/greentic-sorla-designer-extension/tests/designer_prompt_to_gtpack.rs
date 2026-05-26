use greentic_sorla_designer_extension::{
    generate_gtpack, generate_model_from_prompt, validate_model,
};

#[test]
fn designer_prompt_to_gtpack() {
    let prompt =
        include_str!("../../../tests/e2e/fixtures/designer_supplier_contract_risk_prompt.txt");
    let generated = generate_model_from_prompt(serde_json::json!({ "prompt": prompt }));
    assert_eq!(generated["status"], "valid");

    let validated = validate_model(serde_json::json!({
        "model": generated["model"].clone()
    }));
    assert_eq!(validated["status"], "valid");
    assert!(validated["preview"]["summary"]["records"].as_u64().unwrap() >= 3);

    let artifact = generate_gtpack(serde_json::json!({
        "model": generated["model"].clone(),
        "package": {
            "name": "supplier-contract-risk",
            "version": "0.1.0"
        }
    }));
    assert_eq!(artifact["schema"], "greentic.sorla.gtpack-plan.v1");
    assert!(
        artifact["pack_entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["path"] == "assets/sorla/model.cbor")
    );
    assert_eq!(
        artifact["diagnostics"][0]["code"],
        "sorla.gtpack.host_packaging_required"
    );

    let rendered = serde_json::to_string_pretty(&artifact).unwrap();
    assert!(!rendered.contains("/tmp/"));
    assert!(!rendered.to_ascii_lowercase().contains("password"));
    assert!(!rendered.to_ascii_lowercase().contains("tenant_id"));
}
