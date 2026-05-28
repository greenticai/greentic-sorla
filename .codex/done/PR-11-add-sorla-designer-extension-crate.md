# PR 11 — Add Sorla DesignExtension crate inside `greentic-sorla`

## Repository

`greenticai/greentic-sorla`

## Objective

Add a Designer DesignExtension crate to the Sorla repo.

This extension should use the reusable Sorla library APIs from PR 08/09 and implement the existing `greentic-designer-sdk` DesignExtension WIT.

Do not modify the Designer SDK with Sorla-specific logic. Do not add a local, divergent copy of the Designer WIT if the SDK provides one; consume the SDK/WIT as the source of truth.

## New workspace member

Add:

```text
crates/greentic-sorla-designer-extension
```

It should build as a WebAssembly Component for `wasm32-wasip2` if feasible.

Keep this crate out of `default-members` until it can pass the normal local check without requiring optional external SDK/tooling that is not already available. If it is excluded, add explicit docs and a targeted build command.

## Extension responsibilities

The extension should expose Designer tools for:

```text
generate_model_from_prompt
validate_model
improve_model
explain_model
```

Initial implementation can be deterministic and template/rule-based. It does not need to call an LLM itself. The Designer host can provide LLM-generated drafts later.

## Tool 1: `generate_model_from_prompt`

Purpose:

Take a natural-language prompt or structured draft and return a Sorla model draft.

Input:

```json
{
  "type": "object",
  "required": ["prompt"],
  "properties": {
    "prompt": { "type": "string" },
    "constraints": {
      "type": "object",
      "properties": {
        "include_ontology": { "type": "boolean", "default": true },
        "include_agent_endpoints": { "type": "boolean", "default": true },
        "include_retrieval_bindings": { "type": "boolean", "default": false }
      }
    },
    "draft_json": {
      "description": "Optional structured draft produced by the host/LLM.",
      "type": "object"
    }
  }
}
```

Output:

```json
{
  "status": "draft | valid | needs_input",
  "model": {},
  "diagnostics": [],
  "questions": [],
  "preview": {}
}
```

## Tool 2: `validate_model`

Input:

```json
{
  "model": {}
}
```

Output:

```json
{
  "valid": true,
  "diagnostics": [],
  "preview": {}
}
```

## Tool 3: `improve_model`

Input:

```json
{
  "model": {},
  "instruction": "Add approval gates for high-risk actions"
}
```

Output:

```json
{
  "model": {},
  "changes": [],
  "diagnostics": [],
  "preview": {}
}
```

## Tool 4: `explain_model`

Input:

```json
{
  "model": {}
}
```

Output:

```json
{
  "summary": "...",
  "sections": [],
  "preview": {}
}
```

## Design principles

1. Extension returns structured JSON, not prose-only output.
2. Extension always runs Sorla validation before returning `valid`.
3. Extension emits diagnostics and follow-up questions for missing info.
4. Extension never embeds secrets or provider credentials.
5. Extension keeps model generic and domain-agnostic unless user prompt asks for a domain.
6. Domain-specific content belongs in generated model data, not core contracts.

## Dependencies

Depend on:

- `greentic-sorla-lib` or equivalent public facade crate from PR 09/10
- generated WIT bindings from `greentic-designer-sdk` WIT, or an existing shared Greentic interface crate if that is the current SDK packaging
- minimal serde/JSON dependencies

Avoid depending on CLI crate.

## Tests

Add tests for:

- extension manifest exports expected tools
- `validate_model` rejects invalid model
- `generate_model_from_prompt` returns a valid draft for a simple prompt
- diagnostics are stable
- preview is stable
- no filesystem required for validation path
- builds for `wasm32-wasip2` if target available

## Docs

Add:

```text
docs/designer-extension.md
```

Document:

- what the extension does
- how Designer uses it
- how it differs from the CLI
- how the extension uses `sorla-lib`

## Acceptance criteria

```bash
cargo test --all-features
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
bash ci/local_check.sh
```

If the Designer SDK is not available in this repository or on crates.io, update this PR to add a small adapter boundary and mark the WIT build as a documented follow-up rather than checking in placeholder incompatible bindings.
