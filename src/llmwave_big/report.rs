use anyhow::Result;

use super::active_core::ActiveCoreReport;
use super::answer_surface::AnswerSurfaceReport;
use super::atlas::AtlasReport;
use super::consolidation::ConsolidationReport;
use super::coupled_decode_loop::CoupledDecodeLoopReport;
use super::dialogue_state::DialogueStateReport;
use super::eval::BigEvalReport;
use super::evidence_proof::EvidenceProofReport;
use super::field_feedback::FieldFeedbackReport;
use super::hrr_binding::HrrBindingReport;
use super::l2_l3_coupling::L2L3CouplingReport;
use super::l2_word_field::L2WordFieldReport;
use super::l3_schema_bind::L3SchemaBindReport;
use super::lens_scan::LensScanReport;
use super::lexical_birth::LexicalBirthReport;
use super::loader::RuntimeProductReport;
use super::mature_anti_wave::MatureAntiWaveReport;
use super::mini_chat_eval::MiniChatEvalReport;
use super::multi_peak_field::MultiPeakFieldReport;
use super::multi_schema_competition::MultiSchemaCompetitionReport;
use super::open_surface_generation::OpenSurfaceGenerationReport;
use super::query_wave::QueryWaveReport;
use super::reasoning_field::ReasoningFieldReport;
use super::schema_memory_growth::SchemaMemoryGrowthReport;
use super::surface_bank_build::SurfaceBankBuildReport;
use super::surface_bank_fixture::SurfaceBankFixtureReport;
use super::surface_bank_validate::SurfaceBankValidateReport;
use super::surface_corpus_eval::SurfaceCorpusEvalReport;
use super::surface_production::SurfaceProductionReport;
use super::surface_raw_induce::SurfaceRawInduceReport;
use super::surface_reconstruct::SurfaceReconstructReport;
use super::write::WriteReport;
use super::LlmwaveBigReport;
use crate::OutputFormat;

pub(crate) fn print_contract_report(
    report: &LlmwaveBigReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_contract_text(report),
        OutputFormat::Md => print_contract_md(report),
    }
    Ok(())
}

pub(crate) fn print_atlas_report(report: &AtlasReport, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_atlas_text(report),
        OutputFormat::Md => print_atlas_md(report),
    }
    Ok(())
}

pub(crate) fn print_active_core_report(
    report: &ActiveCoreReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_active_core_text(report),
        OutputFormat::Md => print_active_core_md(report),
    }
    Ok(())
}

pub(crate) fn print_l2_word_field_report(
    report: &L2WordFieldReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_l2_word_field_text(report),
        OutputFormat::Md => print_l2_word_field_md(report),
    }
    Ok(())
}

pub(crate) fn print_hrr_binding_report(
    report: &HrrBindingReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_hrr_binding_text(report),
        OutputFormat::Md => print_hrr_binding_md(report),
    }
    Ok(())
}

pub(crate) fn print_l3_schema_bind_report(
    report: &L3SchemaBindReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_l3_schema_bind_text(report),
        OutputFormat::Md => print_l3_schema_bind_md(report),
    }
    Ok(())
}

pub(crate) fn print_l2_l3_coupling_report(
    report: &L2L3CouplingReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_l2_l3_coupling_text(report),
        OutputFormat::Md => print_l2_l3_coupling_md(report),
    }
    Ok(())
}

pub(crate) fn print_coupled_decode_loop_report(
    report: &CoupledDecodeLoopReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_coupled_decode_loop_text(report),
        OutputFormat::Md => print_coupled_decode_loop_md(report),
    }
    Ok(())
}

pub(crate) fn print_multi_schema_competition_report(
    report: &MultiSchemaCompetitionReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_multi_schema_competition_text(report),
        OutputFormat::Md => print_multi_schema_competition_md(report),
    }
    Ok(())
}

pub(crate) fn print_schema_memory_growth_report(
    report: &SchemaMemoryGrowthReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_schema_memory_growth_text(report),
        OutputFormat::Md => print_schema_memory_growth_md(report),
    }
    Ok(())
}

pub(crate) fn print_open_surface_generation_report(
    report: &OpenSurfaceGenerationReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_open_surface_generation_text(report),
        OutputFormat::Md => print_open_surface_generation_md(report),
    }
    Ok(())
}

pub(crate) fn print_reasoning_field_report(
    report: &ReasoningFieldReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_reasoning_field_text(report),
        OutputFormat::Md => print_reasoning_field_md(report),
    }
    Ok(())
}

pub(crate) fn print_dialogue_state_report(
    report: &DialogueStateReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_dialogue_state_text(report),
        OutputFormat::Md => print_dialogue_state_md(report),
    }
    Ok(())
}

pub(crate) fn print_mini_chat_eval_report(
    report: &MiniChatEvalReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_mini_chat_eval_text(report),
        OutputFormat::Md => print_mini_chat_eval_md(report),
    }
    Ok(())
}

pub(crate) fn print_query_wave_report(
    report: &QueryWaveReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_query_wave_text(report),
        OutputFormat::Md => print_query_wave_md(report),
    }
    Ok(())
}

pub(crate) fn print_multi_peak_field_report(
    report: &MultiPeakFieldReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_multi_peak_field_text(report),
        OutputFormat::Md => print_multi_peak_field_md(report),
    }
    Ok(())
}

pub(crate) fn print_lens_scan_report(report: &LensScanReport, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_lens_scan_text(report),
        OutputFormat::Md => print_lens_scan_md(report),
    }
    Ok(())
}

pub(crate) fn print_mature_anti_wave_report(
    report: &MatureAntiWaveReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_mature_anti_wave_text(report),
        OutputFormat::Md => print_mature_anti_wave_md(report),
    }
    Ok(())
}

pub(crate) fn print_evidence_proof_report(
    report: &EvidenceProofReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_evidence_proof_text(report),
        OutputFormat::Md => print_evidence_proof_md(report),
    }
    Ok(())
}

pub(crate) fn print_answer_surface_report(
    report: &AnswerSurfaceReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_answer_surface_text(report),
        OutputFormat::Md => print_answer_surface_md(report),
    }
    Ok(())
}

pub(crate) fn print_field_feedback_report(
    report: &FieldFeedbackReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_field_feedback_text(report),
        OutputFormat::Md => print_field_feedback_md(report),
    }
    Ok(())
}

pub(crate) fn print_lexical_birth_report(
    report: &LexicalBirthReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_lexical_birth_text(report),
        OutputFormat::Md => print_lexical_birth_md(report),
    }
    Ok(())
}

pub(crate) fn print_surface_production_report(
    report: &SurfaceProductionReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_surface_production_text(report),
        OutputFormat::Md => print_surface_production_md(report),
    }
    Ok(())
}

pub(crate) fn print_surface_reconstruct_report(
    report: &SurfaceReconstructReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_surface_reconstruct_text(report),
        OutputFormat::Md => print_surface_reconstruct_md(report),
    }
    Ok(())
}

