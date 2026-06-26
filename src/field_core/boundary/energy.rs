use std::collections::BTreeMap;

use super::util::sample;
use super::{BoundaryDecision, BoundaryEnergy, BoundaryFacts, BoundaryOwnerGravity};

pub(super) fn boundary_owner_gravity(facts: &BoundaryFacts) -> BoundaryOwnerGravity {
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

pub(super) fn boundary_energy(
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
