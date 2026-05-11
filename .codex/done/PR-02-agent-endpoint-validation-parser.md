# PR-02 — Add Agent Endpoint Parser Validation

## Goal

Add semantic validation for `agent_endpoints` after the AST exists.

This PR should prevent malformed endpoint declarations from entering canonical IR or handoff generation.

## Files to touch

- `crates/greentic-sorla-lang/src/parser.rs`
- `crates/greentic-sorla-lang/src/lib.rs`
- `crates/greentic-sorla-lang/src/ast.rs` only if helper methods are useful

## Current-code notes

- Parser diagnostics currently use `Result<ParsedPackage, String>` plus `ParsedPackage.warnings: Vec<ParseWarning>`, not a structured diagnostic type.
- Existing warning paths are string paths such as `records.<name>`. For new agent endpoint validation, prefer stable index paths like `agent_endpoints[0].id` where the PR calls for them, but do not introduce a broad diagnostics refactor in this PR.
- Parser normalization currently only applies v0.1 record-source compatibility. Do not reorder user-authored endpoint inputs during parsing; warn about optional-before-required if desired, and leave deterministic sorting to IR/export layers.
- `ParseWarning` lives in `ast.rs`; importing it into `parser.rs` is the expected path for warning additions.

## Validation rules

### Endpoint identity

- `agent_endpoints[].id` must be non-empty.
- IDs must be unique within a package.
- Recommended format: lowercase kebab/snake style: `[a-z][a-z0-9_-]*`.
- `title` and `intent` must be non-empty.

### Inputs

- Input names must be unique within an endpoint.
- Required inputs should appear before optional inputs in authoring. Emit a warning if optional inputs precede required inputs; do not reorder in the parser.
- `type` must be non-empty.
- If `enum_values` is present, values must be unique and non-empty.

### Outputs

- Output names must be unique within an endpoint.
- `type` must be non-empty.

### Side effects

- `side_effects` values must be non-empty.
- For high-risk endpoints, side effects must not be empty.

### Approval/risk consistency

- `risk: high` must require either:
  - `approval: required`, or
  - `approval: policy-driven`
- `approval: required` should reference at least one backing approval in `backing.approvals`, unless this is intentionally deferred to downstream policy. If deferred, emit a warning.

### Backing references

Validate that backing references point to declared blocks:

- `backing.actions[]` must exist in `actions[].name`
- `backing.events[]` must exist in `events[].name`
- `backing.flows[]` must exist in `flows[].name`
- `backing.policies[]` must exist in `policies[].name`
- `backing.approvals[]` must exist in `approvals[].name`

### Provider requirements

- Provider categories must be non-empty.
- Capabilities must be non-empty strings.
- Capabilities should be unique per category.

## Warning rules

Use existing parser warning mechanics where possible.

Add warnings for:

- Endpoint has no examples.
- Endpoint exposes MCP/OpenAPI/Arazzo but has no outputs.
- Endpoint has sensitive inputs but no approval/policy reference.
- Endpoint has `approval: optional` and `risk: high`, then reject instead of warn.

## Example failing cases

### Duplicate endpoint ID

```yaml
agent_endpoints:
  - id: create_contact
    title: Create Contact
    intent: Create a contact
  - id: create_contact
    title: Duplicate
    intent: Duplicate endpoint
```

Expected error contains:

```txt
duplicate agent endpoint id `create_contact`
```

### High risk without approval

```yaml
agent_endpoints:
  - id: delete_customer
    title: Delete customer
    intent: Delete a customer record
    risk: high
    approval: none
    side_effects:
      - crm.contact.delete
```

Expected error contains:

```txt
high-risk agent endpoint `delete_customer` must use approval: required or approval: policy-driven
```

## Tests

Add tests for:

1. Duplicate endpoint IDs rejected.
2. Duplicate input/output names rejected.
3. Empty intent rejected.
4. High-risk without approval rejected.
5. Backing action reference must exist.
6. Backing event reference must exist.
7. Provider category and capabilities validated.
8. Endpoint without examples emits warning but parses.

## Acceptance criteria

- `cargo test -p greentic-sorla-lang` passes.
- Validation errors include stable, specific paths where practical:
  - `agent_endpoints[0].id`
  - `agent_endpoints[0].inputs[1].name`
- Existing non-agent packages still parse unchanged.
