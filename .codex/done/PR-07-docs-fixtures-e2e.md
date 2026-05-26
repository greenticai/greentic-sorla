# PR-07 — Add YAML-first Designer extension docs, fixtures and e2e tests

Repository: `greenticai/greentic-sorla`

## Goal

Document and test the full YAML-first Designer architecture.

This PR should update the docs that already exist instead of duplicating them. Relevant current docs include:

- `docs/designer-extension.md`
- `docs/e2e/designer-prompt-to-sorla-gtpack.md`
- `docs/extensions-with-gtc.md`
- `docs/sorla-lib.md`
- `docs/agent-endpoints.md`
- `docs/sorla-gtpack.md`

## Docs

Add:

```text
docs/designer-yaml-source-of-truth.md
docs/concept-view-model.md
docs/semantic-patches.md
docs/designer-sdk-extension.md
```

Use these file names only if they remain the clearest split after reviewing the existing docs. Otherwise update/extend existing docs and add only the missing focused pages.

Cover:

- why `sorla.yaml` is source of truth
- role of `answers.json`
- role of canonical IR/design model
- role of `ConceptViewModel`
- semantic patch protocol
- conflict handling with source hash
- Designer extension integration through the real `greentic-extension-sdk-*`/WIT DesignExtension contract
- CLI downscaling
- LLM-assisted patch proposals
- pack generation from YAML

## README update

Add a short section:

```text
Designer integration
```

Explain:

```text
sorla.yaml -> ConceptViewModel -> Designer/CLI -> semantic patch -> sorla.yaml
```

## Fixtures

Add fixtures:

```text
examples/designer-property-management/sorla.yaml
examples/designer-property-management/concept-view.expected.json
examples/designer-property-management/add-postcode.patch.json
examples/designer-property-management/add-postcode.result.yaml
examples/designer-property-management/add-postcode.diff.expected.json
```

Prefer reusing the existing landlord/customer-contact/property-management fixtures where possible. Add a new fixture only when the existing examples cannot cover YAML -> view -> patch -> YAML deterministically.

## E2E tests

Test:

1. parse YAML
2. generate concept view
3. render CLI
4. apply patch
5. validate updated YAML
6. regenerate concept view
7. generate pack entries or `.gtpack` using existing APIs

## Designer SDK tests

If possible, test:

- `describe.json` validation against `greentic-extension-sdk-contract`
- WIT tool listing/invocation contracts, or the closest available SDK test harness
- tool input/output JSON schemas
- `generate_concept_view`
- `apply_sorla_patch`

## Conflict tests

Test:

- base hash mismatch
- stale patch rejected
- force/dry-run behavior if CLI supports it

## Acceptance criteria

- Docs explain full architecture.
- Fixtures are deterministic.
- E2E tests prove YAML -> view -> patch -> YAML.
- Designer extension SDK integration is documented against the real extension SDK/WIT contract.
- CLI and Designer share the same library APIs.
