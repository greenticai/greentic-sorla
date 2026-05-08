# Agent Endpoints

## What Agent Endpoints Are

Agent endpoints are SoRLa declarations for agent-facing business actions, such
as `create_customer_contact`. They describe the intent, inputs, outputs, side
effects, approval behavior, provider requirements, backing model elements, and
example requests for an action that another system may expose to agents later.

They are authoring and handoff metadata. They are not runtime routes.

## What This Repo Owns

`greentic-sorla` owns the authoring language, parser validation, canonical IR,
deterministic handoff manifests, wizard schema, and pack-facing exporter
fragments for agent endpoints.

`greentic-sorla` emits deterministic authoring and handoff metadata for agent
endpoints. It does not serve endpoints, proxy API calls, resolve provider
credentials, own OAuth setup, or assemble final packs/bundles. Those
responsibilities remain with `gtc`, provider repos, and runtime components.

## What This Repo Does Not Own

This repository does not own:

- concrete provider API calls
- OAuth flows or credential resolution
- runtime gateway behavior
- final pack or bundle assembly
- provider-specific OpenAPI operation URLs

Provider repositories implement the concrete integrations. `gtc` decides how to
accept, enrich, package, and expose the deterministic handoff metadata.

## Example SoRLa Authoring YAML

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
      - name: problem_to_solve
        type: string
        authority: local

actions:
  - name: UpsertContact

events:
  - name: ContactCaptured
    record: Contact
    kind: integration

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
    outputs:
      - name: contact_id
        type: string
    side_effects:
      - crm.contact.upsert
      - event.ContactCaptured
    emits:
      event: ContactCaptured
      stream: "website-lead-capture/{email}"
      payload:
        email: "$input.email"
        company_name: "$input.company_name"
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
      events:
        - ContactCaptured
```

The complete golden fixture lives at
`crates/greentic-sorla-pack/tests/golden/customer_contact_agent_endpoints.sorla.yaml`.

## Generated IR

Agent endpoints lower into canonical IR as `agent_endpoints`. Endpoint
collections and nested collections are sorted deterministically by ID or name.
The canonical hash includes the endpoint metadata, so changes to agent endpoint
contracts are visible to downstream consumers.

Mutating endpoints can also declare an executable `emits` plan:

- `event`: the SoRLa event produced by the operation
- `stream`: the stream template downstream execution should append to
- `payload`: structured event payload template

Parser validation checks that `event` names a declared event, that `stream` is
non-empty, and that `$input.<name>` references in the payload point at declared
endpoint inputs.

## Handoff Artifacts

`greentic-sorla-pack` can generate:

- `agent-gateway.json`
- `agent-endpoints.ir.cbor`
- `agent-endpoints.openapi.overlay.yaml`
- `agent-workflows.arazzo.yaml`
- `mcp-tools.json`
- `llms.txt.fragment`

These outputs are handoff artifacts. They describe what downstream tooling can
assemble; they do not contain provider credentials or runtime URLs.

## How `gtc` Should Consume Outputs Later

`gtc` should treat the handoff artifacts as deterministic inputs. It can verify
the schema string, package name/version, canonical IR hash, endpoint IDs, export
visibility, and provider requirement categories before assembling final packs or
runtime gateway configuration.

If both `agent-gateway.json` and canonical IR bytes are supplied, `gtc` should
verify the manifest `ir_hash` against those bytes before trusting the handoff.
The detailed downstream validation contract is documented in
`docs/agent-endpoint-handoff-contract.md`.

## Relationship To OpenAPI, Arazzo, MCP, And `llms.txt`

The OpenAPI export is an overlay fragment that downstream tooling can merge into
provider-specific API descriptions.

The Arazzo export describes one workflow per visible endpoint without inventing
provider URLs.

The MCP export describes tool metadata and JSON input schemas for downstream MCP
tool assembly.

The `llms.txt` fragment summarizes endpoint intent, safety, side effects,
required inputs, and outputs for documentation-oriented consumers.

## Safety Model

Agent endpoint declarations include:

- `risk`: low, medium, or high
- `approval`: none, optional, required, or policy-driven
- `side_effects`: explicit business or integration effects
- `sensitive` inputs
- backing approvals and policies

Parser validation rejects high-risk endpoints without required or policy-driven
approval, rejects invalid backing references, and warns when sensitive inputs do
not reference approval or policy context.
