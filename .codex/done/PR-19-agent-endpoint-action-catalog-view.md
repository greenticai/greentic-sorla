# PR 19 — Emit a SoRLa agent endpoint action catalog view

## Repository

`greenticai/greentic-sorla`

## Objective

Add an optional, deterministic catalog view over existing SoRLa agent endpoints
for search, prompt assistance, and design-time selection.

This must not introduce a second source of truth for runtime actions. The
current source of truth is:

```text
CanonicalIr.agent_endpoints
assets/sorla/executable-contract.json
assets/sorla/agent-gateway.json
assets/sorla/designer-node-types.json
```

The catalog should be a derived design-time view. Runtime binding must continue
to use locked endpoint metadata, not labels, aliases, natural language, or
catalog ordering.

## Artifact

If implemented as a pack asset, emit:

```text
assets/sorla/agent-endpoint-action-catalog.json
```

Schema:

```text
greentic.sorla.agent-endpoint-action-catalog.v1
```

Do not emit `assets/sorla/business-actions.json` unless the repository first
introduces a real Business Action Catalog language/IR model. This PR is scoped
to the current codebase and should derive from agent endpoints.

## Shape

The catalog should be deterministic and compact:

```json
{
  "schema": "greentic.sorla.agent-endpoint-action-catalog.v1",
  "package": {
    "name": "landlord-tenant-sor",
    "version": "0.1.0",
    "ir_hash": "sha256:..."
  },
  "actions": [
    {
      "id": "record_rent_payment",
      "version": "0.1.0",
      "label": "Record rent payment",
      "description": "Record a rent payment through the SoRLa agent endpoint.",
      "intent": "Record a rent payment.",
      "endpoint_ref": {
        "id": "record_rent_payment",
        "package": "landlord-tenant-sor",
        "version": "0.1.0",
        "contract_hash": "sha256:..."
      },
      "input_schema": {},
      "output_schema": {},
      "risk": "medium",
      "approval": "policy-driven",
      "side_effects": [],
      "provider_requirements": [],
      "backing": {
        "actions": [],
        "events": [],
        "flows": [],
        "policies": [],
        "approvals": []
      },
      "design": {
        "aliases": [],
        "tags": ["sorla", "agent-endpoint"]
      }
    }
  ]
}
```

## Requirements

1. Generate from `CanonicalIr.agent_endpoints`; do not parse Designer output to
   reconstruct the catalog.
2. Use the same canonical hash convention as `designer-node-types.json`.
3. Sort actions by stable endpoint ID.
4. Keep labels, aliases, tags, and descriptions design-time only.
5. Reuse existing endpoint input/output schema helpers where possible.
6. Include no secrets, provider credentials, tenant IDs, or absolute paths.
7. If no agent endpoints exist, either omit the asset from packs or emit an
   empty catalog consistently with existing optional artifact behavior.

## Pack / inspect / doctor

If the asset is included in packs:

- add the path to `pack.cbor` SoRLa extension metadata
- cover it with `pack.lock.cbor`
- include an inspect summary such as `agent_endpoint_action_catalog`
- doctor-check schema, package metadata, endpoint IDs, hash format, lock
  coverage, and consistency with `model.cbor`

Do not add runtime execution checks.

## Tests

Add focused tests for:

- deterministic catalog generation from the landlord/tenant or agent endpoint
  golden fixture
- catalog action count equals agent endpoint count
- endpoint refs match package/version/hash from canonical IR
- labels/aliases are not required for runtime identity
- pack inclusion and inspect summary, if emitted as a pack asset
- doctor rejection for unknown endpoint ID or mismatched hash

## Docs

Update SoRLa-local docs only:

```text
docs/agent-endpoints.md
docs/designer-extension.md
docs/sorla-gtpack.md
```

Document that this is a design-time catalog view over agent endpoints, not a
new runtime Business Action Catalog.

## Acceptance criteria

```bash
cargo test -p greentic-sorla-pack -p greentic-sorla-lib
cargo run -p greentic-sorla -- pack examples/landlord-tenant/sorla.yaml --name landlord-tenant-sor --version 0.1.0 --out /tmp/landlord.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/landlord.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/landlord.gtpack
bash ci/local_check.sh
```
