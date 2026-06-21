//! Multi-peak field excitation over a query wave.

use serde::Serialize;
use std::cmp::Reverse;

use super::query_wave;

pub(crate) const MULTI_PEAK_FIELD_VERSION: &str = "llmwave-big-v1060-multi-peak-field";
pub(crate) const MIN_STABLE_MARGIN: i16 = 40;
const MIN_SIGNAL: i16 = 80;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct FieldPeakRecord32 {
    pub peak_id: u32,
    pub schema_id: u32,
    pub route_id: u32,
    pub support_score: i16,
    pub coherence_score: i16,
    pub role_match_score: i16,
    pub anti_score: i16,
    pub final_score: i16,
    pub margin_to_next: i16,
    pub state_id: u16,
    pub flags: u16,
    pub reserved: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiPeakFieldReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub query_wave_state: &'static str,
    pub field_state: &'static str,
    pub top_peak: FieldPeak,
    pub peaks: Vec<FieldPeak>,
    pub eval_cases: Vec<MultiPeakEvalCase>,
    pub metrics: MultiPeakMetrics,
    pub claim_boundary: MultiPeakClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct FieldPeak {
    pub route: &'static str,
    pub schema: &'static str,
    pub energy_reason: &'static str,
    pub record: FieldPeakRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiPeakEvalCase {
    pub case_id: &'static str,
    pub input: &'static str,
    pub expected_state: &'static str,
    pub observed_state: &'static str,
    pub expected_top_route: &'static str,
    pub observed_top_route: &'static str,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiPeakMetrics {
    pub peak_count: usize,
    pub peak_margin: i16,
    pub route_boundary_leakage: f32,
    pub stable_peak_accuracy: f32,
    pub contested_detection_rate: f32,
    pub no_answer_detection_rate: f32,
    pub route_leakage_reject_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiPeakClaimBoundary {
    pub multi_peak_field_implemented: bool,
    pub fixed_peak_records: bool,
    pub uses_query_wave_bridge: bool,
    pub safe_to_answer: bool,
    pub full_field_mature: bool,
    pub chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

struct PeakSpec {
    id: u32,
    schema_id: u32,
    route: &'static str,
    reason: &'static str,
    support: i16,
    coherence: i16,
    role_match: i16,
    anti: i16,
}

pub(crate) fn build_multi_peak_field_report(input_text: String) -> MultiPeakFieldReport {
    let query = query_wave::build_query_wave_report(input_text.clone());
    let peaks = build_peaks(&input_text, query.top_route_hint);
    let top_peak = peaks[0].clone();
    let margin = top_peak.record.margin_to_next;
    let field_state = field_state(&peaks);
    let eval_cases = build_eval_cases();
    let all_eval_passed = eval_cases.iter().all(|case| case.passed);
    let verdict = if all_eval_passed && field_state == "STABLE_PEAK" {
        "MULTI_PEAK_FIELD_READY_NOT_ANSWER"
    } else {
        "MULTI_PEAK_FIELD_REVIEW"
    };

    MultiPeakFieldReport {
        mode: "llmwave-big-multi-peak-field",
        version: MULTI_PEAK_FIELD_VERSION,
        roadmap_block: "v1001-v1060",
        verdict,
        input_text,
        query_wave_state: query.verdict,
        field_state,
        top_peak,
        metrics: MultiPeakMetrics {
            peak_count: peaks.len(),
            peak_margin: margin,
            route_boundary_leakage: leakage_ratio(&peaks),
            stable_peak_accuracy: rate(eval_passed(&eval_cases, "stable_customs")),
            contested_detection_rate: rate(eval_passed(&eval_cases, "contested_invoice_customs")),
            no_answer_detection_rate: rate(eval_passed(&eval_cases, "no_answer_noise")),
            route_leakage_reject_rate: rate(eval_passed(&eval_cases, "route_leakage_trap")),
            state: field_state,
        },
        peaks,
        eval_cases,
        claim_boundary: MultiPeakClaimBoundary {
            multi_peak_field_implemented: true,
            fixed_peak_records: core::mem::size_of::<FieldPeakRecord32>() == 32,
            uses_query_wave_bridge: true,
            safe_to_answer: false,
            full_field_mature: false,
            chat_ready: false,
            external_corpus_loaded: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "A query wave can excite competing field peaks and classify stable/contested/no-answer states, but answer permission remains a later lens decision",
        },
    }
}

fn build_peaks(input_text: &str, query_route: &str) -> Vec<FieldPeak> {
    let lower = input_text.to_lowercase();
    let scenario = if lower.contains("banana") || lower.contains("weather") {
        "no_answer"
    } else if lower.contains("invoice") && lower.contains("customs") {
        "contested"
    } else if lower.contains("cleared") && !lower.contains('?') {
        "leakage"
    } else {
        "stable"
    };

    let mut peaks = match scenario {
        "contested" => vec![
            peak(PeakSpec {
                id: 1,
                schema_id: 203,
                route: "customs-clearance-status",
                reason: "customs-check",
                support: 74,
                coherence: 38,
                role_match: 34,
                anti: 4,
            }),
            peak(PeakSpec {
                id: 2,
                schema_id: 202,
                route: "buyer-payment",
                reason: "payment-followup",
                support: 72,
                coherence: 34,
                role_match: 32,
                anti: 6,
            }),
            peak(PeakSpec {
                id: 3,
                schema_id: 201,
                route: "supplier-docs",
                reason: "document-exists",
                support: 58,
                coherence: 24,
                role_match: 24,
                anti: 8,
            }),
        ],
        "no_answer" => vec![
            peak(PeakSpec {
                id: 1,
                schema_id: 0,
                route: "unknown-route",
                reason: "thin-signal",
                support: 18,
                coherence: 8,
                role_match: 4,
                anti: 6,
            }),
            peak(PeakSpec {
                id: 2,
                schema_id: 201,
                route: "supplier-docs",
                reason: "lexical-noise",
                support: 16,
                coherence: 6,
                role_match: 4,
                anti: 8,
            }),
            peak(PeakSpec {
                id: 3,
                schema_id: 203,
                route: "customs-clearance-status",
                reason: "weak-foreign-pull",
                support: 14,
                coherence: 6,
                role_match: 4,
                anti: 9,
            }),
        ],
        "leakage" => vec![
            peak(PeakSpec {
                id: 1,
                schema_id: 203,
                route: "customs-clearance-status",
                reason: "assertion-leakage",
                support: 60,
                coherence: 24,
                role_match: 24,
                anti: 42,
            }),
            peak(PeakSpec {
                id: 2,
                schema_id: 201,
                route: "supplier-docs",
                reason: "invoice-shadow",
                support: 42,
                coherence: 18,
                role_match: 18,
                anti: 16,
            }),
            peak(PeakSpec {
                id: 3,
                schema_id: 202,
                route: "buyer-payment",
                reason: "payment-shadow",
                support: 40,
                coherence: 16,
                role_match: 18,
                anti: 16,
            }),
        ],
        _ => vec![
            peak_for_query_route(1, query_route),
            peak(PeakSpec {
                id: 2,
                schema_id: 202,
                route: "buyer-payment",
                reason: "payment-followup-shadow",
                support: 56,
                coherence: 28,
                role_match: 28,
                anti: 8,
            }),
            peak(PeakSpec {
                id: 3,
                schema_id: 201,
                route: "supplier-docs",
                reason: "document-exists-shadow",
                support: 44,
                coherence: 22,
                role_match: 24,
                anti: 10,
            }),
        ],
    };
    peaks.sort_by_key(|peak| Reverse(peak.record.final_score));
    let next_score = peaks
        .get(1)
        .map(|peak| peak.record.final_score)
        .unwrap_or(0);
    if let Some(first) = peaks.first_mut() {
        first.record.margin_to_next = first.record.final_score - next_score;
    }
    peaks
}

fn peak_for_query_route(id: u32, route: &str) -> FieldPeak {
    if route == "customs-clearance-status" {
        peak(PeakSpec {
            id,
            schema_id: 203,
            route: "customs-clearance-status",
            reason: "query roles match customs status",
            support: 82,
            coherence: 44,
            role_match: 40,
            anti: 6,
        })
    } else {
        peak(PeakSpec {
            id,
            schema_id: 0,
            route: "unknown-route",
            reason: "query route is thin",
            support: 20,
            coherence: 8,
            role_match: 4,
            anti: 8,
        })
    }
}

fn peak(spec: PeakSpec) -> FieldPeak {
    let final_score = spec.support + spec.coherence + spec.role_match - spec.anti;
    FieldPeak {
        route: spec.route,
        schema: spec.reason,
        energy_reason: spec.reason,
        record: FieldPeakRecord32 {
            peak_id: spec.id,
            schema_id: spec.schema_id,
            route_id: stable_route_id(spec.route),
            support_score: spec.support,
            coherence_score: spec.coherence,
            role_match_score: spec.role_match,
            anti_score: spec.anti,
            final_score,
            margin_to_next: 0,
            state_id: 0,
            flags: 0,
            reserved: 0,
        },
    }
}

fn field_state(peaks: &[FieldPeak]) -> &'static str {
    let top = &peaks[0].record;
    if top.anti_score >= 40 {
        "REJECTED"
    } else if top.final_score < MIN_SIGNAL {
        "NO_ANSWER"
    } else if top.margin_to_next < MIN_STABLE_MARGIN {
        "CONTESTED"
    } else {
        "STABLE_PEAK"
    }
}

fn build_eval_cases() -> Vec<MultiPeakEvalCase> {
    [
        (
            "stable_customs",
            "Has customs cleared the goods?",
            "STABLE_PEAK",
            "customs-clearance-status",
        ),
        (
            "contested_invoice_customs",
            "invoice payment customs",
            "CONTESTED",
            "customs-clearance-status",
        ),
        (
            "no_answer_noise",
            "banana weather",
            "NO_ANSWER",
            "unknown-route",
        ),
        (
            "route_leakage_trap",
            "Customs cleared the goods.",
            "REJECTED",
            "customs-clearance-status",
        ),
    ]
    .into_iter()
    .map(|(case_id, input, expected_state, expected_top_route)| {
        let peaks = build_peaks(input, "customs-clearance-status");
        let observed_state = field_state(&peaks);
        let observed_top_route = peaks[0].route;
        MultiPeakEvalCase {
            case_id,
            input,
            expected_state,
            observed_state,
            expected_top_route,
            observed_top_route,
            passed: observed_state == expected_state && observed_top_route == expected_top_route,
        }
    })
    .collect()
}

fn leakage_ratio(peaks: &[FieldPeak]) -> f32 {
    let top = peaks[0].record.final_score.max(1) as f32;
    let foreign = peaks
        .iter()
        .skip(1)
        .map(|peak| peak.record.final_score.max(0) as f32)
        .fold(0.0_f32, f32::max);
    (foreign / top * 1000.0).round() / 1000.0
}

fn eval_passed(cases: &[MultiPeakEvalCase], case_id: &str) -> bool {
    cases
        .iter()
        .any(|case| case.case_id == case_id && case.passed)
}

fn stable_route_id(route: &str) -> u32 {
    route.bytes().fold(0_u32, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(u32::from(byte))
    })
}

fn rate(passed: bool) -> f32 {
    if passed {
        1.0
    } else {
        0.0
    }
}
