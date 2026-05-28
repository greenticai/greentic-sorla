# Ontology Handoff Scenario

`examples/ontology-business/answers.json` is a deterministic, provider-agnostic
business-domain fixture for the SoRLa-owned ontology handoff path.

The scenario covers:

- wizard answers for records, ontology, semantic aliases, entity-linking, and
  retrieval bindings
- generated SoRLa YAML with ontology-facing sections
- canonical IR lowering and deterministic `.gtpack` compatibility output
- ontology graph/IR assets and retrieval binding JSON/CBOR assets
- SORX validation metadata for ontology, retrieval, provider capability, and
  security policy checks
- `pack doctor`, `pack inspect`, and `pack validation-inspect` summaries

The fixture uses abstract provider categories such as `storage`,
`evidence-store`, and `agent-router`. It does not contain credentials, tenant
IDs, runtime endpoints, or concrete provider configuration.

## Run It

```bash
cargo run -p greentic-sorla -- wizard \
  --answers examples/ontology-business/answers.json \
  --pack-out /tmp/ontology-business.gtpack

cargo run -p greentic-sorla -- pack doctor /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack inspect /tmp/ontology-business.gtpack
cargo run -p greentic-sorla -- pack validation-inspect /tmp/ontology-business.gtpack
```

For a one-command deterministic smoke run:

```bash
bash scripts/e2e/ontology-handoff-smoke.sh
```

The smoke script writes to temporary directories, generates the pack twice, and
checks that repeated output is byte-identical.

## Boundary

This scenario verifies only `greentic-sorla` responsibilities: source authoring,
validation, canonical IR, deterministic handoff assets, and static pack
inspection. Sorx, `gtc`, and provider repositories own runtime assembly,
provider resolution, graph traversal execution, evidence queries, audit output,
public exposure, and promotion decisions.
