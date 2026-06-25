//! Side-effect-free Linux runtime snapshot ingestion.
//!
//! This module converts a user-provided JSON snapshot into temporary Linux
//! profile facts. It never runs `ss`, `nft`, `ufw`, `systemctl`, or any other
//! runtime command. The snapshot is an overlay for a single reasoning pass, not
//! a rewrite of the `.lrf` schema/residual memory.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::linux_residual_memory::LinuxResidualDecodedFact;

pub(crate) const LINUX_RUNTIME_SNAPSHOT_VERSION: &str = "llmwave-big-v-next-linux-runtime-snapshot";

#[derive(Clone)]
pub(crate) struct LinuxRuntimeSnapshotImportConfig {
    pub snapshot: PathBuf,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxRuntimeSnapshotOverlay {
    pub path: String,
    pub fact_count: usize,
    pub firewall_allow_fact_count: usize,
    pub socket_fact_count: usize,
    pub service_state_fact_count: usize,
    pub route_distribution: BTreeMap<String, usize>,
    pub commands_executed: bool,
    pub scanner_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxRuntimeSnapshotImportReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub overlay: LinuxRuntimeSnapshotOverlay,
    pub facts: Vec<LinuxRuntimeSnapshotFactPreview>,
    pub claim_boundary: LinuxRuntimeSnapshotClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxRuntimeSnapshotFactPreview {
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: &'static str,
    pub confidence: u8,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxRuntimeSnapshotClaimBoundary {
    pub runtime_snapshot_imported: bool,
    pub side_effect_free: bool,
    pub commands_executed: bool,
    pub writes_hot_memory: bool,
    pub confirms_exposure_by_itself: bool,
    pub network_scanner_ready: bool,
    pub vulnerability_scanner_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

pub(crate) fn build_linux_runtime_snapshot_import_report(
    config: LinuxRuntimeSnapshotImportConfig,
) -> Result<LinuxRuntimeSnapshotImportReport> {
    let (overlay, facts) = load_runtime_snapshot_overlay(&config.snapshot)?;
    let report = LinuxRuntimeSnapshotImportReport {
        mode: "llmwave-big-linux-snapshot-import",
        version: LINUX_RUNTIME_SNAPSHOT_VERSION,
        verdict: if overlay.fact_count > 0 {
            "LINUX_RUNTIME_SNAPSHOT_IMPORTED_NOT_SCANNER"
        } else {
            "LINUX_RUNTIME_SNAPSHOT_EMPTY_REVIEW"
        },
        overlay,
        facts: facts.iter().map(preview_fact).collect(),
        claim_boundary: LinuxRuntimeSnapshotClaimBoundary {
            runtime_snapshot_imported: !facts.is_empty(),
            side_effect_free: true,
            commands_executed: false,
            writes_hot_memory: false,
            confirms_exposure_by_itself: false,
            network_scanner_ready: false,
            vulnerability_scanner_ready: false,
            safe_claim: "Linux runtime snapshot import converts a provided JSON snapshot into temporary profile facts. It does not run checks, scan the network, mutate hot memory, or prove exposure by itself.",
            blocked_claims: vec![
                "network_scanner_ready",
                "vulnerability_scanner_ready",
                "persistent_training_ready",
            ],
        },
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

pub(crate) fn load_runtime_snapshot_overlay(
    path: &Path,
) -> Result<(LinuxRuntimeSnapshotOverlay, Vec<LinuxResidualDecodedFact>)> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let facts = snapshot_facts(&value);
    let overlay = overlay_summary(path, &facts);
    Ok((overlay, facts))
}

fn snapshot_facts(value: &Value) -> Vec<LinuxResidualDecodedFact> {
    let mut facts = Vec::new();
    collect_firewall_facts(value, &mut facts);
    collect_socket_facts(value, &mut facts);
    collect_service_state_facts(value, &mut facts);
    dedup_facts(facts)
}

fn collect_firewall_facts(value: &Value, facts: &mut Vec<LinuxResidualDecodedFact>) {
    for candidate in [
        value.get("firewall"),
        value.get("firewall_rules"),
        value.get("rules"),
        value.get("ufw"),
        value.get("nft"),
        value.get("iptables"),
    ]
    .into_iter()
    .flatten()
    {
        collect_firewall_value(candidate, facts, "firewall");
    }
}

fn collect_firewall_value(value: &Value, facts: &mut Vec<LinuxResidualDecodedFact>, engine: &str) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_firewall_value(item, facts, engine);
            }
        }
        Value::Object(map) => {
            let engine = string_field(value, &["engine", "source", "name"]).unwrap_or(engine);
            if let Some(allowed_ports) = map
                .get("allow")
                .or_else(|| map.get("allows"))
                .or_else(|| map.get("allowed_ports"))
            {
                collect_allowed_ports(allowed_ports, facts, engine);
            }
            if let Some(rules) = map.get("rules").or_else(|| map.get("firewall_rules")) {
                collect_firewall_value(rules, facts, engine);
            }
            if rule_is_allow(value) {
                if let Some(port) = port_field(value) {
                    let protocol = string_field(value, &["protocol", "proto"]).unwrap_or("tcp");
                    let scope =
                        string_field(value, &["scope", "from", "source"]).unwrap_or("external");
                    push_fact(
                        facts,
                        "linux.firewall.runtime",
                        engine,
                        "allows port",
                        &format!("{port}/{protocol} {scope}"),
                        "positive",
                        96,
                    );
                }
            }
        }
        _ => collect_allowed_ports(value, facts, engine),
    }
}

fn collect_allowed_ports(value: &Value, facts: &mut Vec<LinuxResidualDecodedFact>, engine: &str) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_allowed_ports(item, facts, engine);
            }
        }
        Value::Object(_) => {
            if let Some(port) = port_field(value) {
                let protocol = string_field(value, &["protocol", "proto"]).unwrap_or("tcp");
                let scope = string_field(value, &["scope", "from", "source"]).unwrap_or("external");
                push_fact(
                    facts,
                    "linux.firewall.runtime",
                    engine,
                    "allows port",
                    &format!("{port}/{protocol} {scope}"),
                    "positive",
                    96,
                );
            }
        }
        Value::Number(number) => {
            if let Some(port) = number.as_u64().filter(|port| *port <= u16::MAX as u64) {
                push_fact(
                    facts,
                    "linux.firewall.runtime",
                    engine,
                    "allows port",
                    &format!("{port}/tcp external"),
                    "positive",
                    94,
                );
            }
        }
        Value::String(text) => {
            if let Some(port) = parse_port(text) {
                let protocol = if text.to_ascii_lowercase().contains("udp") {
                    "udp"
                } else {
                    "tcp"
                };
                push_fact(
                    facts,
                    "linux.firewall.runtime",
                    engine,
                    "allows port",
                    &format!("{port}/{protocol} external"),
                    "positive",
                    94,
                );
            }
        }
        _ => {}
    }
}

