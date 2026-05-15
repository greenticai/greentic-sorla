use greentic_sorla_lib::{
    DEFAULT_DESIGNER_COMPONENT_OPERATION, DEFAULT_DESIGNER_COMPONENT_REF, DesignerNodeType,
    DesignerNodeTypeOptions, NormalizeOptions, NormalizedSorlaModel, PackBuildOptions,
    PreviewOptions, ValidateOptions, build_gtpack_entries, generate_preview,
    list_designer_node_types as list_designer_node_types_from_model, normalize_answers,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignerExtensionManifest {
    pub schema: &'static str,
    pub name: &'static str,
    pub tools: Vec<DesignerTool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignerTool {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromptFragment {
    pub id: &'static str,
    pub priority: u16,
    pub content: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnowledgeEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub tags: Vec<&'static str>,
    pub content_json: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GenerateModelRequest {
    pub prompt: String,
    #[serde(default)]
    pub constraints: GenerateConstraints,
    #[serde(default)]
    pub draft_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GenerateConstraints {
    #[serde(default = "default_true")]
    pub include_ontology: bool,
    #[serde(default = "default_true")]
    pub include_agent_endpoints: bool,
    #[serde(default)]
    pub include_retrieval_bindings: bool,
}

impl Default for GenerateConstraints {
    fn default() -> Self {
        Self {
            include_ontology: true,
            include_agent_endpoints: true,
            include_retrieval_bindings: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolOutput {
    pub status: &'static str,
    pub model: Option<NormalizedSorlaModel>,
    pub diagnostics: Vec<serde_json::Value>,
    pub questions: Vec<String>,
    pub preview: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModelRequest {
    pub model: NormalizedSorlaModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ImproveModelRequest {
    pub model: NormalizedSorlaModel,
    pub instruction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListDesignerNodeTypesRequest {
    pub model: NormalizedSorlaModel,
    #[serde(default = "default_component_ref")]
    pub component_ref: String,
    #[serde(default = "default_operation")]
    pub operation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GenerateFlowNodeRequest {
    pub model: NormalizedSorlaModel,
    #[serde(default)]
    pub node_type_id: Option<String>,
    #[serde(default)]
    pub endpoint_id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    pub step_id: String,
    #[serde(default)]
    pub value_mappings: BTreeMap<String, String>,
    #[serde(default = "default_component_ref")]
    pub component_ref: String,
    #[serde(default = "default_operation")]
    pub operation: String,
}

pub fn extension_manifest() -> DesignerExtensionManifest {
    DesignerExtensionManifest {
        schema: "greentic.designer.extension.adapter.v1",
        name: "greentic-sorla",
        tools: vec![
            DesignerTool {
                name: "generate_model_from_prompt",
                description: "Generate a deterministic Sorla model draft from a prompt.",
            },
            DesignerTool {
                name: "validate_model",
                description: "Validate a Sorla model and return diagnostics plus preview.",
            },
            DesignerTool {
                name: "improve_model",
                description: "Apply deterministic improvements to a Sorla model draft.",
            },
            DesignerTool {
                name: "explain_model",
                description: "Explain the Sorla model as structured sections.",
            },
            DesignerTool {
                name: "generate_gtpack",
                description: "Generate deterministic Sorla .gtpack artifact metadata or pack entries.",
            },
            DesignerTool {
                name: "list_designer_node_types",
                description: "List Designer node types generated from SoRLa agent endpoints.",
            },
            DesignerTool {
                name: "generate_flow_node_from_node_type",
                description: "Generate a generic flow node from a locked SoRLa Designer node type.",
            },
        ],
    }
}

pub fn system_prompt_fragments() -> Vec<PromptFragment> {
    vec![
        PromptFragment {
            id: "sorla.modelling.principles",
            priority: 10,
            content: "When generating Sorla models, prefer deterministic system-of-record structures: records, ontology concepts, relationships, actions, events, projections, policies, approvals, agent endpoints, and retrieval bindings. Never invent provider credentials. Return diagnostics and questions when requirements are ambiguous.",
        },
        PromptFragment {
            id: "sorla.ontology.rules",
            priority: 20,
            content: "Use generic ontology concepts and relationship types. Records describe storage shape; ontology describes business meaning. Relationships must reference existing concepts. Avoid domain-specific core fields unless they belong to the user's domain model.",
        },
        PromptFragment {
            id: "sorla.safety.rules",
            priority: 30,
            content: "Side-effectful actions require risk and approval metadata. Sensitive fields must be marked. High-risk agent endpoints should be approval-driven by default.",
        },
    ]
}

pub fn list_entries() -> Vec<KnowledgeEntry> {
    let entries = vec![
        knowledge_entry(
            "sorla-system-of-record-guide",
            "SoRLa system of record guide",
            &["system-of-record", "records"],
            serde_json::json!({
                "guidance": "Model durable business state as records, then layer events, projections, ontology, and agent endpoints around that state."
            }),
        ),
        knowledge_entry(
            "sorla-ontology-guide",
            "SoRLa ontology guide",
            &["ontology", "relationships"],
            serde_json::json!({
                "guidance": "Use ontology concepts for business meaning and relationships for graph semantics. Back concepts with records when storage shape exists."
            }),
        ),
        knowledge_entry(
            "sorla-agent-endpoint-guide",
            "SoRLa agent endpoint guide",
            &["agent-endpoints", "safety"],
            serde_json::json!({
                "guidance": "Agent endpoints need stable IDs, typed inputs and outputs, risk, approval metadata, and optional export surfaces."
            }),
        ),
        knowledge_entry(
            "sorla-retrieval-binding-guide",
            "SoRLa retrieval binding guide",
            &["retrieval", "evidence"],
            serde_json::json!({
                "guidance": "Retrieval bindings declare abstract evidence provider requirements and ontology scopes. They do not contain provider credentials."
            }),
        ),
        knowledge_entry(
            "sorla-policy-approval-guide",
            "SoRLa policy and approval guide",
            &["policy", "approval", "safety"],
            serde_json::json!({
                "guidance": "Policies and approvals document control points. High-risk or side-effectful operations should be approval-driven."
            }),
        ),
        knowledge_entry(
            "example-supplier-contract-risk",
            "Supplier contract risk example",
            &["example", "supplier", "contract", "risk", "ontology"],
            serde_json::json!({
                "answers": supplier_contract_answers(&GenerateModelRequest {
                    prompt: "supplier contract risk".to_string(),
                    constraints: GenerateConstraints::default(),
                    draft_json: None,
                })
            }),
        ),
        knowledge_entry(
            "example-customer-onboarding",
            "Customer onboarding example",
            &["example", "customer", "onboarding"],
            serde_json::json!({
                "answers": example_answers("customer-onboarding", "Customer", "OnboardingCase")
            }),
        ),
        knowledge_entry(
            "example-landlord-tenant",
            "Landlord tenant example",
            &["example", "landlord", "tenant"],
            serde_json::json!({
                "answers": example_answers("landlord-tenant", "Landlord", "Lease")
            }),
        ),
    ];
    entries
}

pub fn get_entry(id: &str) -> Option<KnowledgeEntry> {
    list_entries().into_iter().find(|entry| entry.id == id)
}

pub fn suggest_entries(query: &str, limit: usize) -> Vec<KnowledgeEntry> {
    let query_tokens = tokenize(query);
    let mut scored = list_entries()
        .into_iter()
        .map(|entry| {
            let tag_score = entry
                .tags
                .iter()
                .filter(|tag| query_tokens.iter().any(|token| token == **tag))
                .count()
                * 100;
            let title_tokens = tokenize(entry.title);
            let title_score = title_tokens
                .iter()
                .filter(|token| query_tokens.contains(token))
                .count()
                * 10;
            let category_score =
                usize::from(query_tokens.iter().any(|token| token == entry.category));
            (tag_score + title_score + category_score, entry)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.id.cmp(right.1.id)));
    scored
        .into_iter()
        .filter(|(score, _)| *score > 0)
        .take(limit)
        .map(|(_, entry)| entry)
        .collect()
}

pub fn generate_model_from_prompt(input: serde_json::Value) -> serde_json::Value {
    let request = match serde_json::from_value::<GenerateModelRequest>(input) {
        Ok(request) => request,
        Err(err) => return error_output("designer.input", err.to_string()),
    };
    if request.prompt.trim().is_empty() {
        return error_output("designer.prompt.empty", "prompt must not be empty");
    }

    let answers = supplier_contract_answers(&request);
    let model = match normalize_answers(answers, NormalizeOptions) {
        Ok(model) => model,
        Err(err) => return error_output("sorla.normalize", err),
    };
    validated_output(model)
}

pub fn validate_model(input: serde_json::Value) -> serde_json::Value {
    let request = match serde_json::from_value::<ModelRequest>(input) {
        Ok(request) => request,
        Err(err) => return error_output("designer.input", err.to_string()),
    };
    validated_output(request.model)
}

pub fn improve_model(input: serde_json::Value) -> serde_json::Value {
    let request = match serde_json::from_value::<ImproveModelRequest>(input) {
        Ok(request) => request,
        Err(err) => return error_output("designer.input", err.to_string()),
    };
    let mut output = validated_output(request.model);
    if let Some(object) = output.as_object_mut() {
        object.insert(
            "changes".to_string(),
            serde_json::json!([{
                "kind": "noted-instruction",
                "instruction": request.instruction
            }]),
        );
    }
    output
}

pub fn explain_model(input: serde_json::Value) -> serde_json::Value {
    let request = match serde_json::from_value::<ModelRequest>(input) {
        Ok(request) => request,
        Err(err) => return error_output("designer.input", err.to_string()),
    };
    let preview = match generate_preview(&request.model, PreviewOptions) {
        Ok(preview) => preview,
        Err(err) => return error_output("sorla.preview", err),
    };
    serde_json::json!({
        "summary": format!(
            "{} {} has {} records and {} agent endpoints.",
            preview.summary.package_name,
            preview.summary.package_version,
            preview.summary.records,
            preview.summary.agent_endpoints
        ),
        "sections": preview.cards,
        "preview": preview
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GenerateGtpackRequest {
    pub model: NormalizedSorlaModel,
    pub package: ArtifactPackage,
    #[serde(default)]
    pub options: ArtifactOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ArtifactPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ArtifactOptions {
    #[serde(default = "default_true")]
    pub include_validation_metadata: bool,
    #[serde(default = "default_true")]
    pub include_designer_preview: bool,
}

impl Default for ArtifactOptions {
    fn default() -> Self {
        Self {
            include_validation_metadata: true,
            include_designer_preview: true,
        }
    }
}

pub fn generate_gtpack(input: serde_json::Value) -> serde_json::Value {
    let request = match serde_json::from_value::<GenerateGtpackRequest>(input) {
        Ok(request) => request,
        Err(err) => return artifact_error("designer.input", err.to_string()),
    };
    let report = greentic_sorla_lib::validate_model(&request.model, ValidateOptions);
    if report.has_errors() {
        return serde_json::json!({
            "artifacts": [],
            "diagnostics": report.diagnostics,
            "preview_json": null
        });
    }
    let preview = generate_preview(&request.model, PreviewOptions).ok();
    let entries = match build_gtpack_entries(
        &request.model,
        PackBuildOptions {
            name: Some(request.package.name.clone()),
            version: Some(request.package.version.clone()),
        },
    ) {
        Ok(entries) => entries,
        Err(err) => return artifact_error("sorla.gtpack.entries", err),
    };
    let entry_metadata = entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "path": entry.path,
                "sha256": entry.sha256,
                "size": entry.bytes.len()
            })
        })
        .collect::<Vec<_>>();
    let empty = Vec::new();
    let records = preview
        .as_ref()
        .map(|preview| preview.summary.records)
        .unwrap_or(0);
    let agent_endpoints = preview
        .as_ref()
        .map(|preview| preview.summary.agent_endpoints)
        .unwrap_or(0);
    serde_json::json!({
        "artifacts": [{
            "kind": "gtpack",
            "filename": format!("{}.gtpack", request.package.name),
            "media_type": "application/vnd.greentic.gtpack",
            "sha256": null,
            "bytes_base64": null,
            "metadata_json": {
                "schema": "greentic.sorla.generated-artifact.v1",
                "pack_id": request.package.name,
                "pack_version": request.package.version,
                "records": records,
                "concepts": 0,
                "relationships": 0,
                "agent_endpoints": agent_endpoints,
                "pack_entries": entry_metadata
            }
        }],
        "diagnostics": [{
            "severity": "warning",
            "code": "sorla.gtpack.host_packaging_required",
            "message": "WASM extension returned deterministic pack entries; host/native packaging must produce ZIP bytes.",
            "path": null,
            "suggestion": "Package the returned entries with the native greentic-sorla-lib pack-zip feature when .gtpack bytes are required."
        }],
        "preview_json": if request.options.include_designer_preview {
            serde_json::to_value(preview).unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        },
        "validation_json": if request.options.include_validation_metadata {
            serde_json::to_value(&report.diagnostics).unwrap_or(serde_json::Value::Array(empty))
        } else {
            serde_json::Value::Null
        }
    })
}

pub fn list_designer_node_types(input: serde_json::Value) -> serde_json::Value {
    let request = match serde_json::from_value::<ListDesignerNodeTypesRequest>(input) {
        Ok(request) => request,
        Err(err) => return node_type_error("designer.input", err.to_string()),
    };
    match list_designer_node_types_from_model(
        &request.model,
        DesignerNodeTypeOptions {
            component_ref: request.component_ref,
            operation: request.operation,
        },
    ) {
        Ok(document) => serde_json::json!({
            "nodeTypes": document.node_types,
            "diagnostics": []
        }),
        Err(err) => node_type_error("sorla.designer_node_types", err),
    }
}

pub fn generate_flow_node_from_node_type(input: serde_json::Value) -> serde_json::Value {
    let request = match serde_json::from_value::<GenerateFlowNodeRequest>(input) {
        Ok(request) => request,
        Err(err) => return flow_node_error("designer.input", err.to_string()),
    };
    let document = match list_designer_node_types_from_model(
        &request.model,
        DesignerNodeTypeOptions {
            component_ref: request.component_ref.clone(),
            operation: request.operation.clone(),
        },
    ) {
        Ok(document) => document,
        Err(err) => return flow_node_error("sorla.designer_node_types", err),
    };
    let node_type = match select_node_type(&document.node_types, &request) {
        Ok(node_type) => node_type,
        Err(output) => return output,
    };

    let required_inputs = required_input_names(node_type);
    let missing = required_inputs
        .iter()
        .filter(|name| !request.value_mappings.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return serde_json::json!({
            "flowNode": null,
            "diagnostics": missing
                .into_iter()
                .map(|name| serde_json::json!({
                    "severity": "error",
                    "code": "designer.mapping.missing_required_input",
                    "message": format!("missing mapping for required input `{name}`"),
                    "path": format!("value_mappings.{name}"),
                    "suggestion": "Map the required endpoint input before generating a flow node."
                }))
                .collect::<Vec<_>>()
        });
    }

    let endpoint_ref = node_type
        .config_schema
        .get("properties")
        .and_then(|properties| properties.get("endpoint_ref"))
        .and_then(|endpoint_ref| endpoint_ref.get("const"))
        .cloned()
        .unwrap_or_else(|| serde_json::to_value(&node_type.metadata.endpoint).unwrap());
    let flow_node = serde_json::json!({
        "schema": "greentic.designer.flow-node.v1",
        "id": request.step_id,
        "type": node_type.id.clone(),
        "binding": node_type.binding.clone(),
        "config": {
            "endpoint_ref": endpoint_ref
        },
        "inputs": request.value_mappings,
        "routing": node_type.default_routing.clone(),
        "metadata": {
            "source": "greentic-sorla-designer-extension",
            "endpoint_id": node_type.metadata.endpoint.id.clone(),
            "package": node_type.metadata.endpoint.package.clone(),
            "version": node_type.metadata.endpoint.version.clone(),
            "contract_hash": node_type.metadata.endpoint.contract_hash.clone(),
            "component_ref": node_type.binding.component.reference.clone(),
            "operation": node_type.binding.operation.clone(),
            "risk": node_type.metadata.risk.clone(),
            "approval": node_type.metadata.approval.clone()
        }
    });
    serde_json::json!({
        "flowNode": flow_node,
        "diagnostics": []
    })
}

fn select_node_type<'a>(
    node_types: &'a [DesignerNodeType],
    request: &GenerateFlowNodeRequest,
) -> Result<&'a DesignerNodeType, serde_json::Value> {
    if let Some(node_type_id) = request.node_type_id.as_deref() {
        return node_types
            .iter()
            .find(|node_type| node_type.id == node_type_id)
            .ok_or_else(|| {
                flow_node_error(
                    "designer.node_type.unknown",
                    format!("unknown node type `{node_type_id}`"),
                )
            });
    }
    if let Some(endpoint_id) = request.endpoint_id.as_deref() {
        return node_types
            .iter()
            .find(|node_type| node_type.metadata.endpoint.id == endpoint_id)
            .ok_or_else(|| {
                flow_node_error(
                    "designer.endpoint.unknown",
                    format!("unknown endpoint `{endpoint_id}`"),
                )
            });
    }
    if let Some(label) = request.label.as_deref() {
        let normalized = label.trim().to_ascii_lowercase();
        let matches = node_types
            .iter()
            .filter(|node_type| node_type.label.to_ascii_lowercase() == normalized)
            .collect::<Vec<_>>();
        return match matches.as_slice() {
            [node_type] => Ok(*node_type),
            [] => Err(flow_node_error(
                "designer.label.unknown",
                format!("unknown node label `{label}`"),
            )),
            _ => Err(flow_node_error(
                "designer.label.ambiguous",
                format!("ambiguous node label `{label}`"),
            )),
        };
    }
    Err(flow_node_error(
        "designer.node_type.selection_missing",
        "provide node_type_id, endpoint_id, or label",
    ))
}

fn required_input_names(node_type: &DesignerNodeType) -> BTreeSet<String> {
    node_type
        .input_schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn node_type_error(code: &str, message: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "nodeTypes": [],
        "diagnostics": [{
            "severity": "error",
            "code": code,
            "message": message.into(),
            "path": null,
            "suggestion": null
        }]
    })
}

fn flow_node_error(code: &str, message: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "flowNode": null,
        "diagnostics": [{
            "severity": "error",
            "code": code,
            "message": message.into(),
            "path": null,
            "suggestion": null
        }]
    })
}

fn artifact_error(code: &str, message: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "artifacts": [],
        "diagnostics": [{
            "severity": "error",
            "code": code,
            "message": message.into(),
            "path": null,
            "suggestion": null
        }],
        "preview_json": null
    })
}

fn validated_output(model: NormalizedSorlaModel) -> serde_json::Value {
    let report = greentic_sorla_lib::validate_model(&model, ValidateOptions);
    let preview = generate_preview(&model, PreviewOptions)
        .ok()
        .and_then(|preview| serde_json::to_value(preview).ok());
    serde_json::json!({
        "status": if report.has_errors() { "needs_input" } else { "valid" },
        "model": model,
        "diagnostics": report.diagnostics,
        "questions": [],
        "preview": preview
    })
}

fn error_output(code: &str, message: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "status": "needs_input",
        "model": null,
        "diagnostics": [{
            "severity": "error",
            "code": code,
            "message": message.into(),
            "path": null,
            "suggestion": null
        }],
        "questions": [],
        "preview": null
    })
}

fn supplier_contract_answers(request: &GenerateModelRequest) -> serde_json::Value {
    let include_agent_endpoints = request.constraints.include_agent_endpoints;
    let include_ontology = request.constraints.include_ontology;
    serde_json::json!({
        "schema_version": "0.5",
        "flow": "create",
        "output_dir": ".",
        "locale": "en",
        "package": {
            "name": "supplier-contract-risk",
            "version": "0.1.0"
        },
        "providers": {
            "storage_category": "storage",
            "external_ref_category": "external-ref"
        },
        "records": {
            "default_source": "native",
            "items": [
                {
                    "name": "Supplier",
                    "fields": [
                        {"name": "id", "type": "string", "required": true},
                        {"name": "name", "type": "string", "required": true},
                        {"name": "risk_rating", "type": "string", "required": false}
                    ]
                },
                {
                    "name": "Contract",
                    "fields": [
                        {"name": "id", "type": "string", "required": true},
                        {"name": "supplier_id", "type": "string", "required": true, "references": {"record": "Supplier", "field": "id"}},
                        {"name": "status", "type": "string", "required": true}
                    ]
                },
                {
                    "name": "EvidenceDocument",
                    "fields": [
                        {"name": "id", "type": "string", "required": true},
                        {"name": "contract_id", "type": "string", "required": true, "references": {"record": "Contract", "field": "id"}},
                        {"name": "uri", "type": "string", "required": true}
                    ]
                }
            ]
        },
        "ontology": if include_ontology {
            serde_json::json!({
                "schema": "greentic.sorla.ontology.v1",
                "concepts": [
                    {"id": "supplier", "kind": "entity", "backed_by": {"record": "Supplier"}},
                    {"id": "contract", "kind": "entity", "backed_by": {"record": "Contract"}},
                    {"id": "evidence_document", "kind": "entity", "backed_by": {"record": "EvidenceDocument"}}
                ],
                "relationships": [
                    {"id": "contract_has_supplier", "from": "contract", "to": "supplier"}
                ]
            })
        } else {
            serde_json::Value::Null
        },
        "agent_endpoints": if include_agent_endpoints {
            serde_json::json!({
                "enabled": true,
                "items": [{
                    "id": "add_evidence",
                    "title": "Add evidence",
                    "intent": "Attach evidence to a supplier contract.",
                    "inputs": [
                        {"name": "contract_id", "type": "string", "required": true},
                        {"name": "uri", "type": "string", "required": true}
                    ],
                    "outputs": [
                        {"name": "evidence_id", "type": "string"}
                    ],
                    "risk": "medium",
                    "approval": "policy-driven"
                }]
            })
        } else {
            serde_json::json!({"enabled": false})
        }
    })
}

fn example_answers(package_name: &str, record_a: &str, record_b: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "0.5",
        "flow": "create",
        "output_dir": ".",
        "locale": "en",
        "package": {
            "name": package_name,
            "version": "0.1.0"
        },
        "providers": {
            "storage_category": "storage"
        },
        "records": {
            "default_source": "native",
            "items": [
                {
                    "name": record_a,
                    "fields": [
                        {"name": "id", "type": "string", "required": true},
                        {"name": "name", "type": "string", "required": true}
                    ]
                },
                {
                    "name": record_b,
                    "fields": [
                        {"name": "id", "type": "string", "required": true},
                        {"name": "status", "type": "string", "required": true}
                    ]
                }
            ]
        }
    })
}

fn knowledge_entry(
    id: &'static str,
    title: &'static str,
    tags: &[&'static str],
    content_json: serde_json::Value,
) -> KnowledgeEntry {
    KnowledgeEntry {
        id,
        title,
        category: "sorla",
        tags: tags.to_vec(),
        content_json,
    }
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn default_true() -> bool {
    true
}

fn default_component_ref() -> String {
    DEFAULT_DESIGNER_COMPONENT_REF.to_string()
}

fn default_operation() -> String {
    DEFAULT_DESIGNER_COMPONENT_OPERATION.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_exports_expected_tools() {
        let tools = extension_manifest()
            .tools
            .into_iter()
            .map(|tool| tool.name)
            .collect::<Vec<_>>();
        assert_eq!(
            tools,
            vec![
                "generate_model_from_prompt",
                "validate_model",
                "improve_model",
                "explain_model",
                "generate_gtpack",
                "list_designer_node_types",
                "generate_flow_node_from_node_type"
            ]
        );
    }

    #[test]
    fn prompt_generation_returns_valid_model() {
        let output = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        assert_eq!(output["status"], "valid");
        assert_eq!(output["model"]["package_name"], "supplier-contract-risk");
        assert!(output["preview"]["summary"]["records"].as_u64().unwrap() >= 3);
    }

    #[test]
    fn validate_model_rejects_invalid_model() {
        let output = validate_model(serde_json::json!({
            "model": {
                "package_name": "bad",
                "package_version": "0.1.0",
                "locale": "en",
                "source_yaml": "not: [valid",
                "normalized_answers": {}
            }
        }));
        assert_eq!(output["status"], "needs_input");
        assert_eq!(output["diagnostics"][0]["severity"], "error");
    }

    #[test]
    fn explain_model_is_stable() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let explained = explain_model(serde_json::json!({
            "model": generated["model"].clone()
        }));
        let first = serde_json::to_string_pretty(&explained).unwrap();
        let second = serde_json::to_string_pretty(&explained).unwrap();
        assert_eq!(first, second);
        assert!(
            explained["summary"]
                .as_str()
                .unwrap()
                .contains("supplier-contract-risk")
        );
    }

    #[test]
    fn prompt_fragments_and_knowledge_are_stable_and_safe() {
        let fragments = system_prompt_fragments();
        assert_eq!(
            fragments
                .iter()
                .map(|fragment| fragment.priority)
                .collect::<Vec<_>>(),
            vec![10, 20, 30]
        );
        let entries = list_entries();
        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0].id, "sorla-system-of-record-guide");
        for text in fragments
            .iter()
            .map(|fragment| fragment.content.to_string())
            .chain(entries.iter().map(|entry| entry.content_json.to_string()))
        {
            let lower = text.to_ascii_lowercase();
            assert!(!lower.contains("secret"));
            assert!(!lower.contains("password"));
            assert!(!lower.contains("tenant_id"));
        }
    }

    #[test]
    fn get_and_suggest_entries_are_deterministic() {
        let entry = get_entry("example-supplier-contract-risk").expect("entry exists");
        assert_eq!(entry.title, "Supplier contract risk example");

        let first = suggest_entries("supplier contract ontology", 3);
        let second = suggest_entries("supplier contract ontology", 3);
        assert_eq!(first, second);
        assert_eq!(first[0].id, "example-supplier-contract-risk");
    }

    #[test]
    fn example_entries_validate_with_sorla_lib() {
        for entry in list_entries() {
            let Some(answers) = entry.content_json.get("answers").cloned() else {
                continue;
            };
            let model = greentic_sorla_lib::normalize_answers(answers, NormalizeOptions)
                .unwrap_or_else(|err| panic!("{} should normalize: {err}", entry.id));
            let report = greentic_sorla_lib::validate_model(&model, ValidateOptions);
            assert!(
                !report.has_errors(),
                "{} should validate: {report:?}",
                entry.id
            );
        }
    }

    #[test]
    fn generate_gtpack_returns_deterministic_artifact_entries() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let artifact = generate_gtpack(serde_json::json!({
            "model": generated["model"].clone(),
            "package": {
                "name": "supplier-contract-risk",
                "version": "0.1.0"
            }
        }));
        assert_eq!(artifact["artifacts"][0]["kind"], "gtpack");
        assert_eq!(
            artifact["artifacts"][0]["metadata_json"]["schema"],
            "greentic.sorla.generated-artifact.v1"
        );
        assert_eq!(
            artifact["artifacts"][0]["bytes_base64"],
            serde_json::Value::Null
        );
        assert_eq!(
            artifact["diagnostics"][0]["code"],
            "sorla.gtpack.host_packaging_required"
        );
        let first = serde_json::to_string_pretty(&artifact).unwrap();
        let second = serde_json::to_string_pretty(&artifact).unwrap();
        assert_eq!(first, second);
        assert!(!first.to_ascii_lowercase().contains("password"));
        assert!(!first.to_ascii_lowercase().contains("tenant_id"));
    }

    #[test]
    fn lists_designer_node_types_from_generated_model() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let output = list_designer_node_types(serde_json::json!({
            "model": generated["model"].clone()
        }));
        assert_eq!(output["diagnostics"].as_array().unwrap().len(), 0);
        assert_eq!(output["nodeTypes"].as_array().unwrap().len(), 1);
        assert_eq!(
            output["nodeTypes"][0]["id"],
            "sorla.agent-endpoint.add_evidence"
        );
        assert!(
            output["nodeTypes"][0]["metadata"]["endpoint"]["contract_hash"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
        );
    }

    #[test]
    fn generates_flow_node_from_node_type_with_locked_endpoint_ref() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let output = generate_flow_node_from_node_type(serde_json::json!({
            "model": generated["model"].clone(),
            "node_type_id": "sorla.agent-endpoint.add_evidence",
            "step_id": "add_evidence_step",
            "value_mappings": {
                "contract_id": "$.state.contract_id",
                "uri": "$.state.uri"
            }
        }));
        assert_eq!(output["diagnostics"].as_array().unwrap().len(), 0);
        assert_eq!(output["flowNode"]["id"], "add_evidence_step");
        assert_eq!(
            output["flowNode"]["config"]["endpoint_ref"]["id"],
            "add_evidence"
        );
        assert!(output.to_string().contains("invoke_locked_action"));
        assert!(!output.to_string().contains("free_text"));
        assert_eq!(
            output["flowNode"]["metadata"]["endpoint_id"],
            "add_evidence"
        );
        assert!(
            output["flowNode"]["metadata"]["contract_hash"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
        );
    }

    #[test]
    fn generates_flow_node_by_endpoint_id_and_label() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let by_endpoint = generate_flow_node_from_node_type(serde_json::json!({
            "model": generated["model"].clone(),
            "endpoint_id": "add_evidence",
            "step_id": "add_evidence_by_endpoint",
            "value_mappings": {
                "contract_id": "$.state.contract_id",
                "uri": "$.state.uri"
            }
        }));
        assert_eq!(by_endpoint["diagnostics"].as_array().unwrap().len(), 0);
        assert_eq!(
            by_endpoint["flowNode"]["type"],
            "sorla.agent-endpoint.add_evidence"
        );

        let by_label = generate_flow_node_from_node_type(serde_json::json!({
            "model": generated["model"].clone(),
            "label": "Add Evidence",
            "step_id": "add_evidence_by_label",
            "value_mappings": {
                "contract_id": "$.state.contract_id",
                "uri": "$.state.uri"
            }
        }));
        assert_eq!(by_label["diagnostics"].as_array().unwrap().len(), 0);
        assert_eq!(
            by_label["flowNode"]["metadata"]["operation"],
            "invoke_locked_action"
        );
    }

    #[test]
    fn flow_node_generation_reports_selection_diagnostics() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let missing = generate_flow_node_from_node_type(serde_json::json!({
            "model": generated["model"].clone(),
            "step_id": "add_evidence_step",
            "value_mappings": {}
        }));
        assert_eq!(missing["flowNode"], serde_json::Value::Null);
        assert_eq!(
            missing["diagnostics"][0]["code"],
            "designer.node_type.selection_missing"
        );

        let unknown_endpoint = generate_flow_node_from_node_type(serde_json::json!({
            "model": generated["model"].clone(),
            "endpoint_id": "unknown_endpoint",
            "step_id": "add_evidence_step",
            "value_mappings": {}
        }));
        assert_eq!(
            unknown_endpoint["diagnostics"][0]["code"],
            "designer.endpoint.unknown"
        );
    }

    #[test]
    fn node_type_label_selection_reports_ambiguous_labels() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let document = list_designer_node_types_from_model(
            &serde_json::from_value(generated["model"].clone()).expect("model parses"),
            DesignerNodeTypeOptions::default(),
        )
        .expect("node types generated");
        let first = document.node_types[0].clone();
        let mut second = first.clone();
        second.id = "sorla.agent-endpoint.add_evidence_duplicate".to_string();
        second.metadata.endpoint.id = "add_evidence_duplicate".to_string();
        let request = GenerateFlowNodeRequest {
            model: serde_json::from_value(generated["model"].clone()).expect("model parses"),
            node_type_id: None,
            endpoint_id: None,
            label: Some(first.label.clone()),
            step_id: "ambiguous".to_string(),
            value_mappings: BTreeMap::new(),
            component_ref: default_component_ref(),
            operation: default_operation(),
        };
        let err = select_node_type(&[first, second], &request).expect_err("label is ambiguous");
        assert_eq!(err["diagnostics"][0]["code"], "designer.label.ambiguous");
    }

    #[test]
    fn flow_node_generation_reports_missing_required_mapping() {
        let generated = generate_model_from_prompt(serde_json::json!({
            "prompt": "Create supplier contract risk management"
        }));
        let output = generate_flow_node_from_node_type(serde_json::json!({
            "model": generated["model"].clone(),
            "node_type_id": "sorla.agent-endpoint.add_evidence",
            "step_id": "add_evidence_step",
            "value_mappings": {
                "contract_id": "$.state.contract_id"
            }
        }));
        assert_eq!(output["flowNode"], serde_json::Value::Null);
        assert_eq!(
            output["diagnostics"][0]["code"],
            "designer.mapping.missing_required_input"
        );
    }

    #[test]
    fn generate_gtpack_refuses_invalid_model() {
        let artifact = generate_gtpack(serde_json::json!({
            "model": {
                "package_name": "bad",
                "package_version": "0.1.0",
                "locale": "en",
                "source_yaml": "not: [valid",
                "normalized_answers": {}
            },
            "package": {
                "name": "bad",
                "version": "0.1.0"
            }
        }));
        assert_eq!(artifact["artifacts"].as_array().unwrap().len(), 0);
        assert_eq!(artifact["diagnostics"][0]["severity"], "error");
    }
}
