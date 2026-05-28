# PR 04 - Sorla stack pack: generic application stack contract

## Repo

`greenticai/greentic-sorla`

## Goal

Make Sorla `.gtpack` outputs deployable as generic application stack packs.
Generic Greentic repos should see them as stack packs with capability metadata,
not as Sorla-specific objects.

This PR owns the Sorla-side contract only. The generic capability model and
generic deployer lifecycle live in their own repos; Sorla should adapt to their
current public surfaces rather than re-declare or fork them.

## Cross-repo review notes

`greentic-cap` currently validates capability identifiers with the `cap://`
scheme and models declarations as `offers`, `requires`, `consumes`, and
`profiles`. It does not have first-class `contracts` fields on offers or
requirements yet. Sorla should therefore encode contract IDs in metadata until
the generic capability crate promotes them to typed fields.

`greentic-deployer` deploys bundles through environment `CapabilitySlot`
bindings, `BundleDeployment`, `Revision`, and `TrafficSplit`. Sorla should not
depend on deployer internals, but its pack metadata must give bundle/deployer
tooling enough generic data to treat the pack as an application-stack artifact.

`greentic-start` and `greentic-setup` now route secrets through
`SecretsManager` / `secrets://...` references and no longer rely on plaintext
`setup-answers.json` fallback. Sorla packs must keep setup schemas,
secret-requirement metadata, and runtime templates aligned with that contract.

## Manifest contract

A Sorla stack pack should include a generic stack-pack manifest, for example
`stack-pack.json` and/or `assets/greentic/stack-pack.json`, using generic
capability identifiers:

```yaml
schema: greentic.stack-pack.v1

stack:
  id: invoice-assistant
  kind: application-stack
  version: 1.2.0

offers:
  - id: offer.stack.application
    capability: cap://greentic/stack/application/v1
    metadata:
      contracts:
        - greentic.stack.invoke.v1
        - greentic.stack.routes.v1
      routes:
        - main

requires:
  - id: require.runtime.host
    capability: cap://greentic/runtime/host/v1
    metadata:
      contracts:
        - greentic.runtime.invoke.v1
        - greentic.runtime.traffic.v1
  - id: require.secrets
    capability: cap://greentic/secrets/v1
  - id: require.telemetry
    capability: cap://greentic/telemetry/v1
    optional: true
  - id: require.extension.control
    capability: cap://greentic/extension/control/v1
    optional: true
  - id: require.extension.observer
    capability: cap://greentic/extension/observer/v1
    optional: true
  - id: require.extension.admin
    capability: cap://greentic/extension/admin/v1
    optional: true

routes:
  - id: main
    method: POST
    path: /invoke
    contract: greentic.stack.invoke.v1

setup:
  schema_ref: setup/schema.json
  secret_requirements_ref: assets/secret-requirements.json
```

Also include a `greentic_cap_schema::PackCapabilitySectionV1`-compatible
capability declaration payload where practical. The capability declaration and
the stack-pack manifest should be generated from the same data so capability
resolution, pack doctor, and inspect output cannot drift.

## Artifact requirements

Each built stack pack should include:

- generic stack-pack manifest
- generic capability declaration
- route declarations
- setup schema and secret requirements
- flow/graph/component references
- required secret declarations without plaintext values
- optional admin surface declarations if bundled
- Sorla/Sorx compatibility metadata for existing consumers
- digest/signature metadata where available

## Runtime invocation contract

The runtime invokes the stack through a generic contract:

```json
{
  "schema": "greentic.stack.call.request.v1",
  "call_id": "01...",
  "environment_id": "local",
  "deployment_id": "dep...",
  "revision_id": "rev...",
  "route_id": "main",
  "payload": {},
  "context": {}
}
```

Response:

```json
{
  "schema": "greentic.stack.call.response.v1",
  "call_id": "01...",
  "status": "success",
  "payload": {},
  "usage": {},
  "metadata": {}
}
```

The generic route/contract metadata is a dispatch surface. It must not require
core deployer/start/setup tooling to parse Sorla language semantics. Existing
Sorx-compatible assets can remain in the pack for runtime hosts that understand
them.

## Do not do

- Do not require core tooling to parse Sorla-specific semantics.
- Do not add `gtc sorla` as the main UX.
- Do not encode Sorla-specific names in generic capability IDs.
- Do not add deployer lifecycle orchestration to this repo.
- Do not write plaintext secrets into stack-pack manifests, setup artifacts, or
  runtime templates.

## Tests

- Sorla stack pack validates against the generic application stack schema.
- Capability declaration validates with `greentic-cap` rules, including
  `cap://` capability IDs.
- Capability resolver sees it as `cap://greentic/stack/application/v1`.
- Generic runtime-host requirement is satisfied by a fake runtime in tests.
- Required secrets are declared and exposed to setup.
- Secret declarations produce setup metadata compatible with `secrets://...`
  runtime resolution and do not rely on plaintext `setup-answers.json`.
- Routes validate without product-specific logic.
- `pack doctor` rejects malformed stack-pack/capability metadata and missing
  referenced route/setup/secret artifacts.
- `pack inspect` exposes the generic stack-pack summary.

## Acceptance criteria

A Sorla solution pack can be deployed by `gtc env deploy <env> <bundle>` as a generic stack artifact.
