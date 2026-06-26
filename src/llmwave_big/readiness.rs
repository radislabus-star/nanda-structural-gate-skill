use serde::Serialize;

use crate::field_core;

#[derive(Serialize, Clone)]
pub(crate) struct ReadinessLadderReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub current_level: u8,
    pub current_state: &'static str,
    pub levels: Vec<ReadinessLevel>,
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
    ReadinessClaims {
        field_core_as_sole_engine: sole_engine.field_core_as_sole_engine,
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
