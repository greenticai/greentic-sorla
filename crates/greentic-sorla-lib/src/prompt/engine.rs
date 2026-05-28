use super::{
    DraftAction, DraftApproval, DraftEvent, DraftField, DraftMetric, DraftMetricFilter,
    DraftPolicy, DraftProjection, DraftRecord, LlmCapability, LlmCapabilityConfig, LlmMessage,
    LlmRequest, LlmResponseFormat, LlmRole, PromptAnswer, PromptAnswerKind, PromptAnswerValue,
    PromptAssumption, PromptAssumptionConfidence, PromptPhase, PromptQuestion, PromptQuestionRisk,
    PromptSessionConfig, PromptSessionState, PromptTurnInput, PromptTurnOutput, SorDesignDraft,
};
use crate::{NormalizeOptions, SorlaError, ValidateOptions};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_PROMPT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

pub trait PromptAuthoringEngine {
    fn start_session(&self, config: PromptSessionConfig) -> Result<PromptSessionState, SorlaError>;

    fn next_turn(&self, input: PromptTurnInput) -> Result<PromptTurnOutput, SorlaError>;

    fn generate_answers(
        &self,
        session: PromptSessionState,
    ) -> Result<serde_json::Value, SorlaError>;
}

pub struct DefaultPromptAuthoringEngine<Llm> {
    llm: Llm,
}

#[derive(Debug, Deserialize)]
struct PromptModelOutput {
    assistant_message: String,
    #[serde(default, deserialize_with = "deserialize_prompt_assumptions")]
    assumptions: Vec<PromptAssumption>,
    draft: SorDesignDraft,
    #[serde(default)]
    questions: Vec<PromptQuestion>,
}

fn deserialize_prompt_assumptions<'de, D>(
    deserializer: D,
) -> Result<Vec<PromptAssumption>, D::Error>
where
    D: Deserializer<'de>,
{
    let inputs = Vec::<Value>::deserialize(deserializer)?;
    Ok(inputs
        .into_iter()
        .enumerate()
        .map(|(index, input)| normalize_prompt_assumption(index, input))
        .collect())
}

fn normalize_prompt_assumption(index: usize, input: Value) -> PromptAssumption {
    let fallback_id = format!("llm-assumption-{}", index + 1);
    match input {
        Value::String(text) => PromptAssumption {
            id: fallback_id,
            text,
            confidence: PromptAssumptionConfidence::Medium,
        },
        Value::Object(object) => {
            let text = object
                .get("text")
                .or_else(|| object.get("assumption"))
                .or_else(|| object.get("description"))
                .or_else(|| object.get("summary"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(ToString::to_string)
                .unwrap_or_else(|| {
                    serde_json::to_string(&object).unwrap_or_else(|_| "Assumption".to_string())
                });
            let id = object
                .get("id")
                .or_else(|| object.get("name"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(ToString::to_string)
                .unwrap_or(fallback_id);
            let confidence = object
                .get("confidence")
                .or_else(|| object.get("certainty"))
                .or_else(|| object.get("risk"))
                .and_then(Value::as_str)
                .map(prompt_assumption_confidence)
                .unwrap_or(PromptAssumptionConfidence::Medium);
            PromptAssumption {
                id,
                text,
                confidence,
            }
        }
        other => PromptAssumption {
            id: fallback_id,
            text: other.to_string(),
            confidence: PromptAssumptionConfidence::Medium,
        },
    }
}

fn prompt_assumption_confidence(value: &str) -> PromptAssumptionConfidence {
    match value.trim().to_ascii_lowercase().as_str() {
        "low" => PromptAssumptionConfidence::Low,
        "high" => PromptAssumptionConfidence::High,
        _ => PromptAssumptionConfidence::Medium,
    }
}

impl<Llm> DefaultPromptAuthoringEngine<Llm>
where
    Llm: LlmCapability,
{
    pub fn new(llm: Llm) -> Self {
        Self { llm }
    }
}

impl<Llm> PromptAuthoringEngine for DefaultPromptAuthoringEngine<Llm>
where
    Llm: LlmCapability,
{
    fn start_session(&self, config: PromptSessionConfig) -> Result<PromptSessionState, SorlaError> {
        Ok(PromptSessionState {
            session_id: format!(
                "prompt-session-{}",
                NEXT_PROMPT_SESSION_ID.fetch_add(1, Ordering::Relaxed)
            ),
            phase: PromptPhase::AwaitingBusinessPrompt,
            llm: Some(config.llm),
            business_prompt: None,
            answers_so_far: Vec::new(),
            questions: Vec::new(),
            assumptions: Vec::new(),
            draft_model: None,
        })
    }

    fn next_turn(&self, input: PromptTurnInput) -> Result<PromptTurnOutput, SorlaError> {
        let mut session = input.session;
        match session.phase {
            PromptPhase::AwaitingBusinessPrompt => {
                let business_prompt = input.user_message.trim().to_string();
                if business_prompt.is_empty() {
                    return Err("business prompt must not be empty".to_string());
                }

                let llm_config = session.llm.clone().unwrap_or_else(default_llm_config);
                let allow_fallback = llm_config.provider == "fake";
                let llm_response = self.llm.complete(LlmRequest {
                    provider: llm_config.provider.clone(),
                    model: llm_config.model.clone(),
                    api_key: llm_config.api_key.clone(),
                    endpoint: llm_config.endpoint.clone(),
                    system_prompt: prompt_authoring_system_prompt(&wizard_answers_schema_json()),
                    messages: vec![LlmMessage {
                        role: LlmRole::User,
                        content: business_prompt.clone(),
                    }],
                    response_format: Some(authoring_response_format()),
                })?;
                let model_output = match parse_model_output(&llm_response.content) {
                    Ok(output) => output,
                    Err(_) if allow_fallback => fallback_model_output(&business_prompt),
                    Err(error) => repair_model_output(
                        &self.llm,
                        &llm_config,
                        &business_prompt,
                        &llm_response.content,
                        &error,
                    )?,
                };

                session.business_prompt = Some(business_prompt.clone());
                session.assumptions = if model_output.assumptions.is_empty() {
                    assumptions_for_prompt(&business_prompt)
                } else {
                    model_output.assumptions
                };
                session.draft_model = Some(model_output.draft);
                session.questions = model_output.questions;
                if session.questions.is_empty() {
                    session.phase = PromptPhase::ReviewingDesignPlan;
                } else {
                    session.phase = PromptPhase::AskingQuestions;
                }
                let next_questions = next_questions_for_session(&session);
                if next_questions.is_empty() {
                    apply_planner_output_if_needed(&self.llm, &llm_config, &mut session)?;
                }
                let next_questions = next_questions_for_session(&session);
                let design_plan = if next_questions.is_empty() {
                    session.phase = PromptPhase::ReviewingDesignPlan;
                    session.draft_model.clone()
                } else {
                    session.phase = PromptPhase::AskingQuestions;
                    None
                };

                Ok(PromptTurnOutput {
                    session,
                    assistant_message: model_output.assistant_message,
                    next_questions,
                    design_plan,
                    answers_document: None,
                })
            }
            PromptPhase::AskingQuestions => {
                if should_generate_now(&input.user_message) {
                    session.phase = PromptPhase::ReadyToGenerateAnswers;
                    let answers = self.generate_answers(session.clone())?;
                    session.phase = PromptPhase::Completed;
                    return Ok(PromptTurnOutput {
                        session,
                        assistant_message: "Generated answers.json.".to_string(),
                        next_questions: Vec::new(),
                        design_plan: None,
                        answers_document: Some(answers),
                    });
                }

                let Some(question) = next_questions_for_session(&session).into_iter().next() else {
                    session.phase = PromptPhase::ReviewingDesignPlan;
                    let design_plan = session.draft_model.clone();
                    return Ok(PromptTurnOutput {
                        session,
                        assistant_message: "Review the draft design plan.".to_string(),
                        next_questions: Vec::new(),
                        design_plan,
                        answers_document: None,
                    });
                };

                session.answers_so_far.push(PromptAnswer {
                    question_id: question.id.clone(),
                    value: parse_answer_value(&question.answer_kind, &input.user_message),
                });

                let business_prompt = session.business_prompt.clone().unwrap_or_default();
                if session.draft_model.is_none() {
                    session.draft_model =
                        Some(draft_for_prompt(&business_prompt, &session.answers_so_far));
                }
                let next_questions = next_questions_for_session(&session);
                if next_questions.is_empty() {
                    let llm_config = session.llm.clone().unwrap_or_else(default_llm_config);
                    apply_planner_output_if_needed(&self.llm, &llm_config, &mut session)?;
                    if next_questions_for_session(&session).is_empty() {
                        session.phase = PromptPhase::ReviewingDesignPlan;
                    }
                }
                let next_questions = next_questions_for_session(&session);

                Ok(PromptTurnOutput {
                    assistant_message: if session.phase == PromptPhase::ReviewingDesignPlan {
                        "I have enough to propose a draft design plan.".to_string()
                    } else {
                        "Thanks. I adjusted the draft and have one more question.".to_string()
                    },
                    design_plan: if session.phase == PromptPhase::ReviewingDesignPlan {
                        session.draft_model.clone()
                    } else {
                        None
                    },
                    next_questions,
                    answers_document: None,
                    session,
                })
            }
            PromptPhase::ReviewingDesignPlan | PromptPhase::ReadyToGenerateAnswers => {
                session.phase = PromptPhase::ReadyToGenerateAnswers;
                let answers = self.generate_answers(session.clone())?;
                session.phase = PromptPhase::Completed;
                Ok(PromptTurnOutput {
                    session,
                    assistant_message: "Generated answers.json.".to_string(),
                    next_questions: Vec::new(),
                    design_plan: None,
                    answers_document: Some(answers),
                })
            }
            PromptPhase::Completed => Ok(PromptTurnOutput {
                session,
                assistant_message: "This prompt session is already complete.".to_string(),
                next_questions: Vec::new(),
                design_plan: None,
                answers_document: None,
            }),
        }
    }

    fn generate_answers(
        &self,
        session: PromptSessionState,
    ) -> Result<serde_json::Value, SorlaError> {
        let business_prompt = session.business_prompt.clone().unwrap_or_default();
        let llm_config = session.llm.clone().unwrap_or_else(default_llm_config);
        let draft = session
            .draft_model
            .unwrap_or_else(|| draft_for_prompt(&business_prompt, &[]));
        if llm_config.provider == "fake" {
            let answers = answers_from_draft(&draft);
            validate_answers_document(&answers)?;
            return Ok(answers);
        }

        let generation_response = self.llm.complete(LlmRequest {
            provider: llm_config.provider.clone(),
            model: llm_config.model.clone(),
            api_key: llm_config.api_key.clone(),
            endpoint: llm_config.endpoint.clone(),
            system_prompt: answer_generation_system_prompt(&wizard_answers_schema_json()),
            messages: vec![LlmMessage {
                role: LlmRole::User,
                content: answer_generation_user_prompt(
                    &business_prompt,
                    &session.answers_so_far,
                    &draft,
                ),
            }],
            response_format: Some(answer_response_format()),
        })?;
        let mut answers = match parse_json_value_response(&generation_response.content) {
            Some(answers) => answers,
            None => serde_json::Value::String(generation_response.content),
        };
        let mut validation_error = match validate_answers_document(&answers) {
            Ok(()) => return Ok(answers),
            Err(error) => error,
        };

        if matches!(answers, serde_json::Value::String(_)) {
            validation_error = format!("answer JSON parse failed: {validation_error}");
        }

        for _ in 0..2 {
            let repair_response = self.llm.complete(LlmRequest {
                provider: llm_config.provider.clone(),
                model: llm_config.model.clone(),
                api_key: llm_config.api_key.clone(),
                endpoint: llm_config.endpoint.clone(),
                system_prompt: answer_repair_system_prompt(&wizard_answers_schema_json()),
                messages: vec![LlmMessage {
                    role: LlmRole::User,
                    content: answer_repair_user_prompt(
                        &business_prompt,
                        &validation_error,
                        &answers,
                    ),
                }],
                response_format: Some(answer_response_format()),
            })?;
            answers = parse_json_value_response(&repair_response.content).ok_or_else(|| {
                "prompt LLM returned repair output that was not valid JSON".to_string()
            })?;
            match validate_answers_document(&answers) {
                Ok(()) => return Ok(answers),
                Err(error) => validation_error = error,
            }
        }

        Err(format!(
            "generated answers failed validation after LLM repair: {validation_error}"
        ))
    }
}

fn default_llm_config() -> LlmCapabilityConfig {
    LlmCapabilityConfig {
        provider: "fake".to_string(),
        model: None,
        api_key: None,
        endpoint: None,
        capability_id: None,
    }
}

fn authoring_response_format() -> LlmResponseFormat {
    LlmResponseFormat::JsonSchema {
        name: "greentic_sorla_prompt_authoring".to_string(),
        schema: authoring_output_schema_json(),
        strict: false,
    }
}

fn answer_response_format() -> LlmResponseFormat {
    LlmResponseFormat::JsonSchema {
        name: "greentic_sorla_answers".to_string(),
        schema: answers_response_schema_json(),
        strict: false,
    }
}

fn apply_planner_output_if_needed<Llm>(
    llm: &Llm,
    llm_config: &LlmCapabilityConfig,
    session: &mut PromptSessionState,
) -> Result<(), SorlaError>
where
    Llm: LlmCapability,
{
    if llm_config.provider == "fake" {
        return Ok(());
    }
    let business_prompt = session.business_prompt.clone().unwrap_or_default();
    let response = llm.complete(LlmRequest {
        provider: llm_config.provider.clone(),
        model: llm_config.model.clone(),
        api_key: llm_config.api_key.clone(),
        endpoint: llm_config.endpoint.clone(),
        system_prompt: planner_system_prompt(&wizard_answers_schema_json()),
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: planner_user_prompt(&business_prompt, &session.answers_so_far, session),
        }],
        response_format: Some(authoring_response_format()),
    })?;
    let planner_output = parse_model_output(&response.content).map_err(|error| {
        format!(
            "prompt planner LLM returned JSON that did not match the expected plan schema: {error}"
        )
    })?;
    if !planner_output.assumptions.is_empty() {
        session.assumptions = planner_output.assumptions;
    }
    session.draft_model = Some(planner_output.draft);
    session.questions = planner_output.questions;
    Ok(())
}

fn planner_system_prompt(wizard_schema: &str) -> String {
    format!(
        r#"Objective: convert a customer's natural-language prompt into a high-quality answers.json file that greentic-sorla wizard will use to create a System of Record package.

You are the planning step. Use the customer's original prompt and all follow-up answers to make a detailed implementation plan before answers.json is generated.

A good plan:
- Names the durable business records the System of Record must own, using concise snake_case names.
- Gives each record useful fields with practical scalar types such as uuid, email, url, string, integer, decimal, boolean, date, time, or datetime.
- Adds record-field validation rules when they make the model safer, for example unique identifiers, length limits, numeric bounds, decimal precision/scale, patterns, and temporal before/after bounds.
- Keeps English as the base authoring language. Do not generate a separate sorla.yaml per locale; use stable i18n_key values in the schema and sidecar locale catalogs such as i18n/en.json and i18n/es.json for translated labels.
- Marks sensitive fields, required fields, lifecycle status fields, and external identifiers when the business intent implies them.
- Includes domain actions the system must support, such as join, leave, approve, apply, record, publish, or update.
- Includes events only for meaningful business facts that should be immutable or drive projections.
- Includes projections/read models when the customer asks to show, list, rank, report, or search data.
- Includes metrics/KPIs when the customer asks to track clicks, revenue, costs, conversion, margins, churn, ROAS, CAC, MRR, dashboards, targets, or reporting cadence.
- Includes policies/approvals when ranking, fraud checks, permissions, or business rules matter.
- Avoids generic placeholders like case, item, record, action, event unless the customer prompt is genuinely generic.
- Avoids unrelated domains; do not add landlord, tenant, lease, rent, or maintenance concepts unless the customer asked for them.

Return JSON only using the authoring shape: assistant_message, assumptions, draft, questions. If important scope is still unclear, include targeted questions. Ask only questions whose answers would materially improve the final answers.json. If scope is clear, return an empty questions array and a detailed draft.

The later answers.json must satisfy this wizard --schema:
{wizard_schema}"#
    )
}

fn planner_user_prompt(
    business_prompt: &str,
    answers: &[PromptAnswer],
    session: &PromptSessionState,
) -> String {
    format!(
        "Customer prompt:\n{business_prompt}\n\nFollow-up answers:\n{}\n\nCurrent draft:\n{}",
        serde_json::to_string_pretty(answers).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(&session.draft_model).unwrap_or_else(|_| "null".to_string())
    )
}

fn prompt_authoring_system_prompt(wizard_schema: &str) -> String {
    format!(
        r#"Objective: convert a customer's natural-language prompt into an answers.json file that greentic-sorla wizard will use to create a System of Record package.

You are the discovery step. Extract the likely System of Record scope from the customer's prompt and decide whether any clarifying questions are needed before planning.

A good response:
- Focuses on durable records and business facts, not screens or implementation framework details.
- Proposes domain-specific records, fields, actions, events, projections, policies, and approvals that match the customer prompt.
- Proposes domain-specific metrics/KPIs when the prompt mentions tracking, reporting, dashboards, revenue, cost, conversion, margin, churn, ROAS, CAC, MRR, targets, or thresholds.
- Uses concise snake_case names that can become generated artifact names.
- Treats English as the canonical authoring language and localization as sidecar catalogs keyed by i18n_key, not as translated sorla.yaml variants.
- Asks only high-value clarifying questions when the scope is ambiguous or a risky business rule is missing.
- Does not invent unrelated domain concepts.

Return JSON only with this exact shape:
{{
  "assistant_message": "short user-facing message",
  "assumptions": [{{"id":"kebab-case-id","text":"assumption","confidence":"low|medium|high"}}],
  "draft": {{
    "summary": "short summary",
    "records": [{{"name":"snake_case","description":"...", "fields":[{{"name":"snake_case","type_name":"uuid|email|url|string|integer|decimal|boolean|date|time|datetime","required":true,"sensitive":false,"description":"...","rules":{{"unique":true}}}}]}}],
    "relationships": [],
    "actions": [{{"name":"snake_case","description":"...", "risk":"low|medium|high"}}],
    "events": [{{"name":"snake_case","description":"..."}}],
    "projections": [{{"name":"snake_case","description":"..."}}],
    "metrics": [{{"name":"snake_case","label":"Display label","description":"...","source_record":"record_name","aggregate":"count|sum|average|min|max|count_distinct","field":"field_name_or_null","time_field":"created_at","grain":"day|week|month|quarter|year","unit":"GBP|USD|percent|null","dimensions":["product","campaign"],"formula":null,"depends_on":[],"filters":[{{"field":"status","operator":"equals","value":"paid"}}]}}],
    "policies": [],
    "approvals": [],
    "migrations": [],
    "provider_requirements": []
  }},
  "questions": [{{"id":"domain.question","text":"question relevant to the user's domain","help":null,"answer_kind":{{"kind":"boolean"}},"required":true,"risk":"low","depends_on":[]}}]
}}

The final answers.json produced from this draft must satisfy this Greentic SoRLa wizard --schema:
{wizard_schema}

Ask only questions that are directly relevant to the user's prompt. Do not ask landlord, tenant, lease, rent, or maintenance questions unless the prompt is actually about those concepts.
For metrics/KPIs, ask targeted questions about the source record/event, amount field, recognized statuses, cadence, dimensions, formula inputs, and targets. Do not propose executable formulas or provider-specific query strings.
Prefer a small number of high-value follow-up questions. Use empty questions if the prompt is already sufficient."#
    )
}

fn parse_model_output(content: &str) -> Result<PromptModelOutput, String> {
    serde_json::from_str(content)
        .map_err(|err| format!("authoring JSON did not match expected schema: {err}"))
}

fn repair_model_output<Llm>(
    llm: &Llm,
    llm_config: &LlmCapabilityConfig,
    business_prompt: &str,
    invalid_content: &str,
    parse_error: &str,
) -> Result<PromptModelOutput, SorlaError>
where
    Llm: LlmCapability,
{
    let response = llm.complete(LlmRequest {
        provider: llm_config.provider.clone(),
        model: llm_config.model.clone(),
        api_key: llm_config.api_key.clone(),
        endpoint: llm_config.endpoint.clone(),
        system_prompt: prompt_authoring_repair_system_prompt(&wizard_answers_schema_json()),
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: prompt_authoring_repair_user_prompt(
                business_prompt,
                invalid_content,
                parse_error,
            ),
        }],
        response_format: Some(authoring_response_format()),
    })?;
    parse_model_output(&response.content).map_err(|repair_error| {
        format!(
            "prompt LLM returned JSON that did not match the expected authoring schema; repair also failed: {repair_error}"
        )
    })
}

