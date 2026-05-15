# PR 15 — Emit Designer node types from SoRLa agent endpoints

## Repository

`greenticai/greentic-sorla`

## Objective

Generate Designer node type metadata from existing SoRLa agent endpoints so Designer can add business-friendly nodes such as "Record monthly rent payment" while runtime execution remains bound to deterministic SoRLa endpoint metadata.

SoRLa-specific generation logic belongs here, not in `greentic-designer-sdk`. The current codebase does not have a separate Business Action Catalog artifact; agent endpoints plus their backing actions and executable contract are the source of truth.

## New artifact

Emit:

```text
assets/sorla/designer-node-types.json
```

Schema:

```text
greentic.sorla.designer-node-types.v1
```

This artifact should contain a generic Designer-compatible `nodeTypes` contribution shape. Keep the schema owned by this repo until a shared Designer SDK schema is available.

## Input source

Generate node types from the in-memory canonical IR and generated pack artifacts:

```text
CanonicalIr.agent_endpoints
CanonicalIr.actions
assets/sorla/executable-contract.json
assets/sorla/agent-gateway.json
```

Do not introduce or depend on `assets/sorla/business-actions.json`, `assets/sorla/business-actions.lock.json`, or `docs/business-actions.md` unless a later PR explicitly adds that catalog. If a future business-action catalog exists, it can become an additional input without changing the agent-endpoint path.

## Node type shape

For each public/designable agent endpoint:

```json
{
  "id": "sorla.agent-endpoint.record_rent_payment",
  "version": "0.1.0",
  "label": "Record monthly rent payment",
  "description": "Record a rent payment against an active tenancy through the SoRLa agent endpoint contract.",
  "category": "System of Record",
  "binding": {
    "kind": "component",
    "component": {
      "ref": "oci://ghcr.io/greenticai/components/component-sorx-business:0.1.0"
    },
    "operation": "invoke_locked_action"
  },
  "configSchema": {
    "type": "object",
    "required": ["endpoint_ref"],
    "properties": {
      "endpoint_ref": {
        "const": {
          "id": "record_rent_payment",
          "version": "0.1.0",
          "package": "landlord-tenant",
          "contract_hash": "sha256:..."
        }
      }
    },
    "additionalProperties": false
  },
  "inputSchema": {
    "type": "object",
    "properties": {
      "values": {}
    }
  },
  "outputSchema": {},
  "ui": {
    "fields": [
      {
        "name": "amount",
        "label": "Amount",
        "widget": "number",
        "displayOrder": 10
      }
    ],
    "tags": ["sorla", "agent-endpoint"]
  },
  "defaultRouting": {
    "kind": "out"
  }
}
```

## Important behavior

1. Node labels and aliases are design-time only.
2. Runtime config must include locked `endpoint_ref`.
3. `contract_hash` must come from the canonical IR hash or executable contract hash already emitted by this repo.
4. Input schema must be derived from the agent endpoint input declarations.
5. Component ref should be configurable in generation options.
6. Component operation should default to the documented downstream operation, but tests must not assume this repo owns the downstream component implementation.
7. Output schema must be derived from the agent endpoint output declarations.
8. Risk, approval, visibility, provider requirements, and backing action references should be carried as Designer metadata where useful.
9. Output must be deterministic.

## Doctor / inspect

Update Sorla pack doctor:

- validate node types against the SoRLa-owned Designer node type schema
- verify each node type points to a known agent endpoint
- verify endpoint hash matches canonical IR or executable contract metadata
- verify component binding operation is `invoke_locked_action`
- verify no secrets

Inspect output:

```json
{
  "designer_node_types": {
    "present": true,
    "count": 6
  }
}
```

## Tests

Add tests for:

- node types generated for public/designable agent endpoints
- node type contains locked endpoint ref
- contract hash matches canonical/executable contract metadata
- deterministic output
- invalid agent endpoint metadata fails node generation
- doctor validates artifact

## Docs

Update:

- `docs/designer-extension.md`
- `docs/sorla-gtpack.md`
- `docs/agent-endpoints.md` or `docs/agent-endpoint-handoff-contract.md`

Add section: "Designer node types from agent endpoints".

## Acceptance criteria

```bash
cargo test --all-features
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out /tmp/landlord.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/landlord.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/landlord.gtpack
bash ci/local_check.sh
```
