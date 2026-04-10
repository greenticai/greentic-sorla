# greentic-sorla

`greentic-sorla` is the wizard-first home for the SoRLa language, IR, packaging,
and guided authoring workflow.

The supported product surface is:

```bash
greentic-sorla wizard --schema
greentic-sorla wizard --answers answers.json
```

Provider implementations do not live here. This repo produces provider-agnostic
artifacts and package metadata that can later bind to provider packs from
`greentic-sorla-providers`.

## Workspace Layout

- `crates/greentic-sorla-cli`: public CLI entrypoint
- `crates/greentic-sorla-lang`: authoring-language-facing types
- `crates/greentic-sorla-ir`: canonical IR scaffolding
- `crates/greentic-sorla-pack`: package and manifest scaffolding
- `crates/greentic-sorla-wizard`: deterministic wizard schema generation
- `docs/architecture.md`: repo responsibilities and boundaries
- `docs/product-shape.md`: wizard-first product contract
- `docs/wizard.md`: wizard schema and answer-model notes

## CLI

The current scaffold keeps internal helper commands hidden and reserves the
public surface for the wizard flow.

```bash
cargo run -p greentic-sorla -- wizard --schema
cargo run -p greentic-sorla -- wizard --answers crates/greentic-sorla-cli/examples/answers/create_minimal.json
```

## CI And Releases

This repo uses:

- `ci/local_check.sh` for local pre-submit verification
- `.github/workflows/ci.yml` for PR/push checks
- `.github/workflows/publish.yml` for versioned GitHub release artifacts plus crates.io publishing from tags
- `tools/i18n.sh` for i18n translation lifecycle checks (status/validate/translate)

### Local checks

```bash
bash ci/local_check.sh
```

### Release process

1. Update crate version in `Cargo.toml`.
2. Tag the matching release commit with `vX.Y.Z`.
3. Push the tag and trigger `publish.yml`.
4. The publish workflow verifies:
   - local checks
   - version and tag match
   - packaging and dry-run publish
5. The workflow creates a GitHub Release named exactly as the Cargo version and uploads six binary archives:
   - Linux x86_64
   - Linux arm64
   - Windows x86_64
   - Windows arm64
   - macOS 15 Intel
   - macOS 15 arm64
6. After those release artifacts succeed, the workflow publishes to crates.io using `CARGO_REGISTRY_TOKEN`.
7. `cargo binstall greentic-sorla` resolves against those GitHub release archives.

### Notes

- CI requires `license`, `repository`, `description`, and `readme` metadata on publishable crates.
- Packaging uses `cargo package` and `cargo publish --dry-run` as a required validation step.
- i18n sources and locales live in `i18n/` (`en.json`, `locales.json`, and per-locale JSON files).
- Compatibility notes are planned for `docs/compatibility.md` once milestone PRs move past scaffolding.
