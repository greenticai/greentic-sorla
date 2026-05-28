# PR-04 — Teach prompt-to-answers authoring to design metrics and KPIs

Repository: `greenticai/greentic-sorla`

## Goal

Extend the interactive prompt-to-answers authoring flow so it can identify, ask about, and generate metric/KPI definitions in `answers.json`.

## Current repo alignment

Prompt authoring lives in `greentic-sorla-lib::prompt` and is used by both the
CLI `prompt` command and `greentic-sorla-designer-extension`. The prompt flow
currently produces wizard-compatible `answers.json`, not direct `sorla.yaml`.
Keep metrics generation in `answers.json` and let PR-02 handle YAML/IR
rendering.

`SorDesignDraft` in `crates/greentic-sorla-lib/src/prompt/draft.rs` does not
currently contain metrics. Add metric/KPI draft fields there if adaptive
questioning needs to carry metric intent before final answers generation.

The repo now also has `propose_patch_from_instruction` for YAML semantic patch
proposals. That is a separate Designer/CLI patch flow and should not replace
prompt-to-answers metrics authoring in this PR.

## Prompt behavior

When the user says things like:

```text
I want to track clicks, revenues, costs and KPIs monthly.
```

the prompt engine should ask targeted follow-up questions:

- Which records/events create clicks?
- Which records/events create revenue?
- Which field represents the monetary amount?
- Which statuses count as recognised revenue?
- Where do costs come from?
- Should metrics be daily, weekly, monthly or all three?
- Which dimensions matter, e.g. product, campaign, customer, region?
- Do you need gross margin, CAC, ROAS, conversion rate, MRR or churn?
- Do any KPIs have targets or thresholds?

## Adaptive questioning examples

If user asks for `conversion rate`, ask what counts as a visitor/session and what counts as a conversion.

If user asks for `revenue`, ask which record/event represents revenue, which field is the amount, currency, paid/settled status, and tax inclusion.

If user asks for `cost`, ask whether costs come from supplier invoices, campaigns, labour, subscriptions or manual entries.

If user asks for `gross margin`, ask which metric is revenue, which metric is cost, and whether result should be ratio or percentage.

## Generated answers

The prompt engine should generate `metrics.items` in `answers.json`.

## Safety

The LLM may propose metrics, but deterministic validation must reject invalid metric definitions.

Do not allow arbitrary provider-specific query strings or executable formulas.

## Acceptance criteria

- Prompt engine can infer basic metrics from business language.
- Prompt engine asks adaptive metric/KPI follow-up questions.
- Prompt engine generates valid `metrics.items` in `answers.json`.
- Fake LLM tests cover clicks, revenue, cost and gross margin.
- Existing non-metric prompt flows still work.
