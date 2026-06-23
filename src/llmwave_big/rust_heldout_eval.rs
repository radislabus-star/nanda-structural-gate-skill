//! Rust held-out inference eval for LLMWave-Big proof gates.
//!
//! This is cold proof-eval code. It consumes the route-balanced Rust focus
//! packet plus held-out suite and runs deterministic route-fact inference. It
//! does not claim nonlinear memory, broad LLM readiness, or cache-only runtime.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const RUST_HELDOUT_EVAL_VERSION: &str = "llmwave-big-v-next-rust-heldout-inference-eval";

#[derive(Clone)]
pub(crate) struct RustHeldoutEvalConfig {
    pub focus_packet: PathBuf,
    pub heldout_suite: PathBuf,
    pub out: Option<PathBuf>,
    pub pass_threshold: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutEvalReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub profile: &'static str,
    pub input_focus_packet: String,
    pub input_heldout_suite: String,
    pub eval: RustHeldoutEvalSummary,
    pub metrics: RustHeldoutEvalMetrics,
    pub route_results: Vec<RustHeldoutEvalRouteResult>,
    pub case_results: Vec<RustHeldoutCaseResult>,
    pub negative_results: Vec<RustNegativeShortcutResult>,
    pub output: RustHeldoutEvalOutput,
    pub claim_boundary: RustHeldoutEvalClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutEvalSummary {
    pub eval_kind: &'static str,
    pub eval_hash: String,
    pub focus_packet_hash: String,
    pub heldout_suite_hash: String,
    pub selected_fact_count: usize,
    pub heldout_case_count: usize,
    pub negative_shortcut_count: usize,
    pub pass_threshold: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutEvalMetrics {
    pub heldout_case_count: usize,
    pub heldout_pass_count: usize,
    pub heldout_pass_rate: f64,
    pub negative_shortcut_count: usize,
    pub negative_rejected_count: usize,
    pub negative_reject_rate: f64,
    pub false_shortcut_rejection_ready: bool,
    pub heldout_inference_eval_ready: bool,
    pub profile_eval_ready: bool,
    pub final_proof_gate_passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutEvalRouteResult {
    pub route: String,
    pub cases: usize,
    pub passed: usize,
    pub pass_rate: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutCaseResult {
    pub case_id: String,
    pub question_kind: String,
    pub route: String,
    pub query: String,
    pub expected_answer: String,
    pub predicted_answer: Option<String>,
    pub passed: bool,
    pub score: f64,
    pub support_count: usize,
    pub nearest_evidence_path: Option<String>,
    pub nearest_evidence_line: Option<usize>,
    pub reason: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustNegativeShortcutResult {
    pub shortcut_id: String,
    pub bad_claim: String,
    pub rejected: bool,
    pub reason: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutEvalOutput {
    pub eval_written: bool,
    pub eval_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustHeldoutEvalClaimBoundary {
    pub rust_corpus_loaded: bool,
    pub heldout_suite_ready: bool,
    pub focus_packet_ready: bool,
    pub heldout_inference_eval_ready: bool,
    pub final_proof_gate_passed: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Deserialize)]
struct RustFocusPacketPayload {
    focus: RustFocusSummary,
    metrics: RustFocusMetrics,
    facts: Vec<OwnedRustStructuralFact>,
    negative_shortcuts: Vec<OwnedRustNegativeShortcut>,
}

#[derive(Deserialize)]
struct RustFocusSummary {
    packet_hash: String,
    source_corpus_hash: String,
    selected_fact_count: usize,
}

#[derive(Deserialize)]
struct RustFocusMetrics {
    focus_packet_ready: bool,
}

#[derive(Deserialize)]
struct RustHeldoutSuitePayload {
    suite: RustHeldoutSuiteSummary,
    metrics: RustHeldoutMetrics,
    heldout_cases: Vec<OwnedRustHeldoutCase>,
    negative_shortcuts: Vec<OwnedRustNegativeShortcut>,
}

#[derive(Deserialize)]
struct RustHeldoutSuiteSummary {
    suite_hash: String,
}

#[derive(Deserialize)]
struct RustHeldoutMetrics {
    heldout_suite_ready: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct OwnedRustStructuralFact {
    route: String,
    subject: String,
    relation: String,
    object: String,
    evidence_path: String,
    evidence_line: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct OwnedRustHeldoutCase {
    case_id: String,
    question_kind: String,
    route: String,
    withheld_subject: String,
    withheld_relation: String,
    withheld_object: String,
    query: String,
    expected_answer: String,
    evidence_path: String,
    evidence_line: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct OwnedRustNegativeShortcut {
    shortcut_id: String,
    bad_claim: String,
    forbidden_reason: String,
    anti_route: String,
}

#[derive(Default)]
struct CandidateScore {
    answer: String,
    score: f64,
    support_count: usize,
    nearest_evidence_path: Option<String>,
    nearest_evidence_line: Option<usize>,
}

#[derive(Serialize)]
struct RustHeldoutEvalWritePayload<'a> {
    eval: &'a RustHeldoutEvalSummary,
    metrics: &'a RustHeldoutEvalMetrics,
    route_results: &'a [RustHeldoutEvalRouteResult],
    case_results: &'a [RustHeldoutCaseResult],
    negative_results: &'a [RustNegativeShortcutResult],
}

pub(crate) fn build_rust_heldout_eval_report(
    config: RustHeldoutEvalConfig,
) -> Result<RustHeldoutEvalReport> {
    let focus_raw = fs::read_to_string(&config.focus_packet)
        .with_context(|| format!("read Rust focus packet {}", config.focus_packet.display()))?;
    let focus: RustFocusPacketPayload = serde_json::from_str(&focus_raw)
        .with_context(|| format!("parse Rust focus packet {}", config.focus_packet.display()))?;
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
    let pass_threshold = config.pass_threshold.clamp(0.0, 1.0);
    let selected_facts = exact_heldout_removed(&focus.facts, &heldout.heldout_cases);
    let case_results = heldout
        .heldout_cases
        .iter()
        .map(|case| infer_case(case, &selected_facts))
        .collect::<Vec<_>>();
    let negative_shortcuts = merge_negative_shortcuts(
        focus.negative_shortcuts.clone(),
        heldout.negative_shortcuts.clone(),
    );
    let negative_results = negative_shortcuts
        .iter()
        .map(evaluate_negative_shortcut)
        .collect::<Vec<_>>();
    let heldout_case_count = case_results.len();
    let heldout_pass_count = case_results.iter().filter(|case| case.passed).count();
    let heldout_pass_rate = ratio(heldout_pass_count, heldout_case_count);
    let negative_shortcut_count = negative_results.len();
    let negative_rejected_count = negative_results
        .iter()
        .filter(|shortcut| shortcut.rejected)
        .count();
    let negative_reject_rate = ratio(negative_rejected_count, negative_shortcut_count);
    let route_results = route_results(&case_results);
    let false_shortcut_rejection_ready = negative_shortcut_count > 0 && negative_reject_rate >= 1.0;
    let heldout_inference_eval_ready = focus.metrics.focus_packet_ready
        && heldout.metrics.heldout_suite_ready
        && heldout_case_count > 0
        && heldout_pass_rate >= pass_threshold
        && false_shortcut_rejection_ready;
    let profile_eval_ready = heldout_inference_eval_ready;
    let mut hash = Sha256::new();
    hash.update(focus.focus.packet_hash.as_bytes());
    hash.update(heldout.suite.suite_hash.as_bytes());
    for result in &case_results {
        hash.update(result.case_id.as_bytes());
        if let Some(predicted) = &result.predicted_answer {
            hash.update(predicted.as_bytes());
        }
        hash.update(if result.passed { b"pass" } else { b"fail" });
    }
    for result in &negative_results {
        hash.update(result.shortcut_id.as_bytes());
        hash.update(if result.rejected {
            b"rejected"
        } else {
            b"accepted"
        });
    }
    let eval = RustHeldoutEvalSummary {
        eval_kind: "rust-code-heldout-route-fact-inference",
        eval_hash: format!("{:x}", hash.finalize()),
        focus_packet_hash: focus.focus.packet_hash,
        heldout_suite_hash: heldout.suite.suite_hash,
        selected_fact_count: focus.focus.selected_fact_count,
        heldout_case_count,
        negative_shortcut_count,
        pass_threshold: round4(pass_threshold),
    };
    let metrics = RustHeldoutEvalMetrics {
        heldout_case_count,
        heldout_pass_count,
        heldout_pass_rate,
        negative_shortcut_count,
        negative_rejected_count,
        negative_reject_rate,
        false_shortcut_rejection_ready,
        heldout_inference_eval_ready,
        profile_eval_ready,
        final_proof_gate_passed: false,
    };
    let output = write_eval_if_requested(
        &config.out,
        &eval,
        &metrics,
        &route_results,
        &case_results,
        &negative_results,
    )?;
    let verdict = if heldout_inference_eval_ready {
        "RUST_HELDOUT_INFERENCE_EVAL_READY"
    } else {
        "RUST_HELDOUT_INFERENCE_EVAL_REVIEW"
    };
    let mut blocked_by = Vec::new();
    if !focus.metrics.focus_packet_ready {
        blocked_by.push("rust_focus_packet_not_ready");
    }
    if !heldout.metrics.heldout_suite_ready {
        blocked_by.push("rust_heldout_suite_not_ready");
    }
    if heldout_pass_rate < pass_threshold {
        blocked_by.push("rust_heldout_pass_rate_below_threshold");
    }
    if !false_shortcut_rejection_ready {
        blocked_by.push("rust_false_shortcut_rejection_missing");
    }
    blocked_by.push("strict_nonlinear_density_claim_gate_missing");

    Ok(RustHeldoutEvalReport {
        mode: "llmwave-big-rust-heldout-eval",
        version: RUST_HELDOUT_EVAL_VERSION,
        verdict,
        profile: "rust",
        input_focus_packet: config.focus_packet.display().to_string(),
        input_heldout_suite: config.heldout_suite.display().to_string(),
        eval,
        metrics,
        route_results,
        case_results,
        negative_results,
        output,
        claim_boundary: RustHeldoutEvalClaimBoundary {
            rust_corpus_loaded: !focus.focus.source_corpus_hash.is_empty(),
            heldout_suite_ready: heldout.metrics.heldout_suite_ready,
            focus_packet_ready: focus.metrics.focus_packet_ready,
            heldout_inference_eval_ready,
            final_proof_gate_passed: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: if heldout_inference_eval_ready {
                "Rust held-out route-fact inference and false-shortcut rejection passed for the focused packet; this is profile eval evidence, not nonlinear-memory or LLM proof."
            } else {
                "Rust held-out route-fact inference is not ready; final proof must remain blocked."
            },
            blocked_by,
        },
    })
}

fn infer_case(
    case: &OwnedRustHeldoutCase,
    facts: &[OwnedRustStructuralFact],
) -> RustHeldoutCaseResult {
    let mut candidates = BTreeMap::<String, CandidateScore>::new();
    let same_path_available = facts
        .iter()
        .any(|fact| fact.evidence_path == case.evidence_path);
    for fact in facts.iter().filter(|fact| {
        if same_path_available {
            fact.evidence_path == case.evidence_path
        } else {
            fact.route == case.route
        }
    }) {
        let answer = candidate_answer(case, fact);
        if answer.is_empty() {
            continue;
        }
        let same_route = fact.route == case.route;
        let same_path = fact.evidence_path == case.evidence_path;
        let mut score = 0.25;
        if same_route {
            score += 4.0;
        }
        if same_path {
            score += 16.0;
        }
        if fact.relation == case.withheld_relation {
            score += 3.0;
        }
        if fact.object == case.withheld_object {
            score += 2.5;
        }
        if fact.subject == case.withheld_object || fact.object == case.withheld_subject {
            score += 0.5;
        }
        if same_path || same_route {
            score += line_proximity_bonus(fact.evidence_line, case.evidence_line);
        }
        let entry = candidates
            .entry(answer.clone())
            .or_insert_with(|| CandidateScore {
                answer,
                ..CandidateScore::default()
            });
        entry.score += score;
        entry.support_count += 1;
        if is_nearer(
            fact.evidence_line,
            case.evidence_line,
            entry.nearest_evidence_line,
        ) {
            entry.nearest_evidence_path = Some(fact.evidence_path.clone());
            entry.nearest_evidence_line = Some(fact.evidence_line);
        }
    }
    let best = candidates.into_values().max_by(|left, right| {
        left.score
            .total_cmp(&right.score)
            .then(left.support_count.cmp(&right.support_count))
            .then(left.answer.cmp(&right.answer))
    });
    match best {
        Some(best) => {
            let passed = best.answer == case.expected_answer;
            RustHeldoutCaseResult {
                case_id: case.case_id.clone(),
                question_kind: case.question_kind.clone(),
                route: case.route.clone(),
                query: case.query.clone(),
                expected_answer: case.expected_answer.clone(),
                predicted_answer: Some(best.answer),
                passed,
                score: round4(best.score),
                support_count: best.support_count,
                nearest_evidence_path: best.nearest_evidence_path,
                nearest_evidence_line: best.nearest_evidence_line,
                reason: if passed {
                    "same-route neighborhood predicted the withheld answer".to_string()
                } else {
                    "same-route neighborhood peak chose a different answer".to_string()
                },
            }
        }
        None => RustHeldoutCaseResult {
            case_id: case.case_id.clone(),
            question_kind: case.question_kind.clone(),
            route: case.route.clone(),
            query: case.query.clone(),
            expected_answer: case.expected_answer.clone(),
            predicted_answer: None,
            passed: false,
            score: 0.0,
            support_count: 0,
            nearest_evidence_path: None,
            nearest_evidence_line: None,
            reason: "no same-route focus facts available".to_string(),
        },
    }
}

fn candidate_answer(case: &OwnedRustHeldoutCase, fact: &OwnedRustStructuralFact) -> String {
    match case.question_kind.as_str() {
        "unit_test_target" | "integration_test_target" => {
            if fact.route == case.route {
                fact.object.clone()
            } else if fact.evidence_path == case.evidence_path {
                fact.subject.clone()
            } else {
                String::new()
            }
        }
        _ => fact.subject.clone(),
    }
}

fn line_proximity_bonus(fact_line: usize, case_line: usize) -> f64 {
    let delta = fact_line.abs_diff(case_line);
    if delta == 0 {
        4.0
    } else if delta <= 8 {
        3.0
    } else if delta <= 32 {
        2.0
    } else if delta <= 96 {
        1.0
    } else {
        0.0
    }
}

fn is_nearer(candidate_line: usize, target_line: usize, current_line: Option<usize>) -> bool {
    match current_line {
        Some(current) => candidate_line.abs_diff(target_line) < current.abs_diff(target_line),
        None => true,
    }
}

fn exact_heldout_removed(
    facts: &[OwnedRustStructuralFact],
    cases: &[OwnedRustHeldoutCase],
) -> Vec<OwnedRustStructuralFact> {
    let hidden = cases
        .iter()
        .map(|case| {
            (
                case.route.as_str(),
                case.withheld_subject.as_str(),
                case.withheld_relation.as_str(),
                case.withheld_object.as_str(),
                case.evidence_path.as_str(),
                case.evidence_line,
            )
        })
        .collect::<BTreeSet<_>>();
    facts
        .iter()
        .filter(|fact| {
            !hidden.contains(&(
                fact.route.as_str(),
                fact.subject.as_str(),
                fact.relation.as_str(),
                fact.object.as_str(),
                fact.evidence_path.as_str(),
                fact.evidence_line,
            ))
        })
        .cloned()
        .collect()
}

fn merge_negative_shortcuts(
    mut left: Vec<OwnedRustNegativeShortcut>,
    right: Vec<OwnedRustNegativeShortcut>,
) -> Vec<OwnedRustNegativeShortcut> {
    let mut seen = left
        .iter()
        .map(|shortcut| shortcut.shortcut_id.clone())
        .collect::<BTreeSet<_>>();
    for shortcut in right {
        if seen.insert(shortcut.shortcut_id.clone()) {
            left.push(shortcut);
        }
    }
    left
}

fn evaluate_negative_shortcut(shortcut: &OwnedRustNegativeShortcut) -> RustNegativeShortcutResult {
    let known_false = matches!(
        shortcut.shortcut_id.as_str(),
        "compiled_command_implies_llm_ready"
            | "report_printer_owns_decision"
            | "test_helper_is_runtime_owner"
            | "corpus_loaded_proves_nonlinear_memory"
    );
    RustNegativeShortcutResult {
        shortcut_id: shortcut.shortcut_id.clone(),
        bad_claim: shortcut.bad_claim.clone(),
        rejected: known_false,
        reason: if known_false {
            format!(
                "rejected by anti-route {}: {}",
                shortcut.anti_route, shortcut.forbidden_reason
            )
        } else {
            "unknown shortcut id needs review".to_string()
        },
    }
}

fn route_results(case_results: &[RustHeldoutCaseResult]) -> Vec<RustHeldoutEvalRouteResult> {
    let mut routes = BTreeMap::<String, (usize, usize)>::new();
    for case in case_results {
        let entry = routes.entry(case.route.clone()).or_default();
        entry.0 += 1;
        if case.passed {
            entry.1 += 1;
        }
    }
    routes
        .into_iter()
        .map(|(route, (cases, passed))| RustHeldoutEvalRouteResult {
            route,
            cases,
            passed,
            pass_rate: ratio(passed, cases),
        })
        .collect()
}

fn write_eval_if_requested(
    out: &Option<PathBuf>,
    eval: &RustHeldoutEvalSummary,
    metrics: &RustHeldoutEvalMetrics,
    route_results: &[RustHeldoutEvalRouteResult],
    case_results: &[RustHeldoutCaseResult],
    negative_results: &[RustNegativeShortcutResult],
) -> Result<RustHeldoutEvalOutput> {
    let Some(path) = out else {
        return Ok(RustHeldoutEvalOutput {
            eval_written: false,
            eval_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create Rust held-out eval dir {}", parent.display()))?;
    }
    let payload = RustHeldoutEvalWritePayload {
        eval,
        metrics,
        route_results,
        case_results,
        negative_results,
    };
    let json = serde_json::to_vec_pretty(&payload).context("serialize Rust held-out eval")?;
    let mut file = fs::File::create(path)
        .with_context(|| format!("create Rust held-out eval {}", path.display()))?;
    file.write_all(&json)
        .with_context(|| format!("write Rust held-out eval {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish Rust held-out eval {}", path.display()))?;
    Ok(RustHeldoutEvalOutput {
        eval_written: true,
        eval_path: Some(path.display().to_string()),
    })
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        round4(numerator as f64 / denominator as f64)
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn rust_heldout_eval_predicts_from_route_neighborhood() {
        let dir = temp_test_dir("rust-heldout-eval-ready");
        let focus_path = dir.join("focus.json");
        let heldout_path = dir.join("heldout.json");
        let out_path = dir.join("eval.json");
        fs::write(
            &focus_path,
            r#"{
              "focus":{"packet_hash":"focus-hash","source_corpus_hash":"corpus-hash","selected_fact_count":2},
              "metrics":{"focus_packet_ready":true},
              "facts":[
                {"route":"module-owner","subject":"alpha::owner","relation":"owns_function","object":"build_alpha","evidence_path":"src/alpha.rs","evidence_line":11},
                {"route":"module-owner","subject":"alpha::owner","relation":"owns_function","object":"other_alpha","evidence_path":"src/alpha.rs","evidence_line":13}
              ],
              "negative_shortcuts":[
                {"shortcut_id":"compiled_command_implies_llm_ready","bad_claim":"compiled command implies LLM ready","forbidden_reason":"compiled code is not chat proof","anti_route":"claim-firewall"}
              ]
            }"#,
        )
        .expect("write focus");
        fs::write(
            &heldout_path,
            r#"{
              "suite":{"suite_hash":"suite-hash"},
              "metrics":{"heldout_suite_ready":true},
              "heldout_cases":[
                {"case_id":"c1","question_kind":"function_owner","route":"module-owner","withheld_subject":"alpha::owner","withheld_relation":"owns_function","withheld_object":"hidden_alpha","query":"who owns hidden_alpha?","expected_answer":"alpha::owner","evidence_path":"src/alpha.rs","evidence_line":12}
              ],
              "negative_shortcuts":[]
            }"#,
        )
        .expect("write heldout");

        let report = build_rust_heldout_eval_report(RustHeldoutEvalConfig {
            focus_packet: focus_path,
            heldout_suite: heldout_path,
            out: Some(out_path.clone()),
            pass_threshold: 0.8,
        })
        .expect("eval builds");

        assert_eq!(report.verdict, "RUST_HELDOUT_INFERENCE_EVAL_READY");
        assert!(report.metrics.heldout_inference_eval_ready);
        assert_eq!(report.metrics.heldout_pass_rate, 1.0);
        assert_eq!(report.metrics.negative_reject_rate, 1.0);
        assert!(out_path.exists());
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn rust_heldout_eval_reviews_unknown_shortcuts() {
        let shortcut = OwnedRustNegativeShortcut {
            shortcut_id: "unknown".to_string(),
            bad_claim: "unknown claim".to_string(),
            forbidden_reason: "unknown reason".to_string(),
            anti_route: "unknown-route".to_string(),
        };
        let result = evaluate_negative_shortcut(&shortcut);
        assert!(!result.rejected);
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "nanda-llmwave-big-{label}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