fn prompt_authoring_repair_system_prompt(wizard_schema: &str) -> String {
    format!(
        "Objective: repair prompt-authoring JSON so it can still be used to generate answers.json for greentic-sorla wizard. Return JSON only using the exact authoring shape: assistant_message, assumptions, draft, questions. Preserve the customer's business intent and improve domain specificity where possible. The draft must be suitable for producing answers.json that satisfies this wizard --schema:\n{wizard_schema}"
    )
}

fn prompt_authoring_repair_user_prompt(
    business_prompt: &str,
    invalid_content: &str,
    parse_error: &str,
) -> String {
    format!(
        "Business prompt:\n{business_prompt}\n\nParse/schema error:\n{parse_error}\n\nInvalid authoring JSON/content:\n{invalid_content}"
    )
}

fn parse_json_value_response(content: &str) -> Option<serde_json::Value> {
    serde_json::from_str(content).ok()
}

fn validate_answers_document(answers: &serde_json::Value) -> Result<(), String> {
    let model = crate::normalize_answers(answers.clone(), NormalizeOptions)?;
    let report = crate::validate_model(&model, ValidateOptions);
    let messages = report
        .diagnostics
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.severity == crate::DiagnosticSeverity::Error
                || diagnostic.code.starts_with("sorla.agent_endpoint.")
        })
        .map(|diagnostic| {
            let path = diagnostic.path.unwrap_or_default();
            if path.is_empty() {
                diagnostic.message
            } else {
                format!("{path}: {}", diagnostic.message)
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    if !messages.is_empty() {
        return Err(messages);
    }
    Ok(())
}

fn answer_generation_system_prompt(wizard_schema: &str) -> String {
    format!(
        r#"Objective: generate the final answers.json that greentic-sorla wizard will consume to create a System of Record package.

Use the completed plan/draft and follow-up answers. Return JSON only.

A high-quality answers.json:
- Satisfies the wizard --schema exactly.
- Preserves the customer intent and avoids unrelated example domains.
- Defines a stable package name and version.
- Sets output_dir to the default prompt-generated output path unless the caller overwrites it.
- Generates the latest SoRLa shape: English base names and descriptions in answers/sorla.yaml, stable i18n_key metadata for package, roles, records, fields, events, projections, metrics, ontology concepts/relationships, and agent endpoints, with translations kept in i18n/en.json, i18n/es.json, and later greentic-i18n catalogs instead of separate localized sorla.yaml files.
- Defines roles when different user groups can perform different work. Put role ids in top-level roles, record CRUD permissions in records.items[].access, and endpoint requirements in agent_endpoints.items[].authorization. Do not hide role requirements inside execution.authorization.
- Includes records for each durable business entity, with useful required fields and sensitive markers where appropriate.
- Uses semantic scalar record field types: uuid for stable identifiers, email for email addresses, url for links, datetime for timestamps, date or time for date-only or time-only values, and string only for unconstrained text. Prefer datetime over the legacy timestamp alias.
- Adds record-field rules when useful: unique for primary identifiers, min/max for numeric bounds, min_length/max_length/pattern for text constraints, precision/scale for decimal money or measures, and before/after for date, time, or datetime bounds.
- Keeps rules on record fields only; do not put rules under agent_endpoints.items[].inputs or outputs.
- Sets records.external_ref_system when records.default_source is external or hybrid; use a concise generic value such as external-system when the exact upstream system is not known.
- For hybrid records, marks each field with authority local or external, and includes at least one local and one external field.
- Uses events for immutable business facts and lifecycle moments, not every ordinary field update.
- Uses projections/read models when the customer needs to show lists, rankings, dashboards, or searchable views; each projection should name a source_event that exists in events.items.
- Uses metrics.items for KPIs and reporting measures. Define safe aggregate metrics over records/events and formula metrics only as simple arithmetic over named metrics with depends_on.
- Uses actions for business operations users or agents should request.
- For agent-exposed business operations with non-trivial side effects, include explicit operational_indexes for uniqueness/idempotency constraints and agent_endpoints.items[].execution plans so sorla.yaml is the durable source of truth, not generator heuristics. Use generic steps such as find_one, create, delete_where, increment_where, query with order_by, and when guards.
- Uses policies and approvals for ranking rules, fraud checks, additional permission gates, risky changes, or human review. Authorization roles decide who may invoke or mutate; approvals decide whether a permitted action still needs review.
- Keeps provider requirements abstract and capability-oriented, not hardcoded to a vendor.
- Avoids empty placeholder names like record, field, action, event whenever the plan contains domain-specific names.

The output must validate against this wizard --schema:
{wizard_schema}"#
    )
}

fn answer_generation_user_prompt(
    business_prompt: &str,
    answers: &[PromptAnswer],
    draft: &SorDesignDraft,
) -> String {
    format!(
        "Customer prompt:\n{business_prompt}\n\nFollow-up answers:\n{}\n\nDetailed plan/draft:\n{}",
        serde_json::to_string_pretty(answers).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(draft).unwrap_or_else(|_| "{}".to_string())
    )
}

fn answer_repair_system_prompt(wizard_schema: &str) -> String {
    format!(
        r#"Objective: repair answers.json so greentic-sorla wizard can use it to create the intended System of Record package.

Return JSON only. Keep the customer's business intent, preserve valid domain-specific content, and change only what is necessary to satisfy validation. Prefer fixing structure, missing required fields, invalid enum values, bad references, and schema mismatches over replacing the whole design. Do not wrap the JSON in markdown.

The repaired answers must satisfy this wizard --schema:
{wizard_schema}"#
    )
}

fn answer_repair_user_prompt(
    business_prompt: &str,
    validation_error: &str,
    answers: &serde_json::Value,
) -> String {
    format!(
        "Business prompt:\n{business_prompt}\n\nValidation errors:\n{validation_error}\n\nInvalid answers JSON:\n{}",
        serde_json::to_string_pretty(answers).unwrap_or_else(|_| answers.to_string())
    )
}

fn wizard_answers_schema_json() -> String {
    serde_json::to_string_pretty(&wizard_answers_schema_value())
        .unwrap_or_else(|_| "{}".to_string())
}

fn wizard_answers_schema_value() -> serde_json::Value {
    crate::schema_for_answers().unwrap_or_else(|_| serde_json::json!({ "type": "object" }))
}

fn authoring_output_schema_json() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "assistant_message": { "type": "string" },
            "assumptions": {
                "type": "array",
                "items": {
                    "anyOf": [
                        { "type": "string" },
                        {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "id": { "type": "string" },
                                "text": { "type": "string" },
                                "confidence": { "type": "string", "enum": ["low", "medium", "high"] }
                            },
                            "required": ["id", "text", "confidence"]
                        }
                    ]
                }
            },
            "draft": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "summary": { "type": "string" },
                    "records": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "name": { "type": "string" },
                                "description": { "type": ["string", "null"] },
                                "fields": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "additionalProperties": false,
                                        "properties": {
                                            "name": { "type": "string" },
                                            "type_name": { "type": "string" },
                                            "type": { "type": "string" },
                                            "required": { "type": "boolean" },
                                            "sensitive": { "type": "boolean" },
                                            "description": { "type": ["string", "null"] },
                                            "rules": {
                                                "type": "object",
                                                "additionalProperties": true,
                                                "properties": {
                                                    "min": { "type": ["number", "integer", "string"] },
                                                    "max": { "type": ["number", "integer", "string"] },
                                                    "min_length": { "type": "integer" },
                                                    "max_length": { "type": "integer" },
                                                    "pattern": { "type": "string" },
                                                    "precision": { "type": "integer" },
                                                    "scale": { "type": "integer" },
                                                    "before": { "type": "string" },
                                                    "after": { "type": "string" },
                                                    "unique": { "type": "boolean" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "relationships": { "type": "array", "items": { "type": "object" } },
                    "actions": { "type": "array", "items": { "type": "object" } },
                    "events": { "type": "array", "items": { "type": "object" } },
                    "projections": { "type": "array", "items": { "type": "object" } },
                    "metrics": { "type": "array", "items": { "type": "object" } },
                    "policies": { "type": "array", "items": { "type": "object" } },
                    "approvals": { "type": "array", "items": { "type": "object" } },
                    "migrations": { "type": "array", "items": { "type": "object" } },
                    "provider_requirements": { "type": "array", "items": { "type": "object" } }
                }
            },
            "questions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "id": { "type": "string" },
                        "text": { "type": "string" },
                        "help": { "type": ["string", "null"] },
                        "answer_kind": {
                            "type": "object",
                            "additionalProperties": true,
                            "properties": {
                                "kind": { "type": "string", "enum": ["free-text", "boolean", "single-choice", "multi-choice"] },
                                "choices": { "type": "array", "items": { "type": "string" } }
                            },
                            "required": ["kind"]
                        },
                        "required": { "type": "boolean" },
                        "risk": { "type": "string", "enum": ["low", "medium", "high"] },
                        "depends_on": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["id", "text", "answer_kind"]
                }
            }
        },
        "required": ["assistant_message", "draft", "questions"]
    })
}

