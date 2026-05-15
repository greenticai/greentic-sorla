# PR-10 — Document SORX deployment handoff for concurrent version deployment and public exposure gates

## Repository

greenticai/greentic-sorla

## Objective

Add documentation and examples explaining how SORX should consume validation-enabled SORLA `.gtpack` artifacts after GHCR publish events.

This PR gives SORX implementers an exact handoff model without putting runtime deployment logic in `greentic-sorla`.

Current implementation alignment:

- The implementation of GHCR event handling, digest/signature policy, preview deployment, provider resolution, validation execution, promotion, rollback, and public routing belongs to downstream `greentic-sorx` tooling.
- This PR may document the expected downstream lifecycle and show example data shapes, but it must not add runtime/deployment code to `greentic-sorla`.
- Current SoRLa endpoint metadata has export surfaces, not concrete routes or public aliases. Any route examples in this doc are illustrative SORX-owned examples.

## Required changes

### 1. Add handoff doc

Create:

```text
docs/sorx-deployment-handoff.md
```

### 2. Document event-driven lifecycle

Include this flow as a downstream SORX lifecycle, not as a `greentic-sorla` implementation flow:

```text
GitHub Action publishes .gtpack to GHCR
  -> successful publish event/webhook arrives at SORX
  -> SORX verifies source, digest, signature, and semantic version
  -> SORX pulls immutable pack artifact
  -> SORX runs greentic-sorla-compatible pack doctor checks or equivalent validation
  -> SORX creates isolated preview deployment for that exact pack version
  -> SORX resolves provider requirements
  -> SORX runs embedded validation suite
  -> SORX records certification report
  -> SORX promotes endpoint alias only if validation and exposure policy pass
```

### 3. Document concurrent deployment model

Explain that SORX should model:

```text
pack artifact != deployment instance != public alias
```

Example:

```yaml
deployments:
  - deployment_id: landlord-tenant-v1
    pack_name: landlord-tenant-sor
    pack_version: 1.0.0
    api_base: /sorx/acme/landlord-tenant/v1
    status: public

  - deployment_id: landlord-tenant-v2-preview
    pack_name: landlord-tenant-sor
    pack_version: 2.0.0
    api_base: /sorx/acme/landlord-tenant/v2
    status: preview

aliases:
  stable: landlord-tenant-v1
  preview: landlord-tenant-v2-preview
```

### 4. Document public exposure gate

SORX should only expose public endpoints when:

- pack digest/signature verification succeeds
- `pack doctor` succeeds
- provider requirements resolve
- embedded validation suites required by `promotion_requires` pass
- exposure policy allows promotion
- operator policy does not block the deployment

### 5. Document validation report shape

Add an example certification report shape SORX can emit:

```json
{
  "schema": "greentic.sorx.validation-report.v1",
  "deployment_id": "landlord-tenant-v2-preview",
  "pack": {
    "name": "landlord-tenant-sor",
    "version": "2.0.0",
    "digest": "sha256:..."
  },
  "result": "passed",
  "suites": [
    { "id": "smoke", "result": "passed" },
    { "id": "contract", "result": "passed" },
    { "id": "security", "result": "passed" },
    { "id": "provider", "result": "passed" }
  ],
  "public_exposure_allowed": true
}
```

### 6. Cross-link docs

Add links from:

- README
- `docs/sorla-gtpack.md`
- `docs/sorx-gtpack-validation.md`

Keep README wording clear that these are SORX handoff expectations and examples, not features executed by the `greentic-sorla` binary.

### 7. Keep boundaries clear

Explicitly state:

`greentic-sorla` does not implement GitHub webhooks, GHCR polling, runtime deployment, public routing, SORX validation execution, or rollback. It only emits the deterministic metadata and schemas needed by SORX.

## Acceptance criteria

- SORX implementers can understand exactly how to consume validation-enabled packs.
- Docs cover GHCR successful-publish webhook flow.
- Docs cover concurrent version deployment.
- Docs cover public endpoint gating.
- No runtime code is added.
