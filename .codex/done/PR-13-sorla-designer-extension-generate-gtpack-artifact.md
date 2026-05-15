# PR 13 — Let Sorla DesignExtension generate `.gtpack` artifact output

## Repository

`greenticai/greentic-sorla`

## Objective

Add a `generate_gtpack` tool to the Sorla DesignExtension.

This gets Greentic Designer closer to:

```text
prompt -> validated Sorla model -> Sorla .gtpack
```

The output should follow the generic generated artifact convention from `greentic-designer-sdk`, without adding Sorla-specific behavior to the SDK. Keep this as a legacy `.gtpack` compatibility artifact and do not introduce final runtime bundle generation in this repo.

## Tool: `generate_gtpack`

Input schema:

```json
{
  "type": "object",
  "required": ["model", "package"],
  "properties": {
    "model": { "type": "object" },
    "package": {
      "type": "object",
      "required": ["name", "version"],
      "properties": {
        "name": { "type": "string" },
        "version": { "type": "string" }
      }
    },
    "options": {
      "type": "object",
      "properties": {
        "include_validation_metadata": { "type": "boolean", "default": true },
        "include_designer_preview": { "type": "boolean", "default": true }
      }
    }
  }
}
```

Output schema follows `ArtifactToolOutput`:

```json
{
  "artifacts": [
    {
    "kind": "gtpack",
      "filename": "example-sor.gtpack",
      "media_type": "application/vnd.greentic.gtpack",
      "sha256": "...",
      "bytes_base64": "...",
      "metadata_json": {
        "schema": "greentic.sorla.generated-artifact.v1",
        "pack_id": "example-sor",
        "pack_version": "0.1.0",
        "records": 4,
        "concepts": 5,
        "relationships": 6,
        "agent_endpoints": 3
      }
    }
  ],
  "diagnostics": [],
  "preview_json": {}
}
```

Confirm the exact artifact envelope with the Designer SDK used in PR 11. If the SDK names fields differently from `bytes_base64` or `metadata_json`, use the SDK names and update this spec before implementation.

## Requirements

1. Validate the model before pack generation.
2. Refuse to generate a pack if there are validation errors.
3. Return diagnostics when generation is refused.
4. Generate deterministic `.gtpack` bytes.
5. Compute SHA-256 from bytes.
6. Return base64 bytes.
7. Include stable metadata.
8. Do not use filesystem if `build_gtpack_bytes` or deterministic pack-entry APIs are available.
9. If only native file pack generation exists, use a temporary directory behind a non-WASM feature and document the limitation.
10. Do not embed absolute temp paths, timestamps, usernames, tenant IDs, or credential-like values in metadata.

## WASM consideration

Preferred implementation:

```rust
let pack = greentic_sorla_lib::build_gtpack_bytes(&model, options)?;
```

Fallback implementation:

```rust
let entries = greentic_sorla_lib::build_gtpack_entries(&model, options)?;
let artifact = host_or_native_pack(entries)?;
```

## Tests

Add tests for:

- valid model generates artifact
- artifact SHA matches bytes
- invalid model returns diagnostics and no artifact
- artifact metadata stable
- generated pack passes Sorla doctor
- generated pack can be inspected from bytes
- no secret-like values in metadata
- output JSON conforms to Designer SDK artifact convention

## Docs

Update:

```text
docs/designer-extension.md
docs/sorla-lib.md
docs/sorla-gtpack.md
```

Add example Designer flow:

```text
generate_model_from_prompt
validate_model
generate_gtpack
```

## Acceptance criteria

```bash
cargo test --all-features
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
cargo run -p greentic-sorla -- pack doctor /tmp/generated.gtpack
bash ci/local_check.sh
```

For WASM builds, accept deterministic pack entries if ZIP byte emission remains native-only after PR 10; the tool should return a clear diagnostic rather than pretending to generate unsupported bytes.