fn answers_response_schema_json() -> serde_json::Value {
    serde_json::from_str(
        r#"{
  "type": "object",
  "additionalProperties": true,
  "properties": {
    "schema_version": { "type": "string" },
    "flow": { "type": "string", "enum": ["create", "update"] },
    "output_dir": { "type": "string" },
    "locale": { "type": ["string", "null"] },
    "package": {
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "name": { "type": "string" },
        "version": { "type": "string" }
      }
    },
    "providers": {
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "storage_category": { "type": "string" },
        "external_ref_category": { "type": "string" },
        "hints": { "type": "array", "items": { "type": "string" } }
      }
    },
    "records": {
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "default_source": { "type": "string" },
        "external_ref_system": { "type": "string" },
        "items": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": true,
            "properties": {
              "name": { "type": "string" },
              "source": { "type": "string" },
              "fields": {
                "type": "array",
                "items": {
                  "type": "object",
                  "additionalProperties": true,
                  "properties": {
                    "name": { "type": "string" },
                    "type": { "type": "string" },
                    "required": { "type": "boolean" },
                    "sensitive": { "type": "boolean" },
                    "authority": { "type": "string", "enum": ["local", "external"] },
                    "description": { "type": ["string", "null"] },
                    "enum_values": { "type": "array", "items": { "type": "string" } },
                    "references": {
                      "type": "object",
                      "additionalProperties": true,
                      "properties": {
                        "record": { "type": "string" },
                        "field": { "type": "string" }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    },
    "roles": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true,
        "properties": {
          "id": { "type": "string" },
          "i18n_key": { "type": ["string", "null"] },
          "label": { "type": ["string", "null"] },
          "description": { "type": ["string", "null"] },
          "grants": { "type": "array", "items": { "type": "string" } }
        }
      }
    },
    "actions": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
    "events": { "type": "object", "additionalProperties": true },
    "projections": {
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "mode": { "type": "string" },
        "items": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": true,
            "properties": {
              "name": { "type": "string" },
              "record": { "type": "string" },
              "source_event": { "type": "string" },
              "mode": { "type": "string" }
            }
          }
        }
      }
    },
    "operational_indexes": { "type": "object", "additionalProperties": true },
    "metrics": {
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "enabled": { "type": "boolean" },
        "items": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": true,
            "properties": {
              "name": { "type": "string" },
              "label": { "type": ["string", "null"] },
              "description": { "type": ["string", "null"] },
              "source": {
                "type": ["object", "null"],
                "additionalProperties": true,
                "properties": {
                  "kind": { "type": "string", "enum": ["record", "event", "projection"] },
                  "name": { "type": "string" }
                }
              },
              "measure": {
                "type": ["object", "null"],
                "additionalProperties": true,
                "properties": {
                  "aggregate": { "type": "string", "enum": ["count", "sum", "average", "min", "max", "count_distinct"] },
                  "field": { "type": ["string", "null"] }
                }
              },
              "filters": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
              "time": { "type": ["object", "null"], "additionalProperties": true },
              "window": { "type": ["object", "null"], "additionalProperties": true },
              "unit": { "type": ["string", "null"] },
              "dimensions": { "type": "array", "items": { "type": "string" } },
              "formula": { "type": ["string", "null"] },
              "depends_on": { "type": "array", "items": { "type": "string" } },
              "target": { "type": ["object", "null"], "additionalProperties": true }
            }
          }
        }
      }
    },
    "policies": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
    "approvals": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
    "migrations": { "type": "object", "additionalProperties": true },
    "agent_endpoints": { "type": "object", "additionalProperties": true },
    "output": { "type": "object", "additionalProperties": true }
  },
  "required": ["schema_version", "flow", "output_dir", "package", "providers", "records", "actions", "events", "projections", "policies", "approvals", "migrations", "agent_endpoints", "output"]
}"#,
    )
    .expect("answers response schema should be valid JSON")
}

fn fallback_model_output(prompt: &str) -> PromptModelOutput {
    let draft = draft_for_prompt(prompt, &[]);
    let questions = fallback_questions_for_prompt(prompt);
    PromptModelOutput {
        assistant_message: if questions.is_empty() {
            "I found an initial system-of-record shape and can propose a draft design plan."
                .to_string()
        } else {
            "I found an initial system-of-record shape and need a few decisions.".to_string()
        },
        assumptions: assumptions_for_prompt(prompt),
        draft,
        questions,
    }
}

fn fallback_questions_for_prompt(prompt: &str) -> Vec<PromptQuestion> {
    let normalized = normalize_text(prompt);
    if wants_metrics(&normalized) {
        return metric_question_graph(&normalized);
    }
    if normalized.contains("tenant") || normalized.contains("landlord") {
        return landlord_tenant_question_graph();
    }
    if normalized.contains("waiting") || normalized.contains("waitlist") {
        return vec![
            PromptQuestion {
                id: "waiting_list.scope".to_string(),
                text: "Can a person join more than one lab waiting list at the same time?"
                    .to_string(),
                help: None,
                answer_kind: PromptAnswerKind::Boolean,
                required: true,
                risk: PromptQuestionRisk::Low,
                depends_on: Vec::new(),
            },
            PromptQuestion {
                id: "referral_codes.ranking_rule".to_string(),
                text: "Should referrals always move someone up, or should there be caps or fraud checks?"
                    .to_string(),
                help: None,
                answer_kind: PromptAnswerKind::FreeText,
                required: true,
                risk: PromptQuestionRisk::Medium,
                depends_on: vec!["waiting_list.scope".to_string()],
            },
        ];
    }
    vec![PromptQuestion {
        id: "records.identity".to_string(),
        text: "Which real-world things need stable records in this system?".to_string(),
        help: None,
        answer_kind: PromptAnswerKind::FreeText,
        required: true,
        risk: PromptQuestionRisk::Low,
        depends_on: Vec::new(),
    }]
}

