# PR-02 — Add presentation-neutral ConceptViewModel for Designer and CLI

Repository: `greenticai/greentic-sorla`

## Goal

Add a presentation-neutral `ConceptViewModel` generated from `SorlaDesignModel`.

The same view model should be usable by:

- Greentic Designer browser UI
- CLI text renderer

Do not treat WebChat, Teams or Adaptive Cards as first-class targets in this PR. The model should remain renderer-neutral enough that those renderers could be added later, but this PR should focus on Designer and CLI.

The repo already has `generate_preview`/`SorlaPreview` and Designer node-type/action-catalog logic. Reuse or adapt those concepts where they overlap instead of introducing a duplicate preview surface.

## Architecture

```text
SorlaDesignModel
  -> generate_concept_view()
  -> ConceptViewModel
  -> renderer-specific output
```

## Required API

Add to `greentic-sorla-lib`:

```rust
pub struct ConceptViewInput {
    pub model: SorlaDesignModel,
    pub mode: ConceptViewMode,
    pub renderer_capabilities: Option<RendererCapabilities>,
}

pub struct ConceptViewOutput {
    pub view: ConceptViewModel,
}

pub fn generate_concept_view(input: ConceptViewInput) -> Result<ConceptViewOutput, SorlaError>;
```

## Modes

```rust
pub enum ConceptViewMode {
    Overview,
    Review,
    Edit,
    Cli,
    Designer,
}
```

Mode can influence detail level and available actions, but should not make the view browser-specific.

## ConceptViewModel schema

Use a stable schema name:

```text
greentic.sorla.concept-view.v1
```

Suggested model:

```rust
pub struct ConceptViewModel {
    pub schema: String,
    pub source: SorlaSourceRef,
    pub title: String,
    pub subtitle: Option<String>,
    pub summary: Option<String>,
    pub status: ConceptViewStatus,
    pub sections: Vec<ConceptSection>,
    pub actions: Vec<ConceptAction>,
    pub artifacts: Vec<ConceptArtifact>,
    pub diagnostics: Vec<SorlaDiagnostic>,
}
```

## Sections

Support section kinds:

```text
overview
entity-grid
graph
timeline
projection-list
metric-board
policy-list
approval-flow
agent-endpoint-list
diagnostics
artifacts
```

## Items

Support item kinds:

```text
record-card
field-row
relationship-edge
event-card
projection-card
metric-card
policy-card
approval-card
agent-endpoint-card
diagnostic-card
artifact-card
question-card
```

## Actions

Actions should be semantic and patch-oriented. They are not UI components; they are templates that a Designer or CLI renderer can turn into buttons, menus or commands.

Example:

```json
{
  "id": "add_field.property",
  "label": "Add field",
  "kind": "patch_template",
  "patch_template": "add_field",
  "target": {
    "record": "property"
  }
}
```

The browser can render this as a button. CLI can render it as an available command.

## Renderer capabilities

Add:

```rust
pub struct RendererCapabilities {
    pub cards: bool,
    pub tables: bool,
    pub graphs: bool,
    pub forms: bool,
    pub charts: bool,
    pub cli_tables: bool,
}
```

If graph rendering is unavailable, relationships should still be representable as lists/tables.

`metric-board` and `metric-card` should be emitted only when PR-01 exposes real metric data. Until then, metrics remain reserved/empty.

## No embedded UI formats

Do not embed:

- HTML
- CSS
- React component names
- terminal ANSI codes
- raw Adaptive Card JSON

Those belong in renderers, not the view model.

## Acceptance criteria

- `greentic-sorla-lib` can generate `ConceptViewModel` from `SorlaDesignModel`.
- Records, relationships, events, policies, approvals, agent endpoints, artifacts and diagnostics can be represented.
- Metrics are represented only if the current language model exposes them.
- View model serializes deterministically.
- Designer can render rich UI from the view model.
- CLI can downscale the same view model to text in later PR.
- No HTML/CSS/terminal formatting is embedded.
