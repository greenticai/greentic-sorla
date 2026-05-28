# PR-05 — Add SORX exposure policy metadata to `.gtpack`

## Repository

greenticai/greentic-sorla

## Objective

Add deterministic exposure policy metadata so SORX knows whether endpoints should remain private, can become public candidates, or require explicit operator approval after validation.

This is needed because automatic GHCR-driven deployment should not automatically publish endpoints simply because a pack exists.

Current implementation alignment:

- SoRLa agent endpoints currently have export-surface visibility only: `openapi`, `arazzo`, `mcp`, and `llms_txt`.
- They do not have concrete HTTP route prefixes, public aliases, gateway route IDs, or a `public` visibility state.
- This PR must emit conservative metadata for downstream SORX. It must not implement routing, promotion, public exposure, or deployment in this repo.

## Required changes

### 1. Define exposure policy schema

Add a new contract:

```text
greentic.sorx.exposure-policy.v1
```

Suggested asset path:

```text
assets/sorx/exposure-policy.json
```

### 2. Add Rust types

Create types similar to:

```rust
pub struct SorxExposurePolicy {
    pub schema: String,
    pub default_visibility: EndpointVisibility,
    pub promotion_requires: Vec<String>,
    pub allowed_route_prefixes: Vec<String>,
    pub forbidden_route_prefixes: Vec<String>,
    pub endpoints: Vec<SorxEndpointExposurePolicy>,
}

pub struct SorxEndpointExposurePolicy {
    pub endpoint_id: String,
    pub visibility: EndpointVisibility,
    pub requires_approval: bool,
    pub risk: Option<String>,
    pub export_surfaces: Vec<String>,
    pub route_prefixes: Vec<String>,
}
```

Reuse the `EndpointVisibility` enum from PR-02 where possible.

Implementation note: `route_prefixes` should default to an empty list until route metadata exists. `export_surfaces` should be derived from current endpoint visibility flags.

### 3. Generate default policy

Default behavior:

- Pack-level `default_visibility` must be `private`.
- Exported agent endpoints may be marked `public_candidate`, not `public`; this means "eligible for SORX evaluation", not publicly routed by SoRLa.
- High-risk or side-effectful endpoints must require approval.
- If risk metadata is missing, default to conservative behavior.

Generated policy should include:

```json
{
  "schema": "greentic.sorx.exposure-policy.v1",
  "default_visibility": "private",
  "promotion_requires": [
    "validation_success",
    "security_success",
    "provider_resolution_success"
  ],
  "forbidden_route_prefixes": [
    "/internal",
    "/debug",
    "/admin/raw"
  ]
}
```

### 4. Include in `.gtpack`

Add:

```text
assets/sorx/exposure-policy.json
```

Update `pack.cbor` to reference:

```text
greentic.sorx.exposure-policy.v1
```

Use the same additive manifest strategy as PR-04: add an `exposure_policy` path under the existing `extension.sorx` object unless a backwards-compatible extension registry has already been introduced. In the current manifest shape, `greentic.sorx.exposure-policy.v1` is the asset schema, not a replacement for the runtime extension id.

### 5. Doctor checks

Extend `pack doctor`:

- policy file exists if the manifest references `sorx.exposure_policy`
- schema is valid
- default visibility is not public
- any non-empty forbidden route prefixes are absolute and do not contain `..`
- endpoint IDs referenced by policy exist in agent endpoint metadata
- high-risk endpoints require approval
- exposure policy does not contradict validation manifest promotion requirements

### 6. Inspect output

Include:

```json
{
  "exposure_policy": {
    "default_visibility": "private",
    "public_candidate_endpoints": 2,
    "approval_required_endpoints": 1
  }
}
```

### 7. Tests

Add tests for:

- default policy generation
- high-risk endpoint requires approval
- invalid public default rejected
- unknown endpoint ID rejected
- policy included in pack lock metadata

## Acceptance criteria

- `.gtpack` contains exposure policy.
- Doctor enforces conservative defaults.
- SORX can safely use policy as a promotion gate.

## Non-goals

- Do not implement SORX promotion.
- Do not implement actual gateway routing.
