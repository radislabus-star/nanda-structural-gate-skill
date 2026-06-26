//! Guarded local VPN action planning.
//!
//! This module turns bounded Linux-profile VPN knowledge into concrete local
//! enable/disable/status command plans. It is intentionally dry-run first:
//! execution requires both `--execute` and an explicit network-drop
//! acknowledgement.

use std::process::Command;

use anyhow::{bail, Result};
use serde::Serialize;

pub(crate) const LINUX_VPN_CONTROL_VERSION: &str = "llmwave-big-v-next-linux-vpn-control";

#[derive(Clone)]
pub(crate) struct LinuxVpnActionPlanConfig {
    pub text: Option<String>,
    pub action: Option<String>,
    pub backend: Option<String>,
    pub target: Option<String>,
}

#[derive(Clone)]
pub(crate) struct LinuxVpnControlConfig {
    pub action: String,
    pub backend: String,
    pub target: Option<String>,
    pub execute: bool,
    pub i_understand_network_may_drop: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxVpnControlReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub action: String,
    pub backend: String,
    pub target: String,
    pub plan: LinuxVpnActionPlan,
    pub execution: LinuxVpnExecutionReport,
    pub claim_boundary: LinuxVpnControlClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxVpnActionPlanReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub input_text: Option<String>,
    pub action: String,
    pub backend: String,
    pub target: String,
    pub inference: LinuxVpnActionInference,
    pub plan: LinuxVpnActionPlan,
    pub claim_boundary: LinuxVpnControlClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxVpnActionInference {
    pub action_source: &'static str,
    pub backend_source: &'static str,
    pub target_source: &'static str,
    pub ambiguity: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxVpnActionPlan {
    pub action: String,
    pub backend: String,
    pub target: String,
    pub argv: Vec<String>,
    pub command_preview: String,
    pub verification_commands: Vec<Vec<String>>,
    pub preflight_notes: Vec<&'static str>,
    pub risk_notes: Vec<&'static str>,
    pub requires_sudo: bool,
    pub requires_explicit_execute: bool,
    pub requires_network_drop_confirmation: bool,
    pub secrets_required: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxVpnExecutionReport {
    pub requested: bool,
    pub executed: bool,
    pub blocked: bool,
    pub reason: String,
    pub argv_used: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout_preview: Option<String>,
    pub stderr_preview: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxVpnControlClaimBoundary {
    pub action_plan_ready: bool,
    pub control_ready: bool,
    pub dry_run: bool,
    pub execute_requested: bool,
    pub local_system_mutation_done: bool,
    pub secrets_read: bool,
    pub secrets_printed: bool,
    pub requires_explicit_execute: bool,
    pub requires_network_drop_confirmation: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VpnAction {
    Up,
    Down,
    Status,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VpnBackend {
    WireGuard,
    NetworkManager,
    Systemd,
    TrustTunnel,
}

pub(crate) fn build_linux_vpn_action_plan_report(
    config: LinuxVpnActionPlanConfig,
) -> Result<LinuxVpnActionPlanReport> {
    let text = config.text.clone().unwrap_or_default();
    let (action, action_source, action_ambiguity) = select_action(config.action.as_deref(), &text)?;
    let (backend, backend_source) = select_backend(config.backend.as_deref(), &text)?;
    let (target, target_source) = select_target(config.target.as_deref(), backend, &text);
    let plan = build_plan(action, backend, &target);
    let mut ambiguity = Vec::new();
    if action_ambiguity {
        ambiguity.push("text_mentions_enable_and_disable");
    }
    if config.backend.is_none() && !text_mentions_backend(&text) {
        ambiguity.push("backend_defaulted_to_wireguard");
    }
    if config.target.is_none() && extract_target(backend, &text).is_none() {
        ambiguity.push("target_defaulted");
    }

    Ok(LinuxVpnActionPlanReport {
        mode: "llmwave-big-linux-vpn-action-plan",
        version: LINUX_VPN_CONTROL_VERSION,
        verdict: "LINUX_VPN_ACTION_PLAN_READY_NOT_EXECUTED",
        input_text: config.text,
        action: action.as_str().to_string(),
        backend: backend.as_str().to_string(),
        target,
        inference: LinuxVpnActionInference {
            action_source,
            backend_source,
            target_source,
            ambiguity,
        },
        plan,
        claim_boundary: boundary(false, false, true),
    })
}

pub(crate) fn build_linux_vpn_control_report(
    config: LinuxVpnControlConfig,
) -> Result<LinuxVpnControlReport> {
    let action = parse_action(&config.action)?;
    let backend = parse_backend(&config.backend)?;
    let target = config
        .target
        .clone()
        .unwrap_or_else(|| default_target(backend).to_string());
    let plan = build_plan(action, backend, &target);
    let execution = maybe_execute(
        &plan,
        action,
        config.execute,
        config.i_understand_network_may_drop,
    )?;
    let executed = execution.executed;
    let verdict = if config.execute
        && action.requires_confirmation()
        && !config.i_understand_network_may_drop
    {
        "LINUX_VPN_CONTROL_BLOCKED_CONFIRMATION_REQUIRED"
    } else if executed {
        if execution.exit_code == Some(0) {
            "LINUX_VPN_CONTROL_EXECUTED"
        } else {
            "LINUX_VPN_CONTROL_EXECUTION_FAILED"
        }
    } else {
        "LINUX_VPN_CONTROL_DRY_RUN_READY"
    };

    Ok(LinuxVpnControlReport {
        mode: "llmwave-big-linux-vpn-control",
        version: LINUX_VPN_CONTROL_VERSION,
        verdict,
        action: action.as_str().to_string(),
        backend: backend.as_str().to_string(),
        target,
        plan,
        execution,
        claim_boundary: boundary(config.execute, executed, false),
    })
}

fn maybe_execute(
    plan: &LinuxVpnActionPlan,
    action: VpnAction,
    execute: bool,
    confirmation: bool,
) -> Result<LinuxVpnExecutionReport> {
    if !execute {
        return Ok(LinuxVpnExecutionReport {
            requested: false,
            executed: false,
            blocked: false,
            reason: "dry_run_only".to_string(),
            argv_used: plan.argv.clone(),
            exit_code: None,
            stdout_preview: None,
            stderr_preview: None,
        });
    }

    if action.requires_confirmation() && !confirmation {
        return Ok(LinuxVpnExecutionReport {
            requested: true,
            executed: false,
            blocked: true,
            reason: "network_drop_confirmation_required".to_string(),
            argv_used: plan.argv.clone(),
            exit_code: None,
            stdout_preview: None,
            stderr_preview: None,
        });
    }

    let argv = execution_argv(&plan.argv);
    let output = Command::new(&argv[0]).args(&argv[1..]).output()?;
    Ok(LinuxVpnExecutionReport {
        requested: true,
        executed: true,
        blocked: false,
        reason: "executed_by_explicit_request".to_string(),
        argv_used: argv,
        exit_code: output.status.code(),
        stdout_preview: Some(preview_bytes(&output.stdout)),
        stderr_preview: Some(preview_bytes(&output.stderr)),
    })
}

fn build_plan(action: VpnAction, backend: VpnBackend, target: &str) -> LinuxVpnActionPlan {
    let argv = command_argv(action, backend, target);
    let verification_commands = verification_commands(action, backend, target);
    let requires_sudo = argv.first().is_some_and(|arg| arg == "sudo");
    let requires_network_drop_confirmation = action.requires_confirmation();
    LinuxVpnActionPlan {
        action: action.as_str().to_string(),
        backend: backend.as_str().to_string(),
        target: target.to_string(),
        command_preview: argv.join(" "),
        argv,
        verification_commands,
        preflight_notes: preflight_notes(action, backend),
        risk_notes: risk_notes(action, backend),
        requires_sudo,
        requires_explicit_execute: true,
        requires_network_drop_confirmation,
        secrets_required: false,
    }
}

fn command_argv(action: VpnAction, backend: VpnBackend, target: &str) -> Vec<String> {
    match (backend, action) {
        (VpnBackend::WireGuard, VpnAction::Up) => argv(["sudo", "wg-quick", "up", target]),
        (VpnBackend::WireGuard, VpnAction::Down) => argv(["sudo", "wg-quick", "down", target]),
        (VpnBackend::WireGuard, VpnAction::Status) => argv(["wg", "show", target]),
        (VpnBackend::NetworkManager, VpnAction::Up) => argv(["nmcli", "connection", "up", target]),
        (VpnBackend::NetworkManager, VpnAction::Down) => {
            argv(["nmcli", "connection", "down", target])
        }
        (VpnBackend::NetworkManager, VpnAction::Status) => {
            argv(["nmcli", "connection", "show", "--active"])
        }
        (VpnBackend::Systemd, VpnAction::Up) => argv(["sudo", "systemctl", "start", target]),
        (VpnBackend::Systemd, VpnAction::Down) => argv(["sudo", "systemctl", "stop", target]),
        (VpnBackend::Systemd, VpnAction::Status) => {
            argv(["systemctl", "status", target, "--no-pager"])
        }
        (VpnBackend::TrustTunnel, VpnAction::Up) => argv(["sudo", "systemctl", "start", target]),
        (VpnBackend::TrustTunnel, VpnAction::Down) => argv(["sudo", "systemctl", "stop", target]),
        (VpnBackend::TrustTunnel, VpnAction::Status) => {
            argv(["systemctl", "status", target, "--no-pager"])
        }
    }
}

fn verification_commands(action: VpnAction, backend: VpnBackend, target: &str) -> Vec<Vec<String>> {
    let mut commands = match backend {
        VpnBackend::WireGuard => vec![
            argv(["wg", "show", target]),
            argv(["ip", "route"]),
            argv(["resolvectl", "status"]),
        ],
        VpnBackend::NetworkManager => vec![
            argv(["nmcli", "connection", "show", "--active"]),
            argv(["ip", "route"]),
            argv(["resolvectl", "status"]),
        ],
        VpnBackend::Systemd | VpnBackend::TrustTunnel => vec![
            argv(["systemctl", "status", target, "--no-pager"]),
            argv(["ip", "route"]),
        ],
    };
    if action == VpnAction::Down {
        commands.push(argv(["ip", "route", "get", "1.1.1.1"]));
    }
    commands
}

fn preflight_notes(action: VpnAction, backend: VpnBackend) -> Vec<&'static str> {
    let mut notes = vec![
        "dry-run first; inspect argv before execution",
        "do not print private keys, tokens, QR payloads, or tt URLs",
    ];
    if action.requires_confirmation() {
        notes.push("up/down may drop the active network path");
    }
    if backend == VpnBackend::TrustTunnel {
        notes.push("do not change production relay/cutover state from this command");
    }
    notes
}

fn risk_notes(action: VpnAction, backend: VpnBackend) -> Vec<&'static str> {
    let mut notes = Vec::new();
    if action == VpnAction::Down {
        notes.push("turning VPN off can expose the normal ISP route or break protected access");
    }
    if action == VpnAction::Up {
        notes.push("turning VPN on can replace DNS/routes and interrupt current sessions");
    }
    if backend == VpnBackend::NetworkManager {
        notes.push(
            "NetworkManager profile names are local; verify the target profile before execution",
        );
    }
    notes
}

fn select_action(explicit: Option<&str>, text: &str) -> Result<(VpnAction, &'static str, bool)> {
    if let Some(action) = explicit {
        return Ok((parse_action(action)?, "explicit_arg", false));
    }
    let lower = normalize(text);
    let wants_up = has_any(
        &lower,
        &[
            " up ",
            " on ",
            " enable ",
            " start ",
            " включ",
            " подключ",
            " запу",
        ],
    );
    let wants_down = has_any(
        &lower,
        &[
            " down ",
            " off ",
            " disable ",
            " stop ",
            " выключ",
            " отключ",
            " останов",
        ],
    );
    let wants_status = has_any(
        &lower,
        &[" status ", " check ", " show ", " стат", " провер"],
    );
    if wants_up && !wants_down {
        Ok((VpnAction::Up, "text_inferred", false))
    } else if wants_down && !wants_up {
        Ok((VpnAction::Down, "text_inferred", false))
    } else if wants_status {
        Ok((VpnAction::Status, "text_inferred", false))
    } else if wants_up && wants_down {
        Ok((
            VpnAction::Status,
            "ambiguous_text_defaulted_to_status",
            true,
        ))
    } else {
        Ok((VpnAction::Status, "default_status", false))
    }
}

fn select_backend(explicit: Option<&str>, text: &str) -> Result<(VpnBackend, &'static str)> {
    if let Some(backend) = explicit {
        return Ok((parse_backend(backend)?, "explicit_arg"));
    }
    let lower = normalize(text);
    if lower.contains("wireguard") || lower.contains(" wg ") || lower.contains("wg0") {
        Ok((VpnBackend::WireGuard, "text_inferred"))
    } else if lower.contains("networkmanager")
        || lower.contains("network manager")
        || lower.contains(" nmcli ")
    {
        Ok((VpnBackend::NetworkManager, "text_inferred"))
    } else if lower.contains("trusttunnel") || lower.contains(" tt ") {
        Ok((VpnBackend::TrustTunnel, "text_inferred"))
    } else if lower.contains("systemd")
        || lower.contains("systemctl")
        || lower.contains(" service ")
    {
        Ok((VpnBackend::Systemd, "text_inferred"))
    } else {
        Ok((VpnBackend::WireGuard, "default_wireguard"))
    }
}

fn select_target(
    explicit: Option<&str>,
    backend: VpnBackend,
    text: &str,
) -> (String, &'static str) {
    if let Some(target) = explicit {
        return (target.to_string(), "explicit_arg");
    }
    if let Some(target) = extract_target(backend, text) {
        return (target, "text_inferred");
    }
    (default_target(backend).to_string(), "backend_default")
}

fn parse_action(value: &str) -> Result<VpnAction> {
    match value.trim().to_ascii_lowercase().as_str() {
        "up" | "on" | "enable" | "start" | "включить" | "включи" => Ok(VpnAction::Up),
        "down" | "off" | "disable" | "stop" | "выключить" | "выключи" | "отключить" | "отключи" => {
            Ok(VpnAction::Down)
        }
        "status" | "check" | "show" | "статус" | "проверить" | "проверь" => {
            Ok(VpnAction::Status)
        }
        other => bail!("unsupported VPN action: {other}"),
    }
}

fn parse_backend(value: &str) -> Result<VpnBackend> {
    match value.trim().to_ascii_lowercase().as_str() {
        "wireguard" | "wg" | "wg-quick" => Ok(VpnBackend::WireGuard),
        "networkmanager" | "network-manager" | "nm" | "nmcli" => Ok(VpnBackend::NetworkManager),
        "systemd" | "systemctl" | "service" => Ok(VpnBackend::Systemd),
        "trusttunnel" | "tt" => Ok(VpnBackend::TrustTunnel),
        other => bail!("unsupported VPN backend: {other}"),
    }
}

fn extract_target(backend: VpnBackend, text: &str) -> Option<String> {
    let lower = normalize(text);
    for token in lower.split_whitespace() {
        let clean =
            token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_');
        match backend {
            VpnBackend::WireGuard if clean.starts_with("wg") && clean.len() <= 16 => {
                return Some(clean.to_string());
            }
            VpnBackend::TrustTunnel if clean.contains("trusttunnel") => {
                return Some("trusttunnel".to_string());
            }
            VpnBackend::Systemd if clean.ends_with(".service") || clean.contains('@') => {
                return Some(clean.to_string());
            }
            _ => {}
        }
    }
    None
}

fn text_mentions_backend(text: &str) -> bool {
    let lower = normalize(text);
    lower.contains("wireguard")
        || lower.contains("wg0")
        || lower.contains("networkmanager")
        || lower.contains("nmcli")
        || lower.contains("trusttunnel")
        || lower.contains("systemd")
        || lower.contains("systemctl")
}

fn default_target(backend: VpnBackend) -> &'static str {
    match backend {
        VpnBackend::WireGuard => "wg0",
        VpnBackend::NetworkManager => "vpn",
        VpnBackend::Systemd => "wg-quick@wg0.service",
        VpnBackend::TrustTunnel => "trusttunnel",
    }
}

fn boundary(
    execute_requested: bool,
    local_system_mutation_done: bool,
    action_plan_only: bool,
) -> LinuxVpnControlClaimBoundary {
    LinuxVpnControlClaimBoundary {
        action_plan_ready: true,
        control_ready: !action_plan_only,
        dry_run: !local_system_mutation_done,
        execute_requested,
        local_system_mutation_done,
        secrets_read: false,
        secrets_printed: false,
        requires_explicit_execute: true,
        requires_network_drop_confirmation: true,
        safe_claim: "VPN control can compile a local enable/disable/status plan and dry-run it. Actual up/down execution requires --execute plus --i-understand-network-may-drop.",
        blocked_claims: vec![
            "automatic_vpn_toggle_without_execute",
            "secret_material_read_or_printed",
            "production_relay_cutover",
            "general_llm_ready",
            "nonlinear_memory_proven",
        ],
    }
}

fn execution_argv(argv: &[String]) -> Vec<String> {
    if argv.first().is_some_and(|arg| arg == "sudo") && argv.get(1).is_none_or(|arg| arg != "-n") {
        let mut with_non_interactive = vec!["sudo".to_string(), "-n".to_string()];
        with_non_interactive.extend(argv.iter().skip(1).cloned());
        with_non_interactive
    } else {
        argv.to_vec()
    }
}

fn preview_bytes(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut preview = text.chars().take(600).collect::<String>();
    if text.chars().count() > 600 {
        preview.push_str("...");
    }
    preview
}

fn argv<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.into_iter().map(str::to_string).collect()
}

fn normalize(text: &str) -> String {
    format!(" {} ", text.to_ascii_lowercase())
}

fn has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

impl VpnAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Status => "status",
        }
    }

    fn requires_confirmation(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }
}

