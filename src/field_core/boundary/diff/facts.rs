use serde_json::Value;
use std::collections::BTreeSet;

use super::parser::{
    diff_added_public_api, diff_changed_files, diff_changed_functions, diff_runtime_side_effects,
    is_test_file, path_matches,
};
use super::routes::{
    route_for_action, route_for_changed_file, shared_candidate_files, shared_contract_for_action,
    suggested_shared_actions,
};
use super::types::BoundaryDiffFacts;
use super::version::check_version_bump_contract;

pub(super) fn collect_diff_facts(
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
        .collect::<std::collections::BTreeMap<_, _>>();
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
