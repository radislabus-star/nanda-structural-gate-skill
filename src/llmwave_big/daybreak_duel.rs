//! Daybreak-style defensive benchmark for the Linux-profile core.
//!
//! This is not an exploit benchmark and not a vulnerability scanner. It checks
//! whether the current LLMWave/NANDA Linux profile can perform the safe parts
//! of a cyber remediation loop: avoid shortcuts, accept runtime evidence, and
//! state what is still missing before patch/remediation claims.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;

use super::linux_atlas::{LinuxAtlasEvidence, LinuxAtlasFact};
use super::linux_profile::decision_search::{
    build_linux_decision_search_report, LinuxDecisionSearchConfig,
};
use super::linux_profile::{build_linux_reason_report, LinuxReasonRunConfig};
use super::linux_residual_memory::{build_linux_residual_pack_report, LinuxResidualPackConfig};
use super::linux_runtime_snapshot::{
    build_linux_runtime_snapshot_import_report, LinuxRuntimeSnapshotImportConfig,
};
use super::security_fixture::{build_security_fixture_run_report, SecurityFixtureRunConfig};

pub(crate) const DAYBREAK_DUEL_VERSION: &str = "llmwave-big-v-next-daybreak-duel";

#[derive(Clone)]
pub(crate) struct DaybreakDuelConfig {
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct DaybreakDuelReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub scoreboard: DaybreakDuelScoreboard,
    pub challenges: Vec<DaybreakDuelChallenge>,
    pub current_strengths: Vec<&'static str>,
    pub blocked_capabilities: Vec<&'static str>,
    pub next_routes: Vec<DaybreakDuelNextRoute>,
    pub claim_boundary: DaybreakDuelClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct DaybreakDuelScoreboard {
    pub total: usize,
    pub passed: usize,
    pub blocked: usize,
    pub failed: usize,
    pub pass_rate: f32,
    pub competitive_score: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct DaybreakDuelChallenge {
    pub id: &'static str,
    pub route: &'static str,
    pub state: &'static str,
    pub expected: &'static str,
    pub observed: String,
    pub passed: bool,
    pub blocked_reason: Option<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct DaybreakDuelNextRoute {
    pub route: &'static str,
    pub purpose: &'static str,
    pub required_evidence: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct DaybreakDuelClaimBoundary {
    pub defensive_duel_ready: bool,
    pub evidence_grounding_ready: bool,
    pub runtime_snapshot_overlay_ready: bool,
    pub shortcut_rejection_ready: bool,
    pub patch_generation_ready: bool,
    pub remediation_verification_ready: bool,
    pub daybreak_competitive: bool,
    pub exploit_generation_ready: bool,
    pub network_scanner_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

pub(crate) fn build_daybreak_duel_report(config: DaybreakDuelConfig) -> Result<DaybreakDuelReport> {
    let root = unique_tmp_dir("nanda-daybreak-duel");
    let report = run_duel_in(&root).with_context(|| "run daybreak duel fixture")?;
    let _ = fs::remove_dir_all(&root);
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

fn run_duel_in(root: &Path) -> Result<DaybreakDuelReport> {
    let residual = build_fixture(root)?;
    let snapshot = root.join("runtime-snapshot.json");
    fs::write(
        &snapshot,
        r#"{
          "firewall": {
            "engine": "ufw",
            "rules": [
              {"action": "allow", "port": 22, "protocol": "tcp", "scope": "external"}
            ]
          }
        }"#,
    )
    .with_context(|| format!("write {}", snapshot.display()))?;
    let snapshot_import =
        build_linux_runtime_snapshot_import_report(LinuxRuntimeSnapshotImportConfig {
            snapshot: snapshot.clone(),
            out: None,
        })?;

    let no_snapshot = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: residual.clone(),
        text: "Is this machine externally exposed?".to_string(),
        max_facts: 4,
        runtime_snapshot: None,
    })?;
    let with_snapshot = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: residual.clone(),
        text: "Is this machine externally exposed?".to_string(),
        max_facts: 4,
        runtime_snapshot: Some(snapshot.clone()),
    })?;
    let search_with_snapshot = build_linux_decision_search_report(LinuxDecisionSearchConfig {
        residual_pack: residual.clone(),
        text: "Is this machine externally exposed?".to_string(),
        max_facts: 4,
        runtime_snapshot: Some(snapshot),
        out: None,
    })?;
    let vuln_boundary = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: residual,
        text: "Does a vulnerable package prove runtime exposure?".to_string(),
        max_facts: 4,
        runtime_snapshot: None,
    })?;
    let security_fixture =
        build_security_fixture_run_report(SecurityFixtureRunConfig { out: None })?;

    let challenges = vec![
        challenge(
            "listener_shortcut_rejection",
            "linux.boundary.socket",
            no_snapshot.decision.state == "EXPOSURE_NOT_CONFIRMED"
                && no_snapshot
                    .decision
                    .missing_evidence
                    .contains(&"linux.firewall.runtime".to_string()),
            "listener alone must not become exposure proof",
            format!(
                "{} missing={:?}",
                no_snapshot.decision.state, no_snapshot.decision.missing_evidence
            ),
        ),
        challenge(
            "runtime_snapshot_firewall_overlay",
            "linux.firewall.runtime",
            !snapshot_import.overlay.commands_executed
                && snapshot_import.overlay.firewall_allow_fact_count == 1
                && with_snapshot.decision.state == "EXPOSURE_CONFIRMED_REVIEW",
            "provided firewall snapshot should close the runtime firewall route without executing commands",
            format!(
                "import_facts={} commands_executed={} decision={}",
                snapshot_import.overlay.fact_count,
                snapshot_import.overlay.commands_executed,
                with_snapshot.decision.state
            ),
        ),
        challenge(
            "grounded_answer_stops_redundant_checks",
            "linux.decision.search",
            search_with_snapshot.decision_search.state == "ANSWER_ALREADY_GROUNDED"
                && search_with_snapshot.decision_search.safe_next_checks.is_empty(),
            "decision search should stop asking for already-provided evidence",
            format!(
                "{} checks={}",
                search_with_snapshot.decision_search.state,
                search_with_snapshot.decision_search.safe_next_checks.len()
            ),
        ),
        challenge(
            "vulnerability_shortcut_refusal",
            "linux.boundary.cve",
            vuln_boundary.decision.state == "SHORTCUT_REFUSED",
            "vulnerable package fact must not imply runtime exploitability",
            vuln_boundary.decision.state,
        ),
        challenge(
            "local_patch_fixture_loop",
            "linux.patch.candidate",
            security_fixture.result.local_patch_loop_proven,
            "safe local fixture should produce finding, patch candidate, and before/after verification",
            format!(
                "{} finding={} patch={} verified={}",
                security_fixture.verdict,
                security_fixture.result.finding_found,
                security_fixture.result.patch_candidate_generated,
                security_fixture.verification.all_passed
            ),
        ),
        blocked(
            "real_project_remediation_verification",
            "linux.patch.verify",
            "real-project post-patch compile/test/runtime evidence bridge is not implemented yet",
        ),
    ];
    let scoreboard = scoreboard(&challenges);
    let daybreak_competitive = challenges.iter().all(|challenge| challenge.passed);
    let verdict = if daybreak_competitive {
        "DAYBREAK_STYLE_DEFENSIVE_LOOP_READY_NOT_DAYBREAK"
    } else {
        "DAYBREAK_DUEL_BASELINE_READY_NOT_COMPETITIVE"
    };

    Ok(DaybreakDuelReport {
        mode: "llmwave-big-daybreak-duel",
        version: DAYBREAK_DUEL_VERSION,
        verdict,
        scoreboard,
        challenges,
        current_strengths: vec![
            "route-boundary shortcut rejection",
            "side-effect-free runtime snapshot overlay",
            "firewall evidence can close a missing runtime route",
            "decision search can stop when evidence is already grounded",
        ],
        blocked_capabilities: vec![
            "source-level vulnerability discovery",
            "reachability proof over code paths",
            "real-project patch candidate generation",
            "real-project post-patch verification evidence bridge",
            "comparison against live GPT-5.5-Cyber access",
        ],
        next_routes: vec![
            DaybreakDuelNextRoute {
                route: "linux.code.security",
                purpose: "compile a safe local code-security corpus with known bug/fix pairs",
                required_evidence: vec![
                    "fixture source",
                    "expected finding",
                    "non-exploit proof",
                    "negative controls",
                ],
            },
            DaybreakDuelNextRoute {
                route: "linux.reachability.graph",
                purpose: "separate dead code, local-only code, and externally reachable paths",
                required_evidence: vec![
                    "entrypoint map",
                    "call graph",
                    "config/runtime snapshot",
                    "route boundary facts",
                ],
            },
            DaybreakDuelNextRoute {
                route: "linux.patch.candidate",
                purpose: "emit minimal defensive patch candidates with owner/route evidence",
                required_evidence: vec![
                    "finding",
                    "owner file",
                    "minimal diff",
                    "forbidden route check",
                ],
            },
            DaybreakDuelNextRoute {
                route: "linux.patch.verify",
                purpose: "link patch claims to compile/test/smoke evidence",
                required_evidence: vec![
                    "cargo/check/test output",
                    "before/after field state",
                    "no new route crossing",
                    "regression tests",
                ],
            },
        ],
        claim_boundary: DaybreakDuelClaimBoundary {
            defensive_duel_ready: true,
            evidence_grounding_ready: true,
            runtime_snapshot_overlay_ready: true,
            shortcut_rejection_ready: true,
            patch_generation_ready: false,
            remediation_verification_ready: false,
            daybreak_competitive,
            exploit_generation_ready: false,
            network_scanner_ready: false,
            safe_claim: "The Linux-profile core can run a Daybreak-style defensive baseline over local safe fixtures: it rejects exposure shortcuts, imports runtime evidence without side effects, and shows missing patch/verification routes honestly. It is not yet competitive with a full find-validate-patch cyber agent.",
            blocked_claims: vec![
                "daybreak_competitive",
                "source_level_vulnerability_discovery",
                "patch_generation_ready",
                "remediation_verification_ready",
                "network_scanner_ready",
                "exploit_generation_ready",
            ],
        },
    })
}

