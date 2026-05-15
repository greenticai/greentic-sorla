# PR 18 — SoRLa Designer node type security and compatibility hardening

## Repository

`greenticai/greentic-sorla`

## Objective

Harden the SoRLa-owned Designer node type and locked endpoint reference path.

This PR is scoped to deterministic metadata generated and validated by
`greentic-sorla`. Do not implement Designer SDK schema ownership, Greentic Flow
runtime behavior, component execution, Sorx hash verification, provider calls,
audit sinks, approval runtime decisions, deployment, or bundle assembly.

## Runtime identity rule

Generated SoRLa metadata must never instruct downstream runtime systems to
select behavior by label, alias, natural language, or intent.

The only runtime identity emitted by this repo for Designer node types is:

```text
endpoint_id + package_version + package_name + contract_hash
```

Design-time labels, descriptions, aliases, prompt guidance, and UI fields are
allowed only for search, display, prompt assistance, node selection, and docs.

## Required hardening

Extend SoRLa-local validation where needed:

1. `designer-node-types.json` must fail doctor if a node type lacks
   `endpoint_ref`.
2. `endpoint_ref.contract_hash` must use `sha256:<64 lowercase hex chars>`.
3. `endpoint_ref.id`, `endpoint_ref.package`, and `endpoint_ref.version` must
   match canonical `model.cbor`.
4. Node type binding must be `kind: component`.
5. Node type binding operation must match the configured/default SoRLa operation
   expected by this repo.
6. Required endpoint inputs must appear in the node input schema.
7. Extension-generated flow nodes must include locked `endpoint_ref`.
8. Extension-generated flow nodes must reject missing required mappings.
9. Generated node type metadata must not contain secret-like values.
10. Generated node and flow metadata must not contain free-text runtime action
    selection fields such as `action_label`, `action_alias`, `intent_query`, or
    `natural_language_action`.

If any item is already covered, add focused regression tests instead of
rewriting the implementation.

## Negative tests

Add SoRLa-local tests for malformed generated artifacts or extension requests:

- node type without `endpoint_ref`
- node type with bad hash format
- node type with endpoint ID that is not in canonical IR
- node type with package/version/hash that does not match canonical IR
- node type with non-component binding
- node type with unsupported operation
- generated flow node request missing a required input mapping
- generated flow node request for an unknown node type
- secret-like value in node type metadata
- free-text runtime action selection field in generated node or flow metadata

These tests should live in the pack/facade/Designer extension crates depending
on which layer owns the behavior.

## Docs

Update SoRLa-local docs only:

```text
docs/designer-extension.md
docs/agent-endpoints.md
docs/sorla-gtpack.md
```

Document the security boundary:

- SoRLa emits deterministic locked endpoint metadata.
- SoRLa statically validates its generated pack metadata.
- Downstream Designer, Flow, component, Sorx, provider, audit, and approval
  systems must perform their own runtime validation outside this repo.

## Acceptance criteria

```bash
cargo test -p greentic-sorla-pack -p greentic-sorla-lib -p greentic-sorla-designer-extension
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
bash ci/local_check.sh
```

The implementation must not add cross-repo dependencies, network requirements,
runtime services, provider credentials, component WASM fixtures, or Sorx/Flow
commands to this repository.
