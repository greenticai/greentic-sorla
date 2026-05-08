use std::collections::{BTreeMap, BTreeSet};

use greentic_sorla_ir::{CanonicalIr, MigrationBackfillIr, canonical_hash_hex, lower_package};
use greentic_sorla_lang::parser::parse_package;
use greentic_sorla_pack::{build_artifacts_from_yaml, export_agent_artifacts};
use provider_foundationdb::{FoundationDbConfig, FoundationDbProvider};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sorla_provider_core::{
    AppendEventRequest, ConfigValidator, EventRecord, EventStoreProvider, EventStreamRequest,
    HealthState, PersistProjectionRequest, ProjectionProvider, ProjectionRebuildRequest,
    ProviderError, ProviderHealth,
};

const V1_SCHEMA: &str = include_str!("../../../tests/e2e/fixtures/landlord_sor_v1.yaml");
const V2_SCHEMA: &str = include_str!("../../../tests/e2e/fixtures/landlord_sor_v2.yaml");
const SEED_DATA: &str = include_str!("../../../tests/e2e/fixtures/landlord_seed_data.json");
const STREAM_ID: &str = "landlord-tenant-sor/landlord-1";
const PORTFOLIO_PROJECTION: &str = "LandlordPortfolio";
const PORTFOLIO_KEY: &str = "landlord-1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct E2eOptions {
    pub smoke: bool,
}

impl E2eOptions {
    pub fn full() -> Self {
        Self { smoke: false }
    }

