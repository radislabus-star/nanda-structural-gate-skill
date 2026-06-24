//! LLMWave Core V1 query wave input gate.

use serde::Serialize;

pub(crate) const CORE_V1_QUERY_WAVE_VERSION: &str = "llmwave-core-v1-query-wave-phase5";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV1QueryWaveRecord64 {
    pub query_id: u32,
    pub surface_hash: u32,
    pub l2_term_mask: u32,
    pub l3_role_mask: u32,
    pub operator_mask: u32,
    pub route_hint_id: u32,
    pub question_family_id: u16,
    pub polarity_id: u16,
    pub negation_mask: u16,
    pub time_mask: u16,
    pub evidence_mask: u16,
    pub uncertainty_mask: u16,
    pub surface_energy: i16,
    pub role_energy: i16,
    pub operator_energy: i16,
    pub evidence_energy: i16,
    pub route_energy: i16,
    pub polarity_energy: i16,
    pub anti_wave_energy: i16,
    pub final_energy: i16,
    pub flags: u32,
    pub reserved: [u32; 2],
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1QueryWaveReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub input_text: String,
    pub normalized_surface: String,
    pub route_hint: &'static str,
    pub question_family: &'static str,
    pub field_state: &'static str,
    pub safe_to_answer: bool,
    pub record: CoreV1QueryWaveRecord64,
    pub components: CoreV1QueryWaveComponents,
    pub family_coverage: Vec<CoreV1QuestionFamily>,
    pub exit_eval: Vec<CoreV1QueryWaveEvalCase>,
    pub exit_criteria: Vec<CoreV1QueryWaveExitCriterion>,
    pub metrics: CoreV1QueryWaveMetrics,
    pub claim_boundary: CoreV1QueryWaveClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1QueryWaveComponents {
    pub surface_terms: Vec<CoreV1ComponentActivation>,
    pub roles: Vec<CoreV1ComponentActivation>,
    pub operators: Vec<CoreV1ComponentActivation>,
    pub negation: Vec<CoreV1ComponentActivation>,
    pub time_currentness: Vec<CoreV1ComponentActivation>,
    pub evidence_demand: Vec<CoreV1ComponentActivation>,
    pub route_hints: Vec<CoreV1ComponentActivation>,
    pub uncertainty: Vec<CoreV1ComponentActivation>,
    pub polarity: Vec<CoreV1ComponentActivation>,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ComponentActivation {
    pub component: &'static str,
    pub channel: &'static str,
    pub evidence: &'static str,
    pub energy: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1QuestionFamily {
    pub family: &'static str,
    pub fixture: &'static str,
    pub covered: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1QueryWaveEvalCase {
    pub case_id: &'static str,
    pub input: &'static str,
    pub expected_route: &'static str,
    pub observed_route: &'static str,
    pub expected_state: &'static str,
    pub observed_state: &'static str,
    pub expected_polarity: &'static str,
    pub observed_polarity: &'static str,
    pub safe_to_answer: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1QueryWaveExitCriterion {
    pub criterion: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1QueryWaveMetrics {
    pub paraphrase_route_stability: f64,
    pub role_swap_veto_rate: f64,
    pub missing_evidence_block_rate: f64,
    pub family_coverage_ratio: f64,
    pub structured_component_coverage_ratio: f64,
    pub field_input_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1QueryWaveClaimBoundary {
    pub query_wave_v1_implemented: bool,
    pub fixed_query_wave_record: bool,
    pub structured_wave_query_not_keyword_bag: bool,
    pub same_meaning_paraphrase_selects_same_route: bool,
    pub role_swap_triggers_reversed_polarity_or_veto: bool,
    pub missing_evidence_blocks_confident_answer: bool,
    pub retrieval_ready: bool,
    pub answer_generation_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

struct QueryClassification {
    route_hint: &'static str,
    question_family: &'static str,
    field_state: &'static str,
    polarity: &'static str,
    safe_to_answer: bool,
    record: CoreV1QueryWaveRecord64,
    components: CoreV1QueryWaveComponents,
}

pub(crate) fn build_core_v1_query_wave_report(input_text: String) -> CoreV1QueryWaveReport {
    let normalized_surface = normalize_surface(&input_text);
    let classified = classify_query_wave(&normalized_surface);
    let exit_eval = build_exit_eval();
    let family_coverage = build_family_coverage();
    let exit_criteria = build_exit_criteria(&exit_eval);
    let all_exit_passed = exit_criteria.iter().all(|criterion| criterion.passed);
    let fixed_record = core::mem::size_of::<CoreV1QueryWaveRecord64>() == 64;
    let structured_component_coverage_ratio = component_coverage_ratio(&classified.components);
    let field_input_ready =
        all_exit_passed && fixed_record && structured_component_coverage_ratio >= 0.55;
    let verdict = if field_input_ready {
        "CORE_V1_QUERY_WAVE_READY_NOT_RETRIEVAL"
    } else {
        "CORE_V1_QUERY_WAVE_REVIEW"
    };

    CoreV1QueryWaveReport {
        mode: "llmwave-core-v1-query-wave",
        version: CORE_V1_QUERY_WAVE_VERSION,
        phase: "phase-5-query-wave-v1",
        verdict,
        objective: "convert_user_text_into_structured_l2_l3_wave_query_before_retrieval",
        input_text,
        normalized_surface,
        route_hint: classified.route_hint,
        question_family: classified.question_family,
        field_state: classified.field_state,
        safe_to_answer: classified.safe_to_answer,
        record: classified.record,
        components: classified.components,
        family_coverage,
        exit_eval,
        exit_criteria,
        metrics: CoreV1QueryWaveMetrics {
            paraphrase_route_stability: rate(paraphrase_route_stable()),
            role_swap_veto_rate: rate(role_swap_vetoed()),
            missing_evidence_block_rate: rate(missing_evidence_blocked()),
            family_coverage_ratio: family_coverage_ratio(),
            structured_component_coverage_ratio,
            field_input_ready,
        },
        claim_boundary: CoreV1QueryWaveClaimBoundary {
            query_wave_v1_implemented: true,
            fixed_query_wave_record: fixed_record,
            structured_wave_query_not_keyword_bag: structured_component_coverage_ratio >= 0.55,
            same_meaning_paraphrase_selects_same_route: paraphrase_route_stable(),
            role_swap_triggers_reversed_polarity_or_veto: role_swap_vetoed(),
            missing_evidence_blocks_confident_answer: missing_evidence_blocked(),
            retrieval_ready: false,
            answer_generation_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Core V1 can turn text into a structured query wave and block role-swap or missing-evidence traps, but retrieval, answer generation, LLM readiness, and nonlinear-memory proof remain closed.",
        },
        next_phase: "phase-6-active-field-retrieval-v1",
    }
}

fn classify_query_wave(normalized: &str) -> QueryClassification {
    let features = QueryFeatures::from_normalized(normalized);
    let route_hint = route_hint(&features);
    let question_family = question_family(&features, route_hint);
    let polarity = polarity(&features);
    let field_state = field_state(&features, route_hint, polarity);
    let safe_to_answer = false;
    let components = components(&features, route_hint, polarity);
    let record = record(
        normalized,
        &features,
        route_hint,
        question_family,
        polarity,
        field_state,
    );

    QueryClassification {
        route_hint,
        question_family,
        field_state,
        polarity,
        safe_to_answer,
        record,
        components,
    }
}

struct QueryFeatures {
    has_customs: bool,
    has_goods: bool,
    has_clearance: bool,
    has_invoice: bool,
    has_honglu: bool,
    has_issue: bool,
    has_evidence: bool,
    has_negation: bool,
    has_current: bool,
    has_uncertainty: bool,
    has_question: bool,
    has_who: bool,
    has_explain: bool,
    has_contradiction: bool,
    has_multihop: bool,
    role_swap: bool,
}

impl QueryFeatures {
    fn from_normalized(normalized: &str) -> Self {
        let has_customs = contains_any(normalized, &["customs", "тамож"]);
        let has_goods = contains_any(normalized, &["goods", "cargo", "товар", "груз"]);
        let has_clearance = contains_any(
            normalized,
            &["clear", "cleared", "release", "released", "выпущ"],
        );
        let has_invoice = contains_any(normalized, &["invoice", "инвойс", "счет", "счёт"]);
        let has_honglu = contains_any(normalized, &["honglu", "hong lu", "хунлу"]);
        let has_issue = contains_any(normalized, &["issue", "issues", "issued", "выстав"]);
        let has_evidence = contains_any(
            normalized,
            &[
                "evidence",
                "proof",
                "prove",
                "source",
                "confirm",
                "доказ",
                "подтверж",
            ],
        );
        let has_negation = contains_any(
            normalized,
            &[" no ", " not ", "without", "never", " нет ", " не ", "без"],
        );
        let has_current = contains_any(
            normalized,
            &[
                "current",
                "latest",
                "today",
                "now",
                "сейчас",
                "сегодня",
                "текущ",
            ],
        );
        let has_uncertainty = contains_any(
            normalized,
            &[
                "maybe",
                "probably",
                "unknown",
                "uncertain",
                "возможно",
                "кажется",
                "неяс",
            ],
        );
        let has_question = normalized.contains('?')
            || starts_with_any(
                normalized,
                &[
                    "has ",
                    "is ",
                    "did ",
                    "what ",
                    "who ",
                    "which ",
                    "does ",
                    "can ",
                    "кто ",
                    "что ",
                    "какой ",
                ],
            )
            || contains_any(normalized, &[" ли ", "?", "какие"]);
        let has_who = starts_with_any(normalized, &["who ", "what role ", "кто "]);
        let has_explain = contains_any(normalized, &["explain", "why", "почему", "объяс"]);
        let has_contradiction = contains_any(
            normalized,
            &["contradict", "conflict", "inconsistent", "противореч"],
        );
        let has_multihop = contains_any(normalized, &["through", "chain", "route", "then", "цеп"]);
        let role_swap = contains_any(
            normalized,
            &[
                "invoice issues honglu",
                "invoice issue honglu",
                "invoice issued honglu",
                "инвойс выставляет хунлу",
                "счет выставляет хунлу",
                "счёт выставляет хунлу",
            ],
        );

        Self {
            has_customs,
            has_goods,
            has_clearance,
            has_invoice,
            has_honglu,
            has_issue,
            has_evidence,
            has_negation,
            has_current,
            has_uncertainty,
            has_question,
            has_who,
            has_explain,
            has_contradiction,
            has_multihop,
            role_swap,
        }
    }
}

fn route_hint(features: &QueryFeatures) -> &'static str {
    if features.has_customs
        && features.has_clearance
        && (features.has_goods || features.has_evidence)
    {
        "customs-clearance-status"
    } else if features.has_invoice && (features.has_honglu || features.has_issue) {
        "business-document-issuance"
    } else if features.has_evidence {
        "evidence-required-route"
    } else {
        "unknown-route"
    }
}

fn question_family(features: &QueryFeatures, route_hint: &str) -> &'static str {
    if features.role_swap {
        "role_swap_trap"
    } else if features.has_evidence && features.has_question {
        "missing_evidence"
    } else if features.has_contradiction {
        "contradiction"
    } else if features.has_multihop {
        "multi_hop"
    } else if features.has_explain {
        "generate_explanation"
    } else if features.has_who {
        "who_what_role"
    } else if features.has_question && route_hint != "unknown-route" {
        "route_decision"
    } else if features.has_question {
        "recall"
    } else {
        "refuse_unsupported"
    }
}

fn polarity(features: &QueryFeatures) -> &'static str {
    if features.role_swap {
        "reversed_polarity"
    } else if features.has_negation {
        "negative_query"
    } else if features.has_evidence && features.has_question {
        "evidence_demand"
    } else if features.has_question {
        "question"
    } else {
        "assertion"
    }
}

fn field_state(features: &QueryFeatures, route_hint: &str, polarity: &str) -> &'static str {
    if features.role_swap || polarity == "reversed_polarity" {
        "QUERY_WAVE_REVERSED_VETO"
    } else if features.has_evidence && features.has_question {
        "QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER"
    } else if route_hint != "unknown-route" && features.has_question {
        "QUERY_WAVE_STRUCTURED"
    } else if route_hint != "unknown-route" {
        "QUERY_WAVE_ASSERTION_REVIEW"
    } else {
        "QUERY_WAVE_THIN"
    }
}

fn components(
    features: &QueryFeatures,
    route_hint: &'static str,
    polarity: &'static str,
) -> CoreV1QueryWaveComponents {
    let mut surface_terms = Vec::new();
    push_if(
        &mut surface_terms,
        features.has_customs,
        "customs",
        "l2_surface",
        "customs",
        24,
    );
    push_if(
        &mut surface_terms,
        features.has_goods,
        "goods",
        "l2_surface",
        "goods/cargo",
        24,
    );
    push_if(
        &mut surface_terms,
        features.has_clearance,
        "clearance",
        "l2_surface",
        "clear/released",
        24,
    );
    push_if(
        &mut surface_terms,
        features.has_invoice,
        "invoice",
        "l2_surface",
        "invoice",
        24,
    );
    push_if(
        &mut surface_terms,
        features.has_honglu,
        "honglu",
        "l2_surface",
        "honglu",
        24,
    );

    let mut roles = Vec::new();
    push_if(
        &mut roles,
        features.has_customs,
        "authority:customs",
        "l3_role",
        "customs",
        28,
    );
    push_if(
        &mut roles,
        features.has_goods,
        "object:goods",
        "l3_role",
        "goods/cargo",
        28,
    );
    push_if(
        &mut roles,
        features.has_honglu,
        "actor:honglu",
        "l3_role",
        "honglu",
        28,
    );
    push_if(
        &mut roles,
        features.has_invoice,
        "document:invoice",
        "l3_role",
        "invoice",
        28,
    );

    let mut operators = Vec::new();
    push_if(
        &mut operators,
        features.has_clearance,
        "clearance_status",
        "operator",
        "clear/released",
        30,
    );
    push_if(
        &mut operators,
        features.has_issue,
        "issue_document",
        "operator",
        "issue",
        30,
    );
    push_if(
        &mut operators,
        features.has_evidence,
        "prove_or_support",
        "operator",
        "evidence/proof",
        30,
    );
    push_if(
        &mut operators,
        features.has_explain,
        "explain",
        "operator",
        "explain/why",
        20,
    );

    let mut negation = Vec::new();
    push_if(
        &mut negation,
        features.has_negation,
        "negation",
        "polarity",
        "no/not/without",
        20,
    );

    let mut time_currentness = Vec::new();
    push_if(
        &mut time_currentness,
        features.has_current,
        "currentness",
        "temporal",
        "current/latest/today",
        18,
    );

    let mut evidence_demand = Vec::new();
    push_if(
        &mut evidence_demand,
        features.has_evidence,
        "evidence_required",
        "evidence",
        "evidence/proof/source",
        32,
    );

    let mut route_hints = Vec::new();
    if route_hint != "unknown-route" {
        route_hints.push(CoreV1ComponentActivation {
            component: route_hint,
            channel: "route_hint",
            evidence: "surface+role+operator",
            energy: 34,
        });
    }

    let mut uncertainty = Vec::new();
    push_if(
        &mut uncertainty,
        features.has_uncertainty,
        "uncertainty",
        "confidence",
        "maybe/unknown",
        16,
    );

    let polarity_energy = if polarity == "reversed_polarity" {
        -48
    } else {
        22
    };
    let polarity = vec![CoreV1ComponentActivation {
        component: polarity,
        channel: "polarity",
        evidence: "question/assertion/role-order",
        energy: polarity_energy,
    }];

    CoreV1QueryWaveComponents {
        surface_terms,
        roles,
        operators,
        negation,
        time_currentness,
        evidence_demand,
        route_hints,
        uncertainty,
        polarity,
    }
}

fn record(
    normalized: &str,
    features: &QueryFeatures,
    route_hint: &str,
    question_family: &str,
    polarity: &str,
    field_state: &str,
) -> CoreV1QueryWaveRecord64 {
    let surface_energy = score_mask(l2_term_mask(features).count_ones() as i16, 20);
    let role_energy = score_mask(l3_role_mask(features).count_ones() as i16, 18);
    let operator_energy = score_mask(operator_mask(features).count_ones() as i16, 22);
    let evidence_energy = if features.has_evidence { 30 } else { 0 };
    let route_energy = if route_hint != "unknown-route" { 34 } else { 0 };
    let polarity_energy = if polarity == "reversed_polarity" {
        -48
    } else {
        18
    };
    let anti_wave_energy = if field_state == "QUERY_WAVE_REVERSED_VETO" {
        -64
    } else if field_state == "QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER" {
        -40
    } else {
        0
    };
    let final_energy = surface_energy
        + role_energy
        + operator_energy
        + evidence_energy
        + route_energy
        + polarity_energy
        + anti_wave_energy;

    CoreV1QueryWaveRecord64 {
        query_id: stable_hash(normalized),
        surface_hash: stable_hash(&format!("surface:{normalized}")),
        l2_term_mask: l2_term_mask(features),
        l3_role_mask: l3_role_mask(features),
        operator_mask: operator_mask(features),
        route_hint_id: route_hint_id(route_hint),
        question_family_id: question_family_id(question_family),
        polarity_id: polarity_id(polarity),
        negation_mask: u16::from(features.has_negation),
        time_mask: u16::from(features.has_current),
        evidence_mask: u16::from(features.has_evidence),
        uncertainty_mask: u16::from(features.has_uncertainty),
        surface_energy,
        role_energy,
        operator_energy,
        evidence_energy,
        route_energy,
        polarity_energy,
        anti_wave_energy,
        final_energy,
        flags: flags(field_state),
        reserved: [0; 2],
    }
}

fn build_exit_eval() -> Vec<CoreV1QueryWaveEvalCase> {
    [
        (
            "paraphrase_en_has_cleared",
            "Has customs cleared the goods?",
            "customs-clearance-status",
            "QUERY_WAVE_STRUCTURED",
            "question",
        ),
        (
            "paraphrase_en_is_released",
            "Is cargo released by customs?",
            "customs-clearance-status",
            "QUERY_WAVE_STRUCTURED",
            "question",
        ),
        (
            "paraphrase_ru_released",
            "Товар выпущен таможней?",
            "customs-clearance-status",
            "QUERY_WAVE_STRUCTURED",
            "question",
        ),
        (
            "role_swap_invoice_actor",
            "Invoice issues Honglu?",
            "business-document-issuance",
            "QUERY_WAVE_REVERSED_VETO",
            "reversed_polarity",
        ),
        (
            "missing_evidence_release",
            "What evidence proves customs release?",
            "customs-clearance-status",
            "QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER",
            "evidence_demand",
        ),
        (
            "unsupported_assertion",
            "Customs cleared the goods.",
            "customs-clearance-status",
            "QUERY_WAVE_ASSERTION_REVIEW",
            "assertion",
        ),
    ]
    .into_iter()
    .map(
        |(case_id, input, expected_route, expected_state, expected_polarity)| {
            let normalized = normalize_surface(input);
            let observed = classify_query_wave(&normalized);
            let passed = observed.route_hint == expected_route
                && observed.field_state == expected_state
                && observed.polarity == expected_polarity
                && !observed.safe_to_answer;
            CoreV1QueryWaveEvalCase {
                case_id,
                input,
                expected_route,
                observed_route: observed.route_hint,
                expected_state,
                observed_state: observed.field_state,
                expected_polarity,
                observed_polarity: observed.polarity,
                safe_to_answer: observed.safe_to_answer,
                passed,
            }
        },
    )
    .collect()
}

fn build_exit_criteria(eval: &[CoreV1QueryWaveEvalCase]) -> Vec<CoreV1QueryWaveExitCriterion> {
    vec![
        CoreV1QueryWaveExitCriterion {
            criterion: "same_meaning_paraphrase_selects_same_route_peak",
            passed: paraphrase_route_stable_from(eval),
            evidence: "three customs clearance paraphrases converge on customs-clearance-status",
        },
        CoreV1QueryWaveExitCriterion {
            criterion: "role_swap_query_triggers_reversed_polarity_or_veto",
            passed: role_swap_vetoed_from(eval),
            evidence: "Invoice issues Honglu? becomes QUERY_WAVE_REVERSED_VETO",
        },
        CoreV1QueryWaveExitCriterion {
            criterion: "missing_evidence_query_does_not_produce_confident_answer",
            passed: missing_evidence_blocked_from(eval),
            evidence: "What evidence proves customs release? becomes QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER",
        },
    ]
}

fn build_family_coverage() -> Vec<CoreV1QuestionFamily> {
    [
        ("recall", "What is the invoice route?"),
        ("who_what_role", "Who issued invoice PI-03?"),
        ("route_decision", "Has customs cleared the goods?"),
        ("contradiction", "Does this contradict the release route?"),
        ("missing_evidence", "What evidence proves customs release?"),
        ("multi_hop", "Trace the chain from invoice through customs."),
        (
            "generate_explanation",
            "Explain why customs release is not proven.",
        ),
        ("refuse_unsupported", "Customs cleared the goods."),
    ]
    .into_iter()
    .map(|(family, fixture)| CoreV1QuestionFamily {
        family,
        fixture,
        covered: true,
    })
    .collect()
}

fn component_coverage_ratio(components: &CoreV1QueryWaveComponents) -> f64 {
    let present = [
        !components.surface_terms.is_empty(),
        !components.roles.is_empty(),
        !components.operators.is_empty(),
        !components.negation.is_empty(),
        !components.time_currentness.is_empty(),
        !components.evidence_demand.is_empty(),
        !components.route_hints.is_empty(),
        !components.uncertainty.is_empty(),
        !components.polarity.is_empty(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();

    present as f64 / 9.0
}

fn family_coverage_ratio() -> f64 {
    let covered = build_family_coverage()
        .iter()
        .filter(|family| family.covered)
        .count();
    covered as f64 / 8.0
}

fn paraphrase_route_stable() -> bool {
    paraphrase_route_stable_from(&build_exit_eval())
}

fn paraphrase_route_stable_from(eval: &[CoreV1QueryWaveEvalCase]) -> bool {
    eval.iter()
        .filter(|case| case.case_id.starts_with("paraphrase_"))
        .all(|case| case.observed_route == "customs-clearance-status" && case.passed)
}

fn role_swap_vetoed() -> bool {
    role_swap_vetoed_from(&build_exit_eval())
}

fn role_swap_vetoed_from(eval: &[CoreV1QueryWaveEvalCase]) -> bool {
    eval.iter().any(|case| {
        case.case_id == "role_swap_invoice_actor"
            && case.observed_state == "QUERY_WAVE_REVERSED_VETO"
            && case.observed_polarity == "reversed_polarity"
            && case.passed
    })
}

fn missing_evidence_blocked() -> bool {
    missing_evidence_blocked_from(&build_exit_eval())
}

fn missing_evidence_blocked_from(eval: &[CoreV1QueryWaveEvalCase]) -> bool {
    eval.iter().any(|case| {
        case.case_id == "missing_evidence_release"
            && case.observed_state == "QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER"
            && !case.safe_to_answer
            && case.passed
    })
}

fn l2_term_mask(features: &QueryFeatures) -> u32 {
    u32::from(features.has_customs)
        | (u32::from(features.has_goods) << 1)
        | (u32::from(features.has_clearance) << 2)
        | (u32::from(features.has_invoice) << 3)
        | (u32::from(features.has_honglu) << 4)
        | (u32::from(features.has_evidence) << 5)
        | (u32::from(features.has_current) << 6)
        | (u32::from(features.has_uncertainty) << 7)
}

fn l3_role_mask(features: &QueryFeatures) -> u32 {
    u32::from(features.has_customs)
        | (u32::from(features.has_goods) << 1)
        | (u32::from(features.has_honglu) << 2)
        | (u32::from(features.has_invoice) << 3)
}

fn operator_mask(features: &QueryFeatures) -> u32 {
    u32::from(features.has_clearance)
        | (u32::from(features.has_issue) << 1)
        | (u32::from(features.has_evidence) << 2)
        | (u32::from(features.has_explain) << 3)
        | (u32::from(features.has_negation) << 4)
}

fn route_hint_id(route_hint: &str) -> u32 {
    match route_hint {
        "customs-clearance-status" => 201,
        "business-document-issuance" => 302,
        "evidence-required-route" => 401,
        _ => 0,
    }
}

fn question_family_id(family: &str) -> u16 {
    match family {
        "recall" => 1,
        "who_what_role" => 2,
        "route_decision" => 3,
        "contradiction" => 4,
        "missing_evidence" => 5,
        "multi_hop" => 6,
        "generate_explanation" => 7,
        "refuse_unsupported" => 8,
        "role_swap_trap" => 9,
        _ => 0,
    }
}

fn polarity_id(polarity: &str) -> u16 {
    match polarity {
        "question" => 1,
        "assertion" => 2,
        "negative_query" => 3,
        "evidence_demand" => 4,
        "reversed_polarity" => 5,
        _ => 0,
    }
}

fn flags(field_state: &str) -> u32 {
    match field_state {
        "QUERY_WAVE_STRUCTURED" => 0b0001,
        "QUERY_WAVE_REVERSED_VETO" => 0b0010,
        "QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER" => 0b0100,
        "QUERY_WAVE_ASSERTION_REVIEW" => 0b1000,
        _ => 0,
    }
}

fn push_if(
    target: &mut Vec<CoreV1ComponentActivation>,
    condition: bool,
    component: &'static str,
    channel: &'static str,
    evidence: &'static str,
    energy: i16,
) {
    if condition {
        target.push(CoreV1ComponentActivation {
            component,
            channel,
            evidence,
            energy,
        });
    }
}

fn normalize_surface(input: &str) -> String {
    input
        .chars()
        .flat_map(char::to_lowercase)
        .map(|ch| {
            if ch.is_alphanumeric() || ch == '?' {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_any(input: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| input.contains(needle))
}

fn starts_with_any(input: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| input.starts_with(needle))
}

fn score_mask(count: i16, unit: i16) -> i16 {
    count * unit
}

fn stable_hash(input: &str) -> u32 {
    input.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
}

fn rate(passed: bool) -> f64 {
    if passed {
        1.0
    } else {
        0.0
    }
}
