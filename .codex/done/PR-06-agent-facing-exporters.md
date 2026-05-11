# PR-06 — Add Agent-Facing Exporters

## Goal

Generate deterministic agent-facing metadata from canonical IR:

- `agent-gateway.json`
- OpenAPI overlay fragment
- Arazzo workflow document
- MCP tool descriptor JSON
- `llms.txt` fragment

These are export artifacts for downstream assembly, not runtime endpoints.

## Files to touch

- New crate or module decision:
  - Preferred: add module in `crates/greentic-sorla-pack/src/agent_exports.rs`
  - Alternative: new crate `crates/greentic-sorla-agent-export` if code becomes large
- `crates/greentic-sorla-pack/src/lib.rs`
- Tests in `crates/greentic-sorla-pack`
- Possibly add fixtures under `crates/greentic-sorla-pack/fixtures/`

## Current-code notes

- Prefer a `greentic-sorla-pack` module for the first pass. The crate already owns artifact assembly through `build_artifacts_from_yaml()`.
- Existing `ArtifactSet` has `cbor_artifacts`, `inspect_json`, `agent_tools_json`, and `canonical_hash`. Add agent export fields/artifacts there only if the CLI needs to write them immediately; otherwise keep the lower-level export API separate and covered by pack tests.
- Existing public artifact names include `agent-tools.json`. PR-08 names the MCP handoff file `mcp-tools.json`; do not overload the existing `agent-tools.json` helper unless deliberately preserving backward compatibility.
- `serde_yaml` is already a workspace dependency and can be used for YAML overlay/Arazzo serialization.

## Export API

Add:

```rust
pub struct AgentExportSet {
    pub agent_gateway_json: String,
    pub openapi_overlay_yaml: Option<String>,
    pub arazzo_yaml: Option<String>,
    pub mcp_tools_json: Option<String>,
    pub llms_txt: Option<String>,
}

pub fn export_agent_artifacts(ir: &CanonicalIr) -> AgentExportSet
```

## `agent-gateway.json`

This should serialize the manifest from PR-04.

Suggested file shape:

```json
{
  "schema": "greentic.agent-gateway.handoff.v1",
  "package": {
    "name": "website-lead-capture",
    "version": "0.2.0",
    "ir_version": "0.1",
    "ir_hash": "..."
  },
  "endpoints": [
    {
      "id": "create_customer_contact",
      "title": "Create customer contact",
      "intent": "Capture a customer website enquiry and create or update the CRM contact.",
      "risk": "medium",
      "approval": "optional",
      "inputs": ["company_name", "company_size", "email", "problem_to_solve"],
      "outputs": ["contact_id"],
      "side_effects": ["crm.contact.upsert", "event.ContactCaptured"],
      "exports": {
        "openapi": true,
        "arazzo": true,
        "mcp": true,
        "llms_txt": true
      }
    }
  ]
}
```

## OpenAPI overlay fragment

Do not generate a full provider OpenAPI spec. Generate a Greentic overlay that downstream tooling can merge.

Suggested shape:

```yaml
schema: greentic.openapi.agent-overlay.v1
package: website-lead-capture
operations:
  - operationId: agent_create_customer_contact
    x-greentic-agent:
      endpoint_id: create_customer_contact
      intent: Capture a customer website enquiry and create or update the CRM contact.
      risk: medium
      approval: optional
      side_effects:
        - crm.contact.upsert
        - event.ContactCaptured
      inputs:
        - name: email
          type: string
          required: true
          sensitive: true
      outputs:
        - name: contact_id
          type: string
```

## Arazzo document

Generate simple workflows where each endpoint maps to one workflow.

Suggested shape:

```yaml
arazzo: 1.0.1
info:
  title: website-lead-capture agent workflows
  version: 0.2.0
sourceDescriptions: []
workflows:
  - workflowId: create_customer_contact
    summary: Create customer contact
    description: Capture a customer website enquiry and create or update the CRM contact.
    inputs:
      type: object
      required:
        - email
        - company_name
        - problem_to_solve
      properties:
        email:
          type: string
    steps:
      - stepId: request_create_customer_contact
        description: Request downstream Greentic execution for create_customer_contact.
```

Keep this minimal and valid enough for downstream completion. Avoid inventing provider URLs.

## MCP tools descriptor

Write this descriptor to `mcp-tools.json` when emitted. Keep existing `agent-tools.json` behavior separate unless a compatibility alias is explicitly added.

Suggested shape:

```json
{
  "schema": "greentic.mcp.tools.handoff.v1",
  "tools": [
    {
      "name": "create_customer_contact",
      "title": "Create customer contact",
      "description": "Capture a customer website enquiry and create or update the CRM contact.",
      "inputSchema": {
        "type": "object",
        "required": ["email", "company_name", "problem_to_solve"],
        "properties": {
          "email": {
            "type": "string",
            "description": "Sensitive input"
          }
        }
      },
      "annotations": {
        "risk": "medium",
        "approval": "optional",
        "side_effects": ["crm.contact.upsert"]
      }
    }
  ]
}
```

## `llms.txt` fragment

Suggested content:

```txt
# website-lead-capture agent endpoints

This package exposes handoff metadata for business-safe agent endpoints.

## create_customer_contact

Intent: Capture a customer website enquiry and create or update the CRM contact.
Risk: medium
Approval: optional
Side effects: crm.contact.upsert, event.ContactCaptured
Required inputs: email, company_name, problem_to_solve
Outputs: contact_id
```

## Determinism

- Sort endpoints by ID.
- Sort required inputs lexically in generated schemas.
- Sort properties lexically where possible.
- Use stable JSON/YAML serialization.

## Tests

Add snapshot-style tests without brittle whitespace where possible:

1. Export set includes only enabled export targets.
2. MCP tool schema includes required inputs.
3. OpenAPI overlay includes risk/approval/side effects.
4. Arazzo export includes one workflow per visible endpoint.
5. `llms.txt` includes endpoint intent and safety metadata.
6. Output is deterministic across repeated calls.

## Acceptance criteria

- `cargo test -p greentic-sorla-pack` passes.
- No provider-specific URLs are invented.
- No runtime server is introduced.
- Exported files are clearly marked as handoff metadata.
