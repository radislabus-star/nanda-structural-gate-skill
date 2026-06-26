use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::{
    run_field_pass, FieldAntiWaveLane, FieldClaimBoundary, FieldFamily, FieldLensKind,
    FieldLensOperation, FieldPassInput, FieldPassReport, FieldRecord, FieldRecordKind,
    FIELD_PASS_VERSION,
};
use crate::CORE_VERSION;

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

fn collect_boundary_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(".git")
            || name == "target"
            || name == "node_modules"
            || name == ".nanda"
            || name == "__pycache__"
        {
            continue;
        }
        if path.is_dir() {
            collect_boundary_files(root, &path, out)?;
            continue;
        }
        if !is_boundary_file(&path) {
            continue;
        }
        out.push(
            path.strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string(),
        );
    }
    Ok(())
}

fn is_boundary_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "rs" | "py" | "js" | "ts" | "tsx"))
}

fn infer_single_route(files: &[String], route_for_path: BoundaryPathClassifier) -> Option<String> {
    let routes = files
        .iter()
        .map(|file| route_for_path(file))
        .collect::<BTreeSet<_>>();
    (routes.len() == 1)
        .then(|| routes.into_iter().next())
        .flatten()
}

fn infer_single_owner(files: &[String], owner_for_path: BoundaryPathClassifier) -> Option<String> {
    let owners = files
        .iter()
        .map(|file| owner_for_path(file))
        .collect::<BTreeSet<_>>();
    (owners.len() == 1)
        .then(|| owners.into_iter().next())
        .flatten()
}

fn filter_files_by_owner(
    files: &[String],
    owner: &str,
    owner_for_path: BoundaryPathClassifier,
) -> Vec<String> {
    let owner_norm = normalize_owner_for_match(owner);
    files
        .iter()
        .filter(|file| {
            let file_norm = normalize_owner_for_match(file);
            let auto_owner_norm = normalize_owner_for_match(&owner_for_path(file));
            file_norm.contains(&owner_norm) || auto_owner_norm.contains(&owner_norm)
        })
        .cloned()
        .collect()
}

fn normalize_owner_for_match(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn collect_facts(
    root: &Path,
    files: &[String],
    target_route: Option<&str>,
    route_for_path: BoundaryPathClassifier,
    owner_for_path: BoundaryPathClassifier,
) -> BoundaryFacts {
    let mut facts = BoundaryFacts {
        files: files.to_vec(),
        ..Default::default()
    };
    let mut function_owner = BTreeMap::<String, String>::new();

    for rel in files {
        let path = root.join(rel);
        let route = route_for_path(rel);
        let owner = owner_for_path(rel);
        facts.routes.insert(route.clone());
        facts.owners.insert(owner.clone());
        facts.file_routes.insert(rel.clone(), route.clone());
        facts.file_owners.insert(rel.clone(), owner.clone());
        if target_route.is_some_and(|target| target != route) {
            facts.foreign_route_files.push(rel.clone());
            facts.foreign_route_details.push(json!({
                "file": rel,
                "route": route,
                "target_route": target_route
            }));
        }
        if rel.contains("test") || rel.contains("spec") {
            facts.tests.push(rel.clone());
        }
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let functions = extract_functions(&source);
        for function in &functions {
            function_owner.insert(function.clone(), rel.clone());
        }
        facts.functions.extend(
            functions
                .iter()
                .map(|function| format!("{rel}::{function}")),
        );
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if is_public_api_line(trimmed) {
                facts
                    .public_api
                    .push(format!("{rel}:{}:{trimmed}", line_no + 1));
            }
            if is_shared_state_line(trimmed) {
                facts
                    .shared_state
                    .push(format!("{rel}:{}:{trimmed}", line_no + 1));
            }
            if is_runtime_side_effect_line(trimmed) {
                facts
                    .runtime_side_effects
                    .push(format!("{rel}:{}:{trimmed}", line_no + 1));
            }
        }
        if is_thin_wrapper(&source, &functions) {
            facts.thin_wrappers.push(rel.clone());
        }
    }

    for rel in files {
        let source = match fs::read_to_string(root.join(rel)) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let route = route_for_path(rel);
        for (name, owner_file) in &function_owner {
            if owner_file == rel {
                continue;
            }
            if source.contains(&format!("{name}(")) {
                let other_route = route_for_path(owner_file);
                facts
                    .call_edges
                    .push(format!("{rel} -> {owner_file}::{name}"));
                if other_route != route {
                    facts.foreign_route_files.push(owner_file.clone());
                    facts.foreign_route_details.push(json!({
                        "file": owner_file,
                        "route": other_route,
                        "pulled_by": rel,
                        "call_edge": format!("{rel} -> {owner_file}::{name}")
                    }));
                }
            }
        }
    }
    facts.foreign_route_files.sort();
    facts.foreign_route_files.dedup();
    facts
        .foreign_route_details
        .sort_by_key(|item| item["file"].as_str().unwrap_or("").to_string());
    facts
        .foreign_route_details
        .dedup_by(|left, right| left["file"] == right["file"] && left["route"] == right["route"]);
    facts
}

