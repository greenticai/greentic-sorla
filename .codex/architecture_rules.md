# Extension-First Architecture Rules

For this repository, future work should follow these rules:

- Do not add or preserve code that directly emits final Greentic packs or
  bundles from `greentic-sorla`.
- Do not add or preserve local bundle-builder or pack-builder orchestration in
  this repo.
- Treat `gtc wizard --extensions ...` as the canonical production entrypoint
  for SoRLa-guided composition.
- Treat `gtc` as the owner of extension registry resolution, launcher handoff,
  setup handoff, and start handoff.
- Treat SoRLa outputs from this repo as extension-friendly artifacts, IR,
  source material, and handoff-ready metadata rather than final runtime
  assembly artifacts.
- When existing code still uses pack-oriented terminology, document it as
  legacy naming rather than as proof that this repo owns final assembly.
