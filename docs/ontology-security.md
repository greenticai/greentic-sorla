# Ontology Security

Ontology and retrieval binding artifacts are handoff metadata only. They must
not contain concrete provider credentials, tokens, tenant IDs, absolute machine
paths, or runtime endpoint secrets.

`greentic-sorla pack doctor` statically checks pack paths, lock coverage,
manifest references, ontology/retrieval asset consistency, and common
credential markers. The ontology business smoke test also scans generated
handoff JSON for credential-like values and output-directory leakage.

Sensitivity metadata is preserved through authoring, canonical IR, ontology
graph JSON, and inspectable pack artifacts. This lets downstream Sorx and
provider systems make runtime policy decisions without SoRLa enforcing those
decisions itself.

Runtime policy enforcement, audit redaction, public route exposure, provider
credential validation, and evidence access checks remain downstream
responsibilities.
