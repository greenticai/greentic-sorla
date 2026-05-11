# PR 06: Add i18n-ready wizard support and gtpack-ready package generation

    **Repository:** `greenticai/greentic-sorla`

    ## Objective

    Complete the first end-to-end slice by making the wizard i18n-ready and ensuring generated SoRLa packages are ready to bind to provider gtpacks published from `greentic-sorla-providers`. 

    ## Why this PR exists

    The user explicitly wants the wizard to be i18n compatible and the provider family to publish gtpacks to GHCR. This PR closes the gap between authoring and provider binding by generating package manifests and metadata suitable for pack-based composition.

    ## Scope

    Implement:
- localized wizard key support and default locale handling
- answer documents carrying locale/schema metadata
- gtpack-ready package manifest generation
- explicit provider requirement declarations
- package inspection docs
The goal is not to publish providers from this repo, but to make the package output cleanly consumable by provider packs.

    ## Deliverables

    - i18n key support in wizard schema and answer documents
- base English locale file(s)
- package manifest format for SoRLa outputs
- provider requirement manifest generation
- docs showing how a SoRLa package will later bind to provider gtpacks
- tests for localized schema emission and package metadata generation

    ## Implementation notes for Codex

    Keep the i18n system simple but structurally correct:
- key-based prompts/help
- locale in answers
- fallback to English
For packaging, declare capabilities and provider categories abstractly. Do not hardcode FoundationDB/SharePoint/RAG choices in the generated package; instead declare what the package needs so a later bind step or runtime selection can satisfy those requirements.

    ## Acceptance criteria

    - Wizard schema and answer documents carry locale metadata
- Package output includes provider requirement metadata
- Generated package structure is documented and deterministic
- Tests confirm package metadata includes capability/provider declarations
- Repo now has a coherent first end-to-end story: author -> wizard -> package artifacts

    ## Non-goals

    - Full GHCR binding workflow
- Provider publishing
- Runtime/provider implementation

    ## Suggested files / areas to touch

    - `crates/greentic-sorla-wizard/src/i18n.rs`
- `crates/greentic-sorla-pack/src/manifest.rs`
- `docs/packaging.md`
- `examples/locales/en.json`
