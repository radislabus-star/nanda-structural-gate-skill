use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::field_core::{
    run_field_pass, FieldAntiWaveLane, FieldClaimBoundary, FieldFamily, FieldLensKind,
    FieldLensOperation, FieldPassInput, FieldPassReport, FieldRecord, FieldRecordKind,
    FIELD_PASS_VERSION,
};

#[derive(Debug, Clone, Serialize)]
struct BoundaryDiffFacts {
    action_id: String,
    route: Option<String>,
    shared_contract: Option<Value>,
    version_bump: Option<Value>,
    changed_files: Vec<String>,
    file_routes: BTreeMap<String, String>,
    changed_routes: Vec<String>,
    changed_functions: Vec<String>,
    added_public_api: Vec<String>,
    changed_runtime_side_effects: Vec<String>,
    changed_tests: Vec<String>,
    shared_candidates: Vec<String>,
    foreign_files: Vec<String>,
    foreign_routes: Vec<String>,
    suggested_shared_actions: Vec<String>,
    diff_source: Option<Value>,
    source_mismatch: bool,
    empty_or_unreadable: bool,
    shared_allows_crossing: bool,
    route_crossing: bool,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryDiffDecision {
    verdict: &'static str,
    diff_verdict: &'static str,
    safe_to_edit: bool,
    reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryDiffFieldRecords {
    version: &'static str,
    owner: &'static str,
    record_count: usize,
    changed_file_records: usize,
    route_records: usize,
    public_api_records: usize,
    runtime_side_effect_records: usize,
    test_records: usize,
    foreign_pull_records: usize,
    sample: Vec<FieldRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryDiffFieldEquivalence {
    version: &'static str,
    typed_verdict: String,
    field_verdict: String,
    typed_rank: u8,
    field_rank: u8,
    field_not_more_permissive: bool,
    mismatch_reason: Vec<String>,
}

pub(super) fn boundary_guard_diff(
    atlas: &Value,
    action_id: &str,
    diff: &str,
    diff_source: Option<Value>,
) -> Value {
    let facts = collect_diff_facts(atlas, action_id, diff, diff_source);
    let decision = decide_diff(&facts);
    diff_report(atlas, &facts, &decision, None)
}

pub(super) fn boundary_guard_diff_unreadable(
    atlas: &Value,
    action_id: &str,
    diff_source: Option<Value>,
    diff_error: &str,
) -> Value {
    let mut facts = collect_diff_facts(atlas, action_id, "", diff_source);
    facts.empty_or_unreadable = true;
    let decision = BoundaryDiffDecision {
        verdict: "WATCH",
        diff_verdict: "DIFF_WATCH",
        safe_to_edit: false,
        reason: "empty_or_unreadable_diff",
    };
    diff_report(atlas, &facts, &decision, Some(diff_error.to_string()))
}

fn collect_diff_facts(
    atlas: &Value,
    action_id: &str,
    diff: &str,
    diff_source: Option<Value>,
) -> BoundaryDiffFacts {
    let route = route_for_action(atlas, action_id);
    let mut changed_files = diff_changed_files(diff);
    changed_files.sort();
    changed_files.dedup();

    let shared_contract = shared_contract_for_action(atlas, action_id).cloned();
    let shared_allowed_routes = shared_contract
        .as_ref()
        .and_then(|contract| contract["allowed_routes"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let shared_candidates = shared_candidate_files(shared_contract.as_ref(), &changed_files);
    let allowed = route
        .as_ref()
        .and_then(|route| atlas["routes"][route]["allowed_files"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let file_routes = changed_files
        .iter()
        .map(|file| (file.clone(), route_for_changed_file(atlas, file)))
        .collect::<BTreeMap<_, _>>();
    let changed_routes = file_routes.values().cloned().collect::<BTreeSet<_>>();
    let changed_route_list = changed_routes.iter().cloned().collect::<Vec<_>>();
    let foreign_files = if shared_contract.is_some() {
        changed_files
            .iter()
            .filter(|file| {
                let changed_route = file_routes
                    .get(file.as_str())
                    .cloned()
                    .unwrap_or_else(|| route_for_changed_file(atlas, file));
                !shared_allowed_routes.contains(&changed_route)
            })
            .cloned()
            .collect::<Vec<_>>()
    } else {
        changed_files
            .iter()
            .filter(|file| !allowed.iter().any(|allowed| path_matches(file, allowed)))
            .cloned()
            .collect::<Vec<_>>()
    };
    let foreign_routes = if shared_contract.is_some() {
        changed_routes
            .iter()
            .filter(|changed_route| !shared_allowed_routes.contains(*changed_route))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        changed_routes
            .iter()
            .filter(|changed_route| route.as_ref() != Some(*changed_route))
            .cloned()
            .collect::<Vec<_>>()
    };
    let source_mismatch = diff_source
        .as_ref()
        .and_then(|source| source["mismatch"].as_bool())
        .unwrap_or(false);
    let empty_or_unreadable = diff.trim().is_empty() || changed_files.is_empty();
    let version_bump = if action_id == "shared.version_bump_contract" {
        Some(check_version_bump_contract(atlas, diff, &changed_files))
    } else {
        None
    };
    let shared_allows_crossing = shared_contract.is_some()
        && !changed_files.is_empty()
        && changed_routes
            .iter()
            .all(|changed_route| shared_allowed_routes.contains(changed_route));
    let route_crossing =
        changed_routes.len() > 1 || !foreign_files.is_empty() || !foreign_routes.is_empty();
    let changed_functions = diff_changed_functions(diff);
    let added_public_api = diff_added_public_api(diff);
    let changed_runtime_side_effects = diff_runtime_side_effects(diff);
    let changed_tests = changed_files
        .iter()
        .filter(|file| is_test_file(file))
        .cloned()
        .collect::<Vec<_>>();
    let suggested_shared_actions = suggested_shared_actions(atlas, &changed_routes, &changed_files);

    BoundaryDiffFacts {
        action_id: action_id.to_string(),
        route,
        shared_contract,
        version_bump,
        changed_files,
        file_routes,
        changed_routes: changed_route_list,
        changed_functions,
        added_public_api,
        changed_runtime_side_effects,
        changed_tests,
        shared_candidates,
        foreign_files,
        foreign_routes,
        suggested_shared_actions,
        diff_source,
        source_mismatch,
        empty_or_unreadable,
        shared_allows_crossing,
        route_crossing,
    }
}

fn decide_diff(facts: &BoundaryDiffFacts) -> BoundaryDiffDecision {
    let (verdict, diff_verdict, reason) = if facts.empty_or_unreadable {
        ("WATCH", "DIFF_WATCH", "empty_or_unreadable_diff")
    } else if facts.source_mismatch {
        ("WATCH", "DIFF_WATCH", "diff_source_repo_mismatch")
    } else if let Some(version_bump) = facts.version_bump.as_ref() {
        if !version_bump["scope_ok"].as_bool().unwrap_or(false) {
            ("VETO", "DIFF_VETO", "version_bump_scope_violation")
        } else if !version_bump["consistent"].as_bool().unwrap_or(false) {
            ("WATCH", "DIFF_WATCH", "version_bump_inconsistent")
        } else {
            ("PASS", "DIFF_KEEP", "version_bump_contract_pass")
        }
    } else if (facts.route.is_none() && facts.shared_contract.is_none())
        || (facts.route_crossing && !facts.shared_allows_crossing)
    {
        (
            "VETO",
            "DIFF_SHARED_CONTRACT_REQUIRED",
            "route_crossing_requires_shared_contract",
        )
    } else if !facts.changed_runtime_side_effects.is_empty() && facts.changed_tests.is_empty() {
        (
            "WATCH",
            "DIFF_TESTS_REQUIRED",
            "runtime_side_effect_requires_test",
        )
    } else if !facts.added_public_api.is_empty() {
        ("WATCH", "DIFF_WATCH", "public_api_growth_requires_review")
    } else if facts.route_crossing && facts.shared_allows_crossing {
        ("PASS", "DIFF_KEEP", "shared_contract_allows_route_crossing")
    } else {
        ("PASS", "DIFF_KEEP", "diff_stays_inside_allowed_routes")
    };
    BoundaryDiffDecision {
        verdict,
        diff_verdict,
        safe_to_edit: verdict == "PASS",
        reason,
    }
}

fn diff_report(
    _atlas: &Value,
    facts: &BoundaryDiffFacts,
    decision: &BoundaryDiffDecision,
    diff_error: Option<String>,
) -> Value {
    let route_crossing_decision =
        if facts.action_id == "shared.version_bump_contract" && decision.verdict == "PASS" {
            "allowed by shared.version_bump_contract".to_string()
        } else if decision.verdict == "PASS" && facts.shared_contract.is_some() {
            format!("allowed by {}", facts.action_id)
        } else if facts.route_crossing {
            "allowed only if action_id is an explicit shared contract for these routes".to_string()
        } else if facts.empty_or_unreadable {
            "diff unreadable".to_string()
        } else {
            "single-route diff".to_string()
        };
    let shared_contract = facts.shared_contract.as_ref().map(|contract| {
        json!({
            "action_id": facts.action_id,
            "allowed_routes": contract["allowed_routes"].clone(),
            "reason": contract["reason"].clone(),
            "contract_scope": contract["contract_scope"].clone()
        })
    });
    let field_records = diff_field_records(facts);
    let field_record_bridge = BoundaryDiffFieldRecords::from_facts(facts, &field_records);
    let field_pass = diff_field_pass(facts, decision, &field_records);
    let field_equivalence = diff_field_equivalence(decision, &field_pass);
    let selected_verdict = if field_equivalence.field_not_more_permissive
        && field_pass.verdict == "VETO"
        && decision.verdict != "VETO"
    {
        "VETO"
    } else if field_equivalence.field_not_more_permissive
        && field_pass.verdict == "WATCH"
        && decision.verdict == "PASS"
    {
        "WATCH"
    } else {
        decision.verdict
    };

    json!({
        "mode": "guard-diff",
        "verdict": decision.verdict,
        "safe_to_edit": decision.safe_to_edit,
        "action_id": facts.action_id,
        "route": facts.route,
        "shared_contract": shared_contract,
        "version_bump": facts.version_bump,
        "reason": decision.reason,
        "diff_source": facts.diff_source,
        "diff_error": diff_error,
        "changed_files": facts.changed_files,
        "changed_routes": facts.changed_routes,
        "changed_functions": facts.changed_functions,
        "added_public_api": facts.added_public_api,
        "changed_runtime_side_effects": facts.changed_runtime_side_effects,
        "changed_tests": facts.changed_tests,
        "shared_candidates": facts.shared_candidates,
        "foreign_files": facts.foreign_files,
        "foreign_routes": facts.foreign_routes,
        "route_crossing_report": {
            "changed_routes": facts.changed_routes,
            "shared_candidates": facts.shared_candidates,
            "suggested_shared_actions": facts.suggested_shared_actions,
            "contract_scope": if facts.action_id == "shared.version_bump_contract" { json!("version metadata only") } else { Value::Null },
            "decision": route_crossing_decision
        },
        "boundary_diff_kernel": {
            "version": "boundary-diff-kernel-v1",
            "owner": "field_core::boundary::diff",
            "commands_are_wrappers": true,
            "diff_verdict": decision.diff_verdict,
            "typed_verdict": decision.verdict,
            "file_routes": facts.file_routes,
            "selected_verdict": selected_verdict,
            "selected_safe_to_edit": selected_verdict == "PASS",
            "field_records": field_record_bridge,
            "field_pass": field_pass,
            "field_equivalence": field_equivalence
        },
        "repair_queue": guard_diff_repairs(
            decision.verdict,
            decision.reason,
            &facts.suggested_shared_actions,
        ),
        "read_as": "Diff guard: changed files must stay inside the selected route atlas capsule."
    })
}

impl BoundaryDiffFieldRecords {
    fn from_facts(facts: &BoundaryDiffFacts, records: &[FieldRecord]) -> Self {
        Self {
            version: "boundary-diff-field-records-v1",
            owner: "field_core::boundary::diff",
            record_count: records.len(),
            changed_file_records: facts.changed_files.len(),
            route_records: facts.changed_routes.len(),
            public_api_records: facts.added_public_api.len(),
            runtime_side_effect_records: facts.changed_runtime_side_effects.len(),
            test_records: facts.changed_tests.len(),
            foreign_pull_records: facts.foreign_files.len() + facts.foreign_routes.len(),
            sample: records.iter().take(16).cloned().collect(),
        }
    }
}

fn diff_field_records(facts: &BoundaryDiffFacts) -> Vec<FieldRecord> {
    let mut records = vec![];
    for file in &facts.changed_files {
        let route = Some(route_for_file_from_facts(facts, file));
        push_diff_record(
            &mut records,
            "diff_changed_file",
            "belongs_to_route",
            file,
            route,
            None,
            Some(file.clone()),
        );
    }
    for route in &facts.changed_routes {
        push_diff_record(
            &mut records,
            "diff_changed_route",
            "is_touched",
            route,
            Some(route.clone()),
            None,
            None,
        );
    }
    for item in &facts.added_public_api {
        let file = file_from_diff_ref(item);
        push_diff_record(
            &mut records,
            "diff_public_api",
            "adds_surface",
            item,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.to_string()),
        );
    }
    for item in &facts.changed_runtime_side_effects {
        let file = file_from_diff_ref(item);
        push_diff_record(
            &mut records,
            "diff_runtime_side_effect",
            "requires_test",
            item,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.to_string()),
        );
    }
    for file in &facts.changed_tests {
        push_diff_record(
            &mut records,
            "diff_test",
            "verifies_route",
            file,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.clone()),
        );
    }
    for file in &facts.foreign_files {
        push_diff_record(
            &mut records,
            "diff_foreign_file",
            "pulls_foreign_route",
            file,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.clone()),
        );
    }
    for route in &facts.foreign_routes {
        push_diff_record(
            &mut records,
            "diff_foreign_route",
            "crosses_boundary",
            route,
            Some(route.clone()),
            None,
            None,
        );
    }
    records
}

fn push_diff_record(
    records: &mut Vec<FieldRecord>,
    subject: &str,
    relation: &str,
    object: &str,
    route: Option<String>,
    group: Option<String>,
    evidence_ref: Option<String>,
) {
    records.push(FieldRecord {
        id: format!("boundary-diff-record-{}", records.len()),
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

fn diff_field_pass(
    facts: &BoundaryDiffFacts,
    decision: &BoundaryDiffDecision,
    field_records: &[FieldRecord],
) -> FieldPassReport {
    let route = facts
        .route
        .clone()
        .or_else(|| facts.changed_routes.first().cloned());
    let query = FieldRecord::synthetic(
        "boundary-diff-query",
        FieldRecordKind::StructuralTriad,
        "guard_diff",
        "checks",
        decision.reason.to_string(),
        route.clone(),
        Some(facts.action_id.clone()),
    );
    let mut records = field_records.to_vec();
    let mut witness = query.clone();
    witness.id = "boundary-diff-query-witness".to_string();
    records.push(witness);
    records.push(FieldRecord::synthetic(
        "boundary-diff-typed-decision",
        FieldRecordKind::StructuralTriad,
        "typed_diff_core",
        "emits",
        decision.verdict.to_string(),
        route.clone(),
        Some(facts.action_id.clone()),
    ));
    let mut lenses = vec![FieldLensOperation {
        kind: FieldLensKind::Evidence,
        label: "boundary-diff".to_string(),
        strength: 1,
    }];
    if let Some(route) = route.as_deref() {
        lenses.push(FieldLensOperation {
            kind: FieldLensKind::Route,
            label: route.to_string(),
            strength: 2,
        });
    }
    lenses.push(FieldLensOperation {
        kind: FieldLensKind::Group,
        label: facts.action_id.clone(),
        strength: 1,
    });
    run_field_pass(&FieldPassInput {
        family: FieldFamily::Structural,
        query,
        records,
        lenses,
        anti_waves: diff_anti_waves(facts, decision),
        state_hint: diff_state_hint(decision),
        claim_boundary: FieldClaimBoundary {
            not_llm_ready: false,
            not_nonlinear_memory_proof: false,
            ..FieldClaimBoundary::default()
        },
    })
}

fn diff_anti_waves(
    facts: &BoundaryDiffFacts,
    decision: &BoundaryDiffDecision,
) -> Vec<FieldAntiWaveLane> {
    let mut lanes = vec![];
    for file in facts.foreign_files.iter().take(6) {
        lanes.push(diff_anti_wave_lane(
            lanes.len(),
            "diff_foreign_file",
            "suppresses",
            file,
            Some(route_for_file_from_facts(facts, file)),
            Some(facts.action_id.clone()),
            3,
        ));
    }
    for route in facts.foreign_routes.iter().take(6) {
        lanes.push(diff_anti_wave_lane(
            lanes.len(),
            "diff_foreign_route",
            "suppresses",
            route,
            Some(route.clone()),
            Some(facts.action_id.clone()),
            3,
        ));
    }
    if !facts.changed_runtime_side_effects.is_empty() && facts.changed_tests.is_empty() {
        lanes.push(diff_anti_wave_lane(
            lanes.len(),
            "diff_runtime_side_effect",
            "requires_test",
            "runtime_side_effect_without_test",
            facts.route.clone(),
            Some(facts.action_id.clone()),
            2,
        ));
    }
    if !facts.added_public_api.is_empty() {
        lanes.push(diff_anti_wave_lane(
            lanes.len(),
            "diff_public_api",
            "requires_review",
            "public_api_growth",
            facts.route.clone(),
            Some(facts.action_id.clone()),
            1,
        ));
    }
    if matches!(decision.verdict, "VETO") && lanes.is_empty() {
        lanes.push(diff_anti_wave_lane(
            lanes.len(),
            "diff_veto",
            "blocks",
            decision.reason,
            facts.route.clone(),
            Some(facts.action_id.clone()),
            3,
        ));
    }
    lanes
}

fn diff_anti_wave_lane(
    index: usize,
    subject: &str,
    relation: &str,
    object: &str,
    route: Option<String>,
    group: Option<String>,
    strength: i32,
) -> FieldAntiWaveLane {
    FieldAntiWaveLane {
        id: format!("boundary-diff-anti-wave-{index}"),
        target: object.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        route,
        group,
        strength,
    }
}

fn diff_state_hint(decision: &BoundaryDiffDecision) -> Option<String> {
    match decision.verdict {
        "VETO" => Some("FIELD_CONTESTED".to_string()),
        "WATCH" => Some("FIELD_THIN".to_string()),
        _ => None,
    }
}

fn diff_field_equivalence(
    decision: &BoundaryDiffDecision,
    field_pass: &FieldPassReport,
) -> BoundaryDiffFieldEquivalence {
    let typed_rank = verdict_rank(decision.verdict);
    let field_rank = field_pass_rank(&field_pass.verdict);
    let field_not_more_permissive = field_rank <= typed_rank;
    let mut mismatch_reason = vec![];
    if !field_not_more_permissive {
        mismatch_reason.push("field_more_permissive_than_diff_core".to_string());
    }
    if field_pass.version != FIELD_PASS_VERSION {
        mismatch_reason.push("field_pass_version_mismatch".to_string());
    }
    BoundaryDiffFieldEquivalence {
        version: "boundary-diff-field-equivalence-v1",
        typed_verdict: decision.verdict.to_string(),
        field_verdict: field_pass.verdict.clone(),
        typed_rank,
        field_rank,
        field_not_more_permissive,
        mismatch_reason,
    }
}

fn verdict_rank(verdict: &str) -> u8 {
    match verdict {
        "VETO" => 0,
        "WATCH" => 1,
        "PASS" => 2,
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

fn route_for_action(atlas: &Value, action_id: &str) -> Option<String> {
    let prefix = action_id.split('.').next().unwrap_or("");
    atlas["action_prefixes"][prefix]
        .as_str()
        .map(str::to_string)
        .or_else(|| {
            let normalized = format!("{}-flow", prefix.replace('_', "-"));
            atlas["routes"][&normalized]
                .is_object()
                .then_some(normalized)
        })
}

fn shared_contract_for_action<'a>(atlas: &'a Value, action_id: &str) -> Option<&'a Value> {
    atlas["shared_contracts"][action_id].as_object()?;
    Some(&atlas["shared_contracts"][action_id])
}

fn route_for_changed_file(atlas: &Value, file: &str) -> String {
    let mut exact = vec![];
    let mut prefix = vec![];
    if let Some(routes) = atlas["routes"].as_object() {
        for (route, node) in routes {
            for allowed in node["allowed_files"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                if file == allowed {
                    exact.push(route.clone());
                } else if path_matches(file, allowed) {
                    prefix.push(route.clone());
                }
            }
        }
    }
    exact.sort();
    exact
        .into_iter()
        .next()
        .or_else(|| {
            prefix.sort();
            prefix.into_iter().next()
        })
        .unwrap_or_else(|| generic_route_for_path(file))
}

fn route_for_file_from_facts(facts: &BoundaryDiffFacts, file: &str) -> String {
    facts
        .file_routes
        .get(file)
        .cloned()
        .or_else(|| facts.changed_routes.first().cloned())
        .unwrap_or_else(|| "unknown-flow".to_string())
}

fn generic_route_for_path(file: &str) -> String {
    let lower = file.to_ascii_lowercase();
    if lower.contains("test") || lower.contains("spec") {
        "test-flow".to_string()
    } else if lower.contains("config") || lower.ends_with(".toml") {
        "config-flow".to_string()
    } else if lower.contains("ui") || lower.contains("tray") || lower.contains("extension/") {
        "ui-status-flow".to_string()
    } else if lower.contains("install") || lower.contains("script") {
        "install-flow".to_string()
    } else if lower.contains("runtime") || lower.contains("daemon") {
        "runtime-flow".to_string()
    } else {
        "source-flow".to_string()
    }
}

fn shared_candidate_files(contract: Option<&Value>, changed: &[String]) -> Vec<String> {
    let patterns = contract
        .and_then(|contract| contract["shared_candidates"].as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|item| item.to_ascii_lowercase())
        .collect::<Vec<_>>();
    changed
        .iter()
        .filter(|file| {
            let lower = file.to_ascii_lowercase();
            patterns.iter().any(|pattern| lower.contains(pattern))
        })
        .cloned()
        .collect()
}

fn suggested_shared_actions(
    atlas: &Value,
    changed_routes: &BTreeSet<String>,
    changed: &[String],
) -> Vec<String> {
    atlas["shared_contracts"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(action, contract)| {
            let allowed_routes = contract["allowed_routes"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            let has_shared_candidate = !shared_candidate_files(Some(contract), changed).is_empty();
            (has_shared_candidate
                && changed_routes
                    .iter()
                    .all(|route| allowed_routes.contains(route)))
            .then(|| action.clone())
        })
        .collect()
}

fn check_version_bump_contract(atlas: &Value, diff: &str, changed: &[String]) -> Value {
    let repo = atlas["repo"]
        .as_str()
        .or_else(|| atlas["input"].as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let scope_violations = changed
        .iter()
        .filter(|file| !is_version_owned_file(&repo, file, diff))
        .cloned()
        .collect::<Vec<_>>();
    let cargo_package = parse_cargo_toml_package(&repo.join("Cargo.toml"));
    let cargo_package_name = cargo_package.as_ref().map(|package| package.0.as_str());
    let cargo_version = cargo_package.as_ref().map(|package| package.1.clone());
    let mut checks = vec![];
    let mut violations = vec![];

    if let Some(version) = cargo_version.as_deref() {
        checks.push(json!({"name": "cargo_toml_package_version", "ok": true, "value": version}));
    } else {
        violations.push("Cargo.toml package.version not found".to_string());
        checks.push(json!({"name": "cargo_toml_package_version", "ok": false}));
    }

    if changed.iter().any(|file| file == "Cargo.lock") {
        match (
            cargo_package_name,
            cargo_version.as_deref(),
            cargo_package_name
                .and_then(|name| parse_cargo_lock_package_version(&repo.join("Cargo.lock"), name)),
        ) {
            (Some(package), Some(expected), Some(actual)) if expected == actual => {
                checks.push(json!({"name": "cargo_lock_package_version", "ok": true, "package": package, "value": actual}));
            }
            (Some(package), Some(expected), Some(actual)) => {
                violations.push(format!(
                    "Cargo.lock package {package} version {actual} != Cargo.toml {expected}"
                ));
                checks.push(json!({"name": "cargo_lock_package_version", "ok": false, "package": package, "expected": expected, "actual": actual}));
            }
            (Some(package), _, None) => {
                violations.push(format!("Cargo.lock package {package} version not found"));
                checks.push(
                    json!({"name": "cargo_lock_package_version", "ok": false, "package": package}),
                );
            }
            (Some(package), None, Some(actual)) => {
                violations.push(format!(
                    "Cargo.toml package {package} version not found while Cargo.lock has {actual}"
                ));
                checks.push(
                    json!({"name": "cargo_lock_package_version", "ok": false, "package": package, "actual": actual}),
                );
            }
            (None, _, _) => {
                violations.push("Cargo.toml package.name not found".to_string());
                checks.push(json!({"name": "cargo_lock_package_version", "ok": false}));
            }
        }
    }

    for file in changed
        .iter()
        .filter(|file| file.ends_with("metadata.json"))
    {
        match (cargo_version.as_deref(), read_json_file(&repo.join(file))) {
            (Some(expected), Some(json_value)) => {
                let name_ok = json_value["version-name"].as_str() == Some(expected);
                let numeric_ok = json_value["version"]
                    .as_i64()
                    .is_some_and(|value| version_number_matches(expected, value));
                if !name_ok {
                    violations.push(format!("{file} version-name does not match {expected}"));
                }
                if !numeric_ok {
                    violations.push(format!(
                        "{file} numeric version does not match patch/build for {expected}"
                    ));
                }
                checks.push(json!({
                    "name": "metadata_json_version",
                    "file": file,
                    "ok": name_ok && numeric_ok,
                    "version_name": json_value["version-name"].clone(),
                    "numeric_version": json_value["version"].clone()
                }));
            }
            _ => {
                violations.push(format!("{file} metadata.json unreadable"));
                checks.push(json!({"name": "metadata_json_version", "file": file, "ok": false}));
            }
        }
    }

    for file in changed.iter().filter(|file| is_js_version_file(file)) {
        let versions = parse_app_versions(&repo.join(file));
        if versions.is_empty() {
            continue;
        }
        let file_ok = cargo_version
            .as_deref()
            .is_some_and(|expected| versions.iter().all(|actual| actual == expected));
        if !file_ok {
            violations.push(format!("{file} APP_VERSION does not match Cargo.toml"));
        }
        checks.push(json!({
            "name": "js_app_version",
            "file": file,
            "ok": file_ok,
            "versions": versions
        }));
    }

    if let Some(expected) = cargo_version.as_deref() {
        for file in changed
            .iter()
            .filter(|file| should_scan_for_stale_version(file))
        {
            let stale = stale_version_tokens_for_file(file, &repo.join(file), expected);
            if !stale.is_empty() {
                violations.push(format!(
                    "{file} still contains stale version(s): {}",
                    stale.join(", ")
                ));
                checks.push(
                    json!({"name": "stale_version_scan", "file": file, "ok": false, "stale": stale}),
                );
            }
        }
    }

    json!({
        "contract": "shared.version_bump_contract",
        "contract_scope": "version metadata only",
        "scope_ok": scope_violations.is_empty(),
        "consistent": scope_violations.is_empty() && violations.is_empty() && cargo_version.is_some(),
        "cargo_package": cargo_package_name,
        "cargo_version": cargo_version,
        "scope_violations": scope_violations,
        "violations": violations,
        "checks": checks
    })
}

fn is_version_owned_file(repo: &Path, file: &str, diff: &str) -> bool {
    matches!(file, "Cargo.toml" | "Cargo.lock" | "VERSIONING.md")
        || (file.starts_with("extension/")
            && matches!(
                Path::new(file).file_name().and_then(|name| name.to_str()),
                Some("metadata.json" | "prefs.js" | "settings.js" | "tray_support.js")
            ))
        || (matches!(file, "README.md" | "HOW_IT_WORKS.md")
            && file_or_diff_contains_version(&repo.join(file), file, diff))
}

fn file_or_diff_contains_version(path: &Path, file: &str, diff: &str) -> bool {
    fs::read_to_string(path)
        .map(|content| contains_semver(&content))
        .unwrap_or(false)
        || diff_block_contains_semver(file, diff)
}

fn diff_block_contains_semver(file: &str, diff: &str) -> bool {
    let mut in_file = false;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("diff --git a/") {
            let file_marker = format!(" b/{file}");
            in_file = path.ends_with(&file_marker);
            continue;
        }
        if in_file && contains_semver(line) {
            return true;
        }
    }
    false
}

fn parse_cargo_toml_version(path: &Path) -> Option<String> {
    parse_cargo_toml_package(path).map(|package| package.1)
}

fn parse_cargo_toml_package(path: &Path) -> Option<(String, String)> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_package = false;
    let mut name = None;
    let mut version = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            if !in_package && name.is_some() && version.is_some() {
                break;
            }
        } else if in_package && trimmed.starts_with("name") {
            name = quoted_value(trimmed);
        } else if in_package && trimmed.starts_with("version") {
            version = quoted_value(trimmed);
        }
    }
    Some((name?, version?))
}

fn parse_cargo_lock_package_version(path: &Path, package_name: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_package = false;
    let mut is_target_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if in_package && is_target_package {
                return None;
            }
            in_package = true;
            is_target_package = false;
            continue;
        }
        if !in_package {
            continue;
        }
        if trimmed.starts_with("name") && quoted_value(trimmed).as_deref() == Some(package_name) {
            is_target_package = true;
        } else if is_target_package && trimmed.starts_with("version") {
            return quoted_value(trimmed);
        }
    }
    None
}

fn quoted_value(line: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let mut quoted = line.split(quote);
        let _before = quoted.next()?;
        if let Some(value) = quoted.next() {
            return Some(value.trim().to_string()).filter(|value| !value.is_empty());
        }
    }
    let (_, value) = line.split_once('=')?;
    Some(
        value
            .trim()
            .trim_end_matches(';')
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
    .filter(|value| !value.is_empty())
}

fn read_json_file(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn version_number_matches(version: &str, value: i64) -> bool {
    let parts = version
        .split('.')
        .filter_map(|part| part.parse::<i64>().ok())
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        return false;
    }
    let patch = parts[2];
    let packed = parts[0] * 10_000 + parts[1] * 100 + parts[2];
    value == patch || value == packed
}

fn is_js_version_file(file: &str) -> bool {
    matches!(
        Path::new(file).file_name().and_then(|name| name.to_str()),
        Some("prefs.js" | "settings.js" | "tray_support.js")
    )
}

fn parse_app_versions(path: &Path) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return vec![],
    };
    content
        .lines()
        .filter_map(parse_app_version_definition)
        .collect()
}