fn boundary_decision(
    facts: &BoundaryFacts,
    route: Option<&str>,
    owner: Option<&str>,
) -> BoundaryDecision {
    let evidence_count = facts.functions.len()
        + facts.public_api.len()
        + facts.call_edges.len()
        + facts.shared_state.len()
        + facts.runtime_side_effects.len()
        + facts.tests.len()
        + facts.foreign_route_files.len();
    let multi_route = facts.routes.len() > 1 || !facts.foreign_route_files.is_empty();
    let multi_owner = facts.owners.len() > 1;
    let has_owner = owner.is_some() || !facts.owners.is_empty();
    let has_route = route.is_some() || !facts.routes.is_empty();
    let has_tests = !facts.tests.is_empty();
    let high_public_api = facts.public_api.len() >= 6;
    let thin_wrapper = !facts.thin_wrappers.is_empty()
        && facts.shared_state.is_empty()
        && facts.tests.is_empty()
        && facts.runtime_side_effects.is_empty();

    let owner_clarity_gain = if multi_owner { 3 } else { 0 };
    let foreign_pull_reduction = if multi_route { 3 } else { 0 };
    let test_isolation_gain = if multi_route && has_tests { 1 } else { 0 };
    let state_locality_gain = if !facts.shared_state.is_empty() && !multi_route {
        1
    } else {
        0
    };
    let api_surface_growth = if high_public_api { 2 } else { 0 };
    let adapter_leak = if thin_wrapper { 2 } else { 0 };
    let runtime_risk = if !facts.runtime_side_effects.is_empty() && !has_tests {
        2
    } else {
        0
    };
    let migration_cost = if facts.files.len() > 8 || facts.functions.len() > 30 {
        2
    } else {
        0
    };
    let positive =
        owner_clarity_gain + foreign_pull_reduction + test_isolation_gain + state_locality_gain;
    let negative = api_surface_growth + adapter_leak + runtime_risk + migration_cost;
    let score = positive - negative;

    let owner_filter_failed = facts.owner_filter_requested && !facts.owner_filter_matched;
    let repo_wide_route_pressure = route.is_none() && !facts.foreign_route_files.is_empty();

    let verdict = if owner_filter_failed || !has_owner || !has_route || evidence_count == 0 {
        BoundaryVerdict::Watch
    } else if !facts.foreign_route_files.is_empty() && route.is_some() {
        BoundaryVerdict::Veto
    } else if repo_wide_route_pressure && score <= 0 {
        BoundaryVerdict::Watch
    } else if multi_route && score >= 3 {
        BoundaryVerdict::SplitStrong
    } else if multi_route && score > 0 {
        BoundaryVerdict::SplitWeak
    } else if thin_wrapper {
        BoundaryVerdict::MergeCandidate
    } else {
        BoundaryVerdict::Keep
    };

    let reason = match verdict {
        BoundaryVerdict::Watch if owner_filter_failed => {
            "owner evidence not found in route atlas; explicit owner cannot fall back to whole route"
        }
        BoundaryVerdict::Watch if repo_wide_route_pressure => {
            "repo-wide route pressure found, but evidence is not strong enough for autonomous split; rerun route-scoped with atlas, route, and owner"
        }
        BoundaryVerdict::Watch => "insufficient route, owner, or evidence; NO EVIDENCE => NO CUT",
        BoundaryVerdict::Veto => "target route would cross foreign route files; split/merge is unsafe",
        BoundaryVerdict::SplitStrong => {
            "mixed routes/owners create enough evidence that a boundary should reduce confusion"
        }
        BoundaryVerdict::SplitWeak => {
            "boundary may help, but evidence is not strong enough for autonomous refactor"
        }
        BoundaryVerdict::MergeCandidate => {
            "thin wrapper has no state, tests, runtime side effects, or independent owner evidence"
        }
        BoundaryVerdict::Keep => {
            "current boundary is acceptable; no cut is justified by available evidence"
        }
    };

    let allowed_files = if facts.route_scoped
        && matches!(
            verdict,
            BoundaryVerdict::SplitStrong | BoundaryVerdict::SplitWeak | BoundaryVerdict::Keep
        ) {
        facts.files.clone()
    } else if matches!(verdict, BoundaryVerdict::Keep) {
        sample(&facts.files, 12)
    } else {
        vec![]
    };
    let forbidden_routes = if let Some(route) = route {
        facts
            .routes
            .iter()
            .filter(|item| item.as_str() != route)
            .cloned()
            .collect()
    } else {
        vec![]
    };

    BoundaryDecision {
        verdict: verdict.as_str(),
        route: route.map(str::to_string),
        owner: owner.map(str::to_string),
        reason: reason.to_string(),
        principle: "NO_EVIDENCE_NO_CUT",
        score,
        safe_to_edit: verdict.safe_to_edit(),
        score_components: BoundaryScoreComponents {
            owner_clarity_gain: score_component(owner_clarity_gain, owner_evidence(facts)),
            foreign_pull_reduction: score_component(
                foreign_pull_reduction,
                facts.foreign_route_files.clone(),
            ),
            test_isolation_gain: score_component(test_isolation_gain, facts.tests.clone()),
            state_locality_gain: score_component(state_locality_gain, facts.shared_state.clone()),
            api_surface_growth: score_component(-api_surface_growth, facts.public_api.clone()),
            adapter_leak: score_component(-adapter_leak, facts.thin_wrappers.clone()),
            runtime_risk: score_component(-runtime_risk, facts.runtime_side_effects.clone()),
            migration_cost: score_component(-migration_cost, facts.files.clone()),
        },
        evidence: BoundaryEvidence {
            files: facts.files.clone(),
            functions: sample(&facts.functions, 24),
            owner_edges: owner_evidence(facts),
            call_edges: sample(&facts.call_edges, 24),
            public_api_edges: sample(&facts.public_api, 24),
            foreign_pull: facts.foreign_route_files.clone(),
            foreign_pull_details: sample_values(&facts.foreign_route_details, 16),
            shared_state: sample(&facts.shared_state, 24),
            runtime_side_effects: sample(&facts.runtime_side_effects, 24),
            tests: facts.tests.clone(),
            route_ids: facts.routes.iter().cloned().collect(),
            owner_ids: facts.owners.iter().cloned().collect(),
            owner_filter: BoundaryOwnerFilterEvidence {
                requested: facts.owner_filter_requested,
                matched: facts.owner_filter_matched,
                requested_owner: facts.requested_owner.clone(),
                route_files_considered: facts.route_files_considered,
                matched_files: facts.files.len(),
            },
        },
        allowed_files,
        forbidden_routes,
        must_not_change: must_not_change(verdict),
        required_tests: required_tests(facts),
        repair: repair_tasks(verdict, owner_filter_failed, repo_wide_route_pressure),
    }
}

