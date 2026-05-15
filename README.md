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
greentic-sorla wizard --answers landlord-tenant-pack.json --pack-out landlord-tenant-sor.gtpack
```

For production composition, treat `gtc wizard --extensions ...` as the canonical
entrypoint. The direct `greentic-sorla wizard` flow remains useful for local
development, schema work, fixtures, and extension iteration.

Provider implementations do not live here. This repo produces provider-agnostic
SoRLa artifacts and handoff-ready metadata that can later be assembled by `gtc`
rather than by local pack or bundle builders.

## Agent Endpoints

SoRLa can describe agent-facing business actions such as
`create_customer_contact`. These are lowered into canonical IR and exported as
handoff metadata for downstream `gtc` assembly into OpenAPI overlays, Arazzo
workflows, MCP tool descriptors, and `llms.txt` documentation.

See `docs/agent-endpoints.md` for the authoring model, safety fields, and
`greentic-sorla`/`gtc` ownership boundary.

See `docs/agent-endpoint-handoff-contract.md` for the downstream `gtc` handoff
contract.

## gtpack Handoff

SoRLa packages can be emitted as deterministic `.gtpack` handoff archives for
future `greentic-sorx` consumption. The first supported pack scenario is the
landlord/tenant system of record.

```bash
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack doctor examples/landlord-tenant/landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack inspect examples/landlord-tenant/landlord-tenant-sor.gtpack
cargo run -p greentic-sorla -- pack schema validation
cargo run -p greentic-sorla -- pack validation-inspect examples/landlord-tenant/landlord-tenant-sor.gtpack
```

See `docs/sorla-gtpack.md` for pack contents, determinism rules, and the
`greentic-sorla` / `greentic-sorx` boundary.

Generated packs carry deterministic SORX validation metadata so
downstream SORX tooling can decide whether a deployed pack version is eligible
for public exposure. See `docs/sorx-gtpack-validation.md` for the validation
manifest contract, and `docs/sorx-deployment-handoff.md` for the SORX-owned
GHCR publish, preview deployment, certification, and alias-promotion model.

## End-To-End Scenarios

The landlord/tenant FoundationDB scenario validates SoRLa authoring, migration,
agent endpoint mapping, and provider event/projection behavior through the
sibling `greentic-sorla-providers` workspace.

```bash
cargo xtask e2e landlord-tenant --provider foundationdb
```

See `docs/landlord-tenant-e2e.md` for details and smoke-mode usage.

## Workspace Layout

- `crates/greentic-sorla-cli`: public CLI entrypoint
- `crates/greentic-sorla-lib`: reusable facade used by the CLI and future
  Designer/tooling integrations
- `crates/greentic-sorla-lang`: authoring-language-facing types
- `crates/greentic-sorla-ir`: canonical IR scaffolding
- `crates/greentic-sorla-pack`: abstract artifact and manifest scaffolding
  using legacy pack-oriented naming
- `crates/greentic-sorla-wizard`: deterministic wizard schema generation
- `docs/architecture.md`: repo responsibilities and boundaries
- `docs/agent-endpoints.md`: agent endpoint authoring and handoff contract
- `docs/agent-endpoint-handoff-contract.md`: downstream `gtc` handoff contract
- `docs/landlord-tenant-e2e.md`: FoundationDB-backed landlord/tenant e2e scenario
- `docs/product-shape.md`: wizard-first product contract
- `docs/sorla-gtpack.md`: deterministic SoRLa `.gtpack` handoff contract
- `docs/library-api.md`: reusable library and CLI boundary
- `docs/sorla-lib.md`: stable facade API for Designer/tooling reuse
- `docs/sorx-deployment-handoff.md`: downstream SORX deployment and public
  exposure handoff expectations
- `docs/wizard.md`: wizard schema and answer-model notes
- `docs/extensions-with-gtc.md`: how SoRLa participates in the `gtc`
  extension flow
- `docs/naming-migration.md`: current naming rules and the compatibility mapping
  from legacy package-manifest names to handoff names

## CLI

The current scaffold keeps internal helper commands hidden and reserves the
public surface for the wizard flow. The installed binary is now a thin wrapper
over `greentic-sorla-lib`, so Designer extensions and tests can reuse the same
authoring, validation, and pack logic without invoking a subprocess. This
standalone CLI is a local authoring and extension-development surface, not a
competing pack/bundle toolchain. It also supports deterministic `.gtpack`
handoff output for SoRLa artifacts.

```bash
cargo run -p greentic-sorla -- wizard --schema
cargo run -p greentic-sorla -- wizard --answers crates/greentic-sorla-cli/examples/answers/create_minimal.json
cargo run -p greentic-sorla -- wizard --answers examples/landlord-tenant/answers.json --pack-out landlord-tenant-sor.gtpack
```

For the intended production path, `gtc` should discover and launch
`greentic-sorla` through its extension mechanism and then own the follow-on
assembly flow.

## CI And Releases

This repo uses:

- `ci/local_check.sh` for local pre-submit verification
- `.github/workflows/ci.yml` for PR/push checks
- `.github/workflows/tag-on-version-bump.yml` to tag version bumps merged to `main`
- `.github/workflows/release-binaries.yml` for versioned GitHub release artifacts plus crates.io publishing from tags
- `tools/i18n.sh` for i18n translation lifecycle checks (status/validate/translate)

### Local checks

```bash
bash ci/local_check.sh
```

### Release process

1. Update crate version in `Cargo.toml`.
2. Merge the version bump to `main`.
3. `tag-on-version-bump.yml` creates the matching `vX.Y.Z` tag.
4. `release-binaries.yml` runs on the tag and verifies:
   - local checks
   - version and tag match
   - packaging and dry-run publish
   - validation-enabled `.gtpack` metadata, including schema emission, `pack doctor`, `pack inspect`, and `pack validation-inspect`
5. The workflow creates a GitHub Release named exactly as the Cargo version and uploads six binary archives:
   - Linux x86_64
   - Linux arm64
   - Windows x86_64
   - Windows arm64
   - macOS 15 Intel
   - macOS 15 arm64
6. After those release artifacts succeed, the workflow publishes the internal crates and CLI to crates.io using `CARGO_REGISTRY_TOKEN`.
7. `cargo binstall greentic-sorla` resolves against those GitHub release archives.

### Notes

- CI requires `license`, `repository`, `description`, and `readme` metadata on publishable crates.
- Packaging uses `cargo package` and `cargo publish --dry-run` as a required validation step.
- `ci/local_check.sh` generates a fixture `.gtpack` and verifies embedded SORX validation, exposure policy, and compatibility metadata with `pack doctor`, `pack inspect`, and `pack validation-inspect`.
- i18n sources and locales live in `i18n/` (`en.json`, `locales.json`, and per-locale JSON files).
- Compatibility notes are planned for `docs/compatibility.md` once milestone PRs move past scaffolding.
