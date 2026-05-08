use greentic_sorla_e2e::landlord_tenant::{E2eOptions, run_landlord_tenant_foundationdb};

#[test]
fn landlord_tenant_foundationdb() {
    let smoke = std::env::var("SORLA_E2E_SMOKE")
        .map(|value| value == "1" || value == "true")
        .unwrap_or(false);
    let report = run_landlord_tenant_foundationdb(E2eOptions { smoke })
        .expect("landlord tenant FoundationDB e2e should pass");

    assert!(report.events_written >= if smoke { 4 } else { 14 });
    assert!(report.active_tenants >= 2);
    assert_ne!(report.schema_v1_hash, report.schema_v2_hash);
}
