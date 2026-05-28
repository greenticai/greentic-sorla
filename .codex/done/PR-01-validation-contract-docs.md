# PR-01 — Define SORX validation contract for SoRLa `.gtpack`

## Repository

greenticai/greentic-sorla

## Objective

Introduce the formal documentation contract for embedding SORX-executable validation suites inside SoRLa `.gtpack` artifacts.

The purpose is to make every generated SoRLa runtime pack self-describing and self-validating so `greentic-sorx` can decide whether a deployed pack version is safe to expose publicly.

## Background

Current docs establish that:

- `greentic-sorla` owns authoring, canonical IR, deterministic `.gtpack` handoff artifacts, and abstract metadata.
- Downstream `greentic-sorx` tooling later consumes the pack and performs provider resolution, runtime startup, endpoint exposure, policy enforcement, and runtime assembly.
- The `.gtpack` is the runtime handoff contract.

This PR extends that handoff contract with validation metadata only. `greentic-sorla` must not become a runtime or deployment tool.

Current implementation alignment:

- The active pack manifest is a local SoRLa manifest type with one `extension` JSON object whose `extension` field is `greentic.sorx.runtime.v1`.
- This repo currently emits deterministic SoRLa assets plus SORX startup assets: `start.schema.json`, `start.questions.cbor`, `runtime.template.yaml`, and `provider-bindings.template.yaml`.
- Agent endpoints are authoring/handoff metadata and export surfaces (`openapi`, `arazzo`, `mcp`, `llms_txt`), not concrete HTTP routes or public aliases.
- Validation docs must describe metadata for downstream `greentic-sorx`; they must not require SORX execution, GHCR hooks, runtime deployment, or public routing in this repo.

## Required changes

### 1. Add new documentation file

Create:

```text
_docs/sorx-validation.md_ or _docs/sorx-gtpack-validation.md_
```

Use existing doc style from `docs/sorla-gtpack.md` and `docs/agent-endpoint-handoff-contract.md`.

The document must define:

```text
greentic.sorx.validation.v1
```

as the embedded validation-suite extension schema for SORX.

### 2. Document pack asset layout

Add a new section defining these pack entries:

```text
assets/sorx/tests/test-manifest.json
assets/sorx/tests/data/*.json
assets/sorx/tests/contracts/*.json
assets/sorx/tests/security/*.json
assets/sorx/tests/providers/*.json
assets/sorx/tests/migrations/*.json
assets/sorx/tests/fixtures/*.json
```

Only `test-manifest.json` should be mandatory when validation is enabled. Other directories are optional and referenced by relative path.

### 3. Define test manifest shape

Document this minimum shape:

```json
{
  "schema": "greentic.sorx.validation.v1",
  "suite_version": "1.0.0",
  "package": {
    "name": "landlord-tenant-sor",
    "version": "0.1.0"
  },
  "default_visibility": "private",
  "promotion_requires": [
    "smoke",
    "contract",
    "security",
    "provider"
  ],
  "suites": [
    {
      "id": "smoke",
      "required": true,
      "tests": []
    }
  ]
}
```

### 4. Define supported test kinds

Document the initial test kinds, even if later PRs only scaffold some of them:

- `healthcheck`
- `agent-endpoint`
- `openapi-contract`
- `mcp-tool-contract`
- `arazzo-workflow`
- `provider-capability`
- `provider-connectivity`
- `auth-required`
- `policy-enforced`
- `tenant-isolation`
- `migration-compatibility`
- `rollback-compatibility`

### 5. Define ownership boundary

Make explicit:

`greentic-sorla` owns:

- deterministic validation manifest generation
- schema definitions
- pack inclusion
- doctor/inspect validation of assets
- static consistency checks

`greentic-sorx` owns:

- pulling from GHCR
- verifying signatures/digests
- deploying preview runtimes
- executing tests
- resolving providers
- running migrations
- gating public exposure
- storing validation reports
- promoting/rolling back deployments

These SORX-owned responsibilities must be documented as downstream expectations only. Do not add implementation tasks for them to `greentic-sorla`.

### 6. Update existing docs

Update `docs/sorla-gtpack.md`:

- Add validation assets to the pack contents list.
- Add a new section: `Embedded SORX Validation`.
- State that SORX must not expose public endpoints unless validation succeeds or a local operator policy explicitly overrides it.

Update `README.md`:

- Add a short paragraph under `gtpack Handoff` mentioning embedded validation metadata for downstream SORX.

## Acceptance criteria

- New validation docs exist.
- Existing docs mention validation assets.
- Docs clearly state that validation metadata is deterministic and runtime execution belongs to SORX.
- No code behavior changes are required in this PR.

## Non-goals

- Do not implement test execution.
- Do not implement SORX.
- Do not implement GHCR webhook handling.
- Do not change pack output yet.
