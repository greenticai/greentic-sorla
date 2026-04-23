# Handoff Metadata

PR-06 made the generated wizard output more structured without introducing
concrete provider bindings into `greentic-sorla`.

Those generated files should not be interpreted as final pack or bundle
assembly. `gtc` remains the owner of final assembly. The metadata here is
abstract handoff material that can participate in the shared extension flow.

## Generated Metadata Files

`greentic-sorla wizard --answers <file>` now writes these metadata files under
`.greentic-sorla/generated/`:

- `launcher-handoff.json`
- `package-manifest.json`
- `provider-requirements.json`
- `locale-manifest.json`

`launcher-handoff.json` is the canonical handoff name. `package-manifest.json`
remains as a compatibility alias during migration.

## launcher-handoff.json

The launcher handoff document now captures:

- package identity
- package version
- package kind
- IR version
- locale metadata
- compatibility metadata
- provider requirement declarations
- provider repo and binding mode
- artifact references

The handoff document stays abstract by default. It describes what categories
and contracts are required, but it does not hardcode concrete provider URIs or
claim to be a final runtime assembly document.

## provider-requirements.json

The provider requirements manifest records:

- required capability categories
- optional capability categories
- explicit provider requirement declarations
- provider repo
- abstract binding mode

This gives downstream `gtc`-owned assembly tooling enough information to bind
storage, external reference, and evidence-oriented capabilities without
treating those bindings as part of the SoRLa source package itself.

## locale-manifest.json

The locale manifest records:

- default locale
- fallback locale
- schema version
- reserved core i18n key namespace

The reserved keys are the contract surface that downstream wizard clients can
rely on staying stable while the broader catalog grows.
