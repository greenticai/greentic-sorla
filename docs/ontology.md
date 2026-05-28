# Ontology Authoring

SoRLa can carry an optional provider-agnostic ontology above records, events,
actions, policies, approvals, and agent endpoints. The ontology describes
business meaning for downstream `gtc`, Sorx, retrieval, and agent-facing
handoff tools without making this repository responsible for runtime assembly.

## Scope

`greentic-sorla` owns the authoring model, parser validation, canonical IR, and
wizard answer rendering. It does not execute graph traversal, bind concrete
providers, enforce runtime policy, or expose public routes.

## Authoring Shape

```yaml
ontology:
  schema: greentic.sorla.ontology.v1
  concepts:
    - id: Party
      kind: abstract
      description: A legal or natural actor in the domain.

    - id: Customer
      kind: entity
      extends: Party
      backed_by:
        record: Customer
      sensitivity:
        classification: confidential
        pii: true

  relationships:
    - id: has_contract
      label: has contract
      from: Customer
      to: Contract
      cardinality:
        from: one
        to: many
      backed_by:
        record: CustomerContract
        from_field: customer_id
        to_field: contract_id

  constraints:
    - id: customer_data_policy
      applies_to:
        concept: Customer
      requires_policy: customer_data_access

semantic_aliases:
  concepts:
    Customer:
      - client
      - account holder
  relationships:
    has_contract:
      - covered by

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

Concept and relationship IDs must be stable and URL-safe. Authoring accepts a
single `extends: Party` value or a list, while canonical IR stores inheritance
parents as a sorted list.

## Validation

Static validation rejects:

- duplicate concept, relationship, or constraint IDs
- relationship `from` or `to` references to unknown concepts
- inheritance references to unknown concepts
- inheritance cycles
- `backed_by.record` references to unknown records
- `backed_by.from_field` or `backed_by.to_field` references to unknown fields
- unknown fields in ontology YAML
- unsupported ontology schema versions
- semantic aliases that point to unknown concepts or relationships
- normalized alias collisions across different concepts or relationships
- entity-linking strategies with duplicate or non-URL-safe IDs
- entity-linking strategies that point to unknown concepts
- entity-linking target fields missing from the backing record
- entity-linking confidence values outside `0.0` through `1.0`

Abstract concepts may omit `backed_by`. Entity concepts may also omit backing
when they represent abstract business meaning, but any supplied backing must
refer to existing SoRLa records and fields.

Semantic alias normalization trims leading/trailing whitespace, collapses
internal whitespace to single spaces, and case-folds with lowercase conversion.
Aliases that normalize to the same target are de-duplicated with a parser
warning. Aliases that normalize to different ontology targets are rejected so
downstream retrieval and agent tools do not receive ambiguous meaning.

Entity-linking strategies describe how external text, documents, provider
records, or agent inputs can map to ontology concept instances. When a target
concept is backed by a SoRLa record, `match.target_field` must reference a field
on that backing record. For unbacked concepts, the strategy must declare an
explicit non-record `source_type`; SoRLa records the declaration but does not
execute matching or bind a provider.

## Wizard Answers

Answers documents may include an optional top-level `ontology` section. Existing
answers without ontology remain valid. When ontology answers are supplied, the
wizard writes them into generated `sorla.yaml` and the canonical IR includes the
normalized ontology model.

The landlord/tenant example includes a small ontology fixture to exercise this
path without turning the core model into a landlord/tenant-specific contract.

## Pack Artifacts

When ontology is present, `.gtpack` generation emits:

- `assets/sorla/ontology.graph.json`
- `assets/sorla/ontology.ir.cbor`
- `assets/sorla/ontology.schema.json`

`pack.cbor` declares the `greentic.sorla.ontology.v1` extension and points to
those relative asset paths. Downstream Sorx, `gtc`, GraphRAG, MCP, and OpenAPI
tooling should discover ontology metadata through the manifest instead of
guessing paths.

The graph JSON uses `greentic.sorla.ontology.graph.v1`, includes the package
name/version, carries the SHA-256 hash of `ontology.ir.cbor`, and mirrors the
canonical concepts, relationships, constraints, semantic aliases, and
entity-linking strategies. The CBOR payload is the canonical ontology IR.
Repeated builds with the same source emit byte-identical pack output.

`greentic-sorla pack doctor` validates that ontology assets exist, are covered
by `pack.lock.cbor`, match the canonical IR, avoid unsafe manifest paths, and
only reference backing records and fields present in `model.cbor`.

## SORX Validation Metadata

Ontology-enabled `.gtpack` archives also receive deterministic SORX validation
metadata. Exported packs add a required `ontology` suite to
`promotion_requires`; private-only packs may carry the same suite with
`required: false` so downstream tools can inspect ontology quality without
blocking promotion.

The ontology suite uses metadata-only test kinds:

- `ontology-static`
- `ontology-relationship`
- `ontology-alias`
- `entity-linking`

Retrieval-enabled exported packs add a required `retrieval` suite with the
`retrieval-binding` test kind. Retrieval provider category/capability
requirements are also represented through the existing `provider-capability`
validation tests. These declarations do not execute retrieval or bind concrete
providers inside SoRLa.

For an end-to-end deterministic fixture that exercises ontology, aliases,
entity-linking, retrieval bindings, validation metadata, doctor, and inspect
without external services, see `docs/ontology-handoff-scenario.md`.
