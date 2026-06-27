use serde::Serialize;
use std::collections::BTreeMap;

use super::{BoundaryEnergy, BoundaryFacts, BoundaryOwnerGravity};

#[derive(Debug, Clone, Serialize)]
pub(super) struct BoundaryCenter {
    version: &'static str,
    read_only: bool,
    decision_affects_safe_to_edit: bool,
    route_center: Option<String>,
    route_center_strength: f64,
    second_route_center: Option<String>,
    center_gap: f64,
    owner_center: Option<String>,
    owner_center_strength: f64,
    second_owner_center: Option<String>,
    owner_center_gap: f64,
    route_mass: BTreeMap<String, usize>,
    owner_mass: BTreeMap<String, usize>,
    total_evidence_mass: usize,
    foreign_route_mass: f64,
    cross_owner_mass: f64,
    runtime_risk_mass: f64,
    public_api_mass: f64,
    adapter_leak_mass: f64,
    test_anchor_mass: f64,
    boundary_tax_mass: f64,
    verdict_hint: &'static str,
    read_as: &'static str,
}

pub(super) fn boundary_center(
    facts: &BoundaryFacts,
    owner_gravity: &BoundaryOwnerGravity,
    energy: &BoundaryEnergy,
) -> BoundaryCenter {
    let mut route_mass = BTreeMap::<String, usize>::new();
    let mut owner_mass = BTreeMap::<String, usize>::new();
    let mut total = 0usize;

    add_file_set(
        facts,
        &facts.files,
        4,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );
    add_function_set(
        facts,
        &facts.functions,
        2,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );
    add_line_set(
        facts,
        &facts.public_api,
        2,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );
    add_line_set(
        facts,
        &facts.shared_state,
        2,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );
    add_line_set(
        facts,
        &facts.runtime_side_effects,
        3,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );
    add_file_set(
        facts,
        &facts.tests,
        3,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );
    add_call_edges(facts, &mut route_mass, &mut owner_mass, &mut total);
    add_file_set(
        facts,
        &facts.foreign_route_files,
        5,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );
    add_file_set(
        facts,
        &facts.thin_wrappers,
        3,
        &mut route_mass,
        &mut owner_mass,
        &mut total,
    );

    let (route_center, route_top, second_route_center, route_second) = top_two(&route_mass);
    let (owner_center, owner_top, second_owner_center, owner_second) = top_two(&owner_mass);
    let route_center_strength = ratio(route_top, total);
    let owner_center_strength = ratio(owner_top, total);
    let center_gap = ratio(route_top.saturating_sub(route_second), total);
    let owner_center_gap = ratio(owner_top.saturating_sub(owner_second), total);

    let foreign_route_weight = weight_files(facts, &facts.foreign_route_files, 5);
    let cross_owner_weight = cross_owner_call_weight(facts);
    let runtime_weight = weight_lines(facts, &facts.runtime_side_effects, 3);
    let public_api_weight = weight_lines(facts, &facts.public_api, 2);
    let adapter_leak_weight =
        facts.thin_wrappers.len() * 3 + owner_gravity.adapter_leak_energy.max(0) as usize;
    let test_anchor_weight = weight_files(facts, &facts.tests, 3);
    let boundary_tax_weight = energy.boundary_tax.max(0) as usize;

    let foreign_route_mass = ratio(foreign_route_weight, total);
    let cross_owner_mass = ratio(cross_owner_weight, total);
    let runtime_risk_mass = ratio(runtime_weight, total);
    let public_api_mass = ratio(public_api_weight, total);
    let adapter_leak_mass = ratio(adapter_leak_weight, total);
    let test_anchor_mass = ratio(test_anchor_weight, total);
    let boundary_tax_mass = ratio(boundary_tax_weight, total + boundary_tax_weight);

    let verdict_hint = center_verdict_hint(&CenterHintInput {
        total,
        foreign_route_mass,
        cross_owner_mass,
        owner_conflict: owner_gravity.owner_conflict,
        route_center_strength,
        owner_center_strength,
        center_gap,
        owner_center_gap,
    });

    BoundaryCenter {
        version: "boundary-center-v1-read-only",
        read_only: true,
        decision_affects_safe_to_edit: false,
        route_center,
        route_center_strength,
        second_route_center,
        center_gap,
        owner_center,
        owner_center_strength,
        second_owner_center,
        owner_center_gap,
        route_mass,
        owner_mass,
        total_evidence_mass: total,
        foreign_route_mass,
        cross_owner_mass,
        runtime_risk_mass,
        public_api_mass,
        adapter_leak_mass,
        test_anchor_mass,
        boundary_tax_mass,
        verdict_hint,
        read_as: "Read-only center-of-mass diagnostic: one strong route/owner center means local responsibility is clear; diffuse or foreign mass means split/review before editing.",
    }
}

fn add_file_set(
    facts: &BoundaryFacts,
    files: &[String],
    weight: usize,
    route_mass: &mut BTreeMap<String, usize>,
    owner_mass: &mut BTreeMap<String, usize>,
    total: &mut usize,
) {
    for file in files {
        add_file_mass(facts, file, weight, route_mass, owner_mass, total);
    }
}

