# Extensions With gtc

`greentic-sorla` is intended to participate in the `gtc` extension flow, not to
replace it.

## Ownership Boundary

`gtc` owns:

- extension registry resolution
- extension descriptor resolution
- extension launcher mode
- launcher handoff emission
- setup handoff
- start handoff
- final pack generation
- final bundle generation

`greentic-sorla` owns:

- SoRLa language authoring
- wizard schema and answers flow
- canonical IR
- abstract metadata and source artifacts
- extension-friendly handoff material

## Normal gtc Wizard Mode

In normal passthrough mode, `gtc wizard` runs without delegating to an external
extension wizard. `gtc` remains the only owner of the top-level wizard flow and
any follow-on assembly behavior.

## Extension-Launcher Mode

In extension-launcher mode, `gtc` resolves configured extensions and launches an
extension wizard binary. The relevant high-level flow is:

1. `gtc` reads `--extensions` and any configured registry data.
2. `gtc` resolves extension descriptors from the selected registry.
3. `gtc` launches the extension wizard binary.
4. The extension emits answers and handoff-ready metadata.
5. `gtc` normalizes launcher handoff output.
6. `gtc` continues with any setup/start handoff and final assembly steps.

`greentic-sorla` should plug into this flow as the launched extension wizard
binary rather than as an alternative pack or bundle toolchain.

## Registry And Descriptor Resolution

Registry lookup and descriptor resolution belong to `gtc`. `greentic-sorla`
should not grow local registry logic or duplicate descriptor-selection rules
that already belong to `gtc`.

When repo documentation needs to talk about discovery, it should describe
`greentic-sorla` as being resolved by `gtc`, not as resolving extensions itself.

## Launcher Handoff

The extension wizard may emit answers, IR, manifests, and other abstract
metadata, but these outputs should be framed as launcher handoff material.

They are not final runtime assembly artifacts, and they should not be described
as if `greentic-sorla` owns pack or bundle completion.

## Setup And Start Handoff

If setup or start handoff documents are needed, they should be treated as
downstream `gtc` contracts. `greentic-sorla` may participate by producing
extension-facing inputs, but it does not own the handoff stages themselves.

## How greentic-sorla Fits

The expected role of `greentic-sorla` is:

- direct `greentic-sorla wizard ...` for local development and schema work
- `gtc wizard --extensions ...` for the canonical production path
- source artifacts, IR, and abstract metadata generated here
- final assembly delegated to `gtc`

That boundary should guide future docs and implementation work in this repo.
