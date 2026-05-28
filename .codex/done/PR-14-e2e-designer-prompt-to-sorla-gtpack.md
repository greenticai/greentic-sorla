# PR 14 — E2E overall Designer prompt to Sorla `.gtpack` solution

## Repositories

- `greenticai/greentic-sorla` (primary implementation owner)
- `greenticai/greentic-designer-sdk`
- `greenticai/greentic-sorx` (documented dependency only unless this repo has an installed connector/worktree for it)

## Objective

Add an end-to-end deterministic fixture and test harness for the overall solution built by PR 08 through PR 13:

```text
Designer prompt
  -> Sorla library/facade normalizes prompt output
  -> Sorla DesignExtension generates model
  -> Sorla DesignExtension validates model
  -> Sorla DesignExtension returns preview/diagnostics
  -> Sorla DesignExtension generates Designer artifact JSON
  -> Sorla DesignExtension generates deterministic .gtpack compatibility bytes or deterministic pack entries
  -> Sorx validates artifact (cross-repo/manual when available)
  -> Sorx emits inspect/startup metadata (cross-repo/manual when available)
```

This PR is the final confidence check for the Designer line. It should prove that the public facade, WASM-safe profile, Designer extension tools, prompt templates/knowledge assets, and `.gtpack` artifact output compose into one deterministic flow.

The SoRLa-local portion must be CI-safe and must not require a live LLM, network, credentials, Sorx checkout, or Designer SDK checkout beyond whatever dependency shape was already selected in PR 11. In `greentic-sorla`, implement only the SoRLa-owned side unless the other repositories are explicitly available in the workspace.

## Depends on

- PR 08: reusable CLI/library boundary
- PR 09: stable public facade/API
- PR 10: WASM-friendly facade profile
- PR 11: Sorla Designer extension crate
- PR 12: deterministic prompting and knowledge assets
- PR 13: Designer `.gtpack` artifact output

## Input fixture

Add a deterministic input prompt:

```text
Create a system of record for supplier contract risk management.
It should track parties, suppliers, contracts, obligations, evidence documents,
risk assessments, approvals, and agent endpoints for adding evidence and
assessing contract risk.
```

The extension may use a deterministic rule/template path for this prompt.

The fixture should be stored in this repository so future changes can assert the full generated model, diagnostics, preview, artifact metadata, and pack hash. Prefer a compact fixture path such as:

```text
tests/e2e/fixtures/designer_supplier_contract_risk_prompt.txt
```

## Required output

The SoRLa-local flow must produce deterministic artifacts for:

- Sorla model JSON
- validation report
- preview JSON
- generic Designer artifact JSON
- `.gtpack` bytes, or deterministic pack-entry JSON if PR 10/13 intentionally leave ZIP byte emission native-only for WASM
- artifact SHA-256 when bytes are produced
- diagnostics proving no unsupported host capability was silently used

The optional full cross-repo flow should additionally produce:

- Sorx artifact validation result (cross-repo)
- Sorx inspect result (cross-repo)
- Sorx startup schema result (cross-repo)

Do not check in large generated binary artifacts. Check in stable JSON golden files only when they are small, reviewable, and intentionally part of the contract. Binary `.gtpack` output should be generated into a temp directory during tests.

## Local harness shape

Prefer one SoRLa-local integration test that calls the real extension/library APIs rather than shelling out through the CLI:

```text
crates/greentic-sorla-designer-extension/tests/designer_prompt_to_gtpack.rs
```

The test should assert, in order:

1. The prompt fixture produces a model through `generate_model_from_prompt`.
2. The generated model validates through the PR 09 facade path.
3. `validate_model` returns stable diagnostics and preview JSON.
4. `generate_gtpack` returns Designer artifact JSON using the SDK envelope selected in PR 11.
5. The artifact metadata is deterministic and free of absolute paths, timestamps, usernames, tenant IDs, and credential-like values.
6. If bytes are produced, SHA-256 matches the returned bytes and the bytes pass SoRLa-owned doctor/inspect checks.
7. If WASM cannot produce ZIP bytes, the result returns deterministic pack entries plus a clear diagnostic and the native test path still verifies the generated `.gtpack`.

## Suggested script

For this repo, add a SoRLa-local script if it can run without Sorx checkout or live network access:

```bash
scripts/e2e/designer-sorla-gtpack.sh
```

The script should run only the SoRLa-owned e2e by default and write generated outputs under a caller-provided temp/output directory. If direct cross-repo execution is not practical, add docs with exact manual commands and prerequisites.

## Command outline

```bash
# In greentic-sorla
cargo test -p greentic-sorla-designer-extension designer_prompt_to_gtpack
bash scripts/e2e/designer-sorla-gtpack.sh /tmp/sorla-designer-e2e

# Output artifact JSON to /tmp/sorla-designer-artifact.json

# In greentic-sorx
cargo run -p greentic-sorx -- artifact validate --artifact-json /tmp/sorla-designer-artifact.json --json
```

## Requirements

1. No live LLM.
2. No network.
3. No secrets.
4. Deterministic artifact hash.
5. Stable JSON outputs.
6. Tests document exactly where generated files are written.
7. Cross-repo Sorx validation is documented separately from this repo’s local checks.
8. Do not add Sorx runtime validation, provider catalog execution, or final runtime bundle assembly to `greentic-sorla`.
9. Exercise the real PR 09 facade and PR 11/12/13 extension paths; do not duplicate a fake e2e implementation in the test.
10. Keep any new e2e crate or harness outside normal publish packaging if it depends on optional external repos.
11. The local harness must fail loudly if the extension stops returning a model, validation report, preview, Designer artifact, or pack/pack-entry output.
12. Golden outputs must be normalized for deterministic ordering and platform-independent paths.

## Docs

Add:

```text
docs/e2e/designer-prompt-to-sorla-gtpack.md
```

Document:

- what part is enforced locally in `greentic-sorla`
- what part is manual/cross-repo
- exact prerequisites for Designer SDK and Sorx validation
- output paths
- how to update golden JSON intentionally
- why final runtime bundle assembly remains outside this repo

## Acceptance criteria

The SoRLa-local portion must pass in this repo with:

```bash
cargo test --all-features
cargo test -p greentic-sorla-designer-extension designer_prompt_to_gtpack
bash ci/local_check.sh
```

If the script is added, it must also pass:

```bash
bash scripts/e2e/designer-sorla-gtpack.sh /tmp/sorla-designer-e2e
```

The documented full e2e path should pass on clean checkouts of the relevant repos when those repos are available. If they are not available during implementation, keep the external portion as a clearly marked manual validation path, but still implement the SoRLa-local overall-solution test in this PR.
