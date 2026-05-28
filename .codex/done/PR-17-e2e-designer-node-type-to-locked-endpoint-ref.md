# PR 17 — SoRLa-local e2e for Designer node type to locked endpoint ref

## Repository

`greenticai/greentic-sorla`

## Objective

Prove the SoRLa-owned portion of the Designer node type flow using only this
repository:

```text
SoRLa agent endpoint
  -> assets/sorla/designer-node-types.json
  -> greentic-sorla-designer-extension list_designer_node_types
  -> generate_flow_node_from_node_type
  -> locked endpoint_ref with id/version/package/contract_hash
```

Do not implement Designer SDK validation, Greentic Flow rendering, component
execution, Sorx runtime endpoint execution, provider calls, or cross-repo CI in
this PR. Those remain downstream integration responsibilities.

## Current baseline

PR-15 and PR-16 already implemented the core generator and adapter tools:

- packs emit `assets/sorla/designer-node-types.json`
- pack inspect summarizes `designer_node_types`
- pack doctor validates node types against canonical agent endpoints
- `greentic-sorla-lib` exposes node type generation
- `greentic-sorla-designer-extension` exposes `list_designer_node_types`
- `greentic-sorla-designer-extension` exposes `generate_flow_node_from_node_type`

This PR should add a focused end-to-end fixture/test around that existing path,
not replace the generator.

## Required scenario

Use the existing landlord/tenant fixture or a small SoRLa YAML fixture with at
least one side-effectful agent endpoint. The domain label may be "Record monthly
rent payment" if already present, but the test must assert generic endpoint
metadata rather than business-action catalog files.

The scenario should verify:

1. SoRLa pack generation emits `assets/sorla/designer-node-types.json`.
2. The node type document uses `greentic.sorla.designer-node-types.v1`.
3. Each node type ID is `sorla.agent-endpoint.<endpoint-id>`.
4. `endpoint_ref` includes endpoint ID, package name, package version, and
   `sha256:<canonical-ir-hash>`.
5. The extension's `list_designer_node_types` returns the same node type
   contract for the normalized model.
6. The extension's `generate_flow_node_from_node_type` returns generic JSON with
   the locked `endpoint_ref`.
7. Missing required endpoint input mappings produce diagnostics.
8. No generated node or flow node contains free-text runtime action selection.

## Suggested implementation

Add one deterministic integration test in the SoRLa repo, preferably near the
current Designer extension e2e:

```text
crates/greentic-sorla-designer-extension/tests/designer_node_type_to_locked_endpoint.rs
```

The test may use `greentic-sorla-lib` directly instead of shelling out. If a
script is useful for manual reproduction, keep it SoRLa-local:

```text
scripts/e2e/designer-node-type-locked-endpoint.sh
```

The script must not require sibling repos, network, credentials, Designer SDK,
Greentic Flow, component WASM, or Sorx.

## Docs

Add or update:

```text
docs/e2e/designer-node-type-to-locked-endpoint.md
docs/designer-extension.md
docs/agent-endpoints.md
```

The docs should explicitly state that downstream Designer SDK, Flow, component,
and Sorx runtime checks are out of scope for this repository and should consume
the deterministic SoRLa artifacts later.

## Acceptance criteria

```bash
cargo test -p greentic-sorla-designer-extension designer_node_type_to_locked_endpoint
cargo test -p greentic-sorla-pack -p greentic-sorla-lib -p greentic-sorla-designer-extension
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
bash ci/local_check.sh
```

If `ci/local_check.sh` is too broad for a quick PR run, the targeted tests above
must still pass and the skipped broader check must be documented.