impl BoundaryFieldRecordBridge {
    fn from_facts(facts: &BoundaryFacts, records: &[FieldRecord]) -> Self {
        Self {
            version: "boundary-field-records-v1",
            owner: "field_core::boundary",
            record_count: records.len(),
            file_records: facts.files.len(),
            function_records: facts.functions.len(),
            public_api_records: facts.public_api.len(),
            call_edge_records: facts.call_edges.len(),
            shared_state_records: facts.shared_state.len(),
            runtime_side_effect_records: facts.runtime_side_effects.len(),
            test_records: facts.tests.len(),
            foreign_pull_records: facts.foreign_route_files.len(),
            sample: records.iter().take(16).cloned().collect(),
        }
    }
}

fn boundary_owner_gravity(facts: &BoundaryFacts) -> BoundaryOwnerGravity {
    let mut owner_file_counts = BTreeMap::<String, usize>::new();
    for owner in facts.file_owners.values() {
        *owner_file_counts.entry(owner.clone()).or_default() += 1;
    }
    let dominant_owner = owner_file_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(owner, _)| owner.clone());
    let dominant_count = dominant_owner
        .as_ref()
        .and_then(|owner| owner_file_counts.get(owner))
        .copied()
        .unwrap_or(0);
    let dominant_owner_ratio = if facts.files.is_empty() {
        0.0
    } else {
        round4(dominant_count as f64 / facts.files.len() as f64)
    };
    let cross_owner_call_edges = cross_owner_call_edges(facts);
    let owner_conflict = facts.owner_filter_requested && !facts.owner_filter_matched
        || owner_file_counts.len() > 1
        || !cross_owner_call_edges.is_empty();
    let owner_conflict_energy = (owner_file_counts.len().saturating_sub(1) as i32 * 2)
        + cross_owner_call_edges.len() as i32;
    let adapter_leak_energy = facts.thin_wrappers.len() as i32
        + if !facts.public_api.is_empty() && facts.shared_state.is_empty() {
            1
        } else {
            0
        };
    let verdict_hint = if owner_conflict {
        "OWNER_CONFLICT"
    } else if dominant_owner.is_some() {
        "OWNER_STABLE"
    } else {
        "OWNER_UNRESOLVED"
    };
    BoundaryOwnerGravity {
        version: "boundary-owner-gravity-v1",
        requested_owner: facts.requested_owner.clone(),
        dominant_owner,
        owner_file_counts,
        owner_count: facts.owners.len(),
        dominant_owner_ratio,
        owner_conflict,
        owner_conflict_energy,
        adapter_leak_energy,
        cross_owner_call_edges: sample(&cross_owner_call_edges, 16),
        verdict_hint,
    }
}