impl VpnBackend {
    fn as_str(self) -> &'static str {
        match self {
            Self::WireGuard => "wireguard",
            Self::NetworkManager => "networkmanager",
            Self::Systemd => "systemd",
            Self::TrustTunnel => "trusttunnel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_vpn_action_plan_infers_wireguard_down_without_mutation() {
        let report = build_linux_vpn_action_plan_report(LinuxVpnActionPlanConfig {
            text: Some("turn off wireguard vpn wg0".to_string()),
            action: None,
            backend: None,
            target: None,
        })
        .unwrap();

        assert_eq!(report.verdict, "LINUX_VPN_ACTION_PLAN_READY_NOT_EXECUTED");
        assert_eq!(report.action, "down");
        assert_eq!(report.backend, "wireguard");
        assert_eq!(report.target, "wg0");
        assert_eq!(report.plan.argv, argv(["sudo", "wg-quick", "down", "wg0"]));
        assert!(!report.claim_boundary.local_system_mutation_done);
        assert!(!report.claim_boundary.secrets_read);
    }

    #[test]
    fn linux_vpn_control_down_wireguard_is_dry_run_by_default() {
        let report = build_linux_vpn_control_report(LinuxVpnControlConfig {
            action: "down".to_string(),
            backend: "wireguard".to_string(),
            target: Some("wg0".to_string()),
            execute: false,
            i_understand_network_may_drop: false,
        })
        .unwrap();

        assert_eq!(report.verdict, "LINUX_VPN_CONTROL_DRY_RUN_READY");
        assert_eq!(report.plan.argv, argv(["sudo", "wg-quick", "down", "wg0"]));
        assert!(!report.execution.executed);
        assert!(!report.claim_boundary.local_system_mutation_done);
    }

    #[test]
    fn linux_vpn_control_execute_without_confirmation_is_blocked() {
        let report = build_linux_vpn_control_report(LinuxVpnControlConfig {
            action: "down".to_string(),
            backend: "wireguard".to_string(),
            target: Some("wg0".to_string()),
            execute: true,
            i_understand_network_may_drop: false,
        })
        .unwrap();

        assert_eq!(
            report.verdict,
            "LINUX_VPN_CONTROL_BLOCKED_CONFIRMATION_REQUIRED"
        );
        assert!(report.execution.requested);
        assert!(report.execution.blocked);
        assert!(!report.execution.executed);
        assert!(!report.claim_boundary.local_system_mutation_done);
    }
}
