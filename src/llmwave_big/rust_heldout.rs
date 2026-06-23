//! Rust held-out suite builder for LLMWave-Big proof gates.
//!
//! This is cold proof-prep code. It consumes a Rust structural corpus artifact
//! and emits withheld route questions plus negative shortcuts; it does not
//! claim inference, nonlinear density, or LLM readiness.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const RUST_HELDOUT_VERSION: &str = "llmwave-big-v-next-rust-heldout-build";

#[derive(Clone)]
pub(crate) struct RustHeldoutBuildConfig {
    pub artifact: PathBuf,
    pub out: Option<PathBuf>,
    pub max_cases: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub profile: &'static str,
    pub input_artifact: String,
    pub suite: RustHeldoutSuiteSummary,
    pub metrics: RustHeldoutMetrics,
    pub route_coverage: Vec<RustHeldoutRouteCoverage>,
    pub heldout_cases: Vec<RustHeldoutCase>,
    pub negative_shortcuts: Vec<RustNegativeShortcut>,
    pub output: RustHeldoutOutput,
    pub claim_boundary: RustHeldoutClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutSuiteSummary {
    pub suite_kind: &'static str,
    pub suite_hash: String,
    pub source_corpus_kind: String,
    pub source_corpus_hash: String,
    pub source_fact_count: usize,
    pub train_fact_count: usize,
    pub heldout_case_count: usize,
    pub negative_shortcut_count: usize,
    pub covered_routes: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutMetrics {
    pub heldout_suite_ready: bool,
    pub min_cases_required: usize,
    pub min_routes_required: usize,
    pub route_coverage_ratio: f64,
    pub negative_shortcut_ready: bool,
    pub focus_packet_ready: bool,
    pub final_proof_gate_passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutRouteCoverage {
    pub route: String,
    pub cases: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustHeldoutCase {
    pub case_id: String,
    pub question_kind: &'static str,
    pub route: String,
    pub withheld_subject: String,
    pub withheld_relation: String,
    pub withheld_object: String,
    pub query: String,
    pub expected_answer: String,
    pub evidence_path: String,
    pub evidence_line: usize,
    pub train_mask: RustHeldoutTrainMask,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustHeldoutTrainMask {
    pub hide_subject: String,
    pub hide_relation: String,
    pub hide_object: String,
    pub keep_route_context: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustNegativeShortcut {
    pub shortcut_id: &'static str,
    pub bad_claim: &'static str,
    pub forbidden_reason: &'static str,
    pub anti_route: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutOutput {
    pub suite_written: bool,
    pub suite_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutClaimBoundary {
    pub rust_corpus_loaded: bool,
    pub heldout_suite_ready: bool,
    pub focus_packet_ready: bool,
    pub final_proof_gate_passed: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Deserialize)]
struct RustCorpusArtifactPayload {
    artifact: OwnedRustCorpusArtifact,
    metrics: OwnedRustCorpusMetrics,
    routes: Vec<OwnedRustRouteSummary>,
    facts: Vec<OwnedRustStructuralFact>,
}

#[derive(Deserialize)]
struct OwnedRustCorpusArtifact {
    corpus_kind: String,
    corpus_hash: String,
    rust_files: usize,
}

#[derive(Deserialize)]
struct OwnedRustCorpusMetrics {
    owner_route_count: usize,
}

#[derive(Deserialize)]
struct OwnedRustRouteSummary {
    route: String,
}

#[derive(Deserialize)]
struct OwnedRustStructuralFact {
    route: String,
    subject: String,
    relation: String,
    object: String,
    evidence_path: String,
    evidence_line: usize,
}

pub(crate) fn build_rust_heldout_report(
    config: RustHeldoutBuildConfig,
) -> Result<RustHeldoutBuildReport> {
    let raw = fs::read_to_string(&config.artifact)
        .with_context(|| format!("read Rust corpus artifact {}", config.artifact.display()))?;
    let payload: RustCorpusArtifactPayload = serde_json::from_str(&raw)
        .with_context(|| format!("parse Rust corpus artifact {}", config.artifact.display()))?;
    let max_cases = config.max_cases.max(1);
    let heldout_cases = build_heldout_cases(&payload.facts, max_cases);
    let negative_shortcuts = rust_negative_shortcuts();
    let route_coverage = route_coverage(&heldout_cases);
    let covered_routes = route_coverage.len();
    let min_cases_required = 16usize;
    let min_routes_required = 4usize;
    let heldout_suite_ready =
        heldout_cases.len() >= min_cases_required && covered_routes >= min_routes_required;
    let source_route_count = payload
        .routes
        .iter()
        .filter(|route| !route.route.is_empty())
        .count()
        .max(payload.metrics.owner_route_count.min(1));
    let route_coverage_ratio = if source_route_count == 0 {
        0.0
    } else {
        round4(covered_routes as f64 / source_route_count as f64)
    };
    let train_fact_count = payload.facts.len().saturating_sub(heldout_cases.len());
    let mut hash = Sha256::new();
    hash.update(payload.artifact.corpus_hash.as_bytes());
    for case in &heldout_cases {
        hash.update(case.case_id.as_bytes());
        hash.update(case.withheld_subject.as_bytes());
        hash.update(case.withheld_relation.as_bytes());
        hash.update(case.withheld_object.as_bytes());
    }
    for shortcut in &negative_shortcuts {
        hash.update(shortcut.shortcut_id.as_bytes());
        hash.update(shortcut.bad_claim.as_bytes());
    }
    let suite = RustHeldoutSuiteSummary {
        suite_kind: "rust-code-heldout-suite",
        suite_hash: format!("{:x}", hash.finalize()),
        source_corpus_kind: payload.artifact.corpus_kind.to_string(),
        source_corpus_hash: payload.artifact.corpus_hash,
        source_fact_count: payload.facts.len(),
        train_fact_count,
        heldout_case_count: heldout_cases.len(),
        negative_shortcut_count: negative_shortcuts.len(),
        covered_routes,
    };
    let metrics = RustHeldoutMetrics {
        heldout_suite_ready,
        min_cases_required,
        min_routes_required,
        route_coverage_ratio,
        negative_shortcut_ready: negative_shortcuts.len() >= 3,
        focus_packet_ready: false,
        final_proof_gate_passed: false,
    };
    let output = write_suite_if_requested(
        &config.out,
        &suite,
        &metrics,
        &route_coverage,
        &heldout_cases,
        &negative_shortcuts,
    )?;
    let verdict = if heldout_suite_ready {
        "RUST_HELDOUT_SUITE_READY"
    } else {
        "RUST_HELDOUT_SUITE_REVIEW"
    };
    let mut blocked_by = vec![
        "rust_route_balanced_focus_packet_missing",
        "compile_test_evidence_bridge_missing",
    ];
    if !heldout_suite_ready {
        blocked_by.insert(0, "rust_api_owner_heldout_suite_incomplete");
    }

    Ok(RustHeldoutBuildReport {
        mode: "llmwave-big-rust-heldout-build",
        version: RUST_HELDOUT_VERSION,
        verdict,
        profile: "rust",
        input_artifact: config.artifact.display().to_string(),
        suite,
        metrics,
        route_coverage,
        heldout_cases: heldout_cases.iter().take(128).cloned().collect(),
        negative_shortcuts,
        output,
        claim_boundary: RustHeldoutClaimBoundary {
            rust_corpus_loaded: payload.artifact.rust_files > 0,
            heldout_suite_ready,
            focus_packet_ready: false,
            final_proof_gate_passed: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Rust held-out suite is ready for focus/proof wiring, but it has not proved nonlinear memory or LLM readiness.",
            blocked_by,
        },
    })
}

fn build_heldout_cases(
    facts: &[OwnedRustStructuralFact],
    max_cases: usize,
) -> Vec<RustHeldoutCase> {
    let mut selected = Vec::new();
    let mut per_route = BTreeMap::<String, usize>::new();
    let mut seen = BTreeSet::<String>::new();
    let preferred_routes = [
        "public-api-export",
        "cli-command-dispatch",
        "report-printer",
        "unit-test-proof",
        "module-owner",
    ];

    for route in preferred_routes {
        for fact in facts.iter().filter(|fact| fact.route == route) {
            if selected.len() >= max_cases {
                return selected;
            }
            let count = per_route.entry(route.to_string()).or_insert(0);
            if *count >= route_case_cap(route, max_cases) {
                continue;
            }
            if let Some(case) = heldout_case_for_fact(fact, selected.len()) {
                if seen.insert(format!(
                    "{}\0{}\0{}\0{}",
                    case.route, case.withheld_subject, case.withheld_relation, case.withheld_object
                )) {
                    *count += 1;
                    selected.push(case);
                }
            }
        }
    }
    selected
}

fn route_case_cap(route: &str, max_cases: usize) -> usize {
    let base = (max_cases / 5).max(3);
    match route {
        "module-owner" => base * 2,
        "public-api-export" => base + 2,
        _ => base,
    }
}

fn heldout_case_for_fact(fact: &OwnedRustStructuralFact, index: usize) -> Option<RustHeldoutCase> {
    let (question_kind, query, expected_answer) = match fact.route.as_str() {
        "module-owner" if fact.relation == "owns function" => (
            "function_owner",
            format!("which Rust module owns function {}?", fact.object),
            fact.subject.clone(),
        ),
        "module-owner" if fact.relation == "declares module" => (
            "module_declares_child",
            format!("which Rust module declares child module {}?", fact.object),
            fact.subject.clone(),
        ),
        "public-api-export" => (
            "public_api_owner",
            format!("which Rust module owns public export {}?", fact.object),
            fact.subject.clone(),
        ),
        "cli-command-dispatch" => (
            "cli_dispatch_owner",
            "which Rust module contains this CLI dispatch hint?".to_string(),
            fact.subject.clone(),
        ),
        "report-printer" => (
            "report_printer_owner",
            format!("which Rust module prints or builds report {}?", fact.object),
            fact.subject.clone(),
        ),
        "unit-test-proof" => (
            "unit_test_target",
            format!("which Rust module is verified by test {}?", fact.subject),
            fact.object.clone(),
        ),
        "integration-test-proof" => (
            "integration_test_file",
            format!("which Rust integration route owns {}?", fact.object),
            fact.subject.clone(),
        ),
        _ => return None,
    };

    Some(RustHeldoutCase {
        case_id: format!("rust-heldout-{index:04}-{}", fact.route),
        question_kind,
        route: fact.route.clone(),
        withheld_subject: fact.subject.clone(),
        withheld_relation: fact.relation.to_string(),
        withheld_object: fact.object.clone(),
        query,
        expected_answer,
        evidence_path: fact.evidence_path.clone(),
        evidence_line: fact.evidence_line,
        train_mask: RustHeldoutTrainMask {
            hide_subject: fact.subject.clone(),
            hide_relation: fact.relation.to_string(),
            hide_object: fact.object.clone(),
            keep_route_context: true,
        },
    })
}

fn route_coverage(cases: &[RustHeldoutCase]) -> Vec<RustHeldoutRouteCoverage> {
    let mut counts = BTreeMap::<String, usize>::new();
    for case in cases {
        *counts.entry(case.route.clone()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(route, cases)| RustHeldoutRouteCoverage { route, cases })
        .collect()
}

fn rust_negative_shortcuts() -> Vec<RustNegativeShortcut> {
    vec![
        RustNegativeShortcut {
            shortcut_id: "compiled_command_implies_llm_ready",
            bad_claim: "a compiled Rust command means LLM readiness",
            forbidden_reason: "compilation is implementation evidence, not reasoning or generation proof",
            anti_route: "claim-boundary",
        },
        RustNegativeShortcut {
            shortcut_id: "report_printer_owns_decision",
            bad_claim: "a report printer is the owner of the model decision",
            forbidden_reason: "reporting serializes evidence; decision ownership must remain in the field/proof core",
            anti_route: "report-printer",
        },
        RustNegativeShortcut {
            shortcut_id: "test_helper_is_runtime_owner",
            bad_claim: "a unit test helper owns the production route",
            forbidden_reason: "test evidence can verify a route but must not become the runtime owner",
            anti_route: "unit-test-proof",
        },
        RustNegativeShortcut {
            shortcut_id: "corpus_loaded_proves_nonlinear_memory",
            bad_claim: "loading a Rust corpus proves nonlinear memory",
            forbidden_reason: "nonlinear memory requires held-out transfer, useful-fact density, and false-positive gates",
            anti_route: "nonlinear-memory-claim",
        },
    ]
}

fn write_suite_if_requested(
    out: &Option<PathBuf>,
    suite: &RustHeldoutSuiteSummary,
    metrics: &RustHeldoutMetrics,
    route_coverage: &[RustHeldoutRouteCoverage],
    heldout_cases: &[RustHeldoutCase],
    negative_shortcuts: &[RustNegativeShortcut],
) -> Result<RustHeldoutOutput> {
    let Some(path) = out else {
        return Ok(RustHeldoutOutput {
            suite_written: false,
            suite_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create held-out suite directory {}", parent.display()))?;
    }
    let mut file = fs::File::create(path)
        .with_context(|| format!("create Rust held-out suite {}", path.display()))?;
    let payload = serde_json::json!({
        "suite": suite,
        "metrics": metrics,
        "route_coverage": route_coverage,
        "heldout_cases": heldout_cases,
        "negative_shortcuts": negative_shortcuts,
    });
    serde_json::to_writer_pretty(&mut file, &payload)
        .with_context(|| format!("write Rust held-out suite {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish Rust held-out suite {}", path.display()))?;
    Ok(RustHeldoutOutput {
        suite_written: true,
        suite_path: Some(path.display().to_string()),
    })
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llmwave_big::rust_corpus::{build_rust_corpus_report, RustCorpusBuildConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rust_heldout_builds_from_current_repo_artifact() {
        let dir = temp_test_dir("rust-heldout-ready");
        let artifact_path = dir.join("rust-corpus.json");
        build_rust_corpus_report(RustCorpusBuildConfig {
            repo: PathBuf::from("."),
            out: Some(artifact_path.clone()),
            max_file_bytes: 4 * 1024 * 1024,
        })
        .expect("rust corpus builds");

        let report = build_rust_heldout_report(RustHeldoutBuildConfig {
            artifact: artifact_path,
            out: None,
            max_cases: 64,
        })
        .expect("heldout suite builds");

        assert_eq!(report.verdict, "RUST_HELDOUT_SUITE_READY");
        assert_eq!(report.profile, "rust");
        assert!(report.suite.heldout_case_count >= 16);
        assert!(report.suite.covered_routes >= 4);
        assert!(report.metrics.heldout_suite_ready);
        assert!(report
            .negative_shortcuts
            .iter()
            .any(|shortcut| shortcut.shortcut_id == "compiled_command_implies_llm_ready"));
    }

    #[test]
    fn rust_heldout_keeps_final_claims_closed() {
        let dir = temp_test_dir("rust-heldout-claims");
        let artifact_path = dir.join("rust-corpus.json");
        build_rust_corpus_report(RustCorpusBuildConfig {
            repo: PathBuf::from("."),
            out: Some(artifact_path.clone()),
            max_file_bytes: 4 * 1024 * 1024,
        })
        .expect("rust corpus builds");

        let report = build_rust_heldout_report(RustHeldoutBuildConfig {
            artifact: artifact_path,
            out: None,
            max_cases: 64,
        })
        .expect("heldout suite builds");

        assert!(report.claim_boundary.rust_corpus_loaded);
        assert!(report.claim_boundary.heldout_suite_ready);
        assert!(!report.claim_boundary.focus_packet_ready);
        assert!(!report.claim_boundary.final_proof_gate_passed);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
    }

    fn temp_test_dir(slug: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nanda-{slug}-{stamp}"));
        fs::create_dir_all(&dir).expect("create temp test dir");
        dir
    }
}
