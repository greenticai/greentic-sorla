# Agent Endpoint Handoff Contract

## Contract Purpose

This document defines how `greentic-sorla` emits agent endpoint handoff metadata
and how downstream `gtc` tooling should consume it.

Agent endpoint handoff artifacts describe agent-facing business actions in a
provider-agnostic form. They are deterministic build outputs, not runtime
routes, provider implementations, credential definitions, or final gateway
configuration.

## Ownership Boundaries

`greentic-sorla` emits the contract.

`gtc` decides whether to accept, normalize, enrich, package, and expose it.

Provider repositories implement the concrete API calls, OAuth flows,
credentials, and runtime operations.

This repository should not move `gtc` runtime decisions into SoRLa packaging.
The handoff contract is intentionally narrow: it preserves author intent,
safety metadata, deterministic IR identity, export visibility, and abstract
provider requirements.

## Artifact List

The stable handoff artifact names are:

- `agent-gateway.json`
- `agent-endpoints.ir.cbor`
- `agent-endpoints.openapi.overlay.yaml`
- `agent-workflows.arazzo.yaml`
- `mcp-tools.json`
- `llms.txt.fragment`

Not every artifact has to be generated for every package. `agent-gateway.json`
is the manifest entry point. The OpenAPI overlay, Arazzo workflows, MCP tools,
and `llms.txt` fragment depend on endpoint export visibility.

`agent-endpoints.ir.cbor` carries the canonical IR bytes for the handoff hash
domain when a downstream bundle wants a handoff-named IR payload. Full-package
canonical IR also remains available through the existing `model.cbor` artifact.

## Artifact Naming

The legacy `agent-tools.json` helper remains distinct from `mcp-tools.json`.
`mcp-tools.json` is the MCP handoff descriptor for agent endpoint tools and
uses the `greentic.mcp.tools.handoff.v1` schema string.

The OpenAPI artifact is an overlay fragment, not a full provider OpenAPI
document. The Arazzo artifact is a workflow fragment for downstream completion,
not a provider-bound runtime plan.

## Required Fields

`agent-gateway.json` must include:

- `schema` set to `greentic.agent-gateway.handoff.v1`
- package `name`
- package `version`
- package `ir_version`
- package `ir_hash`
- endpoint IDs
- endpoint titles and intents
- endpoint risk and approval modes
- endpoint input and output names
- endpoint side effects
- endpoint export visibility
- aggregated abstract provider requirement categories

Each endpoint ID must be unique within the package and stable across repeated
emission from the same source package.

## Optional Fields

Optional handoff data may include:

- endpoint examples
- input enum values
- input sensitivity markers
- backing actions, events, policies, approvals, views, records, and external
  sources
- endpoint-level provider requirement categories
- exporter-specific metadata for OpenAPI, Arazzo, MCP, or documentation

Optional fields must be additive. Consumers should not require optional data
unless their own packaging mode explicitly depends on it.

## Determinism Requirements

`greentic-sorla` should keep output ordering stable:

- endpoints sorted by ID
- inputs and outputs sorted by name in canonical IR
- provider categories sorted by category
- provider capabilities sorted lexically
- JSON and YAML maps emitted in deterministic order where the serializer allows
  it

Hashes are derived from canonical serialized IR bytes. A content-equivalent
package should produce the same handoff hash every time.

## How `gtc` Should Validate Handoff

`gtc` should validate the handoff before exposing it:

- the manifest schema string equals `greentic.agent-gateway.handoff.v1`
- package name and version are present
- the IR hash matches supplied IR bytes if both are present
- endpoint IDs are unique
- export files referenced by the manifest exist
- high-risk endpoints have required or policy-driven approval
- provider requirements can be resolved by configured providers
- no provider-specific credential is embedded in handoff metadata

Unknown major schema versions should be rejected. Unknown additive fields on a
known major version may be ignored or preserved by downstream tooling.

## Provider Requirement Resolution

Provider requirements are abstract. For example:

```yaml
provider_requirements:
  - category: crm
    capabilities:
      - contacts.read
      - contacts.write
```

`gtc` or downstream provider selection should resolve those requirements to an
actual provider such as HubSpot, Salesforce, Microsoft Dynamics, or a custom CRM
provider.

The handoff metadata must not embed provider credentials, OAuth client secrets,
tenant tokens, provider-specific base URLs, or runtime auth material.

## Future Compatibility Strategy

The first manifest schema is `greentic.agent-gateway.handoff.v1`.

Additive changes may add optional fields within `v1`. Breaking changes require
a `v2` schema. `gtc` should reject unknown major versions rather than guessing
at runtime behavior.

`greentic-sorla` should keep deterministic output ordering stable so repeated
builds remain comparable in Git, CI, and downstream pack assembly.