fn metric_question_graph(normalized_prompt: &str) -> Vec<PromptQuestion> {
    let mut questions = vec![PromptQuestion {
        id: "metrics.grain".to_string(),
        text: "Should these metrics be daily, weekly, monthly, or reported at multiple grains?"
            .to_string(),
        help: None,
        answer_kind: PromptAnswerKind::MultiChoice {
            choices: vec![
                "day".to_string(),
                "week".to_string(),
                "month".to_string(),
                "quarter".to_string(),
                "year".to_string(),
            ],
        },
        required: true,
        risk: PromptQuestionRisk::Low,
        depends_on: Vec::new(),
    }];
    if normalized_prompt.contains("revenue") {
        questions.push(PromptQuestion {
            id: "metrics.revenue_source".to_string(),
            text: "Which record or event represents recognized revenue, and which field is the monetary amount?"
                .to_string(),
            help: Some("Include statuses such as paid, settled, booked, refunded, or draft if they matter.".to_string()),
            answer_kind: PromptAnswerKind::FreeText,
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["metrics.grain".to_string()],
        });
    }
    if normalized_prompt.contains("cost") || normalized_prompt.contains("costs") {
        questions.push(PromptQuestion {
            id: "metrics.cost_source".to_string(),
            text: "Where should costs come from: invoices, campaigns, labour, subscriptions, or manual entries?"
                .to_string(),
            help: None,
            answer_kind: PromptAnswerKind::MultiChoice {
                choices: vec![
                    "invoices".to_string(),
                    "campaigns".to_string(),
                    "labour".to_string(),
                    "subscriptions".to_string(),
                    "manual entries".to_string(),
                ],
            },
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["metrics.grain".to_string()],
        });
    }
    if normalized_prompt.contains("gross") && normalized_prompt.contains("margin") {
        questions.push(PromptQuestion {
            id: "metrics.gross_margin".to_string(),
            text: "Should gross margin be reported as an amount, a ratio, or a percentage?"
                .to_string(),
            help: None,
            answer_kind: PromptAnswerKind::SingleChoice {
                choices: vec![
                    "amount".to_string(),
                    "ratio".to_string(),
                    "percentage".to_string(),
                ],
            },
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["metrics.grain".to_string()],
        });
    }
    if normalized_prompt.contains("conversion") {
        questions.push(PromptQuestion {
            id: "metrics.conversion_rate".to_string(),
            text: "What counts as a visitor or session, and what counts as a conversion?"
                .to_string(),
            help: None,
            answer_kind: PromptAnswerKind::FreeText,
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["metrics.grain".to_string()],
        });
    }
    questions.push(PromptQuestion {
        id: "metrics.dimensions".to_string(),
        text: "Which dimensions should metrics break down by, such as product, campaign, customer, or region?"
            .to_string(),
        help: None,
        answer_kind: PromptAnswerKind::MultiChoice {
            choices: vec![
                "product".to_string(),
                "campaign".to_string(),
                "customer".to_string(),
                "region".to_string(),
            ],
        },
        required: false,
        risk: PromptQuestionRisk::Low,
        depends_on: vec!["metrics.grain".to_string()],
    });
    questions
}

fn next_questions_for_session(session: &PromptSessionState) -> Vec<PromptQuestion> {
    for question in session.questions.clone() {
        if session
            .answers_so_far
            .iter()
            .any(|answer| answer.question_id == question.id)
        {
            continue;
        }
        if dependencies_are_met(&question, &session.answers_so_far) {
            return vec![question];
        }
    }
    Vec::new()
}

fn dependencies_are_met(question: &PromptQuestion, answers: &[PromptAnswer]) -> bool {
    question.depends_on.iter().all(|dependency| {
        answers
            .iter()
            .any(|answer| answer.question_id == *dependency)
    })
}

fn landlord_tenant_question_graph() -> Vec<PromptQuestion> {
    vec![
        PromptQuestion {
            id: "lease.multiple_tenants".to_string(),
            text: "Can a lease have more than one tenant?".to_string(),
            help: None,
            answer_kind: PromptAnswerKind::Boolean,
            required: true,
            risk: PromptQuestionRisk::Low,
            depends_on: Vec::new(),
        },
        PromptQuestion {
            id: "lease.liability".to_string(),
            text: "Should tenant liability be joint, individual, or both?".to_string(),
            help: None,
            answer_kind: PromptAnswerKind::SingleChoice {
                choices: vec![
                    "joint".to_string(),
                    "individual".to_string(),
                    "both".to_string(),
                ],
            },
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["lease.multiple_tenants".to_string()],
        },
        PromptQuestion {
            id: "payments.immutable".to_string(),
            text: "Should payments be immutable ledger-style events?".to_string(),
            help: None,
            answer_kind: PromptAnswerKind::Boolean,
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["lease.liability".to_string()],
        },
        PromptQuestion {
            id: "maintenance.uses_suppliers".to_string(),
            text: "Do maintenance requests involve external suppliers?".to_string(),
            help: None,
            answer_kind: PromptAnswerKind::Boolean,
            required: true,
            risk: PromptQuestionRisk::Medium,
            depends_on: vec!["payments.immutable".to_string()],
        },
        PromptQuestion {
            id: "supplier.approval_required".to_string(),
            text: "Does supplier work require approval?".to_string(),
            help: None,
            answer_kind: PromptAnswerKind::Boolean,
            required: true,
            risk: PromptQuestionRisk::High,
            depends_on: vec!["maintenance.uses_suppliers".to_string()],
        },
    ]
}

fn parse_answer_value(kind: &PromptAnswerKind, message: &str) -> PromptAnswerValue {
    match kind {
        PromptAnswerKind::FreeText => PromptAnswerValue::FreeText(message.trim().to_string()),
        PromptAnswerKind::Boolean => PromptAnswerValue::Boolean(is_affirmative(message)),
        PromptAnswerKind::SingleChoice { choices } => {
            let normalized = normalize_text(message);
            let choice = choices
                .iter()
                .find(|choice| normalized.contains(&normalize_text(choice)))
                .cloned()
                .unwrap_or_else(|| choices.first().cloned().unwrap_or_default());
            PromptAnswerValue::SingleChoice(choice)
        }
        PromptAnswerKind::MultiChoice { choices } => {
            let normalized = normalize_text(message);
            let selected = choices
                .iter()
                .filter(|choice| normalized.contains(&normalize_text(choice)))
                .cloned()
                .collect::<Vec<_>>();
            PromptAnswerValue::MultiChoice(selected)
        }
    }
}

fn should_generate_now(message: &str) -> bool {
    matches!(
        normalize_text(message).as_str(),
        "generate answers" | "generate answer" | "done" | "skip" | "finish"
    )
}

fn is_affirmative(message: &str) -> bool {
    let normalized = normalize_text(message);
    [
        "yes", "true", "y", "required", "requires", "require", "use", "uses", "multiple",
    ]
    .iter()
    .any(|word| normalized.split_whitespace().any(|token| token == *word))
}

fn normalize_text(input: &str) -> String {
    input
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
}

fn wants_metrics(normalized: &str) -> bool {
    [
        "metric",
        "metrics",
        "kpi",
        "kpis",
        "track",
        "dashboard",
        "report",
        "click",
        "clicks",
        "revenue",
        "cost",
        "costs",
        "gross margin",
        "conversion",
        "roas",
        "cac",
        "mrr",
        "churn",
    ]
    .iter()
    .any(|term| normalized.contains(term))
}

fn assumptions_for_prompt(prompt: &str) -> Vec<PromptAssumption> {
    let mut assumptions = vec![PromptAssumption {
        id: "durable-records".to_string(),
        text: "The business prompt describes durable system-of-record data.".to_string(),
        confidence: PromptAssumptionConfidence::High,
    }];

    let normalized = normalize_text(prompt);
    if normalized.contains("tenant") || normalized.contains("landlord") {
        assumptions.push(PromptAssumption {
            id: "landlord-tenant-domain".to_string(),
            text: "The domain likely needs landlord, tenant, property, lease, payment, and maintenance records.".to_string(),
            confidence: PromptAssumptionConfidence::High,
        });
    }
    if normalized.contains("waiting") || normalized.contains("waitlist") {
        assumptions.push(PromptAssumption {
            id: "waiting-list-domain".to_string(),
            text: "The domain needs lab-scoped waiting lists, people, referral codes, and ranking history.".to_string(),
            confidence: PromptAssumptionConfidence::High,
        });
    }
    if wants_metrics(&normalized) {
        assumptions.push(PromptAssumption {
            id: "metrics-domain".to_string(),
            text: "The prompt asks for metrics or KPIs that should be modeled as validated metric definitions.".to_string(),
            confidence: PromptAssumptionConfidence::High,
        });
    }
    assumptions
}

fn draft_for_prompt(prompt: &str, answers: &[PromptAnswer]) -> SorDesignDraft {
    let normalized = normalize_text(prompt);
    if wants_metrics(&normalized) {
        return metrics_draft(&normalized, answers);
    }
    if normalized.contains("tenant") || normalized.contains("landlord") {
        return landlord_tenant_draft(answers);
    }
    if normalized.contains("waiting") || normalized.contains("waitlist") {
        return waiting_list_draft(answers);
    }

    SorDesignDraft {
        summary: "Prompt-generated system of record".to_string(),
        records: vec![record("case", "Business case", &[field("id", "string")])],
        ..SorDesignDraft::default()
    }
}

fn metrics_draft(normalized_prompt: &str, answers: &[PromptAnswer]) -> SorDesignDraft {
    let grain = metric_grain_answer(answers).unwrap_or_else(|| "month".to_string());
    let dimensions = metric_dimensions_answer(answers);
    let include_clicks =
        normalized_prompt.contains("click") || normalized_prompt.contains("clicks");
    let include_revenue = normalized_prompt.contains("revenue")
        || normalized_prompt.contains("roas")
        || normalized_prompt.contains("mrr")
        || normalized_prompt.contains("gross margin");
    let include_cost =
        normalized_prompt.contains("cost") || normalized_prompt.contains("gross margin");
    let include_gross_margin = normalized_prompt.contains("gross margin");

    let mut records = Vec::new();
    let mut events = Vec::new();
    let mut metrics = Vec::new();

    if include_clicks {
        records.push(record(
            "click",
            "Tracked click or interaction",
            &[
                field("id", "uuid"),
                field("clicked_at", "datetime"),
                field("campaign", "string"),
                field("product", "string"),
                field("customer", "string"),
                field("region", "string"),
            ],
        ));
        events.push(DraftEvent {
            name: "click_tracked".to_string(),
            description: Some("A click was tracked.".to_string()),
        });
        metrics.push(metric_count(
            "clicks",
            "Clicks",
            "click",
            "clicked_at",
            &grain,
            &dimensions,
        ));
    }

    if include_revenue {
        records.push(record(
            "order",
            "Revenue-bearing order",
            &[
                field("id", "uuid"),
                field("amount", "decimal"),
                field("status", "string"),
                field("recognized_at", "datetime"),
                field("campaign", "string"),
                field("product", "string"),
                field("customer", "string"),
                field("region", "string"),
            ],
        ));
        events.push(DraftEvent {
            name: "revenue_recognized".to_string(),
            description: Some("Revenue was recognized for an order.".to_string()),
        });
        metrics.push(DraftMetric {
            name: "revenue".to_string(),
            label: Some("Revenue".to_string()),
            description: Some("Recognized revenue over time.".to_string()),
            source_record: Some("order".to_string()),
            aggregate: Some("sum".to_string()),
            field: Some("amount".to_string()),
            time_field: Some("recognized_at".to_string()),
            grain: Some(grain.clone()),
            unit: Some("GBP".to_string()),
            dimensions: dimensions.clone(),
            formula: None,
            depends_on: Vec::new(),
            filters: vec![DraftMetricFilter {
                field: "status".to_string(),
                operator: "equals".to_string(),
                value: Some(serde_json::json!("paid")),
            }],
        });
    }

    if include_cost {
        records.push(record(
            "cost",
            "Cost entry",
            &[
                field("id", "uuid"),
                field("amount", "decimal"),
                field("incurred_at", "datetime"),
                field("campaign", "string"),
                field("product", "string"),
                field("customer", "string"),
                field("region", "string"),
            ],
        ));
        events.push(DraftEvent {
            name: "cost_incurred".to_string(),
            description: Some("A cost was incurred.".to_string()),
        });
        metrics.push(DraftMetric {
            name: "cost".to_string(),
            label: Some("Cost".to_string()),
            description: Some("Costs over time.".to_string()),
            source_record: Some("cost".to_string()),
            aggregate: Some("sum".to_string()),
            field: Some("amount".to_string()),
            time_field: Some("incurred_at".to_string()),
            grain: Some(grain.clone()),
            unit: Some("GBP".to_string()),
            dimensions: dimensions.clone(),
            formula: None,
            depends_on: Vec::new(),
            filters: Vec::new(),
        });
    }

    if include_gross_margin {
        metrics.push(DraftMetric {
            name: "gross_margin".to_string(),
            label: Some("Gross Margin".to_string()),
            description: Some("Revenue minus cost.".to_string()),
            source_record: None,
            aggregate: None,
            field: None,
            time_field: None,
            grain: None,
            unit: Some("GBP".to_string()),
            dimensions,
            formula: Some("revenue - cost".to_string()),
            depends_on: vec!["revenue".to_string(), "cost".to_string()],
            filters: Vec::new(),
        });
    }

    if records.is_empty() {
        records.push(record(
            "metric_event",
            "Metric source event",
            &[
                field("id", "uuid"),
                field("value", "decimal"),
                field("occurred_at", "datetime"),
            ],
        ));
        metrics.push(DraftMetric {
            name: "tracked_metric".to_string(),
            label: Some("Tracked Metric".to_string()),
            description: Some("Generic metric value over time.".to_string()),
            source_record: Some("metric_event".to_string()),
            aggregate: Some("sum".to_string()),
            field: Some("value".to_string()),
            time_field: Some("occurred_at".to_string()),
            grain: Some(grain),
            unit: None,
            dimensions: Vec::new(),
            formula: None,
            depends_on: Vec::new(),
            filters: Vec::new(),
        });
    }

    SorDesignDraft {
        summary: "Metrics and KPI system of record".to_string(),
        records,
        events,
        projections: vec![DraftProjection {
            name: "metrics_dashboard".to_string(),
            description: Some("Dashboard projection for metric reporting.".to_string()),
        }],
        metrics,
        ..SorDesignDraft::default()
    }
}

