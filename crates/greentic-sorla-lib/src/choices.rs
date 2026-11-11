//! The closed sets of values an answers document may use, in one place.
//!
//! Three things need the same lists and used to carry their own copy:
//!
//! * the validator in `lib.rs`, which refuses a value outside the set;
//! * the schema and prompt text the model reads, which until now said only
//!   `"type": "string"` for most of these fields, so the model never learned
//!   the set and made up `sql`, `auto`, `primary`, `is_not_null`;
//! * the normalizer, which recovers a near miss before validation.
//!
//! Add a value here and all three follow. `CHOICE_FIELDS` is what the schema,
//! the prompt block and the normalizer walk; the validator uses the constants
//! directly. A source-scanning test in `prompt::engine` fails if a
//! `validate_choice` call goes back to a literal list.

pub(crate) const SOURCES: &[&str] = &["native", "external", "hybrid"];
pub(crate) const AUTHORITIES: &[&str] = &["local", "external"];
pub(crate) const EVENT_KINDS: &[&str] = &["domain", "integration"];
pub(crate) const PROJECTION_MODES: &[&str] = &["current-state", "audit-trail"];
pub(crate) const MIGRATION_COMPATIBILITY: &[&str] =
    &["additive", "backward-compatible", "breaking"];
pub(crate) const STORAGE_CATEGORIES: &[&str] = &["storage"];
pub(crate) const EXTERNAL_REF_CATEGORIES: &[&str] = &["external-ref"];
pub(crate) const RISK_LEVELS: &[&str] = &["low", "medium", "high"];
pub(crate) const APPROVAL_MODES: &[&str] = &["none", "optional", "required", "policy-driven"];
pub(crate) const AGENT_ENDPOINT_EXPORTS: &[&str] = &["openapi", "arazzo", "mcp", "llms_txt"];
pub(crate) const ONTOLOGY_CONCEPT_KINDS: &[&str] = &["abstract", "entity"];
pub(crate) const CARDINALITY: &[&str] = &["one", "many"];
pub(crate) const METRIC_SOURCE_KINDS: &[&str] = &["record", "event", "projection"];
pub(crate) const METRIC_AGGREGATES: &[&str] = &[
    "count",
    "sum",
    "avg",
    "average",
    "min",
    "max",
    "distinct_count",
    "count_distinct",
];
pub(crate) const FILTER_OPERATORS: &[&str] = &[
    "equals",
    "not_equals",
    "in",
    "not_in",
    "gt",
    "gte",
    "lt",
    "lte",
    "exists",
    "not_exists",
];
pub(crate) const TIME_GRAINS: &[&str] = &["hour", "day", "week", "month", "quarter", "year"];
pub(crate) const WINDOW_MODES: &[&str] = &["rolling"];
pub(crate) const WINDOW_UNITS: &[&str] = &["hours", "days", "weeks", "months", "quarters", "years"];
pub(crate) const TARGET_OPERATORS: &[&str] = &[">", ">=", "<", "<=", "==", "!="];
pub(crate) const POLICY_PERMISSIONS: &[&str] =
    &["inherit", "public-metadata-only", "requires-policy"];
pub(crate) const RULE_DIRECTIONS: &[&str] = &["incoming", "outgoing", "both"];

/// A field of the answers document that only accepts a closed set of strings.
pub(crate) struct ChoiceField {
    /// Dotted path from the document root; `[]` walks every array element.
    pub path: &'static str,
    pub values: &'static [&'static str],
    /// What the normalizer writes for a value that is not in `values` and not a
    /// near spelling of one. `None` means there is no neutral choice, so the
    /// value is left for the validator: guessing would change what the
    /// document means.
    pub fallback: Option<&'static str>,
}

const fn field(
    path: &'static str,
    values: &'static [&'static str],
    fallback: Option<&'static str>,
) -> ChoiceField {
    ChoiceField {
        path,
        values,
        fallback,
    }
}

pub(crate) const CHOICE_FIELDS: &[ChoiceField] = &[
    field("flow", &["create", "update"], None),
    field(
        "providers.storage_category",
        STORAGE_CATEGORIES,
        Some("storage"),
    ),
    field(
        "providers.external_ref_category",
        EXTERNAL_REF_CATEGORIES,
        Some("external-ref"),
    ),
    field("records.default_source", SOURCES, Some("native")),
    field("records.items[].source", SOURCES, Some("native")),
    field(
        "records.items[].fields[].authority",
        AUTHORITIES,
        Some("local"),
    ),
    field("events.items[].kind", EVENT_KINDS, Some("domain")),
    field("projections.mode", PROJECTION_MODES, Some("current-state")),
    field(
        "projections.items[].mode",
        PROJECTION_MODES,
        Some("current-state"),
    ),
    field(
        "migrations.compatibility",
        MIGRATION_COMPATIBILITY,
        Some("additive"),
    ),
    field(
        "migrations.items[].compatibility",
        MIGRATION_COMPATIBILITY,
        Some("additive"),
    ),
    // Risk and approval have their own mapping (see `known_risk` and
    // `known_approval` in `prompt::engine`): the fallback must never weaken a
    // review, so it is not a single constant.
    field("agent_endpoints.default_risk", RISK_LEVELS, None),
    field("agent_endpoints.items[].risk", RISK_LEVELS, None),
    field("agent_endpoints.default_approval", APPROVAL_MODES, None),
    field("agent_endpoints.items[].approval", APPROVAL_MODES, None),
    field("agent_endpoints.exports[]", AGENT_ENDPOINT_EXPORTS, None),
    field("metrics.items[].source.kind", METRIC_SOURCE_KINDS, None),
    field("metrics.items[].measure.aggregate", METRIC_AGGREGATES, None),
    field("metrics.items[].filters[].operator", FILTER_OPERATORS, None),
    field("metrics.items[].time.grain", TIME_GRAINS, None),
    field("metrics.items[].window.mode", WINDOW_MODES, None),
    field("metrics.items[].window.unit", WINDOW_UNITS, None),
    field("metrics.items[].target.operator", TARGET_OPERATORS, None),
];
