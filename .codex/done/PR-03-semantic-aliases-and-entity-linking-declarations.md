# PR 03 — Add semantic aliases and entity-linking declarations

## Repository

`greenticai/greentic-sorla`

## Objective

Add generic semantic alias and entity-linking declarations so user language, documents, provider records, and agent inputs can map to ontology concepts and entity instances.

This must remain domain-agnostic.

This PR depends on PR 01's ontology AST/IR and PR 02's ontology handoff artifact shape. Keep the new sections optional and extension-first; do not add provider credentials, tenant identifiers, or final runtime binding logic to SoRLa.

## New model sections

Add optional sections:

```yaml
semantic_aliases:
  concepts:
    Customer:
      - client
      - account holder
      - buyer
    Contract:
      - agreement
      - subscription
  relationships:
    owns:
      - belongs to
      - controlled by
    governed_by:
      - under
      - covered by

entity_linking:
  strategies:
    - id: exact_external_id
      applies_to: Customer
      match:
        source_field: external_id
        target_field: id
      confidence: 1.0

    - id: email_match
      applies_to: Customer
      match:
        source_field: email
        target_field: email
      confidence: 0.95
      sensitivity:
        pii: true
```

## Requirements

1. Aliases are optional.
2. Aliases must be deterministic and sorted in emitted artifacts.
3. Alias collision behavior must be deterministic:
   - reject exact normalized alias collisions that map to different concepts/relationships;
   - allow duplicates that normalize to the same target only if they are de-duplicated with a warning;
   - document the normalization rules, including case-folding and whitespace trimming.
4. Entity-linking strategies must reference existing ontology concepts.
5. Field mappings must reference known records/fields when the target concept is backed by a record. For unbacked abstract concepts, validation must fail unless the strategy declares an explicit non-record source type.
6. Sensitivity markers must be preserved into ontology artifacts.
7. No provider-specific credentials or tenant details may appear.
8. Strategy IDs must be unique and URL-safe.
9. Confidence must be bounded from `0.0` to `1.0`.

## Artifact output

Extend:

```text
assets/sorla/ontology.graph.json
assets/sorla/ontology.ir.cbor
```

with:

```json
{
  "semantic_aliases": {
    "concepts": {},
    "relationships": {}
  },
  "entity_linking": {
    "strategies": []
  }
}
```

Optionally emit:

```text
assets/sorla/entity-linking.schema.json
```

## Tests

Add tests for:

- valid aliases
- duplicate alias handling
- alias collision across two different ontology targets is rejected
- alias sorting
- unknown concept in alias map
- unknown relationship in alias map
- valid entity-linking strategy
- unknown field in linking strategy
- out-of-range confidence rejection
- duplicate strategy ID rejection
- emitted graph contains aliases and linking strategies

## Docs

Add examples in:

- `docs/ontology.md`
- `docs/entity-linking.md`

## Acceptance criteria

```bash
cargo test --all-features
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out /tmp/sor.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/sor.gtpack
bash ci/local_check.sh
```
