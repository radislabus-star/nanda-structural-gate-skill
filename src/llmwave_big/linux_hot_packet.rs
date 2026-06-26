//! Binary Linux Active Field packet and scan path.
//!
//! This is the Linux SysField cold-to-hot bridge. The hot loop scans fixed
//! 64-byte records loaded from a `.laf` packet. The string table is kept in the
//! same file for explanation and operator output, but it is accounted as cold
//! label data rather than part of the 6 MiB fixed-record hot budget.

use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::linux_active_field::{
    build_linux_active_fact_window, score_linux_fact_terms, LinuxFactScoreTerms,
};
use super::linux_atlas::{LinuxAtlasFact, LINUX_ATLAS_VERSION};
use crate::WAVE_DIM;

pub(crate) const LINUX_HOT_PACKET_VERSION: &str = "llmwave-big-v-next-linux-hot-packet";

const LAF_MAGIC: &[u8; 8] = b"LLMWLAF1";
const LAF_FORMAT_VERSION: u32 = 1;
const LAF_HEADER_BYTES: usize = 64;
const LAF_RECORD_BYTES: usize = 64;
const SIX_MIB_BYTES: usize = 6 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct LinuxHotPackConfig {
    pub atlas_dir: PathBuf,
    pub max_active_facts: usize,
    pub out: PathBuf,
}

#[derive(Clone)]
pub(crate) struct LinuxHotAskConfig {
    pub hot_pack: PathBuf,
    pub query: String,
    pub top_k: usize,
}

#[derive(Clone)]
pub(crate) struct LinuxHotEvalConfig {
    pub hot_pack: PathBuf,
    pub top_k: usize,
}

#[derive(Clone)]
pub(crate) struct LinuxDomainRunConfig {
    pub hot_pack: PathBuf,
    pub query: String,
    pub top_k: usize,
}

#[derive(Clone)]
pub(crate) struct LinuxCacheProofConfig {
    pub hot_pack: PathBuf,
    pub query: String,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub samples: usize,
}

