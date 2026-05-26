# Concept View Model

`ConceptViewModel` is a presentation-neutral view over `SorlaDesignModel`.
Designer can render it as cards, graphs, forms, and diagnostics; the CLI can
downscale the same structure to text with `render_concept_view_cli`.

The stable schema is `greentic.sorla.concept-view.v1`. The model contains:

- source hash and source metadata
- title, summary, and status
- sections for overview, records, relationships, events, projections, policies,
  approvals, agent endpoints, diagnostics, and artifacts
- semantic actions such as `add_record` and `add_field.<record>`
- diagnostics from parsing and validation

The view model never contains HTML, CSS, React component names, ANSI terminal
formatting, Adaptive Card JSON, provider credentials, or runtime secrets.
