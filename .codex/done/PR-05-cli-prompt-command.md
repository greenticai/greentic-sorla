# PR-05 — Add `greentic-sorla prompt` CLI command

## Goal

Expose the prompt-to-answers engine through the CLI.

The CLI is only a terminal frontend over `greentic-sorla-lib`.

Repo reality check: the installed binary crate delegates to `greentic_sorla_cli::main()`, which delegates to `greentic_sorla_lib::main()`. The `prompt` subcommand should therefore be added to the existing `Cli` / `Commands` implementation in `greentic-sorla-lib`, while keeping `crates/greentic-sorla-cli` as a wrapper.

## Command

```bash
greentic-sorla prompt
```

## Options

```bash
greentic-sorla prompt \
  --answers-out answers.json \
  --llm-provider openai \
  --llm-model <MODEL>
```

Support:

```text
--answers-out <FILE>
--resume <FILE>
--session-out <FILE>
--locale <LOCALE>
--llm-provider <PROVIDER>
--llm-model <MODEL>
--llm-api-key <KEY>
--llm-endpoint <URL>
--llm-capability-id <ID>
```

There must be no `--no-llm`.

## Behavior

Example:

```text
Describe the System of Record you want to create.

> We manage rental properties for landlords and tenants.

I found these likely records:
- Landlord
- Tenant
- Property
- Lease
- Payment
- MaintenanceRequest

Question 1:
Can a lease have more than one tenant?

> yes

Question 2:
Should tenant liability be joint, individual, or both?

> joint liability
```

At completion:

```text
Generated answers.json.

Next possible step:
greentic-sorla wizard --answers answers.json
```

The `prompt` command should not run the wizard or generate `.gtpack`.

Do not add provider-specific defaults in the CLI. Provider/model defaults should come from config, environment, or the host capability resolver once that exists.

## Session persistence

If `--session-out` is provided, write the current `PromptSessionState` after each turn.

If `--resume` is provided, resume from that state.

## Acceptance criteria

- `greentic-sorla prompt` starts an interactive CLI session.
- CLI requires/resolves an LLM provider/capability.
- CLI writes only `answers.json` as domain output.
- CLI does not generate `sorla.yaml` or `.gtpack`.
- CLI can resume a session.
- CLI uses `greentic-sorla-lib` prompt engine, not duplicated logic.
- Root and command help remain localized consistently with the existing custom help rendering.
