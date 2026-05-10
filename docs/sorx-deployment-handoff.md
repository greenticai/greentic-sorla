# SORX Deployment Handoff

## Purpose

This document describes how downstream SORX tooling should consume
validation-enabled SoRLa `.gtpack` artifacts after they are published.

It is a handoff model, not runtime behavior implemented by `greentic-sorla`.
`greentic-sorla` emits deterministic pack metadata, schemas, and static doctor
checks. It does not implement GitHub webhooks, GHCR polling, digest or signature
policy, runtime deployment, provider resolution, public routing, validation
execution, promotion, rollback, or alias management.

Downstream tooling should discover installed binaries through the normal release
or binstall path. These examples intentionally do not depend on a sibling source
checkout.

## Event-Driven Lifecycle

A successful publish event should be treated as the start of a SORX-owned
certification lifecycle:

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

The static pack checks can be performed with an installed `greentic-sorla`
binary:

```bash
greentic-sorla pack doctor landlord-tenant-sor.gtpack
greentic-sorla pack inspect landlord-tenant-sor.gtpack
greentic-sorla pack validation-inspect landlord-tenant-sor.gtpack
```

SORX may also implement equivalent checks natively, but it must preserve the
same contract boundaries: static metadata verification happens before runtime
deployment, and runtime validation happens before public exposure.

## Concurrent Deployment Model

SORX should model pack artifacts, deployment instances, and public aliases as
separate records:

```text
pack artifact != deployment instance != public alias
```

An immutable pack version can have zero or more deployment instances. A public
alias points at one deployment instance at a time and should only move after the
target deployment is certified.

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

The `api_base` values above are illustrative SORX-owned routes. Current SoRLa
endpoint metadata exposes abstract export surfaces, not concrete public aliases
or route prefixes.

## Public Exposure Gate

SORX should only expose public endpoints when all required gates pass:

- pack source, digest, signature, and semantic version verification succeeds
- `pack doctor` or an equivalent static validation succeeds
- provider requirements resolve
- embedded validation suites required by `promotion_requires` pass
- exposure policy allows promotion
- operator policy does not block the deployment

The exposure policy is conservative by default. `public_candidate` means an
endpoint may be considered for public exposure by SORX after certification. It
does not mean `greentic-sorla` has created a public route.

## Certification Report Shape

SORX should persist a certification report for the exact deployment instance
that was validated. A report can use this shape:

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

The report is SORX-owned output. It should reference immutable pack identity,
deployment identity, suite results, and the final exposure decision. It should
not rewrite or mutate the source `.gtpack`.

## Rollback And Promotion

Because aliases are separate from deployments, rollback can be modeled as an
alias update back to a previously certified deployment. The pack artifact remains
immutable, and the previous deployment report remains the evidence for why that
deployment is eligible for public traffic.

Promotion should be an atomic SORX operation:

- verify the candidate report still matches the deployed pack digest
- verify the report result still satisfies operator policy
- move the alias from the old deployment to the certified deployment
- record the alias change for audit

`greentic-sorla` provides the deterministic metadata needed for this decision,
but SORX owns the decision and the routing change.

