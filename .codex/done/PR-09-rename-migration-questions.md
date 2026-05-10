# Review Questions For Proposed PR 3

This rename pass is optional, but the current codebase has pack-oriented naming in docs, CLI help, schema text, generated file names, Rust types, and crate names. A rename will have wider fallout than just docs.

## Naming Decision Questions

- What is the single canonical replacement term: `assembly-intent`, `extension-intent`, `handoff-intent`, or something else? The repo should avoid mixing multiple near-synonyms in filenames, Rust types, docs, and CLI text.
- Should the word `package` remain valid for SoRLa source authoring, with only final-assembly terms renamed, or is the intent to remove package-first vocabulary almost everywhere?
- Should `greentic-sorla-pack` be renamed at the crate/package level, or only re-described while keeping the published crate name stable?

## Backward Compatibility Questions

- If `package-manifest.json` is renamed, should update flows continue reading the old file name during a migration window?
- If `package-manifest.cbor` is renamed, do existing tests, fixtures, or downstream tooling need dual-write or dual-read support first?
- If schema/help/i18n keys change, is key stability still required for existing wizard consumers, or can this be a clean break?
- If example answers or generated directories are renamed, do we need explicit migration notes for repositories that already contain `.greentic-sorla/generated/` output?

## Blast Radius Questions

- Are embedded i18n strings in `crates/greentic-sorla-cli/src/embedded_i18n.rs` in scope for the rename? They contain a large amount of package-oriented wording across many locales.
- Should Rust type names such as `PackageManifest`, `package_version`, and `package_name` change now, or only user-facing filenames/docs?
- Should docs such as `docs/packaging.md`, `docs/wizard.md`, `docs/architecture.md`, and `.codex/repo_overview.md` all be updated in the same PR so the repo does not end up with split terminology?
- Should generated block markers or on-disk directories change, or are those intentionally stable even if artifact names change?

## Migration Strategy Questions

- Is the expected migration path "rename in place", "write both old and new names temporarily", or "introduce new names and drop old ones immediately"?
- Should PR 3 include a dedicated migration doc listing old names and new names, including crate names, file names, JSON field names, and CLI/help text changes?
- If crate renames happen, are publish/release implications acceptable for this repo, or should package names stay stable even if module/docs naming changes?
