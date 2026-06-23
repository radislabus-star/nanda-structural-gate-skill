//! Rust route-balanced focus packet builder for LLMWave-Big proof gates.
//!
//! This is cold proof-prep code. It consumes the Rust structural corpus plus
//! held-out suite and emits a route-balanced focus packet; it is not the final
//! proof gate and does not claim nonlinear memory or LLM readiness.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const RUST_FOCUS_VERSION: &str = "llmwave-big-v-next-rust-focus-build";

#[derive(Clone)]
pub(crate) struct RustFocusBuildConfig {
    pub artifact: PathBuf,
    pub heldout_suite: PathBuf,
    pub out: Option<PathBuf>,
    pub max_facts: usize,
    pub route_fact_cap: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustFocusBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub profile: &'static str,
    pub input_artifact: String,
    pub input_heldout_suite: String,
    pub focus: RustFocusSummary,
    pub metrics: RustFocusMetrics,
    pub route_distribution_before: Vec<RustFocusRouteCount>,
    pub route_distribution_after: Vec<RustFocusRouteCount>,
    pub selected_facts: Vec<OwnedRustStructuralFact>,
    pub heldout_cases: Vec<OwnedRustHeldoutCase>,
    pub negative_shortcuts: Vec<OwnedRustNegativeShortcut>,
    pub output: RustFocusOutput,
    pub claim_boundary: RustFocusClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustFocusSummary {
    pub packet_kind: &'static str,
    pub packet_hash: String,
    pub source_corpus_hash: String,
    pub heldout_suite_hash: String,
    pub source_fact_count: usize,
    pub selected_fact_count: usize,
    pub heldout_case_count: usize,
    pub negative_shortcut_count: usize,
    pub route_count: usize,
    pub max_facts: usize,
    pub route_fact_cap: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustFocusMetrics {
    pub route_balance_before: f64,
    pub route_balance_after: f64,
    pub source_to_focus_ratio: f64,
    pub heldout_routes_covered: bool,
    pub exact_withheld_facts_removed: usize,
    pub focus_packet_ready: bool,
    pub final_proof_gate_passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustFocusRouteCount {
    pub route: String,
    pub fact_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustFocusOutput {
    pub focus_written: bool,
    pub focus_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustFocusClaimBoundary {
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
    facts: Vec<OwnedRustStructuralFact>,
}

#[derive(Deserialize)]
struct OwnedRustCorpusArtifact {
    corpus_hash: String,
    rust_files: usize,
}

#[derive(Deserialize)]
struct RustHeldoutSuitePayload {
    suite: OwnedRustHeldoutSuite,
    metrics: OwnedRustHeldoutMetrics,
    heldout_cases: Vec<OwnedRustHeldoutCase>,
    negative_shortcuts: Vec<OwnedRustNegativeShortcut>,
}

#[derive(Deserialize)]
struct OwnedRustHeldoutSuite {
    suite_hash: String,
    heldout_case_count: usize,
}

#[derive(Deserialize)]
struct OwnedRustHeldoutMetrics {
    heldout_suite_ready: bool,
}

struct RustFocusWritePayload<'a> {
    focus: &'a RustFocusSummary,
    metrics: &'a RustFocusMetrics,
    route_distribution_before: &'a [RustFocusRouteCount],
    route_distribution_after: &'a [RustFocusRouteCount],
    selected_facts: &'a [OwnedRustStructuralFact],
    heldout_cases: &'a [OwnedRustHeldoutCase],
    negative_shortcuts: &'a [OwnedRustNegativeShortcut],
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct OwnedRustStructuralFact {
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub evidence_path: String,
    pub evidence_line: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct OwnedRustHeldoutCase {
    pub case_id: String,
    pub question_kind: String,
    pub route: String,
    pub withheld_subject: String,
    pub withheld_relation: String,
    pub withheld_object: String,
    pub query: String,
    pub expected_answer: String,
    pub evidence_path: String,
    pub evidence_line: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct OwnedRustNegativeShortcut {
    pub shortcut_id: String,
    pub bad_claim: String,
    pub forbidden_reason: String,
    pub anti_route: String,
}

pub(crate) fn build_rust_focus_report(
    config: RustFocusBuildConfig,
) -> Result<RustFocusBuildReport> {
    let artifact_raw = fs::read_to_string(&config.artifact)
        .with_context(|| format!("read Rust corpus artifact {}", config.artifact.display()))?;
    let corpus: RustCorpusArtifactPayload = serde_json::from_str(&artifact_raw)
        .with_context(|| format!("parse Rust corpus artifact {}", config.artifact.display()))?;
    let heldout_raw = fs::read_to_string(&config.heldout_suite).with_context(|| {
        format!(
            "read Rust held-out suite {}",
            config.heldout_suite.display()
        )
    })?;
    let heldout: RustHeldoutSuitePayload =
        serde_json::from_str(&heldout_raw).with_context(|| {
            format!(
                "parse Rust held-out suite {}",
                config.heldout_suite.display()
            )
        })?;
    let max_facts = config.max_facts.max(1);
    let route_fact_cap = config.route_fact_cap.max(1);
    let route_distribution_before = route_counts(&corpus.facts);
    let route_balance_before = balance_ratio(&route_distribution_before);
    let withheld_keys = heldout_exact_keys(&heldout.heldout_cases);
    let selected_facts =
        select_focus_facts(&corpus.facts, &withheld_keys, max_facts, route_fact_cap);
    let exact_withheld_facts_removed = corpus
        .facts
        .iter()
        .filter(|fact| withheld_keys.contains(&fact_key(fact)))
        .count();
    let route_distribution_after = route_counts(&selected_facts);
    let route_balance_after = balance_ratio(&route_distribution_after);
    let heldout_routes_covered = heldout
        .heldout_cases
        .iter()
        .map(|case| case.route.as_str())
        .collect::<BTreeSet<_>>()
        .iter()
        .all(|route| {
            route_distribution_after
                .iter()
                .any(|count| count.route == *route && count.fact_count > 0)
        });
    let focus_packet_ready = corpus.artifact.rust_files > 0
        && heldout.metrics.heldout_suite_ready
        && selected_facts.len() <= max_facts
        && route_balance_after <= 3.0
        && heldout_routes_covered;
    let mut hash = Sha256::new();
    hash.update(corpus.artifact.corpus_hash.as_bytes());
    hash.update(heldout.suite.suite_hash.as_bytes());
    for fact in &selected_facts {
        hash.update(fact.route.as_bytes());
        hash.update(fact.subject.as_bytes());
        hash.update(fact.relation.as_bytes());
        hash.update(fact.object.as_bytes());
    }
    for case in &heldout.heldout_cases {
        hash.update(case.case_id.as_bytes());
    }
    let focus = RustFocusSummary {
        packet_kind: "rust-code-route-balanced-focus",
        packet_hash: format!("{:x}", hash.finalize()),
        source_corpus_hash: corpus.artifact.corpus_hash,
        heldout_suite_hash: heldout.suite.suite_hash,
        source_fact_count: corpus.facts.len(),
        selected_fact_count: selected_facts.len(),
        heldout_case_count: heldout.suite.heldout_case_count,
        negative_shortcut_count: heldout.negative_shortcuts.len(),
        route_count: route_distribution_after.len(),
        max_facts,
        route_fact_cap,
    };
    let metrics = RustFocusMetrics {
        route_balance_before,
        route_balance_after,
        source_to_focus_ratio: ratio(selected_facts.len(), corpus.facts.len()),
        heldout_routes_covered,
        exact_withheld_facts_removed,
        focus_packet_ready,
        final_proof_gate_passed: false,
    };
    let output = write_focus_if_requested(
        &config.out,
        RustFocusWritePayload {
            focus: &focus,
            metrics: &metrics,
            route_distribution_before: &route_distribution_before,
            route_distribution_after: &route_distribution_after,
            selected_facts: &selected_facts,
            heldout_cases: &heldout.heldout_cases,
            negative_shortcuts: &heldout.negative_shortcuts,
        },
    )?;
    let verdict = if focus_packet_ready {
        "RUST_FOCUS_PACKET_READY"
    } else {
        "RUST_FOCUS_PACKET_REVIEW"
    };
    let mut blocked_by = vec!["compile_test_evidence_bridge_missing"];
    if !focus_packet_ready {
        blocked_by.insert(0, "rust_route_balanced_focus_packet_incomplete");
    }

    Ok(RustFocusBuildReport {
        mode: "llmwave-big-rust-focus-build",
        version: RUST_FOCUS_VERSION,
        verdict,
        profile: "rust",
        input_artifact: config.artifact.display().to_string(),
        input_heldout_suite: config.heldout_suite.display().to_string(),
        focus,
        metrics,
        route_distribution_before,
        route_distribution_after,
        selected_facts: selected_facts.iter().take(512).cloned().collect(),
        heldout_cases: heldout.heldout_cases.iter().take(128).cloned().collect(),
        negative_shortcuts: heldout.negative_shortcuts,
        output,
        claim_boundary: RustFocusClaimBoundary {
            rust_corpus_loaded: corpus.artifact.rust_files > 0,
            heldout_suite_ready: heldout.metrics.heldout_suite_ready,
            focus_packet_ready,
            final_proof_gate_passed: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Rust route-balanced focus packet is ready for final-proof wiring, but it has not proved nonlinear memory or LLM readiness.",
            blocked_by,
        },
    })
}

fn select_focus_facts(
    facts: &[OwnedRustStructuralFact],
    withheld_keys: &BTreeSet<String>,
    max_facts: usize,
    route_fact_cap: usize,
) -> Vec<OwnedRustStructuralFact> {
    let mut selected = Vec::new();
    let mut per_route = BTreeMap::<String, usize>::new();
    for fact in facts {
        if selected.len() >= max_facts {
            break;
        }
        if withheld_keys.contains(&fact_key(fact)) {
            continue;
        }
        let count = per_route.entry(fact.route.clone()).or_insert(0);
        if *count >= route_fact_cap {
            continue;
        }
        *count += 1;
        selected.push(fact.clone());
    }
    selected
}

fn heldout_exact_keys(cases: &[OwnedRustHeldoutCase]) -> BTreeSet<String> {
    cases
        .iter()
        .map(|case| {
            [
                case.route.as_str(),
                case.withheld_subject.as_str(),
                case.withheld_relation.as_str(),
                case.withheld_object.as_str(),
            ]
            .join("\0")
        })
        .collect()
}

fn fact_key(fact: &OwnedRustStructuralFact) -> String {
    [
        fact.route.as_str(),
        fact.subject.as_str(),
        fact.relation.as_str(),
        fact.object.as_str(),
    ]
    .join("\0")
}

fn route_counts(facts: &[OwnedRustStructuralFact]) -> Vec<RustFocusRouteCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for fact in facts {
        *counts.entry(fact.route.clone()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(route, fact_count)| RustFocusRouteCount { route, fact_count })
        .collect()
}

fn balance_ratio(counts: &[RustFocusRouteCount]) -> f64 {
    let Some(max) = counts.iter().map(|count| count.fact_count).max() else {
        return 0.0;
    };
    let Some(min) = counts
        .iter()
        .map(|count| count.fact_count)
        .filter(|count| *count > 0)
        .min()
    else {
        return 0.0;
    };
    round4(max as f64 / min as f64)
}

fn write_focus_if_requested(
    out: &Option<PathBuf>,
    payload: RustFocusWritePayload<'_>,
) -> Result<RustFocusOutput> {
    let Some(path) = out else {
        return Ok(RustFocusOutput {
            focus_written: false,
            focus_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create Rust focus directory {}", parent.display()))?;
    }
    let mut file = fs::File::create(path)
        .with_context(|| format!("create Rust focus packet {}", path.display()))?;
    let payload = serde_json::json!({
        "focus": payload.focus,
        "metrics": payload.metrics,
        "route_distribution_before": payload.route_distribution_before,
        "route_distribution_after": payload.route_distribution_after,
        "facts": payload.selected_facts,
        "heldout_cases": payload.heldout_cases,
        "negative_shortcuts": payload.negative_shortcuts,
    });
    serde_json::to_writer_pretty(&mut file, &payload)
        .with_context(|| format!("write Rust focus packet {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish Rust focus packet {}", path.display()))?;
    Ok(RustFocusOutput {
        focus_written: true,
        focus_path: Some(path.display().to_string()),
    })
}

fn ratio(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        round4(part as f64 / total as f64)
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llmwave_big::rust_corpus::{build_rust_corpus_report, RustCorpusBuildConfig};
    use crate::llmwave_big::rust_heldout::{build_rust_heldout_report, RustHeldoutBuildConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rust_focus_builds_route_balanced_packet() {
        let dir = temp_test_dir("rust-focus-ready");
        let artifact_path = dir.join("rust-corpus.json");
        let heldout_path = dir.join("rust-heldout.json");
        build_rust_corpus_report(RustCorpusBuildConfig {
            repo: PathBuf::from("."),
            out: Some(artifact_path.clone()),
            max_file_bytes: 4 * 1024 * 1024,
        })
        .expect("rust corpus builds");
        build_rust_heldout_report(RustHeldoutBuildConfig {
            artifact: artifact_path.clone(),
            out: Some(heldout_path.clone()),
            max_cases: 64,
        })
        .expect("rust heldout builds");

        let report = build_rust_focus_report(RustFocusBuildConfig {
            artifact: artifact_path,
            heldout_suite: heldout_path,
            out: None,
            max_facts: 15_000,
            route_fact_cap: 256,
        })
        .expect("rust focus builds");

        assert_eq!(report.verdict, "RUST_FOCUS_PACKET_READY");
        assert!(report.metrics.route_balance_before > report.metrics.route_balance_after);
        assert!(report.metrics.route_balance_after <= 3.0);
        assert!(report.metrics.heldout_routes_covered);
        assert!(report.claim_boundary.focus_packet_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn rust_focus_removes_exact_heldout_facts() {
        let dir = temp_test_dir("rust-focus-heldout-mask");
        let artifact_path = dir.join("rust-corpus.json");
        let heldout_path = dir.join("rust-heldout.json");
        build_rust_corpus_report(RustCorpusBuildConfig {
            repo: PathBuf::from("."),
            out: Some(artifact_path.clone()),
            max_file_bytes: 4 * 1024 * 1024,
        })
        .expect("rust corpus builds");
        build_rust_heldout_report(RustHeldoutBuildConfig {
            artifact: artifact_path.clone(),
            out: Some(heldout_path.clone()),
            max_cases: 64,
        })
        .expect("rust heldout builds");

        let report = build_rust_focus_report(RustFocusBuildConfig {
            artifact: artifact_path,
            heldout_suite: heldout_path,
            out: None,
            max_facts: 15_000,
            route_fact_cap: 256,
        })
        .expect("rust focus builds");

        assert!(report.metrics.exact_withheld_facts_removed > 0);
        for case in &report.heldout_cases {
            assert!(!report.selected_facts.iter().any(|fact| {
                fact.route == case.route
                    && fact.subject == case.withheld_subject
                    && fact.relation == case.withheld_relation
                    && fact.object == case.withheld_object
            }));
        }
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
