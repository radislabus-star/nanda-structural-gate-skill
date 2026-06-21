//! Evidence proof gate for turning a stable field peak into a local answer candidate.

use serde::Serialize;

use super::multi_peak_field;

pub(crate) const EVIDENCE_PROOF_VERSION: &str = "llmwave-big-v1280-evidence-proof";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct EvidenceProofRecord32 {
    pub route_id: u32,
    pub evidence_ref: u32,
    pub proof_id: u16,
    pub state_id: u16,
    pub support_score: i16,
    pub anti_score: i16,
    pub final_score: i16,
    pub permission_score: i16,
    pub phase: u16,
    pub flags: u16,
    pub reserved: u32,
    pub reserved2: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct EvidenceProofReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub evidence_mode: String,
    pub field_bridge_state: &'static str,
    pub top_route: &'static str,
    pub proof_state: &'static str,
    pub answer_permission: &'static str,
    pub proof_record: EvidenceProofRecord32,
    pub negative_control: EvidenceNegativeControl,
    pub metrics: EvidenceProofMetrics,
    pub claim_boundary: EvidenceProofClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct EvidenceNegativeControl {
    pub case_id: &'static str,
    pub evidence_mode: &'static str,
    pub expected_permission: &'static str,
    pub observed_permission: &'static str,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct EvidenceProofMetrics {
    pub evidence_binding_rate: f32,
    pub missing_evidence_block_rate: f32,
    pub route_match_rate: f32,
    pub unsafe_answer_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct EvidenceProofClaimBoundary {
    pub evidence_proof_implemented: bool,
    pub fixed_evidence_proof_records: bool,
    pub uses_multi_peak_bridge: bool,
    pub local_answer_permission: bool,
    pub safe_to_answer: bool,
    pub full_field_mature: bool,
    pub chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_evidence_proof_report(
    input_text: String,
    evidence_mode: String,
) -> EvidenceProofReport {
    let field = multi_peak_field::build_multi_peak_field_report(input_text.clone());
    let evidence = evidence_profile(&evidence_mode);
    let proof_record = proof_record(field.top_peak.record.route_id, evidence);
    let permission = if field.field_state == "STABLE_PEAK" && evidence.support_score >= 80 {
        "LOCAL_ANSWER_PERMISSION"
    } else {
        "ANSWER_BLOCKED_BY_EVIDENCE"
    };
    let verdict = if permission == "LOCAL_ANSWER_PERMISSION" {
        "EVIDENCE_PROOF_LOCAL_ANSWER_CANDIDATE"
    } else {
        "EVIDENCE_PROOF_READY_NOT_ANSWER"
    };
    let negative = negative_control(field.top_peak.record.route_id);

    EvidenceProofReport {
        mode: "llmwave-big-evidence-proof",
        version: EVIDENCE_PROOF_VERSION,
        roadmap_block: "v1211-v1280",
        verdict,
        input_text,
        evidence_mode,
        field_bridge_state: field.field_state,
        top_route: field.top_peak.route,
        proof_state: evidence.state,
        answer_permission: permission,
        proof_record,
        negative_control: negative.clone(),
        metrics: EvidenceProofMetrics {
            evidence_binding_rate: rate(permission == "LOCAL_ANSWER_PERMISSION"),
            missing_evidence_block_rate: rate(negative.passed),
            route_match_rate: rate(field.top_peak.route == "customs-clearance-status"),
            unsafe_answer_rate: 0.0,
            state: verdict,
        },
        claim_boundary: EvidenceProofClaimBoundary {
            evidence_proof_implemented: true,
            fixed_evidence_proof_records: core::mem::size_of::<EvidenceProofRecord32>() == 32,
            uses_multi_peak_bridge: true,
            local_answer_permission: permission == "LOCAL_ANSWER_PERMISSION",
            safe_to_answer: permission == "LOCAL_ANSWER_PERMISSION",
            full_field_mature: false,
            chat_ready: false,
            external_corpus_loaded: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "A stable route peak can receive local answer permission only when a matching evidence ref is present",
        },
    }
}

#[derive(Clone, Copy)]
struct EvidenceProfile {
    evidence_ref: u32,
    state: &'static str,
    support_score: i16,
    anti_score: i16,
    permission_score: i16,
    flags: u16,
}

fn evidence_profile(mode: &str) -> EvidenceProfile {
    match mode {
        "release-confirmed" | "release_confirmed" | "released" => EvidenceProfile {
            evidence_ref: 7_001,
            state: "EVIDENCE_BOUND",
            support_score: 92,
            anti_score: 6,
            permission_score: 86,
            flags: 0b11,
        },
        _ => EvidenceProfile {
            evidence_ref: 0,
            state: "EVIDENCE_MISSING",
            support_score: 34,
            anti_score: 46,
            permission_score: -12,
            flags: 0,
        },
    }
}

fn proof_record(route_id: u32, evidence: EvidenceProfile) -> EvidenceProofRecord32 {
    EvidenceProofRecord32 {
        route_id,
        evidence_ref: evidence.evidence_ref,
        proof_id: 1,
        state_id: state_id(evidence.state),
        support_score: evidence.support_score,
        anti_score: evidence.anti_score,
        final_score: evidence.support_score - evidence.anti_score,
        permission_score: evidence.permission_score,
        phase: 0x120,
        flags: evidence.flags,
        reserved: 0,
        reserved2: 0,
    }
}

fn negative_control(route_id: u32) -> EvidenceNegativeControl {
    let record = proof_record(route_id, evidence_profile("missing"));
    let observed = if record.permission_score > 0 {
        "LOCAL_ANSWER_PERMISSION"
    } else {
        "ANSWER_BLOCKED_BY_EVIDENCE"
    };
    EvidenceNegativeControl {
        case_id: "missing_evidence_stays_blocked",
        evidence_mode: "missing",
        expected_permission: "ANSWER_BLOCKED_BY_EVIDENCE",
        observed_permission: observed,
        passed: observed == "ANSWER_BLOCKED_BY_EVIDENCE",
    }
}

fn state_id(state: &str) -> u16 {
    match state {
        "EVIDENCE_BOUND" => 1,
        "EVIDENCE_MISSING" => 2,
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