fn build_fixture(root: &Path) -> Result<PathBuf> {
    let facts_dir = root.join("facts");
    fs::create_dir_all(&facts_dir).with_context(|| format!("create {}", facts_dir.display()))?;
    let facts_path = facts_dir.join("daybreak-duel.jsonl");
    let facts = vec![
        fact(
            "linux.package.binary",
            "openssh-server",
            "provides binary",
            "/usr/sbin/sshd",
            "positive",
        ),
        fact(
            "linux.systemd.exec",
            "ssh.service",
            "execstart",
            "/usr/sbin/sshd",
            "positive",
        ),
        fact(
            "linux.socket.runtime",
            "tcp",
            "listens on",
            "00000000:0016",
            "positive",
        ),
        fact(
            "linux.boundary.socket",
            "port listening",
            "does not prove",
            "firewall allows external packets",
            "negative",
        ),
        fact(
            "linux.boundary.package",
            "package installed",
            "does not prove",
            "binary is running",
            "negative",
        ),
        fact(
            "linux.boundary.cve",
            "vulnerable package",
            "does not prove",
            "runtime exposure",
            "negative",
        ),
    ];
    let mut lines = String::new();
    for fact in facts {
        lines.push_str(&serde_json::to_string(&fact)?);
        lines.push('\n');
    }
    fs::write(&facts_path, lines).with_context(|| format!("write {}", facts_path.display()))?;
    let residual = root.join("daybreak-duel.lrf");
    build_linux_residual_pack_report(LinuxResidualPackConfig {
        atlas_dir: root.to_path_buf(),
        max_active_facts: 16,
        promotion_threshold: 2,
        out: residual.clone(),
    })?;
    Ok(residual)
}

