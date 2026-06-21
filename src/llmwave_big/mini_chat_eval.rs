//! Mini chat eval over the constrained LLMWave-Big core chain.

use serde::Serialize;

use super::{dialogue_state, open_surface_generation, reasoning_field, schema_memory_growth};

pub(crate) const MINI_CHAT_EVAL_VERSION: &str = "llmwave-big-v950-mini-chat-eval";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct MiniChatEvalCase32 {
    pub case_id: u16,
    pub source_layer_id: u16,
    pub expected_state_id: u16,
    pub observed_state_id: u16,
    pub support_score: i16,
    pub anti_score: i16,
    pub final_score: i16,
    pub passed: u16,
    pub flags: u16,
    pub reserved: u16,
    pub reserved2: u32,
    pub reserved3: u32,
    pub reserved4: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct MiniChatEvalReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub dialogue_bridge_state: &'static str,
    pub evaluation_scope: &'static str,
    pub sample_question: &'static str,
    pub sample_answer: &'static str,
    pub eval_cases: Vec<MiniChatEvalCaseReport>,
    pub metrics: MiniChatEvalMetrics,
    pub claim_boundary: MiniChatEvalClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct MiniChatEvalCaseReport {
    pub case_id: &'static str,
    pub kind: &'static str,
    pub input: &'static str,
    pub expected_behavior: &'static str,
    pub observed_behavior: &'static str,
    pub passed: bool,
    pub record: MiniChatEvalCase32,
}

#[derive(Serialize, Clone)]
pub(crate) struct MiniChatEvalMetrics {
    pub case_count: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub grounded_answer_rate: f32,
    pub unsupported_reject_rate: f32,
    pub route_splice_reject_rate: f32,
    pub one_off_schema_reject_rate: f32,
    pub surface_exact_rate: f32,
    pub broad_generalization_score: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct MiniChatEvalClaimBoundary {
    pub mini_chat_eval_implemented: bool,
    pub mini_chat_candidate: bool,
    pub fixed_eval_case_records: bool,
    pub uses_dialogue_state_bridge: bool,
    pub full_llm_ready: bool,
    pub multi_turn_chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub broad_reasoning_proven: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

struct CaseSpec {
    numeric_id: u16,
    case_id: &'static str,
    kind: &'static str,
    input: &'static str,
    expected_behavior: &'static str,
    observed_behavior: &'static str,
    passed: bool,
    support_score: i16,
    anti_score: i16,
}

pub(crate) fn build_mini_chat_eval_report() -> MiniChatEvalReport {
    let dialogue = dialogue_state::build_dialogue_state_report();
    let reasoning = reasoning_field::build_reasoning_field_report();
    let surface = open_surface_generation::build_open_surface_generation_report();
    let growth = schema_memory_growth::build_schema_memory_growth_report();

    let grounded_answer_passed = dialogue.verdict == "DIALOGUE_STATE_READY_NOT_CHAT"
        && dialogue.constrained_answer.contains("Not proven")
        && dialogue.constrained_answer.contains("declaration evidence");
    let unsupported_reject_passed =
        dialogue.trap.rejected && dialogue.metrics.unsupported_answer_reject_rate == 1.0;
    let route_splice_reject_passed =
        surface.trap.rejected && surface.generation_metrics.trap_reject_rate == 1.0;
    let one_off_schema_reject_passed =
        growth.negative_control.rejected && growth.memory_metrics.false_promotion_rate == 0.0;
    let surface_exact_passed = surface.generation_metrics.exact_surface
        && surface.materialized_surface == "Honglu issued invoice PI-03 to Rustrade"
        && reasoning.premise_surface == surface.materialized_surface;

    let eval_cases = vec![
        build_case(CaseSpec {
            numeric_id: 1,
            case_id: "grounded_clearance_answer",
            kind: "answer",
            input: "Has customs cleared the goods?",
            expected_behavior: "answer with not-proven boundary",
            observed_behavior: dialogue.constrained_answer,
            passed: grounded_answer_passed,
            support_score: 74,
            anti_score: 12,
        }),
        build_case(CaseSpec {
            numeric_id: 2,
            case_id: "unsupported_clearance",
            kind: "refusal",
            input: "Yes, customs cleared the goods.",
            expected_behavior: "reject unsupported certainty",
            observed_behavior: dialogue.trap.reason,
            passed: unsupported_reject_passed,
            support_score: 28,
            anti_score: 62,
        }),
        build_case(CaseSpec {
            numeric_id: 3,
            case_id: "route_splice_surface",
            kind: "anti-wave",
            input: "Honglu paid invoice PI-03 to Rustrade",
            expected_behavior: "reject route-spliced verb",
            observed_behavior: surface.trap.reason,
            passed: route_splice_reject_passed,
            support_score: 30,
            anti_score: 58,
        }),
        build_case(CaseSpec {
            numeric_id: 4,
            case_id: "one_off_schema_noise",
            kind: "memory",
            input: "warehouse signs invoice",
            expected_behavior: "reject one-off schema promotion",
            observed_behavior: "insufficient repeated evidence",
            passed: one_off_schema_reject_passed,
            support_score: 22,
            anti_score: 44,
        }),
        build_case(CaseSpec {
            numeric_id: 5,
            case_id: "exact_constrained_surface",
            kind: "generation",
            input: "supplier-docs schema",
            expected_behavior: "materialize exact constrained surface",
            observed_behavior: surface.materialized_surface,
            passed: surface_exact_passed,
            support_score: 82,
            anti_score: 6,
        }),
    ];

    let passed_cases = eval_cases.iter().filter(|case| case.passed).count();
    let failed_cases = eval_cases.len() - passed_cases;
    let state = if failed_cases == 0 && dialogue.verdict == "DIALOGUE_STATE_READY_NOT_CHAT" {
        "MINI_CHAT_EVAL_PASS_NOT_GENERAL_LLM"
    } else {
        "MINI_CHAT_EVAL_REVIEW"
    };

    MiniChatEvalReport {
        mode: "llmwave-big-mini-chat-eval",
        version: MINI_CHAT_EVAL_VERSION,
        roadmap_block: "v861-v950",
        verdict: state,
        dialogue_bridge_state: dialogue.verdict,
        evaluation_scope: "embedded_fixture_chain_only",
        sample_question: dialogue.user_question,
        sample_answer: dialogue.constrained_answer,
        eval_cases,
        metrics: MiniChatEvalMetrics {
            case_count: 5,
            passed_cases,
            failed_cases,
            grounded_answer_rate: rate(grounded_answer_passed),
            unsupported_reject_rate: rate(unsupported_reject_passed),
            route_splice_reject_rate: rate(route_splice_reject_passed),
            one_off_schema_reject_rate: rate(one_off_schema_reject_passed),
            surface_exact_rate: rate(surface_exact_passed),
            broad_generalization_score: 0.0,
            state,
        },
        claim_boundary: MiniChatEvalClaimBoundary {
            mini_chat_eval_implemented: true,
            mini_chat_candidate: failed_cases == 0,
            fixed_eval_case_records: core::mem::size_of::<MiniChatEvalCase32>() == 32,
            uses_dialogue_state_bridge: true,
            full_llm_ready: false,
            multi_turn_chat_ready: false,
            external_corpus_loaded: false,
            broad_reasoning_proven: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "The embedded LLMWave-Big chain passes a small constrained answer/refusal eval, but is not a general LLM or chat model",
        },
    }
}

fn build_case(spec: CaseSpec) -> MiniChatEvalCaseReport {
    MiniChatEvalCaseReport {
        case_id: spec.case_id,
        kind: spec.kind,
        input: spec.input,
        expected_behavior: spec.expected_behavior,
        observed_behavior: spec.observed_behavior,
        passed: spec.passed,
        record: MiniChatEvalCase32 {
            case_id: spec.numeric_id,
            source_layer_id: 860,
            expected_state_id: spec.numeric_id + 900,
            observed_state_id: spec.numeric_id + 900,
            support_score: spec.support_score,
            anti_score: spec.anti_score,
            final_score: spec.support_score - spec.anti_score,
            passed: u16::from(spec.passed),
            flags: if spec.passed { 1 } else { 0 },
            reserved: 0,
            reserved2: 0,
            reserved3: 0,
            reserved4: 0,
        },
    }
}

fn rate(passed: bool) -> f32 {
    if passed {
        1.0
    } else {
        0.0
    }
}
