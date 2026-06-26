use crate::field_core::{
    run_field_pass, FieldAntiWaveLane, FieldClaimBoundary, FieldFamily, FieldLensKind,
    FieldLensOperation, FieldPassInput, FieldPassReport, FieldRecord, FieldRecordKind,
    FIELD_PASS_VERSION,
};

use super::{
    BoundaryDecision, BoundaryFacts, BoundaryFieldEngine, BoundaryFieldEnginePolicy,
    BoundaryFieldEquivalence, BoundaryFieldPassAdmission,
};

pub(super) fn boundary_field_pass_admission(
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

pub(super) fn boundary_field_equivalence(
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

pub(super) fn boundary_field_engine(
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
