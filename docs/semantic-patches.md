# Semantic Patches

SoRLa design edits use `SorlaPatch` with schema
`greentic.sorla.patch.v1`. A patch is a semantic operation list, not raw JSON
Patch and not replacement YAML.

Each patch includes:

- source kind and `base_hash`
- optional author and intent
- operations such as `add_record`, `rename_record`, `delete_record`,
  `add_field`, `update_field`, and `remove_field`

`apply_sorla_patch` parses the current YAML, verifies the hash, applies the
operations to the parsed model, renders deterministic YAML, validates it, and
returns a `ConceptDiff` plus a refreshed `ConceptViewModel`.

If `base_hash` does not match the current source hash, the patch is rejected
with `base_hash_mismatch` and the user or host must refresh before retrying.

LLM-assisted edits use `propose_patch_from_instruction`: the LLM returns patch
JSON only. The proposal is parsed, risk-tagged, and previewed through the same
patch engine before any caller writes YAML.

## Change History

When an update writes a changed `sorla.yaml`, Sorla also writes a journal entry
under `.greentic-sorla/history/` using schema
`greentic.sorla.change-history.v1`.

The journal records:

- the source path and update mode
- before and after source hashes
- a `ConceptDiff` with added, updated, and removed concepts
- the before/after YAML snapshots
- a rollback snapshot whose `restore_yaml` can be used to restore the previous
  generated source

Wizard updates journal changes to the generated YAML block. Designer semantic
patches journal the full source YAML because they rewrite the canonical file
directly.
