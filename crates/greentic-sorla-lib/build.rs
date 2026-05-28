use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let i18n_dir = manifest_dir.join("i18n");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("embedded_i18n.rs");

    println!("cargo:rerun-if-changed={}", i18n_dir.display());

    let catalogs = read_catalogs(&i18n_dir).unwrap_or_else(|err| {
        panic!(
            "failed to embed i18n catalogs from {}: {err}",
            i18n_dir.display()
        )
    });

    let mut generated =
        String::from("pub fn locale_json(locale: &str) -> Option<&'static str> {\n");
    generated.push_str("    match locale {\n");
    for (locale, raw_json) in catalogs {
        generated.push_str("        ");
        generated.push_str(&rust_string_literal(&locale));
        generated.push_str(" => Some(");
        generated.push_str(&rust_raw_string_literal(&raw_json));
        generated.push_str("),\n");
    }
    generated.push_str("        _ => None,\n");
    generated.push_str("    }\n");
    generated.push_str("}\n");

    fs::write(out_file, generated).expect("failed to write embedded i18n module");
}

fn read_catalogs(i18n_dir: &Path) -> io::Result<Vec<(String, String)>> {
    let mut catalogs = Vec::new();

    for entry in fs::read_dir(i18n_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());

        let locale = path
            .file_stem()
            .and_then(|value| value.to_str())
            .expect("i18n file name should be valid UTF-8")
            .to_string();
        let raw_json = fs::read_to_string(&path)?;
        catalogs.push((locale, raw_json));
    }

    catalogs.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(catalogs)
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn rust_raw_string_literal(value: &str) -> String {
    for hashes in 0..16 {
        let delimiter = "#".repeat(hashes);
        let terminator = format!("\"{delimiter}");
        if !value.contains(&terminator) {
            return format!("r{delimiter}\"{value}\"{delimiter}");
        }
    }

    rust_string_literal(value)
}
