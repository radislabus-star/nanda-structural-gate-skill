//! Residual-only write boundary for fact traces that schemas cannot reconstruct.

use serde::Serialize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ResidualRecord {
    pub schema_id: u32,
    pub subject_id: u32,
    pub object_id: u32,
    pub phase_delta: i16,
    pub evidence_ref: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct ResidualExample {
    pub id: u32,
    pub fact: &'static str,
    pub attached_schema: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct ResidualStoreReport {
    pub version: &'static str,
    pub contract: &'static str,
    pub records: Vec<ResidualRecord>,
    pub examples: Vec<ResidualExample>,
}

pub(crate) fn build_residual_store_report() -> ResidualStoreReport {
    ResidualStoreReport {
        version: "v165-residual-store",
        contract: "private_facts_are_residuals_attached_to_reusable_schemas",
        records: vec![
            record(101, 2_001, 3_001, 17, 10_001),
            record(102, 2_002, 3_002, -9, 10_002),
            record(103, 2_003, 3_003, 11, 10_003),
            record(105, 2_004, 3_004, 5, 10_004),
        ],
        examples: vec![
            example(1_001, "Honglu issued PI-03", 101),
            example(1_002, "Rustrade pays Honglu", 102),
            example(1_003, "Fanta made by Huizhou plant", 103),
            example(1_004, "function A calls function B", 105),
        ],
    }
}

fn record(
    schema_id: u32,
    subject_id: u32,
    object_id: u32,
    phase_delta: i16,
    evidence_ref: u32,
) -> ResidualRecord {
    ResidualRecord {
        schema_id,
        subject_id,
        object_id,
        phase_delta,
        evidence_ref,
    }
}

fn example(id: u32, fact: &'static str, attached_schema: u32) -> ResidualExample {
    ResidualExample {
        id,
        fact,
        attached_schema,
    }
}