fn collect_socket_facts(value: &Value, facts: &mut Vec<LinuxResidualDecodedFact>) {
    for candidate in [
        value.get("sockets"),
        value.get("listeners"),
        value.get("listening_sockets"),
    ]
    .into_iter()
    .flatten()
    {
        collect_socket_value(candidate, facts);
    }
}

fn collect_socket_value(value: &Value, facts: &mut Vec<LinuxResidualDecodedFact>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_socket_value(item, facts);
            }
        }
        Value::Object(_) => {
            let Some(port) = port_field(value) else {
                return;
            };
            let address = string_field(value, &["local_address", "address", "addr", "bind"])
                .unwrap_or("0.0.0.0");
            let protocol = string_field(value, &["protocol", "proto"]).unwrap_or("tcp");
            push_fact(
                facts,
                "linux.socket.runtime",
                protocol,
                "listens on",
                &format!("{address}:{port}"),
                "positive",
                88,
            );
        }
        _ => {}
    }
}

fn collect_service_state_facts(value: &Value, facts: &mut Vec<LinuxResidualDecodedFact>) {
    for candidate in [
        value.get("services"),
        value.get("systemd"),
        value.get("service_states"),
    ]
    .into_iter()
    .flatten()
    {
        collect_service_state_value(candidate, facts);
    }
}

