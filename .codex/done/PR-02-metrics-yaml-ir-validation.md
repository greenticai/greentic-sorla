# PR-02 — Render metrics into SoRLa YAML, canonical IR and validation

Repository: `greenticai/greentic-sorla`

## Goal

Render `metrics` from `answers.json` into generated `sorla.yaml`, lower them into canonical IR, and add deterministic validation.

This PR builds on PR-01.

## Current repo alignment

The language AST is in `crates/greentic-sorla-lang/src/ast.rs`; it currently has
top-level records, events, projections, actions, policies, approvals,
migrations, provider requirements, and agent endpoints, but no `metrics` field.
Add metrics there first so `parse_package` and canonical YAML rendering can
round-trip the top-level `metrics:` section.

The canonical IR is in `crates/greentic-sorla-ir/src/lib.rs`; `PackageIr`
currently carries package metadata and separate vectors for records, events,
projections, policies, approvals, provider requirements, and agent endpoints.
Add metric IR types to that crate rather than inventing a parallel metric model
inside `greentic-sorla-lib`.

The Designer model already reserves `SorlaMetricView` and concept-view
`metric-board`/`metric-card` rendering paths in `greentic-sorla-lib`, but
`parse_sorla_yaml` currently fills metrics with `Vec::new()`. This PR should
connect those reserved fields to real lowered metric data.

## SoRLa YAML shape

Render metrics as a top-level section:

```yaml
metrics:
  - name: daily_clicks
    label: Daily Clicks
    description: Count of click events per day.
    source:
      kind: event
      name: click_recorded
    measure:
      aggregate: count
    time:
      field: occurred_at
      grain: day
    unit: count

  - name: monthly_revenue
    label: Monthly Revenue
    source:
      kind: record
      name: payment
    measure:
      aggregate: sum
      field: amount
    filters:
      - field: status
        operator: equals
        value: paid
    time:
      field: paid_at
      grain: month
    unit: GBP
    dimensions:
      - product_id
      - region

  - name: gross_margin
    label: Gross Margin
    formula: "(monthly_revenue - monthly_cost) / monthly_revenue"
    depends_on:
      - monthly_revenue
      - monthly_cost
    unit: percentage
```

## IR model

Add canonical IR types for `IrMetric`, `IrMetricSource`, `IrMetricMeasure`, `IrMetricFilter`, `IrMetricTime`, `IrMetricWindow`, and `IrMetricTarget`.

Use `source + measure` for aggregate metrics and `formula + depends_on` for derived metrics.

## Validation rules

### Name validation

- metric names must be unique
- metric names must be stable identifiers
- no duplicate names with records/events/projections if that causes ambiguity

### Source validation

For aggregate metrics:

- source kind must be one of `record`, `event`, `projection`
- source name must exist
- measure field must exist unless aggregate is `count`
- time field must exist if `time` is set
- dimension fields must exist on the source shape where possible

### Aggregate validation

Allowed aggregates:

```text
count
sum
avg
min
max
distinct_count
```

Rules:

- `count` does not require a field
- `sum`, `avg`, `min`, `max`, `distinct_count` require a field
- numeric aggregates should warn/error if field type is obviously non-numeric
- `distinct_count` can work for string/id fields

### Filter validation

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

### Time validation

Allowed grains:

```text
hour
day
week
month
quarter
year
```

### Formula validation

For this PR:

- formula metric must have `depends_on`
- all dependencies must reference known metric names
- detect simple cycles in metric dependencies
- do not execute arbitrary formulas

### Target validation

Targets should support:

```json
{
  "operator": ">=",
  "value": 100000,
  "unit": "GBP"
}
```

Allowed operators:

```text
>
>=
<
<=
==
!=
```

## Preview

Extend the existing `SorlaPreviewSummary` produced by `generate_preview`:

```text
metrics: N
```

If preview cards exist, add a metrics card listing metric names.

## Acceptance criteria

- Generated `sorla.yaml` includes metrics.
- Canonical IR carries metrics.
- Validation rejects invalid metric sources, fields, aggregates, grains, filters and dependency cycles.
- Preview includes metric count and metric card.
- Existing fixtures remain deterministic.
- New fixtures cover aggregate metrics and formula metrics.
