# greentic-sorla-cli

`greentic-sorla-cli` provides the `greentic-sorla` binary and keeps the supported
user experience centered on:

- `greentic-sorla wizard --schema`
- `greentic-sorla wizard --answers <file>`

This binary is the local SoRLa wizard and extension-development entrypoint. It
does not own final pack or bundle generation; `gtc` remains the owner of final
assembly and extension handoff.
