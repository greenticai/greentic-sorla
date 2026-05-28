# PR 21 — Polish Designer node type metadata from SoRLa agent endpoints

## Repository

`greenticai/greentic-sorla`

## Objective

Improve the Designer-friendly metadata already emitted in
`assets/sorla/designer-node-types.json`.

This PR should build on PR-15 and PR-16. Do not introduce
`business-actions.json`, Flow runtime schemas, component execution, or
downstream SDK-specific fields that are not present in this repo.

## Current baseline

The pack crate already emits generic node type JSON with:

- stable `sorla.agent-endpoint.<id>` node type IDs
- component binding metadata
- locked `endpoint_ref`
- input and output schemas
- UI fields
- risk, approval, side-effect, backing, export, and provider metadata

## Required improvements

Improve only SoRLa-owned generation:

1. Ensure field labels are stable and human-readable for snake_case and kebab-case
   endpoint inputs.
2. Ensure widgets are deterministic for known scalar types.
3. Preserve endpoint descriptions and input/output descriptions when present.
4. Add design-time aliases/tags only if they can be derived deterministically
   from current endpoint metadata.
5. Ensure metadata carries enough context for Designer search without becoming
   runtime identity.
6. Keep component ref and operation configurable through existing generation
   options.
7. Keep output deterministic and secret-free.

## Compatibility

The existing `greentic.sorla.designer-node-types.v1` schema should remain
backward compatible. If a new optional field is needed, add it in a way old
consumers can ignore.

Do not rename existing fields such as `nodeTypes`, `configSchema`,
`inputSchema`, `outputSchema`, or `defaultRouting`.

## Tests

Add focused tests for:

- stable labels for `contract_id`, `rent-payment-id`, and similar names
- widget mapping for string, number/integer, boolean, and enum inputs
- endpoint descriptions flow into node type descriptions
- sensitive inputs do not leak actual values
- generated metadata is deterministic
- pack doctor still accepts the enhanced node type document

## Docs

Update:

```text
docs/designer-extension.md
docs/agent-endpoints.md
```

Explain which metadata is design-time only and which locked fields downstream
runtime systems must consume.

## Acceptance criteria

```bash
cargo test -p greentic-sorla-pack -p greentic-sorla-lib
cargo test -p greentic-sorla-designer-extension
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
bash ci/local_check.sh
```