fn boundary_energy(
    facts: &BoundaryFacts,
    decision: &BoundaryDecision,
    owner_gravity: &BoundaryOwnerGravity,
) -> BoundaryEnergy {
    let owner_clarity_gain = decision.score_components.owner_clarity_gain.score;
    let foreign_pull_reduction = decision.score_components.foreign_pull_reduction.score;
    let test_isolation_gain = decision.score_components.test_isolation_gain.score;
    let state_locality_gain = decision.score_components.state_locality_gain.score;
    let api_surface_growth = -decision.score_components.api_surface_growth.score;
    let adapter_leak = -decision.score_components.adapter_leak.score;
    let runtime_risk = -decision.score_components.runtime_risk.score;
    let migration_cost = -decision.score_components.migration_cost.score;
    let wrapper_tax = facts.thin_wrappers.len() as i32;
    let boundary_tax = api_surface_growth
        + adapter_leak
        + runtime_risk
        + migration_cost
        + wrapper_tax
        + owner_gravity.owner_conflict_energy;
    let net =
        owner_clarity_gain + foreign_pull_reduction + test_isolation_gain + state_locality_gain
            - boundary_tax;
    let verdict_hint = if matches!(decision.verdict, "VETO") || owner_gravity.owner_conflict {
        "NO_CUT_REPAIR_OWNER_OR_ROUTE"
    } else if net >= 3 {
        "CUT_CAN_REDUCE_CONFUSION"
    } else if net < 0 {
        "BOUNDARY_TAX_TOO_HIGH"
    } else {
        "NO_CUT_KEEP_BOUNDARY"
    };
    BoundaryEnergy {
        version: "boundary-energy-v1",
        owner_clarity_gain,
        foreign_pull_reduction,
        test_isolation_gain,
        state_locality_gain,
        api_surface_growth,
        adapter_leak,
        runtime_risk,
        migration_cost,
        wrapper_tax,
        owner_conflict_energy: owner_gravity.owner_conflict_energy,
        boundary_tax,
        net,
        verdict_hint,
        read_as: "Positive energy must exceed boundary tax before a split/merge is justified.",
    }
}

