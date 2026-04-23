# Wizard Schema

PR-04 makes `greentic-sorla wizard --schema` a real deterministic contract
rather than a placeholder.

This standalone wizard remains useful for local development, schema work, and
extension iteration. For production composition, `gtc` should be treated as the
owner of extension orchestration and final assembly.

## Supported Flows

The schema now explicitly covers both:

- create
- update

When `greentic-sorla wizard` runs without `--schema` or `--answers`, the CLI
now starts an interactive wizard powered by `greentic-qa-lib` and then reuses
the same answers application pipeline as `--answers`.

Update metadata is part of the schema so clients can understand that:

- partial answers are supported for update flows
- generated content is owned by the wizard
- user-authored content outside generated sections is preserved

## Scope Covered By The Schema

The schema currently includes sections for:

- package bootstrap terminology for the SoRLa source layout
- package update terminology for the SoRLa source layout
- handoff terminology for generated extension metadata
- provider requirements
- external source declarations
- events and projections
- compatibility choices
- output preferences

Some of this naming is still legacy pack-oriented wording. In the target
architecture, these answers are inputs to `gtc` extension handoff rather than a
claim that `greentic-sorla` owns final pack or bundle generation.

## i18n Contract

The schema carries stable i18n key references for:

- wizard title and description
- section title and description
- question label and help
- action labels
- validation messaging

The key namespace is intended to stay stable for the core required keys while
the catalog can expand in later milestones.

The schema now also carries:

- `locale`: the selected locale for this schema emission
- `fallback_locale`: the reserved fallback locale (`en`)

`wizard --answers` uses this locale order:

- explicit `answers.locale`
- `SORLA_LOCALE`
- previous locked locale during update flows
- `en`

## Answer Documents

`--schema` defines the stable question IDs and defaults. `--answers` now uses
those IDs as the deterministic control plane for create and update flows.

- full create documents
- partial update documents
- deterministic regeneration of wizard-owned content

Interactive mode is intentionally just a frontend over that same model:

- `greentic-sorla wizard` asks for the core answers interactively
- collected answers are converted into an `AnswersDocument`
- the normal `apply_answers` path performs validation and file generation

## Generated Ownership

`wizard --answers` currently writes:

- `sorla.yaml`
- `.greentic-sorla/generated/answers.lock.json`
- `.greentic-sorla/generated/package-manifest.json`
- `.greentic-sorla/generated/launcher-handoff.json`
- `.greentic-sorla/generated/provider-requirements.json`
- `.greentic-sorla/generated/locale-manifest.json`
- selected generated artifacts under `.greentic-sorla/generated/`

The package source file uses explicit generated block markers. Updates replace
only the generated block and preserve user-authored content outside it.

These generated files should be understood as abstract source artifacts and
handoff-ready metadata. They are not final Greentic packs or bundles, and they
do not replace `gtc` as the assembly owner.

## Extension-Friendly Metadata

The generated metadata is intentionally abstract and extension-friendly. It
records:

- package identity and version
- IR version
- locale metadata
- compatibility/update metadata
- required and optional provider capability categories
- provider requirement declarations
- provider repo and abstract binding mode
- artifact references

Concrete provider bindings are intentionally not required at this stage.

Concrete runtime assembly remains downstream work for `gtc`.

## Examples

Sample answer documents live in:

- `crates/greentic-sorla-cli/examples/answers/create_minimal.json`
- `crates/greentic-sorla-cli/examples/answers/update_minimal.json`
