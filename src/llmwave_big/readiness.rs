use serde::Serialize;

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
    pub fixture_reasoning_ready: bool,
    pub artifact_grounded_qa_ready: bool,
    pub constrained_answer_generation_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum ClaimGateKind {
    #[value(name = "field-core-sole-engine")]
    FieldCoreSoleEngine,
    #[value(name = "fixture-reasoning")]
    FixtureReasoning,
    #[value(name = "artifact-grounded-qa")]
    ArtifactGroundedQa,
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
                state: "WATCH",
                evidence: vec![
                    "hot feedback memory exists",
                    "broad multi-turn eval missing",
                ],
            },
            ReadinessLevel {
                level: 5,
                name: "small domain LLMWave ready",
                state: "WATCH",
                evidence: vec![
                    "domain artifact path exists",
                    "domain corpus eval threshold missing",
                ],
            },
            ReadinessLevel {
                level: 6,
                name: "broad chat candidate",
                state: "BLOCKED",
                evidence: vec!["external broad chat eval missing", "safety eval missing"],
            },
            ReadinessLevel {
                level: 7,
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
        ClaimGateKind::FieldCoreSoleEngine => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "field-core-sole-engine",
            verdict: "CLAIM_ALLOWED",
            allowed: true,
            evidence: vec![
                "structural family sole engine",
                "packed family sole engine",
                "cognitive family sole engine",
            ],
            missing_evidence: vec![],
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
        ClaimGateKind::LlmReady => ClaimGateReport {
            mode: "llmwave-big-claim-gate",
            version: "llmwave-big-v-next-claim-gate",
            claim: "llm-ready",
            verdict: "CLAIM_BLOCKED",
            allowed: false,
            evidence: vec!["field-core sole engine", "local fixture reasoning"],
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
    ReadinessClaims {
        field_core_as_sole_engine: true,
        fixture_reasoning_ready: true,
        artifact_grounded_qa_ready: true,
        constrained_answer_generation_ready: true,
        broad_chat_llm_ready: false,
        nonlinear_memory_proven: false,
    }
}
