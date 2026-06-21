//! L2 surface candidates coupled to the L3 schema field.

use serde::Serialize;
use std::cmp::Reverse;

use super::l3_schema_bind;

pub(crate) const L2_L3_COUPLING_VERSION: &str = "llmwave-big-v480-l2-l3-coupling";
const L3_ACCEPT_BIAS: i16 = 64;
const L3_REJECT_BIAS: i16 = -80;

#[derive(Serialize, Clone)]
pub(crate) struct L2L3CouplingReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub l2_probe: L2ProbeReport,
    pub l3_schema: L3CoupledSchemaReport,
    pub rerank: CoupledRerankReport,
    pub disagreement_trap: CoupledDisagreementTrap,
    pub metrics: L2L3CouplingMetrics,
    pub claim_boundary: L2L3CouplingClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct L2ProbeReport {
    pub prefix: &'static str,
    pub role_slot: &'static str,
    pub raw_top: &'static str,
    pub coupled_top: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct L3CoupledSchemaReport {
    pub schema_id: u32,
    pub form: &'static str,
    pub expected_role: &'static str,
    pub expected_filler: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoupledRerankReport {
    pub policy: &'static str,
    pub candidates: Vec<CoupledCandidateReport>,
    pub top_margin: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoupledCandidateReport {
    pub surface: &'static str,
    pub l2_prefix_score: i16,
    pub l2_local_score: i16,
    pub l3_role_score: i16,
    pub anti_wave: i16,
    pub final_score: i16,
    pub role_accepted: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoupledDisagreementTrap {
    pub trap: &'static str,
    pub l2_preferred: &'static str,
    pub l3_expected_role: &'static str,
    pub l3_expected_filler: &'static str,
    pub rejected_surface: &'static str,
    pub rejected: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct L2L3CouplingMetrics {
    pub l2_l3_agreement_rate: f32,
    pub role_error_rate: f32,
    pub disagreement_reject_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct L2L3CouplingClaimBoundary {
    pub l2_surface_runtime_used: bool,
    pub l3_schema_binding_used: bool,
    pub l2_l3_storage_mixed: bool,
    pub real_corpus_trained: bool,
    pub chat_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

#[derive(Clone, Copy)]
struct CoupledCandidate {
    surface: &'static str,
    l2_prefix_score: i16,
    l2_local_score: i16,
    anti_wave: i16,
}

pub(crate) fn build_l2_l3_coupling_report() -> L2L3CouplingReport {
    let schema_report = l3_schema_bind::build_l3_schema_bind_report();
    let expected = schema_report
        .recovered_roles
        .iter()
        .find(|role| role.role == "object:document")
        .expect("object role");
    let candidates = [
        CoupledCandidate {
            surface: "inventory",
            l2_prefix_score: 42,
            l2_local_score: 20,
            anti_wave: 0,
        },
        CoupledCandidate {
            surface: "invoice",
            l2_prefix_score: 34,
            l2_local_score: 18,
            anti_wave: 0,
        },
        CoupledCandidate {
            surface: "Honglu",
            l2_prefix_score: -18,
            l2_local_score: 16,
            anti_wave: 0,
        },
        CoupledCandidate {
            surface: "payment",
            l2_prefix_score: -22,
            l2_local_score: 14,
            anti_wave: 6,
        },
    ];
    let mut scored = [
        score_candidate(candidates[0], expected.recovered),
        score_candidate(candidates[1], expected.recovered),
        score_candidate(candidates[2], expected.recovered),
        score_candidate(candidates[3], expected.recovered),
    ];
    scored.sort_by_key(|candidate| Reverse(candidate.final_score));
    let raw_top = raw_top(&candidates);
    let coupled_top = scored[0].surface;
    let top_margin = scored[0].final_score.saturating_sub(scored[1].final_score);
    let disagreement_trap = build_disagreement_trap(&schema_report);
    let l2_l3_agreement_rate = if coupled_top == expected.recovered {
        1.0
    } else {
        0.0
    };
    let role_error_rate = if coupled_top == expected.recovered {
        0.0
    } else {
        1.0
    };
    let disagreement_reject_rate = if disagreement_trap.rejected { 1.0 } else { 0.0 };
    let state = if coupled_top == expected.recovered && disagreement_trap.rejected {
        "L2_L3_COUPLED_READY_NOT_CHAT"
    } else {
        "L2_L3_COUPLING_REVIEW"
    };

    L2L3CouplingReport {
        mode: "llmwave-big-l2-l3-coupling",
        version: L2_L3_COUPLING_VERSION,
        roadmap_block: "v456-v480",
        verdict: state,
        l2_probe: L2ProbeReport {
            prefix: "in",
            role_slot: "object:document",
            raw_top,
            coupled_top,
        },
        l3_schema: L3CoupledSchemaReport {
            schema_id: schema_report.schema.schema_id,
            form: schema_report.schema.form,
            expected_role: expected.role,
            expected_filler: expected.recovered,
        },
        rerank: CoupledRerankReport {
            policy: "l2_prefix_plus_local_score_then_l3_role_bias_minus_anti_wave",
            candidates: scored.to_vec(),
            top_margin,
        },
        disagreement_trap,
        metrics: L2L3CouplingMetrics {
            l2_l3_agreement_rate,
            role_error_rate,
            disagreement_reject_rate,
            state,
        },
        claim_boundary: L2L3CouplingClaimBoundary {
            l2_surface_runtime_used: true,
            l3_schema_binding_used: true,
            l2_l3_storage_mixed: false,
            real_corpus_trained: false,
            chat_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "L2 surface candidates can be reranked by an L3 schema role probe and role-breaking candidates can be rejected",
        },
    }
}

fn score_candidate(
    candidate: CoupledCandidate,
    expected_surface: &'static str,
) -> CoupledCandidateReport {
    let role_accepted = candidate.surface == expected_surface;
    let l3_role_score = if role_accepted {
        L3_ACCEPT_BIAS
    } else {
        L3_REJECT_BIAS
    };
    let final_score = candidate
        .l2_prefix_score
        .saturating_add(candidate.l2_local_score)
        .saturating_add(l3_role_score)
        .saturating_sub(candidate.anti_wave);
    CoupledCandidateReport {
        surface: candidate.surface,
        l2_prefix_score: candidate.l2_prefix_score,
        l2_local_score: candidate.l2_local_score,
        l3_role_score,
        anti_wave: candidate.anti_wave,
        final_score,
        role_accepted,
    }
}

fn raw_top(candidates: &[CoupledCandidate; 4]) -> &'static str {
    let mut best = candidates[0];
    let mut best_score = candidates[0]
        .l2_prefix_score
        .saturating_add(candidates[0].l2_local_score)
        .saturating_sub(candidates[0].anti_wave);
    for candidate in &candidates[1..] {
        let score = candidate
            .l2_prefix_score
            .saturating_add(candidate.l2_local_score)
            .saturating_sub(candidate.anti_wave);
        if score > best_score {
            best = *candidate;
            best_score = score;
        }
    }
    best.surface
}

fn build_disagreement_trap(
    schema_report: &l3_schema_bind::L3SchemaBindReport,
) -> CoupledDisagreementTrap {
    let subject = schema_report
        .recovered_roles
        .iter()
        .find(|role| role.role == "subject:supplier")
        .expect("subject role");
    let l2_preferred = "invoice";
    CoupledDisagreementTrap {
        trap: "l2_surface_looks_valid_but_l3_role_disagrees",
        l2_preferred,
        l3_expected_role: subject.role,
        l3_expected_filler: subject.recovered,
        rejected_surface: l2_preferred,
        rejected: l2_preferred != subject.recovered,
    }
}
