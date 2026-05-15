# Ontology Production Readiness

SoRLa ontology support is production-oriented handoff metadata. This repository
owns authoring validation, canonical IR, deterministic artifacts, static doctor
checks, and inspectable summaries. `gtc`, Sorx, and provider repositories own
runtime assembly, provider resolution, graph traversal, evidence queries,
public exposure, audit output, and promotion decisions.

## Versioned Artifacts

SoRLa emits and validates these schema identifiers:

- `greentic.sorla.ontology.v1`
- `greentic.sorla.ontology.graph.v1`
- `greentic.sorla.retrieval-bindings.v1`
- `greentic.sorx.validation.v1`

Unknown major versions are rejected by parser or doctor checks. Additive
metadata must be explicitly modeled in the authoring AST or generated JSON
schema; unknown authoring fields are rejected.

## Determinism

The production checks cover stable wizard schema output, canonical CBOR,
deterministic JSON, byte-identical `.gtpack` output for the ontology business
fixture, and stable doctor/inspect/validation-inspect summaries. Run:

```bash
bash scripts/e2e/ontology-handoff-smoke.sh
bash ci/local_check.sh
```

## Operability

Use these commands for static CI gates:

```bash
greentic-sorla pack schema ontology
greentic-sorla pack schema retrieval-bindings
greentic-sorla pack doctor ontology-business.gtpack
greentic-sorla pack inspect ontology-business.gtpack
greentic-sorla pack validation-inspect ontology-business.gtpack
```

These commands do not contact providers or execute runtime validation.
