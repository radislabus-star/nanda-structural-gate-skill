use anyhow::Result;

use super::active_core::ActiveCoreReport;
use super::atlas::AtlasReport;
use super::consolidation::ConsolidationReport;
use super::eval::BigEvalReport;
use super::l2_word_field::L2WordFieldReport;
use super::lexical_birth::LexicalBirthReport;
use super::loader::RuntimeProductReport;
use super::surface_production::SurfaceProductionReport;
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
    println!("record_bytes: {}", report.candidate_cache.record_bytes);
    println!(
        "sync: {}/{}",
        report.sync_policy.l2_update, report.sync_policy.l3_update
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
