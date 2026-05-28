# PR 22 — Improve Designer extension locked endpoint node UX

## Repository

`greenticai/greentic-sorla`

## Objective

Improve the existing SoRLa Designer extension tools for selecting generated
node types and producing locked generic flow-node JSON.

This PR applies only to `crates/greentic-sorla-designer-extension` and reusable
SoRLa facade APIs. Do not add Greentic Flow runtime dependencies, component
WASM execution, Sorx commands, provider calls, network access, or a vendored
Designer SDK/WIT.

## Current baseline

The extension already exposes:

```text
list_designer_node_types
generate_flow_node_from_node_type
```

Those tools work with `NormalizedSorlaModel` and generated SoRLa endpoint node
types.

## Required improvements

Add SoRLa-local UX and validation improvements:

1. Allow selecting a node type by exact node type ID.
2. Optionally allow design-time lookup by endpoint ID or label, but resolve it
   immediately to a locked node type ID and endpoint ref.
3. Return diagnostics for ambiguous or unknown design-time selections.
4. Return diagnostics for missing required value mappings.
5. Include endpoint ID, package name, version, contract hash, component ref, and
   operation in the generated generic flow-node JSON.
6. Never emit runtime fields such as `action_label`, `action_alias`,
   `intent_query`, or `natural_language_action`.
7. Keep output deterministic for identical model/input/options.

## Input / output

Prefer extending the existing `generate_flow_node_from_node_type` request shape
instead of adding a new tool. If a new helper is useful, keep it as a pure
SoRLa adapter helper and document it.

The output remains generic JSON:

```json
{
  "flowNode": {
    "schema": "greentic.designer.flow-node.v1",
    "id": "record_payment",
    "type": "sorla.agent-endpoint.record_rent_payment",
    "config": {
      "endpoint_ref": {
        "id": "record_rent_payment",
        "package": "landlord-tenant-sor",
        "version": "0.1.0",
        "contract_hash": "sha256:..."
      }
    }
  },
  "diagnostics": []
}
```

Do not claim this validates against a Greentic Flow schema unless such a schema
fixture exists in this repository.

## Tests

Add tests for:

- exact node type ID selection
- endpoint ID or label selection resolves to locked node type, if supported
- ambiguous label returns diagnostics
- missing mapping returns diagnostics
- generated JSON contains locked endpoint ref
- generated JSON contains no free-text runtime action selection fields
- output is deterministic

## Docs

Update:

```text
docs/designer-extension.md
docs/e2e/designer-node-type-to-locked-endpoint.md
```

Docs must keep downstream Designer/Flow/component/Sorx runtime validation out
of scope for this repository.

## Acceptance criteria

```bash
cargo test -p greentic-sorla-designer-extension
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
bash ci/local_check.sh
```
