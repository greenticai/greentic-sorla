# Product Shape

`greentic-sorla` should feel like `gtc`: a guided product with a clear primary
workflow, not a toolbox of unrelated top-level verbs.

Publicly supported UX:

- `greentic-sorla wizard --schema`
- `greentic-sorla wizard --answers <file>`

Internal helper commands are allowed when they unblock development or testing,
but they must stay hidden or clearly unstable so the public surface remains
focused.

The initial crate layout for the MVP is:

- `crates/greentic-sorla-cli`
- `crates/greentic-sorla-lang`
- `crates/greentic-sorla-ir`
- `crates/greentic-sorla-pack`
- `crates/greentic-sorla-wizard`

Compatibility notes will live in a later `docs/compatibility.md` once the
language and wizard flows settle.
