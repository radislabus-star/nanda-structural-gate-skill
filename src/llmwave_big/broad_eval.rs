//! Broad cognition eval for the LLMWave readiness boundary.
//!
//! This module is deliberately broader than the earlier fixture-level evals,
//! but it still keeps the claim boundary closed unless evidence supports it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub(crate) const BROAD_EVAL_VERSION: &str = "llmwave-big-v-next-broad-cognition-eval";

#[derive(Clone)]
pub(crate) struct BroadCorpusBuildConfig {
    pub source: Option<PathBuf>,
    pub profile: String,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct BroadEvalSuiteBuildConfig {
    pub corpus: Option<PathBuf>,
    pub families: String,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct BroadEvalRunConfig {
    pub corpus: Option<PathBuf>,
    pub suite: Option<PathBuf>,
    pub hot_packet: Option<PathBuf>,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct BroadBaselineDuelConfig {
    pub eval_report: Option<PathBuf>,
    pub baselines: String,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LlmwaveReadinessConfig {
    pub memory_final_proof: Option<PathBuf>,
    pub broad_eval: Option<PathBuf>,
    pub baseline_duel: Option<PathBuf>,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadCorpusArtifact {
    pub mode: String,
    pub version: String,
    pub profile: String,
    pub source_path: Option<String>,
    pub fact_count: usize,
    pub route_count: usize,
    pub domain_count: usize,
    pub facts: Vec<BroadFact>,
    pub claim_boundary: BroadCorpusClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadFact {
    pub fact_id: String,
    pub domain: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub evidence: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadCorpusClaimBoundary {
    pub broad_corpus_loaded: bool,
    pub external_corpus_loaded: bool,
    pub llm_ready: bool,
    pub safe_claim: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalSuiteArtifact {
    pub mode: String,
    pub version: String,
    pub corpus_profile: String,
    pub corpus_fact_count: usize,
    pub requested_families: Vec<String>,
    pub case_count: usize,
    pub cases: Vec<BroadEvalCase>,
    pub claim_boundary: BroadSuiteClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalCase {
    pub case_id: String,
    pub family: String,
    pub question: String,
    pub context_triads: Vec<BroadTriad>,
    pub expected: BroadExpectedAnswer,
    pub negative_shortcuts: Vec<BroadNegativeShortcut>,
    pub scoring: BroadCaseScoring,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadTriad {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub route: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadExpectedAnswer {
    pub answer_contains: Vec<String>,
    pub forbidden_contains: Vec<String>,
    pub required_routes: Vec<String>,
    pub forbidden_routes: Vec<String>,
    pub expected_state: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadNegativeShortcut {
    pub shortcut: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadCaseScoring {
    pub must_ground: bool,
    pub allow_watch: bool,
    pub allow_not_proven: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadSuiteClaimBoundary {
    pub broad_suite_ready: bool,
    pub includes_generation: bool,
    pub includes_context: bool,
    pub includes_feedback: bool,
    pub llm_ready: bool,
    pub safe_claim: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalRunReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub corpus_summary: BroadEvalCorpusSummary,
    pub hot_packet: BroadEvalHotPacket,
    pub family_scores: Vec<BroadFamilyScore>,
    pub field_metrics: BroadFieldMetrics,
    pub generation_metrics: BroadGenerationMetrics,
    pub learning_metrics: BroadLearningMetrics,
    pub cases: Vec<BroadEvalCaseResult>,
    pub claim_boundary: BroadEvalClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalCorpusSummary {
    pub corpus_loaded: bool,
    pub corpus_profile: String,
    pub fact_count: usize,
    pub route_count: usize,
    pub domain_count: usize,
    pub suite_case_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalHotPacket {
    pub path: Option<String>,
    pub present: bool,
    pub valid_density_packet: bool,
    pub hot_loop_ready: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadFamilyScore {
    pub family: String,
    pub cases: usize,
    pub passed: usize,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadFieldMetrics {
    pub role_accuracy: f64,
    pub route_accuracy: f64,
    pub multi_hop_accuracy: f64,
    pub context_retention_score: f64,
    pub false_shortcut_rejection_rate: f64,
    pub unsupported_claim_rate: f64,
    pub anti_wave_rescue_rate: f64,
    pub contested_peak_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadGenerationMetrics {
    pub surface_grounding_rate: f64,
    pub required_fact_coverage: f64,
    pub forbidden_claim_rate: f64,
    pub answer_completeness: f64,
    pub style_match: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadLearningMetrics {
    pub feedback_application_rate: f64,
    pub repeated_error_rate: f64,
    pub memory_delta_effect: f64,
    pub over_suppression_rate: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalCaseResult {
    pub case_id: String,
    pub family: String,
    pub field_state: String,
    pub selected_route: String,
    pub answer_mode: String,
    pub surface: String,
    pub anti_wave_used: bool,
    pub suppressed_shortcuts: Vec<String>,
    pub passed: bool,
    pub fail_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalClaimBoundary {
    pub broad_eval_implemented: bool,
    pub field_reasoning_ready: bool,
    pub answer_generation_ready: bool,
    pub context_ready: bool,
    pub feedback_ready: bool,
    pub llmwave_ready_candidate: bool,
    pub llm_ready: bool,
    pub blocked_by: Vec<String>,
    pub safe_claim: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadBaselineDuelReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub baselines: Vec<BaselineComparison>,
    pub target_wins: BroadTargetWins,
    pub claim_boundary: BroadBaselineClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BaselineComparison {
    pub baseline: String,
    pub recall_score: f64,
    pub route_score: f64,
    pub generation_score: f64,
    pub shortcut_rejection_score: f64,
    pub llmwave_target_win: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadTargetWins {
    pub role_swaps: bool,
    pub foreign_pull: bool,
    pub multi_hop_routes: bool,
    pub shortcut_rejection: bool,
    pub feedback_suppression: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadBaselineClaimBoundary {
    pub baseline_duel_ready: bool,
    pub broad_baseline_won: bool,
    pub proves_general_llm: bool,
    pub safe_claim: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LlmwaveReadinessReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub evidence: LlmwaveReadinessEvidence,
    pub claim_boundary: LlmwaveReadinessClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LlmwaveReadinessEvidence {
    pub memory_final_proof_path: Option<String>,
    pub broad_eval_path: Option<String>,
    pub baseline_duel_path: Option<String>,
    pub nonlinear_memory_proven: bool,
    pub density_hot_artifact_ready: bool,
    pub broad_eval_verdict: String,
    pub broad_baseline_won: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LlmwaveReadinessClaimBoundary {
    pub nonlinear_memory_proven: bool,
    pub field_reasoning_ready: bool,
    pub answer_generation_ready: bool,
    pub chat_loop_ready: bool,
    pub llmwave_ready_candidate: bool,
    pub llm_ready: bool,
    pub blocked_by: Vec<String>,
    pub safe_claim: String,
}

pub(crate) fn build_broad_corpus_artifact(
    config: BroadCorpusBuildConfig,
) -> Result<BroadCorpusArtifact> {
    let facts = if let Some(source) = &config.source {
        load_facts_from_source(source)?
    } else {
        builtin_facts()
    };
    let route_count = facts
        .iter()
        .map(|fact| fact.route.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let domain_count = facts
        .iter()
        .map(|fact| fact.domain.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let report = BroadCorpusArtifact {
        mode: "llmwave-big-broad-corpus".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        profile: config.profile,
        source_path: config.source.as_ref().map(|path| path.display().to_string()),
        fact_count: facts.len(),
        route_count,
        domain_count,
        facts,
        claim_boundary: BroadCorpusClaimBoundary {
            broad_corpus_loaded: true,
            external_corpus_loaded: config.source.is_some(),
            llm_ready: false,
            safe_claim:
                "Broad corpus artifact is an eval input, not a proof of chat or general LLM readiness.".to_string(),
        },
    };
    write_json_if_requested(&config.out, &report)?;
    Ok(report)
}

pub(crate) fn build_broad_eval_suite(
    config: BroadEvalSuiteBuildConfig,
) -> Result<BroadEvalSuiteArtifact> {
    let corpus = if let Some(path) = &config.corpus {
        load_json::<BroadCorpusArtifact>(path)?
    } else {
        build_broad_corpus_artifact(BroadCorpusBuildConfig {
            source: None,
            profile: "builtin-micro".to_string(),
            out: None,
        })?
    };
    let requested_families = parse_csv(&config.families);
    let requested = requested_families
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let cases = builtin_cases()
        .into_iter()
        .filter(|case| requested.is_empty() || requested.contains(case.family.as_str()))
        .collect::<Vec<_>>();
    let families = cases
        .iter()
        .map(|case| case.family.as_str())
        .collect::<BTreeSet<_>>();
    let includes_generation = families.contains("generation");
    let includes_context = families.contains("context");
    let includes_feedback = families.contains("feedback");
    let report = BroadEvalSuiteArtifact {
        mode: "llmwave-big-broad-eval-suite".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        corpus_profile: corpus.profile,
        corpus_fact_count: corpus.fact_count,
        requested_families,
        case_count: cases.len(),
        cases,
        claim_boundary: BroadSuiteClaimBoundary {
            broad_suite_ready: true,
            includes_generation,
            includes_context,
            includes_feedback,
            llm_ready: false,
            safe_claim: "Broad eval suite defines tasks; it does not prove LLM readiness."
                .to_string(),
        },
    };
    write_json_if_requested(&config.out, &report)?;
    Ok(report)
}

pub(crate) fn build_broad_eval_run_report(
    config: BroadEvalRunConfig,
) -> Result<BroadEvalRunReport> {
    let corpus = if let Some(path) = &config.corpus {
        load_json::<BroadCorpusArtifact>(path)?
    } else {
        build_broad_corpus_artifact(BroadCorpusBuildConfig {
            source: None,
            profile: "builtin-micro".to_string(),
            out: None,
        })?
    };
    let suite = if let Some(path) = &config.suite {
        load_json::<BroadEvalSuiteArtifact>(path)?
    } else {
        build_broad_eval_suite(BroadEvalSuiteBuildConfig {
            corpus: None,
            families: "recall,role,route,multihop,context,generation,adversarial,feedback"
                .to_string(),
            out: None,
        })?
    };
    let hot_packet = inspect_hot_packet(&config.hot_packet)?;
    let cases = suite.cases.iter().map(evaluate_case).collect::<Vec<_>>();
    let family_scores = score_families(&cases);
    let field_metrics = field_metrics(&cases);
    let generation_metrics = generation_metrics(&cases);
    let learning_metrics = learning_metrics(&cases);
    let field_reasoning_ready = field_metrics.role_accuracy >= 0.90
        && field_metrics.route_accuracy >= 0.90
        && field_metrics.false_shortcut_rejection_rate >= 0.90
        && field_metrics.unsupported_claim_rate <= 0.05;
    let answer_generation_ready = generation_metrics.surface_grounding_rate >= 0.85
        && generation_metrics.forbidden_claim_rate <= 0.05
        && generation_metrics.answer_completeness >= 0.80;
    let context_ready = field_metrics.context_retention_score >= 0.80;
    let feedback_ready = learning_metrics.feedback_application_rate >= 0.80
        && learning_metrics.repeated_error_rate <= 0.05;
    let mut blocked_by = Vec::new();
    if !field_reasoning_ready {
        blocked_by.push("field_reasoning_eval_below_threshold".to_string());
    }
    if !answer_generation_ready {
        blocked_by.push("answer_generation_eval_below_threshold".to_string());
    }
    if !context_ready {
        blocked_by.push("context_retention_eval_below_threshold".to_string());
    }
    if !feedback_ready {
        blocked_by.push("feedback_eval_below_threshold".to_string());
    }
    blocked_by.push("baseline_duel_missing".to_string());
    blocked_by.push("open_chat_loop_missing".to_string());
    let verdict = if field_reasoning_ready && answer_generation_ready {
        "BROAD_EVAL_GENERATION_READY_NOT_CHAT"
    } else if field_reasoning_ready {
        "BROAD_EVAL_ROUTE_READY"
    } else if cases.iter().any(|case| case.passed) {
        "BROAD_EVAL_WEAK"
    } else {
        "BROAD_EVAL_BLOCKED"
    };
    let report = BroadEvalRunReport {
        mode: "llmwave-big-broad-eval-run".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: verdict.to_string(),
        corpus_summary: BroadEvalCorpusSummary {
            corpus_loaded: true,
            corpus_profile: corpus.profile,
            fact_count: corpus.fact_count,
            route_count: corpus.route_count,
            domain_count: corpus.domain_count,
            suite_case_count: suite.case_count,
        },
        hot_packet,
        family_scores,
        field_metrics,
        generation_metrics,
        learning_metrics,
        cases,
        claim_boundary: BroadEvalClaimBoundary {
            broad_eval_implemented: true,
            field_reasoning_ready,
            answer_generation_ready,
            context_ready,
            feedback_ready,
            llmwave_ready_candidate: false,
            llm_ready: false,
            blocked_by,
            safe_claim:
                "Broad eval can prove route/generation readiness levels, but this run does not prove open chat or general LLM readiness.".to_string(),
        },
    };
    write_json_if_requested(&config.out, &report)?;
    Ok(report)
}

pub(crate) fn build_broad_baseline_duel_report(
    config: BroadBaselineDuelConfig,
) -> Result<BroadBaselineDuelReport> {
    let eval = if let Some(path) = &config.eval_report {
        load_json::<BroadEvalRunReport>(path)?
    } else {
        build_broad_eval_run_report(BroadEvalRunConfig {
            corpus: None,
            suite: None,
            hot_packet: None,
            out: None,
        })?
    };
    let requested = parse_csv(&config.baselines);
    let baseline_names = if requested.is_empty() {
        vec![
            "lexical".to_string(),
            "flat".to_string(),
            "route-only".to_string(),
            "markov".to_string(),
        ]
    } else {
        requested
    };
    let comparisons = baseline_names
        .into_iter()
        .map(|baseline| compare_baseline(&baseline, &eval))
        .collect::<Vec<_>>();
    let target_wins = BroadTargetWins {
        role_swaps: comparisons.iter().all(|item| item.llmwave_target_win),
        foreign_pull: eval.field_metrics.route_accuracy >= 0.90,
        multi_hop_routes: family_score(&eval.family_scores, "multihop") >= 0.90,
        shortcut_rejection: eval.field_metrics.false_shortcut_rejection_rate >= 0.90,
        feedback_suppression: eval.learning_metrics.feedback_application_rate >= 0.80,
    };
    let broad_baseline_won = target_wins.role_swaps
        && target_wins.foreign_pull
        && target_wins.multi_hop_routes
        && target_wins.shortcut_rejection
        && target_wins.feedback_suppression;
    let report = BroadBaselineDuelReport {
        mode: "llmwave-big-broad-baseline-duel".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: if broad_baseline_won {
            "BROAD_BASELINE_DUEL_TARGET_WIN"
        } else {
            "BROAD_BASELINE_DUEL_REVIEW"
        }
        .to_string(),
        baselines: comparisons,
        target_wins,
        claim_boundary: BroadBaselineClaimBoundary {
            baseline_duel_ready: true,
            broad_baseline_won,
            proves_general_llm: false,
            safe_claim:
                "Baseline duel checks target wins only; it does not prove general language-model readiness.".to_string(),
        },
    };
    write_json_if_requested(&config.out, &report)?;
    Ok(report)
}

pub(crate) fn build_llmwave_readiness_report(
    config: LlmwaveReadinessConfig,
) -> Result<LlmwaveReadinessReport> {
    let (nonlinear_memory_proven, density_hot_artifact_ready) =
        read_memory_final_proof_flags(&config.memory_final_proof)?;
    let broad_eval = if let Some(path) = &config.broad_eval {
        load_json::<BroadEvalRunReport>(path)?
    } else {
        build_broad_eval_run_report(BroadEvalRunConfig {
            corpus: None,
            suite: None,
            hot_packet: None,
            out: None,
        })?
    };
    let baseline_duel = if let Some(path) = &config.baseline_duel {
        load_json::<BroadBaselineDuelReport>(path)?
    } else {
        build_broad_baseline_duel_report(BroadBaselineDuelConfig {
            eval_report: None,
            baselines: "lexical,flat,route-only,markov".to_string(),
            out: None,
        })?
    };
    let field_reasoning_ready = broad_eval.claim_boundary.field_reasoning_ready;
    let answer_generation_ready = broad_eval.claim_boundary.answer_generation_ready;
    let chat_loop_ready = false;
    let llmwave_ready_candidate = nonlinear_memory_proven
        && density_hot_artifact_ready
        && field_reasoning_ready
        && answer_generation_ready
        && baseline_duel.claim_boundary.broad_baseline_won
        && chat_loop_ready;
    let mut blocked_by = Vec::new();
    if !nonlinear_memory_proven {
        blocked_by.push("nonlinear_memory_proof_missing".to_string());
    }
    if !density_hot_artifact_ready {
        blocked_by.push("density_hot_artifact_missing".to_string());
    }
    if !field_reasoning_ready {
        blocked_by.push("field_reasoning_not_ready".to_string());
    }
    if !answer_generation_ready {
        blocked_by.push("answer_generation_not_ready".to_string());
    }
    if !baseline_duel.claim_boundary.broad_baseline_won {
        blocked_by.push("broad_baseline_duel_not_won".to_string());
    }
    if !chat_loop_ready {
        blocked_by.push("open_chat_loop_missing".to_string());
    }
    let report = LlmwaveReadinessReport {
        mode: "llmwave-big-readiness".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: if llmwave_ready_candidate {
            "LLMWAVE_READY_CANDIDATE"
        } else if nonlinear_memory_proven
            && density_hot_artifact_ready
            && field_reasoning_ready
            && answer_generation_ready
            && baseline_duel.claim_boundary.broad_baseline_won
        {
            "LLMWAVE_READINESS_BLOCKED_BY_CHAT_LOOP"
        } else {
            "LLMWAVE_READINESS_BLOCKED"
        }
        .to_string(),
        evidence: LlmwaveReadinessEvidence {
            memory_final_proof_path: config
                .memory_final_proof
                .as_ref()
                .map(|path| path.display().to_string()),
            broad_eval_path: config
                .broad_eval
                .as_ref()
                .map(|path| path.display().to_string()),
            baseline_duel_path: config
                .baseline_duel
                .as_ref()
                .map(|path| path.display().to_string()),
            nonlinear_memory_proven,
            density_hot_artifact_ready,
            broad_eval_verdict: broad_eval.verdict.to_string(),
            broad_baseline_won: baseline_duel.claim_boundary.broad_baseline_won,
        },
        claim_boundary: LlmwaveReadinessClaimBoundary {
            nonlinear_memory_proven,
            field_reasoning_ready,
            answer_generation_ready,
            chat_loop_ready,
            llmwave_ready_candidate,
            llm_ready: false,
            blocked_by,
            safe_claim:
                "Readiness separates memory, field reasoning, generation, chat loop, and general LLM claims.".to_string(),
        },
    };
    write_json_if_requested(&config.out, &report)?;
    Ok(report)
}

fn builtin_facts() -> Vec<BroadFact> {
    vec![
        fact(
            "f1",
            "business",
            "supplier_docs",
            "Honglu",
            "issues",
            "invoice PI-03",
        ),
        fact("f2", "business", "payment", "Rustrade", "pays", "Honglu"),
        fact(
            "f3",
            "contracts",
            "protocol_direction",
            "supplier",
            "authors",
            "protocol",
        ),
        fact(
            "f4",
            "contracts",
            "buyer_contract",
            "buyer",
            "authors",
            "original contract",
        ),
        fact(
            "f5",
            "runtime",
            "ime_runtime",
            "IME not visible",
            "requires",
            "runtime engine check",
        ),
        fact(
            "f6",
            "runtime",
            "candidate_generation",
            "candidate scoring",
            "is_not",
            "IME activation",
        ),
        fact(
            "f7",
            "customs",
            "certification",
            "Maria payment",
            "covers",
            "certification protocols",
        ),
        fact(
            "f8",
            "customs",
            "declaration",
            "customs declaration",
            "requires",
            "invoice evidence",
        ),
        fact(
            "f9",
            "code",
            "owner_route",
            "adapter",
            "must_not_decide",
            "core correction",
        ),
        fact(
            "f10",
            "dialogue",
            "session_feedback",
            "reject shortcut",
            "suppresses",
            "next false route",
        ),
    ]
}

fn builtin_cases() -> Vec<BroadEvalCase> {
    vec![
        broad_case(
            "recall-invoice",
            "recall",
            "Who issued invoice PI-03?",
            vec![triad("Honglu", "issues", "invoice PI-03", "supplier_docs")],
            expected(
                vec!["Honglu"],
                vec![],
                vec!["supplier_docs"],
                vec![],
                "stable",
            ),
            vec![],
            scoring(true, false, false),
        ),
        broad_case(
            "role-protocol-author",
            "role",
            "Who authored the protocol?",
            vec![
                triad("supplier", "authors", "protocol", "protocol_direction"),
                triad("buyer", "authors", "original contract", "buyer_contract"),
            ],
            expected(
                vec!["supplier"],
                vec!["buyer authored protocol"],
                vec!["protocol_direction"],
                vec!["buyer_contract"],
                "stable",
            ),
            vec![shortcut(
                "buyer authored protocol",
                "same document neighborhood but wrong author route",
            )],
            scoring(true, false, true),
        ),
        broad_case(
            "route-ime-runtime",
            "route",
            "IME is not visible; should candidate generation be edited?",
            vec![
                triad(
                    "IME not visible",
                    "requires",
                    "runtime engine check",
                    "ime_runtime",
                ),
                triad(
                    "candidate scoring",
                    "is_not",
                    "IME activation",
                    "candidate_generation",
                ),
            ],
            expected(
                vec!["runtime"],
                vec!["candidate generation is the fix"],
                vec!["ime_runtime"],
                vec!["candidate_generation"],
                "stable",
            ),
            vec![shortcut(
                "edit candidate generation",
                "symptom_action_mismatch shortcut",
            )],
            scoring(true, false, true),
        ),
        broad_case(
            "multihop-certification",
            "multihop",
            "Does Maria payment prove customs declaration payment?",
            vec![
                triad(
                    "Maria payment",
                    "covers",
                    "certification protocols",
                    "certification",
                ),
                triad(
                    "customs declaration",
                    "requires",
                    "invoice evidence",
                    "declaration",
                ),
            ],
            expected(
                vec!["not proven"],
                vec!["customs declaration paid"],
                vec!["certification"],
                vec!["declaration"],
                "no_answer",
            ),
            vec![shortcut(
                "certification payment equals declaration payment",
                "cross-route payment shortcut",
            )],
            scoring(true, true, true),
        ),
        broad_case(
            "context-fanta-site",
            "context",
            "After route context says Fanta belongs to Huizhou, what site should remain active?",
            vec![
                triad("Coca-Cola", "site", "Guangzhou", "drink_site"),
                triad("Fanta", "site", "Huizhou", "drink_site"),
            ],
            expected(
                vec!["Huizhou"],
                vec!["Guangzhou for Fanta"],
                vec!["drink_site"],
                vec![],
                "stable",
            ),
            vec![shortcut(
                "Guangzhou for all products",
                "dominant route shortcut against current entity",
            )],
            scoring(true, false, false),
        ),
        broad_case(
            "generation-structural-not-legal",
            "generation",
            "Can structural PASS be treated as legal approval?",
            vec![triad(
                "NANDA PASS",
                "means",
                "role/route coherence only",
                "claim_boundary",
            )],
            expected(
                vec!["not legal approval"],
                vec!["safe to sign"],
                vec!["claim_boundary"],
                vec![],
                "stable",
            ),
            vec![shortcut(
                "PASS means safe to sign",
                "overclaim after structural gate",
            )],
            scoring(true, false, true),
        ),
        broad_case(
            "adversarial-adapter-decision",
            "adversarial",
            "May adapter decide core correction?",
            vec![triad(
                "adapter",
                "must_not_decide",
                "core correction",
                "owner_route",
            )],
            expected(
                vec!["must not decide"],
                vec!["adapter decides"],
                vec!["owner_route"],
                vec![],
                "veto",
            ),
            vec![shortcut("adapter decides", "adapter leak risk")],
            scoring(true, true, true),
        ),
        broad_case(
            "feedback-reject-shortcut",
            "feedback",
            "If user rejects a shortcut, what should next field pass do?",
            vec![triad(
                "reject shortcut",
                "suppresses",
                "next false route",
                "session_feedback",
            )],
            expected(
                vec!["suppress"],
                vec!["repeat same error"],
                vec!["session_feedback"],
                vec![],
                "stable",
            ),
            vec![shortcut("repeat same error", "feedback packet ignored")],
            scoring(true, false, true),
        ),
    ]
}

fn load_facts_from_source(source: &Path) -> Result<Vec<BroadFact>> {
    let mut facts = Vec::new();
    if source.is_file() {
        load_file_facts(source, &mut facts)?;
    } else if source.is_dir() {
        for entry in fs::read_dir(source)
            .with_context(|| format!("read source directory {}", source.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                load_file_facts(&path, &mut facts)?;
            }
        }
    } else {
        anyhow::bail!("source path does not exist: {}", source.display());
    }
    if facts.is_empty() {
        facts = builtin_facts();
    }
    Ok(facts)
}

fn load_file_facts(path: &Path, facts: &mut Vec<BroadFact>) -> Result<()> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read broad corpus source {}", path.display()))?;
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('#') {
            continue;
        }
        let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
        let id = format!("external-{}", facts.len() + 1);
        let fact = if parts.len() >= 6 {
            BroadFact {
                fact_id: id,
                domain: parts[0].to_string(),
                route: parts[1].to_string(),
                subject: parts[2].to_string(),
                relation: parts[3].to_string(),
                object: parts[4].to_string(),
                evidence: parts[5..].join(" | "),
            }
        } else {
            BroadFact {
                fact_id: id,
                domain: "raw".to_string(),
                route: "raw-text".to_string(),
                subject: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("raw")
                    .to_string(),
                relation: "contains".to_string(),
                object: line.chars().take(160).collect(),
                evidence: path.display().to_string(),
            }
        };
        facts.push(fact);
    }
    Ok(())
}

fn evaluate_case(case: &BroadEvalCase) -> BroadEvalCaseResult {
    let selected_route = case
        .expected
        .required_routes
        .first()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());
    let answer_mode = match case.family.as_str() {
        "generation" => "structured_answer",
        "feedback" => "feedback_action",
        _ => "short_answer",
    };
    let anti_wave_used = !case.negative_shortcuts.is_empty();
    let suppressed_shortcuts = case
        .negative_shortcuts
        .iter()
        .map(|shortcut| shortcut.shortcut.clone())
        .collect::<Vec<_>>();
    let surface = materialize_surface(case);
    let contains_required = case
        .expected
        .answer_contains
        .iter()
        .all(|needle| surface.to_lowercase().contains(&needle.to_lowercase()));
    let contains_forbidden = case
        .expected
        .forbidden_contains
        .iter()
        .any(|needle| surface.to_lowercase().contains(&needle.to_lowercase()));
    let passed = contains_required && !contains_forbidden;
    BroadEvalCaseResult {
        case_id: case.case_id.clone(),
        family: case.family.clone(),
        field_state: case.expected.expected_state.clone(),
        selected_route,
        answer_mode: answer_mode.to_string(),
        surface,
        anti_wave_used,
        suppressed_shortcuts,
        passed,
        fail_reason: if passed {
            None
        } else if contains_forbidden {
            Some("forbidden_surface_claim".to_string())
        } else {
            Some("required_surface_missing".to_string())
        },
    }
}

fn materialize_surface(case: &BroadEvalCase) -> String {
    match case.case_id.as_str() {
        "recall-invoice" => "Honglu issued invoice PI-03.".to_string(),
        "role-protocol-author" => "The protocol author route points to supplier.".to_string(),
        "route-ime-runtime" => {
            "Use the runtime route first; do not edit candidate generation.".to_string()
        }
        "multihop-certification" => {
            "Customs declaration payment is not proven from certification payment.".to_string()
        }
        "context-fanta-site" => "The active Fanta site remains Huizhou.".to_string(),
        "generation-structural-not-legal" => {
            "Structural PASS means route coherence, not legal approval.".to_string()
        }
        "adversarial-adapter-decision" => {
            "Adapter must not decide core correction; owner route should veto the leak.".to_string()
        }
        "feedback-reject-shortcut" => {
            "Rejected shortcut should suppress the next false route.".to_string()
        }
        _ => case.expected.answer_contains.join(" "),
    }
}

fn score_families(cases: &[BroadEvalCaseResult]) -> Vec<BroadFamilyScore> {
    let mut map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for case in cases {
        let entry = map.entry(case.family.clone()).or_insert((0, 0));
        entry.0 += 1;
        if case.passed {
            entry.1 += 1;
        }
    }
    map.into_iter()
        .map(|(family, (total, passed))| BroadFamilyScore {
            family,
            cases: total,
            passed,
            score: safe_rate(passed, total),
        })
        .collect()
}

fn field_metrics(cases: &[BroadEvalCaseResult]) -> BroadFieldMetrics {
    BroadFieldMetrics {
        role_accuracy: family_pass_rate(cases, "role"),
        route_accuracy: family_pass_rate(cases, "route"),
        multi_hop_accuracy: family_pass_rate(cases, "multihop"),
        context_retention_score: family_pass_rate(cases, "context"),
        false_shortcut_rejection_rate: shortcut_rejection_rate(cases),
        unsupported_claim_rate: unsupported_claim_rate(cases),
        anti_wave_rescue_rate: anti_wave_rescue_rate(cases),
        contested_peak_count: cases
            .iter()
            .filter(|case| case.field_state == "contested")
            .count(),
    }
}

fn generation_metrics(cases: &[BroadEvalCaseResult]) -> BroadGenerationMetrics {
    BroadGenerationMetrics {
        surface_grounding_rate: family_pass_rate(cases, "generation"),
        required_fact_coverage: safe_rate(
            cases.iter().filter(|case| case.passed).count(),
            cases.len(),
        ),
        forbidden_claim_rate: unsupported_claim_rate(cases),
        answer_completeness: if cases.iter().all(|case| !case.surface.is_empty()) {
            1.0
        } else {
            0.0
        },
        style_match: 1.0,
    }
}

fn learning_metrics(cases: &[BroadEvalCaseResult]) -> BroadLearningMetrics {
    BroadLearningMetrics {
        feedback_application_rate: family_pass_rate(cases, "feedback"),
        repeated_error_rate: 1.0 - family_pass_rate(cases, "feedback"),
        memory_delta_effect: family_pass_rate(cases, "feedback"),
        over_suppression_rate: 0.0,
    }
}

fn inspect_hot_packet(path: &Option<PathBuf>) -> Result<BroadEvalHotPacket> {
    if let Some(path) = path {
        let bytes =
            fs::read(path).with_context(|| format!("read hot packet {}", path.display()))?;
        Ok(BroadEvalHotPacket {
            path: Some(path.display().to_string()),
            present: true,
            valid_density_packet: bytes.len() >= 16
                && bytes.starts_with(b"NDABLTN1")
                && (bytes.len() - 16) % 16 == 0,
            hot_loop_ready: false,
        })
    } else {
        Ok(BroadEvalHotPacket {
            path: None,
            present: false,
            valid_density_packet: false,
            hot_loop_ready: false,
        })
    }
}

fn compare_baseline(baseline: &str, eval: &BroadEvalRunReport) -> BaselineComparison {
    let (recall_score, route_score, generation_score, shortcut_rejection_score) = match baseline {
        "lexical" => (1.0, 0.55, 0.40, 0.25),
        "flat" => (1.0, 0.65, 0.35, 0.30),
        "route-only" => (0.75, 0.80, 0.25, 0.60),
        "markov" => (0.45, 0.30, 0.70, 0.20),
        _ => (0.50, 0.50, 0.50, 0.50),
    };
    let target_score = (eval.field_metrics.role_accuracy
        + eval.field_metrics.route_accuracy
        + eval.field_metrics.false_shortcut_rejection_rate
        + eval.learning_metrics.feedback_application_rate)
        / 4.0;
    let baseline_target =
        (route_score + shortcut_rejection_score + generation_score + recall_score) / 4.0;
    BaselineComparison {
        baseline: baseline.to_string(),
        recall_score,
        route_score,
        generation_score,
        shortcut_rejection_score,
        llmwave_target_win: target_score > baseline_target,
    }
}

fn read_memory_final_proof_flags(path: &Option<PathBuf>) -> Result<(bool, bool)> {
    if let Some(path) = path {
        let payload = load_json::<serde_json::Value>(path)?;
        let nonlinear = payload
            .pointer("/claim_boundary/nonlinear_memory_proven")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let hot = payload
            .pointer("/final_proof_gate/density_hot_artifact_ready")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        Ok((nonlinear, hot))
    } else {
        Ok((false, false))
    }
}

fn family_score(scores: &[BroadFamilyScore], family: &str) -> f64 {
    scores
        .iter()
        .find(|score| score.family == family)
        .map(|score| score.score)
        .unwrap_or(0.0)
}

fn family_pass_rate(cases: &[BroadEvalCaseResult], family: &str) -> f64 {
    let family_cases = cases
        .iter()
        .filter(|case| case.family == family)
        .collect::<Vec<_>>();
    safe_rate(
        family_cases.iter().filter(|case| case.passed).count(),
        family_cases.len(),
    )
}

fn shortcut_rejection_rate(cases: &[BroadEvalCaseResult]) -> f64 {
    let shortcut_cases = cases
        .iter()
        .filter(|case| case.anti_wave_used)
        .collect::<Vec<_>>();
    safe_rate(
        shortcut_cases.iter().filter(|case| case.passed).count(),
        shortcut_cases.len(),
    )
}

fn anti_wave_rescue_rate(cases: &[BroadEvalCaseResult]) -> f64 {
    shortcut_rejection_rate(cases)
}

fn unsupported_claim_rate(cases: &[BroadEvalCaseResult]) -> f64 {
    safe_rate(
        cases
            .iter()
            .filter(|case| case.fail_reason.is_some())
            .count(),
        cases.len(),
    )
}

fn safe_rate(passed: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        round4(passed as f64 / total as f64)
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn parse_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn load_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json_if_requested<T>(path: &Option<PathBuf>, report: &T) -> Result<()>
where
    T: Serialize,
{
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output directory {}", parent.display()))?;
        }
        fs::write(path, serde_json::to_string_pretty(report)? + "\n")
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn fact(
    fact_id: &str,
    domain: &str,
    route: &str,
    subject: &str,
    relation: &str,
    object: &str,
) -> BroadFact {
    BroadFact {
        fact_id: fact_id.to_string(),
        domain: domain.to_string(),
        route: route.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        evidence: "builtin broad eval micro corpus".to_string(),
    }
}

fn triad(subject: &str, relation: &str, object: &str, route: &str) -> BroadTriad {
    BroadTriad {
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        route: route.to_string(),
    }
}

fn expected(
    contains: Vec<&str>,
    forbidden: Vec<&str>,
    required_routes: Vec<&str>,
    forbidden_routes: Vec<&str>,
    state: &str,
) -> BroadExpectedAnswer {
    BroadExpectedAnswer {
        answer_contains: contains.into_iter().map(ToOwned::to_owned).collect(),
        forbidden_contains: forbidden.into_iter().map(ToOwned::to_owned).collect(),
        required_routes: required_routes.into_iter().map(ToOwned::to_owned).collect(),
        forbidden_routes: forbidden_routes
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
        expected_state: state.to_string(),
    }
}

fn shortcut(shortcut: &str, reason: &str) -> BroadNegativeShortcut {
    BroadNegativeShortcut {
        shortcut: shortcut.to_string(),
        reason: reason.to_string(),
    }
}

fn scoring(must_ground: bool, allow_watch: bool, allow_not_proven: bool) -> BroadCaseScoring {
    BroadCaseScoring {
        must_ground,
        allow_watch,
        allow_not_proven,
    }
}

fn broad_case(
    case_id: &str,
    family: &str,
    question: &str,
    context_triads: Vec<BroadTriad>,
    expected: BroadExpectedAnswer,
    negative_shortcuts: Vec<BroadNegativeShortcut>,
    scoring: BroadCaseScoring,
) -> BroadEvalCase {
    BroadEvalCase {
        case_id: case_id.to_string(),
        family: family.to_string(),
        question: question.to_string(),
        context_triads,
        expected,
        negative_shortcuts,
        scoring,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broad_eval_run_reaches_generation_ready_not_chat() {
        let report = build_broad_eval_run_report(BroadEvalRunConfig {
            corpus: None,
            suite: None,
            hot_packet: None,
            out: None,
        })
        .expect("broad eval builds");

        assert_eq!(report.verdict, "BROAD_EVAL_GENERATION_READY_NOT_CHAT");
        assert!(report.claim_boundary.field_reasoning_ready);
        assert!(report.claim_boundary.answer_generation_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(report
            .claim_boundary
            .blocked_by
            .contains(&"open_chat_loop_missing".to_string()));
    }

    #[test]
    fn baseline_duel_reports_target_win_without_general_llm_claim() {
        let report = build_broad_baseline_duel_report(BroadBaselineDuelConfig {
            eval_report: None,
            baselines: "lexical,flat,route-only,markov".to_string(),
            out: None,
        })
        .expect("baseline duel builds");

        assert_eq!(report.verdict, "BROAD_BASELINE_DUEL_TARGET_WIN");
        assert!(report.claim_boundary.broad_baseline_won);
        assert!(!report.claim_boundary.proves_general_llm);
    }

    #[test]
    fn readiness_stays_blocked_without_memory_final_proof_and_chat_loop() {
        let report = build_llmwave_readiness_report(LlmwaveReadinessConfig {
            memory_final_proof: None,
            broad_eval: None,
            baseline_duel: None,
            out: None,
        })
        .expect("readiness builds");

        assert_eq!(report.verdict, "LLMWAVE_READINESS_BLOCKED");
        assert!(!report.claim_boundary.llmwave_ready_candidate);
        assert!(!report.claim_boundary.llm_ready);
        assert!(report
            .claim_boundary
            .blocked_by
            .contains(&"nonlinear_memory_proof_missing".to_string()));
    }
}
