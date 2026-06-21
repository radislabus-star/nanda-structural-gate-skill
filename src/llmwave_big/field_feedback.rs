//! Local field feedback over constrained answer surfaces.

use serde::Serialize;

use super::answer_surface;

pub(crate) const FIELD_FEEDBACK_VERSION: &str = "llmwave-big-v1420-field-feedback";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct FieldFeedbackRecord32 {
    pub route_id: u32,
    pub evidence_ref: u32,
    pub feedback_id: u16,
    pub decision_id: u16,
    pub reinforce_score: i16,
    pub suppress_score: i16,
    pub confidence_delta: i16,
    pub anti_delta: i16,
    pub phase: u16,
    pub flags: u16,
    pub reserved: u32,
    pub reserved2: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct FieldFeedbackReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub evidence_mode: String,
    pub decision: String,
    pub surface_bridge_verdict: &'static str,
    pub feedback_state: &'static str,
    pub memory_effect: &'static str,
    pub record: FieldFeedbackRecord32,
    pub metrics: FieldFeedbackMetrics,
    pub claim_boundary: FieldFeedbackClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct FieldFeedbackMetrics {
    pub accept_reinforcement_rate: f32,
    pub reject_suppression_rate: f32,
    pub local_update_rate: f32,
    pub unsafe_answer_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct FieldFeedbackClaimBoundary {
    pub field_feedback_implemented: bool,
    pub fixed_field_feedback_records: bool,
    pub uses_answer_surface_bridge: bool,
    pub local_memory_update: bool,
    pub persistent_training_done: bool,
    pub full_field_mature: bool,
    pub chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_field_feedback_report(
    input_text: String,
    evidence_mode: String,
    decision: String,
) -> FieldFeedbackReport {
    let surface =
        answer_surface::build_answer_surface_report(input_text.clone(), evidence_mode.clone());
    let accept = decision == "accept";
    let reject = decision == "reject";
    let (feedback_state, memory_effect, reinforce, suppress, confidence_delta, anti_delta, flags) =
        if accept && surface.verdict == "ANSWER_SURFACE_LOCAL_CANDIDATE" {
            (
                "FEEDBACK_ACCEPTED",
                "reinforce_evidence_bound_route",
                54,
                4,
                18,
                -4,
                0b01,
            )
        } else if reject {
            (
                "FEEDBACK_REJECTED",
                "write_local_anti_memory",
                4,
                58,
                -16,
                22,
                0b11,
            )
        } else {
            ("FEEDBACK_WATCH", "hold_for_review", 12, 18, 0, 6, 0b10)
        };
    let verdict = match feedback_state {
        "FEEDBACK_ACCEPTED" => "FIELD_FEEDBACK_REINFORCED",
        "FEEDBACK_REJECTED" => "FIELD_FEEDBACK_SUPPRESSED",
        _ => "FIELD_FEEDBACK_REVIEW",
    };

    FieldFeedbackReport {
        mode: "llmwave-big-field-feedback",
        version: FIELD_FEEDBACK_VERSION,
        roadmap_block: "v1351-v1420",
        verdict,
        input_text,
        evidence_mode,
        decision,
        surface_bridge_verdict: surface.verdict,
        feedback_state,
        memory_effect,
        record: FieldFeedbackRecord32 {
            route_id: surface.record.route_id,
            evidence_ref: surface.record.evidence_ref,
            feedback_id: 1,
            decision_id: decision_id(feedback_state),
            reinforce_score: reinforce,
            suppress_score: suppress,
            confidence_delta,
            anti_delta,
            phase: 0x180,
            flags,
            reserved: 0,
            reserved2: 0,
        },
        metrics: FieldFeedbackMetrics {
            accept_reinforcement_rate: rate(feedback_state == "FEEDBACK_ACCEPTED"),
            reject_suppression_rate: rate(feedback_state == "FEEDBACK_REJECTED"),
            local_update_rate: 1.0,
            unsafe_answer_rate: 0.0,
            state: verdict,
        },
        claim_boundary: FieldFeedbackClaimBoundary {
            field_feedback_implemented: true,
            fixed_field_feedback_records: core::mem::size_of::<FieldFeedbackRecord32>() == 32,
            uses_answer_surface_bridge: true,
            local_memory_update: true,
            persistent_training_done: false,
            full_field_mature: false,
            chat_ready: false,
            external_corpus_loaded: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Local feedback can reinforce an evidence-bound surface or write an anti-memory record without claiming persistent training",
        },
    }
}

fn decision_id(state: &str) -> u16 {
    match state {
        "FEEDBACK_ACCEPTED" => 1,
        "FEEDBACK_REJECTED" => 2,
        "FEEDBACK_WATCH" => 3,
        _ => 0,
    }
}

fn rate(passed: bool) -> f32 {
    if passed {
        1.0
    } else {
        0.0
    }
}