fn should_scan_for_stale_version(file: &str) -> bool {
    file == "Cargo.toml" || file.ends_with("metadata.json") || is_js_version_file(file)
}

fn parse_app_version_definition(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed
        .strip_prefix("export ")
        .unwrap_or(trimmed)
        .trim_start();
    let rest = rest.strip_prefix("const ")?.trim_start();
    let rest = rest.strip_prefix("APP_VERSION")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value = rest[quote.len_utf8()..].split(quote).next()?.trim();
    Some(value.to_string()).filter(|value| !value.is_empty())
}

fn stale_version_tokens_for_file(file: &str, path: &Path, expected: &str) -> Vec<String> {
    if file == "Cargo.toml" {
        return parse_cargo_toml_version(path)
            .filter(|version| version != expected)
            .into_iter()
            .collect();
    }
    if file.ends_with("metadata.json") {
        return read_json_file(path)
            .and_then(|json_value| json_value["version-name"].as_str().map(str::to_string))
            .filter(|version| version != expected)
            .into_iter()
            .collect();
    }
    if is_js_version_file(file) {
        return parse_app_versions(path)
            .into_iter()
            .filter(|version| version != expected)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
    }
    vec![]
}

fn contains_semver(value: &str) -> bool {
    !semver_tokens(value).is_empty()
}

