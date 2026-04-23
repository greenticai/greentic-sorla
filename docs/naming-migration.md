# Naming Migration

PR-09 standardizes the repo on `handoff` for extension-integration concepts
while keeping `package` for SoRLa source authoring.

## Canonical Terms

- `package`: the SoRLa source layout authored in `sorla.yaml`
- `handoff`: metadata and artifacts handed to `gtc`

## Migration Strategy

The migration is additive.

- Existing crate names stay stable.
- Existing JSON and CBOR filenames keep working.
- New canonical handoff names are written alongside legacy names where this repo
  owns the file output directly.

## Current Mapping

- `package-manifest.json` -> legacy compatibility alias for `launcher-handoff.json`
- `package-manifest.cbor` -> legacy compatibility alias for `launcher-handoff.cbor`
- `PackageManifest` -> legacy compatibility alias for `HandoffManifest`
- `scaffold_manifest` -> legacy compatibility alias for `scaffold_handoff_manifest`
- `build_artifacts_from_yaml` -> legacy compatibility alias for
  `build_handoff_artifacts_from_yaml`

## Stability Notes

- Wizard answer keys such as `package.name` and `package.version` remain stable.
- `.greentic-sorla/generated/` remains stable.
- Generated block markers in `sorla.yaml` remain stable.
- Published crate names remain stable.
