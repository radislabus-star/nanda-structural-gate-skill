//! Rust compile/test evidence bridge for LLMWave-Big proof gates.
//!
//! This is cold proof-prep code. It turns saved command evidence into a
//! machine-readable bridge linked to a Rust focus packet; it does not run cargo
//! as a hidden side effect and does not prove nonlinear memory or LLM readiness.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const RUST_COMPILE_EVIDENCE_VERSION: &str =
    "llmwave-big-v-next-rust-compile-evidence-build";

#[derive(Clone)]
pub(crate) struct RustCompileEvidenceBuildConfig {
    pub focus_packet: PathBuf,
    pub check_evidence: PathBuf,
    pub test_evidence: PathBuf,
    pub clippy_evidence: PathBuf,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustCompileEvidenceBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub profile: &'static str,
    pub input_focus_packet: String,
    pub evidence: RustCompileEvidenceSummary,
    pub command_evidence: Vec<RustCommandEvidenceSummary>,
    pub route_evidence: Vec<RustRouteEvidence>,
    pub output: RustCompileEvidenceOutput,
    pub claim_boundary: RustCompileEvidenceClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustCompileEvidenceSummary {
    pub evidence_kind: &'static str,
    pub evidence_hash: String,
    pub focus_packet_hash: String,
    pub focus_packet_ready: bool,
    pub selected_fact_count: usize,
    pub unit_test_proof_facts: usize,
    pub commands_required: usize,
    pub commands_passed: usize,
    pub compile_test_evidence_bridge_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustCommandEvidenceSummary {
    pub role: &'static str,
    pub command: String,
    pub exit_code: i32,
    pub passed: bool,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustRouteEvidence {
    pub route: &'static str,
    pub evidence_role: &'static str,
    pub passed: bool,
    pub reason: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustCompileEvidenceOutput {
    pub evidence_written: bool,
    pub evidence_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct RustCompileEvidenceClaimBoundary {
    pub rust_corpus_loaded: bool,
    pub heldout_suite_ready: bool,
    pub focus_packet_ready: bool,
    pub compile_test_evidence_bridge_ready: bool,
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
    facts: Vec<RustFocusFact>,
}

#[derive(Deserialize)]
struct RustFocusSummary {
    packet_hash: String,
    selected_fact_count: usize,
}

#[derive(Deserialize)]
struct RustFocusMetrics {
    focus_packet_ready: bool,
}

#[derive(Deserialize)]
struct RustFocusFact {
    route: String,
}

#[derive(Deserialize)]
struct RustCommandEvidencePayload {
    command: String,
    exit_code: i32,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
}

pub(crate) fn build_rust_compile_evidence_report(
    config: RustCompileEvidenceBuildConfig,
) -> Result<RustCompileEvidenceBuildReport> {
    let focus_raw = fs::read_to_string(&config.focus_packet)
        .with_context(|| format!("read Rust focus packet {}", config.focus_packet.display()))?;
    let focus: RustFocusPacketPayload = serde_json::from_str(&focus_raw)
        .with_context(|| format!("parse Rust focus packet {}", config.focus_packet.display()))?;

    let check = read_command_evidence("cargo-check", &config.check_evidence)?;
    let test = read_command_evidence("cargo-test", &config.test_evidence)?;
    let clippy = read_command_evidence("cargo-clippy", &config.clippy_evidence)?;
    let command_evidence = vec![check, test, clippy];
    let commands_passed = command_evidence
        .iter()
        .filter(|evidence| evidence.passed)
        .count();
    let unit_test_proof_facts = focus
        .facts
        .iter()
        .filter(|fact| fact.route == "unit-test-proof")
        .count();
    let compile_test_evidence_bridge_ready = focus.metrics.focus_packet_ready
        && commands_passed == command_evidence.len()
        && unit_test_proof_facts > 0;
    let route_evidence = build_route_evidence(
        focus.metrics.focus_packet_ready,
        unit_test_proof_facts,
        &command_evidence,
    );
    let mut hash = Sha256::new();
    hash.update(focus.focus.packet_hash.as_bytes());
    for evidence in &command_evidence {
        hash.update(evidence.role.as_bytes());
        hash.update(evidence.command.as_bytes());
        hash.update(evidence.exit_code.to_le_bytes());
        hash.update(evidence.stdout_bytes.to_le_bytes());
        hash.update(evidence.stderr_bytes.to_le_bytes());
    }
    let evidence = RustCompileEvidenceSummary {
        evidence_kind: "rust-code-compile-test-evidence",
        evidence_hash: format!("{:x}", hash.finalize()),
        focus_packet_hash: focus.focus.packet_hash,
        focus_packet_ready: focus.metrics.focus_packet_ready,
        selected_fact_count: focus.focus.selected_fact_count,
        unit_test_proof_facts,
        commands_required: command_evidence.len(),
        commands_passed,
        compile_test_evidence_bridge_ready,
    };
    let output =
        write_evidence_if_requested(&config.out, &evidence, &command_evidence, &route_evidence)?;
    let verdict = if compile_test_evidence_bridge_ready {
        "RUST_COMPILE_EVIDENCE_READY"
    } else {
        "RUST_COMPILE_EVIDENCE_REVIEW"
    };
    let mut blocked_by = vec!["rust_heldout_inference_eval_missing"];
    if !compile_test_evidence_bridge_ready {
        blocked_by.insert(0, "compile_test_evidence_bridge_incomplete");
    }

    Ok(RustCompileEvidenceBuildReport {
        mode: "llmwave-big-rust-compile-evidence-build",
        version: RUST_COMPILE_EVIDENCE_VERSION,
        verdict,
        profile: "rust",
        input_focus_packet: config.focus_packet.display().to_string(),
        evidence,
        command_evidence,
        route_evidence,
        output,
        claim_boundary: RustCompileEvidenceClaimBoundary {
            rust_corpus_loaded: true,
            heldout_suite_ready: true,
            focus_packet_ready: focus.metrics.focus_packet_ready,
            compile_test_evidence_bridge_ready,
            heldout_inference_eval_ready: false,
            final_proof_gate_passed: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Rust compile/test evidence is linked to the focus packet, but held-out inference eval is still required before final proof claims.",
            blocked_by,
        },
    })
}

fn read_command_evidence(role: &'static str, path: &PathBuf) -> Result<RustCommandEvidenceSummary> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read command evidence {}", path.display()))?;
    let payload: RustCommandEvidencePayload = serde_json::from_str(&raw)
        .with_context(|| format!("parse command evidence {}", path.display()))?;
    Ok(RustCommandEvidenceSummary {
        role,
        command: payload.command,
        exit_code: payload.exit_code,
        passed: payload.exit_code == 0,
        stdout_bytes: payload.stdout.len(),
        stderr_bytes: payload.stderr.len(),
    })
}

fn build_route_evidence(
    focus_packet_ready: bool,
    unit_test_proof_facts: usize,
    commands: &[RustCommandEvidenceSummary],
) -> Vec<RustRouteEvidence> {
    let command_passed = |role: &str| {
        commands
            .iter()
            .any(|evidence| evidence.role == role && evidence.passed)
    };
    vec![
        RustRouteEvidence {
            route: "module-owner",
            evidence_role: "cargo-check",
            passed: focus_packet_ready && command_passed("cargo-check"),
            reason: "type checking links module ownership facts to compilable Rust source",
        },
        RustRouteEvidence {
            route: "public-api-export",
            evidence_role: "cargo-check",
            passed: focus_packet_ready && command_passed("cargo-check"),
            reason: "public exports are accepted by the Rust compiler",
        },
        RustRouteEvidence {
            route: "unit-test-proof",
            evidence_role: "cargo-test",
            passed: unit_test_proof_facts > 0 && command_passed("cargo-test"),
            reason: "unit-test-proof facts exist in focus and cargo test passed",
        },
        RustRouteEvidence {
            route: "claim-boundary",
            evidence_role: "cargo-clippy",
            passed: command_passed("cargo-clippy"),
            reason: "clippy warning gate passed for claim-boundary code",
        },
    ]
}

fn write_evidence_if_requested(
    out: &Option<PathBuf>,
    evidence: &RustCompileEvidenceSummary,
    command_evidence: &[RustCommandEvidenceSummary],
    route_evidence: &[RustRouteEvidence],
) -> Result<RustCompileEvidenceOutput> {
    let Some(path) = out else {
        return Ok(RustCompileEvidenceOutput {
            evidence_written: false,
            evidence_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "create Rust compile evidence directory {}",
                parent.display()
            )
        })?;
    }
    let mut file = fs::File::create(path)
        .with_context(|| format!("create Rust compile evidence {}", path.display()))?;
    let payload = serde_json::json!({
        "evidence": evidence,
        "command_evidence": command_evidence,
        "route_evidence": route_evidence,
    });
    serde_json::to_writer_pretty(&mut file, &payload)
        .with_context(|| format!("write Rust compile evidence {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish Rust compile evidence {}", path.display()))?;
    Ok(RustCompileEvidenceOutput {
        evidence_written: true,
        evidence_path: Some(path.display().to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llmwave_big::rust_corpus::{build_rust_corpus_report, RustCorpusBuildConfig};
    use crate::llmwave_big::rust_focus::{build_rust_focus_report, RustFocusBuildConfig};
    use crate::llmwave_big::rust_heldout::{build_rust_heldout_report, RustHeldoutBuildConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rust_compile_evidence_builds_bridge_from_saved_commands() {
        let dir = temp_test_dir("rust-compile-evidence-ready");
        let focus_path = build_focus_fixture(&dir);
        let check = write_command(
            &dir,
            "check.json",
            "cargo check --all-targets --all-features",
            0,
        );
        let test = write_command(&dir, "test.json", "scripts/test-local.sh", 0);
        let clippy = write_command(
            &dir,
            "clippy.json",
            "cargo clippy --all-targets --all-features -- -D warnings",
            0,
        );

        let report = build_rust_compile_evidence_report(RustCompileEvidenceBuildConfig {
            focus_packet: focus_path,
            check_evidence: check,
            test_evidence: test,
            clippy_evidence: clippy,
            out: None,
        })
        .expect("compile evidence builds");

        assert_eq!(report.verdict, "RUST_COMPILE_EVIDENCE_READY");
        assert!(report.evidence.compile_test_evidence_bridge_ready);
        assert_eq!(report.evidence.commands_passed, 3);
        assert!(report
            .route_evidence
            .iter()
            .any(|route| route.route == "unit-test-proof" && route.passed));
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn rust_compile_evidence_rejects_failed_command() {
        let dir = temp_test_dir("rust-compile-evidence-failed");
        let focus_path = build_focus_fixture(&dir);
        let check = write_command(
            &dir,
            "check.json",
            "cargo check --all-targets --all-features",
            0,
        );
        let test = write_command(&dir, "test.json", "scripts/test-local.sh", 1);
        let clippy = write_command(
            &dir,
            "clippy.json",
            "cargo clippy --all-targets --all-features -- -D warnings",
            0,
        );

        let report = build_rust_compile_evidence_report(RustCompileEvidenceBuildConfig {
            focus_packet: focus_path,
            check_evidence: check,
            test_evidence: test,
            clippy_evidence: clippy,
            out: None,
        })
        .expect("compile evidence builds");

        assert_eq!(report.verdict, "RUST_COMPILE_EVIDENCE_REVIEW");
        assert!(!report.evidence.compile_test_evidence_bridge_ready);
        assert!(report
            .claim_boundary
            .blocked_by
            .contains(&"compile_test_evidence_bridge_incomplete"));
    }

    fn build_focus_fixture(dir: &std::path::Path) -> PathBuf {
        let artifact_path = dir.join("rust-corpus.json");
        let heldout_path = dir.join("rust-heldout.json");
        let focus_path = dir.join("rust-focus.json");
        build_rust_corpus_report(RustCorpusBuildConfig {
            repo: PathBuf::from("."),
            out: Some(artifact_path.clone()),
            max_file_bytes: 4 * 1024 * 1024,
        })
        .expect("rust corpus builds");
        build_rust_heldout_report(RustHeldoutBuildConfig {
            artifact: artifact_path.clone(),
            out: Some(heldout_path.clone()),
            max_cases: 64,
        })
        .expect("rust heldout builds");
        build_rust_focus_report(RustFocusBuildConfig {
            artifact: artifact_path,
            heldout_suite: heldout_path,
            out: Some(focus_path.clone()),
            max_facts: 15_000,
            route_fact_cap: 256,
        })
        .expect("rust focus builds");
        focus_path
    }

    fn write_command(dir: &std::path::Path, name: &str, command: &str, exit_code: i32) -> PathBuf {
        let path = dir.join(name);
        let payload = serde_json::json!({
            "command": command,
            "exit_code": exit_code,
            "stdout": "ok\n",
            "stderr": "",
        });
        fs::write(&path, serde_json::to_string_pretty(&payload).expect("json"))
            .expect("write command evidence");
        path
    }

    fn temp_test_dir(slug: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nanda-{slug}-{stamp}"));
        fs::create_dir_all(&dir).expect("create temp test dir");
        dir
    }
}
