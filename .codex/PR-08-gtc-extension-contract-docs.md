# PR-08 — Document gtc Extension Contract for Agent Endpoint Handoff

## Goal

Document exactly how `gtc` should consume the agent endpoint artifacts emitted by `greentic-sorla`.

This PR is documentation-first and should not implement `gtc` behavior inside this repository.

## Files to touch

- `docs/extensions-with-gtc.md`
- New doc: `docs/agent-endpoint-handoff-contract.md`
- `docs/naming-migration.md` if artifact naming needs compatibility notes
- `README.md` small cross-link only

## Current-code notes

- `docs/extensions-with-gtc.md` and `docs/naming-migration.md` do not currently exist. Create them only if this PR needs those broader docs; otherwise touch `docs/agent-endpoint-handoff-contract.md`, `docs/architecture.md` or `docs/artifacts.md`, and `README.md`.
- Existing artifact docs live in `docs/artifacts.md`, and packaging docs live in `docs/packaging.md`. Add cross-links there if artifact names are introduced.
- Existing generated helper output `agent-tools.json` is already documented. Keep the new MCP handoff file name `mcp-tools.json` distinct, or document any compatibility alias explicitly.

## New document structure

Create `docs/agent-endpoint-handoff-contract.md`.

Recommended sections:

1. Contract purpose
2. Ownership boundaries
3. Artifact list
4. Artifact naming
5. Required fields
6. Optional fields
7. Determinism requirements
8. How `gtc` should validate handoff
9. How provider repos should consume requirements
10. Future compatibility strategy

## Artifact list

Document these outputs:

```txt
agent-gateway.json
agent-endpoints.ir.cbor
agent-endpoints.openapi.overlay.yaml
agent-workflows.arazzo.yaml
mcp-tools.json
llms.txt.fragment
```

Clarify that not all artifacts must be generated if endpoint visibility disables them.

## Ownership boundary text

Use explicit language:

```md
`greentic-sorla` emits the contract.
`gtc` decides whether to accept, normalize, enrich, package, and expose it.
Provider repositories implement the concrete API calls, OAuth flows, credentials, and runtime operations.
```

## `gtc` validation expectations

Document that `gtc` should validate:

- schema string equals `greentic.agent-gateway.handoff.v1`
- package name/version are present
- IR hash matches supplied IR bytes if both are present
- endpoint IDs are unique
- export files referenced by manifest exist
- high-risk endpoints have required or policy-driven approval
- provider requirements can be resolved by configured providers
- no provider-specific credential is embedded in handoff metadata

## Provider resolution expectations

Document that provider requirements are abstract, for example:

```yaml
provider_requirements:
  - category: crm
    capabilities:
      - contacts.read
      - contacts.write
```

`gtc` or downstream provider selection should resolve that to an actual provider such as HubSpot, Salesforce, Microsoft Dynamics, or a custom CRM provider.

## Future compatibility

Add versioning guidance:

- Start with `greentic.agent-gateway.handoff.v1`.
- Additive changes may add optional fields.
- Breaking changes require `v2`.
- `gtc` should reject unknown major versions.
- `greentic-sorla` should keep deterministic output ordering stable.

## README update

Add a link:

```md
See `docs/agent-endpoint-handoff-contract.md` for the downstream `gtc` handoff contract.
```

## Tests

Docs-only PR may not need tests. If repo has docs linting, ensure it passes.

## Acceptance criteria

- The docs clearly explain how agent endpoint metadata flows from SoRLa to `gtc`.
- No code ownership is moved from `gtc` into `greentic-sorla`.
- Artifact names and schema names are stable and searchable.
