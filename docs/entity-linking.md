# Entity Linking

SoRLa can optionally declare semantic aliases and entity-linking strategies for
ontology concepts. These declarations are provider-agnostic source metadata for
downstream Sorx, `gtc`, retrieval, MCP, OpenAPI, and agent tooling.

They do not execute matching, query providers, store embeddings, expose tenant
data, or carry credentials.

## Semantic Aliases

```yaml
semantic_aliases:
  concepts:
    Customer:
      - client
      - account holder
  relationships:
    owns:
      - belongs to
      - owned by
```

Aliases must point to concepts or relationships declared in `ontology`. During
lowering, aliases are normalized by trimming whitespace, collapsing repeated
whitespace to a single space, and converting to lowercase. Normalized aliases
are sorted and de-duplicated in canonical IR and pack artifacts.

If two aliases normalize to the same value for the same target, parsing emits a
warning and keeps one canonical alias. If the same normalized alias points to a
different concept or relationship, parsing fails.

## Entity-Linking Strategies

```yaml
entity_linking:
  strategies:
    - id: email_match
      applies_to: Customer
      match:
        source_field: email
        target_field: email
      confidence: 0.95
      sensitivity:
        pii: true
```

Strategy IDs must be unique and URL-safe. `applies_to` must reference an
ontology concept. `confidence` must be between `0.0` and `1.0`.

When the target concept has `backed_by.record`, `match.target_field` must exist
on that record. `match.source_field` is a source-side declaration for downstream
tooling and is validated as non-empty, not as a provider-specific field.

For unbacked concepts, a strategy must declare an explicit non-record
`source_type`, for example `document`, `agent-input`, or `external-id`. This
keeps abstract concepts usable without pretending SoRLa owns runtime matching.

## Pack Output

Ontology-enabled `.gtpack` files include aliases and entity-linking strategies
inside:

- `assets/sorla/ontology.graph.json`
- `assets/sorla/ontology.ir.cbor`

`greentic-sorla pack doctor` checks that the graph mirrors the canonical
ontology IR, including aliases and entity-linking strategies.
