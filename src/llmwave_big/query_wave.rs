//! Query wave excitation for the mature field path.

use serde::Serialize;

pub(crate) const QUERY_WAVE_VERSION: &str = "llmwave-big-v1000-query-wave";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct QueryWaveRecord32 {
    pub query_id: u32,
    pub l2_surface_hash: u32,
    pub l3_schema_hint_id: u32,
    pub role_mask: u16,
    pub operator_mask: u16,
    pub polarity_id: u16,
    pub question_kind_id: u16,
    pub surface_energy: i16,
    pub role_energy: i16,
    pub final_energy: i16,
    pub flags: u16,
    pub reserved: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct QueryWaveReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub normalized_surface: String,
    pub top_route_hint: &'static str,
    pub question_polarity: &'static str,
    pub record: QueryWaveRecord32,
    pub l2_surface_excitations: Vec<SurfaceExcitation>,
    pub role_excitations: Vec<RoleExcitation>,
    pub operator_excitations: Vec<OperatorExcitation>,
    pub paraphrase_eval: Vec<QueryWaveEvalCase>,
    pub metrics: QueryWaveMetrics,
    pub claim_boundary: QueryWaveClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceExcitation {
    pub token: &'static str,
    pub layer: &'static str,
    pub energy: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct RoleExcitation {
    pub role: &'static str,
    pub evidence_token: &'static str,
    pub energy: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct OperatorExcitation {
    pub operator: &'static str,
    pub evidence_token: &'static str,
    pub energy: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct QueryWaveEvalCase {
    pub case_id: &'static str,
    pub input: &'static str,
    pub expected_route: &'static str,
    pub observed_route: &'static str,
    pub expected_state: &'static str,
    pub observed_state: &'static str,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct QueryWaveMetrics {
    pub paraphrase_route_recall: f32,
    pub role_hint_accuracy: f32,
    pub operator_hint_accuracy: f32,
    pub question_polarity_accuracy: f32,
    pub assertion_reject_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct QueryWaveClaimBoundary {
    pub query_wave_implemented: bool,
    pub fixed_query_wave_records: bool,
    pub text_to_field_excitation_ready: bool,
    pub full_field_mature: bool,
    pub chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_query_wave_report(input_text: String) -> QueryWaveReport {
    let normalized_surface = normalize_surface(&input_text);
    let classified = classify_query(&normalized_surface);
    let paraphrase_eval = build_paraphrase_eval();
    let passed_cases = paraphrase_eval.iter().filter(|case| case.passed).count();
    let all_passed = passed_cases == paraphrase_eval.len()
        && classified.route == "customs-clearance-status"
        && classified.state == "QUERY_WAVE_FOCUSED";
    let verdict = if all_passed {
        "QUERY_WAVE_READY_NOT_FIELD_MATURE"
    } else {
        "QUERY_WAVE_REVIEW"
    };

    QueryWaveReport {
        mode: "llmwave-big-query-wave",
        version: QUERY_WAVE_VERSION,
        roadmap_block: "v951-v1000",
        verdict,
        input_text,
        normalized_surface,
        top_route_hint: classified.route,
        question_polarity: classified.polarity,
        record: classified.record,
        l2_surface_excitations: classified.surface,
        role_excitations: classified.roles,
        operator_excitations: classified.operators,
        paraphrase_eval,
        metrics: QueryWaveMetrics {
            paraphrase_route_recall: rate(all_eval_routes_pass()),
            role_hint_accuracy: rate(all_eval_roles_pass()),
            operator_hint_accuracy: rate(all_eval_operators_pass()),
            question_polarity_accuracy: rate(all_eval_polarity_pass()),
            assertion_reject_rate: rate(assertion_trap_rejected()),
            state: verdict,
        },
        claim_boundary: QueryWaveClaimBoundary {
            query_wave_implemented: true,
            fixed_query_wave_records: core::mem::size_of::<QueryWaveRecord32>() == 32,
            text_to_field_excitation_ready: all_passed,
            full_field_mature: false,
            chat_ready: false,
            external_corpus_loaded: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Input text can excite a compact query wave with role/operator/polarity hints, but mature field selection and chat remain unproven",
        },
    }
}

struct ClassifiedQuery {
    route: &'static str,
    polarity: &'static str,
    state: &'static str,
    record: QueryWaveRecord32,
    surface: Vec<SurfaceExcitation>,
    roles: Vec<RoleExcitation>,
    operators: Vec<OperatorExcitation>,
}

fn classify_query(normalized: &str) -> ClassifiedQuery {
    let has_customs = contains_any(normalized, &["customs", "тамож"]);
    let has_clear = contains_any(normalized, &["clear", "cleared", "выпущ"]);
    let has_goods = contains_any(normalized, &["goods", "cargo", "товар", "груз"]);
    let has_question = contains_any(normalized, &["?", "has ", "is ", "did ", "ли "]);
    let has_invoice = contains_any(normalized, &["invoice", "pi-03", "счет"]);

    let route = if has_customs && has_clear && has_goods {
        "customs-clearance-status"
    } else if has_invoice {
        "business-document-status"
    } else {
        "unknown-route"
    };
    let polarity = if has_question {
        "question_status"
    } else {
        "assertion_claim"
    };
    let state = if route == "customs-clearance-status" && polarity == "question_status" {
        "QUERY_WAVE_FOCUSED"
    } else if polarity == "assertion_claim" {
        "QUERY_WAVE_ASSERTION_TRAP"
    } else {
        "QUERY_WAVE_THIN"
    };

    let surface_energy = score_bool(has_customs) + score_bool(has_clear) + score_bool(has_goods);
    let role_energy = score_bool(has_customs) + score_bool(has_goods);
    let operator_energy = score_bool(has_clear);
    let final_energy = surface_energy + role_energy + operator_energy;

    ClassifiedQuery {
        route,
        polarity,
        state,
        record: QueryWaveRecord32 {
            query_id: stable_hash(normalized),
            l2_surface_hash: stable_hash(&format!("l2:{normalized}")),
            l3_schema_hint_id: if route == "customs-clearance-status" {
                203
            } else {
                0
            },
            role_mask: role_mask(has_customs, has_goods),
            operator_mask: if has_clear { 0b0001 } else { 0 },
            polarity_id: if polarity == "question_status" { 1 } else { 2 },
            question_kind_id: if has_question { 11 } else { 0 },
            surface_energy,
            role_energy,
            final_energy,
            flags: if state == "QUERY_WAVE_FOCUSED" { 1 } else { 0 },
            reserved: 0,
        },
        surface: vec![
            SurfaceExcitation {
                token: "customs",
                layer: "l2_surface",
                energy: score_bool(has_customs),
            },
            SurfaceExcitation {
                token: "cleared",
                layer: "l2_surface",
                energy: score_bool(has_clear),
            },
            SurfaceExcitation {
                token: "goods",
                layer: "l2_surface",
                energy: score_bool(has_goods),
            },
        ],
        roles: vec![
            RoleExcitation {
                role: "actor:customs",
                evidence_token: "customs",
                energy: score_bool(has_customs),
            },
            RoleExcitation {
                role: "object:goods",
                evidence_token: "goods",
                energy: score_bool(has_goods),
            },
        ],
        operators: vec![OperatorExcitation {
            operator: "clearance_status",
            evidence_token: "cleared",
            energy: score_bool(has_clear),
        }],
    }
}

fn build_paraphrase_eval() -> Vec<QueryWaveEvalCase> {
    [
        (
            "en_has_cleared",
            "Has customs cleared the goods?",
            "QUERY_WAVE_FOCUSED",
        ),
        (
            "en_is_cleared",
            "Is the cargo cleared by customs?",
            "QUERY_WAVE_FOCUSED",
        ),
        (
            "ru_released",
            "Товар выпущен таможней?",
            "QUERY_WAVE_FOCUSED",
        ),
        (
            "assertion_trap",
            "Customs cleared the goods.",
            "QUERY_WAVE_ASSERTION_TRAP",
        ),
    ]
    .into_iter()
    .map(|(case_id, input, expected_state)| {
        let normalized = normalize_surface(input);
        let observed = classify_query(&normalized);
        let expected_route = "customs-clearance-status";
        QueryWaveEvalCase {
            case_id,
            input,
            expected_route,
            observed_route: observed.route,
            expected_state,
            observed_state: observed.state,
            passed: observed.route == expected_route && observed.state == expected_state,
        }
    })
    .collect()
}

fn normalize_surface(input: &str) -> String {
    input
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .replace([',', '.'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_any(input: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| input.contains(needle))
}

fn role_mask(has_customs: bool, has_goods: bool) -> u16 {
    u16::from(has_customs) | (u16::from(has_goods) << 1)
}

fn score_bool(value: bool) -> i16 {
    if value {
        32
    } else {
        0
    }
}

fn stable_hash(input: &str) -> u32 {
    input.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
}

fn all_eval_routes_pass() -> bool {
    build_paraphrase_eval()
        .iter()
        .all(|case| case.observed_route == case.expected_route)
}

fn all_eval_roles_pass() -> bool {
    [
        "Has customs cleared the goods?",
        "Is the cargo cleared by customs?",
    ]
    .iter()
    .all(|input| {
        let normalized = normalize_surface(input);
        let observed = classify_query(&normalized);
        observed.record.role_mask == 0b0011
    })
}

fn all_eval_operators_pass() -> bool {
    build_paraphrase_eval()
        .iter()
        .filter(|case| case.case_id != "assertion_trap")
        .all(|case| {
            let normalized = normalize_surface(case.input);
            let observed = classify_query(&normalized);
            observed.record.operator_mask == 0b0001
        })
}

fn all_eval_polarity_pass() -> bool {
    let question = classify_query(&normalize_surface("Has customs cleared the goods?"));
    let assertion = classify_query(&normalize_surface("Customs cleared the goods."));
    question.polarity == "question_status" && assertion.polarity == "assertion_claim"
}

fn assertion_trap_rejected() -> bool {
    build_paraphrase_eval()
        .iter()
        .any(|case| case.case_id == "assertion_trap" && case.passed)
}

fn rate(passed: bool) -> f32 {
    if passed {
        1.0
    } else {
        0.0
    }
}