fn boundary_field_pass_admission(
    facts: &BoundaryFacts,
    decision: &BoundaryDecision,
    field_records: &[FieldRecord],
    route: Option<&str>,
    owner: Option<&str>,
) -> BoundaryFieldPassAdmission {
    let query = FieldRecord::synthetic(
        "boundary-query",
        FieldRecordKind::StructuralTriad,
        "boundary_decision",
        "evaluates",
        route.unwrap_or(decision.verdict).to_string(),
        route.map(str::to_string).or_else(|| decision.route.clone()),
        owner.map(str::to_string).or_else(|| decision.owner.clone()),
    );
    let mut records = field_records.to_vec();
    let mut query_witness = query.clone();
    query_witness.id = "boundary-query-witness".to_string();
    records.push(query_witness);
    records.push(FieldRecord::synthetic(
        "boundary-typed-decision",
        FieldRecordKind::StructuralTriad,
        "typed_boundary_core",
        "emits",
        decision.verdict.to_string(),
        decision.route.clone(),
        decision.owner.clone(),
    ));
    if records.is_empty() {
        records.push(FieldRecord::synthetic(
            "boundary-no-evidence",
            FieldRecordKind::StructuralTriad,
            "boundary_evidence",
            "is",
            "empty",
            route.map(str::to_string),
            owner.map(str::to_string),
        ));
    }
    let mut lenses = vec![FieldLensOperation {
        kind: FieldLensKind::Evidence,
        label: "boundary-economics".to_string(),
        strength: 1,
    }];
    if let Some(route) = route {
        lenses.push(FieldLensOperation {
            kind: FieldLensKind::Route,
            label: route.to_string(),
            strength: 2,
        });
    }
    if let Some(owner) = owner {
        lenses.push(FieldLensOperation {
            kind: FieldLensKind::Group,
            label: owner.to_string(),
            strength: 1,
        });
    }
    let anti_waves = boundary_anti_waves(facts, decision, route, owner);
    let state_hint = boundary_field_state_hint(decision, facts, anti_waves.len());
    let field_pass = run_field_pass(&FieldPassInput {
        family: FieldFamily::Structural,
        query: query.clone(),
        records,
        lenses: lenses.clone(),
        anti_waves: anti_waves.clone(),
        state_hint: state_hint.clone(),
        claim_boundary: FieldClaimBoundary {
            not_llm_ready: false,
            not_nonlinear_memory_proof: false,
            ..FieldClaimBoundary::default()
        },
    });
    BoundaryFieldPassAdmission {
        version: "boundary-field-pass-admission-v1",
        mode: "dual-run",
        query,
        lenses,
        anti_waves,
        state_hint,
        field_pass,
    }
}

fn boundary_anti_waves(
    facts: &BoundaryFacts,
    decision: &BoundaryDecision,
    route: Option<&str>,
    owner: Option<&str>,
) -> Vec<FieldAntiWaveLane> {
    let mut lanes = vec![];
    for file in facts.foreign_route_files.iter().take(8) {
        lanes.push(boundary_anti_wave_lane(
            lanes.len(),
            "foreign_route",
            "suppresses",
            file,
            route,
            owner,
            3,
        ));
    }
    if facts.owner_filter_requested && !facts.owner_filter_matched {
        lanes.push(boundary_anti_wave_lane(
            lanes.len(),
            "owner_filter",
            "rejects",
            facts.requested_owner.as_deref().unwrap_or("unknown_owner"),
            route,
            owner,
            3,
        ));
    }
    if !facts.runtime_side_effects.is_empty() && facts.tests.is_empty() {
        lanes.push(boundary_anti_wave_lane(
            lanes.len(),
            "runtime_risk",
            "requires_test",
            "runtime_side_effect_without_route_test",
            route,
            owner,
            2,
        ));
    }
    if matches!(decision.verdict, "VETO") && lanes.is_empty() {
        lanes.push(boundary_anti_wave_lane(
            lanes.len(),
            "boundary_veto",
            "blocks",
            "unsafe_refactor",
            route,
            owner,
            3,
        ));
    }
    if !decision.safe_to_edit && !matches!(decision.verdict, "VETO") {
        lanes.push(boundary_anti_wave_lane(
            lanes.len(),
            "boundary_not_safe_to_edit",
            "keeps_review_state",
            decision.verdict,
            route,
            owner,
            1,
        ));
    }
    lanes
}

