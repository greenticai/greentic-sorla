# PR-06: Integrate Answers V2 with Wizard and Preserve Backward Compatibility

## Summary

Update `greentic-sorla wizard --answers answers.json` so it can read both legacy answers and `sorla.answers.v2`.

For v2, the wizard should compile semantic answers into an expanded plan, then convert that plan into the existing resolved model/rendering path where possible.

## Motivation

The prompt command should be able to emit a simpler semantic `answers.json` v2 while the wizard continues to generate:

- `sorla.yaml`
- generated artifacts
- optional `.gtpack`

This PR connects the new authoring model to the existing wizard output path.

## Current flow

```text
wizard --answers answers.json
  -> reads answers.json
  -> resolves into ResolvedAnswers
  -> applies defaults/inference
  -> renders YAML
  -> writes artifacts/gtpack
```

## New flow

```text
wizard --answers answers.json
  -> detect version
  -> if legacy: existing flow
  -> if v2 create: compile domain to ExpandedSorlaPlan
  -> if v2 update: read existing YAML, apply operations, compile expanded plan
  -> convert to ResolvedAnswers or renderer input
  -> render YAML/artifacts/gtpack
```

## CLI behavior

### Create mode

```bash
greentic-sorla wizard --answers answers.json --pack-out answers.gtpack
```

Works if `answers.json` is v2 create mode.

### Update mode

For v2 update mode, require an existing SorLA file:

```bash
greentic-sorla wizard --answers update.answers.json --sorla-yaml sorla.yaml --out sorla.updated.yaml
```

If omitted:

```text
Error: answers.json is mode=update, but --sorla-yaml was not provided.
```

## Detection

Add:

```rust
match detect_answers_version(&value) {
    AnswersVersion::Legacy => run_legacy_wizard(...),
    AnswersVersion::V2 => run_v2_wizard(...),
}
```

Detection:

```rust
pub enum AnswersVersion {
    Legacy,
    V2,
}
```

V2 if:

```json
{ "version": "sorla.answers.v2" }
```

## Conversion strategy

Preferred approach:

```text
ExpandedSorlaPlan -> ResolvedAnswers -> existing YAML renderer
```

If `ResolvedAnswers` is too restrictive, add an intermediate renderer input but keep existing renderer behavior unchanged initially.

Add:

```rust
pub fn expanded_plan_to_resolved_answers(plan: ExpandedSorlaPlan) -> Result<ResolvedAnswers, CompileError>
```

## Derived preview

Optionally write a debug artifact:

```bash
--expanded-plan-out expanded.sorla.plan.json
```

This is valuable for debugging because it shows deterministic generated consequences before YAML rendering.

## Diagnostics

If v2 compilation produces warnings, print them before rendering:

```text
Warnings:
- contractor is used as an actor but not a record.
- payment was mentioned but no payment provider is configured.
```

Do not fail on warnings by default.

Fail on errors:

```text
Error:
- relationship quote.maintenance_request references missing record maintenance_request
```

## Backward compatibility

Existing legacy answers should continue to work unchanged.

Existing command examples should remain valid:

```bash
greentic-sorla wizard --answers answers.json --pack-out answers.gtpack
```

If no `version` is present, treat as legacy.

## Tests

Add tests for:

- legacy answers still run through legacy path.
- v2 create answers generate YAML.
- v2 update answers require existing YAML.
- v2 update answers can apply add_record operation.
- compiler warnings are surfaced.
- expanded plan debug output is written when requested.

## Acceptance criteria

- Wizard supports legacy and v2 answers.
- V2 create mode can generate SorLA YAML.
- V2 update mode can patch existing SorLA YAML.
- Existing commands keep working.
- Diagnostics are visible and helpful.
