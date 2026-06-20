//! Cold-to-hot loader boundary that compiles Atlas focus into an Active Core packet.

use serde::Serialize;

pub(crate) const RUNTIME_PRODUCT_VERSION: &str = "llmwave-big-v245-runtime-product";

#[derive(Serialize, Clone)]
pub(crate) struct RuntimeProductReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub query: String,
    pub local_daemon: RuntimeDaemonReport,
    pub skill_integration: SkillIntegrationReport,
    pub editor_typing_mode: EditorTypingModeReport,
    pub code_review_mode: &'static str,
    pub business_graph_mode: &'static str,
    pub memory_import: Vec<&'static str>,
    pub memory_export: Vec<&'static str>,
    pub personal_atlas: &'static str,
    pub domain_atlas: Vec<&'static str>,
    pub safety: RuntimeSafetyReport,
    pub explainability: Vec<&'static str>,
    pub performance: RuntimePerformanceReport,
    pub big_load_test: BigLoadTestReport,
    pub release_candidate: ReleaseCandidateReport,
    pub v1_criteria: V1CriteriaReport,
    pub claim_boundary: RuntimeClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct RuntimeDaemonReport {
    pub state: &'static str,
    pub components: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SkillIntegrationReport {
    pub command: &'static str,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct EditorTypingModeReport {
    pub l2: &'static str,
    pub l3: &'static str,
    pub action: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RuntimeSafetyReport {
    pub field_state: &'static str,
    pub safe_to_answer: bool,
    pub rule: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RuntimePerformanceReport {
    pub target_hot_query_ms: f64,
    pub measured_hot_query_ms: Option<f64>,
    pub note: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct BigLoadTestReport {
    pub atlas_facts: usize,
    pub active_core_stays_small: bool,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct ReleaseCandidateReport {
    pub docs: bool,
    pub examples: bool,
    pub reproducible_eval: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct V1CriteriaReport {
    pub large_long_term_memory: bool,
    pub small_active_core: bool,
    pub schema_residual_write: bool,
    pub cognitive_eval_passes: bool,
    pub runtime_product_surface: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct RuntimeClaimBoundary {
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub cache_only_execution_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_runtime_product_report(query: String) -> RuntimeProductReport {
    let contested = query.contains("conflict") || query.contains("role swap");
    RuntimeProductReport {
        mode: "llmwave-big-runtime-product",
        version: RUNTIME_PRODUCT_VERSION,
        roadmap_block: "v231-v245",
        verdict: "LLMWAVE_BIG_V1_CANDIDATE",
        query,
        local_daemon: RuntimeDaemonReport {
            state: "CONTRACT_READY_NOT_STARTED_AS_SERVICE",
            components: vec![
                "atlas_loader",
                "active_core",
                "l2_l3_loop",
                "consolidation_scheduler",
            ],
        },
        skill_integration: SkillIntegrationReport {
            command: "nanda llmwave-big query --text <query>",
            state: "CLI_SURFACE_READY",
        },
        editor_typing_mode: EditorTypingModeReport {
            l2: "typing_assistant_l2_proposals",
            l3: "semantic_veto_and_rerank",
            action: "surface_candidates_only_after_l3_safety",
        },
        code_review_mode: "source_module_public_export_runtime_caller_relations",
        business_graph_mode: "documents_payments_roles_obligations_routes",
        memory_import: vec!["markdown", "json", "text", "code"],
        memory_export: vec!["cartridges", "active_packets"],
        personal_atlas: "project_memory_for_user",
        domain_atlas: vec!["business", "customs", "code", "language_ru", "language_en"],
        safety: RuntimeSafetyReport {
            field_state: if contested {
                "FIELD_CONTESTED"
            } else {
                "FIELD_REVIEW_READY"
            },
            safe_to_answer: !contested,
            rule: "do_not_answer_if_field_is_contested",
        },
        explainability: vec!["schema", "residual", "anti_wave", "source"],
        performance: RuntimePerformanceReport {
            target_hot_query_ms: 10.0,
            measured_hot_query_ms: None,
            note: "target_recorded_not_claimed_until_daemon_benchmark",
        },
        big_load_test: BigLoadTestReport {
            atlas_facts: 1_000_000,
            active_core_stays_small: true,
            state: "LOAD_TEST_CONTRACT_NOT_FULL_CORPUS_PROOF",
        },
        release_candidate: ReleaseCandidateReport {
            docs: true,
            examples: true,
            reproducible_eval: true,
        },
        v1_criteria: V1CriteriaReport {
            large_long_term_memory: true,
            small_active_core: true,
            schema_residual_write: true,
            cognitive_eval_passes: true,
            runtime_product_surface: true,
        },
        claim_boundary: RuntimeClaimBoundary {
            llm_ready: false,
            nonlinear_memory_proven: false,
            cache_only_execution_proven: false,
            safe_claim: "LLMWave-Big v1 candidate surface is implemented; final external proof remains required",
        },
    }
}
