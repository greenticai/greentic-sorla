# Retrieval Bindings

SoRLa retrieval bindings connect ontology concepts or relationships to abstract
evidence providers. They are deterministic handoff metadata for Sorx, `gtc`,
GraphRAG, document stores, vector systems, and future provider adapters.

They do not execute retrieval, embed credentials, resolve tenants, or assemble a
runtime bundle.

## Authoring

```yaml
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: primary_evidence
      category: evidence
      required_capabilities:
        - evidence.query
        - entity.link

  scopes:
    - id: customer_evidence
      applies_to:
        concept: Customer
      provider: primary_evidence
      filters:
        entity_scope:
          include_self: true
          include_related:
            - relationship: owns
              direction: outgoing
              max_depth: 1
```

Provider entries reuse the existing abstract category/capability vocabulary.
`category` and `required_capabilities` describe what downstream tooling must
resolve; they must not contain concrete credentials, tokens, passwords, or
provider-specific tenant data.

## Validation

Static validation checks:

- provider IDs are unique and URL-safe
- provider categories and capabilities are non-empty
- scope IDs are unique and URL-safe
- scope providers reference declared retrieval providers
- scope targets reference known ontology concepts or relationships
- traversal rules reference known ontology relationships
- traversal direction is one of `incoming`, `outgoing`, or `both`
- traversal `max_depth` is within `0..=5`
- secret-like markers such as tokens or passwords do not appear in provider
  categories or capabilities

## Pack Output

When retrieval bindings are present, `.gtpack` generation emits:

- `assets/sorla/retrieval-bindings.json`
- `assets/sorla/retrieval-bindings.ir.cbor`

`pack.cbor` references these assets under the SoRLa extension metadata. The JSON
and CBOR forms carry the same canonical retrieval binding IR; `pack doctor`
checks that both forms match `model.cbor` and are covered by `pack.lock.cbor`.
