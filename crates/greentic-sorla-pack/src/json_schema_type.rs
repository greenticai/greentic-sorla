//! Map a SoRLa field type onto a JSON Schema property.
//!
//! An agent endpoint's `input_schema` / `output_schema` is read as JSON Schema
//! by everything downstream: an LLM provider validates it as a tool's
//! `parameters` before the call is made, and sorx's manager view reads it back
//! with `type` + `format`. SoRLa's own type names (`uuid`, `email`, `datetime`,
//! `decimal`, …) are not JSON Schema types, and a provider refuses the whole
//! request when one appears as `type` — every turn of a worker that binds the
//! tool fails, not just the call that uses the field.
//!
//! The mapping mirrors what sorx's `manager::view_model::field_type` reads
//! back: `string` + `format` for the string-shaped types, so a SoRLa type
//! survives the round trip.

/// Write `type` (and `format`, `items` where one is implied) for `sorla_type`
/// into `property`.
///
/// An unrecognised type gets no `type` at all rather than a guess: an untyped
/// property accepts any value, while a wrong `type` would make a provider
/// refuse arguments the endpoint would have taken.
pub(crate) fn insert_json_schema_type(
    property: &mut serde_json::Map<String, serde_json::Value>,
    sorla_type: &str,
) {
    let (json_type, format) = match sorla_type.trim().to_ascii_lowercase().as_str() {
        "string" | "enum" | "text" => ("string", None),
        "uuid" => ("string", Some("uuid")),
        "email" => ("string", Some("email")),
        "url" | "uri" => ("string", Some("uri")),
        "date" => ("string", Some("date")),
        "time" => ("string", Some("time")),
        "datetime" | "timestamp" => ("string", Some("date-time")),
        "integer" | "int" | "u32" | "i32" | "u64" | "i64" => ("integer", None),
        "number" | "decimal" | "float" | "double" => ("number", None),
        "boolean" | "bool" => ("boolean", None),
        "array" => ("array", None),
        "object" | "json" => ("object", None),
        _ => return,
    };
    property.insert(
        "type".to_string(),
        serde_json::Value::String(json_type.to_string()),
    );
    if let Some(format) = format {
        property.insert(
            "format".to_string(),
            serde_json::Value::String(format.to_string()),
        );
    }
    if json_type == "array" {
        property.insert("items".to_string(), serde_json::json!({}));
    }
}

#[cfg(test)]
mod tests {
    use super::insert_json_schema_type;

    fn schema_for(sorla_type: &str) -> serde_json::Value {
        let mut property = serde_json::Map::new();
        insert_json_schema_type(&mut property, sorla_type);
        serde_json::Value::Object(property)
    }

    #[test]
    fn string_shaped_types_carry_the_format_sorx_reads_back() {
        assert_eq!(
            schema_for("datetime"),
            serde_json::json!({"type": "string", "format": "date-time"})
        );
        assert_eq!(
            schema_for("timestamp"),
            serde_json::json!({"type": "string", "format": "date-time"})
        );
        assert_eq!(
            schema_for("uuid"),
            serde_json::json!({"type": "string", "format": "uuid"})
        );
        assert_eq!(
            schema_for("email"),
            serde_json::json!({"type": "string", "format": "email"})
        );
        assert_eq!(
            schema_for("url"),
            serde_json::json!({"type": "string", "format": "uri"})
        );
        assert_eq!(
            schema_for("date"),
            serde_json::json!({"type": "string", "format": "date"})
        );
        assert_eq!(
            schema_for("time"),
            serde_json::json!({"type": "string", "format": "time"})
        );
    }

    #[test]
    fn numeric_and_boolean_aliases_become_json_schema_types() {
        assert_eq!(schema_for("decimal"), serde_json::json!({"type": "number"}));
        assert_eq!(schema_for("float"), serde_json::json!({"type": "number"}));
        assert_eq!(schema_for("int"), serde_json::json!({"type": "integer"}));
        assert_eq!(schema_for("u32"), serde_json::json!({"type": "integer"}));
        assert_eq!(schema_for("bool"), serde_json::json!({"type": "boolean"}));
    }

    #[test]
    fn an_array_declares_items_so_providers_accept_it() {
        assert_eq!(
            schema_for("array"),
            serde_json::json!({"type": "array", "items": {}})
        );
    }

    #[test]
    fn an_unknown_type_is_left_untyped_rather_than_guessed() {
        assert_eq!(schema_for("money"), serde_json::json!({}));
    }

    #[test]
    fn every_emitted_type_is_a_json_schema_type() {
        const JSON_SCHEMA_TYPES: [&str; 7] = [
            "string", "number", "integer", "boolean", "object", "array", "null",
        ];
        for sorla_type in [
            "string",
            "decimal",
            "integer",
            "boolean",
            "uuid",
            "email",
            "url",
            "date",
            "time",
            "datetime",
            "enum",
            "array",
            "timestamp",
            "bool",
            "int",
            "number",
            "float",
            "double",
            "u32",
        ] {
            let schema = schema_for(sorla_type);
            let emitted = schema["type"].as_str().unwrap_or_else(|| {
                panic!("SoRLa field type `{sorla_type}` must map to a JSON Schema type")
            });
            assert!(
                JSON_SCHEMA_TYPES.contains(&emitted),
                "`{sorla_type}` mapped to `{emitted}`, which is not a JSON Schema type"
            );
        }
    }
}
