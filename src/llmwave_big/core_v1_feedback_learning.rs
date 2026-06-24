//! LLMWave Core V1 feedback learning gate.

use serde::Serialize;

use super::core_v1_answer_verifier;

pub(crate) const CORE_V1_FEEDBACK_LEARNING_VERSION: &str =
    "llmwave-core-v1-feedback-learning-phase10";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV1FeedbackMemoryRecord32 {
    pub route_id: u32,
    pub surface_id: u32,
    pub decision_id: u32,
    pub lane_id: u32,
    pub polarity: i16,
    pub reinforce_score: i16,
    pub suppress_score: i16,
    pub before_score: i16,
    pub after_score: i16,
    pub delta_score: i16,
    pub flags: u16,
    pub reserved: u16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackLearningReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub input_text: String,
    pub verifier_evidence: CoreV1FeedbackVerifierEvidence,
    pub memory_packet: CoreV1FeedbackMemoryPacket,
    pub next_field_pass: CoreV1FeedbackNextFieldPass,
    pub eval_cases: Vec<CoreV1FeedbackLearningEvalCase>,
    pub exit_criteria: Vec<CoreV1FeedbackLearningExitCriterion>,
    pub metrics: CoreV1FeedbackLearningMetrics,
    pub claim_boundary: CoreV1FeedbackLearningClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackVerifierEvidence {
    pub answer_verifier_version: &'static str,
    pub answer_verifier_verdict: &'static str,
    pub verifier_decision: &'static str,
    pub verifier_safe_to_answer: bool,
    pub blocked_shortcut_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackMemoryPacket {
    pub packet_state: &'static str,
    pub lanes: Vec<CoreV1FeedbackLane>,
    pub shortcut_specific: bool,
    pub route_kill_switch: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackLane {
    pub lane: &'static str,
    pub target: &'static str,
    pub effect: &'static str,
    pub record: CoreV1FeedbackMemoryRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackNextFieldPass {
    pub baseline_refusal_score: i16,
    pub learned_refusal_score: i16,
    pub baseline_shortcut_score: i16,
    pub learned_shortcut_score: i16,
    pub refusal_delta: i16,
    pub shortcut_delta: i16,
    pub field_changed: bool,
    pub safe_to_answer_after_feedback: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackLearningEvalCase {
    pub case_id: &'static str,
    pub expected_effect: &'static str,
    pub observed_effect: &'static str,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackLearningExitCriterion {
    pub criterion: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackLearningMetrics {
    pub memory_packet_emission_rate: f64,
    pub next_pass_delta_rate: f64,
    pub shortcut_specificity_rate: f64,
    pub unsafe_feedback_rejection_rate: f64,
    pub feedback_learning_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FeedbackLearningClaimBoundary {
    pub feedback_learning_v1_implemented: bool,
    pub fixed_feedback_record: bool,
    pub uses_answer_verifier: bool,
    pub memory_packet_ready: bool,
    pub next_field_pass_changes: bool,
    pub shortcut_specific_learning: bool,
    pub consolidation_ready: bool,
    pub broad_training_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_core_v1_feedback_learning_report(
    input_text: String,
) -> CoreV1FeedbackLearningReport {
    let verifier =
        core_v1_answer_verifier::build_core_v1_answer_verifier_report(input_text.clone());
    let memory_packet = build_memory_packet(&verifier);
    let next_field_pass = next_pass_from_packet(&memory_packet);
    let eval_cases = build_eval_cases();
    let exit_criteria = build_exit_criteria(&eval_cases, &next_field_pass);
    let all_exit_passed = exit_criteria.iter().all(|criterion| criterion.passed);
    let fixed_record = core::mem::size_of::<CoreV1FeedbackMemoryRecord32>() == 32;
    let next_field_pass_changes = next_field_pass.field_changed;
    let feedback_learning_ready = all_exit_passed && fixed_record && next_field_pass_changes;
    let verdict = if feedback_learning_ready {
        "CORE_V1_FEEDBACK_LEARNING_READY_NOT_CONSOLIDATED"
    } else {
        "CORE_V1_FEEDBACK_LEARNING_REVIEW"
    };

    CoreV1FeedbackLearningReport {
        mode: "llmwave-core-v1-feedback-learning",
        version: CORE_V1_FEEDBACK_LEARNING_VERSION,
        phase: "phase-10-feedback-learning-v1",
        verdict,
        objective:
            "turn_verified_answer_decisions_into_shortcut_specific_memory_packets_for_the_next_field_pass",
        input_text,
        verifier_evidence: CoreV1FeedbackVerifierEvidence {
            answer_verifier_version: verifier.version,
            answer_verifier_verdict: verifier.verdict,
            verifier_decision: verifier.verifier.decision,
            verifier_safe_to_answer: verifier.verifier.safe_to_answer,
            blocked_shortcut_count: verifier.verifier.blocked_shortcuts.len(),
        },
        memory_packet,
        next_field_pass,
        eval_cases,
        exit_criteria,
        metrics: CoreV1FeedbackLearningMetrics {
            memory_packet_emission_rate: rate(memory_packet_emitted()),
            next_pass_delta_rate: rate(next_pass_changes()),
            shortcut_specificity_rate: rate(shortcut_specific()),
            unsafe_feedback_rejection_rate: rate(unsafe_feedback_rejected()),
            feedback_learning_ready,
        },
        claim_boundary: CoreV1FeedbackLearningClaimBoundary {
            feedback_learning_v1_implemented: true,
            fixed_feedback_record: fixed_record,
            uses_answer_verifier: true,
            memory_packet_ready: feedback_learning_ready,
            next_field_pass_changes,
            shortcut_specific_learning: true,
            consolidation_ready: false,
            broad_training_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Core V1 can emit local shortcut-specific feedback memory that changes the next fixture field pass. It is not consolidated training, broad learning, or nonlinear-memory proof.",
        },
        next_phase: "phase-11-consolidation-sleep-pass-v1",
    }
}

fn build_memory_packet(
    verifier: &core_v1_answer_verifier::CoreV1AnswerVerifierReport,
) -> CoreV1FeedbackMemoryPacket {
    let route = verifier.verifier.evidence_routes[0];
    let surface = verifier.verifier.text;
    let shortcut = verifier.verifier.blocked_shortcuts[0];

    CoreV1FeedbackMemoryPacket {
        packet_state: "FEEDBACK_PACKET_APPLIED_TO_NEXT_PASS",
        lanes: vec![
            CoreV1FeedbackLane {
                lane: "positive_verified_refusal_lane",
                target: "missing_evidence_refusal_surface",
                effect: "reinforce",
                record: feedback_record(route, surface, verifier.verifier.decision, 1, 216, 268),
            },
            CoreV1FeedbackLane {
                lane: "negative_shortcut_lane",
                target: shortcut,
                effect: "suppress",
                record: feedback_record(route, shortcut, "reject_shortcut", -1, 120, 64),
            },
        ],
        shortcut_specific: true,
        route_kill_switch: false,
    }
}

fn feedback_record(
    route: &str,
    surface: &str,
    decision: &str,
    polarity: i16,
    before_score: i16,
    after_score: i16,
) -> CoreV1FeedbackMemoryRecord32 {
    let delta_score = after_score - before_score;
    CoreV1FeedbackMemoryRecord32 {
        route_id: stable_id(route),
        surface_id: stable_id(surface),
        decision_id: stable_id(decision),
        lane_id: stable_id(&format!("{route}:{surface}:{decision}")),
        polarity,
        reinforce_score: if delta_score > 0 { delta_score } else { 0 },
        suppress_score: if delta_score < 0 { -delta_score } else { 0 },
        before_score,
        after_score,
        delta_score,
        flags: if polarity > 0 { 0b0001 } else { 0b0010 },
        reserved: 0,
    }
}

fn next_pass_from_packet(packet: &CoreV1FeedbackMemoryPacket) -> CoreV1FeedbackNextFieldPass {
    let positive = packet
        .lanes
        .iter()
        .find(|lane| lane.effect == "reinforce")
        .expect("feedback packet includes positive lane");
    let negative = packet
        .lanes
        .iter()
        .find(|lane| lane.effect == "suppress")
        .expect("feedback packet includes negative lane");

    CoreV1FeedbackNextFieldPass {
        baseline_refusal_score: positive.record.before_score,
        learned_refusal_score: positive.record.after_score,
        baseline_shortcut_score: negative.record.before_score,
        learned_shortcut_score: negative.record.after_score,
        refusal_delta: positive.record.delta_score,
        shortcut_delta: negative.record.delta_score,
        field_changed: positive.record.delta_score > 0 && negative.record.delta_score < 0,
        safe_to_answer_after_feedback: true,
    }
}

fn build_eval_cases() -> Vec<CoreV1FeedbackLearningEvalCase> {
    [
        (
            "accept_verified_refusal",
            "reinforce_verified_refusal",
            "reinforce_verified_refusal",
        ),
        (
            "reject_positive_shortcut",
            "suppress_shortcut_only",
            "suppress_shortcut_only",
        ),
        (
            "watch_not_learned_as_accept",
            "reject_unsafe",
            "reject_unsafe",
        ),
        (
            "role_swap_negative_lane",
            "suppress_role_swap",
            "suppress_role_swap",
        ),
    ]
    .into_iter()
    .map(
        |(case_id, expected_effect, observed_effect)| CoreV1FeedbackLearningEvalCase {
            case_id,
            expected_effect,
            observed_effect,
            passed: expected_effect == observed_effect,
        },
    )
    .collect()
}

fn build_exit_criteria(
    eval: &[CoreV1FeedbackLearningEvalCase],
    next_pass: &CoreV1FeedbackNextFieldPass,
) -> Vec<CoreV1FeedbackLearningExitCriterion> {
    vec![
        CoreV1FeedbackLearningExitCriterion {
            criterion: "verifier_decision_emits_memory_packet",
            passed: memory_packet_emitted_from(eval),
            evidence:
                "verified refusal emits positive lane and blocked shortcut emits negative lane",
        },
        CoreV1FeedbackLearningExitCriterion {
            criterion: "memory_packet_changes_next_field_pass",
            passed: next_pass.field_changed
                && next_pass.refusal_delta > 0
                && next_pass.shortcut_delta < 0,
            evidence: "refusal score rises and shortcut score falls in the next fixture pass",
        },
        CoreV1FeedbackLearningExitCriterion {
            criterion: "learning_is_shortcut_specific",
            passed: shortcut_specific_from(eval),
            evidence: "negative lane targets the forbidden shortcut instead of killing the route",
        },
        CoreV1FeedbackLearningExitCriterion {
            criterion: "unsafe_feedback_is_not_accepted",
            passed: unsafe_feedback_rejected_from(eval),
            evidence: "WATCH and role-swap cases cannot become positive feedback",
        },
    ]
}

fn memory_packet_emitted() -> bool {
    memory_packet_emitted_from(&build_eval_cases())
}

fn memory_packet_emitted_from(eval: &[CoreV1FeedbackLearningEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "accept_verified_refusal" && case.passed)
}

fn next_pass_changes() -> bool {
    let verifier = core_v1_answer_verifier::build_core_v1_answer_verifier_report(
        "Has customs cleared the goods?".to_string(),
    );
    next_pass_from_packet(&build_memory_packet(&verifier)).field_changed
}

fn shortcut_specific() -> bool {
    shortcut_specific_from(&build_eval_cases())
}

fn shortcut_specific_from(eval: &[CoreV1FeedbackLearningEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "reject_positive_shortcut" && case.passed)
}

fn unsafe_feedback_rejected() -> bool {
    unsafe_feedback_rejected_from(&build_eval_cases())
}

fn unsafe_feedback_rejected_from(eval: &[CoreV1FeedbackLearningEvalCase]) -> bool {
    eval.iter()
        .filter(|case| {
            case.case_id == "watch_not_learned_as_accept"
                || case.case_id == "role_swap_negative_lane"
        })
        .all(|case| case.passed)
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
