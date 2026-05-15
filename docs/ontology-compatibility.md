# Ontology Compatibility

SoRLa ontology compatibility is schema-major based:

- known `v1` schema identifiers are accepted
- unknown major versions are rejected with stable parser or doctor errors
- unknown authoring YAML fields are rejected by the AST contract
- generated JSON metadata is constrained to explicitly modeled fields

The current SoRLa-owned schema identifiers are:

- `greentic.sorla.ontology.v1`
- `greentic.sorla.ontology.graph.v1`
- `greentic.sorla.retrieval-bindings.v1`

The embedded SORX validation manifest uses `greentic.sorx.validation.v1`.
SoRLa only emits the manifest and statically validates it; downstream Sorx owns
runtime execution and promotion reports.

Additive changes within `v1` should be represented in the Rust types, JSON
schema output, doctor checks, and docs together. Breaking changes require a
new major schema.
