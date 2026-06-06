//! Wasm component glue for the SoRLa Designer extension.
//!
//! This module is compiled only for `wasm32` (gated by the `#[cfg]` on the
//! `mod component` declaration in `lib.rs`). It wires the generated WIT world
//! exports to the existing pure-Rust JSON-boundary functions in `crate::*`, and
//! provides the [`HostLlm`] adapter that backs the prompt-authoring engine with
//! the designer host's per-tenant LLM import.

use crate::bindings::exports::greentic::extension_base::{lifecycle, manifest};
use crate::bindings::exports::greentic::extension_design::{
    knowledge, prompting, tools, validation,
};
use crate::bindings::greentic::extension_base::types;
use crate::bindings::greentic::extension_host::llm as host_llm;
use greentic_sorla_lib::prompt::llm::{
    LlmCapability, LlmRequest, LlmResponse, LlmResponseFormat, LlmRole,
};

/// Marker type the generated `bindings::export!` macro attaches Guest impls to.
pub(crate) struct Component;

// --- extension-base: manifest + lifecycle ----------------------------------

impl manifest::Guest for Component {
    fn get_identity() -> types::ExtensionIdentity {
        let manifest = crate::extension_manifest();
        types::ExtensionIdentity {
            id: manifest.metadata.id.to_string(),
            version: manifest.metadata.version.to_string(),
            kind: types::Kind::Design,
        }
    }

    fn get_offered() -> Vec<types::CapabilityRef> {
        // Mirror the JSON manifest's offered capabilities so the host registry
        // sees the same set whether it reads describe.json or queries the
        // component directly.
        vec![
            types::CapabilityRef {
                id: "greentic:sorla/design".to_string(),
                version: "1.0.0".to_string(),
            },
            types::CapabilityRef {
                id: "greentic:sorla/patch".to_string(),
                version: "1.0.0".to_string(),
            },
        ]
    }

    fn get_required() -> Vec<types::CapabilityRef> {
        Vec::new()
    }
}

impl lifecycle::Guest for Component {
    fn init(_config_json: String) -> Result<(), types::ExtensionError> {
        Ok(())
    }

    fn shutdown() {}
}

// --- extension-design: tools ------------------------------------------------

impl tools::Guest for Component {
    fn list_tools() -> Vec<tools::ToolDefinition> {
        crate::list_tools()
            .into_iter()
            .map(|tool| tools::ToolDefinition {
                name: tool.name.to_string(),
                description: tool.description.to_string(),
                input_schema_json: tool.input_schema_json,
                output_schema_json: tool.output_schema_json,
                // chat-vs-studio split: studio-only tools advertise "studio" so
                // the designer's chat tool-def builder filters them out.
                capabilities: Some(crate::tool_runtime_contexts(tool.name)),
                agentic_worker_metadata: None,
            })
            .collect()
    }

    fn invoke_tool(name: String, args_json: String) -> Result<String, types::ExtensionError> {
        // Reuse the native JSON dispatch; map the stringly error into the WIT
        // variant the same way the reference llm-openai extension does.
        crate::invoke_tool(&name, &args_json).map_err(types::ExtensionError::InvalidInput)
    }
}

// --- extension-design: validation -------------------------------------------

impl validation::Guest for Component {
    fn validate_content(content_type: String, content_json: String) -> validation::ValidateResult {
        let report = crate::validate_content(&content_type, &content_json);
        let valid = report
            .get("valid")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let diagnostics = report
            .get("diagnostics")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().map(diagnostic_from_json).collect())
            .unwrap_or_default();
        validation::ValidateResult { valid, diagnostics }
    }
}

// --- extension-design: prompting --------------------------------------------

impl prompting::Guest for Component {
    fn system_prompt_fragments() -> Vec<prompting::PromptFragment> {
        crate::system_prompt_fragments()
            .into_iter()
            .map(|fragment| prompting::PromptFragment {
                section: fragment.id.to_string(),
                content_markdown: fragment.content.to_string(),
                priority: u32::from(fragment.priority),
            })
            .collect()
    }
}

// --- extension-design: knowledge --------------------------------------------

