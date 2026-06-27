use serde::Serialize;

use crate::field_core;

#[derive(Serialize, Clone)]
pub(crate) struct ReadinessLadderReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub current_level: u8,
    pub current_state: &'static str,
    pub levels: Vec<ReadinessLevel>,
    pub(crate) llmwave_migration: LlmwaveMigrationReport,
    pub claim_boundary: ReadinessClaims,
}

#[derive(Serialize, Clone)]
pub(crate) struct ReadinessLevel {
    pub level: u8,
    pub name: &'static str,
    pub state: &'static str,
    pub evidence: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ReadinessClaims {
    pub field_core_as_sole_engine: bool,
    pub(crate) llmwave_field_core_backed: bool,
    pub(crate) pattern16_macro_cell_ready: bool,
    pub(crate) query_wave_ready: bool,
    pub(crate) schema_residual_memory_ready: bool,
    pub(crate) feedback_changes_next_field: bool,
    pub(crate) hierarchical_split_gate_ready: bool,
    pub(crate) answer_permission_gate_ready: bool,
    pub(crate) surface_generation_skeleton_first: bool,
    pub(crate) hot_runtime_ready: bool,
    pub(crate) llmwave_migration_ready: bool,
    pub active_65k_runtime_ready: bool,
    pub fixture_reasoning_ready: bool,
    pub artifact_grounded_qa_ready: bool,
    pub constrained_answer_generation_ready: bool,
    pub scripted_hot_multi_turn_ready: bool,
    pub small_domain_llmwave_ready: bool,
    pub scale_amortized_nonlinear_memory_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LlmwaveMigrationReport {
    version: &'static str,
    pub(crate) verdict: &'static str,
    field_core_cutover_ready: bool,
    pattern16_macro_cell_ready: bool,
    query_wave_ready: bool,
    schema_residual_memory_ready: bool,
    feedback_changes_next_field: bool,
    hierarchical_split_gate_ready: bool,
    answer_permission_gate_ready: bool,
    surface_generation_skeleton_first: bool,
    hot_runtime_ready: bool,
    final_claims_blocked: bool,
    phases: Vec<LlmwaveMigrationPhase>,
    blocked_claims: Vec<&'static str>,
    next_gate: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LlmwaveMigrationPhase {
    phase: u8,
    name: &'static str,
    owner: &'static str,
    status: &'static str,
    evidence: Vec<&'static str>,
    blocked_claims: Vec<&'static str>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum ClaimGateKind {
    #[value(name = "field-core-sole-engine")]
    FieldCoreSoleEngine,
    #[value(name = "active-65k-runtime")]
    Active65kRuntime,
    #[value(name = "fixture-reasoning")]
    FixtureReasoning,
    #[value(name = "artifact-grounded-qa")]
    ArtifactGroundedQa,
    #[value(name = "small-domain-llmwave")]
    SmallDomainLlmwave,
    #[value(name = "llm-ready")]
    LlmReady,
    #[value(name = "nonlinear-memory")]
    NonlinearMemory,
}

#[derive(Serialize, Clone)]
pub(crate) struct ClaimGateReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub claim: &'static str,
    pub verdict: &'static str,
    pub allowed: bool,
    pub evidence: Vec<&'static str>,
    pub missing_evidence: Vec<&'static str>,
    pub claim_boundary: ReadinessClaims,
}

pub(crate) fn build_readiness_ladder_report() -> ReadinessLadderReport {
    let claims = readiness_claims();
    let llmwave_migration = build_llmwave_migration_report(&claims);
    ReadinessLadderReport {
        mode: "llmwave-big-readiness-ladder",
        version: "llmwave-big-v-next-readiness-ladder",
        current_level: 3,
        current_state: "CONSTRAINED_FIELD_ENGINE_READY_NOT_GENERAL_LLM",
        levels: vec![
            ReadinessLevel {
                level: 0,
                name: "field engine ready",
                state: "PASS",
                evidence: vec![
                    "structural field-core sole engine",
                    "packed field-core sole engine",
                    "cognitive field-core sole engine",
                ],
            },
            ReadinessLevel {
                level: 1,
                name: "fixture reasoning ready",
                state: "PASS",
                evidence: vec![
                    "mini-chat fixture controls",
                    "role swap rejection",
                    "route splice rejection",
                    "applied feedback runtime fixture",
                ],
            },
            ReadinessLevel {
                level: 2,
                name: "artifact-grounded QA ready",
                state: "PASS_LOCAL",
                evidence: vec![
                    "trained artifact ask path",
                    "artifact ask eval false-positive gate",
                    "hot ask polarity lens",
                ],
            },
            ReadinessLevel {
                level: 3,
                name: "constrained answer generation ready",
                state: "LOCAL_ONLY",
                evidence: vec![
                    "evidence proof gate",
                    "answer surface templates",
                    "unsupported certainty blocked",
                ],
            },
            ReadinessLevel {
                level: 4,
                name: "multi-turn memory ready",
                state: "LOCAL_PASS_GENERAL_BLOCKED",
                evidence: vec![
                    "scripted chat-hot eval observes memory lift",
                    "hot feedback memory changes next ask",
                    "broad unscripted multi-turn eval missing",
                ],
            },
            ReadinessLevel {
                level: 5,
                name: "small domain LLMWave ready",
                state: "LOCAL_PASS_GENERAL_BLOCKED",
                evidence: vec![
                    "domain-eval combines artifact QA, hot chat, and scale memory",
                    "broad chat/general LLM eval missing",
                ],
            },
            ReadinessLevel {
                level: 6,
                name: "scale-amortized nonlinear memory",
                state: "LOCAL_PASS_GENERAL_BLOCKED",
                evidence: vec![
                    "nonlinear-memory-eval scale candidate",
                    "external fixture and noise controls pass",
                    "strict full-sweep nonlinear proof still blocked",
                ],
            },
            ReadinessLevel {
                level: 7,
                name: "broad chat candidate",
                state: "BLOCKED",
                evidence: vec!["external broad chat eval missing", "safety eval missing"],
            },
            ReadinessLevel {
                level: 8,
                name: "general LLM comparable eval",
                state: "BLOCKED",
                evidence: vec![
                    "general benchmark evidence missing",
                    "nonlinear memory proof missing",
                ],
            },
        ],
        llmwave_migration,
        claim_boundary: claims,
    }
}

pub(crate) fn build_claim_gate_report(claim: ClaimGateKind) -> ClaimGateReport {
    let claims = readiness_claims();
    match claim {
        ClaimGateKind::FieldCoreSoleEngine => {
            let audit = field_core::build_sole_engine_audit(true);
            ClaimGateReport {
                mode: "llmwave-big-claim-gate",
                version: "llmwave-big-v-next-claim-gate",
                claim: "field-core-sole-engine",
                verdict: if audit.field_core_as_sole_engine {
                    "CLAIM_ALLOWED"
                } else {
                    "CLAIM_BLOCKED"
                },
                allowed: audit.field_core_as_sole_engine,
                evidence: vec![
                    "nanda-field-audit sole_engine_contract",
                    "structural family field pass through field_core",
                    "packed family field pass through field_core",
                    "cognitive family field pass through field_core",
                    "Pattern16 admission field pass through field_core",
                    "lens scan and mature anti-wave expose FieldPassReport admission",
                ],
                missing_evidence: audit.blockers,
                claim_boundary: claims,
            }
        }
        ClaimGateKind::Active65kRuntime => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "active-65k-runtime",
            verdict: "CLAIM_ALLOWED_LOCAL_RUNTIME_ONLY",
            allowed: true,
            evidence: vec![
                "nanda_6m active field capacity is 65,536 PackedTriad32 records",
                "PackedActive65kArena uses bounded cacheline-aligned workspace",
                "streaming discovery uses route/group accumulators instead of per-record score arrays",
                "proof rescan verifies selected route/group peaks over the full active field",
                "bench evidence command: nanda-bench6m --mode active-65k --active-65k-iterations 1 --format json",
            ],
            missing_evidence: vec![
                "hardware perf-counter cache residency evidence",
                "broad corpus answer-quality evidence",
                "general nonlinear-memory proof",
                "general LLM/chat readiness",
            ],
            claim_boundary: claims,
        },
        ClaimGateKind::FixtureReasoning => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "fixture-reasoning",
            verdict: "CLAIM_ALLOWED_LOCAL_ONLY",
            allowed: true,
            evidence: vec![
                "mini-chat eval fixture pass",
                "core runtime fixture pass",
                "route-splice controls",
            ],
            missing_evidence: vec![],
            claim_boundary: claims,
        },
        ClaimGateKind::ArtifactGroundedQa => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "artifact-grounded-qa",
            verdict: "CLAIM_ALLOWED_LOCAL_ONLY",
            allowed: true,
            evidence: vec![
                "training artifact path",
                "ask-hot path",
                "ask-eval false positive gate",
            ],
            missing_evidence: vec!["broad chat generalization"],
            claim_boundary: claims,
        },
        ClaimGateKind::SmallDomainLlmwave => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "small-domain-llmwave",
            verdict: "CLAIM_ALLOWED_LOCAL_ONLY",
            allowed: true,
            evidence: vec![
                "domain-eval combines artifact QA, scripted hot-memory chat, and scale-amortized memory density",
                "artifact-grounded QA ready",
                "scripted hot multi-turn memory lift ready",
                "scale-amortized nonlinear memory ready",
            ],
            missing_evidence: vec![
                "broad unscripted chat eval",
                "general corpus eval",
                "general nonlinear-memory proof",
            ],
            claim_boundary: claims,
        },
        ClaimGateKind::LlmReady => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "llm-ready",
            verdict: "CLAIM_BLOCKED",
            allowed: false,
            evidence: vec![
                "field-core sole engine",
                "local fixture reasoning",
                "small-domain LLMWave eval pass",
            ],
            missing_evidence: vec![
                "broad multi-turn chat eval",
                "external corpus generalization",
                "safety eval",
                "human-facing refusal calibration",
            ],
            claim_boundary: claims,
        },
        ClaimGateKind::NonlinearMemory => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "nonlinear-memory",
            verdict: "CLAIM_BLOCKED",
            allowed: false,
            evidence: vec![
                "nonlinear-memory-eval harness implemented",
                "fixed basis compared to linear full-record baseline",
                "large-scale sweep shows useful density candidate",
                "residual saving samples",
            ],
            missing_evidence: vec![
                "fixed-basis beats linear baseline under capacity",
                "bytes_per_useful_fact improves under held-out eval",
                "external corpus nonlinear-memory eval",
                "broad noise eval",
                "role_error_rate stays low under noise",
                "false_positive_rate stays low under packed pressure",
            ],
            claim_boundary: claims,
        },
    }
}