fn metric_count(
    name: &str,
    label: &str,
    source_record: &str,
    time_field: &str,
    grain: &str,
    dimensions: &[String],
) -> DraftMetric {
    DraftMetric {
        name: name.to_string(),
        label: Some(label.to_string()),
        description: Some(format!("{label} over time.")),
        source_record: Some(source_record.to_string()),
        aggregate: Some("count".to_string()),
        field: None,
        time_field: Some(time_field.to_string()),
        grain: Some(grain.to_string()),
        unit: None,
        dimensions: dimensions.to_vec(),
        formula: None,
        depends_on: Vec::new(),
        filters: Vec::new(),
    }
}

fn waiting_list_draft(_answers: &[PromptAnswer]) -> SorDesignDraft {
    SorDesignDraft {
        summary: "Greentic lab waiting list system of record".to_string(),
        records: vec![
            record(
                "lab",
                "Greentic lab that owns a separate waiting list",
                &[field("id", "uuid"), field("name", "string")],
            ),
            record(
                "person",
                "Person joining one or more waiting lists",
                &[
                    field("id", "uuid"),
                    field("email", "email"),
                    field("display_name", "string"),
                ],
            ),
            record(
                "waiting_list_entry",
                "Person's position and state on a lab waiting list",
                &[
                    field("id", "uuid"),
                    field("lab_id", "uuid"),
                    field("person_id", "uuid"),
                    field("status", "string"),
                    field("rank_score", "integer"),
                ],
            ),
            record(
                "referral_code",
                "Referral code that can improve waiting list rank",
                &[
                    field("id", "uuid"),
                    field("code", "string"),
                    field("owner_person_id", "uuid"),
                    field("lab_id", "uuid"),
                ],
            ),
            record(
                "referral",
                "Accepted referral between two people",
                &[
                    field("id", "uuid"),
                    field("referral_code_id", "uuid"),
                    field("referred_person_id", "uuid"),
                ],
            ),
        ],
        actions: vec![
            DraftAction {
                name: "join_waiting_list".to_string(),
                description: Some(
                    "Join a lab waiting list, optionally with a referral code.".to_string(),
                ),
                risk: super::DraftRisk::Low,
            },
            DraftAction {
                name: "leave_waiting_list".to_string(),
                description: Some("Leave a lab waiting list.".to_string()),
                risk: super::DraftRisk::Low,
            },
            DraftAction {
                name: "apply_referral_code".to_string(),
                description: Some("Apply a referral code and update ranking inputs.".to_string()),
                risk: super::DraftRisk::Medium,
            },
        ],
        events: vec![
            DraftEvent {
                name: "waiting_list_joined".to_string(),
                description: None,
            },
            DraftEvent {
                name: "waiting_list_left".to_string(),
                description: None,
            },
            DraftEvent {
                name: "referral_accepted".to_string(),
                description: None,
            },
        ],
        projections: vec![DraftProjection {
            name: "visible_waiting_list".to_string(),
            description: Some("Public or member-facing view of waiting list position.".to_string()),
        }],
        policies: vec![DraftPolicy {
            name: "referral_ranking_policy".to_string(),
            description: Some("Controls how referrals affect waiting list ordering.".to_string()),
        }],
        ..SorDesignDraft::default()
    }
}

fn landlord_tenant_draft(answers: &[PromptAnswer]) -> SorDesignDraft {
    let supplier_approval = boolean_answer(answers, "supplier.approval_required").unwrap_or(true);
    SorDesignDraft {
        summary: "Landlord tenant property management system of record".to_string(),
        records: vec![
            record(
                "landlord",
                "Property owner",
                &[field("id", "uuid"), field("name", "string")],
            ),
            record(
                "tenant",
                "Lease tenant",
                &[field("id", "uuid"), field("name", "string")],
            ),
            record(
                "property",
                "Managed property",
                &[field("id", "uuid"), field("address", "string")],
            ),
            record(
                "lease",
                "Rental lease",
                &[field("id", "uuid"), field("status", "string")],
            ),
            record(
                "payment",
                "Rent payment",
                &[field("id", "uuid"), field("amount", "decimal")],
            ),
            record(
                "maintenance_request",
                "Maintenance request",
                &[field("id", "uuid"), field("status", "string")],
            ),
            record(
                "supplier",
                "Maintenance supplier",
                &[field("id", "uuid"), field("name", "string")],
            ),
        ],
        actions: vec![DraftAction {
            name: "approve_supplier_work".to_string(),
            description: Some("Approve supplier work for a maintenance request.".to_string()),
            risk: super::DraftRisk::Medium,
        }],
        events: vec![
            DraftEvent {
                name: "lease_started".to_string(),
                description: None,
            },
            DraftEvent {
                name: "payment_recorded".to_string(),
                description: None,
            },
            DraftEvent {
                name: "maintenance_request_opened".to_string(),
                description: None,
            },
        ],
        projections: vec![DraftProjection {
            name: "active_leases".to_string(),
            description: None,
        }],
        policies: vec![DraftPolicy {
            name: "supplier_work_policy".to_string(),
            description: Some("Controls supplier work authorization.".to_string()),
        }],
        approvals: if supplier_approval {
            vec![DraftApproval {
                name: "supplier_work_approval".to_string(),
                description: Some("Requires approval before supplier work proceeds.".to_string()),
                required: true,
            }]
        } else {
            Vec::new()
        },
        ..SorDesignDraft::default()
    }
}

fn record(name: &str, description: &str, fields: &[DraftField]) -> DraftRecord {
    DraftRecord {
        name: name.to_string(),
        description: Some(description.to_string()),
        fields: fields.to_vec(),
    }
}

fn field(name: &str, type_name: &str) -> DraftField {
    DraftField {
        name: name.to_string(),
        type_name: type_name.to_string(),
        required: true,
        sensitive: false,
        description: None,
        rules: None,
    }
}

fn boolean_answer(answers: &[PromptAnswer], question_id: &str) -> Option<bool> {
    answers.iter().find_map(|answer| {
        if answer.question_id == question_id
            && let PromptAnswerValue::Boolean(value) = answer.value
        {
            return Some(value);
        }
        None
    })
}

fn metric_grain_answer(answers: &[PromptAnswer]) -> Option<String> {
    answers.iter().find_map(|answer| {
        if answer.question_id != "metrics.grain" {
            return None;
        }
        match &answer.value {
            PromptAnswerValue::SingleChoice(value) => Some(value.clone()),
            PromptAnswerValue::MultiChoice(values) => values.first().cloned(),
            PromptAnswerValue::FreeText(value) => {
                let normalized = normalize_text(value);
                ["day", "week", "month", "quarter", "year"]
                    .into_iter()
                    .find(|grain| normalized.contains(grain))
                    .map(str::to_string)
            }
            PromptAnswerValue::Boolean(_) => None,
        }
    })
}

fn metric_dimensions_answer(answers: &[PromptAnswer]) -> Vec<String> {
    answers
        .iter()
        .find_map(|answer| {
            if answer.question_id != "metrics.dimensions" {
                return None;
            }
            match &answer.value {
                PromptAnswerValue::MultiChoice(values) => Some(values.clone()),
                PromptAnswerValue::SingleChoice(value) => Some(vec![value.clone()]),
                PromptAnswerValue::FreeText(value) => {
                    let normalized = normalize_text(value);
                    Some(
                        ["product", "campaign", "customer", "region"]
                            .into_iter()
                            .filter(|dimension| normalized.contains(dimension))
                            .map(str::to_string)
                            .collect(),
                    )
                }
                PromptAnswerValue::Boolean(_) => None,
            }
        })
        .unwrap_or_else(|| vec!["product".to_string(), "campaign".to_string()])
}

