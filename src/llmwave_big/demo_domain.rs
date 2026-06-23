//! One-command small-domain LLMWave demo.
//!
//! This is cold orchestration code. It intentionally writes a tiny reproducible
//! domain corpus, builds a training artifact, packs the hot core, and runs the
//! existing eval gates. It does not claim broad LLM readiness.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::domain_eval::{self, DomainEvalReport};
use super::training::{self, HotChatEvalReport, HotPackReport, TrainingCompileReport};

pub(crate) const DEMO_DOMAIN_VERSION: &str = "llmwave-big-v1910-demo-domain";

const DEMO_CORPUS: &str = r#"Honglu issues invoice. invoice requires payment.
Payment supports customs declaration. declaration requires evidence.
Honglu issues invoice. invoice requires payment.
evidence blocks unsupported answer.
"#;

const DEMO_ASK_EVAL: &str = r#"{"cases":[
  {"id":"requires","query":"what does invoice require","expected_contains":"invoice requires payment","expected_safe_to_answer":true},
  {"id":"unknown","query":"moonlight customs route","expected_contains":"","expected_safe_to_answer":false}
]}
"#;

const DEMO_CHAT_SCRIPT: &str = r#"ask broker requires invoice
learn accept: broker | requires | invoice
ask broker requires invoice
exit
"#;

