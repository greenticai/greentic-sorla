# PR 20 — Harden endpoint contract hash and pack lock metadata

## Repository

`greenticai/greentic-sorla`

## Objective

Strengthen the SoRLa-owned contract hash and pack lock behavior used by
Designer node types, generated flow-node JSON, and any optional action catalog
view.

The current code already uses canonical IR hashes in locked endpoint refs and
`pack.lock.cbor` for archive entry integrity. This PR should add missing
regression coverage and targeted validation rather than introducing
`business-actions.lock.json`.

## Current source of truth

Use existing artifacts:

```text
assets/sorla/model.cbor
assets/sorla/executable-contract.json
assets/sorla/designer-node-types.json
pack.lock.cbor
```

If PR-19 adds `assets/sorla/agent-endpoint-action-catalog.json`, include it in
the same validation pattern.

## Hashing rules

1. Contract hash must be derived from canonical SoRLa IR bytes or the existing
   canonical IR hash helper.
2. Hash strings exposed in JSON metadata must use:

   ```text
   sha256:<64 lowercase hex chars>
   ```

3. The hash must change when endpoint contract-relevant metadata changes.
4. The hash must be stable across repeated generation from identical input.
5. Labels, aliases, descriptions, and UI-only fields must not become runtime
   identity.

If the implementation needs a narrower endpoint-level hash later, add it as a
new SoRLa-owned field with explicit derivation rules. Do not silently replace
the existing canonical IR hash semantics.

## Doctor rules

Extend or confirm `pack doctor` checks for:

- `designer-node-types.json` endpoint refs use the canonical hash
- optional catalog endpoint refs use the canonical hash
- malformed hash formats are rejected
- `pack.lock.cbor` covers every emitted JSON metadata asset
- lock entry size/hash matches archive contents
- stale or tampered JSON metadata fails closed

Do not add Sorx runtime hash verification, component execution, or provider
checks in this repo.

## Tests

Add or verify tests for:

- deterministic hash across repeated pack builds
- hash changes when endpoint input/output/risk/approval/backing metadata changes
- doctor rejects `sha256:not-a-real-hash`
- doctor rejects uppercase or non-hex hash strings
- doctor rejects mismatched endpoint ref hash
- doctor rejects a metadata asset not covered by `pack.lock.cbor`
- generated extension flow-node JSON contains the locked hash

## Docs

Update:

```text
docs/agent-endpoints.md
docs/designer-extension.md
docs/sorla-gtpack.md
```

Document the SoRLa-owned guarantee: generated metadata carries deterministic
locked endpoint refs and pack lock coverage. Runtime hash enforcement remains a
downstream responsibility.

## Acceptance criteria

```bash
cargo test -p greentic-sorla-pack -p greentic-sorla-lib -p greentic-sorla-designer-extension
cargo run -p greentic-sorla -- pack doctor /tmp/landlord.gtpack
bash ci/local_check.sh
```
