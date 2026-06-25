//! Linux-profile decision search: goal -> missing evidence -> safe checks.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Serialize;

use super::{
    build_linux_reason_report, LinuxProfileBoundary, LinuxReasonReport, LinuxReasonRunConfig,
    LINUX_PROFILE_VERSION,
};

#[derive(Clone)]
pub(crate) struct LinuxDecisionSearchConfig {
    pub residual_pack: PathBuf,
    pub text: String,
    pub max_facts: usize,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDecisionSearchReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub question: String,
    pub reason: LinuxReasonReport,
    pub decision_search: LinuxDecisionSearch,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDecisionSearch {
    pub state: String,
    pub answer_now: bool,
    pub known_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub safe_next_checks: Vec<LinuxDecisionCheck>,
    pub forbidden_shortcuts: Vec<String>,
    pub route_contract: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDecisionCheck {
    pub check_id: String,
    pub purpose: String,
    pub expected_route: String,
    pub command_hint: String,
    pub side_effect_free: bool,
}

pub(crate) fn build_linux_decision_search_report(
    config: LinuxDecisionSearchConfig,
) -> Result<LinuxDecisionSearchReport> {
    let reason = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: config.residual_pack,
        text: config.text.clone(),
        max_facts: config.max_facts.max(1),
    })?;
    let decision_search = decision_search_from_reason(&reason);
    let ready = !decision_search.safe_next_checks.is_empty() || reason.decision.answer_allowed;
    let report = LinuxDecisionSearchReport {
        mode: "llmwave-big-linux-decision-search",
        version: LINUX_PROFILE_VERSION,
        verdict: if ready {
            "LINUX_DECISION_SEARCH_READY_NOT_SCANNER"
        } else {
            "LINUX_DECISION_SEARCH_REVIEW"
        },
        question: config.text,
        reason,
        decision_search,
        claim_boundary: boundary(),
    };
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

fn decision_search_from_reason(reason: &LinuxReasonReport) -> LinuxDecisionSearch {
    let known_evidence = reason
        .evidence_chain
        .iter()
        .map(|step| {
            format!(
                "{} [{}]: {} {} {}",
                step.role, step.route, step.subject, step.relation, step.object
            )
        })
        .collect::<Vec<_>>();
    let mut missing = reason.decision.missing_evidence.clone();
    let mut checks = Vec::new();
    match reason.query_wave.intent.as_str() {
        "external_exposure" => {
            push_missing(
                &mut missing,
                "matching firewall allow for same endpoint/port",
            );
            push_missing(&mut missing, "effective runtime service or process state");
            push_missing(&mut missing, "network namespace/interface boundary");
            checks.push(check(
                "runtime-sockets",
                "list listening sockets with process context",
                "linux.socket.runtime",
                "ss -lntp",
            ));
            checks.push(check(
                "service-state",
                "confirm service activity for the listener owner",
                "linux.systemd.exec",
                "systemctl is-active <service>",
            ));
            checks.push(check(
                "firewall-rules",
                "confirm effective allow/drop rules for the observed port",
                "linux.firewall.runtime",
                "nft list ruleset / ufw status",
            ));
            checks.push(check(
                "route-scope",
                "confirm interface and route scope before claiming internet reachability",
                "linux.routing.runtime",
                "ip route; ip addr show",
            ));
        }
        "package_runtime_boundary" => {
            push_missing(&mut missing, "runtime process/service evidence");
            checks.push(check(
                "process-state",
                "separate installed package from running process",
                "linux.runtime.process",
                "pgrep -a <binary>",
            ));
            checks.push(check(
                "service-state",
                "separate installed package from active systemd service",
                "linux.systemd.exec",
                "systemctl is-active <service>",
            ));
        }
        "vulnerability_boundary" => {
            push_missing(&mut missing, "installed vulnerable version evidence");
            push_missing(&mut missing, "runtime reachability evidence");
            push_missing(
                &mut missing,
                "configuration/exploitability condition evidence",
            );
            checks.push(check(
                "package-version",
                "confirm exact installed package version",
                "linux.package.version",
                "dpkg-query -W <package>",
            ));
            checks.push(check(
                "runtime-exposure",
                "confirm whether the vulnerable component is reachable",
                "linux.socket.runtime",
                "ss -lntp; systemctl is-active <service>",
            ));
        }
        "service_exec" | "service_package_chain" => {
            push_missing(&mut missing, "package owner for service binary");
            checks.push(check(
                "service-unit",
                "read service ExecStart without changing runtime state",
                "linux.systemd.exec",
                "systemctl cat <service>",
            ));
            checks.push(check(
                "binary-owner",
                "map service binary to package owner",
                "linux.package.binary",
                "dpkg -S <path>",
            ));
        }
        "bind_scope" | "listener_summary" => {
            push_missing(&mut missing, "endpoint-specific bind scope evidence");
            checks.push(check(
                "runtime-sockets",
                "list listening sockets and bind addresses",
                "linux.socket.runtime",
                "ss -lntp",
            ));
        }
        "command_provider" | "file_owner" => {
            checks.push(check(
                "package-index",
                "confirm package ownership through package database",
                "linux.package.binary",
                "dpkg -S <path> / command -v <cmd>",
            ));
        }
        _ => {
            push_missing(&mut missing, "supported Linux-profile intent");
        }
    }
    missing.sort();
    missing.dedup();
    LinuxDecisionSearch {
        state: if reason.decision.answer_allowed && missing.is_empty() {
            "ANSWER_ALREADY_GROUNDED".to_string()
        } else if reason.decision.answer_allowed {
            "ANSWER_WITH_MISSING_PROOF_PATH".to_string()
        } else {
            "SEARCH_REQUIRED".to_string()
        },
        answer_now: reason.decision.answer_allowed && missing.is_empty(),
        known_evidence,
        missing_evidence: missing,
        safe_next_checks: checks,
        forbidden_shortcuts: reason.query_wave.forbidden_shortcuts.clone(),
        route_contract: reason.query_wave.required_routes.clone(),
    }
}

fn check(
    check_id: &str,
    purpose: &str,
    expected_route: &str,
    command_hint: &str,
) -> LinuxDecisionCheck {
    LinuxDecisionCheck {
        check_id: check_id.to_string(),
        purpose: purpose.to_string(),
        expected_route: expected_route.to_string(),
        command_hint: command_hint.to_string(),
        side_effect_free: true,
    }
}

fn push_missing(missing: &mut Vec<String>, item: &str) {
    if !missing.iter().any(|existing| existing == item) {
        missing.push(item.to_string());
    }
}

fn boundary() -> LinuxProfileBoundary {
    LinuxProfileBoundary {
        linux_profile_query_wave_ready: true,
        linux_profile_reasoning_ready: true,
        linux_profile_broad_eval_ready: false,
        linux_profile_broad_chat_ready: false,
        linux_profile_nonlinear_memory_proven: false,
        general_llm_ready: false,
        open_domain_chat_ready: false,
        vulnerability_scanner_ready: false,
        network_scanner_ready: false,
        safe_claim: "Linux decision search proposes side-effect-free evidence checks; it is not a scanner and does not execute commands.".to_string(),
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