    pub fn smoke() -> Self {
        Self { smoke: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct E2eReport {
    pub events_written: usize,
    pub active_tenants: usize,
    pub schema_v1_hash: String,
    pub schema_v2_hash: String,
    pub smoke: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct SeedData {
    landlords: Vec<Value>,
    properties: Vec<Value>,
    units: Vec<Value>,
    tenants: Vec<Value>,
    tenancies: Vec<Value>,
    payments: Vec<Value>,
    maintenance_requests: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
struct PortfolioProjection {
    landlords: BTreeMap<String, Value>,
    properties: BTreeMap<String, Value>,
    units: BTreeMap<String, Value>,
    tenants: BTreeMap<String, Value>,
    tenancies: BTreeMap<String, Value>,
    payments: BTreeMap<String, Value>,
    maintenance_requests: BTreeMap<String, Value>,
    applied_migrations: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentResult {
    endpoint_id: &'static str,
    status: &'static str,
    data: Value,
}

pub fn run_landlord_tenant_foundationdb(options: E2eOptions) -> Result<E2eReport, String> {
    let v1 = compile_schema(V1_SCHEMA)?;
    let v2 = compile_schema(V2_SCHEMA)?;
    assert_schema_shape(&v1.ir, &v2.ir)?;

    let exports = export_agent_artifacts(&v2.ir);
    let mcp_tools = exports
        .mcp_tools_json
        .as_deref()
        .ok_or_else(|| "expected v2 schema to export MCP tools".to_string())?;
    assert!(mcp_tools.contains("create_tenant"));
    assert!(mcp_tools.contains("assign_tenant_to_unit"));

    let provider = FoundationDbProvider::new(FoundationDbConfig {
        cluster_file: "/tmp/fdb.cluster".into(),
        tenant_prefix: "tenant/landlord-e2e".into(),
    });
    assert_provider_ready(&provider)?;

    let seed = seed_data()?;
    let mut expected_revision = 0;
    let mut projection = PortfolioProjection::default();

    for event in seed_events(&seed, options.smoke)? {
        let record = append_event(&provider, event, expected_revision)?;
        expected_revision = record.revision;
    }

    let events = read_all_events(&provider)?;
    if options.smoke {
        assert!(
            events.len() >= 4,
            "smoke run should write meaningful seed events"
        );
    } else {
        assert!(
            events.len() >= 10,
            "full run should write realistic landlord data"
        );
    }
    apply_events(&mut projection, &events)?;
    assert_seed_projection(&projection, options.smoke)?;
    persist_projection(&provider, &projection, expected_revision)?;

    let fetched = fetch_projection(&provider)?;
    assert_eq!(projection.tenants.len(), fetched.tenants.len());
    assert_eq!(active_tenants(&fetched).len(), 1);

    apply_v2_migration(&mut projection, &v2.ir)?;
    let once = projection.clone();
    apply_v2_migration(&mut projection, &v2.ir)?;
    assert_eq!(projection, once, "migration should be idempotent");

    persist_projection(&provider, &projection, expected_revision)?;
    let migrated = fetch_projection(&provider)?;
    assert_migrated_projection(&migrated)?;

    let create_sarah = agent_create_tenant(
        &v2.ir,
        &provider,
        &mut projection,
        &mut expected_revision,
        "Sarah Ahmed",
        "sarah@example.com",
        "email",
    )?;
    assert_eq!(create_sarah.endpoint_id, "create_tenant");
    assert_eq!(create_sarah.data["tenant_id"], "tenant-sarah-ahmed");

    let assign_sarah =
        agent_assign_tenant_to_unit(&v2.ir, &provider, &mut projection, &mut expected_revision)?;
    assert_eq!(assign_sarah.endpoint_id, "assign_tenant_to_unit");
    assert_eq!(assign_sarah.data["unit_id"], "unit-2b");

    let payment =
        agent_record_rent_payment(&v2.ir, &provider, &mut projection, &mut expected_revision)?;
    assert_eq!(payment.endpoint_id, "record_rent_payment");
    assert_eq!(payment.data["amount"], 1250);

    let maintenance =
        agent_add_maintenance_request(&v2.ir, &provider, &mut projection, &mut expected_revision)?;
    assert_eq!(maintenance.endpoint_id, "add_maintenance_request");
    assert_eq!(maintenance.data["summary"], "heating not working");

    let active = agent_list_active_tenants(&projection)?;
    assert_eq!(active.endpoint_id, "list_active_tenants");
    assert!(
        active.data["active_tenants"]
            .as_array()
            .expect("active tenants should be an array")
            .iter()
            .any(|tenant| tenant["full_name"] == "Sarah Ahmed")
    );

    let contact = agent_update_contact_preference(
        &v2.ir,
        &provider,
        &mut projection,
        &mut expected_revision,
        "email",
    )?;
    assert_eq!(contact.endpoint_id, "update_tenant_contact_preference");
    assert_eq!(
        projection.tenants["tenant-sarah-ahmed"]["preferred_contact_method"],
        "email"
    );

    let invalid = agent_create_tenant(
        &v2.ir,
        &provider,
        &mut projection,
        &mut expected_revision,
        "",
        "invalid@example.com",
        "email",
    );
    assert_useful_error(invalid, "full_name is required");

    persist_projection(&provider, &projection, expected_revision)?;
    let final_projection = fetch_projection(&provider)?;
    assert_eq!(
        final_projection.tenants["tenant-sarah-ahmed"]["preferred_contact_method"],
        "email"
    );

    let checkpoint = provider
        .rebuild_projection(ProjectionRebuildRequest {
            projection_name: PORTFOLIO_PROJECTION.into(),
            from_checkpoint: None,
        })
        .map_err(provider_error)?;
    assert_eq!(
        checkpoint.checkpoint_token,
        format!("{PORTFOLIO_PROJECTION}@{expected_revision}")
    );

    Ok(E2eReport {
        events_written: expected_revision as usize,
        active_tenants: active_tenants(&final_projection).len(),
        schema_v1_hash: canonical_hash_hex(&v1.ir),
        schema_v2_hash: canonical_hash_hex(&v2.ir),
        smoke: options.smoke,
    })
}

struct CompiledSchema {
    ir: greentic_sorla_ir::CanonicalIr,
}

fn compile_schema(source: &str) -> Result<CompiledSchema, String> {
    let parsed = parse_package(source)?;
    let ir = lower_package(&parsed.package);
    let built = build_artifacts_from_yaml(source)?;

    assert_eq!(built.canonical_hash, canonical_hash_hex(&ir));
    assert!(built.cbor_artifacts.contains_key("model.cbor"));
    assert!(built.inspect_json.contains("\"agent_endpoints\""));
    assert!(
        built
            .executable_contract_json
            .contains("greentic.sorla.executable-contract.v1")
    );

    Ok(CompiledSchema { ir })
}

fn assert_schema_shape(v1: &CanonicalIr, v2: &CanonicalIr) -> Result<(), String> {
    let v1_records = record_names(v1);
    let v2_records = record_names(v2);
    for expected in [
        "Landlord",
        "Property",
        "Unit",
        "Tenant",
        "Tenancy",
        "Payment",
        "MaintenanceRequest",
    ] {
        if !v1_records.contains(expected) || !v2_records.contains(expected) {
            return Err(format!("missing expected record `{expected}`"));
        }
    }

    let tenant_v1 = record_field_names(v1, "Tenant")?;
    let tenant_v2 = record_field_names(v2, "Tenant")?;
    assert!(!tenant_v1.contains("preferred_contact_method"));
    assert!(tenant_v2.contains("preferred_contact_method"));
    assert!(record_field_names(v2, "Unit")?.contains("energy_rating"));
    assert!(record_field_names(v2, "Tenancy")?.contains("deposit_amount"));
    assert_eq!(
        record_field_reference(v2, "Property", "landlord_id")?,
        ("Landlord".to_string(), "id".to_string())
    );
    assert_eq!(
        record_field_reference(v2, "Tenancy", "unit_id")?,
        ("Unit".to_string(), "id".to_string())
    );
    assert_eq!(
        record_field_reference(v2, "Payment", "tenancy_id")?,
        ("Tenancy".to_string(), "id".to_string())
    );
    assert_eq!(v1.agent_endpoints.len(), 6);
    assert_eq!(v2.agent_endpoints.len(), 6);
    let migration = migration(v2, "landlord-tenant-v2-fields")?;
    assert_eq!(
        migration.idempotence_key.as_deref(),
        Some("landlord-tenant-v2-fields")
    );
    assert_eq!(migration.backfills.len(), 11);
    assert_eq!(
        endpoint_emit_event(v2, "create_tenant")?,
        "TenantCreated".to_string()
    );
    assert_eq!(
        endpoint_emit_event(v2, "update_tenant_contact_preference")?,
        "TenantContactPreferenceUpdated".to_string()
    );
    Ok(())
}

fn record_names(ir: &CanonicalIr) -> BTreeSet<&str> {
    ir.records
        .iter()
        .map(|record| record.name.as_str())
        .collect()
}

fn record_field_names(ir: &CanonicalIr, record_name: &str) -> Result<BTreeSet<String>, String> {
    let record = ir
        .records
        .iter()
        .find(|record| record.name == record_name)
        .ok_or_else(|| format!("missing record `{record_name}`"))?;
    Ok(record
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect())
}

fn record_field_reference(
    ir: &CanonicalIr,
    record_name: &str,
    field_name: &str,
) -> Result<(String, String), String> {
    let record = ir
        .records
        .iter()
        .find(|record| record.name == record_name)
        .ok_or_else(|| format!("missing record `{record_name}`"))?;
    let field = record
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .ok_or_else(|| format!("missing field `{record_name}.{field_name}`"))?;
    let reference = field
        .references
        .as_ref()
        .ok_or_else(|| format!("missing reference for `{record_name}.{field_name}`"))?;
    Ok((reference.record.clone(), reference.field.clone()))
}

fn assert_provider_ready(provider: &FoundationDbProvider) -> Result<(), String> {
    let health = provider.health().map_err(provider_error)?;
    assert_eq!(health.state, HealthState::Ready);
    provider
        .validate_config(r#"{"cluster_file":"/tmp/fdb.cluster","tenant_prefix":"tenant/e2e"}"#)
        .map_err(provider_error)?;
    let invalid = provider.validate_config(r#"{"cluster_file":"","tenant_prefix":""}"#);
    assert_useful_error(invalid.map(|_| ()), "cluster_file must not be empty");
    Ok(())
}

fn seed_data() -> Result<SeedData, String> {
    serde_json::from_str(SEED_DATA).map_err(|err| format!("invalid seed data: {err}"))
}

fn seed_events(seed: &SeedData, smoke: bool) -> Result<Vec<DomainEvent>, String> {
    let mut events = Vec::new();
    push_many(&mut events, "LandlordCreated", &seed.landlords);
    push_many(&mut events, "PropertyCreated", &seed.properties);
    push_many(&mut events, "UnitCreated", &seed.units);
    push_many(&mut events, "TenantCreated", &seed.tenants);
    push_many(&mut events, "TenancyCreated", &seed.tenancies);
    if !smoke {
        push_many(&mut events, "PaymentRecorded", &seed.payments);
        push_many(
            &mut events,
            "MaintenanceRequestCreated",
            &seed.maintenance_requests,
        );
    }
    validate_references_from_events(&events)?;
    Ok(events)
}

fn push_many(events: &mut Vec<DomainEvent>, event_type: &'static str, items: &[Value]) {
    events.extend(items.iter().cloned().map(|payload| DomainEvent {
        event_type,
        payload,
    }));
}

#[derive(Debug, Clone)]
struct DomainEvent {
    event_type: &'static str,
    payload: Value,
}

fn append_event(
    provider: &FoundationDbProvider,
    event: DomainEvent,
    expected_revision: u64,
) -> Result<EventRecord, String> {
    provider
        .append_event(AppendEventRequest {
            stream_id: STREAM_ID.into(),
            event_type: event.event_type.into(),
            payload: serde_json::to_string(&event.payload)
                .map_err(|err| format!("event payload serialize failed: {err}"))?,
            expected_revision: Some(expected_revision),
        })
        .map_err(provider_error)
}

fn read_all_events(provider: &FoundationDbProvider) -> Result<Vec<EventRecord>, String> {
    provider
        .read_event_stream(EventStreamRequest {
            stream_id: STREAM_ID.into(),
            from_revision: 1,
            limit: 1000,
        })
        .map_err(provider_error)
}

fn apply_events(
    projection: &mut PortfolioProjection,
    events: &[EventRecord],
) -> Result<(), String> {
    for event in events {
        let payload: Value = serde_json::from_str(&event.payload)
            .map_err(|err| format!("invalid event payload: {err}"))?;
        apply_event(projection, &event.event_type, payload)?;
    }
    validate_projection_references(projection)
}

fn apply_event(
    projection: &mut PortfolioProjection,
    event_type: &str,
    payload: Value,
) -> Result<(), String> {
    match event_type {
        "LandlordCreated" => insert_payload(&mut projection.landlords, payload),
        "PropertyCreated" => insert_payload(&mut projection.properties, payload),
        "UnitCreated" => insert_payload(&mut projection.units, payload),
        "TenantCreated" => insert_payload(&mut projection.tenants, normalize_tenant(payload)),
        "TenancyCreated" => insert_payload(&mut projection.tenancies, payload),
        "PaymentRecorded" => insert_payload(&mut projection.payments, payload),
        "MaintenanceRequestCreated" => {
            insert_payload(&mut projection.maintenance_requests, payload)
        }
        "TenantContactPreferenceUpdated" => {
            let tenant_id = string_field(&payload, "tenant_id")?;
            let method = string_field(&payload, "preferred_contact_method")?;
            let tenant = projection
                .tenants
                .get_mut(&tenant_id)
                .ok_or_else(|| format!("unknown tenant `{tenant_id}`"))?;
            tenant["preferred_contact_method"] = Value::String(method);
            Ok(())
        }
        other => Err(format!("unsupported event type `{other}`")),
    }
}

fn insert_payload(map: &mut BTreeMap<String, Value>, payload: Value) -> Result<(), String> {
    let id = string_field(&payload, "id")?;
    map.insert(id, payload);
    Ok(())
}

fn normalize_tenant(mut payload: Value) -> Value {
    if payload.get("preferred_contact_method").is_none() {
        payload["preferred_contact_method"] = Value::Null;
    }
    payload
}

fn string_field(value: &Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|item| !item.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("{field} is required"))
}

fn number_field(value: &Value, field: &str) -> Result<i64, String> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{field} is required"))
}

fn validate_references_from_events(events: &[DomainEvent]) -> Result<(), String> {
    let mut projection = PortfolioProjection::default();
    for event in events {
        apply_event(&mut projection, event.event_type, event.payload.clone())?;
    }
    validate_projection_references(&projection)
}

fn validate_projection_references(projection: &PortfolioProjection) -> Result<(), String> {
    for property in projection.properties.values() {
        let landlord_id = string_field(property, "landlord_id")?;
        if !projection.landlords.contains_key(&landlord_id) {
            return Err(format!(
                "property references unknown landlord `{landlord_id}`"
            ));
        }
    }
    for unit in projection.units.values() {
        let property_id = string_field(unit, "property_id")?;
        if !projection.properties.contains_key(&property_id) {
            return Err(format!("unit references unknown property `{property_id}`"));
        }
    }
    for tenancy in projection.tenancies.values() {
        let tenant_id = string_field(tenancy, "tenant_id")?;
        let unit_id = string_field(tenancy, "unit_id")?;
        if !projection.tenants.contains_key(&tenant_id) {
            return Err(format!("tenancy references unknown tenant `{tenant_id}`"));
        }
        if !projection.units.contains_key(&unit_id) {
            return Err(format!("tenancy references unknown unit `{unit_id}`"));
        }
    }
    for payment in projection.payments.values() {
        let tenancy_id = string_field(payment, "tenancy_id")?;
        if !projection.tenancies.contains_key(&tenancy_id) {
            return Err(format!("payment references unknown tenancy `{tenancy_id}`"));
        }
    }
    Ok(())
}

fn assert_seed_projection(projection: &PortfolioProjection, smoke: bool) -> Result<(), String> {
    assert_eq!(projection.landlords.len(), 1);
    assert_eq!(projection.properties.len(), 1);
    assert_eq!(projection.units.len(), 2);
    assert_eq!(projection.tenants.len(), 2);
    assert_eq!(active_tenants(projection).len(), 1);
    if !smoke {
        assert_eq!(projection.payments.len(), 1);
        assert_eq!(projection.maintenance_requests.len(), 1);
    }
    Ok(())
}

fn persist_projection(
    provider: &FoundationDbProvider,
    projection: &PortfolioProjection,
    revision: u64,
) -> Result<(), String> {
    provider
        .persist_projection(PersistProjectionRequest {
            projection_name: PORTFOLIO_PROJECTION.into(),
            projection_key: PORTFOLIO_KEY.into(),
            state_json: serde_json::to_string(projection)
                .map_err(|err| format!("projection serialize failed: {err}"))?,
            last_applied_revision: revision,
        })
        .map_err(provider_error)?;
    Ok(())
}

fn fetch_projection(provider: &FoundationDbProvider) -> Result<PortfolioProjection, String> {
    let record = provider
        .get_projection(PORTFOLIO_PROJECTION, PORTFOLIO_KEY)
        .map_err(provider_error)?
        .ok_or_else(|| "missing persisted landlord portfolio projection".to_string())?;
    serde_json::from_str(&record.state_json)
        .map_err(|err| format!("projection deserialize failed: {err}"))
}

fn active_tenants(projection: &PortfolioProjection) -> Vec<Value> {
    projection
        .tenancies
        .values()
        .filter(|tenancy| tenancy["status"] == "active")
        .filter_map(|tenancy| tenancy["tenant_id"].as_str())
        .filter_map(|tenant_id| projection.tenants.get(tenant_id))
        .cloned()
        .collect()
}

fn apply_v2_migration(
    projection: &mut PortfolioProjection,
    ir: &CanonicalIr,
) -> Result<(), String> {
    let migration = migration(ir, "landlord-tenant-v2-fields")?;
    let idempotence_key = migration
        .idempotence_key
        .as_deref()
        .unwrap_or(migration.name.as_str());
    if !projection.applied_migrations.insert(idempotence_key.into()) {
        return Ok(());
    }

    for backfill in &migration.backfills {
        apply_backfill(projection, backfill)?;
    }

    Ok(())
}

fn migration<'a>(
    ir: &'a CanonicalIr,
    name: &str,
) -> Result<&'a greentic_sorla_ir::CompatibilityIr, String> {
    ir.compatibility
        .iter()
        .find(|migration| migration.name == name)
        .ok_or_else(|| format!("missing migration `{name}`"))
}

fn apply_backfill(
    projection: &mut PortfolioProjection,
    backfill: &MigrationBackfillIr,
) -> Result<(), String> {
    match backfill.record.as_str() {
        "Landlord" => backfill_values(&mut projection.landlords, backfill),
        "Property" => backfill_values(&mut projection.properties, backfill),
        "Unit" => backfill_values(&mut projection.units, backfill),
        "Tenant" => backfill_values(&mut projection.tenants, backfill),
        "Tenancy" => backfill_values(&mut projection.tenancies, backfill),
        "Payment" => backfill_values(&mut projection.payments, backfill),
        "MaintenanceRequest" => backfill_values(&mut projection.maintenance_requests, backfill),
        other => Err(format!("unsupported backfill record `{other}`")),
    }
}

fn backfill_values(
    values: &mut BTreeMap<String, Value>,
    backfill: &MigrationBackfillIr,
) -> Result<(), String> {
    for value in values.values_mut() {
        ensure_default(value, &backfill.field, backfill.default.clone());
    }
    Ok(())
}

fn ensure_default(value: &mut Value, field: &str, default: Value) {
    if value.get(field).is_none() {
        value[field] = default;
    }
}

fn endpoint_emit_event(ir: &CanonicalIr, endpoint_id: &str) -> Result<String, String> {
    ir.agent_endpoints
        .iter()
        .find(|endpoint| endpoint.id == endpoint_id)
        .and_then(|endpoint| endpoint.emits.as_ref())
        .map(|emit| emit.event.clone())
        .ok_or_else(|| format!("missing executable emit contract for `{endpoint_id}`"))
}

fn assert_agent_emits(
    ir: &CanonicalIr,
    endpoint_id: &str,
    expected_event: &str,
) -> Result<(), String> {
    let actual = endpoint_emit_event(ir, endpoint_id)?;
    if actual != expected_event {
        return Err(format!(
            "agent endpoint `{endpoint_id}` emits `{actual}` instead of `{expected_event}`"
        ));
    }
    Ok(())
}

fn assert_migrated_projection(projection: &PortfolioProjection) -> Result<(), String> {
    assert!(
        projection
            .applied_migrations
            .contains("landlord-tenant-v2-fields")
    );
    assert!(
        projection.tenants["tenant-alice"]
            .get("date_of_birth")
            .is_some()
    );
    assert!(projection.units["unit-2b"].get("energy_rating").is_some());
    assert!(
        projection.tenancies["tenancy-active-1"]
            .get("deposit_amount")
            .is_some()
    );
    assert_eq!(active_tenants(projection).len(), 1);
    validate_projection_references(projection)
}

fn agent_create_tenant(
    ir: &CanonicalIr,
    provider: &FoundationDbProvider,
    projection: &mut PortfolioProjection,
    expected_revision: &mut u64,
    full_name: &str,
    email: &str,
    preferred_contact_method: &str,
) -> Result<AgentResult, String> {
    assert_agent_emits(ir, "create_tenant", "TenantCreated")?;
    if full_name.trim().is_empty() {
        return Err("create_tenant: full_name is required".into());
    }
    if !email.contains('@') {
        return Err("create_tenant: email must contain @".into());
    }
    if !["email", "phone"].contains(&preferred_contact_method) {
        return Err("create_tenant: preferred_contact_method must be email or phone".into());
    }

    let tenant_id = format!(
        "tenant-{}",
        full_name
            .to_ascii_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
    );
    let payload = json!({
        "id": tenant_id,
        "full_name": full_name,
        "email": email,
        "phone": null,
        "date_of_birth": null,
        "emergency_contact_name": null,
        "emergency_contact_phone": null,
        "preferred_contact_method": preferred_contact_method
    });
    append_and_apply(
        provider,
        projection,
        expected_revision,
        "TenantCreated",
        payload.clone(),
    )?;
    Ok(AgentResult {
        endpoint_id: "create_tenant",
        status: "ok",
        data: json!({"tenant_id": payload["id"]}),
    })
}

fn agent_assign_tenant_to_unit(
    ir: &CanonicalIr,
    provider: &FoundationDbProvider,
    projection: &mut PortfolioProjection,
    expected_revision: &mut u64,
) -> Result<AgentResult, String> {
    assert_agent_emits(ir, "assign_tenant_to_unit", "TenancyCreated")?;
    if !projection.tenants.contains_key("tenant-sarah-ahmed") {
        return Err("assign_tenant_to_unit: tenant does not exist".into());
    }
    if !projection.units.contains_key("unit-2b") {
        return Err("assign_tenant_to_unit: unit does not exist".into());
    }
    let payload = json!({
        "id": "tenancy-sarah-2b",
        "tenant_id": "tenant-sarah-ahmed",
        "unit_id": "unit-2b",
        "start_date": "2026-06-01",
        "end_date": null,
        "status": "active",
        "deposit_amount": null,
        "deposit_scheme_reference": null,
        "renewal_notice_date": null
    });
    append_and_apply(
        provider,
        projection,
        expected_revision,
        "TenancyCreated",
        payload.clone(),
    )?;
    Ok(AgentResult {
        endpoint_id: "assign_tenant_to_unit",
        status: "ok",
        data: json!({"tenancy_id": payload["id"], "unit_id": payload["unit_id"]}),
    })
}

fn agent_record_rent_payment(
    ir: &CanonicalIr,
    provider: &FoundationDbProvider,
    projection: &mut PortfolioProjection,
    expected_revision: &mut u64,
) -> Result<AgentResult, String> {
    assert_agent_emits(ir, "record_rent_payment", "PaymentRecorded")?;
    let tenancy = projection
        .tenancies
        .get("tenancy-sarah-2b")
        .ok_or_else(|| "record_rent_payment: active tenancy does not exist".to_string())?;
    assert_eq!(tenancy["status"], "active");
    let payload = json!({
        "id": "payment-sarah-2026-06",
        "tenancy_id": "tenancy-sarah-2b",
        "amount": 1250,
        "paid_on": "2026-06-01",
        "status": "paid"
    });
    assert_eq!(number_field(&payload, "amount")?, 1250);
    append_and_apply(
        provider,
        projection,
        expected_revision,
        "PaymentRecorded",
        payload.clone(),
    )?;
    Ok(AgentResult {
        endpoint_id: "record_rent_payment",
        status: "ok",
        data: json!({"payment_id": payload["id"], "amount": payload["amount"]}),
    })
}

fn agent_add_maintenance_request(
    ir: &CanonicalIr,
    provider: &FoundationDbProvider,
    projection: &mut PortfolioProjection,
    expected_revision: &mut u64,
) -> Result<AgentResult, String> {
    assert_agent_emits(ir, "add_maintenance_request", "MaintenanceRequestCreated")?;
    if !projection.units.contains_key("unit-2b") {
        return Err("add_maintenance_request: unit does not exist".into());
    }
    let payload = json!({
        "id": "maintenance-heating-2b",
        "unit_id": "unit-2b",
        "tenant_id": "tenant-sarah-ahmed",
        "summary": "heating not working",
        "status": "open"
    });
    append_and_apply(
        provider,
        projection,
        expected_revision,
        "MaintenanceRequestCreated",
        payload.clone(),
    )?;
    Ok(AgentResult {
        endpoint_id: "add_maintenance_request",
        status: "ok",
        data: json!({"maintenance_request_id": payload["id"], "summary": payload["summary"]}),
    })
}

fn agent_list_active_tenants(projection: &PortfolioProjection) -> Result<AgentResult, String> {
    Ok(AgentResult {
        endpoint_id: "list_active_tenants",
        status: "ok",
        data: json!({"active_tenants": active_tenants(projection)}),
    })
}

fn agent_update_contact_preference(
    ir: &CanonicalIr,
    provider: &FoundationDbProvider,
    projection: &mut PortfolioProjection,
    expected_revision: &mut u64,
    method: &str,
) -> Result<AgentResult, String> {
    assert_agent_emits(
        ir,
        "update_tenant_contact_preference",
        "TenantContactPreferenceUpdated",
    )?;
    if !["email", "phone"].contains(&method) {
        return Err("update_tenant_contact_preference: invalid method".into());
    }
    let payload = json!({
        "tenant_id": "tenant-sarah-ahmed",
        "preferred_contact_method": method
    });
    append_and_apply(
        provider,
        projection,
        expected_revision,
        "TenantContactPreferenceUpdated",
        payload.clone(),
    )?;
    Ok(AgentResult {
        endpoint_id: "update_tenant_contact_preference",
        status: "ok",
        data: json!({"tenant_id": "tenant-sarah-ahmed", "preferred_contact_method": method}),
    })
}

fn append_and_apply(
    provider: &FoundationDbProvider,
    projection: &mut PortfolioProjection,
    expected_revision: &mut u64,
    event_type: &'static str,
    payload: Value,
) -> Result<(), String> {
    let record = append_event(
        provider,
        DomainEvent {
            event_type,
            payload: payload.clone(),
        },
        *expected_revision,
    )?;
    *expected_revision = record.revision;
    apply_event(projection, event_type, payload)?;
    validate_projection_references(projection)
}

fn assert_useful_error<T, E: std::fmt::Display>(result: Result<T, E>, expected: &str) {
    let err = result
        .err()
        .expect("operation should have failed with a useful error")
        .to_string();
    assert!(
        err.contains(expected),
        "expected error to contain `{expected}`, got `{err}`"
    );
}

fn provider_error(err: ProviderError) -> String {
    err.to_string()
}

#[cfg(test)]
mod tests {
    use super::{E2eOptions, run_landlord_tenant_foundationdb};

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
}
