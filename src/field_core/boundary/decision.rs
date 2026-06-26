use serde_json::{json, Value};

use super::util::{sample, sample_values};
use super::{
    BoundaryDecision, BoundaryEvidence, BoundaryFacts, BoundaryOwnerFilterEvidence,
    BoundaryScoreComponent, BoundaryScoreComponents, BoundaryVerdict,
};

pub(super) fn boundary_decision(
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

fn owner_evidence(facts: &BoundaryFacts) -> Vec<String> {
    facts
        .owners
        .iter()
        .map(|owner| format!("owner:{owner}"))
        .collect()
}

pub(super) fn score_component(score: i32, evidence: Vec<String>) -> BoundaryScoreComponent {
    BoundaryScoreComponent {
        score,
        counted: !evidence.is_empty(),
        evidence: sample(&evidence, 16),
    }
}

pub(super) fn empty_components() -> Value {
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
