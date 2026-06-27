use serde_json::{json, Value};

use super::types::{BoundaryDiffDecision, BoundaryDiffFacts};

pub(super) fn decide_diff(facts: &BoundaryDiffFacts) -> BoundaryDiffDecision {
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

pub(super) fn guard_diff_repairs(
    verdict: &str,
    reason: &str,
    suggested_shared_actions: &[String],
) -> Value {
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
