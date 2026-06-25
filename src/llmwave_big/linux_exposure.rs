//! Linux exposure reasoning over schema/residual memory.
//!
//! This is not a scanner and not a vulnerability finder. It consumes the `.lrf`
//! schema/residual packet, reconstructs cold labels for explanation, and checks
//! whether runtime-socket facts are strong enough to justify an exposure claim.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
    LINUX_RESIDUAL_MEMORY_VERSION,
};
use super::linux_runtime_snapshot::{load_runtime_snapshot_overlay, LinuxRuntimeSnapshotOverlay};

pub(crate) const LINUX_EXPOSURE_VERSION: &str = "llmwave-big-v-next-linux-exposure";

#[derive(Clone)]
pub(crate) struct LinuxExposureConfig {
    pub residual_pack: PathBuf,
    pub max_candidates: usize,
    pub runtime_snapshot: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub residual_memory_version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub runtime_snapshot_overlay: Option<LinuxRuntimeSnapshotOverlay>,
    pub route_distribution: BTreeMap<String, usize>,
    pub exposure_field: LinuxExposureField,
    pub eval: LinuxExposureEvalSummary,
    pub reasoning_contract: LinuxExposureReasoningContract,
    pub claim_boundary: LinuxExposureClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureField {
    pub state: &'static str,
    pub candidate_count: usize,
    pub external_binding_count: usize,
    pub localhost_binding_count: usize,
    pub firewall_allow_fact_count: usize,
    pub service_exec_fact_count: usize,
    pub systemd_socket_fact_count: usize,
    pub package_binary_fact_count: usize,
    pub boundary_fact_count: usize,
    pub boundary_socket_present: bool,
    pub route_or_dns_context_present: bool,
    pub safe_to_claim_external_exposure: bool,
    pub candidates: Vec<LinuxExposureCandidate>,
    pub controls: Vec<LinuxExposureControl>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureCandidate {
    pub subject: String,
    pub relation: String,
    pub endpoint: String,
    pub protocol: String,
    pub port: Option<u16>,
    pub bind_scope: &'static str,
    pub memory_kind: &'static str,
    pub confidence: u8,
    pub score: i64,
    pub firewall_allow_evidence: bool,
    pub service_context_evidence: bool,
    pub verdict: &'static str,
    pub blockers: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureControl {
    pub rule: &'static str,
    pub present: bool,
    pub effect: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureEvalSummary {
    pub cases: Vec<LinuxExposureEvalCase>,
    pub metrics: LinuxExposureEvalMetrics,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureEvalCase {
    pub id: &'static str,
    pub expected: &'static str,
    pub observed: &'static str,
    pub applicable: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureEvalMetrics {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f32,
    pub false_exposure_claim_rate: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureReasoningContract {
    pub input_memory: &'static str,
    pub uses_schema_residual_packet: bool,
    pub scans_binary_memory_then_decodes_cold_labels_for_explain: bool,
    pub requires_socket_runtime_fact: bool,
    pub requires_firewall_allow_for_confirmed_external_exposure: bool,
    pub applies_negative_boundary_facts: bool,
    pub does_not_scan_network: bool,
    pub does_not_exploit: bool,
    pub does_not_claim_vulnerability: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxExposureClaimBoundary {
    pub residual_pack_loaded: bool,
    pub binary_schema_residual_memory_used: bool,
    pub linux_profile_nonlinear_memory_proven: bool,
    pub exposure_layer_ready: bool,
    pub exposure_reasoning_eval_passed: bool,
    pub external_exposure_confirmed: bool,
    pub vulnerability_scan_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Clone)]
struct ExposureStats {
    firewall_allow_fact_count: usize,
    service_exec_fact_count: usize,
    systemd_socket_fact_count: usize,
    package_binary_fact_count: usize,
    boundary_fact_count: usize,
    boundary_socket_present: bool,
    route_or_dns_context_present: bool,
}

pub(crate) fn build_linux_exposure_report(
    config: LinuxExposureConfig,
) -> Result<LinuxExposureReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let mut facts = packet.facts;
    let runtime_snapshot_overlay = if let Some(path) = config.runtime_snapshot {
        let (overlay, overlay_facts) = load_runtime_snapshot_overlay(&path)?;
        facts.extend(overlay_facts);
        Some(overlay)
    } else {
        None
    };
    Ok(build_linux_exposure_report_from_facts(
        packet.summary,
        &facts,
        runtime_snapshot_overlay,
        config.max_candidates,
    ))
}

pub(crate) fn build_linux_exposure_report_from_facts(
    summary: LinuxResidualDecodedSummary,
    facts: &[LinuxResidualDecodedFact],
    runtime_snapshot_overlay: Option<LinuxRuntimeSnapshotOverlay>,
    max_candidates: usize,
) -> LinuxExposureReport {
    let route_distribution = count_routes(facts);
    let field = build_exposure_field(facts, max_candidates.max(1));
    let eval = eval_exposure_field(&field, facts);
    let exposure_reasoning_eval_passed =
        eval.metrics.total > 0 && eval.metrics.total == eval.metrics.passed;
    let linux_profile_nonlinear_memory_proven = summary.binary_hot_sections_fit_6m
        && summary.schema_record_count > 0
        && summary.residual_record_count > 0
        && summary.beats_direct_fixed64;
    let exposure_layer_ready =
        linux_profile_nonlinear_memory_proven && exposure_reasoning_eval_passed;
    let external_exposure_confirmed = field.safe_to_claim_external_exposure;
    let verdict = if exposure_layer_ready {
        "LINUX_EXPOSURE_REASONING_READY_NOT_SCANNER"
    } else if !facts.is_empty() {
        "LINUX_EXPOSURE_REASONING_REVIEW"
    } else {
        "LINUX_EXPOSURE_REASONING_BLOCKED"
    };

    LinuxExposureReport {
        mode: "llmwave-big-linux-exposure-run",
        version: LINUX_EXPOSURE_VERSION,
        residual_memory_version: LINUX_RESIDUAL_MEMORY_VERSION,
        verdict,
        residual_pack: summary,
        runtime_snapshot_overlay,
        route_distribution,
        exposure_field: field,
        eval,
        reasoning_contract: LinuxExposureReasoningContract {
            input_memory: "lrf-schema-residual-binary-packet",
            uses_schema_residual_packet: true,
            scans_binary_memory_then_decodes_cold_labels_for_explain: true,
            requires_socket_runtime_fact: true,
            requires_firewall_allow_for_confirmed_external_exposure: true,
            applies_negative_boundary_facts: true,
            does_not_scan_network: true,
            does_not_exploit: true,
            does_not_claim_vulnerability: true,
        },
        claim_boundary: LinuxExposureClaimBoundary {
            residual_pack_loaded: true,
            binary_schema_residual_memory_used: true,
            linux_profile_nonlinear_memory_proven,
            exposure_layer_ready,
            exposure_reasoning_eval_passed,
            external_exposure_confirmed,
            vulnerability_scan_ready: false,
            broad_chat_llm_ready: false,
            safe_claim: "Linux exposure reasoning is ready as a boundary-aware readout over the .lrf schema/residual memory. It can separate local listeners, firewall evidence, and negative boundary facts; it is not a network scanner, exploit tool, vulnerability proof, or broad chat LLM.",
            blocked_claims: if exposure_layer_ready {
                vec!["vulnerability_scan_ready", "broad_chat_llm_ready"]
            } else {
                vec![
                    "linux_profile_schema_residual_memory_required",
                    "exposure_reasoning_eval_required",
                    "vulnerability_scan_ready",
                    "broad_chat_llm_ready",
                ]
            },
        },
    }
}

fn build_exposure_field(
    facts: &[LinuxResidualDecodedFact],
    max_candidates: usize,
) -> LinuxExposureField {
    let stats = exposure_stats(facts);
    let mut candidates = facts
        .iter()
        .filter(|fact| is_socket_runtime_fact(fact))
        .map(|fact| candidate_from_socket_fact(fact, facts, &stats))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.endpoint.cmp(&right.endpoint))
    });
    let candidate_count = candidates.len();
    let external_binding_count = candidates
        .iter()
        .filter(|candidate| matches!(candidate.bind_scope, "any" | "interface"))
        .count();
    let localhost_binding_count = candidates
        .iter()
        .filter(|candidate| candidate.bind_scope == "localhost")
        .count();
    let safe_to_claim_external_exposure = candidates
        .iter()
        .any(|candidate| candidate.verdict == "EXTERNAL_EXPOSURE_CONFIRMED_BY_FIREWALL");
    let state = if safe_to_claim_external_exposure {
        "EXPOSURE_CONFIRMED_REVIEW"
    } else if external_binding_count > 0 {
        "LOCAL_LISTENER_EXTERNAL_BINDING_REVIEW"
    } else if candidate_count > 0 {
        "LOCAL_LISTENER_NOT_EXTERNAL_PROOF"
    } else {
        "NO_RUNTIME_SOCKET_EVIDENCE"
    };
    LinuxExposureField {
        state,
        candidate_count,
        external_binding_count,
        localhost_binding_count,
        firewall_allow_fact_count: stats.firewall_allow_fact_count,
        service_exec_fact_count: stats.service_exec_fact_count,
        systemd_socket_fact_count: stats.systemd_socket_fact_count,
        package_binary_fact_count: stats.package_binary_fact_count,
        boundary_fact_count: stats.boundary_fact_count,
        boundary_socket_present: stats.boundary_socket_present,
        route_or_dns_context_present: stats.route_or_dns_context_present,
        safe_to_claim_external_exposure,
        candidates: candidates.into_iter().take(max_candidates).collect(),
        controls: exposure_controls(&stats),
    }
}

fn candidate_from_socket_fact(
    fact: &LinuxResidualDecodedFact,
    facts: &[LinuxResidualDecodedFact],
    stats: &ExposureStats,
) -> LinuxExposureCandidate {
    let endpoint = parse_socket_endpoint(&fact.subject, &fact.object);
    let firewall_allow_evidence = firewall_allows_endpoint(facts, endpoint.port);
    let service_context_evidence =
        stats.service_exec_fact_count > 0 || stats.systemd_socket_fact_count > 0;
    let external_scope = matches!(endpoint.bind_scope, "any" | "interface");
    let mut blockers = Vec::new();
    let verdict = if endpoint.bind_scope == "localhost" {
        blockers.push("localhost_binding_only");
        "LOCAL_ONLY_NOT_EXTERNAL_EXPOSURE"
    } else if external_scope && firewall_allow_evidence {
        "EXTERNAL_EXPOSURE_CONFIRMED_BY_FIREWALL"
    } else if external_scope {
        if stats.boundary_socket_present {
            blockers.push("port_listening_does_not_prove_firewall_allows_external_packets");
        }
        blockers.push("firewall_allow_evidence_missing");
        "LOCAL_LISTENER_REVIEW_NOT_CONFIRMED"
    } else {
        blockers.push("bind_scope_unknown");
        "SOCKET_FACT_REVIEW"
    };
    let mut score = i64::from(fact.confidence);
    if external_scope {
        score += 20;
    }
    if endpoint.bind_scope == "localhost" {
        score -= 25;
    }
    if firewall_allow_evidence {
        score += 55;
    }
    if service_context_evidence {
        score += 10;
    }
    if stats.boundary_socket_present && !firewall_allow_evidence {
        score -= 18;
    }

    LinuxExposureCandidate {
        subject: fact.subject.clone(),
        relation: fact.relation.clone(),
        endpoint: fact.object.clone(),
        protocol: endpoint.protocol,
        port: endpoint.port,
        bind_scope: endpoint.bind_scope,
        memory_kind: fact.memory_kind,
        confidence: fact.confidence,
        score,
        firewall_allow_evidence,
        service_context_evidence,
        verdict,
        blockers,
    }
}

fn eval_exposure_field(
    field: &LinuxExposureField,
    facts: &[LinuxResidualDecodedFact],
) -> LinuxExposureEvalSummary {
    let package_only_has_no_candidate = !field
        .candidates
        .iter()
        .any(|candidate| candidate.subject.contains("package"));
    let socket_boundary_blocks_false_confirm = !field.candidates.iter().any(|candidate| {
        candidate.verdict == "EXTERNAL_EXPOSURE_CONFIRMED_BY_FIREWALL"
            && !candidate.firewall_allow_evidence
    });
    let local_listener_blocked = field
        .candidates
        .iter()
        .filter(|candidate| candidate.bind_scope == "localhost")
        .all(|candidate| candidate.verdict == "LOCAL_ONLY_NOT_EXTERNAL_EXPOSURE");
    let firewall_matched_external_candidate_exists = field.candidates.iter().any(|candidate| {
        candidate.firewall_allow_evidence && matches!(candidate.bind_scope, "any" | "interface")
    });
    let positive_firewall_confirms = if !firewall_matched_external_candidate_exists {
        true
    } else {
        field
            .candidates
            .iter()
            .any(|candidate| candidate.verdict == "EXTERNAL_EXPOSURE_CONFIRMED_BY_FIREWALL")
    };
    let has_package_fact = facts.iter().any(|fact| {
        fact.route.contains("package")
            && !is_socket_runtime_fact(fact)
            && fact.polarity == "positive"
    });
    let cases = vec![
        LinuxExposureEvalCase {
            id: "package-fact-is-not-exposure",
            expected: "package facts must not create socket candidates",
            observed: if package_only_has_no_candidate {
                "package fact stayed out of exposure candidates"
            } else {
                "package fact leaked into exposure candidates"
            },
            applicable: has_package_fact,
            passed: !has_package_fact || package_only_has_no_candidate,
        },
        LinuxExposureEvalCase {
            id: "socket-boundary-prevents-false-confirm",
            expected: "listener cannot be confirmed without firewall allow evidence",
            observed: if socket_boundary_blocks_false_confirm {
                "no unbacked exposure confirmation"
            } else {
                "unbacked exposure confirmation detected"
            },
            applicable: field.boundary_socket_present,
            passed: !field.boundary_socket_present || socket_boundary_blocks_false_confirm,
        },
        LinuxExposureEvalCase {
            id: "localhost-binding-is-local-only",
            expected: "localhost listener is local-only",
            observed: if local_listener_blocked {
                "localhost candidates are local-only"
            } else {
                "localhost candidate became external"
            },
            applicable: field.localhost_binding_count > 0,
            passed: field.localhost_binding_count == 0 || local_listener_blocked,
        },
        LinuxExposureEvalCase {
            id: "firewall-allow-required-for-confirmed-exposure",
            expected: "firewall allow plus listener can confirm exposure for review",
            observed: if positive_firewall_confirms {
                "confirmed exposure has firewall evidence or no allow fact exists"
            } else {
                "firewall allow fact exists but no candidate used it"
            },
            applicable: firewall_matched_external_candidate_exists,
            passed: positive_firewall_confirms,
        },
    ];
    let total = cases.len();
    let passed = cases.iter().filter(|case| case.passed).count();
    let false_exposure_claims = field
        .candidates
        .iter()
        .filter(|candidate| {
            candidate.verdict == "EXTERNAL_EXPOSURE_CONFIRMED_BY_FIREWALL"
                && !candidate.firewall_allow_evidence
        })
        .count();
    LinuxExposureEvalSummary {
        cases,
        metrics: LinuxExposureEvalMetrics {
            total,
            passed,
            pass_rate: ratio(passed, total),
            false_exposure_claim_rate: ratio(false_exposure_claims, field.candidate_count),
        },
    }
}

fn exposure_stats(facts: &[LinuxResidualDecodedFact]) -> ExposureStats {
    ExposureStats {
        firewall_allow_fact_count: facts
            .iter()
            .filter(|fact| is_firewall_allow_fact(fact))
            .count(),
        service_exec_fact_count: facts
            .iter()
            .filter(|fact| fact.route == "linux.systemd.exec")
            .count(),
        systemd_socket_fact_count: facts
            .iter()
            .filter(|fact| fact.route == "linux.systemd.socket")
            .count(),
        package_binary_fact_count: facts
            .iter()
            .filter(|fact| fact.route == "linux.package.binary")
            .count(),
        boundary_fact_count: facts
            .iter()
            .filter(|fact| fact.route.contains("boundary") || fact.polarity == "negative")
            .count(),
        boundary_socket_present: facts
            .iter()
            .any(|fact| fact.route == "linux.boundary.socket"),
        route_or_dns_context_present: facts.iter().any(|fact| {
            matches!(
                fact.route.as_str(),
                "linux.routing.runtime" | "linux.dns.runtime"
            )
        }),
    }
}

fn exposure_controls(stats: &ExposureStats) -> Vec<LinuxExposureControl> {
    vec![
        LinuxExposureControl {
            rule: "installed_package_not_running",
            present: true,
            effect: "package or binary facts are excluded from exposure candidates",
        },
        LinuxExposureControl {
            rule: "listening_socket_not_exposed",
            present: stats.boundary_socket_present,
            effect: "local listener candidates need firewall allow evidence before confirmation",
        },
        LinuxExposureControl {
            rule: "runtime_context_not_health",
            present: stats.service_exec_fact_count > 0 || stats.route_or_dns_context_present,
            effect: "service, route, and DNS facts are supporting context only",
        },
    ]
}

fn count_routes(facts: &[LinuxResidualDecodedFact]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for fact in facts {
        *counts.entry(fact.route.clone()).or_insert(0) += 1;
    }
    counts
}

fn is_socket_runtime_fact(fact: &LinuxResidualDecodedFact) -> bool {
    matches!(
        fact.route.as_str(),
        "linux.socket.runtime" | "linux.systemd.socket"
    ) && fact.polarity == "positive"
}

fn is_firewall_allow_fact(fact: &LinuxResidualDecodedFact) -> bool {
    if fact.polarity != "positive" {
        return false;
    }
    let haystack = format!(
        "{} {} {} {}",
        fact.route, fact.subject, fact.relation, fact.object
    )
    .to_ascii_lowercase();
    (haystack.contains("firewall")
        || haystack.contains("ufw")
        || haystack.contains("iptables")
        || haystack.contains("nft"))
        && (haystack.contains("allow")
            || haystack.contains("accept")
            || haystack.contains("open")
            || haystack.contains("published"))
}

fn firewall_allows_endpoint(facts: &[LinuxResidualDecodedFact], port: Option<u16>) -> bool {
    let Some(port) = port else {
        return false;
    };
    facts.iter().any(|fact| {
        if !is_firewall_allow_fact(fact) {
            return false;
        }
        let haystack = format!("{} {} {}", fact.subject, fact.relation, fact.object);
        haystack.contains(&port.to_string())
    })
}

struct ParsedSocketEndpoint {
    protocol: String,
    port: Option<u16>,
    bind_scope: &'static str,
}

fn parse_socket_endpoint(subject: &str, object: &str) -> ParsedSocketEndpoint {
    let protocol = subject
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .find(|part| matches!(*part, "tcp" | "tcp6" | "udp" | "udp6"))
        .unwrap_or(subject)
        .to_string();
    if let Some((address_hex, port_hex)) = object.split_once(':') {
        if !address_hex.is_empty() && port_hex.len() >= 2 && is_hexish(address_hex) {
            return ParsedSocketEndpoint {
                protocol,
                port: u16::from_str_radix(&port_hex[..port_hex.len().min(4)], 16).ok(),
                bind_scope: bind_scope_from_hex_address(address_hex),
            };
        }
    }
    let lower = object.to_ascii_lowercase();
    let bind_scope = if lower.contains("127.0.0.1")
        || lower.contains("localhost")
        || lower.contains("[::1]")
        || lower.contains("::1")
    {
        "localhost"
    } else if lower.contains("0.0.0.0") || lower.contains("[::]") || lower.contains(":::") {
        "any"
    } else {
        "unknown"
    };
    ParsedSocketEndpoint {
        protocol,
        port: parse_port_from_text(object),
        bind_scope,
    }
}

fn bind_scope_from_hex_address(address_hex: &str) -> &'static str {
    let upper = address_hex.to_ascii_uppercase();
    if upper.chars().all(|ch| ch == '0') {
        "any"
    } else if upper == "0100007F"
        || (upper.len() == 32
            && upper.starts_with("000000000000000000000000")
            && upper.ends_with("01000000"))
    {
        "localhost"
    } else {
        "interface"
    }
}

fn is_hexish(value: &str) -> bool {
    value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn parse_port_from_text(value: &str) -> Option<u16> {
    value
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u16>().ok())
        .next_back()
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
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn exposure_reasoning_confirms_only_with_firewall_evidence() {
        let root = fixture_root("linux-exposure-confirmed", true);
        let out = root.join("linux.lrf");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 12,
            promotion_threshold: 2,
            out: out.clone(),
        })
        .unwrap();

        let report = build_linux_exposure_report(LinuxExposureConfig {
            residual_pack: out,
            max_candidates: 8,
            runtime_snapshot: None,
        })
        .unwrap();
        assert_eq!(report.verdict, "LINUX_EXPOSURE_REASONING_READY_NOT_SCANNER");
        assert!(report.claim_boundary.exposure_layer_ready);
        assert!(report.claim_boundary.linux_profile_nonlinear_memory_proven);
        assert!(!report.claim_boundary.broad_chat_llm_ready);
        assert!(report.exposure_field.safe_to_claim_external_exposure);
        assert_eq!(report.eval.metrics.passed, report.eval.metrics.total);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn exposure_reasoning_blocks_package_only_memory() {
        let root = fixture_root("linux-exposure-package-only", false);
        let out = root.join("linux.lrf");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 4,
            promotion_threshold: 2,
            out: out.clone(),
        })
        .unwrap();

