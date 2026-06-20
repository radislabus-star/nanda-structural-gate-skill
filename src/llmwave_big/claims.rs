use serde::Serialize;

#[derive(Serialize, Clone)]
pub(crate) struct ClaimBoundary {
    pub contract_version: &'static str,
    pub current_state: &'static str,
    pub allowed_states: Vec<&'static str>,
    pub forbidden_claims: Vec<&'static str>,
    pub claims: ClaimFlags,
    pub safe_claim: &'static str,
    pub escalation_rules: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ClaimFlags {
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub cache_only_execution_proven: bool,
    pub big_cognition_proven: bool,
    pub contract_defined: bool,
}

pub(crate) fn build_claim_boundary() -> ClaimBoundary {
    ClaimBoundary {
        contract_version: "v160-claim-boundary",
        current_state: "BIG_MODEL_NOT_PROVEN",
        allowed_states: vec![
            "BIG_MODEL_NOT_PROVEN",
            "BIG_MEMORY_INDEXED",
            "ACTIVE_CORE_WORKS",
            "SCHEMA_COMPRESSION_WORKS",
            "COGNITIVE_LIFT_CANDIDATE",
            "LLMWAVE_BIG_CANDIDATE",
        ],
        forbidden_claims: vec![
            "LLM_READY",
            "NONLINEAR_MEMORY_PROVEN",
            "CACHE_ONLY_EXECUTION_PROVEN",
            "100K_RECORDS_MEANS_COGNITION",
            "PHASE_COHERENCE_MEANS_UNDERSTANDING",
        ],
        claims: ClaimFlags {
            llm_ready: false,
            nonlinear_memory_proven: false,
            cache_only_execution_proven: false,
            big_cognition_proven: false,
            contract_defined: true,
        },
        safe_claim: "v158-v160 defines the LLMWave-Big contract, metrics, and claim firewall only",
        escalation_rules: vec![
            "do_not_raise_claim_state_without_eval_verdict",
            "do_not_treat_more_records_as_more_cognition",
            "do_not_use_coherence_without_role_error_and_false_positive_metrics",
            "do_not_claim_cache_only_execution_from_cold_json_cli_timing",
        ],
    }
}
