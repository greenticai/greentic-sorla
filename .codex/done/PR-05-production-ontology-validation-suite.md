# PR 05 — Add production ontology validation suite metadata

## Repository

`greenticai/greentic-sorla`

## Objective

Extend SoRLa `.gtpack` validation metadata so Sorx and CI can verify ontology, relationship, retrieval, and provider compatibility before public exposure or deployment promotion.

This must extend the existing `greentic.sorx.validation.v1` manifest shape in `crates/greentic-sorla-pack/src/sorx_validation.rs`: suites have `id`, optional `title`, `required`, and `tests`; gating is expressed through top-level `promotion_requires`. Do not replace the schema with a suite-level `kind` / `required_for_public_exposure` shape.

## New validation suites

Extend `assets/sorx/tests/test-manifest.json` with deterministic suite IDs and test kinds. Candidate suite IDs:

```text
ontology
retrieval
provider
security
```

Add new test `kind` enum variants only where needed, for example:

```text
ontology-static
ontology-relationship
ontology-alias
entity-linking
retrieval-binding
provider-capability
policy-enforced
```

Prefer reusing existing `provider-capability` and `policy-enforced` test kinds where they already cover the requirement.

## Test manifest shape

```json
{
  "schema": "greentic.sorx.validation.v1",
  "suite_version": "1.0.0",
  "package": {
    "name": "example-sor",
    "version": "0.1.0"
  },
  "default_visibility": "private",
  "promotion_requires": [
    "ontology",
    "retrieval",
    "provider"
  ],
  "suites": [
    {
      "id": "ontology",
      "title": "Ontology handoff checks",
      "required": true,
      "tests": [
        {
          "kind": "ontology-static",
          "id": "ontology-static",
          "required": true
        }
      ]
    },
    {
      "id": "retrieval",
      "title": "Retrieval binding checks",
      "required": true,
      "tests": [
        {
          "kind": "retrieval-binding",
          "id": "retrieval-bindings",
          "required": true
        }
      ]
    },
    {
      "id": "provider",
      "title": "Provider capability checks",
      "required": true,
      "tests": [
        {
          "kind": "provider-capability",
          "id": "provider-evidence-capabilities",
          "provider_category": "evidence",
          "capabilities": [
            "evidence.query",
            "entity.link"
          ],
          "required": true
        }
      ]
    }
  ]
}
```

## Requirements

1. Packs with public-candidate or agent-exported endpoints should add ontology/provider compatibility suites to `promotion_requires` when ontology artifacts exist.
2. High-risk or side-effectful endpoints should require policy validation.
3. Validation metadata must be deterministic.
4. Sorla generates metadata only. Sorx executes validation.
5. No runtime secrets should be present.
6. Private-only packs may include ontology suites with `required: false`; they must not be added to `promotion_requires` unless promotion policy requires them.

## Tests

Add tests for:

- validation manifest contains ontology suites
- public-candidate/exported endpoint pack adds required ontology/provider suites to `promotion_requires`
- private-only pack can mark ontology validation as recommended (`required: false`) without adding it to `promotion_requires`
- doctor validates suite references
- schema rejects the obsolete `kind` / `required_for_public_exposure` suite shape
- deterministic output

## Docs

Update:

- `docs/sorx-gtpack-validation.md`
- `docs/sorla-gtpack.md`
- `docs/ontology.md`

## Acceptance criteria

```bash
cargo test --all-features
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out /tmp/sor.gtpack
cargo run -p greentic-sorla -- pack validation-inspect /tmp/sor.gtpack
bash ci/local_check.sh
```
