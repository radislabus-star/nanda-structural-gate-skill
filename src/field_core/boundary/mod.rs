use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::field_core::{FieldAntiWaveLane, FieldLensOperation, FieldPassReport, FieldRecord};
use crate::CORE_VERSION;

mod decision;
mod diff;
mod energy;
mod facts;
mod field_pass;
mod records;
mod util;

use decision::{boundary_decision, empty_components, score_component};
use energy::{boundary_energy, boundary_owner_gravity};
use facts::{
    collect_boundary_files, collect_facts, filter_files_by_owner, infer_single_owner,
    infer_single_route,
};
use field_pass::{
    boundary_field_engine, boundary_field_equivalence, boundary_field_pass_admission,
};
use records::boundary_field_records;

pub(crate) type BoundaryPathClassifier = fn(&str) -> String;

#[derive(Default)]
struct BoundaryFacts {
    files: Vec<String>,
    file_routes: BTreeMap<String, String>,
    file_owners: BTreeMap<String, String>,
    functions: Vec<String>,
    public_api: Vec<String>,
    call_edges: Vec<String>,
    shared_state: Vec<String>,
    runtime_side_effects: Vec<String>,
    tests: Vec<String>,
    routes: BTreeSet<String>,
    owners: BTreeSet<String>,
    thin_wrappers: Vec<String>,
    foreign_route_files: Vec<String>,
    foreign_route_details: Vec<Value>,
    runtime_checks: Vec<String>,
    route_scoped: bool,
    owner_filter_requested: bool,
    owner_filter_matched: bool,
    requested_owner: Option<String>,
    route_files_considered: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundaryVerdict {
    SplitStrong,
    SplitWeak,
    Keep,
    MergeCandidate,
    Veto,
    Watch,
}

impl BoundaryVerdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::SplitStrong => "SPLIT_STRONG",
            Self::SplitWeak => "SPLIT_WEAK",
            Self::Keep => "KEEP",
            Self::MergeCandidate => "MERGE_CANDIDATE",
            Self::Veto => "VETO",
            Self::Watch => "WATCH",
        }
    }

    fn safe_to_edit(self) -> bool {
        matches!(self, Self::SplitStrong | Self::Keep)
    }
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryDecision {
    verdict: &'static str,
    route: Option<String>,
    owner: Option<String>,
    reason: String,
    principle: &'static str,
    score: i32,
    safe_to_edit: bool,
    score_components: BoundaryScoreComponents,
    evidence: BoundaryEvidence,
    allowed_files: Vec<String>,
    forbidden_routes: Vec<String>,
    must_not_change: Vec<String>,
    required_tests: Vec<String>,
    repair: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryScoreComponents {
    owner_clarity_gain: BoundaryScoreComponent,
    foreign_pull_reduction: BoundaryScoreComponent,
    test_isolation_gain: BoundaryScoreComponent,
    state_locality_gain: BoundaryScoreComponent,
    api_surface_growth: BoundaryScoreComponent,
    adapter_leak: BoundaryScoreComponent,
    runtime_risk: BoundaryScoreComponent,
    migration_cost: BoundaryScoreComponent,
}

impl BoundaryScoreComponents {
    fn empty() -> Self {
        Self {
            owner_clarity_gain: score_component(0, vec![]),
            foreign_pull_reduction: score_component(0, vec![]),
            test_isolation_gain: score_component(0, vec![]),
            state_locality_gain: score_component(0, vec![]),
            api_surface_growth: score_component(0, vec![]),
            adapter_leak: score_component(0, vec![]),
            runtime_risk: score_component(0, vec![]),
            migration_cost: score_component(0, vec![]),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryScoreComponent {
    score: i32,
    counted: bool,
    evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryEvidence {
    files: Vec<String>,
    functions: Vec<String>,
    owner_edges: Vec<String>,
    call_edges: Vec<String>,
    public_api_edges: Vec<String>,
    foreign_pull: Vec<String>,
    foreign_pull_details: Vec<Value>,
    shared_state: Vec<String>,
    runtime_side_effects: Vec<String>,
    tests: Vec<String>,
    route_ids: Vec<String>,
    owner_ids: Vec<String>,
    owner_filter: BoundaryOwnerFilterEvidence,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryOwnerFilterEvidence {
    requested: bool,
    matched: bool,
    requested_owner: Option<String>,
    route_files_considered: usize,
    matched_files: usize,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryFieldRecordBridge {
    version: &'static str,
    owner: &'static str,
    record_count: usize,
    file_records: usize,
    function_records: usize,
    public_api_records: usize,
    call_edge_records: usize,
    shared_state_records: usize,
    runtime_side_effect_records: usize,
    test_records: usize,
    foreign_pull_records: usize,
    sample: Vec<FieldRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryFieldPassAdmission {
    version: &'static str,
    mode: &'static str,
    query: FieldRecord,
    lenses: Vec<FieldLensOperation>,
    anti_waves: Vec<FieldAntiWaveLane>,
    state_hint: Option<String>,
    field_pass: FieldPassReport,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryFieldEquivalence {
    version: &'static str,
    old_verdict: String,
    field_verdict: String,
    old_safe_to_edit: bool,
    field_safe_to_answer: bool,
    old_rank: u8,
    field_rank: u8,
    field_not_more_permissive: bool,
    cutover_ready: bool,
    mismatch_reason: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryFieldEngine {
    version: &'static str,
    selected_engine: &'static str,
    candidate_allowed: bool,
    cutover_applied: bool,
    top_level_boundary_decision_preserved: bool,
    selected_verdict: String,
    selected_safe_to_edit: bool,
    policy: BoundaryFieldEnginePolicy,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryFieldEnginePolicy {
    requires_not_more_permissive: bool,
    requires_field_pass_version: &'static str,
    can_be_stricter_than_typed_core: bool,
    cannot_be_more_permissive_than_typed_core: bool,
    public_json_compatibility: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryOwnerGravity {
    version: &'static str,
    requested_owner: Option<String>,
    dominant_owner: Option<String>,
    owner_file_counts: BTreeMap<String, usize>,
    owner_count: usize,
    dominant_owner_ratio: f64,
    owner_conflict: bool,
    owner_conflict_energy: i32,
    adapter_leak_energy: i32,
    cross_owner_call_edges: Vec<String>,
    verdict_hint: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryEnergy {
    version: &'static str,
    owner_clarity_gain: i32,
    foreign_pull_reduction: i32,
    test_isolation_gain: i32,
    state_locality_gain: i32,
    api_surface_growth: i32,
    adapter_leak: i32,
    runtime_risk: i32,
    migration_cost: i32,
    wrapper_tax: i32,
    owner_conflict_energy: i32,
    boundary_tax: i32,
    net: i32,
    verdict_hint: &'static str,
    read_as: &'static str,
}

pub(crate) fn boundary_report(
    input: &Path,
    atlas: Option<&Value>,
    atlas_label: Option<String>,
    route: Option<&str>,
    owner: Option<&str>,
    route_for_path: BoundaryPathClassifier,
    owner_for_path: BoundaryPathClassifier,
) -> Result<Value> {
    let root = if input.is_dir() {
        input.to_path_buf()
    } else {
        input
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    let mut route_scoped = false;
    let mut runtime_checks = vec![];
    let mut files = vec![];
    if let (Some(atlas), Some(route)) = (atlas, route) {
        if let Some(route_node) = atlas["routes"][route].as_object() {
            route_scoped = true;
            files = route_node["allowed_files"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            runtime_checks = route_node["runtime_checks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
        }
    }
    if files.is_empty() && input.is_dir() {
        collect_boundary_files(&root, input, &mut files)?;
    } else if files.is_empty() && input.is_file() {
        files.push(input.to_string_lossy().to_string());
    }
    let route_files_considered = files.len();
    let mut owner_filter_requested = false;
    let mut owner_filter_matched = true;
    if route_scoped {
        if let Some(owner) = owner {
            owner_filter_requested = true;
            let filtered = filter_files_by_owner(&files, owner, owner_for_path);
            if !filtered.is_empty() {
                files = filtered;
            } else {
                owner_filter_matched = false;
                files.clear();
            }
        }
    }
    files.sort();

    let target_route = route
        .map(str::to_string)
        .or_else(|| infer_single_route(&files, route_for_path));
    let target_owner = owner
        .map(str::to_string)
        .or_else(|| infer_single_owner(&files, owner_for_path));
    let mut facts = collect_facts(
        &root,
        &files,
        target_route.as_deref(),
        route_for_path,
        owner_for_path,
    );
    facts.runtime_checks = runtime_checks;
    facts.route_scoped = route_scoped;
    facts.owner_filter_requested = owner_filter_requested;
    facts.owner_filter_matched = owner_filter_matched;
    facts.requested_owner = owner.map(str::to_string);
    facts.route_files_considered = route_files_considered;
    let decision = boundary_decision(&facts, target_route.as_deref(), target_owner.as_deref());
    let field_records = boundary_field_records(&facts);
    let field_record_bridge = BoundaryFieldRecordBridge::from_facts(&facts, &field_records);
    let owner_gravity = boundary_owner_gravity(&facts);
    let boundary_energy = boundary_energy(&facts, &decision, &owner_gravity);
    let field_admission = boundary_field_pass_admission(
        &facts,
        &decision,
        &field_records,
        target_route.as_deref(),
        target_owner.as_deref(),
    );
    let field_equivalence = boundary_field_equivalence(&decision, &field_admission.field_pass);
    let field_engine = boundary_field_engine(&decision, &field_equivalence);

    Ok(json!({
        "mode": "boundary-economics",
        "core_version": CORE_VERSION,
        "boundary_core": {
            "owner": "field_core::boundary",
            "version": "boundary-economics-core-v2-field-pass",
            "commands_are_wrappers": true
        },
        "input": input.display().to_string(),
        "atlas": atlas_label,
        "scope": if route_scoped { "route-scoped" } else { "repo-wide" },
        "target_route": target_route,
        "target_owner": target_owner,
        "boundary_decision": decision,
        "boundary_field_records": field_record_bridge,
        "boundary_field_pass": field_admission,
        "field_equivalence": field_equivalence,
        "boundary_field_engine": field_engine,
        "owner_gravity": owner_gravity,
        "boundary_energy": boundary_energy,
        "read_as": "NANDA Boundary Economics: NO EVIDENCE => NO CUT. Split/merge decisions require route, owner, state, API, runtime, and test evidence."
    }))
}

pub(crate) fn boundary_from_guard_action(atlas: &Value, action_out: &Value) -> Value {
    let route = action_out["route"].as_str();
    let route_node = route.map(|route| &atlas["routes"][route]);
    let owner = route_node
        .and_then(|node| node["owners"].as_array())
        .and_then(|owners| owners.first())
        .and_then(Value::as_str);
    let allowed_files = route_node
        .map(|node| node["allowed_files"].clone())
        .unwrap_or_else(|| json!([]));
    let verdict = if action_out["verdict"].as_str() == Some("PASS") && route.is_some() {
        "KEEP"
    } else if action_out["verdict"].as_str() == Some("HARD_STOP") {
        "VETO"
    } else {
        "WATCH"
    };
    json!({
        "verdict": verdict,
        "route": route,
        "owner": owner,
        "reason": if verdict == "KEEP" { "action route is known; no boundary cut is requested" } else { "action route is not sufficiently proven for boundary edits" },
        "principle": "NO_EVIDENCE_NO_CUT",
        "score_components": empty_components(),
        "evidence": {
            "owner_edges": [],
            "foreign_pull": [],
            "shared_state": [],
            "runtime_side_effects": [],
            "tests": []
        },
        "allowed_files": allowed_files,
        "forbidden_routes": route_node.map(|node| node["forbidden_routes"].clone()).unwrap_or_else(|| json!([])),
        "must_not_change": [],
        "required_tests": route_node.map(|node| node["runtime_checks"].clone()).unwrap_or_else(|| json!([])),
        "repair": if verdict == "WATCH" { json!(["provide route, owner, evidence, and diff before split/merge"]) } else { json!([]) }
    })
}

pub(crate) fn boundary_guard_diff(
    atlas: &Value,
    action_id: &str,
    diff_text: &str,
    diff_source: Option<Value>,
) -> Value {
    diff::boundary_guard_diff(atlas, action_id, diff_text, diff_source)
}

pub(crate) fn boundary_guard_diff_unreadable(
    atlas: &Value,
    action_id: &str,
    diff_source: Option<Value>,
    diff_error: &str,
) -> Value {
    diff::boundary_guard_diff_unreadable(atlas, action_id, diff_source, diff_error)
}

pub(crate) fn boundary_from_guard_diff(atlas: &Value, diff_out: &Value) -> Value {
    let route = diff_out["route"].as_str();
    let route_node = route.map(|route| &atlas["routes"][route]);
    let foreign_routes = diff_out["foreign_routes"].clone();
    let foreign_files = diff_out["foreign_files"].clone();
    let has_foreign = foreign_routes
        .as_array()
        .is_some_and(|items| !items.is_empty())
        || foreign_files
            .as_array()
            .is_some_and(|items| !items.is_empty());
    let verdict = if has_foreign {
        "VETO"
    } else if diff_out["verdict"].as_str() == Some("PASS") {
        "KEEP"
    } else {
        "WATCH"
    };
    json!({
        "verdict": verdict,
        "route": route,
        "owner": route_node.and_then(|node| node["owners"].as_array()).and_then(|owners| owners.first()).and_then(Value::as_str),
        "reason": if verdict == "VETO" { "diff changes forbidden route or file; boundary refactor is unsafe" } else if verdict == "KEEP" { "diff stays inside selected route capsule; no boundary cut is requested" } else { "diff evidence is insufficient for boundary decision" },
        "principle": "NO_EVIDENCE_NO_CUT",
        "score_components": empty_components(),
        "evidence": {
            "owner_edges": [],
            "foreign_pull": foreign_routes,
            "shared_state": [],
            "runtime_side_effects": [],
            "tests": []
        },
        "allowed_files": route_node.map(|node| node["allowed_files"].clone()).unwrap_or_else(|| json!([])),
        "forbidden_routes": route_node.map(|node| node["forbidden_routes"].clone()).unwrap_or_else(|| json!([])),
        "must_not_change": ["foreign routes", "unowned runtime behavior"],
        "required_tests": route_node.map(|node| node["runtime_checks"].clone()).unwrap_or_else(|| json!([])),
        "repair": if verdict == "VETO" { json!(["split the diff by route", "choose an action_id that owns all changed files"]) } else { json!([]) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn source_rel(path: &str) -> &str {
        path.rsplit_once("/src/").map_or(path, |(_, rel)| rel)
    }

    fn route_for_path(path: &str) -> String {
        let path = source_rel(path);
        if path.contains("ime/") || path.starts_with("ime_") {
            "ime-display-flow".to_string()
        } else if path.contains("nanda/") || path.starts_with("nanda_") {
            "nanda-field-flow".to_string()
        } else {
            "runtime-flow".to_string()
        }
    }

    fn owner_for_path(path: &str) -> String {
        let path = source_rel(path);
        if path.contains("runtime_one.rs") {
            "src::runtime_one.rs".to_string()
        } else if path.contains("ime/") {
            "ImeDisplayOwner".to_string()
        } else if path.contains("nanda/") {
            "NandaFieldOwner".to_string()
        } else {
            "RuntimeOwner".to_string()
        }
    }

    fn temp_repo(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "nanda-boundary-kernel-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(path.join("src")).expect("create temp repo");
        path
    }

    #[test]
    fn boundary_keep_uses_field_pass_without_relaxing_verdict() {
        let repo = temp_repo("keep");
        let file = repo.join("src/runtime_one.rs");
        fs::write(
            &file,
            "static STATE: std::sync::OnceLock<u8> = std::sync::OnceLock::new();\n\
             pub(crate) fn handle_manual_trigger_runtime() { let _ = STATE.get(); }\n\
             fn route_private() {}\n\
             fn route_private_two() {}\n",
        )
        .expect("write keep fixture");

        let report = boundary_report(
            &file,
            None,
            None,
            Some("runtime-flow"),
            Some("src::runtime_one.rs"),
            route_for_path,
            owner_for_path,
        )
        .expect("boundary report");

        assert_eq!(
            report["boundary_decision"]["verdict"],
            "KEEP",
            "{}",
            serde_json::to_string_pretty(&report).unwrap()
        );
        assert_eq!(
            report["boundary_field_pass"]["field_pass"]["verdict"],
            "PASS"
        );
        assert_eq!(
            report["field_equivalence"]["field_not_more_permissive"],
            true
        );
        assert_eq!(report["boundary_field_engine"]["selected_verdict"], "KEEP");
        assert_eq!(report["owner_gravity"]["verdict_hint"], "OWNER_STABLE");
    }

    #[test]
    fn boundary_mixed_route_selects_field_veto() {
        let repo = temp_repo("mixed");
        fs::create_dir_all(repo.join("src/ime")).expect("create ime dir");
        fs::create_dir_all(repo.join("src/nanda")).expect("create nanda dir");
        fs::write(
            repo.join("src/ime/display.rs"),
            "pub fn show_candidate() { score_candidate(); }\n",
        )
        .expect("write ime");
        fs::write(
            repo.join("src/nanda/scoring.rs"),
            "pub fn score_candidate() {}\n",
        )
        .expect("write nanda");

        let report = boundary_report(
            &repo,
            None,
            None,
            Some("ime-display-flow"),
            Some("ImeDisplayOwner"),
            route_for_path,
            owner_for_path,
        )
        .expect("boundary report");

        assert_eq!(
            report["boundary_field_pass"]["field_pass"]["verdict"],
            "VETO"
        );
        assert_eq!(
            report["field_equivalence"]["field_not_more_permissive"],
            true
        );
        assert_eq!(report["boundary_field_engine"]["selected_verdict"], "VETO");
        assert_eq!(report["owner_gravity"]["verdict_hint"], "OWNER_CONFLICT");
    }

    #[test]
    fn boundary_runtime_risk_downgrades_keep_to_watch() {
        let repo = temp_repo("runtime-risk");
        let file = repo.join("src/runtime_risk.rs");
        fs::write(
            &file,
            "pub(crate) fn touch_runtime() { let _ = std::process::Command::new(\"true\"); }\n",
        )
        .expect("write runtime risk");

        let report = boundary_report(
            &file,
            None,
            None,
            Some("runtime-flow"),
            Some("RuntimeOwner"),
            route_for_path,
            owner_for_path,
        )
        .expect("boundary report");

        assert_eq!(
            report["boundary_decision"]["verdict"],
            "KEEP",
            "{}",
            serde_json::to_string_pretty(&report).unwrap()
        );
        assert_eq!(
            report["boundary_field_pass"]["field_pass"]["verdict"],
            "WATCH"
        );
        assert_eq!(
            report["boundary_field_pass"]["field_pass"]["coherence_state"],
            "FIELD_THIN"
        );
        assert_eq!(
            report["field_equivalence"]["field_not_more_permissive"],
            true
        );
        assert_eq!(report["boundary_field_engine"]["selected_verdict"], "WATCH");
    }
}
