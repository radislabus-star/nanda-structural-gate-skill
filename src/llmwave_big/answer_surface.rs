//! Constrained answer materialization from the evidence proof gate.

use serde::Serialize;

use super::evidence_proof;

pub(crate) const ANSWER_SURFACE_VERSION: &str = "llmwave-big-v1350-answer-surface";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct AnswerSurfaceRecord32 {
    pub route_id: u32,
    pub evidence_ref: u32,
    pub template_id: u16,
    pub state_id: u16,
    pub proof_score: i16,
    pub anti_score: i16,
    pub surface_score: i16,
    pub permission_score: i16,
    pub flags: u16,
    pub reserved: u16,
    pub reserved2: u32,
    pub reserved3: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct AnswerSurfaceReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub evidence_mode: String,
    pub proof_bridge_verdict: &'static str,
    pub answer_state: &'static str,
    pub answer_text: &'static str,
    pub record: AnswerSurfaceRecord32,
    pub metrics: AnswerSurfaceMetrics,
    pub claim_boundary: AnswerSurfaceClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct AnswerSurfaceMetrics {
    pub constrained_template_rate: f32,
    pub evidence_ref_copy_rate: f32,
    pub unsupported_confirmation_rate: f32,
    pub unsafe_answer_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct AnswerSurfaceClaimBoundary {
    pub answer_surface_implemented: bool,
    pub fixed_answer_surface_records: bool,
    pub uses_evidence_proof_bridge: bool,
    pub local_answer_surface: bool,
    pub free_form_generation: bool,
    pub full_field_mature: bool,
    pub chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_answer_surface_report(
    input_text: String,
    evidence_mode: String,
) -> AnswerSurfaceReport {
    let proof =
        evidence_proof::build_evidence_proof_report(input_text.clone(), evidence_mode.clone());
    let confirmed = proof.answer_permission == "LOCAL_ANSWER_PERMISSION";
    let (answer_state, answer_text, template_id, anti_score, surface_score, flags) = if confirmed {
        (
            "LOCAL_EVIDENCE_BOUND_ANSWER",
            "Customs clearance is confirmed by evidence ref 7001.",
            2,
            4,
            88,
            0b11,
        )
    } else {
        (
            "NOT_PROVEN_ANSWER",
            "Not proven: customs clearance evidence is missing.",
            1,
            42,
            64,
            0b01,
        )
    };
    let verdict = if confirmed {
        "ANSWER_SURFACE_LOCAL_CANDIDATE"
    } else {
        "ANSWER_SURFACE_NOT_PROVEN"
    };

    AnswerSurfaceReport {
        mode: "llmwave-big-answer-surface",
        version: ANSWER_SURFACE_VERSION,
        roadmap_block: "v1281-v1350",
        verdict,
        input_text,
        evidence_mode,
        proof_bridge_verdict: proof.verdict,
        answer_state,
        answer_text,
        record: AnswerSurfaceRecord32 {
            route_id: proof.proof_record.route_id,
            evidence_ref: proof.proof_record.evidence_ref,
            template_id,
            state_id: state_id(answer_state),
            proof_score: proof.proof_record.permission_score,
            anti_score,
            surface_score,
            permission_score: proof.proof_record.permission_score,
            flags,
            reserved: 0,
            reserved2: 0,
            reserved3: 0,
        },
        metrics: AnswerSurfaceMetrics {
            constrained_template_rate: 1.0,
            evidence_ref_copy_rate: rate(confirmed),
            unsupported_confirmation_rate: 0.0,
            unsafe_answer_rate: 0.0,
            state: verdict,
        },
        claim_boundary: AnswerSurfaceClaimBoundary {
            answer_surface_implemented: true,
            fixed_answer_surface_records: core::mem::size_of::<AnswerSurfaceRecord32>() == 32,
            uses_evidence_proof_bridge: true,
            local_answer_surface: true,
            free_form_generation: false,
            full_field_mature: false,
            chat_ready: false,
            external_corpus_loaded: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Evidence proof can select a constrained answer template without enabling free-form chat",
        },
    }
}

fn state_id(state: &str) -> u16 {
    match state {
        "NOT_PROVEN_ANSWER" => 1,
        "LOCAL_EVIDENCE_BOUND_ANSWER" => 2,
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