pub(crate) fn print_surface_corpus_eval_report(
    report: &SurfaceCorpusEvalReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_surface_corpus_eval_text(report),
        OutputFormat::Md => print_surface_corpus_eval_md(report),
    }
    Ok(())
}

pub(crate) fn print_surface_bank_build_report(
    report: &SurfaceBankBuildReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_surface_bank_build_text(report),
        OutputFormat::Md => print_surface_bank_build_md(report),
    }
    Ok(())
}

pub(crate) fn print_surface_bank_validate_report(
    report: &SurfaceBankValidateReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_surface_bank_validate_text(report),
        OutputFormat::Md => print_surface_bank_validate_md(report),
    }
    Ok(())
}

pub(crate) fn print_surface_bank_fixture_report(
    report: &SurfaceBankFixtureReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_surface_bank_fixture_text(report),
        OutputFormat::Md => print_surface_bank_fixture_md(report),
    }
    Ok(())
}

pub(crate) fn print_surface_raw_induce_report(
    report: &SurfaceRawInduceReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_surface_raw_induce_text(report),
        OutputFormat::Md => print_surface_raw_induce_md(report),
    }
    Ok(())
}

pub(crate) fn print_write_report(report: &WriteReport, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_write_text(report),
        OutputFormat::Md => print_write_md(report),
    }
    Ok(())
}

pub(crate) fn print_consolidation_report(
    report: &ConsolidationReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_consolidation_text(report),
        OutputFormat::Md => print_consolidation_md(report),
    }
    Ok(())
}

pub(crate) fn print_big_eval_report(report: &BigEvalReport, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_big_eval_text(report),
        OutputFormat::Md => print_big_eval_md(report),
    }
    Ok(())
}

pub(crate) fn print_runtime_product_report(
    report: &RuntimeProductReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => print_runtime_product_text(report),
        OutputFormat::Md => print_runtime_product_md(report),
    }
    Ok(())
}

fn print_contract_text(report: &LlmwaveBigReport) {
    println!("LLMWave-Big Contract");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("state: {}", report.claim_boundary.current_state);
    println!("safe_claim: {}", report.claim_boundary.safe_claim);
    println!(
        "active_core_bytes: {}",
        report.contract.active_core_budget.budget_bytes
    );
    println!(
        "runtime_focus_triads: {}",
        report
            .contract
            .active_core_budget
            .runtime_focus_triad_capacity
    );
    println!("layers:");
    for layer in &report.contract.layers {
        println!("  - {}: {}", layer.name, layer.responsibility);
    }
    println!("required_metrics:");
    for metric in &report.bigness_metrics.required_metrics {
        println!(
            "  - {} ({}, {})",
            metric.name, metric.unit, metric.direction
        );
    }
    println!("forbidden_claims:");
    for claim in &report.claim_boundary.forbidden_claims {
        println!("  - {claim}");
    }
}

fn print_atlas_text(report: &AtlasReport) {
    println!("LLMWave-Big Wave Atlas");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("state: {}", report.state);
    println!("doctor: {}", report.doctor.verdict);
    println!("records:");
    for record in &report.record_formats {
        println!("  - {}: {} bytes", record.name, record.bytes);
    }
    println!("indexes:");
    for index in &report.indexes {
        println!("  - {}: {} -> {}", index.name, index.input, index.output);
    }
    println!("loader:");
    println!(
        "  top_symbols={} top_schemas={} evidence_refs={} fits_active_core={}",
        report.loader_preview.top_symbols.len(),
        report.loader_preview.top_schemas.len(),
        report.loader_preview.evidence_refs.len(),
        report.loader_preview.fits_active_core_contract
    );
}

fn print_active_core_text(report: &ActiveCoreReport) {
    println!("LLMWave-Big Active Core");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("total_bytes: {}", report.budget.total_bytes);
    println!(
        "fits_nanda_6m_budget: {}",
        report.budget.fits_nanda_6m_budget
    );
    println!("packet_records:");
    println!(
        "  schema: {} bytes",
        report.packet_format.schema_record_bytes
    );
    println!(
        "  residual: {} bytes",
        report.packet_format.residual_record_bytes
    );
    println!("cycle:");
    println!(
        "  top_schema={} top_score={} margin={} safe_to_answer={}",
        report.cycle.top_schema_id,
        report.cycle.top_score,
        report.cycle.margin,
        report.cycle.safe_to_answer
    );
}

fn print_l2_word_field_text(report: &L2WordFieldReport) {
    println!("LLMWave-Big L2 Word Field");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("prefix: {}", report.prefix_wave.prefix);
    println!("top_token: {}", report.candidate_cache.top_token_label);
    println!("margin: {}", report.candidate_cache.margin);
    println!("runtime_top_surface: {}", report.runtime_field.top_surface);
    println!("runtime_margin: {}", report.runtime_field.margin);
    println!("runtime_state: {}", report.runtime_field.field_state);
    println!("record_bytes: {}", report.candidate_cache.record_bytes);
    println!(
        "sync: {}/{}",
        report.sync_policy.l2_update, report.sync_policy.l3_update
    );
}

