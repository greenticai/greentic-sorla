# PR 02 — Emit deterministic ontology artifacts in `.gtpack`

## Repository

`greenticai/greentic-sorla`

## Objective

Emit deterministic ontology handoff artifacts through the existing SoRLa pack compatibility path.

The artifact must be stable, provider-agnostic, and consumable by `greentic-sorx`, GraphRAG-style retrieval systems, MCP tools, OpenAPI overlays, and future `gtc` bundle assembly.

Follow `.codex/architecture_rules.md`: do not introduce new final pack/bundle assembly orchestration in `greentic-sorla`. Reuse the existing legacy `--pack-out` compatibility path only for deterministic handoff metadata, and document the assets as extension-friendly source material for `gtc`/Sorx rather than as runtime assembly ownership.

## New pack artifacts

When an ontology is present, emit:

```text
assets/sorla/ontology.graph.json
assets/sorla/ontology.ir.cbor
assets/sorla/ontology.schema.json
```

Optionally emit:

```text
assets/sorla/ontology.summary.json
assets/sorla/ontology.llms.txt.fragment
```

Do not emit domain-specific hard-coded paths.

## `ontology.graph.json` shape

Use a deterministic graph shape:

```json
{
  "schema": "greentic.sorla.ontology.graph.v1",
  "package": {
    "name": "example-sor",
    "version": "0.1.0"
  },
  "ir_hash": "...",
  "concepts": [
    {
      "id": "Customer",
      "kind": "entity",
      "extends": ["Party"],
      "backing": {
        "record": "Customer"
      },
      "sensitivity": {
        "classification": "confidential",
        "pii": true
      }
    }
  ],
  "relationships": [
    {
      "id": "owns",
      "from": "Party",
      "to": "Asset",
      "cardinality": {
        "from": "many",
        "to": "many"
      },
      "backing": {
        "record": "Ownership",
        "from_field": "party_id",
        "to_field": "asset_id"
      }
    }
  ],
  "indexes": {
    "concepts_by_id": true,
    "relationships_by_id": true
  }
}
```

## Determinism requirements

1. Sort concepts by ID.
2. Sort relationships by ID.
3. Sort provider requirements lexically.
4. Sort aliases lexically when present.
5. Normalize JSON key ordering using the repo’s current deterministic strategy.
6. Ensure CBOR is canonical.
7. Ensure repeated builds produce byte-identical `.gtpack` output for the same input.

## Pack manifest integration

Update the existing manifest metadata entries so Sorx/`gtc` can discover the ontology artifact by schema/extension.

Do not add a parallel manifest system or hard-code final runtime assembly semantics.

Add or extend a pack extension declaration:

```text
greentic.sorla.ontology.v1
```

Do not require Sorx to guess file paths when manifest metadata can point to them.

## Doctor / inspect updates

Update:

```bash
greentic-sorla pack doctor <pack.gtpack>
greentic-sorla pack inspect <pack.gtpack>
greentic-sorla pack validation-inspect <pack.gtpack>
```

to validate and display ontology metadata.

Doctor should check:

- graph artifact exists when ontology is declared
- ontology IR hash matches emitted IR bytes
- graph concepts match IR concepts
- graph relationships match IR relationships
- no absolute paths
- no secrets
- all referenced backing records/fields exist
- manifest paths are relative, stay inside the archive asset namespace, and do not contain `..`

## Tests

Add tests for:

- pack contains ontology artifacts
- manifest references ontology artifacts
- deterministic repeat output
- byte-identical repeated `--pack-out` output when ontology is present
- doctor fails if ontology graph is missing
- doctor fails if ontology graph references unknown concept
- doctor fails if manifest metadata points outside the archive asset namespace
- inspect emits stable ontology metadata

## Docs

Update:

- `docs/sorla-gtpack.md`
- `docs/ontology.md`
- `docs/agent-endpoint-handoff-contract.md` if agent endpoints can reference ontology concepts

## Acceptance criteria

```bash
cargo test --all-features
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out /tmp/sor.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/sor.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/sor.gtpack
cargo run -p greentic-sorla -- pack validation-inspect /tmp/sor.gtpack
bash ci/local_check.sh
```

The same input must produce stable artifact hashes across repeated runs.
