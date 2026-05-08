# PR-07 — Add Fixtures, Docs, and Local Checks for Agent Endpoints

## Goal

Add end-to-end fixtures and documentation showing how agentic endpoints are authored and exported, while preserving the repository boundary.

## Files to touch

- `docs/agent-endpoints.md`
- `docs/architecture.md`
- `docs/product-shape.md`
- `README.md`
- `crates/greentic-sorla-cli/examples/answers/create_agent_endpoints.json`
- New fixture directory. Prefer the existing pack golden-test convention unless a top-level fixture convention is introduced:
  - `crates/greentic-sorla-pack/tests/golden/customer_contact_agent_endpoints.sorla.yaml`
  - `crates/greentic-sorla-pack/tests/golden/customer_contact_agent_endpoints.inspect.json`
  - `crates/greentic-sorla-pack/tests/golden/customer_contact_agent_endpoints.agent-gateway.json`
- `ci/local_check.sh` only if extra validation commands are added

## Current-code notes

- Existing golden fixtures live under `crates/greentic-sorla-pack/tests/golden/`; use that location for pack-owned parse/lower/export tests unless there is a deliberate move to top-level fixtures.
- `ci/local_check.sh` already runs fmt, clippy, tests, build, docs, package checks, and optional i18n validation. Avoid adding custom fixture commands unless they cover behavior not already exercised by `cargo test --all-features`.
- The public CLI package name is `greentic-sorla`, and the only supported public commands are `wizard --schema` and `wizard --answers`.

## New documentation

Create `docs/agent-endpoints.md` with sections:

1. What agent endpoints are
2. What this repo owns
3. What this repo does not own
4. Example SoRLa authoring YAML
5. Generated IR
6. Handoff artifacts
7. How `gtc` should consume the outputs later
8. Relationship to OpenAPI, Arazzo, MCP, and `llms.txt`
9. Safety model: risk, approval, side effects, sensitive inputs

## Important boundary language

Use wording like:

```md
`greentic-sorla` emits deterministic authoring and handoff metadata for agent endpoints.
It does not serve endpoints, proxy API calls, resolve provider credentials, own OAuth setup, or assemble final packs/bundles.
Those responsibilities remain with `gtc`, provider repos, and runtime components.
```

## README update

Add a short section:

```md
## Agent Endpoints

SoRLa can describe agent-facing business actions such as `create_customer_contact`.
These are lowered into canonical IR and exported as handoff metadata for downstream `gtc` assembly into OpenAPI overlays, Arazzo workflows, MCP tool descriptors, and `llms.txt` documentation.
```

## Fixture example

Add a complete fixture for website lead/contact capture:

```yaml
package:
  name: website-lead-capture
  version: 0.2.0

records:
  - name: Contact
    source: hybrid
    external_ref:
      system: hubspot
      key: email
      authoritative: true
    fields:
      - name: email
        type: string
        authority: external
      - name: contact_persona
        type: string
        authority: external
      - name: problem_to_solve
        type: string
        authority: local
      - name: company_size
        type: string
        authority: local

actions:
  - name: UpsertContact
  - name: SendConfirmation

events:
  - name: ContactCaptured
    record: Contact
    kind: integration
    emits:
      - name: email
        type: string
      - name: contact_persona
        type: string

policies:
  - name: CustomerContactPolicy

approvals:
  - name: ReviewHighRiskContact

agent_endpoints:
  - id: create_customer_contact
    title: Create customer contact
    intent: Capture a customer enquiry and create or update the CRM contact.
    inputs:
      - name: email
        type: string
        required: true
        sensitive: true
      - name: company_name
        type: string
        required: true
      - name: company_size
        type: string
        required: false
      - name: problem_to_solve
        type: string
        required: true
    outputs:
      - name: contact_id
        type: string
    side_effects:
      - crm.contact.upsert
      - event.ContactCaptured
    risk: medium
    approval: policy-driven
    provider_requirements:
      - category: crm
        capabilities:
          - contacts.read
          - contacts.write
    backing:
      actions:
        - UpsertContact
        - SendConfirmation
      events:
        - ContactCaptured
      policies:
        - CustomerContactPolicy
      approvals:
        - ReviewHighRiskContact
```

## Local checks

Update `ci/local_check.sh` only if there is a new validation command. Otherwise avoid changing CI.

If a hidden/internal export command exists later, add a check like:

```bash
cargo run -p greentic-sorla -- __internal inspect-agent-fixture crates/greentic-sorla-pack/tests/golden/customer_contact_agent_endpoints.sorla.yaml
```

Do not expose this as public CLI unless product direction changes.

## Tests

Add tests that:

1. Parse the fixture.
2. Lower it into IR.
3. Generate handoff manifest.
4. Generate exporter artifacts.
5. Assert deterministic output.

## Acceptance criteria

- Docs accurately preserve the `greentic-sorla`/`gtc` ownership boundary.
- Fixture demonstrates a realistic CRM contact endpoint.
- Local checks still pass.
- No new public top-level CLI command is added unless hidden/unstable.
