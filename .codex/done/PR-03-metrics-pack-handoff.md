# PR-03 — Include metrics in deterministic `.gtpack` handoff metadata

Repository: `greenticai/greentic-sorla`

## Goal

Package metric definitions into deterministic `.gtpack` handoff metadata so
SORX and related tooling can load, validate and expose metric definitions.
Metric execution remains a downstream runtime/provider concern and should not
be implemented in this repo.

## Current repo alignment

`greentic-sorla-pack` already emits deterministic `.gtpack` archives with
SoRLa assets under `assets/sorla/` and generic Greentic stack-pack metadata
under `assets/greentic/`. Add metric metadata as an additive SoRLa handoff
asset, and reference it from existing inspect/doctor/lock/manifest metadata
where those structures already track assets.

The pack builder starts from `build_handoff_artifacts_from_yaml`, so this PR
depends on PR-02 metrics being present in the canonical IR. Do not parse
metrics directly from YAML a second time in the pack crate.

The Designer extension's WASM-safe gtpack tools currently return deterministic
pack-entry plans, not SDK `ArtifactToolOutput` objects with ZIP bytes. If
metrics affect extension output, update the pack-entry metadata and tests, not
the SDK artifact envelope.

## Pack artifacts

Add a deterministic artifact such as:

```text
assets/sorla/metrics.json
```

## Artifact schema

Example:

```json
{
  "schema": "greentic.sorla.metrics.v1",
  "package": {
    "name": "commerce-sor",
    "version": "0.1.0"
  },
  "metrics": [
    {
      "name": "daily_clicks",
      "label": "Daily Clicks",
      "source": {
        "kind": "event",
        "name": "click_recorded"
      },
      "measure": {
        "aggregate": "count"
      },
      "time": {
        "field": "occurred_at",
        "grain": "day"
      },
      "unit": "count"
    }
  ]
}
```

## Determinism

The generated `metrics.json` must be deterministic:

- stable ordering by metric name
- stable JSON formatting
- stable default omission/presence rules
- stable hash included in pack metadata if existing handoff metadata tracks artifact hashes

## Validation manifest

Extend SORX validation metadata so downstream SORX can know:

- whether metrics are present
- metrics schema version
- metric count
- metric artifact path
- metric compatibility requirements

## Provider requirements

If a metric requires provider execution, add provider requirement declarations
as handoff metadata only.

Examples:

```text
metrics.aggregate.count
metrics.aggregate.sum
metrics.aggregate.avg
metrics.dimension.group_by
metrics.time_bucket.day
metrics.time_bucket.month
metrics.formula.basic
metrics.window.rolling
```

## Pack doctor

Extend `pack doctor` to validate:

- metrics artifact exists when metrics are declared
- metrics artifact schema is valid
- referenced runtime metadata is consistent
- required provider capabilities are declared
- artifact hash is stable if applicable

## Inspect

Extend `pack inspect` output to include metric names/count/artifact.

## Acceptance criteria

- `.gtpack` contains deterministic metrics metadata.
- `pack doctor` validates metrics metadata.
- `pack inspect` reports metric names/count/artifact.
- Validation metadata includes metric compatibility/capability requirements.
- SORX can load the artifact in later PRs; execution is not added here.