fn answers_from_draft(draft: &SorDesignDraft) -> serde_json::Value {
    let is_landlord_tenant = draft.records.iter().any(|record| record.name == "lease")
        && draft.records.iter().any(|record| record.name == "tenant");
    let is_waiting_list = draft
        .records
        .iter()
        .any(|record| record.name == "waiting_list_entry")
        && draft
            .actions
            .iter()
            .any(|action| action.name == "join_waiting_list");
    if is_waiting_list {
        return waiting_list_answers_from_draft();
    }
    let records = draft
        .records
        .iter()
        .map(|record| {
            serde_json::json!({
                "name": record.name,
                "source": "native",
                "fields": record.fields.iter().map(|field| {
                    let mut field_value = serde_json::json!({
                        "name": field.name,
                        "type": field.type_name,
                        "required": field.required,
                        "sensitive": field.sensitive,
                        "description": field.description
                    });
                    let rules = field
                        .rules
                        .clone()
                        .or_else(|| inferred_field_rules(&record.name, field));
                    if let Some(rules) = rules
                        && let Some(object) = field_value.as_object_mut()
                    {
                        object.insert("rules".to_string(), rules);
                    }
                    field_value
                }).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    let event_items = draft
        .events
        .iter()
        .map(|event| {
            let record = infer_event_record(&event.name, draft);
            serde_json::json!({
                "name": event.name,
                "record": record,
                "kind": "domain",
                "emits": [{ "name": format!("{record}_id"), "type": "uuid" }]
            })
        })
        .collect::<Vec<_>>();
    let projection_items = draft
        .projections
        .iter()
        .filter_map(|projection| {
            let first_event = event_items.first()?;
            Some(serde_json::json!({
                "name": projection.name,
                "record": first_event["record"],
                "source_event": first_event["name"],
                "mode": "current-state"
            }))
        })
        .collect::<Vec<_>>();
    let metric_items = draft
        .metrics
        .iter()
        .map(metric_answer_value)
        .collect::<Vec<_>>();

    serde_json::json!({
        "schema_version": "0.5",
        "flow": "create",
        "output_dir": "target/greentic-sorla-prompt-generated",
        "package": {
            "name": "prompt-generated-sor",
            "version": "0.1.0"
        },
        "providers": {
            "storage_category": "storage",
            "hints": ["prompt-authoring"]
        },
        "records": {
            "default_source": "native",
            "items": records
        },
        "actions": draft.actions.iter().map(|action| serde_json::json!({ "name": action.name })).collect::<Vec<_>>(),
        "events": {
            "enabled": !event_items.is_empty(),
            "items": event_items
        },
        "projections": {
            "mode": "current-state",
            "items": projection_items
        },
        "metrics": {
            "enabled": !metric_items.is_empty(),
            "items": metric_items
        },
        "policies": draft.policies.iter().map(|policy| serde_json::json!({ "name": policy.name })).collect::<Vec<_>>(),
        "approvals": draft.approvals.iter().map(|approval| serde_json::json!({ "name": approval.name })).collect::<Vec<_>>(),
        "migrations": {
            "compatibility": "additive"
        },
        "agent_endpoints": {
            "enabled": is_landlord_tenant,
            "ids": if is_landlord_tenant { vec![
                "create_tenant",
                "record_rent_payment",
                "add_maintenance_request"
            ] } else { Vec::<&str>::new() },
            "default_risk": "medium",
            "default_approval": "policy-driven",
            "exports": ["openapi", "arazzo", "mcp", "llms_txt"],
            "provider_category": "storage"
        },
        "output": {
            "include_agent_tools": true
        }
    })
}

fn inferred_field_rules(record_name: &str, field: &DraftField) -> Option<serde_json::Value> {
    let name = field.name.as_str();
    match field.type_name.as_str() {
        "uuid" if name == "id" || name == format!("{record_name}_id") => {
            Some(serde_json::json!({ "unique": true }))
        }
        "email" => Some(serde_json::json!({ "max_length": 320 })),
        "url" => Some(serde_json::json!({ "max_length": 2048 })),
        "decimal" if matches!(name, "amount" | "revenue" | "cost" | "value") => {
            Some(serde_json::json!({ "min": 0, "precision": 12, "scale": 2 }))
        }
        "integer" if name.contains("count") || name.contains("score") || name.contains("rank") => {
            Some(serde_json::json!({ "min": 0 }))
        }
        "string" if matches!(name, "name" | "display_name") => {
            Some(serde_json::json!({ "min_length": 1, "max_length": 160 }))
        }
        "string" if name.contains("code") => {
            Some(serde_json::json!({ "min_length": 1, "max_length": 64 }))
        }
        "string" if name == "status" => Some(serde_json::json!({ "max_length": 64 })),
        _ => None,
    }
}

fn waiting_list_answers_from_draft() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "0.5",
        "flow": "create",
        "output_dir": "target/greentic-sorla-prompt-generated",
        "package": {
            "name": "prompt-generated-sor",
            "version": "0.1.0"
        },
        "providers": {
            "storage_category": "storage",
            "hints": ["prompt-authoring"]
        },
        "records": {
            "default_source": "native",
            "items": [
                {
                    "name": "lab",
                    "source": "native",
                    "fields": [
                        { "name": "lab_id", "type": "uuid", "required": true, "sensitive": false, "rules": { "unique": true } },
                        { "name": "name", "type": "string", "required": true, "sensitive": false, "rules": { "min_length": 1, "max_length": 160 } }
                    ]
                },
                {
                    "name": "waiting_list_entry",
                    "source": "native",
                    "fields": [
                        { "name": "entry_id", "type": "uuid", "required": true, "sensitive": false, "rules": { "unique": true } },
                        { "name": "lab_id", "type": "uuid", "required": true, "sensitive": false },
                        { "name": "user_id", "type": "uuid", "required": true, "sensitive": false },
                        { "name": "email", "type": "email", "required": true, "sensitive": false, "rules": { "max_length": 320 } },
                        { "name": "name", "type": "string", "required": true, "sensitive": false, "rules": { "min_length": 1, "max_length": 160 } },
                        { "name": "invitation_code", "type": "string", "required": true, "sensitive": false, "rules": { "min_length": 6, "max_length": 64 } },
                        { "name": "invited_by_code", "type": "string", "required": false, "sensitive": false, "rules": { "min_length": 6, "max_length": 64 } },
                        { "name": "referrer_entry_id", "type": "uuid", "required": false, "sensitive": false },
                        { "name": "referred_count", "type": "integer", "required": true, "sensitive": false, "rules": { "min": 0 } },
                        { "name": "joined_at", "type": "datetime", "required": true, "sensitive": false }
                    ]
                }
            ]
        },
        "actions": [
            { "name": "join_waiting_list", "description": "Add a user to a lab waiting list once by email, optionally using an invitation code.", "risk": "medium" },
            { "name": "leave_waiting_list", "description": "Remove a user from a lab waiting list by email.", "risk": "medium" },
            { "name": "show_waiting_list", "description": "Retrieve the ordered waiting list for a lab.", "risk": "low" },
            { "name": "retrieve_invitation_code", "description": "Retrieve the existing invitation code for an entry.", "risk": "low" }
        ],
        "events": { "enabled": false, "items": [] },
        "projections": { "mode": "current-state", "items": [] },
        "operational_indexes": {
            "schema": "greentic.sorla.operational-indexes.v1",
            "indexes": [
                {
                    "id": "waiting_list_entry_lab_email_unique",
                    "record": "waiting_list_entry",
                    "kind": "composite",
                    "unique": true,
                    "fields": ["lab_id", "email"]
                },
                {
                    "id": "waiting_list_entry_lab_invitation_code_unique",
                    "record": "waiting_list_entry",
                    "kind": "composite",
                    "unique": true,
                    "fields": ["lab_id", "invitation_code"]
                }
            ],
            "query_requirements": [
                {
                    "id": "join_waiting_list_idempotency",
                    "used_by": { "agent_endpoint": "join_waiting_list" },
                    "requires_index": "waiting_list_entry_lab_email_unique"
                }
            ]
        },
        "metrics": {
            "enabled": true,
            "items": [
                {
                    "name": "number_in_waiting_list",
                    "label": "Number in waiting list",
                    "source": { "kind": "record", "name": "waiting_list_entry" },
                    "measure": { "aggregate": "count" },
                    "dimensions": ["lab_id"]
                }
            ]
        },
        "policies": [],
        "approvals": [],
        "migrations": { "compatibility": "additive" },
        "agent_endpoints": {
            "enabled": true,
            "default_risk": "medium",
            "default_approval": "policy-driven",
            "exports": ["openapi", "arazzo", "mcp", "llms_txt"],
            "provider_category": "storage",
            "items": [
                {
                    "id": "join_waiting_list",
                    "title": "Join waiting list",
                    "intent": "Add a user to a lab waiting list once by email, optionally using an invitation code, and return their invitation code and current list metrics.",
                    "inputs": [
                        { "name": "lab_id", "type": "uuid", "required": true },
                        { "name": "email", "type": "email", "required": true },
                        { "name": "name", "type": "string", "required": true },
                        { "name": "invited_by_code", "type": "string", "required": false }
                    ],
                    "outputs": [
                        { "name": "entry_id", "type": "string" },
                        { "name": "invitation_code", "type": "string" },
                        { "name": "position", "type": "integer" },
                        { "name": "number_in_waiting_list", "type": "integer" }
                    ],
                    "side_effects": ["action.join_waiting_list"],
                    "backing": { "actions": ["join_waiting_list"] },
                    "execution": {
                        "kind": "record_mutation",
                        "action": "join_waiting_list",
                        "idempotency": "return_existing",
                        "target": "waiting_list_entries",
                        "constraints": {
                            "idempotency": {
                                "mode": "return_existing",
                                "index": "waiting_list_entry_lab_email_unique",
                                "fields": ["lab_id", "email"]
                            },
                            "unique": [
                                {
                                    "index": "waiting_list_entry_lab_email_unique",
                                    "record": "waiting_list_entry",
                                    "kind": "composite",
                                    "fields": ["lab_id", "email"]
                                },
                                {
                                    "index": "waiting_list_entry_lab_invitation_code_unique",
                                    "record": "waiting_list_entry",
                                    "kind": "composite",
                                    "fields": ["lab_id", "invitation_code"]
                                }
                            ]
                        },
                        "steps": [
                            {
                                "op": "find_one",
                                "as": "referrer",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": {
                                    "lab_id": "$input.lab_id",
                                    "invitation_code": "$input.invited_by_code"
                                },
                                "required": true,
                                "when": { "present": "$input.invited_by_code" }
                            },
                            {
                                "op": "create",
                                "as": "entry",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "input": {
                                    "email": "$input.email",
                                    "entry_id": "$generated.entry_id",
                                    "invitation_code": "$generated.invitation_code",
                                    "invited_by_code": "$input.invited_by_code",
                                    "joined_at": "$now",
                                    "lab_id": "$input.lab_id",
                                    "name": "$input.name",
                                    "referred_count": 0,
                                    "referrer_entry_id": "$steps.referrer.data.entry_id",
                                    "user_id": "$generated.uuid"
                                }
                            },
                            {
                                "op": "increment_where",
                                "as": "referrer_increment",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": {
                                    "lab_id": "$input.lab_id",
                                    "invitation_code": "$input.invited_by_code"
                                },
                                "increments": { "referred_count": 1 },
                                "when": {
                                    "all": [
                                        { "present": "$input.invited_by_code" },
                                        { "equals": ["$steps.entry.created", true] }
                                    ]
                                }
                            },
                            {
                                "op": "query",
                                "as": "waiting_list",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": { "lab_id": "$input.lab_id" },
                                "order_by": [
                                    { "field": "referred_count", "direction": "desc" },
                                    { "field": "joined_at", "direction": "asc" }
                                ]
                            }
                        ],
                        "return": {
                            "entry_id": "$steps.entry.record.data.entry_id",
                            "invitation_code": "$steps.entry.record.data.invitation_code",
                            "number_in_waiting_list": "$steps.waiting_list.count",
                            "position": "$steps.waiting_list.count"
                        }
                    }
                },
                {
                    "id": "leave_waiting_list",
                    "title": "Leave waiting list",
                    "intent": "Remove a user from a lab waiting list by email and decrement the referrer count if their invitation was used.",
                    "inputs": [
                        { "name": "lab_id", "type": "string", "required": true },
                        { "name": "email", "type": "email", "required": true }
                    ],
                    "outputs": [
                        { "name": "deleted_count", "type": "integer" }
                    ],
                    "side_effects": ["action.leave_waiting_list"],
                    "backing": { "actions": ["leave_waiting_list"] },
                    "execution": {
                        "kind": "record_mutation",
                        "action": "leave_waiting_list",
                        "target": "waiting_list_entries",
                        "steps": [
                            {
                                "op": "find_one",
                                "as": "leaving_entry",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": {
                                    "email": "$input.email",
                                    "lab_id": "$input.lab_id"
                                },
                                "required": true
                            },
                            {
                                "op": "delete_where",
                                "as": "leave",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": {
                                    "email": "$input.email",
                                    "lab_id": "$input.lab_id"
                                }
                            },
                            {
                                "op": "increment_where",
                                "as": "referrer_decrement",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": {
                                    "entry_id": "$steps.leaving_entry.data.referrer_entry_id",
                                    "lab_id": "$input.lab_id"
                                },
                                "increments": { "referred_count": -1 },
                                "when": {
                                    "all": [
                                        { "present": "$steps.leaving_entry.data.referrer_entry_id" },
                                        { "equals": ["$steps.leave.deleted_count", 1] }
                                    ]
                                }
                            }
                        ],
                        "return": { "deleted_count": "$steps.leave.deleted_count" }
                    }
                },
                {
                    "id": "show_waiting_list",
                    "title": "Show waiting list",
                    "intent": "Retrieve the ordered waiting list for a lab, sorted by referral count descending and join time ascending.",
                    "inputs": [
                        { "name": "lab_id", "type": "string", "required": true }
                    ],
                    "outputs": [
                        { "name": "entries", "type": "array" },
                        { "name": "count", "type": "integer" }
                    ],
                    "side_effects": ["action.show_waiting_list"],
                    "backing": { "actions": ["show_waiting_list"] },
                    "execution": {
                        "kind": "record_query",
                        "action": "show_waiting_list",
                        "target": "waiting_list_entries",
                        "steps": [
                            {
                                "op": "query",
                                "as": "waiting_list",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": { "lab_id": "$input.lab_id" },
                                "order_by": [
                                    { "field": "referred_count", "direction": "desc" },
                                    { "field": "joined_at", "direction": "asc" }
                                ]
                            }
                        ],
                        "return": {
                            "entries": "$steps.waiting_list.records",
                            "count": "$steps.waiting_list.count"
                        }
                    }
                },
                {
                    "id": "retrieve_invitation_code",
                    "title": "Retrieve invitation code",
                    "intent": "Retrieve an existing invitation code for an entry.",
                    "inputs": [
                        { "name": "entry_id", "type": "string", "required": true }
                    ],
                    "outputs": [
                        { "name": "invitation_code", "type": "string" }
                    ],
                    "side_effects": ["action.retrieve_invitation_code"],
                    "backing": { "actions": ["retrieve_invitation_code"] },
                    "execution": {
                        "kind": "record_query",
                        "action": "retrieve_invitation_code",
                        "target": "waiting_list_entries",
                        "steps": [
                            {
                                "op": "find_one",
                                "as": "entry",
                                "entity": "waiting_list_entry",
                                "collection": "waiting_list_entries",
                                "where": { "entry_id": "$input.entry_id" },
                                "required": true
                            }
                        ],
                        "return": { "invitation_code": "$steps.entry.data.invitation_code" }
                    }
                }
            ]
        },
        "output": {
            "include_agent_tools": true
        }
    })
}