fn collect_service_state_value(value: &Value, facts: &mut Vec<LinuxResidualDecodedFact>) {
    match value {
        Value::Object(map) => {
            for (name, state) in map {
                if let Some(state) = state.as_str() {
                    push_fact(
                        facts,
                        "linux.systemd.state",
                        name,
                        "state",
                        state,
                        if state.eq_ignore_ascii_case("active") {
                            "positive"
                        } else {
                            "negative"
                        },
                        82,
                    );
                } else {
                    collect_service_state_value(state, facts);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                if let Some(name) = string_field(item, &["service", "unit", "name"]) {
                    let state = string_field(item, &["state", "active_state", "status"])
                        .unwrap_or("unknown");
                    push_fact(
                        facts,
                        "linux.systemd.state",
                        name,
                        "state",
                        state,
                        if state.eq_ignore_ascii_case("active") {
                            "positive"
                        } else {
                            "negative"
                        },
                        82,
                    );
                }
            }
        }
        _ => {}
    }
}

fn overlay_summary(path: &Path, facts: &[LinuxResidualDecodedFact]) -> LinuxRuntimeSnapshotOverlay {
    let mut route_distribution = BTreeMap::new();
    for fact in facts {
        *route_distribution.entry(fact.route.clone()).or_insert(0) += 1;
    }
    LinuxRuntimeSnapshotOverlay {
        path: path.display().to_string(),
        fact_count: facts.len(),
        firewall_allow_fact_count: facts
            .iter()
            .filter(|fact| fact.route == "linux.firewall.runtime" && fact.polarity == "positive")
            .count(),
        socket_fact_count: facts
            .iter()
            .filter(|fact| fact.route == "linux.socket.runtime")
            .count(),
        service_state_fact_count: facts
            .iter()
            .filter(|fact| fact.route == "linux.systemd.state")
            .count(),
        route_distribution,
        commands_executed: false,
        scanner_ready: false,
    }
}

fn preview_fact(fact: &LinuxResidualDecodedFact) -> LinuxRuntimeSnapshotFactPreview {
    LinuxRuntimeSnapshotFactPreview {
        route: fact.route.clone(),
        subject: fact.subject.clone(),
        relation: fact.relation.clone(),
        object: fact.object.clone(),
        polarity: fact.polarity,
        confidence: fact.confidence,
    }
}

fn dedup_facts(facts: Vec<LinuxResidualDecodedFact>) -> Vec<LinuxResidualDecodedFact> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for fact in facts {
        let key = (
            fact.route.clone(),
            fact.subject.clone(),
            fact.relation.clone(),
            fact.object.clone(),
            fact.polarity,
        );
        if seen.insert(key) {
            out.push(fact);
        }
    }
    out
}

fn push_fact(
    facts: &mut Vec<LinuxResidualDecodedFact>,
    route: &str,
    subject: &str,
    relation: &str,
    object: &str,
    polarity: &'static str,
    confidence: u8,
) {
    facts.push(LinuxResidualDecodedFact {
        route: route.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        polarity,
        confidence,
        memory_kind: "runtime-snapshot",
    });
}

fn rule_is_allow(value: &Value) -> bool {
    let text = [
        string_field(value, &["action", "target", "policy", "verdict"]),
        string_field(value, &["effect", "decision"]),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_ascii_lowercase();
    text.contains("allow")
        || text.contains("accept")
        || text.contains("open")
        || text.contains("permit")
}

fn port_field(value: &Value) -> Option<u16> {
    for key in ["port", "dport", "destination_port", "local_port"] {
        let Some(candidate) = value.get(key) else {
            continue;
        };
        if let Some(number) = candidate
            .as_u64()
            .filter(|number| *number <= u16::MAX as u64)
        {
            return Some(number as u16);
        }
        if let Some(text) = candidate.as_str() {
            if let Some(port) = parse_port(text) {
                return Some(port);
            }
        }
    }
    None
}

fn string_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| value.get(*key)?.as_str())
}

fn parse_port(text: &str) -> Option<u16> {
    text.split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .find_map(|part| part.parse::<u16>().ok())
}

fn write_json_if_requested<T: Serialize>(out: Option<&Path>, report: &T) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn runtime_snapshot_imports_firewall_socket_and_service_facts() {
        let path = temp_snapshot();
        fs::write(
            &path,
            r#"{
              "firewall": {
                "engine": "ufw",
                "rules": [
                  {"action": "allow", "port": 22, "protocol": "tcp", "scope": "external"}
                ]
              },
              "listeners": [
                {"protocol": "tcp", "local_address": "0.0.0.0", "port": 22}
              ],
              "services": {"ssh.service": "active"}
            }"#,
        )
        .unwrap();
        let (overlay, facts) = load_runtime_snapshot_overlay(&path).unwrap();
        assert_eq!(overlay.firewall_allow_fact_count, 1);
        assert_eq!(overlay.socket_fact_count, 1);
        assert_eq!(overlay.service_state_fact_count, 1);
        assert!(facts
            .iter()
            .any(|fact| fact.route == "linux.firewall.runtime" && fact.object.contains("22/tcp")));
        let _ = fs::remove_file(path);
    }

    fn temp_snapshot() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("nanda-linux-runtime-snapshot-{nonce}.json"))
    }
}
