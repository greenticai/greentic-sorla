# Designer YAML Source Of Truth

`sorla.yaml` is the editable source for Designer and CLI workflows.
`answers.json` remains the wizard/prompt interchange format, but once a package
has source YAML, design tools should parse, view, patch, and validate that YAML
directly.

```text
sorla.yaml
  -> SorlaDesignModel
  -> ConceptViewModel
  -> Designer or CLI renderer
  -> SorlaPatch
  -> updated sorla.yaml
```

The library facade exposes `parse_sorla_yaml`, `generate_concept_view`, and
`apply_sorla_patch` for this flow. Designers must not rewrite YAML with raw text
edits; patches carry the source hash and are rejected when the user is editing a
stale model.

Pack generation still uses the existing canonical lowering path, so YAML-first
editing does not create a second model contract.
