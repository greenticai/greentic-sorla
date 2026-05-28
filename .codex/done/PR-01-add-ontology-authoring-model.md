# PR 01 — Add generic ontology authoring model to `greentic-sorla`

## Repository

`greenticai/greentic-sorla`

## Objective

Add a first-class, provider-agnostic ontology authoring model to SoRLa.

The ontology model should sit above records/events/actions and express generic business meaning:

- concepts
- entity types
- relationship types
- relationship paths
- inheritance / extension
- cardinality
- constraints
- sensitivity markers
- policy hooks
- provider requirement hints

Do **not** make the model specific to buildings, floors, tenants, customers, CRM, SharePoint, or any other domain. Domain examples can appear only in fixtures.

## Current context

SoRLa already supports records, references, actions, events, projections, provider requirements, policies, approvals, and agent endpoints. This PR adds a semantic ontology layer without breaking existing record-first workflows.

Follow `.codex/architecture_rules.md`: this PR owns source/AST/IR/wizard schema and static validation only. Do not add new final pack or bundle assembly paths here; downstream artifact emission belongs in later handoff/metadata PRs and must remain compatible with `gtc` extension-first composition.

## Proposed schema shape

Add an optional top-level `ontology` section to answer documents and generated SoRLa YAML:

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

    - id: Contract
      kind: entity
      backed_by:
        record: Contract

    - id: Asset
      kind: entity
      backed_by:
        record: Asset

  relationships:
    - id: owns
      label: owns
      from: Party
      to: Asset
      cardinality:
        from: many
        to: many
      backed_by:
        record: Ownership
        from_field: party_id
        to_field: asset_id

    - id: governed_by
      label: governed by
      from: Asset
      to: Contract
      cardinality:
        from: many
        to: one

  constraints:
    - id: no_private_data_without_policy
      applies_to:
        concept: Customer
      requires_policy: customer_data_access
```

## Required Rust model additions

Add typed structs/enums in the appropriate language-facing and IR crates:

- `OntologyModel`
- `ConceptId`
- `ConceptKind`
- `ConceptDefinition`
- `RelationshipId`
- `RelationshipDefinition`
- `RelationshipCardinality`
- `OntologyBacking`
- `OntologyConstraint`
- `OntologySensitivity`
- `OntologyPolicyHook`
- `OntologyProviderRequirement`

Prefer strongly typed structs over raw JSON maps. Use serde with deterministic field names.

Mirror the existing AST style in `crates/greentic-sorla-lang/src/ast.rs`: new authoring structs should use `#[serde(deny_unknown_fields)]`, default empty vectors for optional lists, and `skip_serializing_if` only where the current AST/IR conventions already do so.

## Validation rules

Add static validation:

1. Concept IDs must be unique.
2. Relationship IDs must be unique.
3. `from` and `to` concept references must exist.
4. `extends` must reference an existing concept and must not create cycles.
5. `backed_by.record` must reference an existing SoRLa record.
6. `backed_by.from_field` and `backed_by.to_field` must exist when supplied.
7. Concept and relationship IDs must be stable and URL-safe.
8. Sensitivity and policy hooks must be optional, additive, and non-breaking.
9. Unknown future fields must be rejected to match the current `#[serde(deny_unknown_fields)]` authoring contract.
10. Abstract concepts may omit `backed_by`; entity concepts with `backed_by` must validate against existing records/fields.
11. `extends` must have one canonical shape across authoring and artifacts. Prefer a list in IR/artifacts even if the v1 authoring YAML accepts a single parent for ergonomics.

## CLI / wizard behavior

Update:

```bash
greentic-sorla wizard --schema
greentic-sorla wizard --answers examples/.../answers.json
```

so the generated schema allows optional ontology answers.

The wizard should not require ontology fields for existing fixtures.

## Tests

Add tests for:

- valid ontology model
- duplicate concept ID failure
- duplicate relationship ID failure
- relationship referencing unknown concept
- inheritance cycle rejection
- backing record missing
- backing field missing
- old answer files still pass
- generated schema includes ontology section
- unknown ontology field is rejected
- abstract concept without backing is accepted
- artifact/IR representation sorts inheritance parents deterministically, if multiple inheritance is supported

## Documentation

Add or update:

- `docs/ontology.md`
- `docs/architecture.md`
- `examples/landlord-tenant/answers.json` with a small ontology example

## Acceptance criteria

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo run -p greentic-sorla -- wizard --schema
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out /tmp/landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/landlord-tenant-sor.gtpack
bash ci/local_check.sh
```

Existing fixtures must remain compatible.
