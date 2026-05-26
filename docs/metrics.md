# SoRLa Metrics

SoRLa metrics describe validated business measures and KPIs next to the
records, events, and projections that feed them. They are first-class authoring
metadata: they lower into canonical IR, appear in previews and concept views,
and are exported in deterministic `.gtpack` handoff assets.

Metrics are separate from records/events/projections because they answer a
different question. Records define durable state, events define immutable
business facts, and projections define read models. Metrics define how a
consumer should measure the business: which source to aggregate, which field to
sum or count, which time grain to use, and which formulas or KPI targets matter.

## Authoring

Wizard answers can declare metrics under `metrics.items`. SoRLa YAML renders the
same section under top-level `metrics:`.

Aggregate metrics require:

- `source.kind`: `record`, `event`, or `projection`
- `source.name`: the named source
- `measure.aggregate`: `count`, `sum`, `avg`/`average`, `min`, `max`, or
  `distinct_count`/`count_distinct`
- `measure.field`: required for every aggregate except `count`

Formula metrics require:

- `formula`: a simple expression over named metrics
- `depends_on`: the exact metric names referenced by the formula

Formula strings are metadata, not executable provider query strings. Validation
checks dependencies and cycles; downstream runtimes remain responsible for safe
evaluation semantics.

## Time, Windows, And Dimensions

Metrics can declare `time.field` and `time.grain`. Supported grains are `hour`,
`day`, `week`, `month`, `quarter`, and `year`.

Rolling windows use:

```yaml
window:
  mode: rolling
  size: 3
  unit: months
```

Window size must be greater than zero. Supported units are `hours`, `days`,
`weeks`, `months`, `quarters`, and `years`.

Dimensions name breakdown fields such as `campaign`, `product`, `customer`, or
`region`. For record-backed aggregate metrics, each dimension must exist on the
source record. Formula metrics can carry dimensions as alignment metadata.

## Filters, Units, And Targets

Filters constrain aggregate inputs with provider-neutral operators:
`equals`, `not_equals`, `in`, `not_in`, `gt`, `gte`, `lt`, `lte`, `exists`, and
`not_exists`.

Units are free-form metadata such as `GBP`, `USD`, `percent`, or `ratio`.

Targets define KPI thresholds:

```yaml
target:
  operator: ">="
  value: 100000
  unit: GBP
```

Supported target operators are `>`, `>=`, `<`, `<=`, `==`, and `!=`.

## Validation

Validation rejects:

- unknown metric sources
- sums or other non-count aggregates without a field
- aggregate fields missing from record sources
- dimension fields missing from record sources
- unsupported aggregates, grains, filter operators, window units, and target
  operators
- zero-size rolling windows
- formula dependencies that do not exist
- formula dependency cycles

## Handoff

When metrics are present, `.gtpack` generation emits:

- `assets/sorla/metrics.json`

The pack manifest declares `greentic.sorla.metrics.v1` under the SoRLa extension
metadata and points to that JSON asset. Pack inspection summarizes metric names
and counts. Pack doctor verifies that `metrics.json` matches `model.cbor`,
package metadata, canonical IR hash, and `pack.lock.cbor`.

Metrics are provider-agnostic handoff metadata. They do not require concrete SQL,
OLAP, event-store, or dashboard providers at authoring time. Providers should
advertise capability categories such as storage, event log, projections, or
analytics outside the metric definition itself.

See `examples/metrics-commerce/` for a deterministic commerce KPI fixture with
clicks, revenue, cost, conversion rate, gross margin, and campaign ROAS.
