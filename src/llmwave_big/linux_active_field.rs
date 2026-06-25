//! Route-balanced Linux Active Field over the append-only Linux Atlas.
//!
//! This is the cold-to-active bridge: it reads Atlas fact packs, selects a
//! bounded route-balanced working set, and runs evidence-aware probes over that
//! active field. It does not claim broad LLM readiness or nonlinear memory.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::linux_atlas::{LinuxAtlasEvidence, LinuxAtlasFact, LINUX_ATLAS_VERSION};

pub(crate) const LINUX_ACTIVE_FIELD_VERSION: &str = "llmwave-big-v-next-linux-active-field";
const PACKED_ACTIVE_RECORD_BYTES: usize = 64;
const ROUTE_INDEX_RECORD_BYTES: usize = 64;
const SIX_MIB_BYTES: usize = 6 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct LinuxActiveFieldConfig {
    pub atlas_dir: PathBuf,
    pub max_active_facts: usize,
    pub queries: Vec<String>,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActiveFieldReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub atlas_version: &'static str,
    pub verdict: &'static str,
    pub atlas_input: LinuxActiveAtlasInput,
    pub active_pack: LinuxActivePackReport,
    pub memory_budget: LinuxActiveMemoryBudget,
    pub probe_results: Vec<LinuxActiveProbeResult>,
    pub boundary_notes: Vec<LinuxBoundaryNote>,
    pub sample_active_facts: Vec<LinuxAtlasFact>,
    pub outputs: LinuxActiveOutputs,
    pub claim_boundary: LinuxActiveClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActiveAtlasInput {
    pub atlas_dir: String,
    pub fact_pack_count: usize,
    pub fact_pack_paths: Vec<String>,
    pub fact_count: usize,
    pub route_count: usize,
    pub layer_count: usize,
    pub corpus_hash: String,
    pub route_counts: BTreeMap<String, usize>,
    pub layer_counts: BTreeMap<String, usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActivePackReport {
    pub max_active_facts: usize,
    pub selected_facts: usize,
    pub selected_route_count: usize,
    pub route_balanced: bool,
    pub selection_policy: &'static str,
    pub search_scope: &'static str,
    pub route_distribution: BTreeMap<String, usize>,
    pub route_quotas: BTreeMap<String, usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActiveMemoryBudget {
    pub packed_active_record_bytes: usize,
    pub active_records_bytes: usize,
    pub route_index_bytes: usize,
    pub estimated_total_bytes: usize,
    pub hot_budget_bytes: usize,
    pub fits_6m_hot_projection: bool,
    pub binary_hot_packet_written: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActiveProbeResult {
    pub query: String,
    pub state: &'static str,
    pub top_score: i64,
    pub top_facts: Vec<LinuxActiveProbeFact>,
    pub interpretation: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActiveProbeFact {
    pub score: i64,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub confidence: u8,
    pub evidence: LinuxAtlasEvidence,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxBoundaryNote {
    pub rule: &'static str,
    pub meaning: &'static str,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActiveOutputs {
    pub report_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxActiveClaimBoundary {
    pub linux_atlas_loaded: bool,
    pub active_field_ready: bool,
    pub active_65k_projection_ready: bool,
    pub route_balanced_focus_ready: bool,
    pub query_probe_ready: bool,
    pub binary_hot_packet_ready: bool,
    pub exposure_layer_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Default)]
struct AtlasScan {
    fact_count: usize,
    route_counts: BTreeMap<String, usize>,
    layer_counts: BTreeMap<String, usize>,
    corpus_hash: String,
}

pub(crate) fn build_linux_active_field_report(
    config: LinuxActiveFieldConfig,
) -> Result<LinuxActiveFieldReport> {
    let max_active_facts = config.max_active_facts.max(1);
    let fact_paths = discover_fact_paths(&config.atlas_dir)?;
    let scan = scan_fact_paths(&fact_paths)?;
    let route_quotas = balanced_route_quotas(&scan.route_counts, max_active_facts);
    let active_facts = select_active_facts(&fact_paths, &route_quotas, max_active_facts)?;
    let route_distribution = count_selected_routes(&active_facts);
    let selected_route_count = route_distribution.len();
    let active_records_bytes = active_facts.len() * PACKED_ACTIVE_RECORD_BYTES;
    let route_index_bytes = selected_route_count * ROUTE_INDEX_RECORD_BYTES;
    let estimated_total_bytes = active_records_bytes + route_index_bytes;
    let fits_6m_hot_projection = estimated_total_bytes <= SIX_MIB_BYTES;
    let queries = build_probe_queries(config.queries);
    let probe_results = run_probe_queries(&active_facts, &queries);
    let query_probe_ready = probe_results
        .iter()
        .any(|probe| matches!(probe.state, "FOCUSED" | "BOUNDARY_FOCUSED"));
    let verdict = if active_facts.is_empty() {
        "LINUX_ACTIVE_FIELD_EMPTY"
    } else if query_probe_ready {
        "LINUX_ACTIVE_FIELD_READY_NOT_LLM"
    } else {
        "LINUX_ACTIVE_FIELD_REVIEW_NOT_LLM"
    };
    let report_path = config.out.as_ref().map(|path| path.display().to_string());
    let report = LinuxActiveFieldReport {
        mode: "llmwave-big-linux-active-field",
        version: LINUX_ACTIVE_FIELD_VERSION,
        atlas_version: LINUX_ATLAS_VERSION,
        verdict,
        atlas_input: LinuxActiveAtlasInput {
            atlas_dir: config.atlas_dir.display().to_string(),
            fact_pack_count: fact_paths.len(),
            fact_pack_paths: fact_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            fact_count: scan.fact_count,
            route_count: scan.route_counts.len(),
            layer_count: scan.layer_counts.len(),
            corpus_hash: scan.corpus_hash,
            route_counts: scan.route_counts,
            layer_counts: scan.layer_counts,
        },
        active_pack: LinuxActivePackReport {
            max_active_facts,
            selected_facts: active_facts.len(),
            selected_route_count,
            route_balanced: selected_route_count > 1 || active_facts.len() <= 1,
            selection_policy: "round_robin_route_quota_over_append_only_packs",
            search_scope: "active_pack_only",
            route_distribution,
            route_quotas,
        },
        memory_budget: LinuxActiveMemoryBudget {
            packed_active_record_bytes: PACKED_ACTIVE_RECORD_BYTES,
            active_records_bytes,
            route_index_bytes,
            estimated_total_bytes,
            hot_budget_bytes: SIX_MIB_BYTES,
            fits_6m_hot_projection,
            binary_hot_packet_written: false,
        },
        probe_results,
        boundary_notes: linux_boundary_notes(),
        sample_active_facts: active_facts.iter().take(32).cloned().collect(),
        outputs: LinuxActiveOutputs { report_path },
        claim_boundary: LinuxActiveClaimBoundary {
            linux_atlas_loaded: scan.fact_count > 0,
            active_field_ready: !active_facts.is_empty(),
            active_65k_projection_ready: max_active_facts >= 65_536
                && !active_facts.is_empty()
                && active_facts.len() == scan.fact_count.min(max_active_facts)
                && fits_6m_hot_projection,
            route_balanced_focus_ready: !active_facts.is_empty(),
            query_probe_ready,
            binary_hot_packet_ready: false,
            exposure_layer_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: "Linux Active Field builds a bounded route-balanced working set over Linux Atlas facts and runs review-grade probes. It is not a general LLM, not a vulnerability scanner, and not nonlinear-memory proof.",
            blocked_by: vec![
                "linux_binary_hot_packet_missing",
                "linux_full_eval_suite_missing",
                "linux_exposure_layer_missing",
                "general_llm_eval_missing",
                "nonlinear_memory_final_proof_missing",
            ],
        },
    };
    if let Some(path) = config.out {
        write_report(&path, &report)?;
    }
    Ok(report)
}

fn discover_fact_paths(atlas_dir: &Path) -> Result<Vec<PathBuf>> {
    let pack_log = atlas_dir.join("packs.jsonl");
    let mut paths = BTreeSet::new();
    if pack_log.exists() {
        let file =
            fs::File::open(&pack_log).with_context(|| format!("open {}", pack_log.display()))?;
        for line in BufReader::new(file).lines() {
            let line = line.with_context(|| format!("read {}", pack_log.display()))?;
            if line.trim().is_empty() {
                continue;
            }
            let value: serde_json::Value = serde_json::from_str(&line)
                .with_context(|| format!("parse pack log {}", pack_log.display()))?;
            let Some(raw_path) = value.get("facts_path").and_then(|value| value.as_str()) else {
                continue;
            };
            let path = resolve_fact_path(atlas_dir, raw_path);
            if path.exists() {
                paths.insert(path);
            }
        }
    }

    if paths.is_empty() {
        let facts_dir = atlas_dir.join("facts");
        let Ok(entries) = fs::read_dir(&facts_dir) else {
            anyhow::bail!(
                "no linux atlas fact packs found under {}",
                atlas_dir.display()
            );
        };
        for entry in entries {
            let path = entry
                .with_context(|| format!("read {}", facts_dir.display()))?
                .path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                paths.insert(path);
            }
        }
    }

    Ok(paths.into_iter().collect())
}

fn resolve_fact_path(atlas_dir: &Path, raw_path: &str) -> PathBuf {
    let path = PathBuf::from(raw_path);
    if path.exists() || path.is_absolute() {
        return path;
    }
    if let Ok(stripped) = path.strip_prefix(atlas_dir) {
        return atlas_dir.join(stripped);
    }
    if let Ok(stripped) = path.strip_prefix(".nanda/linux-atlas") {
        return atlas_dir.join(stripped);
    }
    atlas_dir.join(path)
}

fn scan_fact_paths(paths: &[PathBuf]) -> Result<AtlasScan> {
    let mut scan = AtlasScan::default();
    let mut hash = Sha256::new();
    for path in paths {
        let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        for (line_index, line) in BufReader::new(file).lines().enumerate() {
            let line = line.with_context(|| format!("read {}", path.display()))?;
            if line.trim().is_empty() {
                continue;
            }
            let fact: LinuxAtlasFact = serde_json::from_str(&line)
                .with_context(|| format!("parse {}:{}", path.display(), line_index + 1))?;
            scan.fact_count += 1;
            *scan.route_counts.entry(fact.route.clone()).or_insert(0) += 1;
            *scan.layer_counts.entry(fact.layer.clone()).or_insert(0) += 1;
            hash.update(fact.fact_id.as_bytes());
            hash.update(fact.route.as_bytes());
            hash.update(fact.subject.as_bytes());
            hash.update(fact.relation.as_bytes());
            hash.update(fact.object.as_bytes());
        }
    }
    scan.corpus_hash = format!("{:x}", hash.finalize());
    Ok(scan)
}

fn balanced_route_quotas(
    route_counts: &BTreeMap<String, usize>,
    max_active_facts: usize,
) -> BTreeMap<String, usize> {
    let mut quotas = route_counts
        .keys()
        .map(|route| (route.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut remaining = route_counts.values().sum::<usize>().min(max_active_facts);
    while remaining > 0 {
        let mut progressed = false;
        for (route, available) in route_counts {
            let quota = quotas.entry(route.clone()).or_insert(0);
            if *quota < *available {
                *quota += 1;
                remaining -= 1;
                progressed = true;
                if remaining == 0 {
                    break;
                }
            }
        }
        if !progressed {
            break;
        }
    }
    quotas
}

fn select_active_facts(
    paths: &[PathBuf],
    route_quotas: &BTreeMap<String, usize>,
    max_active_facts: usize,
) -> Result<Vec<LinuxAtlasFact>> {
    let mut selected = Vec::new();
    let mut selected_by_route = BTreeMap::<String, usize>::new();
    for path in paths {
        let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        for (line_index, line) in BufReader::new(file).lines().enumerate() {
            if selected.len() >= max_active_facts {
                return Ok(selected);
            }
            let line = line.with_context(|| format!("read {}", path.display()))?;
            if line.trim().is_empty() {
                continue;
            }
            let fact: LinuxAtlasFact = serde_json::from_str(&line)
                .with_context(|| format!("parse {}:{}", path.display(), line_index + 1))?;
            let quota = route_quotas.get(&fact.route).copied().unwrap_or(0);
            let used = selected_by_route.entry(fact.route.clone()).or_insert(0);
            if *used < quota {
                *used += 1;
                selected.push(fact);
            }
        }
    }
    Ok(selected)
}

fn count_selected_routes(facts: &[LinuxAtlasFact]) -> BTreeMap<String, usize> {
    let mut route_counts = BTreeMap::new();
    for fact in facts {
        *route_counts.entry(fact.route.clone()).or_insert(0) += 1;
    }
    route_counts
}

fn build_probe_queries(user_queries: Vec<String>) -> Vec<String> {
    let defaults = [
        "which package provides command bash",
        "which package provides command systemctl",
        "what systemd service exec starts binary",
        "package installed does not prove binary is running",
        "port listening does not prove firewall allows external packets",
    ];
    let mut seen = BTreeSet::new();
    let mut queries = Vec::new();
    for query in defaults
        .into_iter()
        .chain(user_queries.iter().map(String::as_str))
    {
        let trimmed = query.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_ascii_lowercase()) {
            queries.push(trimmed.to_string());
        }
    }
    queries
}

fn run_probe_queries(facts: &[LinuxAtlasFact], queries: &[String]) -> Vec<LinuxActiveProbeResult> {
    queries
        .iter()
        .map(|query| {
            let mut scored = facts
                .iter()
                .filter_map(|fact| {
                    let score = score_fact(query, fact);
                    (score > 0).then_some((score, fact))
                })
                .collect::<Vec<_>>();
            scored.sort_by(|left, right| {
                right
                    .0
                    .cmp(&left.0)
                    .then_with(|| left.1.fact_id.cmp(&right.1.fact_id))
            });
            let top_facts = scored
                .into_iter()
                .take(5)
                .map(|(score, fact)| LinuxActiveProbeFact {
                    score,
                    route: fact.route.clone(),
                    subject: fact.subject.clone(),
                    relation: fact.relation.clone(),
                    object: fact.object.clone(),
                    polarity: fact.polarity.clone(),
                    confidence: fact.confidence,
                    evidence: fact.evidence.clone(),
                })
                .collect::<Vec<_>>();
            let top_score = top_facts.first().map(|fact| fact.score).unwrap_or(0);
            let state = probe_state(
                top_score,
                top_facts.first().map(|fact| fact.polarity.as_str()),
            );
            LinuxActiveProbeResult {
                query: query.clone(),
                state,
                top_score,
                interpretation: probe_interpretation(state, &top_facts),
                top_facts,
            }
        })
        .collect()
}

fn score_fact(query: &str, fact: &LinuxAtlasFact) -> i64 {
    let query_lower = query.to_ascii_lowercase();
    let subject = fact.subject.to_ascii_lowercase();
    let relation = fact.relation.to_ascii_lowercase();
    let object = fact.object.to_ascii_lowercase();
    let route = fact.route.to_ascii_lowercase();
    let mut score = i64::from(fact.confidence) / 10;
    if let Some(command) = command_query_target(&query_lower) {
        if !route_can_answer_command_provider(&route) {
            return 0;
        }
        if fact_matches_command_anchor(&command, &subject, &object) {
            score += if route.contains("command") { 76 } else { 62 };
        } else {
            return 0;
        }
    }

    if !subject.is_empty() && query_lower.contains(&subject) {
        score += 35;
    }
    if !object.is_empty() && query_lower.contains(&object) {
        score += 30;
    }
    if !relation.is_empty() && query_lower.contains(&relation) {
        score += 24;
    }
    if query_lower.contains("command")
        && query_lower.contains("provid")
        && route.contains("command")
    {
        score += 44;
    }
    if (query_lower.contains("systemd") || query_lower.contains("service"))
        && route.contains("systemd")
    {
        score += 34;
    }
    if query_lower.contains("runtime") && fact.layer.contains("runtime") {
        score += 24;
    }
    if (query_lower.contains("not prove")
        || query_lower.contains("does not prove")
        || query_lower.contains("prove"))
        && (fact.polarity == "negative" || route.contains("boundary"))
    {
        score += 42;
    }
    if query_lower.contains("firewall") && object.contains("firewall") {
        score += 24;
    }
    if query_lower.contains("running") && object.contains("running") {
        score += 24;
    }

    let tokens = query_tokens(&query_lower);
    for token in tokens {
        if subject.split_ascii_whitespace().any(|part| part == token) {
            score += 9;
        }
        if relation.split_ascii_whitespace().any(|part| part == token) {
            score += 8;
        }
        if object.split_ascii_whitespace().any(|part| part == token) {
            score += 7;
        }
        if route.split('.').any(|part| part == token) {
            score += 5;
        }
    }
    score
}

fn query_tokens(query: &str) -> Vec<&str> {
    query
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() > 2)
        .collect()
}

fn command_query_target(query: &str) -> Option<String> {
    let tokens = query_tokens(query);
    for pair in tokens.windows(2) {
        if pair[0] == "command" {
            return Some(pair[1].to_string());
        }
    }
    None
}

fn route_can_answer_command_provider(route: &str) -> bool {
    route.contains("command")
        || route == "linux.package.binary"
        || route == "linux.package.admin-binary"
}

fn fact_matches_command_anchor(command: &str, subject: &str, object: &str) -> bool {
    let suffix = format!("/{command}");
    subject == command
        || object == command
        || subject.ends_with(&suffix)
        || object.ends_with(&suffix)
}

fn probe_state(top_score: i64, polarity: Option<&str>) -> &'static str {
    if top_score >= 90 && polarity == Some("negative") {
        "BOUNDARY_FOCUSED"
    } else if top_score >= 80 {
        "FOCUSED"
    } else if top_score >= 35 {
        "REVIEW"
    } else {
        "NO_MATCH"
    }
}

fn probe_interpretation(state: &str, top_facts: &[LinuxActiveProbeFact]) -> String {
    let Some(top) = top_facts.first() else {
        return "No active-field fact matched this probe.".to_string();
    };
    match state {
        "BOUNDARY_FOCUSED" => format!(
            "Boundary probe focused on {}: {} {} {}.",
            top.route, top.subject, top.relation, top.object
        ),
        "FOCUSED" => format!(
            "Active field found {} through route {}.",
            top.object, top.route
        ),
        "REVIEW" => format!(
            "Weak active-field match; inspect {} -> {} -> {} before acting.",
            top.subject, top.relation, top.object
        ),
        _ => "No safe active-field match; keep this as review-only.".to_string(),
    }
}

fn linux_boundary_notes() -> Vec<LinuxBoundaryNote> {
    vec![
        LinuxBoundaryNote {
            rule: "installed_package_not_running",
            meaning: "Package ownership facts do not prove a process is running.",
            active: true,
        },
        LinuxBoundaryNote {
            rule: "listening_socket_not_exposed",
            meaning: "A local listener does not prove external reachability or firewall exposure.",
            active: true,
        },
        LinuxBoundaryNote {
            rule: "service_unit_not_success",
            meaning:
                "A systemd unit file or active service does not prove the application is healthy.",
            active: true,
        },
    ]
}

fn write_report(path: &Path, report: &LinuxActiveFieldReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, report)
        .with_context(|| format!("write {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fact(route: &str, subject: &str, relation: &str, object: &str) -> LinuxAtlasFact {
        LinuxAtlasFact {
            fact_id: format!("test.{route}.{subject}.{object}"),
            layer: if route.contains("boundary") {
                "negative-boundary".to_string()
            } else {
                "linux-knowledge".to_string()
            },
            domain: "test".to_string(),
            route: route.to_string(),
            subject: subject.to_string(),
            relation: relation.to_string(),
            object: object.to_string(),
            polarity: if route.contains("boundary") {
                "negative".to_string()
            } else {
                "positive".to_string()
            },
            confidence: 90,
            evidence: LinuxAtlasEvidence {
                source_kind: "test".to_string(),
                path: "fixture".to_string(),
                line: 1,
                extractor: "test".to_string(),
            },
        }
    }

    #[test]
    fn balanced_quotas_limit_dominant_routes() {
        let mut routes = BTreeMap::new();
        routes.insert("dominant".to_string(), 100);
        routes.insert("rare".to_string(), 2);
        let quotas = balanced_route_quotas(&routes, 10);
        assert_eq!(quotas.get("rare"), Some(&2));
        assert_eq!(quotas.get("dominant"), Some(&8));
    }

    #[test]
    fn probe_scores_command_provider_above_unrelated_fact() {
        let provider = test_fact(
            "linux.apt.command.provider",
            "bash",
            "provided by package",
            "bash",
        );
        let unrelated = test_fact(
            "linux.package.summary",
            "coreutils",
            "summary",
            "basic file utilities",
        );
        assert!(
            score_fact("which package provides command bash", &provider)
                > score_fact("which package provides command bash", &unrelated)
        );
    }

    #[test]
    fn command_probe_requires_exact_command_anchor() {
        let false_match = test_fact(
            "linux.apt.command.package-command",
            "debianutils",
            "provides command",
            "which",
        );
        assert_eq!(
            score_fact("which package provides command systemctl", &false_match),
            0
        );
    }

    #[test]
    fn command_probe_rejects_question_word_manpage_noise() {
        let false_match = test_fact("linux.manpage.index", "which", "documented in section", "1");
        assert_eq!(
            score_fact("which package provides command systemctl", &false_match),
            0
        );
    }

    #[test]
    fn command_probe_rejects_systemd_usage_as_provider() {
        let usage = test_fact(
            "linux.systemd.exec",
            "initrd-switch-root.service",
            "execstart",
            "systemctl",
        );
        assert_eq!(
            score_fact("which package provides command systemctl", &usage),
            0
        );
    }

    #[test]
    fn command_probe_accepts_package_binary_provider() {
        let provider = test_fact(
            "linux.package.binary",
            "systemd",
            "provides binary",
            "/usr/bin/systemctl",
        );
        assert!(score_fact("which package provides command systemctl", &provider) >= 80);
    }

    #[test]
    fn boundary_probe_focuses_negative_rule() {
        let facts = vec![test_fact(
            "linux.boundary.package",
            "package installed",
            "does not prove",
            "binary is running",
        )];
        let probes = run_probe_queries(
            &facts,
            &["package installed does not prove binary is running".to_string()],
        );
        assert_eq!(probes[0].state, "BOUNDARY_FOCUSED");
    }
}