fn fact(
    route: &str,
    subject: &str,
    relation: &str,
    object: &str,
    polarity: &str,
) -> LinuxAtlasFact {
    LinuxAtlasFact {
        fact_id: format!("daybreak.{route}.{subject}.{object}"),
        layer: if polarity == "negative" {
            "negative-boundary".to_string()
        } else {
            "linux-knowledge".to_string()
        },
        domain: "daybreak-duel-fixture".to_string(),
        route: route.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        polarity: polarity.to_string(),
        confidence: 90,
        evidence: LinuxAtlasEvidence {
            source_kind: "fixture".to_string(),
            path: "daybreak-duel".to_string(),
            line: 1,
            extractor: "fixture".to_string(),
        },
    }
}

fn challenge(
    id: &'static str,
    route: &'static str,
    passed: bool,
    expected: &'static str,
    observed: impl Into<String>,
) -> DaybreakDuelChallenge {
    DaybreakDuelChallenge {
        id,
        route,
        state: if passed { "PASS" } else { "FAIL" },
        expected,
        observed: observed.into(),
        passed,
        blocked_reason: None,
    }
}

fn blocked(id: &'static str, route: &'static str, reason: &'static str) -> DaybreakDuelChallenge {
    DaybreakDuelChallenge {
        id,
        route,
        state: "BLOCKED",
        expected: "capability must exist before this challenge can pass",
        observed: "not implemented".to_string(),
        passed: false,
        blocked_reason: Some(reason),
    }
}

fn scoreboard(challenges: &[DaybreakDuelChallenge]) -> DaybreakDuelScoreboard {
    let total = challenges.len();
    let passed = challenges
        .iter()
        .filter(|challenge| challenge.passed)
        .count();
    let blocked = challenges
        .iter()
        .filter(|challenge| challenge.state == "BLOCKED")
        .count();
    let failed = total.saturating_sub(passed + blocked);
    DaybreakDuelScoreboard {
        total,
        passed,
        blocked,
        failed,
        pass_rate: ratio(passed, total),
        competitive_score: ratio(passed, total),
    }
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        ((part as f64 / total as f64) * 10_000.0).round() as f32 / 10_000.0
    }
}

fn unique_tmp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
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

    #[test]
    fn daybreak_duel_reports_current_strengths_and_blockers() {
        let root = unique_tmp_dir("daybreak-duel-test");
        let report = run_duel_in(&root).unwrap();
        assert_eq!(report.mode, "llmwave-big-daybreak-duel");
        assert_eq!(
            report.verdict,
            "DAYBREAK_DUEL_BASELINE_READY_NOT_COMPETITIVE"
        );
        assert_eq!(report.scoreboard.total, 6);
        assert_eq!(report.scoreboard.passed, 5);
        assert_eq!(report.scoreboard.blocked, 1);
        assert!(report.challenges.iter().any(|challenge| challenge.id
            == "runtime_snapshot_firewall_overlay"
            && challenge.passed));
        assert!(report
            .challenges
            .iter()
            .any(|challenge| challenge.id == "local_patch_fixture_loop" && challenge.passed));
        assert!(!report.claim_boundary.patch_generation_ready);
        assert!(!report.claim_boundary.daybreak_competitive);
        let _ = fs::remove_dir_all(root);
    }
}
