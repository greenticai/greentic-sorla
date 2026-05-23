# PR-06 — Integrate prompt engine into Sorla Designer component/extension

## Goal

Enable the separately built Sorla Designer component/extension to use `greentic-sorla-lib` at runtime for interactive prompt-to-answers authoring.

The component is built once. The business prompt does not generate component source, assets, or build config.

## Existing crate

The workspace already includes:

```text
crates/greentic-sorla-designer-extension
```

Use this existing boundary where possible.

Repo reality check: this crate already exposes deterministic Designer adapter tools such as `generate_model_from_prompt`, `validate_model`, `generate_gtpack`, `list_designer_node_types`, and `generate_flow_node_from_node_type`. It currently generates deterministic fixture-like answers and uses `greentic-sorla-lib` for normalization, validation, preview, and pack entries. This PR should extend that adapter with prompt-session tools instead of replacing the existing deterministic tools.

## Runtime responsibilities

At runtime the component should call `greentic-sorla-lib` to:

- start a prompt session
- request/use an LLM capability
- ask adaptive follow-up questions
- maintain session state
- render questions in channel-friendly form
- show the evolving design plan
- generate `answers.json`
- optionally call wizard/apply or pack APIs when the host flow asks for it

## UI neutrality

The prompt SDK must not assume a specific UI.

It should return structured messages/questions that can be rendered in:

- CLI
- WebChat GUI
- Microsoft Teams
- Adaptive Cards
- other Greentic messaging channels

## Artifacts

The component may emit `answers.json` as an artifact/output.

If requested by the host flow, the component may then call separate `greentic-sorla-lib` APIs to generate:

```text
sorla.yaml
.gtpack
```

This is not part of the prompt engine output.

In WASM builds, continue using the existing `build_gtpack_entries` pattern rather than trying to produce ZIP bytes inside the extension. Native hosts can use `build_gtpack_bytes` / `build_gtpack_file`.

## Capability declaration

The component should declare/use an LLM capability, preferably the canonical Greentic capability name, or:

```text
greentic.cap.llm
```

It should not hardcode one provider.

Do not embed provider API keys or concrete provider credentials in extension input/output JSON.

## Acceptance criteria

- Sorla Designer component uses `greentic-sorla-lib` prompt engine.
- Component can render at least one prompt session turn as structured output.
- Component can persist/restore prompt session state.
- Component can emit `answers.json`.
- Component does not generate its own source/assets/build config at runtime.
- Component can optionally continue from `answers.json` to `sorla.yaml`/`.gtpack` using separate library APIs.
- Existing deterministic Designer adapter tests continue to pass.
