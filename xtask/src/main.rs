use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage());
    };

    match command {
        "e2e" => run_e2e(&args[1..]),
        "-h" | "--help" => {
            println!("{}", usage());
            Ok(())
        }
        other => Err(format!("unknown xtask command `{other}`\n\n{}", usage())),
    }
}

fn run_e2e(args: &[String]) -> Result<(), String> {
    let Some(scenario) = args.first().map(String::as_str) else {
        return Err(usage());
    };
    if scenario != "landlord-tenant" {
        return Err(format!("unknown e2e scenario `{scenario}`\n\n{}", usage()));
    }

    let mut provider = "foundationdb".to_string();
    let mut smoke = false;
    let mut iter = args[1..].iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => {
                provider = iter
                    .next()
                    .ok_or_else(|| "--provider requires a value".to_string())?
                    .clone();
            }
            "--smoke" => smoke = true,
            other => return Err(format!("unknown e2e argument `{other}`\n\n{}", usage())),
        }
    }

    if provider != "foundationdb" {
        return Err(format!(
            "unsupported provider `{provider}` for landlord-tenant e2e"
        ));
    }

    let mut command = Command::new("cargo");
    command.args([
        "test",
        "--manifest-path",
        "crates/greentic-sorla-e2e/Cargo.toml",
        "landlord_tenant_foundationdb",
        "--",
        "--nocapture",
    ]);
    if smoke {
        command.env("SORLA_E2E_SMOKE", "1");
    }

    let status = command
        .status()
        .map_err(|err| format!("failed to run cargo test for e2e: {err}"))?;
    if !status.success() {
        return Err(format!("landlord-tenant e2e failed with status {status}"));
    }

    Ok(())
}

fn usage() -> String {
    "usage: cargo xtask e2e landlord-tenant --provider foundationdb [--smoke]".into()
}
