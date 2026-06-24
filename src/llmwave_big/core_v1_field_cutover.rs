//! LLMWave Core V1 field-core cutover report.

use serde::Serialize;

use crate::field_core::{
    FIELD_COMPUTE_VERSION, FIELD_CORE_VERSION, FIELD_PASS_VERSION, FIELD_RUNTIME_VERSION,
};

pub(crate) const CORE_V1_FIELD_CUTOVER_VERSION: &str = "llmwave-core-v1-field-cutover-phase2";

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FieldCutoverReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub field_core_versions: CoreV1FieldCoreVersions,
    pub operation_contract: Vec<CoreV1FieldOperation>,
    pub family_cutovers: Vec<CoreV1FamilyCutover>,
    pub phase_2_exit_criteria: CoreV1FieldCutoverExitCriteria,
    pub claim_boundary: CoreV1FieldCutoverClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FieldCoreVersions {
    pub field_core: &'static str,
    pub field_compute: &'static str,
    pub field_pass: &'static str,
    pub field_runtime: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FieldOperation {
    pub operation: &'static str,
    pub owner: &'static str,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FamilyCutover {
    pub family: &'static str,
    pub old_role: &'static str,
    pub field_core_role: &'static str,
    pub cutover_state: &'static str,
    pub sole_field_operations_engine: bool,
    pub still_guarded_by: Vec<&'static str>,
    pub evidence: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FieldCutoverExitCriteria {
    pub phase_1_contract_present: bool,
    pub shared_field_operations_present: bool,
    pub structural_family_mapped: bool,
    pub packed_family_mapped: bool,
    pub cognitive_family_mapped: bool,
    pub claim_boundary_preserved: bool,
    pub docs_updated: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1FieldCutoverClaimBoundary {
    pub field_core_as_sole_field_operations_engine: bool,
    pub field_core_as_sole_llmwave_core_engine: bool,
    pub evidence_bound_answer_ready: bool,
    pub feedback_learning_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
    pub blocked_by: Vec<&'static str>,
}

pub(crate) fn build_core_v1_field_cutover_report() -> CoreV1FieldCutoverReport {
    CoreV1FieldCutoverReport {
        mode: "llmwave-core-v1-field-cutover",
        version: CORE_V1_FIELD_CUTOVER_VERSION,
        phase: "phase-2-field-core-cutover",
        verdict: "CORE_V1_FIELD_OPERATIONS_CUTOVER_RECORDED_NOT_LLM",
        objective: "make_field_core_the_single_owner_of_shared_peak_coherence_verdict_anti_wave_readout_operations",
        field_core_versions: CoreV1FieldCoreVersions {
            field_core: FIELD_CORE_VERSION,
            field_compute: FIELD_COMPUTE_VERSION,
            field_pass: FIELD_PASS_VERSION,
            field_runtime: FIELD_RUNTIME_VERSION,
        },
        operation_contract: core_v1_field_operations(),
        family_cutovers: core_v1_family_cutovers(),
        phase_2_exit_criteria: CoreV1FieldCutoverExitCriteria {
            phase_1_contract_present: true,
            shared_field_operations_present: true,
            structural_family_mapped: true,
            packed_family_mapped: true,
            cognitive_family_mapped: true,
            claim_boundary_preserved: true,
            docs_updated: true,
        },
        claim_boundary: CoreV1FieldCutoverClaimBoundary {
            field_core_as_sole_field_operations_engine: true,
            field_core_as_sole_llmwave_core_engine: false,
            evidence_bound_answer_ready: false,
            feedback_learning_ready: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Phase 2 records field_core as the shared field-operation engine for peak, coherence, verdict, anti-wave, readout, and local-path reports.",
            forbidden_claims: vec![
                "LLM ready",
                "general chatbot ready",
                "nonlinear memory proven",
                "Core V1 memory writer implemented",
                "Core V1 query wave implemented",
                "answer verifier implemented",
                "feedback learning loop complete",
            ],
            blocked_by: vec![
                "memory_writer_v1_missing",
                "query_wave_v1_missing",
                "answer_surface_verifier_missing",
                "feedback_learning_loop_missing",
                "broad_eval_missing",
            ],
        },
        next_phase: "phase-3-memory-writer-v1",
    }
}

fn core_v1_field_operations() -> Vec<CoreV1FieldOperation> {
    vec![
        CoreV1FieldOperation {
            operation: "peak_detection",
            owner: "field_core::peak::FieldPeakResult",
            evidence: "UnifiedFieldReport::to_value projects field_pass peak through field_core.",
        },
        CoreV1FieldOperation {
            operation: "coherence_state",
            owner: "field_core::coherence::FieldCoherenceResult",
            evidence: "FieldPassReport carries coherence_state and field verdict from field_core.",
        },
        CoreV1FieldOperation {
            operation: "verdict",
            owner: "field_core::coherence::field_verdict_for_state",
            evidence: "field_core maps focused/thin/contested/reversed states to PASS/WATCH/VETO.",
        },
        CoreV1FieldOperation {
            operation: "anti_wave",
            owner: "field_core::anti_wave::FieldAntiWaveEffect",
            evidence: "Unified reports include anti_wave_effect and FieldPass anti-support lanes.",
        },
        CoreV1FieldOperation {
            operation: "readout",
            owner: "field_core::readout::FieldReadoutResult",
            evidence: "Unified reports expose field_pass readout-compatible peak and evidence refs.",
        },
        CoreV1FieldOperation {
            operation: "local_path",
            owner: "field_core::readout::FieldLocalPathResult",
            evidence: "Field reports keep local-path ownership in field_core instead of family adapters.",
        },
        CoreV1FieldOperation {
            operation: "runtime_dual_run",
            owner: "field_core::runtime::FieldRuntimeDualRun",
            evidence: "structural, packed, and cognitive reports compare legacy/domain output against field_pass.",
        },
    ]
}

fn core_v1_family_cutovers() -> Vec<CoreV1FamilyCutover> {
    vec![
        CoreV1FamilyCutover {
            family: "structural-search",
            old_role: "domain_search_computed_peaks_and_verdicts_directly",
            field_core_role: "structural_dual_run_and_field_engine_decision_use_field_pass",
            cutover_state: "mapped_through_structural-field-engine-v1",
            sole_field_operations_engine: true,
            still_guarded_by: vec![
                "structural_cutover_suite",
                "field_not_more_permissive",
                "route_split_when_large",
            ],
            evidence: vec![
                "field_core::structural_dual_run_from_search",
                "field_core::structural_field_engine_decision",
                "nanda-field-cutover --suite structural-standard",
            ],
        },
        CoreV1FamilyCutover {
            family: "packed-runtime",
            old_role: "packed_hot_core_kept_a_typed_peak_decision",
            field_core_role:
                "packed_record_view_and_packed_field_engine_guard_use_field_pass_contract",
            cutover_state: "mapped_with_hot_loop_guard",
            sole_field_operations_engine: true,
            still_guarded_by: vec![
                "no_json_string_heap_hashmap_inner_loop",
                "typed_packed_decision_core",
                "hot_bench_guard",
            ],
            evidence: vec![
                "field_core::packed_dual_run_from_pack",
                "field_core::packed_field_engine_decision",
                "nanda_6m::evaluate_packed_peak_decision",
            ],
        },
        CoreV1FamilyCutover {
            family: "llmwave-cognitive",
            old_role: "llmwave_big_reports_projected_field_state_locally",
            field_core_role:
                "with_unified_field_projects_reports_through_field_core_and_cognitive_guard",
            cutover_state: "mapped_not_chat_engine",
            sole_field_operations_engine: true,
            still_guarded_by: vec![
                "not_llm_ready",
                "nonlinear_memory_not_proven",
                "full_field_mature_not_proven",
            ],
            evidence: vec![
                "llmwave_big::report::with_unified_field",
                "field_core::cognitive_dual_run_from_report",
                "field_core::cognitive_field_engine_decision",
            ],
        },
    ]
}
