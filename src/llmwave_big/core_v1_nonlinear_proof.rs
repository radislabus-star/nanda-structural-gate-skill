//! LLMWave Core V1 nonlinear memory proof gate.

use serde::Serialize;

use super::core_v1_memory_writer;
use super::nonlinear_memory_eval::{self, NonlinearProofPolicyKind};

pub(crate) const CORE_V1_NONLINEAR_PROOF_VERSION: &str = "llmwave-core-v1-nonlinear-proof-phase4";

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1NonlinearProofReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub writer_evidence: CoreV1WriterEvidence,
    pub ladder_evidence: CoreV1LadderEvidence,
    pub eval_evidence: CoreV1NonlinearEvalEvidence,
    pub proof_metrics: CoreV1NonlinearProofMetrics,
    pub proof_gates: Vec<CoreV1NonlinearProofGate>,
    pub claim_boundary: CoreV1NonlinearProofClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1WriterEvidence {
    pub residual_write_path_active: bool,
    pub raw_dictionary_is_not_primary_memory: bool,
    pub writer_saving_ratio: f64,
    pub schema_reuse_ratio: f64,
    pub residual_saving_ratio: f64,
    pub rejected_duplicate_count: usize,
    pub rejected_noise_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1LadderEvidence {
    pub ladder_ready: bool,
    pub max_facts: usize,
    pub scale_points: usize,
    pub amortized_win_point: Option<usize>,
    pub standalone_break_even_point: Option<usize>,
    pub bytes_fall_points: usize,
    pub min_heldout_pass_rate: f64,
    pub max_role_error_rate: f64,
    pub max_false_positive_rate: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1NonlinearEvalEvidence {
    pub eval_verdict: &'static str,
    pub fixed_basis_used: bool,
    pub linear_baseline_compared: bool,
    pub useful_density_candidate: bool,
    pub external_corpus_present: bool,
    pub heldout_inference_present: bool,
    pub broad_noise_eval_present: bool,
    pub selected_policy_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1NonlinearProofMetrics {
    pub bytes_per_useful_fact_falls_at_three_scale_points: bool,
    pub writer_beats_raw_dictionary_fixture: bool,
    pub heldout_quality_bound_to_writer: bool,
    pub baseline_delta_present: bool,
    pub near_duplicate_leakage_control_present: bool,
    pub role_error_rate_bounded: bool,
    pub false_positive_rate_bounded: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1NonlinearProofGate {
    pub gate: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1NonlinearProofClaimBoundary {
    pub nonlinear_memory_proof_gate_implemented: bool,
    pub nonlinear_memory_candidate: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

pub(crate) fn build_core_v1_nonlinear_proof_report() -> CoreV1NonlinearProofReport {
    let writer = core_v1_memory_writer::build_core_v1_memory_writer_report();
    let ladder = nonlinear_memory_eval::build_nonlinear_memory_ladder_report(100_000);
    let eval = nonlinear_memory_eval::build_nonlinear_memory_eval_report(
        None,
        NonlinearProofPolicyKind::ScaleAmortized,
    )
    .expect("embedded nonlinear proof report should not fail without external corpus");

    let bytes_fall_points = falling_byte_points(&ladder.ladder);
    let writer_evidence = CoreV1WriterEvidence {
        residual_write_path_active: writer.claim_boundary.residual_write_path_active,
        raw_dictionary_is_not_primary_memory: writer
            .claim_boundary
            .raw_dictionary_is_not_primary_memory,
        writer_saving_ratio: writer.byte_report.writer_saving_ratio,
        schema_reuse_ratio: writer.schema_residual_summary.schema_reuse_ratio,
        residual_saving_ratio: writer.schema_residual_summary.residual_saving_ratio,
        rejected_duplicate_count: writer.rejected.rejected_duplicate_count,
        rejected_noise_count: writer.rejected.rejected_noise_count,
    };
    let ladder_evidence = CoreV1LadderEvidence {
        ladder_ready: ladder.aggregate.phase1_ready,
        max_facts: ladder.max_facts,
        scale_points: ladder.ladder.len(),
        amortized_win_point: ladder.aggregate.amortized_win_point,
        standalone_break_even_point: ladder.aggregate.standalone_break_even_point,
        bytes_fall_points,
        min_heldout_pass_rate: ladder.aggregate.min_heldout_pass_rate,
        max_role_error_rate: ladder
            .ladder
            .iter()
            .map(|point| point.role_error_rate)
            .fold(0.0_f64, f64::max),
        max_false_positive_rate: ladder
            .ladder
            .iter()
            .map(|point| point.false_positive_rate)
            .fold(0.0_f64, f64::max),
    };
    let eval_evidence = CoreV1NonlinearEvalEvidence {
        eval_verdict: eval.verdict,
        fixed_basis_used: eval.claim_boundary.fixed_basis_used,
        linear_baseline_compared: eval.claim_boundary.linear_baseline_compared,
        useful_density_candidate: eval.claim_boundary.useful_density_candidate,
        external_corpus_present: eval.gates.external_corpus_present,
        heldout_inference_present: eval.gates.heldout_inference_present,
        broad_noise_eval_present: eval.gates.broad_noise_eval_present,
        selected_policy_proven: eval.claim_boundary.selected_policy_proven,
    };
    let proof_metrics = CoreV1NonlinearProofMetrics {
        bytes_per_useful_fact_falls_at_three_scale_points: bytes_fall_points >= 3,
        writer_beats_raw_dictionary_fixture: writer.byte_report.writer_saving_ratio > 0.0,
        heldout_quality_bound_to_writer: false,
        baseline_delta_present: eval.claim_boundary.linear_baseline_compared,
        near_duplicate_leakage_control_present: false,
        role_error_rate_bounded: ladder_evidence.max_role_error_rate <= 0.05,
        false_positive_rate_bounded: ladder_evidence.max_false_positive_rate <= 0.08,
    };
    let proof_gates = proof_gates(
        &writer_evidence,
        &ladder_evidence,
        &eval_evidence,
        &proof_metrics,
    );
    let all_gates_passed = proof_gates.iter().all(|gate| gate.passed);
    let nonlinear_memory_candidate = writer_evidence.writer_saving_ratio > 0.0
        && ladder_evidence.ladder_ready
        && eval_evidence.useful_density_candidate
        && proof_metrics.bytes_per_useful_fact_falls_at_three_scale_points
        && proof_metrics.role_error_rate_bounded
        && proof_metrics.false_positive_rate_bounded;
    let nonlinear_memory_proven = nonlinear_memory_candidate && all_gates_passed;
    let verdict = if nonlinear_memory_proven {
        "CORE_V1_NONLINEAR_MEMORY_PROOF_PASS"
    } else if nonlinear_memory_candidate {
        "CORE_V1_NONLINEAR_MEMORY_CANDIDATE_BLOCKED"
    } else {
        "CORE_V1_NONLINEAR_MEMORY_REVIEW"
    };
    let blocked_by = blocked_by(&eval, &proof_metrics);

    CoreV1NonlinearProofReport {
        mode: "llmwave-core-v1-nonlinear-proof",
        version: CORE_V1_NONLINEAR_PROOF_VERSION,
        phase: "phase-4-nonlinear-memory-proof-v1",
        verdict,
        objective:
            "prove_or_block_nonlinear_memory_by_combining_writer_density_scale_ladder_heldout_and_baseline_gates",
        writer_evidence,
        ladder_evidence,
        eval_evidence,
        proof_metrics,
        proof_gates,
        claim_boundary: CoreV1NonlinearProofClaimBoundary {
            nonlinear_memory_proof_gate_implemented: true,
            nonlinear_memory_candidate,
            nonlinear_memory_proven,
            llm_ready: false,
            safe_claim: if nonlinear_memory_proven {
                "Core V1 nonlinear memory passed all configured proof gates."
            } else if nonlinear_memory_candidate {
                "Core V1 shows a nonlinear-memory candidate, but proof remains blocked by missing held-out, leakage, or external evidence."
            } else {
                "Core V1 nonlinear-memory proof is implemented but current evidence is insufficient."
            },
            blocked_by,
        },
        next_phase: "phase-5-query-wave-v1",
    }
}

fn falling_byte_points(points: &[nonlinear_memory_eval::NonlinearMemoryLadderPoint]) -> usize {
    points
        .windows(2)
        .filter(|pair| {
            pair[1].fixed_basis_amortized.bytes_per_useful_fact
                < pair[0].fixed_basis_amortized.bytes_per_useful_fact
        })
        .count()
}

fn proof_gates(
    writer: &CoreV1WriterEvidence,
    ladder: &CoreV1LadderEvidence,
    eval: &CoreV1NonlinearEvalEvidence,
    metrics: &CoreV1NonlinearProofMetrics,
) -> Vec<CoreV1NonlinearProofGate> {
    vec![
        gate(
            "writer_beats_raw_dictionary_fixture",
            metrics.writer_beats_raw_dictionary_fixture,
            "core_v1_memory_writer.byte_report.writer_saving_ratio",
        ),
        gate(
            "raw_dictionary_is_not_primary_memory",
            writer.raw_dictionary_is_not_primary_memory,
            "core_v1_memory_writer.claim_boundary.raw_dictionary_is_not_primary_memory",
        ),
        gate(
            "bytes_per_useful_fact_falls_at_three_scale_points",
            metrics.bytes_per_useful_fact_falls_at_three_scale_points,
            "nonlinear_memory_ladder.fixed_basis_amortized.bytes_per_useful_fact",
        ),
        gate(
            "scale_ladder_ready",
            ladder.ladder_ready && ladder.scale_points >= 5,
            "nonlinear_memory_ladder.aggregate.phase1_ready",
        ),
        gate(
            "linear_baseline_compared",
            eval.linear_baseline_compared,
            "nonlinear_memory_eval.claim_boundary.linear_baseline_compared",
        ),
        gate(
            "heldout_quality_bound_to_writer",
            metrics.heldout_quality_bound_to_writer,
            "missing: heldout quality is not yet tied to Phase 3 writer output",
        ),
        gate(
            "external_corpus_present",
            eval.external_corpus_present,
            "missing: no external corpus path passed into Core V1 proof gate",
        ),
        gate(
            "near_duplicate_leakage_control_present",
            metrics.near_duplicate_leakage_control_present,
            "missing: leakage controls must be bound to writer output",
        ),
        gate(
            "role_error_rate_bounded",
            metrics.role_error_rate_bounded,
            "nonlinear_memory_ladder.max_role_error_rate",
        ),
        gate(
            "false_positive_rate_bounded",
            metrics.false_positive_rate_bounded,
            "nonlinear_memory_ladder.max_false_positive_rate",
        ),
    ]
}

fn gate(gate: &'static str, passed: bool, evidence: &'static str) -> CoreV1NonlinearProofGate {
    CoreV1NonlinearProofGate {
        gate,
        passed,
        evidence,
    }
}

fn blocked_by(
    eval: &nonlinear_memory_eval::NonlinearMemoryEvalReport,
    metrics: &CoreV1NonlinearProofMetrics,
) -> Vec<&'static str> {
    let mut blocked = Vec::new();
    if !eval.gates.external_corpus_present {
        blocked.push("external_corpus_missing");
    }
    if !eval.gates.heldout_inference_present || !metrics.heldout_quality_bound_to_writer {
        blocked.push("heldout_quality_not_bound_to_memory_writer");
    }
    if !metrics.near_duplicate_leakage_control_present {
        blocked.push("near_duplicate_leakage_control_missing");
    }
    if !eval.gates.broad_noise_eval_present {
        blocked.push("broad_noise_eval_missing");
    }
    if !eval.claim_boundary.selected_policy_proven {
        blocked.push("selected_policy_not_proven");
    }
    blocked
}
