# Designer node type to locked endpoint

This local e2e covers the SoRLa-owned path from an agent endpoint to Designer
node type metadata and generic flow-node JSON.

```text
SoRLa agent endpoint
  -> assets/sorla/designer-node-types.json
  -> list_designer_node_types
  -> generate_flow_node_from_node_type
  -> locked endpoint_ref
```

The locked reference contains endpoint ID, package name, package version, and a
`sha256:<canonical-ir-hash>` contract hash. Labels, aliases, descriptions, and
prompt text are design-time metadata only.

Run the focused test with:

```bash
cargo test -p greentic-sorla-designer-extension designer_node_type_to_locked_endpoint
```

The extension can resolve a flow-node request by exact node type ID, endpoint
ID, or label. Endpoint ID and label are design-time selectors only; the emitted
flow node still carries the locked `endpoint_ref` and never emits free-text
runtime action selection fields.

This test does not require a live LLM, network, credentials, a Designer SDK
checkout, Greentic Flow, component WASM, Sorx, or provider repositories.
Downstream systems should consume the deterministic SoRLa artifacts and perform
their own runtime validation in their own repositories.
