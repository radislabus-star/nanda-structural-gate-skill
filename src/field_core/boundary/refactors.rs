use anyhow::Result;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::decision::boundary_decision;
use super::energy::{boundary_energy, boundary_owner_gravity};
use super::facts::{
    collect_boundary_files, collect_facts, filter_files_by_owner, infer_single_owner,
    infer_single_route,
};
use super::util::sample;
use super::{BoundaryFacts, BoundaryPathClassifier};
use crate::CORE_VERSION;

struct SelectedRefactorFiles {
    files: Vec<String>,
    route_scoped: bool,
    owner_filter_matched: bool,
    route_files_considered: usize,
    runtime_checks: Vec<String>,
}

struct RefactorCandidateContext<'a> {
    root: &'a Path,
    route_for_path: BoundaryPathClassifier,
    owner_for_path: BoundaryPathClassifier,
    runtime_checks: &'a [String],
    route_scoped: bool,
    owner_filter_requested: bool,
    owner_filter_matched: bool,
    requested_owner: Option<&'a str>,
    route_files_considered: usize,
}

pub(super) fn boundary_refactor_finder(
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
    let selected = select_refactor_files(&root, input, atlas, route, owner, owner_for_path)?;
    let ctx = RefactorCandidateContext {
        root: &root,
        route_for_path,
        owner_for_path,
        runtime_checks: selected.runtime_checks.as_slice(),
        route_scoped: selected.route_scoped,
        owner_filter_requested: owner.is_some(),
        owner_filter_matched: selected.owner_filter_matched,
        requested_owner: owner,
        route_files_considered: selected.route_files_considered,
    };
    let mut candidates = vec![];

    add_group_candidates(
        &mut candidates,
        "route",
        &files_by_route(&selected.files, route_for_path),
        &ctx,
    );
    add_group_candidates(
        &mut candidates,
        "owner",
        &files_by_owner(&selected.files, owner_for_path),
        &ctx,
    );
    add_file_candidates(&mut candidates, &selected.files, &ctx);

    candidates.sort_by(|left, right| {
        let left_rank = left["rank_score"].as_i64().unwrap_or(0);
        let right_rank = right["rank_score"].as_i64().unwrap_or(0);
        right_rank.cmp(&left_rank).then_with(|| {
            left["subject"]
                .as_str()
                .unwrap_or("")
                .cmp(right["subject"].as_str().unwrap_or(""))
        })
    });
    candidates.truncate(32);
    for (index, candidate) in candidates.iter_mut().enumerate() {
        if let Some(object) = candidate.as_object_mut() {
            object.insert("rank".to_string(), json!(index + 1));
        }
    }

    let strong = candidates
        .iter()
        .filter(|candidate| candidate["verdict"] == "SPLIT_STRONG")
        .count();
    let merge = candidates
        .iter()
        .filter(|candidate| candidate["verdict"] == "MERGE_CANDIDATE")
        .count();
    let veto = candidates
        .iter()
        .filter(|candidate| candidate["verdict"] == "VETO")
        .count();
    let watch = candidates
        .iter()
        .filter(|candidate| candidate["verdict"] == "WATCH")
        .count();

    Ok(json!({
        "mode": "boundary-refactor-finder",
        "core_version": CORE_VERSION,
        "boundary_core": {
            "owner": "field_core::boundary",
            "version": "boundary-refactor-finder-v1",
            "commands_are_wrappers": true,
            "policy": "NO_EVIDENCE_NO_CUT"
        },
        "input": input.display().to_string(),
        "atlas": atlas_label,
        "scope": if selected.route_scoped { "route-scoped" } else { "repo-wide" },
        "target_route": route,
        "target_owner": owner,
        "safe_to_edit": false,
        "verdict": "REFACTOR_CANDIDATES_READY",
        "candidate_summary": {
            "total": candidates.len(),
            "split_strong": strong,
            "merge_candidate": merge,
            "veto": veto,
            "watch": watch,
            "keep_or_review": candidates.len().saturating_sub(strong + merge + veto + watch)
        },
        "ranking_policy": {
            "no_size_only_split": true,
            "requires_evidence": [
                "route/owner conflict",
                "foreign pull",
                "thin wrapper tax",
                "public API growth",
                "runtime side effect without tests",
                "shared state locality"
            ],
            "read_as": "Candidates are refactor pressure signals, not edit permission. Route-scoped guard-action/guard-diff still gates the actual change."
        },
        "refactor_candidates": candidates,
        "read_as": "Boundary finder ranks possible split/merge/keep pressure while preserving NO EVIDENCE => NO CUT."
    }))
}