fn semver_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .filter(|part| {
            let pieces = part.split('.').collect::<Vec<_>>();
            pieces.len() == 3
                && pieces
                    .iter()
                    .all(|piece| !piece.is_empty() && piece.chars().all(|ch| ch.is_ascii_digit()))
        })
        .map(str::to_string)
        .collect()
}

fn diff_changed_files(diff: &str) -> Vec<String> {
    let mut out = vec![];
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            if rest != "/dev/null" {
                out.push(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            if rest != "/dev/null" {
                out.push(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("diff --git a/") {
            if let Some((left, _)) = rest.split_once(" b/") {
                out.push(left.to_string());
            }
        }
    }
    out
}

fn diff_changed_functions(diff: &str) -> Vec<String> {
    let mut current_file = None::<String>;
    let mut out = vec![];
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git a/") {
            current_file = rest.split_once(" b/").map(|(left, _)| left.to_string());
            continue;
        }
        if let Some(hunk) = line.strip_prefix("@@") {
            if let Some(file) = current_file.as_deref() {
                let symbol = hunk
                    .rsplit_once("@@")
                    .map(|(_, after)| after.trim())
                    .filter(|after| !after.is_empty())
                    .unwrap_or("hunk");
                out.push(format!("{file}::{symbol}"));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn diff_added_public_api(diff: &str) -> Vec<String> {
    diff_added_lines(diff)
        .into_iter()
        .filter(|(_, line)| is_public_api_line(line))
        .map(|(file, line)| format!("{file}:{}", line.trim()))
        .collect()
}

fn diff_runtime_side_effects(diff: &str) -> Vec<String> {
    diff_added_lines(diff)
        .into_iter()
        .filter(|(_, line)| is_runtime_side_effect_line(line))
        .map(|(file, line)| format!("{file}:{}", line.trim()))
        .collect()
}

fn diff_added_lines(diff: &str) -> Vec<(String, String)> {
    let mut current_file = None::<String>;
    let mut out = vec![];
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git a/") {
            current_file = rest.split_once(" b/").map(|(left, _)| left.to_string());
            continue;
        }
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if let (Some(file), Some(added)) = (current_file.as_ref(), line.strip_prefix('+')) {
            out.push((file.clone(), added.to_string()));
        }
    }
    out
}

fn is_public_api_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("pub ")
        || trimmed.starts_with("pub(")
        || trimmed.starts_with("pub(crate)")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("pub(crate) fn ")
}

fn is_runtime_side_effect_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "std::process::command",
        "command::new",
        "fs::write",
        "file::create",
        "remove_file",
        "remove_dir",
        "rename(",
        "systemctl",
        "service ",
        "dbus",
        "spawn(",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn is_test_file(file: &str) -> bool {
    file.contains("/tests/")
        || file.starts_with("tests/")
        || file.contains("_test")
        || file.contains("test_")
        || file.ends_with(".spec.js")
}

fn file_from_diff_ref(item: &str) -> &str {
    item.split_once(':').map_or(item, |(file, _)| file)
}

fn path_matches(changed: &str, allowed: &str) -> bool {
    changed == allowed || changed.starts_with(allowed) || allowed.starts_with(changed)
}

fn guard_diff_repairs(verdict: &str, reason: &str, suggested_shared_actions: &[String]) -> Value {
    if verdict == "PASS" {
        return json!([]);
    }
    match reason {
        "version_bump_scope_violation" => {
            json!([repair(
                "version_bump_scope_violation",
                "Keep shared.version_bump_contract limited to version-owned files; split real code/UI/runtime changes into their own guarded diff."
            )])
        }
        "version_bump_inconsistent" => {
            json!([repair(
                "version_bump_inconsistent",
                "Make Cargo.toml, Cargo.lock, metadata.json, and APP_VERSION values agree before accepting the version bump."
            )])
        }
        "empty_or_unreadable_diff" => {
            json!([repair(
                "empty_or_unreadable_diff",
                "Provide a non-empty git diff from the target repository before allowing edits."
            )])
        }
        "diff_source_repo_mismatch" => {
            json!([repair(
                "diff_source_repo_mismatch",
                "Regenerate the diff from the same repository as the route atlas."
            )])
        }
        "public_api_growth_requires_review" => {
            json!([repair(
                "public_api_growth_requires_review",
                "Review public API growth and add an explicit contract or route test before accepting the diff."
            )])
        }
        "runtime_side_effect_requires_test" => {
            json!([repair(
                "runtime_side_effect_requires_test",
                "Add or change a route-specific test/smoke for the runtime side effect before accepting the diff."
            )])
        }
        "route_crossing_requires_shared_contract" if !suggested_shared_actions.is_empty() => {
            json!([repair(
                "route_crossing_requires_shared_contract",
                &format!(
                    "Use one of these shared action_id values if the crossing is intentional: {}",
                    suggested_shared_actions.join(", ")
                )
            )])
        }
        _ => json!([repair(
            "side_effect_creep",
            "Split the diff or choose an action_id that owns all changed routes.",
        )]),
    }
}

fn repair(kind: &str, repair: &str) -> Value {
    json!({
        "kind": kind,
        "priority": if kind == "side_effect_creep" || kind == "route_crossing_requires_shared_contract" { "high" } else { "medium" },
        "repair": repair
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn atlas(repo: &Path) -> Value {
        json!({
            "mode": "route-atlas",
            "repo": repo.display().to_string(),
            "input": repo.display().to_string(),
            "routes": {
                "ime-display-flow": {
                    "allowed_files": ["src/ime/display.rs"],
                    "owners": ["ImeDisplayOwner"],
                    "runtime_checks": ["ime smoke"],
                    "forbidden_routes": ["manual-trigger-flow", "runtime-flow", "source-flow"]
                },
                "manual-trigger-flow": {
                    "allowed_files": ["src/manual/event.rs"],
                    "owners": ["ManualTriggerOwner"],
                    "runtime_checks": ["manual smoke"],
                    "forbidden_routes": ["ime-display-flow", "runtime-flow", "source-flow"]
                },
                "runtime-flow": {
                    "allowed_files": ["src/runtime/run.rs"],
                    "owners": ["RuntimeOwner"],
                    "runtime_checks": ["runtime smoke"],
                    "forbidden_routes": ["ime-display-flow", "manual-trigger-flow", "source-flow"]
                },
                "test-flow": {
                    "allowed_files": ["tests/runtime.rs"],
                    "owners": ["TestOwner"],
                    "runtime_checks": ["cargo test runtime"],
                    "forbidden_routes": []
                },
                "source-flow": {
                    "allowed_files": ["Cargo.toml", "src/manual_toggle.rs"],
                    "owners": ["SourceOwner"],
                    "runtime_checks": ["cargo check"],
                    "forbidden_routes": []
                }
            },
            "action_prefixes": {
                "ime": "ime-display-flow",
                "manual": "manual-trigger-flow",
                "runtime": "runtime-flow",
                "shared": "shared-contract"
            },
            "shared_contracts": {
                "shared.manual_toggle_contract": {
                    "allowed_routes": ["source-flow", "ime-display-flow"],
                    "shared_candidates": ["manual_toggle"],
                    "reason": "manual toggle bridges source and display"
                },
                "shared.version_bump_contract": {
                    "allowed_routes": ["source-flow", "config-flow", "ui-status-flow", "install-flow"],
                    "shared_candidates": ["Cargo.toml", "version"],
                    "contract_scope": "version metadata only",
                    "reason": "version metadata only"
                }
            }
        })
    }

    fn temp_repo(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("nanda-boundary-diff-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp repo");
        path
    }

    #[test]
    fn boundary_diff_empty_is_watch() {
        let repo = temp_repo("empty");
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", "", None);
        assert_eq!(out["verdict"], "WATCH");
        assert_eq!(out["reason"], "empty_or_unreadable_diff");
        assert_eq!(
            out["boundary_diff_kernel"]["owner"],
            "field_core::boundary::diff"
        );
    }

    #[test]
    fn boundary_diff_single_route_passes() {
        let repo = temp_repo("single");
        let diff = "\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n\
@@ -1 +1 @@\n\
-fn show() {}\n\
+fn show() { let visible = true; }\n";
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", diff, None);
        assert_eq!(out["verdict"], "PASS");
        assert_eq!(out["boundary_diff_kernel"]["diff_verdict"], "DIFF_KEEP");
        assert_eq!(
            out["boundary_diff_kernel"]["field_equivalence"]["field_not_more_permissive"],
            true
        );
    }

    #[test]
    fn boundary_diff_foreign_route_is_veto() {
        let repo = temp_repo("foreign");
        let diff = "\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n\
diff --git a/src/manual/event.rs b/src/manual/event.rs\n\
--- a/src/manual/event.rs\n\
+++ b/src/manual/event.rs\n";
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", diff, None);
        assert_eq!(out["verdict"], "VETO");
        assert_eq!(
            out["boundary_diff_kernel"]["diff_verdict"],
            "DIFF_SHARED_CONTRACT_REQUIRED"
        );
        assert_eq!(out["boundary_diff_kernel"]["selected_verdict"], "VETO");
    }

    #[test]
    fn boundary_diff_shared_contract_passes() {
        let repo = temp_repo("shared");
        let diff = "\
diff --git a/src/manual_toggle.rs b/src/manual_toggle.rs\n\
--- a/src/manual_toggle.rs\n\
+++ b/src/manual_toggle.rs\n\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n";
        let out = boundary_guard_diff(&atlas(&repo), "shared.manual_toggle_contract", diff, None);
        assert_eq!(out["verdict"], "PASS");
        assert_eq!(out["reason"], "shared_contract_allows_route_crossing");
    }

    #[test]
    fn boundary_diff_version_bump_passes() {
        let repo = temp_repo("version");
        fs::write(
            repo.join("Cargo.toml"),
            "[package]\nname = \"nanda-test\"\nversion = \"1.2.3\"\n",
        )
        .expect("write cargo");
        let diff = "\
diff --git a/Cargo.toml b/Cargo.toml\n\
--- a/Cargo.toml\n\
+++ b/Cargo.toml\n";
        let out = boundary_guard_diff(&atlas(&repo), "shared.version_bump_contract", diff, None);
        assert_eq!(out["verdict"], "PASS");
        assert_eq!(out["reason"], "version_bump_contract_pass");
    }

    #[test]
    fn boundary_diff_public_api_growth_is_watch() {
        let repo = temp_repo("public-api");
        let diff = "\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n\
@@ -1 +1 @@\n\
+pub fn new_display_api() {}\n";
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", diff, None);
        assert_eq!(out["verdict"], "WATCH");
        assert_eq!(out["reason"], "public_api_growth_requires_review");
    }

    #[test]
    fn boundary_diff_runtime_side_effect_requires_test() {
        let repo = temp_repo("runtime");
        let diff = "\
diff --git a/src/runtime/run.rs b/src/runtime/run.rs\n\
--- a/src/runtime/run.rs\n\
+++ b/src/runtime/run.rs\n\
@@ -1 +1 @@\n\
+fn run() { let _ = std::process::Command::new(\"true\"); }\n";
        let out = boundary_guard_diff(&atlas(&repo), "runtime.restart", diff, None);
        assert_eq!(out["verdict"], "WATCH");
        assert_eq!(out["reason"], "runtime_side_effect_requires_test");
        assert_eq!(
            out["boundary_diff_kernel"]["diff_verdict"],
            "DIFF_TESTS_REQUIRED"
        );
    }
}
