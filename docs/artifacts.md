# Artifact Layout

PR-03 introduces a deterministic IR and the first provider-facing artifact set.

## Goals

- keep downstream consumers off the raw author-authored YAML
- provide a stable contract for runtime packs and provider packs
- make serialization deterministic from day one

## Canonical Rules

- field ordering is deterministic
- name-based collections are sorted canonically before emission
- empty/default optional fields are omitted where the serializer supports it
- hashes are derived from the canonical serialized form only
- provider requirements remain abstract and category-based

## Current Artifact Set

The current pack emitter produces:

- `model.cbor`
- `actions.cbor`
- `events.cbor`
- `projections.cbor`
- `policies.cbor`
- `approvals.cbor`
- `views.cbor`
- `external-sources.cbor`
- `compatibility.cbor`
- `provider-contract.cbor`
- `package-manifest.cbor`
- `agent-tools.json`
- `executable-contract.json`

`model.cbor` contains the full canonical IR. The split artifacts are intended to
give downstream consumers narrower machine-readable contracts without requiring
them to parse user-authored YAML.

## Agent Endpoint Handoff Artifacts

When canonical IR contains `agent_endpoints`, `greentic-sorla-pack` can generate
deterministic handoff artifacts:

- `agent-gateway.json`
- `agent-endpoints.ir.cbor`
- `agent-endpoints.openapi.overlay.yaml`
- `agent-workflows.arazzo.yaml`
- `mcp-tools.json`
- `llms.txt.fragment`

These artifacts are provider-agnostic. They describe intent, inputs, outputs,
side effects, risk, approval behavior, provider requirement categories, and
export visibility for downstream `gtc` assembly. They do not contain concrete
provider URLs, credentials, OAuth configuration, or runtime gateway code.

The detailed downstream validation and ownership contract is documented in
`docs/agent-endpoint-handoff-contract.md`.

## gtpack Layout

`greentic-sorla wizard --answers <file> --pack-out <file.gtpack>` and
`greentic-sorla pack <file>` write deterministic `.gtpack` archives. Inside the
pack, SoRLa artifacts live under `assets/sorla/` and Sorx startup handoff assets
live under `assets/sorx/`.

The pack root includes `pack.cbor`, `pack.lock.cbor`, `manifest.cbor`, and
`manifest.json`. `pack.cbor` declares the `greentic.sorx.runtime.v1` extension,
and `pack.lock.cbor` records deterministic size and digest metadata for archive
entries.

See `docs/sorla-gtpack.md` for the command, required entries, optional agent
endpoint entries, and doctor checks.

## Executable Contract Artifact

`executable-contract.json` summarizes the parts of canonical IR that an e2e
harness or downstream executor can use directly:

- record field relationships declared with `references`
- migrations, including `idempotence_key` and `backfills`
- agent operations with first-class `emits` plans
- an operation result/error contract for downstream executors

The artifact carries the package name, version, and canonical IR hash. Consumers
should verify the hash before trusting it as an executable contract. The schema
is documented in `docs/spec/executable-contracts.md`.

## Current Scope

This milestone covers deterministic lowering and artifact emission for the
implemented v0.2 language slice. It does not yet cover import expansion, full
compiler output, runtime binding, or provider publishing.