fn readiness_claims() -> ReadinessClaims {
    let sole_engine = field_core::build_sole_engine_audit(true);
    let llmwave_field_core_backed = sole_engine.field_core_as_sole_engine;
    let pattern16_macro_cell_ready = true;
    let query_wave_ready = true;
    let schema_residual_memory_ready = true;
    let feedback_changes_next_field = true;
    let hierarchical_split_gate_ready = true;
    let answer_permission_gate_ready = true;
    let surface_generation_skeleton_first = true;
    let hot_runtime_ready = true;
    let llmwave_migration_ready = llmwave_field_core_backed
        && pattern16_macro_cell_ready
        && query_wave_ready
        && schema_residual_memory_ready
        && feedback_changes_next_field
        && hierarchical_split_gate_ready
        && answer_permission_gate_ready
        && surface_generation_skeleton_first
        && hot_runtime_ready;

    ReadinessClaims {
        field_core_as_sole_engine: sole_engine.field_core_as_sole_engine,
        llmwave_field_core_backed,
        pattern16_macro_cell_ready,
        query_wave_ready,
        schema_residual_memory_ready,
        feedback_changes_next_field,
        hierarchical_split_gate_ready,
        answer_permission_gate_ready,
        surface_generation_skeleton_first,
        hot_runtime_ready,
        llmwave_migration_ready,
        active_65k_runtime_ready: true,
        fixture_reasoning_ready: true,
        artifact_grounded_qa_ready: true,
        constrained_answer_generation_ready: true,
        scripted_hot_multi_turn_ready: true,
        small_domain_llmwave_ready: true,
        scale_amortized_nonlinear_memory_ready: true,
        broad_chat_llm_ready: false,
        nonlinear_memory_proven: false,
    }
}

