use crate::field_core::{
    run_field_pass, FieldAntiWaveLane, FieldClaimBoundary, FieldFamily, FieldLensKind,
    FieldLensOperation, FieldPassInput, FieldPassReport, FieldRecord, FieldRecordKind,
    FIELD_PASS_VERSION,
};

use super::routes::route_for_file_from_facts;
use super::types::{BoundaryDiffDecision, BoundaryDiffFacts, BoundaryDiffFieldEquivalence};

pub(super) fn diff_field_pass(
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

pub(super) fn diff_field_equivalence(
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
