# PR: Add operational index and query/search requirement declarations

Repo: `greenticai/greentic-sorla`

## Goal

Allow SoRLa packages to declare provider-agnostic operational index and query
requirements that downstream Sorx/provider tooling can resolve, without
duplicating existing ontology relationships or retrieval bindings.

## Current-code assumptions

- Ontology relationships already exist under `ontology.relationships` and lower
  into canonical ontology IR plus ontology graph artifacts.
- Record field references already exist through field-level `references` and
  appear in `executable-contract.json` as relationships.
- Retrieval/search-adjacent evidence metadata already exists under
  `retrieval_bindings`, with provider categories, capabilities, ontology scopes,
  traversal filters, and pack assets.
- There is no general SoRLa route/query language today. Validation cannot say
  "route/query requires undeclared index" unless this PR also introduces a
  concrete static query requirement surface on projections, views, or agent
  endpoints.
- Concrete provider bindings belong outside this repo. SoRLa should emit
  abstract provider requirement metadata, not storage-engine-specific index DDL.

## Proposed authoring shape

Add index requirements as their own provider-agnostic section:

```yaml
operational_indexes:
  schema: greentic.sorla.operational-indexes.v1
  indexes:
    - id: tenant_by_email
      record: Tenant
      kind: exact
      fields:
        - email

    - id: active_tenants_by_property
      record: Tenancy
      kind: composite
      fields:
        - property_id
        - status

  query_requirements:
    - id: active_tenant_lookup
      used_by:
        projection: ActiveTenants
      requires_index: active_tenants_by_property
      scan_ok: false
```

Keep semantic search and evidence retrieval aligned with the existing
`retrieval_bindings` model. If text-search requirements are needed, model them
as abstract retrieval/index capabilities rather than a separate provider binding:

```yaml
retrieval_bindings:
  schema: greentic.sorla.retrieval-bindings.v1
  providers:
    - id: tenant_evidence
      category: evidence
      required_capabilities:
        - evidence.query
        - text.search
        - entity.link
```

## Implementation notes

- Add language AST, parser validation, canonical IR lowering, and deterministic
  JSON/CBOR handoff artifacts for `operational_indexes`.
- Validate record and field references against declared records.
- Validate `query_requirements.used_by` only against static surfaces that exist
  in this repo: projections, views, and agent endpoints. Do not invent runtime
  routes in SoRLa.
- If a projection/view/agent endpoint declares a static query requirement, fail
  validation unless it references a declared index or explicitly sets
  `scan_ok: true`.
- Add pack metadata under the SoRLa extension and doctor checks so the index
  artifacts match `model.cbor` and are covered by `pack.lock.cbor`.
- Reuse existing ontology/retrieval validation paths where possible instead of
  creating a second ontology graph or search provider model.

## Acceptance criteria

- IR includes deterministic operational index and query requirement metadata.
- Pack output includes provider-agnostic index requirement handoff assets and
  extension metadata; it does not include concrete provider binding DDL.
- Validation rejects unknown records, fields, indexes, and `used_by` targets.
- Static query requirements fail without a declared index unless `scan_ok: true`
  is explicit.
- Landlord/tenant fixture includes exact and composite index requirements tied
  to existing records/projections.
- Existing ontology relationships and retrieval bindings continue to pass
  without migration or duplicate declarations.
