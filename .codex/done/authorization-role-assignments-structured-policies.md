# PR: Extend SoRLa authorization with role assignments and structured policies

## Summary

Build on the authorization model that already exists in SoRLa instead of
re-adding it. Today the language and IR already support:

- top-level `roles[]` with `id`, optional `i18n_key`, optional `label`, optional
  `description`, and `grants`
- record CRUD access rules in `records[].access.{read,create,update,delete}`
  with `roles` and `policies`
- endpoint invocation authorization in `agent_endpoints[].authorization` with
  `roles.any_of`, `roles.all_of`, `policies`, and JSON `conditions`
- named `policies` and `approvals` lists
- endpoint authorization repeated in `assets/sorla/agent-gateway.json`

This PR should add the missing pieces around assignment and structured policy
rules: who receives a role, where it applies, and how named policy references
resolve to executable constraints.

## Motivation

Capabilities decide which systems are connected. SoRLa roles and authorization
decide what a tenant, team, user, service, component, or workflow is allowed to
do inside a system of record. Direct endpoint calls and event-driven flows must
not bypass business authorization.

The current model can name roles and reference them from records/endpoints, but
it does not yet model principal-to-role assignments or structured policy rules
that can be validated and handed off as explicit artifacts.

## Current Model

### Existing roles

```yaml
roles:
  - id: landlord_admin
    label: Landlord admin
    description: Full access to landlord operations
    grants:
      - records.*
      - agent_endpoints.*
  - id: property_manager
    label: Property manager
    description: Manage assigned properties and work orders
  - id: contractor
    label: Contractor
    description: Maintenance contractor role
  - id: tenant_user
    label: Tenant
    description: Occupant/end-user role
```

### Existing record access

```yaml
records:
  - name: maintenance_request
    access:
      read:
        roles:
          - landlord_admin
          - property_manager
          - contractor
      create:
        roles:
          - landlord_admin
          - property_manager
          - tenant_user
      update:
        roles:
          - landlord_admin
          - property_manager
        policies:
          - assigned_property_scope
```

### Existing endpoint authorization

```yaml
agent_endpoints:
  - id: add_maintenance_request
    title: Add maintenance request
    intent: Create an open maintenance request for a unit.
    authorization:
      roles:
        any_of:
          - landlord_admin
          - property_manager
          - tenant_user
      policies:
        - tenant_can_create_own_request
```

## Proposed Additions

### Role assignments

Add first-class role assignment declarations. These should map a role to one or
more principals and an optional scope.

```yaml
role_assignments:
  - role: landlord_admin
    tenant: acme-housing
  - role: property_manager
    tenant: acme-housing
    team: north-region
  - role: contractor
    tenant: acme-housing
    team: contractors
    service: plumbing-dispatch
  - role: tenant_user
    tenant: acme-housing
    user: user_123
```

Supported assignee dimensions:

- `tenant`
- `team`
- `user`
- `service`
- `component`
- `workflow`

### Structured policy declarations

Extend current named `policies` into structured declarations while preserving
backwards compatibility for existing `policies: [{ name: ... }]` files.

```yaml
policies:
  - name: assigned_property_scope
    description: Property managers can act only on assigned buildings.
    allow:
      operations:
        - add_maintenance_request
        - assign_contractor
      events:
        subscribe:
          - maintenance_request_created
        publish:
          - contractor_assigned
      constraints:
        - field: building_id
          operator: equals
          value:
            context: team.building_ids

  - name: tenant_can_create_own_request
    allow:
      operations:
        - add_maintenance_request
      constraints:
        - field: tenant_id
          operator: equals
          value:
            context: user_id
```

Keep policy references in the existing places:

- `records[].access.*.policies`
- `agent_endpoints[].authorization.policies`
- `agent_endpoints[].backing.policies`

## Normalized Pack Artifacts

Current packs already include the full canonical model and endpoint handoff
artifacts. This PR should add normalized sidecar artifacts only for the new
assignment/policy rule material:

- `assets/sorla/role-assignments.json`
- `assets/sorla/policy-rules.json`
- `assets/sorla/policy-rules.schema.json`

Do not assume `assets/sorla/roles.json` exists today. Roles are already present
in `assets/sorla/model.cbor` and exposed through the canonical IR; adding a
separate roles JSON artifact can be considered later if a concrete consumer
needs it.

## Validation Rules

SoRLa validation should ensure:

- all role assignment `role` values refer to declared `roles[].id`
- each assignment has at least one assignee dimension
- assignment dimensions use supported actor keys only
- all structured policy operation references point to known actions or agent
  endpoint ids
- all structured policy event references point to known events
- all referenced record fields exist when field constraints are used
- supported context variables are explicit and validated, initially:
  `context.tenant`, `context.team`, `context.user_id`, `context.service`,
  `context.component`, and `context.workflow`
- wildcard role grants remain valid only on explicitly privileged roles, such
  as admin/operator roles, or emit warnings until enforcement policy is settled

Existing validation for `records[].access` and
`agent_endpoints[].authorization` role/policy references should remain intact.

## MVP Scope

Include now:

- role assignment AST/parser/IR support
- tenant/team/user/service/component/workflow assignment dimensions
- structured policy declarations with operation and event references
- simple equality constraints against supported context variables and record
  fields
- normalized JSON artifacts for role assignments and structured policy rules
- backwards compatibility for existing named-only policies

Defer:

- cross-tenant grants
- complex ABAC expression language
- field-level masking/filtering, except as metadata if it falls out naturally
- standalone `roles.json` unless a downstream runtime needs it immediately

## Acceptance Criteria

- Existing SoRLa files with roles, record access, endpoint authorization, and
  named-only policies continue to parse and package unchanged.
- SoRLa can parse and lower `role_assignments`.
- SoRLa can parse and lower structured policy rules while preserving named-only
  policy compatibility.
- Pack output includes normalized role assignment and structured policy rule
  artifacts.
- Invalid assignment roles, policy operations, policy events, context variables,
  and field constraints fail validation with actionable paths.
- Role-less SoRLa packs continue to work; authorization remains deny-by-default
  at runtime when no explicit allow rule applies.

## Test Plan

- Parser tests for `role_assignments` and structured policy syntax.
- IR lowering tests for assignments and structured policies.
- Packaging tests for generated `role-assignments.json`,
  `policy-rules.json`, and `policy-rules.schema.json`.
- Validation tests for missing role, empty assignment principal, unknown
  operation, unknown event, unknown field, and invalid context variable.
- Backwards compatibility tests for existing role-less and named-policy-only
  SoRLa packs.
