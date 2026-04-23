# greentic-sorla

`greentic-sorla` is the wizard-first home for the SoRLa language, IR, and
extension-facing authoring workflow on top of `gtc`.

`gtc` owns final pack generation, bundle generation, extension launching,
extension handoff, setup handoff, and start handoff. `greentic-sorla` does not
own final runtime assembly. This repo produces SoRLa source outputs, canonical
IR, and abstract metadata that `gtc` can consume through the shared extension
mechanism.

The supported product surface is:

```bash
greentic-sorla wizard --schema
greentic-sorla wizard --answers answers.json
```

For production composition, treat `gtc wizard --extensions ...` as the canonical
entrypoint. The direct `greentic-sorla wizard` flow remains useful for local
development, schema work, fixtures, and extension iteration.

Provider implementations do not live here. This repo produces provider-agnostic
SoRLa artifacts and handoff-ready metadata that can later be assembled by `gtc`
rather than by local pack or bundle builders.

## Workspace Layout

- `crates/greentic-sorla-cli`: public CLI entrypoint
- `crates/greentic-sorla-lang`: authoring-language-facing types
- `crates/greentic-sorla-ir`: canonical IR scaffolding
- `crates/greentic-sorla-pack`: abstract artifact and manifest scaffolding
  using legacy pack-oriented naming
- `crates/greentic-sorla-wizard`: deterministic wizard schema generation
- `docs/architecture.md`: repo responsibilities and boundaries
- `docs/product-shape.md`: wizard-first product contract
- `docs/wizard.md`: wizard schema and answer-model notes
- `docs/extensions-with-gtc.md`: how SoRLa participates in the `gtc`
  extension flow
- `docs/naming-migration.md`: current naming rules and the compatibility mapping
  from legacy package-manifest names to handoff names

## CLI

The current scaffold keeps internal helper commands hidden and reserves the
public surface for the wizard flow. This standalone CLI is a local authoring
and extension-development surface, not a competing pack/bundle toolchain.

```bash
cargo run -p greentic-sorla -- wizard --schema
cargo run -p greentic-sorla -- wizard --answers crates/greentic-sorla-cli/examples/answers/create_minimal.json
```

For the intended production path, `gtc` should discover and launch
`greentic-sorla` through its extension mechanism and then own the follow-on
assembly flow.

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