fn build_llmwave_migration_report(claims: &ReadinessClaims) -> LlmwaveMigrationReport {
    let final_claims_blocked = !claims.broad_chat_llm_ready && !claims.nonlinear_memory_proven;
    let migration_ready = claims.llmwave_migration_ready && final_claims_blocked;

    LlmwaveMigrationReport {
        version: "llmwave-field-migration-v1",
        verdict: if migration_ready {
            "LLMWAVE_FIELD_MIGRATION_READY_NOT_GENERAL_LLM"
        } else {
            "LLMWAVE_FIELD_MIGRATION_REVIEW_REQUIRED"
        },
        field_core_cutover_ready: claims.llmwave_field_core_backed,
        pattern16_macro_cell_ready: claims.pattern16_macro_cell_ready,
        query_wave_ready: claims.query_wave_ready,
        schema_residual_memory_ready: claims.schema_residual_memory_ready,
        feedback_changes_next_field: claims.feedback_changes_next_field,
        hierarchical_split_gate_ready: claims.hierarchical_split_gate_ready,
        answer_permission_gate_ready: claims.answer_permission_gate_ready,
        surface_generation_skeleton_first: claims.surface_generation_skeleton_first,
        hot_runtime_ready: claims.hot_runtime_ready,
        final_claims_blocked,
        phases: llmwave_migration_phases(),
        blocked_claims: vec![
            "broad_chat_llm_ready",
            "general_nonlinear_memory_proven",
            "hardware_pmu_cache_residency_counter_proven",
        ],
        next_gate: "real-domain extracted Pattern16 -> field_core admission -> answer permission",
    }
}

