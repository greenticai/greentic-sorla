# Sorla Designer Extension

`crates/greentic-sorla-designer-extension` is the deterministic adapter crate
for Greentic Designer. It depends on `greentic-sorla-lib` rather than the
`greentic-sorla` CLI package, so Designer uses the same normalization,
validation, and preview paths as local tooling.

The crate currently exposes a small JSON tool boundary because the
`greentic-designer-sdk` WIT package is not present in this repository. When the
SDK is available, this adapter boundary should be wired to the real
DesignExtension exports instead of checking in divergent local WIT.

The extension tools are:

- `generate_model_from_prompt`
- `validate_model`
- `improve_model`
- `explain_model`
- `generate_gtpack`
- `list_designer_node_types`
- `generate_flow_node_from_node_type`

The adapter also exposes deterministic prompting and knowledge helpers:

- `system_prompt_fragments`
- `list_entries`
- `get_entry`
- `suggest_entries`

The extension-safe flow is:

```text
prompt or structured draft
  -> normalize_answers or model construction
  -> validate_model
  -> generate_preview
  -> generate_gtpack
```

Designer node types are generated from SoRLa agent endpoints, not from a
separate business-action catalog. `list_designer_node_types` returns the generic
`nodeTypes` contribution shape from the normalized model using the same
generator that writes `assets/sorla/designer-node-types.json` into packs.
Packs also include `assets/sorla/agent-endpoint-action-catalog.json`, a
design-time catalog view over the same canonical agent endpoints. It is useful
for search and prompt assistance, but it is not a runtime action registry.

`generate_flow_node_from_node_type` validates the requested node type, required
endpoint input mappings, component binding, and locked endpoint reference before
returning a generic flow node JSON object. The generated node carries
`endpoint_ref` metadata with endpoint ID, package version, package name, and
canonical contract hash, so runtime selection does not depend on free-text
action intent. Requests can select by exact node type ID, endpoint ID, or label;
endpoint ID and label are resolved immediately to one locked node type. Unknown
or ambiguous design-time selections return diagnostics instead of falling back
to natural-language action matching.

Generated node and flow-node metadata must not contain runtime selection fields
such as `action_label`, `action_alias`, `intent_query`, or
`natural_language_action`. Labels, aliases, tags, descriptions, and prompt
context remain design-time only. Downstream Designer, Flow, component, Sorx,
provider, audit, and approval systems must perform their own runtime validation
outside this repository.

For native hosts, `build_gtpack_bytes` can produce a deterministic `.gtpack`
compatibility artifact. For `wasm32-wasip2` extension builds, prefer
`build_gtpack_entries` first and let the host turn those entries into an archive
when ZIP output or filesystem access is not available.

Designer extension code must not shell out to the CLI, read credentials, use
network access, or depend on absolute filesystem paths for normalization,
validation, preview, or pack-entry planning.

Designer hosts can combine:

```text
user prompt
+ system_prompt_fragments()
+ suggest_entries(query, limit)
+ generate_model_from_prompt / validate_model / explain_model
```

Prompt fragments describe SoRLa modelling, ontology, and safety rules. Knowledge
entries provide deterministic guides and examples using the same answers/model
shape accepted by `greentic-sorla-lib`.

`generate_gtpack` validates the model before returning artifact output. In the
current WASM adapter it returns deterministic pack-entry metadata with
`bytes_base64: null` and a warning diagnostic that host/native packaging must
produce ZIP bytes. This keeps the Designer artifact contract honest until the
SDK/WIT host packaging path is available.

Build the adapter for WASI with:

```bash
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
```