fn select_refactor_files(
    root: &Path,
    input: &Path,
    atlas: Option<&Value>,
    route: Option<&str>,
    owner: Option<&str>,
    owner_for_path: BoundaryPathClassifier,
) -> Result<SelectedRefactorFiles> {
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
        collect_boundary_files(root, input, &mut files)?;
    } else if files.is_empty() && input.is_file() {
        files.push(input.to_string_lossy().to_string());
    }
    let route_files_considered = files.len();
    let mut owner_filter_matched = true;
    if route_scoped {
        if let Some(owner) = owner {
            let filtered = filter_files_by_owner(&files, owner, owner_for_path);
            if filtered.is_empty() {
                owner_filter_matched = false;
                files.clear();
            } else {
                files = filtered;
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(SelectedRefactorFiles {
        files,
        route_scoped,
        owner_filter_matched,
        route_files_considered,
        runtime_checks,
    })
}

fn files_by_route(
    files: &[String],
    route_for_path: BoundaryPathClassifier,
) -> BTreeMap<String, Vec<String>> {
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for file in files {
        groups
            .entry(route_for_path(file))
            .or_default()
            .push(file.clone());
    }
    groups
}

fn files_by_owner(
    files: &[String],
    owner_for_path: BoundaryPathClassifier,
) -> BTreeMap<String, Vec<String>> {
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for file in files {
        groups
            .entry(owner_for_path(file))
            .or_default()
            .push(file.clone());
    }
    groups
}

fn add_group_candidates(
    out: &mut Vec<Value>,
    subject_kind: &str,
    groups: &BTreeMap<String, Vec<String>>,
    ctx: &RefactorCandidateContext<'_>,
) {
    for (group, files) in groups {
        if files.len() < 2 {
            continue;
        }
        let target_route = infer_single_route(files, ctx.route_for_path);
        let target_owner = infer_single_owner(files, ctx.owner_for_path);
        if let Some(candidate) = candidate_for_files(
            subject_kind,
            group,
            ctx.root,
            files,
            target_route.as_deref(),
            target_owner.as_deref(),
            ctx,
        ) {
            out.push(candidate);
        }
    }
}

fn add_file_candidates(out: &mut Vec<Value>, files: &[String], ctx: &RefactorCandidateContext<'_>) {
    for file in files {
        let target_route = (ctx.route_for_path)(file);
        let target_owner = (ctx.owner_for_path)(file);
        if let Some(candidate) = candidate_for_files(
            "file",
            file,
            ctx.root,
            std::slice::from_ref(file),
            Some(&target_route),
            Some(&target_owner),
            ctx,
        ) {
            out.push(candidate);
        }
    }
}

fn candidate_for_files(
    subject_kind: &str,
    subject: &str,
    root: &Path,
    files: &[String],
    target_route: Option<&str>,
    target_owner: Option<&str>,
    ctx: &RefactorCandidateContext<'_>,
) -> Option<Value> {
    let mut facts = collect_facts(
        root,
        files,
        target_route,
        ctx.route_for_path,
        ctx.owner_for_path,
    );
    facts.runtime_checks = ctx.runtime_checks.to_vec();
    facts.route_scoped = ctx.route_scoped;
    facts.owner_filter_requested = ctx.owner_filter_requested;
    facts.owner_filter_matched = ctx.owner_filter_matched;
    facts.requested_owner = ctx.requested_owner.map(str::to_string);
    facts.route_files_considered = ctx.route_files_considered;
    let decision = boundary_decision(&facts, target_route, target_owner);
    let pressure = pressure_score(&facts);
    let include = decision.verdict != "KEEP" || pressure >= 8;
    if !include {
        return None;
    }
    let owner_gravity = boundary_owner_gravity(&facts);
    let boundary_energy = boundary_energy(&facts, &decision, &owner_gravity);
    let rank_score = verdict_rank(decision.verdict) + pressure as i64 + decision.score as i64;
    Some(json!({
        "rank": 0,
        "rank_score": rank_score,
        "subject_kind": subject_kind,
        "subject": subject,
        "verdict": decision.verdict,
        "score": decision.score,
        "pressure_score": pressure,
        "route": decision.route,
        "owner": decision.owner,
        "reason": decision.reason,
        "why_ranked": why_ranked(&facts, decision.verdict, pressure),
        "evidence": refactor_evidence(&facts),
        "owner_gravity": owner_gravity,
        "boundary_energy": boundary_energy,
        "contract": {
            "allowed_files": decision.allowed_files,
            "forbidden_routes": decision.forbidden_routes,
            "must_not_change": decision.must_not_change,
            "required_tests": decision.required_tests,
            "repair": decision.repair
        },
        "read_as": read_as(decision.verdict)
    }))
}

fn pressure_score(facts: &BoundaryFacts) -> usize {
    facts.foreign_route_files.len() * 8
        + facts.owners.len().saturating_sub(1) * 5
        + facts.routes.len().saturating_sub(1) * 5
        + facts.thin_wrappers.len() * 5
        + facts.runtime_side_effects.len() * 3
        + facts.shared_state.len() * 2
        + facts.public_api.len()
        + facts.functions.len() / 6
        + facts.call_edges.len() / 4
        + if facts.tests.is_empty() && !facts.runtime_side_effects.is_empty() {
            6
        } else {
            0
        }
}

fn verdict_rank(verdict: &str) -> i64 {
    match verdict {
        "VETO" => 100,
        "SPLIT_STRONG" => 90,
        "MERGE_CANDIDATE" => 80,
        "SPLIT_WEAK" => 70,
        "WATCH" => 60,
        "KEEP" => 20,
        _ => 10,
    }
}

fn refactor_evidence(facts: &BoundaryFacts) -> Value {
    json!({
        "files_count": facts.files.len(),
        "functions_count": facts.functions.len(),
        "public_api_count": facts.public_api.len(),
        "call_edges_count": facts.call_edges.len(),
        "shared_state_count": facts.shared_state.len(),
        "runtime_side_effect_count": facts.runtime_side_effects.len(),
        "tests_count": facts.tests.len(),
        "foreign_pull_count": facts.foreign_route_files.len(),
        "routes": facts.routes.iter().cloned().collect::<Vec<_>>(),
        "owners": facts.owners.iter().cloned().collect::<Vec<_>>(),
        "thin_wrappers": facts.thin_wrappers.clone(),
        "sample_files": sample(&facts.files, 10),
        "sample_functions": sample(&facts.functions, 12),
        "sample_foreign_pull": sample(&facts.foreign_route_files, 10),
        "sample_runtime_side_effects": sample(&facts.runtime_side_effects, 10),
        "sample_public_api": sample(&facts.public_api, 10)
    })
}

fn why_ranked(facts: &BoundaryFacts, verdict: &str, pressure: usize) -> Vec<String> {
    let mut reasons = vec![];
    if verdict != "KEEP" {
        reasons.push(format!("boundary verdict is {verdict}"));
    }
    if !facts.foreign_route_files.is_empty() {
        reasons.push("foreign route pull is present".to_string());
    }
    if facts.owners.len() > 1 {
        reasons.push("multiple owners share the candidate boundary".to_string());
    }
    if facts.routes.len() > 1 {
        reasons.push("multiple routes share the candidate boundary".to_string());
    }
    if !facts.thin_wrappers.is_empty() {
        reasons.push("thin wrapper tax is present".to_string());
    }
    if !facts.runtime_side_effects.is_empty() && facts.tests.is_empty() {
        reasons.push("runtime side effects lack nearby route tests".to_string());
    }
    if verdict == "KEEP" && pressure >= 8 {
        reasons.push("pressure is visible, but NO_EVIDENCE_NO_CUT keeps the boundary".to_string());
    }
    if reasons.is_empty() {
        reasons.push("low-priority review candidate".to_string());
    }
    reasons
}

fn read_as(verdict: &str) -> &'static str {
    match verdict {
        "SPLIT_STRONG" => {
            "Evidence supports a split candidate; still guard the actual diff by route."
        }
        "SPLIT_WEAK" => "Potential split pressure; prepare only a small reviewed step.",
        "MERGE_CANDIDATE" => "Thin wrapper or low-value boundary; prepare a separate merge plan.",
        "VETO" => "Unsafe boundary crossing; repair ownership/route first.",
        "WATCH" => "Evidence is insufficient; do not cut.",
        "KEEP" => "Pressure is visible but not enough to cut.",
        _ => "Review candidate only.",
    }
}
