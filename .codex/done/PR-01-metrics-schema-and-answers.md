# PR-01 — Add metrics/KPI support to SoRLa wizard schema and answers.json

Repository: `greenticai/greentic-sorla`

## Goal

Add first-class `metrics` declarations to the SoRLa answer model so a business system of record can define trusted calculations and KPIs.

Metrics should be declared in `answers.json` and then rendered into SoRLa YAML and IR by later PRs.

## Rationale

A system of record should define not only the facts it stores, but also the business metrics derived from those facts.

Examples:

- daily clicks
- weekly revenue
- monthly costs
- gross margin
- conversion rate
- CAC
- ROAS
- MRR
- churn
- SLA breach rate

Without first-class metric definitions, every app, dashboard, provider, agent and reporting flow will reinvent calculations differently.

## Scope

Extend the wizard schema and answer document model to support:

```json
{
  "metrics": {
    "items": [
      {
        "name": "monthly_revenue",
        "label": "Monthly Revenue",
        "description": "Total paid revenue by calendar month.",
        "source": {
          "kind": "record",
          "name": "payment"
        },
        "measure": {
          "aggregate": "sum",
          "field": "amount"
        },
        "filters": [
          {
            "field": "status",
            "operator": "equals",
            "value": "paid"
          }
        ],
        "time": {
          "field": "paid_at",
          "grain": "month"
        },
        "unit": "GBP",
        "dimensions": [
          "product_id",
          "region"
        ]
      }
    ]
  }
}
```

## Current repo alignment

As of the current worktree, `AnswersDocument` and the concrete answer structs
live privately in `crates/greentic-sorla-lib/src/lib.rs`. The public
`greentic-sorla` binary is a thin wrapper over `greentic-sorla-lib`, so this PR
should update the library first and let the CLI pick it up through existing
facade paths.

`greentic-sorla wizard --schema` currently emits a custom wizard-question
document (`WizardSchema` with `WizardSection` and `WizardQuestion`), not a full
JSON Schema for `answers.json`. Add a metrics section to that existing question
catalog and keep any richer nested answer shape documented in examples/tests.

The YAML/parser/IR/design-model layers do not yet parse real metrics. The
current `SorlaDesignModel` and `ConceptViewModel` contain reserved metric view
slots, but they are empty until later PRs add YAML and IR support. Do not make
PR-01 assert metrics appear in design views or packs.

## Suggested answer types

Add internal answer model types for `MetricAnswers`, `MetricItemAnswer`, `MetricSourceAnswer`, `MetricMeasureAnswer`, `MetricFilterAnswer`, `MetricTimeAnswer`, `MetricWindowAnswer`, and `MetricTargetAnswer`.

The current `AnswersDocument` is private in `greentic-sorla-lib`. Extend it internally first. Avoid unnecessary public API churn.

## Source model

Support sources:

```text
record
event
projection
```

JSON shape:

```json
{
  "source": {
    "kind": "record",
    "name": "payment"
  }
}
```

## Measure model

Support:

```text
count
sum
avg
min
max
distinct_count
```

Examples:

```json
{ "aggregate": "count" }
{ "aggregate": "sum", "field": "amount" }
{ "aggregate": "avg", "field": "duration_seconds" }
{ "aggregate": "distinct_count", "field": "customer_id" }
```

## Time model

Support:

```text
hour
day
week
month
quarter
year
```

Example:

```json
{
  "time": {
    "field": "occurred_at",
    "grain": "day"
  }
}
```

## Window model

Support rolling windows:

```json
{
  "window": {
    "mode": "rolling",
    "size": 7,
    "unit": "days"
  }
}
```

Allowed units:

```text
hours
days
weeks
months
quarters
years
```

## Filters

Support a deterministic, restricted filter model:

```json
{
  "field": "status",
  "operator": "equals",
  "value": "paid"
}
```

Allowed operators:

```text
equals
not_equals
in
not_in
gt
gte
lt
lte
exists
not_exists
```

## Formula metrics

Support formula metrics for derived KPIs:

```json
{
  "name": "gross_margin",
  "formula": "(monthly_revenue - monthly_cost) / monthly_revenue",
  "depends_on": ["monthly_revenue", "monthly_cost"],
  "unit": "percentage"
}
```

Important: this PR should only model and validate the formula string shape lightly. A later PR should add strict expression parsing/validation. Do not execute arbitrary formulas.

## Wizard schema

Extend `greentic-sorla wizard --schema` with a new section:

```text
metrics
```

Questions should cover:

- whether metrics/KPIs are required
- metric names
- source record/event/projection
- aggregation
- time field and grain
- dimensions
- units
- optional target/threshold

## Backwards compatibility

Existing `answers.json` files without `metrics` must continue to work.

## Acceptance criteria

- `answers.json` supports a top-level `metrics` section.
- `greentic-sorla wizard --schema` includes metric-related questions/fields.
- Existing fixtures still validate.
- New fixture validates `daily_clicks`, `monthly_revenue`, `monthly_cost`, and `gross_margin`.
- No runtime metric execution is added in this PR.
- No provider-specific behavior is added in this PR.
