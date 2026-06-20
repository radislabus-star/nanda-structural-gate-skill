//! Schema record boundary for reusable role, route, and operator patterns.

use serde::Serialize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct SchemaRecord {
    pub id: u32,
    pub operator_id: u16,
    pub subject_role: u16,
    pub object_role: u16,
    pub route_id: u16,
    pub centroid_id: u32,
    pub confidence: u8,
}

#[derive(Serialize, Clone)]
pub(crate) struct SchemaExample {
    pub id: u32,
    pub form: &'static str,
    pub route: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SchemaAtlasReport {
    pub version: &'static str,
    pub contract: &'static str,
    pub records: Vec<SchemaRecord>,
    pub examples: Vec<SchemaExample>,
}

pub(crate) fn build_schema_atlas_report() -> SchemaAtlasReport {
    SchemaAtlasReport {
        version: "v164-schema-atlas",
        contract: "schemas_are_cognitive_forms_not_individual_facts",
        records: vec![
            record(101, 3, 11, 21, 31, 401, 94),
            record(102, 4, 12, 22, 31, 402, 92),
            record(103, 1, 13, 23, 32, 403, 96),
            record(104, 2, 14, 24, 32, 404, 89),
            record(105, 6, 15, 25, 33, 405, 88),
            record(106, 5, 16, 26, 34, 406, 87),
        ],
        examples: vec![
            example(101, "supplier issues invoice", "business_docs"),
            example(102, "buyer pays supplier", "business_docs"),
            example(103, "declaration requires documents", "customs"),
            example(104, "certificate supports clearance", "customs"),
            example(105, "function calls dependency", "code_rust"),
            example(106, "config controls runtime", "code_rust"),
        ],
    }
}

fn record(
    id: u32,
    operator_id: u16,
    subject_role: u16,
    object_role: u16,
    route_id: u16,
    centroid_id: u32,
    confidence: u8,
) -> SchemaRecord {
    SchemaRecord {
        id,
        operator_id,
        subject_role,
        object_role,
        route_id,
        centroid_id,
        confidence,
    }
}

fn example(id: u32, form: &'static str, route: &'static str) -> SchemaExample {
    SchemaExample { id, form, route }
}