fn print_hrr_binding_text(report: &HrrBindingReport) {
    println!("LLMWave-Big HRR Binding");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("role_recall: {:.3}", report.metrics.role_recall);
    println!("noisy_role_recall: {:.3}", report.metrics.noisy_role_recall);
    println!(
        "collision_reject_rate: {:.3}",
        report.metrics.collision_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_l3_schema_bind_text(report: &L3SchemaBindReport) {
    println!("LLMWave-Big L3 Schema Bind");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("schema_id: {}", report.schema.schema_id);
    println!(
        "schema_role_recall: {:.3}",
        report.metrics.schema_role_recall
    );
    println!("role_error_rate: {:.3}", report.metrics.role_error_rate);
    println!(
        "role_swap_reject_rate: {:.3}",
        report.metrics.role_swap_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_l2_l3_coupling_text(report: &L2L3CouplingReport) {
    println!("LLMWave-Big L2/L3 Coupling");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("prefix: {}", report.l2_probe.prefix);
    println!("role_slot: {}", report.l2_probe.role_slot);
    println!("raw_top: {}", report.l2_probe.raw_top);
    println!("coupled_top: {}", report.l2_probe.coupled_top);
    println!("top_margin: {}", report.rerank.top_margin);
    println!(
        "disagreement_reject_rate: {:.3}",
        report.metrics.disagreement_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_coupled_decode_loop_text(report: &CoupledDecodeLoopReport) {
    println!("LLMWave-Big Coupled Decode Loop");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("bridge_state: {}", report.bridge_state);
    println!("final_sequence: {}", report.final_sequence.join(" "));
    println!("completed_steps: {}", report.metrics.completed_steps);
    println!("sequence_exact: {}", report.metrics.sequence_exact);
    println!(
        "bad_continuation_reject_rate: {:.3}",
        report.metrics.bad_continuation_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_multi_schema_competition_text(report: &MultiSchemaCompetitionReport) {
    println!("LLMWave-Big Multi-Schema Competition");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("decode_bridge_state: {}", report.decode_bridge_state);
    println!("selected_schema_id: {}", report.metrics.selected_schema_id);
    println!("selected_route: {}", report.selected_route.route);
    println!("top_margin: {}", report.metrics.top_margin);
    println!(
        "route_splice_reject_rate: {:.3}",
        report.metrics.route_splice_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_schema_memory_growth_text(report: &SchemaMemoryGrowthReport) {
    println!("LLMWave-Big Schema Memory Growth");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!(
        "competition_bridge_state: {}",
        report.competition_bridge_state
    );
    println!("observed_fact_count: {}", report.observed_fact_count);
    println!("promoted_count: {}", report.memory_metrics.promoted_count);
    println!("rejected_count: {}", report.memory_metrics.rejected_count);
    println!(
        "schema_reuse_ratio: {:.3}",
        report.memory_metrics.schema_reuse_ratio
    );
    println!(
        "false_promotion_rate: {:.3}",
        report.memory_metrics.false_promotion_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_open_surface_generation_text(report: &OpenSurfaceGenerationReport) {
    println!("LLMWave-Big Open Surface Generation");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!(
        "schema_growth_bridge_state: {}",
        report.schema_growth_bridge_state
    );
    println!("selected_route: {}", report.selected_schema.route);
    println!("surface: {}", report.materialized_surface);
    println!("step_count: {}", report.generation_metrics.step_count);
    println!("exact_surface: {}", report.generation_metrics.exact_surface);
    println!(
        "trap_reject_rate: {:.3}",
        report.generation_metrics.trap_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_reasoning_field_text(report: &ReasoningFieldReport) {
    println!("LLMWave-Big Reasoning Field");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("surface_bridge_state: {}", report.surface_bridge_state);
    println!("premise_surface: {}", report.premise_surface);
    println!("hop_count: {}", report.metrics.hop_count);
    println!("chain_exact: {}", report.metrics.chain_exact);
    println!(
        "missing_evidence_reject_rate: {:.3}",
        report.metrics.missing_evidence_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_dialogue_state_text(report: &DialogueStateReport) {
    println!("LLMWave-Big Dialogue State");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("reasoning_bridge_state: {}", report.reasoning_bridge_state);
    println!("question: {}", report.user_question);
    println!("answer_state: {}", report.answer_state);
    println!("answer: {}", report.constrained_answer);
    println!(
        "unsupported_answer_reject_rate: {:.3}",
        report.metrics.unsupported_answer_reject_rate
    );
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_lexical_birth_text(report: &LexicalBirthReport) {
    println!("LLMWave-Big Lexical Birth");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("word_definition: {}", report.word_definition);
    println!("sample_surface: {}", report.sample.surface);
    println!("sample_gate: {}", report.sample.gate.verdict);
    println!("sample_score: {}", report.sample.gate.total_score);
    println!(
        "rejection_control: {}",
        report.rejection_control.gate.verdict
    );
    println!(
        "claims: corpus_proven={} generator_ready={} nonlinear_density_proven={}",
        report.claim_boundary.corpus_proven,
        report.claim_boundary.generator_ready,
        report.claim_boundary.nonlinear_density_proven
    );
}

fn print_surface_production_text(report: &SurfaceProductionReport) {
    println!("LLMWave-Big Surface Production");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("primary_rule: {}", report.production_law.primary_rule);
    println!("selected_path: {}", report.selected.production_path);
    println!("selected_score: {}", report.selected.final_score);
    println!(
        "materialized_preview: {}",
        report.selected.materialized_preview
    );
    println!(
        "claims: real_corpus_trained={} free_form_spelling_proven={} nonlinear_surface_memory_proven={}",
        report.claim_boundary.real_corpus_trained,
        report.claim_boundary.free_form_spelling_proven,
        report.claim_boundary.nonlinear_surface_memory_proven
    );
}

fn print_surface_reconstruct_text(report: &SurfaceReconstructReport) {
    println!("LLMWave-Big Surface Reconstruct");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("cases: {}", report.eval.cases);
    println!("exact_match_rate: {:.3}", report.eval.exact_match_rate);
    println!("fallback_rate: {:.3}", report.eval.fallback_rate);
    println!(
        "bytes_per_reconstructable_surface: {:.3}",
        report.eval.bytes_per_reconstructable_surface
    );
    println!("state: {}", report.eval.state);
    println!(
        "claims: real_corpus_trained={} free_form_spelling_proven={} nonlinear_surface_memory_proven={}",
        report.claim_boundary.real_corpus_trained,
        report.claim_boundary.free_form_spelling_proven,
        report.claim_boundary.nonlinear_surface_memory_proven
    );
}

fn print_surface_corpus_eval_text(report: &SurfaceCorpusEvalReport) {
    println!("LLMWave-Big Surface Corpus Eval");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("forms: {}", report.corpus.total_forms);
    println!("family_bytes: {}", report.baselines.family_template_bytes);
    println!("direct_bytes: {}", report.baselines.direct_lookup_bytes);
    println!(
        "family_vs_direct_saving_ratio: {:.3}",
        report.baselines.family_vs_direct_saving_ratio
    );
    println!(
        "exact_match_rate: {:.3}",
        report.reconstruction.exact_match_rate
    );
    println!(
        "nonlinear_surface_memory_proven: {}",
        report.verdict_boundary.nonlinear_surface_memory_proven
    );
}

fn print_surface_bank_build_text(report: &SurfaceBankBuildReport) {
    println!("LLMWave-Big Surface Bank Build");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("observed_forms: {}", report.corpus.observed_forms);
    println!(
        "accepted_families: {}",
        report.bank_summary.accepted_family_count
    );
    println!(
        "rejected_fragments: {}",
        report.bank_summary.rejected_fragment_count
    );
    println!("total_bank_bytes: {}", report.bank_summary.total_bank_bytes);
    println!(
        "held_out_exact_match_rate: {:.3}",
        report.eval.held_out_exact_match_rate
    );
    println!(
        "nonlinear_surface_memory_proven: {}",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
}

fn print_surface_bank_validate_text(report: &SurfaceBankValidateReport) {
    println!("LLMWave-Big Surface Bank Validate");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!(
        "positive_accept_rate: {:.3}",
        report.metrics.positive_accept_rate
    );
    println!(
        "negative_reject_rate: {:.3}",
        report.metrics.negative_reject_rate
    );
    println!(
        "shuffle_stability_rate: {:.3}",
        report.metrics.shuffle_stability_rate
    );
    println!("false_family_rate: {:.3}", report.metrics.false_family_rate);
    println!(
        "nonlinear_surface_memory_proven: {}",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
}

fn print_surface_bank_fixture_text(report: &SurfaceBankFixtureReport) {
    println!("LLMWave-Big Surface Bank Fixture");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("corpus_path: {}", report.corpus_path);
    println!("families: {}", report.corpus.family_count);
    println!("observed_forms: {}", report.corpus.observed_forms);
    println!("held_out_forms: {}", report.corpus.held_out_forms);
    println!(
        "positive_exact_match_rate: {:.3}",
        report.metrics.positive_exact_match_rate
    );
    println!(
        "negative_reject_rate: {:.3}",
        report.metrics.negative_reject_rate
    );
    println!("total_bank_bytes: {}", report.baselines.total_bank_bytes);
    println!(
        "nonlinear_surface_memory_proven: {}",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
}

fn print_surface_raw_induce_text(report: &SurfaceRawInduceReport) {
    println!("LLMWave-Big Surface Raw Induce");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("corpus_path: {}", report.corpus_path);
    println!("raw_forms: {}", report.corpus.raw_forms);
    println!(
        "suffix_inventory_source: {}",
        report.corpus.suffix_inventory_source
    );
    println!(
        "derived_suffix_count: {}",
        report.metrics.derived_suffix_count
    );
    println!(
        "induced_family_count: {}",
        report.metrics.induced_family_count
    );
    println!(
        "expected_root_recall: {:.3}",
        report.metrics.expected_root_recall
    );
    println!(
        "held_out_exact_match_rate: {:.3}",
        report.metrics.held_out_exact_match_rate
    );
    println!(
        "negative_reject_rate: {:.3}",
        report.metrics.negative_reject_rate
    );
    println!("noise_reject_rate: {:.3}", report.metrics.noise_reject_rate);
    println!(
        "nonlinear_surface_memory_proven: {}",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
}

fn print_write_text(report: &WriteReport) {
    println!("LLMWave-Big Schema/Residual Write");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("sample: {}", report.sample_input);
    println!("score: {}", report.reconstructability.total);
    println!("decision: {}", report.write_decision.decision);
    println!("bytes_written: {}", report.write_decision.bytes_written);
    println!(
        "residual_saving_ratio: {:.4}",
        report.write_curve.residual_saving_ratio
    );
    println!("curve_state: {}", report.write_curve.state);
}

fn print_consolidation_text(report: &ConsolidationReport) {
    println!("LLMWave-Big Consolidation Sleep");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!(
        "sleep: {} -> {}",
        report.sleep_pass.input, report.sleep_pass.output
    );
    println!(
        "memory_bytes: {} -> {}",
        report.eval.before.memory_bytes, report.eval.after.memory_bytes
    );
    println!(
        "role_safety: {:.3} -> {:.3}",
        report.eval.before.role_safety, report.eval.after.role_safety
    );
    println!(
        "compression_score: {:.3}",
        report.cognitive_compression_score
    );
}

fn print_big_eval_text(report: &BigEvalReport) {
    println!("LLMWave-Big Cognition Eval");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("cases: {}", report.cases.len());
    println!("score: {:.3}", report.cognitive_score.total);
    println!("llm_ready: {}", report.claim_boundary.llm_ready);
    println!(
        "nonlinear_memory_proven: {}",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_runtime_product_text(report: &RuntimeProductReport) {
    println!("LLMWave-Big Runtime Product");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("query: {}", report.query);
    println!("field_state: {}", report.safety.field_state);
    println!("safe_to_answer: {}", report.safety.safe_to_answer);
    println!(
        "target_hot_query_ms: {:.3}",
        report.performance.target_hot_query_ms
    );
    println!("llm_ready: {}", report.claim_boundary.llm_ready);
}

fn print_contract_md(report: &LlmwaveBigReport) {
    println!("# LLMWave-Big Contract");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- state: `{}`", report.claim_boundary.current_state);
    println!("- safe_claim: {}", report.claim_boundary.safe_claim);
    println!();
    println!("## Layers");
    println!();
    for layer in &report.contract.layers {
        println!("- `{}`: {}", layer.name, layer.responsibility);
    }
    println!();
    println!("## Required Metrics");
    println!();
    for metric in &report.bigness_metrics.required_metrics {
        println!(
            "- `{}`: {} / {}",
            metric.name, metric.unit, metric.direction
        );
    }
    println!();
    println!("## Forbidden Claims");
    println!();
    for claim in &report.claim_boundary.forbidden_claims {
        println!("- `{claim}`");
    }
}

fn print_atlas_md(report: &AtlasReport) {
    println!("# LLMWave-Big Wave Atlas");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- state: `{}`", report.state);
    println!("- doctor: `{}`", report.doctor.verdict);
    println!();
    println!("## Record Formats");
    println!();
    for record in &report.record_formats {
        println!("- `{}`: {} bytes", record.name, record.bytes);
    }
    println!();
    println!("## Loader Preview");
    println!();
    println!(
        "- top symbols: `{}`",
        report.loader_preview.top_symbols.len()
    );
    println!(
        "- top schemas: `{}`",
        report.loader_preview.top_schemas.len()
    );
    println!(
        "- evidence refs: `{}`",
        report.loader_preview.evidence_refs.len()
    );
    println!(
        "- fits active core contract: `{}`",
        report.loader_preview.fits_active_core_contract
    );
}

fn print_active_core_md(report: &ActiveCoreReport) {
    println!("# LLMWave-Big Active Core");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- total bytes: `{}`", report.budget.total_bytes);
    println!(
        "- fits NANDA-6M budget: `{}`",
        report.budget.fits_nanda_6m_budget
    );
    println!();
    println!("## Runtime Cycle");
    println!();
    println!("- top schema: `{}`", report.cycle.top_schema_id);
    println!("- top score: `{}`", report.cycle.top_score);
    println!("- margin: `{}`", report.cycle.margin);
    println!("- safe to answer: `{}`", report.cycle.safe_to_answer);
}

fn print_l2_word_field_md(report: &L2WordFieldReport) {
    println!("# LLMWave-Big L2 Word Field");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- prefix: `{}`", report.prefix_wave.prefix);
    println!("- top token: `{}`", report.candidate_cache.top_token_label);
    println!("- margin: `{}`", report.candidate_cache.margin);
    println!(
        "- runtime top surface: `{}`",
        report.runtime_field.top_surface
    );
    println!("- runtime margin: `{}`", report.runtime_field.margin);
    println!("- runtime state: `{}`", report.runtime_field.field_state);
    println!();
    println!("## Runtime Field");
    println!();
    for candidate in &report.runtime_field.candidates {
        println!(
            "- `{}`: final={} prefix={} family={} suffix={} l3={} anti={}",
            candidate.surface,
            candidate.final_score,
            candidate.prefix_resonance,
            candidate.family_resonance,
            candidate.suffix_resonance,
            candidate.l3_phase_bias,
            candidate.anti_wave
        );
    }
    println!();
    println!("## Candidate Sample");
    println!();
    for candidate in &report.candidate_cache.sample {
        println!(
            "- `{}`: final={} anti={}",
            candidate.label, candidate.final_score, candidate.anti_score
        );
    }
}

fn print_hrr_binding_md(report: &HrrBindingReport) {
    println!("# LLMWave-Big HRR Binding");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- wave_dim: `{}`", report.wave_dim);
    println!();
    println!("## Bindings");
    println!();
    for binding in &report.bindings {
        println!(
            "- `{}` -> expected `{}`, recovered `{}`, margin `{}`",
            binding.role, binding.filler, binding.recovered, binding.margin
        );
    }
    println!();
    println!("## Metrics");
    println!();
    println!("- role recall: `{:.3}`", report.metrics.role_recall);
    println!(
        "- noisy role recall: `{:.3}`",
        report.metrics.noisy_role_recall
    );
    println!(
        "- collision reject rate: `{:.3}`",
        report.metrics.collision_reject_rate
    );
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_l3_schema_bind_md(report: &L3SchemaBindReport) {
    println!("# LLMWave-Big L3 Schema Bind");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- schema id: `{}`", report.schema.schema_id);
    println!("- form: `{}`", report.schema.form);
    println!();
    println!("## Recovered Roles");
    println!();
    for role in &report.recovered_roles {
        println!(
            "- `{}` -> expected `{}`, recovered `{}`, margin `{}`",
            role.role, role.expected, role.recovered, role.margin
        );
    }
    println!();
    println!("## Trap");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.role_swap_trap.wrong_claim, report.role_swap_trap.rejected
    );
}

fn print_l2_l3_coupling_md(report: &L2L3CouplingReport) {
    println!("# LLMWave-Big L2/L3 Coupling");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- raw top: `{}`", report.l2_probe.raw_top);
    println!("- coupled top: `{}`", report.l2_probe.coupled_top);
    println!("- schema: `{}`", report.l3_schema.form);
    println!();
    println!("## Rerank");
    println!();
    for candidate in &report.rerank.candidates {
        println!(
            "- `{}` final `{}` l3 `{}` accepted `{}`",
            candidate.surface,
            candidate.final_score,
            candidate.l3_role_score,
            candidate.role_accepted
        );
    }
    println!();
    println!("## Trap");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.disagreement_trap.rejected_surface, report.disagreement_trap.rejected
    );
}

fn print_coupled_decode_loop_md(report: &CoupledDecodeLoopReport) {
    println!("# LLMWave-Big Coupled Decode Loop");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- bridge_state: `{}`", report.bridge_state);
    println!("- final sequence: `{}`", report.final_sequence.join(" "));
    println!();
    println!("## Steps");
    println!();
    for step in &report.accepted_steps {
        println!(
            "- step `{}` `{}` raw `{}` -> accepted `{}` margin `{}`",
            step.step, step.role, step.raw_top, step.accepted, step.margin
        );
    }
    println!();
    println!("## Trap");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.bad_continuation_trap.trap, report.bad_continuation_trap.rejected
    );
}

fn print_multi_schema_competition_md(report: &MultiSchemaCompetitionReport) {
    println!("# LLMWave-Big Multi-Schema Competition");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- selected route: `{}`", report.selected_route.route);
    println!("- top margin: `{}`", report.metrics.top_margin);
    println!();
    println!("## Peaks");
    println!();
    for peak in &report.peaks {
        println!(
            "- schema `{}` `{}` final `{}` margin `{}`",
            peak.schema_id, peak.route, peak.final_score, peak.margin
        );
    }
    println!();
    println!("## Trap");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.route_splice_trap.trap, report.route_splice_trap.rejected
    );
}

fn print_schema_memory_growth_md(report: &SchemaMemoryGrowthReport) {
    println!("# LLMWave-Big Schema Memory Growth");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- observed facts: `{}`", report.observed_fact_count);
    println!("- promoted: `{}`", report.memory_metrics.promoted_count);
    println!("- rejected: `{}`", report.memory_metrics.rejected_count);
    println!();
    println!("## Promoted Schemas");
    println!();
    for schema in &report.promoted_schemas {
        println!(
            "- `{}` `{}` support `{}` strength `{}`",
            schema.route, schema.form, schema.support_count, schema.strength
        );
    }
    println!();
    println!("## Negative Control");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.negative_control.proposed_form, report.negative_control.rejected
    );
}

fn print_open_surface_generation_md(report: &OpenSurfaceGenerationReport) {
    println!("# LLMWave-Big Open Surface Generation");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- selected route: `{}`", report.selected_schema.route);
    println!("- surface: `{}`", report.materialized_surface);
    println!();
    println!("## Surface Plan");
    println!();
    for step in &report.surface_plan {
        println!(
            "- `{}` `{}` via `{}`",
            step.slot, step.surface, step.production_path
        );
    }
    println!();
    println!("## Trap");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.trap.proposed_surface, report.trap.rejected
    );
}

fn print_reasoning_field_md(report: &ReasoningFieldReport) {
    println!("# LLMWave-Big Reasoning Field");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- premise: `{}`", report.premise_surface);
    println!();
    println!("## Hops");
    println!();
    for hop in &report.hops {
        println!(
            "- `{}` `{}` `{}` -> `{}` score `{}`",
            hop.from, hop.relation, hop.to, hop.evidence, hop.final_score
        );
    }
    println!();
    println!("## Trap");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.trap.proposed_inference, report.trap.rejected
    );
}

fn print_dialogue_state_md(report: &DialogueStateReport) {
    println!("# LLMWave-Big Dialogue State");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- question: `{}`", report.user_question);
    println!("- answer_state: `{}`", report.answer_state);
    println!();
    println!("## Answer");
    println!();
    println!("{}", report.constrained_answer);
    println!();
    println!("## Trap");
    println!();
    println!(
        "- `{}` rejected: `{}`",
        report.trap.unsafe_answer, report.trap.rejected
    );
}

fn print_lexical_birth_md(report: &LexicalBirthReport) {
    println!("# LLMWave-Big Lexical Birth");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- read as: {}", report.read_as);
    println!("- word definition: {}", report.word_definition);
    println!();
    println!("## Birth Stages");
    println!();
    for stage in &report.birth_stages {
        println!(
            "- `{}`: {} -> {}",
            stage.stage, stage.input_signal, stage.gate
        );
    }
    println!();
    println!("## Sample Gate");
    println!();
    println!("- surface: `{}`", report.sample.surface);
    println!("- verdict: `{}`", report.sample.gate.verdict);
    println!("- score: `{}`", report.sample.gate.total_score);
    println!(
        "- rejection control: `{}`",
        report.rejection_control.gate.verdict
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!("- corpus proven: `{}`", report.claim_boundary.corpus_proven);
    println!(
        "- generator ready: `{}`",
        report.claim_boundary.generator_ready
    );
    println!(
        "- nonlinear density proven: `{}`",
        report.claim_boundary.nonlinear_density_proven
    );
}

fn print_surface_production_md(report: &SurfaceProductionReport) {
    println!("# LLMWave-Big Surface Production");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- read as: {}", report.read_as);
    println!("- primary rule: {}", report.production_law.primary_rule);
    println!();
    println!("## Record Formats");
    println!();
    for record in &report.record_formats {
        println!("- `{}`: {} bytes", record.name, record.bytes);
    }
    println!();
    println!("## Selected Path");
    println!();
    println!("- production path: `{}`", report.selected.production_path);
    println!("- program id: `{}`", report.selected.program_id);
    println!("- final score: `{}`", report.selected.final_score);
    println!(
        "- materialized preview: `{}`",
        report.selected.materialized_preview
    );
    println!(
        "- materialization scope: `{}`",
        report.selected.materialization_scope
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- real corpus trained: `{}`",
        report.claim_boundary.real_corpus_trained
    );
    println!(
        "- free-form spelling proven: `{}`",
        report.claim_boundary.free_form_spelling_proven
    );
    println!(
        "- nonlinear surface memory proven: `{}`",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
}

fn print_surface_reconstruct_md(report: &SurfaceReconstructReport) {
    println!("# LLMWave-Big Surface Reconstruct");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- read as: {}", report.read_as);
    println!();
    println!("## Eval");
    println!();
    println!("- cases: `{}`", report.eval.cases);
    println!("- exact matches: `{}`", report.eval.exact_matches);
    println!("- exact match rate: `{:.3}`", report.eval.exact_match_rate);
    println!("- fallback rate: `{:.3}`", report.eval.fallback_rate);
    println!(
        "- bytes per reconstructable surface: `{:.3}`",
        report.eval.bytes_per_reconstructable_surface
    );
    println!("- state: `{}`", report.eval.state);
    println!();
    println!("## Cases");
    println!();
    for case in &report.cases {
        println!(
            "- `{}` via `{}`: `{}` exact={}",
            case.case_id, case.path, case.reconstructed, case.exact_match
        );
    }
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- real corpus trained: `{}`",
        report.claim_boundary.real_corpus_trained
    );
    println!(
        "- free-form spelling proven: `{}`",
        report.claim_boundary.free_form_spelling_proven
    );
    println!(
        "- nonlinear surface memory proven: `{}`",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
    println!(
        "- hot core UTF-8 free: `{}`",
        report.claim_boundary.hot_core_utf8_free
    );
}

fn print_surface_corpus_eval_md(report: &SurfaceCorpusEvalReport) {
    println!("# LLMWave-Big Surface Corpus Eval");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- read as: {}", report.read_as);
    println!();
    println!("## Corpus");
    println!();
    println!("- productive forms: `{}`", report.corpus.productive_forms);
    println!("- total forms: `{}`", report.corpus.total_forms);
    println!("- held-out forms: `{}`", report.corpus.held_out_forms);
    println!();
    println!("## Baselines");
    println!();
    println!(
        "- direct lookup bytes: `{}`",
        report.baselines.direct_lookup_bytes
    );
    println!(
        "- per-form program bytes: `{}`",
        report.baselines.per_form_program_bytes
    );
    println!(
        "- family template bytes: `{}`",
        report.baselines.family_template_bytes
    );
    println!(
        "- family/direct saving ratio: `{:.3}`",
        report.baselines.family_vs_direct_saving_ratio
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- useful density candidate: `{}`",
        report.verdict_boundary.useful_density_candidate
    );
    println!(
        "- nonlinear surface memory proven: `{}`",
        report.verdict_boundary.nonlinear_surface_memory_proven
    );
    println!(
        "- real corpus trained: `{}`",
        report.verdict_boundary.real_corpus_trained
    );
}

fn print_surface_bank_build_md(report: &SurfaceBankBuildReport) {
    println!("# LLMWave-Big Surface Bank Build");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- read as: {}", report.read_as);
    println!();
    println!("## Build");
    println!();
    println!("- observed forms: `{}`", report.corpus.observed_forms);
    println!("- held-out forms: `{}`", report.corpus.held_out_forms);
    println!(
        "- accepted families: `{}`",
        report.bank_summary.accepted_family_count
    );
    println!(
        "- rejected fragments: `{}`",
        report.bank_summary.rejected_fragment_count
    );
    println!(
        "- total bank bytes: `{}`",
        report.bank_summary.total_bank_bytes
    );
    println!(
        "- direct lookup bytes: `{}`",
        report.bank_summary.direct_lookup_baseline_bytes
    );
    println!("- saving ratio: `{:.3}`", report.bank_summary.saving_ratio);
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- useful density candidate: `{}`",
        report.claim_boundary.useful_density_candidate
    );
    println!(
        "- nonlinear surface memory proven: `{}`",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
    println!(
        "- real corpus trained: `{}`",
        report.claim_boundary.real_corpus_trained
    );
}

fn print_surface_bank_validate_md(report: &SurfaceBankValidateReport) {
    println!("# LLMWave-Big Surface Bank Validate");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- read as: {}", report.read_as);
    println!();
    println!("## Controls");
    println!();
    println!(
        "- positive accept rate: `{:.3}`",
        report.metrics.positive_accept_rate
    );
    println!(
        "- negative reject rate: `{:.3}`",
        report.metrics.negative_reject_rate
    );
    println!(
        "- false family rate: `{:.3}`",
        report.metrics.false_family_rate
    );
    println!("- shuffle stability: `{}`", report.shuffle_stability.state);
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- validation passed: `{}`",
        report.claim_boundary.validation_passed
    );
    println!(
        "- nonlinear surface memory proven: `{}`",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
    println!(
        "- real corpus trained: `{}`",
        report.claim_boundary.real_corpus_trained
    );
}

fn print_surface_bank_fixture_md(report: &SurfaceBankFixtureReport) {
    println!("# LLMWave-Big Surface Bank Fixture");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- corpus: `{}`", report.corpus_path);
    println!("- read as: {}", report.read_as);
    println!();
    println!("## Fixture");
    println!();
    println!("- families: `{}`", report.corpus.family_count);
    println!("- observed forms: `{}`", report.corpus.observed_forms);
    println!("- held-out forms: `{}`", report.corpus.held_out_forms);
    println!("- negative controls: `{}`", report.corpus.negative_controls);
    println!("- rare forms: `{}`", report.corpus.rare_forms);
    println!();
    println!("## Metrics");
    println!();
    println!(
        "- positive exact match rate: `{:.3}`",
        report.metrics.positive_exact_match_rate
    );
    println!(
        "- negative reject rate: `{:.3}`",
        report.metrics.negative_reject_rate
    );
    println!(
        "- rare copy-span rate: `{:.3}`",
        report.metrics.rare_copy_span_rate
    );
    println!(
        "- total bank bytes: `{}`",
        report.baselines.total_bank_bytes
    );
    println!(
        "- direct lookup bytes: `{}`",
        report.baselines.direct_lookup_baseline_bytes
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- external fixture loaded: `{}`",
        report.claim_boundary.external_fixture_loaded
    );
    println!(
        "- nonlinear surface memory proven: `{}`",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
    println!(
        "- real corpus trained: `{}`",
        report.claim_boundary.real_corpus_trained
    );
}

fn print_surface_raw_induce_md(report: &SurfaceRawInduceReport) {
    println!("# LLMWave-Big Surface Raw Induce");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- corpus: `{}`", report.corpus_path);
    println!("- read as: {}", report.read_as);
    println!();
    println!("## Raw Forms");
    println!();
    println!("- raw forms: `{}`", report.corpus.raw_forms);
    println!("- suffix inventory: `{}`", report.corpus.suffix_inventory);
    println!(
        "- suffix inventory source: `{}`",
        report.corpus.suffix_inventory_source
    );
    println!(
        "- derived suffix count: `{}`",
        report.metrics.derived_suffix_count
    );
    println!(
        "- induced families: `{}`",
        report.metrics.induced_family_count
    );
    println!(
        "- expected root recall: `{:.3}`",
        report.metrics.expected_root_recall
    );
    println!(
        "- held-out exact match rate: `{:.3}`",
        report.metrics.held_out_exact_match_rate
    );
    println!(
        "- negative reject rate: `{:.3}`",
        report.metrics.negative_reject_rate
    );
    println!(
        "- noise reject rate: `{:.3}`",
        report.metrics.noise_reject_rate
    );
    println!(
        "- rejected collision roots: `{}`",
        report.rejected_collision_roots.len()
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- roots given to inducer: `{}`",
        report.claim_boundary.roots_given_to_inducer
    );
    println!(
        "- nonlinear surface memory proven: `{}`",
        report.claim_boundary.nonlinear_surface_memory_proven
    );
    println!(
        "- real corpus trained: `{}`",
        report.claim_boundary.real_corpus_trained
    );
}

fn print_write_md(report: &WriteReport) {
    println!("# LLMWave-Big Schema/Residual Write");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- sample: `{}`", report.sample_input);
    println!(
        "- reconstructability score: `{}`",
        report.reconstructability.total
    );
    println!("- write decision: `{}`", report.write_decision.decision);
    println!("- bytes written: `{}`", report.write_decision.bytes_written);
    println!(
        "- residual saving ratio: `{:.4}`",
        report.write_curve.residual_saving_ratio
    );
    println!("- curve state: `{}`", report.write_curve.state);
}

fn print_consolidation_md(report: &ConsolidationReport) {
    println!("# LLMWave-Big Consolidation Sleep");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- conflict state: `{}`", report.conflict_preservation.state);
    println!(
        "- memory bytes: `{}` -> `{}`",
        report.eval.before.memory_bytes, report.eval.after.memory_bytes
    );
    println!(
        "- role safety: `{:.3}` -> `{:.3}`",
        report.eval.before.role_safety, report.eval.after.role_safety
    );
    println!(
        "- cognitive compression score: `{:.3}`",
        report.cognitive_compression_score
    );
}

fn print_big_eval_md(report: &BigEvalReport) {
    println!("# LLMWave-Big Cognition Eval");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- cases: `{}`", report.cases.len());
    println!("- cognitive score: `{:.3}`", report.cognitive_score.total);
    println!("- LLM ready: `{}`", report.claim_boundary.llm_ready);
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_runtime_product_md(report: &RuntimeProductReport) {
    println!("# LLMWave-Big Runtime Product");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- query: `{}`", report.query);
    println!("- field state: `{}`", report.safety.field_state);
    println!("- safe to answer: `{}`", report.safety.safe_to_answer);
    println!(
        "- target hot query ms: `{:.3}`",
        report.performance.target_hot_query_ms
    );
    println!("- LLM ready: `{}`", report.claim_boundary.llm_ready);
}

fn print_mini_chat_eval_text(report: &MiniChatEvalReport) {
    println!("LLMWave-Big Mini Chat Eval");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("dialogue_bridge_state: {}", report.dialogue_bridge_state);
    println!("case_count: {}", report.metrics.case_count);
    println!("passed_cases: {}", report.metrics.passed_cases);
    println!("failed_cases: {}", report.metrics.failed_cases);
    println!("full_llm_ready: {}", report.claim_boundary.full_llm_ready);
}

fn print_mini_chat_eval_md(report: &MiniChatEvalReport) {
    println!("# LLMWave-Big Mini Chat Eval");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- dialogue bridge: `{}`", report.dialogue_bridge_state);
    println!("- scope: `{}`", report.evaluation_scope);
    println!("- cases: `{}`", report.metrics.case_count);
    println!("- passed: `{}`", report.metrics.passed_cases);
    println!("- failed: `{}`", report.metrics.failed_cases);
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- mini chat candidate: `{}`",
        report.claim_boundary.mini_chat_candidate
    );
    println!(
        "- full LLM ready: `{}`",
        report.claim_boundary.full_llm_ready
    );
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_query_wave_text(report: &QueryWaveReport) {
    println!("LLMWave-Big Query Wave");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("input: {}", report.input_text);
    println!("route: {}", report.top_route_hint);
    println!("polarity: {}", report.question_polarity);
    println!(
        "fixed_query_wave_records: {}",
        report.claim_boundary.fixed_query_wave_records
    );
}

fn print_query_wave_md(report: &QueryWaveReport) {
    println!("# LLMWave-Big Query Wave");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- input: `{}`", report.input_text);
    println!("- route hint: `{}`", report.top_route_hint);
    println!("- polarity: `{}`", report.question_polarity);
    println!();
    println!("## Metrics");
    println!();
    println!(
        "- paraphrase route recall: `{:.3}`",
        report.metrics.paraphrase_route_recall
    );
    println!(
        "- role hint accuracy: `{:.3}`",
        report.metrics.role_hint_accuracy
    );
    println!(
        "- operator hint accuracy: `{:.3}`",
        report.metrics.operator_hint_accuracy
    );
    println!(
        "- assertion reject rate: `{:.3}`",
        report.metrics.assertion_reject_rate
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- full field mature: `{}`",
        report.claim_boundary.full_field_mature
    );
    println!("- chat ready: `{}`", report.claim_boundary.chat_ready);
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_multi_peak_field_text(report: &MultiPeakFieldReport) {
    println!("LLMWave-Big Multi-Peak Field");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("field_state: {}", report.field_state);
    println!("top_route: {}", report.top_peak.route);
    println!("peak_margin: {}", report.metrics.peak_margin);
    println!("safe_to_answer: {}", report.claim_boundary.safe_to_answer);
}

fn print_multi_peak_field_md(report: &MultiPeakFieldReport) {
    println!("# LLMWave-Big Multi-Peak Field");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- field state: `{}`", report.field_state);
    println!("- top route: `{}`", report.top_peak.route);
    println!();
    println!("## Metrics");
    println!();
    println!("- peaks: `{}`", report.metrics.peak_count);
    println!("- peak margin: `{}`", report.metrics.peak_margin);
    println!(
        "- route boundary leakage: `{:.3}`",
        report.metrics.route_boundary_leakage
    );
    println!(
        "- contested detection: `{:.3}`",
        report.metrics.contested_detection_rate
    );
    println!(
        "- no-answer detection: `{:.3}`",
        report.metrics.no_answer_detection_rate
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- safe to answer: `{}`",
        report.claim_boundary.safe_to_answer
    );
    println!(
        "- full field mature: `{}`",
        report.claim_boundary.full_field_mature
    );
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_lens_scan_text(report: &LensScanReport) {
    println!("LLMWave-Big Lens Scan");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("field_bridge_state: {}", report.field_bridge_state);
    println!("answer_decision: {}", report.answer_decision);
    println!(
        "lens_agreement_rate: {:.3}",
        report.metrics.lens_agreement_rate
    );
}

fn print_lens_scan_md(report: &LensScanReport) {
    println!("# LLMWave-Big Lens Scan");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- field bridge: `{}`", report.field_bridge_state);
    println!("- answer decision: `{}`", report.answer_decision);
    println!();
    println!("## Metrics");
    println!();
    println!("- lenses: `{}`", report.metrics.lens_count);
    println!("- pass: `{}`", report.metrics.pass_count);
    println!("- watch: `{}`", report.metrics.watch_count);
    println!("- block: `{}`", report.metrics.block_count);
    println!(
        "- lens agreement: `{:.3}`",
        report.metrics.lens_agreement_rate
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- safe to answer: `{}`",
        report.claim_boundary.safe_to_answer
    );
    println!("- chat ready: `{}`", report.claim_boundary.chat_ready);
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_mature_anti_wave_text(report: &MatureAntiWaveReport) {
    println!("LLMWave-Big Mature Anti-Wave");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("lens_bridge_verdict: {}", report.lens_bridge_verdict);
    println!(
        "anti_field_state: {}",
        report.field_after_anti.anti_field_state
    );
    println!(
        "answer_decision: {}",
        report.field_after_anti.answer_decision
    );
    println!("lane_count: {}", report.metrics.lane_count);
    println!("suppress_total: {}", report.metrics.suppress_total);
    println!(
        "support_preserved_total: {}",
        report.metrics.support_preserved_total
    );
}

fn print_mature_anti_wave_md(report: &MatureAntiWaveReport) {
    println!("# LLMWave-Big Mature Anti-Wave");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- lens bridge: `{}`", report.lens_bridge_verdict);
    println!(
        "- anti field state: `{}`",
        report.field_after_anti.anti_field_state
    );
    println!(
        "- answer decision: `{}`",
        report.field_after_anti.answer_decision
    );
    println!();
    println!("## Metrics");
    println!();
    println!("- lanes: `{}`", report.metrics.lane_count);
    println!("- suppress total: `{}`", report.metrics.suppress_total);
    println!(
        "- support preserved: `{}`",
        report.metrics.support_preserved_total
    );
    println!("- locality floor: `{}`", report.metrics.locality_floor);
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- safe to answer: `{}`",
        report.claim_boundary.safe_to_answer
    );
    println!("- chat ready: `{}`", report.claim_boundary.chat_ready);
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_evidence_proof_text(report: &EvidenceProofReport) {
    println!("LLMWave-Big Evidence Proof");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("evidence_mode: {}", report.evidence_mode);
    println!("field_bridge_state: {}", report.field_bridge_state);
    println!("proof_state: {}", report.proof_state);
    println!("answer_permission: {}", report.answer_permission);
    println!(
        "missing_evidence_block_rate: {:.3}",
        report.metrics.missing_evidence_block_rate
    );
}

fn print_evidence_proof_md(report: &EvidenceProofReport) {
    println!("# LLMWave-Big Evidence Proof");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- evidence mode: `{}`", report.evidence_mode);
    println!("- proof state: `{}`", report.proof_state);
    println!("- answer permission: `{}`", report.answer_permission);
    println!();
    println!("## Metrics");
    println!();
    println!(
        "- evidence binding: `{:.3}`",
        report.metrics.evidence_binding_rate
    );
    println!(
        "- missing evidence block: `{:.3}`",
        report.metrics.missing_evidence_block_rate
    );
    println!(
        "- unsafe answer: `{:.3}`",
        report.metrics.unsafe_answer_rate
    );
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- local answer permission: `{}`",
        report.claim_boundary.local_answer_permission
    );
    println!("- chat ready: `{}`", report.claim_boundary.chat_ready);
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_answer_surface_text(report: &AnswerSurfaceReport) {
    println!("LLMWave-Big Answer Surface");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("proof_bridge_verdict: {}", report.proof_bridge_verdict);
    println!("answer_state: {}", report.answer_state);
    println!("answer_text: {}", report.answer_text);
    println!(
        "free_form_generation: {}",
        report.claim_boundary.free_form_generation
    );
}

fn print_answer_surface_md(report: &AnswerSurfaceReport) {
    println!("# LLMWave-Big Answer Surface");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- proof bridge: `{}`", report.proof_bridge_verdict);
    println!("- answer state: `{}`", report.answer_state);
    println!("- answer text: `{}`", report.answer_text);
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- free form generation: `{}`",
        report.claim_boundary.free_form_generation
    );
    println!(
        "- local answer surface: `{}`",
        report.claim_boundary.local_answer_surface
    );
    println!("- chat ready: `{}`", report.claim_boundary.chat_ready);
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}

fn print_field_feedback_text(report: &FieldFeedbackReport) {
    println!("LLMWave-Big Field Feedback");
    println!("version: {}", report.version);
    println!("roadmap_block: {}", report.roadmap_block);
    println!("verdict: {}", report.verdict);
    println!("decision: {}", report.decision);
    println!("feedback_state: {}", report.feedback_state);
    println!("memory_effect: {}", report.memory_effect);
    println!(
        "persistent_training_done: {}",
        report.claim_boundary.persistent_training_done
    );
}

fn print_field_feedback_md(report: &FieldFeedbackReport) {
    println!("# LLMWave-Big Field Feedback");
    println!();
    println!("- version: `{}`", report.version);
    println!("- roadmap_block: `{}`", report.roadmap_block);
    println!("- verdict: `{}`", report.verdict);
    println!("- decision: `{}`", report.decision);
    println!("- feedback state: `{}`", report.feedback_state);
    println!("- memory effect: `{}`", report.memory_effect);
    println!();
    println!("## Claim Boundary");
    println!();
    println!(
        "- local memory update: `{}`",
        report.claim_boundary.local_memory_update
    );
    println!(
        "- persistent training done: `{}`",
        report.claim_boundary.persistent_training_done
    );
    println!("- chat ready: `{}`", report.claim_boundary.chat_ready);
    println!(
        "- nonlinear memory proven: `{}`",
        report.claim_boundary.nonlinear_memory_proven
    );
}
