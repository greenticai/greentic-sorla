# PR 03: Implement canonical IR and provider-facing artifact generation

    **Repository:** `greenticai/greentic-sorla`

    ## Objective

    Build the canonical intermediate representation and artifact generation pipeline for SoRLa v0.2 so that `greentic-sorla` produces stable, deterministic outputs that runtime packs and provider packs can consume.

    ## Why this PR exists

    The language alone is not enough. Providers, runtime components, and demos need a stable contract describing records, actions, events, projections, external references, policies, and compatibility rules. This PR creates that contract layer.

    ## Scope

    Implement a canonical IR that is:
- fully resolved
- import-expanded
- deterministic
- versioned
- hashable where appropriate
Then generate artifacts such as:
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
- `agent-tools.json`
Keep the artifact structure inspectable and stable.

    ## Deliverables

    - IR crate structures for records, events, projections, external refs, provider requirements
- semantic lowering from AST to IR
- artifact emission code
- golden tests ensuring deterministic output
- an `inspect` or debug representation suitable for tests and docs

    ## Implementation notes for Codex

    Make the IR explicitly separate:
- business meaning
- read-model/projection definitions
- provider/runtime needs
This is important because `greentic-sorla-providers` should implement the provider contract without needing to parse user-authored YAML directly. Build a few fixture packages and generate golden outputs from them. Avoid embedding FoundationDB assumptions directly into the IR; provider requirements should be declared abstractly, with provider selection handled later.

    ## Acceptance criteria

    - Fixture packages compile to deterministic IR and artifacts
- Tests show artifacts change only when inputs change materially
- External refs and projections are represented in artifacts
- Provider requirements are emitted in machine-readable form
- Artifact naming/layout is documented and stable enough for downstream repos

    ## Non-goals

    - Full end-user CLI polish
- Provider implementations
- Full wizard UI
- Pack publishing

    ## Suggested files / areas to touch

    - `crates/greentic-sorla-ir/`
- `crates/greentic-sorla-pack/`
- `tests/golden/*`
- `docs/artifacts.md`
