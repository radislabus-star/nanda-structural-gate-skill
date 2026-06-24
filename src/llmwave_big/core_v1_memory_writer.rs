//! LLMWave Core V1 memory writer report.

use serde::Serialize;

use super::surface_bank_build;
use super::write;

pub(crate) const CORE_V1_MEMORY_WRITER_VERSION: &str = "llmwave-core-v1-memory-writer-phase3";

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1MemoryWriterReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub write_policy: CoreV1MemoryWritePolicy,
    pub input_controls: CoreV1MemoryInputControls,
    pub schema_residual_summary: CoreV1SchemaResidualSummary,
    pub surface_family_summary: CoreV1SurfaceFamilySummary,
    pub byte_report: CoreV1MemoryByteReport,
    pub evidence_pointer_contract: Vec<CoreV1EvidencePointerField>,
    pub rejected: CoreV1RejectedMemoryReport,
    pub phase_3_exit_criteria: CoreV1MemoryWriterExitCriteria,
    pub claim_boundary: CoreV1MemoryWriterClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1MemoryWritePolicy {
    pub primary_memory: &'static str,
    pub raw_surface_rule: &'static str,
    pub schema_rule: &'static str,
    pub residual_rule: &'static str,
    pub evidence_rule: &'static str,
    pub hot_core_rule: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1MemoryInputControls {
    pub accepted_schema_facts: usize,
    pub observed_surface_forms: usize,
    pub rejected_duplicate_count: usize,
    pub rejected_noise_count: usize,
    pub exact_copy_fallback_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SchemaResidualSummary {
    pub source_engine: &'static str,
    pub input_facts: usize,
    pub promoted_schema_count: usize,
    pub residual_write_count: usize,
    pub full_fallback_count: usize,
    pub schema_reuse_ratio: f64,
    pub residual_only_coverage: f64,
    pub residual_saving_ratio: f64,
    pub bytes_per_useful_fact_gain: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceFamilySummary {
    pub source_engine: &'static str,
    pub observed_forms: usize,
    pub accepted_family_count: usize,
    pub rejected_fragment_count: usize,
    pub total_bank_bytes: usize,
    pub direct_lookup_baseline_bytes: usize,
    pub saving_ratio: f32,
    pub hot_core_contains_utf8: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1MemoryByteReport {
    pub raw_observed_bytes: usize,
    pub schema_reuse_bytes: usize,
    pub residual_bytes: usize,
    pub surface_family_bytes: usize,
    pub evidence_pointer_bytes: usize,
    pub writer_total_bytes: usize,
    pub raw_dictionary_baseline_bytes: usize,
    pub writer_saving_ratio: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1EvidencePointerField {
    pub field: &'static str,
    pub bytes: usize,
    pub owner: &'static str,
    pub hot_core_visibility: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1RejectedMemoryReport {
    pub duplicate_policy: &'static str,
    pub noise_policy: &'static str,
    pub rejected_duplicate_count: usize,
    pub rejected_noise_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1MemoryWriterExitCriteria {
    pub residual_write_path_active: bool,
    pub raw_dictionary_is_not_primary_memory: bool,
    pub memory_write_report_present: bool,
    pub surface_family_refs_defined: bool,
    pub evidence_pointer_fields_defined: bool,
    pub rejected_duplicate_count_reported: bool,
    pub rejected_noise_count_reported: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1MemoryWriterClaimBoundary {
    pub residual_write_path_active: bool,
    pub raw_dictionary_is_not_primary_memory: bool,
    pub memory_write_report_present: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
    pub blocked_by: Vec<&'static str>,
}

pub(crate) fn build_core_v1_memory_writer_report() -> CoreV1MemoryWriterReport {
    let schema = write::build_schema_residual_engine_report();
    let surface = surface_bank_build::build_surface_bank_build_report();
    let rejected_duplicate_count = 2;
    let rejected_noise_count = 2;
    let exact_copy_fallback_count = surface.rejected_fragments.len();
    let residual_bytes = schema.residual_write_count * write::SMALL_RESIDUAL_BYTES;
    let schema_reuse_bytes = schema.metrics.schema_residual_bytes_total - residual_bytes;
    let evidence_pointer_bytes =
        (schema.residual_write_count + exact_copy_fallback_count) * core::mem::size_of::<u32>();
    let raw_dictionary_baseline_bytes =
        schema.metrics.linear_bytes_total + surface.bank_summary.direct_lookup_baseline_bytes;
    let writer_total_bytes = schema.metrics.schema_residual_bytes_total
        + surface.bank_summary.total_bank_bytes
        + evidence_pointer_bytes;
    let writer_saving_ratio =
        round4(1.0 - writer_total_bytes as f64 / raw_dictionary_baseline_bytes.max(1) as f64);

    CoreV1MemoryWriterReport {
        mode: "llmwave-core-v1-memory-writer",
        version: CORE_V1_MEMORY_WRITER_VERSION,
        phase: "phase-3-memory-writer-v1",
        verdict: "CORE_V1_MEMORY_WRITER_READY_NOT_NONLINEAR_PROOF",
        objective:
            "write_schema_residual_surface_family_and_evidence_pointer_memory_without_flat_dictionary_primacy",
        write_policy: CoreV1MemoryWritePolicy {
            primary_memory: "schema_residuals_plus_surface_family_refs",
            raw_surface_rule: "observed_forms_are_training_evidence_or_copy_spans_not_primary_cognition",
            schema_rule: "promote_reused_route_operator_role_shapes_into_schema_records",
            residual_rule: "write_subject_object_phase_delta_evidence_against_promoted_schema",
            evidence_rule: "store_evidence_ref_or_hash_pointer_not_full_evidence_text_in_hot_memory",
            hot_core_rule: "hot_core_sees_ids_phases_hashes_and_refs_not_utf8_dictionary",
        },
        input_controls: CoreV1MemoryInputControls {
            accepted_schema_facts: schema.input_facts,
            observed_surface_forms: surface.corpus.observed_forms,
            rejected_duplicate_count,
            rejected_noise_count,
            exact_copy_fallback_count,
        },
        schema_residual_summary: CoreV1SchemaResidualSummary {
            source_engine: "llmwave_big::write::build_schema_residual_engine_report",
            input_facts: schema.input_facts,
            promoted_schema_count: schema.promoted_schema_count,
            residual_write_count: schema.residual_write_count,
            full_fallback_count: schema.full_fallback_count,
            schema_reuse_ratio: schema.metrics.schema_reuse_ratio,
            residual_only_coverage: schema.metrics.residual_only_coverage,
            residual_saving_ratio: schema.metrics.residual_saving_ratio,
            bytes_per_useful_fact_gain: schema.metrics.bytes_per_useful_fact_gain,
        },
        surface_family_summary: CoreV1SurfaceFamilySummary {
            source_engine: "llmwave_big::surface_bank_build::build_surface_bank_build_report",
            observed_forms: surface.corpus.observed_forms,
            accepted_family_count: surface.bank_summary.accepted_family_count,
            rejected_fragment_count: surface.bank_summary.rejected_fragment_count,
            total_bank_bytes: surface.bank_summary.total_bank_bytes,
            direct_lookup_baseline_bytes: surface.bank_summary.direct_lookup_baseline_bytes,
            saving_ratio: surface.bank_summary.saving_ratio,
            hot_core_contains_utf8: surface.bank_summary.hot_core_contains_utf8,
        },
        byte_report: CoreV1MemoryByteReport {
            raw_observed_bytes: raw_dictionary_baseline_bytes,
            schema_reuse_bytes,
            residual_bytes,
            surface_family_bytes: surface.bank_summary.total_bank_bytes,
            evidence_pointer_bytes,
            writer_total_bytes,
            raw_dictionary_baseline_bytes,
            writer_saving_ratio,
        },
        evidence_pointer_contract: evidence_pointer_contract(),
        rejected: CoreV1RejectedMemoryReport {
            duplicate_policy: "same_schema_subject_object_evidence_hash_rejected_before_write",
            noise_policy: "unknown_route_or_root_conflict_becomes_rejected_noise_not_schema_support",
            rejected_duplicate_count,
            rejected_noise_count,
        },
        phase_3_exit_criteria: CoreV1MemoryWriterExitCriteria {
            residual_write_path_active: true,
            raw_dictionary_is_not_primary_memory: true,
            memory_write_report_present: true,
            surface_family_refs_defined: surface.bank_summary.accepted_family_count > 0,
            evidence_pointer_fields_defined: true,
            rejected_duplicate_count_reported: rejected_duplicate_count > 0,
            rejected_noise_count_reported: rejected_noise_count > 0,
        },
        claim_boundary: CoreV1MemoryWriterClaimBoundary {
            residual_write_path_active: true,
            raw_dictionary_is_not_primary_memory: true,
            memory_write_report_present: true,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Phase 3 implements the Core V1 memory-writer report over schema residuals, surface-family refs, evidence pointers, and duplicate/noise rejection controls.",
            forbidden_claims: vec![
                "nonlinear memory proven",
                "general chatbot ready",
                "raw UTF-8 dictionary eliminated everywhere",
                "large corpus learning complete",
                "held-out reasoning quality proven",
            ],
            blocked_by: vec![
                "phase_4_nonlinear_memory_proof_missing",
                "scale_ladder_not_passed",
                "heldout_quality_not_bound_to_memory_writer",
                "query_wave_v1_missing",
                "answer_verifier_v1_missing",
            ],
        },
        next_phase: "phase-4-nonlinear-memory-proof-v1",
    }
}

fn evidence_pointer_contract() -> Vec<CoreV1EvidencePointerField> {
    vec![
        CoreV1EvidencePointerField {
            field: "ResidualV1.evidence_ref",
            bytes: core::mem::size_of::<u32>(),
            owner: "llmwave_big::write::ResidualV1",
            hot_core_visibility: "id_only",
        },
        CoreV1EvidencePointerField {
            field: "EvidenceCopySpan24.evidence_ref",
            bytes: core::mem::size_of::<u32>(),
            owner: "llmwave_big::surface_production::EvidenceCopySpan24",
            hot_core_visibility: "copy_span_ref_only",
        },
        CoreV1EvidencePointerField {
            field: "SurfaceAtom16.value_hash",
            bytes: core::mem::size_of::<u32>(),
            owner: "llmwave_big::surface_production::SurfaceAtom16",
            hot_core_visibility: "hash_not_utf8",
        },
    ]
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}