#[derive(Clone)]
pub(crate) struct LinuxPmuCacheProofConfig {
    pub hot_pack: PathBuf,
    pub query: String,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub samples: usize,
    pub max_cache_miss_rate: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotPackReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub atlas_version: &'static str,
    pub verdict: &'static str,
    pub atlas_dir: String,
    pub out: String,
    pub source: LinuxHotPackSource,
    pub packet: LinuxHotPacketSummary,
    pub schema_residual_memory: LinuxSchemaResidualMemory,
    pub claim_boundary: LinuxHotClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotPackSource {
    pub fact_pack_count: usize,
    pub atlas_fact_count: usize,
    pub selected_facts: usize,
    pub route_count: usize,
    pub corpus_hash: String,
    pub route_distribution: BTreeMap<String, usize>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotPacketSummary {
    pub format: &'static str,
    pub header_bytes: usize,
    pub fixed_record_bytes: usize,
    pub fixed_record_count: usize,
    pub hot_loop_record_bytes: usize,
    pub cold_label_count: usize,
    pub cold_label_table_bytes: usize,
    pub actual_file_bytes: usize,
    pub hot_budget_bytes: usize,
    pub fixed_records_fit_6m: bool,
    pub whole_file_fits_6m: bool,
    pub labels_are_cold_explain_data: bool,
    pub json_used_in_hot_scan: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxSchemaResidualMemory {
    pub state: &'static str,
    pub schema_key: &'static str,
    pub promotion_threshold: usize,
    pub promoted_schema_count: usize,
    pub residual_fact_count: usize,
    pub full_fallback_fact_count: usize,
    pub direct_fixed_record_bytes: usize,
    pub schema_record_bytes: usize,
    pub residual_record_bytes: usize,
    pub fallback_record_bytes: usize,
    pub estimated_schema_residual_bytes: usize,
    pub residual_saving_bytes: isize,
    pub schema_reuse_ratio: f32,
    pub residual_saving_ratio: f32,
    pub nonlinear_memory_proven: bool,
    pub notes: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotClaimBoundary {
    pub linux_atlas_loaded: bool,
    pub active_65k_projection_ready: bool,
    pub binary_hot_packet_written: bool,
    pub fixed_records_scan_ready: bool,
    pub cold_labels_required_for_human_output: bool,
    pub cache_only_execution_proven: bool,
    pub exposure_layer_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCacheProofReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub hot_pack: LinuxCacheProofPackSummary,
    pub compiled_query: LinuxCacheProofQuerySummary,
    pub runtime_contract: LinuxCacheRuntimeContract,
    pub benchmark: LinuxCacheBenchmarkSummary,
    pub claim_boundary: LinuxCacheProofClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxPmuCacheProofReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub hot_pack: LinuxCacheProofPackSummary,
    pub compiled_query: LinuxCacheProofQuerySummary,
    pub software_runtime_contract: LinuxCacheRuntimeContract,
    pub software_benchmark: LinuxCacheBenchmarkSummary,
    pub pmu: LinuxPmuCounterSummary,
    pub claim_boundary: LinuxPmuCacheProofClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCacheProofPackSummary {
    pub path: String,
    pub file_bytes: usize,
    pub wave_dim: u32,
    pub fixed_record_count: usize,
    pub route_count: usize,
    pub corpus_hash64: u64,
    pub fixed_record_bytes: usize,
    pub hot_loop_record_bytes: usize,
    pub cold_label_count: usize,
    pub cold_label_table_bytes: usize,
    pub hot_budget_bytes: usize,
    pub detected_l3_bytes: Option<usize>,
    pub fixed_records_fit_6m: bool,
    pub fixed_records_fit_detected_l3: Option<bool>,
    pub whole_file_fits_6m: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCacheProofQuerySummary {
    pub raw_query: String,
    pub token_count: usize,
    pub token_hash_count: usize,
    pub route_hint_hash_count: usize,
    pub relation_hint_hash_count: usize,
    pub boundary_intent: bool,
    pub positive_intent: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCacheRuntimeContract {
    pub records_loaded_from: &'static str,
    pub full_active_scan: bool,
    pub fixed_record_only_inner_loop: bool,
    pub json_used_in_hot_loop: bool,
    pub labels_read_from_packet: bool,
    pub label_decode_in_hot_loop: bool,
    pub file_io_in_hot_loop: bool,
    pub heap_allocation_in_inner_loop: bool,
    pub per_record_score_arrays: bool,
    pub cold_label_table_excluded: bool,
    pub measured_loop_uses_numeric_hashes: bool,
    pub hardware_perf_counters_used: bool,
    pub hardware_cache_miss_rate_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCacheBenchmarkSummary {
    pub warmup_iterations: usize,
    pub measured_samples: usize,
    pub iterations_per_sample: usize,
    pub measured_scans: usize,
    pub records_per_scan: usize,
    pub measured_record_visits: u128,
    pub ns_per_scan_min: f64,
    pub ns_per_scan_p50: f64,
    pub ns_per_scan_p95: f64,
    pub ns_per_scan_max: f64,
    pub ns_per_record_p50: f64,
    pub scans_per_second_p50: f64,
    pub records_per_second_p50: f64,
    pub top_score: i64,
    pub margin: i64,
    pub top_record_index: Option<usize>,
    pub top_fact_hash: Option<u64>,
    pub checksum: u64,
    pub sample_ns_per_scan: Vec<f64>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCacheProofClaimBoundary {
    pub binary_hot_packet_loaded: bool,
    pub fixed_records_fit_6m: bool,
    pub fixed_records_fit_detected_l3: Option<bool>,
    pub fixed_record_runtime_measured: bool,
    pub no_json_labels_or_file_io_in_hot_loop: bool,
    pub full_active_scan_measured: bool,
    pub cache_only_execution_proven: bool,
    pub hardware_cache_residency_counter_proven: bool,
    pub exposure_layer_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxPmuCounterSummary {
    pub counter_status: &'static str,
    pub measured_scope: &'static str,
    pub attempted_events: Vec<&'static str>,
    pub available_events: Vec<&'static str>,
    pub blocked_reason: Option<String>,
    pub perf_event_open_errno: Option<i32>,
    pub cache_references: Option<u64>,
    pub cache_misses: Option<u64>,
    pub cache_miss_rate: Option<f64>,
    pub max_cache_miss_rate: f64,
    pub cache_miss_rate_under_threshold: Option<bool>,
    pub elapsed_ns: Option<u128>,
    pub measured_scans: usize,
    pub measured_record_visits: u128,
    pub checksum: Option<u64>,
    pub read_as: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxPmuCacheProofClaimBoundary {
    pub binary_hot_packet_loaded: bool,
    pub fixed_records_fit_6m: bool,
    pub fixed_records_fit_detected_l3: Option<bool>,
    pub software_cache_only_execution_proven: bool,
    pub hardware_perf_counters_used: bool,
    pub hardware_cache_residency_counter_proven: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotAskReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub query: String,
    pub hot_pack: LinuxHotAskPackSummary,
    pub field: LinuxHotAskField,
    pub answer: LinuxHotAnswer,
    pub claim_boundary: LinuxHotAskClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotAskPackSummary {
    pub path: String,
    pub bytes_scanned: usize,
    pub fixed_record_count: usize,
    pub fixed_record_bytes: usize,
    pub cold_label_count: usize,
    pub cold_label_table_bytes: usize,
    pub fixed_records_fit_6m: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotAskField {
    pub state: &'static str,
    pub top_score: i64,
    pub margin: i64,
    pub top_facts: Vec<LinuxHotAskFact>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotAskFact {
    pub score: i64,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: &'static str,
    pub confidence: u8,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotAnswer {
    pub state: &'static str,
    pub safe_to_answer: bool,
    pub text: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotAskClaimBoundary {
    pub binary_hot_packet_loaded: bool,
    pub fixed_records_scanned: bool,
    pub json_used_in_hot_scan: bool,
    pub cold_labels_used_for_display: bool,
    pub cache_only_execution_proven: bool,
    pub exposure_layer_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotEvalReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub hot_pack: LinuxHotAskPackSummary,
    pub cases: Vec<LinuxHotEvalCaseReport>,
    pub metrics: LinuxHotEvalMetrics,
    pub claim_boundary: LinuxHotEvalClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotEvalCaseReport {
    pub id: &'static str,
    pub query: &'static str,
    pub expected_state: &'static str,
    pub expected_route: &'static str,
    pub expected_polarity: &'static str,
    pub expected_object_match: &'static str,
    pub expected_object_contains: &'static str,
    pub field_state: &'static str,
    pub field_top_score: i64,
    pub field_top_route: Option<String>,
    pub field_top_object: Option<String>,
    pub field_top_polarity: Option<&'static str>,
    pub lexical_top_route: Option<String>,
    pub lexical_top_object: Option<String>,
    pub lexical_top_score: i64,
    pub field_beats_lexical_baseline: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotEvalMetrics {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f32,
    pub lexical_duel_total: usize,
    pub lexical_duel_wins: usize,
    pub lexical_duel_win_rate: f32,
    pub boundary_cases: usize,
    pub boundary_cases_passed: usize,
    pub false_positive_rate: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxHotEvalClaimBoundary {
    pub binary_hot_packet_loaded: bool,
    pub fixed_records_scanned: bool,
    pub lexical_baseline_duel_run: bool,
    pub linux_domain_eval_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub cache_only_execution_proven: bool,
    pub safe_claim: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainRunReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub hot_pack: String,
    pub query_wave: LinuxDomainQueryWave,
    pub reasoning_loop: LinuxDomainReasoningLoop,
    pub answer_surface: LinuxDomainAnswerSurface,
    pub verifier: LinuxDomainVerifier,
    pub feedback_learning: LinuxDomainFeedbackLearning,
    pub eval_metrics: LinuxHotEvalMetrics,
    pub claim_boundary: LinuxDomainClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainQueryWave {
    pub raw_query: String,
    pub tokens: Vec<String>,
    pub command_anchor: Option<String>,
    pub boundary_intent: bool,
    pub route_hint: &'static str,
    pub polarity: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainReasoningLoop {
    pub steps: Vec<&'static str>,
    pub field_state: &'static str,
    pub selected_route: Option<String>,
    pub selected_object: Option<String>,
    pub lexical_trap_count: usize,
    pub baseline_duel_passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainAnswerSurface {
    pub state: &'static str,
    pub safe_to_answer: bool,
    pub text: String,
    pub free_form_generation: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainVerifier {
    pub state: &'static str,
    pub answer_allowed: bool,
    pub hot_scan_safe: bool,
    pub eval_gate_passed: bool,
    pub false_positive_rate: f32,
    pub reason_codes: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainFeedbackLearning {
    pub state: &'static str,
    pub feedback_packet_preview_ready: bool,
    pub persistent_memory_written: bool,
    pub accepted_route: Option<String>,
    pub rejected_lexical_traps: Vec<LinuxDomainRejectedTrap>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainRejectedTrap {
    pub case_id: &'static str,
    pub lexical_route: Option<String>,
    pub lexical_object: Option<String>,
    pub field_route: Option<String>,
    pub field_object: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxDomainClaimBoundary {
    pub linux_domain_core_ready: bool,
    pub binary_hot_packet_loaded: bool,
    pub query_wave_used: bool,
    pub reasoning_loop_used: bool,
    pub answer_verified: bool,
    pub feedback_learning_preview_ready: bool,
    pub persistent_training_done: bool,
    pub exposure_layer_ready: bool,
    pub cache_only_execution_proven: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

#[derive(Clone)]
struct PackedLinuxFact64 {
    fact_hash: u64,
    route_hash: u64,
    subject_hash: u64,
    relation_hash: u64,
    object_hash: u64,
    route_id: u32,
    subject_id: u32,
    relation_id: u32,
    object_id: u32,
    confidence: u8,
    polarity_id: u8,
    reserved_u16: u16,
    evidence_hash: u32,
}

struct LinuxHotPacketImage {
    records: Vec<PackedLinuxFact64>,
    labels: Vec<String>,
    cold_label_table_bytes: usize,
    actual_file_bytes: usize,
}

struct LinuxHotFixedRecordImage {
    records: Vec<PackedLinuxFact64>,
    header: LafHeaderFields,
    actual_file_bytes: usize,
    cold_label_table_bytes: usize,
    detected_l3_bytes: Option<usize>,
}

#[derive(Clone, Copy)]
struct LafHeaderFields {
    wave_dim: u32,
    record_count: usize,
    label_count: usize,
    route_count: usize,
    record_bytes: usize,
    corpus_hash: u64,
}

struct LinuxHotNumericQuery {
    raw_query: String,
    token_hashes: Vec<u64>,
    route_hint_hashes: Vec<u64>,
    relation_hint_hashes: Vec<u64>,
    boundary_intent: bool,
    positive_intent: bool,
}

#[derive(Clone, Copy)]
struct LinuxHotNumericScan {
    top_score: i64,
    second_score: i64,
    top_record_index: Option<usize>,
    top_fact_hash: Option<u64>,
    checksum: u64,
}

struct LabelTableBuilder {
    ids: BTreeMap<String, u32>,
    labels: Vec<String>,
}

impl LabelTableBuilder {
    fn new() -> Self {
        Self {
            ids: BTreeMap::new(),
            labels: Vec::new(),
        }
    }

    fn intern(&mut self, value: &str) -> u32 {
        if let Some(id) = self.ids.get(value) {
            return *id;
        }
        let id = self.labels.len().min(u32::MAX as usize) as u32;
        self.labels.push(value.to_string());
        self.ids.insert(value.to_string(), id);
        id
    }
}

pub(crate) fn build_linux_hot_pack_report(
    config: LinuxHotPackConfig,
) -> Result<LinuxHotPackReport> {
    let max_active_facts = config.max_active_facts.max(1);
    let window = build_linux_active_fact_window(&config.atlas_dir, max_active_facts)?;
    let mut labels = LabelTableBuilder::new();
    let mut records = Vec::with_capacity(window.active_facts.len());
    let mut route_distribution = BTreeMap::<String, usize>::new();
    for fact in &window.active_facts {
        *route_distribution.entry(fact.route.clone()).or_insert(0) += 1;
        records.push(pack_fact(fact, &mut labels));
    }
    let schema_residual_memory = compute_schema_residual_memory(&window.active_facts);

    if let Some(parent) = config.out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create linux hot packet dir {}", parent.display()))?;
    }
    let mut writer = BufWriter::new(
        fs::File::create(&config.out)
            .with_context(|| format!("create linux hot packet {}", config.out.display()))?,
    );
    write_laf_header(
        &mut writer,
        records.len(),
        labels.labels.len(),
        route_distribution.len(),
        &window.scan.corpus_hash,
    )?;
    for record in &records {
        write_record(&mut writer, record)?;
    }
    let cold_label_table_bytes = write_label_table(&mut writer, &labels.labels)?;
    writer.flush()?;
    let actual_file_bytes = fs::metadata(&config.out)
        .with_context(|| format!("stat linux hot packet {}", config.out.display()))?
        .len() as usize;
    let hot_loop_record_bytes = records.len() * LAF_RECORD_BYTES;
    let fixed_records_fit_6m = hot_loop_record_bytes <= SIX_MIB_BYTES;
    let whole_file_fits_6m = actual_file_bytes <= SIX_MIB_BYTES;
    let verdict = if records.is_empty() {
        "LINUX_HOT_PACKET_EMPTY"
    } else if fixed_records_fit_6m {
        "LINUX_HOT_PACKET_READY_NOT_CACHE_ONLY_PROOF"
    } else {
        "LINUX_HOT_PACKET_RECORDS_EXCEED_6M"
    };

    Ok(LinuxHotPackReport {
        mode: "llmwave-big-linux-pack-hot",
        version: LINUX_HOT_PACKET_VERSION,
        atlas_version: LINUX_ATLAS_VERSION,
        verdict,
        atlas_dir: config.atlas_dir.display().to_string(),
        out: config.out.display().to_string(),
        source: LinuxHotPackSource {
            fact_pack_count: window.fact_paths.len(),
            atlas_fact_count: window.scan.fact_count,
            selected_facts: records.len(),
            route_count: route_distribution.len(),
            corpus_hash: window.scan.corpus_hash,
            route_distribution,
        },
        packet: LinuxHotPacketSummary {
            format: "laf-v1-fixed64-plus-cold-label-table",
            header_bytes: LAF_HEADER_BYTES,
            fixed_record_bytes: LAF_RECORD_BYTES,
            fixed_record_count: records.len(),
            hot_loop_record_bytes,
            cold_label_count: labels.labels.len(),
            cold_label_table_bytes,
            actual_file_bytes,
            hot_budget_bytes: SIX_MIB_BYTES,
            fixed_records_fit_6m,
            whole_file_fits_6m,
            labels_are_cold_explain_data: true,
            json_used_in_hot_scan: false,
        },
        schema_residual_memory,
        claim_boundary: LinuxHotClaimBoundary {
            linux_atlas_loaded: window.scan.fact_count > 0,
            active_65k_projection_ready: max_active_facts >= 65_536
                && records.len() == window.scan.fact_count.min(max_active_facts)
                && fixed_records_fit_6m,
            binary_hot_packet_written: true,
            fixed_records_scan_ready: fixed_records_fit_6m && !records.is_empty(),
            cold_labels_required_for_human_output: true,
            cache_only_execution_proven: false,
            exposure_layer_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: "Linux hot packet materializes the active Linux field as fixed 64-byte records and can scan those records without JSON. It is not a general LLM, not exposure analysis, and not nonlinear-memory proof.",
            blocked_by: vec![
                "linux_hot_eval_suite_missing",
                "linux_answer_verifier_missing",
                "cache_residency_perf_counter_missing",
                "general_llm_eval_missing",
                "nonlinear_memory_final_proof_missing",
            ],
        },
    })
}

fn compute_schema_residual_memory(facts: &[LinuxAtlasFact]) -> LinuxSchemaResidualMemory {
    const PROMOTION_THRESHOLD: usize = 2;
    const SCHEMA_RECORD_BYTES: usize = 32;
    const RESIDUAL_RECORD_BYTES: usize = 24;
    const FALLBACK_RECORD_BYTES: usize = LAF_RECORD_BYTES;

    let mut schema_counts = BTreeMap::<(String, String, String), usize>::new();
    for fact in facts {
        *schema_counts
            .entry((
                fact.route.clone(),
                fact.relation.clone(),
                fact.polarity.clone(),
            ))
            .or_insert(0) += 1;
    }
    let mut promoted_schema_count = 0usize;
    let mut residual_fact_count = 0usize;
    let mut full_fallback_fact_count = 0usize;
    for count in schema_counts.values().copied() {
        if count >= PROMOTION_THRESHOLD {
            promoted_schema_count += 1;
            residual_fact_count += count;
        } else {
            full_fallback_fact_count += count;
        }
    }
    let direct_fixed_record_bytes = facts.len() * LAF_RECORD_BYTES;
    let estimated_schema_residual_bytes = promoted_schema_count * SCHEMA_RECORD_BYTES
        + residual_fact_count * RESIDUAL_RECORD_BYTES
        + full_fallback_fact_count * FALLBACK_RECORD_BYTES;
    let residual_saving_bytes =
        direct_fixed_record_bytes as isize - estimated_schema_residual_bytes as isize;
    let schema_reuse_ratio = if facts.is_empty() {
        0.0
    } else {
        round4(residual_fact_count as f32 / facts.len() as f32)
    };
    let residual_saving_ratio = if direct_fixed_record_bytes == 0 {
        0.0
    } else {
        round4(residual_saving_bytes.max(0) as f32 / direct_fixed_record_bytes as f32)
    };
    let state = if facts.is_empty() {
        "SCHEMA_RESIDUAL_EMPTY"
    } else if residual_saving_bytes > 0 && schema_reuse_ratio >= 0.25 {
        "SCHEMA_RESIDUAL_CANDIDATE"
    } else {
        "SCHEMA_RESIDUAL_REVIEW"
    };
    LinuxSchemaResidualMemory {
        state,
        schema_key: "route+relation+polarity",
        promotion_threshold: PROMOTION_THRESHOLD,
        promoted_schema_count,
        residual_fact_count,
        full_fallback_fact_count,
        direct_fixed_record_bytes,
        schema_record_bytes: SCHEMA_RECORD_BYTES,
        residual_record_bytes: RESIDUAL_RECORD_BYTES,
        fallback_record_bytes: FALLBACK_RECORD_BYTES,
        estimated_schema_residual_bytes,
        residual_saving_bytes,
        schema_reuse_ratio,
        residual_saving_ratio,
        nonlinear_memory_proven: false,
        notes: vec![
            "schema residual accounting is measured over selected active facts",
            "one-off facts remain full fallback records instead of being forced into bad schemas",
            "a positive saving is profile evidence only, not global nonlinear-memory proof",
        ],
    }
}

pub(crate) fn build_linux_cache_proof_report(
    config: LinuxCacheProofConfig,
) -> Result<LinuxCacheProofReport> {
    let fixed = parse_laf_fixed_records_only(&config.hot_pack)?;
    let query = compile_linux_hot_numeric_query(&config.query);
    let iterations = config.iterations.max(1);
    let samples = config.samples.max(1);
    let warmup_iterations = config.warmup_iterations;
    let benchmark = benchmark_linux_fixed_records(
        &fixed.records,
        &query,
        iterations,
        warmup_iterations,
        samples,
    );
    let hot_loop_record_bytes = fixed.records.len() * LAF_RECORD_BYTES;
    let fixed_records_fit_6m = hot_loop_record_bytes <= SIX_MIB_BYTES;
    let fixed_records_fit_detected_l3 = fixed
        .detected_l3_bytes
        .map(|l3_bytes| hot_loop_record_bytes <= l3_bytes);
    let whole_file_fits_6m = fixed.actual_file_bytes <= SIX_MIB_BYTES;
    let full_active_scan = benchmark.records_per_scan == fixed.records.len();
    let no_json_labels_or_file_io = true;
    let fixed_record_runtime_measured =
        benchmark.measured_scans > 0 && benchmark.measured_record_visits > 0;
    let cache_only_execution_proven = !fixed.records.is_empty()
        && fixed_records_fit_6m
        && fixed_record_runtime_measured
        && full_active_scan
        && no_json_labels_or_file_io
        && benchmark.top_score > 0;
    let verdict = if cache_only_execution_proven {
        "LINUX_CACHE_ONLY_EXECUTION_PROVEN"
    } else if fixed.records.is_empty() {
        "LINUX_CACHE_PROOF_EMPTY_PACKET"
    } else if !fixed_records_fit_6m {
        "LINUX_CACHE_PROOF_RECORDS_EXCEED_6M"
    } else {
        "LINUX_CACHE_PROOF_REVIEW"
    };

    Ok(LinuxCacheProofReport {
        mode: "llmwave-big-linux-cache-proof",
        version: LINUX_HOT_PACKET_VERSION,
        verdict,
        hot_pack: LinuxCacheProofPackSummary {
            path: config.hot_pack.display().to_string(),
            file_bytes: fixed.actual_file_bytes,
            wave_dim: fixed.header.wave_dim,
            fixed_record_count: fixed.records.len(),
            route_count: fixed.header.route_count,
            corpus_hash64: fixed.header.corpus_hash,
            fixed_record_bytes: fixed.header.record_bytes,
            hot_loop_record_bytes,
            cold_label_count: fixed.header.label_count,
            cold_label_table_bytes: fixed.cold_label_table_bytes,
            hot_budget_bytes: SIX_MIB_BYTES,
            detected_l3_bytes: fixed.detected_l3_bytes,
            fixed_records_fit_6m,
            fixed_records_fit_detected_l3,
            whole_file_fits_6m,
        },
        compiled_query: LinuxCacheProofQuerySummary {
            raw_query: query.raw_query.clone(),
            token_count: query.token_hashes.len(),
            token_hash_count: query.token_hashes.len(),
            route_hint_hash_count: query.route_hint_hashes.len(),
            relation_hint_hash_count: query.relation_hint_hashes.len(),
            boundary_intent: query.boundary_intent,
            positive_intent: query.positive_intent,
        },
        runtime_contract: LinuxCacheRuntimeContract {
            records_loaded_from: "laf-header-plus-fixed-record-section-only",
            full_active_scan,
            fixed_record_only_inner_loop: true,
            json_used_in_hot_loop: false,
            labels_read_from_packet: false,
            label_decode_in_hot_loop: false,
            file_io_in_hot_loop: false,
            heap_allocation_in_inner_loop: false,
            per_record_score_arrays: false,
            cold_label_table_excluded: true,
            measured_loop_uses_numeric_hashes: true,
            hardware_perf_counters_used: false,
            hardware_cache_miss_rate_proven: false,
        },
        benchmark,
        claim_boundary: LinuxCacheProofClaimBoundary {
            binary_hot_packet_loaded: true,
            fixed_records_fit_6m,
            fixed_records_fit_detected_l3,
            fixed_record_runtime_measured,
            no_json_labels_or_file_io_in_hot_loop: no_json_labels_or_file_io,
            full_active_scan_measured: full_active_scan,
            cache_only_execution_proven,
            hardware_cache_residency_counter_proven: false,
            exposure_layer_ready: false,
            broad_chat_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: "The Linux .laf fixed-record section fits the 6 MiB hot budget and the measured hot loop scans only numeric fixed records without JSON, labels, file I/O, heap allocation, or per-record score arrays. Hardware cache-miss counters are not used, so this is a software cache-budget runtime proof, not a PMU residency proof.",
            blocked_claims: vec![
                "hardware_cache_miss_rate_proven",
                "broad_chat_llm_ready",
                "nonlinear_memory_proven",
                "exposure_layer_ready",
            ],
        },
    })
}

pub(crate) fn build_linux_pmu_cache_proof_report(
    config: LinuxPmuCacheProofConfig,
) -> Result<LinuxPmuCacheProofReport> {
    let fixed = parse_laf_fixed_records_only(&config.hot_pack)?;
    let query = compile_linux_hot_numeric_query(&config.query);
    let iterations = config.iterations.max(1);
    let samples = config.samples.max(1);
    let warmup_iterations = config.warmup_iterations;
    let software_benchmark = benchmark_linux_fixed_records(
        &fixed.records,
        &query,
        iterations,
        warmup_iterations,
        samples,
    );
    let hot_loop_record_bytes = fixed.records.len() * LAF_RECORD_BYTES;
    let fixed_records_fit_6m = hot_loop_record_bytes <= SIX_MIB_BYTES;
    let fixed_records_fit_detected_l3 = fixed
        .detected_l3_bytes
        .map(|l3_bytes| hot_loop_record_bytes <= l3_bytes);
    let whole_file_fits_6m = fixed.actual_file_bytes <= SIX_MIB_BYTES;
    let full_active_scan = software_benchmark.records_per_scan == fixed.records.len();
    let software_cache_only_execution_proven = !fixed.records.is_empty()
        && fixed_records_fit_6m
        && software_benchmark.measured_scans > 0
        && software_benchmark.measured_record_visits > 0
        && full_active_scan
        && software_benchmark.top_score > 0;
    let pmu = measure_linux_hot_loop_pmu(
        &fixed.records,
        &query,
        iterations,
        warmup_iterations,
        samples,
        config.max_cache_miss_rate.max(0.0),
    );
    let hardware_perf_counters_used = pmu.counter_status == "MEASURED";
    let hardware_cache_residency_counter_proven = software_cache_only_execution_proven
        && fixed_records_fit_detected_l3.unwrap_or(fixed_records_fit_6m)
        && pmu.cache_miss_rate_under_threshold == Some(true);
    let verdict = if hardware_cache_residency_counter_proven {
        "LINUX_PMU_CACHE_RESIDENCY_PROVEN"
    } else if pmu.counter_status == "MEASURED" {
        "LINUX_PMU_CACHE_RESIDENCY_REVIEW"
    } else {
        "LINUX_PMU_CACHE_RESIDENCY_BLOCKED"
    };

    let mut blocked_claims = vec!["broad_chat_llm_ready", "nonlinear_memory_proven"];
    if !hardware_cache_residency_counter_proven {
        blocked_claims.push("hardware_cache_residency_counter_proven");
    }
    if !software_cache_only_execution_proven {
        blocked_claims.push("software_cache_only_execution_proven");
    }

    Ok(LinuxPmuCacheProofReport {
        mode: "llmwave-big-linux-pmu-cache-proof",
        version: LINUX_HOT_PACKET_VERSION,
        verdict,
        hot_pack: LinuxCacheProofPackSummary {
            path: config.hot_pack.display().to_string(),
            file_bytes: fixed.actual_file_bytes,
            wave_dim: fixed.header.wave_dim,
            fixed_record_count: fixed.records.len(),
            route_count: fixed.header.route_count,
            corpus_hash64: fixed.header.corpus_hash,
            fixed_record_bytes: fixed.header.record_bytes,
            hot_loop_record_bytes,
            cold_label_count: fixed.header.label_count,
            cold_label_table_bytes: fixed.cold_label_table_bytes,
            hot_budget_bytes: SIX_MIB_BYTES,
            detected_l3_bytes: fixed.detected_l3_bytes,
            fixed_records_fit_6m,
            fixed_records_fit_detected_l3,
            whole_file_fits_6m,
        },
        compiled_query: LinuxCacheProofQuerySummary {
            raw_query: query.raw_query.clone(),
            token_count: query.token_hashes.len(),
            token_hash_count: query.token_hashes.len(),
            route_hint_hash_count: query.route_hint_hashes.len(),
            relation_hint_hash_count: query.relation_hint_hashes.len(),
            boundary_intent: query.boundary_intent,
            positive_intent: query.positive_intent,
        },
        software_runtime_contract: LinuxCacheRuntimeContract {
            records_loaded_from: "laf-header-plus-fixed-record-section-only",
            full_active_scan,
            fixed_record_only_inner_loop: true,
            json_used_in_hot_loop: false,
            labels_read_from_packet: false,
            label_decode_in_hot_loop: false,
            file_io_in_hot_loop: false,
            heap_allocation_in_inner_loop: false,
            per_record_score_arrays: false,
            cold_label_table_excluded: true,
            measured_loop_uses_numeric_hashes: true,
            hardware_perf_counters_used,
            hardware_cache_miss_rate_proven: hardware_cache_residency_counter_proven,
        },
        software_benchmark,
        pmu,
        claim_boundary: LinuxPmuCacheProofClaimBoundary {
            binary_hot_packet_loaded: true,
            fixed_records_fit_6m,
            fixed_records_fit_detected_l3,
            software_cache_only_execution_proven,
            hardware_perf_counters_used,
            hardware_cache_residency_counter_proven,
            broad_chat_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: "The Linux .laf fixed-record loop is measured with hardware PMU cache-reference/cache-miss counters when the host kernel allows perf_event_open. A PASS means the hot fixed-record section fits the cache budget and PMU cache miss rate stayed under the configured threshold. This is not broad chat or nonlinear-memory proof.",
            blocked_claims,
        },
    })
}

fn benchmark_linux_fixed_records(
    records: &[PackedLinuxFact64],
    query: &LinuxHotNumericQuery,
    iterations: usize,
    warmup_iterations: usize,
    samples: usize,
) -> LinuxCacheBenchmarkSummary {
    let mut last_scan = LinuxHotNumericScan {
        top_score: 0,
        second_score: 0,
        top_record_index: None,
        top_fact_hash: None,
        checksum: 0,
    };
    for _ in 0..warmup_iterations {
        last_scan = black_box(scan_linux_fixed_records_numeric(records, query));
    }

    let mut sample_ns_per_scan = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        for _ in 0..iterations {
            last_scan = black_box(scan_linux_fixed_records_numeric(records, query));
        }
        let elapsed = start.elapsed().as_nanos() as f64;
        sample_ns_per_scan.push(elapsed / iterations as f64);
    }

    let mut sorted = sample_ns_per_scan.clone();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let min = *sorted.first().unwrap_or(&0.0);
    let max = *sorted.last().unwrap_or(&0.0);
    let p50 = percentile(&sorted, 0.50);
    let p95 = percentile(&sorted, 0.95);
    let records_per_scan = records.len();
    let ns_per_record_p50 = if records_per_scan == 0 {
        0.0
    } else {
        p50 / records_per_scan as f64
    };
    let scans_per_second_p50 = if p50 <= 0.0 {
        0.0
    } else {
        1_000_000_000.0 / p50
    };
    let records_per_second_p50 = scans_per_second_p50 * records_per_scan as f64;
    let measured_scans = samples.saturating_mul(iterations);
    LinuxCacheBenchmarkSummary {
        warmup_iterations,
        measured_samples: samples,
        iterations_per_sample: iterations,
        measured_scans,
        records_per_scan,
        measured_record_visits: measured_scans as u128 * records_per_scan as u128,
        ns_per_scan_min: round2(min),
        ns_per_scan_p50: round2(p50),
        ns_per_scan_p95: round2(p95),
        ns_per_scan_max: round2(max),
        ns_per_record_p50: round4_f64(ns_per_record_p50),
        scans_per_second_p50: round2(scans_per_second_p50),
        records_per_second_p50: round2(records_per_second_p50),
        top_score: last_scan.top_score,
        margin: last_scan.top_score - last_scan.second_score,
        top_record_index: last_scan.top_record_index,
        top_fact_hash: last_scan.top_fact_hash,
        checksum: last_scan.checksum,
        sample_ns_per_scan: sample_ns_per_scan.into_iter().map(round2).collect(),
    }
}

fn measure_linux_hot_loop_pmu(
    records: &[PackedLinuxFact64],
    query: &LinuxHotNumericQuery,
    iterations: usize,
    warmup_iterations: usize,
    samples: usize,
    max_cache_miss_rate: f64,
) -> LinuxPmuCounterSummary {
    let attempted_events = vec!["cache-references", "cache-misses"];
    if records.is_empty() {
        return LinuxPmuCounterSummary {
            counter_status: "BLOCKED",
            measured_scope: "linux-laf-fixed-record-hot-loop",
            attempted_events,
            available_events: Vec::new(),
            blocked_reason: Some("empty_fixed_record_section".to_string()),
            perf_event_open_errno: None,
            cache_references: None,
            cache_misses: None,
            cache_miss_rate: None,
            max_cache_miss_rate: round4_f64(max_cache_miss_rate),
            cache_miss_rate_under_threshold: None,
            elapsed_ns: None,
            measured_scans: 0,
            measured_record_visits: 0,
            checksum: None,
            read_as: "No PMU proof was attempted because the hot loop has no records.",
        };
    }

    for _ in 0..warmup_iterations {
        black_box(scan_linux_fixed_records_numeric(records, query));
    }

    let measured_scans = samples.saturating_mul(iterations.max(1));
    let measured_record_visits = measured_scans as u128 * records.len() as u128;
    match platform_pmu::measure_cache_counters(|| {
        let mut checksum = 0u64;
        for scan_index in 0..measured_scans {
            checksum = checksum.rotate_left(9)
                ^ black_box(scan_linux_fixed_records_numeric(records, query)).checksum
                ^ (scan_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
        checksum
    }) {
        PmuMeasureOutcome::Measured(measured) => {
            let cache_miss_rate = if measured.cache_references == 0 {
                None
            } else {
                Some(round6(
                    measured.cache_misses as f64 / measured.cache_references as f64,
                ))
            };
            let cache_miss_rate_under_threshold =
                cache_miss_rate.map(|rate| rate <= max_cache_miss_rate);
            LinuxPmuCounterSummary {
                counter_status: if cache_miss_rate.is_some() {
                    "MEASURED"
                } else {
                    "NO_REFERENCES"
                },
                measured_scope: "linux-laf-fixed-record-hot-loop",
                attempted_events,
                available_events: vec!["cache-references", "cache-misses"],
                blocked_reason: None,
                perf_event_open_errno: None,
                cache_references: Some(measured.cache_references),
                cache_misses: Some(measured.cache_misses),
                cache_miss_rate,
                max_cache_miss_rate: round4_f64(max_cache_miss_rate),
                cache_miss_rate_under_threshold,
                elapsed_ns: Some(measured.elapsed_ns),
                measured_scans,
                measured_record_visits,
                checksum: Some(measured.checksum),
                read_as: "Hardware PMU counters were read around the fixed-record hot loop only, after warmup. The result is a host/runtime measurement, not a portable model theorem.",
            }
        }
        PmuMeasureOutcome::Blocked(blocked) => LinuxPmuCounterSummary {
            counter_status: "BLOCKED",
            measured_scope: "linux-laf-fixed-record-hot-loop",
            attempted_events,
            available_events: Vec::new(),
            blocked_reason: Some(blocked.reason),
            perf_event_open_errno: blocked.errno,
            cache_references: None,
            cache_misses: None,
            cache_miss_rate: None,
            max_cache_miss_rate: round4_f64(max_cache_miss_rate),
            cache_miss_rate_under_threshold: None,
            elapsed_ns: None,
            measured_scans,
            measured_record_visits,
            checksum: None,
            read_as: "PMU proof is blocked by host/kernel support or permissions. This keeps hardware_cache_residency_counter_proven=false instead of inventing a result.",
        },
    }
}

fn scan_linux_fixed_records_numeric(
    records: &[PackedLinuxFact64],
    query: &LinuxHotNumericQuery,
) -> LinuxHotNumericScan {
    let mut top_score = i64::MIN;
    let mut second_score = i64::MIN;
    let mut top_record_index = None;
    let mut top_fact_hash = None;
    let mut checksum = 0u64;
    for (index, record) in records.iter().enumerate() {
        let score = score_linux_fixed_record_numeric(record, query);
        checksum = checksum.rotate_left(7)
            ^ record.fact_hash
            ^ record.route_hash
            ^ ((score as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        if score > top_score {
            second_score = top_score;
            top_score = score;
            top_record_index = Some(index);
            top_fact_hash = Some(record.fact_hash);
        } else if score > second_score {
            second_score = score;
        }
    }
    LinuxHotNumericScan {
        top_score: top_score.max(0),
        second_score: second_score.max(0),
        top_record_index,
        top_fact_hash,
        checksum,
    }
}

fn score_linux_fixed_record_numeric(
    record: &PackedLinuxFact64,
    query: &LinuxHotNumericQuery,
) -> i64 {
    let mut score = i64::from(record.confidence) / 10;
    if query.boundary_intent && record.polarity_id == polarity_id("negative") {
        score += 55;
    }
    if query.positive_intent && record.polarity_id == polarity_id("positive") {
        score += 12;
    }
    if contains_hash(&query.route_hint_hashes, record.route_hash) {
        score += 35;
    }
    if contains_hash(&query.relation_hint_hashes, record.relation_hash) {
        score += 45;
    }
    if contains_hash(&query.token_hashes, record.object_hash) {
        score += 95;
    }
    if contains_hash(&query.token_hashes, record.subject_hash) {
        score += 75;
    }
    if contains_hash(&query.token_hashes, record.relation_hash) {
        score += 15;
    }
    score
}

fn compile_linux_hot_numeric_query(query: &str) -> LinuxHotNumericQuery {
    let query_lower = query.to_ascii_lowercase();
    let mut token_hashes = query_lower
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '/' && ch != '.' && ch != '-')
        .filter(|token| token.len() > 1)
        .map(hash64)
        .collect::<Vec<_>>();
    token_hashes.sort_unstable();
    token_hashes.dedup();

    let mut route_hints = Vec::new();
    let mut relation_hints = Vec::new();
    let boundary_intent =
        query_lower.contains("does not prove") || query_lower.contains("not prove");
    let positive_intent = !boundary_intent;
    if query_lower.contains("command") || query_lower.contains("binary") {
        route_hints.extend([
            "linux.apt.command.provider",
            "linux.apt.command.package-command",
            "linux.package.binary",
        ]);
        relation_hints.extend(["provides command", "provided by package", "provides binary"]);
    }
    if query_lower.contains("systemd") || query_lower.contains("service") {
        route_hints.push("linux.systemd.exec");
        relation_hints.push("execstart");
    }
    if boundary_intent {
        route_hints.extend(["linux.boundary.package", "linux.boundary.socket"]);
        relation_hints.push("does not prove");
    }
    let mut route_hint_hashes = route_hints.into_iter().map(hash64).collect::<Vec<_>>();
    route_hint_hashes.sort_unstable();
    route_hint_hashes.dedup();
    let mut relation_hint_hashes = relation_hints.into_iter().map(hash64).collect::<Vec<_>>();
    relation_hint_hashes.sort_unstable();
    relation_hint_hashes.dedup();

    LinuxHotNumericQuery {
        raw_query: query.to_string(),
        token_hashes,
        route_hint_hashes,
        relation_hint_hashes,
        boundary_intent,
        positive_intent,
    }
}

fn contains_hash(hashes: &[u64], value: u64) -> bool {
    hashes.binary_search(&value).is_ok()
}

fn scan_linux_hot_packet(
    packet: &LinuxHotPacketImage,
    query: &str,
    top_k: usize,
) -> LinuxHotAskField {
    let mut scored = packet
        .records
        .iter()
        .filter_map(|record| {
            let route = label(&packet.labels, record.route_id)?;
            let subject = label(&packet.labels, record.subject_id)?;
            let relation = label(&packet.labels, record.relation_id)?;
            let object = label(&packet.labels, record.object_id)?;
            let polarity = polarity_label(record.polarity_id);
            let score = score_linux_fact_terms(LinuxFactScoreTerms {
                query,
                route,
                subject,
                relation,
                object,
                polarity,
                layer: "",
                confidence: record.confidence,
            });
            (score > 0).then_some((score, record, route, subject, relation, object, polarity))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.fact_hash.cmp(&right.1.fact_hash))
    });
    let top_score = scored.first().map(|item| item.0).unwrap_or(0);
    let second_score = scored.get(1).map(|item| item.0).unwrap_or(0);
    let margin = top_score - second_score;
    let top_facts = scored
        .into_iter()
        .take(top_k)
        .map(
            |(score, record, route, subject, relation, object, polarity)| LinuxHotAskFact {
                score,
                route: route.to_string(),
                subject: subject.to_string(),
                relation: relation.to_string(),
                object: object.to_string(),
                polarity,
                confidence: record.confidence,
            },
        )
        .collect::<Vec<_>>();
    let state = linux_hot_state(top_score, top_facts.first().map(|fact| fact.polarity));
    LinuxHotAskField {
        state,
        top_score,
        margin,
        top_facts,
    }
}

#[derive(Clone, Copy)]
struct BuiltinLinuxHotEvalCase {
    id: &'static str,
    query: &'static str,
    expected_state: &'static str,
    expected_route: &'static str,
    expected_polarity: &'static str,
    expected_object_match: &'static str,
    expected_object_contains: &'static str,
}

fn linux_hot_eval_cases() -> Vec<BuiltinLinuxHotEvalCase> {
    vec![
        BuiltinLinuxHotEvalCase {
            id: "command-provider-bash",
            query: "which package provides command bash",
            expected_state: "FOCUSED",
            expected_route: "linux.apt.command",
            expected_polarity: "positive",
            expected_object_match: "command_anchor",
            expected_object_contains: "bash",
        },
        BuiltinLinuxHotEvalCase {
            id: "command-provider-systemctl",
            query: "which package provides command systemctl",
            expected_state: "FOCUSED",
            expected_route: "linux.package.binary",
            expected_polarity: "positive",
            expected_object_match: "command_anchor",
            expected_object_contains: "systemctl",
        },
        BuiltinLinuxHotEvalCase {
            id: "negative-package-installed-not-running",
            query: "package installed does not prove binary is running",
            expected_state: "BOUNDARY_FOCUSED",
            expected_route: "linux.boundary.package",
            expected_polarity: "negative",
            expected_object_match: "contains",
            expected_object_contains: "binary is running",
        },
        BuiltinLinuxHotEvalCase {
            id: "negative-port-listening-not-firewall",
            query: "port listening does not prove firewall allows external packets",
            expected_state: "BOUNDARY_FOCUSED",
            expected_route: "linux.boundary.socket",
            expected_polarity: "negative",
            expected_object_match: "contains",
            expected_object_contains: "firewall",
        },
    ]
}

fn eval_linux_hot_case(
    packet: &LinuxHotPacketImage,
    case: BuiltinLinuxHotEvalCase,
    top_k: usize,
) -> LinuxHotEvalCaseReport {
    let field = scan_linux_hot_packet(packet, case.query, top_k);
    let lexical = lexical_baseline_top(packet, case.query);
    let field_top = field.top_facts.first();
    let route_ok = field_top
        .map(|fact| fact.route.contains(case.expected_route))
        .unwrap_or(false);
    let object_ok = field_top
        .map(|fact| object_matches_case(case, &fact.object))
        .unwrap_or(false);
    let polarity_ok = field_top
        .map(|fact| fact.polarity == case.expected_polarity)
        .unwrap_or(false);
    let passed = field.state == case.expected_state && route_ok && object_ok && polarity_ok;
    let lexical_passed = lexical
        .as_ref()
        .map(|fact| {
            fact.route.contains(case.expected_route)
                && object_matches_case(case, &fact.object)
                && fact.polarity == case.expected_polarity
        })
        .unwrap_or(false);
    let field_beats_lexical_baseline = passed && !lexical_passed;
    LinuxHotEvalCaseReport {
        id: case.id,
        query: case.query,
        expected_state: case.expected_state,
        expected_route: case.expected_route,
        expected_polarity: case.expected_polarity,
        expected_object_match: case.expected_object_match,
        expected_object_contains: case.expected_object_contains,
        field_state: field.state,
        field_top_score: field.top_score,
        field_top_route: field_top.map(|fact| fact.route.clone()),
        field_top_object: field_top.map(|fact| fact.object.clone()),
        field_top_polarity: field_top.map(|fact| fact.polarity),
        lexical_top_route: lexical.as_ref().map(|fact| fact.route.clone()),
        lexical_top_object: lexical.as_ref().map(|fact| fact.object.clone()),
        lexical_top_score: lexical.as_ref().map(|fact| fact.score).unwrap_or(0),
        field_beats_lexical_baseline,
        passed,
    }
}

fn object_matches_case(case: BuiltinLinuxHotEvalCase, object: &str) -> bool {
    match case.expected_object_match {
        "command_anchor" => {
            object == case.expected_object_contains
                || object.ends_with(&format!("/{}", case.expected_object_contains))
        }
        _ => object.contains(case.expected_object_contains),
    }
}

fn lexical_baseline_top(packet: &LinuxHotPacketImage, query: &str) -> Option<LinuxHotAskFact> {
    let query_tokens = query
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() > 2)
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    packet
        .records
        .iter()
        .filter_map(|record| {
            let route = label(&packet.labels, record.route_id)?;
            let subject = label(&packet.labels, record.subject_id)?;
            let relation = label(&packet.labels, record.relation_id)?;
            let object = label(&packet.labels, record.object_id)?;
            let polarity = polarity_label(record.polarity_id);
            let haystack = format!("{route} {subject} {relation} {object}").to_ascii_lowercase();
            let mut score = i64::from(record.confidence) / 10;
            for token in &query_tokens {
                if haystack.contains(token) {
                    score += 10;
                }
            }
            (score > 0).then_some(LinuxHotAskFact {
                score,
                route: route.to_string(),
                subject: subject.to_string(),
                relation: relation.to_string(),
                object: object.to_string(),
                polarity,
                confidence: record.confidence,
            })
        })
        .max_by(|left, right| {
            left.score
                .cmp(&right.score)
                .then_with(|| right.route.cmp(&left.route))
        })
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        round4(part as f32 / total as f32)
    }
}

fn build_linux_domain_query_wave(query: &str) -> LinuxDomainQueryWave {
    let tokens = query
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() > 1)
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let command_anchor = tokens
        .windows(2)
        .find_map(|pair| (pair[0] == "command").then(|| pair[1].clone()));
    let query_lower = query.to_ascii_lowercase();
    let boundary_intent =
        query_lower.contains("does not prove") || query_lower.contains("not prove");
    let route_hint = if command_anchor.is_some() {
        "command-provider"
    } else if boundary_intent {
        "negative-boundary"
    } else if query_lower.contains("systemd") || query_lower.contains("service") {
        "systemd-runtime"
    } else {
        "linux-general"
    };
    let polarity = if boundary_intent {
        "negative"
    } else {
        "positive"
    };
    LinuxDomainQueryWave {
        raw_query: query.to_string(),
        tokens,
        command_anchor,
        boundary_intent,
        route_hint,
        polarity,
    }
}

pub(crate) fn build_linux_hot_ask_report(config: LinuxHotAskConfig) -> Result<LinuxHotAskReport> {
    let hot_bytes = fs::read(&config.hot_pack)
        .with_context(|| format!("read linux hot packet {}", config.hot_pack.display()))?;
    let packet = parse_laf_packet(&hot_bytes)?;
    let field = scan_linux_hot_packet(&packet, &config.query, config.top_k.max(1));
    let state = field.state;
    let safe_to_answer = matches!(state, "FOCUSED" | "BOUNDARY_FOCUSED");
    let verdict = if safe_to_answer {
        "LINUX_HOT_SCAN_READY_NOT_LLM"
    } else {
        "LINUX_HOT_SCAN_REVIEW_NOT_LLM"
    };
    let answer_text = build_linux_hot_answer_text(state, &field.top_facts);

    Ok(LinuxHotAskReport {
        mode: "llmwave-big-linux-ask-hot",
        version: LINUX_HOT_PACKET_VERSION,
        verdict,
        query: config.query,
        hot_pack: LinuxHotAskPackSummary {
            path: config.hot_pack.display().to_string(),
            bytes_scanned: packet.actual_file_bytes,
            fixed_record_count: packet.records.len(),
            fixed_record_bytes: LAF_RECORD_BYTES,
            cold_label_count: packet.labels.len(),
            cold_label_table_bytes: packet.cold_label_table_bytes,
            fixed_records_fit_6m: packet.records.len() * LAF_RECORD_BYTES <= SIX_MIB_BYTES,
        },
        field,
        answer: LinuxHotAnswer {
            state,
            safe_to_answer,
            text: answer_text,
        },
        claim_boundary: LinuxHotAskClaimBoundary {
            binary_hot_packet_loaded: true,
            fixed_records_scanned: true,
            json_used_in_hot_scan: false,
            cold_labels_used_for_display: true,
            cache_only_execution_proven: false,
            exposure_layer_ready: false,
            broad_chat_llm_ready: false,
            nonlinear_memory_proven: false,
        },
    })
}

pub(crate) fn build_linux_hot_eval_report(
    config: LinuxHotEvalConfig,
) -> Result<LinuxHotEvalReport> {
    let hot_bytes = fs::read(&config.hot_pack)
        .with_context(|| format!("read linux hot packet {}", config.hot_pack.display()))?;
    let packet = parse_laf_packet(&hot_bytes)?;
    let top_k = config.top_k.max(1);
    let cases = linux_hot_eval_cases()
        .into_iter()
        .map(|case| eval_linux_hot_case(&packet, case, top_k))
        .collect::<Vec<_>>();
    let total = cases.len();
    let passed = cases.iter().filter(|case| case.passed).count();
    let lexical_duel_total = cases.len();
    let lexical_duel_wins = cases
        .iter()
        .filter(|case| case.field_beats_lexical_baseline)
        .count();
    let boundary_cases = cases
        .iter()
        .filter(|case| case.expected_polarity == "negative")
        .count();
    let boundary_cases_passed = cases
        .iter()
        .filter(|case| case.expected_polarity == "negative" && case.passed)
        .count();
    let false_positive_count = cases
        .iter()
        .filter(|case| !case.passed && matches!(case.field_state, "FOCUSED" | "BOUNDARY_FOCUSED"))
        .count();
    let pass_rate = ratio(passed, total);
    let lexical_duel_win_rate = ratio(lexical_duel_wins, lexical_duel_total);
    let false_positive_rate = ratio(false_positive_count, total);
    let verdict = if total > 0 && passed == total && lexical_duel_wins >= 1 {
        "LINUX_HOT_EVAL_PASS_NOT_LLM"
    } else {
        "LINUX_HOT_EVAL_REVIEW_NOT_LLM"
    };
    Ok(LinuxHotEvalReport {
        mode: "llmwave-big-linux-hot-eval",
        version: LINUX_HOT_PACKET_VERSION,
        verdict,
        hot_pack: LinuxHotAskPackSummary {
            path: config.hot_pack.display().to_string(),
            bytes_scanned: packet.actual_file_bytes,
            fixed_record_count: packet.records.len(),
            fixed_record_bytes: LAF_RECORD_BYTES,
            cold_label_count: packet.labels.len(),
            cold_label_table_bytes: packet.cold_label_table_bytes,
            fixed_records_fit_6m: packet.records.len() * LAF_RECORD_BYTES <= SIX_MIB_BYTES,
        },
        cases,
        metrics: LinuxHotEvalMetrics {
            total,
            passed,
            pass_rate,
            lexical_duel_total,
            lexical_duel_wins,
            lexical_duel_win_rate,
            boundary_cases,
            boundary_cases_passed,
            false_positive_rate,
        },
        claim_boundary: LinuxHotEvalClaimBoundary {
            binary_hot_packet_loaded: true,
            fixed_records_scanned: true,
            lexical_baseline_duel_run: true,
            linux_domain_eval_ready: verdict == "LINUX_HOT_EVAL_PASS_NOT_LLM",
            broad_chat_llm_ready: false,
            nonlinear_memory_proven: false,
            cache_only_execution_proven: false,
            safe_claim: "Linux hot eval checks fixed-record retrieval against a lexical baseline on domain probes. It is a local Linux-domain gate, not a general LLM or nonlinear-memory proof.",
        },
    })
}

pub(crate) fn build_linux_domain_run_report(
    config: LinuxDomainRunConfig,
) -> Result<LinuxDomainRunReport> {
    let ask = build_linux_hot_ask_report(LinuxHotAskConfig {
        hot_pack: config.hot_pack.clone(),
        query: config.query.clone(),
        top_k: config.top_k,
    })?;
    let eval = build_linux_hot_eval_report(LinuxHotEvalConfig {
        hot_pack: config.hot_pack.clone(),
        top_k: config.top_k,
    })?;
    let query_wave = build_linux_domain_query_wave(&config.query);
    let top_fact = ask.field.top_facts.first();
    let lexical_traps = eval
        .cases
        .iter()
        .filter(|case| case.field_beats_lexical_baseline)
        .map(|case| LinuxDomainRejectedTrap {
            case_id: case.id,
            lexical_route: case.lexical_top_route.clone(),
            lexical_object: case.lexical_top_object.clone(),
            field_route: case.field_top_route.clone(),
            field_object: case.field_top_object.clone(),
        })
        .collect::<Vec<_>>();
    let hot_scan_safe = ask.answer.safe_to_answer;
    let eval_gate_passed = eval.verdict == "LINUX_HOT_EVAL_PASS_NOT_LLM";
    let answer_allowed =
        hot_scan_safe && eval_gate_passed && eval.metrics.false_positive_rate == 0.0;
    let mut reason_codes = Vec::new();
    if !hot_scan_safe {
        reason_codes.push("hot_scan_not_safe");
    }
    if !eval_gate_passed {
        reason_codes.push("linux_domain_eval_not_passed");
    }
    if eval.metrics.false_positive_rate > 0.0 {
        reason_codes.push("false_positive_rate_nonzero");
    }
    if reason_codes.is_empty() {
        reason_codes.push("local_linux_domain_answer_verified");
    }
    let verifier_state = if answer_allowed {
        "LINUX_DOMAIN_VERIFIED"
    } else {
        "LINUX_DOMAIN_REVIEW"
    };
    let verdict = if answer_allowed {
        "LINUX_DOMAIN_LLMWAVE_READY_NOT_GENERAL_LLM"
    } else {
        "LINUX_DOMAIN_LLMWAVE_REVIEW_NOT_GENERAL_LLM"
    };
    Ok(LinuxDomainRunReport {
        mode: "llmwave-big-linux-domain-run",
        version: LINUX_HOT_PACKET_VERSION,
        verdict,
        hot_pack: config.hot_pack.display().to_string(),
        query_wave,
        reasoning_loop: LinuxDomainReasoningLoop {
            steps: vec![
                "query_wave",
                "fixed_record_hot_scan",
                "route_peak_selection",
                "lexical_baseline_duel",
                "answer_verifier",
                "feedback_packet_preview",
            ],
            field_state: ask.field.state,
            selected_route: top_fact.map(|fact| fact.route.clone()),
            selected_object: top_fact.map(|fact| fact.object.clone()),
            lexical_trap_count: lexical_traps.len(),
            baseline_duel_passed: eval.metrics.lexical_duel_wins >= 1,
        },
        answer_surface: LinuxDomainAnswerSurface {
            state: ask.answer.state,
            safe_to_answer: ask.answer.safe_to_answer,
            text: ask.answer.text,
            free_form_generation: false,
        },
        verifier: LinuxDomainVerifier {
            state: verifier_state,
            answer_allowed,
            hot_scan_safe,
            eval_gate_passed,
            false_positive_rate: eval.metrics.false_positive_rate,
            reason_codes,
        },
        feedback_learning: LinuxDomainFeedbackLearning {
            state: if lexical_traps.is_empty() {
                "FEEDBACK_PREVIEW_NO_TRAPS"
            } else {
                "FEEDBACK_PREVIEW_READY"
            },
            feedback_packet_preview_ready: true,
            persistent_memory_written: false,
            accepted_route: top_fact.map(|fact| fact.route.clone()),
            rejected_lexical_traps: lexical_traps,
        },
        eval_metrics: eval.metrics,
        claim_boundary: LinuxDomainClaimBoundary {
            linux_domain_core_ready: answer_allowed,
            binary_hot_packet_loaded: true,
            query_wave_used: true,
            reasoning_loop_used: true,
            answer_verified: answer_allowed,
            feedback_learning_preview_ready: true,
            persistent_training_done: false,
            exposure_layer_ready: false,
            cache_only_execution_proven: false,
            broad_chat_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: "Linux Domain LLMWave can answer constrained Linux hot-field queries after a local eval/verifier gate. It is not a general LLM, not an exposure scanner, and not nonlinear-memory proof.",
            forbidden_claims: vec![
                "general LLM ready",
                "broad chat ready",
                "full cache-only execution proven",
                "nonlinear memory proven",
                "vulnerability scanner ready",
            ],
        },
    })
}

fn pack_fact(fact: &LinuxAtlasFact, labels: &mut LabelTableBuilder) -> PackedLinuxFact64 {
    PackedLinuxFact64 {
        fact_hash: hash64(&fact.fact_id),
        route_hash: hash64(&fact.route),
        subject_hash: hash64(&fact.subject),
        relation_hash: hash64(&fact.relation),
        object_hash: hash64(&fact.object),
        route_id: labels.intern(&fact.route),
        subject_id: labels.intern(&fact.subject),
        relation_id: labels.intern(&fact.relation),
        object_id: labels.intern(&fact.object),
        confidence: fact.confidence,
        polarity_id: polarity_id(&fact.polarity),
        reserved_u16: 0,
        evidence_hash: hash32(&format!(
            "{}:{}:{}",
            fact.evidence.source_kind, fact.evidence.path, fact.evidence.line
        )),
    }
}

fn write_laf_header(
    writer: &mut impl Write,
    record_count: usize,
    label_count: usize,
    route_count: usize,
    corpus_hash: &str,
) -> Result<()> {
    writer.write_all(LAF_MAGIC)?;
    write_u32(writer, LAF_FORMAT_VERSION)?;
    write_u32(writer, WAVE_DIM as u32)?;
    write_u32(writer, record_count as u32)?;
    write_u32(writer, label_count as u32)?;
    write_u32(writer, route_count as u32)?;
    write_u32(writer, LAF_RECORD_BYTES as u32)?;
    write_u64(writer, hash64(corpus_hash))?;
    let written = 8 + 6 * 4 + 8;
    writer.write_all(&vec![0u8; LAF_HEADER_BYTES - written])?;
    Ok(())
}

fn write_record(writer: &mut impl Write, record: &PackedLinuxFact64) -> Result<()> {
    write_u64(writer, record.fact_hash)?;
    write_u64(writer, record.route_hash)?;
    write_u64(writer, record.subject_hash)?;
    write_u64(writer, record.relation_hash)?;
    write_u64(writer, record.object_hash)?;
    write_u32(writer, record.route_id)?;
    write_u32(writer, record.subject_id)?;
    write_u32(writer, record.relation_id)?;
    write_u32(writer, record.object_id)?;
    writer.write_all(&[record.confidence, record.polarity_id])?;
    write_u16(writer, record.reserved_u16)?;
    write_u32(writer, record.evidence_hash)?;
    Ok(())
}

fn write_label_table(writer: &mut impl Write, labels: &[String]) -> Result<usize> {
    let mut bytes = 0usize;
    for (id, label) in labels.iter().enumerate() {
        let label_bytes = label.as_bytes();
        write_u32(writer, id as u32)?;
        write_u32(writer, label_bytes.len() as u32)?;
        writer.write_all(label_bytes)?;
        bytes += 8 + label_bytes.len();
    }
    Ok(bytes)
}

fn parse_laf_fixed_records_only(path: &PathBuf) -> Result<LinuxHotFixedRecordImage> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("open linux hot packet {}", path.display()))?;
    let actual_file_bytes = file
        .metadata()
        .with_context(|| format!("stat linux hot packet {}", path.display()))?
        .len() as usize;
    let mut header_bytes = [0u8; LAF_HEADER_BYTES];
    file.read_exact(&mut header_bytes)
        .with_context(|| format!("read linux hot packet header {}", path.display()))?;
    let header = parse_laf_header(&header_bytes)?;
    let records_bytes = header
        .record_count
        .checked_mul(LAF_RECORD_BYTES)
        .context("linux hot packet fixed record bytes overflow")?;
    if LAF_HEADER_BYTES + records_bytes > actual_file_bytes {
        bail!("truncated linux hot packet fixed record section");
    }
    let mut records = Vec::with_capacity(header.record_count);
    let mut record_bytes = [0u8; LAF_RECORD_BYTES];
    for _ in 0..header.record_count {
        file.read_exact(&mut record_bytes)
            .with_context(|| format!("read linux hot fixed record {}", path.display()))?;
        let mut cursor = Cursor::new(&record_bytes[..]);
        records.push(read_record(&mut cursor)?);
    }
    Ok(LinuxHotFixedRecordImage {
        records,
        header,
        actual_file_bytes,
        cold_label_table_bytes: actual_file_bytes.saturating_sub(LAF_HEADER_BYTES + records_bytes),
        detected_l3_bytes: detect_l3_cache_bytes(),
    })
}

fn parse_laf_packet(bytes: &[u8]) -> Result<LinuxHotPacketImage> {
    if bytes.len() < LAF_HEADER_BYTES {
        bail!("truncated linux hot packet header");
    }
    let header = parse_laf_header(&bytes[..LAF_HEADER_BYTES])?;
    let record_count = header.record_count;
    let label_count = header.label_count;
    let mut cursor = Cursor::new(bytes);
    cursor.set_position(LAF_HEADER_BYTES as u64);
    let records_bytes = record_count
        .checked_mul(LAF_RECORD_BYTES)
        .context("linux hot packet record bytes overflow")?;
    if LAF_HEADER_BYTES + records_bytes > bytes.len() {
        bail!("truncated linux hot packet records");
    }
    let mut records = Vec::with_capacity(record_count);
    for _ in 0..record_count {
        records.push(read_record(&mut cursor)?);
    }
    let label_table_start = cursor.position() as usize;
    let mut labels = vec![String::new(); label_count];
    for _ in 0..label_count {
        let id = read_u32(&mut cursor)? as usize;
        let len = read_u32(&mut cursor)? as usize;
        if id >= label_count {
            bail!("linux hot packet label id out of range");
        }
        let end = cursor
            .position()
            .checked_add(len as u64)
            .context("linux hot packet label cursor overflow")?;
        if end > bytes.len() as u64 {
            bail!("truncated linux hot packet label");
        }
        let start = cursor.position() as usize;
        let end_usize = end as usize;
        labels[id] = std::str::from_utf8(&bytes[start..end_usize])
            .context("linux hot packet label is not utf-8")?
            .to_string();
        cursor.set_position(end);
    }
    Ok(LinuxHotPacketImage {
        records,
        labels,
        cold_label_table_bytes: bytes.len().saturating_sub(label_table_start),
        actual_file_bytes: bytes.len(),
    })
}

fn parse_laf_header(bytes: &[u8]) -> Result<LafHeaderFields> {
    let mut cursor = Cursor::new(bytes);
    let mut magic = [0u8; 8];
    cursor.read_exact(&mut magic)?;
    if &magic != LAF_MAGIC {
        bail!("invalid linux hot packet magic");
    }
    let version = read_u32(&mut cursor)?;
    if version != LAF_FORMAT_VERSION {
        bail!("unsupported linux hot packet version {version}");
    }
    let wave_dim = read_u32(&mut cursor)?;
    let record_count = read_u32(&mut cursor)? as usize;
    let label_count = read_u32(&mut cursor)? as usize;
    let route_count = read_u32(&mut cursor)? as usize;
    let record_bytes = read_u32(&mut cursor)? as usize;
    if record_bytes != LAF_RECORD_BYTES {
        bail!("unsupported linux hot record size {record_bytes}");
    }
    let corpus_hash = read_u64(&mut cursor)?;
    Ok(LafHeaderFields {
        wave_dim,
        record_count,
        label_count,
        route_count,
        record_bytes,
        corpus_hash,
    })
}

fn read_record(cursor: &mut Cursor<&[u8]>) -> Result<PackedLinuxFact64> {
    Ok(PackedLinuxFact64 {
        fact_hash: read_u64(cursor)?,
        route_hash: read_u64(cursor)?,
        subject_hash: read_u64(cursor)?,
        relation_hash: read_u64(cursor)?,
        object_hash: read_u64(cursor)?,
        route_id: read_u32(cursor)?,
        subject_id: read_u32(cursor)?,
        relation_id: read_u32(cursor)?,
        object_id: read_u32(cursor)?,
        confidence: read_u8(cursor)?,
        polarity_id: read_u8(cursor)?,
        reserved_u16: read_u16(cursor)?,
        evidence_hash: read_u32(cursor)?,
    })
}

fn label(labels: &[String], id: u32) -> Option<&str> {
    labels.get(id as usize).map(String::as_str)
}

fn linux_hot_state(top_score: i64, polarity: Option<&str>) -> &'static str {
    if top_score >= 90 && polarity == Some("negative") {
        "BOUNDARY_FOCUSED"
    } else if top_score >= 80 {
        "FOCUSED"
    } else if top_score >= 35 {
        "REVIEW"
    } else {
        "NO_MATCH"
    }
}

fn build_linux_hot_answer_text(state: &str, facts: &[LinuxHotAskFact]) -> String {
    let Some(top) = facts.first() else {
        return "No Linux hot-packet fact matched this query.".to_string();
    };
    match state {
        "BOUNDARY_FOCUSED" => format!(
            "Boundary fact: {} {} {}.",
            top.subject, top.relation, top.object
        ),
        "FOCUSED" => format!(
            "Focused fact: {} {} {} via {}.",
            top.subject, top.relation, top.object, top.route
        ),
        "REVIEW" => format!(
            "Weak hot-field match: {} {} {}; inspect evidence before acting.",
            top.subject, top.relation, top.object
        ),
        _ => "No safe Linux hot-packet answer; keep this query review-only.".to_string(),
    }
}

fn polarity_id(value: &str) -> u8 {
    match value {
        "positive" => 1,
        "negative" => 2,
        _ => 0,
    }
}

fn polarity_label(value: u8) -> &'static str {
    match value {
        1 => "positive",
        2 => "negative",
        _ => "unknown",
    }
}

fn hash64(value: &str) -> u64 {
    let digest = Sha256::digest(value.as_bytes());
    u64::from_le_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ])
}

fn hash32(value: &str) -> u32 {
    let digest = Sha256::digest(value.as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

fn round4(value: f32) -> f32 {
    (value * 10_000.0).round() / 10_000.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round4_f64(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len().saturating_sub(1)) as f64 * percentile).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn detect_l3_cache_bytes() -> Option<usize> {
    let cache_root = std::path::Path::new("/sys/devices/system/cpu/cpu0/cache");
    let entries = fs::read_dir(cache_root).ok()?;
    let mut best = None;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(level) = fs::read_to_string(path.join("level")).ok() else {
            continue;
        };
        if level.trim() != "3" {
            continue;
        }
        let Some(size) = fs::read_to_string(path.join("size")).ok() else {
            continue;
        };
        let Some(bytes) = parse_linux_cache_size_bytes(size.trim()) else {
            continue;
        };
        best = Some(best.map_or(bytes, |current: usize| current.max(bytes)));
    }
    best
}

fn parse_linux_cache_size_bytes(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    let split_at = trimmed
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let number = trimmed[..split_at].parse::<usize>().ok()?;
    let unit = trimmed[split_at..].trim().to_ascii_uppercase();
    match unit.as_str() {
        "K" | "KB" | "KIB" => number.checked_mul(1024),
        "M" | "MB" | "MIB" => number.checked_mul(1024 * 1024),
        "" => Some(number),
        _ => None,
    }
}

fn write_u16(writer: &mut impl Write, value: u16) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_u32(writer: &mut impl Write, value: u32) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_u64(writer: &mut impl Write, value: u64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut bytes = [0u8; 1];
    cursor.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut bytes = [0u8; 2];
    cursor.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut bytes = [0u8; 4];
    cursor.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let mut bytes = [0u8; 8];
    cursor.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

struct PmuMeasuredCounters {
    cache_references: u64,
    cache_misses: u64,
    elapsed_ns: u128,
    checksum: u64,
}

struct PmuBlocked {
    reason: String,
    errno: Option<i32>,
}

enum PmuMeasureOutcome {
    Measured(PmuMeasuredCounters),
    Blocked(PmuBlocked),
}

#[cfg(target_os = "linux")]
mod platform_pmu {
    use super::{PmuBlocked, PmuMeasureOutcome, PmuMeasuredCounters};
    use std::ffi::c_void;
    use std::io;
    use std::mem;
    use std::os::raw::{c_int, c_long, c_ulong};
    use std::time::Instant;

    const PERF_TYPE_HARDWARE: u32 = 0;
    const PERF_COUNT_HW_CACHE_REFERENCES: u64 = 2;
    const PERF_COUNT_HW_CACHE_MISSES: u64 = 3;
    const PERF_EVENT_IOC_ENABLE: c_ulong = 0x2400;
    const PERF_EVENT_IOC_DISABLE: c_ulong = 0x2401;
    const PERF_EVENT_IOC_RESET: c_ulong = 0x2403;
    const PERF_ATTR_DISABLED: u64 = 1 << 0;
    const PERF_ATTR_EXCLUDE_KERNEL: u64 = 1 << 5;
    const PERF_ATTR_EXCLUDE_HV: u64 = 1 << 6;

    #[cfg(target_arch = "x86_64")]
    const SYS_PERF_EVENT_OPEN: c_long = 298;
    #[cfg(target_arch = "aarch64")]
    const SYS_PERF_EVENT_OPEN: c_long = 241;
    #[cfg(target_arch = "arm")]
    const SYS_PERF_EVENT_OPEN: c_long = 364;
    #[cfg(target_arch = "riscv64")]
    const SYS_PERF_EVENT_OPEN: c_long = 241;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PerfEventAttr {
        type_: u32,
        size: u32,
        config: u64,
        sample_period_or_freq: u64,
        sample_type: u64,
        read_format: u64,
        flags: u64,
        wakeup_events_or_watermark: u32,
        bp_type: u32,
        bp_addr_or_config1: u64,
        bp_len_or_config2: u64,
        branch_sample_type: u64,
        sample_regs_user: u64,
        sample_stack_user: u32,
        clockid: i32,
        sample_regs_intr: u64,
        aux_watermark: u32,
        sample_max_stack: u16,
        reserved_2: u16,
        aux_sample_size: u32,
        reserved_3: u32,
        sig_data: u64,
        config3: u64,
    }

    unsafe extern "C" {
        fn syscall(num: c_long, ...) -> c_long;
        fn close(fd: c_int) -> c_int;
        fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
        fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }

    pub(super) fn measure_cache_counters(run: impl FnOnce() -> u64) -> PmuMeasureOutcome {
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64"
        )))]
        {
            let checksum = run();
            return PmuMeasureOutcome::Blocked(PmuBlocked {
                reason: format!(
                    "perf_event_open_syscall_not_defined_for_arch; checksum={checksum}"
                ),
                errno: None,
            });
        }

        let refs_fd = match open_hardware_counter(PERF_COUNT_HW_CACHE_REFERENCES) {
            Ok(fd) => fd,
            Err(blocked) => {
                let checksum = run();
                return PmuMeasureOutcome::Blocked(PmuBlocked {
                    reason: format!(
                        "open_cache_references_failed:{}; checksum={checksum}",
                        blocked.reason
                    ),
                    errno: blocked.errno,
                });
            }
        };
        let misses_fd = match open_hardware_counter(PERF_COUNT_HW_CACHE_MISSES) {
            Ok(fd) => fd,
            Err(blocked) => {
                unsafe {
                    close(refs_fd);
                }
                let checksum = run();
                return PmuMeasureOutcome::Blocked(PmuBlocked {
                    reason: format!(
                        "open_cache_misses_failed:{}; checksum={checksum}",
                        blocked.reason
                    ),
                    errno: blocked.errno,
                });
            }
        };

        let reset_ok = counter_ioctl(refs_fd, PERF_EVENT_IOC_RESET)
            && counter_ioctl(misses_fd, PERF_EVENT_IOC_RESET);
        let enable_ok = counter_ioctl(refs_fd, PERF_EVENT_IOC_ENABLE)
            && counter_ioctl(misses_fd, PERF_EVENT_IOC_ENABLE);
        if !reset_ok || !enable_ok {
            unsafe {
                close(refs_fd);
                close(misses_fd);
            }
            let checksum = run();
            return PmuMeasureOutcome::Blocked(PmuBlocked {
                reason: format!("perf_counter_ioctl_failed; checksum={checksum}"),
                errno: io::Error::last_os_error().raw_os_error(),
            });
        }

        let start = Instant::now();
        let checksum = run();
        let elapsed_ns = start.elapsed().as_nanos();
        let disable_ok = counter_ioctl(refs_fd, PERF_EVENT_IOC_DISABLE)
            && counter_ioctl(misses_fd, PERF_EVENT_IOC_DISABLE);
        let refs = read_counter(refs_fd);
        let misses = read_counter(misses_fd);
        unsafe {
            close(refs_fd);
            close(misses_fd);
        }
        if !disable_ok {
            return PmuMeasureOutcome::Blocked(PmuBlocked {
                reason: format!("perf_counter_disable_failed; checksum={checksum}"),
                errno: io::Error::last_os_error().raw_os_error(),
            });
        }
        match (refs, misses) {
            (Ok(cache_references), Ok(cache_misses)) => {
                PmuMeasureOutcome::Measured(PmuMeasuredCounters {
                    cache_references,
                    cache_misses,
                    elapsed_ns,
                    checksum,
                })
            }
            (Err(blocked), _) | (_, Err(blocked)) => PmuMeasureOutcome::Blocked(blocked),
        }
    }

    fn open_hardware_counter(config: u64) -> Result<c_int, PmuBlocked> {
        let attr = PerfEventAttr {
            type_: PERF_TYPE_HARDWARE,
            size: mem::size_of::<PerfEventAttr>() as u32,
            config,
            sample_period_or_freq: 0,
            sample_type: 0,
            read_format: 0,
            flags: PERF_ATTR_DISABLED | PERF_ATTR_EXCLUDE_KERNEL | PERF_ATTR_EXCLUDE_HV,
            wakeup_events_or_watermark: 0,
            bp_type: 0,
            bp_addr_or_config1: 0,
            bp_len_or_config2: 0,
            branch_sample_type: 0,
            sample_regs_user: 0,
            sample_stack_user: 0,
            clockid: 0,
            sample_regs_intr: 0,
            aux_watermark: 0,
            sample_max_stack: 0,
            reserved_2: 0,
            aux_sample_size: 0,
            reserved_3: 0,
            sig_data: 0,
            config3: 0,
        };
        let fd = unsafe { syscall(SYS_PERF_EVENT_OPEN, &attr, 0, -1, -1, 0) };
        if fd < 0 {
            let err = io::Error::last_os_error();
            Err(PmuBlocked {
                reason: err.to_string(),
                errno: err.raw_os_error(),
            })
        } else {
            Ok(fd as c_int)
        }
    }

    fn counter_ioctl(fd: c_int, request: c_ulong) -> bool {
        unsafe { ioctl(fd, request, 0) == 0 }
    }

    fn read_counter(fd: c_int) -> Result<u64, PmuBlocked> {
        let mut value = 0u64;
        let result = unsafe {
            read(
                fd,
                (&mut value as *mut u64).cast::<c_void>(),
                mem::size_of::<u64>(),
            )
        };
        if result == mem::size_of::<u64>() as isize {
            Ok(value)
        } else {
            let err = io::Error::last_os_error();
            Err(PmuBlocked {
                reason: err.to_string(),
                errno: err.raw_os_error(),
            })
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod platform_pmu {
    use super::{PmuBlocked, PmuMeasureOutcome};

    pub(super) fn measure_cache_counters(run: impl FnOnce() -> u64) -> PmuMeasureOutcome {
        let checksum = run();
        PmuMeasureOutcome::Blocked(PmuBlocked {
            reason: format!("perf_event_open_available_only_on_linux; checksum={checksum}"),
            errno: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::linux_atlas::LinuxAtlasEvidence;
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn linux_hot_packet_roundtrip_focuses_command_provider() {
        let root = unique_tmp_dir("linux-hot-roundtrip");
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let facts = vec![
            test_fact(
                "linux.apt.command.provider",
                "bash",
                "provided by package",
                "bash",
                "positive",
            ),
            test_fact(
                "linux.apt.command.provider",
                "sh",
                "provided by package",
                "dash",
                "positive",
            ),
            test_fact(
                "linux.boundary.package",
                "package installed",
                "does not prove",
                "binary is running",
                "negative",
            ),
        ];
        let mut file = fs::File::create(&facts_path).unwrap();
        for fact in facts {
            serde_json::to_writer(&mut file, &fact).unwrap();
            file.write_all(b"\n").unwrap();
        }
        let hot_path = root.join("linux.laf");
        let pack = build_linux_hot_pack_report(LinuxHotPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 3,
            out: hot_path.clone(),
        })
        .unwrap();
        assert_eq!(pack.packet.fixed_record_count, 3);
        assert!(pack.packet.fixed_records_fit_6m);
        assert_eq!(
            pack.schema_residual_memory.state,
            "SCHEMA_RESIDUAL_CANDIDATE"
        );
        assert!(pack.schema_residual_memory.residual_saving_bytes > 0);

        let ask = build_linux_hot_ask_report(LinuxHotAskConfig {
            hot_pack: hot_path,
            query: "which package provides command bash".to_string(),
            top_k: 3,
        })
        .unwrap();
        assert_eq!(ask.field.state, "FOCUSED");
        assert_eq!(ask.field.top_facts[0].object, "bash");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn linux_hot_packet_keeps_boundary_negative_focus() {
        let root = unique_tmp_dir("linux-hot-boundary");
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let fact = test_fact(
            "linux.boundary.package",
            "package installed",
            "does not prove",
            "binary is running",
            "negative",
        );
        let mut file = fs::File::create(&facts_path).unwrap();
        serde_json::to_writer(&mut file, &fact).unwrap();
        file.write_all(b"\n").unwrap();
        let hot_path = root.join("linux.laf");
        build_linux_hot_pack_report(LinuxHotPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 1,
            out: hot_path.clone(),
        })
        .unwrap();
        let ask = build_linux_hot_ask_report(LinuxHotAskConfig {
            hot_pack: hot_path,
            query: "package installed does not prove binary is running".to_string(),
            top_k: 1,
        })
        .unwrap();
        assert_eq!(ask.field.state, "BOUNDARY_FOCUSED");
        assert!(ask.answer.safe_to_answer);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn linux_cache_proof_uses_fixed_records_without_labels() {
        let root = unique_tmp_dir("linux-cache-proof");
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let facts = vec![
            test_fact(
                "linux.apt.command.provider",
                "bash",
                "provided by package",
                "bash",
                "positive",
            ),
            test_fact(
                "linux.apt.command.package-command",
                "checkbashisms",
                "provides command",
                "checkbashisms",
                "positive",
            ),
        ];
        let mut file = fs::File::create(&facts_path).unwrap();
        for fact in facts {
            serde_json::to_writer(&mut file, &fact).unwrap();
            file.write_all(b"\n").unwrap();
        }
        let hot_path = root.join("linux.laf");
        build_linux_hot_pack_report(LinuxHotPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 2,
            out: hot_path.clone(),
        })
        .unwrap();

        let proof = build_linux_cache_proof_report(LinuxCacheProofConfig {
            hot_pack: hot_path,
            query: "which package provides command bash".to_string(),
            iterations: 2,
            warmup_iterations: 1,
            samples: 2,
        })
        .unwrap();
        assert_eq!(proof.verdict, "LINUX_CACHE_ONLY_EXECUTION_PROVEN");
        assert!(proof.claim_boundary.cache_only_execution_proven);
        assert!(proof.runtime_contract.fixed_record_only_inner_loop);
        assert!(!proof.runtime_contract.labels_read_from_packet);
        assert!(!proof.runtime_contract.json_used_in_hot_loop);
        assert_eq!(proof.benchmark.records_per_scan, 2);
        assert!(proof.benchmark.top_score > 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn linux_pmu_cache_proof_reports_measured_or_blocked() {
        let root = unique_tmp_dir("linux-pmu-cache-proof");
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let facts = vec![
            test_fact(
                "linux.apt.command.provider",
                "bash",
                "provided by package",
                "bash",
                "positive",
            ),
            test_fact(
                "linux.boundary.package",
                "package installed",
                "does not prove",
                "binary is running",
                "negative",
            ),
        ];
        let mut file = fs::File::create(&facts_path).unwrap();
        for fact in facts {
            serde_json::to_writer(&mut file, &fact).unwrap();
            file.write_all(b"\n").unwrap();
        }
        let hot_path = root.join("linux.laf");
        build_linux_hot_pack_report(LinuxHotPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 2,
            out: hot_path.clone(),
        })
        .unwrap();

        let proof = build_linux_pmu_cache_proof_report(LinuxPmuCacheProofConfig {
            hot_pack: hot_path,
            query: "which package provides command bash".to_string(),
            iterations: 2,
            warmup_iterations: 1,
            samples: 2,
            max_cache_miss_rate: 1.0,
        })
        .unwrap();
        assert_eq!(proof.mode, "llmwave-big-linux-pmu-cache-proof");
        assert!(proof.claim_boundary.software_cache_only_execution_proven);
        assert!(matches!(
            proof.pmu.counter_status,
            "MEASURED" | "NO_REFERENCES" | "BLOCKED"
        ));
        if proof.pmu.counter_status == "MEASURED" {
            assert!(proof.pmu.cache_references.unwrap_or(0) > 0);
            assert!(proof.pmu.cache_misses.is_some());
        } else {
            assert!(proof.pmu.blocked_reason.is_some() || proof.pmu.cache_references == Some(0));
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn linux_cache_size_parser_handles_sysfs_units() {
        assert_eq!(parse_linux_cache_size_bytes("8192K"), Some(8 * 1024 * 1024));
        assert_eq!(parse_linux_cache_size_bytes("8M"), Some(8 * 1024 * 1024));
        assert_eq!(parse_linux_cache_size_bytes("4096"), Some(4096));
    }

    fn test_fact(
        route: &str,
        subject: &str,
        relation: &str,
        object: &str,
        polarity: &str,
    ) -> LinuxAtlasFact {
        LinuxAtlasFact {
            fact_id: format!("test.{route}.{subject}.{object}"),
            layer: if polarity == "negative" {
                "negative-boundary".to_string()
            } else {
                "linux-knowledge".to_string()
            },
            domain: "linux-test".to_string(),
            route: route.to_string(),
            subject: subject.to_string(),
            relation: relation.to_string(),
            object: object.to_string(),
            polarity: polarity.to_string(),
            confidence: 90,
            evidence: LinuxAtlasEvidence {
                source_kind: "fixture".to_string(),
                path: "fixture".to_string(),
                line: 1,
                extractor: "fixture".to_string(),
            },
        }
    }

    fn unique_tmp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }
}
