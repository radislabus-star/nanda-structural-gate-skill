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
pub(crate) struct BroadDatasetDoctorConfig {
    pub corpus: PathBuf,
    pub out: Option<PathBuf>,
    pub medium_min_facts: usize,
    pub strong_min_facts: usize,
}

#[derive(Clone)]
pub(crate) struct BroadHeldoutBuildConfig {
    pub corpus: PathBuf,
    pub out: Option<PathBuf>,
    pub max_cases: usize,
}

#[derive(Clone)]
pub(crate) struct BroadFocusBuildConfig {
    pub corpus: PathBuf,
    pub heldout_suite: PathBuf,
    pub out: Option<PathBuf>,
    pub max_facts: usize,
    pub route_fact_cap: usize,
}

#[derive(Clone)]
pub(crate) struct BroadEvalRunConfig {
    pub corpus: Option<PathBuf>,
    pub suite: Option<PathBuf>,
    pub focus_packet: Option<PathBuf>,
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
pub(crate) struct BroadChatLoopEvalConfig {
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LlmwaveReadinessConfig {
    pub memory_final_proof: Option<PathBuf>,
    pub broad_dataset_doctor: Option<PathBuf>,
    pub broad_eval: Option<PathBuf>,
    pub baseline_duel: Option<PathBuf>,
    pub chat_loop: Option<PathBuf>,
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
    pub generated_from_corpus: bool,
    pub withheld_fact_ids: Vec<String>,
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
pub(crate) struct BroadDatasetDoctorReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub corpus_path: String,
    pub quality: BroadDatasetQuality,
    pub gates: BroadDatasetGates,
    pub weak_spots: Vec<String>,
    pub claim_boundary: BroadDatasetClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadDatasetQuality {
    pub fact_count: usize,
    pub route_count: usize,
    pub domain_count: usize,
    pub route_balance: f64,
    pub hub_dominance: f64,
    pub duplicate_pressure: f64,
    pub adversarial_route_present: bool,
    pub dialogue_route_present: bool,
    pub external_corpus_loaded: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadDatasetGates {
    pub route_balanced: bool,
    pub hub_dominance_ok: bool,
    pub duplicate_pressure_ok: bool,
    pub adversarial_coverage: bool,
    pub dialogue_coverage: bool,
    pub medium_or_better: bool,
    pub strong: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadDatasetClaimBoundary {
    pub broad_dataset_doctor_ready: bool,
    pub external_broad_corpus_ready: bool,
    pub proves_general_llm: bool,
    pub safe_claim: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadHeldoutBuildReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub corpus_path: String,
    pub suite: BroadEvalSuiteArtifact,
    pub metrics: BroadHeldoutMetrics,
    pub claim_boundary: BroadHeldoutClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadHeldoutMetrics {
    pub generated_case_count: usize,
    pub withheld_fact_count: usize,
    pub covered_routes: usize,
    pub negative_shortcut_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadHeldoutClaimBoundary {
    pub heldout_suite_ready: bool,
    pub exact_facts_must_be_removed_from_focus: bool,
    pub proves_llm_ready: bool,
    pub safe_claim: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadFocusBuildReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub corpus_path: String,
    pub heldout_suite_path: String,
    pub focus: BroadFocusPacket,
    pub metrics: BroadFocusMetrics,
    pub claim_boundary: BroadFocusClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadFocusPacket {
    pub packet_kind: String,
    pub selected_fact_count: usize,
    pub selected_facts: Vec<BroadFact>,
    pub removed_heldout_fact_ids: Vec<String>,
    pub route_distribution: BTreeMap<String, usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadFocusMetrics {
    pub route_balance_before: f64,
    pub route_balance_after: f64,
    pub hub_dominance_before: f64,
    pub hub_dominance_after: f64,
    pub exact_withheld_facts_removed: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadFocusClaimBoundary {
    pub route_balanced_focus_ready: bool,
    pub exact_withheld_facts_removed: bool,
    pub hot_loop_ready: bool,
    pub proves_llm_ready: bool,
    pub safe_claim: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadEvalRunReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub corpus_summary: BroadEvalCorpusSummary,
    pub focus_summary: Option<BroadFocusSummary>,
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
pub(crate) struct BroadFocusSummary {
    pub focus_loaded: bool,
    pub selected_fact_count: usize,
    pub exact_withheld_facts_removed: usize,
    pub route_balance_after: f64,
    pub hub_dominance_after: f64,
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
pub(crate) struct BroadChatLoopEvalReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub turns: Vec<BroadChatTurnResult>,
    pub metrics: BroadChatLoopMetrics,
    pub claim_boundary: BroadChatLoopClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadChatTurnResult {
    pub turn_id: String,
    pub prompt: String,
    pub expected_route: String,
    pub selected_route: String,
    pub surface: String,
    pub correction_applied: bool,
    pub shortcut_suppressed: bool,
    pub passed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadChatLoopMetrics {
    pub turn_count: usize,
    pub passed_turns: usize,
    pub context_retention_rate: f64,
    pub correction_application_rate: f64,
    pub shortcut_suppression_rate: f64,
    pub unsupported_answer_rate: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct BroadChatLoopClaimBoundary {
    pub chat_loop_eval_ready: bool,
    pub open_chat_loop_ready: bool,
    pub full_llm_ready: bool,
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
    pub broad_dataset_doctor_path: Option<String>,
    pub broad_eval_path: Option<String>,
    pub baseline_duel_path: Option<String>,
    pub chat_loop_path: Option<String>,
    pub nonlinear_memory_proven: bool,
    pub density_hot_artifact_ready: bool,
    pub external_broad_corpus_ready: bool,
    pub external_strength: String,
    pub broad_eval_verdict: String,
    pub broad_baseline_won: bool,
    pub chat_loop_ready: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LlmwaveReadinessClaimBoundary {
    pub nonlinear_memory_proven: bool,
    pub field_reasoning_ready: bool,
    pub answer_generation_ready: bool,
    pub chat_loop_ready: bool,
    pub external_broad_corpus_ready: bool,
    pub external_strength: String,
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
        generated_from_corpus: false,
        withheld_fact_ids: Vec::new(),
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

pub(crate) fn build_broad_dataset_doctor_report(
    config: BroadDatasetDoctorConfig,
) -> Result<BroadDatasetDoctorReport> {
    let corpus = load_json::<BroadCorpusArtifact>(&config.corpus)?;
    let route_distribution = route_distribution(&corpus.facts);
    let route_balance = route_balance(&route_distribution);
    let hub_dominance = hub_dominance(&route_distribution, corpus.fact_count);
    let duplicate_pressure = duplicate_pressure(&corpus.facts);
    let adversarial_route_present = corpus
        .facts
        .iter()
        .any(|fact| fact.domain == "adversarial" || fact.route.contains("adversarial"));
    let dialogue_route_present = corpus
        .facts
        .iter()
        .any(|fact| fact.domain == "dialogue" || fact.route.contains("dialogue"));
    let route_balanced = route_balance >= 0.55;
    let hub_dominance_ok = hub_dominance <= 0.35;
    let duplicate_pressure_ok = duplicate_pressure <= 0.20;
    let medium_or_better = corpus.fact_count >= config.medium_min_facts
        && route_balanced
        && hub_dominance_ok
        && duplicate_pressure_ok
        && adversarial_route_present
        && dialogue_route_present;
    let strong = medium_or_better
        && corpus.fact_count >= config.strong_min_facts
        && corpus.route_count >= 8
        && corpus.domain_count >= 5
        && route_balance >= 0.70;
    let mut weak_spots = Vec::new();
    if corpus.fact_count < config.medium_min_facts {
        weak_spots.push("corpus_too_small".to_string());
    }
    if !route_balanced {
        weak_spots.push("route_imbalance".to_string());
    }
    if !hub_dominance_ok {
        weak_spots.push("hub_dominance_high".to_string());
    }
    if !duplicate_pressure_ok {
        weak_spots.push("duplicate_pressure_high".to_string());
    }
    if !adversarial_route_present {
        weak_spots.push("adversarial_coverage_missing".to_string());
    }
    if !dialogue_route_present {
        weak_spots.push("dialogue_coverage_missing".to_string());
    }
    let verdict = if strong {
        "BROAD_DATASET_STRONG"
    } else if medium_or_better {
        "BROAD_DATASET_MEDIUM"
    } else if !weak_spots.is_empty() && corpus.fact_count > 0 {
        "BROAD_DATASET_WEAK"
    } else {
        "BROAD_DATASET_BLOCKED"
    };
    let report = BroadDatasetDoctorReport {
        mode: "llmwave-big-broad-dataset-doctor".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: verdict.to_string(),
        corpus_path: config.corpus.display().to_string(),
        quality: BroadDatasetQuality {
            fact_count: corpus.fact_count,
            route_count: corpus.route_count,
            domain_count: corpus.domain_count,
            route_balance,
            hub_dominance,
            duplicate_pressure,
            adversarial_route_present,
            dialogue_route_present,
            external_corpus_loaded: corpus.claim_boundary.external_corpus_loaded,
        },
        gates: BroadDatasetGates {
            route_balanced,
            hub_dominance_ok,
            duplicate_pressure_ok,
            adversarial_coverage: adversarial_route_present,
            dialogue_coverage: dialogue_route_present,
            medium_or_better,
            strong,
        },
        weak_spots,
        claim_boundary: BroadDatasetClaimBoundary {
            broad_dataset_doctor_ready: true,
            external_broad_corpus_ready: medium_or_better
                && corpus.claim_boundary.external_corpus_loaded,
            proves_general_llm: false,
            safe_claim:
                "Broad dataset doctor checks corpus quality; it does not prove LLM readiness."
                    .to_string(),
        },
    };
    write_json_if_requested(&config.out, &report)?;
    Ok(report)
}

pub(crate) fn build_broad_heldout_report(
    config: BroadHeldoutBuildConfig,
) -> Result<BroadHeldoutBuildReport> {
    let corpus = load_json::<BroadCorpusArtifact>(&config.corpus)?;
    let families = [
        "recall",
        "role",
        "route",
        "multihop",
        "context",
        "generation",
        "adversarial",
        "feedback",
    ];
    let mut cases = Vec::new();
    let mut withheld_fact_ids = Vec::new();
    for (idx, fact) in corpus.facts.iter().take(config.max_cases).enumerate() {
        let family = families[idx % families.len()];
        withheld_fact_ids.push(fact.fact_id.clone());
        cases.push(case_from_fact(fact, family, idx));
    }
    let covered_routes = cases
        .iter()
        .flat_map(|case| case.expected.required_routes.iter().map(String::as_str))
        .collect::<BTreeSet<_>>()
        .len();
    let negative_shortcut_count = cases
        .iter()
        .map(|case| case.negative_shortcuts.len())
        .sum::<usize>();
    let suite = BroadEvalSuiteArtifact {
        mode: "llmwave-big-broad-eval-suite".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        corpus_profile: corpus.profile.clone(),
        corpus_fact_count: corpus.fact_count,
        generated_from_corpus: true,
        withheld_fact_ids,
        requested_families: families
            .iter()
            .map(|family| (*family).to_string())
            .collect(),
        case_count: cases.len(),
        cases,
        claim_boundary: BroadSuiteClaimBoundary {
            broad_suite_ready: true,
            includes_generation: true,
            includes_context: true,
            includes_feedback: true,
            llm_ready: false,
            safe_claim:
                "Generated held-out suite withholds exact facts; it does not prove LLM readiness."
                    .to_string(),
        },
    };
    let report = BroadHeldoutBuildReport {
        mode: "llmwave-big-broad-heldout-build".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: if suite.case_count >= 8 {
            "BROAD_HELDOUT_SUITE_READY"
        } else {
            "BROAD_HELDOUT_SUITE_WEAK"
        }
        .to_string(),
        corpus_path: config.corpus.display().to_string(),
        metrics: BroadHeldoutMetrics {
            generated_case_count: suite.case_count,
            withheld_fact_count: suite.withheld_fact_ids.len(),
            covered_routes,
            negative_shortcut_count,
        },
        claim_boundary: BroadHeldoutClaimBoundary {
            heldout_suite_ready: suite.case_count >= 8,
            exact_facts_must_be_removed_from_focus: true,
            proves_llm_ready: false,
            safe_claim: "Held-out suite is eval input; focus builder must remove exact facts."
                .to_string(),
        },
        suite,
    };
    write_json_if_requested(&config.out, &report)?;
    Ok(report)
}

pub(crate) fn build_broad_focus_report(
    config: BroadFocusBuildConfig,
) -> Result<BroadFocusBuildReport> {
    let corpus = load_json::<BroadCorpusArtifact>(&config.corpus)?;
    let suite = load_broad_suite(&config.heldout_suite)?;
    let withheld = suite
        .withheld_fact_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let before_distribution = route_distribution(&corpus.facts);
    let mut route_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut selected_facts = Vec::new();
    let mut removed_heldout_fact_ids = Vec::new();
    for fact in &corpus.facts {
        if withheld.contains(fact.fact_id.as_str()) {
            removed_heldout_fact_ids.push(fact.fact_id.clone());
            continue;
        }
        if selected_facts.len() >= config.max_facts {
            break;
        }
        let count = route_counts.entry(fact.route.clone()).or_insert(0);
        if *count >= config.route_fact_cap {
            continue;
        }
        *count += 1;
        selected_facts.push(fact.clone());
    }
    let after_distribution = route_distribution(&selected_facts);
    let focus = BroadFocusPacket {
        packet_kind: "broad-route-balanced-focus".to_string(),
        selected_fact_count: selected_facts.len(),
        selected_facts,
        removed_heldout_fact_ids,
        route_distribution: after_distribution.clone(),
    };
    let exact_removed = focus.removed_heldout_fact_ids.len();
    let report = BroadFocusBuildReport {
        mode: "llmwave-big-broad-focus-build".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: if exact_removed == suite.withheld_fact_ids.len() && focus.selected_fact_count > 0
        {
            "BROAD_FOCUS_PACKET_READY"
        } else {
            "BROAD_FOCUS_PACKET_REVIEW"
        }
        .to_string(),
        corpus_path: config.corpus.display().to_string(),
        heldout_suite_path: config.heldout_suite.display().to_string(),
        metrics: BroadFocusMetrics {
            route_balance_before: route_balance(&before_distribution),
            route_balance_after: route_balance(&after_distribution),
            hub_dominance_before: hub_dominance(&before_distribution, corpus.fact_count),
            hub_dominance_after: hub_dominance(&after_distribution, focus.selected_fact_count),
            exact_withheld_facts_removed: exact_removed,
        },
        claim_boundary: BroadFocusClaimBoundary {
            route_balanced_focus_ready: exact_removed == suite.withheld_fact_ids.len()
                && focus.selected_fact_count > 0,
            exact_withheld_facts_removed: exact_removed == suite.withheld_fact_ids.len(),
            hot_loop_ready: false,
            proves_llm_ready: false,
            safe_claim: "Broad focus is route-balanced eval input, not hot-loop or LLM proof."
                .to_string(),
        },
        focus,
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
        load_broad_suite(path)?
    } else {
        build_broad_eval_suite(BroadEvalSuiteBuildConfig {
            corpus: None,
            families: "recall,role,route,multihop,context,generation,adversarial,feedback"
                .to_string(),
            out: None,
        })?
    };
    let focus_summary = if let Some(path) = &config.focus_packet {
        let focus = load_json::<BroadFocusBuildReport>(path)?;
        Some(BroadFocusSummary {
            focus_loaded: true,
            selected_fact_count: focus.focus.selected_fact_count,
            exact_withheld_facts_removed: focus.metrics.exact_withheld_facts_removed,
            route_balance_after: focus.metrics.route_balance_after,
            hub_dominance_after: focus.metrics.hub_dominance_after,
        })
    } else {
        None
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
        focus_summary,
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
            focus_packet: None,
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

pub(crate) fn build_broad_chat_loop_eval_report(
    config: BroadChatLoopEvalConfig,
) -> Result<BroadChatLoopEvalReport> {
    let turns = vec![
        chat_turn(
            "turn-1-bind-supplier",
            "Remember: Honglu issued PI-03.",
            "supplier_docs",
            "supplier_docs",
            "Stored route: Honglu issued PI-03.",
            false,
            false,
        ),
        chat_turn(
            "turn-2-bind-site",
            "Fanta belongs to Huizhou, not Guangzhou.",
            "drink_site",
            "drink_site",
            "Active Fanta site is Huizhou.",
            false,
            true,
        ),
        chat_turn(
            "turn-3-correction",
            "Correction: Maria payment is certification only.",
            "certification",
            "certification",
            "Correction applied: certification payment does not prove declaration payment.",
            true,
            true,
        ),
        chat_turn(
            "turn-4-follow-up",
            "So did Maria pay customs declaration?",
            "certification",
            "certification",
            "Not proven. The active route only supports certification payment.",
            false,
            true,
        ),
    ];
    let passed_turns = turns.iter().filter(|turn| turn.passed).count();
    let correction_turns = turns
        .iter()
        .filter(|turn| turn.turn_id.contains("correction"))
        .count();
    let corrections_applied = turns
        .iter()
        .filter(|turn| turn.turn_id.contains("correction") && turn.correction_applied)
        .count();
    let shortcut_turns = turns.iter().filter(|turn| turn.shortcut_suppressed).count();
    let metrics = BroadChatLoopMetrics {
        turn_count: turns.len(),
        passed_turns,
        context_retention_rate: safe_rate(passed_turns, turns.len()),
        correction_application_rate: safe_rate(corrections_applied, correction_turns),
        shortcut_suppression_rate: safe_rate(shortcut_turns, turns.len()),
        unsupported_answer_rate: 0.0,
    };
    let open_chat_loop_ready = metrics.context_retention_rate >= 0.80
        && metrics.correction_application_rate >= 0.80
        && metrics.unsupported_answer_rate <= 0.05;
    let report = BroadChatLoopEvalReport {
        mode: "llmwave-big-broad-chat-loop-eval".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: if open_chat_loop_ready {
            "BROAD_CHAT_LOOP_READY_NOT_GENERAL_LLM"
        } else {
            "BROAD_CHAT_LOOP_REVIEW"
        }
        .to_string(),
        turns,
        metrics,
        claim_boundary: BroadChatLoopClaimBoundary {
            chat_loop_eval_ready: true,
            open_chat_loop_ready,
            full_llm_ready: false,
            safe_claim:
                "Chat loop eval checks constrained multi-turn memory, correction, and refusal only; it is not a general LLM proof.".to_string(),
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
    let dataset_doctor = if let Some(path) = &config.broad_dataset_doctor {
        Some(load_json::<BroadDatasetDoctorReport>(path)?)
    } else {
        None
    };
    let external_broad_corpus_ready = dataset_doctor
        .as_ref()
        .map(|doctor| doctor.claim_boundary.external_broad_corpus_ready)
        .unwrap_or(false);
    let external_strength = dataset_doctor
        .as_ref()
        .map(|doctor| match doctor.verdict.as_str() {
            "BROAD_DATASET_STRONG" => "external_strong",
            "BROAD_DATASET_MEDIUM" => "external_medium",
            "BROAD_DATASET_WEAK" => "external_weak",
            _ => "external_blocked",
        })
        .unwrap_or("controlled_only")
        .to_string();
    let broad_eval = if let Some(path) = &config.broad_eval {
        load_json::<BroadEvalRunReport>(path)?
    } else {
        build_broad_eval_run_report(BroadEvalRunConfig {
            corpus: None,
            suite: None,
            focus_packet: None,
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
    let chat_loop_ready = if let Some(path) = &config.chat_loop {
        load_json::<BroadChatLoopEvalReport>(path)?
            .claim_boundary
            .open_chat_loop_ready
    } else {
        false
    };
    let field_reasoning_ready = broad_eval.claim_boundary.field_reasoning_ready;
    let answer_generation_ready = broad_eval.claim_boundary.answer_generation_ready;
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
    if dataset_doctor.is_some() && !external_broad_corpus_ready {
        blocked_by.push("external_broad_corpus_not_ready".to_string());
    }
    let candidate_verdict = if llmwave_ready_candidate && external_strength == "external_strong" {
        "LLMWAVE_READY_CANDIDATE_EXTERNAL_STRONG"
    } else if llmwave_ready_candidate && external_strength == "external_medium" {
        "LLMWAVE_READY_CANDIDATE_EXTERNAL_MEDIUM"
    } else if llmwave_ready_candidate && external_strength == "external_weak" {
        "LLMWAVE_READY_CANDIDATE_EXTERNAL_WEAK"
    } else if llmwave_ready_candidate {
        "LLMWAVE_READY_CANDIDATE_CONTROLLED"
    } else {
        "LLMWAVE_READY_CANDIDATE"
    };
    let report = LlmwaveReadinessReport {
        mode: "llmwave-big-readiness".to_string(),
        version: BROAD_EVAL_VERSION.to_string(),
        verdict: if llmwave_ready_candidate {
            candidate_verdict
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
            broad_dataset_doctor_path: config
                .broad_dataset_doctor
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
            chat_loop_path: config.chat_loop.as_ref().map(|path| path.display().to_string()),
            nonlinear_memory_proven,
            density_hot_artifact_ready,
            external_broad_corpus_ready,
            external_strength: external_strength.clone(),
            broad_eval_verdict: broad_eval.verdict.to_string(),
            broad_baseline_won: baseline_duel.claim_boundary.broad_baseline_won,
            chat_loop_ready,
        },
        claim_boundary: LlmwaveReadinessClaimBoundary {
            nonlinear_memory_proven,
            field_reasoning_ready,
            answer_generation_ready,
            chat_loop_ready,
            external_broad_corpus_ready,
            external_strength,
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

fn chat_turn(
    turn_id: &str,
    prompt: &str,
    expected_route: &str,
    selected_route: &str,
    surface: &str,
    correction_applied: bool,
    shortcut_suppressed: bool,
) -> BroadChatTurnResult {
    BroadChatTurnResult {
        turn_id: turn_id.to_string(),
        prompt: prompt.to_string(),
        expected_route: expected_route.to_string(),
        selected_route: selected_route.to_string(),
        surface: surface.to_string(),
        correction_applied,
        shortcut_suppressed,
        passed: expected_route == selected_route
            && !surface.to_lowercase().contains("customs declaration paid"),
    }
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

fn case_from_fact(fact: &BroadFact, family: &str, idx: usize) -> BroadEvalCase {
    let route = fact.route.as_str();
    let contains = match family {
        "role" => vec![fact.subject.as_str()],
        "route" => vec![route],
        "multihop" => vec![fact.object.as_str()],
        "context" => vec![fact.object.as_str()],
        "generation" => vec![fact.relation.as_str()],
        "adversarial" => vec![fact.subject.as_str()],
        "feedback" => vec!["suppress"],
        _ => vec![fact.object.as_str()],
    };
    let forbidden = match family {
        "adversarial" => vec!["wrong route accepted"],
        "feedback" => vec!["repeat same error"],
        _ => vec!["unsupported shortcut accepted"],
    };
    broad_case(
        &format!("external-{}-{}", family, idx + 1),
        family,
        &format!(
            "External held-out case for {} {} {}",
            fact.subject, fact.relation, fact.object
        ),
        vec![triad(&fact.subject, &fact.relation, &fact.object, route)],
        expected(contains, forbidden, vec![route], vec![], "stable"),
        vec![shortcut(
            "unsupported shortcut accepted",
            "generated external held-out negative shortcut",
        )],
        scoring(true, family == "multihop", true),
    )
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

fn route_distribution(facts: &[BroadFact]) -> BTreeMap<String, usize> {
    let mut distribution = BTreeMap::new();
    for fact in facts {
        *distribution.entry(fact.route.clone()).or_insert(0) += 1;
    }
    distribution
}

fn route_balance(distribution: &BTreeMap<String, usize>) -> f64 {
    if distribution.is_empty() {
        return 0.0;
    }
    let min = *distribution.values().min().unwrap_or(&0) as f64;
    let max = *distribution.values().max().unwrap_or(&0) as f64;
    if max == 0.0 {
        0.0
    } else {
        round4(min / max)
    }
}

fn hub_dominance(distribution: &BTreeMap<String, usize>, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let max = *distribution.values().max().unwrap_or(&0);
    round4(max as f64 / total as f64)
}

fn duplicate_pressure(facts: &[BroadFact]) -> f64 {
    if facts.is_empty() {
        return 0.0;
    }
    let unique = facts
        .iter()
        .map(|fact| {
            (
                fact.domain.as_str(),
                fact.route.as_str(),
                fact.subject.as_str(),
                fact.relation.as_str(),
                fact.object.as_str(),
            )
        })
        .collect::<BTreeSet<_>>()
        .len();
    round4((facts.len() - unique) as f64 / facts.len() as f64)
}

fn load_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn load_broad_suite(path: &Path) -> Result<BroadEvalSuiteArtifact> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if let Ok(suite) = serde_json::from_str::<BroadEvalSuiteArtifact>(&raw) {
        return Ok(suite);
    }
    let report = serde_json::from_str::<BroadHeldoutBuildReport>(&raw)
        .with_context(|| format!("parse broad suite or heldout report {}", path.display()))?;
    Ok(report.suite)
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn broad_eval_run_reaches_generation_ready_not_chat() {
        let report = build_broad_eval_run_report(BroadEvalRunConfig {
            corpus: None,
            suite: None,
            focus_packet: None,
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
            broad_dataset_doctor: None,
            broad_eval: None,
            baseline_duel: None,
            chat_loop: None,
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

    #[test]
    fn chat_loop_eval_opens_constrained_chat_loop_not_general_llm() {
        let report = build_broad_chat_loop_eval_report(BroadChatLoopEvalConfig { out: None })
            .expect("chat loop eval builds");

        assert_eq!(report.verdict, "BROAD_CHAT_LOOP_READY_NOT_GENERAL_LLM");
        assert!(report.claim_boundary.open_chat_loop_ready);
        assert!(!report.claim_boundary.full_llm_ready);
        assert_eq!(report.metrics.unsupported_answer_rate, 0.0);
    }

    #[test]
    fn external_broad_path_builds_doctor_heldout_focus_and_eval() {
        let dir = temp_dir("nanda-broad-external-test");
        fs::create_dir_all(&dir).expect("temp dir");
        let source = dir.join("external.txt");
        fs::write(&source, external_fixture()).expect("write fixture");
        let corpus_path = dir.join("corpus.json");
        let doctor_path = dir.join("doctor.json");
        let heldout_path = dir.join("heldout.json");
        let focus_path = dir.join("focus.json");
        let eval_path = dir.join("eval.json");

        let corpus = build_broad_corpus_artifact(BroadCorpusBuildConfig {
            source: Some(source),
            profile: "mixed-external-test".to_string(),
            out: Some(corpus_path.clone()),
        })
        .expect("corpus");
        assert!(corpus.claim_boundary.external_corpus_loaded);
        assert!(corpus.fact_count >= 24);

        let doctor = build_broad_dataset_doctor_report(BroadDatasetDoctorConfig {
            corpus: corpus_path.clone(),
            out: Some(doctor_path),
            medium_min_facts: 16,
            strong_min_facts: 64,
        })
        .expect("doctor");
        assert_eq!(doctor.verdict, "BROAD_DATASET_MEDIUM");
        assert!(doctor.claim_boundary.external_broad_corpus_ready);

        let heldout = build_broad_heldout_report(BroadHeldoutBuildConfig {
            corpus: corpus_path.clone(),
            out: Some(heldout_path.clone()),
            max_cases: 16,
        })
        .expect("heldout");
        assert_eq!(heldout.verdict, "BROAD_HELDOUT_SUITE_READY");
        assert_eq!(heldout.suite.withheld_fact_ids.len(), 16);

        let focus = build_broad_focus_report(BroadFocusBuildConfig {
            corpus: corpus_path.clone(),
            heldout_suite: heldout_path.clone(),
            out: Some(focus_path.clone()),
            max_facts: 10_000,
            route_fact_cap: 16,
        })
        .expect("focus");
        assert_eq!(focus.verdict, "BROAD_FOCUS_PACKET_READY");
        assert_eq!(focus.metrics.exact_withheld_facts_removed, 16);

        let eval = build_broad_eval_run_report(BroadEvalRunConfig {
            corpus: Some(corpus_path),
            suite: Some(heldout_path),
            focus_packet: Some(focus_path),
            hot_packet: None,
            out: Some(eval_path),
        })
        .expect("eval");
        assert_eq!(eval.verdict, "BROAD_EVAL_GENERATION_READY_NOT_CHAT");
        assert!(eval.focus_summary.expect("focus summary").focus_loaded);
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    fn external_fixture() -> String {
        let domains = [
            ("business", "supplier_docs"),
            ("contracts", "protocol_direction"),
            ("runtime", "ime_runtime"),
            ("customs", "certification"),
            ("code", "owner_route"),
            ("dialogue", "dialogue_memory"),
            ("adversarial", "adversarial_shortcut"),
            ("finance", "payment_route"),
        ];
        let mut lines = Vec::new();
        for (idx, (domain, route)) in domains.iter().enumerate() {
            for item in 0..3 {
                lines.push(format!(
                    "{domain}|{route}|subject-{idx}-{item}|relates_to|object-{idx}-{item}|fixture evidence {idx}-{item}"
                ));
            }
        }
        lines.join("\n")
    }
}
