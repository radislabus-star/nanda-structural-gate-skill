use serde::Serialize;

use super::nanda_6m;

#[derive(Serialize, Clone)]
pub(crate) struct BigModelContract {
    pub model_name: &'static str,
    pub contract_version: &'static str,
    pub goal: &'static str,
    pub hard_distinctions: Vec<&'static str>,
    pub layers: Vec<LayerContract>,
    pub active_core_budget: ActiveCoreBudget,
}

#[derive(Serialize, Clone)]
pub(crate) struct LayerContract {
    pub name: &'static str,
    pub residency: &'static str,
    pub responsibility: &'static str,
    pub input: &'static str,
    pub output: &'static str,
    pub must_not_contain: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ActiveCoreBudget {
    pub budget_bytes: usize,
    pub wave_dim: usize,
    pub triad_capacity: usize,
    pub runtime_focus_triad_capacity: usize,
    pub runtime_focus_field_requests: usize,
    pub contract_note: &'static str,
}

pub(crate) fn build_contract() -> BigModelContract {
    BigModelContract {
        model_name: "LLMWave-Big",
        contract_version: "v158-big-model-contract",
        goal: "large_long_term_wave_memory_with_small_cache_resident_active_core",
        hard_distinctions: vec![
            "model_size_is_not_active_core_size",
            "context_size_is_not_hot_memory_size",
            "long_term_memory_size_is_not_cache_resident_focus",
        ],
        layers: vec![
            LayerContract {
                name: "Wave Atlas",
                residency: "cold_or_warm_long_term_memory",
                responsibility: "facts_schemas_documents_code_cartridges",
                input: "indexed_symbols_operators_schemas_residuals_evidence",
                output: "candidate_active_focus_sets",
                must_not_contain: vec!["hot_inner_loop_state"],
            },
            LayerContract {
                name: "Active Core",
                residency: "small_hot_focus_target_6_to_8_mb",
                responsibility: "excite_settle_peak_reconstruct_active_packet",
                input: "compact_ids_phases_seeds_lanes_centroids",
                output: "resonant_peak_support_anti_support_verdict_signals",
                must_not_contain: vec!["json", "strings", "heap_hashmaps_in_inner_loop"],
            },
            LayerContract {
                name: "L2 Word Field",
                residency: "fast_local_surface_field",
                responsibility: "token_root_morpheme_word_candidates",
                input: "prefix_surface_context_l3_bias",
                output: "ranked_surface_candidates",
                must_not_contain: vec!["schema_route_authority", "document_evidence_text"],
            },
            LayerContract {
                name: "L3 Schema Field",
                residency: "active_schema_operator_route_field",
                responsibility: "operators_roles_routes_schema_cognition",
                input: "active_packet_schema_records_residuals",
                output: "schema_bias_role_expectations_route_vetoes",
                must_not_contain: vec!["surface_token_storage", "morpheme_cache"],
            },
            LayerContract {
                name: "Residual Store",
                residency: "cold_to_warm_private_fact_traces",
                responsibility: "record_only_non_reconstructable_fact_delta",
                input: "failed_reconstruction_or_new_specific_fact",
                output: "residual_records_for_later_consolidation",
                must_not_contain: vec!["duplicate_schema_information"],
            },
            LayerContract {
                name: "Consolidator",
                residency: "cold_background_sleep_pass",
                responsibility:
                    "promote_reused_residuals_into_schemas_split_bad_schemas_decay_noise",
                input: "residual_store_schema_usage_feedback",
                output: "updated_schema_operator_residual_tables",
                must_not_contain: vec!["hot_request_latency_path"],
            },
            LayerContract {
                name: "Loader",
                residency: "cold_to_hot_boundary",
                responsibility: "select_focus_compile_active_packet_preserve_l2_l3_separation",
                input: "query_task_atlas_indexes_budget",
                output: "active_packet_for_hot_core",
                must_not_contain: vec!["claim_decisions_without_eval"],
            },
        ],
        active_core_budget: ActiveCoreBudget {
            budget_bytes: nanda_6m::BUDGET_BYTES,
            wave_dim: nanda_6m::WAVE_DIM,
            triad_capacity: nanda_6m::TRIAD_CAPACITY,
            runtime_focus_triad_capacity: nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY,
            runtime_focus_field_requests: nanda_6m::RUNTIME_FOCUS_FIELD_REQUESTS,
            contract_note: "budget_is_existing_nanda_6m_hot_contract_not_big_model_size",
        },
    }
}
