# Designer Prompt To SoRLa gtpack E2E

This scenario is the SoRLa-local acceptance harness for the Designer extension
line:

```text
prompt
  -> generate_model_from_prompt
  -> validate_model
  -> generate_gtpack
  -> deterministic pack-entry artifact metadata
```

The fixture lives at:

```text
tests/e2e/fixtures/designer_supplier_contract_risk_prompt.txt
```

Run the local portion with:

```bash
cargo test -p greentic-sorla-designer-extension designer_prompt_to_gtpack
bash scripts/e2e/designer-sorla-gtpack.sh /tmp/sorla-designer-e2e
```

The local test does not require a live LLM, network access, secrets, a Sorx
checkout, or a Designer SDK checkout. It verifies the real SoRLa-owned extension
path and asserts that generated artifact metadata is deterministic and free of
absolute temp paths or credential-like values.

## Cross-Repo Manual Step

When `greentic-sorx` and the Designer SDK are available, use their native host
packaging path to turn returned pack entries into `.gtpack` bytes, then validate:

```bash
cargo run -p greentic-sorx -- artifact validate --artifact-json /tmp/sorla-designer-artifact.json --json
```

Final runtime bundle assembly, provider execution, and Sorx startup validation
remain downstream responsibilities.
