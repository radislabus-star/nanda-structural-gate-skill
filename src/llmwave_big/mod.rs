use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

use super::{nanda_6m, OutputFormat, CORE_VERSION, EXIT_PASS, WAVE_DIM};

pub mod active_core;
pub mod answer_surface;
pub mod atlas;
pub mod consolidation;
pub mod coupled_decode_loop;
pub mod dialogue_state;
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
pub mod loader;
pub mod mature_anti_wave;
pub mod mini_chat_eval;
pub mod multi_peak_field;
pub mod multi_schema_competition;
pub mod open_surface_generation;
pub mod operators;
pub mod query_wave;
pub mod reasoning_field;
pub mod residuals;
pub mod schema_memory_growth;
pub mod schemas;
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
    /// Print the v206-v218 consolidation/sleep contract.
    Consolidate(LlmwaveBigConsolidateArgs),
    /// Print the v219-v230 Big Cognition Eval report.
    Eval(LlmwaveBigEvalArgs),
    /// Run the v231-v245 runtime product query surface.
    Query(LlmwaveBigQueryArgs),
    /// Compile a real corpus into LLMWave-Big training records.
    Train(LlmwaveBigTrainArgs),
    /// Ask a compiled LLMWave-Big training artifact.
    Ask(LlmwaveBigAskArgs),
    /// Evaluate ask behavior over a compiled training artifact.
    #[command(name = "ask-eval")]
    AskEval(LlmwaveBigAskEvalArgs),
    /// Pack a training artifact into a binary hot Active Core file.
    #[command(name = "pack-hot")]
    PackHot(LlmwaveBigPackHotArgs),
}

#[derive(Parser)]
struct LlmwaveBigContractArgs {
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
struct LlmwaveBigPackHotArgs {
    #[arg(long)]
    artifact: PathBuf,
    #[arg(long)]
    out: PathBuf,
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
    }
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
