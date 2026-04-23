# Product Shape

`greentic-sorla` should feel like `gtc`: a guided product with a clear primary
workflow, not a toolbox of unrelated top-level verbs.

Publicly supported UX:

- `greentic-sorla wizard --schema`
- `greentic-sorla wizard --answers <file>`

Production composition should still be described in terms of `gtc`, not in
terms of `greentic-sorla` owning pack or bundle assembly. The intended product
shape is:

- `gtc wizard --extensions ...` for production extension orchestration
- `greentic-sorla wizard ...` for local authoring, schema work, fixtures, and
  extension development

Internal helper commands are allowed when they unblock development or testing,
but they must stay hidden or clearly unstable so the public surface remains
focused.

The initial crate layout for the MVP is:

- `crates/greentic-sorla-cli`
- `crates/greentic-sorla-lang`
- `crates/greentic-sorla-ir`
- `crates/greentic-sorla-pack` (legacy name; currently owns abstract artifact
  scaffolding, not final pack generation)
- `crates/greentic-sorla-wizard`

Compatibility notes will live in a later `docs/compatibility.md` once the
language and wizard flows settle.
