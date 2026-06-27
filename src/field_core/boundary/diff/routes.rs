use serde_json::Value;
use std::collections::BTreeSet;

use super::parser::path_matches;
use super::types::BoundaryDiffFacts;

pub(super) fn route_for_action(atlas: &Value, action_id: &str) -> Option<String> {
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

pub(super) fn shared_contract_for_action<'a>(
    atlas: &'a Value,
    action_id: &str,
) -> Option<&'a Value> {
    atlas["shared_contracts"][action_id].as_object()?;
    Some(&atlas["shared_contracts"][action_id])
}

pub(super) fn route_for_changed_file(atlas: &Value, file: &str) -> String {
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

pub(super) fn route_for_file_from_facts(facts: &BoundaryDiffFacts, file: &str) -> String {
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

pub(super) fn shared_candidate_files(contract: Option<&Value>, changed: &[String]) -> Vec<String> {
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

pub(super) fn suggested_shared_actions(
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
