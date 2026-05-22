# PR: Add versioned SoRLa view contracts

Repo: `greenticai/greentic-sorla`

## Goal

Let a SoRLa package describe multiple versioned read/write view contracts over
the existing canonical IR model while preserving the current simple `views`
authoring form.

## Current-code assumptions

- `package.name` and `package.version` are already the package identity; do not
  introduce a separate `sor_id` identity unless a downstream contract requires
  it.
- `CanonicalIr` already carries the canonical model in `model.cbor`; `ir_version`
  is the compiler contract version and must not be reused as a business model
  version.
- `views` already exists in the v0.2 language and currently lowers as
  `Vec<NamedItemIr>`.
- `views.cbor` is already emitted in the loose artifact set and in `.gtpack`
  under `assets/sorla/`; this PR should enrich that artifact, not invent a
  parallel `assets/sorla/views.cbor` path as if no view artifact exists.
- Follow the extension-first boundary: SoRLa emits deterministic handoff
  metadata and static validation, while `gtc`/Sorx own runtime routing and final
  assembly.

## Proposed authoring shape

Keep legacy simple views valid:

```yaml
views:
  - name: TenantSummary
```

Add an extended view object shape:

```yaml
views:
  - name: TenantSummary
    version: 1.0.0
    mode: read-only
    maps_from:
      record: Tenant
      fields:
        tenant_id: id
        display_name: full_name
        preferred_contact: preferred_contact_method

  - name: TenantWriteV2
    version: 2.0.0
    mode: read-write
    maps_from:
      record: Tenant
      fields:
        tenant_id: id
        full_name: full_name
    writes:
      agent_endpoint: update_tenant_contact
      input_mapping:
        tenant_id: tenant_id
        phone: preferred_contact
```

If package-level version grouping is needed, add it as optional metadata rather
than replacing package identity:

```yaml
view_versions:
  canonical_model_version: 2.0.0
  served:
    - 1.0.0
    - 2.0.0
  accepts_writes_from:
    - 2.0.0
```

## Implementation notes

- Replace or extend the language `NamedBlock` view representation with a typed
  `ViewDecl` while keeping `- name: ...` compatibility.
- Lower into a richer canonical view IR instead of `NamedItemIr`, and include
  the richer view data in the canonical hash through `model.cbor`.
- Continue emitting `views.cbor`; update pack doctor to verify the `.gtpack`
  copy matches the view section in `model.cbor` and is covered by
  `pack.lock.cbor`.
- Validate record/field references, endpoint references used by write mappings,
  unique `(name, version)` pairs, and read-only/read-write consistency.
- Keep runtime concerns out of this PR: no public route creation, no Sorx
  deployment behavior, and no final bundle assembly.

## Acceptance criteria

- Parser accepts both legacy simple views and typed versioned view declarations.
- Canonical IR represents versioned view contracts without changing the meaning
  of `ir_version`.
- Existing examples and golden tests remain valid with the default/simple view
  shape.
- `views.cbor` and `assets/sorla/views.cbor` are enriched consistently with
  `model.cbor`.
- Landlord/tenant fixture demonstrates at least two concurrently served view
  versions and one read-only legacy view.
- `pack doctor` statically validates view artifact consistency and lock
  coverage without executing runtime routes.
