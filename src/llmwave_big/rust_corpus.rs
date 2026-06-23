//! Rust-oriented structural corpus builder for LLMWave-Big proof gates.
//!
//! This is cold corpus code. It scans Rust source files and emits structural
//! facts for owner/route proof work; it is not part of the hot field loop.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const RUST_CORPUS_VERSION: &str = "llmwave-big-v-next-rust-corpus-build";

#[derive(Clone)]
pub(crate) struct RustCorpusBuildConfig {
    pub repo: PathBuf,
    pub out: Option<PathBuf>,
    pub max_file_bytes: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustCorpusBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub profile: &'static str,
    pub repo: String,
    pub artifact: RustCorpusArtifact,
    pub metrics: RustCorpusMetrics,
    pub routes: Vec<RustRouteSummary>,
    pub facts: Vec<RustStructuralFact>,
    pub output: RustCorpusOutput,
    pub claim_boundary: RustCorpusClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustCorpusArtifact {
    pub corpus_kind: &'static str,
    pub corpus_hash: String,
    pub rust_files: usize,
    pub modules: usize,
    pub functions: usize,
    pub structs: usize,
    pub enums: usize,
    pub impl_blocks: usize,
    pub tests: usize,
    pub public_exports: usize,
    pub cli_dispatch_hints: usize,
    pub report_printer_hints: usize,
    pub route_count: usize,
    pub fact_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustCorpusMetrics {
    pub route_balance_ratio: f64,
    pub max_route_facts: usize,
    pub min_route_facts: usize,
    pub owner_route_count: usize,
    pub proof_route_count: usize,
    pub route_balanced: bool,
    pub heldout_suite_ready: bool,
    pub final_proof_gate_passed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustRouteSummary {
    pub route: &'static str,
    pub fact_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustStructuralFact {
    pub route: &'static str,
    pub subject: String,
    pub relation: &'static str,
    pub object: String,
    pub evidence_path: String,
    pub evidence_line: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustCorpusOutput {
    pub artifact_written: bool,
    pub artifact_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RustCorpusClaimBoundary {
    pub rust_corpus_loaded: bool,
    pub heldout_suite_ready: bool,
    pub focus_packet_ready: bool,
    pub final_proof_gate_passed: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

struct RustFileScan {
    path: PathBuf,
    module: String,
    facts: Vec<RustStructuralFact>,
    modules: usize,
    functions: usize,
    structs: usize,
    enums: usize,
    impl_blocks: usize,
    tests: usize,
    public_exports: usize,
    cli_dispatch_hints: usize,
    report_printer_hints: usize,
}

pub(crate) fn build_rust_corpus_report(
    config: RustCorpusBuildConfig,
) -> Result<RustCorpusBuildReport> {
    let repo = config
        .repo
        .canonicalize()
        .with_context(|| format!("canonicalize repo {}", config.repo.display()))?;
    let rust_files = collect_rust_files(&repo)?;
    let mut scans = Vec::new();
    for path in rust_files {
        let metadata = fs::metadata(&path)
            .with_context(|| format!("read metadata for Rust file {}", path.display()))?;
        if metadata.len() as usize > config.max_file_bytes {
            continue;
        }
        scans.push(scan_rust_file(&repo, &path)?);
    }
    scans.sort_by(|a, b| a.path.cmp(&b.path));

    let mut facts = Vec::new();
    let mut modules = 0usize;
    let mut functions = 0usize;
    let mut structs = 0usize;
    let mut enums = 0usize;
    let mut impl_blocks = 0usize;
    let mut tests = 0usize;
    let mut public_exports = 0usize;
    let mut cli_dispatch_hints = 0usize;
    let mut report_printer_hints = 0usize;
    let mut hash = Sha256::new();
    for scan in &scans {
        hash.update(scan.path.to_string_lossy().as_bytes());
        hash.update([0]);
        modules += scan.modules;
        functions += scan.functions;
        structs += scan.structs;
        enums += scan.enums;
        impl_blocks += scan.impl_blocks;
        tests += scan.tests;
        public_exports += scan.public_exports;
        cli_dispatch_hints += scan.cli_dispatch_hints;
        report_printer_hints += scan.report_printer_hints;
        for fact in &scan.facts {
            hash.update(fact.route.as_bytes());
            hash.update(fact.subject.as_bytes());
            hash.update(fact.relation.as_bytes());
            hash.update(fact.object.as_bytes());
            facts.push(fact.clone());
        }
    }
    let route_counts = route_counts(&facts);
    let routes = route_counts
        .iter()
        .map(|(route, fact_count)| RustRouteSummary {
            route,
            fact_count: *fact_count,
        })
        .collect::<Vec<_>>();
    let max_route_facts = route_counts.values().copied().max().unwrap_or(0);
    let min_route_facts = route_counts.values().copied().min().unwrap_or(0);
    let route_balance_ratio = if min_route_facts == 0 {
        0.0
    } else {
        round4(max_route_facts as f64 / min_route_facts as f64)
    };
    let route_balanced = !route_counts.is_empty() && route_balance_ratio <= 12.0;
    let artifact = RustCorpusArtifact {
        corpus_kind: "rust-code-structural-corpus",
        corpus_hash: format!("{:x}", hash.finalize()),
        rust_files: scans.len(),
        modules,
        functions,
        structs,
        enums,
        impl_blocks,
        tests,
        public_exports,
        cli_dispatch_hints,
        report_printer_hints,
        route_count: route_counts.len(),
        fact_count: facts.len(),
    };
    let metrics = RustCorpusMetrics {
        route_balance_ratio,
        max_route_facts,
        min_route_facts,
        owner_route_count: route_counts.get("module-owner").copied().unwrap_or(0),
        proof_route_count: route_counts.get("unit-test-proof").copied().unwrap_or(0)
            + route_counts
                .get("integration-test-proof")
                .copied()
                .unwrap_or(0),
        route_balanced,
        heldout_suite_ready: false,
        final_proof_gate_passed: false,
    };
    let output = write_artifact_if_requested(&config.out, &artifact, &metrics, &routes, &facts)?;
    let verdict = if artifact.rust_files > 0
        && artifact.functions > 0
        && artifact.route_count >= 4
        && artifact.fact_count > artifact.rust_files
    {
        "RUST_CORPUS_ARTIFACT_READY"
    } else {
        "RUST_CORPUS_ARTIFACT_REVIEW"
    };

    Ok(RustCorpusBuildReport {
        mode: "llmwave-big-rust-corpus-build",
        version: RUST_CORPUS_VERSION,
        verdict,
        profile: "rust",
        repo: repo.display().to_string(),
        artifact,
        metrics,
        routes,
        facts: facts.iter().take(512).cloned().collect(),
        output,
        claim_boundary: RustCorpusClaimBoundary {
            rust_corpus_loaded: verdict == "RUST_CORPUS_ARTIFACT_READY",
            heldout_suite_ready: false,
            focus_packet_ready: false,
            final_proof_gate_passed: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Rust structural corpus artifact is built from source routes, but held-out and focus gates are still required before final proof claims.",
            blocked_by: vec![
                "rust_api_owner_heldout_suite_missing",
                "rust_route_balanced_focus_packet_missing",
                "compile_test_evidence_bridge_missing",
            ],
        },
    })
}

fn write_artifact_if_requested(
    out: &Option<PathBuf>,
    artifact: &RustCorpusArtifact,
    metrics: &RustCorpusMetrics,
    routes: &[RustRouteSummary],
    facts: &[RustStructuralFact],
) -> Result<RustCorpusOutput> {
    let Some(path) = out else {
        return Ok(RustCorpusOutput {
            artifact_written: false,
            artifact_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create artifact directory {}", parent.display()))?;
    }
    let mut file = fs::File::create(path)
        .with_context(|| format!("create Rust corpus artifact {}", path.display()))?;
    let payload = serde_json::json!({
        "artifact": artifact,
        "metrics": metrics,
        "routes": routes,
        "facts": facts,
    });
    serde_json::to_writer_pretty(&mut file, &payload)
        .with_context(|| format!("write Rust corpus artifact {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish Rust corpus artifact {}", path.display()))?;
    Ok(RustCorpusOutput {
        artifact_written: true,
        artifact_path: Some(path.display().to_string()),
    })
}

fn collect_rust_files(repo: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_rust_files_inner(repo, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_rust_files_inner(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("read directory {}", path.display()))? {
        let entry = entry.with_context(|| format!("read directory entry {}", path.display()))?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name == ".git" || name == "target" || name == ".nanda" {
            continue;
        }
        let file_type = entry
            .file_type()
            .with_context(|| format!("read file type {}", entry_path.display()))?;
        if file_type.is_dir() {
            collect_rust_files_inner(&entry_path, files)?;
        } else if entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn scan_rust_file(repo: &Path, path: &Path) -> Result<RustFileScan> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("read Rust file {}", path.display()))?;
    let rel = path.strip_prefix(repo).unwrap_or(path);
    let evidence_path = rel.display().to_string();
    let module = module_name(rel);
    let mut scan = RustFileScan {
        path: path.to_path_buf(),
        module: module.clone(),
        facts: Vec::new(),
        modules: 1,
        functions: 0,
        structs: 0,
        enums: 0,
        impl_blocks: 0,
        tests: 0,
        public_exports: 0,
        cli_dispatch_hints: 0,
        report_printer_hints: 0,
    };
    let mut next_fn_is_test = false;
    for (line_index, line) in raw.lines().enumerate() {
        let line_no = line_index + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("#[test]") || trimmed.starts_with("#[tokio::test]") {
            next_fn_is_test = true;
            continue;
        }
        if let Some(name) = mod_name(trimmed) {
            scan.facts.push(fact(
                "module-owner",
                &scan.module,
                "declares module",
                &name,
                &evidence_path,
                line_no,
            ));
        }
        if let Some(name) = item_name(trimmed, "struct") {
            scan.structs += 1;
            if is_public(trimmed) {
                scan.public_exports += 1;
                scan.facts.push(fact(
                    "public-api-export",
                    &scan.module,
                    "exports struct",
                    &name,
                    &evidence_path,
                    line_no,
                ));
            }
        }
        if let Some(name) = item_name(trimmed, "enum") {
            scan.enums += 1;
            if is_public(trimmed) {
                scan.public_exports += 1;
                scan.facts.push(fact(
                    "public-api-export",
                    &scan.module,
                    "exports enum",
                    &name,
                    &evidence_path,
                    line_no,
                ));
            }
        }
        if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
            scan.impl_blocks += 1;
            scan.facts.push(fact(
                "module-owner",
                &scan.module,
                "defines impl block",
                trimmed,
                &evidence_path,
                line_no,
            ));
        }
        if let Some(name) = function_name(trimmed) {
            scan.functions += 1;
            scan.facts.push(fact(
                "module-owner",
                &scan.module,
                "owns function",
                &name,
                &evidence_path,
                line_no,
            ));
            if is_public(trimmed) {
                scan.public_exports += 1;
                scan.facts.push(fact(
                    "public-api-export",
                    &scan.module,
                    "exports function",
                    &name,
                    &evidence_path,
                    line_no,
                ));
            }
            if next_fn_is_test || name.starts_with("test_") || name.contains("_test") {
                scan.tests += 1;
                scan.facts.push(fact(
                    "unit-test-proof",
                    &name,
                    "verifies module",
                    &scan.module,
                    &evidence_path,
                    line_no,
                ));
            }
            if name.starts_with("print_") || name.contains("_report") {
                scan.report_printer_hints += 1;
                scan.facts.push(fact(
                    "report-printer",
                    &scan.module,
                    "prints or builds report",
                    &name,
                    &evidence_path,
                    line_no,
                ));
            }
            next_fn_is_test = false;
        }
        if trimmed.contains("LlmwaveBigCommand::") || trimmed.contains("#[command(") {
            scan.cli_dispatch_hints += 1;
            scan.facts.push(fact(
                "cli-command-dispatch",
                &scan.module,
                "contains cli dispatch hint",
                trimmed,
                &evidence_path,
                line_no,
            ));
        }
        if evidence_path.contains("/tests/") || evidence_path.starts_with("tests/") {
            scan.facts.push(fact(
                "integration-test-proof",
                &scan.module,
                "belongs to integration test file",
                &evidence_path,
                &evidence_path,
                line_no,
            ));
        }
    }
    Ok(scan)
}

fn route_counts(facts: &[RustStructuralFact]) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for fact in facts {
        *counts.entry(fact.route).or_insert(0) += 1;
    }
    counts
}

fn module_name(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        let part = component.as_os_str().to_string_lossy();
        if part == "src" || part == "tests" {
            continue;
        }
        let clean = part
            .trim_end_matches(".rs")
            .trim_end_matches("/mod")
            .replace('-', "_");
        if clean != "mod" && !clean.is_empty() {
            parts.push(clean);
        }
    }
    if parts.is_empty() {
        "crate".to_string()
    } else {
        parts.join("::")
    }
}

fn mod_name(trimmed: &str) -> Option<String> {
    if !(trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")) {
        return None;
    }
    trimmed
        .split_whitespace()
        .last()
        .map(clean_ident)
        .filter(|name| !name.is_empty())
}

fn item_name(trimmed: &str, keyword: &str) -> Option<String> {
    let tokens = trimmed.split_whitespace().collect::<Vec<_>>();
    tokens
        .windows(2)
        .find(|pair| pair[0] == keyword)
        .map(|pair| clean_ident(pair[1]))
        .filter(|name| !name.is_empty())
}

fn function_name(trimmed: &str) -> Option<String> {
    let pos = trimmed.find("fn ")?;
    let rest = &trimmed[pos + 3..];
    let name = rest
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .next()
        .unwrap_or_default();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn clean_ident(raw: &str) -> String {
    raw.trim_matches(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .to_string()
}

fn is_public(trimmed: &str) -> bool {
    trimmed.starts_with("pub ")
        || trimmed.starts_with("pub(crate)")
        || trimmed.starts_with("pub(super)")
        || trimmed.starts_with("pub(in ")
}

fn fact(
    route: &'static str,
    subject: &str,
    relation: &'static str,
    object: &str,
    evidence_path: &str,
    evidence_line: usize,
) -> RustStructuralFact {
    RustStructuralFact {
        route,
        subject: subject.to_string(),
        relation,
        object: object.to_string(),
        evidence_path: evidence_path.to_string(),
        evidence_line,
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_corpus_builds_current_repo_artifact() {
        let report = build_rust_corpus_report(RustCorpusBuildConfig {
            repo: PathBuf::from("."),
            out: None,
            max_file_bytes: 4 * 1024 * 1024,
        })
        .expect("rust corpus builds");

        assert_eq!(report.verdict, "RUST_CORPUS_ARTIFACT_READY");
        assert_eq!(report.profile, "rust");
        assert!(report.artifact.rust_files > 10);
        assert!(report.artifact.functions > 100);
        assert!(report.artifact.fact_count > report.artifact.rust_files);
        assert!(report
            .routes
            .iter()
            .any(|route| route.route == "module-owner"));
    }

    #[test]
    fn rust_corpus_keeps_final_claims_closed() {
        let report = build_rust_corpus_report(RustCorpusBuildConfig {
            repo: PathBuf::from("."),
            out: None,
            max_file_bytes: 4 * 1024 * 1024,
        })
        .expect("rust corpus builds");

        assert!(report.claim_boundary.rust_corpus_loaded);
        assert!(!report.claim_boundary.heldout_suite_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
    }
}