        let report = build_linux_exposure_report(LinuxExposureConfig {
            residual_pack: out,
            max_candidates: 8,
            runtime_snapshot: None,
        })
        .unwrap();
        assert_eq!(report.exposure_field.state, "NO_RUNTIME_SOCKET_EVIDENCE");
        assert!(!report.exposure_field.safe_to_claim_external_exposure);
        assert_eq!(report.eval.metrics.false_exposure_claim_rate, 0.0);
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_root(prefix: &str, include_socket: bool) -> PathBuf {
        let root = unique_tmp_dir(prefix);
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let mut facts = vec![
            test_fact(
                "linux.package.binary",
                "openssh-server",
                "provides binary",
                "/usr/sbin/sshd",
                "positive",
            ),
            test_fact(
                "linux.package.binary",
                "systemd",
                "provides binary",
                "/usr/bin/systemctl",
                "positive",
            ),
            test_fact(
                "linux.boundary.socket",
                "port listening",
                "does not prove",
                "firewall allows external packets",
                "negative",
            ),
            test_fact(
                "linux.boundary.package",
                "package installed",
                "does not prove",
                "binary is running",
                "negative",
            ),
        ];
        if include_socket {
            facts.extend([
                test_fact(
                    "linux.socket.runtime",
                    "tcp",
                    "listens on",
                    "00000000:0016",
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
                    "linux.systemd.exec",
                    "ssh.service",
                    "execstart",
                    "/usr/sbin/sshd",
                    "positive",
                ),
                test_fact(
                    "linux.firewall.runtime",
                    "ufw",
                    "allows port",
                    "22/tcp",
                    "positive",
                ),
            ]);
        }
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
            domain: "linux-exposure-test".to_string(),
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
