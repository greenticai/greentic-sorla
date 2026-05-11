# PR-05 — Add Wizard Questions for Agent Endpoints

## Goal

Extend the wizard schema so users can define agentic endpoints through the existing wizard-first product surface.

This PR should keep the supported public UX unchanged:

```bash
greentic-sorla wizard --schema
greentic-sorla wizard --answers answers.json
```

## Files to touch

- `crates/greentic-sorla-cli/src/lib.rs`
- `crates/greentic-sorla-wizard/src/lib.rs`
- `crates/greentic-sorla-wizard/src/schema.rs`
- `i18n/en.json`
- Locale files and `crates/greentic-sorla-cli/src/embedded_i18n.rs` if the repo's i18n workflow requires embedded catalogs to stay in sync
- Existing schema tests

## Current-code notes

- `greentic-sorla-cli::default_schema()` is the concrete source of truth for the public wizard schema. `greentic-sorla-wizard::default_schema()` adapts the CLI schema into its own product descriptor shape.
- `greentic-sorla-cli` already executes answers via `wizard --answers`; answers execution is no longer scaffold-only. This PR must decide whether agent endpoint answers only affect schema metadata or also generate `agent_endpoints:` YAML. If they generate YAML, update `AnswersDocument`, `ResolvedAnswers`, answer validation, `render_package_yaml()`, the interactive QA spec, and create/update tests.
- Existing i18n key style is plural: `wizard.sections.*`, `wizard.questions.*`, `wizard.choices.*`, and `wizard.artifacts.*`. Use that style for new keys.
- `WizardSection` has no section-level `visibility`; only `WizardQuestion` has `visibility`. Keep the section visible and gate dependent questions, or explicitly add section visibility as a separate schema change.
- Existing embedded i18n is generated into `crates/greentic-sorla-cli/src/embedded_i18n.rs`; if `i18n/en.json` changes, keep embedded English and any required locale validation/generation workflow in mind.

## Wizard schema additions

Add a new section:

```text
agent-endpoints
```

Suggested title key:

```text
wizard.sections.agent_endpoints.title
```

Suggested description key:

```text
wizard.sections.agent_endpoints.description
```

## Questions

Add questions that are simple enough for the first iteration:

### Enable agent endpoints

```json
{
  "id": "agent_endpoints.enabled",
  "kind": "boolean",
  "required": true,
  "default_value": "false"
}
```

### Endpoint IDs

```json
{
  "id": "agent_endpoints.ids",
  "kind": "text_list",
  "required": false,
  "visibility": {
    "depends_on": "agent_endpoints.enabled",
    "equals": "true"
  }
}
```

### Default risk

```json
{
  "id": "agent_endpoints.default_risk",
  "kind": "single_select",
  "required": true,
  "default_value": "medium",
  "choices": ["low", "medium", "high"],
  "visibility": {
    "depends_on": "agent_endpoints.enabled",
    "equals": "true"
  }
}
```

### Default approval mode

```json
{
  "id": "agent_endpoints.default_approval",
  "kind": "single_select",
  "required": true,
  "default_value": "policy-driven",
  "choices": ["none", "optional", "required", "policy-driven"],
  "visibility": {
    "depends_on": "agent_endpoints.enabled",
    "equals": "true"
  }
}
```

### Export targets

```json
{
  "id": "agent_endpoints.exports",
  "kind": "multi_select",
  "required": true,
  "default_value": "openapi,arazzo,mcp,llms_txt",
  "choices": ["openapi", "arazzo", "mcp", "llms_txt"],
  "visibility": {
    "depends_on": "agent_endpoints.enabled",
    "equals": "true"
  }
}
```

### Provider category

```json
{
  "id": "agent_endpoints.provider_category",
  "kind": "text",
  "required": false,
  "default_value": "api-gateway",
  "visibility": {
    "depends_on": "agent_endpoints.enabled",
    "equals": "true"
  }
}
```

## i18n keys

Add English keys similar to:

```json
{
  "wizard.sections.agent_endpoints.title": "Agent endpoints",
  "wizard.sections.agent_endpoints.description": "Describe business-safe actions that agents can discover and request through downstream gateway exports.",
  "wizard.questions.agent_endpoints_enabled.label": "Expose agentic endpoints?",
  "wizard.questions.agent_endpoints_enabled.help": "Enable this when agents should discover safe business actions such as creating a contact or requesting approval.",
  "wizard.questions.agent_endpoint_ids.label": "Endpoint identifiers",
  "wizard.questions.agent_endpoint_ids.help": "Use stable IDs such as create_customer_contact or request_partner_followup.",
  "wizard.questions.agent_endpoint_default_risk.label": "Default endpoint risk",
  "wizard.questions.agent_endpoint_default_approval.label": "Default approval behavior",
  "wizard.questions.agent_endpoint_exports.label": "Agent-facing export targets",
  "wizard.questions.agent_endpoint_provider_category.label": "Default provider category"
}
```

## Tests

Add/update tests that confirm:

1. Schema remains deterministic.
2. `agent-endpoints` section appears in create flow.
3. Dependent questions are hidden unless `agent_endpoints.enabled` is true.
4. i18n keys referenced by schema exist in `i18n/en.json` if current tooling validates this.
5. Existing create/update flow tests pass.

## Acceptance criteria

- `greentic-sorla wizard --schema` includes `agent-endpoints`.
- Public CLI shape remains unchanged.
- If agent endpoint answers are accepted by `wizard --answers`, generated `sorla.yaml` includes valid `agent_endpoints:` YAML that parses under PR-02 validation.
- If this PR intentionally limits itself to schema-only fields, document that `wizard --answers` ignores those fields for now and add validation/tests that make that behavior explicit.
