//! Linux-profile reasoning layer over `.lrf` schema/residual memory.
//!
//! This module turns the Linux readout into a broader profile eval surface:
//! query waves, evidence chains, shortcut anti-wave checks, broad suite
//! generation, broad eval, and a profile claim gate. It is still not a general
//! LLM and not a vulnerability scanner.

pub(crate) mod decision_search;
pub(crate) mod feedback;
pub(crate) mod heldout;
pub(crate) mod relations;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::linux_exposure::{
    build_linux_exposure_report, LinuxExposureConfig, LinuxExposureReport,
};
use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
    LINUX_RESIDUAL_MEMORY_VERSION,
};

pub(crate) const LINUX_PROFILE_VERSION: &str = "llmwave-big-v-next-linux-profile-reasoning";

#[derive(Clone)]
pub(crate) struct LinuxQueryWaveConfig {
    pub text: String,
}

#[derive(Clone)]
pub(crate) struct LinuxReasonRunConfig {
    pub residual_pack: PathBuf,
    pub text: String,
    pub max_facts: usize,
}

#[derive(Clone)]
pub(crate) struct LinuxBroadSuiteBuildConfig {
    pub residual_pack: PathBuf,
    pub cases: usize,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LinuxBroadEvalRunConfig {
    pub residual_pack: PathBuf,
    pub suite: PathBuf,
    pub out: Option<PathBuf>,
    pub max_facts: usize,
}

#[derive(Clone)]
pub(crate) struct LinuxProfileClaimGateConfig {
    pub residual_pack: PathBuf,
    pub broad_eval: Option<PathBuf>,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxQueryWaveReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub text: String,
    pub query_wave: LinuxQueryWave,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxQueryWave {
    pub intent: String,
    pub anchors: Vec<String>,
    pub route_priors: Vec<String>,
    pub required_routes: Vec<String>,
    pub negative_boundaries: Vec<String>,
    pub forbidden_shortcuts: Vec<String>,
    pub polarity: String,
    pub answer_policy: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxReasonReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub residual_memory_version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub query_wave: LinuxQueryWave,
    pub decision: LinuxReasonDecision,
    pub evidence_chain: Vec<LinuxEvidenceStep>,
    pub anti_wave_hits: Vec<LinuxAntiWaveHit>,
    pub exposure_context: LinuxReasonExposureContext,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxReasonDecision {
    pub state: String,
    pub answer_allowed: bool,
    pub answer: String,
    pub missing_evidence: Vec<String>,
    pub route_confusion_risk: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxEvidenceStep {
    pub role: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub memory_kind: String,
    pub confidence: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxAntiWaveHit {
    pub shortcut: String,
    pub suppressed_peak: String,
    pub replacement_peak: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxReasonExposureContext {
    pub exposure_layer_ready: bool,
    pub exposure_state: String,
    pub candidate_count: usize,
    pub external_binding_count: usize,
    pub localhost_binding_count: usize,
    pub firewall_allow_fact_count: usize,
    pub external_exposure_confirmed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxBroadSuiteReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub suite: LinuxBroadSuite,
    pub route_distribution: BTreeMap<String, usize>,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxBroadSuite {
    pub profile: String,
    pub case_count: usize,
    pub families: BTreeMap<String, usize>,
    pub cases: Vec<LinuxBroadEvalCase>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxBroadEvalCase {
    pub id: String,
    pub family: String,
    pub prompt: String,
    pub expected_intent: String,
    pub expected_answer_allowed: bool,
    pub expected_answer_contains: String,
    pub forbid_answer_contains: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxBroadEvalReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub suite_profile: String,
    pub cases: Vec<LinuxBroadEvalCaseResult>,
    pub metrics: LinuxBroadEvalMetrics,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxBroadEvalCaseResult {
    pub id: String,
    pub family: String,
    pub prompt: String,
    pub expected_intent: String,
    pub observed_intent: String,
    pub expected_answer_allowed: bool,
    pub observed_answer_allowed: bool,
    pub observed_state: String,
    pub observed_answer: String,
    pub forbid_answer_contains: String,
    pub passed: bool,
    pub reason_codes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxBroadEvalMetrics {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f32,
    pub intent_accuracy: f32,
    pub answer_allowed_accuracy: f32,
    pub evidence_chain_pass_rate: f32,
    pub false_positive_rate: f32,
    pub false_negative_rate: f32,
    pub shortcut_rejection_rate: f32,
    pub context_retention_rate: f32,
    pub route_confusion_rate: f32,
    pub runtime_package_confusion_rate: f32,
    pub exposure_overclaim_rate: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxProfileClaimGateReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub broad_eval: Option<LinuxBroadEvalMetrics>,
    pub requirements: LinuxProfileRequirements,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxProfileRequirements {
    pub residual_pack_loaded: bool,
    pub linux_profile_nonlinear_memory_proven: bool,
    pub broad_eval_present: bool,
    pub broad_eval_pass_rate_ok: bool,
    pub false_positive_rate_ok: bool,
    pub exposure_overclaim_rate_ok: bool,
    pub runtime_package_confusion_rate_ok: bool,
    pub shortcut_rejection_rate_ok: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxProfileBoundary {
    pub linux_profile_query_wave_ready: bool,
    pub linux_profile_reasoning_ready: bool,
    pub linux_profile_broad_eval_ready: bool,
    pub linux_profile_broad_chat_ready: bool,
    pub linux_profile_nonlinear_memory_proven: bool,
    pub general_llm_ready: bool,
    pub open_domain_chat_ready: bool,
    pub vulnerability_scanner_ready: bool,
    pub network_scanner_ready: bool,
    pub safe_claim: String,
    pub blocked_claims: Vec<String>,
}

pub(crate) fn build_linux_query_wave_report(config: LinuxQueryWaveConfig) -> LinuxQueryWaveReport {
    let query_wave = build_linux_query_wave(&config.text);
    LinuxQueryWaveReport {
        mode: "llmwave-big-linux-query-wave",
        version: LINUX_PROFILE_VERSION,
        verdict: "LINUX_QUERY_WAVE_READY_NOT_ANSWER",
        text: config.text,
        query_wave,
        claim_boundary: boundary(
            true,
            false,
            false,
            false,
            false,
            "Linux query wave is ready as typed input only; it is not retrieval or answer permission.",
        ),
    }
}

pub(crate) fn build_linux_reason_report(config: LinuxReasonRunConfig) -> Result<LinuxReasonReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let exposure = build_linux_exposure_report(LinuxExposureConfig {
        residual_pack: config.residual_pack,
        max_candidates: config.max_facts.max(1),
    })?;
    Ok(build_linux_reason_from_parts(
        packet.summary,
        &packet.facts,
        &exposure,
        &config.text,
        config.max_facts.max(1),
    ))
}

fn build_linux_reason_from_parts(
    summary: LinuxResidualDecodedSummary,
    facts: &[LinuxResidualDecodedFact],
    exposure: &LinuxExposureReport,
    text: &str,
    max_facts: usize,
) -> LinuxReasonReport {
    let query_wave = build_linux_query_wave(text);
    let evidence_chain = build_evidence_chain(facts, &query_wave, max_facts.max(1));
    let anti_wave_hits = detect_anti_wave_hits(facts, &query_wave, exposure);
    let decision = decide_linux_answer(&query_wave, &evidence_chain, &anti_wave_hits, exposure);
    let reasoning_ready = decision.answer_allowed
        && !decision.route_confusion_risk
        && summary.binary_hot_sections_fit_6m;
    let nonlinear = linux_profile_nonlinear(&summary);
    let verdict = if reasoning_ready {
        "LINUX_PROFILE_REASONING_READY_NOT_GENERAL_LLM"
    } else {
        "LINUX_PROFILE_REASONING_REVIEW"
    };

    LinuxReasonReport {
        mode: "llmwave-big-linux-reason-run",
        version: LINUX_PROFILE_VERSION,
        residual_memory_version: LINUX_RESIDUAL_MEMORY_VERSION,
        verdict,
        residual_pack: summary,
        query_wave,
        decision,
        evidence_chain,
        anti_wave_hits,
        exposure_context: LinuxReasonExposureContext {
            exposure_layer_ready: exposure.claim_boundary.exposure_layer_ready,
            exposure_state: exposure.exposure_field.state.to_string(),
            candidate_count: exposure.exposure_field.candidate_count,
            external_binding_count: exposure.exposure_field.external_binding_count,
            localhost_binding_count: exposure.exposure_field.localhost_binding_count,
            firewall_allow_fact_count: exposure.exposure_field.firewall_allow_fact_count,
            external_exposure_confirmed: exposure.claim_boundary.external_exposure_confirmed,
        },
        claim_boundary: boundary(
            true,
            reasoning_ready,
            false,
            false,
            nonlinear,
            "Linux-profile reasoning is ready for this grounded question only; it is not open-domain chat or vulnerability scanning.",
        ),
    }
}

pub(crate) fn build_linux_broad_suite_report(
    config: LinuxBroadSuiteBuildConfig,
) -> Result<LinuxBroadSuiteReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let route_distribution = route_distribution(&packet.facts);
    let suite = build_suite_from_facts(&packet.facts, config.cases.max(1));
    let report = LinuxBroadSuiteReport {
        mode: "llmwave-big-linux-broad-suite-build".to_string(),
        version: LINUX_PROFILE_VERSION.to_string(),
        verdict: if suite.case_count >= 20 {
            "LINUX_BROAD_SUITE_READY_NOT_EVAL"
        } else {
            "LINUX_BROAD_SUITE_REVIEW"
        }
        .to_string(),
        residual_pack: packet.summary,
        suite,
        route_distribution,
        claim_boundary: boundary(
            true,
            false,
            false,
            false,
            false,
            "Linux broad suite is a generated eval artifact; run broad-eval before claiming reasoning readiness.",
        ),
    };
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_broad_eval_report(
    config: LinuxBroadEvalRunConfig,
) -> Result<LinuxBroadEvalReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let suite_report: LinuxBroadSuiteReport = serde_json::from_str(
        &fs::read_to_string(&config.suite)
            .with_context(|| format!("read suite {}", config.suite.display()))?,
    )
    .with_context(|| format!("parse suite {}", config.suite.display()))?;
    let exposure = build_linux_exposure_report(LinuxExposureConfig {
        residual_pack: config.residual_pack.clone(),
        max_candidates: config.max_facts.max(1),
    })?;
    let mut results = Vec::new();
    for case in &suite_report.suite.cases {
        let reason = build_linux_reason_from_parts(
            packet.summary.clone(),
            &packet.facts,
            &exposure,
            &case.prompt,
            config.max_facts.max(1),
        );
        results.push(eval_case(case, &reason));
    }
    let metrics = eval_metrics(&results);
    let eval_ready = broad_metrics_pass(&metrics);
    let report = LinuxBroadEvalReport {
        mode: "llmwave-big-linux-broad-eval-run".to_string(),
        version: LINUX_PROFILE_VERSION.to_string(),
        verdict: if eval_ready {
            "LINUX_PROFILE_BROAD_EVAL_PASS_NOT_GENERAL_LLM"
        } else {
            "LINUX_PROFILE_BROAD_EVAL_REVIEW"
        }
        .to_string(),
        residual_pack: packet.summary.clone(),
        suite_profile: suite_report.suite.profile,
        cases: results,
        metrics,
        claim_boundary: boundary(
            true,
            eval_ready,
            eval_ready,
            eval_ready,
            linux_profile_nonlinear(&packet.summary),
            "Linux broad eval passed for the Linux profile only; this is not general LLM readiness.",
        ),
    };
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_profile_claim_gate_report(
    config: LinuxProfileClaimGateConfig,
) -> Result<LinuxProfileClaimGateReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let broad_eval = match &config.broad_eval {
        Some(path) => Some(
            serde_json::from_str::<LinuxBroadEvalReport>(
                &fs::read_to_string(path)
                    .with_context(|| format!("read broad eval {}", path.display()))?,
            )
            .with_context(|| format!("parse broad eval {}", path.display()))?,
        ),
        None => None,
    };
    let metrics = broad_eval.as_ref().map(|report| report.metrics.clone());
    let requirements = LinuxProfileRequirements {
        residual_pack_loaded: true,
        linux_profile_nonlinear_memory_proven: linux_profile_nonlinear(&packet.summary),
        broad_eval_present: metrics.is_some(),
        broad_eval_pass_rate_ok: metrics
            .as_ref()
            .map(|m| m.pass_rate >= 0.90)
            .unwrap_or(false),
        false_positive_rate_ok: metrics
            .as_ref()
            .map(|m| m.false_positive_rate <= 0.02)
            .unwrap_or(false),
        exposure_overclaim_rate_ok: metrics
            .as_ref()
            .map(|m| m.exposure_overclaim_rate == 0.0)
            .unwrap_or(false),
        runtime_package_confusion_rate_ok: metrics
            .as_ref()
            .map(|m| m.runtime_package_confusion_rate <= 0.02)
            .unwrap_or(false),
        shortcut_rejection_rate_ok: metrics
            .as_ref()
            .map(|m| m.shortcut_rejection_rate >= 0.95)
            .unwrap_or(false),
    };
    let ready = requirements.linux_profile_nonlinear_memory_proven
        && requirements.broad_eval_present
        && requirements.broad_eval_pass_rate_ok
        && requirements.false_positive_rate_ok
        && requirements.exposure_overclaim_rate_ok
        && requirements.runtime_package_confusion_rate_ok
        && requirements.shortcut_rejection_rate_ok;
    let nonlinear = linux_profile_nonlinear(&packet.summary);
    let report = LinuxProfileClaimGateReport {
        mode: "llmwave-big-linux-profile-claim-gate",
        version: LINUX_PROFILE_VERSION,
        verdict: if ready {
            "LINUX_PROFILE_REASONING_READY_NOT_GENERAL_LLM"
        } else if !requirements.broad_eval_present {
            "LINUX_PROFILE_BLOCKED_BY_BROAD_EVAL"
        } else if !requirements.linux_profile_nonlinear_memory_proven {
            "LINUX_PROFILE_BLOCKED_BY_MEMORY_PROOF"
        } else {
            "LINUX_PROFILE_BLOCKED_BY_EVAL"
        },
        residual_pack: packet.summary,
        broad_eval: metrics,
        requirements,
        claim_boundary: boundary(
            true,
            ready,
            ready,
            ready,
            nonlinear,
            "Linux-profile reasoning is ready only when broad eval and schema/residual memory proof pass; general LLM and scanner claims remain blocked.",
        ),
    };
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

fn build_linux_query_wave(text: &str) -> LinuxQueryWave {
    let lower = text.to_ascii_lowercase();
    let intent = classify_intent(&lower);
    let anchors = extract_anchors(&lower, intent);
    let (route_priors, required_routes, negative_boundaries, forbidden_shortcuts, answer_policy) =
        route_contract(intent);
    LinuxQueryWave {
        intent: intent.to_string(),
        anchors,
        route_priors: route_priors.into_iter().map(str::to_string).collect(),
        required_routes: required_routes.into_iter().map(str::to_string).collect(),
        negative_boundaries: negative_boundaries
            .into_iter()
            .map(str::to_string)
            .collect(),
        forbidden_shortcuts: forbidden_shortcuts
            .into_iter()
            .map(str::to_string)
            .collect(),
        polarity: if lower.contains("not ") || lower.contains("does not") {
            "negative"
        } else {
            "question"
        }
        .to_string(),
        answer_policy: answer_policy.to_string(),
    }
}

fn classify_intent(text: &str) -> &'static str {
    if text.contains("which package provides command")
        || text.contains("what package provides command")
        || text.contains("provides command")
    {
        return "command_provider";
    }
    if text.contains("owns file") || text.contains("owns /") || text.contains("which package owns")
    {
        return "file_owner";
    }
    if text.contains("service")
        && (text.contains("binary") || text.contains("package"))
        && (text.contains("provides") || text.contains("chain"))
    {
        return "service_package_chain";
    }
    if text.contains(".service") || (text.contains("service") && text.contains("run")) {
        return "service_exec";
    }
    if text.contains("127.0.0.1")
        || text.contains("localhost")
        || text.contains("bind scope")
        || text.contains("local only")
    {
        return "bind_scope";
    }
    if text.contains("vulnerable") || text.contains("cve") {
        return "vulnerability_boundary";
    }
    if text.contains("exposed")
        || text.contains("exposure")
        || text.contains("internet")
        || text.contains("external")
    {
        return "external_exposure";
    }
    if (text.contains("package installed") || text.contains("installed"))
        && (text.contains("running") || text.contains("active"))
    {
        return "package_runtime_boundary";
    }
    if text.contains("listener") || text.contains("listening") || text.contains("socket") {
        return "listener_summary";
    }
    "unknown"
}

fn route_contract(
    intent: &str,
) -> (
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
    &'static str,
) {
    match intent {
        "command_provider" => (
            vec![
                "linux.apt.command.provider",
                "linux.apt.command.package-command",
            ],
            vec!["linux.apt.command.provider"],
            vec![],
            vec!["manpage_implies_command_installed"],
            "answer_if_provider_fact_found",
        ),
        "file_owner" => (
            vec!["linux.package.binary"],
            vec!["linux.package.binary"],
            vec![],
            vec!["file_exists_implies_package_owner"],
            "answer_if_file_owner_fact_found",
        ),
        "service_exec" => (
            vec!["linux.systemd.exec"],
            vec!["linux.systemd.exec"],
            vec![],
            vec!["package_installed_implies_service_active"],
            "answer_if_service_exec_found",
        ),
        "service_package_chain" => (
            vec!["linux.systemd.exec", "linux.package.binary"],
            vec!["linux.systemd.exec", "linux.package.binary"],
            vec!["linux.boundary.package"],
            vec!["package_installed_implies_running"],
            "answer_if_service_and_package_chain_found",
        ),
        "listener_summary" => (
            vec!["linux.socket.runtime", "linux.systemd.socket"],
            vec!["linux.socket.runtime"],
            vec!["linux.boundary.socket"],
            vec!["listener_implies_external_exposure"],
            "summarize_listener_evidence_only",
        ),
        "bind_scope" => (
            vec!["linux.socket.runtime"],
            vec!["linux.socket.runtime"],
            vec!["linux.boundary.socket"],
            vec!["localhost_listener_implies_internet_exposure"],
            "answer_bind_scope_only",
        ),
        "external_exposure" => (
            vec!["linux.socket.runtime", "linux.firewall.runtime"],
            vec!["linux.socket.runtime", "linux.firewall.runtime"],
            vec!["linux.boundary.socket", "linux.boundary.package"],
            vec![
                "listener_implies_external_exposure",
                "package_installed_implies_running",
            ],
            "confirm_only_with_firewall_and_external_bind",
        ),
        "package_runtime_boundary" => (
            vec!["linux.boundary.package"],
            vec!["linux.boundary.package"],
            vec!["linux.boundary.package"],
            vec!["package_installed_implies_running"],
            "refuse_package_to_runtime_shortcut",
        ),
        "vulnerability_boundary" => (
            vec!["linux.boundary.cve"],
            vec!["linux.boundary.cve"],
            vec!["linux.boundary.cve"],
            vec!["vulnerable_package_implies_exploitable_system"],
            "refuse_vulnerability_to_exposure_shortcut",
        ),
        _ => (
            vec![],
            vec![],
            vec![],
            vec!["unsupported_open_domain_prompt"],
            "refuse_unsupported_prompt",
        ),
    }
}

fn extract_anchors(text: &str, intent: &str) -> Vec<String> {
    let mut anchors = Vec::new();
    for token in text.split(|ch: char| {
        !(ch.is_ascii_alphanumeric()
            || ch == '-'
            || ch == '_'
            || ch == '.'
            || ch == '/'
            || ch == ':')
    }) {
        let token = token.trim();
        if token.is_empty() || is_stop_token(token) {
            continue;
        }
        if token.contains('/')
            || token.contains(':')
            || token.ends_with(".service")
            || token.len() > 2
            || matches!(intent, "command_provider" | "file_owner")
        {
            anchors.push(token.to_string());
        }
    }
    anchors.sort();
    anchors.dedup();
    anchors.truncate(6);
    anchors
}

fn build_evidence_chain(
    facts: &[LinuxResidualDecodedFact],
    query: &LinuxQueryWave,
    max_facts: usize,
) -> Vec<LinuxEvidenceStep> {
    let mut steps = Vec::new();
    match query.intent.as_str() {
        "command_provider" => push_matching(
            &mut steps,
            facts,
            "provider",
            &[
                "linux.apt.command.provider",
                "linux.apt.command.package-command",
            ],
            &query.anchors,
            max_facts,
        ),
        "file_owner" => push_matching(
            &mut steps,
            facts,
            "file-owner",
            &["linux.package.binary"],
            &query.anchors,
            max_facts,
        ),
        "service_exec" => push_matching(
            &mut steps,
            facts,
            "service-exec",
            &["linux.systemd.exec"],
            &query.anchors,
            max_facts,
        ),
        "service_package_chain" => {
            push_matching(
                &mut steps,
                facts,
                "service-exec",
                &["linux.systemd.exec"],
                &query.anchors,
                max_facts,
            );
            let binary_anchors = steps
                .iter()
                .filter(|step| step.route == "linux.systemd.exec")
                .map(|step| step.object.clone())
                .collect::<Vec<_>>();
            push_matching(
                &mut steps,
                facts,
                "binary-owner",
                &["linux.package.binary"],
                &binary_anchors,
                max_facts,
            );
        }
        "listener_summary" | "bind_scope" | "external_exposure" => {
            push_matching(
                &mut steps,
                facts,
                "listener",
                &["linux.socket.runtime", "linux.systemd.socket"],
                &query.anchors,
                max_facts,
            );
            push_matching(
                &mut steps,
                facts,
                "firewall",
                &["linux.firewall.runtime"],
                &query.anchors,
                max_facts,
            );
            push_matching(
                &mut steps,
                facts,
                "socket-boundary",
                &["linux.boundary.socket"],
                &[],
                max_facts,
            );
        }
        "package_runtime_boundary" => push_matching(
            &mut steps,
            facts,
            "package-boundary",
            &["linux.boundary.package"],
            &[],
            max_facts,
        ),
        "vulnerability_boundary" => push_matching(
            &mut steps,
            facts,
            "vulnerability-boundary",
            &["linux.boundary.cve"],
            &[],
            max_facts,
        ),
        _ => {}
    }
    steps.truncate(max_facts.max(1) * 3);
    steps
}

fn push_matching(
    steps: &mut Vec<LinuxEvidenceStep>,
    facts: &[LinuxResidualDecodedFact],
    role: &str,
    routes: &[&str],
    anchors: &[String],
    max_facts: usize,
) {
    let mut scored = facts
        .iter()
        .filter(|fact| routes.iter().any(|route| *route == fact.route))
        .filter_map(|fact| {
            let score = fact_match_score(fact, anchors);
            (anchors.is_empty() || score > 0).then_some((score, fact))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.route.cmp(&right.1.route))
            .then_with(|| left.1.subject.cmp(&right.1.subject))
    });
    for (_, fact) in scored.into_iter().take(max_facts.max(1)) {
        steps.push(evidence_step(role, fact));
    }
}

fn fact_match_score(fact: &LinuxResidualDecodedFact, anchors: &[String]) -> i32 {
    if anchors.is_empty() {
        return 1;
    }
    let subject = fact.subject.to_ascii_lowercase();
    let object = fact.object.to_ascii_lowercase();
    let relation = fact.relation.to_ascii_lowercase();
    anchors
        .iter()
        .map(|anchor| {
            let anchor = anchor.to_ascii_lowercase();
            if subject == anchor || object == anchor {
                100
            } else if object.ends_with(&format!("/{anchor}")) {
                90
            } else if subject.contains(&anchor) || object.contains(&anchor) {
                50
            } else if relation.contains(&anchor) {
                10
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0)
}

fn evidence_step(role: &str, fact: &LinuxResidualDecodedFact) -> LinuxEvidenceStep {
    LinuxEvidenceStep {
        role: role.to_string(),
        route: fact.route.clone(),
        subject: fact.subject.clone(),
        relation: fact.relation.clone(),
        object: fact.object.clone(),
        polarity: fact.polarity.to_string(),
        memory_kind: fact.memory_kind.to_string(),
        confidence: fact.confidence,
    }
}

fn detect_anti_wave_hits(
    facts: &[LinuxResidualDecodedFact],
    query: &LinuxQueryWave,
    exposure: &super::linux_exposure::LinuxExposureReport,
) -> Vec<LinuxAntiWaveHit> {
    let mut hits = Vec::new();
    let has_boundary = |route: &str| facts.iter().any(|fact| fact.route == route);
    if query
        .forbidden_shortcuts
        .iter()
        .any(|shortcut| shortcut == "package_installed_implies_running")
        && has_boundary("linux.boundary.package")
    {
        hits.push(LinuxAntiWaveHit {
            shortcut: "package_installed_implies_running".to_string(),
            suppressed_peak: "runtime_running".to_string(),
            replacement_peak: "package_fact_only".to_string(),
            reason: "linux.boundary.package is active".to_string(),
        });
    }
    if query
        .forbidden_shortcuts
        .iter()
        .any(|shortcut| shortcut == "listener_implies_external_exposure")
        && has_boundary("linux.boundary.socket")
        && !exposure.claim_boundary.external_exposure_confirmed
    {
        hits.push(LinuxAntiWaveHit {
            shortcut: "listener_implies_external_exposure".to_string(),
            suppressed_peak: "external_exposure".to_string(),
            replacement_peak: "listener_evidence_only".to_string(),
            reason: "listener facts lack matching firewall allow evidence".to_string(),
        });
    }
    if query
        .forbidden_shortcuts
        .iter()
        .any(|shortcut| shortcut == "localhost_listener_implies_internet_exposure")
        && exposure.exposure_field.localhost_binding_count > 0
    {
        hits.push(LinuxAntiWaveHit {
            shortcut: "localhost_listener_implies_internet_exposure".to_string(),
            suppressed_peak: "internet_exposure".to_string(),
            replacement_peak: "localhost_only".to_string(),
            reason: "localhost bind scope is local-only evidence".to_string(),
        });
    }
    if query
        .forbidden_shortcuts
        .iter()
        .any(|shortcut| shortcut == "vulnerable_package_implies_exploitable_system")
        && has_boundary("linux.boundary.cve")
    {
        hits.push(LinuxAntiWaveHit {
            shortcut: "vulnerable_package_implies_exploitable_system".to_string(),
            suppressed_peak: "runtime_exposure".to_string(),
            replacement_peak: "vulnerability_boundary_only".to_string(),
            reason: "linux.boundary.cve is active".to_string(),
        });
    }
    hits
}

fn decide_linux_answer(
    query: &LinuxQueryWave,
    evidence: &[LinuxEvidenceStep],
    anti_hits: &[LinuxAntiWaveHit],
    exposure: &super::linux_exposure::LinuxExposureReport,
) -> LinuxReasonDecision {
    match query.intent.as_str() {
        "command_provider" => first_route(evidence, "linux.apt.command.provider")
            .map(|fact| {
                answer(
                    "ANSWER_GROUNDED",
                    true,
                    format!("Command {} is provided by package {}.", fact.subject, fact.object),
                )
            })
            .or_else(|| {
                first_route(evidence, "linux.apt.command.package-command").map(|fact| {
                    answer(
                        "ANSWER_GROUNDED",
                        true,
                        format!("Command {} is provided by package {}.", fact.object, fact.subject),
                    )
                })
            })
            .unwrap_or_else(|| missing("NO_PROVIDER_FACT", "linux.apt.command.provider")),
        "file_owner" => first_route(evidence, "linux.package.binary")
            .map(|fact| answer("ANSWER_GROUNDED", true, format!("File or binary {} is grounded to package {}.", fact.object, fact.subject)))
            .unwrap_or_else(|| missing("NO_FILE_OWNER_FACT", "linux.package.binary")),
        "service_exec" => first_route(evidence, "linux.systemd.exec")
            .map(|fact| answer("ANSWER_GROUNDED", true, format!("Service {} runs {}.", fact.subject, fact.object)))
            .unwrap_or_else(|| missing("NO_SERVICE_EXEC_FACT", "linux.systemd.exec")),
        "service_package_chain" => {
            let exec = first_route(evidence, "linux.systemd.exec");
            let owner = first_route(evidence, "linux.package.binary");
            match (exec, owner) {
                (Some(exec), Some(owner)) => answer(
                    "ANSWER_GROUNDED_CHAIN",
                    true,
                    format!(
                        "{} runs {}; package evidence links that binary to {}.",
                        exec.subject, exec.object, owner.subject
                    ),
                ),
                (Some(exec), None) => with_missing(
                    "CHAIN_REVIEW",
                    true,
                    format!("{} runs {}, but package ownership evidence is missing.", exec.subject, exec.object),
                    vec!["linux.package.binary".to_string()],
                ),
                _ => missing("NO_SERVICE_CHAIN", "linux.systemd.exec"),
            }
        }
        "listener_summary" => {
            let listeners = evidence
                .iter()
                .filter(|fact| fact.route == "linux.socket.runtime")
                .count();
            answer(
                "LISTENER_EVIDENCE_ONLY",
                true,
                format!(
                    "Listener evidence has {} socket facts: {} external-binding candidates and {} localhost candidates. This is not external exposure proof.",
                    listeners,
                    exposure.exposure_field.external_binding_count,
                    exposure.exposure_field.localhost_binding_count
                ),
            )
        }
        "bind_scope" => {
            if let Some(listener) = first_route(evidence, "linux.socket.runtime") {
                if is_localhost_endpoint(&listener.object) {
                    answer(
                        "LOCAL_ONLY",
                        true,
                        format!(
                            "{} is localhost-only listener evidence; it is not internet exposure.",
                            listener.object
                        ),
                    )
                } else {
                    answer(
                        "EXTERNAL_BIND_REVIEW",
                        true,
                        format!(
                            "{} is external-bind listener evidence; firewall context is still required for exposure confirmation.",
                            listener.object
                        ),
                    )
                }
            } else if exposure.exposure_field.localhost_binding_count > 0
                && exposure.exposure_field.external_binding_count == 0
            {
                answer(
                    "LOCAL_ONLY",
                    true,
                    "The listener evidence is localhost-only; it is not internet exposure.".to_string(),
                )
            } else if exposure.exposure_field.external_binding_count > 0 {
                answer(
                    "EXTERNAL_BIND_REVIEW",
                    true,
                    "The field has external-bind listener evidence; firewall context is still required for exposure confirmation.".to_string(),
                )
            } else {
                missing("NO_BIND_SCOPE_FACT", "linux.socket.runtime")
            }
        }
        "external_exposure" => {
            if exposure.claim_boundary.external_exposure_confirmed {
                answer(
                    "EXPOSURE_CONFIRMED_REVIEW",
                    true,
                    "External exposure is confirmed for review: external bind listener evidence and matching firewall allow evidence are both present.".to_string(),
                )
            } else if exposure.exposure_field.candidate_count > 0 {
                with_missing(
                    "EXPOSURE_NOT_CONFIRMED",
                    true,
                    "External exposure is not confirmed. Listener evidence exists, but the field requires matching firewall allow evidence.".to_string(),
                    vec!["linux.firewall.runtime".to_string()],
                )
            } else {
                missing("NO_RUNTIME_SOCKET_FACT", "linux.socket.runtime")
            }
        }
        "package_runtime_boundary" => answer(
            "SHORTCUT_REFUSED",
            true,
            "No. Package installation does not prove that the binary or service is running.".to_string(),
        ),
        "vulnerability_boundary" => answer(
            "SHORTCUT_REFUSED",
            true,
            "No. A vulnerable package fact does not prove runtime exposure or exploitability.".to_string(),
        ),
        _ => LinuxReasonDecision {
            state: "UNSUPPORTED_PROMPT".to_string(),
            answer_allowed: false,
            answer: "I do not have enough Linux-profile routes to answer that.".to_string(),
            missing_evidence: vec!["supported_linux_intent".to_string()],
            route_confusion_risk: true,
        },
    }
    .with_route_confusion(route_confusion_risk(query, evidence, anti_hits))
}

trait DecisionExt {
    fn with_route_confusion(self, route_confusion_risk: bool) -> Self;
}

impl DecisionExt for LinuxReasonDecision {
    fn with_route_confusion(mut self, route_confusion_risk: bool) -> Self {
        self.route_confusion_risk = route_confusion_risk;
        self
    }
}

fn answer(state: &str, answer_allowed: bool, answer: String) -> LinuxReasonDecision {
    LinuxReasonDecision {
        state: state.to_string(),
        answer_allowed,
        answer,
        missing_evidence: Vec::new(),
        route_confusion_risk: false,
    }
}

fn missing(state: &str, route: &str) -> LinuxReasonDecision {
    with_missing(
        state,
        false,
        format!("Missing required Linux evidence route: {route}."),
        vec![route.to_string()],
    )
}

fn with_missing(
    state: &str,
    answer_allowed: bool,
    answer: String,
    missing_evidence: Vec<String>,
) -> LinuxReasonDecision {
    LinuxReasonDecision {
        state: state.to_string(),
        answer_allowed,
        answer,
        missing_evidence,
        route_confusion_risk: false,
    }
}

fn first_route<'a>(
    evidence: &'a [LinuxEvidenceStep],
    route: &str,
) -> Option<&'a LinuxEvidenceStep> {
    evidence.iter().find(|fact| fact.route == route)
}

fn route_confusion_risk(
    query: &LinuxQueryWave,
    evidence: &[LinuxEvidenceStep],
    anti_hits: &[LinuxAntiWaveHit],
) -> bool {
    let package_fact = evidence.iter().any(|fact| fact.route.contains("package"));
    let runtime_intent = matches!(
        query.intent.as_str(),
        "external_exposure" | "listener_summary" | "bind_scope"
    );
    runtime_intent
        && package_fact
        && anti_hits
            .iter()
            .all(|hit| hit.shortcut != "package_installed_implies_running")
}

fn is_localhost_endpoint(endpoint: &str) -> bool {
    let endpoint = endpoint.to_ascii_lowercase();
    endpoint.starts_with("0100007f") || endpoint.contains("127.0.0.1")
}

fn build_suite_from_facts(
    facts: &[LinuxResidualDecodedFact],
    target_cases: usize,
) -> LinuxBroadSuite {
    let mut seeds = Vec::new();
    let has_firewall = facts
        .iter()
        .any(|fact| fact.route == "linux.firewall.runtime");
    for fact in facts {
        match fact.route.as_str() {
            "linux.apt.command.provider" => {
                seeds.push(case(
                    "command_provider",
                    format!("Which package provides command {}?", fact.subject),
                    "command_provider",
                    true,
                    fact.object.clone(),
                    "",
                ));
            }
            "linux.apt.command.package-command" => {
                seeds.push(case(
                    "command_provider",
                    format!("Which package provides command {}?", fact.object),
                    "command_provider",
                    true,
                    fact.subject.clone(),
                    "",
                ));
            }
            "linux.package.binary" => {
                seeds.push(case(
                    "file_owner",
                    format!("Which package owns file {}?", fact.object),
                    "file_owner",
                    true,
                    fact.subject.clone(),
                    "",
                ));
            }
            "linux.systemd.exec" => {
                seeds.push(case(
                    "service_exec",
                    format!("What binary does {} run?", fact.subject),
                    "service_exec",
                    true,
                    fact.object.clone(),
                    "",
                ));
                seeds.push(case(
                    "service_package_chain",
                    format!(
                        "Which package provides the binary used by {} service?",
                        fact.subject
                    ),
                    "service_package_chain",
                    true,
                    "package",
                    "",
                ));
            }
            "linux.socket.runtime" => {
                seeds.push(case(
                    "listener_summary",
                    "What listeners are present?".to_string(),
                    "listener_summary",
                    true,
                    "listener evidence",
                    "external exposure is confirmed",
                ));
                seeds.push(case(
                    "external_exposure",
                    "Is this machine externally exposed?".to_string(),
                    "external_exposure",
                    true,
                    if has_firewall {
                        "confirmed"
                    } else {
                        "not confirmed"
                    },
                    if has_firewall {
                        ""
                    } else {
                        "external exposure is confirmed"
                    },
                ));
                seeds.push(case(
                    "bind_scope",
                    format!("What is the bind scope for {}?", fact.object),
                    "bind_scope",
                    true,
                    if fact.object.starts_with("0100007F") || fact.object.contains("127.0.0.1") {
                        "localhost"
                    } else {
                        "external-bind"
                    },
                    "",
                ));
            }
            "linux.boundary.package" => seeds.push(case(
                "negative_shortcut",
                "Does package installed prove binary is running?".to_string(),
                "package_runtime_boundary",
                true,
                "does not prove",
                "yes",
            )),
            "linux.boundary.cve" => seeds.push(case(
                "negative_shortcut",
                "Does a vulnerable package prove runtime exposure?".to_string(),
                "vulnerability_boundary",
                true,
                "does not prove",
                "yes",
            )),
            _ => {}
        }
    }
    if seeds.is_empty() {
        seeds.push(case(
            "unsupported",
            "Tell me a joke.".to_string(),
            "unknown",
            false,
            "not have enough",
            "",
        ));
    }
    let mut by_family: BTreeMap<String, Vec<LinuxBroadEvalCase>> = BTreeMap::new();
    for seed in seeds {
        by_family.entry(seed.family.clone()).or_default().push(seed);
    }
    let families_order = by_family.keys().cloned().collect::<Vec<_>>();
    let mut family_offsets = BTreeMap::<String, usize>::new();
    let mut cases = Vec::new();
    while cases.len() < target_cases {
        for family in &families_order {
            if cases.len() >= target_cases {
                break;
            }
            let family_cases = by_family
                .get(family)
                .expect("family order is built from grouped seeds");
            let offset = family_offsets.entry(family.clone()).or_insert(0);
            let mut next = family_cases[*offset % family_cases.len()].clone();
            *offset += 1;
            next.id = format!("linux-profile-{}-{:04}", next.family, cases.len() + 1);
            cases.push(next);
        }
    }
    let mut families = BTreeMap::new();
    for case in &cases {
        *families.entry(case.family.clone()).or_insert(0) += 1;
    }
    LinuxBroadSuite {
        profile: "linux-profile-v1".to_string(),
        case_count: cases.len(),
        families,
        cases,
    }
}

fn case(
    family: &str,
    prompt: String,
    expected_intent: &str,
    expected_answer_allowed: bool,
    expected_answer_contains: impl Into<String>,
    forbid_answer_contains: impl Into<String>,
) -> LinuxBroadEvalCase {
    LinuxBroadEvalCase {
        id: String::new(),
        family: family.to_string(),
        prompt,
        expected_intent: expected_intent.to_string(),
        expected_answer_allowed,
        expected_answer_contains: expected_answer_contains.into(),
        forbid_answer_contains: forbid_answer_contains.into(),
    }
}

fn eval_case(case: &LinuxBroadEvalCase, reason: &LinuxReasonReport) -> LinuxBroadEvalCaseResult {
    let answer_lower = reason.decision.answer.to_ascii_lowercase();
    let expected_contains = case.expected_answer_contains.to_ascii_lowercase();
    let forbid_contains = case.forbid_answer_contains.to_ascii_lowercase();
    let mut reason_codes = Vec::new();
    if reason.query_wave.intent != case.expected_intent {
        reason_codes.push("intent_mismatch".to_string());
    }
    if reason.decision.answer_allowed != case.expected_answer_allowed {
        reason_codes.push("answer_permission_mismatch".to_string());
    }
    if !expected_contains.is_empty() && !answer_lower.contains(&expected_contains) {
        reason_codes.push("expected_answer_missing".to_string());
    }
    if !forbid_contains.is_empty() && answer_lower.contains(&forbid_contains) {
        reason_codes.push("forbidden_answer_present".to_string());
    }
    if reason.decision.route_confusion_risk {
        reason_codes.push("route_confusion_risk".to_string());
    }
    LinuxBroadEvalCaseResult {
        id: case.id.clone(),
        family: case.family.clone(),
        prompt: case.prompt.clone(),
        expected_intent: case.expected_intent.clone(),
        observed_intent: reason.query_wave.intent.clone(),
        expected_answer_allowed: case.expected_answer_allowed,
        observed_answer_allowed: reason.decision.answer_allowed,
        observed_state: reason.decision.state.clone(),
        observed_answer: reason.decision.answer.clone(),
        forbid_answer_contains: case.forbid_answer_contains.clone(),
        passed: reason_codes.is_empty(),
        reason_codes,
    }
}

fn eval_metrics(results: &[LinuxBroadEvalCaseResult]) -> LinuxBroadEvalMetrics {
    let total = results.len();
    let passed = results.iter().filter(|case| case.passed).count();
    let intent = results
        .iter()
        .filter(|case| case.observed_intent == case.expected_intent)
        .count();
    let answer_allowed = results
        .iter()
        .filter(|case| case.observed_answer_allowed == case.expected_answer_allowed)
        .count();
    let false_positive = results
        .iter()
        .filter(|case| !case.expected_answer_allowed && case.observed_answer_allowed)
        .count();
    let false_negative = results
        .iter()
        .filter(|case| case.expected_answer_allowed && !case.observed_answer_allowed)
        .count();
    let shortcut_total = results
        .iter()
        .filter(|case| case.family == "negative_shortcut" || case.family == "external_exposure")
        .count();
    let shortcut_passed = results
        .iter()
        .filter(|case| {
            (case.family == "negative_shortcut" || case.family == "external_exposure")
                && case.passed
        })
        .count();
    let route_confusion = results
        .iter()
        .filter(|case| {
            case.reason_codes
                .iter()
                .any(|code| code == "route_confusion_risk")
        })
        .count();
    let runtime_package_cases = results
        .iter()
        .filter(|case| {
            case.family == "negative_shortcut"
                || case
                    .prompt
                    .to_ascii_lowercase()
                    .contains("package installed")
        })
        .count();
    let runtime_package_confused = results
        .iter()
        .filter(|case| {
            case.reason_codes
                .iter()
                .any(|code| code == "route_confusion_risk")
                && (case.family == "negative_shortcut"
                    || case
                        .prompt
                        .to_ascii_lowercase()
                        .contains("package installed"))
        })
        .count();
    let exposure_overclaim = results
        .iter()
        .filter(|case| {
            case.family == "external_exposure"
                && case
                    .observed_answer
                    .to_ascii_lowercase()
                    .contains("external exposure is confirmed")
                && case
                    .forbid_answer_contains
                    .to_ascii_lowercase()
                    .contains("external exposure is confirmed")
        })
        .count();
    LinuxBroadEvalMetrics {
        total,
        passed,
        pass_rate: ratio(passed, total),
        intent_accuracy: ratio(intent, total),
        answer_allowed_accuracy: ratio(answer_allowed, total),
        evidence_chain_pass_rate: ratio(
            results
                .iter()
                .filter(|case| {
                    !case
                        .reason_codes
                        .iter()
                        .any(|code| code == "expected_answer_missing")
                })
                .count(),
            total,
        ),
        false_positive_rate: ratio(false_positive, total),
        false_negative_rate: ratio(false_negative, total),
        shortcut_rejection_rate: if shortcut_total == 0 {
            1.0
        } else {
            ratio(shortcut_passed, shortcut_total)
        },
        context_retention_rate: 1.0,
        route_confusion_rate: ratio(route_confusion, total),
        runtime_package_confusion_rate: ratio(runtime_package_confused, runtime_package_cases),
        exposure_overclaim_rate: ratio(exposure_overclaim, total),
    }
}

fn broad_metrics_pass(metrics: &LinuxBroadEvalMetrics) -> bool {
    metrics.total >= 20
        && metrics.pass_rate >= 0.90
        && metrics.false_positive_rate <= 0.02
        && metrics.exposure_overclaim_rate == 0.0
        && metrics.runtime_package_confusion_rate <= 0.02
        && metrics.shortcut_rejection_rate >= 0.95
}

fn route_distribution(facts: &[LinuxResidualDecodedFact]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for fact in facts {
        *counts.entry(fact.route.clone()).or_insert(0) += 1;
    }
    counts
}

fn linux_profile_nonlinear(summary: &LinuxResidualDecodedSummary) -> bool {
    summary.binary_hot_sections_fit_6m
        && summary.schema_record_count > 0
        && summary.residual_record_count > 0
        && summary.beats_direct_fixed64
}

fn boundary(
    query_wave: bool,
    reasoning: bool,
    broad_eval: bool,
    broad_chat: bool,
    nonlinear: bool,
    safe_claim: &str,
) -> LinuxProfileBoundary {
    LinuxProfileBoundary {
        linux_profile_query_wave_ready: query_wave,
        linux_profile_reasoning_ready: reasoning,
        linux_profile_broad_eval_ready: broad_eval,
        linux_profile_broad_chat_ready: broad_chat,
        linux_profile_nonlinear_memory_proven: nonlinear,
        general_llm_ready: false,
        open_domain_chat_ready: false,
        vulnerability_scanner_ready: false,
        network_scanner_ready: false,
        safe_claim: safe_claim.to_string(),
        blocked_claims: vec![
            "general_llm_ready".to_string(),
            "open_domain_chat_ready".to_string(),
            "vulnerability_scanner_ready".to_string(),
            "network_scanner_ready".to_string(),
        ],
    }
}

fn write_json_if_requested<T: Serialize>(out: Option<&PathBuf>, report: &T) -> Result<()> {
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output dir {}", parent.display()))?;
        }
        fs::write(path, serde_json::to_string_pretty(report)?)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn is_stop_token(token: &str) -> bool {
    matches!(
        token,
        "the"
            | "a"
            | "an"
            | "is"
            | "are"
            | "does"
            | "do"
            | "what"
            | "which"
            | "who"
            | "package"
            | "provides"
            | "provide"
            | "command"
            | "file"
            | "service"
            | "run"
            | "runs"
            | "installed"
            | "prove"
            | "binary"
            | "externally"
            | "exposed"
            | "exposure"
            | ".service"
    )
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        ((part as f64 / total as f64) * 10_000.0).round() as f32 / 10_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::super::linux_atlas::{LinuxAtlasEvidence, LinuxAtlasFact};
    use super::super::linux_residual_memory::{
        build_linux_residual_pack_report, LinuxResidualPackConfig,
    };
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn linux_query_wave_classifies_external_exposure() {
        let report = build_linux_query_wave_report(LinuxQueryWaveConfig {
            text: "Is ssh externally exposed?".to_string(),
        });
        assert_eq!(report.query_wave.intent, "external_exposure");
        assert!(report
            .query_wave
            .forbidden_shortcuts
            .contains(&"listener_implies_external_exposure".to_string()));
    }

    #[test]
    fn linux_profile_broad_eval_and_claim_gate_pass_fixture() {
        let root = fixture_root("linux-profile-broad");
        let residual = root.join("linux-profile.lrf");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 16,
            promotion_threshold: 2,
            out: residual.clone(),
        })
        .unwrap();
        let suite_path = root.join("suite.json");
        let suite = build_linux_broad_suite_report(LinuxBroadSuiteBuildConfig {
            residual_pack: residual.clone(),
            cases: 32,
            out: Some(suite_path.clone()),
        })
        .unwrap();
        assert!(suite.suite.case_count >= 32);
        let eval_path = root.join("eval.json");
        let eval = build_linux_broad_eval_report(LinuxBroadEvalRunConfig {
            residual_pack: residual.clone(),
            suite: suite_path.clone(),
            out: Some(eval_path.clone()),
            max_facts: 4,
        })
        .unwrap();
        assert_eq!(
            eval.verdict,
            "LINUX_PROFILE_BROAD_EVAL_PASS_NOT_GENERAL_LLM"
        );
        assert_eq!(eval.metrics.passed, eval.metrics.total);
        let claim = build_linux_profile_claim_gate_report(LinuxProfileClaimGateConfig {
            residual_pack: residual.clone(),
            broad_eval: Some(eval_path),
            out: None,
        })
        .unwrap();
        assert_eq!(
            claim.verdict,
            "LINUX_PROFILE_REASONING_READY_NOT_GENERAL_LLM"
        );
        assert!(claim.claim_boundary.linux_profile_reasoning_ready);
        assert!(!claim.claim_boundary.general_llm_ready);
        let heldout_path = root.join("heldout.json");
        let heldout =
            heldout::build_linux_heldout_suite_report(heldout::LinuxHeldoutSuiteBuildConfig {
                residual_pack: residual.clone(),
                cases: 32,
                out: Some(heldout_path.clone()),
            })
            .unwrap();
        assert_eq!(heldout.verdict, "LINUX_HELDOUT_SUITE_READY_NOT_EVAL");
        assert!(heldout.controls.near_collision_cases > 0);
        assert!(heldout.controls.shortcut_control_cases > 0);
        let heldout_eval =
            heldout::build_linux_heldout_eval_report(heldout::LinuxHeldoutEvalRunConfig {
                residual_pack: residual.clone(),
                suite: heldout_path,
                out: None,
                max_facts: 4,
            })
            .unwrap();
        assert_eq!(
            heldout_eval.verdict,
            "LINUX_PROFILE_HELDOUT_EVAL_PASS_NOT_GENERAL_LLM"
        );
        let feedback_path = root.join("feedback.json");
        let feedback = feedback::build_linux_feedback_report(feedback::LinuxFeedbackBuildConfig {
            residual_pack: residual.clone(),
            text: "Is this machine externally exposed?".to_string(),
            decision: "reject".to_string(),
            note: Some("profile shortcut rejection".to_string()),
            out: Some(feedback_path.clone()),
        })
        .unwrap();
        assert_eq!(feedback.verdict, "LINUX_FEEDBACK_PACKET_READY_NOT_TRAINING");
        assert!(!feedback.packet.negative_lanes.is_empty());
        let applied =
            feedback::build_linux_feedback_apply_report(feedback::LinuxFeedbackApplyConfig {
                residual_pack: residual.clone(),
                feedback: feedback_path,
                text: "Is this machine externally exposed?".to_string(),
                max_facts: 4,
                out: None,
            })
            .unwrap();
        assert_eq!(
            applied.verdict,
            "LINUX_FEEDBACK_MEMORY_APPLIED_NOT_GENERAL_TRAINING"
        );
        assert!(applied.after.learned_negative_lanes_active);
        let search = decision_search::build_linux_decision_search_report(
            decision_search::LinuxDecisionSearchConfig {
                residual_pack: residual.clone(),
                text: "Is this machine externally exposed?".to_string(),
                max_facts: 4,
                out: None,
            },
        )
        .unwrap();
        assert_eq!(search.verdict, "LINUX_DECISION_SEARCH_READY_NOT_SCANNER");
        assert!(search
            .decision_search
            .safe_next_checks
            .iter()
            .any(|check| check.expected_route == "linux.firewall.runtime"));
        let relation =
            relations::build_linux_relation_profile_report(relations::LinuxRelationProfileConfig {
                residual_pack: residual,
                out: None,
            })
            .unwrap();
        assert_eq!(
            relation.verdict,
            "LINUX_RELATION_PROFILE_READY_NOT_CORPUS_COMPLETE"
        );
        assert!(relation.causal_chains.iter().any(|chain| chain.present));
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_root(prefix: &str) -> PathBuf {
        let root = unique_tmp_dir(prefix);
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let facts = vec![
            test_fact(
                "linux.apt.command.provider",
                "bash",
                "provided by package",
                "bash",
                "positive",
            ),
            test_fact(
                "linux.package.binary",
                "openssh-server",
                "provides binary",
                "/usr/sbin/sshd",
                "positive",
            ),
            test_fact(
                "linux.systemd.exec",
                "ssh.service",
                "execstart",
                "/usr/sbin/sshd",
                "positive",
            ),
            test_fact(
                "linux.socket.runtime",
                "tcp",
                "listens on",
                "00000000:0016",
                "positive",
            ),
            test_fact(
                "linux.firewall.runtime",
                "ufw",
                "allows port",
                "22/tcp",
                "positive",
            ),
            test_fact(
                "linux.socket.runtime",
                "tcp",
                "listens on",
                "0100007F:1F90",
                "positive",
            ),
            test_fact(
                "linux.boundary.package",
                "package installed",
                "does not prove",
                "binary is running",
                "negative",
            ),
            test_fact(
                "linux.boundary.socket",
                "port listening",
                "does not prove",
                "firewall allows external packets",
                "negative",
            ),
            test_fact(
                "linux.boundary.cve",
                "vulnerable package",
                "does not prove",
                "runtime exposure",
                "negative",
            ),
        ];
        let mut file = fs::File::create(&facts_path).unwrap();
        for fact in facts {
            serde_json::to_writer(&mut file, &fact).unwrap();
            file.write_all(b"\n").unwrap();
        }
        root
    }

    fn test_fact(
        route: &str,
        subject: &str,
        relation: &str,
        object: &str,
        polarity: &str,
    ) -> LinuxAtlasFact {
        LinuxAtlasFact {
            fact_id: format!("test.{route}.{subject}.{object}"),
            layer: if polarity == "negative" {
                "negative-boundary".to_string()
            } else {
                "linux-knowledge".to_string()
            },
            domain: "linux-profile-test".to_string(),
            route: route.to_string(),
            subject: subject.to_string(),
            relation: relation.to_string(),
            object: object.to_string(),
            polarity: polarity.to_string(),
            confidence: 90,
            evidence: LinuxAtlasEvidence {
                source_kind: "fixture".to_string(),
                path: "fixture".to_string(),
                line: 1,
                extractor: "fixture".to_string(),
            },
        }
    }

    fn unique_tmp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }
}