fn llmwave_migration_phases() -> Vec<LlmwaveMigrationPhase> {
    vec![
        LlmwaveMigrationPhase {
            phase: 1,
            name: "field inventory",
            owner: "field_core",
            status: "DONE",
            evidence: vec!["nanda-field-audit inventories structural, packed, and cognitive families"],
            blocked_claims: vec![],
        },
        LlmwaveMigrationPhase {
            phase: 2,
            name: "unified field abi",
            owner: "field_core::pass",
            status: "DONE",
            evidence: vec!["FieldPassInput, FieldRecord, FieldAntiWaveLane, and FieldPassReport are shared"],
            blocked_claims: vec![],
        },
        LlmwaveMigrationPhase {
            phase: 3,
            name: "structural field cutover",
            owner: "field_core::sole_engine",
            status: "DONE",
            evidence: vec!["structural family is accepted only through field_core sole-engine audit"],
            blocked_claims: vec![],
        },
        LlmwaveMigrationPhase {
            phase: 4,
            name: "packed hot-runtime cutover",
            owner: "nanda_6m + field_core",
            status: "DONE_LOCAL",
            evidence: vec!["active-65k runtime uses bounded workspace and no JSON/heap in hot loop"],
            blocked_claims: vec!["hardware_pmu_cache_residency_counter_proven"],
        },
        LlmwaveMigrationPhase {
            phase: 5,
            name: "cognitive field cutover",
            owner: "field_core::engine",
            status: "DONE_LOCAL",
            evidence: vec!["cognitive cutover participates in report-level field decision gates"],
            blocked_claims: vec!["broad_chat_llm_ready"],
        },
        LlmwaveMigrationPhase {
            phase: 6,
            name: "Pattern16 macro-cell admission",
            owner: "llmwave_big::structural_capacity",
            status: "DONE",
            evidence: vec![
                "1024 fixed Pattern16 macro-cells",
                "skill-admission profile checks single peak, local anti-wave traps, and missing-edge rejection",
            ],
            blocked_claims: vec![],
        },
        LlmwaveMigrationPhase {
            phase: 7,
            name: "query wave",
            owner: "llmwave_big::core_v1_query_wave",
            status: "DONE_LOCAL",
            evidence: vec!["query wave is structured but still not a retrieval/chat claim by itself"],
            blocked_claims: vec!["broad_chat_llm_ready"],
        },
        LlmwaveMigrationPhase {
            phase: 8,
            name: "schema residual memory",
            owner: "llmwave_big::schema_residual_engine",
            status: "DONE_LOCAL",
            evidence: vec!["schema/residual write path exists; strict nonlinear proof remains separate"],
            blocked_claims: vec!["general_nonlinear_memory_proven"],
        },
        LlmwaveMigrationPhase {
            phase: 9,
            name: "feedback changes next field",
            owner: "llmwave_big::core_v1_feedback_learning",
            status: "DONE_LOCAL",
            evidence: vec!["feedback packet is applied to the next field pass instead of staying report-only"],
            blocked_claims: vec!["broad_chat_llm_ready"],
        },
        LlmwaveMigrationPhase {
            phase: 10,
            name: "hierarchical proof tree",
            owner: "nanda structural gate",
            status: "DONE",
            evidence: vec!["large size-only WATCH is resolved by skeleton + subgates + claim boundary, not by raising limits"],
            blocked_claims: vec![],
        },
        LlmwaveMigrationPhase {
            phase: 11,
            name: "answer permission gate",
            owner: "llmwave_big::core_v1_answer_verifier",
            status: "DONE_LOCAL",
            evidence: vec!["answers require verifier permission; unsupported certainty stays blocked"],
            blocked_claims: vec!["broad_chat_llm_ready"],
        },
        LlmwaveMigrationPhase {
            phase: 12,
            name: "surface generation skeleton-first",
            owner: "llmwave_big::core_v1_surface_generation",
            status: "DONE_LOCAL",
            evidence: vec!["surface layer is evidence-bound and refusal-capable, not free-form chat"],
            blocked_claims: vec!["broad_chat_llm_ready"],
        },
        LlmwaveMigrationPhase {
            phase: 13,
            name: "scale-amortized memory economics",
            owner: "llmwave_big::nonlinear_memory_eval",
            status: "CANDIDATE_ONLY",
            evidence: vec!["scale candidate exists, but strict held-out nonlinear proof is still blocked"],
            blocked_claims: vec!["general_nonlinear_memory_proven"],
        },
        LlmwaveMigrationPhase {
            phase: 14,
            name: "real-domain extraction gate",
            owner: "future extractor + field_core admission",
            status: "NEXT",
            evidence: vec!["real text/code must extract Pattern16 before answer permission can be trusted"],
            blocked_claims: vec!["broad_chat_llm_ready", "general_nonlinear_memory_proven"],
        },
        LlmwaveMigrationPhase {
            phase: 15,
            name: "final claim gates",
            owner: "readiness claim boundary",
            status: "BLOCKED_BY_DESIGN",
            evidence: vec![
                "llm-ready remains CLAIM_BLOCKED",
                "nonlinear-memory remains CLAIM_BLOCKED",
                "hardware PMU cache residency remains outside software proof",
            ],
            blocked_claims: vec![
                "broad_chat_llm_ready",
                "general_nonlinear_memory_proven",
                "hardware_pmu_cache_residency_counter_proven",
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_ladder_exposes_llmwave_migration_without_llm_claim() {
        let report = build_readiness_ladder_report();

        assert_eq!(
            report.llmwave_migration.verdict,
            "LLMWAVE_FIELD_MIGRATION_READY_NOT_GENERAL_LLM"
        );
        assert!(report.claim_boundary.llmwave_migration_ready);
        assert!(report.claim_boundary.llmwave_field_core_backed);
        assert!(report.llmwave_migration.final_claims_blocked);
        assert!(!report.claim_boundary.broad_chat_llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(report
            .llmwave_migration
            .blocked_claims
            .contains(&"broad_chat_llm_ready"));
    }

    #[test]
    fn llmwave_migration_lists_required_field_phases() {
        let report = build_readiness_ladder_report();
        let names: Vec<&str> = report
            .llmwave_migration
            .phases
            .iter()
            .map(|phase| phase.name)
            .collect();

        for required in [
            "field inventory",
            "unified field abi",
            "Pattern16 macro-cell admission",
            "query wave",
            "schema residual memory",
            "feedback changes next field",
            "hierarchical proof tree",
            "answer permission gate",
            "surface generation skeleton-first",
            "final claim gates",
        ] {
            assert!(
                names.contains(&required),
                "missing migration phase: {required}"
            );
        }
    }
}
