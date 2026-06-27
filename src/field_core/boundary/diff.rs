mod decision;
mod facts;
mod field_pass;
mod parser;
mod records;
mod report;
mod routes;
mod types;
mod version;

use serde_json::Value;

use decision::decide_diff;
use facts::collect_diff_facts;
use report::diff_report;
use types::BoundaryDiffDecision;

pub(super) fn boundary_guard_diff(
    atlas: &Value,
    action_id: &str,
    diff: &str,
    diff_source: Option<Value>,
) -> Value {
    let facts = collect_diff_facts(atlas, action_id, diff, diff_source);
    let decision = decide_diff(&facts);
    diff_report(&facts, &decision, None)
}

pub(super) fn boundary_guard_diff_unreadable(
    atlas: &Value,
    action_id: &str,
    diff_source: Option<Value>,
    diff_error: &str,
) -> Value {
    let mut facts = collect_diff_facts(atlas, action_id, "", diff_source);
    facts.empty_or_unreadable = true;
    let decision = BoundaryDiffDecision {
        verdict: "WATCH",
        diff_verdict: "DIFF_WATCH",
        safe_to_edit: false,
        reason: "empty_or_unreadable_diff",
    };
    diff_report(&facts, &decision, Some(diff_error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn atlas(repo: &Path) -> Value {
        json!({
            "mode": "route-atlas",
            "repo": repo.display().to_string(),
            "input": repo.display().to_string(),
            "routes": {
                "ime-display-flow": {
                    "allowed_files": ["src/ime/display.rs"],
                    "owners": ["ImeDisplayOwner"],
                    "runtime_checks": ["ime smoke"],
                    "forbidden_routes": ["manual-trigger-flow", "runtime-flow", "source-flow"]
                },
                "manual-trigger-flow": {
                    "allowed_files": ["src/manual/event.rs"],
                    "owners": ["ManualTriggerOwner"],
                    "runtime_checks": ["manual smoke"],
                    "forbidden_routes": ["ime-display-flow", "runtime-flow", "source-flow"]
                },
                "runtime-flow": {
                    "allowed_files": ["src/runtime/run.rs"],
                    "owners": ["RuntimeOwner"],
                    "runtime_checks": ["runtime smoke"],
                    "forbidden_routes": ["ime-display-flow", "manual-trigger-flow", "source-flow"]
                },
                "test-flow": {
                    "allowed_files": ["tests/runtime.rs"],
                    "owners": ["TestOwner"],
                    "runtime_checks": ["cargo test runtime"],
                    "forbidden_routes": []
                },
                "source-flow": {
                    "allowed_files": ["Cargo.toml", "src/manual_toggle.rs"],
                    "owners": ["SourceOwner"],
                    "runtime_checks": ["cargo check"],
                    "forbidden_routes": []
                }
            },
            "action_prefixes": {
                "ime": "ime-display-flow",
                "manual": "manual-trigger-flow",
                "runtime": "runtime-flow",
                "shared": "shared-contract"
            },
            "shared_contracts": {
                "shared.manual_toggle_contract": {
                    "allowed_routes": ["source-flow", "ime-display-flow"],
                    "shared_candidates": ["manual_toggle"],
                    "reason": "manual toggle bridges source and display"
                },
                "shared.version_bump_contract": {
                    "allowed_routes": ["source-flow", "config-flow", "ui-status-flow", "install-flow"],
                    "shared_candidates": ["Cargo.toml", "version"],
                    "contract_scope": "version metadata only",
                    "reason": "version metadata only"
                }
            }
        })
    }

    fn temp_repo(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("nanda-boundary-diff-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp repo");
        path
    }

    #[test]
    fn boundary_diff_empty_is_watch() {
        let repo = temp_repo("empty");
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", "", None);
        assert_eq!(out["verdict"], "WATCH");
        assert_eq!(out["reason"], "empty_or_unreadable_diff");
        assert_eq!(
            out["boundary_diff_kernel"]["owner"],
            "field_core::boundary::diff"
        );
    }

    #[test]
    fn boundary_diff_single_route_passes() {
        let repo = temp_repo("single");
        let diff = "\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n\
@@ -1 +1 @@\n\
-fn show() {}\n\
+fn show() { let visible = true; }\n";
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", diff, None);
        assert_eq!(out["verdict"], "PASS");
        assert_eq!(out["boundary_diff_kernel"]["diff_verdict"], "DIFF_KEEP");
        assert_eq!(
            out["boundary_diff_kernel"]["field_equivalence"]["field_not_more_permissive"],
            true
        );
    }

    #[test]
    fn boundary_diff_foreign_route_is_veto() {
        let repo = temp_repo("foreign");
        let diff = "\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n\
diff --git a/src/manual/event.rs b/src/manual/event.rs\n\
--- a/src/manual/event.rs\n\
+++ b/src/manual/event.rs\n";
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", diff, None);
        assert_eq!(out["verdict"], "VETO");
        assert_eq!(
            out["boundary_diff_kernel"]["diff_verdict"],
            "DIFF_SHARED_CONTRACT_REQUIRED"
        );
        assert_eq!(out["boundary_diff_kernel"]["selected_verdict"], "VETO");
    }

    #[test]
    fn boundary_diff_shared_contract_passes() {
        let repo = temp_repo("shared");
        let diff = "\
diff --git a/src/manual_toggle.rs b/src/manual_toggle.rs\n\
--- a/src/manual_toggle.rs\n\
+++ b/src/manual_toggle.rs\n\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n";
        let out = boundary_guard_diff(&atlas(&repo), "shared.manual_toggle_contract", diff, None);
        assert_eq!(out["verdict"], "PASS");
        assert_eq!(out["reason"], "shared_contract_allows_route_crossing");
    }

    #[test]
    fn boundary_diff_version_bump_passes() {
        let repo = temp_repo("version");
        fs::write(
            repo.join("Cargo.toml"),
            "[package]\nname = \"nanda-test\"\nversion = \"1.2.3\"\n",
        )
        .expect("write cargo");
        let diff = "\
diff --git a/Cargo.toml b/Cargo.toml\n\
--- a/Cargo.toml\n\
+++ b/Cargo.toml\n";
        let out = boundary_guard_diff(&atlas(&repo), "shared.version_bump_contract", diff, None);
        assert_eq!(out["verdict"], "PASS");
        assert_eq!(out["reason"], "version_bump_contract_pass");
    }

    #[test]
    fn boundary_diff_public_api_growth_is_watch() {
        let repo = temp_repo("public-api");
        let diff = "\
diff --git a/src/ime/display.rs b/src/ime/display.rs\n\
--- a/src/ime/display.rs\n\
+++ b/src/ime/display.rs\n\
@@ -1 +1 @@\n\
+pub fn new_display_api() {}\n";
        let out = boundary_guard_diff(&atlas(&repo), "ime.show_candidate", diff, None);
        assert_eq!(out["verdict"], "WATCH");
        assert_eq!(out["reason"], "public_api_growth_requires_review");
    }

    #[test]
    fn boundary_diff_runtime_side_effect_requires_test() {
        let repo = temp_repo("runtime");
        let diff = "\
diff --git a/src/runtime/run.rs b/src/runtime/run.rs\n\
--- a/src/runtime/run.rs\n\
+++ b/src/runtime/run.rs\n\
@@ -1 +1 @@\n\
+fn run() { let _ = std::process::Command::new(\"true\"); }\n";
        let out = boundary_guard_diff(&atlas(&repo), "runtime.restart", diff, None);
        assert_eq!(out["verdict"], "WATCH");
        assert_eq!(out["reason"], "runtime_side_effect_requires_test");
        assert_eq!(
            out["boundary_diff_kernel"]["diff_verdict"],
            "DIFF_TESTS_REQUIRED"
        );
    }
}
