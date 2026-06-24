//! LLMWave Core V1 architectural contract.

use serde::Serialize;

pub(crate) const CORE_V1_CONTRACT_VERSION: &str = "llmwave-core-v1-contract-phase1";

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ContractReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub model_loop: Vec<&'static str>,
    pub components: Vec<CoreV1Component>,
    pub required_boundaries: Vec<CoreV1Boundary>,
    pub claim_boundary: CoreV1ClaimBoundary,
    pub phase_1_exit_criteria: CoreV1ExitCriteria,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1Component {
    pub name: &'static str,
    pub route: &'static str,
    pub responsibility: &'static str,
    pub input: &'static str,
    pub output: &'static str,
    pub must_not_own: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1Boundary {
    pub rule: &'static str,
    pub owner: &'static str,
    pub enforced_by: &'static str,
    pub blocked_claim_if_broken: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ClaimBoundary {
    pub core_contract_recorded: bool,
    pub claim_boundary_table_present: bool,
    pub field_core_as_sole_engine: bool,
    pub evidence_bound_answer_ready: bool,
    pub feedback_learning_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ExitCriteria {
    pub core_contract_recorded: bool,
    pub claim_boundary_table_present: bool,
    pub l2_l3_boundary_recorded: bool,
    pub verifier_generator_boundary_recorded: bool,
    pub feedback_packet_boundary_recorded: bool,
    pub implementation_started: bool,
}

pub(crate) fn build_core_v1_contract_report() -> CoreV1ContractReport {
    CoreV1ContractReport {
        mode: "llmwave-core-v1-contract",
        version: CORE_V1_CONTRACT_VERSION,
        phase: "phase-1-core-v1-contract",
        verdict: "CORE_V1_CONTRACT_RECORDED_NOT_IMPLEMENTED",
        objective: "define_the_single_model_loop_and_claim_boundaries_before_core_execution",
        model_loop: vec![
            "corpus_or_atlas",
            "nonlinear_memory_write",
            "active_field",
            "route_schema_retrieval",
            "surface_generation",
            "evidence_and_anti_wave_verification",
            "feedback_learning",
            "consolidation",
            "baseline_eval",
        ],
        components: core_v1_components(),
        required_boundaries: core_v1_boundaries(),
        claim_boundary: CoreV1ClaimBoundary {
            core_contract_recorded: true,
            claim_boundary_table_present: true,
            field_core_as_sole_engine: false,
            evidence_bound_answer_ready: false,
            feedback_learning_ready: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Phase 1 records the LLMWave Core V1 architecture and claim boundary only.",
            forbidden_claims: vec![
                "LLM ready",
                "general chatbot ready",
                "nonlinear memory proven",
                "cache-only execution proven",
                "full semantic understanding proven",
                "GPT-class comparison won",
            ],
            blocked_by: vec![
                "field_core_cutover_missing",
                "memory_writer_v1_missing",
                "query_wave_v1_missing",
                "answer_verifier_v1_missing",
                "feedback_learning_loop_missing",
                "scale_baseline_eval_missing",
            ],
        },
        phase_1_exit_criteria: CoreV1ExitCriteria {
            core_contract_recorded: true,
            claim_boundary_table_present: true,
            l2_l3_boundary_recorded: true,
            verifier_generator_boundary_recorded: true,
            feedback_packet_boundary_recorded: true,
            implementation_started: false,
        },
        next_phase: "phase-2-field-core-cutover",
    }
}

fn core_v1_components() -> Vec<CoreV1Component> {
    vec![
        CoreV1Component {
            name: "Cold Atlas",
            route: "cold-atlas",
            responsibility: "store_large_corpus_indexes_evidence_schema_candidates",
            input: "public_safe_corpus_or_project_artifact",
            output: "candidate_focus_sets_and_evidence_refs",
            must_not_own: vec!["hot_inner_loop", "answer_authorization"],
        },
        CoreV1Component {
            name: "Memory Writer",
            route: "memory-write",
            responsibility:
                "write_schema_surface_and_residual_memory_without_flat_token_table_primacy",
            input: "observed_facts_surfaces_feedback",
            output: "schema_residuals_surface_families_evidence_refs",
            must_not_own: vec!["final_claims", "answer_text_without_verifier"],
        },
        CoreV1Component {
            name: "Active Core",
            route: "active-core",
            responsibility: "settle_small_focus_packet_inside_hot_field_budget",
            input: "compact_focus_records_lanes_centroids",
            output: "peaks_support_anti_support_field_state",
            must_not_own: vec!["cold_evidence_text", "json_hot_loop", "raw_utf8_dictionary"],
        },
        CoreV1Component {
            name: "Field Engine",
            route: "field-core",
            responsibility: "compute_peak_coherence_verdict_anti_wave_and_feedback_delta",
            input: "field_input_records_lenses_feedback",
            output: "field_pass_report",
            must_not_own: vec!["surface_prose_generation", "cold_loader_policy"],
        },
        CoreV1Component {
            name: "Schema Field",
            route: "l3-schema-field",
            responsibility: "own_roles_routes_operators_evidence_and_decision_structure",
            input: "schema_records_operator_records_route_context",
            output: "schema_answer_plan_route_vetoes_role_expectations",
            must_not_own: vec!["surface_token_storage", "morpheme_cache"],
        },
        CoreV1Component {
            name: "Surface Field",
            route: "l2-surface-field",
            responsibility: "produce_roots_morphemes_word_forms_phrase_and_style_candidates",
            input: "surface_context_schema_bias_style_profile",
            output: "ranked_surface_candidates",
            must_not_own: vec!["route_authority", "schema_truth", "evidence_approval"],
        },
        CoreV1Component {
            name: "Answer Generator",
            route: "answer-surface",
            responsibility: "materialize_evidence_bound_surface_from_schema_answer_plan",
            input: "schema_answer_plan_field_evidence_surface_memory",
            output: "draft_answer_surface",
            must_not_own: vec!["self_authorized_pass", "unsupported_fact_creation"],
        },
        CoreV1Component {
            name: "Verifier",
            route: "answer-verifier",
            responsibility: "authorize_or_reject_draft_answer_against_field_evidence_and_anti_wave",
            input: "draft_answer_field_pass_evidence_refs_claim_policy",
            output: "answer_pass_watch_veto_or_no_evidence",
            must_not_own: vec!["new_answer_generation", "memory_write_side_effects"],
        },
        CoreV1Component {
            name: "Feedback Memory",
            route: "feedback-memory",
            responsibility:
                "turn_accept_reject_watch_correct_feedback_into_explicit_memory_packets",
            input: "answer_report_feedback_decision_correction",
            output: "positive_lanes_negative_lanes_schema_surface_corrections",
            must_not_own: vec!["silent_memory_mutation", "unrelated_route_updates"],
        },
        CoreV1Component {
            name: "Consolidator",
            route: "consolidation",
            responsibility: "merge_duplicates_promote_schemas_decay_noise_preserve_conflicts",
            input: "memory_packets_feedback_residuals_eval_metrics",
            output: "compacted_memory_next_atlas",
            must_not_own: vec!["request_hot_path", "claim_opening_without_eval"],
        },
        CoreV1Component {
            name: "Eval Harness",
            route: "eval-harness",
            responsibility: "compare_memory_field_answer_feedback_against_baselines",
            input: "heldout_suites_baselines_reports",
            output: "claim_gate_evidence",
            must_not_own: vec!["runtime_answer_authority", "training_data_generation"],
        },
    ]
}

fn core_v1_boundaries() -> Vec<CoreV1Boundary> {
    vec![
        CoreV1Boundary {
            rule: "L2 does not own L3 schema decisions.",
            owner: "Schema Field",
            enforced_by: "l2_l3_boundary_tests_and_answer_verifier",
            blocked_claim_if_broken: "small_domain_llmwave",
        },
        CoreV1Boundary {
            rule: "L3 does not store raw UTF-8 dictionary as primary cognition.",
            owner: "Memory Writer",
            enforced_by: "memory_writer_density_and_surface_family_reports",
            blocked_claim_if_broken: "nonlinear_memory_candidate",
        },
        CoreV1Boundary {
            rule: "Verifier does not generate.",
            owner: "Verifier",
            enforced_by: "answer_verifier_contract_tests",
            blocked_claim_if_broken: "evidence_bound_chat_ready",
        },
        CoreV1Boundary {
            rule: "Generator does not self-authorize PASS.",
            owner: "Answer Generator",
            enforced_by: "answer_surface_must_pass_verifier",
            blocked_claim_if_broken: "llm_ready",
        },
        CoreV1Boundary {
            rule: "Feedback changes memory only through explicit packets.",
            owner: "Feedback Memory",
            enforced_by: "feedback_packet_audit_and_route_guard",
            blocked_claim_if_broken: "feedback_learning_ready",
        },
    ]
}
