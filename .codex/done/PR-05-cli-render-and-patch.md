# PR-05 — Add CLI rendering and patch commands using the same Designer model

Repository: `greenticai/greentic-sorla`

## Goal

Downscale the same Designer concept model to CLI.

The CLI should use the exact same `SorlaDesignModel`, `ConceptViewModel` and semantic patch APIs as the Designer extension.

This PR depends on PR-01 through PR-03. Do not invent separate CLI-only parse, view or patch logic.

The current CLI already has wizard/prompt/pack-style flows. Add commands under the existing CLI conventions and avoid duplicating existing `pack` or prompt/session commands.

## Commands

Add commands such as:

```bash
greentic-sorla design view sorla.yaml
greentic-sorla design view sorla.yaml --json
greentic-sorla design patch sorla.yaml --patch patch.json
greentic-sorla design add-field sorla.yaml --record property --name postcode --type string --required
greentic-sorla design validate sorla.yaml
```

Exact names can be adjusted to match current CLI conventions.

If the current binary uses a different command namespace than `greentic-sorla`, follow the existing binary and subcommand naming.

## CLI renderer

Add a text renderer:

```rust
pub fn render_concept_view_cli(view: &ConceptViewModel) -> String;
```

Output example:

```text
Property Management
Status: valid

Records:
  landlord
    landlord_id: string required
    name: string required
    email: email required sensitive

  property
    property_id: string required
    landlord_id: string required -> landlord.landlord_id
    address: string required

Diagnostics:
  OK
```

Keep the CLI renderer text-only; it should not depend on the Designer SDK or emit terminal ANSI formatting from the shared model.

## Patch CLI

`design patch` should:

1. read `sorla.yaml`
2. read patch JSON
3. call `apply_sorla_patch`
4. write updated YAML
5. print concept diff

Example diff output:

```text
Applied patch:
  + Added postcode field to Property

Validation:
  OK
```

## Safety

- Refuse to apply patch if base hash mismatches unless `--force` is explicitly provided.
- If `--dry-run`, print diff and diagnostics but do not write file.
- If invalid, do not write file.

## Acceptance criteria

- CLI can render a concept view from `sorla.yaml`.
- CLI can output concept view JSON.
- CLI can apply a semantic patch.
- CLI prints concept diff.
- CLI and Designer extension use the same library APIs.
- CLI does not duplicate existing pack/prompt commands.
- Tests cover CLI rendering and patch application.
