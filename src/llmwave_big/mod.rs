use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

use super::{nanda_6m, OutputFormat, CORE_VERSION, EXIT_PASS, WAVE_DIM};

pub mod active_core;
pub mod atlas;
pub mod consolidation;
pub mod eval;
pub mod l2_word_field;
pub mod l3_schema_field;
pub mod lexical_birth;
pub mod loader;
pub mod operators;
pub mod residuals;
pub mod schemas;
pub mod surface_bank_build;
pub mod surface_bank_fixture;
pub mod surface_bank_validate;
pub mod surface_corpus_eval;
pub mod surface_production;
pub mod surface_raw_induce;
pub mod surface_reconstruct;
pub mod symbols;
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
