use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::field_core::FieldRecord;

#[derive(Debug, Clone, Serialize)]
pub(super) struct BoundaryDiffFacts {
    pub(super) action_id: String,
    pub(super) route: Option<String>,
    pub(super) shared_contract: Option<Value>,
    pub(super) version_bump: Option<Value>,
    pub(super) changed_files: Vec<String>,
    pub(super) file_routes: BTreeMap<String, String>,
    pub(super) changed_routes: Vec<String>,
    pub(super) changed_functions: Vec<String>,
    pub(super) added_public_api: Vec<String>,
    pub(super) changed_runtime_side_effects: Vec<String>,
    pub(super) changed_tests: Vec<String>,
    pub(super) shared_candidates: Vec<String>,
    pub(super) foreign_files: Vec<String>,
    pub(super) foreign_routes: Vec<String>,
    pub(super) suggested_shared_actions: Vec<String>,
    pub(super) diff_source: Option<Value>,
    pub(super) source_mismatch: bool,
    pub(super) empty_or_unreadable: bool,
    pub(super) shared_allows_crossing: bool,
    pub(super) route_crossing: bool,
}

impl BoundaryDiffFacts {
    pub(super) fn public_api_growth_allowed(&self) -> bool {
        if self.added_public_api.is_empty() {
            return true;
        }
        let Some(contract) = self.shared_contract.as_ref() else {
            return false;
        };
        let internal_allowed = contract["allow_internal_api_growth"]
            .as_bool()
            .unwrap_or(false);
        let tested_public_allowed = contract["allow_tested_public_api_growth"]
            .as_bool()
            .unwrap_or(false)
            && !self.changed_tests.is_empty();
        self.added_public_api.iter().all(|item| {
            (internal_allowed && Self::is_internal_public_api(item)) || tested_public_allowed
        })
    }

    fn is_internal_public_api(item: &str) -> bool {
        item.contains(":pub(crate)") || item.contains(":pub(super)") || item.contains(":pub(in ")
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct BoundaryDiffDecision {
    pub(super) verdict: &'static str,
    pub(super) diff_verdict: &'static str,
    pub(super) safe_to_edit: bool,
    pub(super) reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct BoundaryDiffFieldRecords {
    pub(super) version: &'static str,
    pub(super) owner: &'static str,
    pub(super) record_count: usize,
    pub(super) changed_file_records: usize,
    pub(super) route_records: usize,
    pub(super) public_api_records: usize,
    pub(super) runtime_side_effect_records: usize,
    pub(super) test_records: usize,
    pub(super) foreign_pull_records: usize,
    pub(super) sample: Vec<FieldRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct BoundaryDiffFieldEquivalence {
    pub(super) version: &'static str,
    pub(super) typed_verdict: String,
    pub(super) field_verdict: String,
    pub(super) typed_rank: u8,
    pub(super) field_rank: u8,
    pub(super) field_not_more_permissive: bool,
    pub(super) mismatch_reason: Vec<String>,
}
