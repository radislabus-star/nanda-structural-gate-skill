use crate::field_core::{FieldRecord, FieldRecordKind};

use super::parser::file_from_diff_ref;
use super::routes::route_for_file_from_facts;
use super::types::{BoundaryDiffFacts, BoundaryDiffFieldRecords};

impl BoundaryDiffFieldRecords {
    pub(super) fn from_facts(facts: &BoundaryDiffFacts, records: &[FieldRecord]) -> Self {
        Self {
            version: "boundary-diff-field-records-v1",
            owner: "field_core::boundary::diff",
            record_count: records.len(),
            changed_file_records: facts.changed_files.len(),
            route_records: facts.changed_routes.len(),
            public_api_records: facts.added_public_api.len(),
            runtime_side_effect_records: facts.changed_runtime_side_effects.len(),
            test_records: facts.changed_tests.len(),
            foreign_pull_records: facts.foreign_files.len() + facts.foreign_routes.len(),
            sample: records.iter().take(16).cloned().collect(),
        }
    }
}

pub(super) fn diff_field_records(facts: &BoundaryDiffFacts) -> Vec<FieldRecord> {
    let mut records = vec![];
    for file in &facts.changed_files {
        let route = Some(route_for_file_from_facts(facts, file));
        push_diff_record(
            &mut records,
            "diff_changed_file",
            "belongs_to_route",
            file,
            route,
            None,
            Some(file.clone()),
        );
    }
    for route in &facts.changed_routes {
        push_diff_record(
            &mut records,
            "diff_changed_route",
            "is_touched",
            route,
            Some(route.clone()),
            None,
            None,
        );
    }
    for item in &facts.added_public_api {
        let file = file_from_diff_ref(item);
        push_diff_record(
            &mut records,
            "diff_public_api",
            "adds_surface",
            item,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.to_string()),
        );
    }
    for item in &facts.changed_runtime_side_effects {
        let file = file_from_diff_ref(item);
        push_diff_record(
            &mut records,
            "diff_runtime_side_effect",
            "requires_test",
            item,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.to_string()),
        );
    }
    for file in &facts.changed_tests {
        push_diff_record(
            &mut records,
            "diff_test",
            "verifies_route",
            file,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.clone()),
        );
    }
    for file in &facts.foreign_files {
        push_diff_record(
            &mut records,
            "diff_foreign_file",
            "pulls_foreign_route",
            file,
            Some(route_for_file_from_facts(facts, file)),
            None,
            Some(file.clone()),
        );
    }
    for route in &facts.foreign_routes {
        push_diff_record(
            &mut records,
            "diff_foreign_route",
            "crosses_boundary",
            route,
            Some(route.clone()),
            None,
            None,
        );
    }
    records
}

fn push_diff_record(
    records: &mut Vec<FieldRecord>,
    subject: &str,
    relation: &str,
    object: &str,
    route: Option<String>,
    group: Option<String>,
    evidence_ref: Option<String>,
) {
    records.push(FieldRecord {
        id: format!("boundary-diff-record-{}", records.len()),
        kind: FieldRecordKind::StructuralTriad,
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        route,
        group,
        confidence: 255,
        evidence_ref,
    });
}
