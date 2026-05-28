# PR-02 — Add LLM capability abstraction for prompt authoring

## Goal

Add a provider-agnostic LLM capability abstraction used by the prompt authoring engine.

The prompt flow requires an LLM capability. There must be no `--no-llm` mode.

## Required behavior

The prompt engine must not be hardcoded to a single provider.

It should support provider implementations such as:

- OpenAI
- Anthropic / Claude
- Ollama
- AWS Bedrock
- future Greentic LLM providers

This PR should introduce the abstraction only. Concrete production providers can be added incrementally or delegated to existing Greentic capability/provider systems.

Repo reality check: this repository has abstract provider requirement metadata, but no existing LLM capability contract or production LLM provider implementation. Provider implementations generally live outside this repo, so keep this crate-side abstraction minimal and provider-agnostic.

## Suggested module

```text
crates/greentic-sorla-lib/src/prompt/llm.rs
```

## Suggested trait

```rust
pub trait LlmCapability {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SorlaError>;
}

pub struct LlmRequest {
    pub provider: String,
    pub model: Option<String>,
    pub system_prompt: String,
    pub messages: Vec<LlmMessage>,
    pub response_format: Option<LlmResponseFormat>,
}

pub struct LlmMessage {
    pub role: LlmRole,
    pub content: String,
}

pub enum LlmRole {
    System,
    User,
    Assistant,
}

pub enum LlmResponseFormat {
    Text,
    Json,
}

pub struct LlmResponse {
    pub content: String,
}
```

If the repo already has a Greentic capability contract elsewhere, use or wrap that instead of inventing a parallel abstraction.

Prefer making this trait object-safe and easy to fake in tests. If future async support is needed, add it later behind the provider integration rather than forcing an async dependency into this first abstraction.

## Capability naming

Use the canonical Greentic capability name if it exists. Otherwise reserve:

```text
greentic.cap.llm
```

No canonical LLM capability name currently appears in this repo. Treat `greentic.cap.llm` as a placeholder contract name and document it as provisional.

The API should allow a capability ID to be passed:

```rust
pub struct LlmCapabilityConfig {
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub capability_id: Option<String>,
}
```

## Security

- Do not log API keys.
- Keep API key optional so it can come from a provider/capability/secrets mechanism.
- Treat inline `api_key` as a development/local convenience, not the preferred production path.
- Ensure `LlmCapabilityConfig` does not print secrets through derived debug output.

## Acceptance criteria

- Prompt authoring has a provider-agnostic LLM trait/config.
- No provider is hardcoded into the prompt engine.
- No `--no-llm` path is introduced.
- Tests can use a fake/mock LLM capability.
- Secrets/API keys are not printed in debug output or logs.
