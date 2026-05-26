# Designer SDK Extension

`greentic-sorla-designer-extension` targets the current
`greentic-extension-sdk-*` shape from the sibling Designer SDK. The adapter
matches the `greentic:extension-design` tools boundary: list tools with JSON
schemas, invoke tools by name with JSON arguments, validate content, provide
prompt fragments, and expose knowledge entries.

YAML-first tools:

- `parse_sorla_yaml`
- `generate_concept_view`
- `apply_sorla_patch`
- `propose_patch_from_instruction`
- `validate_sorla_yaml`
- `generate_gtpack_from_sorla_yaml`

Existing prompt/session, normalized-model, gtpack-entry, Designer node-type,
and locked flow-node tools remain available. The WASM-safe gtpack tools return
deterministic pack-entry plans; native hosts are responsible for packaging ZIP
bytes and producing SDK `ArtifactToolOutput` values with real bytes or URIs and
valid SHA-256 digests.

The extension does not read provider credentials, embed secrets, shell out to
the CLI, or return browser-specific UI formats.
