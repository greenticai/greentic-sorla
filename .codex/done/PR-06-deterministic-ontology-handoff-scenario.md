# PR 06 — Deterministic ontology handoff scenario

## Repository

`greenticai/greentic-sorla`

## Objective

Add a deterministic ontology-driven scenario that proves the SoRLa side of the flow works end to end: authoring, wizard answers, canonical IR, ontology artifacts, retrieval binding metadata, validation metadata, and doctor/inspect output.

This PR is scoped to `greentic-sorla` only. Do not add Sorx runtime commands, provider catalog execution, graph traversal execution, evidence query execution, policy runtime decisions, audit runtime output, or cross-repo orchestration.

Follow `.codex/architecture_rules.md`: SoRLa emits extension-friendly source material and handoff metadata. `gtc`, Sorx, and provider repositories own downstream runtime assembly and execution.

## Scenario requirements

Use a generic business-domain fixture. Domain-specific names are allowed in fixture data, but all core contracts must use generic ontology types.

Add a fixture under:

```text
examples/ontology-business/
```

Recommended concepts:

```text
Party
Customer
Supplier
Contract
Asset
Obligation
EvidenceDocument
```

Recommended relationships:

```text
Customer has_contract Contract
Supplier fulfills_obligation Obligation
Contract governs Asset
EvidenceDocument supports Contract
```

Recommended actions or agent endpoints:

```text
CreateCustomer
AttachEvidenceToContract
ListContractsForCustomer
AssessObligationRisk
```

Keep the fixture provider-agnostic. Provider requirements may name abstract categories and capabilities only.

## Flow

The scenario should verify only SoRLa-owned behavior:

1. `greentic-sorla wizard --answers` accepts ontology-enabled answers.
2. Generated SoRLa YAML includes ontology, aliases/entity-linking when present, retrieval bindings when present, and validation metadata inputs.
3. Canonical IR is deterministic and includes the ontology-facing sections added in PRs 01-05.
4. The legacy `--pack-out` compatibility path emits deterministic handoff assets.
5. `pack doctor` validates ontology, retrieval, and validation metadata.
6. `pack inspect` and `pack validation-inspect` expose stable summaries.
7. Repeated generation from the same answers produces byte-identical handoff output.

## Required scripts

Add a repo-local deterministic smoke script:

```bash
scripts/e2e/ontology-handoff-smoke.sh
```

The script should be CI-safe, avoid external services, use temporary output directories, and run only `greentic-sorla` commands plus local shell/JQ checks.

If the repo prefers central CI ownership, this may instead be implemented as an extension to `ci/local_check.sh`, but keep a documented one-command path for the scenario.

## Tests

Add tests for:

- ontology business answers parse and generate
- generated SoRLa YAML contains the ontology sections
- generated handoff artifacts include ontology graph/IR, retrieval bindings, and validation metadata
- doctor succeeds on the generated scenario
- inspect output contains stable ontology and retrieval summaries
- repeated generation is byte-identical for the generated `.gtpack`
- no absolute paths, provider secrets, tenant IDs, or runtime credentials appear in generated artifacts

## Docs

Add:

```text
docs/ontology-handoff-scenario.md
```

The doc should describe the SoRLa-owned handoff scenario and explicitly name downstream Sorx/`gtc` execution as out of scope for this repo.

## Acceptance criteria

```bash
cargo test --all-features
cargo run -p greentic-sorla -- wizard --answers examples/ontology-business/answers.json --pack-out /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack doctor /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack validation-inspect /tmp/ontology-business.gtpack
bash scripts/e2e/ontology-handoff-smoke.sh
bash ci/local_check.sh
```

The same input must produce stable hashes across repeated local runs.