fn add_function_set(
    facts: &BoundaryFacts,
    functions: &[String],
    weight: usize,
    route_mass: &mut BTreeMap<String, usize>,
    owner_mass: &mut BTreeMap<String, usize>,
    total: &mut usize,
) {
    for function in functions {
        if let Some(file) = function.split_once("::").map(|(file, _)| file) {
            add_file_mass(facts, file, weight, route_mass, owner_mass, total);
        }
    }
}

fn add_line_set(
    facts: &BoundaryFacts,
    lines: &[String],
    weight: usize,
    route_mass: &mut BTreeMap<String, usize>,
    owner_mass: &mut BTreeMap<String, usize>,
    total: &mut usize,
) {
    for line in lines {
        if let Some(file) = line.split_once(':').map(|(file, _)| file) {
            add_file_mass(facts, file, weight, route_mass, owner_mass, total);
        }
    }
}

fn add_call_edges(
    facts: &BoundaryFacts,
    route_mass: &mut BTreeMap<String, usize>,
    owner_mass: &mut BTreeMap<String, usize>,
    total: &mut usize,
) {
    for edge in &facts.call_edges {
        let Some((caller, rhs)) = edge.split_once(" -> ") else {
            continue;
        };
        let callee = rhs.split_once("::").map_or(rhs, |(file, _)| file);
        add_file_mass(facts, caller, 1, route_mass, owner_mass, total);
        add_file_mass(facts, callee, 1, route_mass, owner_mass, total);
    }
}

fn add_file_mass(
    facts: &BoundaryFacts,
    file: &str,
    weight: usize,
    route_mass: &mut BTreeMap<String, usize>,
    owner_mass: &mut BTreeMap<String, usize>,
    total: &mut usize,
) {
    if let Some(route) = facts.file_routes.get(file) {
        *route_mass.entry(route.clone()).or_default() += weight;
    }
    if let Some(owner) = facts.file_owners.get(file) {
        *owner_mass.entry(owner.clone()).or_default() += weight;
    }
    if facts.file_routes.contains_key(file) || facts.file_owners.contains_key(file) {
        *total += weight;
    }
}

fn weight_files(facts: &BoundaryFacts, files: &[String], weight: usize) -> usize {
    files
        .iter()
        .filter(|file| facts.file_routes.contains_key(file.as_str()))
        .count()
        * weight
}

fn weight_lines(facts: &BoundaryFacts, lines: &[String], weight: usize) -> usize {
    lines
        .iter()
        .filter_map(|line| line.split_once(':').map(|(file, _)| file))
        .filter(|file| facts.file_routes.contains_key(*file))
        .count()
        * weight
}

fn cross_owner_call_weight(facts: &BoundaryFacts) -> usize {
    facts
        .call_edges
        .iter()
        .filter(|edge| {
            let Some((caller, rhs)) = edge.split_once(" -> ") else {
                return false;
            };
            let callee = rhs.split_once("::").map_or(rhs, |(file, _)| file);
            let Some(caller_owner) = facts.file_owners.get(caller) else {
                return false;
            };
            let Some(callee_owner) = facts.file_owners.get(callee) else {
                return false;
            };
            caller_owner != callee_owner
        })
        .count()
        * 2
}

fn top_two(map: &BTreeMap<String, usize>) -> (Option<String>, usize, Option<String>, usize) {
    let mut items = map.iter().collect::<Vec<_>>();
    items.sort_by(|(left_key, left_value), (right_key, right_value)| {
        right_value
            .cmp(left_value)
            .then_with(|| left_key.cmp(right_key))
    });
    let first = items.first().map(|(key, value)| ((*key).clone(), **value));
    let second = items.get(1).map(|(key, value)| ((*key).clone(), **value));
    (
        first.as_ref().map(|(key, _)| key.clone()),
        first.map(|(_, value)| value).unwrap_or(0),
        second.as_ref().map(|(key, _)| key.clone()),
        second.map(|(_, value)| value).unwrap_or(0),
    )
}

struct CenterHintInput {
    total: usize,
    foreign_route_mass: f64,
    cross_owner_mass: f64,
    owner_conflict: bool,
    route_center_strength: f64,
    owner_center_strength: f64,
    center_gap: f64,
    owner_center_gap: f64,
}

fn center_verdict_hint(input: &CenterHintInput) -> &'static str {
    if input.total == 0 {
        "CENTER_UNRESOLVED_NO_EVIDENCE"
    } else if input.foreign_route_mass > 0.0 {
        "CENTER_FOREIGN_PULL_REVIEW"
    } else if input.owner_conflict || input.cross_owner_mass >= 0.12 {
        "CENTER_OWNER_CONFLICT_REVIEW"
    } else if input.route_center_strength >= 0.75
        && input.owner_center_strength >= 0.75
        && input.center_gap >= 0.5
        && input.owner_center_gap >= 0.5
    {
        "CENTER_STABLE_SAFE_LOCAL_EDIT"
    } else if input.route_center_strength >= 0.7 && input.center_gap >= 0.4 {
        "CENTER_ROUTE_STABLE_OWNER_REVIEW"
    } else {
        "CENTER_DIFFUSE_WATCH"
    }
}

fn ratio(value: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        round4(value as f64 / total as f64)
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}
