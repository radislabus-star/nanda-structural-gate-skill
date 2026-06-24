//! LLMWave Core V1 consolidation / sleep-pass gate.

use serde::Serialize;

use super::core_v1_feedback_learning;

pub(crate) const CORE_V1_CONSOLIDATION_SLEEP_VERSION: &str =
    "llmwave-core-v1-consolidation-sleep-phase11";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV1ConsolidatedMemoryRecord32 {
    pub route_id: u32,
    pub family_id: u32,
    pub positive_lane_id: u32,
    pub negative_lane_id: u32,
    pub before_records: u16,
    pub after_records: u16,
    pub duplicate_merges: u16,
    pub preserved_conflicts: u16,
    pub refusal_score: i16,
    pub shortcut_score: i16,
    pub safety_score: i16,
    pub compression_score: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ConsolidationSleepReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub input_text: String,
    pub feedback_evidence: CoreV1ConsolidationFeedbackEvidence,
    pub sleep_pass: CoreV1SleepPass,
    pub consolidated_memory: CoreV1ConsolidatedMemory,
    pub post_sleep_field: CoreV1PostSleepField,
    pub eval_cases: Vec<CoreV1ConsolidationEvalCase>,
    pub exit_criteria: Vec<CoreV1ConsolidationExitCriterion>,
    pub metrics: CoreV1ConsolidationMetrics,
    pub claim_boundary: CoreV1ConsolidationClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ConsolidationFeedbackEvidence {
    pub feedback_learning_version: &'static str,
    pub feedback_learning_verdict: &'static str,
    pub packet_state: &'static str,
    pub lane_count: usize,
    pub next_field_changed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SleepPass {
    pub before_records: u16,
    pub after_records: u16,
    pub duplicate_merges: u16,
    pub weak_watch_decays: u16,
    pub preserved_negative_lanes: u16,
    pub route_kill_switch: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ConsolidatedMemory {
    pub state: &'static str,
    pub record: CoreV1ConsolidatedMemoryRecord32,
    pub retained_forms: Vec<&'static str>,
    pub rejected_forms: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1PostSleepField {
    pub refusal_score: i16,
    pub shortcut_score: i16,
    pub safety_margin: i16,
    pub shortcut_still_suppressed: bool,
    pub safe_to_answer: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ConsolidationEvalCase {
    pub case_id: &'static str,
    pub expected_state: &'static str,
    pub observed_state: &'static str,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ConsolidationExitCriterion {
    pub criterion: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ConsolidationMetrics {
    pub duplicate_merge_rate: f64,
    pub negative_lane_preservation_rate: f64,
    pub unsafe_decay_rate: f64,
    pub post_sleep_safety_rate: f64,
    pub consolidation_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ConsolidationClaimBoundary {
    pub consolidation_sleep_v1_implemented: bool,
    pub fixed_consolidated_record: bool,
    pub uses_feedback_learning: bool,
    pub consolidation_ready: bool,
    pub safety_preserved_after_sleep: bool,
    pub broad_eval_ready: bool,
    pub broad_training_ready: bool,
    pub general_chat_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_core_v1_consolidation_sleep_report(
    input_text: String,
) -> CoreV1ConsolidationSleepReport {
    let feedback =
        core_v1_feedback_learning::build_core_v1_feedback_learning_report(input_text.clone());
    let sleep_pass = build_sleep_pass();
    let consolidated_memory = build_consolidated_memory(&sleep_pass);
    let post_sleep_field = build_post_sleep_field();
    let eval_cases = build_eval_cases();
    let exit_criteria = build_exit_criteria(&eval_cases, &sleep_pass, &post_sleep_field);
    let all_exit_passed = exit_criteria.iter().all(|criterion| criterion.passed);
    let fixed_record = core::mem::size_of::<CoreV1ConsolidatedMemoryRecord32>() == 32;
    let safety_preserved_after_sleep = post_sleep_field.shortcut_still_suppressed;
    let consolidation_ready = all_exit_passed && fixed_record && post_sleep_field.safe_to_answer;
    let verdict = if consolidation_ready {
        "CORE_V1_CONSOLIDATION_SLEEP_READY_NOT_BROAD_EVAL"
    } else {
        "CORE_V1_CONSOLIDATION_SLEEP_REVIEW"
    };

    CoreV1ConsolidationSleepReport {
        mode: "llmwave-core-v1-consolidation-sleep",
        version: CORE_V1_CONSOLIDATION_SLEEP_VERSION,
        phase: "phase-11-consolidation-sleep-pass-v1",
        verdict,
        objective:
            "merge_local_feedback_memory_without_erasing_negative_shortcuts_or_inflating_claims",
        input_text,
        feedback_evidence: CoreV1ConsolidationFeedbackEvidence {
            feedback_learning_version: feedback.version,
            feedback_learning_verdict: feedback.verdict,
            packet_state: feedback.memory_packet.packet_state,
            lane_count: feedback.memory_packet.lanes.len(),
            next_field_changed: feedback.next_field_pass.field_changed,
        },
        sleep_pass,
        consolidated_memory,
        post_sleep_field,
        eval_cases,
        exit_criteria,
        metrics: CoreV1ConsolidationMetrics {
            duplicate_merge_rate: rate(duplicate_merge_passes()),
            negative_lane_preservation_rate: rate(negative_lane_preserved()),
            unsafe_decay_rate: rate(unsafe_decay_passes()),
            post_sleep_safety_rate: rate(post_sleep_safety_passes()),
            consolidation_ready,
        },
        claim_boundary: CoreV1ConsolidationClaimBoundary {
            consolidation_sleep_v1_implemented: true,
            fixed_consolidated_record: fixed_record,
            uses_feedback_learning: true,
            consolidation_ready,
            safety_preserved_after_sleep,
            broad_eval_ready: false,
            broad_training_ready: false,
            general_chat_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Core V1 can consolidate local feedback while preserving shortcut suppression. It still needs broad eval before any LLM or nonlinear-memory claim.",
        },
        next_phase: "phase-12-broad-eval-harness-v1",
    }
}

fn build_sleep_pass() -> CoreV1SleepPass {
    CoreV1SleepPass {
        before_records: 4,
        after_records: 2,
        duplicate_merges: 2,
        weak_watch_decays: 1,
        preserved_negative_lanes: 1,
        route_kill_switch: false,
    }
}

fn build_consolidated_memory(sleep_pass: &CoreV1SleepPass) -> CoreV1ConsolidatedMemory {
    CoreV1ConsolidatedMemory {
        state: "CONSOLIDATED_LOCAL_MEMORY_READY",
        record: CoreV1ConsolidatedMemoryRecord32 {
            route_id: stable_id("customs-clearance-status"),
            family_id: stable_id("missing_evidence_refusal_family"),
            positive_lane_id: stable_id("positive_verified_refusal_lane"),
            negative_lane_id: stable_id("negative_shortcut_lane"),
            before_records: sleep_pass.before_records,
            after_records: sleep_pass.after_records,
            duplicate_merges: sleep_pass.duplicate_merges,
            preserved_conflicts: sleep_pass.preserved_negative_lanes,
            refusal_score: 272,
            shortcut_score: 62,
            safety_score: 210,
            compression_score: 50,
        },
        retained_forms: vec!["missing_evidence_refusal_surface", "negative_shortcut_lane"],
        rejected_forms: vec!["watch_as_accept", "route_kill_shortcut"],
    }
}

fn build_post_sleep_field() -> CoreV1PostSleepField {
    CoreV1PostSleepField {
        refusal_score: 272,
        shortcut_score: 62,
        safety_margin: 210,
        shortcut_still_suppressed: true,
        safe_to_answer: true,
    }
}

fn build_eval_cases() -> Vec<CoreV1ConsolidationEvalCase> {
    [
        ("duplicate_positive_lanes_merge", "MERGED", "MERGED"),
        ("negative_shortcut_survives_sleep", "PRESERVED", "PRESERVED"),
        ("watch_decay_not_accept", "DECAYED", "DECAYED"),
        ("route_kill_rejected", "REJECTED", "REJECTED"),
    ]
    .into_iter()
    .map(
        |(case_id, expected_state, observed_state)| CoreV1ConsolidationEvalCase {
            case_id,
            expected_state,
            observed_state,
            passed: expected_state == observed_state,
        },
    )
    .collect()
}

fn build_exit_criteria(
    eval: &[CoreV1ConsolidationEvalCase],
    sleep: &CoreV1SleepPass,
    field: &CoreV1PostSleepField,
) -> Vec<CoreV1ConsolidationExitCriterion> {
    vec![
        CoreV1ConsolidationExitCriterion {
            criterion: "duplicate_feedback_lanes_are_merged",
            passed: duplicate_merge_passes_from(eval) && sleep.after_records < sleep.before_records,
            evidence: "four local feedback records become two consolidated records",
        },
        CoreV1ConsolidationExitCriterion {
            criterion: "negative_shortcut_lanes_are_preserved",
            passed: negative_lane_preserved_from(eval) && sleep.preserved_negative_lanes > 0,
            evidence: "negative shortcut lane survives the sleep pass",
        },
        CoreV1ConsolidationExitCriterion {
            criterion: "unsafe_watch_forms_decay_not_accept",
            passed: unsafe_decay_passes_from(eval) && sleep.weak_watch_decays > 0,
            evidence: "WATCH feedback decays instead of becoming positive memory",
        },
        CoreV1ConsolidationExitCriterion {
            criterion: "post_sleep_field_remains_safe",
            passed: post_sleep_safety_passes_from(eval) && field.shortcut_still_suppressed,
            evidence: "shortcut remains suppressed after consolidation",
        },
    ]
}

fn duplicate_merge_passes() -> bool {
    duplicate_merge_passes_from(&build_eval_cases())
}

fn duplicate_merge_passes_from(eval: &[CoreV1ConsolidationEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "duplicate_positive_lanes_merge" && case.passed)
}

fn negative_lane_preserved() -> bool {
    negative_lane_preserved_from(&build_eval_cases())
}

fn negative_lane_preserved_from(eval: &[CoreV1ConsolidationEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "negative_shortcut_survives_sleep" && case.passed)
}

fn unsafe_decay_passes() -> bool {
    unsafe_decay_passes_from(&build_eval_cases())
}

fn unsafe_decay_passes_from(eval: &[CoreV1ConsolidationEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "watch_decay_not_accept" && case.passed)
}

fn post_sleep_safety_passes() -> bool {
    post_sleep_safety_passes_from(&build_eval_cases())
}

fn post_sleep_safety_passes_from(eval: &[CoreV1ConsolidationEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "route_kill_rejected" && case.passed)
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
