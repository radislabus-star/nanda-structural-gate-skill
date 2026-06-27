use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::{fs, path::PathBuf};

use super::{nanda_6m, OutputFormat, CORE_VERSION, EXIT_PASS, EXIT_WATCH, WAVE_DIM};

pub mod active_core;
pub mod answer_surface;
pub mod atlas;
pub mod broad_eval;
pub mod consolidation;
pub mod core_v1_active_retrieval;
pub mod core_v1_answer_verifier;
pub mod core_v1_broad_eval_harness;
pub mod core_v1_consolidation_sleep;
pub mod core_v1_contract;
pub mod core_v1_feedback_learning;
pub mod core_v1_field_cutover;
pub mod core_v1_memory_writer;
pub mod core_v1_nonlinear_proof;
pub mod core_v1_query_wave;
pub mod core_v1_schema_reasoning;
pub mod core_v1_surface_generation;
pub mod core_v2;
pub mod core_v3;
pub mod coupled_decode_loop;
pub mod daybreak_duel;
pub mod demo_domain;
pub mod density_ablation;
pub mod density_proof_doctor;
pub mod dialogue_state;
pub mod domain_eval;
pub mod eval;
pub mod evidence_proof;
pub mod field_feedback;
pub mod field_runtime;
pub mod hrr_binding;
pub mod l2_l3_coupling;
pub mod l2_word_field;
pub mod l3_schema_bind;
pub mod l3_schema_field;
pub mod lens_scan;
pub mod lexical_birth;
pub mod linux_active_field;
pub mod linux_atlas;
pub mod linux_atlas_projection;
pub mod linux_center_learning;
pub mod linux_chat;
pub mod linux_chat_core;
pub mod linux_chat_v1;
pub mod linux_chat_v2;
pub mod linux_exposure;
pub mod linux_hot_packet;
pub mod linux_profile;
pub mod linux_residual_memory;
pub mod linux_runtime_snapshot;
pub mod linux_spectral_memory;
pub mod linux_vpn_control;
pub mod linux_vpn_training;
pub mod loader;
pub mod mature_anti_wave;
pub mod memory_final_proof;
pub mod memory_physics;
pub mod memory_proof_path;
pub mod mini_chat_eval;
pub mod multi_peak_field;
pub mod multi_profile_density_suite;
pub mod multi_schema_competition;
pub mod nonlinear_memory_eval;
pub mod open_surface_generation;
pub mod operators;
pub mod persistent_wave_memory;
pub mod profile_density_build;
pub mod query_wave;
pub mod readiness;
pub mod reasoning_field;
pub mod residuals;
pub mod rust_compile_evidence;
pub mod rust_corpus;
pub mod rust_focus;
pub mod rust_heldout;
pub mod rust_heldout_eval;
pub mod schema_memory_growth;
pub mod schemas;
pub mod security_fixture;
pub mod strict_density_claim_gate;
pub mod structural_capacity;
pub mod surface_bank_build;
pub mod surface_bank_fixture;
pub mod surface_bank_validate;
pub mod surface_corpus_eval;
pub mod surface_production;
pub mod surface_raw_induce;
pub mod surface_reconstruct;
pub mod symbols;
pub mod training;
pub mod write;

mod claims;
mod contract;
mod metrics;
mod report;

const LLMWAVE_BIG_VERSION: &str = "llmwave-big-v160-contract-boundary";

#[derive(Parser)]
pub(super) struct LlmwaveBigArgs {
    #[command(subcommand)]
    command: LlmwaveBigCommand,
}

#[derive(Subcommand)]
enum LlmwaveBigCommand {
    /// Print the Phase 1 LLMWave Core V1 execution contract.
    #[command(name = "core-v1-contract", alias = "core-contract")]
    CoreV1Contract(LlmwaveBigCoreV1ContractArgs),
    /// Print the Phase 2 LLMWave Core V1 field-core cutover report.
    #[command(name = "core-v1-field-cutover", alias = "core-field-cutover")]
    CoreV1FieldCutover(LlmwaveBigCoreV1FieldCutoverArgs),
    /// Print the Phase 3 LLMWave Core V1 memory writer report.
    #[command(name = "core-v1-memory-writer", alias = "core-memory-writer")]
    CoreV1MemoryWriter(LlmwaveBigCoreV1MemoryWriterArgs),
    /// Print the Phase 4 LLMWave Core V1 nonlinear memory proof gate.
    #[command(name = "core-v1-nonlinear-proof", alias = "core-nonlinear-proof")]
    CoreV1NonlinearProof(LlmwaveBigCoreV1NonlinearProofArgs),
    /// Print the Phase 5 LLMWave Core V1 query wave input gate.
    #[command(name = "core-v1-query-wave", alias = "core-query-wave")]
    CoreV1QueryWave(LlmwaveBigCoreV1QueryWaveArgs),
    /// Print the Phase 6 LLMWave Core V1 active field retrieval gate.
    #[command(name = "core-v1-active-retrieval", alias = "core-active-retrieval")]
    CoreV1ActiveRetrieval(LlmwaveBigCoreV1ActiveRetrievalArgs),
    /// Print the Phase 7 LLMWave Core V1 schema reasoning gate.
    #[command(name = "core-v1-schema-reasoning", alias = "core-schema-reasoning")]
    CoreV1SchemaReasoning(LlmwaveBigCoreV1SchemaReasoningArgs),
    /// Print the Phase 8 LLMWave Core V1 surface generation gate.
    #[command(name = "core-v1-surface-generation", alias = "core-surface-generation")]
    CoreV1SurfaceGeneration(LlmwaveBigCoreV1SurfaceGenerationArgs),
    /// Print the Phase 9 LLMWave Core V1 answer verifier gate.
    #[command(name = "core-v1-answer-verifier", alias = "core-answer-verifier")]
    CoreV1AnswerVerifier(LlmwaveBigCoreV1AnswerVerifierArgs),
    /// Print the Phase 10 LLMWave Core V1 feedback learning gate.
    #[command(name = "core-v1-feedback-learning", alias = "core-feedback-learning")]
    CoreV1FeedbackLearning(LlmwaveBigCoreV1FeedbackLearningArgs),
    /// Print the Phase 11 LLMWave Core V1 consolidation sleep-pass gate.
    #[command(
        name = "core-v1-consolidation-sleep",
        alias = "core-consolidation-sleep"
    )]
    CoreV1ConsolidationSleep(LlmwaveBigCoreV1ConsolidationSleepArgs),
    /// Print the Phase 12 LLMWave Core V1 broad eval harness gate.
    #[command(name = "core-v1-broad-eval", alias = "core-broad-eval")]
    CoreV1BroadEvalHarness(LlmwaveBigCoreV1BroadEvalHarnessArgs),
    /// Print the LLMWave Core V2 staged pipeline contract.
    #[command(name = "core-v2-contract")]
    CoreV2Contract(LlmwaveBigCoreV2ContractArgs),
    /// Print the LLMWave Core V2 public-safe corpus artifact gate.
    #[command(name = "core-v2-corpus")]
    CoreV2Corpus(LlmwaveBigCoreV2CorpusArgs),
    /// Print the LLMWave Core V2 held-out suite gate.
    #[command(name = "core-v2-heldout")]
    CoreV2Heldout(LlmwaveBigCoreV2HeldoutArgs),
    /// Print the LLMWave Core V2 route-balanced focus gate.
    #[command(name = "core-v2-focus")]
    CoreV2Focus(LlmwaveBigCoreV2FocusArgs),
    /// Print the LLMWave Core V2 density proof gate.
    #[command(name = "core-v2-density")]
    CoreV2Density(LlmwaveBigCoreV2DensityArgs),
    /// Run the LLMWave Core V2 local route field.
    #[command(name = "core-v2-run")]
    CoreV2Run(LlmwaveBigCoreV2RunArgs),
    /// Print the LLMWave Core V2 hot packet storage gate.
    #[command(name = "core-v2-pack-hot")]
    CoreV2PackHot(LlmwaveBigCoreV2PackHotArgs),
    /// Print the LLMWave Core V2 hard claim gate.
    #[command(name = "core-v2-claim-gate")]
    CoreV2ClaimGate(LlmwaveBigCoreV2ClaimGateArgs),
    /// Print the LLMWave Core V3 Goal/Action/Constraint/Solution plan.
    #[command(name = "core-v3-plan")]
    CoreV3Plan(LlmwaveBigCoreV3PlanArgs),
    /// Run the LLMWave Core V3 solution-search field.
    #[command(name = "core-v3-solution-search")]
    CoreV3SolutionSearch(LlmwaveBigCoreV3SolutionSearchArgs),
    /// Check the LLMWave Core V3 1M active projection against the 6 MiB budget.
    #[command(name = "core-v3-pack-1m")]
    CoreV3Pack1m(LlmwaveBigCoreV3Pack1mArgs),
    /// Print the LLMWave Core V3 hard claim gate.
    #[command(name = "core-v3-claim-gate")]
    CoreV3ClaimGate(LlmwaveBigCoreV3ClaimGateArgs),
    /// Print the v158-v160 Big Model Contract and claim boundary.
    Contract(LlmwaveBigContractArgs),
    /// Print the v161-v170 Wave Atlas file/index/loader contract.
    Atlas(LlmwaveBigAtlasArgs),
    /// Print the v171-v180 hot Active Core contract and sample cycle.
    ActiveCore(LlmwaveBigActiveCoreArgs),
    /// Print the v181-v190 L2 Word Field contract and surface sample.
    L2(LlmwaveBigL2Args),
    /// Run the v391-v430 HRR/VSA role-filler binding core.
    Hrr(LlmwaveBigHrrArgs),
    /// Run the v431-v455 L3 schema binding core.
    SchemaBind(LlmwaveBigSchemaBindArgs),
    /// Run the v456-v480 L2/L3 coupling core.
    #[command(name = "l2-l3-couple", alias = "l2l3-couple")]
    L2L3Couple(LlmwaveBigL2L3CoupleArgs),
    /// Run the v481-v520 recurrent L2/L3 decode loop.
    DecodeLoop(LlmwaveBigDecodeLoopArgs),
    /// Run the v521-v560 multi-schema competition core.
    MultiSchema(LlmwaveBigMultiSchemaArgs),
    /// Run the v561-v620 schema memory growth core.
    SchemaGrow(LlmwaveBigSchemaGrowArgs),
    /// Run the v621-v700 open surface generation core.
    SurfaceGenerate(LlmwaveBigSurfaceGenerateArgs),
    /// Run the v701-v780 multi-step reasoning field.
    ReasonField(LlmwaveBigReasonFieldArgs),
    /// Run the v781-v860 dialogue state core.
    DialogueState(LlmwaveBigDialogueStateArgs),
    /// Run the v861-v950 mini chat eval boundary.
    #[command(name = "mini-chat-eval", alias = "chat-eval")]
    MiniChatEval(LlmwaveBigMiniChatEvalArgs),
    /// Run the v951-v1000 query wave core.
    #[command(name = "query-wave")]
    QueryWave(LlmwaveBigQueryWaveArgs),
    /// Run the v1001-v1060 multi-peak field core.
    #[command(name = "multi-peak-field", alias = "field-peaks")]
    MultiPeakField(LlmwaveBigMultiPeakFieldArgs),
    /// Run the v1061-v1140 field lens scan.
    #[command(name = "lens-scan")]
    LensScan(LlmwaveBigLensScanArgs),
    /// Run the v1141-v1210 mature anti-wave layer.
    #[command(name = "mature-anti-wave", alias = "anti-wave")]
    MatureAntiWave(LlmwaveBigMatureAntiWaveArgs),
    /// Run the v1211-v1280 evidence proof gate.
    #[command(name = "evidence-proof")]
    EvidenceProof(LlmwaveBigEvidenceProofArgs),
    /// Run the v1281-v1350 constrained answer surface.
    #[command(name = "answer-surface")]
    AnswerSurface(LlmwaveBigAnswerSurfaceArgs),
    /// Run the v1351-v1420 local field feedback layer.
    #[command(name = "field-feedback")]
    FieldFeedback(LlmwaveBigFieldFeedbackArgs),
    /// Run the v1421-v1480 applied feedback memory packet.
    #[command(name = "feedback-memory")]
    FeedbackMemory(LlmwaveBigFeedbackMemoryArgs),
    /// Run the v1481-v1540 feedback-aware field pass.
    #[command(name = "feedback-aware-field")]
    FeedbackAwareField(LlmwaveBigFeedbackAwareFieldArgs),
    /// Run the v1541-v1600 applied anti-memory check.
    #[command(name = "applied-anti-memory")]
    AppliedAntiMemory(LlmwaveBigAppliedAntiMemoryArgs),
    /// Run the v1601-v1660 persistent memory store.
    #[command(name = "memory-store")]
    MemoryStore(LlmwaveBigMemoryStoreArgs),
    /// Run the v1661-v1720 before/after learning eval.
    #[command(name = "learning-eval")]
    LearningEval(LlmwaveBigLearningEvalArgs),
    /// Run the v1721-v1780 memory consolidation eval.
    #[command(name = "memory-consolidate")]
    MemoryConsolidate(LlmwaveBigMemoryConsolidateArgs),
    /// Run the v1781-v1840 full runtime pipeline.
    #[command(name = "run")]
    Run(LlmwaveBigRunArgs),
    /// Run the v1841-v1900 core readiness gate.
    #[command(name = "core-eval")]
    CoreEval(LlmwaveBigCoreEvalArgs),
    /// Run the fixed 1024 Pattern16 structural-capacity core gate.
    #[command(name = "structural-capacity", alias = "capacity-1024")]
    StructuralCapacity(LlmwaveBigStructuralCapacityArgs),
    /// Show the LLMWave readiness ladder without claiming broad LLM readiness.
    #[command(name = "readiness-ladder", alias = "readiness")]
    ReadinessLadder(LlmwaveBigReadinessLadderArgs),
    /// Gate a public claim such as llm-ready or nonlinear-memory.
    #[command(name = "claim-gate")]
    ClaimGate(LlmwaveBigClaimGateArgs),
    /// Compare fixed-basis residual memory against a linear fact baseline.
    #[command(name = "nonlinear-memory-eval", alias = "density-proof")]
    NonlinearMemoryEval(LlmwaveBigNonlinearMemoryEvalArgs),
    /// Build the Phase 1 nonlinear-memory density ladder.
    #[command(name = "nonlinear-memory-ladder", alias = "density-ladder")]
    NonlinearMemoryLadder(LlmwaveBigNonlinearMemoryLadderArgs),
    /// Print the v246-v252 literature-grounded lexical birth mechanism.
    WordBirth(LlmwaveBigWordBirthArgs),
    /// Print the v253-v260 surface production memory contract.
    SurfaceProduction(LlmwaveBigSurfaceProductionArgs),
    /// Run the v261-v270 cold surface reconstruction eval.
    SurfaceReconstruct(LlmwaveBigSurfaceReconstructArgs),
    /// Run the v271-v280 corpus surface-density candidate eval.
    SurfaceCorpusEval(LlmwaveBigSurfaceCorpusEvalArgs),
    /// Build the v281-v290 observed surface-family bank.
    SurfaceBankBuild(LlmwaveBigSurfaceBankBuildArgs),
    /// Validate the v291-v300 surface bank with negative controls.
    SurfaceBankValidate(LlmwaveBigSurfaceBankValidateArgs),
    /// Load and validate the v301-v310 external surface corpus fixture.
    SurfaceBankFixture(LlmwaveBigSurfaceBankFixtureArgs),
    /// Induce v311-v320 surface families from raw forms.
    SurfaceRawInduce(LlmwaveBigSurfaceRawInduceArgs),
    /// Print the v191-v205 schema/residual write contract.
    Write(LlmwaveBigWriteArgs),
    /// Run the Phase 2-3 schema reuse and residual-only write engine.
    #[command(name = "schema-residual-engine", alias = "residual-write-engine")]
    SchemaResidualEngine(LlmwaveBigSchemaResidualEngineArgs),
    /// Run the Phase 4-5 collision/noise and anti-wave memory physics.
    #[command(name = "memory-physics", alias = "anti-wave-memory")]
    MemoryPhysics(LlmwaveBigMemoryPhysicsArgs),
    /// Run the Phase 6-8 held-out inference, basis economics, and atlas bridge.
    #[command(name = "memory-proof-path", alias = "heldout-atlas")]
    MemoryProofPath(LlmwaveBigMemoryProofPathArgs),
    /// Run the Phase 9-12 field recall, LLMWave bridge, big-corpus, and final proof gate.
    #[command(name = "memory-final-proof", alias = "final-proof")]
    MemoryFinalProof(LlmwaveBigMemoryFinalProofArgs),
    /// Print the v206-v218 consolidation/sleep contract.
    Consolidate(LlmwaveBigConsolidateArgs),
    /// Print the v219-v230 Big Cognition Eval report.
    Eval(LlmwaveBigEvalArgs),
    /// Run the v231-v245 runtime product query surface.
    Query(LlmwaveBigQueryArgs),
    /// Compile a real corpus into LLMWave-Big training records.
    Train(LlmwaveBigTrainArgs),
    /// Build a Rust-oriented structural corpus artifact for proof gates.
    #[command(name = "rust-corpus-build", alias = "rust-corpus")]
    RustCorpusBuild(LlmwaveBigRustCorpusBuildArgs),
    /// Build Rust held-out route questions from a Rust structural corpus artifact.
    #[command(name = "rust-heldout-build", alias = "rust-heldout")]
    RustHeldoutBuild(LlmwaveBigRustHeldoutBuildArgs),
    /// Build a route-balanced Rust focus packet from corpus and held-out artifacts.
    #[command(name = "rust-focus-build", alias = "rust-focus")]
    RustFocusBuild(LlmwaveBigRustFocusBuildArgs),
    /// Build Rust compile/test evidence linked to a focus packet.
    #[command(name = "rust-compile-evidence-build", alias = "rust-compile-evidence")]
    RustCompileEvidenceBuild(LlmwaveBigRustCompileEvidenceBuildArgs),
    /// Evaluate held-out Rust route-fact inference over a focus packet.
    #[command(name = "rust-heldout-eval", alias = "rust-inference-eval")]
    RustHeldoutEval(LlmwaveBigRustHeldoutEvalArgs),
    /// Check strict Rust profile density evidence before nonlinear claims.
    #[command(name = "strict-density-claim-gate", alias = "density-claim")]
    StrictDensityClaimGate(LlmwaveBigStrictDensityClaimGateArgs),
    /// Build a generic non-Rust density profile artifact from a relation corpus.
    #[command(name = "profile-density-build", alias = "profile-density")]
    ProfileDensityBuild(LlmwaveBigProfileDensityBuildArgs),
    /// Aggregate independent density profiles before general nonlinear claims.
    #[command(name = "multi-profile-density-suite", alias = "density-suite")]
    MultiProfileDensitySuite(LlmwaveBigMultiProfileDensitySuiteArgs),
    /// Diagnose whether multi-profile density evidence is strong or only formal.
    #[command(name = "density-proof-doctor", alias = "density-doctor")]
    DensityProofDoctor(LlmwaveBigDensityProofDoctorArgs),
    /// Run suite-level profile ablation and baseline-duel hooks.
    #[command(name = "density-ablation")]
    DensityAblation(LlmwaveBigDensityAblationArgs),
    /// Build a broad cognition eval corpus artifact.
    #[command(name = "broad-corpus-build")]
    BroadCorpusBuild(LlmwaveBigBroadCorpusBuildArgs),
    /// Build a broad cognition eval suite.
    #[command(name = "broad-eval-suite-build")]
    BroadEvalSuiteBuild(LlmwaveBigBroadEvalSuiteBuildArgs),
    /// Diagnose whether a broad corpus is balanced enough for external proof.
    #[command(name = "broad-dataset-doctor")]
    BroadDatasetDoctor(LlmwaveBigBroadDatasetDoctorArgs),
    /// Generate held-out broad cognition cases from a broad corpus.
    #[command(name = "broad-heldout-build")]
    BroadHeldoutBuild(LlmwaveBigBroadHeldoutBuildArgs),
    /// Build a route-balanced broad focus packet with held-out facts removed.
    #[command(name = "broad-focus-build")]
    BroadFocusBuild(LlmwaveBigBroadFocusBuildArgs),
    /// Run broad cognition eval over route, generation, context, feedback, and shortcuts.
    #[command(name = "broad-eval-run")]
    BroadEvalRun(LlmwaveBigBroadEvalRunArgs),
    /// Compare broad cognition eval against lexical/flat/route/markov baselines.
    #[command(name = "broad-baseline-duel")]
    BroadBaselineDuel(LlmwaveBigBroadBaselineDuelArgs),
    /// Evaluate constrained multi-turn context, correction, and shortcut refusal.
    #[command(name = "broad-chat-loop-eval")]
    BroadChatLoopEval(LlmwaveBigBroadChatLoopEvalArgs),
    /// Combine memory proof, broad eval, and baseline duel into readiness boundary.
    #[command(name = "llmwave-readiness")]
    LlmwaveReadiness(LlmwaveBigLlmwaveReadinessArgs),
    /// Ask a compiled LLMWave-Big training artifact.
    Ask(LlmwaveBigAskArgs),
    /// Evaluate ask behavior over a compiled training artifact.
    #[command(name = "ask-eval")]
    AskEval(LlmwaveBigAskEvalArgs),
    /// Pack a training artifact into a binary hot Active Core file.
    #[command(name = "pack-hot")]
    PackHot(LlmwaveBigPackHotArgs),
    /// Ask through a binary hot Active Core pack, with cold labels for display.
    #[command(name = "ask-hot")]
    AskHot(LlmwaveBigAskHotArgs),
    /// Convert batch feedback into persistent hot-memory overlay records.
    #[command(name = "learn-hot")]
    LearnHot(LlmwaveBigLearnHotArgs),
    /// Interactive/scripted shell over ask-hot plus learn-hot memory.
    #[command(name = "chat-hot")]
    ChatHot(LlmwaveBigChatHotArgs),
    /// Evaluate scripted hot chat learning and answer lift.
    #[command(name = "chat-hot-eval")]
    ChatHotEval(LlmwaveBigChatHotEvalArgs),
    /// Evaluate the small-domain LLMWave path end-to-end.
    #[command(name = "domain-eval")]
    DomainEval(LlmwaveBigDomainEvalArgs),
    /// Build and evaluate the bundled one-command small-domain demo.
    #[command(name = "demo-domain")]
    DemoDomain(LlmwaveBigDemoDomainArgs),
    /// Build an append-only local Linux knowledge atlas from system metadata.
    #[command(name = "linux-atlas-build", alias = "linux-atlas")]
    LinuxAtlasBuild(LlmwaveBigLinuxAtlasBuildArgs),
    /// Build a route-balanced Linux Active Field over the append-only Linux Atlas.
    #[command(name = "linux-active-field", alias = "linux-field")]
    LinuxActiveField(LlmwaveBigLinuxActiveFieldArgs),
    /// Pack the Linux Active Field into a binary fixed-record hot packet.
    #[command(name = "linux-pack-hot", alias = "linux-hot-pack")]
    LinuxPackHot(LlmwaveBigLinuxPackHotArgs),
    /// Scan a binary Linux hot packet without JSON.
    #[command(name = "linux-ask-hot", alias = "linux-hot-scan")]
    LinuxAskHot(LlmwaveBigLinuxAskHotArgs),
    /// Run Linux hot-packet domain eval and lexical baseline duel.
    #[command(name = "linux-hot-eval")]
    LinuxHotEval(LlmwaveBigLinuxHotEvalArgs),
    /// Run the constrained Linux Domain LLMWave path over a hot packet.
    #[command(name = "linux-domain-run")]
    LinuxDomainRun(LlmwaveBigLinuxDomainRunArgs),
    /// Gate the Linux Atlas -> 6 MiB cognitive projection.
    #[command(name = "linux-atlas-projection", alias = "phase18-atlas-projection")]
    LinuxAtlasProjection(LlmwaveBigLinuxAtlasProjectionArgs),
    /// Prove the Linux hot loop scans only fixed records inside the 6 MiB cache budget.
    #[command(name = "linux-cache-proof", alias = "linux-cache-bench")]
    LinuxCacheProof(LlmwaveBigLinuxCacheProofArgs),
    /// Measure Linux hot-loop cache residency with hardware PMU counters.
    #[command(name = "linux-pmu-cache-proof", alias = "linux-hardware-cache-proof")]
    LinuxPmuCacheProof(LlmwaveBigLinuxPmuCacheProofArgs),
    /// Pack the Linux Active Field into binary schema/residual memory.
    #[command(name = "linux-pack-residual", alias = "linux-residual-pack")]
    LinuxPackResidual(LlmwaveBigLinuxPackResidualArgs),
    /// Prove Linux schema/residual binary memory over the residual packet.
    #[command(name = "linux-residual-proof", alias = "linux-nonlinear-proof")]
    LinuxResidualProof(LlmwaveBigLinuxResidualProofArgs),
    /// Run boundary-aware Linux exposure reasoning over the residual packet.
    #[command(name = "linux-exposure-run", alias = "linux-exposure")]
    LinuxExposureRun(LlmwaveBigLinuxExposureRunArgs),
    /// Import a side-effect-free Linux runtime snapshot as temporary profile facts.
    #[command(name = "linux-snapshot-import")]
    LinuxSnapshotImport(LlmwaveBigLinuxSnapshotImportArgs),
    /// Run a safe Daybreak-style defensive benchmark over the Linux-profile core.
    #[command(name = "daybreak-duel")]
    DaybreakDuel(LlmwaveBigDaybreakDuelArgs),
    /// Run a safe local find -> patch -> verify security fixture.
    #[command(name = "security-fixture-run")]
    SecurityFixtureRun(LlmwaveBigSecurityFixtureRunArgs),
    /// Run constrained Linux-profile chat/readout over schema/residual memory.
    #[command(name = "linux-chat-run", alias = "linux-chat")]
    LinuxChatRun(LlmwaveBigLinuxChatRunArgs),
    /// Run Linux Chat V1: bounded multi-turn chat over Linux-profile memory.
    #[command(name = "linux-chat-v1", alias = "linux-chat-loop")]
    LinuxChatV1(LlmwaveBigLinuxChatV1Args),
    /// Run the built-in Linux Chat V1 eval script.
    #[command(name = "linux-chat-v1-eval")]
    LinuxChatV1Eval(LlmwaveBigLinuxChatV1EvalArgs),
    /// Run Linux Chat V2: persistent wave-memory learning from dialogue feedback.
    #[command(name = "linux-chat-v2")]
    LinuxChatV2(LlmwaveBigLinuxChatV2Args),
    /// Run the built-in Linux Chat V2 persistent learning eval script.
    #[command(name = "linux-chat-v2-eval")]
    LinuxChatV2Eval(LlmwaveBigLinuxChatV2EvalArgs),
    /// Run dynamic center learning over Linux `.lrf` schema/residual memory.
    #[command(name = "linux-center-learn")]
    LinuxCenterLearn(LlmwaveBigLinuxCenterLearnArgs),
    /// Train safe local VPN routes into persistent Linux-profile wave memory.
    #[command(name = "linux-vpn-train")]
    LinuxVpnTrain(LlmwaveBigLinuxVpnTrainArgs),
    /// Train and evaluate local VPN routes through Linux Chat V2.
    #[command(name = "linux-vpn-train-eval")]
    LinuxVpnTrainEval(LlmwaveBigLinuxVpnTrainEvalArgs),
    /// Compile a guarded local VPN enable/disable/status command plan.
    #[command(name = "linux-vpn-action-plan")]
    LinuxVpnActionPlan(LlmwaveBigLinuxVpnActionPlanArgs),
    /// Dry-run or explicitly execute a guarded local VPN control command.
    #[command(name = "linux-vpn-control")]
    LinuxVpnControl(LlmwaveBigLinuxVpnControlArgs),
    /// Compile a Linux question into a typed Linux-profile query wave.
    #[command(name = "linux-query-wave")]
    LinuxQueryWave(LlmwaveBigLinuxQueryWaveArgs),
    /// Build a Linux-profile broad eval suite from residual memory.
    #[command(name = "linux-broad-suite-build")]
    LinuxBroadSuiteBuild(LlmwaveBigLinuxBroadSuiteBuildArgs),
    /// Run Linux-profile broad eval over residual memory.
    #[command(name = "linux-broad-eval-run")]
    LinuxBroadEvalRun(LlmwaveBigLinuxBroadEvalRunArgs),
    /// Run Linux-profile evidence-chain reasoning for one question.
    #[command(name = "linux-reason-run")]
    LinuxReasonRun(LlmwaveBigLinuxReasonRunArgs),
    /// Gate Linux-profile reasoning/chat claims from memory and eval evidence.
    #[command(name = "linux-profile-claim-gate", alias = "linux-chat-profile-gate")]
    LinuxProfileClaimGate(LlmwaveBigLinuxProfileClaimGateArgs),
    /// Compile `.lrf` + `.lwm` overlays into a unified Linux ChatCore cache.
    #[command(name = "linux-chat-core-build")]
    LinuxChatCoreBuild(LlmwaveBigLinuxChatCoreBuildArgs),
    /// Fast authority gate for the unified Linux ChatCore cache.
    #[command(name = "linux-chat-core-gate")]
    LinuxChatCoreGate(LlmwaveBigLinuxChatCoreGateArgs),
    /// Heavy profile/eval gate for the unified Linux ChatCore cache.
    #[command(name = "linux-chat-core-profile-gate")]
    LinuxChatCoreProfileGate(LlmwaveBigLinuxChatCoreGateArgs),
    /// Return a compact grounded packet through the unified Linux ChatCore cache.
    #[command(name = "linux-chat-core-ask")]
    LinuxChatCoreAsk(LlmwaveBigLinuxChatCoreAskArgs),
    /// Write ChatCore feedback into a source `.lwm` overlay and mark cache stale.
    #[command(name = "linux-chat-core-learn")]
    LinuxChatCoreLearn(LlmwaveBigLinuxChatCoreLearnArgs),
    /// Prove the ChatCore overlay -> stale -> rebuild -> improved answer loop.
    #[command(name = "linux-chat-core-learn-eval")]
    LinuxChatCoreLearnEval(LlmwaveBigLinuxChatCoreLearnEvalArgs),
    /// Build a Linux-profile held-out eval suite with near-collision controls.
    #[command(name = "linux-heldout-suite-build")]
    LinuxHeldoutSuiteBuild(LlmwaveBigLinuxHeldoutSuiteBuildArgs),
    /// Run the Linux-profile held-out eval suite.
    #[command(name = "linux-heldout-eval-run")]
    LinuxHeldoutEvalRun(LlmwaveBigLinuxHeldoutEvalRunArgs),
    /// Build a Linux-profile local feedback memory packet.
    #[command(name = "linux-feedback-build")]
    LinuxFeedbackBuild(LlmwaveBigLinuxFeedbackBuildArgs),
    /// Apply a Linux-profile feedback memory packet to a new field pass.
    #[command(name = "linux-feedback-apply")]
    LinuxFeedbackApply(LlmwaveBigLinuxFeedbackApplyArgs),
    /// Search for missing evidence/actions needed to make a Linux decision.
    #[command(name = "linux-decision-search")]
    LinuxDecisionSearch(LlmwaveBigLinuxDecisionSearchArgs),
    /// Report Linux relation-family coverage over a residual packet.
    #[command(name = "linux-relation-profile")]
    LinuxRelationProfile(LlmwaveBigLinuxRelationProfileArgs),
}