#[derive(Serialize, Clone)]
pub(crate) struct DemoDomainReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub demo_dir: String,
    pub files: DemoDomainFiles,
    pub steps: DemoDomainSteps,
    pub metrics: DemoDomainMetrics,
    pub claim_boundary: DemoDomainClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoDomainFiles {
    pub corpus: String,
    pub artifact: String,
    pub ask_suite: String,
    pub hot_pack: String,
    pub chat_script: String,
    pub chat_eval_memory: String,
    pub domain_chat_memory: String,
    pub nonlinear_corpus: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoDomainSteps {
    pub corpus_written: bool,
    pub training: DemoStepSummary,
    pub hot_pack: DemoStepSummary,
    pub hot_chat_eval: DemoHotChatEvalSummary,
    pub nonlinear_memory_eval: DemoNonlinearEvalSummary,
    pub domain_eval: DemoDomainEvalSummary,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoStepSummary {
    pub version: String,
    pub verdict: String,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoHotChatEvalSummary {
    pub version: String,
    pub verdict: String,
    pub memory_lift_observed: bool,
    pub false_safe_before_learning: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoNonlinearEvalSummary {
    pub verdict: String,
    pub selected_policy: String,
    pub selected_policy_proven: bool,
    pub scale_amortized_nonlinear_memory_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoDomainEvalSummary {
    pub version: String,
    pub verdict: String,
    pub passed_components: usize,
    pub pass_rate: f32,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoDomainMetrics {
    pub component_count: usize,
    pub passed_components: usize,
    pub pass_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct DemoDomainClaimBoundary {
    pub demo_domain_command_ready: bool,
    pub small_domain_llmwave_ready: bool,
    pub scripted_hot_multi_turn_ready: bool,
    pub scale_amortized_nonlinear_memory_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub general_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_demo_domain_report(
    out_dir: &Path,
    nonlinear_corpus_path: &Path,
    top_k: usize,
) -> Result<DemoDomainReport> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create demo directory {}", out_dir.display()))?;

    let files = DemoDomainPaths::new(out_dir, nonlinear_corpus_path);
    write_demo_inputs(&files)?;
    remove_generated_outputs(&files)?;

    let training = training::compile_training_corpus(training::TrainingConfig {
        inputs: vec![files.corpus.clone()],
        out: Some(files.artifact.clone()),
        vocab_cap: 128,
        transition_cap: 256,
        active_chunk_cap: 64,
        chunk_tokens: 8,
        hot_budget_bytes: 6 * 1024 * 1024,
        max_file_bytes: 4 * 1024 * 1024,
        extensions: training::parse_extensions("txt"),
    })?;
    let hot_pack = training::pack_hot_artifact(&files.artifact, &files.hot_pack)?;
    let hot_chat_eval = training::eval_hot_chat_session(
        &files.hot_pack,
        &files.artifact,
        &files.chat_eval_memory,
        &files.chat_script,
        top_k,
    )?;
    let domain_eval = domain_eval::build_domain_eval_report(
        &files.artifact,
        &files.ask_suite,
        &files.hot_pack,
        &files.chat_script,
        &files.domain_chat_memory,
        nonlinear_corpus_path,
        top_k,
    )?;

    let training_passed = training.verdict == "TRAINING_ARTIFACT_READY_NOT_LLM";
    let hot_pack_passed = hot_pack.verdict == "HOT_PACK_READY_NOT_CACHE_ONLY_PROOF"
        && hot_pack.claim_boundary.binary_hot_pack_written
        && hot_pack.bytes.fits_hot_budget;
    let hot_chat_passed = hot_chat_eval.verdict == "HOT_CHAT_EVAL_PASS_NOT_GENERAL_LLM"
        && hot_chat_eval.metrics.memory_lift_observed
        && !hot_chat_eval.metrics.false_safe_before_learning;
    let nonlinear_passed = domain_eval.nonlinear_memory_eval.selected_policy_proven
        && domain_eval
            .nonlinear_memory_eval
            .scale_amortized_nonlinear_memory_proven;
    let domain_passed = domain_eval.verdict == "SMALL_DOMAIN_LLMWAVE_EVAL_PASS_NOT_BROAD_LLM"
        && domain_eval.claim_boundary.small_domain_llmwave_ready;
    let passed_components = [
        training_passed,
        hot_pack_passed,
        hot_chat_passed,
        nonlinear_passed,
        domain_passed,
    ]
    .into_iter()
    .filter(|passed| *passed)
    .count();
    let demo_ready = passed_components == 5;
    let verdict = if demo_ready {
        "DEMO_DOMAIN_PASS_NOT_BROAD_LLM"
    } else {
        "DEMO_DOMAIN_REVIEW_NOT_BROAD_LLM"
    };

    Ok(DemoDomainReport {
        mode: "llmwave-big-demo-domain",
        version: DEMO_DOMAIN_VERSION,
        verdict,
        demo_dir: display_path(out_dir),
        files: files.to_report(),
        steps: DemoDomainSteps {
            corpus_written: true,
            training: training_step(&training, training_passed),
            hot_pack: hot_pack_step(&hot_pack, hot_pack_passed),
            hot_chat_eval: hot_chat_step(&hot_chat_eval, hot_chat_passed),
            nonlinear_memory_eval: nonlinear_step(&domain_eval, nonlinear_passed),
            domain_eval: domain_step(&domain_eval, domain_passed),
        },
        metrics: DemoDomainMetrics {
            component_count: 5,
            passed_components,
            pass_rate: ratio(passed_components, 5),
            state: verdict,
        },
        claim_boundary: DemoDomainClaimBoundary {
            demo_domain_command_ready: demo_ready,
            small_domain_llmwave_ready: domain_eval.claim_boundary.small_domain_llmwave_ready,
            scripted_hot_multi_turn_ready: domain_eval.claim_boundary.scripted_hot_multi_turn_ready,
            scale_amortized_nonlinear_memory_ready: domain_eval
                .claim_boundary
                .scale_amortized_nonlinear_memory_ready,
            broad_chat_llm_ready: false,
            general_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: if demo_ready {
                "One command builds and evaluates a tiny small-domain LLMWave path; it is not broad chat, not a general LLM, and not proof of nonlinear memory."
            } else {
                "The one-command small-domain demo needs review; do not claim model readiness."
            },
        },
    })
}

struct DemoDomainPaths {
    corpus: PathBuf,
    artifact: PathBuf,
    ask_suite: PathBuf,
    hot_pack: PathBuf,
    chat_script: PathBuf,
    chat_eval_memory: PathBuf,
    domain_chat_memory: PathBuf,
    nonlinear_corpus: PathBuf,
}

impl DemoDomainPaths {
    fn new(out_dir: &Path, nonlinear_corpus: &Path) -> Self {
        Self {
            corpus: out_dir.join("corpus.txt"),
            artifact: out_dir.join("project-artifact.json"),
            ask_suite: out_dir.join("ask-eval.json"),
            hot_pack: out_dir.join("project.hot.bin"),
            chat_script: out_dir.join("chat.script"),
            chat_eval_memory: out_dir.join("chat-eval-memory.json"),
            domain_chat_memory: out_dir.join("domain-chat-memory.json"),
            nonlinear_corpus: nonlinear_corpus.to_path_buf(),
        }
    }

    fn to_report(&self) -> DemoDomainFiles {
        DemoDomainFiles {
            corpus: display_path(&self.corpus),
            artifact: display_path(&self.artifact),
            ask_suite: display_path(&self.ask_suite),
            hot_pack: display_path(&self.hot_pack),
            chat_script: display_path(&self.chat_script),
            chat_eval_memory: display_path(&self.chat_eval_memory),
            domain_chat_memory: display_path(&self.domain_chat_memory),
            nonlinear_corpus: display_path(&self.nonlinear_corpus),
        }
    }
}

fn write_demo_inputs(files: &DemoDomainPaths) -> Result<()> {
    fs::write(&files.corpus, DEMO_CORPUS)
        .with_context(|| format!("failed to write {}", files.corpus.display()))?;
    fs::write(&files.ask_suite, DEMO_ASK_EVAL)
        .with_context(|| format!("failed to write {}", files.ask_suite.display()))?;
    fs::write(&files.chat_script, DEMO_CHAT_SCRIPT)
        .with_context(|| format!("failed to write {}", files.chat_script.display()))?;
    Ok(())
}

fn remove_generated_outputs(files: &DemoDomainPaths) -> Result<()> {
    for path in [
        &files.artifact,
        &files.hot_pack,
        &files.chat_eval_memory,
        &files.domain_chat_memory,
    ] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).with_context(|| format!("failed to remove {}", path.display()))
            }
        }
    }
    Ok(())
}

fn training_step(report: &TrainingCompileReport, passed: bool) -> DemoStepSummary {
    DemoStepSummary {
        version: report.version.to_string(),
        verdict: report.verdict.to_string(),
        passed,
    }
}

fn hot_pack_step(report: &HotPackReport, passed: bool) -> DemoStepSummary {
    DemoStepSummary {
        version: report.version.to_string(),
        verdict: report.verdict.to_string(),
        passed,
    }
}

fn hot_chat_step(report: &HotChatEvalReport, passed: bool) -> DemoHotChatEvalSummary {
    DemoHotChatEvalSummary {
        version: report.version.to_string(),
        verdict: report.verdict.to_string(),
        memory_lift_observed: report.metrics.memory_lift_observed,
        false_safe_before_learning: report.metrics.false_safe_before_learning,
        passed,
    }
}

fn nonlinear_step(report: &DomainEvalReport, passed: bool) -> DemoNonlinearEvalSummary {
    DemoNonlinearEvalSummary {
        verdict: report.nonlinear_memory_eval.verdict.clone(),
        selected_policy: report.nonlinear_memory_eval.selected_policy.clone(),
        selected_policy_proven: report.nonlinear_memory_eval.selected_policy_proven,
        scale_amortized_nonlinear_memory_ready: report
            .nonlinear_memory_eval
            .scale_amortized_nonlinear_memory_proven,
        nonlinear_memory_proven: report.nonlinear_memory_eval.nonlinear_memory_proven,
        passed,
    }
}

fn domain_step(report: &DomainEvalReport, passed: bool) -> DemoDomainEvalSummary {
    DemoDomainEvalSummary {
        version: report.version.to_string(),
        verdict: report.verdict.to_string(),
        passed_components: report.metrics.passed_components,
        pass_rate: report.metrics.pass_rate,
        passed,
    }
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        ((numerator as f32 / denominator as f32) * 10_000.0).round() / 10_000.0
    }
}
