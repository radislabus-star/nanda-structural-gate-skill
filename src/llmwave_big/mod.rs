use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;

use super::{nanda_6m, OutputFormat, CORE_VERSION, EXIT_PASS, WAVE_DIM};

pub mod active_core;
pub mod atlas;
pub mod consolidation;
pub mod eval;
pub mod l2_word_field;
pub mod l3_schema_field;
pub mod loader;
pub mod operators;
pub mod residuals;
pub mod schemas;
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
