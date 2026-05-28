# PR-05 — Add SoRLa metrics docs, fixtures and tests

Repository: `greenticai/greentic-sorla`

## Goal

Document first-class SoRLa metrics and add deterministic fixtures/tests.

## Docs

Add:

```text
docs/metrics.md
```

Cover:

- what metrics are
- why they are separate from records/events/projections
- aggregate metrics
- formula metrics
- time grains
- rolling windows
- dimensions
- filters
- units
- KPI targets
- validation rules
- provider capability requirements
- SORX handoff expectations

## Fixtures

Add examples:

```text
examples/metrics-commerce/answers.json
examples/metrics-commerce/sorla.yaml.expected
examples/metrics-commerce/metrics.json.expected
```

The repo now also has YAML-first Designer fixtures under
`examples/designer-property-management/`. Keep metrics fixtures separate unless
they explicitly test the Designer concept-view/semantic-patch path. If they do,
add expected concept-view or diff fixture files alongside the metrics examples
rather than modifying unrelated Designer fixtures.

Suggested model:

Records/events:

- click
- visitor
- order
- payment
- cost_entry
- campaign

Metrics:

- daily_clicks
- monthly_revenue
- monthly_cost
- conversion_rate
- gross_margin
- campaign_roas

## Tests

Add tests for:

- answer parsing
- YAML rendering
- IR lowering
- validation failures
- pack artifact generation
- doctor/inspect output
- concept-view/preview metric rendering, because `SorlaDesignModel` already has
  reserved metric view fields

## Negative tests

Invalid cases:

- metric source does not exist
- sum over missing field
- unsupported aggregate
- unsupported grain
- formula dependency missing
- formula dependency cycle
- dimension field missing
- invalid rolling window size

## Acceptance criteria

- Docs explain metrics clearly.
- Fixtures are deterministic.
- Tests cover positive and negative cases.
- CI/local checks pass.
