//! Linux-profile relation coverage map.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Serialize;

use super::{LinuxProfileBoundary, LINUX_PROFILE_VERSION};
use crate::llmwave_big::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedSummary,
};

#[derive(Clone)]
pub(crate) struct LinuxRelationProfileConfig {
    pub residual_pack: PathBuf,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxRelationProfileReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub relation_families: Vec<LinuxRelationFamily>,
    pub route_distribution: BTreeMap<String, usize>,
    pub causal_chains: Vec<LinuxCausalChain>,
    pub missing_relation_routes: Vec<String>,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxRelationFamily {
    pub family: &'static str,
    pub purpose: &'static str,
    pub routes: Vec<&'static str>,
    pub present_routes: Vec<String>,
    pub missing_routes: Vec<String>,
    pub fact_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCausalChain {
    pub chain_id: &'static str,
    pub purpose: &'static str,
    pub routes: Vec<&'static str>,
    pub present: bool,
    pub missing_routes: Vec<String>,
}

pub(crate) fn build_linux_relation_profile_report(
    config: LinuxRelationProfileConfig,
) -> Result<LinuxRelationProfileReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let route_distribution =
        packet
            .facts
            .iter()
            .fold(BTreeMap::<String, usize>::new(), |mut acc, fact| {
                *acc.entry(fact.route.clone()).or_insert(0) += 1;
                acc
            });
    let relation_families = relation_families(&route_distribution);
    let causal_chains = causal_chains(&route_distribution);
    let mut missing_relation_routes = relation_families
        .iter()
        .flat_map(|family| family.missing_routes.clone())
        .collect::<Vec<_>>();
    missing_relation_routes.sort();
    missing_relation_routes.dedup();
    let report = LinuxRelationProfileReport {
        mode: "llmwave-big-linux-relation-profile",
        version: LINUX_PROFILE_VERSION,
        verdict: if causal_chains.iter().any(|chain| chain.present) {
            "LINUX_RELATION_PROFILE_READY_NOT_CORPUS_COMPLETE"
        } else {
            "LINUX_RELATION_PROFILE_REVIEW"
        },
        residual_pack: packet.summary,
        relation_families,
        route_distribution,
        causal_chains,
        missing_relation_routes,
        claim_boundary: boundary(),
    };
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

fn relation_families(route_distribution: &BTreeMap<String, usize>) -> Vec<LinuxRelationFamily> {
    [
        (
            "package_surface",
            "package, command, binary, file ownership",
            vec![
                "linux.package.version",
                "linux.package.binary",
                "linux.package.config",
                "linux.apt.command.provider",
                "linux.apt.command.package-command",
            ],
        ),
        (
            "service_runtime",
            "systemd unit execution and dependency shape",
            vec![
                "linux.systemd.exec",
                "linux.systemd.dependency",
                "linux.systemd.identity",
                "linux.systemd.socket",
            ],
        ),
        (
            "network_runtime",
            "socket, DNS, route, firewall/exposure evidence",
            vec![
                "linux.socket.runtime",
                "linux.routing.runtime",
                "linux.dns.runtime",
                "linux.firewall.runtime",
            ],
        ),
        (
            "negative_boundaries",
            "anti-shortcut facts that block unsafe causal jumps",
            vec![
                "linux.boundary.package",
                "linux.boundary.socket",
                "linux.boundary.service",
                "linux.boundary.dns",
                "linux.boundary.container",
                "linux.boundary.cve",
            ],
        ),
        (
            "authority_identity",
            "OS identity and owner/user/group context",
            vec!["linux.identity", "linux.systemd.identity"],
        ),
    ]
    .into_iter()
    .map(|(family, purpose, routes)| {
        let present_routes = routes
            .iter()
            .filter(|route| route_distribution.contains_key(**route))
            .map(|route| route.to_string())
            .collect::<Vec<_>>();
        let missing_routes = routes
            .iter()
            .filter(|route| !route_distribution.contains_key(**route))
            .map(|route| route.to_string())
            .collect::<Vec<_>>();
        let fact_count = routes
            .iter()
            .map(|route| route_distribution.get(*route).copied().unwrap_or(0))
            .sum();
        LinuxRelationFamily {
            family,
            purpose,
            routes,
            present_routes,
            missing_routes,
            fact_count,
        }
    })
    .collect()
}

fn causal_chains(route_distribution: &BTreeMap<String, usize>) -> Vec<LinuxCausalChain> {
    [
        (
            "command_to_package",
            "answer command provider questions from command/package facts",
            vec!["linux.apt.command.provider"],
        ),
        (
            "service_to_package",
            "connect service ExecStart to package owner for the binary",
            vec!["linux.systemd.exec", "linux.package.binary"],
        ),
        (
            "exposure_proof",
            "separate listener, bind scope, and firewall evidence before exposure claims",
            vec![
                "linux.socket.runtime",
                "linux.firewall.runtime",
                "linux.boundary.socket",
            ],
        ),
        (
            "runtime_not_package_shortcut",
            "block package-installed implies running shortcut",
            vec!["linux.boundary.package", "linux.systemd.exec"],
        ),
        (
            "vulnerability_boundary",
            "block vulnerable-package implies exploitable-runtime shortcut",
            vec!["linux.boundary.cve", "linux.socket.runtime"],
        ),
    ]
    .into_iter()
    .map(|(chain_id, purpose, routes)| {
        let missing_routes = routes
            .iter()
            .filter(|route| !route_distribution.contains_key(**route))
            .map(|route| route.to_string())
            .collect::<Vec<_>>();
        LinuxCausalChain {
            chain_id,
            purpose,
            routes,
            present: missing_routes.is_empty(),
            missing_routes,
        }
    })
    .collect()
}

fn boundary() -> LinuxProfileBoundary {
    LinuxProfileBoundary {
        linux_profile_query_wave_ready: true,
        linux_profile_reasoning_ready: false,
        linux_profile_broad_eval_ready: false,
        linux_profile_broad_chat_ready: false,
        linux_profile_nonlinear_memory_proven: false,
        general_llm_ready: false,
        open_domain_chat_ready: false,
        vulnerability_scanner_ready: false,
        network_scanner_ready: false,
        safe_claim: "Linux relation profile maps coverage and missing relation routes; it is not a completed corpus or general model proof.".to_string(),
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
