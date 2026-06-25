//! Safe local security fixture: find -> patch -> verify.
//!
//! This module intentionally uses a tiny toy path traversal fixture under a
//! temporary directory. It does not scan the host, exploit a real program, or
//! touch network state. The point is to make the core produce a concrete
//! defensive result: a finding, a minimal patch candidate, and before/after
//! behavior evidence.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;

pub(crate) const SECURITY_FIXTURE_VERSION: &str = "llmwave-big-v-next-security-fixture-run";

const VULNERABLE_SOURCE: &str = r#"use std::fs;
use std::io;
use std::path::Path;

pub fn read_user_file(base_dir: &Path, user_path: &str) -> io::Result<String> {
    let candidate = base_dir.join(user_path);
    fs::read_to_string(candidate)
}
"#;

const PATCHED_SOURCE: &str = r#"use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

pub fn read_user_file(base_dir: &Path, user_path: &str) -> io::Result<String> {
    let base = base_dir.canonicalize()?;
    let candidate = base_dir.join(user_path).canonicalize()?;
    if !candidate.starts_with(&base) {
        return Err(io::Error::new(ErrorKind::PermissionDenied, "path escapes base_dir"));
    }
    fs::read_to_string(candidate)
}
"#;

#[derive(Clone)]
pub(crate) struct SecurityFixtureRunConfig {
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityFixtureRunReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub result: SecurityFixtureResult,
    pub finding: SecurityFinding,
    pub patch_candidate: SecurityPatchCandidate,
    pub verification: SecurityVerification,
    pub claim_boundary: SecurityFixtureClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityFixtureResult {
    pub finding_found: bool,
    pub patch_candidate_generated: bool,
    pub before_exploit_reaches_forbidden_path: bool,
    pub after_forbidden_path_blocked: bool,
    pub regression_normal_file_still_reads: bool,
    pub local_patch_loop_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityFinding {
    pub finding_id: &'static str,
    pub class: &'static str,
    pub severity: &'static str,
    pub vulnerable_route: &'static str,
    pub owner: &'static str,
    pub evidence: Vec<SecurityEvidence>,
    pub route_field: SecurityRouteField,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityEvidence {
    pub role: &'static str,
    pub fact: &'static str,
    pub source_span: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityRouteField {
    pub state: &'static str,
    pub route_peak: &'static str,
    pub cause_chain: Vec<&'static str>,
    pub forbidden_shortcuts: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityPatchCandidate {
    pub patch_id: &'static str,
    pub file: &'static str,
    pub owner: &'static str,
    pub changed_function: &'static str,
    pub diff: &'static str,
    pub patched_source_preview: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityVerification {
    pub before: SecurityVerificationCase,
    pub after: SecurityVerificationCase,
    pub regression: SecurityVerificationCase,
    pub all_passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityVerificationCase {
    pub case_id: &'static str,
    pub input: &'static str,
    pub expected: &'static str,
    pub observed: String,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct SecurityFixtureClaimBoundary {
    pub local_security_fixture_ready: bool,
    pub finding_ready: bool,
    pub patch_candidate_ready: bool,
    pub verification_ready: bool,
    pub real_project_scanner_ready: bool,
    pub exploit_generation_ready: bool,
    pub broad_daybreak_competitive: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

pub(crate) fn build_security_fixture_run_report(
    config: SecurityFixtureRunConfig,
) -> Result<SecurityFixtureRunReport> {
    let root = unique_tmp_dir("nanda-security-fixture");
    let report = run_fixture_in(&root).with_context(|| "run security fixture")?;
    let _ = fs::remove_dir_all(&root);
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

fn run_fixture_in(root: &Path) -> Result<SecurityFixtureRunReport> {
    let layout = FixtureLayout::create(root)?;
    let finding_found = detect_path_traversal(VULNERABLE_SOURCE);
    let before_read = vulnerable_read_user_file(&layout.base_dir, "../secret.txt");
    let after_escape = patched_read_user_file(&layout.base_dir, "../secret.txt");
    let after_normal = patched_read_user_file(&layout.base_dir, "public.txt");

    let before = SecurityVerificationCase {
        case_id: "before_path_escape_reaches_secret",
        input: "../secret.txt",
        expected: "vulnerable version reads the forbidden file",
        observed: observation(&before_read),
        passed: before_read
            .as_ref()
            .map(|value| value == "outside-secret")
            .unwrap_or(false),
    };
    let after = SecurityVerificationCase {
        case_id: "after_path_escape_is_blocked",
        input: "../secret.txt",
        expected: "patched version blocks traversal outside base_dir",
        observed: observation(&after_escape),
        passed: after_escape
            .as_ref()
            .err()
            .map(|error| error.kind() == ErrorKind::PermissionDenied)
            .unwrap_or(false),
    };
    let regression = SecurityVerificationCase {
        case_id: "after_normal_file_still_reads",
        input: "public.txt",
        expected: "patched version still reads an allowed file",
        observed: observation(&after_normal),
        passed: after_normal
            .as_ref()
            .map(|value| value == "inside-public")
            .unwrap_or(false),
    };
    let all_passed = before.passed && after.passed && regression.passed;
    let local_patch_loop_proven = finding_found && all_passed;
    let result = SecurityFixtureResult {
        finding_found,
        patch_candidate_generated: finding_found,
        before_exploit_reaches_forbidden_path: before.passed,
        after_forbidden_path_blocked: after.passed,
        regression_normal_file_still_reads: regression.passed,
        local_patch_loop_proven,
    };

    Ok(SecurityFixtureRunReport {
        mode: "llmwave-big-security-fixture-run",
        version: SECURITY_FIXTURE_VERSION,
        verdict: if local_patch_loop_proven {
            "DEFENSIVE_PATCH_PROVEN_LOCAL_FIXTURE"
        } else {
            "DEFENSIVE_PATCH_FIXTURE_REVIEW"
        },
        result,
        finding: finding(),
        patch_candidate: patch_candidate(),
        verification: SecurityVerification {
            before,
            after,
            regression,
            all_passed,
        },
        claim_boundary: SecurityFixtureClaimBoundary {
            local_security_fixture_ready: local_patch_loop_proven,
            finding_ready: finding_found,
            patch_candidate_ready: finding_found,
            verification_ready: all_passed,
            real_project_scanner_ready: false,
            exploit_generation_ready: false,
            broad_daybreak_competitive: false,
            safe_claim: "NANDA can complete a safe local defensive fixture loop: identify a path traversal route, emit a minimal patch candidate, and verify before/after behavior in a temporary fixture. This is not a real-project scanner or exploit generator.",
            blocked_claims: vec![
                "real_project_scanner_ready",
                "exploit_generation_ready",
                "broad_daybreak_competitive",
                "untrusted_code_patch_autonomy",
            ],
        },
    })
}

fn detect_path_traversal(source: &str) -> bool {
    source.contains("base_dir.join(user_path)")
        && source.contains("fs::read_to_string(candidate)")
        && !source.contains("canonicalize")
        && !source.contains("starts_with")
}

fn vulnerable_read_user_file(base_dir: &Path, user_path: &str) -> io::Result<String> {
    let candidate = base_dir.join(user_path);
    fs::read_to_string(candidate)
}

fn patched_read_user_file(base_dir: &Path, user_path: &str) -> io::Result<String> {
    let base = base_dir.canonicalize()?;
    let candidate = base_dir.join(user_path).canonicalize()?;
    if !candidate.starts_with(&base) {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "path escapes base_dir",
        ));
    }
    fs::read_to_string(candidate)
}

fn observation(result: &io::Result<String>) -> String {
    match result {
        Ok(value) => format!("Ok({value})"),
        Err(error) => format!("Err({})", error.kind()),
    }
}

fn finding() -> SecurityFinding {
    SecurityFinding {
        finding_id: "local.path_traversal.read_user_file",
        class: "PATH_TRAVERSAL_RISK",
        severity: "high-local-fixture",
        vulnerable_route: "file.read.user_path",
        owner: "read_user_file",
        evidence: vec![
            SecurityEvidence {
                role: "user-controlled input",
                fact: "`user_path` is accepted as a raw string argument",
                source_span: "read_user_file(base_dir: &Path, user_path: &str)",
            },
            SecurityEvidence {
                role: "unsafe path join",
                fact: "`base_dir.join(user_path)` is used without canonical boundary check",
                source_span: "let candidate = base_dir.join(user_path);",
            },
            SecurityEvidence {
                role: "sensitive operation",
                fact: "joined path is read directly",
                source_span: "fs::read_to_string(candidate)",
            },
        ],
        route_field: SecurityRouteField {
            state: "FINDING_FOCUSED",
            route_peak: "file.read.user_path",
            cause_chain: vec![
                "user_path",
                "base_dir.join(user_path)",
                "fs::read_to_string(candidate)",
                "outside file reachable with ../secret.txt",
            ],
            forbidden_shortcuts: vec![
                "string_contains_path_separator_is_not_always_vulnerable",
                "path_join_is_not_vulnerable_without_sensitive_sink",
            ],
        },
    }
}

fn patch_candidate() -> SecurityPatchCandidate {
    SecurityPatchCandidate {
        patch_id: "patch.path_traversal.canonical_base_prefix",
        file: "fixtures/security/path_traversal/src/lib.rs",
        owner: "read_user_file",
        changed_function: "read_user_file",
        diff: r#"--- a/fixtures/security/path_traversal/src/lib.rs
+++ b/fixtures/security/path_traversal/src/lib.rs
@@
-use std::io;
+use std::io::{self, ErrorKind};
@@
 pub fn read_user_file(base_dir: &Path, user_path: &str) -> io::Result<String> {
-    let candidate = base_dir.join(user_path);
+    let base = base_dir.canonicalize()?;
+    let candidate = base_dir.join(user_path).canonicalize()?;
+    if !candidate.starts_with(&base) {
+        return Err(io::Error::new(ErrorKind::PermissionDenied, "path escapes base_dir"));
+    }
     fs::read_to_string(candidate)
 }
"#,
        patched_source_preview: PATCHED_SOURCE,
    }
}

struct FixtureLayout {
    base_dir: PathBuf,
}

impl FixtureLayout {
    fn create(root: &Path) -> Result<Self> {
        let base_dir = root.join("public");
        fs::create_dir_all(&base_dir).with_context(|| format!("create {}", base_dir.display()))?;
        fs::write(base_dir.join("public.txt"), "inside-public")
            .with_context(|| "write public fixture file")?;
        fs::write(root.join("secret.txt"), "outside-secret")
            .with_context(|| "write secret fixture file")?;
        Ok(Self { base_dir })
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
    fn security_fixture_finds_patches_and_verifies_path_traversal() {
        let root = unique_tmp_dir("security-fixture-test");
        let report = run_fixture_in(&root).unwrap();
        assert_eq!(report.verdict, "DEFENSIVE_PATCH_PROVEN_LOCAL_FIXTURE");
        assert_eq!(report.finding.class, "PATH_TRAVERSAL_RISK");
        assert!(report.result.finding_found);
        assert!(report.result.before_exploit_reaches_forbidden_path);
        assert!(report.result.after_forbidden_path_blocked);
        assert!(report.result.regression_normal_file_still_reads);
        assert!(report.verification.all_passed);
        assert!(!report.claim_boundary.real_project_scanner_ready);
        let _ = fs::remove_dir_all(root);
    }
}
