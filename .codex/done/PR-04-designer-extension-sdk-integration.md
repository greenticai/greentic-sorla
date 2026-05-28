# PR-04 — Integrate SoRLa extension with the Greentic extension SDK

Repository: `greenticai/greentic-sorla`

## Goal

Expose the SoRLa Designer extension as a real Greentic DesignExtension component using the current `greentic-extension-sdk-*` contracts.

The extension should operate on `sorla.yaml` as source of truth and expose tools for parsing, rendering and patching.

## Important instruction

Do not guess a crate named `greentic-designer-sdk`. The current sibling SDK repo uses `greentic-extension-sdk-*` crates and a WIT world under `greentic:extension-design`.

Codex should inspect the actual SDK crate/API shape from the workspace, registry, local dependency, or GitHub repository. Adapt the implementation to the real SDK interfaces.

If the SDK is missing capabilities required by this PR, create a small compatibility adapter in the SoRLa repo and document the SDK gap clearly.

## Existing crate

The workspace already has:

```text
crates/greentic-sorla-designer-extension
```

Use or refactor this crate rather than creating a duplicate extension.

The current extension is a JSON adapter over `greentic-sorla-lib`, not yet a WIT DesignExtension component. It already exposes tools such as prompt/session/model generation, gtpack entry generation, Designer node-type listing and locked flow node generation. Extend it to the YAML-first design model without removing those tools.

## SDK contract shape

Target the actual SDK surfaces:

- `greentic-extension-sdk-contract` for `describe.json`/artifact contracts and validation.
- `greentic-extension-sdk-cli` templates as examples for DesignExtension packaging.
- WIT exports for `extension-base/manifest`, lifecycle, tools, validation, prompting and knowledge.
- `tools.list-tools` and `tools.invoke-tool(name,args-json)` with JSON schemas for each tool.
- `validation.validate-content(content-type, content-json)`.
- `prompting.system-prompt-fragments`.
- `knowledge.list/get/suggest`.

The descriptor should be a `DesignExtension` descriptor with `apiVersion: greentic.ai/v1`. The SDK only requires runtime gtpack contributions for DesignExtensions when node types are contributed; keep that rule aligned with the contract crate.

## Extension manifest

Register/list tools through the real WIT tools interface, for example conceptually:

```text
parse_sorla_yaml
generate_concept_view
apply_sorla_patch
propose_patch_from_instruction
render_concept_view
validate_sorla_yaml
generate_gtpack_from_sorla_yaml
```

Exact registration should follow the SDK.

## Tool contracts

### parse_sorla_yaml

Input:

```json
{
  "source_yaml": "..."
}
```

Output:

```json
{
  "model": {},
  "diagnostics": [],
  "source_hash": "sha256:..."
}
```

### generate_concept_view

Input:

```json
{
  "source_yaml": "...",
  "mode": "designer",
  "renderer_capabilities": {
    "cards": true,
    "graphs": true,
    "forms": true
  }
}
```

Output:

```json
{
  "concept_view": {}
}
```

### apply_sorla_patch

Input:

```json
{
  "source_yaml": "...",
  "patch": {}
}
```

Output:

```json
{
  "updated_yaml": "...",
  "old_hash": "sha256:...",
  "new_hash": "sha256:...",
  "diagnostics": [],
  "concept_diff": {},
  "concept_view": {}
}
```

### validate_sorla_yaml

Input:

```json
{
  "source_yaml": "..."
}
```

Output:

```json
{
  "diagnostics": [],
  "valid": true
}
```

### generate_gtpack_from_sorla_yaml

Input:

```json
{
  "source_yaml": "...",
  "pack_name": "property-management",
  "pack_version": "0.1.0"
}
```

Output should use existing deterministic pack planning/build APIs.

The current `generate_gtpack` tool returns generated pack entries and metadata. The latest SDK artifact contract expects a valid `ArtifactToolOutput` shape when returning generated artifacts: each artifact needs either bytes or a URI and a lowercase 64-character SHA-256. Do not return `sha256: null` or `bytes_base64: null` as an SDK artifact. Either:

- return deterministic pack entries as a non-artifact tool payload, or
- have the host/native packaging path create bytes/URI plus SHA-256 and wrap it as `ArtifactToolOutput`.

## Designer UI expectations

The extension should return presentation-neutral data, especially `ConceptViewModel`.

Greentic Designer should be able to render:

- cards
- graph/canvas
- timelines
- KPI cards
- diagnostics
- actions
- artifacts

But the extension should not return HTML or React-specific components.

## Security

- No provider credentials in generated views.
- No inline secrets in tool outputs.
- Any LLM capability should be resolved separately and not hardcoded here.

## Acceptance criteria

- `greentic-sorla-designer-extension` targets the real `greentic-extension-sdk-*`/WIT DesignExtension contract.
- Extension exposes YAML-first tools.
- Existing prompt/session, gtpack, node-type and flow-node tools remain available.
- Extension returns `ConceptViewModel` and semantic patch outputs.
- Extension does not rewrite YAML directly outside the patch engine.
- Extension can be loaded as a Greentic Designer plug-in.
- Tests cover `describe.json` validation, WIT tool listing/invocation contracts or the closest available SDK test harness.
