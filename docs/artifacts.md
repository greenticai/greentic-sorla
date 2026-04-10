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

`model.cbor` contains the full canonical IR. The split artifacts are intended to
give downstream consumers narrower machine-readable contracts without requiring
them to parse user-authored YAML.

## Current Scope

This milestone covers deterministic lowering and artifact emission for the
implemented v0.2 language slice. It does not yet cover import expansion, full
compiler output, runtime binding, or provider publishing.
