# PR 12 — Add prompting and knowledge support to Sorla DesignExtension

## Repository

`greenticai/greentic-sorla`

## Objective

Extend the Sorla DesignExtension with prompt fragments and knowledge entries so Greentic Designer can guide LLM-assisted Sorla generation.

The extension should not own the LLM. It should provide structured guidance, examples, and validation tools that the Designer host can use.

## Implement WIT interfaces

Implement the DesignExtension `prompting` and `knowledge` exports from the SDK/WIT used in PR 11:

```text
system-prompt-fragments
list-entries
get-entry
suggest-entries
```

## Prompt fragments

Add prompt fragments such as:

### `sorla.modelling.principles`

```text
When generating Sorla models, prefer deterministic system-of-record structures:
records, ontology concepts, relationships, actions, events, projections, policies,
approvals, agent endpoints, and retrieval bindings. Never invent provider credentials.
Return diagnostics and questions when requirements are ambiguous.
```

### `sorla.ontology.rules`

```text
Use generic ontology concepts and relationship types. Records describe storage shape;
ontology describes business meaning. Relationships must reference existing concepts.
Avoid domain-specific core fields unless they belong to the user’s domain model.
```

### `sorla.safety.rules`

```text
Side-effectful actions require risk and approval metadata. Sensitive fields must be marked.
High-risk agent endpoints should be approval-driven by default.
```

## Knowledge entries

Add deterministic knowledge entries:

```text
sorla-system-of-record-guide
sorla-ontology-guide
sorla-agent-endpoint-guide
sorla-retrieval-binding-guide
sorla-policy-approval-guide
example-supplier-contract-risk
example-customer-onboarding
example-landlord-tenant
```

Each entry should include:

```json
{
  "id": "...",
  "title": "...",
  "category": "sorla",
  "tags": ["ontology", "system-of-record"],
  "content_json": {}
}
```

Where an entry includes a model example, store it as the same normalized model/answers shape accepted by the public facade from PR 09. Do not introduce a second example schema.

## Suggestion behavior

`suggest-entries(query, limit)` should be deterministic.

Use simple scoring:

1. exact tag match
2. title token match
3. category match
4. stable lexical ordering

No LLM required.

## Tests

Add tests for:

- prompt fragments present
- prompt priorities stable
- knowledge list stable
- get-entry works
- suggest-entries deterministic
- example entries contain valid JSON
- examples can be validated by `sorla-lib` where applicable
- no prompt fragment or knowledge entry contains credentials, concrete tenant IDs, or provider secrets

## Docs

Update:

```text
docs/designer-extension.md
docs/sorla-lib.md
```

Explain how the Designer host can combine:

```text
user prompt
+ prompt fragments
+ suggested knowledge entries
+ tool calls
```

## Acceptance criteria

```bash
cargo test --all-features
cargo build -p greentic-sorla-designer-extension --target wasm32-wasip2
bash ci/local_check.sh
```
