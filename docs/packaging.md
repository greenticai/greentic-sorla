# Packaging Metadata

PR-06 makes the generated wizard output more gtpack-ready without introducing
concrete provider bindings into `greentic-sorla`.

## Generated Metadata Files

`greentic-sorla wizard --answers <file>` now writes these metadata files under
`.greentic-sorla/generated/`:

- `package-manifest.json`
- `provider-requirements.json`
- `locale-manifest.json`

These are generated alongside the existing lock file and selected artifact set.

## package-manifest.json

The package manifest now captures:

- package identity
- package version
- package kind
- IR version
- locale metadata
- compatibility metadata
- provider requirement declarations
- provider repo and binding mode
- artifact references

The manifest stays abstract by default. It describes what categories and
contracts are required, but it does not hardcode concrete provider URIs.

## provider-requirements.json

The provider requirements manifest records:

- required capability categories
- optional capability categories
- explicit provider requirement declarations
- provider repo
- abstract binding mode

This gives future provider-pack tooling enough information to bind storage,
external reference, and evidence-oriented capabilities without treating those
bindings as part of the SoRLa source package itself.

## locale-manifest.json

The locale manifest records:

- default locale
- fallback locale
- schema version
- reserved core i18n key namespace

The reserved keys are the contract surface that downstream wizard clients can
rely on staying stable while the broader catalog grows.
