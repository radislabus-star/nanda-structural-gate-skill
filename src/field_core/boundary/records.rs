use crate::field_core::{FieldRecord, FieldRecordKind};

use super::{BoundaryFacts, BoundaryFieldRecordBridge};

impl BoundaryFieldRecordBridge {
    pub(super) fn from_facts(facts: &BoundaryFacts, records: &[FieldRecord]) -> Self {
        Self {
            version: "boundary-field-records-v1",
            owner: "field_core::boundary",
            record_count: records.len(),
            file_records: facts.files.len(),
            function_records: facts.functions.len(),
            public_api_records: facts.public_api.len(),
            call_edge_records: facts.call_edges.len(),
            shared_state_records: facts.shared_state.len(),
            runtime_side_effect_records: facts.runtime_side_effects.len(),
            test_records: facts.tests.len(),
            foreign_pull_records: facts.foreign_route_files.len(),
            sample: records.iter().take(16).cloned().collect(),
        }
    }
}

pub(super) fn boundary_field_records(facts: &BoundaryFacts) -> Vec<FieldRecord> {
    let mut records = vec![];
    for file in &facts.files {
        push_boundary_record(
            &mut records,
            "boundary_file",
            "belongs_to_route",
            file,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.clone()),
        );
    }
    for function in &facts.functions {
        let file = file_from_function(function);
        push_boundary_record(
            &mut records,
            "boundary_function",
            "declared_in",
            function,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for edge in &facts.public_api {
        let file = file_from_line_ref(edge);
        push_boundary_record(
            &mut records,
            "boundary_public_api",
            "exposes",
            edge,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for edge in &facts.call_edges {
        let file = edge.split(" -> ").next().unwrap_or(edge);
        push_boundary_record(
            &mut records,
            "boundary_call_edge",
            "connects",
            edge,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for item in &facts.shared_state {
        let file = file_from_line_ref(item);
        push_boundary_record(
            &mut records,
            "boundary_shared_state",
            "holds_state",
            item,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for item in &facts.runtime_side_effects {
        let file = file_from_line_ref(item);
        push_boundary_record(
            &mut records,
            "boundary_runtime_side_effect",
            "mutates_runtime",
            item,
            route_for_fact_file(facts, file),
            owner_for_fact_file(facts, file),
            Some(file.to_string()),
        );
    }
    for test in &facts.tests {
        push_boundary_record(
            &mut records,
            "boundary_test",
            "verifies_route",
            test,
            route_for_fact_file(facts, test),
            owner_for_fact_file(facts, test),
            Some(test.clone()),
        );
    }
    for foreign_file in &facts.foreign_route_files {
        push_boundary_record(
            &mut records,
            "boundary_foreign_pull",
            "pulls_foreign_route",
            foreign_file,
            route_for_fact_file(facts, foreign_file),
            owner_for_fact_file(facts, foreign_file),
            Some(foreign_file.clone()),
        );
    }
    records
}

fn push_boundary_record(
    records: &mut Vec<FieldRecord>,
    subject: &str,
    relation: &str,
    object: &str,
    route: Option<String>,
    group: Option<String>,
    evidence_ref: Option<String>,
) {
    records.push(FieldRecord {
        id: format!("boundary-record-{}", records.len()),
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

fn route_for_fact_file(facts: &BoundaryFacts, file: &str) -> Option<String> {
    facts.file_routes.get(file).cloned()
}

fn owner_for_fact_file(facts: &BoundaryFacts, file: &str) -> Option<String> {
    facts.file_owners.get(file).cloned()
}

fn file_from_function(function: &str) -> &str {
    function.split_once("::").map_or(function, |(file, _)| file)
}

fn file_from_line_ref(item: &str) -> &str {
    item.split_once(':').map_or(item, |(file, _)| file)
}
