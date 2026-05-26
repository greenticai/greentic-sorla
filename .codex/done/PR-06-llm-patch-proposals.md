# PR-06 — Add LLM-assisted patch proposal flow

Repository: `greenticai/greentic-sorla`

## Goal

Allow natural-language instructions to propose semantic patches against `sorla.yaml`.

The LLM should not rewrite `sorla.yaml` directly. It should propose `SorlaPatch` operations that are then validated by the patch engine.

This PR depends on PR-03. It should reuse the existing prompt/LLM abstractions under `greentic_sorla_lib::prompt` where possible rather than introducing an unrelated provider layer.

## Architecture

```text
source sorla.yaml
  + user instruction
  + LLM capability
  -> patch proposal
  -> user review
  -> apply_sorla_patch()
  -> updated sorla.yaml
```

## Required API

Add to `greentic-sorla-lib`:

```rust
pub struct ProposePatchInput {
    pub source_yaml: String,
    pub instruction: String,
    pub llm: Option<LlmCapabilityConfig>,
}

pub struct ProposePatchOutput {
    pub patch: SorlaPatch,
    pub explanation: String,
    pub risks: Vec<PatchRisk>,
    pub preview_diff: Option<ConceptDiff>,
}

pub fn propose_patch_from_instruction(
    input: ProposePatchInput,
    llm: &dyn LlmCapability,
) -> Result<ProposePatchOutput, SorlaError>;
```

Reuse existing prompt/LLM capability abstractions if already implemented.

In Designer/plugin mode, provider credentials and model selection should normally be resolved by the host or existing capability plumbing. Do not put secrets or provider credentials into extension tool inputs or outputs.

## Designer extension tool

Expose through the DesignExtension tool interface:

```text
propose_patch_from_instruction
```

Use the real `greentic-extension-sdk-*` WIT tools interface described in PR-04; `greentic-designer-sdk` is not the actual crate name in the current sibling SDK.

Input:

```json
{
  "source_yaml": "...",
  "instruction": "Add supplier approval before maintenance work starts"
}
```

Output:

```json
{
  "patch_proposal": {},
  "explanation": "This adds Supplier, supplier_id on MaintenanceRequest and a Supplier Work Approval.",
  "risks": [],
  "preview_diff": {}
}
```

## CLI

Add:

```bash
greentic-sorla design propose-patch sorla.yaml "Add supplier approval before maintenance work starts" --llm-config <configured-capability> --out patch.json
```

Use placeholder docs for model names or read them from existing configuration. Do not hardcode speculative model names in tests or examples.

## Safety

- No direct YAML generation from LLM.
- LLM output must parse as `SorlaPatch`.
- Patch must pass deterministic validation before application.
- High-risk changes should be marked in `risks`.
- Side-effectful/approval-related changes should be explicit.

## Example

Instruction:

```text
Add maintenance suppliers and approval before work starts.
```

Proposed changes:

```text
+ Supplier record
+ supplier_id field on MaintenanceRequest
+ Supplier Work Approval
```

Patch operations:

```text
add_record supplier
add_field maintenance_request.supplier_id
add_approval supplier_work_approval
```

## Acceptance criteria

- LLM-assisted flow produces semantic patch proposals.
- LLM does not output final YAML.
- Patch proposal can be reviewed before applying.
- Patch proposal validates through the same patch engine.
- CLI and Designer extension expose the proposal flow.
- Tests use fake LLM responses.
