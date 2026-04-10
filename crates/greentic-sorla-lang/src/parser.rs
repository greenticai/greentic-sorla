use crate::ast::{FieldAuthority, Package, ParseWarning, ParsedPackage, Record, RecordSource};

pub fn parse_package(input: &str) -> Result<ParsedPackage, String> {
    let mut package: Package = serde_yaml::from_str(input)
        .map_err(|err| format!("failed to parse SoRLa package: {err}"))?;

    let warnings = apply_v0_1_compatibility(&mut package);
    validate_package(&package)?;

    Ok(ParsedPackage { package, warnings })
}

fn apply_v0_1_compatibility(package: &mut Package) -> Vec<ParseWarning> {
    let mut warnings = Vec::new();

    for record in &mut package.records {
        if record.source.is_none() {
            record.source = Some(RecordSource::Native);
            warnings.push(ParseWarning {
                path: format!("records.{}", record.name),
                message: "missing `source`; defaulted to `native` for additive v0.1 compatibility"
                    .to_string(),
            });
        }
    }

    warnings
}

fn validate_package(package: &Package) -> Result<(), String> {
    for record in &package.records {
        validate_record(record)?;
    }

    Ok(())
}

fn validate_record(record: &Record) -> Result<(), String> {
    match record
        .source
        .as_ref()
        .expect("source is normalized before validation")
    {
        RecordSource::Native => Ok(()),
        RecordSource::External => {
            if record.external_ref.is_none() {
                return Err(format!(
                    "record `{}` uses source `external` and must declare `external_ref`",
                    record.name
                ));
            }
            Ok(())
        }
        RecordSource::Hybrid => {
            let external_ref = record.external_ref.as_ref().ok_or_else(|| {
                format!(
                    "record `{}` uses source `hybrid` and must declare `external_ref`",
                    record.name
                )
            })?;

            if !external_ref.authoritative {
                return Err(format!(
                    "record `{}` uses source `hybrid` and must mark `external_ref.authoritative: true`",
                    record.name
                ));
            }

            for field in &record.fields {
                if field.authority.is_none() {
                    return Err(format!(
                        "record `{}` uses source `hybrid`; field `{}` must declare `authority: local|external`",
                        record.name, field.name
                    ));
                }
            }

            let has_local = record
                .fields
                .iter()
                .any(|field| field.authority == Some(FieldAuthority::Local));
            let has_external = record
                .fields
                .iter()
                .any(|field| field.authority == Some(FieldAuthority::External));

            if !has_local || !has_external {
                return Err(format!(
                    "record `{}` uses source `hybrid`; fields must include both `local` and `external` authority",
                    record.name
                ));
            }

            Ok(())
        }
    }
}
