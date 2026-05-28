# Prompt Authoring

`greentic-sorla prompt` is the interactive path for turning a business
description into wizard-compatible `answers.json`.

The prompt flow has a deliberately narrow boundary:

```text
greentic-sorla prompt -> answers.json
greentic-sorla wizard --answers answers.json -> sorla.yaml / optional .gtpack
```

The prompt engine does not generate `sorla.yaml`, `.gtpack`, runtime config,
deployment artifacts, component source, component assets, or
`build-answers.json`. It only produces the answers document that the existing
deterministic wizard/apply pipeline already accepts.

## Architecture

The shared prompt engine lives in `greentic-sorla-lib`. The CLI and Designer
extension are frontends over that library API:

- CLI: terminal input/output and session files.
- Designer/WebChat/Teams hosts: structured messages, questions, design plans,
  session JSON, and artifacts.
- Library: prompt session state, adaptive question selection, deterministic
  answers conversion, schema validation, and existing apply/pack facades.

The existing wizard pipeline remains the deterministic boundary. Prompt output
should be validated with `normalize_answers` / `validate_model` and can then be
applied with `apply_answers` or `greentic-sorla wizard --answers`.

Prompt authoring can include metrics and KPIs in `answers.json` when the user
asks to track clicks, revenue, costs, conversion, gross margin, ROAS, CAC, MRR,
churn, targets, or reporting cadence. The prompt engine asks targeted follow-up
questions about sources, amount fields, recognized statuses, grains, dimensions,
formula inputs, and KPI thresholds, then emits `metrics.items` for the wizard
pipeline to validate and render.

## LLM Capability

Prompt authoring requires an LLM capability. There is no `--no-llm` mode.

This repository defines the provider-agnostic prompt-side contract and uses fake
LLM implementations in tests. Concrete provider implementations and credential
resolution belong outside this repository or in a host capability system.

Provider credentials must not be embedded in generated answers, prompt session
state, Designer output, or logs. Inline CLI API keys are development-only input
and are redacted from debug output.

## CLI Usage

```bash
greentic-sorla prompt \
  --answers-out answers.json \
  --llm-provider openai \
  --llm-model <MODEL>
```

Useful options:

```text
--answers-out <FILE>
--resume <FILE>
--session-out <FILE>
--locale <LOCALE>
--llm-provider <PROVIDER>
--llm-model <MODEL>
--llm-api-key <KEY>
--llm-endpoint <URL>
--llm-capability-id <ID>
```

After prompt completion, continue through the existing deterministic pipeline:

```bash
greentic-sorla wizard --answers answers.json
greentic-sorla wizard --answers answers.json --pack-out handoff.gtpack
```

## SDK Usage

Native callers can use `greentic-sorla-lib` directly:

```rust
let model = greentic_sorla_lib::normalize_answers(
    answers_json,
    greentic_sorla_lib::NormalizeOptions::default(),
)?;
let report = greentic_sorla_lib::validate_model(
    &model,
    greentic_sorla_lib::ValidateOptions::default(),
);
```

For filesystem apply behavior, use `apply_answers`. For native ZIP packaging,
use `build_gtpack_bytes`, `build_gtpack_file`, or `pack_from_answers`.

For WASM-facing Designer flows, use `normalize_answers`, `generate_preview`, and
`build_gtpack_entries`; the host should package returned entries when actual
`.gtpack` bytes are required.

## Designer And Messaging Hosts

The Designer extension exposes prompt-session tools alongside its existing
deterministic tools:

- `start_prompt_session`
- `continue_prompt_session`
- `generate_prompt_answers`

These return structured JSON containing:

- serialized prompt session state
- assistant message text
- structured questions
- optional design plan
- optional `answers_json`
- diagnostics

Hosts such as WebChat, Teams, or Adaptive Cards should render these structures
in channel-native UI and persist the session JSON between turns.

## Tests

Prompt tests should use fake LLM capabilities and deterministic responses.
Regression coverage should verify:

- prompt sessions start, advance, save, and resume
- adaptive follow-up questions respond to previous answers
- generated `answers.json` validates through the library facade
- generated metric/KPI answers validate through the same facade
- generated answers apply through the wizard pipeline
- prompt output does not generate `sorla.yaml`, `.gtpack`, component `src/`,
  component `assets/`, or `build-answers.json` directly
