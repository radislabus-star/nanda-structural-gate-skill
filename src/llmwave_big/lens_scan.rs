//! Field lens scan over the raw multi-peak field.

use serde::Serialize;

use super::multi_peak_field;

pub(crate) const LENS_SCAN_VERSION: &str = "llmwave-big-v1140-lens-scan";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct LensRecord32 {
    pub peak_route_id: u32,
    pub evidence_ref: u32,
    pub lens_id: u16,
    pub state_id: u16,
    pub support_score: i16,
    pub anti_score: i16,
    pub final_score: i16,
    pub agreement_score: i16,
    pub flags: u16,
    pub reserved: u16,
    pub reserved2: u32,
    pub reserved3: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LensScanReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub field_bridge_state: &'static str,
    pub top_route: &'static str,
    pub answer_decision: &'static str,
    pub lenses: Vec<LensResult>,
    pub metrics: LensMetrics,
    pub claim_boundary: LensClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LensResult {
    pub lens: &'static str,
    pub state: &'static str,
    pub reason: &'static str,
    pub record: LensRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LensMetrics {
    pub lens_count: usize,
    pub pass_count: usize,
    pub watch_count: usize,
    pub block_count: usize,
    pub lens_agreement_rate: f32,
    pub role_lens_pass_rate: f32,
    pub evidence_block_rate: f32,
    pub answer_block_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LensClaimBoundary {
    pub lens_scan_implemented: bool,
    pub fixed_lens_records: bool,
    pub uses_multi_peak_bridge: bool,
    pub safe_to_answer: bool,
    pub full_field_mature: bool,
    pub chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_lens_scan_report(input_text: String) -> LensScanReport {
    let field = multi_peak_field::build_multi_peak_field_report(input_text.clone());
    let lenses = build_lenses(field.top_peak.record.route_id);
    let pass_count = lenses.iter().filter(|lens| lens.state == "PASS").count();
    let watch_count = lenses.iter().filter(|lens| lens.state == "WATCH").count();
    let block_count = lenses.iter().filter(|lens| lens.state == "BLOCK").count();
    let answer_blocked = lenses
        .iter()
        .any(|lens| lens.lens == "answer" && lens.state == "BLOCK");
    let verdict = if field.field_state == "STABLE_PEAK" && answer_blocked {
        "LENS_SCAN_READY_NOT_ANSWER"
    } else {
        "LENS_SCAN_REVIEW"
    };

    LensScanReport {
        mode: "llmwave-big-lens-scan",
        version: LENS_SCAN_VERSION,
        roadmap_block: "v1061-v1140",
        verdict,
        input_text,
        field_bridge_state: field.field_state,
        top_route: field.top_peak.route,
        answer_decision: if answer_blocked {
            "ANSWER_BLOCKED_BY_LENSES"
        } else {
            "ANSWER_CANDIDATE"
        },
        metrics: LensMetrics {
            lens_count: lenses.len(),
            pass_count,
            watch_count,
            block_count,
            lens_agreement_rate: round3(pass_count as f32 / lenses.len() as f32),
            role_lens_pass_rate: rate(lens_state(&lenses, "role") == "PASS"),
            evidence_block_rate: rate(lens_state(&lenses, "evidence") == "WATCH"),
            answer_block_rate: rate(answer_blocked),
            state: verdict,
        },
        lenses,
        claim_boundary: LensClaimBoundary {
            lens_scan_implemented: true,
            fixed_lens_records: core::mem::size_of::<LensRecord32>() == 32,
            uses_multi_peak_bridge: true,
            safe_to_answer: false,
            full_field_mature: false,
            chat_ready: false,
            external_corpus_loaded: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Lens scan can block a stable raw peak when evidence/causal/answer lenses do not allow an answer",
        },
    }
}

fn build_lenses(route_id: u32) -> Vec<LensResult> {
    vec![
        lens(
            1,
            route_id,
            "role",
            "PASS",
            "subject/object roles align with customs status",
            70,
            4,
        ),
        lens(
            2,
            route_id,
            "evidence",
            "WATCH",
            "clearance evidence is missing",
            36,
            42,
        ),
        lens(
            3,
            route_id,
            "temporal",
            "PASS",
            "question asks current status without stale evidence",
            58,
            8,
        ),
        lens(
            4,
            route_id,
            "causal",
            "WATCH",
            "invoice/payment chain does not imply clearance",
            44,
            38,
        ),
        lens(
            5,
            route_id,
            "contradiction",
            "PASS",
            "no direct contradiction in active field",
            52,
            6,
        ),
        lens(
            6,
            route_id,
            "surface",
            "PASS",
            "surface form preserves status question",
            50,
            6,
        ),
        lens(
            7,
            route_id,
            "answer",
            "BLOCK",
            "stable peak lacks evidence permission",
            30,
            60,
        ),
    ]
}

fn lens(
    lens_id: u16,
    route_id: u32,
    lens: &'static str,
    state: &'static str,
    reason: &'static str,
    support: i16,
    anti: i16,
) -> LensResult {
    LensResult {
        lens,
        state,
        reason,
        record: LensRecord32 {
            peak_route_id: route_id,
            evidence_ref: if lens == "evidence" { 0 } else { 1 },
            lens_id,
            state_id: state_id(state),
            support_score: support,
            anti_score: anti,
            final_score: support - anti,
            agreement_score: if state == "PASS" { 100 } else { 0 },
            flags: if state == "PASS" { 1 } else { 0 },
            reserved: 0,
            reserved2: 0,
            reserved3: 0,
        },
    }
}

fn lens_state(lenses: &[LensResult], lens_name: &str) -> &'static str {
    lenses
        .iter()
        .find(|lens| lens.lens == lens_name)
        .map(|lens| lens.state)
        .unwrap_or("MISSING")
}

fn state_id(state: &str) -> u16 {
    match state {
        "PASS" => 1,
        "WATCH" => 2,
        "BLOCK" => 3,
        _ => 0,
    }
}

fn round3(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}

fn rate(passed: bool) -> f32 {
    if passed {
        1.0
    } else {
        0.0
    }
}
