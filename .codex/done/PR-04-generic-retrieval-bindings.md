# PR 04 — Add generic retrieval bindings for ontology-scoped evidence

## Repository

`greenticai/greentic-sorla`

## Objective

Add generic retrieval binding declarations that connect ontology concepts and relationships to evidence providers, external references, vector providers, document stores, or future GraphRAG systems.

Do not use domain-specific fields such as `building_id` or `floor_id` in the core model.

This PR depends on PR 01's ontology model and should integrate with the existing provider requirement vocabulary instead of creating a second incompatible provider model. It should emit extension-friendly handoff metadata only; do not add runtime retrieval execution, provider credentials, or final assembly ownership to this repo.

## Proposed authoring shape

```yaml
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: primary_evidence
      category: evidence
      required_capabilities:
        - evidence.query
        - entity.link

    - id: primary_external_ref
      category: external-ref
      required_capabilities:
        - external-reference.resolve

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

    - id: contract_evidence
      applies_to:
        concept: Contract
      provider: primary_evidence
      filters:
        entity_scope:
          include_self: true
          include_related:
            - relationship: governed_by
              direction: incoming
              max_depth: 2
```

## Required concepts

Add typed model support for:

- `RetrievalBindings`
- `RetrievalProviderRequirement`
- `RetrievalScope`
- `EntityScope`
- `RelationshipTraversalRule`
- `RetrievalFilter`
- `RetrievalPermissionMode`

## Validation rules

1. Retrieval providers must reference abstract provider categories/capabilities, not concrete credentials.
2. Scope IDs must be unique.
3. Concepts referenced by scopes must exist.
4. Relationships referenced by traversal rules must exist.
5. Depth values must be bounded with a concrete v1 maximum. Use `0..=5` unless a stronger existing project convention is found.
6. Retrieval bindings must not include hard-coded provider secrets.
7. Retrieval bindings must be additive and optional.
8. Retrieval provider IDs must be unique and URL-safe.
9. Scope `provider` references must point to a declared retrieval provider.
10. `direction` must be a closed enum such as `incoming`, `outgoing`, or `both`.
11. Required capabilities should be sorted and should reuse the same deterministic aggregation style as `ProviderRequirementIr`.

## Artifacts

Emit:

```text
assets/sorla/retrieval-bindings.json
assets/sorla/retrieval-bindings.ir.cbor
```

Also reference these from the pack manifest.

Use the existing legacy pack compatibility path for these assets; do not introduce new pack-builder orchestration.

## Tests

Add tests for:

- valid retrieval binding
- unknown concept rejection
- unknown relationship rejection
- invalid depth rejection
- scope references unknown provider rejection
- duplicate provider ID rejection
- deterministic output ordering
- doctor/inspect displays retrieval metadata
- old fixtures without retrieval bindings still pass

## Docs

Add:

- `docs/retrieval-bindings.md`

## Acceptance criteria

```bash
cargo test --all-features
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out /tmp/sor.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/sor.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/sor.gtpack
bash ci/local_check.sh
```