fn infer_event_record<'a>(event_name: &str, draft: &'a SorDesignDraft) -> &'a str {
    for record in &draft.records {
        if event_name.contains(&record.name) {
            return &record.name;
        }
    }
    if event_name.contains("click") {
        return "click";
    }
    if event_name.contains("revenue") {
        return "order";
    }
    if event_name.contains("cost") {
        return "cost";
    }
    if event_name.contains("payment") {
        return "payment";
    }
    if event_name.contains("maintenance") {
        return "maintenance_request";
    }
    if event_name.contains("lease") {
        return "lease";
    }
    draft
        .records
        .first()
        .map(|record| record.name.as_str())
        .unwrap_or("record")
}

fn metric_answer_value(metric: &DraftMetric) -> serde_json::Value {
    let mut value = serde_json::json!({
        "name": metric.name,
        "label": metric.label,
        "description": metric.description,
        "filters": metric.filters.iter().map(|filter| {
            serde_json::json!({
                "field": filter.field,
                "operator": filter.operator,
                "value": filter.value
            })
        }).collect::<Vec<_>>(),
        "unit": metric.unit,
        "dimensions": metric.dimensions,
        "formula": metric.formula,
        "depends_on": metric.depends_on
    });
    if let Some(source_record) = &metric.source_record {
        value["source"] = serde_json::json!({
            "kind": "record",
            "name": source_record
        });
    }
    if let Some(aggregate) = &metric.aggregate {
        value["measure"] = serde_json::json!({
            "aggregate": aggregate,
            "field": metric.field
        });
    }
    if let (Some(time_field), Some(grain)) = (&metric.time_field, &metric.grain) {
        value["time"] = serde_json::json!({
            "field": time_field,
            "grain": grain
        });
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{LlmResponse, LlmResponseFormat};
    use std::cell::{Cell, RefCell};

    struct FakePromptLlm;

    fn assert_schema_response_format(format: Option<LlmResponseFormat>) {
        match format {
            Some(LlmResponseFormat::JsonSchema { schema, .. }) => {
                assert!(schema.is_object());
            }
            other => panic!("expected JSON schema response format, got {other:?}"),
        }
    }

    impl LlmCapability for FakePromptLlm {
        fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SorlaError> {
            assert_schema_response_format(request.response_format);
            Ok(LlmResponse {
                content: "{}".to_string(),
            })
        }
    }

    struct RepairPromptLlm {
        calls: Cell<usize>,
    }

    impl LlmCapability for RepairPromptLlm {
        fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SorlaError> {
            assert_schema_response_format(request.response_format);
            self.calls.set(self.calls.get() + 1);
            Ok(LlmResponse {
                content: serde_json::json!({
                    "schema_version": "0.5",
                    "flow": "create",
                    "output_dir": "target/repaired",
                    "package": { "name": "repaired-sor", "version": "0.1.0" },
                    "providers": { "storage_category": "storage", "hints": [] },
                    "records": {
                        "default_source": "native",
                        "items": [{
                            "name": "case",
                            "source": "native",
                            "fields": [{ "name": "id", "type": "string", "required": true, "sensitive": false }]
                        }]
                    },
                    "actions": [],
                    "events": { "enabled": false, "items": [] },
                    "projections": { "mode": "current-state", "items": [] },
                    "policies": [],
                    "approvals": [],
                    "migrations": { "compatibility": "additive" },
                    "agent_endpoints": {
                        "enabled": false,
                        "ids": [],
                        "default_risk": "medium",
                        "default_approval": "policy-driven",
                        "exports": ["openapi"],
                        "provider_category": "storage"
                    },
                    "output": { "include_agent_tools": true }
                })
                .to_string(),
            })
        }
    }

    struct AuthoringRepairLlm {
        calls: Cell<usize>,
        system_prompts: RefCell<Vec<String>>,
    }

    impl LlmCapability for AuthoringRepairLlm {
        fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SorlaError> {
            assert_schema_response_format(request.response_format);
            self.calls.set(self.calls.get() + 1);
            self.system_prompts.borrow_mut().push(request.system_prompt);
            let content = if self.calls.get() == 1 {
                serde_json::json!({ "records": [] }).to_string()
            } else {
                serde_json::json!({
                    "assistant_message": "I repaired the authoring draft.",
                    "assumptions": [],
                    "draft": {
                        "summary": "Waiting list system",
                        "records": [{
                            "name": "waiting_list_entry",
                            "description": "A person on a lab waiting list.",
                            "fields": [{ "name": "id", "type_name": "string", "required": true, "sensitive": false, "description": null }]
                        }],
                        "relationships": [],
                        "actions": [],
                        "events": [],
                        "projections": [],
                        "policies": [],
                        "approvals": [],
                        "migrations": [],
                        "provider_requirements": []
                    },
                    "questions": []
                })
                .to_string()
            };
            Ok(LlmResponse { content })
        }
    }

    struct PlannerQuestionLlm {
        calls: Cell<usize>,
    }

    impl LlmCapability for PlannerQuestionLlm {
        fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SorlaError> {
            assert_schema_response_format(request.response_format);
            self.calls.set(self.calls.get() + 1);
            let question = if self.calls.get() == 1 {
                serde_json::json!({
                    "id": "scope.initial",
                    "text": "Which labs need a waiting list?",
                    "help": null,
                    "answer_kind": { "kind": "free-text" },
                    "required": true,
                    "risk": "low",
                    "depends_on": []
                })
            } else {
                assert!(request.system_prompt.contains("planning step"));
                serde_json::json!({
                    "id": "scope.visibility",
                    "text": "Should the visible waiting list show exact positions?",
                    "help": null,
                    "answer_kind": { "kind": "boolean" },
                    "required": true,
                    "risk": "medium",
                    "depends_on": []
                })
            };
            Ok(LlmResponse {
                content: serde_json::json!({
                    "assistant_message": "I need one more planning detail.",
                    "assumptions": [],
                    "draft": {
                        "summary": "Waiting list system",
                        "records": [{
                            "name": "waiting_list_entry",
                            "description": "A person on a lab waiting list.",
                            "fields": [{ "name": "id", "type_name": "string", "required": true, "sensitive": false, "description": null }]
                        }],
                        "relationships": [],
                        "actions": [],
                        "events": [],
                        "projections": [],
                        "policies": [],
                        "approvals": [],
                        "migrations": [],
                        "provider_requirements": []
                    },
                    "questions": [question]
                })
                .to_string(),
            })
        }
    }

    fn config() -> PromptSessionConfig {
        PromptSessionConfig {
            locale: Some("en".to_string()),
            schema_version: Some("0.5".to_string()),
            package_name_hint: Some("landlord-tenant-sor".to_string()),
            package_version_hint: Some("0.1.0".to_string()),
            llm: super::super::LlmCapabilityConfig {
                provider: "fake".to_string(),
                model: None,
                api_key: None,
                endpoint: None,
                capability_id: None,
            },
        }
    }

    #[test]
    fn prompt_session_advances_saves_and_resumes() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let session = engine.start_session(config()).expect("session starts");
        assert_eq!(session.phase, PromptPhase::AwaitingBusinessPrompt);

        let output = engine
            .next_turn(PromptTurnInput {
                session,
                user_message: "We manage rental properties for landlords and tenants.".to_string(),
            })
            .expect("business prompt accepted");
        assert_eq!(output.session.phase, PromptPhase::AskingQuestions);
        assert_eq!(output.next_questions[0].id, "lease.multiple_tenants");

        let encoded = serde_json::to_string(&output.session).expect("session serializes");
        let resumed: PromptSessionState = serde_json::from_str(&encoded).expect("session resumes");
        assert_eq!(resumed.business_prompt, output.session.business_prompt);
    }

    #[test]
    fn follow_up_questions_adapt_to_previous_answers() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config()).unwrap(),
                user_message: "We manage rental properties for landlords and tenants.".to_string(),
            })
            .unwrap();
        let output = engine
            .next_turn(PromptTurnInput {
                session: output.session,
                user_message: "yes".to_string(),
            })
            .unwrap();

        assert_eq!(output.next_questions[0].id, "lease.liability");
    }

    #[test]
    fn landlord_tenant_answers_validate_against_wizard_schema() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let mut output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config()).unwrap(),
                user_message: "We manage rental properties for landlords and tenants.".to_string(),
            })
            .unwrap();
        for answer in [
            "yes",
            "joint",
            "yes, payments are immutable",
            "yes, suppliers do the work",
            "supplier work requires approval",
        ] {
            output = engine
                .next_turn(PromptTurnInput {
                    session: output.session,
                    user_message: answer.to_string(),
                })
                .unwrap();
        }

        let answers = engine
            .generate_answers(output.session)
            .expect("answers generate");
        crate::normalize_answers(answers.clone(), NormalizeOptions)
            .expect("generated answers validate");
        let record_names = answers["records"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| record["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        for expected in [
            "landlord",
            "tenant",
            "property",
            "lease",
            "payment",
            "maintenance_request",
            "supplier",
        ] {
            assert!(record_names.contains(&expected));
        }
        let event_names = answers["events"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert!(event_names.contains(&"lease_started"));
        assert!(event_names.contains(&"payment_recorded"));
        assert!(event_names.contains(&"maintenance_request_opened"));
        assert_eq!(answers["approvals"][0]["name"], "supplier_work_approval");
    }

    #[test]
    fn generic_prompt_answers_do_not_reference_landlord_records() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config()).unwrap(),
                user_message: "We handle customer onboarding cases.".to_string(),
            })
            .unwrap();
        let answers = engine
            .generate_answers(output.session)
            .expect("generic answers generate");
        crate::normalize_answers(answers.clone(), NormalizeOptions)
            .expect("generic answers validate");
        assert_eq!(answers["records"]["items"][0]["name"], "case");
        assert_eq!(answers["events"]["items"].as_array().unwrap().len(), 0);
        assert_eq!(answers["agent_endpoints"]["enabled"], false);
    }

    #[test]
    fn waiting_list_prompt_does_not_ask_lease_questions() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config()).unwrap(),
                user_message: "I want to create a system for waiting lists in which different greentic labs can have a separate waiting list. We also need referral codes so that if you refer many people you go up in the waiting list. We should also be able to show the waiting list. Leave the waiting list.".to_string(),
            })
            .unwrap();

        assert_eq!(output.next_questions[0].id, "waiting_list.scope");
        assert!(!output.next_questions[0].text.contains("lease"));
        let record_names = output
            .session
            .draft_model
            .as_ref()
            .unwrap()
            .records
            .iter()
            .map(|record| record.name.as_str())
            .collect::<Vec<_>>();
        assert!(record_names.contains(&"waiting_list_entry"));
        assert!(!record_names.contains(&"lease"));

        let answers = engine
            .generate_answers(output.session)
            .expect("waiting-list answers generate");
        assert_eq!(
            answers["operational_indexes"]["indexes"][0]["id"],
            "waiting_list_entry_lab_email_unique"
        );
        assert_eq!(
            answers["agent_endpoints"]["items"][0]["execution"]["steps"][1]["input"]["user_id"],
            "$generated.uuid"
        );
        assert_eq!(
            answers["agent_endpoints"]["items"][0]["execution"]["steps"][1]["input"]["invitation_code"],
            "$generated.invitation_code"
        );
        assert_eq!(
            answers["agent_endpoints"]["items"][1]["execution"]["steps"][2]["op"],
            "increment_where"
        );
        crate::normalize_answers(answers, NormalizeOptions).expect("waiting-list answers validate");
    }

    #[test]
    fn metrics_prompt_asks_adaptive_kpi_questions() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config()).unwrap(),
                user_message:
                    "I want to track clicks, revenues, costs and KPIs monthly with gross margin."
                        .to_string(),
            })
            .unwrap();

        assert_eq!(output.next_questions[0].id, "metrics.grain");
        let questions = output.session.questions;
        assert!(
            questions
                .iter()
                .any(|question| question.id == "metrics.revenue_source")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.id == "metrics.cost_source")
        );
        assert!(
            questions
                .iter()
                .any(|question| question.id == "metrics.gross_margin")
        );
        let draft = output.session.draft_model.as_ref().unwrap();
        assert!(draft.metrics.iter().any(|metric| metric.name == "clicks"));
        assert!(draft.metrics.iter().any(|metric| metric.name == "revenue"));
        assert!(draft.metrics.iter().any(|metric| metric.name == "cost"));
        assert!(
            draft
                .metrics
                .iter()
                .any(|metric| metric.name == "gross_margin")
        );
    }

    #[test]
    fn fake_prompt_generates_valid_metric_answers() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config()).unwrap(),
                user_message:
                    "Track clicks, revenue, costs, gross margin, ROAS and KPI dashboards monthly."
                        .to_string(),
            })
            .unwrap();

        let answers = engine
            .generate_answers(output.session)
            .expect("metric answers generate");
        crate::normalize_answers(answers.clone(), NormalizeOptions)
            .expect("metric answers validate");
        let metric_names = answers["metrics"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|metric| metric["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert!(metric_names.contains(&"clicks"));
        assert!(metric_names.contains(&"revenue"));
        assert!(metric_names.contains(&"cost"));
        assert!(metric_names.contains(&"gross_margin"));
        assert_eq!(
            answers["metrics"]["items"][3]["depends_on"],
            serde_json::json!(["revenue", "cost"])
        );
    }

    #[test]
    fn generate_answers_command_writes_from_current_draft() {
        let engine = DefaultPromptAuthoringEngine::new(FakePromptLlm);
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config()).unwrap(),
                user_message: "We handle customer onboarding cases.".to_string(),
            })
            .unwrap();
        let output = engine
            .next_turn(PromptTurnInput {
                session: output.session,
                user_message: "generate answers".to_string(),
            })
            .unwrap();

        assert_eq!(output.session.phase, PromptPhase::Completed);
        assert!(output.answers_document.is_some());
    }

    #[test]
    fn malformed_authoring_output_is_repaired_with_wizard_schema_context() {
        let llm = AuthoringRepairLlm {
            calls: Cell::new(0),
            system_prompts: RefCell::new(Vec::new()),
        };
        let engine = DefaultPromptAuthoringEngine::new(llm);
        let mut config = config();
        config.llm.provider = "openai".to_string();
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config).unwrap(),
                user_message: "Create waiting lists per lab.".to_string(),
            })
            .unwrap();

        assert_eq!(output.assistant_message, "I repaired the authoring draft.");
        assert_eq!(
            output.session.draft_model.as_ref().unwrap().records[0].name,
            "waiting_list_entry"
        );
        assert_eq!(engine.llm.calls.get(), 3);
        let prompts = engine.llm.system_prompts.borrow();
        assert!(prompts[0].contains("wizard --schema"));
        assert!(prompts[0].contains("records"));
        assert!(prompts[1].contains("wizard --schema"));
        assert!(prompts[2].contains("wizard --schema"));
        assert!(prompts[2].contains("planning step"));
    }

    #[test]
    fn authoring_questions_default_optional_metadata() {
        let output = parse_model_output(
            &serde_json::json!({
                "assistant_message": "I need one decision.",
                "assumptions": [],
                "draft": {
                    "summary": "Feature coverage test system",
                    "records": [],
                    "relationships": [],
                    "actions": [],
                    "events": [],
                    "projections": [],
                    "policies": [],
                    "approvals": [],
                    "migrations": [],
                    "provider_requirements": []
                },
                "questions": [{
                    "id": "scope.coverage",
                    "text": "Should the test include every supported operation?",
                    "answer_kind": { "kind": "boolean" }
                }]
            })
            .to_string(),
        )
        .expect("missing question metadata should default");

        assert!(output.questions[0].required);
        assert_eq!(output.questions[0].risk, PromptQuestionRisk::Medium);
        assert!(output.questions[0].depends_on.is_empty());
    }

    #[test]
    fn planner_can_ask_additional_scope_questions() {
        let engine = DefaultPromptAuthoringEngine::new(PlannerQuestionLlm {
            calls: Cell::new(0),
        });
        let mut config = config();
        config.llm.provider = "openai".to_string();
        let output = engine
            .next_turn(PromptTurnInput {
                session: engine.start_session(config).unwrap(),
                user_message: "Create waiting lists per lab.".to_string(),
            })
            .unwrap();
        assert_eq!(output.next_questions[0].id, "scope.initial");

        let output = engine
            .next_turn(PromptTurnInput {
                session: output.session,
                user_message: "All labs need separate lists.".to_string(),
            })
            .unwrap();

        assert_eq!(output.next_questions[0].id, "scope.visibility");
        assert_eq!(output.session.phase, PromptPhase::AskingQuestions);
        assert_eq!(engine.llm.calls.get(), 2);
    }

    #[test]
    fn model_output_accepts_string_assumptions() {
        let output = parse_model_output(
            &serde_json::json!({
                "assistant_message": "Planned.",
                "assumptions": [
                    "Each Greentic lab operates independently and maintains its own waiting list."
                ],
                "draft": {
                    "summary": "Waiting list system",
                    "records": [{
                        "name": "waiting_list_entry",
                        "description": "A waiting list entry.",
                        "fields": [{ "name": "id", "type_name": "string", "required": true, "sensitive": false, "description": null }]
                    }],
                    "relationships": [],
                    "actions": [],
                    "events": [],
                    "projections": [],
                    "policies": [],
                    "approvals": [],
                    "migrations": [],
                    "provider_requirements": []
                },
                "questions": []
            })
            .to_string(),
        )
        .unwrap();

        assert_eq!(output.assumptions[0].id, "llm-assumption-1");
        assert_eq!(
            output.assumptions[0].text,
            "Each Greentic lab operates independently and maintains its own waiting list."
        );
    }

    #[test]
    fn model_output_accepts_loose_assumption_objects() {
        let output = parse_model_output(
            &serde_json::json!({
                "assistant_message": "Planned.",
                "assumptions": [
                    {
                        "name": "coverage",
                        "assumption": "Made-up fields are acceptable for feature coverage.",
                        "certainty": "very likely"
                    },
                    {
                        "description": "All available features should be exercised.",
                        "risk": "high"
                    }
                ],
                "draft": {
                    "summary": "Feature coverage system",
                    "records": [],
                    "relationships": [],
                    "actions": [],
                    "events": [],
                    "projections": [],
                    "policies": [],
                    "approvals": [],
                    "migrations": [],
                    "provider_requirements": []
                },
                "questions": []
            })
            .to_string(),
        )
        .expect("loose assumption objects should normalize");

        assert_eq!(output.assumptions[0].id, "coverage");
        assert_eq!(
            output.assumptions[0].text,
            "Made-up fields are acceptable for feature coverage."
        );
        assert_eq!(
            output.assumptions[0].confidence,
            PromptAssumptionConfidence::Medium
        );
        assert_eq!(output.assumptions[1].id, "llm-assumption-2");
        assert_eq!(
            output.assumptions[1].confidence,
            PromptAssumptionConfidence::High
        );
    }

    #[test]
    fn model_output_defaults_missing_draft_summary() {
        let output = parse_model_output(
            &serde_json::json!({
                "assistant_message": "Planned.",
                "assumptions": [],
                "draft": {
                    "records": [{
                        "name": "user",
                        "fields": [{ "name": "email", "type": "string" }]
                    }]
                },
                "questions": []
            })
            .to_string(),
        )
        .unwrap();

        assert_eq!(output.draft.summary, "");
        assert_eq!(output.draft.records[0].fields[0].type_name, "string");
        assert!(output.draft.records[0].fields[0].required);
    }

    #[test]
    fn llm_prompts_explain_answers_json_objective() {
        let schema = "{}";
        for prompt in [
            prompt_authoring_system_prompt(schema),
            planner_system_prompt(schema),
            answer_generation_system_prompt(schema),
            answer_repair_system_prompt(schema),
        ] {
            assert!(prompt.contains("answers.json"));
            assert!(prompt.contains("greentic-sorla wizard"));
            assert!(prompt.contains("System of Record"));
        }
        assert!(answer_generation_system_prompt(schema).contains("high-quality answers.json"));
        assert!(answer_generation_system_prompt(schema).contains("records"));
        assert!(answer_generation_system_prompt(schema).contains("projections"));
    }

    #[test]
    fn answer_response_schema_is_openai_json_schema_object() {
        let LlmResponseFormat::JsonSchema { schema, .. } = answer_response_format() else {
            panic!("answers should request a JSON schema response format");
        };
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["records"].is_object());
        assert!(schema["properties"]["metrics"].is_object());
        assert!(schema["properties"]["operational_indexes"].is_object());
        assert_eq!(
            schema["properties"]["agent_endpoints"]["additionalProperties"],
            true
        );
        assert_eq!(
            schema["properties"]["metrics"]["properties"]["items"]["items"]["properties"]["measure"]
                ["properties"]["aggregate"]["enum"][0],
            "count"
        );
        assert!(
            schema["properties"]["records"]["properties"]
                .as_object()
                .unwrap()
                .contains_key("external_ref_system")
        );
        assert_eq!(
            schema["properties"]["records"]["properties"]["items"]["items"]["properties"]["fields"]
                ["items"]["properties"]["authority"]["type"],
            "string"
        );
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("records"))
        );
    }

    #[test]
    fn invalid_answers_are_repaired_with_llm_before_returning() {
        let engine = DefaultPromptAuthoringEngine::new(RepairPromptLlm {
            calls: Cell::new(0),
        });
        let mut session = engine.start_session(config()).unwrap();
        session.business_prompt = Some("Repair a bad draft".to_string());
        session.llm = Some(LlmCapabilityConfig {
            provider: "openai".to_string(),
            model: None,
            api_key: None,
            endpoint: None,
            capability_id: None,
        });
        session.draft_model = Some(SorDesignDraft {
            summary: "Invalid draft".to_string(),
            records: Vec::new(),
            events: vec![DraftEvent {
                name: "orphan_event".to_string(),
                description: None,
            }],
            projections: vec![DraftProjection {
                name: "orphan_projection".to_string(),
                description: None,
            }],
            ..SorDesignDraft::default()
        });

        let answers = engine.generate_answers(session).expect("repair succeeds");
        crate::normalize_answers(answers.clone(), NormalizeOptions)
            .expect("repaired answers normalize");
        assert_eq!(answers["package"]["name"], "repaired-sor");
    }
}