impl knowledge::Guest for Component {
    fn list_entries(category_filter: Option<String>) -> Vec<knowledge::EntrySummary> {
        crate::list_entries()
            .into_iter()
            .filter(|entry| match &category_filter {
                Some(filter) => entry.category == filter,
                None => true,
            })
            .map(entry_summary)
            .collect()
    }

    fn get_entry(id: String) -> Result<knowledge::Entry, types::ExtensionError> {
        match crate::get_entry(&id) {
            Some(entry) => Ok(knowledge::Entry {
                id: entry.id.to_string(),
                title: entry.title.to_string(),
                category: entry.category.to_string(),
                tags: entry.tags.iter().map(|tag| (*tag).to_string()).collect(),
                content_json: entry.content_json.to_string(),
            }),
            None => Err(types::ExtensionError::InvalidInput(format!(
                "knowledge entry '{id}' not found"
            ))),
        }
    }

    fn suggest_entries(query: String, limit: u32) -> Vec<knowledge::EntrySummary> {
        crate::suggest_entries(&query, limit as usize)
            .into_iter()
            .map(entry_summary)
            .collect()
    }
}

fn entry_summary(entry: crate::KnowledgeEntry) -> knowledge::EntrySummary {
    knowledge::EntrySummary {
        id: entry.id.to_string(),
        title: entry.title.to_string(),
        category: entry.category.to_string(),
        tags: entry.tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}

/// Map a JSON diagnostic emitted by the native validators into the WIT record.
/// The native validators stringly-encode severity; anything other than the
/// known severities is treated as an error so problems are never silently
/// downgraded.
fn diagnostic_from_json(value: &serde_json::Value) -> types::Diagnostic {
    let severity = match value.get("severity").and_then(serde_json::Value::as_str) {
        Some("warning") => types::Severity::Warning,
        Some("info") => types::Severity::Info,
        Some("hint") => types::Severity::Hint,
        _ => types::Severity::Error,
    };
    types::Diagnostic {
        severity,
        code: value
            .get("code")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        message: value
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        path: value
            .get("path")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
    }
}

// --- HostLlm adapter --------------------------------------------------------

/// `LlmCapability` backed by the designer host's `llm` import. Credentials and
/// provider/model selection are host-owned (resolved from the extension's
/// declared `sorla_composer` role per tenant) — `request.provider`/`api_key`/
/// `endpoint` are intentionally ignored.
pub struct HostLlm;

impl LlmCapability for HostLlm {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, greentic_sorla_lib::SorlaError> {
        let messages = request
            .messages
            .iter()
            .map(|message| host_llm::LlmMessage {
                role: match message.role {
                    LlmRole::System => "system".to_string(),
                    LlmRole::User => "user".to_string(),
                    LlmRole::Assistant => "assistant".to_string(),
                },
                content: message.content.clone(),
            })
            .collect();

        let response_format = request.response_format.map(|format| match format {
            LlmResponseFormat::Text => host_llm::ResponseFormat::Text,
            LlmResponseFormat::Json => host_llm::ResponseFormat::Json,
            // The host's json-schema variant carries the serialized schema
            // document; we only have a `Value`, so serialize it back to a
            // string. The `name`/`strict` hints are designer-side and have no
            // host-side counterpart in the current contract.
            LlmResponseFormat::JsonSchema { schema, .. } => {
                host_llm::ResponseFormat::JsonSchema(schema.to_string())
            }
        });

        let host_request = host_llm::LlmRequest {
            // The host selects the role from the extension's single declared
            // `sorla_composer` role, so no hint is needed.
            role_hint: None,
            system_prompt: request.system_prompt,
            messages,
            response_format,
        };

        match host_llm::complete(&host_request) {
            // `SorlaError` is a `String`, matching the crate's existing error
            // convention; surface the host failure verbatim with a stable
            // prefix so callers can distinguish host LLM errors.
            Ok(response) => Ok(LlmResponse {
                content: response.content,
                usage: None,
            }),
            Err(message) => Err(format!("host LLM completion failed: {message}")),
        }
    }
}

crate::bindings::export!(Component with_types_in crate::bindings);
