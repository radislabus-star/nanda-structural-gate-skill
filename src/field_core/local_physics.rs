use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(crate) const FIELD_LOCAL_PHYSICS_AUDIT_VERSION: &str = "field-local-physics-audit-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldLocalPhysicsAudit {
    pub version: &'static str,
    pub source_root: String,
    pub source_scan_available: bool,
    pub verdict: &'static str,
    pub read_as: &'static str,
    pub totals: FieldLocalPhysicsTotals,
    pub categories: Vec<FieldLocalPhysicsCategoryCount>,
    pub findings: Vec<FieldLocalPhysicsFinding>,
    pub blockers: Vec<&'static str>,
    pub next_migrations: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FieldLocalPhysicsTotals {
    pub scanned_files: usize,
    pub files_with_field_terms: usize,
    pub field_core_owned: usize,
    pub field_core_backed_wrappers: usize,
    pub packed_numeric_kernels: usize,
    pub structural_legacy_readouts: usize,
    pub domain_fixture_readouts: usize,
    pub local_physics_candidates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldLocalPhysicsCategoryCount {
    pub category: &'static str,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldLocalPhysicsFinding {
    pub path: String,
    pub category: &'static str,
    pub risk: &'static str,
    pub owner: &'static str,
    pub reason: &'static str,
    pub matched_terms: Vec<&'static str>,
}

pub(crate) fn build_local_physics_audit(root: impl AsRef<Path>) -> FieldLocalPhysicsAudit {
    let source_root = root.as_ref().join("src");
    let mut files = vec![];
    collect_rs_files(&source_root, &mut files);
    files.sort();

    let mut totals = FieldLocalPhysicsTotals {
        scanned_files: files.len(),
        ..FieldLocalPhysicsTotals::default()
    };
    let mut findings = vec![];

    for path in files {
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        let matched_terms = matched_physics_terms(&source);
        if matched_terms.is_empty() {
            continue;
        }

        totals.files_with_field_terms += 1;
        let relative_path = path
            .strip_prefix(root.as_ref())
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let finding = classify_physics_file(&relative_path, &source, matched_terms);
        bump_total(&mut totals, finding.category);
        findings.push(finding);
    }

    let blockers = if totals.local_physics_candidates > 0 {
        vec!["unclassified_local_physics_candidates_found"]
    } else {
        vec![]
    };
    let verdict = if !source_root.exists() {
        "SOURCE_SCAN_UNAVAILABLE"
    } else if totals.local_physics_candidates > 0 {
        "LOCAL_PHYSICS_REVIEW_REQUIRED"
    } else if totals.domain_fixture_readouts > 0
        || totals.packed_numeric_kernels > 0
        || totals.structural_legacy_readouts > 0
    {
        "FIELD_CORE_SOLE_ENGINE_WITH_REVIEW_DEBT"
    } else {
        "FIELD_CORE_ONLY"
    };

    FieldLocalPhysicsAudit {
        version: FIELD_LOCAL_PHYSICS_AUDIT_VERSION,
        source_root: source_root.to_string_lossy().to_string(),
        source_scan_available: source_root.exists(),
        verdict,
        read_as: "This is a source inventory for remaining field-physics surfaces. It does not weaken the sole-engine claim; it identifies wrappers, numeric kernels, fixture readouts, and unclassified local candidates for future migration.",
        totals,
        categories: category_counts(&findings),
        findings,
        blockers,
        next_migrations: vec![
            "move unclassified local physics candidates behind FieldPassInput or a typed field_core owner",
            "turn domain fixture readouts into field_core-backed admission checks before using them as answer permission",
            "keep packed numeric kernels as hot arithmetic only; route/verdict authority must stay in field_core",
            "rerun nanda-field-audit after each migration and require local_physics_candidates=0",
        ],
    }
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn matched_physics_terms(source: &str) -> Vec<&'static str> {
    PHYSICS_TERMS
        .iter()
        .copied()
        .filter(|term| source.contains(term))
        .collect()
}

fn classify_physics_file(
    path: &str,
    source: &str,
    matched_terms: Vec<&'static str>,
) -> FieldLocalPhysicsFinding {
    let uses_field_core = FIELD_CORE_MARKERS
        .iter()
        .any(|marker| source.contains(marker));
    let (category, risk, owner, reason) = if path.starts_with("src/field_core/") {
        (
            "field_core_owned",
            "LOW",
            "field_core",
            "field physics is implemented inside the shared owner",
        )
    } else if uses_field_core {
        (
            "field_core_backed_wrapper",
            "LOW",
            "field_core consumer",
            "file exposes or adapts field terms through field_core",
        )
    } else if path.starts_with("src/nanda_6m/") || path == "src/nanda_6m.rs" {
        (
            "packed_numeric_kernel",
            "REVIEW_ONLY",
            "nanda_6m numeric hot kernel",
            "low-level packed arithmetic is allowed only as numeric kernel; verdict authority stays in field_core",
        )
    } else if is_structural_legacy_readout(path) {
        (
            "structural_legacy_readout",
            "MIGRATION_DEBT",
            "structural/search legacy report",
            "legacy structural command/report still carries local field readout terms; trust must flow through field_core audit/cutover gates",
        )
    } else if path.starts_with("src/llmwave_big/") {
        (
            "domain_fixture_readout",
            "MIGRATION_DEBT",
            "llmwave_big local fixture",
            "fixture/domain report still carries local field readout terms; answer permission must come from field_core gates",
        )
    } else {
        (
            "local_physics_candidate",
            "WATCH",
            "unclassified local module",
            "field terms appear outside field_core without a recognized field_core adapter marker",
        )
    };

    FieldLocalPhysicsFinding {
        path: path.to_string(),
        category,
        risk,
        owner,
        reason,
        matched_terms,
    }
}

fn bump_total(totals: &mut FieldLocalPhysicsTotals, category: &str) {
    match category {
        "field_core_owned" => totals.field_core_owned += 1,
        "field_core_backed_wrapper" => totals.field_core_backed_wrappers += 1,
        "packed_numeric_kernel" => totals.packed_numeric_kernels += 1,
        "structural_legacy_readout" => totals.structural_legacy_readouts += 1,
        "domain_fixture_readout" => totals.domain_fixture_readouts += 1,
        "local_physics_candidate" => totals.local_physics_candidates += 1,
        _ => {}
    }
}

fn is_structural_legacy_readout(path: &str) -> bool {
    path.starts_with("src/commands/")
        || matches!(
            path,
            "src/dataset_doctor.rs"
                | "src/decode.rs"
                | "src/eval.rs"
                | "src/feedback.rs"
                | "src/field_plate.rs"
                | "src/main.rs"
                | "src/map_gate.rs"
                | "src/model.rs"
                | "src/pattern_bank.rs"
                | "src/pattern_store.rs"
                | "src/proof.rs"
                | "src/report.rs"
                | "src/search.rs"
        )
}

fn category_counts(findings: &[FieldLocalPhysicsFinding]) -> Vec<FieldLocalPhysicsCategoryCount> {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for finding in findings {
        *counts.entry(finding.category).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(category, count)| FieldLocalPhysicsCategoryCount { category, count })
        .collect()
}

const PHYSICS_TERMS: &[&str] = &[
    "FieldPass",
    "run_field_pass",
    "safe_to_answer",
    "field_state",
    "top_peak",
    "peak_margin",
    "coherence",
    "anti_wave",
    "anti-wave",
    "suppression",
    "verdict",
];

const FIELD_CORE_MARKERS: &[&str] = &[
    "field_core::",
    "run_field_pass",
    "FieldPassInput",
    "FieldPassReport",
    "UnifiedFieldReport",
    "with_unified_field",
    "field_core_admission",
    "cognitive_field_engine",
    "field_runtime",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_physics_classifier_keeps_field_core_as_owner() {
        let finding = classify_physics_file(
            "src/field_core/peak.rs",
            "pub fn detect_field_peak() { let safe_to_answer = true; }",
            vec!["safe_to_answer"],
        );

        assert_eq!(finding.category, "field_core_owned");
        assert_eq!(finding.risk, "LOW");
    }

    #[test]
    fn local_physics_classifier_marks_unbacked_llmwave_readout_as_debt() {
        let finding = classify_physics_file(
            "src/llmwave_big/example.rs",
            "let top_peak = route; let safe_to_answer = true;",
            vec!["safe_to_answer", "top_peak"],
        );

        assert_eq!(finding.category, "domain_fixture_readout");
        assert_eq!(finding.risk, "MIGRATION_DEBT");
    }

    #[test]
    fn local_physics_classifier_marks_field_core_adapter_as_wrapper() {
        let finding = classify_physics_file(
            "src/llmwave_big/report.rs",
            "fn with_unified_field() { let safe_to_answer = field_core::run(); }",
            vec!["safe_to_answer"],
        );

        assert_eq!(finding.category, "field_core_backed_wrapper");
        assert_eq!(finding.risk, "LOW");
    }

    #[test]
    fn local_physics_classifier_marks_structural_legacy_readout_as_debt() {
        let finding = classify_physics_file(
            "src/proof.rs",
            "let top_peak = route; let verdict = \"WATCH\";",
            vec!["top_peak", "verdict"],
        );

        assert_eq!(finding.category, "structural_legacy_readout");
        assert_eq!(finding.risk, "MIGRATION_DEBT");
    }
}