fn boundary_anti_wave_lane(
    index: usize,
    subject: &str,
    relation: &str,
    object: &str,
    route: Option<&str>,
    owner: Option<&str>,
    strength: i32,
) -> FieldAntiWaveLane {
    FieldAntiWaveLane {
        id: format!("boundary-anti-wave-{index}"),
        target: object.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        route: route.map(str::to_string),
        group: owner.map(str::to_string),
        strength,
    }
}

fn boundary_field_state_hint(
    decision: &BoundaryDecision,
    facts: &BoundaryFacts,
    anti_wave_count: usize,
) -> Option<String> {
    match decision.verdict {
        "WATCH" | "SPLIT_WEAK" | "MERGE_CANDIDATE" => Some("FIELD_THIN".to_string()),
        "VETO" => Some("FIELD_CONTESTED".to_string()),
        _ if facts.files.is_empty() => Some("FIELD_THIN".to_string()),
        _ if anti_wave_count > 0 && facts.foreign_route_files.is_empty() => {
            Some("FIELD_THIN".to_string())
        }
        _ if anti_wave_count > 0 => None,
        _ => None,
    }
}

fn boundary_field_equivalence(
    decision: &BoundaryDecision,
    field_pass: &FieldPassReport,
) -> BoundaryFieldEquivalence {
    let old_rank = boundary_verdict_rank(decision.verdict);
    let field_rank = field_pass_rank(&field_pass.verdict);
    let field_not_more_permissive = field_rank <= old_rank;
    let mut mismatch_reason = vec![];
    if !field_not_more_permissive {
        mismatch_reason.push("field_more_permissive_than_typed_boundary_core".to_string());
    }
    if field_pass.version != FIELD_PASS_VERSION {
        mismatch_reason.push("field_pass_version_mismatch".to_string());
    }
    let cutover_ready = field_not_more_permissive && mismatch_reason.is_empty();
    BoundaryFieldEquivalence {
        version: "boundary-field-equivalence-v1",
        old_verdict: decision.verdict.to_string(),
        field_verdict: field_pass.verdict.clone(),
        old_safe_to_edit: decision.safe_to_edit,
        field_safe_to_answer: field_pass.safe_to_answer,
        old_rank,
        field_rank,
        field_not_more_permissive,
        cutover_ready,
        mismatch_reason,
    }
}

fn boundary_field_engine(
    decision: &BoundaryDecision,
    equivalence: &BoundaryFieldEquivalence,
) -> BoundaryFieldEngine {
    let candidate_allowed = equivalence.cutover_ready;
    let selected_verdict = if !candidate_allowed {
        decision.verdict.to_string()
    } else if equivalence.field_verdict == "VETO" {
        "VETO".to_string()
    } else if equivalence.field_verdict == "WATCH" && decision.safe_to_edit {
        "WATCH".to_string()
    } else {
        decision.verdict.to_string()
    };
    let selected_safe_to_edit = matches!(selected_verdict.as_str(), "SPLIT_STRONG" | "KEEP");
    BoundaryFieldEngine {
        version: "boundary-field-engine-v1",
        selected_engine: if candidate_allowed {
            "field-core-boundary-admission"
        } else {
            "typed-boundary-core"
        },
        candidate_allowed,
        cutover_applied: false,
        top_level_boundary_decision_preserved: true,
        selected_verdict,
        selected_safe_to_edit,
        policy: BoundaryFieldEnginePolicy {
            requires_not_more_permissive: true,
            requires_field_pass_version: FIELD_PASS_VERSION,
            can_be_stricter_than_typed_core: true,
            cannot_be_more_permissive_than_typed_core: true,
            public_json_compatibility: "boundary_decision remains the stable public contract; boundary_field_engine is the field cutover candidate",
        },
    }
}

fn boundary_verdict_rank(verdict: &str) -> u8 {
    match verdict {
        "VETO" => 0,
        "WATCH" | "SPLIT_WEAK" | "MERGE_CANDIDATE" => 1,
        "SPLIT_STRONG" | "KEEP" => 2,
        _ => 1,
    }
}

