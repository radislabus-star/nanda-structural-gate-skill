//! Linux-profile held-out and near-collision eval.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    build_linux_broad_eval_report, LinuxBroadEvalCase, LinuxBroadEvalReport,
    LinuxBroadEvalRunConfig, LinuxBroadSuite, LinuxBroadSuiteReport, LinuxProfileBoundary,
    LINUX_PROFILE_VERSION,
};
use crate::llmwave_big::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
};

#[derive(Clone)]
pub(crate) struct LinuxHeldoutSuiteBuildConfig {
    pub residual_pack: PathBuf,
    pub cases: usize,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LinuxHeldoutEvalRunConfig {
    pub residual_pack: PathBuf,
    pub suite: PathBuf,
    pub out: Option<PathBuf>,
    pub max_facts: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxHeldoutSuiteReport {
    pub mode: String,
    pub version: String,
    pub verdict: String,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub suite: LinuxBroadSuite,
    pub route_distribution: BTreeMap<String, usize>,
    pub controls: LinuxHeldoutControls,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxHeldoutControls {
    pub exact_cases: usize,
    pub near_collision_cases: usize,
    pub shortcut_control_cases: usize,
    pub endpoint_scope_cases: usize,
}

pub(crate) fn build_linux_heldout_suite_report(
    config: LinuxHeldoutSuiteBuildConfig,
) -> Result<LinuxHeldoutSuiteReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let (suite, controls) = build_heldout_suite_from_facts(&packet.facts, config.cases.max(1));
    let report = LinuxHeldoutSuiteReport {
        mode: "llmwave-big-linux-heldout-suite-build".to_string(),
        version: LINUX_PROFILE_VERSION.to_string(),
        verdict: if suite.case_count >= 20 {
            "LINUX_HELDOUT_SUITE_READY_NOT_EVAL"
        } else {
            "LINUX_HELDOUT_SUITE_REVIEW"
        }
        .to_string(),
        residual_pack: packet.summary,
        suite,
        route_distribution: route_distribution(&packet.facts),
        controls,
        claim_boundary: claim_boundary(false),
    };
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_heldout_eval_report(
    config: LinuxHeldoutEvalRunConfig,
) -> Result<LinuxBroadEvalReport> {
    let suite_report: LinuxHeldoutSuiteReport = serde_json::from_str(
        &fs::read_to_string(&config.suite)
            .with_context(|| format!("read held-out suite {}", config.suite.display()))?,
    )
    .with_context(|| format!("parse held-out suite {}", config.suite.display()))?;
    let broad_suite = LinuxBroadSuiteReport {
        mode: "llmwave-big-linux-broad-suite-build".to_string(),
        version: suite_report.version.to_string(),
        verdict: "LINUX_BROAD_SUITE_READY_NOT_EVAL".to_string(),
        residual_pack: suite_report.residual_pack.clone(),
        suite: suite_report.suite,
        route_distribution: suite_report.route_distribution,
        claim_boundary: suite_report.claim_boundary,
    };
    let temp_suite_path = config.suite.with_extension("broad-compatible.tmp.json");
    fs::write(
        &temp_suite_path,
        serde_json::to_string_pretty(&broad_suite)?,
    )
    .with_context(|| format!("write {}", temp_suite_path.display()))?;
    let mut report = build_linux_broad_eval_report(LinuxBroadEvalRunConfig {
        residual_pack: config.residual_pack,
        suite: temp_suite_path.clone(),
        out: None,
        max_facts: config.max_facts,
    })?;
    let _ = fs::remove_file(&temp_suite_path);
    report.mode = "llmwave-big-linux-heldout-eval-run".to_string();
    report.suite_profile = "linux-profile-heldout-v1".to_string();
    report.verdict = if report.metrics.total >= 20
        && report.metrics.pass_rate >= 0.95
        && report.metrics.false_positive_rate == 0.0
        && report.metrics.shortcut_rejection_rate >= 0.95
        && report.metrics.exposure_overclaim_rate == 0.0
    {
        "LINUX_PROFILE_HELDOUT_EVAL_PASS_NOT_GENERAL_LLM"
    } else {
        "LINUX_PROFILE_HELDOUT_EVAL_REVIEW"
    }
    .to_string();
    report.claim_boundary.linux_profile_broad_eval_ready =
        report.verdict == "LINUX_PROFILE_HELDOUT_EVAL_PASS_NOT_GENERAL_LLM";
    report.claim_boundary.safe_claim =
        "Linux-profile held-out eval is profile evidence only; general LLM and scanner claims remain blocked."
            .to_string();
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

fn build_heldout_suite_from_facts(
    facts: &[LinuxResidualDecodedFact],
    target_cases: usize,
) -> (LinuxBroadSuite, LinuxHeldoutControls) {
    let mut exact = Vec::new();
    let mut near = Vec::new();
    let mut shortcuts = Vec::new();
    let mut endpoint = Vec::new();
    let has_firewall = facts
        .iter()
        .any(|fact| fact.route == "linux.firewall.runtime");

    for fact in facts {
        match fact.route.as_str() {
            "linux.apt.command.provider" => {
                let missing_command = format!("definitely-missing-command-{}", near.len() + 1);
                exact.push(case(
                    "heldout_exact",
                    format!("Which package provides command {}?", fact.subject),
                    "command_provider",
                    true,
                    fact.object.clone(),
                    "",
                ));
                near.push(case(
                    "near_collision",
                    format!("Which package provides command {missing_command}?"),
                    "command_provider",
                    false,
                    "missing required linux evidence",
                    fact.object.clone(),
                ));
            }
            "linux.apt.command.package-command" => {
                let missing_command = format!("definitely-missing-command-{}", near.len() + 1);
                exact.push(case(
                    "heldout_exact",
                    format!("Which package provides command {}?", fact.object),
                    "command_provider",
                    true,
                    fact.subject.clone(),
                    "",
                ));
                near.push(case(
                    "near_collision",
                    format!("Which package provides command {missing_command}?"),
                    "command_provider",
                    false,
                    "missing required linux evidence",
                    fact.subject.clone(),
                ));
            }
            "linux.systemd.exec" => {
                let missing_service =
                    format!("definitely-missing-service-{}.service", near.len() + 1);
                exact.push(case(
                    "heldout_exact",
                    format!("What binary does {} run?", fact.subject),
                    "service_exec",
                    true,
                    fact.object.clone(),
                    "",
                ));
                near.push(case(
                    "near_collision",
                    format!("What binary does {missing_service} run?"),
                    "service_exec",
                    false,
                    "missing required linux evidence",
                    fact.object.clone(),
                ));
            }
            "linux.socket.runtime" => {
                endpoint.push(case(
                    "endpoint_scope",
                    format!("What is the bind scope for {}?", fact.object),
                    "bind_scope",
                    true,
                    if is_localhost_endpoint(&fact.object) {
                        "localhost"
                    } else {
                        "external-bind"
                    },
                    "",
                ));
                shortcuts.push(case(
                    "shortcut_control",
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
            }
            "linux.boundary.package" => shortcuts.push(case(
                "shortcut_control",
                "Does package installed prove binary is running?".to_string(),
                "package_runtime_boundary",
                true,
                "does not prove",
                "yes",
            )),
            "linux.boundary.cve" => shortcuts.push(case(
                "shortcut_control",
                "Does a vulnerable package prove runtime exposure?".to_string(),
                "vulnerability_boundary",
                true,
                "does not prove",
                "yes",
            )),
            _ => {}
        }
    }

    if exact.is_empty() {
        exact.push(case(
            "heldout_exact",
            "Which package provides command bash?".to_string(),
            "command_provider",
            false,
            "missing required linux evidence",
            "",
        ));
    }
    if near.is_empty() {
        near.push(case(
            "near_collision",
            "Which package provides command definitely-missing?".to_string(),
            "command_provider",
            false,
            "missing required linux evidence",
            "",
        ));
    }
    if shortcuts.is_empty() {
        shortcuts.push(case(
            "shortcut_control",
            "Does package installed prove binary is running?".to_string(),
            "package_runtime_boundary",
            true,
            "does not prove",
            "yes",
        ));
    }
    if endpoint.is_empty() {
        endpoint.push(case(
            "endpoint_scope",
            "What is the bind scope for 127.0.0.1:8080?".to_string(),
            "bind_scope",
            false,
            "missing required linux evidence",
            "",
        ));
    }

    let pools = [exact, near, shortcuts, endpoint];
    let mut offsets = vec![0usize; pools.len()];
    let mut cases = Vec::new();
    while cases.len() < target_cases {
        for (pool_index, pool) in pools.iter().enumerate() {
            if cases.len() >= target_cases {
                break;
            }
            let mut next = pool[offsets[pool_index] % pool.len()].clone();
            offsets[pool_index] += 1;
            next.id = format!("linux-heldout-{}-{:04}", next.family, cases.len() + 1);
            cases.push(next);
        }
    }
    let mut families = BTreeMap::new();
    for case in &cases {
        *families.entry(case.family.clone()).or_insert(0) += 1;
    }
    let controls = LinuxHeldoutControls {
        exact_cases: cases
            .iter()
            .filter(|case| case.family == "heldout_exact")
            .count(),
        near_collision_cases: cases
            .iter()
            .filter(|case| case.family == "near_collision")
            .count(),
        shortcut_control_cases: cases
            .iter()
            .filter(|case| case.family == "shortcut_control")
            .count(),
        endpoint_scope_cases: cases
            .iter()
            .filter(|case| case.family == "endpoint_scope")
            .count(),
    };
    (
        LinuxBroadSuite {
            profile: "linux-profile-heldout-v1".to_string(),
            case_count: cases.len(),
            families,
            cases,
        },
        controls,
    )
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

fn route_distribution(facts: &[LinuxResidualDecodedFact]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for fact in facts {
        *counts.entry(fact.route.clone()).or_insert(0) += 1;
    }
    counts
}

fn is_localhost_endpoint(endpoint: &str) -> bool {
    let endpoint = endpoint.to_ascii_lowercase();
    endpoint.starts_with("0100007f") || endpoint.contains("127.0.0.1")
}

fn claim_boundary(heldout_ready: bool) -> LinuxProfileBoundary {
    LinuxProfileBoundary {
        linux_profile_query_wave_ready: true,
        linux_profile_reasoning_ready: heldout_ready,
        linux_profile_broad_eval_ready: heldout_ready,
        linux_profile_broad_chat_ready: heldout_ready,
        linux_profile_nonlinear_memory_proven: false,
        general_llm_ready: false,
        open_domain_chat_ready: false,
        vulnerability_scanner_ready: false,
        network_scanner_ready: false,
        safe_claim: "Linux held-out suite is profile eval evidence only; run held-out eval and claim gate before broader claims.".to_string(),
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
