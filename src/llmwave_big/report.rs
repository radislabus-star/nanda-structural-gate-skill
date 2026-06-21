use anyhow::Result;

use super::active_core::ActiveCoreReport;
use super::atlas::AtlasReport;
use super::consolidation::ConsolidationReport;
use super::eval::BigEvalReport;
use super::hrr_binding::HrrBindingReport;
use super::l2_word_field::L2WordFieldReport;
use super::l3_schema_bind::L3SchemaBindReport;
use super::lexical_birth::LexicalBirthReport;
use super::loader::RuntimeProductReport;
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
