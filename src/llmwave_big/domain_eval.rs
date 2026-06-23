//! Small-domain LLMWave eval over artifact QA, hot chat, and memory density.

use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use super::nonlinear_memory_eval::{self, NonlinearProofPolicyKind};
use super::training;

pub(crate) const DOMAIN_EVAL_VERSION: &str = "llmwave-big-v1909-domain-eval";

#[derive(Serialize, Clone)]
pub(crate) struct DomainEvalReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub artifact_eval: DomainArtifactEvalSummary,
    pub hot_chat_eval: DomainHotChatEvalSummary,
    pub nonlinear_memory_eval: DomainNonlinearMemoryEvalSummary,
    pub metrics: DomainEvalMetrics,
    pub claim_boundary: DomainEvalClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct DomainArtifactEvalSummary {
    pub verdict: String,
    pub answer_accuracy: f32,
    pub false_positive_rate: f32,
    pub false_negative_rate: f32,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DomainHotChatEvalSummary {
    pub verdict: String,
    pub memory_lift_observed: bool,
    pub false_safe_before_learning: bool,
    pub pass_rate: f32,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DomainNonlinearMemoryEvalSummary {
    pub verdict: String,
    pub selected_policy: String,
    pub selected_policy_proven: bool,
    pub scale_amortized_nonlinear_memory_proven: bool,
    pub nonlinear_memory_proven: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DomainEvalMetrics {
    pub component_count: usize,
    pub passed_components: usize,
    pub pass_rate: f32,
    pub answer_accuracy: f32,
    pub false_positive_rate: f32,
    pub memory_lift_observed: bool,
    pub scale_amortized_memory_ready: bool,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct DomainEvalClaimBoundary {
    pub small_domain_llmwave_eval_implemented: bool,
    pub small_domain_llmwave_ready: bool,
    pub artifact_grounded_qa_ready: bool,
    pub scripted_hot_multi_turn_ready: bool,
    pub scale_amortized_nonlinear_memory_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub general_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_domain_eval_report(
    artifact_path: &Path,
    ask_suite_path: &Path,
    hot_pack_path: &Path,
    chat_script_path: &Path,
    chat_memory_path: &Path,
    nonlinear_corpus_path: &Path,
    top_k: usize,
) -> Result<DomainEvalReport> {
    let artifact = training::eval_training_artifact(artifact_path, ask_suite_path, top_k)?;
    let chat = training::eval_hot_chat_session(
        hot_pack_path,
        artifact_path,
        chat_memory_path,
        chat_script_path,
        top_k,
    )?;
    let nonlinear = nonlinear_memory_eval::build_nonlinear_memory_eval_report(
        Some(nonlinear_corpus_path),
        NonlinearProofPolicyKind::ScaleAmortized,
    )?;

    let artifact_passed = artifact.verdict == "ARTIFACT_ASK_EVAL_PASS_NOT_GENERAL_LLM"
        && artifact.metrics.false_positive_rate == 0.0;
    let hot_chat_passed = chat.verdict == "HOT_CHAT_EVAL_PASS_NOT_GENERAL_LLM"
        && chat.metrics.memory_lift_observed
        && !chat.metrics.false_safe_before_learning;
    let nonlinear_passed = nonlinear.proof_policy.selected_policy_proven
        && nonlinear
            .claim_boundary
            .scale_amortized_nonlinear_memory_proven;
    let passed_components = [artifact_passed, hot_chat_passed, nonlinear_passed]
        .into_iter()
        .filter(|passed| *passed)
        .count();
    let small_domain_ready = passed_components == 3;
    let state = if small_domain_ready {
        "SMALL_DOMAIN_LLMWAVE_EVAL_PASS_NOT_BROAD_LLM"
    } else {
        "SMALL_DOMAIN_LLMWAVE_EVAL_REVIEW"
    };

    Ok(DomainEvalReport {
        mode: "llmwave-big-domain-eval",
        version: DOMAIN_EVAL_VERSION,
        verdict: state,
        artifact_eval: DomainArtifactEvalSummary {
            verdict: artifact.verdict.to_string(),
            answer_accuracy: artifact.metrics.answer_accuracy,
            false_positive_rate: artifact.metrics.false_positive_rate,
            false_negative_rate: artifact.metrics.false_negative_rate,
            passed: artifact_passed,
        },
        hot_chat_eval: DomainHotChatEvalSummary {
            verdict: chat.verdict.to_string(),
            memory_lift_observed: chat.metrics.memory_lift_observed,
            false_safe_before_learning: chat.metrics.false_safe_before_learning,
            pass_rate: chat.metrics.pass_rate,
            passed: hot_chat_passed,
        },
        nonlinear_memory_eval: DomainNonlinearMemoryEvalSummary {
            verdict: nonlinear.verdict.to_string(),
            selected_policy: nonlinear.proof_policy.selected.to_string(),
            selected_policy_proven: nonlinear.proof_policy.selected_policy_proven,
            scale_amortized_nonlinear_memory_proven: nonlinear
                .claim_boundary
                .scale_amortized_nonlinear_memory_proven,
            nonlinear_memory_proven: nonlinear.claim_boundary.nonlinear_memory_proven,
            passed: nonlinear_passed,
        },
        metrics: DomainEvalMetrics {
            component_count: 3,
            passed_components,
            pass_rate: ratio(passed_components, 3),
            answer_accuracy: artifact.metrics.answer_accuracy,
            false_positive_rate: artifact.metrics.false_positive_rate,
            memory_lift_observed: chat.metrics.memory_lift_observed,
            scale_amortized_memory_ready: nonlinear_passed,
            state,
        },
        claim_boundary: DomainEvalClaimBoundary {
            small_domain_llmwave_eval_implemented: true,
            small_domain_llmwave_ready: small_domain_ready,
            artifact_grounded_qa_ready: artifact_passed,
            scripted_hot_multi_turn_ready: hot_chat_passed,
            scale_amortized_nonlinear_memory_ready: nonlinear_passed,
            broad_chat_llm_ready: false,
            general_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: if small_domain_ready {
                "LLMWave-Big passed a small-domain eval combining artifact QA, scripted hot-memory chat, and scale-amortized memory density; this is not broad LLM readiness."
            } else {
                "LLMWave-Big small-domain eval is incomplete; do not claim model readiness."
            },
        },
    })
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        ((numerator as f32 / denominator as f32) * 10_000.0).round() / 10_000.0
    }
}
