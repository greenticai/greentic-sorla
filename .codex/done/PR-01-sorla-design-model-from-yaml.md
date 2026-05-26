# PR-01 — Add YAML-first SorlaDesignModel facade

Repository: `greenticai/greentic-sorla`

## Goal

Make `sorla.yaml` the source of truth for Designer editing by adding a public library API that parses, validates and normalizes SoRLa YAML into a `SorlaDesignModel`.

This model should be used by the CLI and Designer extension. It should not be browser-specific.

This is a facade over the existing language/parser/lowering pipeline, not a second canonical model. The repo already has an answers/prompt path exposed through `NormalizedSorlaModel`; keep that path working and add YAML-first APIs alongside it.

## Architecture

```text
sorla.yaml
  -> greentic-sorla-lang parse/lower
  -> SorlaDesignModel
  -> validation diagnostics
```

## Required library API

Add to `greentic-sorla-lib`:

```rust
pub struct ParseSorlaInput {
    pub source_yaml: String,
    pub source_path: Option<PathBuf>,
}

pub struct ParseSorlaOutput {
    pub model: SorlaDesignModel,
    pub diagnostics: Vec<SorlaDiagnostic>,
}

pub fn parse_sorla_yaml(input: ParseSorlaInput) -> Result<ParseSorlaOutput, SorlaError>;
```

If equivalent internal functions already exist, expose stable wrappers rather than duplicating logic.

Implementation notes:

- Reuse `greentic-sorla-lang::parse_package` and the existing lowering/canonical IR path.
- Do not duplicate validation that already exists in `greentic-sorla-lib`; add conversion/adaptation layers where needed.
- `SorlaDiagnostic` already exists. Extend it only where the new API genuinely needs stable JSON round-tripping, such as adding `Deserialize`.
- Keep existing Designer extension tools such as prompt/session generation, gtpack generation, node types and flow node generation intact.

## Source tracking

Add:

```rust
pub struct SorlaSourceRef {
    pub kind: SorlaSourceKind,
    pub path: Option<String>,
    pub hash: String,
    pub schema_version: Option<String>,
}

pub enum SorlaSourceKind {
    SorlaYaml,
}
```

The hash should be deterministic, probably SHA-256 of canonical source bytes or canonical parsed YAML. Pick one and document it.

This hash is used by semantic patches to avoid applying stale browser/CLI edits to a changed YAML file.

## Design model

Add:

```rust
pub struct SorlaDesignModel {
    pub source: SorlaSourceRef,
    pub package: Option<SorlaPackageView>,
    pub records: Vec<SorlaRecordView>,
    pub relationships: Vec<SorlaRelationshipView>,
    pub events: Vec<SorlaEventView>,
    pub projections: Vec<SorlaProjectionView>,
    pub metrics: Vec<SorlaMetricView>,
    pub policies: Vec<SorlaPolicyView>,
    pub approvals: Vec<SorlaApprovalView>,
    pub agent_endpoints: Vec<SorlaAgentEndpointView>,
    pub provider_requirements: Vec<SorlaProviderRequirementView>,
    pub diagnostics: Vec<SorlaDiagnostic>,
}
```

These types should be stable, serializable, and suitable for Designer/CLI consumption.

Do not put HTML, CSS, terminal formatting or Adaptive Card JSON in these structs.

The struct may expose reserved/empty vectors for future language features, but it must not invent semantics that the current SoRLa language does not parse or lower.

## Fields

Records should include:

- name
- label
- description
- fields
- source location if feasible
- sensitive/PII markers
- references

Fields should include:

- name
- type
- required
- sensitive
- enum values
- references
- description

Relationships should be derived from explicit ontology/relationship sections and from field references where appropriate.

Metrics should be included only if metrics support already exists in the current language/lowering path. If not, keep `metrics` as an empty reserved vector and document that patches/views cannot edit metrics yet.

## Diagnostics

Use existing `SorlaDiagnostic` where possible.

Diagnostics should include:

- severity
- code
- message
- path
- suggestion

## Determinism

Parsing and normalization should be deterministic:

- stable ordering where possible
- stable hashes
- stable diagnostics
- stable serialization

## Acceptance criteria

- `greentic-sorla-lib` exposes `parse_sorla_yaml`.
- `ParseSorlaOutput` includes `SorlaDesignModel`.
- `SorlaDesignModel` includes source hash.
- Existing generated `sorla.yaml` fixtures parse successfully.
- Invalid YAML returns useful diagnostics.
- No Designer-specific UI code is introduced.