fn field_pass_rank(verdict: &str) -> u8 {
    match verdict {
        "VETO" => 0,
        "PASS" => 2,
        _ => 1,
    }
}

fn cross_owner_call_edges(facts: &BoundaryFacts) -> Vec<String> {
    facts
        .call_edges
        .iter()
        .filter_map(|edge| {
            let (caller, callee) = call_edge_files(edge)?;
            let caller_owner = facts.file_owners.get(caller)?;
            let callee_owner = facts.file_owners.get(callee)?;
            (caller_owner != callee_owner).then(|| edge.clone())
        })
        .collect()
}

fn call_edge_files(edge: &str) -> Option<(&str, &str)> {
    let (caller, rhs) = edge.split_once(" -> ")?;
    let callee = rhs.split_once("::").map_or(rhs, |(file, _)| file);
    Some((caller, callee))
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn boundary_field_records(facts: &BoundaryFacts) -> Vec<FieldRecord> {
    let mut records = vec![];
    for file in &facts.files {
        push_boundary_record(
            &mut records,
            "boundary_file",
            "belongs_to_route",
            file,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.clone()),
        );
    }
    for function in &facts.functions {
        let file = file_from_function(function);
        push_boundary_record(
            &mut records,
            "boundary_function",
            "declared_in",
            function,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for edge in &facts.public_api {
        let file = file_from_line_ref(edge);
        push_boundary_record(
            &mut records,
            "boundary_public_api",
            "exposes",
            edge,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for edge in &facts.call_edges {
        let file = edge.split(" -> ").next().unwrap_or(edge);
        push_boundary_record(
            &mut records,
            "boundary_call_edge",
            "connects",
            edge,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for item in &facts.shared_state {
        let file = file_from_line_ref(item);
        push_boundary_record(
            &mut records,
            "boundary_shared_state",
            "holds_state",
            item,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for item in &facts.runtime_side_effects {
        let file = file_from_line_ref(item);
        push_boundary_record(
            &mut records,
            "boundary_runtime_side_effect",
            "mutates_runtime",
            item,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for test in &facts.tests {
        push_boundary_record(
            &mut records,
            "boundary_test",
            "verifies_route",
            test,
            route_for_fact_file(facts, test),
            owner_for_fact_file(facts, test),
            Some(test.clone()),
        );
    }
    for foreign_file in &facts.foreign_route_files {
        push_boundary_record(
            &mut records,
            "boundary_foreign_pull",
            "pulls_foreign_route",
            foreign_file,
            route_for_fact_file(facts, foreign_file),
            owner_for_fact_file(facts, foreign_file),
            Some(foreign_file.clone()),
        );
    }
    records
}

fn push_boundary_record(
    records: &mut Vec<FieldRecord>,
    subject: &str,
    relation: &str,
    object: &str,
    route: Option<String>,
    group: Option<String>,
    evidence_ref: Option<String>,
) {
    records.push(FieldRecord {
        id: format!("boundary-record-{}", records.len()),
        kind: FieldRecordKind::StructuralTriad,
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        route,
        group,
        confidence: 255,
        evidence_ref,
    });
}

fn route_for_fact_file(facts: &BoundaryFacts, file: &str) -> Option<String> {
    facts.file_routes.get(file).cloned()
}

fn owner_for_fact_file(facts: &BoundaryFacts, file: &str) -> Option<String> {
    facts.file_owners.get(file).cloned()
}

fn file_from_function(function: &str) -> &str {
    function.split_once("::").map_or(function, |(file, _)| file)
}

fn file_from_line_ref(item: &str) -> &str {
    item.split_once(':').map_or(item, |(file, _)| file)
}

fn extract_functions(source: &str) -> Vec<String> {
    let mut out = vec![];
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed
            .strip_prefix("fn ")
            .or_else(|| trimmed.strip_prefix("pub fn "))
            .or_else(|| trimmed.strip_prefix("pub(crate) fn "))
        {
            let name = rest
                .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .next()
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

fn is_public_api_line(line: &str) -> bool {
    line.starts_with("pub fn ")
        || line.starts_with("pub(crate) fn ")
        || line.starts_with("pub struct ")
        || line.starts_with("pub enum ")
        || line.starts_with("pub trait ")
}

fn is_shared_state_line(line: &str) -> bool {
    [
        "static ",
        "OnceLock",
        "Mutex",
        "RwLock",
        "Atomic",
        "thread_local!",
        "Arc<Mutex",
    ]
    .iter()
    .any(|needle| line.contains(needle))
}

fn is_runtime_side_effect_line(line: &str) -> bool {
    [
        "Command::new",
        "spawn(",
        "systemctl",
        "gsettings",
        "dbus",
        "DBus",
        "ibus",
        "fs::write",
        "std::process",
    ]
    .iter()
    .any(|needle| line.contains(needle))
}

fn is_thin_wrapper(source: &str, functions: &[String]) -> bool {
    let non_empty = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .count();
    !functions.is_empty()
        && non_empty <= 24
        && !source.contains("struct ")
        && !source.contains("enum ")
        && !source.contains("static ")
}

fn owner_evidence(facts: &BoundaryFacts) -> Vec<String> {
    facts
        .owners
        .iter()
        .map(|owner| format!("owner:{owner}"))
        .collect()
}

fn score_component(score: i32, evidence: Vec<String>) -> BoundaryScoreComponent {
    BoundaryScoreComponent {
        score,
        counted: !evidence.is_empty(),
        evidence: sample(&evidence, 16),
    }
}

fn sample(items: &[String], cap: usize) -> Vec<String> {
    items.iter().take(cap).cloned().collect()
}

fn sample_values(items: &[Value], cap: usize) -> Vec<Value> {
    items.iter().take(cap).cloned().collect()
}

fn empty_components() -> Value {
    serde_json::to_value(BoundaryScoreComponents::empty()).unwrap_or_else(|_| json!({}))
}

fn must_not_change(verdict: BoundaryVerdict) -> Vec<String> {
    match verdict {
        BoundaryVerdict::SplitStrong | BoundaryVerdict::SplitWeak => [
            "foreign routes",
            "public behavior outside contract",
            "runtime side effects without tests",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        BoundaryVerdict::MergeCandidate => {
            ["owner public API without human review", "runtime behavior"]
                .into_iter()
                .map(str::to_string)
                .collect()
        }
        BoundaryVerdict::Keep => vec!["module boundary".to_string()],
        _ => vec!["code boundary".to_string()],
    }
}

fn required_tests(facts: &BoundaryFacts) -> Vec<String> {
    let mut tests = vec![];
    tests.extend(facts.runtime_checks.iter().take(5).cloned());
    tests.extend(facts.tests.iter().take(5).cloned());
    tests.sort();
    tests.dedup();
    if !tests.is_empty() {
        tests
    } else if !facts.runtime_side_effects.is_empty() {
        vec!["add route-specific runtime smoke before refactor".to_string()]
    } else {
        vec!["add or identify route-specific test before changing boundary".to_string()]
    }
}

fn repair_tasks(
    verdict: BoundaryVerdict,
    owner_filter_failed: bool,
    repo_wide_route_pressure: bool,
) -> Vec<String> {
    if owner_filter_failed {
        return [
            "use an owner that matches the selected route atlas",
            "rebuild atlas if the owner map is stale",
            "do not expand to the whole route after owner mismatch",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
    }
    if repo_wide_route_pressure && verdict == BoundaryVerdict::Watch {
        return [
            "build or refresh the route atlas",
            "select the route with boundary pressure",
            "rerun boundary economics with --atlas, --route, and --owner before cutting",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
    }
    match verdict {
        BoundaryVerdict::Watch => vec![
            "collect owner, route, public API, state, runtime, and test evidence before split/merge"
                .to_string(),
        ],
        BoundaryVerdict::Veto => [
            "do not cut across foreign route",
            "split the refactor by route owner",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        BoundaryVerdict::SplitStrong => [
            "extract behind owner-owned API",
            "keep forbidden routes out of diff",
            "run required tests",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        BoundaryVerdict::SplitWeak => [
            "prepare a small mechanical step only",
            "ask for human review before semantic changes",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        BoundaryVerdict::MergeCandidate => [
            "merge wrapper into owner module",
            "or make helper private behind owner public method",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        BoundaryVerdict::Keep => vec![],
    }
}
