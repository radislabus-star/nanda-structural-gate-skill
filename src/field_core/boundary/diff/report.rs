use serde_json::{json, Value};

use super::decision::guard_diff_repairs;
use super::field_pass::{diff_field_equivalence, diff_field_pass};
use super::records::diff_field_records;
use super::types::{BoundaryDiffDecision, BoundaryDiffFacts, BoundaryDiffFieldRecords};

pub(super) fn diff_report(
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
