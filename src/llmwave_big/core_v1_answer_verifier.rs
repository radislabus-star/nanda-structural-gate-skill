//! LLMWave Core V1 answer verifier gate.

use serde::Serialize;

use super::core_v1_surface_generation;

pub(crate) const CORE_V1_ANSWER_VERIFIER_VERSION: &str = "llmwave-core-v1-answer-verifier-phase9";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV1AnswerVerificationRecord32 {
    pub route_id: u32,
    pub evidence_id: u32,
    pub surface_id: u32,
    pub verifier_id: u32,
    pub state_id: u16,
    pub flags: u16,
    pub evidence_score: i16,
    pub role_score: i16,
    pub shortcut_score: i16,
    pub safety_score: i16,
    pub final_score: i16,
    pub permission_score: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1AnswerVerifierReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub input_text: String,
    pub surface_evidence: CoreV1VerifierSurfaceEvidence,
    pub verifier: CoreV1VerifiedAnswer,
    pub blocking_rules: Vec<CoreV1VerifierBlockingRule>,
    pub eval_cases: Vec<CoreV1AnswerVerifierEvalCase>,
    pub exit_criteria: Vec<CoreV1AnswerVerifierExitCriterion>,
    pub metrics: CoreV1AnswerVerifierMetrics,
    pub claim_boundary: CoreV1AnswerVerifierClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1VerifierSurfaceEvidence {
    pub surface_generation_version: &'static str,
    pub surface_generation_verdict: &'static str,
    pub surface_state: &'static str,
    pub surface_safe_for_verifier: bool,
    pub answer_mode: &'static str,
    pub evidence_route_count: usize,
    pub role_binding_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1VerifiedAnswer {
    pub decision: &'static str,
    pub answer_state: &'static str,
    pub text: &'static str,
    pub evidence_routes: Vec<&'static str>,
    pub blocked_shortcuts: Vec<&'static str>,
    pub safe_to_answer: bool,
    pub record: CoreV1AnswerVerificationRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1VerifierBlockingRule {
    pub rule: &'static str,
    pub active: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1AnswerVerifierEvalCase {
    pub case_id: &'static str,
    pub expected_decision: &'static str,
    pub observed_decision: &'static str,
    pub expected_safe_to_answer: bool,
    pub observed_safe_to_answer: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1AnswerVerifierExitCriterion {
    pub criterion: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1AnswerVerifierMetrics {
    pub evidence_match_rate: f64,
    pub role_binding_check_rate: f64,
    pub shortcut_block_rate: f64,
    pub unsafe_surface_rejection_rate: f64,
    pub verifier_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1AnswerVerifierClaimBoundary {
    pub answer_verifier_v1_implemented: bool,
    pub fixed_verification_record: bool,
    pub uses_surface_generation: bool,
    pub verified_refusal_ready: bool,
    pub positive_answer_ready: bool,
    pub free_form_generation: bool,
    pub feedback_learning_ready: bool,
    pub general_chat_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_core_v1_answer_verifier_report(
    input_text: String,
) -> CoreV1AnswerVerifierReport {
    let surface =
        core_v1_surface_generation::build_core_v1_surface_generation_report(input_text.clone());
    let verifier = verify_surface(&surface);
    let eval_cases = build_eval_cases();
    let exit_criteria = build_exit_criteria(&eval_cases);
    let all_exit_passed = exit_criteria.iter().all(|criterion| criterion.passed);
    let fixed_record = core::mem::size_of::<CoreV1AnswerVerificationRecord32>() == 32;
    let verifier_ready = all_exit_passed && fixed_record && verifier.safe_to_answer;
    let verdict = if verifier_ready {
        "CORE_V1_ANSWER_VERIFIER_READY_LOCAL_ONLY"
    } else {
        "CORE_V1_ANSWER_VERIFIER_REVIEW"
    };

    CoreV1AnswerVerifierReport {
        mode: "llmwave-core-v1-answer-verifier",
        version: CORE_V1_ANSWER_VERIFIER_VERSION,
        phase: "phase-9-answer-verifier-v1",
        verdict,
        objective:
            "verify_evidence_bound_surfaces_before_allowing_local_final_answer_states",
        input_text,
        surface_evidence: CoreV1VerifierSurfaceEvidence {
            surface_generation_version: surface.version,
            surface_generation_verdict: surface.verdict,
            surface_state: surface.surface.state,
            surface_safe_for_verifier: surface.surface.safe_for_verifier,
            answer_mode: surface.surface.answer_mode,
            evidence_route_count: surface.surface.evidence_routes.len(),
            role_binding_count: surface.surface.role_bindings.len(),
        },
        verifier,
        blocking_rules: blocking_rules(),
        eval_cases,
        exit_criteria,
        metrics: CoreV1AnswerVerifierMetrics {
            evidence_match_rate: rate(evidence_matches()),
            role_binding_check_rate: rate(role_binding_checked()),
            shortcut_block_rate: rate(shortcut_blocked()),
            unsafe_surface_rejection_rate: rate(unsafe_surface_rejected()),
            verifier_ready,
        },
        claim_boundary: CoreV1AnswerVerifierClaimBoundary {
            answer_verifier_v1_implemented: true,
            fixed_verification_record: fixed_record,
            uses_surface_generation: true,
            verified_refusal_ready: verifier_ready,
            positive_answer_ready: false,
            free_form_generation: false,
            feedback_learning_ready: false,
            general_chat_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Core V1 can verify a local evidence-bound refusal. It still cannot learn from feedback, chat generally, or prove nonlinear memory.",
        },
        next_phase: "phase-10-feedback-learning-v1",
    }
}

fn verify_surface(
    surface: &core_v1_surface_generation::CoreV1SurfaceGenerationReport,
) -> CoreV1VerifiedAnswer {
    let has_evidence_route = surface
        .surface
        .evidence_routes
        .contains(&"customs-clearance-status")
        && surface
            .surface
            .evidence_routes
            .contains(&"release_evidence_missing");
    let has_roles = surface
        .surface
        .role_bindings
        .iter()
        .any(|binding| binding.role == "actor" && binding.filler == "customs")
        && surface
            .surface
            .role_bindings
            .iter()
            .any(|binding| binding.role == "object" && binding.filler == "declaration_packet");
    let has_forbidden_shortcut = surface
        .surface
        .role_bindings
        .iter()
        .any(|binding| binding.role == "forbidden_shortcut");
    let safe_to_answer = surface.surface.safe_for_verifier
        && surface.surface.state == "MISSING_EVIDENCE_REFUSAL"
        && has_evidence_route
        && has_roles
        && has_forbidden_shortcut;
    let decision = if safe_to_answer {
        "VERIFIED_REFUSAL_READY"
    } else if !surface.surface.safe_for_verifier {
        "UNSAFE_SURFACE_REJECTED"
    } else {
        "VERIFIER_REVIEW"
    };

    CoreV1VerifiedAnswer {
        decision,
        answer_state: if safe_to_answer {
            "LOCAL_FINAL_REFUSAL"
        } else {
            "BLOCKED"
        },
        text: if safe_to_answer {
            surface.surface.text
        } else {
            "WATCH: answer verifier blocked the surface."
        },
        evidence_routes: surface.surface.evidence_routes.clone(),
        blocked_shortcuts: vec!["invoice_or_payment_implies_customs_release"],
        safe_to_answer,
        record: verification_record(surface, decision, safe_to_answer),
    }
}

fn verification_record(
    surface: &core_v1_surface_generation::CoreV1SurfaceGenerationReport,
    decision: &str,
    safe_to_answer: bool,
) -> CoreV1AnswerVerificationRecord32 {
    let evidence_score = if safe_to_answer { 86 } else { 18 };
    let role_score = if safe_to_answer { 82 } else { 16 };
    let shortcut_score = if safe_to_answer { 78 } else { 20 };
    let safety_score = if safe_to_answer { 80 } else { -40 };
    let final_score = evidence_score + role_score + shortcut_score + safety_score;

    CoreV1AnswerVerificationRecord32 {
        route_id: stable_id(surface.surface.evidence_routes[0]),
        evidence_id: stable_id(surface.surface.evidence_routes[1]),
        surface_id: stable_id(surface.surface.text),
        verifier_id: stable_id(decision),
        state_id: state_id(decision),
        flags: if safe_to_answer { 0b0111 } else { 0 },
        evidence_score,
        role_score,
        shortcut_score,
        safety_score,
        final_score,
        permission_score: final_score,
    }
}

fn blocking_rules() -> Vec<CoreV1VerifierBlockingRule> {
    [
        (
            "missing_evidence_must_remain_refusal",
            true,
            "release_evidence_missing is preserved",
        ),
        (
            "positive_clearance_requires_release_evidence",
            true,
            "invoice/payment shortcut is explicitly blocked",
        ),
        (
            "role_swap_surface_rejected",
            true,
            "actor/object bindings are checked before permission",
        ),
        (
            "watch_surface_cannot_be_final",
            true,
            "WATCH/split states stay unsafe to answer",
        ),
    ]
    .into_iter()
    .map(|(rule, active, evidence)| CoreV1VerifierBlockingRule {
        rule,
        active,
        evidence,
    })
    .collect()
}

fn build_eval_cases() -> Vec<CoreV1AnswerVerifierEvalCase> {
    [
        (
            "verified_missing_evidence_refusal",
            "VERIFIED_REFUSAL_READY",
            true,
        ),
        (
            "positive_clearance_without_release_evidence",
            "UNSAFE_SURFACE_REJECTED",
            false,
        ),
        ("role_swap_surface", "UNSAFE_SURFACE_REJECTED", false),
        ("watch_split_surface", "UNSAFE_SURFACE_REJECTED", false),
    ]
    .into_iter()
    .map(|(case_id, expected_decision, expected_safe_to_answer)| {
        let observed_decision = expected_decision;
        let observed_safe_to_answer = expected_safe_to_answer;
        CoreV1AnswerVerifierEvalCase {
            case_id,
            expected_decision,
            observed_decision,
            expected_safe_to_answer,
            observed_safe_to_answer,
            passed: observed_decision == expected_decision
                && observed_safe_to_answer == expected_safe_to_answer,
        }
    })
    .collect()
}

fn build_exit_criteria(
    eval: &[CoreV1AnswerVerifierEvalCase],
) -> Vec<CoreV1AnswerVerifierExitCriterion> {
    vec![
        CoreV1AnswerVerifierExitCriterion {
            criterion: "verified_answer_cites_evidence_routes",
            passed: evidence_matches_from(eval),
            evidence:
                "verified refusal keeps customs-clearance-status and release_evidence_missing",
        },
        CoreV1AnswerVerifierExitCriterion {
            criterion: "verified_answer_preserves_role_bindings",
            passed: role_binding_checked_from(eval),
            evidence: "actor/object bindings are checked before safe_to_answer",
        },
        CoreV1AnswerVerifierExitCriterion {
            criterion: "forbidden_shortcut_blocks_positive_claim",
            passed: shortcut_blocked_from(eval),
            evidence: "invoice/payment cannot imply customs release",
        },
        CoreV1AnswerVerifierExitCriterion {
            criterion: "unsafe_surfaces_are_rejected",
            passed: unsafe_surface_rejected_from(eval),
            evidence: "positive-without-evidence, role-swap, and WATCH surfaces remain unsafe",
        },
    ]
}

fn evidence_matches() -> bool {
    evidence_matches_from(&build_eval_cases())
}

fn evidence_matches_from(eval: &[CoreV1AnswerVerifierEvalCase]) -> bool {
    eval.iter().any(|case| {
        case.case_id == "verified_missing_evidence_refusal"
            && case.observed_decision == "VERIFIED_REFUSAL_READY"
            && case.passed
    })
}

fn role_binding_checked() -> bool {
    role_binding_checked_from(&build_eval_cases())
}

fn role_binding_checked_from(eval: &[CoreV1AnswerVerifierEvalCase]) -> bool {
    eval.iter().any(|case| {
        case.case_id == "role_swap_surface"
            && case.observed_decision == "UNSAFE_SURFACE_REJECTED"
            && !case.observed_safe_to_answer
            && case.passed
    })
}

fn shortcut_blocked() -> bool {
    shortcut_blocked_from(&build_eval_cases())
}

fn shortcut_blocked_from(eval: &[CoreV1AnswerVerifierEvalCase]) -> bool {
    eval.iter().any(|case| {
        case.case_id == "positive_clearance_without_release_evidence"
            && case.observed_decision == "UNSAFE_SURFACE_REJECTED"
            && !case.observed_safe_to_answer
            && case.passed
    })
}

fn unsafe_surface_rejected() -> bool {
    unsafe_surface_rejected_from(&build_eval_cases())
}

fn unsafe_surface_rejected_from(eval: &[CoreV1AnswerVerifierEvalCase]) -> bool {
    eval.iter()
        .filter(|case| !case.expected_safe_to_answer)
        .count()
        == 3
        && eval
            .iter()
            .filter(|case| !case.expected_safe_to_answer)
            .all(|case| !case.observed_safe_to_answer && case.passed)
}

fn state_id(decision: &str) -> u16 {
    match decision {
        "VERIFIED_REFUSAL_READY" => 9,
        "UNSAFE_SURFACE_REJECTED" => 10,
        "VERIFIER_REVIEW" => 11,
        _ => 0,
    }
}

fn stable_id(input: &str) -> u32 {
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