#[derive(Parser)]
struct LlmwaveBigContractArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1ContractArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1FieldCutoverArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1MemoryWriterArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1NonlinearProofArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1QueryWaveArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1ActiveRetrievalArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1SchemaReasoningArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1SurfaceGenerationArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1AnswerVerifierArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1FeedbackLearningArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1ConsolidationSleepArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV1BroadEvalHarnessArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2ContractArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2CorpusArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2HeldoutArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2FocusArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2DensityArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2RunArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2PackHotArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV2ClaimGateArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV3PlanArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV3SolutionSearchArgs {
    #[arg(long, default_value = "confirm customs clearance")]
    goal: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV3Pack1mArgs {
    #[arg(long, default_value = core_v3::DEFAULT_1M_MANIFEST)]
    manifest: PathBuf,
    #[arg(long, default_value = core_v3::DEFAULT_1M_FOCUS)]
    focus: PathBuf,
    #[arg(long, default_value = core_v3::DEFAULT_1M_HELDOUT)]
    heldout: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreV3ClaimGateArgs {
    #[arg(long, default_value = "confirm customs clearance")]
    goal: String,
    #[arg(long, default_value = core_v3::DEFAULT_1M_MANIFEST)]
    manifest: PathBuf,
    #[arg(long, default_value = core_v3::DEFAULT_1M_FOCUS)]
    focus: PathBuf,
    #[arg(long, default_value = core_v3::DEFAULT_1M_HELDOUT)]
    heldout: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigAtlasArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigActiveCoreArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigL2Args {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigHrrArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSchemaBindArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigL2L3CoupleArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigDecodeLoopArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMultiSchemaArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSchemaGrowArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceGenerateArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigReasonFieldArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigDialogueStateArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMiniChatEvalArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigQueryWaveArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMultiPeakFieldArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLensScanArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMatureAntiWaveArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigEvidenceProofArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, default_value = "missing")]
    evidence_mode: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigAnswerSurfaceArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, default_value = "missing")]
    evidence_mode: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigFieldFeedbackArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, default_value = "release-confirmed")]
    evidence_mode: String,
    #[arg(long, default_value = "accept")]
    decision: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigFeedbackMemoryArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, default_value = "release-confirmed")]
    evidence_mode: String,
    #[arg(long, default_value = "accept")]
    decision: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigFeedbackAwareFieldArgs {
    #[arg(long, default_value = "Has customs cleared the goods?")]
    text: String,
    #[arg(long, default_value = "accept")]
    memory_mode: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigAppliedAntiMemoryArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMemoryStoreArgs {
    #[arg(long, default_value = ".nanda/llmwave-big-memory.json")]
    path: PathBuf,
    #[arg(long, default_value = "apply")]
    action: String,
    #[arg(long, default_value = "accept")]
    decision: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLearningEvalArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMemoryConsolidateArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigRunArgs {
    #[arg(long, default_value = "release-confirmed")]
    evidence_mode: String,
    #[arg(long, default_value = "accept")]
    decision: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigCoreEvalArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigStructuralCapacityArgs {
    #[arg(long, default_value_t = 13)]
    seed: u64,
    #[arg(long, default_value_t = 2)]
    seeds: usize,
    #[arg(long = "noise-edges", default_value_t = 4)]
    noise_edges: usize,
    #[arg(long = "noise-profile", value_enum, default_value = "default")]
    noise_profile: LlmwaveBigStructuralCapacityNoiseProfile,
    #[arg(long = "hot-budget-bytes", default_value_t = 6 * 1024 * 1024)]
    hot_budget_bytes: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Clone, Copy, ValueEnum)]
enum LlmwaveBigStructuralCapacityNoiseProfile {
    Default,
    SkillAdmission,
}

impl From<LlmwaveBigStructuralCapacityNoiseProfile>
    for structural_capacity::StructuralCapacityNoiseProfile
{
    fn from(value: LlmwaveBigStructuralCapacityNoiseProfile) -> Self {
        match value {
            LlmwaveBigStructuralCapacityNoiseProfile::Default => Self::Default,
            LlmwaveBigStructuralCapacityNoiseProfile::SkillAdmission => Self::SkillAdmission,
        }
    }
}

#[derive(Parser)]
struct LlmwaveBigReadinessLadderArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigClaimGateArgs {
    #[arg(long, value_enum)]
    claim: readiness::ClaimGateKind,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigNonlinearMemoryEvalArgs {
    #[arg(long)]
    corpus: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "strict-full-sweep")]
    proof_policy: nonlinear_memory_eval::NonlinearProofPolicyKind,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigNonlinearMemoryLadderArgs {
    #[arg(long, default_value_t = 100_000)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigWordBirthArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceProductionArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceReconstructArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceCorpusEvalArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceBankBuildArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceBankValidateArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceBankFixtureArgs {
    #[arg(long, default_value = "examples/llmwave-big-surface-corpus.json")]
    corpus: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSurfaceRawInduceArgs {
    #[arg(
        long,
        default_value = "examples/llmwave-big-raw-surface-corpus-ru.json"
    )]
    corpus: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigWriteArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSchemaResidualEngineArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMemoryPhysicsArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMemoryProofPathArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMemoryFinalProofArgs {
    #[arg(long)]
    artifact: Option<PathBuf>,
    #[arg(long = "heldout-suite")]
    heldout_suite: Option<PathBuf>,
    #[arg(long = "focus-packet")]
    focus_packet: Option<PathBuf>,
    #[arg(long = "compile-evidence")]
    compile_evidence: Option<PathBuf>,
    #[arg(long = "heldout-eval")]
    heldout_eval: Option<PathBuf>,
    #[arg(long = "strict-density-evidence")]
    strict_density_evidence: Option<PathBuf>,
    #[arg(long = "multi-profile-density-evidence")]
    multi_profile_density_evidence: Option<PathBuf>,
    #[arg(long = "density-doctor-evidence")]
    density_doctor_evidence: Option<PathBuf>,
    #[arg(long = "density-hot-packet")]
    density_hot_packet: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "general")]
    profile: memory_final_proof::MemoryProofProfile,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigConsolidateArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigEvalArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigQueryArgs {
    #[arg(long, default_value = "supplier invoice payment customs")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigTrainArgs {
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 65536)]
    vocab_cap: usize,
    #[arg(long, default_value_t = 262144)]
    transition_cap: usize,
    #[arg(long, default_value_t = 32768)]
    active_chunk_cap: usize,
    #[arg(long, default_value_t = 64)]
    chunk_tokens: usize,
    #[arg(long, default_value_t = 6 * 1024 * 1024)]
    hot_budget_bytes: usize,
    #[arg(long, default_value_t = 4 * 1024 * 1024)]
    max_file_bytes: usize,
    #[arg(long, default_value_t = training::default_extensions_csv())]
    extensions: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigRustCorpusBuildArgs {
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 4 * 1024 * 1024)]
    max_file_bytes: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigRustHeldoutBuildArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 64)]
    max_cases: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigRustFocusBuildArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long = "heldout-suite")]
    heldout_suite: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 15_000)]
    max_facts: usize,
    #[arg(long, default_value_t = 256)]
    route_fact_cap: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigRustCompileEvidenceBuildArgs {
    #[arg(long = "focus-packet")]
    focus_packet: PathBuf,
    #[arg(long = "check-evidence")]
    check_evidence: PathBuf,
    #[arg(long = "test-evidence")]
    test_evidence: PathBuf,
    #[arg(long = "clippy-evidence")]
    clippy_evidence: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigRustHeldoutEvalArgs {
    #[arg(long = "focus-packet")]
    focus_packet: PathBuf,
    #[arg(long = "heldout-suite")]
    heldout_suite: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 0.80)]
    pass_threshold: f64,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigStrictDensityClaimGateArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long = "focus-packet")]
    focus_packet: PathBuf,
    #[arg(long = "heldout-eval")]
    heldout_eval: PathBuf,
    #[arg(long = "compile-evidence")]
    compile_evidence: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigProfileDensityBuildArgs {
    #[arg(long)]
    profile: String,
    #[arg(long)]
    corpus: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigMultiProfileDensitySuiteArgs {
    #[arg(long = "rust-density")]
    rust_density: Option<PathBuf>,
    #[arg(long = "profile-evidence")]
    profile_evidence: Vec<String>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 3)]
    min_profiles: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigDensityProofDoctorArgs {
    #[arg(long)]
    suite: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 3)]
    medium_profile_min: usize,
    #[arg(long, default_value_t = 5)]
    strong_profile_min: usize,
    #[arg(long, default_value_t = 50)]
    min_fact_count: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigDensityAblationArgs {
    #[arg(long)]
    suite: PathBuf,
    #[arg(long = "out-hot-packet")]
    out_hot_packet: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadCorpusBuildArgs {
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long, default_value = "builtin-micro")]
    profile: String,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxAtlasBuildArgs {
    #[arg(long, default_value = "/")]
    root: PathBuf,
    #[arg(long = "out-dir", default_value = ".nanda/linux-atlas")]
    out_dir: PathBuf,
    #[arg(long = "pack-kind", default_value = "base")]
    pack_kind: String,
    #[arg(long = "max-facts", default_value_t = 1_000_000)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxActiveFieldArgs {
    #[arg(long = "atlas-dir", default_value = ".nanda/linux-atlas")]
    atlas_dir: PathBuf,
    #[arg(long = "max-active-facts", default_value_t = 65_536)]
    max_active_facts: usize,
    #[arg(long = "query")]
    query: Vec<String>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxPackHotArgs {
    #[arg(long = "atlas-dir", default_value = ".nanda/linux-atlas")]
    atlas_dir: PathBuf,
    #[arg(long = "max-active-facts", default_value_t = 65_536)]
    max_active_facts: usize,
    #[arg(long, default_value = ".nanda/linux-active/linux-active-65k.laf")]
    out: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxAskHotArgs {
    #[arg(
        long = "hot-pack",
        default_value = ".nanda/linux-active/linux-active-65k.laf"
    )]
    hot_pack: PathBuf,
    #[arg(long = "query")]
    query: String,
    #[arg(long = "top-k", default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxHotEvalArgs {
    #[arg(
        long = "hot-pack",
        default_value = ".nanda/linux-active/linux-active-65k.laf"
    )]
    hot_pack: PathBuf,
    #[arg(long = "top-k", default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxDomainRunArgs {
    #[arg(
        long = "hot-pack",
        default_value = ".nanda/linux-active/linux-active-65k.laf"
    )]
    hot_pack: PathBuf,
    #[arg(long = "query")]
    query: String,
    #[arg(long = "top-k", default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxAtlasProjectionArgs {
    #[arg(long = "atlas-dir", default_value = ".nanda/linux-atlas")]
    atlas_dir: PathBuf,
    #[arg(
        long = "hot-pack",
        default_value = ".nanda/linux-active/linux-active-65k.laf"
    )]
    hot_pack: PathBuf,
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "query", default_value = "which package provides command bash")]
    query: String,
    #[arg(long = "top-k", default_value_t = 5)]
    top_k: usize,
    #[arg(long = "iterations", default_value_t = 64)]
    iterations: usize,
    #[arg(long = "warmup-iterations", default_value_t = 8)]
    warmup_iterations: usize,
    #[arg(long = "samples", default_value_t = 5)]
    samples: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxCacheProofArgs {
    #[arg(
        long = "hot-pack",
        default_value = ".nanda/linux-active/linux-active-65k.laf"
    )]
    hot_pack: PathBuf,
    #[arg(long = "query")]
    query: String,
    #[arg(long = "iterations", default_value_t = 64)]
    iterations: usize,
    #[arg(long = "warmup-iterations", default_value_t = 8)]
    warmup_iterations: usize,
    #[arg(long = "samples", default_value_t = 5)]
    samples: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxPmuCacheProofArgs {
    #[arg(
        long = "hot-pack",
        default_value = ".nanda/linux-active/linux-active-65k.laf"
    )]
    hot_pack: PathBuf,
    #[arg(long = "query", default_value = "which package provides command bash")]
    query: String,
    #[arg(long = "iterations", default_value_t = 64)]
    iterations: usize,
    #[arg(long = "warmup-iterations", default_value_t = 8)]
    warmup_iterations: usize,
    #[arg(long = "samples", default_value_t = 5)]
    samples: usize,
    #[arg(long = "max-cache-miss-rate", default_value_t = 0.02)]
    max_cache_miss_rate: f64,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxPackResidualArgs {
    #[arg(long = "atlas-dir", default_value = ".nanda/linux-atlas")]
    atlas_dir: PathBuf,
    #[arg(long = "max-active-facts", default_value_t = 65_536)]
    max_active_facts: usize,
    #[arg(long = "promotion-threshold", default_value_t = 2)]
    promotion_threshold: usize,
    #[arg(long, default_value = ".nanda/linux-active/linux-active-65k.lrf")]
    out: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxResidualProofArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "query")]
    query: String,
    #[arg(long = "top-k", default_value_t = 5)]
    top_k: usize,
    #[arg(long = "iterations", default_value_t = 64)]
    iterations: usize,
    #[arg(long = "warmup-iterations", default_value_t = 8)]
    warmup_iterations: usize,
    #[arg(long = "samples", default_value_t = 5)]
    samples: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxExposureRunArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "max-candidates", default_value_t = 16)]
    max_candidates: usize,
    #[arg(long = "runtime-snapshot")]
    runtime_snapshot: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxSnapshotImportArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigDaybreakDuelArgs {
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigSecurityFixtureRunArgs {
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatRunArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "prompt")]
    prompt: Vec<String>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatV1Args {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "prompt")]
    prompt: Vec<String>,
    #[arg(long = "script")]
    script: Option<PathBuf>,
    #[arg(long = "runtime-snapshot")]
    runtime_snapshot: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatV1EvalArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "runtime-snapshot")]
    runtime_snapshot: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatV2Args {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(
        long = "memory",
        default_value = ".nanda/linux-active/linux-chat-v2.lwm"
    )]
    memory: PathBuf,
    #[arg(long = "prompt")]
    prompt: Vec<String>,
    #[arg(long = "script")]
    script: Option<PathBuf>,
    #[arg(long = "runtime-snapshot")]
    runtime_snapshot: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long = "reset-memory", default_value_t = false)]
    reset_memory: bool,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatV2EvalArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(
        long = "memory",
        default_value = ".nanda/linux-active/linux-chat-v2-eval.lwm"
    )]
    memory: PathBuf,
    #[arg(long = "runtime-snapshot")]
    runtime_snapshot: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxCenterLearnArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(
        long = "memory",
        default_value = ".nanda/linux-active/linux-center-learning.lwm"
    )]
    memory: PathBuf,
    #[arg(long = "script")]
    script: Option<PathBuf>,
    #[arg(long = "heldout-eval")]
    heldout_eval: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long = "reset-memory", default_value_t = false)]
    reset_memory: bool,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxVpnTrainArgs {
    #[arg(long = "memory", default_value = ".nanda/linux-active/linux-vpn.lwm")]
    memory: PathBuf,
    #[arg(long = "reset-memory", default_value_t = false)]
    reset_memory: bool,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxVpnTrainEvalArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(
        long = "memory",
        default_value = ".nanda/linux-active/linux-vpn-eval.lwm"
    )]
    memory: PathBuf,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxVpnActionPlanArgs {
    #[arg(long = "text")]
    text: Option<String>,
    #[arg(long = "action")]
    action: Option<String>,
    #[arg(long = "backend")]
    backend: Option<String>,
    #[arg(long = "target")]
    target: Option<String>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxVpnControlArgs {
    #[arg(long = "action")]
    action: String,
    #[arg(long = "backend")]
    backend: String,
    #[arg(long = "target")]
    target: Option<String>,
    #[arg(long = "execute", default_value_t = false)]
    execute: bool,
    #[arg(long = "i-understand-network-may-drop", default_value_t = false)]
    i_understand_network_may_drop: bool,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxQueryWaveArgs {
    #[arg(long = "text")]
    text: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxBroadSuiteBuildArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "cases", default_value_t = 100)]
    cases: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxBroadEvalRunArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long)]
    suite: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxReasonRunArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "text")]
    text: String,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long = "runtime-snapshot")]
    runtime_snapshot: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxProfileClaimGateArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "broad-eval")]
    broad_eval: Option<PathBuf>,
    #[arg(long = "heldout-eval")]
    heldout_eval: Option<PathBuf>,
    #[arg(long = "run-chat-learning-eval", default_value_t = false)]
    run_chat_learning_eval: bool,
    #[arg(
        long = "chat-learning-memory",
        default_value = ".nanda/linux-active/linux-chat-profile.lwm"
    )]
    chat_learning_memory: PathBuf,
    #[arg(long = "run-center-learning-eval", default_value_t = false)]
    run_center_learning_eval: bool,
    #[arg(
        long = "center-learning-memory",
        default_value = ".nanda/linux-active/linux-center-learning.lwm"
    )]
    center_learning_memory: PathBuf,
    #[arg(long = "center-learning-script")]
    center_learning_script: Option<PathBuf>,
    #[arg(long = "run-vpn-training-eval", default_value_t = false)]
    run_vpn_training_eval: bool,
    #[arg(
        long = "vpn-memory",
        default_value = ".nanda/linux-active/linux-chat-profile-vpn.lwm"
    )]
    vpn_memory: PathBuf,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatCoreBuildArgs {
    #[arg(long = "memory-root")]
    memory_root: Option<PathBuf>,
    #[arg(long = "profile")]
    profile: Option<PathBuf>,
    #[arg(long = "residual-pack")]
    residual_pack: Option<PathBuf>,
    #[arg(long = "dialogue-overlay")]
    dialogue_overlay: Option<PathBuf>,
    #[arg(long = "centers-overlay")]
    centers_overlay: Option<PathBuf>,
    #[arg(long = "vpn-overlay")]
    vpn_overlay: Option<PathBuf>,
    #[arg(long = "broad-eval")]
    broad_eval: Option<PathBuf>,
    #[arg(long = "heldout-eval")]
    heldout_eval: Option<PathBuf>,
    #[arg(long = "cache-dir")]
    cache_dir: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatCoreGateArgs {
    #[arg(long = "memory-root")]
    memory_root: Option<PathBuf>,
    #[arg(long = "profile")]
    profile: Option<PathBuf>,
    #[arg(long = "residual-pack")]
    residual_pack: Option<PathBuf>,
    #[arg(long = "dialogue-overlay")]
    dialogue_overlay: Option<PathBuf>,
    #[arg(long = "centers-overlay")]
    centers_overlay: Option<PathBuf>,
    #[arg(long = "vpn-overlay")]
    vpn_overlay: Option<PathBuf>,
    #[arg(long = "broad-eval")]
    broad_eval: Option<PathBuf>,
    #[arg(long = "heldout-eval")]
    heldout_eval: Option<PathBuf>,
    #[arg(long = "center-learning-script")]
    center_learning_script: Option<PathBuf>,
    #[arg(long = "cache-dir")]
    cache_dir: Option<PathBuf>,
    #[arg(long = "manifest")]
    manifest: Option<PathBuf>,
    #[arg(long = "rebuild-cache", default_value_t = false)]
    rebuild_cache: bool,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatCoreAskArgs {
    #[arg(long = "text")]
    text: String,
    #[arg(long = "memory-root")]
    memory_root: Option<PathBuf>,
    #[arg(long = "residual-pack")]
    residual_pack: Option<PathBuf>,
    #[arg(long = "dialogue-overlay")]
    dialogue_overlay: Option<PathBuf>,
    #[arg(long = "centers-overlay")]
    centers_overlay: Option<PathBuf>,
    #[arg(long = "vpn-overlay")]
    vpn_overlay: Option<PathBuf>,
    #[arg(long = "cache-dir")]
    cache_dir: Option<PathBuf>,
    #[arg(long = "manifest")]
    manifest: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long = "packet-profile")]
    packet_profile: Option<String>,
    #[arg(long = "target-packet-tokens")]
    target_packet_tokens: Option<u64>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatCoreLearnArgs {
    #[arg(long = "memory-root")]
    memory_root: Option<PathBuf>,
    #[arg(long = "profile")]
    profile: Option<PathBuf>,
    #[arg(long = "residual-pack")]
    residual_pack: Option<PathBuf>,
    #[arg(long = "dialogue-overlay")]
    dialogue_overlay: Option<PathBuf>,
    #[arg(long = "centers-overlay")]
    centers_overlay: Option<PathBuf>,
    #[arg(long = "vpn-overlay")]
    vpn_overlay: Option<PathBuf>,
    #[arg(long = "broad-eval")]
    broad_eval: Option<PathBuf>,
    #[arg(long = "heldout-eval")]
    heldout_eval: Option<PathBuf>,
    #[arg(long = "cache-dir")]
    cache_dir: Option<PathBuf>,
    #[arg(long = "accept", conflicts_with = "reject")]
    accept: Option<String>,
    #[arg(long = "reject", conflicts_with = "accept")]
    reject: Option<String>,
    #[arg(long = "domain")]
    domain: Option<String>,
    #[arg(long = "overlay")]
    overlay: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxChatCoreLearnEvalArgs {
    #[arg(long = "memory-root")]
    memory_root: Option<PathBuf>,
    #[arg(long = "profile")]
    profile: Option<PathBuf>,
    #[arg(long = "residual-pack")]
    residual_pack: Option<PathBuf>,
    #[arg(long = "cache-dir")]
    cache_dir: Option<PathBuf>,
    #[arg(long = "reset-scratch", default_value_t = false)]
    reset_scratch: bool,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

struct ResolvedLinuxChatCorePaths {
    profile: PathBuf,
    residual_pack: PathBuf,
    dialogue_overlay: PathBuf,
    centers_overlay: PathBuf,
    vpn_overlay: PathBuf,
    broad_eval: Option<PathBuf>,
    heldout_eval: Option<PathBuf>,
    cache_dir: PathBuf,
}

struct LinuxChatCorePathOverrides {
    profile: Option<PathBuf>,
    residual_pack: Option<PathBuf>,
    dialogue_overlay: Option<PathBuf>,
    centers_overlay: Option<PathBuf>,
    vpn_overlay: Option<PathBuf>,
    broad_eval: Option<PathBuf>,
    heldout_eval: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
}

fn resolve_linux_chat_core_paths(
    memory_root: Option<PathBuf>,
    overrides: LinuxChatCorePathOverrides,
) -> ResolvedLinuxChatCorePaths {
    let root = normalize_existing_path(
        memory_root.unwrap_or_else(|| PathBuf::from(".nanda/linux-active")),
    );
    let root_profile = root.join("chat-core.profile.json");
    let default_profile = if root_profile.exists() {
        normalize_existing_path(root_profile)
    } else {
        normalize_existing_path(PathBuf::from(
            linux_chat_core::DEFAULT_LINUX_CHAT_CORE_PROFILE,
        ))
    };
    let optional_existing = |path: PathBuf| path.exists().then(|| normalize_existing_path(path));
    ResolvedLinuxChatCorePaths {
        profile: overrides
            .profile
            .map(normalize_existing_path)
            .unwrap_or(default_profile),
        residual_pack: overrides
            .residual_pack
            .map(normalize_existing_path)
            .unwrap_or_else(|| root.join("linux-active-65k.lrf")),
        dialogue_overlay: overrides
            .dialogue_overlay
            .map(normalize_existing_path)
            .unwrap_or_else(|| root.join("linux-chat-profile.lwm")),
        centers_overlay: overrides
            .centers_overlay
            .map(normalize_existing_path)
            .unwrap_or_else(|| root.join("linux-center-learning.lwm")),
        vpn_overlay: overrides
            .vpn_overlay
            .map(normalize_existing_path)
            .unwrap_or_else(|| root.join("linux-chat-profile-vpn.lwm")),
        broad_eval: overrides
            .broad_eval
            .map(normalize_existing_path)
            .or_else(|| optional_existing(root.join("linux-broad-eval.json"))),
        heldout_eval: overrides
            .heldout_eval
            .map(normalize_existing_path)
            .or_else(|| optional_existing(root.join("linux-heldout-eval.json"))),
        cache_dir: overrides
            .cache_dir
            .map(normalize_existing_path)
            .unwrap_or_else(|| root.join("cache")),
    }
}

fn normalize_existing_path(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}

#[derive(Parser)]
struct LlmwaveBigLinuxHeldoutSuiteBuildArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "cases", default_value_t = 100)]
    cases: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxHeldoutEvalRunArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long)]
    suite: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxFeedbackBuildArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "text")]
    text: String,
    #[arg(long = "decision", default_value = "reject")]
    decision: String,
    #[arg(long)]
    note: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxFeedbackApplyArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long)]
    feedback: PathBuf,
    #[arg(long = "text")]
    text: String,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxDecisionSearchArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long = "text")]
    text: String,
    #[arg(long = "max-facts", default_value_t = 4)]
    max_facts: usize,
    #[arg(long = "runtime-snapshot")]
    runtime_snapshot: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLinuxRelationProfileArgs {
    #[arg(
        long = "residual-pack",
        default_value = ".nanda/linux-active/linux-active-65k.lrf"
    )]
    residual_pack: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadEvalSuiteBuildArgs {
    #[arg(long)]
    corpus: Option<PathBuf>,
    #[arg(
        long,
        default_value = "recall,role,route,multihop,context,generation,adversarial,feedback"
    )]
    families: String,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadDatasetDoctorArgs {
    #[arg(long)]
    corpus: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 16)]
    medium_min_facts: usize,
    #[arg(long, default_value_t = 64)]
    strong_min_facts: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadHeldoutBuildArgs {
    #[arg(long)]
    corpus: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 32)]
    max_cases: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadFocusBuildArgs {
    #[arg(long)]
    corpus: PathBuf,
    #[arg(long = "heldout-suite")]
    heldout_suite: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = 15_000)]
    max_facts: usize,
    #[arg(long, default_value_t = 256)]
    route_fact_cap: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadEvalRunArgs {
    #[arg(long)]
    corpus: Option<PathBuf>,
    #[arg(long)]
    suite: Option<PathBuf>,
    #[arg(long = "focus-packet")]
    focus_packet: Option<PathBuf>,
    #[arg(long = "hot-packet")]
    hot_packet: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadBaselineDuelArgs {
    #[arg(long = "eval-report")]
    eval_report: Option<PathBuf>,
    #[arg(long, default_value = "lexical,flat,route-only,markov")]
    baselines: String,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigBroadChatLoopEvalArgs {
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLlmwaveReadinessArgs {
    #[arg(long = "memory-final-proof")]
    memory_final_proof: Option<PathBuf>,
    #[arg(long = "broad-dataset-doctor")]
    broad_dataset_doctor: Option<PathBuf>,
    #[arg(long = "broad-eval")]
    broad_eval: Option<PathBuf>,
    #[arg(long = "baseline-duel")]
    baseline_duel: Option<PathBuf>,
    #[arg(long = "chat-loop")]
    chat_loop: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigPackHotArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigAskHotArgs {
    #[arg(long)]
    hot_pack: PathBuf,
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    memory: Option<PathBuf>,
    #[arg(long)]
    text: String,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigLearnHotArgs {
    #[arg(long)]
    feedback: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigChatHotArgs {
    #[arg(long)]
    hot_pack: PathBuf,
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    memory: PathBuf,
    #[arg(long)]
    script: Option<PathBuf>,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigChatHotEvalArgs {
    #[arg(long)]
    hot_pack: PathBuf,
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    memory: PathBuf,
    #[arg(long)]
    script: PathBuf,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigDomainEvalArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    ask_suite: PathBuf,
    #[arg(long)]
    hot_pack: PathBuf,
    #[arg(long)]
    chat_script: PathBuf,
    #[arg(long)]
    chat_memory: PathBuf,
    #[arg(long)]
    nonlinear_corpus: PathBuf,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigDemoDomainArgs {
    #[arg(long, default_value = ".nanda/llmwave-big-demo")]
    out_dir: PathBuf,
    #[arg(
        long,
        default_value = "examples/llmwave-big-nonlinear-memory-corpus.json"
    )]
    nonlinear_corpus: PathBuf,
    #[arg(long, default_value_t = 3)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigAskArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    text: String,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct LlmwaveBigAskEvalArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    suite: PathBuf,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Serialize, Clone)]
pub(crate) struct LlmwaveBigReport {
    pub command: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub implementation_state: &'static str,
    pub core_version: &'static str,
    pub nanda_6m_version: &'static str,
    pub wave_dim: usize,
    pub contract: contract::BigModelContract,
    pub bigness_metrics: metrics::BignessMetricsContract,
    pub claim_boundary: claims::ClaimBoundary,
    pub engineering_rules: EngineeringRulesReport,
    pub next_versions: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct EngineeringRulesReport {
    pub source: &'static str,
    pub pattern_store_boundary: &'static str,
    pub hot_core_rules: Vec<&'static str>,
    pub atlas_rules: Vec<&'static str>,
    pub l2_l3_boundary: Vec<&'static str>,
}

pub(super) fn cmd(args: LlmwaveBigArgs) -> Result<u8> {
    match args.command {
        LlmwaveBigCommand::CoreV1Contract(args) => {
            let report = core_v1_contract::build_core_v1_contract_report();
            report::print_core_v1_contract_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1FieldCutover(args) => {
            let report = core_v1_field_cutover::build_core_v1_field_cutover_report();
            report::print_core_v1_field_cutover_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1MemoryWriter(args) => {
            let report = core_v1_memory_writer::build_core_v1_memory_writer_report();
            report::print_core_v1_memory_writer_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1NonlinearProof(args) => {
            let report = core_v1_nonlinear_proof::build_core_v1_nonlinear_proof_report();
            report::print_core_v1_nonlinear_proof_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1QueryWave(args) => {
            let report = core_v1_query_wave::build_core_v1_query_wave_report(args.text);
            report::print_core_v1_query_wave_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1ActiveRetrieval(args) => {
            let report = core_v1_active_retrieval::build_core_v1_active_retrieval_report(args.text);
            report::print_core_v1_active_retrieval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1SchemaReasoning(args) => {
            let report = core_v1_schema_reasoning::build_core_v1_schema_reasoning_report(args.text);
            report::print_core_v1_schema_reasoning_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1SurfaceGeneration(args) => {
            let report =
                core_v1_surface_generation::build_core_v1_surface_generation_report(args.text);
            report::print_core_v1_surface_generation_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1AnswerVerifier(args) => {
            let report = core_v1_answer_verifier::build_core_v1_answer_verifier_report(args.text);
            report::print_core_v1_answer_verifier_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1FeedbackLearning(args) => {
            let report =
                core_v1_feedback_learning::build_core_v1_feedback_learning_report(args.text);
            report::print_core_v1_feedback_learning_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1ConsolidationSleep(args) => {
            let report =
                core_v1_consolidation_sleep::build_core_v1_consolidation_sleep_report(args.text);
            report::print_core_v1_consolidation_sleep_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV1BroadEvalHarness(args) => {
            let report =
                core_v1_broad_eval_harness::build_core_v1_broad_eval_harness_report(args.text);
            report::print_core_v1_broad_eval_harness_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2Contract(args) => {
            let report = core_v2::build_core_v2_contract_report();
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Contract",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2Corpus(args) => {
            let report = core_v2::build_core_v2_corpus_report();
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Corpus",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2Heldout(args) => {
            let report = core_v2::build_core_v2_heldout_report();
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Heldout",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2Focus(args) => {
            let report = core_v2::build_core_v2_focus_report();
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Focus",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2Density(args) => {
            let report = core_v2::build_core_v2_density_report();
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Density",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2Run(args) => {
            let report = core_v2::build_core_v2_run_report(args.text);
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Run",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2PackHot(args) => {
            let report = core_v2::build_core_v2_pack_hot_report();
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Hot Packet",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV2ClaimGate(args) => {
            let report = core_v2::build_core_v2_claim_gate_report();
            report::print_core_v2_report(
                &report,
                &args.format,
                "LLMWave Core V2 Claim Gate",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV3Plan(args) => {
            let report = core_v3::build_core_v3_plan_report();
            report::print_core_v3_report(
                &report,
                &args.format,
                "LLMWave Core V3 Plan",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV3SolutionSearch(args) => {
            let report = core_v3::build_core_v3_solution_search_report(args.goal);
            report::print_core_v3_report(
                &report,
                &args.format,
                "LLMWave Core V3 Solution Search",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV3Pack1m(args) => {
            let paths = core_v3::CoreV3ExternalPaths {
                manifest: args.manifest,
                focus: args.focus,
                heldout: args.heldout,
            };
            let report = core_v3::build_core_v3_million_pack_report(&paths)?;
            report::print_core_v3_report(
                &report,
                &args.format,
                "LLMWave Core V3 1M Active Projection",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreV3ClaimGate(args) => {
            let paths = core_v3::CoreV3ExternalPaths {
                manifest: args.manifest,
                focus: args.focus,
                heldout: args.heldout,
            };
            let report = core_v3::build_core_v3_claim_gate_report(args.goal, &paths)?;
            report::print_core_v3_report(
                &report,
                &args.format,
                "LLMWave Core V3 Claim Gate",
                report.verdict,
            )?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Contract(args) => {
            let report = build_contract_report();
            report::print_contract_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Atlas(args) => {
            let report = atlas::build_atlas_report();
            report::print_atlas_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::ActiveCore(args) => {
            let report = active_core::build_active_core_report();
            report::print_active_core_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::L2(args) => {
            let report =
                l2_word_field::build_l2_word_field_report(l3_schema_field::business_invoice_bias());
            report::print_l2_word_field_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Hrr(args) => {
            let report = hrr_binding::build_hrr_binding_report();
            report::print_hrr_binding_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SchemaBind(args) => {
            let report = l3_schema_bind::build_l3_schema_bind_report();
            report::print_l3_schema_bind_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::L2L3Couple(args) => {
            let report = l2_l3_coupling::build_l2_l3_coupling_report();
            report::print_l2_l3_coupling_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::DecodeLoop(args) => {
            let report = coupled_decode_loop::build_coupled_decode_loop_report();
            report::print_coupled_decode_loop_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MultiSchema(args) => {
            let report = multi_schema_competition::build_multi_schema_competition_report();
            report::print_multi_schema_competition_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SchemaGrow(args) => {
            let report = schema_memory_growth::build_schema_memory_growth_report();
            report::print_schema_memory_growth_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceGenerate(args) => {
            let report = open_surface_generation::build_open_surface_generation_report();
            report::print_open_surface_generation_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::ReasonField(args) => {
            let report = reasoning_field::build_reasoning_field_report();
            report::print_reasoning_field_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::DialogueState(args) => {
            let report = dialogue_state::build_dialogue_state_report();
            report::print_dialogue_state_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MiniChatEval(args) => {
            let report = mini_chat_eval::build_mini_chat_eval_report();
            report::print_mini_chat_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::QueryWave(args) => {
            let report = query_wave::build_query_wave_report(args.text);
            report::print_query_wave_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MultiPeakField(args) => {
            let report = multi_peak_field::build_multi_peak_field_report(args.text);
            report::print_multi_peak_field_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LensScan(args) => {
            let report = lens_scan::build_lens_scan_report(args.text);
            report::print_lens_scan_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MatureAntiWave(args) => {
            let report = mature_anti_wave::build_mature_anti_wave_report(args.text);
            report::print_mature_anti_wave_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::EvidenceProof(args) => {
            let report = evidence_proof::build_evidence_proof_report(args.text, args.evidence_mode);
            report::print_evidence_proof_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::AnswerSurface(args) => {
            let report = answer_surface::build_answer_surface_report(args.text, args.evidence_mode);
            report::print_answer_surface_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::FieldFeedback(args) => {
            let report = field_feedback::build_field_feedback_report(
                args.text,
                args.evidence_mode,
                args.decision,
            );
            report::print_field_feedback_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::FeedbackMemory(args) => {
            let report = field_runtime::build_applied_feedback_memory_report(
                args.text,
                args.evidence_mode,
                args.decision,
            );
            report::print_applied_feedback_memory_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::FeedbackAwareField(args) => {
            let report =
                field_runtime::build_feedback_aware_field_report(args.text, args.memory_mode);
            report::print_feedback_aware_field_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::AppliedAntiMemory(args) => {
            let report = field_runtime::build_applied_anti_memory_report();
            report::print_applied_anti_memory_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MemoryStore(args) => {
            let report = field_runtime::build_persistent_memory_store_report(
                &args.path,
                args.action,
                args.decision,
            )?;
            report::print_persistent_memory_store_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LearningEval(args) => {
            let report = field_runtime::build_learning_eval_report();
            report::print_learning_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MemoryConsolidate(args) => {
            let report = field_runtime::build_memory_consolidate_report();
            report::print_memory_consolidate_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Run(args) => {
            let report =
                field_runtime::build_runtime_pipeline_report(args.evidence_mode, args.decision);
            report::print_runtime_pipeline_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::CoreEval(args) => {
            let report = field_runtime::build_core_eval_report();
            report::print_core_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::StructuralCapacity(args) => {
            let report = structural_capacity::build_structural_capacity_report(
                structural_capacity::StructuralCapacityConfig {
                    seed: args.seed,
                    seeds: args.seeds,
                    noise_edges: args.noise_edges,
                    hot_budget_bytes: args.hot_budget_bytes,
                    noise_profile: args.noise_profile.into(),
                },
            );
            print_structural_capacity_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::ReadinessLadder(args) => {
            let report = readiness::build_readiness_ladder_report();
            report::print_readiness_ladder_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::ClaimGate(args) => {
            let report = readiness::build_claim_gate_report(args.claim);
            let exit = if report.allowed {
                EXIT_PASS
            } else {
                super::EXIT_WATCH
            };
            report::print_claim_gate_report(&report, &args.format)?;
            Ok(exit)
        }
        LlmwaveBigCommand::NonlinearMemoryEval(args) => {
            let report = nonlinear_memory_eval::build_nonlinear_memory_eval_report(
                args.corpus.as_deref(),
                args.proof_policy,
            )?;
            report::print_nonlinear_memory_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::NonlinearMemoryLadder(args) => {
            let report =
                nonlinear_memory_eval::build_nonlinear_memory_ladder_report(args.max_facts);
            report::print_nonlinear_memory_ladder_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::WordBirth(args) => {
            let report = lexical_birth::build_lexical_birth_report();
            report::print_lexical_birth_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceProduction(args) => {
            let report = surface_production::build_surface_production_report();
            report::print_surface_production_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceReconstruct(args) => {
            let report = surface_reconstruct::build_surface_reconstruct_report();
            report::print_surface_reconstruct_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceCorpusEval(args) => {
            let report = surface_corpus_eval::build_surface_corpus_eval_report();
            report::print_surface_corpus_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceBankBuild(args) => {
            let report = surface_bank_build::build_surface_bank_build_report();
            report::print_surface_bank_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceBankValidate(args) => {
            let report = surface_bank_validate::build_surface_bank_validate_report();
            report::print_surface_bank_validate_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceBankFixture(args) => {
            let report = surface_bank_fixture::build_surface_bank_fixture_report(&args.corpus)?;
            report::print_surface_bank_fixture_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SurfaceRawInduce(args) => {
            let report = surface_raw_induce::build_surface_raw_induce_report(&args.corpus)?;
            report::print_surface_raw_induce_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Write(args) => {
            let report = write::build_write_report();
            report::print_write_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SchemaResidualEngine(args) => {
            let report = write::build_schema_residual_engine_report();
            report::print_schema_residual_engine_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MemoryPhysics(args) => {
            let report = memory_physics::build_memory_physics_report();
            report::print_memory_physics_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MemoryProofPath(args) => {
            let report = memory_proof_path::build_memory_proof_path_report();
            report::print_memory_proof_path_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MemoryFinalProof(args) => {
            let report = memory_final_proof::build_memory_final_proof_report(
                memory_final_proof::MemoryFinalProofConfig {
                    profile: args.profile,
                    artifact: args.artifact,
                    heldout_suite: args.heldout_suite,
                    focus_packet: args.focus_packet,
                    compile_evidence: args.compile_evidence,
                    heldout_eval: args.heldout_eval,
                    strict_density_evidence: args.strict_density_evidence,
                    multi_profile_density_evidence: args.multi_profile_density_evidence,
                    density_doctor_evidence: args.density_doctor_evidence,
                    density_hot_packet: args.density_hot_packet,
                },
            )?;
            report::print_memory_final_proof_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Consolidate(args) => {
            let report = consolidation::build_consolidation_report();
            report::print_consolidation_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Eval(args) => {
            let report = eval::build_big_eval_report();
            report::print_big_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Query(args) => {
            let report = loader::build_runtime_product_report(args.text);
            report::print_runtime_product_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Train(args) => {
            let report = training::compile_training_corpus(training::TrainingConfig {
                inputs: args.inputs,
                out: args.out,
                vocab_cap: args.vocab_cap,
                transition_cap: args.transition_cap,
                active_chunk_cap: args.active_chunk_cap,
                chunk_tokens: args.chunk_tokens,
                hot_budget_bytes: args.hot_budget_bytes,
                max_file_bytes: args.max_file_bytes,
                extensions: training::parse_extensions(&args.extensions),
            })?;
            report::print_training_compile_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::RustCorpusBuild(args) => {
            let report =
                rust_corpus::build_rust_corpus_report(rust_corpus::RustCorpusBuildConfig {
                    repo: args.repo,
                    out: args.out,
                    max_file_bytes: args.max_file_bytes,
                })?;
            report::print_rust_corpus_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::RustHeldoutBuild(args) => {
            let report =
                rust_heldout::build_rust_heldout_report(rust_heldout::RustHeldoutBuildConfig {
                    artifact: args.artifact,
                    out: args.out,
                    max_cases: args.max_cases,
                })?;
            report::print_rust_heldout_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::RustFocusBuild(args) => {
            let report = rust_focus::build_rust_focus_report(rust_focus::RustFocusBuildConfig {
                artifact: args.artifact,
                heldout_suite: args.heldout_suite,
                out: args.out,
                max_facts: args.max_facts,
                route_fact_cap: args.route_fact_cap,
            })?;
            report::print_rust_focus_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::RustCompileEvidenceBuild(args) => {
            let report = rust_compile_evidence::build_rust_compile_evidence_report(
                rust_compile_evidence::RustCompileEvidenceBuildConfig {
                    focus_packet: args.focus_packet,
                    check_evidence: args.check_evidence,
                    test_evidence: args.test_evidence,
                    clippy_evidence: args.clippy_evidence,
                    out: args.out,
                },
            )?;
            report::print_rust_compile_evidence_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::RustHeldoutEval(args) => {
            let report = rust_heldout_eval::build_rust_heldout_eval_report(
                rust_heldout_eval::RustHeldoutEvalConfig {
                    focus_packet: args.focus_packet,
                    heldout_suite: args.heldout_suite,
                    out: args.out,
                    pass_threshold: args.pass_threshold,
                },
            )?;
            report::print_rust_heldout_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::StrictDensityClaimGate(args) => {
            let report = strict_density_claim_gate::build_strict_density_claim_gate_report(
                strict_density_claim_gate::StrictDensityClaimGateConfig {
                    artifact: args.artifact,
                    focus_packet: args.focus_packet,
                    heldout_eval: args.heldout_eval,
                    compile_evidence: args.compile_evidence,
                    out: args.out,
                },
            )?;
            report::print_strict_density_claim_gate_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::ProfileDensityBuild(args) => {
            let report = profile_density_build::build_profile_density_report(
                profile_density_build::ProfileDensityBuildConfig {
                    profile: args.profile,
                    corpus: args.corpus,
                    out: args.out,
                },
            )?;
            report::print_profile_density_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::MultiProfileDensitySuite(args) => {
            let report = multi_profile_density_suite::build_multi_profile_density_suite_report(
                multi_profile_density_suite::MultiProfileDensitySuiteConfig {
                    rust_density: args.rust_density,
                    profile_evidence: args.profile_evidence,
                    out: args.out,
                    min_profiles: args.min_profiles,
                },
            )?;
            report::print_multi_profile_density_suite_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::DensityProofDoctor(args) => {
            let report = density_proof_doctor::build_density_proof_doctor_report(
                density_proof_doctor::DensityProofDoctorConfig {
                    suite: args.suite,
                    out: args.out,
                    medium_profile_min: args.medium_profile_min,
                    strong_profile_min: args.strong_profile_min,
                    min_fact_count: args.min_fact_count,
                },
            )?;
            report::print_density_proof_doctor_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::DensityAblation(args) => {
            let report = density_ablation::build_density_ablation_report(
                density_ablation::DensityAblationConfig {
                    suite: args.suite,
                    out_hot_packet: args.out_hot_packet,
                },
            )?;
            report::print_density_ablation_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadCorpusBuild(args) => {
            let report =
                broad_eval::build_broad_corpus_artifact(broad_eval::BroadCorpusBuildConfig {
                    source: args.source,
                    profile: args.profile,
                    out: args.out,
                })?;
            report::print_broad_corpus_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadEvalSuiteBuild(args) => {
            let report =
                broad_eval::build_broad_eval_suite(broad_eval::BroadEvalSuiteBuildConfig {
                    corpus: args.corpus,
                    families: args.families,
                    out: args.out,
                })?;
            report::print_broad_eval_suite_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadDatasetDoctor(args) => {
            let report = broad_eval::build_broad_dataset_doctor_report(
                broad_eval::BroadDatasetDoctorConfig {
                    corpus: args.corpus,
                    out: args.out,
                    medium_min_facts: args.medium_min_facts,
                    strong_min_facts: args.strong_min_facts,
                },
            )?;
            report::print_broad_dataset_doctor_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadHeldoutBuild(args) => {
            let report =
                broad_eval::build_broad_heldout_report(broad_eval::BroadHeldoutBuildConfig {
                    corpus: args.corpus,
                    out: args.out,
                    max_cases: args.max_cases,
                })?;
            report::print_broad_heldout_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadFocusBuild(args) => {
            let report = broad_eval::build_broad_focus_report(broad_eval::BroadFocusBuildConfig {
                corpus: args.corpus,
                heldout_suite: args.heldout_suite,
                out: args.out,
                max_facts: args.max_facts,
                route_fact_cap: args.route_fact_cap,
            })?;
            report::print_broad_focus_build_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadEvalRun(args) => {
            let report = broad_eval::build_broad_eval_run_report(broad_eval::BroadEvalRunConfig {
                corpus: args.corpus,
                suite: args.suite,
                focus_packet: args.focus_packet,
                hot_packet: args.hot_packet,
                out: args.out,
            })?;
            report::print_broad_eval_run_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadBaselineDuel(args) => {
            let report = broad_eval::build_broad_baseline_duel_report(
                broad_eval::BroadBaselineDuelConfig {
                    eval_report: args.eval_report,
                    baselines: args.baselines,
                    out: args.out,
                },
            )?;
            report::print_broad_baseline_duel_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::BroadChatLoopEval(args) => {
            let report = broad_eval::build_broad_chat_loop_eval_report(
                broad_eval::BroadChatLoopEvalConfig { out: args.out },
            )?;
            report::print_broad_chat_loop_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LlmwaveReadiness(args) => {
            let report =
                broad_eval::build_llmwave_readiness_report(broad_eval::LlmwaveReadinessConfig {
                    memory_final_proof: args.memory_final_proof,
                    broad_dataset_doctor: args.broad_dataset_doctor,
                    broad_eval: args.broad_eval,
                    baseline_duel: args.baseline_duel,
                    chat_loop: args.chat_loop,
                    out: args.out,
                })?;
            report::print_llmwave_readiness_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::Ask(args) => {
            let report = training::ask_training_artifact(&args.artifact, args.text, args.top_k)?;
            report::print_artifact_ask_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::AskEval(args) => {
            let report = training::eval_training_artifact(&args.artifact, &args.suite, args.top_k)?;
            report::print_artifact_ask_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::PackHot(args) => {
            let report = training::pack_hot_artifact(&args.artifact, &args.out)?;
            report::print_hot_pack_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::AskHot(args) => {
            let report = training::ask_hot_pack(
                &args.hot_pack,
                &args.artifact,
                args.text,
                args.top_k,
                args.memory.as_deref(),
            )?;
            report::print_hot_ask_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LearnHot(args) => {
            let report = training::learn_hot_memory(&args.feedback, &args.out)?;
            report::print_hot_learn_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::ChatHot(args) => {
            let report = training::chat_hot_session(
                &args.hot_pack,
                &args.artifact,
                &args.memory,
                args.script.as_deref(),
                args.top_k,
            )?;
            report::print_hot_chat_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::ChatHotEval(args) => {
            let report = training::eval_hot_chat_session(
                &args.hot_pack,
                &args.artifact,
                &args.memory,
                &args.script,
                args.top_k,
            )?;
            report::print_hot_chat_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::DomainEval(args) => {
            let report = domain_eval::build_domain_eval_report(
                &args.artifact,
                &args.ask_suite,
                &args.hot_pack,
                &args.chat_script,
                &args.chat_memory,
                &args.nonlinear_corpus,
                args.top_k,
            )?;
            report::print_domain_eval_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::DemoDomain(args) => {
            let report = demo_domain::build_demo_domain_report(
                &args.out_dir,
                &args.nonlinear_corpus,
                args.top_k,
            )?;
            report::print_demo_domain_report(&report, &args.format)?;
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxAtlasBuild(args) => {
            let pack_kind = match args.pack_kind.as_str() {
                "base" => linux_atlas::LinuxAtlasPackKind::Base,
                "delta" => linux_atlas::LinuxAtlasPackKind::Delta,
                other => anyhow::bail!("unknown linux atlas pack kind: {other}; use base or delta"),
            };
            let report =
                linux_atlas::build_linux_atlas_report(linux_atlas::LinuxAtlasBuildConfig {
                    root: args.root,
                    out_dir: args.out_dir,
                    pack_kind,
                    max_facts: args.max_facts,
                })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("facts: {}", report.artifact.fact_count);
                    println!("routes: {}", report.artifact.route_count);
                    println!("facts_path: {}", report.outputs.facts_path);
                    println!("current_path: {}", report.outputs.current_path);
                    println!(
                        "active_65k_pack_ready: {}",
                        report.claim_boundary.active_65k_pack_ready
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Atlas");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- facts: `{}`", report.artifact.fact_count);
                    println!("- routes: `{}`", report.artifact.route_count);
                    println!("- append only: `{}`", report.artifact.append_only);
                    println!("- facts path: `{}`", report.outputs.facts_path);
                    println!("- current path: `{}`", report.outputs.current_path);
                    println!("- llm ready: `{}`", report.claim_boundary.llm_ready);
                    println!(
                        "- nonlinear memory proven: `{}`",
                        report.claim_boundary.nonlinear_memory_proven
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxActiveField(args) => {
            let report = linux_active_field::build_linux_active_field_report(
                linux_active_field::LinuxActiveFieldConfig {
                    atlas_dir: args.atlas_dir,
                    max_active_facts: args.max_active_facts,
                    queries: args.query,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("atlas_facts: {}", report.atlas_input.fact_count);
                    println!("selected_facts: {}", report.active_pack.selected_facts);
                    println!(
                        "selected_routes: {}",
                        report.active_pack.selected_route_count
                    );
                    println!(
                        "fits_6m_hot_projection: {}",
                        report.memory_budget.fits_6m_hot_projection
                    );
                    println!(
                        "active_65k_projection_ready: {}",
                        report.claim_boundary.active_65k_projection_ready
                    );
                    if let Some(first) = report.probe_results.first() {
                        println!("first_probe: {} => {}", first.query, first.state);
                    }
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Active Field");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- atlas facts: `{}`", report.atlas_input.fact_count);
                    println!("- selected facts: `{}`", report.active_pack.selected_facts);
                    println!(
                        "- selected routes: `{}`",
                        report.active_pack.selected_route_count
                    );
                    println!(
                        "- fits 6 MiB projection: `{}`",
                        report.memory_budget.fits_6m_hot_projection
                    );
                    println!("- llm ready: `{}`", report.claim_boundary.llm_ready);
                    println!(
                        "- nonlinear memory proven: `{}`",
                        report.claim_boundary.nonlinear_memory_proven
                    );
                    println!();
                    println!("## Probes");
                    for probe in &report.probe_results {
                        println!(
                            "- `{}`: `{}` score `{}`",
                            probe.query, probe.state, probe.top_score
                        );
                    }
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxPackHot(args) => {
            let report = linux_hot_packet::build_linux_hot_pack_report(
                linux_hot_packet::LinuxHotPackConfig {
                    atlas_dir: args.atlas_dir,
                    max_active_facts: args.max_active_facts,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("selected_facts: {}", report.source.selected_facts);
                    println!(
                        "fixed_records_fit_6m: {}",
                        report.packet.fixed_records_fit_6m
                    );
                    println!("hot_loop_bytes: {}", report.packet.hot_loop_record_bytes);
                    println!("cold_label_bytes: {}", report.packet.cold_label_table_bytes);
                    println!("out: {}", report.out);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Hot Packet");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- selected facts: `{}`", report.source.selected_facts);
                    println!(
                        "- fixed records fit 6 MiB: `{}`",
                        report.packet.fixed_records_fit_6m
                    );
                    println!(
                        "- hot-loop bytes: `{}`",
                        report.packet.hot_loop_record_bytes
                    );
                    println!(
                        "- cold label bytes: `{}`",
                        report.packet.cold_label_table_bytes
                    );
                    println!(
                        "- cache-only execution proven: `{}`",
                        report.claim_boundary.cache_only_execution_proven
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxAskHot(args) => {
            let report = linux_hot_packet::build_linux_hot_ask_report(
                linux_hot_packet::LinuxHotAskConfig {
                    hot_pack: args.hot_pack,
                    query: args.query,
                    top_k: args.top_k,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("state: {}", report.field.state);
                    println!("top_score: {}", report.field.top_score);
                    println!("safe_to_answer: {}", report.answer.safe_to_answer);
                    println!("answer: {}", report.answer.text);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Hot Scan");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- state: `{}`", report.field.state);
                    println!("- top score: `{}`", report.field.top_score);
                    println!("- safe to answer: `{}`", report.answer.safe_to_answer);
                    println!("- answer: `{}`", report.answer.text);
                    println!(
                        "- JSON used in hot scan: `{}`",
                        report.claim_boundary.json_used_in_hot_scan
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxHotEval(args) => {
            let report = linux_hot_packet::build_linux_hot_eval_report(
                linux_hot_packet::LinuxHotEvalConfig {
                    hot_pack: args.hot_pack,
                    top_k: args.top_k,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("passed: {}/{}", report.metrics.passed, report.metrics.total);
                    println!(
                        "lexical_duel_wins: {}/{}",
                        report.metrics.lexical_duel_wins, report.metrics.lexical_duel_total
                    );
                    println!(
                        "false_positive_rate: {}",
                        report.metrics.false_positive_rate
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Hot Eval");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- passed: `{}/{}`",
                        report.metrics.passed, report.metrics.total
                    );
                    println!(
                        "- lexical duel wins: `{}/{}`",
                        report.metrics.lexical_duel_wins, report.metrics.lexical_duel_total
                    );
                    println!(
                        "- false positive rate: `{}`",
                        report.metrics.false_positive_rate
                    );
                    println!(
                        "- nonlinear memory proven: `{}`",
                        report.claim_boundary.nonlinear_memory_proven
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxDomainRun(args) => {
            let report = linux_hot_packet::build_linux_domain_run_report(
                linux_hot_packet::LinuxDomainRunConfig {
                    hot_pack: args.hot_pack,
                    query: args.query,
                    top_k: args.top_k,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("route_hint: {}", report.query_wave.route_hint);
                    println!("field_state: {}", report.reasoning_loop.field_state);
                    println!("answer_allowed: {}", report.verifier.answer_allowed);
                    println!("answer: {}", report.answer_surface.text);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Domain Run");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- route hint: `{}`", report.query_wave.route_hint);
                    println!("- field state: `{}`", report.reasoning_loop.field_state);
                    println!("- answer allowed: `{}`", report.verifier.answer_allowed);
                    println!("- answer: `{}`", report.answer_surface.text);
                    println!(
                        "- broad chat ready: `{}`",
                        report.claim_boundary.broad_chat_llm_ready
                    );
                    println!(
                        "- nonlinear memory proven: `{}`",
                        report.claim_boundary.nonlinear_memory_proven
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxAtlasProjection(args) => {
            let report = linux_atlas_projection::build_linux_atlas_projection_report(
                linux_atlas_projection::LinuxAtlasProjectionConfig {
                    atlas_dir: args.atlas_dir,
                    hot_pack: args.hot_pack,
                    residual_pack: args.residual_pack,
                    query: args.query,
                    top_k: args.top_k,
                    iterations: args.iterations,
                    warmup_iterations: args.warmup_iterations,
                    samples: args.samples,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("atlas_bytes: {}", report.atlas.total_bytes);
                    println!("active_facts: {}", report.projection.active_fact_count);
                    println!("laf_file_bytes: {}", report.runtime.laf_file_bytes);
                    println!("lrf_file_bytes: {}", report.runtime.lrf_file_bytes);
                    println!(
                        "lrf_hot_sections_bytes: {}",
                        report.runtime.lrf_hot_sections_bytes
                    );
                    println!(
                        "cache_only_execution_proven: {}",
                        report.claim_boundary.cache_only_execution_proven
                    );
                    println!(
                        "linux_profile_nonlinear_memory_proven: {}",
                        report.claim_boundary.linux_profile_nonlinear_memory_proven
                    );
                    println!(
                        "lossless_atlas_storage_in_6m: {}",
                        report.claim_boundary.atlas_lossless_storage_in_6m
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Atlas -> 6 MiB Projection");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- atlas bytes: `{}`", report.atlas.total_bytes);
                    println!(
                        "- known facts: `{:?}`",
                        report.projection.source_known_fact_count
                    );
                    println!("- active facts: `{}`", report.projection.active_fact_count);
                    println!("- `.laf` bytes: `{}`", report.runtime.laf_file_bytes);
                    println!("- `.lrf` bytes: `{}`", report.runtime.lrf_file_bytes);
                    println!(
                        "- `.lrf` hot sections bytes: `{}`",
                        report.runtime.lrf_hot_sections_bytes
                    );
                    println!(
                        "- cache-only execution proven: `{}`",
                        report.claim_boundary.cache_only_execution_proven
                    );
                    println!(
                        "- Linux-profile nonlinear memory proven: `{}`",
                        report.claim_boundary.linux_profile_nonlinear_memory_proven
                    );
                    println!(
                        "- lossless Atlas storage in 6 MiB: `{}`",
                        report.claim_boundary.atlas_lossless_storage_in_6m
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxCacheProof(args) => {
            let report = linux_hot_packet::build_linux_cache_proof_report(
                linux_hot_packet::LinuxCacheProofConfig {
                    hot_pack: args.hot_pack,
                    query: args.query,
                    iterations: args.iterations,
                    warmup_iterations: args.warmup_iterations,
                    samples: args.samples,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "cache_only_execution_proven: {}",
                        report.claim_boundary.cache_only_execution_proven
                    );
                    println!("records: {}", report.hot_pack.fixed_record_count);
                    println!("hot_loop_bytes: {}", report.hot_pack.hot_loop_record_bytes);
                    println!("ns_per_scan_p50: {}", report.benchmark.ns_per_scan_p50);
                    println!("ns_per_record_p50: {}", report.benchmark.ns_per_record_p50);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Cache Proof");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- cache-only execution proven: `{}`",
                        report.claim_boundary.cache_only_execution_proven
                    );
                    println!("- fixed records: `{}`", report.hot_pack.fixed_record_count);
                    println!(
                        "- hot-loop bytes: `{}`",
                        report.hot_pack.hot_loop_record_bytes
                    );
                    println!("- p50 ns/scan: `{}`", report.benchmark.ns_per_scan_p50);
                    println!("- p50 ns/record: `{}`", report.benchmark.ns_per_record_p50);
                    println!(
                        "- labels read from packet: `{}`",
                        report.runtime_contract.labels_read_from_packet
                    );
                    println!(
                        "- hardware cache counters used: `{}`",
                        report.runtime_contract.hardware_perf_counters_used
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxPmuCacheProof(args) => {
            let report = linux_hot_packet::build_linux_pmu_cache_proof_report(
                linux_hot_packet::LinuxPmuCacheProofConfig {
                    hot_pack: args.hot_pack,
                    query: args.query,
                    iterations: args.iterations,
                    warmup_iterations: args.warmup_iterations,
                    samples: args.samples,
                    max_cache_miss_rate: args.max_cache_miss_rate,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("pmu_status: {}", report.pmu.counter_status);
                    println!(
                        "hardware_perf_counters_used: {}",
                        report.claim_boundary.hardware_perf_counters_used
                    );
                    println!(
                        "hardware_cache_residency_counter_proven: {}",
                        report
                            .claim_boundary
                            .hardware_cache_residency_counter_proven
                    );
                    println!(
                        "software_cache_only_execution_proven: {}",
                        report.claim_boundary.software_cache_only_execution_proven
                    );
                    println!("cache_miss_rate: {:?}", report.pmu.cache_miss_rate);
                    if let Some(reason) = &report.pmu.blocked_reason {
                        println!("blocked_reason: {reason}");
                    }
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux PMU Cache Proof");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- PMU status: `{}`", report.pmu.counter_status);
                    println!(
                        "- hardware counters used: `{}`",
                        report.claim_boundary.hardware_perf_counters_used
                    );
                    println!(
                        "- hardware cache residency proven: `{}`",
                        report
                            .claim_boundary
                            .hardware_cache_residency_counter_proven
                    );
                    println!(
                        "- software cache-only proven: `{}`",
                        report.claim_boundary.software_cache_only_execution_proven
                    );
                    println!("- cache miss rate: `{:?}`", report.pmu.cache_miss_rate);
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxPackResidual(args) => {
            let report = linux_residual_memory::build_linux_residual_pack_report(
                linux_residual_memory::LinuxResidualPackConfig {
                    atlas_dir: args.atlas_dir,
                    max_active_facts: args.max_active_facts,
                    promotion_threshold: args.promotion_threshold,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("selected_facts: {}", report.source.selected_facts);
                    println!("schemas: {}", report.packet.schema_record_count);
                    println!("residuals: {}", report.packet.residual_record_count);
                    println!("fallbacks: {}", report.packet.fallback_record_count);
                    println!(
                        "binary_hot_sections_bytes: {}",
                        report.packet.binary_hot_sections_bytes
                    );
                    println!(
                        "residual_saving_bytes: {}",
                        report.economics.residual_saving_bytes
                    );
                    println!("out: {}", report.out);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Schema/Residual Packet");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- selected facts: `{}`", report.source.selected_facts);
                    println!("- schemas: `{}`", report.packet.schema_record_count);
                    println!("- residuals: `{}`", report.packet.residual_record_count);
                    println!("- fallbacks: `{}`", report.packet.fallback_record_count);
                    println!(
                        "- binary hot sections bytes: `{}`",
                        report.packet.binary_hot_sections_bytes
                    );
                    println!(
                        "- direct fixed64 baseline bytes: `{}`",
                        report.packet.direct_fixed_baseline_bytes
                    );
                    println!(
                        "- residual saving bytes: `{}`",
                        report.economics.residual_saving_bytes
                    );
                    println!(
                        "- nonlinear memory proven: `{}`",
                        report.claim_boundary.nonlinear_memory_proven
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxResidualProof(args) => {
            if let Some(report) =
                linux_residual_memory::linux_residual_repack_required_report(&args.residual_pack)?
            {
                match args.format {
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    OutputFormat::Text => {
                        println!("{}", report.verdict);
                        println!("residual_pack: {}", report.residual_pack);
                        println!("detected_magic: {}", report.detected.magic);
                        println!("required_magic: {}", report.required.magic);
                        println!("required_format: {}", report.required.format_name);
                        println!("safe_to_use: {}", report.safe_to_use);
                        println!("repack_command: {}", report.repack_command);
                    }
                    OutputFormat::Md => {
                        println!("# LLMWave Linux Residual Repack Required");
                        println!();
                        println!("- verdict: `{}`", report.verdict);
                        println!("- residual pack: `{}`", report.residual_pack);
                        println!("- detected magic: `{}`", report.detected.magic);
                        println!("- required magic: `{}`", report.required.magic);
                        println!("- required format: `{}`", report.required.format_name);
                        println!("- safe to use: `{}`", report.safe_to_use);
                        println!();
                        println!("```bash");
                        println!("{}", report.repack_command);
                        println!("```");
                    }
                }
                return Ok(EXIT_WATCH);
            }
            let report = linux_residual_memory::build_linux_residual_proof_report(
                linux_residual_memory::LinuxResidualProofConfig {
                    residual_pack: args.residual_pack,
                    query: args.query,
                    top_k: args.top_k,
                    iterations: args.iterations,
                    warmup_iterations: args.warmup_iterations,
                    samples: args.samples,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "nonlinear_memory_proven: {}",
                        report.claim_boundary.nonlinear_memory_proven
                    );
                    println!("schemas: {}", report.residual_pack.schema_record_count);
                    println!("residuals: {}", report.residual_pack.residual_record_count);
                    println!("fallbacks: {}", report.residual_pack.fallback_record_count);
                    println!(
                        "binary_hot_sections_bytes: {}",
                        report.residual_pack.binary_hot_sections_bytes
                    );
                    println!(
                        "direct_fixed_baseline_bytes: {}",
                        report.residual_pack.direct_fixed_baseline_bytes
                    );
                    println!(
                        "eval_passed: {}/{}",
                        report.eval.metrics.passed, report.eval.metrics.total
                    );
                    println!("ns_per_scan_p50: {}", report.benchmark.ns_per_scan_p50);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Schema/Residual Proof");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- nonlinear memory proven: `{}`",
                        report.claim_boundary.nonlinear_memory_proven
                    );
                    println!(
                        "- Linux-profile nonlinear memory proven: `{}`",
                        report.claim_boundary.linux_profile_nonlinear_memory_proven
                    );
                    println!("- schemas: `{}`", report.residual_pack.schema_record_count);
                    println!(
                        "- residuals: `{}`",
                        report.residual_pack.residual_record_count
                    );
                    println!(
                        "- fallbacks: `{}`",
                        report.residual_pack.fallback_record_count
                    );
                    println!(
                        "- binary hot sections bytes: `{}`",
                        report.residual_pack.binary_hot_sections_bytes
                    );
                    println!(
                        "- direct fixed64 baseline bytes: `{}`",
                        report.residual_pack.direct_fixed_baseline_bytes
                    );
                    println!(
                        "- eval passed: `{}/{}`",
                        report.eval.metrics.passed, report.eval.metrics.total
                    );
                    println!(
                        "- broad chat ready: `{}`",
                        report.claim_boundary.broad_chat_llm_ready
                    );
                    println!(
                        "- exposure ready: `{}`",
                        report.claim_boundary.exposure_layer_ready
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxExposureRun(args) => {
            let report =
                linux_exposure::build_linux_exposure_report(linux_exposure::LinuxExposureConfig {
                    residual_pack: args.residual_pack,
                    max_candidates: args.max_candidates,
                    runtime_snapshot: args.runtime_snapshot,
                })?;
            if let Some(out) = args.out.as_ref() {
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(out, serde_json::to_string_pretty(&report)?)?;
            }
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "exposure_layer_ready: {}",
                        report.claim_boundary.exposure_layer_ready
                    );
                    println!("state: {}", report.exposure_field.state);
                    println!("candidates: {}", report.exposure_field.candidate_count);
                    println!(
                        "external_binding_count: {}",
                        report.exposure_field.external_binding_count
                    );
                    println!(
                        "safe_to_claim_external_exposure: {}",
                        report.exposure_field.safe_to_claim_external_exposure
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Exposure Reasoning");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- exposure layer ready: `{}`",
                        report.claim_boundary.exposure_layer_ready
                    );
                    println!("- state: `{}`", report.exposure_field.state);
                    println!("- candidates: `{}`", report.exposure_field.candidate_count);
                    println!(
                        "- external bindings: `{}`",
                        report.exposure_field.external_binding_count
                    );
                    println!(
                        "- firewall allow facts: `{}`",
                        report.exposure_field.firewall_allow_fact_count
                    );
                    println!(
                        "- broad chat ready: `{}`",
                        report.claim_boundary.broad_chat_llm_ready
                    );
                    println!(
                        "- vulnerability scan ready: `{}`",
                        report.claim_boundary.vulnerability_scan_ready
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxSnapshotImport(args) => {
            let report = linux_runtime_snapshot::build_linux_runtime_snapshot_import_report(
                linux_runtime_snapshot::LinuxRuntimeSnapshotImportConfig {
                    snapshot: args.snapshot,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("facts: {}", report.overlay.fact_count);
                    println!(
                        "firewall_allow_facts: {}",
                        report.overlay.firewall_allow_fact_count
                    );
                    println!("commands_executed: {}", report.overlay.commands_executed);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Runtime Snapshot Import");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- facts: `{}`", report.overlay.fact_count);
                    println!(
                        "- firewall allow facts: `{}`",
                        report.overlay.firewall_allow_fact_count
                    );
                    println!(
                        "- commands executed: `{}`",
                        report.overlay.commands_executed
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::DaybreakDuel(args) => {
            let report =
                daybreak_duel::build_daybreak_duel_report(daybreak_duel::DaybreakDuelConfig {
                    out: args.out,
                })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "passed: {}/{}",
                        report.scoreboard.passed, report.scoreboard.total
                    );
                    println!("blocked: {}", report.scoreboard.blocked);
                    println!(
                        "daybreak_competitive: {}",
                        report.claim_boundary.daybreak_competitive
                    );
                    println!(
                        "patch_generation_ready: {}",
                        report.claim_boundary.patch_generation_ready
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Daybreak Duel");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- passed: `{}/{}`",
                        report.scoreboard.passed, report.scoreboard.total
                    );
                    println!("- blocked: `{}`", report.scoreboard.blocked);
                    println!(
                        "- daybreak competitive: `{}`",
                        report.claim_boundary.daybreak_competitive
                    );
                    println!(
                        "- patch generation ready: `{}`",
                        report.claim_boundary.patch_generation_ready
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::SecurityFixtureRun(args) => {
            let report = security_fixture::build_security_fixture_run_report(
                security_fixture::SecurityFixtureRunConfig { out: args.out },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("finding: {}", report.finding.class);
                    println!("route: {}", report.finding.vulnerable_route);
                    println!(
                        "before_escape_reaches_secret: {}",
                        report.result.before_exploit_reaches_forbidden_path
                    );
                    println!(
                        "after_escape_blocked: {}",
                        report.result.after_forbidden_path_blocked
                    );
                    println!(
                        "normal_file_still_reads: {}",
                        report.result.regression_normal_file_still_reads
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Security Fixture Run");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- finding: `{}`", report.finding.class);
                    println!("- route: `{}`", report.finding.vulnerable_route);
                    println!(
                        "- before escape reaches secret: `{}`",
                        report.result.before_exploit_reaches_forbidden_path
                    );
                    println!(
                        "- after escape blocked: `{}`",
                        report.result.after_forbidden_path_blocked
                    );
                    println!(
                        "- normal file still reads: `{}`",
                        report.result.regression_normal_file_still_reads
                    );
                    println!("- patch id: `{}`", report.patch_candidate.patch_id);
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatRun(args) => {
            let report = linux_chat::build_linux_chat_report(linux_chat::LinuxChatConfig {
                residual_pack: args.residual_pack,
                prompt: args.prompt,
                max_facts: args.max_facts,
            })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "linux_profile_broad_chat_ready: {}",
                        report.claim_boundary.linux_profile_broad_chat_ready
                    );
                    println!(
                        "broad_chat_llm_ready: {}",
                        report.claim_boundary.broad_chat_llm_ready
                    );
                    println!(
                        "general_llm_ready: {}",
                        report.claim_boundary.general_llm_ready
                    );
                    println!(
                        "eval_passed: {}/{}",
                        report.eval.metrics.passed, report.eval.metrics.total
                    );
                    println!("turns: {}", report.turns.len());
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Profile Chat");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- Linux-profile broad chat ready: `{}`",
                        report.claim_boundary.linux_profile_broad_chat_ready
                    );
                    println!(
                        "- broad chat ready: `{}`",
                        report.claim_boundary.broad_chat_llm_ready
                    );
                    println!(
                        "- general LLM ready: `{}`",
                        report.claim_boundary.general_llm_ready
                    );
                    println!(
                        "- eval passed: `{}/{}`",
                        report.eval.metrics.passed, report.eval.metrics.total
                    );
                    println!(
                        "- false positive rate: `{}`",
                        report.eval.metrics.false_positive_rate
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatV1(args) => {
            let report =
                linux_chat_v1::build_linux_chat_v1_report(linux_chat_v1::LinuxChatV1Config {
                    residual_pack: args.residual_pack,
                    prompt: args.prompt,
                    script: args.script,
                    max_facts: args.max_facts,
                    runtime_snapshot: args.runtime_snapshot,
                })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "linux_chat_v1_ready: {}",
                        report.claim_boundary.linux_chat_v1_ready
                    );
                    println!(
                        "general_llm_ready: {}",
                        report.claim_boundary.general_llm_ready
                    );
                    println!("eval_passed: {}/{}", report.eval.passed, report.eval.total);
                    println!(
                        "shortcut_rejection_rate: {}",
                        report.eval.shortcut_rejection_rate
                    );
                    println!("turns: {}", report.turn_count);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Chat V1");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- Linux Chat V1 ready: `{}`",
                        report.claim_boundary.linux_chat_v1_ready
                    );
                    println!(
                        "- general LLM ready: `{}`",
                        report.claim_boundary.general_llm_ready
                    );
                    println!(
                        "- eval passed: `{}/{}`",
                        report.eval.passed, report.eval.total
                    );
                    println!(
                        "- shortcut rejection rate: `{}`",
                        report.eval.shortcut_rejection_rate
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatV1Eval(args) => {
            let report =
                linux_chat_v1::build_linux_chat_v1_report(linux_chat_v1::LinuxChatV1Config {
                    residual_pack: args.residual_pack,
                    prompt: Vec::new(),
                    script: None,
                    max_facts: args.max_facts,
                    runtime_snapshot: args.runtime_snapshot,
                })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "linux_chat_v1_ready: {}",
                        report.claim_boundary.linux_chat_v1_ready
                    );
                    println!(
                        "general_llm_ready: {}",
                        report.claim_boundary.general_llm_ready
                    );
                    println!("eval_passed: {}/{}", report.eval.passed, report.eval.total);
                    println!(
                        "context_retention_rate: {}",
                        report.eval.context_retention_rate
                    );
                    println!("correction_pass_rate: {}", report.eval.correction_pass_rate);
                    println!(
                        "shortcut_rejection_rate: {}",
                        report.eval.shortcut_rejection_rate
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Chat V1 Eval");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- Linux Chat V1 ready: `{}`",
                        report.claim_boundary.linux_chat_v1_ready
                    );
                    println!(
                        "- eval passed: `{}/{}`",
                        report.eval.passed, report.eval.total
                    );
                    println!(
                        "- context retention rate: `{}`",
                        report.eval.context_retention_rate
                    );
                    println!(
                        "- correction pass rate: `{}`",
                        report.eval.correction_pass_rate
                    );
                    println!(
                        "- shortcut rejection rate: `{}`",
                        report.eval.shortcut_rejection_rate
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatV2(args) => {
            let report =
                linux_chat_v2::build_linux_chat_v2_report(linux_chat_v2::LinuxChatV2Config {
                    residual_pack: args.residual_pack,
                    memory: args.memory,
                    prompt: args.prompt,
                    script: args.script,
                    max_facts: args.max_facts,
                    runtime_snapshot: args.runtime_snapshot,
                    reset_memory: args.reset_memory,
                })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "dialogue_learning_ready: {}",
                        report.claim_boundary.dialogue_learning_ready
                    );
                    println!(
                        "persistent_wave_memory_ready: {}",
                        report.claim_boundary.persistent_wave_memory_ready
                    );
                    println!(
                        "general_llm_ready: {}",
                        report.claim_boundary.general_llm_ready
                    );
                    println!("eval_passed: {}/{}", report.eval.passed, report.eval.total);
                    println!("deltas_written: {}", report.eval.deltas_written);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Chat V2");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- dialogue learning ready: `{}`",
                        report.claim_boundary.dialogue_learning_ready
                    );
                    println!(
                        "- persistent wave memory ready: `{}`",
                        report.claim_boundary.persistent_wave_memory_ready
                    );
                    println!(
                        "- eval passed: `{}/{}`",
                        report.eval.passed, report.eval.total
                    );
                    println!("- deltas written: `{}`", report.eval.deltas_written);
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatV2Eval(args) => {
            let report =
                linux_chat_v2::build_linux_chat_v2_report(linux_chat_v2::LinuxChatV2Config {
                    residual_pack: args.residual_pack,
                    memory: args.memory,
                    prompt: Vec::new(),
                    script: None,
                    max_facts: args.max_facts,
                    runtime_snapshot: args.runtime_snapshot,
                    reset_memory: true,
                })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "dialogue_learning_ready: {}",
                        report.claim_boundary.dialogue_learning_ready
                    );
                    println!("memory_lift_observed: {}", report.eval.memory_lift_observed);
                    println!(
                        "negative_lane_replay_observed: {}",
                        report.eval.negative_lane_replay_observed
                    );
                    println!("eval_passed: {}/{}", report.eval.passed, report.eval.total);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Chat V2 Eval");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- dialogue learning ready: `{}`",
                        report.claim_boundary.dialogue_learning_ready
                    );
                    println!(
                        "- memory lift observed: `{}`",
                        report.eval.memory_lift_observed
                    );
                    println!(
                        "- negative lane replay observed: `{}`",
                        report.eval.negative_lane_replay_observed
                    );
                    println!(
                        "- eval passed: `{}/{}`",
                        report.eval.passed, report.eval.total
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxCenterLearn(args) => {
            let report = linux_center_learning::build_linux_center_learn_report(
                linux_center_learning::LinuxCenterLearnConfig {
                    residual_pack: args.residual_pack,
                    memory: args.memory,
                    script: args.script,
                    heldout_eval: args.heldout_eval,
                    max_facts: args.max_facts,
                    reset_memory: args.reset_memory,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "linux_profile_dynamic_learning_ready: {}",
                        report.claim_boundary.linux_profile_dynamic_learning_ready
                    );
                    println!(
                        "target_query_improved: {}",
                        report
                            .dynamic_center_learning
                            .before_after
                            .target_query_improved
                    );
                    println!(
                        "memory_lift_observed: {}",
                        report
                            .dynamic_center_learning
                            .before_after
                            .memory_lift_observed
                    );
                    println!(
                        "anti_center_replay_observed: {}",
                        report
                            .dynamic_center_learning
                            .before_after
                            .anti_center_replay_observed
                    );
                    println!(
                        "general_llm_ready: {}",
                        report.claim_boundary.general_llm_ready
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Center Learn");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- Linux-profile dynamic learning ready: `{}`",
                        report.claim_boundary.linux_profile_dynamic_learning_ready
                    );
                    println!(
                        "- target query improved: `{}`",
                        report
                            .dynamic_center_learning
                            .before_after
                            .target_query_improved
                    );
                    println!(
                        "- memory lift observed: `{}`",
                        report
                            .dynamic_center_learning
                            .before_after
                            .memory_lift_observed
                    );
                    println!(
                        "- anti-center replay observed: `{}`",
                        report
                            .dynamic_center_learning
                            .before_after
                            .anti_center_replay_observed
                    );
                    println!(
                        "- general LLM ready: `{}`",
                        report.claim_boundary.general_llm_ready
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxVpnTrain(args) => {
            let report = linux_vpn_training::build_linux_vpn_train_report(
                linux_vpn_training::LinuxVpnTrainConfig {
                    memory: args.memory,
                    reset_memory: args.reset_memory,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "local_vpn_training_ready: {}",
                        report.claim_boundary.local_vpn_training_ready
                    );
                    println!("inserted_delta_count: {}", report.inserted_delta_count);
                    println!(
                        "local_system_mutation_done: {}",
                        report.claim_boundary.local_system_mutation_done
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux VPN Train");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- local VPN training ready: `{}`",
                        report.claim_boundary.local_vpn_training_ready
                    );
                    println!("- inserted deltas: `{}`", report.inserted_delta_count);
                    println!(
                        "- local system mutation done: `{}`",
                        report.claim_boundary.local_system_mutation_done
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxVpnTrainEval(args) => {
            let report = linux_vpn_training::build_linux_vpn_train_eval_report(
                linux_vpn_training::LinuxVpnTrainEvalConfig {
                    residual_pack: args.residual_pack,
                    memory: args.memory,
                    max_facts: args.max_facts,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "local_vpn_training_ready: {}",
                        report.claim_boundary.local_vpn_training_ready
                    );
                    println!("eval_passed: {}/{}", report.eval.passed, report.eval.total);
                    println!(
                        "local_system_mutation_done: {}",
                        report.claim_boundary.local_system_mutation_done
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux VPN Train Eval");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- local VPN training ready: `{}`",
                        report.claim_boundary.local_vpn_training_ready
                    );
                    println!(
                        "- eval passed: `{}/{}`",
                        report.eval.passed, report.eval.total
                    );
                    println!(
                        "- local system mutation done: `{}`",
                        report.claim_boundary.local_system_mutation_done
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxVpnActionPlan(args) => {
            let report = linux_vpn_control::build_linux_vpn_action_plan_report(
                linux_vpn_control::LinuxVpnActionPlanConfig {
                    text: args.text,
                    action: args.action,
                    backend: args.backend,
                    target: args.target,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("action: {}", report.action);
                    println!("backend: {}", report.backend);
                    println!("target: {}", report.target);
                    println!("command: {}", report.plan.command_preview);
                    println!(
                        "local_system_mutation_done: {}",
                        report.claim_boundary.local_system_mutation_done
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux VPN Action Plan");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- action: `{}`", report.action);
                    println!("- backend: `{}`", report.backend);
                    println!("- target: `{}`", report.target);
                    println!("- command: `{}`", report.plan.command_preview);
                    println!(
                        "- local system mutation done: `{}`",
                        report.claim_boundary.local_system_mutation_done
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxVpnControl(args) => {
            let report = linux_vpn_control::build_linux_vpn_control_report(
                linux_vpn_control::LinuxVpnControlConfig {
                    action: args.action,
                    backend: args.backend,
                    target: args.target,
                    execute: args.execute,
                    i_understand_network_may_drop: args.i_understand_network_may_drop,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("action: {}", report.action);
                    println!("backend: {}", report.backend);
                    println!("target: {}", report.target);
                    println!("command: {}", report.plan.command_preview);
                    println!("executed: {}", report.execution.executed);
                    println!(
                        "local_system_mutation_done: {}",
                        report.claim_boundary.local_system_mutation_done
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux VPN Control");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- action: `{}`", report.action);
                    println!("- backend: `{}`", report.backend);
                    println!("- target: `{}`", report.target);
                    println!("- command: `{}`", report.plan.command_preview);
                    println!("- executed: `{}`", report.execution.executed);
                    println!(
                        "- local system mutation done: `{}`",
                        report.claim_boundary.local_system_mutation_done
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxQueryWave(args) => {
            let report =
                linux_profile::build_linux_query_wave_report(linux_profile::LinuxQueryWaveConfig {
                    text: args.text,
                });
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("intent: {}", report.query_wave.intent);
                    println!("anchors: {}", report.query_wave.anchors.join(","));
                    println!("answer_policy: {}", report.query_wave.answer_policy);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Query Wave");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- intent: `{}`", report.query_wave.intent);
                    println!("- anchors: `{}`", report.query_wave.anchors.join(", "));
                    println!(
                        "- forbidden shortcuts: `{}`",
                        report.query_wave.forbidden_shortcuts.join(", ")
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxBroadSuiteBuild(args) => {
            let report = linux_profile::build_linux_broad_suite_report(
                linux_profile::LinuxBroadSuiteBuildConfig {
                    residual_pack: args.residual_pack,
                    cases: args.cases,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("cases: {}", report.suite.case_count);
                    println!("families: {}", report.suite.families.len());
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Broad Suite");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- cases: `{}`", report.suite.case_count);
                    println!("- families: `{}`", report.suite.families.len());
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxBroadEvalRun(args) => {
            let report = linux_profile::build_linux_broad_eval_report(
                linux_profile::LinuxBroadEvalRunConfig {
                    residual_pack: args.residual_pack,
                    suite: args.suite,
                    out: args.out,
                    max_facts: args.max_facts,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("passed: {}/{}", report.metrics.passed, report.metrics.total);
                    println!(
                        "false_positive_rate: {}",
                        report.metrics.false_positive_rate
                    );
                    println!(
                        "shortcut_rejection_rate: {}",
                        report.metrics.shortcut_rejection_rate
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Broad Eval");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- passed: `{}/{}`",
                        report.metrics.passed, report.metrics.total
                    );
                    println!("- pass rate: `{}`", report.metrics.pass_rate);
                    println!(
                        "- exposure overclaim rate: `{}`",
                        report.metrics.exposure_overclaim_rate
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxReasonRun(args) => {
            let report =
                linux_profile::build_linux_reason_report(linux_profile::LinuxReasonRunConfig {
                    residual_pack: args.residual_pack,
                    text: args.text,
                    max_facts: args.max_facts,
                    runtime_snapshot: args.runtime_snapshot,
                })?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("intent: {}", report.query_wave.intent);
                    println!("state: {}", report.decision.state);
                    println!("answer_allowed: {}", report.decision.answer_allowed);
                    println!("answer: {}", report.decision.answer);
                    println!("evidence_steps: {}", report.evidence_chain.len());
                    println!("anti_wave_hits: {}", report.anti_wave_hits.len());
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Reason Run");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- intent: `{}`", report.query_wave.intent);
                    println!("- state: `{}`", report.decision.state);
                    println!("- answer allowed: `{}`", report.decision.answer_allowed);
                    println!("- answer: {}", report.decision.answer);
                    println!("- evidence steps: `{}`", report.evidence_chain.len());
                    println!("- anti-wave hits: `{}`", report.anti_wave_hits.len());
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxProfileClaimGate(args) => {
            let report = linux_profile::build_linux_profile_claim_gate_report(
                linux_profile::LinuxProfileClaimGateConfig {
                    residual_pack: args.residual_pack,
                    broad_eval: args.broad_eval,
                    heldout_eval: args.heldout_eval,
                    run_chat_learning_eval: args.run_chat_learning_eval,
                    chat_learning_memory: args.chat_learning_memory,
                    run_center_learning_eval: args.run_center_learning_eval,
                    center_learning_memory: args.center_learning_memory,
                    center_learning_script: args.center_learning_script,
                    run_vpn_training_eval: args.run_vpn_training_eval,
                    vpn_memory: args.vpn_memory,
                    max_facts: args.max_facts,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "linux_profile_reasoning_ready: {}",
                        report.claim_boundary.linux_profile_reasoning_ready
                    );
                    println!(
                        "linux_profile_broad_chat_ready: {}",
                        report.claim_boundary.linux_profile_broad_chat_ready
                    );
                    println!("chat_target_ready: {}", report.chat_target.ready);
                    println!("chat_target_verdict: {}", report.chat_target.verdict);
                    println!(
                        "general_llm_ready: {}",
                        report.claim_boundary.general_llm_ready
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Profile Claim Gate");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- Linux-profile reasoning ready: `{}`",
                        report.claim_boundary.linux_profile_reasoning_ready
                    );
                    println!(
                        "- Linux-profile broad chat ready: `{}`",
                        report.claim_boundary.linux_profile_broad_chat_ready
                    );
                    println!("- chat target ready: `{}`", report.chat_target.ready);
                    println!("- chat target verdict: `{}`", report.chat_target.verdict);
                    println!(
                        "- general LLM ready: `{}`",
                        report.claim_boundary.general_llm_ready
                    );
                    println!("- safe claim: {}", report.claim_boundary.safe_claim);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatCoreBuild(args) => {
            let paths = resolve_linux_chat_core_paths(
                args.memory_root,
                LinuxChatCorePathOverrides {
                    profile: args.profile,
                    residual_pack: args.residual_pack,
                    dialogue_overlay: args.dialogue_overlay,
                    centers_overlay: args.centers_overlay,
                    vpn_overlay: args.vpn_overlay,
                    broad_eval: args.broad_eval,
                    heldout_eval: args.heldout_eval,
                    cache_dir: args.cache_dir,
                },
            );
            let report = linux_chat_core::build_linux_chat_core_cache_report(
                linux_chat_core::LinuxChatCoreBuildConfig {
                    profile: paths.profile,
                    residual_pack: paths.residual_pack,
                    dialogue_overlay: paths.dialogue_overlay,
                    centers_overlay: paths.centers_overlay,
                    vpn_overlay: paths.vpn_overlay,
                    broad_eval: paths.broad_eval,
                    heldout_eval: paths.heldout_eval,
                    cache_dir: paths.cache_dir,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("profile_id: {}", report.spec.profile_id);
                    println!("hot_format: {}", report.cache.hot_format);
                    println!("hot_path: {}", report.cache.hot_path);
                    println!("hot_bytes: {}", report.cache.hot_bytes);
                    println!("hot_fits_6m_budget: {}", report.cache.hot_fits_6m_budget);
                    println!("hot_bytes_per_fact: {:.2}", report.cache.hot_bytes_per_fact);
                    println!(
                        "hot_readout_record_count: {}",
                        report.cache.hot_readout_record_count
                    );
                    println!(
                        "hot_domain_record_count: {}",
                        report.cache.hot_domain_record_count
                    );
                    println!("index_path: {}", report.cache.index_path);
                    println!(
                        "json_index_required_for_answer_authority: {}",
                        report.cache.json_index_required_for_answer_authority
                    );
                    println!("manifest_path: {}", report.cache.manifest_path);
                    println!(
                        "cache_is_source_of_truth: {}",
                        report.claim_boundary.cache_is_source_of_truth
                    );
                    println!(
                        "source_estimated_tokens: {}",
                        report.token_economics.source_artifacts_estimated_tokens
                    );
                    println!(
                        "cache_estimated_tokens: {}",
                        report.token_economics.cache_estimated_tokens
                    );
                    println!(
                        "cache_is_runtime_index_not_prompt_payload: {}",
                        report
                            .token_economics
                            .cache_is_runtime_index_not_prompt_payload
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux ChatCore Build");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- profile: `{}`", report.spec.profile_id);
                    println!("- hot format: `{}`", report.cache.hot_format);
                    println!("- hot cache: `{}`", report.cache.hot_path);
                    println!("- hot bytes: `{}`", report.cache.hot_bytes);
                    println!(
                        "- hot fits 6 MiB budget: `{}`",
                        report.cache.hot_fits_6m_budget
                    );
                    println!(
                        "- hot bytes per fact: `{:.2}`",
                        report.cache.hot_bytes_per_fact
                    );
                    println!(
                        "- hot readout records: `{}`",
                        report.cache.hot_readout_record_count
                    );
                    println!(
                        "- JSON index required for answer authority: `{}`",
                        report.cache.json_index_required_for_answer_authority
                    );
                    println!("- index: `{}`", report.cache.index_path);
                    println!("- manifest: `{}`", report.cache.manifest_path);
                    println!(
                        "- cache is source of truth: `{}`",
                        report.claim_boundary.cache_is_source_of_truth
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatCoreGate(args) => {
            let paths = resolve_linux_chat_core_paths(
                args.memory_root,
                LinuxChatCorePathOverrides {
                    profile: args.profile,
                    residual_pack: args.residual_pack,
                    dialogue_overlay: args.dialogue_overlay,
                    centers_overlay: args.centers_overlay,
                    vpn_overlay: args.vpn_overlay,
                    broad_eval: args.broad_eval,
                    heldout_eval: args.heldout_eval,
                    cache_dir: args.cache_dir,
                },
            );
            let report = linux_chat_core::build_linux_chat_core_gate_report(
                linux_chat_core::LinuxChatCoreGateConfig {
                    profile: paths.profile,
                    residual_pack: paths.residual_pack,
                    dialogue_overlay: paths.dialogue_overlay,
                    centers_overlay: paths.centers_overlay,
                    vpn_overlay: paths.vpn_overlay,
                    broad_eval: paths.broad_eval,
                    heldout_eval: paths.heldout_eval,
                    center_learning_script: args.center_learning_script,
                    cache_dir: paths.cache_dir,
                    manifest: args.manifest,
                    rebuild_cache: args.rebuild_cache,
                    max_facts: args.max_facts,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("cache_fresh: {}", report.cache_status.cache_fresh);
                    println!("hot_present: {}", report.cache_status.hot_present);
                    println!("debug_index_present: {}", report.cache_status.index_present);
                    println!("safe_to_use_cache: {}", report.chat_core.safe_to_use_cache);
                    println!(
                        "safe_to_answer_from_cache: {}",
                        report.chat_core.safe_to_answer_from_cache
                    );
                    println!("stale_reasons: {}", report.cache_status.stale_reasons.len());
                    println!(
                        "source_estimated_tokens: {}",
                        report.token_economics.source_artifacts_estimated_tokens
                    );
                    println!(
                        "cache_estimated_tokens: {}",
                        report.token_economics.cache_estimated_tokens
                    );
                    println!(
                        "cache_is_runtime_index_not_prompt_payload: {}",
                        report
                            .token_economics
                            .cache_is_runtime_index_not_prompt_payload
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux ChatCore Gate");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- cache fresh: `{}`", report.cache_status.cache_fresh);
                    println!("- hot present: `{}`", report.cache_status.hot_present);
                    println!(
                        "- debug index present: `{}`",
                        report.cache_status.index_present
                    );
                    println!(
                        "- safe to answer from cache: `{}`",
                        report.chat_core.safe_to_answer_from_cache
                    );
                    println!(
                        "- stale reasons: `{}`",
                        report.cache_status.stale_reasons.len()
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatCoreProfileGate(args) => {
            let paths = resolve_linux_chat_core_paths(
                args.memory_root,
                LinuxChatCorePathOverrides {
                    profile: args.profile,
                    residual_pack: args.residual_pack,
                    dialogue_overlay: args.dialogue_overlay,
                    centers_overlay: args.centers_overlay,
                    vpn_overlay: args.vpn_overlay,
                    broad_eval: args.broad_eval,
                    heldout_eval: args.heldout_eval,
                    cache_dir: args.cache_dir,
                },
            );
            let report = linux_chat_core::build_linux_chat_core_profile_gate_report(
                linux_chat_core::LinuxChatCoreGateConfig {
                    profile: paths.profile,
                    residual_pack: paths.residual_pack,
                    dialogue_overlay: paths.dialogue_overlay,
                    centers_overlay: paths.centers_overlay,
                    vpn_overlay: paths.vpn_overlay,
                    broad_eval: paths.broad_eval,
                    heldout_eval: paths.heldout_eval,
                    center_learning_script: args.center_learning_script,
                    cache_dir: paths.cache_dir,
                    manifest: args.manifest,
                    rebuild_cache: args.rebuild_cache,
                    max_facts: args.max_facts,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("cache_fresh: {}", report.cache_status.cache_fresh);
                    println!("hot_present: {}", report.cache_status.hot_present);
                    println!("debug_index_present: {}", report.cache_status.index_present);
                    println!("safe_to_use_cache: {}", report.chat_core.safe_to_use_cache);
                    println!(
                        "safe_to_answer_from_cache: {}",
                        report.chat_core.safe_to_answer_from_cache
                    );
                    println!(
                        "profile_gate_ready: {}",
                        report.chat_core.profile_gate_ready
                    );
                    println!("stale_reasons: {}", report.cache_status.stale_reasons.len());
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux ChatCore Profile Gate");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- cache fresh: `{}`", report.cache_status.cache_fresh);
                    println!("- hot present: `{}`", report.cache_status.hot_present);
                    println!(
                        "- profile gate ready: `{}`",
                        report.chat_core.profile_gate_ready
                    );
                    println!(
                        "- safe to answer from cache: `{}`",
                        report.chat_core.safe_to_answer_from_cache
                    );
                    println!(
                        "- stale reasons: `{}`",
                        report.cache_status.stale_reasons.len()
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatCoreAsk(args) => {
            let paths = resolve_linux_chat_core_paths(
                args.memory_root,
                LinuxChatCorePathOverrides {
                    profile: None,
                    residual_pack: args.residual_pack,
                    dialogue_overlay: args.dialogue_overlay,
                    centers_overlay: args.centers_overlay,
                    vpn_overlay: args.vpn_overlay,
                    broad_eval: None,
                    heldout_eval: None,
                    cache_dir: args.cache_dir,
                },
            );
            let report = linux_chat_core::build_linux_chat_core_ask_report(
                linux_chat_core::LinuxChatCoreAskConfig {
                    text: args.text,
                    residual_pack: paths.residual_pack,
                    dialogue_overlay: paths.dialogue_overlay,
                    centers_overlay: paths.centers_overlay,
                    vpn_overlay: paths.vpn_overlay,
                    cache_dir: paths.cache_dir,
                    manifest: args.manifest,
                    max_facts: args.max_facts,
                    packet_profile: args.packet_profile,
                    target_packet_tokens: args.target_packet_tokens,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("cache_fresh: {}", report.cache_status.cache_fresh);
                    println!("answer_allowed: {}", report.grounded_packet.answer_allowed);
                    println!("readout_source: {}", report.grounded_packet.readout_source);
                    println!(
                        "domain_suites: {}",
                        report.grounded_packet.domain_suites.join(",")
                    );
                    println!("state: {}", report.grounded_packet.decision_state);
                    println!("answer: {}", report.grounded_packet.answer);
                    println!(
                        "grounded_packet_estimated_tokens: {}",
                        report.token_economics.grounded_packet_estimated_tokens
                    );
                    println!("packet_profile: {}", report.grounded_packet.packet_profile);
                    println!(
                        "inferred_packet_profile: {}",
                        report.grounded_packet.inferred_packet_profile
                    );
                    println!(
                        "requested_packet_profile: {}",
                        report
                            .grounded_packet
                            .requested_packet_profile
                            .as_deref()
                            .unwrap_or("none")
                    );
                    println!(
                        "packet_profile_downgrade_blocked: {}",
                        report.grounded_packet.packet_profile_downgrade_blocked
                    );
                    println!(
                        "packet_budget_tokens: {}",
                        report.grounded_packet.packet_budget_tokens
                    );
                    println!("packet_tokens: {}", report.grounded_packet.packet_tokens);
                    println!(
                        "packet_semantic_tokens: {}",
                        report.grounded_packet.packet_semantic_tokens
                    );
                    println!(
                        "packet_prompt_payload_tokens: {}",
                        report.grounded_packet.packet_prompt_payload_tokens
                    );
                    println!(
                        "packet_over_budget: {}",
                        report.grounded_packet.packet_over_budget
                    );
                    println!(
                        "packet_underfilled: {}",
                        report.grounded_packet.packet_underfilled
                    );
                    println!(
                        "packet_truncated: {}",
                        report.grounded_packet.packet_truncated
                    );
                    println!(
                        "actual_answer_context_estimated_tokens: {}",
                        report
                            .token_economics
                            .actual_answer_context_estimated_tokens
                    );
                    println!(
                        "estimated_tokens_saved_vs_source: {}",
                        report.token_economics.estimated_tokens_saved_vs_source
                    );
                    println!(
                        "cache_is_runtime_index_not_prompt_payload: {}",
                        report
                            .token_economics
                            .cache_is_runtime_index_not_prompt_payload
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux ChatCore Ask");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- cache fresh: `{}`", report.cache_status.cache_fresh);
                    println!(
                        "- answer allowed: `{}`",
                        report.grounded_packet.answer_allowed
                    );
                    println!(
                        "- readout source: `{}`",
                        report.grounded_packet.readout_source
                    );
                    println!(
                        "- packet profile: `{}`",
                        report.grounded_packet.packet_profile
                    );
                    println!(
                        "- inferred/requested profile: `{}` / `{}`",
                        report.grounded_packet.inferred_packet_profile,
                        report
                            .grounded_packet
                            .requested_packet_profile
                            .as_deref()
                            .unwrap_or("none")
                    );
                    println!(
                        "- packet semantic tokens: `{}` / `{}`",
                        report.grounded_packet.packet_semantic_tokens,
                        report.grounded_packet.packet_budget_tokens
                    );
                    println!(
                        "- packet prompt payload tokens: `{}`",
                        report.grounded_packet.packet_prompt_payload_tokens
                    );
                    println!(
                        "- packet over budget: `{}`",
                        report.grounded_packet.packet_over_budget
                    );
                    println!("- state: `{}`", report.grounded_packet.decision_state);
                    println!("- answer: {}", report.grounded_packet.answer);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatCoreLearn(args) => {
            let paths = resolve_linux_chat_core_paths(
                args.memory_root,
                LinuxChatCorePathOverrides {
                    profile: args.profile,
                    residual_pack: args.residual_pack,
                    dialogue_overlay: args.dialogue_overlay,
                    centers_overlay: args.centers_overlay,
                    vpn_overlay: args.vpn_overlay,
                    broad_eval: args.broad_eval,
                    heldout_eval: args.heldout_eval,
                    cache_dir: args.cache_dir,
                },
            );
            let report = linux_chat_core::build_linux_chat_core_learn_report(
                linux_chat_core::LinuxChatCoreLearnConfig {
                    profile: paths.profile,
                    residual_pack: paths.residual_pack,
                    dialogue_overlay: paths.dialogue_overlay,
                    centers_overlay: paths.centers_overlay,
                    vpn_overlay: paths.vpn_overlay,
                    broad_eval: paths.broad_eval,
                    heldout_eval: paths.heldout_eval,
                    cache_dir: paths.cache_dir,
                    accept: args.accept,
                    reject: args.reject,
                    domain: args.domain,
                    overlay: args.overlay,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "overlay_written: {}",
                        report.learning_update.overlay_written
                    );
                    println!(
                        "cache_marked_stale: {}",
                        report.learning_update.cache_marked_stale
                    );
                    println!(
                        "hot_not_mutated_directly: {}",
                        report.learning_update.hot_not_mutated_directly
                    );
                    println!(
                        "selected_overlay: {}",
                        report
                            .selected_overlay
                            .as_ref()
                            .map(|overlay| overlay.overlay_id.as_str())
                            .unwrap_or("none")
                    );
                    println!(
                        "route: {}",
                        report
                            .learned_delta
                            .as_ref()
                            .map(|delta| delta.route.as_str())
                            .unwrap_or("none")
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux ChatCore Learn");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- overlay written: `{}`",
                        report.learning_update.overlay_written
                    );
                    println!(
                        "- cache marked stale: `{}`",
                        report.learning_update.cache_marked_stale
                    );
                    println!(
                        "- hot mutated directly: `{}`",
                        !report.learning_update.hot_not_mutated_directly
                    );
                    println!(
                        "- overlay: `{}`",
                        report
                            .selected_overlay
                            .as_ref()
                            .map(|overlay| overlay.overlay_id.as_str())
                            .unwrap_or("none")
                    );
                    println!(
                        "- route: `{}`",
                        report
                            .learned_delta
                            .as_ref()
                            .map(|delta| delta.route.as_str())
                            .unwrap_or("none")
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxChatCoreLearnEval(args) => {
            let paths = resolve_linux_chat_core_paths(
                args.memory_root,
                LinuxChatCorePathOverrides {
                    profile: args.profile,
                    residual_pack: args.residual_pack,
                    dialogue_overlay: None,
                    centers_overlay: None,
                    vpn_overlay: None,
                    broad_eval: None,
                    heldout_eval: None,
                    cache_dir: args.cache_dir,
                },
            );
            let report = linux_chat_core::build_linux_chat_core_learn_eval_report(
                linux_chat_core::LinuxChatCoreLearnEvalConfig {
                    profile: paths.profile,
                    residual_pack: paths.residual_pack,
                    cache_dir: paths.cache_dir,
                    reset_scratch: args.reset_scratch,
                    max_facts: args.max_facts,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("before_blocked: {}", report.learning_loop.before_blocked);
                    println!("overlay_written: {}", report.learning_loop.overlay_written);
                    println!(
                        "cache_marked_stale: {}",
                        report.learning_loop.cache_marked_stale
                    );
                    println!("hot_rebuilt: {}", report.learning_loop.hot_rebuilt);
                    println!(
                        "target_query_improved: {}",
                        report.learning_loop.target_query_improved
                    );
                    println!(
                        "anti_center_replay_observed: {}",
                        report.learning_loop.anti_center_replay_observed
                    );
                    println!(
                        "unrelated_route_preserved: {}",
                        report.learning_loop.unrelated_route_preserved
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux ChatCore Learn Eval");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- target query improved: `{}`",
                        report.learning_loop.target_query_improved
                    );
                    println!(
                        "- answer from hot: `{}`",
                        report.learning_loop.answer_from_hot
                    );
                    println!(
                        "- anti-wave observed: `{}`",
                        report.learning_loop.anti_center_replay_observed
                    );
                    println!(
                        "- unrelated route preserved: `{}`",
                        report.learning_loop.unrelated_route_preserved
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxHeldoutSuiteBuild(args) => {
            let report = linux_profile::heldout::build_linux_heldout_suite_report(
                linux_profile::heldout::LinuxHeldoutSuiteBuildConfig {
                    residual_pack: args.residual_pack,
                    cases: args.cases,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("cases: {}", report.suite.case_count);
                    println!(
                        "near_collision_cases: {}",
                        report.controls.near_collision_cases
                    );
                    println!(
                        "shortcut_control_cases: {}",
                        report.controls.shortcut_control_cases
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Held-Out Suite");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- cases: `{}`", report.suite.case_count);
                    println!(
                        "- near-collision cases: `{}`",
                        report.controls.near_collision_cases
                    );
                    println!(
                        "- shortcut-control cases: `{}`",
                        report.controls.shortcut_control_cases
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxHeldoutEvalRun(args) => {
            let report = linux_profile::heldout::build_linux_heldout_eval_report(
                linux_profile::heldout::LinuxHeldoutEvalRunConfig {
                    residual_pack: args.residual_pack,
                    suite: args.suite,
                    out: args.out,
                    max_facts: args.max_facts,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("passed: {}/{}", report.metrics.passed, report.metrics.total);
                    println!(
                        "false_positive_rate: {}",
                        report.metrics.false_positive_rate
                    );
                    println!(
                        "shortcut_rejection_rate: {}",
                        report.metrics.shortcut_rejection_rate
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Held-Out Eval");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- passed: `{}/{}`",
                        report.metrics.passed, report.metrics.total
                    );
                    println!("- pass rate: `{}`", report.metrics.pass_rate);
                    println!(
                        "- false-positive rate: `{}`",
                        report.metrics.false_positive_rate
                    );
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxFeedbackBuild(args) => {
            let report = linux_profile::feedback::build_linux_feedback_report(
                linux_profile::feedback::LinuxFeedbackBuildConfig {
                    residual_pack: args.residual_pack,
                    text: args.text,
                    decision: args.decision,
                    note: args.note,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("negative_lanes: {}", report.packet.negative_lanes.len());
                    println!("positive_lanes: {}", report.packet.positive_lanes.len());
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Feedback Build");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- negative lanes: `{}`", report.packet.negative_lanes.len());
                    println!("- positive lanes: `{}`", report.packet.positive_lanes.len());
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxFeedbackApply(args) => {
            let report = linux_profile::feedback::build_linux_feedback_apply_report(
                linux_profile::feedback::LinuxFeedbackApplyConfig {
                    residual_pack: args.residual_pack,
                    feedback: args.feedback,
                    text: args.text,
                    max_facts: args.max_facts,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!(
                        "negative_lanes_matched: {}",
                        report.applied.negative_lanes_matched
                    );
                    println!(
                        "positive_lanes_matched: {}",
                        report.applied.positive_lanes_matched
                    );
                    println!("after_state: {}", report.after.state);
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Feedback Apply");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!(
                        "- negative lanes matched: `{}`",
                        report.applied.negative_lanes_matched
                    );
                    println!(
                        "- positive lanes matched: `{}`",
                        report.applied.positive_lanes_matched
                    );
                    println!("- after state: `{}`", report.after.state);
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxDecisionSearch(args) => {
            let report = linux_profile::decision_search::build_linux_decision_search_report(
                linux_profile::decision_search::LinuxDecisionSearchConfig {
                    residual_pack: args.residual_pack,
                    text: args.text,
                    max_facts: args.max_facts,
                    runtime_snapshot: args.runtime_snapshot,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("state: {}", report.decision_search.state);
                    println!(
                        "missing_evidence: {}",
                        report.decision_search.missing_evidence.len()
                    );
                    println!(
                        "safe_next_checks: {}",
                        report.decision_search.safe_next_checks.len()
                    );
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Decision Search");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- state: `{}`", report.decision_search.state);
                    println!();
                    println!("## Safe Next Checks");
                    for check in &report.decision_search.safe_next_checks {
                        println!(
                            "- `{}` -> `{}` ({})",
                            check.check_id, check.expected_route, check.command_hint
                        );
                    }
                }
            }
            Ok(EXIT_PASS)
        }
        LlmwaveBigCommand::LinuxRelationProfile(args) => {
            let report = linux_profile::relations::build_linux_relation_profile_report(
                linux_profile::relations::LinuxRelationProfileConfig {
                    residual_pack: args.residual_pack,
                    out: args.out,
                },
            )?;
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Text => {
                    println!("{}", report.verdict);
                    println!("families: {}", report.relation_families.len());
                    println!(
                        "present_chains: {}",
                        report
                            .causal_chains
                            .iter()
                            .filter(|chain| chain.present)
                            .count()
                    );
                    println!("missing_routes: {}", report.missing_relation_routes.len());
                }
                OutputFormat::Md => {
                    println!("# LLMWave Linux Relation Profile");
                    println!();
                    println!("- verdict: `{}`", report.verdict);
                    println!("- families: `{}`", report.relation_families.len());
                    println!(
                        "- missing routes: `{}`",
                        report.missing_relation_routes.len()
                    );
                }
            }
            Ok(EXIT_PASS)
        }
    }
}

fn print_structural_capacity_report(
    report: &structural_capacity::StructuralCapacityReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => {
            println!("{}", report.verdict);
            println!("noise_profile: {}", report.workload.noise_profile);
            println!("seeds: {}", report.workload.seeds);
            println!(
                "noise_edges_per_noisy_case: {}",
                report.workload.noise_edges_per_noisy_case
            );
            println!("pattern_shape: {}", report.workload.pattern_shape);
            println!("patterns: {}", report.workload.patterns);
            println!("edges_per_pattern: {}", report.workload.edges_per_pattern);
            println!("active_facts: {}", report.workload.active_facts);
            println!(
                "fixed_pattern_count: {}",
                report.workload.fixed_pattern_count
            );
            println!(
                "fixed_pattern_shape: {}",
                report.workload.fixed_pattern_shape
            );
            println!(
                "smaller_pattern_modes_available: {}",
                report.workload.smaller_pattern_modes_available
            );
            println!(
                "smaller_pattern_shapes_available: {}",
                report.workload.smaller_pattern_shapes_available
            );
            println!("total_cases: {}", report.metrics.total_cases);
            println!(
                "clean_retrieval_pass_rate: {}",
                report.metrics.clean_retrieval_pass_rate
            );
            println!(
                "noisy_retrieval_pass_rate: {}",
                report.metrics.noisy_retrieval_pass_rate
            );
            println!("false_accept_rate: {}", report.metrics.false_accept_rate);
            println!(
                "false_negative_rate: {}",
                report.metrics.false_negative_rate
            );
            println!(
                "missing_edge_rejection_pass_rate: {}",
                report.metrics.missing_edge_rejection_pass_rate
            );
            println!("min_noisy_margin: {}", report.metrics.min_noisy_margin);
            println!(
                "single_peak_under_noise: {}",
                report.gates.single_peak_under_noise
            );
            println!(
                "field_core_lens_admission: {}",
                report.gates.field_core_lens_admission
            );
            println!(
                "lens_admission_peak: {}",
                report.lens_admission.field_pass_peak_target
            );
            println!(
                "lens_admission_verdict: {}",
                report.lens_admission.field_pass_verdict
            );
            println!(
                "anti_wave_traps_reject_false_peaks: {}",
                report.gates.anti_wave_traps_reject_false_peaks
            );
            println!("hot_bytes: {}", report.memory.hot_bytes);
            println!(
                "bytes_per_useful_pattern: {}",
                report.memory.bytes_per_useful_pattern
            );
            println!(
                "structural_capacity_1024_ready: {}",
                report.claim_boundary.structural_capacity_1024_ready
            );
            println!(
                "global_nonlinear_memory_proven: {}",
                report.claim_boundary.global_nonlinear_memory_proven
            );
            println!(
                "broad_chat_llm_ready: {}",
                report.claim_boundary.broad_chat_llm_ready
            );
        }
        OutputFormat::Md => {
            println!("# LLMWave Big Structural Capacity");
            println!();
            println!("- verdict: `{}`", report.verdict);
            println!("- noise profile: `{}`", report.workload.noise_profile);
            println!("- seeds: `{}`", report.workload.seeds);
            println!(
                "- noise edges per noisy case: `{}`",
                report.workload.noise_edges_per_noisy_case
            );
            println!("- pattern shape: `{}`", report.workload.pattern_shape);
            println!("- patterns: `{}`", report.workload.patterns);
            println!(
                "- edges per pattern: `{}`",
                report.workload.edges_per_pattern
            );
            println!("- active facts: `{}`", report.workload.active_facts);
            println!(
                "- fixed pattern count: `{}`",
                report.workload.fixed_pattern_count
            );
            println!(
                "- fixed pattern shape: `{}`",
                report.workload.fixed_pattern_shape
            );
            println!(
                "- smaller pattern modes available: `{}`",
                report.workload.smaller_pattern_modes_available
            );
            println!(
                "- smaller pattern shapes available: `{}`",
                report.workload.smaller_pattern_shapes_available
            );
            println!("- total cases: `{}`", report.metrics.total_cases);
            println!(
                "- clean retrieval pass rate: `{}`",
                report.metrics.clean_retrieval_pass_rate
            );
            println!(
                "- noisy retrieval pass rate: `{}`",
                report.metrics.noisy_retrieval_pass_rate
            );
            println!(
                "- false accept rate: `{}`",
                report.metrics.false_accept_rate
            );
            println!(
                "- false negative rate: `{}`",
                report.metrics.false_negative_rate
            );
            println!(
                "- missing edge rejection pass rate: `{}`",
                report.metrics.missing_edge_rejection_pass_rate
            );
            println!("- min noisy margin: `{}`", report.metrics.min_noisy_margin);
            println!(
                "- single peak under noise: `{}`",
                report.gates.single_peak_under_noise
            );
            println!(
                "- field-core lens admission: `{}`",
                report.gates.field_core_lens_admission
            );
            println!(
                "- lens admission peak: `{}`",
                report.lens_admission.field_pass_peak_target
            );
            println!(
                "- lens admission verdict: `{}`",
                report.lens_admission.field_pass_verdict
            );
            println!(
                "- anti-wave traps reject false peaks: `{}`",
                report.gates.anti_wave_traps_reject_false_peaks
            );
            println!("- hot bytes: `{}`", report.memory.hot_bytes);
            println!(
                "- bytes per useful pattern: `{}`",
                report.memory.bytes_per_useful_pattern
            );
            println!(
                "- nonlinear memory proven: `{}`",
                report.claim_boundary.global_nonlinear_memory_proven
            );
            println!(
                "- broad chat LLM ready: `{}`",
                report.claim_boundary.broad_chat_llm_ready
            );
        }
    }
    Ok(())
}

fn build_contract_report() -> LlmwaveBigReport {
    LlmwaveBigReport {
        command: "llmwave-big contract",
        version: LLMWAVE_BIG_VERSION,
        roadmap_block: "v158-v160",
        implementation_state: "CONTRACT_ONLY_NOT_A_BIG_LLM",
        core_version: CORE_VERSION,
        nanda_6m_version: nanda_6m::VERSION,
        wave_dim: WAVE_DIM,
        contract: contract::build_contract(),
        bigness_metrics: metrics::build_bigness_metrics(),
        claim_boundary: claims::build_claim_boundary(),
        engineering_rules: EngineeringRulesReport {
            source: "LLMWAVE_BIG_ENGINEERING_RULES.md",
            pattern_store_boundary:
                "do_not_add_new_llmwave_big_architecture_to_src_pattern_store_rs",
            hot_core_rules: vec![
                "no_json_in_hot_core",
                "no_strings_in_hot_core",
                "no_heap_or_hashmap_in_inner_loop",
                "fixed_size_records",
                "explicit_byte_budget",
                "bench6m_coverage_required_before_speed_claims",
            ],
            atlas_rules: vec![
                "wave_atlas_may_be_large",
                "loader_must_select_small_active_packet",
                "cold_labels_and_evidence_stay_outside_active_core",
                "active_records_use_compact_ids_phases_seeds_evidence_refs",
            ],
            l2_l3_boundary: vec![
                "l2_word_field_surface_tokens_roots_morphemes_words",
                "l3_schema_field_operators_roles_routes_schema_cognition",
                "l2_l3_interaction_is_bias_or_projection_not_shared_storage",
            ],
        },
        next_versions: vec![
            "v161_atlas_file_format",
            "v162_symbol_atom_table",
            "v163_operator_table",
            "v164_schema_table",
            "v165_residual_records",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn big_contract_keeps_claims_unproven() {
        let report = build_contract_report();
        assert_eq!(report.roadmap_block, "v158-v160");
        assert_eq!(report.claim_boundary.current_state, "BIG_MODEL_NOT_PROVEN");
        assert!(!report.claim_boundary.claims.llm_ready);
        assert!(!report.claim_boundary.claims.nonlinear_memory_proven);
        assert!(!report.claim_boundary.claims.cache_only_execution_proven);
    }

    #[test]
    fn core_v1_contract_records_phase_1_boundaries_without_llm_claims() {
        let report = core_v1_contract::build_core_v1_contract_report();
        assert_eq!(report.mode, "llmwave-core-v1-contract");
        assert_eq!(report.verdict, "CORE_V1_CONTRACT_RECORDED_NOT_IMPLEMENTED");
        assert!(report.claim_boundary.core_contract_recorded);
        assert!(report.claim_boundary.claim_boundary_table_present);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.field_core_as_sole_engine);
        assert!(!report.phase_1_exit_criteria.implementation_started);
        assert_eq!(report.components.len(), 11);
        assert_eq!(report.required_boundaries.len(), 5);
        assert!(report
            .components
            .iter()
            .any(|component| component.name == "Memory Writer"
                && component.must_not_own.contains(&"final_claims")));
        assert!(report
            .required_boundaries
            .iter()
            .any(|boundary| boundary.rule == "Verifier does not generate."));
    }

    #[test]
    fn core_v1_field_cutover_records_shared_field_engine_without_llm_claims() {
        let report = core_v1_field_cutover::build_core_v1_field_cutover_report();
        assert_eq!(report.mode, "llmwave-core-v1-field-cutover");
        assert_eq!(
            report.verdict,
            "CORE_V1_FIELD_OPERATIONS_CUTOVER_RECORDED_NOT_LLM"
        );
        assert_eq!(report.family_cutovers.len(), 3);
        assert_eq!(report.operation_contract.len(), 7);
        assert!(report
            .operation_contract
            .iter()
            .any(|operation| operation.operation == "anti_wave"
                && operation.owner.contains("field_core::anti_wave")));
        assert!(report
            .family_cutovers
            .iter()
            .all(|family| family.sole_field_operations_engine));
        assert!(
            report
                .claim_boundary
                .field_core_as_sole_field_operations_engine
        );
        assert!(!report.claim_boundary.field_core_as_sole_llmwave_core_engine);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-3-memory-writer-v1");
    }

    #[test]
    fn core_v1_memory_writer_records_residual_surface_and_evidence_memory() {
        let report = core_v1_memory_writer::build_core_v1_memory_writer_report();
        assert_eq!(report.mode, "llmwave-core-v1-memory-writer");
        assert_eq!(
            report.verdict,
            "CORE_V1_MEMORY_WRITER_READY_NOT_NONLINEAR_PROOF"
        );
        assert!(report.phase_3_exit_criteria.residual_write_path_active);
        assert!(
            report
                .phase_3_exit_criteria
                .raw_dictionary_is_not_primary_memory
        );
        assert!(report.phase_3_exit_criteria.memory_write_report_present);
        assert!(report.schema_residual_summary.residual_write_count > 0);
        assert!(report.surface_family_summary.accepted_family_count > 0);
        assert!(report.rejected.rejected_duplicate_count > 0);
        assert!(report.rejected.rejected_noise_count > 0);
        assert!(report.byte_report.writer_saving_ratio > 0.0);
        assert!(report
            .evidence_pointer_contract
            .iter()
            .any(|field| field.field == "ResidualV1.evidence_ref"));
        assert!(report.claim_boundary.residual_write_path_active);
        assert!(report.claim_boundary.raw_dictionary_is_not_primary_memory);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
        assert_eq!(report.next_phase, "phase-4-nonlinear-memory-proof-v1");
    }

    #[test]
    fn core_v1_nonlinear_proof_blocks_final_claim_without_heldout_binding() {
        let report = core_v1_nonlinear_proof::build_core_v1_nonlinear_proof_report();
        assert_eq!(report.mode, "llmwave-core-v1-nonlinear-proof");
        assert_eq!(report.verdict, "CORE_V1_NONLINEAR_MEMORY_CANDIDATE_BLOCKED");
        assert!(
            report
                .proof_metrics
                .bytes_per_useful_fact_falls_at_three_scale_points
        );
        assert!(report.proof_metrics.writer_beats_raw_dictionary_fixture);
        assert!(report.proof_metrics.role_error_rate_bounded);
        assert!(report.proof_metrics.false_positive_rate_bounded);
        assert!(!report.proof_metrics.heldout_quality_bound_to_writer);
        assert!(!report.eval_evidence.external_corpus_present);
        assert!(report.claim_boundary.nonlinear_memory_candidate);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
        assert!(report
            .claim_boundary
            .blocked_by
            .contains(&"heldout_quality_not_bound_to_memory_writer"));
        assert_eq!(report.next_phase, "phase-5-query-wave-v1");
    }

    #[test]
    fn core_v1_query_wave_blocks_role_swap_and_missing_evidence() {
        let report = core_v1_query_wave::build_core_v1_query_wave_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-query-wave");
        assert_eq!(report.verdict, "CORE_V1_QUERY_WAVE_READY_NOT_RETRIEVAL");
        assert_eq!(
            core::mem::size_of::<core_v1_query_wave::CoreV1QueryWaveRecord64>(),
            64
        );
        assert_eq!(report.route_hint, "customs-clearance-status");
        assert_eq!(report.field_state, "QUERY_WAVE_STRUCTURED");
        assert!(!report.safe_to_answer);
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert!(report
            .exit_eval
            .iter()
            .any(|case| case.case_id == "role_swap_invoice_actor"
                && case.observed_state == "QUERY_WAVE_REVERSED_VETO"
                && case.passed));
        assert!(report
            .exit_eval
            .iter()
            .any(|case| case.case_id == "missing_evidence_release"
                && case.observed_state == "QUERY_WAVE_MISSING_EVIDENCE_NO_ANSWER"
                && !case.safe_to_answer
                && case.passed));
        assert!(report.claim_boundary.query_wave_v1_implemented);
        assert!(
            report
                .claim_boundary
                .same_meaning_paraphrase_selects_same_route
        );
        assert!(
            report
                .claim_boundary
                .role_swap_triggers_reversed_polarity_or_veto
        );
        assert!(
            report
                .claim_boundary
                .missing_evidence_blocks_confident_answer
        );
        assert!(!report.claim_boundary.retrieval_ready);
        assert!(!report.claim_boundary.answer_generation_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-6-active-field-retrieval-v1");
    }

    #[test]
    fn core_v1_active_retrieval_covers_field_states_and_blocks_traps() {
        let report = core_v1_active_retrieval::build_core_v1_active_retrieval_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-active-retrieval");
        assert_eq!(
            report.verdict,
            "CORE_V1_ACTIVE_FIELD_RETRIEVAL_READY_NOT_REASONING"
        );
        assert_eq!(
            core::mem::size_of::<core_v1_active_retrieval::CoreV1RoutePeakRecord32>(),
            32
        );
        assert_eq!(report.output.field_state, "FIELD_FOCUSED");
        assert_eq!(report.output.top_peak, "customs-clearance-status");
        assert!(report.output.safe_to_answer);
        assert!(report.output.peak_margin > 0.4);
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert_eq!(
            report.metrics.required_field_states_covered,
            report.metrics.required_field_state_count
        );
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "thin_assertion_trap"
                && case.observed_state == "FIELD_THIN"
                && !case.safe_to_answer
                && case.passed));
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "contested_invoice_customs"
                && case.observed_state == "FIELD_CONTESTED"
                && !case.safe_to_answer
                && case.passed));
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "reversed_invoice_actor"
                && case.observed_state == "FIELD_REVERSED"
                && !case.safe_to_answer
                && case.passed));
        assert!(report.claim_boundary.active_field_retrieval_v1_implemented);
        assert!(report.claim_boundary.retrieval_ready);
        assert!(report.claim_boundary.lexical_traps_blocked);
        assert!(
            report
                .claim_boundary
                .contested_fields_block_answer_generation
        );
        assert!(report.claim_boundary.anti_wave_suppression_local);
        assert!(!report.claim_boundary.schema_reasoning_ready);
        assert!(!report.claim_boundary.answer_generation_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-7-schema-reasoning-v1");
    }

    #[test]
    fn core_v1_schema_reasoning_propagates_missing_dependency() {
        let report = core_v1_schema_reasoning::build_core_v1_schema_reasoning_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-schema-reasoning");
        assert_eq!(report.verdict, "CORE_V1_SCHEMA_REASONING_READY_NOT_SURFACE");
        assert_eq!(
            core::mem::size_of::<core_v1_schema_reasoning::CoreV1SchemaAnswerPlanRecord64>(),
            64
        );
        assert_eq!(
            report.answer_plan.answer_state,
            "MISSING_DEPENDENCY_DECLARATION_PACKET"
        );
        assert_eq!(
            report.answer_plan.forbidden_shortcut,
            "invoice_or_payment_implies_customs_release"
        );
        assert!(report.answer_plan.safe_for_surface_generation);
        assert!(report
            .dependency_chain
            .iter()
            .any(|step| step.operator == "missing" && step.state == "missing_blocks_answer"));
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "multi_hop_missing_c"
                && case.observed_state == "MISSING_DEPENDENCY_DECLARATION_PACKET"
                && case.passed));
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "contradiction_refusal"
                && case.observed_state == "CONTRADICTION_REFUSED_UNSUPPORTED"
                && !case.observed_surface_permission
                && case.passed));
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "role_swap_block"
                && case.observed_state == "ROLE_SWAP_BLOCKED"
                && !case.observed_surface_permission
                && case.passed));
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert!(report.claim_boundary.schema_reasoning_v1_implemented);
        assert!(report.claim_boundary.schema_reasoning_ready);
        assert!(report.claim_boundary.multi_hop_reasoning_ready);
        assert!(report.claim_boundary.contradiction_refusal_ready);
        assert!(report.claim_boundary.role_swap_block_ready);
        assert!(!report.claim_boundary.surface_generation_ready);
        assert!(!report.claim_boundary.answer_verifier_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-8-surface-generation-v1");
    }

    #[test]
    fn core_v1_surface_generation_materializes_evidence_bound_refusal() {
        let report = core_v1_surface_generation::build_core_v1_surface_generation_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-surface-generation");
        assert_eq!(
            report.verdict,
            "CORE_V1_SURFACE_GENERATION_READY_NOT_VERIFIED"
        );
        assert_eq!(
            core::mem::size_of::<core_v1_surface_generation::CoreV1SurfaceCandidateRecord32>(),
            32
        );
        assert_eq!(report.surface.answer_mode, "missing evidence refusal");
        assert_eq!(report.surface.state, "MISSING_EVIDENCE_REFUSAL");
        assert!(report.surface.safe_for_verifier);
        assert!(report.surface.text.contains("customs release still needs"));
        assert!(report
            .surface
            .evidence_routes
            .contains(&"customs-clearance-status"));
        assert!(report
            .surface
            .role_bindings
            .iter()
            .any(|binding| binding.role == "forbidden_shortcut"));
        assert!(report
            .forbidden_behavior
            .iter()
            .all(|behavior| behavior.blocked));
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert!(report.claim_boundary.surface_generation_v1_implemented);
        assert!(report.claim_boundary.evidence_bound_surface_ready);
        assert!(!report.claim_boundary.free_form_generation);
        assert!(!report.claim_boundary.answer_verifier_ready);
        assert!(!report.claim_boundary.final_answer_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-9-answer-verifier-v1");
    }

    #[test]
    fn core_v1_answer_verifier_allows_only_verified_refusal() {
        let report = core_v1_answer_verifier::build_core_v1_answer_verifier_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-answer-verifier");
        assert_eq!(report.verdict, "CORE_V1_ANSWER_VERIFIER_READY_LOCAL_ONLY");
        assert_eq!(
            core::mem::size_of::<core_v1_answer_verifier::CoreV1AnswerVerificationRecord32>(),
            32
        );
        assert_eq!(report.verifier.decision, "VERIFIED_REFUSAL_READY");
        assert_eq!(report.verifier.answer_state, "LOCAL_FINAL_REFUSAL");
        assert!(report.verifier.safe_to_answer);
        assert!(report
            .verifier
            .evidence_routes
            .contains(&"release_evidence_missing"));
        assert!(report
            .verifier
            .blocked_shortcuts
            .contains(&"invoice_or_payment_implies_customs_release"));
        assert!(report.blocking_rules.iter().all(|rule| rule.active));
        assert!(report.eval_cases.iter().any(|case| case.case_id
            == "positive_clearance_without_release_evidence"
            && case.observed_decision == "UNSAFE_SURFACE_REJECTED"
            && !case.observed_safe_to_answer
            && case.passed));
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert!(report.claim_boundary.answer_verifier_v1_implemented);
        assert!(report.claim_boundary.verified_refusal_ready);
        assert!(!report.claim_boundary.positive_answer_ready);
        assert!(!report.claim_boundary.feedback_learning_ready);
        assert!(!report.claim_boundary.general_chat_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-10-feedback-learning-v1");
    }

    #[test]
    fn core_v1_feedback_learning_changes_next_field_pass_locally() {
        let report = core_v1_feedback_learning::build_core_v1_feedback_learning_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-feedback-learning");
        assert_eq!(
            report.verdict,
            "CORE_V1_FEEDBACK_LEARNING_READY_NOT_CONSOLIDATED"
        );
        assert_eq!(
            core::mem::size_of::<core_v1_feedback_learning::CoreV1FeedbackMemoryRecord32>(),
            32
        );
        assert_eq!(
            report.memory_packet.packet_state,
            "FEEDBACK_PACKET_APPLIED_TO_NEXT_PASS"
        );
        assert_eq!(report.memory_packet.lanes.len(), 2);
        assert!(report.memory_packet.shortcut_specific);
        assert!(!report.memory_packet.route_kill_switch);
        assert!(report.next_field_pass.field_changed);
        assert!(report.next_field_pass.refusal_delta > 0);
        assert!(report.next_field_pass.shortcut_delta < 0);
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "reject_positive_shortcut"
                && case.observed_effect == "suppress_shortcut_only"
                && case.passed));
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert!(report.claim_boundary.feedback_learning_v1_implemented);
        assert!(report.claim_boundary.memory_packet_ready);
        assert!(report.claim_boundary.next_field_pass_changes);
        assert!(report.claim_boundary.shortcut_specific_learning);
        assert!(!report.claim_boundary.consolidation_ready);
        assert!(!report.claim_boundary.broad_training_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-11-consolidation-sleep-pass-v1");
    }

    #[test]
    fn core_v1_consolidation_sleep_merges_without_erasing_safety() {
        let report = core_v1_consolidation_sleep::build_core_v1_consolidation_sleep_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-consolidation-sleep");
        assert_eq!(
            report.verdict,
            "CORE_V1_CONSOLIDATION_SLEEP_READY_NOT_BROAD_EVAL"
        );
        assert_eq!(
            core::mem::size_of::<core_v1_consolidation_sleep::CoreV1ConsolidatedMemoryRecord32>(),
            32
        );
        assert_eq!(
            report.consolidated_memory.state,
            "CONSOLIDATED_LOCAL_MEMORY_READY"
        );
        assert!(report.sleep_pass.after_records < report.sleep_pass.before_records);
        assert!(report.sleep_pass.preserved_negative_lanes > 0);
        assert!(!report.sleep_pass.route_kill_switch);
        assert!(report.post_sleep_field.shortcut_still_suppressed);
        assert!(report.post_sleep_field.safe_to_answer);
        assert!(report
            .consolidated_memory
            .retained_forms
            .contains(&"negative_shortcut_lane"));
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "watch_decay_not_accept" && case.passed));
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert!(report.claim_boundary.consolidation_sleep_v1_implemented);
        assert!(report.claim_boundary.consolidation_ready);
        assert!(report.claim_boundary.safety_preserved_after_sleep);
        assert!(!report.claim_boundary.broad_eval_ready);
        assert!(!report.claim_boundary.broad_training_ready);
        assert!(!report.claim_boundary.general_chat_ready);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.next_phase, "phase-12-broad-eval-harness-v1");
    }

    #[test]
    fn core_v1_broad_eval_harness_passes_local_controls_without_hard_claims() {
        let report = core_v1_broad_eval_harness::build_core_v1_broad_eval_harness_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.mode, "llmwave-core-v1-broad-eval-harness");
        assert_eq!(report.verdict, "CORE_V1_BROAD_EVAL_HARNESS_READY_NOT_LLM");
        assert_eq!(
            core::mem::size_of::<core_v1_broad_eval_harness::CoreV1EvalCaseRecord32>(),
            32
        );
        assert_eq!(report.suite.cases.len(), 10);
        assert_eq!(report.suite.failed, 0);
        assert_eq!(report.suite.false_positive_count, 0);
        assert_eq!(report.suite.false_negative_count, 0);
        assert!(report
            .suite
            .cases
            .iter()
            .any(|case| case.case_id == "broad_corpus_claim_blocked"
                && case.observed == "BROAD_CORPUS_MISSING"
                && case.passed));
        assert!(report.blockers.iter().all(|blocker| blocker.active));
        assert!(report
            .exit_criteria
            .iter()
            .all(|criterion| criterion.passed));
        assert!(report.claim_boundary.broad_eval_harness_v1_implemented);
        assert!(report.claim_boundary.local_core_v1_pipeline_ready);
        assert!(report.claim_boundary.safety_controls_ready);
        assert!(!report.claim_boundary.real_broad_corpus_loaded);
        assert!(!report.claim_boundary.broad_generalization_proven);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(
            report.next_phase,
            "core-v1-real-broad-corpus-and-density-proof"
        );
    }

    #[test]
    fn atlas_contract_keeps_cold_evidence_out_of_active_core() {
        let report = atlas::build_atlas_report();
        assert_eq!(report.roadmap_block, "v161-v170");
        assert_eq!(report.doctor.verdict, "ATLAS_SAMPLE_OK");
        assert!(report
            .active_packet_contract
            .must_not_contain
            .contains(&"evidence_text"));
        assert!(report
            .loader_preview
            .evidence_refs
            .iter()
            .all(|evidence_ref| *evidence_ref > 0));
    }

    #[test]
    fn atlas_contract_has_required_record_formats() {
        let report = atlas::build_atlas_report();
        let names: Vec<_> = report
            .record_formats
            .iter()
            .map(|record| record.name)
            .collect();
        for required in [
            "SymbolAtom",
            "OperatorAtom",
            "SchemaRecord",
            "ResidualRecord",
        ] {
            assert!(names.contains(&required));
        }
        assert!(report
            .indexes
            .iter()
            .any(|index| index.name == "query_wave_to_candidate_schemas"));
    }

    #[test]
    fn active_core_budget_matches_nanda_6m_budget() {
        let report = active_core::build_active_core_report();
        assert_eq!(report.roadmap_block, "v171-v180");
        assert_eq!(report.budget.total_bytes, nanda_6m::BUDGET_BYTES);
        assert!(report.budget.fits_nanda_6m_budget);
        assert_eq!(report.packet_format.schema_record_bytes, 32);
        assert_eq!(report.packet_format.residual_record_bytes, 32);
    }

    #[test]
    fn active_core_sample_cycle_is_ready_but_not_llm_proof() {
        let report = active_core::build_active_core_report();
        assert_eq!(report.cycle.verdict, "ACTIVE_CORE_READY");
        assert!(report.cycle.safe_to_answer);
        assert!(report.cycle.margin > 0);
        assert_eq!(report.loader_eval.sample_lifted_operator, 3);
        assert_eq!(report.loader_eval.sample_lifted_schema, 101);
    }

    #[test]
    fn l2_word_field_uses_l3_bias_without_l3_storage_mix() {
        let report =
            l2_word_field::build_l2_word_field_report(l3_schema_field::business_invoice_bias());
        assert_eq!(report.roadmap_block, "v361-v390");
        assert_eq!(report.verdict, "L2_READY");
        assert_eq!(report.candidate_cache.record_bytes, 32);
        assert_eq!(report.candidate_cache.top_token_label, "invoice");
        assert_eq!(report.l3_bias.operator, "issues");
        assert_eq!(report.sync_policy.l2_update, "per_keystroke");
        assert_eq!(report.runtime_field.top_surface, "счете");
        assert_eq!(
            report.runtime_field.field_state,
            "L2_WAVE_RUNTIME_READY_NOT_CHAT"
        );
    }

    #[test]
    fn l2_anti_wave_suppresses_schema_breaking_prefix_match() {
        let report =
            l2_word_field::build_l2_word_field_report(l3_schema_field::business_invoice_bias());
        let inventory = report
            .candidate_cache
            .sample
            .iter()
            .find(|candidate| candidate.label == "inventory")
            .expect("inventory candidate");
        assert!(inventory.anti_score > 0);
        assert!(inventory.final_score < report.candidate_cache.sample[0].final_score);
    }

    #[test]
    fn l2_wave_runtime_suppresses_near_root_prefix_trap() {
        let report =
            l2_word_field::build_l2_word_field_report(l3_schema_field::business_invoice_bias());
        let top = report
            .runtime_field
            .candidates
            .iter()
            .find(|candidate| candidate.surface == "счете")
            .expect("top surface");
        let trap = report
            .runtime_field
            .candidates
            .iter()
            .find(|candidate| candidate.surface == "счетчик")
            .expect("prefix trap");
        assert_eq!(report.runtime_field.top_family, "счет");
        assert!(report.runtime_field.margin >= l2_word_field::L2_MIN_READY_MARGIN);
        assert!(trap.prefix_resonance >= top.prefix_resonance);
        assert!(trap.anti_wave > 0);
        assert!(trap.final_score < top.final_score);
        assert!(!report.runtime_field.claim_boundary.chat_ready);
        assert!(!report.runtime_field.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn hrr_binding_recovers_role_fillers_with_cleanup() {
        let report = hrr_binding::build_hrr_binding_report();
        assert_eq!(report.roadmap_block, "v391-v430");
        assert_eq!(report.verdict, "HRR_BINDING_READY_NOT_NONLINEAR_PROOF");
        assert_eq!(report.metrics.role_recall, 1.0);
        assert_eq!(report.metrics.noisy_role_recall, 1.0);
        assert_eq!(report.metrics.ambiguous_cleanup_rate, 0.0);
        assert!(report.bindings.iter().all(|binding| binding.exact));
        assert!(report
            .bindings
            .iter()
            .any(|binding| binding.role == "supplier" && binding.recovered == "Honglu"));
    }

    #[test]
    fn hrr_binding_rejects_role_collision_trap() {
        let report = hrr_binding::build_hrr_binding_report();
        assert!(report.collision_eval.rejected);
        assert_eq!(report.collision_eval.trap_role, "supplier");
        assert_eq!(report.collision_eval.expected_filler, "Honglu");
        assert_eq!(report.collision_eval.rejected_filler, "Rustrade");
        assert!(report.collision_eval.expected_score > report.collision_eval.rejected_score);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
    }

    #[test]
    fn l3_schema_bind_recovers_schema_roles() {
        let report = l3_schema_bind::build_l3_schema_bind_report();
        assert_eq!(report.roadmap_block, "v431-v455");
        assert_eq!(report.verdict, "L3_SCHEMA_BIND_READY_NOT_LLM");
        assert_eq!(report.schema.schema_id, 101);
        assert_eq!(report.schema.operator_id, 3);
        assert_eq!(report.metrics.schema_role_recall, 1.0);
        assert_eq!(report.metrics.role_error_rate, 0.0);
        assert!(report
            .recovered_roles
            .iter()
            .any(|role| role.role == "subject:supplier" && role.recovered == "Honglu"));
        assert!(report
            .recovered_roles
            .iter()
            .any(|role| role.role == "object:document" && role.recovered == "invoice"));
    }

    #[test]
    fn l3_schema_bind_rejects_role_swap() {
        let report = l3_schema_bind::build_l3_schema_bind_report();
        assert!(report.role_swap_trap.rejected);
        assert_eq!(report.role_swap_trap.wrong_claim, "invoice issues Honglu");
        assert_eq!(report.role_swap_trap.recovered_subject, "Honglu");
        assert_eq!(report.role_swap_trap.recovered_object, "invoice");
        assert_eq!(report.metrics.role_swap_reject_rate, 1.0);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
    }

    #[test]
    fn l2_l3_coupling_reranks_surface_by_schema_role() {
        let report = l2_l3_coupling::build_l2_l3_coupling_report();
        assert_eq!(report.roadmap_block, "v456-v480");
        assert_eq!(report.verdict, "L2_L3_COUPLED_READY_NOT_CHAT");
        assert_eq!(report.l2_probe.raw_top, "inventory");
        assert_eq!(report.l2_probe.coupled_top, "invoice");
        assert_eq!(report.l3_schema.schema_id, 101);
        assert_eq!(report.l3_schema.expected_role, "object:document");
        assert_eq!(report.l3_schema.expected_filler, "invoice");
        assert_eq!(report.metrics.l2_l3_agreement_rate, 1.0);
        assert_eq!(report.metrics.role_error_rate, 0.0);
        assert!(report.rerank.top_margin > 0);
    }

    #[test]
    fn l2_l3_coupling_rejects_role_disagreement_trap() {
        let report = l2_l3_coupling::build_l2_l3_coupling_report();
        assert!(report.disagreement_trap.rejected);
        assert_eq!(
            report.disagreement_trap.trap,
            "l2_surface_looks_valid_but_l3_role_disagrees"
        );
        assert_eq!(report.disagreement_trap.l2_preferred, "invoice");
        assert_eq!(
            report.disagreement_trap.l3_expected_role,
            "subject:supplier"
        );
        assert_eq!(report.disagreement_trap.l3_expected_filler, "Honglu");
        assert_eq!(report.metrics.disagreement_reject_rate, 1.0);
        assert!(!report.claim_boundary.l2_l3_storage_mixed);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn coupled_decode_loop_generates_schema_sequence() {
        let report = coupled_decode_loop::build_coupled_decode_loop_report();
        assert_eq!(report.roadmap_block, "v481-v520");
        assert_eq!(report.verdict, "COUPLED_DECODE_LOOP_READY_NOT_CHAT");
        assert_eq!(report.bridge_state, "L2_L3_COUPLED_READY_NOT_CHAT");
        assert_eq!(report.final_sequence, vec!["Honglu", "issues", "invoice"]);
        assert_eq!(report.metrics.completed_steps, 3);
        assert!(report.metrics.sequence_exact);
        assert_eq!(report.metrics.role_error_rate, 0.0);
        assert_eq!(report.accepted_steps[0].raw_top, "invoice");
        assert_eq!(report.accepted_steps[0].accepted, "Honglu");
        assert_eq!(report.accepted_steps[2].raw_top, "inventory");
        assert_eq!(report.accepted_steps[2].accepted, "invoice");
        assert_eq!(
            core::mem::size_of::<coupled_decode_loop::CoupledStep32>(),
            32
        );
        assert!(report.claim_boundary.fixed_step_records);
    }

    #[test]
    fn coupled_decode_loop_stops_bad_continuation() {
        let report = coupled_decode_loop::build_coupled_decode_loop_report();
        assert!(report.bad_continuation_trap.rejected);
        assert_eq!(
            report.bad_continuation_trap.trap,
            "invoice_issues_honglu_role_break"
        );
        assert_eq!(report.bad_continuation_trap.stopped_at_step, 1);
        assert_eq!(
            report.bad_continuation_trap.expected_role,
            "subject:supplier"
        );
        assert_eq!(report.bad_continuation_trap.rejected_surface, "invoice");
        assert_eq!(report.metrics.bad_continuation_reject_rate, 1.0);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn multi_schema_competition_selects_coherent_route() {
        let report = multi_schema_competition::build_multi_schema_competition_report();
        assert_eq!(report.roadmap_block, "v521-v560");
        assert_eq!(report.verdict, "MULTI_SCHEMA_COMPETITION_READY_NOT_CHAT");
        assert_eq!(
            report.decode_bridge_state,
            "COUPLED_DECODE_LOOP_READY_NOT_CHAT"
        );
        assert_eq!(report.active_schemas.len(), 4);
        assert_eq!(report.metrics.selected_schema_id, 101);
        assert_eq!(report.selected_route.route, "supplier-docs");
        assert_eq!(
            report.selected_route.sequence,
            vec!["Honglu", "issues", "invoice"]
        );
        assert!(report.metrics.top_margin > 0);
        assert_eq!(report.metrics.schema_selection_error_rate, 0.0);
        assert_eq!(
            core::mem::size_of::<multi_schema_competition::SchemaPeak32>(),
            32
        );
        assert!(report.claim_boundary.fixed_peak_records);
    }

    #[test]
    fn multi_schema_competition_rejects_route_splice() {
        let report = multi_schema_competition::build_multi_schema_competition_report();
        assert!(report.route_splice_trap.individually_plausible);
        assert!(!report.route_splice_trap.selected_as_whole_route);
        assert!(report.route_splice_trap.rejected);
        assert_eq!(
            report.route_splice_trap.trap,
            "route_splice_honglu_pays_invoice"
        );
        assert_eq!(
            report.route_splice_trap.proposed_sequence,
            vec!["Honglu", "pays", "invoice"]
        );
        assert_eq!(report.metrics.route_splice_reject_rate, 1.0);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn schema_memory_growth_promotes_repeated_routes() {
        let report = schema_memory_growth::build_schema_memory_growth_report();
        assert_eq!(report.roadmap_block, "v561-v620");
        assert_eq!(report.verdict, "SCHEMA_MEMORY_GROWTH_READY_NOT_CHAT");
        assert_eq!(
            report.competition_bridge_state,
            "MULTI_SCHEMA_COMPETITION_READY_NOT_CHAT"
        );
        assert_eq!(report.observed_fact_count, 11);
        assert_eq!(report.memory_metrics.promoted_count, 3);
        assert!(report
            .promoted_schemas
            .iter()
            .any(|schema| schema.route == "supplier-docs" && schema.support_count == 3));
        assert!(report
            .promoted_schemas
            .iter()
            .any(|schema| schema.route == "buyer-payment" && schema.support_count == 3));
        assert!(report
            .promoted_schemas
            .iter()
            .any(|schema| schema.route == "customs-check" && schema.support_count == 3));
        assert_eq!(
            core::mem::size_of::<schema_memory_growth::LearnedSchema32>(),
            32
        );
        assert!(report.claim_boundary.fixed_learned_schema_records);
    }

    #[test]
    fn schema_memory_growth_rejects_one_off_trap() {
        let report = schema_memory_growth::build_schema_memory_growth_report();
        assert!(report.negative_control.rejected);
        assert_eq!(
            report.negative_control.trap,
            "single_observation_should_not_promote_schema"
        );
        assert_eq!(
            report.negative_control.proposed_form,
            "warehouse signs invoice"
        );
        assert!(!report.negative_control.promoted);
        assert_eq!(report.memory_metrics.false_promotion_rate, 0.0);
        assert_eq!(report.memory_metrics.role_error_rate, 0.0);
        assert!(report.rejected_candidates.iter().any(|candidate| {
            candidate.route == "warehouse-noise"
                && candidate.reason == "insufficient_repeated_evidence"
        }));
        assert!(!report.claim_boundary.external_corpus_loaded);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn open_surface_generation_materializes_role_safe_phrase() {
        let report = open_surface_generation::build_open_surface_generation_report();
        assert_eq!(report.roadmap_block, "v621-v700");
        assert_eq!(report.verdict, "OPEN_SURFACE_GENERATION_READY_NOT_CHAT");
        assert_eq!(
            report.schema_growth_bridge_state,
            "SCHEMA_MEMORY_GROWTH_READY_NOT_CHAT"
        );
        assert_eq!(report.selected_schema.route, "supplier-docs");
        assert_eq!(
            report.materialized_surface,
            "Honglu issued invoice PI-03 to Rustrade"
        );
        assert_eq!(report.surface_plan.len(), 6);
        assert!(report.generation_metrics.exact_surface);
        assert_eq!(report.generation_metrics.grammar_error_rate, 0.0);
        assert_eq!(report.generation_metrics.role_surface_error_rate, 0.0);
        assert_eq!(
            core::mem::size_of::<open_surface_generation::SurfaceStep32>(),
            32
        );
        assert!(report.claim_boundary.fixed_surface_step_records);
    }

    #[test]
    fn open_surface_generation_rejects_route_splice_verb() {
        let report = open_surface_generation::build_open_surface_generation_report();
        assert!(report.trap.rejected);
        assert_eq!(report.trap.trap, "surface_route_splice");
        assert_eq!(
            report.trap.proposed_surface,
            "Honglu paid invoice PI-03 to Rustrade"
        );
        assert_eq!(report.generation_metrics.trap_reject_rate, 1.0);
        assert!(!report.claim_boundary.external_corpus_loaded);
        assert!(!report.claim_boundary.free_form_chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn reasoning_field_propagates_multi_step_chain() {
        let report = reasoning_field::build_reasoning_field_report();
        assert_eq!(report.roadmap_block, "v701-v780");
        assert_eq!(report.verdict, "MULTI_STEP_REASONING_FIELD_READY_NOT_CHAT");
        assert_eq!(
            report.surface_bridge_state,
            "OPEN_SURFACE_GENERATION_READY_NOT_CHAT"
        );
        assert_eq!(
            report.premise_surface,
            "Honglu issued invoice PI-03 to Rustrade"
        );
        assert_eq!(report.hops.len(), 3);
        assert_eq!(report.metrics.hop_count, 3);
        assert!(report.metrics.chain_exact);
        assert_eq!(report.metrics.contradiction_rate, 0.0);
        assert!(report
            .inferred_state
            .contains(&"payment_should_follow_invoice"));
        assert_eq!(core::mem::size_of::<reasoning_field::ReasoningHop32>(), 32);
        assert!(report.claim_boundary.fixed_reasoning_hop_records);
    }

    #[test]
    fn reasoning_field_rejects_missing_evidence_shortcut() {
        let report = reasoning_field::build_reasoning_field_report();
        assert!(report.trap.rejected);
        assert_eq!(report.trap.trap, "missing_evidence_shortcut");
        assert_eq!(report.trap.proposed_inference, "customs cleared goods");
        assert_eq!(report.metrics.missing_evidence_reject_rate, 1.0);
        assert!(!report.claim_boundary.external_corpus_loaded);
        assert!(!report.claim_boundary.broad_reasoning_proven);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn dialogue_state_answers_with_not_proven_boundary() {
        let report = dialogue_state::build_dialogue_state_report();
        assert_eq!(report.roadmap_block, "v781-v860");
        assert_eq!(report.verdict, "DIALOGUE_STATE_READY_NOT_CHAT");
        assert_eq!(
            report.reasoning_bridge_state,
            "MULTI_STEP_REASONING_FIELD_READY_NOT_CHAT"
        );
        assert_eq!(report.answer_state, "WATCH_UNSUPPORTED_CLEARANCE");
        assert!(report.constrained_answer.contains("Not proven"));
        assert!(report.constrained_answer.contains("declaration evidence"));
        assert_eq!(report.metrics.grounded_answer_rate, 1.0);
        assert_eq!(report.metrics.context_retention_rate, 1.0);
        assert_eq!(core::mem::size_of::<dialogue_state::DialogueTurn32>(), 32);
        assert!(report.claim_boundary.fixed_dialogue_turn_records);
    }

    #[test]
    fn dialogue_state_rejects_unsupported_clearance_answer() {
        let report = dialogue_state::build_dialogue_state_report();
        assert!(report.trap.rejected);
        assert_eq!(report.trap.trap, "unsupported_clearance_answer");
        assert_eq!(report.trap.unsafe_answer, "Yes, customs cleared the goods.");
        assert_eq!(report.metrics.unsupported_answer_reject_rate, 1.0);
        assert!(!report.claim_boundary.multi_turn_chat_ready);
        assert!(!report.claim_boundary.broad_reasoning_proven);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn mini_chat_eval_passes_all_control_cases() {
        let report = mini_chat_eval::build_mini_chat_eval_report();
        assert_eq!(report.roadmap_block, "v861-v950");
        assert_eq!(report.verdict, "MINI_CHAT_EVAL_PASS_NOT_GENERAL_LLM");
        assert_eq!(
            report.dialogue_bridge_state,
            "DIALOGUE_STATE_READY_NOT_CHAT"
        );
        assert_eq!(report.eval_cases.len(), 5);
        assert_eq!(report.metrics.passed_cases, 5);
        assert_eq!(report.metrics.failed_cases, 0);
        assert_eq!(report.metrics.grounded_answer_rate, 1.0);
        assert_eq!(report.metrics.unsupported_reject_rate, 1.0);
        assert_eq!(report.metrics.route_splice_reject_rate, 1.0);
        assert_eq!(report.metrics.surface_exact_rate, 1.0);
        assert_eq!(
            core::mem::size_of::<mini_chat_eval::MiniChatEvalCase32>(),
            32
        );
        assert!(report.claim_boundary.fixed_eval_case_records);
        assert!(report.claim_boundary.mini_chat_candidate);
    }

    #[test]
    fn mini_chat_eval_keeps_hard_claims_false() {
        let report = mini_chat_eval::build_mini_chat_eval_report();
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "unsupported_clearance" && case.passed));
        assert!(report
            .eval_cases
            .iter()
            .any(|case| case.case_id == "route_splice_surface" && case.passed));
        assert!(!report.claim_boundary.full_llm_ready);
        assert!(!report.claim_boundary.multi_turn_chat_ready);
        assert!(!report.claim_boundary.external_corpus_loaded);
        assert!(!report.claim_boundary.broad_reasoning_proven);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn query_wave_extracts_roles_operator_and_polarity() {
        let report =
            query_wave::build_query_wave_report("Has customs cleared the goods?".to_string());
        assert_eq!(report.roadmap_block, "v951-v1000");
        assert_eq!(report.verdict, "QUERY_WAVE_READY_NOT_FIELD_MATURE");
        assert_eq!(report.top_route_hint, "customs-clearance-status");
        assert_eq!(report.question_polarity, "question_status");
        assert!(report
            .role_excitations
            .iter()
            .any(|role| role.role == "actor:customs" && role.energy > 0));
        assert!(report
            .operator_excitations
            .iter()
            .any(|operator| operator.operator == "clearance_status" && operator.energy > 0));
        assert_eq!(core::mem::size_of::<query_wave::QueryWaveRecord32>(), 32);
        assert!(report.claim_boundary.fixed_query_wave_records);
    }

    #[test]
    fn query_wave_keeps_paraphrases_on_same_route() {
        let report =
            query_wave::build_query_wave_report("Has customs cleared the goods?".to_string());
        assert_eq!(report.metrics.paraphrase_route_recall, 1.0);
        assert_eq!(report.metrics.role_hint_accuracy, 1.0);
        assert_eq!(report.metrics.operator_hint_accuracy, 1.0);
        assert_eq!(report.metrics.assertion_reject_rate, 1.0);
        assert!(report.paraphrase_eval.iter().all(|case| case.passed));
        assert!(!report.claim_boundary.full_field_mature);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn multi_peak_field_selects_stable_customs_peak() {
        let report = multi_peak_field::build_multi_peak_field_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.roadmap_block, "v1001-v1060");
        assert_eq!(report.verdict, "MULTI_PEAK_FIELD_READY_NOT_ANSWER");
        assert_eq!(report.field_state, "STABLE_PEAK");
        assert_eq!(report.top_peak.route, "customs-clearance-status");
        assert!(report.metrics.peak_margin >= multi_peak_field::MIN_STABLE_MARGIN);
        assert_eq!(
            core::mem::size_of::<multi_peak_field::FieldPeakRecord32>(),
            32
        );
        assert!(report.claim_boundary.fixed_peak_records);
    }

    #[test]
    fn multi_peak_field_detects_contested_and_no_answer_states() {
        let report = multi_peak_field::build_multi_peak_field_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.metrics.stable_peak_accuracy, 1.0);
        assert_eq!(report.metrics.contested_detection_rate, 1.0);
        assert_eq!(report.metrics.no_answer_detection_rate, 1.0);
        assert_eq!(report.metrics.route_leakage_reject_rate, 1.0);
        assert!(report.eval_cases.iter().all(|case| case.passed));
        assert!(!report.claim_boundary.safe_to_answer);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn lens_scan_blocks_answer_when_evidence_lens_fails() {
        let report =
            lens_scan::build_lens_scan_report("Has customs cleared the goods?".to_string());
        assert_eq!(report.roadmap_block, "v1061-v1140");
        assert_eq!(report.verdict, "LENS_SCAN_READY_NOT_ANSWER");
        assert_eq!(report.field_bridge_state, "STABLE_PEAK");
        assert_eq!(report.answer_decision, "ANSWER_BLOCKED_BY_LENSES");
        assert!(report
            .lenses
            .iter()
            .any(|lens| lens.lens == "evidence" && lens.state == "WATCH"));
        assert_eq!(core::mem::size_of::<lens_scan::LensRecord32>(), 32);
    }

    #[test]
    fn lens_scan_reports_agreement_without_chat_claim() {
        let report =
            lens_scan::build_lens_scan_report("Has customs cleared the goods?".to_string());
        assert_eq!(report.metrics.role_lens_pass_rate, 1.0);
        assert_eq!(report.metrics.evidence_block_rate, 1.0);
        assert_eq!(report.metrics.answer_block_rate, 1.0);
        assert!(report.metrics.lens_agreement_rate > 0.5);
        assert!(report.claim_boundary.fixed_lens_records);
        assert!(!report.claim_boundary.safe_to_answer);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn mature_anti_wave_compiles_blocking_lenses_into_local_lanes() {
        let report = mature_anti_wave::build_mature_anti_wave_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(report.roadmap_block, "v1141-v1210");
        assert_eq!(report.verdict, "MATURE_ANTI_WAVE_READY_NOT_ANSWER");
        assert_eq!(report.lens_bridge_verdict, "LENS_SCAN_READY_NOT_ANSWER");
        assert_eq!(report.metrics.lane_count, 3);
        assert_eq!(report.metrics.evidence_lane_rate, 1.0);
        assert_eq!(report.metrics.causal_lane_rate, 1.0);
        assert_eq!(report.metrics.answer_lane_rate, 1.0);
        assert!(
            report.field_after_anti.suppress_total
                > report.field_after_anti.support_preserved_total
        );
        assert_eq!(
            core::mem::size_of::<mature_anti_wave::AntiLaneRecord32>(),
            32
        );
    }

    #[test]
    fn mature_anti_wave_does_not_grant_answer_permission() {
        let report = mature_anti_wave::build_mature_anti_wave_report(
            "Has customs cleared the goods?".to_string(),
        );
        assert_eq!(
            report.field_after_anti.answer_decision,
            "ANSWER_BLOCKED_BY_ANTI_WAVE"
        );
        assert_eq!(report.metrics.unsafe_answer_rate, 0.0);
        assert!(report.claim_boundary.fixed_anti_lane_records);
        assert!(report.claim_boundary.local_suppression_only);
        assert!(!report.claim_boundary.safe_to_answer);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn evidence_proof_blocks_missing_evidence() {
        let report = evidence_proof::build_evidence_proof_report(
            "Has customs cleared the goods?".to_string(),
            "missing".to_string(),
        );
        assert_eq!(report.roadmap_block, "v1211-v1280");
        assert_eq!(report.verdict, "EVIDENCE_PROOF_READY_NOT_ANSWER");
        assert_eq!(report.proof_state, "EVIDENCE_MISSING");
        assert_eq!(report.answer_permission, "ANSWER_BLOCKED_BY_EVIDENCE");
        assert_eq!(report.metrics.missing_evidence_block_rate, 1.0);
        assert_eq!(
            core::mem::size_of::<evidence_proof::EvidenceProofRecord32>(),
            32
        );
        assert!(!report.claim_boundary.local_answer_permission);
        assert!(!report.claim_boundary.safe_to_answer);
    }

    #[test]
    fn evidence_proof_allows_only_bound_evidence_candidate() {
        let report = evidence_proof::build_evidence_proof_report(
            "Has customs cleared the goods?".to_string(),
            "release-confirmed".to_string(),
        );
        assert_eq!(report.verdict, "EVIDENCE_PROOF_LOCAL_ANSWER_CANDIDATE");
        assert_eq!(report.proof_state, "EVIDENCE_BOUND");
        assert_eq!(report.answer_permission, "LOCAL_ANSWER_PERMISSION");
        assert_eq!(report.metrics.evidence_binding_rate, 1.0);
        assert_eq!(report.metrics.route_match_rate, 1.0);
        assert_eq!(report.metrics.unsafe_answer_rate, 0.0);
        assert!(report.negative_control.passed);
        assert!(report.claim_boundary.local_answer_permission);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn answer_surface_materializes_not_proven_when_evidence_is_missing() {
        let report = answer_surface::build_answer_surface_report(
            "Has customs cleared the goods?".to_string(),
            "missing".to_string(),
        );
        assert_eq!(report.roadmap_block, "v1281-v1350");
        assert_eq!(report.verdict, "ANSWER_SURFACE_NOT_PROVEN");
        assert_eq!(report.answer_state, "NOT_PROVEN_ANSWER");
        assert!(report.answer_text.contains("Not proven"));
        assert_eq!(report.metrics.unsupported_confirmation_rate, 0.0);
        assert_eq!(
            core::mem::size_of::<answer_surface::AnswerSurfaceRecord32>(),
            32
        );
    }

    #[test]
    fn answer_surface_uses_evidence_bound_template_without_chat_claim() {
        let report = answer_surface::build_answer_surface_report(
            "Has customs cleared the goods?".to_string(),
            "release-confirmed".to_string(),
        );
        assert_eq!(report.verdict, "ANSWER_SURFACE_LOCAL_CANDIDATE");
        assert_eq!(report.answer_state, "LOCAL_EVIDENCE_BOUND_ANSWER");
        assert!(report.answer_text.contains("evidence ref 7001"));
        assert_eq!(report.metrics.constrained_template_rate, 1.0);
        assert_eq!(report.metrics.evidence_ref_copy_rate, 1.0);
        assert!(!report.claim_boundary.free_form_generation);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn field_feedback_reinforces_accepted_evidence_bound_surface() {
        let report = field_feedback::build_field_feedback_report(
            "Has customs cleared the goods?".to_string(),
            "release-confirmed".to_string(),
            "accept".to_string(),
        );
        assert_eq!(report.roadmap_block, "v1351-v1420");
        assert_eq!(report.verdict, "FIELD_FEEDBACK_REINFORCED");
        assert_eq!(report.feedback_state, "FEEDBACK_ACCEPTED");
        assert_eq!(report.memory_effect, "reinforce_evidence_bound_route");
        assert_eq!(report.metrics.accept_reinforcement_rate, 1.0);
        assert_eq!(
            core::mem::size_of::<field_feedback::FieldFeedbackRecord32>(),
            32
        );
    }

    #[test]
    fn field_feedback_rejected_surface_writes_local_anti_memory() {
        let report = field_feedback::build_field_feedback_report(
            "Has customs cleared the goods?".to_string(),
            "release-confirmed".to_string(),
            "reject".to_string(),
        );
        assert_eq!(report.verdict, "FIELD_FEEDBACK_SUPPRESSED");
        assert_eq!(report.feedback_state, "FEEDBACK_REJECTED");
        assert_eq!(report.memory_effect, "write_local_anti_memory");
        assert_eq!(report.metrics.reject_suppression_rate, 1.0);
        assert!(!report.claim_boundary.persistent_training_done);
        assert!(!report.claim_boundary.chat_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn applied_feedback_memory_can_feed_next_field_pass() {
        let report = field_runtime::build_applied_feedback_memory_report(
            "Has customs cleared the goods?".to_string(),
            "release-confirmed".to_string(),
            "accept".to_string(),
        );
        assert_eq!(report.roadmap_block, "v1421-v1480");
        assert_eq!(report.verdict, "FEEDBACK_MEMORY_READY");
        assert_eq!(report.metrics.record_count, 1);
        assert_eq!(report.metrics.reinforce_count, 1);
        assert!(report.claim_boundary.can_feed_next_field_pass);
        assert_eq!(
            core::mem::size_of::<field_runtime::AppliedMemoryRecord32>(),
            32
        );
    }

    #[test]
    fn feedback_aware_field_changes_next_pass_scores() {
        let reinforced = field_runtime::build_feedback_aware_field_report(
            "Has customs cleared the goods?".to_string(),
            "accept".to_string(),
        );
        let suppressed = field_runtime::build_feedback_aware_field_report(
            "Has customs cleared the goods?".to_string(),
            "reject".to_string(),
        );
        assert_eq!(reinforced.roadmap_block, "v1481-v1540");
        assert!(reinforced.metrics.adjusted_top_score > reinforced.metrics.baseline_top_score);
        assert!(suppressed.metrics.adjusted_top_score < suppressed.metrics.baseline_top_score);
        assert_eq!(suppressed.metrics.unsafe_answer_rate, 0.0);
        assert!(!reinforced.claim_boundary.persistent_training_done);
    }

    #[test]
    fn applied_anti_memory_suppresses_false_route_and_preserves_true_route() {
        let report = field_runtime::build_applied_anti_memory_report();
        assert_eq!(report.roadmap_block, "v1541-v1600");
        assert_eq!(report.verdict, "APPLIED_ANTI_MEMORY_READY");
        assert!(report.claim_boundary.suppresses_false_route);
        assert!(report.claim_boundary.preserves_true_route);
        assert!(!report.claim_boundary.global_memory_deleted);
    }

    #[test]
    fn persistent_memory_store_writes_reusable_packet() {
        let path = std::env::temp_dir().join("llmwave-big-memory-test.json");
        let report = field_runtime::build_persistent_memory_store_report(
            &path,
            "apply".to_string(),
            "accept".to_string(),
        )
        .expect("memory store writes");
        assert_eq!(report.roadmap_block, "v1601-v1660");
        assert_eq!(report.verdict, "PERSISTENT_MEMORY_STORE_READY");
        assert_eq!(report.store.record_count, 1);
        assert!(report.claim_boundary.reusable_across_process);
        assert!(path.exists());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn learning_eval_proves_feedback_changes_fixture_field() {
        let report = field_runtime::build_learning_eval_report();
        assert_eq!(report.roadmap_block, "v1661-v1720");
        assert_eq!(report.verdict, "LEARNING_EVAL_PASS_FIXTURE");
        assert!(report.metrics.accepted_route_lift > 0);
        assert!(report.metrics.rejected_route_suppression > 0);
        assert_eq!(report.metrics.regression_rate, 0.0);
        assert!(!report.claim_boundary.broad_learning_proven);
    }

    #[test]
    fn memory_consolidation_controls_duplicate_growth() {
        let report = field_runtime::build_memory_consolidate_report();
        assert_eq!(report.roadmap_block, "v1721-v1780");
        assert!(report.records_after < report.records_before);
        assert!(report.memory_bytes_after < report.memory_bytes_before);
        assert!(report.claim_boundary.duplicate_growth_controlled);
        assert!(!report.claim_boundary.conflict_auto_resolved);
    }

    #[test]
    fn runtime_pipeline_runs_full_fixture_chain() {
        let report = field_runtime::build_runtime_pipeline_report(
            "release-confirmed".to_string(),
            "accept".to_string(),
        );
        assert_eq!(report.roadmap_block, "v1781-v1840");
        assert_eq!(report.verdict, "RUNTIME_PIPELINE_READY_FIXTURE");
        assert_eq!(report.final_state, "LOCAL_EVIDENCE_BOUND_PIPELINE");
        assert!(report.claim_boundary.full_pipeline_implemented);
        assert!(!report.claim_boundary.free_form_generation);
    }

    #[test]
    fn core_eval_reaches_runtime_ready_fixture_without_llm_claims() {
        let report = field_runtime::build_core_eval_report();
        assert_eq!(report.roadmap_block, "v1841-v1900");
        assert_eq!(report.verdict, "CORE_RUNTIME_READY_FIXTURE");
        assert!(report.criteria.feedback_applied_to_next_run);
        assert!(report.criteria.anti_memory_suppresses_rejected_shortcut);
        assert!(report.criteria.memory_persists_across_process_restart);
        assert!(report.claim_boundary.core_runtime_ready);
        assert!(!report.claim_boundary.full_llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }

    #[test]
    fn lexical_birth_mechanism_uses_literature_stages() {
        let report = lexical_birth::build_lexical_birth_report();
        assert_eq!(report.roadmap_block, "v246-v252");
        assert_eq!(report.verdict, "LEXICAL_BIRTH_MECHANISM_READY");
        for stage in [
            "segmentation",
            "fast_mapping",
            "cross_situational_convergence",
            "usage_exemplar_strengthening",
            "grammar_integration",
            "attractor_cleanup",
            "anti_confusion",
        ] {
            assert!(report.birth_stages.iter().any(|item| item.stage == stage));
        }
    }

    #[test]
    fn lexical_birth_accepts_only_stable_nonconfused_sample() {
        let report = lexical_birth::build_lexical_birth_report();
        assert_eq!(report.sample.gate.verdict, "WORD_ACCEPTED");
        assert!(report.sample.binding_record.is_some());
        assert_eq!(
            report.rejection_control.gate.verdict,
            "WORD_REJECTED_OR_WAITING"
        );
        assert!(report.rejection_control.binding_record.is_none());
        assert!(!report.claim_boundary.corpus_proven);
        assert!(!report.claim_boundary.generator_ready);
        assert!(!report.claim_boundary.nonlinear_density_proven);
    }

    #[test]
    fn lexical_birth_records_are_fixed_size_boundaries() {
        let report = lexical_birth::build_lexical_birth_report();
        assert!(report
            .record_formats
            .iter()
            .any(|record| { record.name == "LexicalBirthCandidate32" && record.bytes == 32 }));
        assert!(report
            .record_formats
            .iter()
            .any(|record| { record.name == "LexicalBindingRecord32" && record.bytes == 32 }));
    }

    #[test]
    fn lexical_birth_surface_is_produced_not_token_string_lookup() {
        let report = lexical_birth::build_lexical_birth_report();
        assert!(report
            .surface_production
            .primary_rule
            .contains("do_not_store_words_as_token_id_to_string"));
        assert!(report
            .surface_production
            .production_layers
            .iter()
            .any(|layer| layer.layer == "morpheme_atoms"));
        assert!(report
            .surface_production
            .production_layers
            .iter()
            .any(|layer| layer.layer == "evidence_copy_span"));
        assert!(!report
            .next_engine_steps
            .iter()
            .any(|step| step.contains("token_id_to_utf8")));
    }

    #[test]
    fn surface_production_records_are_fixed_size_boundaries() {
        assert_eq!(
            core::mem::size_of::<surface_production::SurfaceAtom16>(),
            16
        );
        assert_eq!(
            core::mem::size_of::<surface_production::SurfaceProgram32>(),
            32
        );
        assert_eq!(
            core::mem::size_of::<surface_production::EvidenceCopySpan24>(),
            24
        );
        assert_eq!(
            core::mem::size_of::<surface_production::SurfaceProductionCandidate32>(),
            32
        );
    }

    #[test]
    fn surface_production_selects_composition_before_flat_lookup() {
        let report = surface_production::build_surface_production_report();
        assert_eq!(report.roadmap_block, "v253-v260");
        assert_eq!(report.verdict, "SURFACE_PRODUCTION_READY");
        assert_eq!(report.selected.production_path, "surface_program");
        assert!(report
            .production_law
            .primary_rule
            .contains("do_not_store_words_as_token_id_to_utf8"));
        assert!(report
            .atoms
            .iter()
            .any(|atom| atom.layer == "morpheme_atoms"));
        assert!(report
            .copy_spans
            .iter()
            .any(|span| span.role == "exact rare form recovery"));
        assert!(!report.claim_boundary.real_corpus_trained);
        assert!(!report.claim_boundary.free_form_spelling_proven);
        assert!(!report.claim_boundary.nonlinear_surface_memory_proven);
    }

    #[test]
    fn surface_reconstruct_materializes_all_three_paths() {
        let report = surface_reconstruct::build_surface_reconstruct_report();
        assert_eq!(report.roadmap_block, "v261-v270");
        assert_eq!(report.verdict, "SURFACE_RECONSTRUCT_READY");
        assert_eq!(report.eval.cases, 4);
        assert_eq!(report.eval.exact_matches, 4);
        assert_eq!(report.eval.exact_match_rate, 1.0);
        assert!(report
            .cases
            .iter()
            .any(|case| case.path == "surface_program" && case.reconstructed == "invoice"));
        assert!(report.cases.iter().any(|case| {
            case.path == "evidence_copy_span" && case.reconstructed == "PI-HL-RLTG-GZ-20260611-03"
        }));
        assert!(report
            .cases
            .iter()
            .any(|case| case.path == "byte_fallback" && case.reconstructed == "zxq"));
    }

    #[test]
    fn surface_reconstruct_keeps_density_claim_unproven() {
        let report = surface_reconstruct::build_surface_reconstruct_report();
        assert_eq!(
            report.eval.state,
            "TOY_RECONSTRUCTION_PASS_NOT_DENSITY_PROOF"
        );
        assert!(report.eval.program_reuse_ratio > 1.0);
        assert!(report.bank_summary.total_surface_memory_bytes > 0);
        assert!(!report.bank_summary.hot_core_contains_utf8);
        assert!(report.claim_boundary.hot_core_utf8_free);
        assert!(!report.claim_boundary.real_corpus_trained);
        assert!(!report.claim_boundary.free_form_spelling_proven);
        assert!(!report.claim_boundary.nonlinear_surface_memory_proven);
    }

    #[test]
    fn surface_corpus_eval_shows_family_template_density_candidate() {
        let report = surface_corpus_eval::build_surface_corpus_eval_report();
        assert_eq!(report.roadmap_block, "v271-v280");
        assert_eq!(report.verdict, "SURFACE_DENSITY_CANDIDATE_NOT_PROVEN");
        assert_eq!(report.corpus.productive_forms, 512);
        assert_eq!(report.reconstruction.exact_match_rate, 1.0);
        assert_eq!(report.reconstruction.held_out_exact_match_rate, 1.0);
        assert!(report.baselines.family_template_bytes < report.baselines.direct_lookup_bytes);
        assert!(report.baselines.family_template_bytes < report.baselines.per_form_program_bytes);
        assert!(report.baselines.family_vs_direct_saving_ratio > 0.0);
        assert_eq!(report.family_reuse.state, "FAMILY_REUSE_VISIBLE");
    }

    #[test]
    fn surface_corpus_eval_keeps_nonlinear_claim_unproven() {
        let report = surface_corpus_eval::build_surface_corpus_eval_report();
        assert!(report.verdict_boundary.useful_density_candidate);
        assert!(!report.verdict_boundary.nonlinear_surface_memory_proven);
        assert!(!report.verdict_boundary.real_corpus_trained);
        assert!(!report.verdict_boundary.free_form_spelling_proven);
        assert!(report
            .sample_cases
            .iter()
            .any(|case| case.held_out && case.exact_match));
    }

    #[test]
    fn surface_bank_build_promotes_observed_families() {
        let report = surface_bank_build::build_surface_bank_build_report();
        assert_eq!(report.roadmap_block, "v281-v290");
        assert_eq!(report.verdict, "SURFACE_BANK_BUILD_READY_NOT_REAL_TRAINING");
        assert_eq!(report.accepted_families.len(), 3);
        assert_eq!(report.eval.held_out_exact_match_rate, 1.0);
        assert!(report.accepted_families.iter().any(|family| family
            .held_out_reconstructions
            .contains(&"routing".to_string())));
        assert!(report
            .rejected_fragments
            .iter()
            .any(|fragment| fragment.path == "evidence_copy_span"));
    }

    #[test]
    fn surface_bank_build_keeps_claims_honest() {
        let report = surface_bank_build::build_surface_bank_build_report();
        assert_eq!(
            report.eval.state,
            "OBSERVED_BANK_BUILD_PASS_NOT_DENSITY_PROOF"
        );
        assert!(report.claim_boundary.useful_density_candidate);
        assert!(!report.claim_boundary.real_corpus_trained);
        assert!(!report.claim_boundary.nonlinear_surface_memory_proven);
        assert!(!report.claim_boundary.free_form_spelling_proven);
        assert!(!report.bank_summary.hot_core_contains_utf8);
    }

    #[test]
    fn surface_bank_validate_rejects_false_families() {
        let report = surface_bank_validate::build_surface_bank_validate_report();
        assert_eq!(report.roadmap_block, "v291-v300");
        assert_eq!(
            report.verdict,
            "SURFACE_BANK_VALIDATE_READY_NOT_REAL_TRAINING"
        );
        assert_eq!(report.metrics.positive_accept_rate, 1.0);
        assert_eq!(report.metrics.negative_reject_rate, 1.0);
        assert_eq!(report.metrics.false_family_rate, 0.0);
        assert!(report
            .negative_controls
            .iter()
            .any(|control| { control.case_id == "invoiceing_trap" && !control.accepted }));
        assert!(report
            .negative_controls
            .iter()
            .any(|control| { control.case_id == "rare_code_family_trap" && !control.accepted }));
    }

    #[test]
    fn surface_bank_validate_keeps_order_and_claims_honest() {
        let report = surface_bank_validate::build_surface_bank_validate_report();
        assert_eq!(
            report.shuffle_stability.state,
            "ORDER_STABLE_ON_EMBEDDED_CORPUS"
        );
        assert_eq!(report.shuffle_stability.stability_rate, 1.0);
        assert_eq!(report.shuffle_stability.unstable_family_count, 0);
        assert!(report.claim_boundary.validation_passed);
        assert!(!report.claim_boundary.real_corpus_trained);
        assert!(!report.claim_boundary.nonlinear_surface_memory_proven);
        assert!(!report.claim_boundary.free_form_spelling_proven);
        assert!(!report.claim_boundary.order_invariance_proven);
    }

    #[test]
    fn surface_bank_fixture_loads_external_corpus() {
        let report = surface_bank_fixture::build_surface_bank_fixture_report(std::path::Path::new(
            "examples/llmwave-big-surface-corpus.json",
        ))
        .expect("fixture report");
        assert_eq!(report.roadmap_block, "v301-v310");
        assert_eq!(
            report.verdict,
            "SURFACE_BANK_FIXTURE_READY_NOT_REAL_TRAINING"
        );
        assert_eq!(report.corpus.family_count, 6);
        assert!(report.metrics.fixture_loaded);
        assert_eq!(report.metrics.positive_exact_match_rate, 1.0);
        assert_eq!(report.metrics.negative_reject_rate, 1.0);
        assert_eq!(report.metrics.rare_copy_span_rate, 1.0);
    }

    #[test]
    fn surface_bank_fixture_keeps_claims_honest() {
        let report = surface_bank_fixture::build_surface_bank_fixture_report(std::path::Path::new(
            "examples/llmwave-big-surface-corpus.json",
        ))
        .expect("fixture report");
        assert_eq!(
            report.metrics.state,
            "EXTERNAL_FIXTURE_PASS_NOT_GENERAL_PROOF"
        );
        assert!(report.claim_boundary.external_fixture_loaded);
        assert!(!report.claim_boundary.real_corpus_trained);
        assert!(!report.claim_boundary.nonlinear_surface_memory_proven);
        assert!(!report.claim_boundary.free_form_spelling_proven);
        assert!(report.baselines.total_bank_bytes > 0);
    }

    #[test]
    fn surface_raw_induce_finds_russian_families_from_forms() {
        let report = surface_raw_induce::build_surface_raw_induce_report(std::path::Path::new(
            "examples/llmwave-big-raw-surface-corpus-ru.json",
        ))
        .expect("raw induce report");
        assert_eq!(report.roadmap_block, "v311-v320");
        assert_eq!(report.verdict, "SURFACE_RAW_INDUCE_READY_NOT_REAL_TRAINING");
        assert_eq!(report.metrics.induced_family_count, 6);
        assert_eq!(report.metrics.expected_root_recall, 1.0);
        assert_eq!(report.metrics.held_out_exact_match_rate, 1.0);
        assert!(report
            .induced_families
            .iter()
            .any(|family| family.root == "декларац"));
    }

    #[test]
    fn surface_raw_induce_keeps_claims_honest() {
        let report = surface_raw_induce::build_surface_raw_induce_report(std::path::Path::new(
            "examples/llmwave-big-raw-surface-corpus-ru.json",
        ))
        .expect("raw induce report");
        assert_eq!(report.metrics.negative_reject_rate, 1.0);
        assert_eq!(report.metrics.false_family_rate, 0.0);
        assert_eq!(report.metrics.state, "RAW_INDUCTION_PASS_NOT_GENERAL_PROOF");
        assert!(report.claim_boundary.raw_forms_used);
        assert!(!report.claim_boundary.roots_given_to_inducer);
        assert!(!report.claim_boundary.real_corpus_trained);
        assert!(!report.claim_boundary.nonlinear_surface_memory_proven);
    }

    #[test]
    fn surface_raw_induce_rejects_near_root_noise() {
        let report = surface_raw_induce::build_surface_raw_induce_report(std::path::Path::new(
            "examples/llmwave-big-raw-surface-corpus-ru-noisy.json",
        ))
        .expect("noisy raw induce report");
        assert_eq!(report.roadmap_block, "v321-v330");
        assert_eq!(report.metrics.induced_family_count, 6);
        assert_eq!(report.metrics.expected_root_recall, 1.0);
        assert_eq!(report.metrics.noise_reject_rate, 1.0);
        assert_eq!(report.metrics.false_family_rate, 0.0);
        assert_eq!(
            report.metrics.state,
            "NOISY_RAW_INDUCTION_PASS_NOT_GENERAL_PROOF"
        );
        assert!(report
            .rejected_collision_roots
            .iter()
            .any(|root| root.root == "счетчик"));
        assert!(report
            .rejected_collision_roots
            .iter()
            .any(|root| root.root == "договоренност"));
        assert!(!report
            .induced_families
            .iter()
            .any(|family| family.root == "счетчик"));
    }

    #[test]
    fn surface_raw_induce_derives_suffix_inventory_from_raw_forms() {
        let report = surface_raw_induce::build_surface_raw_induce_report(std::path::Path::new(
            "examples/llmwave-big-raw-surface-corpus-ru-derived.json",
        ))
        .expect("derived raw induce report");
        assert_eq!(report.roadmap_block, "v331-v360");
        assert_eq!(
            report.corpus.suffix_inventory_source,
            "derived_from_raw_forms"
        );
        assert!(report.derived_suffix_inventory.enabled);
        assert_eq!(report.metrics.manual_suffix_count, 0);
        assert!(report.metrics.derived_suffix_count >= 8);
        assert_eq!(report.metrics.induced_family_count, 9);
        assert_eq!(report.metrics.expected_root_recall, 1.0);
        assert_eq!(report.metrics.held_out_exact_match_rate, 1.0);
        assert_eq!(report.metrics.noise_reject_rate, 1.0);
        assert_eq!(report.metrics.false_family_rate, 0.0);
        assert_eq!(
            report.metrics.state,
            "DERIVED_SUFFIX_RAW_INDUCTION_PASS_NOT_GENERAL_PROOF"
        );
        assert!(report
            .derived_suffix_inventory
            .suffixes
            .iter()
            .any(|suffix| suffix.suffix == "е"));
        assert!(report
            .induced_families
            .iter()
            .any(|family| family.root == "деклараци"));
        assert!(!report
            .induced_families
            .iter()
            .any(|family| family.root == "счетчик"));
    }

    #[test]
    fn write_report_keeps_nonlinear_claim_unproven() {
        let report = write::build_write_report();
        assert_eq!(report.roadmap_block, "v191-v205");
        assert_eq!(report.verdict, "RESIDUAL_SAVING");
        assert_eq!(
            report.write_curve.state,
            "SYNTHETIC_CONTRACT_CURVE_NOT_NONLINEAR_PROOF"
        );
        assert!(report.write_curve.residual_saving_ratio > 0.0);
        assert!(report.compression_safety.safe);
    }

    #[test]
    fn write_residual_v1_has_expected_size_and_fields() {
        let report = write::build_write_report();
        assert_eq!(report.residual_format_v1.bytes, 20);
        assert_eq!(report.write_decision.residual.schema_id, 101);
        assert_eq!(report.write_decision.residual.evidence_ref, 10_001);
        assert!(report
            .residual_format_v1
            .fields
            .contains(&"phase_delta:i16"));
    }

    #[test]
    fn consolidation_preserves_conflicts_and_safety() {
        let report = consolidation::build_consolidation_report();
        assert_eq!(report.roadmap_block, "v206-v218");
        assert_eq!(report.verdict, "CONSOLIDATION_SAFE");
        assert_eq!(report.conflict_preservation.state, "CONFLICTS_PRESERVED");
        assert!(report.eval.safe);
        assert_eq!(report.duplicate_merge.new_records_created, 0);
    }

    #[test]
    fn consolidation_reduces_memory_without_role_safety_regression() {
        let report = consolidation::build_consolidation_report();
        assert!(report.eval.after.memory_bytes < report.eval.before.memory_bytes);
        assert!(report.eval.after.role_safety >= report.eval.before.role_safety);
        assert!(report.eval.after.false_positives <= report.eval.before.false_positives);
        assert!(report.cognitive_compression_score > 1.0);
    }

    #[test]
    fn big_eval_reports_cognitive_lift_without_final_claims() {
        let report = eval::build_big_eval_report();
        assert_eq!(report.roadmap_block, "v219-v230");
        assert_eq!(report.verdict, "COGNITIVE_LIFT");
        assert!(report.cases.iter().all(|case| case.passed));
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.candidate_ready);
    }

    #[test]
    fn big_eval_covers_required_task_families() {
        let report = eval::build_big_eval_report();
        for task_type in [
            "inference",
            "role_swap",
            "contradiction",
            "multi_hop",
            "missing_evidence",
            "generation",
            "style",
            "code",
            "business",
        ] {
            assert!(report.cases.iter().any(|case| case.task_type == task_type));
        }
        assert!(report.cognitive_score.total >= 0.8);
    }

    #[test]
    fn runtime_product_reports_v1_candidate_without_llm_claim() {
        let report =
            loader::build_runtime_product_report("supplier invoice payment customs".to_string());
        assert_eq!(report.roadmap_block, "v231-v245");
        assert_eq!(report.verdict, "LLMWAVE_BIG_V1_CANDIDATE");
        assert!(report.v1_criteria.large_long_term_memory);
        assert!(report.v1_criteria.small_active_core);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.cache_only_execution_proven);
    }

    #[test]
    fn active65k_runtime_claim_stays_local_only() {
        let report = readiness::build_claim_gate_report(readiness::ClaimGateKind::Active65kRuntime);
        assert_eq!(report.claim, "active-65k-runtime");
        assert_eq!(report.verdict, "CLAIM_ALLOWED_LOCAL_RUNTIME_ONLY");
        assert!(report.allowed);
        assert!(report.claim_boundary.active_65k_runtime_ready);
        assert!(!report.claim_boundary.broad_chat_llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(report
            .missing_evidence
            .contains(&"general LLM/chat readiness"));
    }

    #[test]
    fn training_compiles_real_text_into_field_records() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("llmwave-big-train-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        let corpus = dir.join("corpus.txt");
        let out = dir.join("artifact.json");
        fs::write(
            &corpus,
            "Honglu issues invoice. invoice requires payment. \
             payment supports customs declaration. Honglu issues invoice. \
             declaration requires evidence. evidence blocks unsupported answer.",
        )
        .unwrap();

        let report = training::compile_training_corpus(training::TrainingConfig {
            inputs: vec![corpus],
            out: Some(out.clone()),
            vocab_cap: 128,
            transition_cap: 256,
            active_chunk_cap: 64,
            chunk_tokens: 8,
            hot_budget_bytes: 64 * 1024,
            max_file_bytes: 1024 * 1024,
            extensions: training::parse_extensions("txt"),
        })
        .unwrap();

        assert_eq!(report.verdict, "TRAINING_ARTIFACT_READY_NOT_LLM");
        assert!(report.corpus.tokens_seen > 20);
        assert!(report.field_budget.transition_records > 0);
        assert!(report.field_budget.fits_hot_budget);
        assert!(report.output.artifact_written);
        assert!(!report.claim_boundary.chat_llm_ready);
        let artifact = training::load_training_artifact(&out).unwrap();
        assert!(!artifact.records.tokens.is_empty());
        assert!(!artifact.records.transitions.is_empty());
        assert_eq!(artifact.version, training::TRAINING_VERSION);
        let ask =
            training::ask_training_artifact(&out, "what requires evidence".to_string(), 3).unwrap();
        assert!(ask.claim_boundary.trained_field_used);
        assert!(!ask.claim_boundary.broad_chat_llm_ready);
        assert!(!ask.field.top_chunk_peaks.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn runtime_product_blocks_contested_field() {
        let report = loader::build_runtime_product_report("role swap conflict".to_string());
        assert_eq!(report.safety.field_state, "FIELD_CONTESTED");
        assert!(!report.safety.safe_to_answer);
    }

    #[test]
    fn big_contract_separates_l2_and_l3() {
        let report = build_contract_report();
        let layers: Vec<_> = report
            .contract
            .layers
            .iter()
            .map(|layer| layer.name)
            .collect();
        assert!(layers.contains(&"L2 Word Field"));
        assert!(layers.contains(&"L3 Schema Field"));
        assert!(report
            .engineering_rules
            .l2_l3_boundary
            .contains(&"l2_word_field_surface_tokens_roots_morphemes_words"));
        assert!(report
            .engineering_rules
            .l2_l3_boundary
            .contains(&"l3_schema_field_operators_roles_routes_schema_cognition"));
    }

    #[test]
    fn bigness_metrics_include_required_nonlinearity_gates() {
        let report = build_contract_report();
        let names: Vec<_> = report
            .bigness_metrics
            .required_metrics
            .iter()
            .map(|metric| metric.name)
            .collect();
        for required in [
            "bytes_per_useful_fact",
            "schema_reuse_ratio",
            "residual_saving_ratio",
            "role_error_rate",
            "false_positive_rate",
            "inference_score",
        ] {
            assert!(names.contains(&required));
        }
    }
}
