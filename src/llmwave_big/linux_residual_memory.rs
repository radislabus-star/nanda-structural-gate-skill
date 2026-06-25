//! Binary Linux schema/residual memory packet and proof path.
//!
//! `.laf` is the direct fixed-record Linux hot packet. `.lrf` is the nonlinear
//! memory candidate: repeated route+relation+polarity facts are promoted into
//! compact schema records, per-fact subject/object details become residuals, and
//! one-off facts remain full fixed fallbacks. The proof path scans those binary
//! sections directly and keeps labels as cold explain data.

use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::linux_active_field::{
    build_linux_active_fact_window, score_linux_fact_terms, LinuxFactScoreTerms,
};
use super::linux_atlas::{LinuxAtlasFact, LINUX_ATLAS_VERSION};
use crate::WAVE_DIM;

pub(crate) const LINUX_RESIDUAL_MEMORY_VERSION: &str = "llmwave-big-v-next-linux-schema-residual";

const LRF_MAGIC: &[u8; 8] = b"LLMWLRF1";
const LRF_FORMAT_VERSION: u32 = 1;
const LRF_HEADER_BYTES: usize = 64;
const SCHEMA_RECORD_BYTES: usize = 32;
const RESIDUAL_RECORD_BYTES: usize = 32;
const FALLBACK_RECORD_BYTES: usize = 64;
const DIRECT_FIXED_RECORD_BYTES: usize = 64;
const SIX_MIB_BYTES: usize = 6 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct LinuxResidualPackConfig {
    pub atlas_dir: PathBuf,
    pub max_active_facts: usize,
    pub promotion_threshold: usize,
    pub out: PathBuf,
}

#[derive(Clone)]
pub(crate) struct LinuxResidualProofConfig {
    pub residual_pack: PathBuf,
    pub query: String,
    pub top_k: usize,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub samples: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualPackReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub atlas_version: &'static str,
    pub verdict: &'static str,
    pub atlas_dir: String,
    pub out: String,
    pub source: LinuxResidualSource,
    pub packet: LinuxResidualPacketSummary,
    pub economics: LinuxResidualEconomics,
    pub claim_boundary: LinuxResidualPackClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualSource {
    pub fact_pack_count: usize,
    pub atlas_fact_count: usize,
    pub selected_facts: usize,
    pub route_count: usize,
    pub corpus_hash: String,
    pub route_distribution: BTreeMap<String, usize>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualPacketSummary {
    pub format: &'static str,
    pub header_bytes: usize,
    pub schema_record_bytes: usize,
    pub residual_record_bytes: usize,
    pub fallback_record_bytes: usize,
    pub schema_record_count: usize,
    pub residual_record_count: usize,
    pub fallback_record_count: usize,
    pub represented_fact_count: usize,
    pub binary_hot_sections_bytes: usize,
    pub direct_fixed_record_bytes: usize,
    pub direct_fixed_baseline_bytes: usize,
    pub cold_label_count: usize,
    pub cold_label_table_bytes: usize,
    pub actual_file_bytes: usize,
    pub hot_budget_bytes: usize,
    pub binary_hot_sections_fit_6m: bool,
    pub whole_file_fits_6m: bool,
    pub labels_are_cold_explain_data: bool,
    pub json_used_in_hot_scan: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualEconomics {
    pub schema_key: &'static str,
    pub promotion_threshold: usize,
    pub promoted_schema_count: usize,
    pub residual_fact_count: usize,
    pub full_fallback_fact_count: usize,
    pub residual_saving_bytes: isize,
    pub residual_saving_ratio: f32,
    pub schema_reuse_ratio: f32,
    pub bytes_per_fact_direct_fixed64: f32,
    pub bytes_per_fact_schema_residual_hot: f32,
    pub binary_schema_residual_written: bool,
    pub notes: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualPackClaimBoundary {
    pub linux_atlas_loaded: bool,
    pub binary_schema_residual_memory_written: bool,
    pub schema_records_written: bool,
    pub residual_records_written: bool,
    pub fallback_records_preserve_one_off_facts: bool,
    pub binary_hot_sections_fit_6m: bool,
    pub proof_scan_run: bool,
    pub nonlinear_memory_proven: bool,
    pub exposure_layer_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualProofReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualProofPackSummary,
    pub compiled_query: LinuxResidualQuerySummary,
    pub runtime_contract: LinuxResidualRuntimeContract,
    pub benchmark: LinuxResidualBenchmarkSummary,
    pub eval: LinuxResidualEvalSummary,
    pub economics: LinuxResidualEconomics,
    pub claim_boundary: LinuxResidualProofClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualProofPackSummary {
    pub path: String,
    pub file_bytes: usize,
    pub wave_dim: u32,
    pub represented_fact_count: usize,
    pub schema_record_count: usize,
    pub residual_record_count: usize,
    pub fallback_record_count: usize,
    pub route_count: usize,
    pub corpus_hash64: u64,
    pub promotion_threshold: usize,
    pub schema_record_bytes: usize,
    pub residual_record_bytes: usize,
    pub fallback_record_bytes: usize,
    pub binary_hot_sections_bytes: usize,
    pub direct_fixed_baseline_bytes: usize,
    pub cold_label_count: usize,
    pub cold_label_table_bytes: usize,
    pub hot_budget_bytes: usize,
    pub detected_l3_bytes: Option<usize>,
    pub binary_hot_sections_fit_6m: bool,
    pub binary_hot_sections_fit_detected_l3: Option<bool>,
    pub beats_direct_fixed64: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualQuerySummary {
    pub raw_query: String,
    pub token_count: usize,
    pub token_hash_count: usize,
    pub route_hint_hash_count: usize,
    pub relation_hint_hash_count: usize,
    pub boundary_intent: bool,
    pub positive_intent: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualRuntimeContract {
    pub records_loaded_from: &'static str,
    pub schema_residual_binary_sections_scanned: bool,
    pub fallback_fixed_records_scanned: bool,
    pub full_represented_fact_scan: bool,
    pub json_used_in_hot_loop: bool,
    pub labels_used_in_hot_loop: bool,
    pub file_io_in_hot_loop: bool,
    pub heap_allocation_in_inner_loop: bool,
    pub per_record_score_arrays: bool,
    pub measured_loop_uses_numeric_hashes: bool,
    pub cold_label_table_excluded_from_hot_loop: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualBenchmarkSummary {
    pub warmup_iterations: usize,
    pub measured_samples: usize,
    pub iterations_per_sample: usize,
    pub measured_scans: usize,
    pub represented_facts_per_scan: usize,
    pub measured_fact_visits: u128,
    pub ns_per_scan_min: f64,
    pub ns_per_scan_p50: f64,
    pub ns_per_scan_p95: f64,
    pub ns_per_scan_max: f64,
    pub ns_per_fact_p50: f64,
    pub scans_per_second_p50: f64,
    pub represented_facts_per_second_p50: f64,
    pub top_score: i64,
    pub margin: i64,
    pub top_kind: Option<&'static str>,
    pub top_index: Option<usize>,
    pub checksum: u64,
    pub sample_ns_per_scan: Vec<f64>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualEvalSummary {
    pub cases: Vec<LinuxResidualEvalCaseReport>,
    pub metrics: LinuxResidualEvalMetrics,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualEvalCaseReport {
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
    pub memory_kind: Option<&'static str>,
    pub lexical_top_route: Option<String>,
    pub lexical_top_object: Option<String>,
    pub lexical_top_score: i64,
    pub field_beats_lexical_baseline: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualEvalMetrics {
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
pub(crate) struct LinuxResidualProofClaimBoundary {
    pub binary_schema_residual_memory_loaded: bool,
    pub actual_binary_schema_residual_memory: bool,
    pub schema_reuse_ratio_threshold_met: bool,
    pub residual_saving_positive: bool,
    pub beats_direct_fixed64: bool,
    pub linux_domain_eval_passed: bool,
    pub false_positive_rate_zero: bool,
    pub binary_hot_sections_fit_6m: bool,
    pub nonlinear_memory_proven: bool,
    pub linux_profile_nonlinear_memory_proven: bool,
    pub exposure_layer_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub cache_only_execution_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualDecodedPacket {
    pub summary: LinuxResidualDecodedSummary,
    pub facts: Vec<LinuxResidualDecodedFact>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxResidualDecodedSummary {
    pub path: String,
    pub file_bytes: usize,
    pub wave_dim: u32,
    pub represented_fact_count: usize,
    pub schema_record_count: usize,
    pub residual_record_count: usize,
    pub fallback_record_count: usize,
    pub route_count: usize,
    pub corpus_hash64: u64,
    pub promotion_threshold: usize,
    pub binary_hot_sections_bytes: usize,
    pub direct_fixed_baseline_bytes: usize,
    pub cold_label_count: usize,
    pub cold_label_table_bytes: usize,
    pub binary_hot_sections_fit_6m: bool,
    pub beats_direct_fixed64: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxResidualDecodedFact {
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: &'static str,
    pub confidence: u8,
    pub memory_kind: &'static str,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SchemaKey {
    route: String,
    relation: String,
    polarity: String,
}

#[derive(Clone)]
struct LinuxSchemaRecord32 {
    schema_hash: u64,
    route_hash: u64,
    relation_hash: u64,
    route_id: u16,
    relation_id: u16,
    polarity_id: u8,
    mean_confidence: u8,
    fact_count_bucket: u16,
}

#[derive(Clone)]
struct LinuxResidualRecord32 {
    schema_index: u32,
    subject_hash: u64,
    object_hash: u64,
    subject_id: u32,
    object_id: u32,
    confidence: u8,
    evidence_hash16: u16,
    flags: u8,
}

#[derive(Clone)]
struct LinuxFallbackRecord64 {
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

struct LinuxResidualPacketImage {
    schemas: Vec<LinuxSchemaRecord32>,
    residuals: Vec<LinuxResidualRecord32>,
    fallbacks: Vec<LinuxFallbackRecord64>,
    labels: Vec<String>,
}

struct LinuxResidualSectionsImage {
    header: LrfHeaderFields,
    schemas: Vec<LinuxSchemaRecord32>,
    residuals: Vec<LinuxResidualRecord32>,
    fallbacks: Vec<LinuxFallbackRecord64>,
    actual_file_bytes: usize,
    cold_label_table_bytes: usize,
    detected_l3_bytes: Option<usize>,
}

#[derive(Clone, Copy)]
struct LrfHeaderFields {
    wave_dim: u32,
    schema_count: usize,
    residual_count: usize,
    fallback_count: usize,
    label_count: usize,
    route_count: usize,
    schema_record_bytes: usize,
    residual_record_bytes: usize,
    fallback_record_bytes: usize,
    corpus_hash: u64,
    represented_fact_count: usize,
    promotion_threshold: usize,
}

struct LabelTableBuilder {
    ids: BTreeMap<String, u32>,
    labels: Vec<String>,
}

struct LrfHeaderWriteSpec<'a> {
    schema_count: usize,
    residual_count: usize,
    fallback_count: usize,
    label_count: usize,
    route_count: usize,
    corpus_hash: &'a str,
    represented_fact_count: usize,
    promotion_threshold: usize,
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

struct LinuxResidualNumericQuery {
    raw_query: String,
    token_hashes: Vec<u64>,
    route_hint_hashes: Vec<u64>,
    relation_hint_hashes: Vec<u64>,
    boundary_intent: bool,
    positive_intent: bool,
}

#[derive(Clone, Copy)]
struct LinuxResidualNumericScan {
    top_score: i64,
    second_score: i64,
    top_kind: Option<&'static str>,
    top_index: Option<usize>,
    checksum: u64,
}

#[derive(Clone)]
struct LinuxResidualAskField {
    state: &'static str,
    top_score: i64,
    top_facts: Vec<LinuxResidualAskFact>,
}

#[derive(Clone)]
struct LinuxResidualAskFact {
    score: i64,
    route: String,
    object: String,
    polarity: &'static str,
    memory_kind: &'static str,
}

#[derive(Clone, Copy)]
struct BuiltinLinuxResidualEvalCase {
    id: &'static str,
    query: &'static str,
    expected_state: &'static str,
    expected_route: &'static str,
    expected_polarity: &'static str,
    expected_object_match: &'static str,
    expected_object_contains: &'static str,
}

pub(crate) fn build_linux_residual_pack_report(
    config: LinuxResidualPackConfig,
) -> Result<LinuxResidualPackReport> {
    let max_active_facts = config.max_active_facts.max(1);
    let promotion_threshold = config.promotion_threshold.max(2);
    let window = build_linux_active_fact_window(&config.atlas_dir, max_active_facts)?;
    let mut route_distribution = BTreeMap::<String, usize>::new();
    for fact in &window.active_facts {
        *route_distribution.entry(fact.route.clone()).or_insert(0) += 1;
    }
    let mut labels = LabelTableBuilder::new();
    let (schemas, residuals, fallbacks) =
        compile_schema_residual_records(&window.active_facts, promotion_threshold, &mut labels);

    if let Some(parent) = config.out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create linux residual packet dir {}", parent.display()))?;
    }
    let mut writer = BufWriter::new(
        fs::File::create(&config.out)
            .with_context(|| format!("create linux residual packet {}", config.out.display()))?,
    );
    write_lrf_header(
        &mut writer,
        LrfHeaderWriteSpec {
            schema_count: schemas.len(),
            residual_count: residuals.len(),
            fallback_count: fallbacks.len(),
            label_count: labels.labels.len(),
            route_count: route_distribution.len(),
            corpus_hash: &window.scan.corpus_hash,
            represented_fact_count: window.active_facts.len(),
            promotion_threshold,
        },
    )?;
    for record in &schemas {
        write_schema_record(&mut writer, record)?;
    }
    for record in &residuals {
        write_residual_record(&mut writer, record)?;
    }
    for record in &fallbacks {
        write_fallback_record(&mut writer, record)?;
    }
    let cold_label_table_bytes = write_label_table(&mut writer, &labels.labels)?;
    writer.flush()?;

    let actual_file_bytes = fs::metadata(&config.out)
        .with_context(|| format!("stat linux residual packet {}", config.out.display()))?
        .len() as usize;
    let packet = packet_summary(
        schemas.len(),
        residuals.len(),
        fallbacks.len(),
        labels.labels.len(),
        cold_label_table_bytes,
        actual_file_bytes,
    );
    let binary_hot_sections_fit_6m = packet.binary_hot_sections_fit_6m;
    let economics = economics_summary(
        promotion_threshold,
        schemas.len(),
        residuals.len(),
        fallbacks.len(),
        packet.represented_fact_count,
        true,
    );
    let verdict = if packet.represented_fact_count == 0 {
        "LINUX_SCHEMA_RESIDUAL_PACKET_EMPTY"
    } else if packet.binary_hot_sections_fit_6m && economics.residual_saving_bytes > 0 {
        "LINUX_SCHEMA_RESIDUAL_PACKET_READY_NOT_PROOF"
    } else {
        "LINUX_SCHEMA_RESIDUAL_PACKET_REVIEW"
    };

    Ok(LinuxResidualPackReport {
        mode: "llmwave-big-linux-pack-residual",
        version: LINUX_RESIDUAL_MEMORY_VERSION,
        atlas_version: LINUX_ATLAS_VERSION,
        verdict,
        atlas_dir: config.atlas_dir.display().to_string(),
        out: config.out.display().to_string(),
        source: LinuxResidualSource {
            fact_pack_count: window.fact_paths.len(),
            atlas_fact_count: window.scan.fact_count,
            selected_facts: window.active_facts.len(),
            route_count: route_distribution.len(),
            corpus_hash: window.scan.corpus_hash,
            route_distribution,
        },
        packet,
        economics,
        claim_boundary: LinuxResidualPackClaimBoundary {
            linux_atlas_loaded: window.scan.fact_count > 0,
            binary_schema_residual_memory_written: true,
            schema_records_written: !schemas.is_empty(),
            residual_records_written: !residuals.is_empty(),
            fallback_records_preserve_one_off_facts: !fallbacks.is_empty()
                || window.active_facts.is_empty(),
            binary_hot_sections_fit_6m,
            proof_scan_run: false,
            nonlinear_memory_proven: false,
            exposure_layer_ready: false,
            broad_chat_llm_ready: false,
            safe_claim: "Linux schema/residual memory has been written as real binary sections. It is not nonlinear-memory proof until the binary proof scan and eval pass.",
            blocked_by: vec![
                "linux_schema_residual_proof_scan_missing",
                "linux_schema_residual_eval_missing",
                "exposure_layer_missing",
                "broad_chat_eval_missing",
            ],
        },
    })
}

pub(crate) fn build_linux_residual_proof_report(
    config: LinuxResidualProofConfig,
) -> Result<LinuxResidualProofReport> {
    let packet = parse_lrf_packet(&fs::read(&config.residual_pack).with_context(|| {
        format!(
            "read linux residual packet {}",
            config.residual_pack.display()
        )
    })?)?;
    let sections = parse_lrf_sections_only(&config.residual_pack)?;
    let query = compile_linux_residual_numeric_query(&config.query);
    let iterations = config.iterations.max(1);
    let samples = config.samples.max(1);
    let benchmark = benchmark_linux_residual_sections(
        &sections,
        &query,
        iterations,
        config.warmup_iterations,
        samples,
    );
    let top_k = config.top_k.max(1);
    let eval = run_linux_residual_eval(&packet, top_k);
    let packet_summary =
        proof_packet_summary(&sections, config.residual_pack.display().to_string());
    let binary_hot_sections_fit_6m = packet_summary.binary_hot_sections_fit_6m;
    let economics = economics_summary(
        sections.header.promotion_threshold,
        sections.schemas.len(),
        sections.residuals.len(),
        sections.fallbacks.len(),
        sections.header.represented_fact_count,
        true,
    );
    let schema_reuse_ratio_threshold_met = economics.schema_reuse_ratio >= 0.20;
    let residual_saving_positive = economics.residual_saving_bytes > 0;
    let beats_direct_fixed64 = packet_summary.beats_direct_fixed64;
    let linux_domain_eval_passed =
        eval.metrics.total > 0 && eval.metrics.passed == eval.metrics.total;
    let false_positive_rate_zero = eval.metrics.false_positive_rate == 0.0;
    let full_represented_fact_scan =
        benchmark.represented_facts_per_scan == sections.header.represented_fact_count;
    let nonlinear_memory_proven = packet_summary.binary_hot_sections_fit_6m
        && !sections.schemas.is_empty()
        && !sections.residuals.is_empty()
        && residual_saving_positive
        && beats_direct_fixed64
        && schema_reuse_ratio_threshold_met
        && full_represented_fact_scan
        && benchmark.top_score > 0
        && linux_domain_eval_passed
        && false_positive_rate_zero;
    let verdict = if nonlinear_memory_proven {
        "LINUX_SCHEMA_RESIDUAL_MEMORY_PROVEN"
    } else if packet_summary.binary_hot_sections_fit_6m && benchmark.top_score > 0 {
        "LINUX_SCHEMA_RESIDUAL_MEMORY_REVIEW"
    } else {
        "LINUX_SCHEMA_RESIDUAL_MEMORY_BLOCKED"
    };

    Ok(LinuxResidualProofReport {
        mode: "llmwave-big-linux-residual-proof",
        version: LINUX_RESIDUAL_MEMORY_VERSION,
        verdict,
        residual_pack: packet_summary,
        compiled_query: LinuxResidualQuerySummary {
            raw_query: query.raw_query.clone(),
            token_count: query.token_hashes.len(),
            token_hash_count: query.token_hashes.len(),
            route_hint_hash_count: query.route_hint_hashes.len(),
            relation_hint_hash_count: query.relation_hint_hashes.len(),
            boundary_intent: query.boundary_intent,
            positive_intent: query.positive_intent,
        },
        runtime_contract: LinuxResidualRuntimeContract {
            records_loaded_from: "lrf-header-plus-schema-residual-fallback-sections",
            schema_residual_binary_sections_scanned: true,
            fallback_fixed_records_scanned: true,
            full_represented_fact_scan,
            json_used_in_hot_loop: false,
            labels_used_in_hot_loop: false,
            file_io_in_hot_loop: false,
            heap_allocation_in_inner_loop: false,
            per_record_score_arrays: false,
            measured_loop_uses_numeric_hashes: true,
            cold_label_table_excluded_from_hot_loop: true,
        },
        benchmark,
        eval,
        economics,
        claim_boundary: LinuxResidualProofClaimBoundary {
            binary_schema_residual_memory_loaded: true,
            actual_binary_schema_residual_memory: true,
            schema_reuse_ratio_threshold_met,
            residual_saving_positive,
            beats_direct_fixed64,
            linux_domain_eval_passed,
            false_positive_rate_zero,
            binary_hot_sections_fit_6m,
            nonlinear_memory_proven,
            linux_profile_nonlinear_memory_proven: nonlinear_memory_proven,
            exposure_layer_ready: false,
            broad_chat_llm_ready: false,
            cache_only_execution_proven: false,
            safe_claim: "Linux-profile nonlinear memory is proven only for this binary schema/residual packet: repeated schemas are stored once, residuals carry per-fact variation, one-off facts remain fallbacks, the binary hot sections beat direct fixed64 bytes, and the Linux eval passes. This is not broad chat readiness or exposure reasoning.",
            blocked_claims: if nonlinear_memory_proven {
                vec!["exposure_layer_ready", "broad_chat_llm_ready"]
            } else {
                vec![
                    "linux_schema_residual_eval_missing_or_failed",
                    "schema_reuse_or_residual_saving_too_weak",
                    "exposure_layer_ready",
                    "broad_chat_llm_ready",
                ]
            },
        },
    })
}

pub(crate) fn load_linux_residual_decoded_packet(
    residual_pack: &PathBuf,
) -> Result<LinuxResidualDecodedPacket> {
    let bytes = fs::read(residual_pack)
        .with_context(|| format!("read linux residual packet {}", residual_pack.display()))?;
    let packet = parse_lrf_packet(&bytes)?;
    let sections = parse_lrf_sections_only(residual_pack)?;
    let binary_hot_sections_bytes = sections.schemas.len() * SCHEMA_RECORD_BYTES
        + sections.residuals.len() * RESIDUAL_RECORD_BYTES
        + sections.fallbacks.len() * FALLBACK_RECORD_BYTES;
    let direct_fixed_baseline_bytes =
        sections.header.represented_fact_count * DIRECT_FIXED_RECORD_BYTES;
    let mut facts = Vec::with_capacity(sections.header.represented_fact_count);
    for residual in &packet.residuals {
        let Some(schema) = packet.schemas.get(residual.schema_index as usize) else {
            continue;
        };
        let Some(route) = label(&packet.labels, u32::from(schema.route_id)) else {
            continue;
        };
        let Some(relation) = label(&packet.labels, u32::from(schema.relation_id)) else {
            continue;
        };
        let Some(subject) = label(&packet.labels, residual.subject_id) else {
            continue;
        };
        let Some(object) = label(&packet.labels, residual.object_id) else {
            continue;
        };
        facts.push(LinuxResidualDecodedFact {
            route: route.to_string(),
            subject: subject.to_string(),
            relation: relation.to_string(),
            object: object.to_string(),
            polarity: polarity_label(schema.polarity_id),
            confidence: residual.confidence,
            memory_kind: "residual",
        });
    }
    for fallback in &packet.fallbacks {
        let Some(route) = label(&packet.labels, fallback.route_id) else {
            continue;
        };
        let Some(subject) = label(&packet.labels, fallback.subject_id) else {
            continue;
        };
        let Some(relation) = label(&packet.labels, fallback.relation_id) else {
            continue;
        };
        let Some(object) = label(&packet.labels, fallback.object_id) else {
            continue;
        };
        facts.push(LinuxResidualDecodedFact {
            route: route.to_string(),
            subject: subject.to_string(),
            relation: relation.to_string(),
            object: object.to_string(),
            polarity: polarity_label(fallback.polarity_id),
            confidence: fallback.confidence,
            memory_kind: "fallback",
        });
    }
    Ok(LinuxResidualDecodedPacket {
        summary: LinuxResidualDecodedSummary {
            path: residual_pack.display().to_string(),
            file_bytes: sections.actual_file_bytes,
            wave_dim: sections.header.wave_dim,
            represented_fact_count: sections.header.represented_fact_count,
            schema_record_count: sections.schemas.len(),
            residual_record_count: sections.residuals.len(),
            fallback_record_count: sections.fallbacks.len(),
            route_count: sections.header.route_count,
            corpus_hash64: sections.header.corpus_hash,
            promotion_threshold: sections.header.promotion_threshold,
            binary_hot_sections_bytes,
            direct_fixed_baseline_bytes,
            cold_label_count: sections.header.label_count,
            cold_label_table_bytes: sections.cold_label_table_bytes,
            binary_hot_sections_fit_6m: binary_hot_sections_bytes <= SIX_MIB_BYTES,
            beats_direct_fixed64: binary_hot_sections_bytes < direct_fixed_baseline_bytes,
        },
        facts,
    })
}

fn compile_schema_residual_records(
    facts: &[LinuxAtlasFact],
    promotion_threshold: usize,
    labels: &mut LabelTableBuilder,
) -> (
    Vec<LinuxSchemaRecord32>,
    Vec<LinuxResidualRecord32>,
    Vec<LinuxFallbackRecord64>,
) {
    let mut stats = BTreeMap::<SchemaKey, (usize, usize)>::new();
    for fact in facts {
        let key = SchemaKey {
            route: fact.route.clone(),
            relation: fact.relation.clone(),
            polarity: fact.polarity.clone(),
        };
        let entry = stats.entry(key).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += usize::from(fact.confidence);
    }

    let mut schemas = Vec::new();
    let mut schema_indices = BTreeMap::<SchemaKey, u32>::new();
    for (key, (count, confidence_sum)) in &stats {
        if *count < promotion_threshold {
            continue;
        }
        let index = schemas.len().min(u32::MAX as usize) as u32;
        schema_indices.insert(key.clone(), index);
        let mean_confidence = (confidence_sum / count).min(u8::MAX as usize) as u8;
        schemas.push(LinuxSchemaRecord32 {
            schema_hash: schema_hash(key),
            route_hash: hash64(&key.route),
            relation_hash: hash64(&key.relation),
            route_id: labels.intern(&key.route).min(u16::MAX as u32) as u16,
            relation_id: labels.intern(&key.relation).min(u16::MAX as u32) as u16,
            polarity_id: polarity_id(&key.polarity),
            mean_confidence,
            fact_count_bucket: (*count).min(u16::MAX as usize) as u16,
        });
    }

    let mut residuals = Vec::new();
    let mut fallbacks = Vec::new();
    for fact in facts {
        let key = SchemaKey {
            route: fact.route.clone(),
            relation: fact.relation.clone(),
            polarity: fact.polarity.clone(),
        };
        if let Some(schema_index) = schema_indices.get(&key).copied() {
            residuals.push(LinuxResidualRecord32 {
                schema_index,
                subject_hash: hash64(&fact.subject),
                object_hash: hash64(&fact.object),
                subject_id: labels.intern(&fact.subject),
                object_id: labels.intern(&fact.object),
                confidence: fact.confidence,
                evidence_hash16: hash16(&format!(
                    "{}:{}:{}",
                    fact.evidence.source_kind, fact.evidence.path, fact.evidence.line
                )),
                flags: 0,
            });
        } else {
            fallbacks.push(LinuxFallbackRecord64 {
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
            });
        }
    }

    (schemas, residuals, fallbacks)
}

fn packet_summary(
    schema_count: usize,
    residual_count: usize,
    fallback_count: usize,
    label_count: usize,
    cold_label_table_bytes: usize,
    actual_file_bytes: usize,
) -> LinuxResidualPacketSummary {
    let represented_fact_count = residual_count + fallback_count;
    let binary_hot_sections_bytes = schema_count * SCHEMA_RECORD_BYTES
        + residual_count * RESIDUAL_RECORD_BYTES
        + fallback_count * FALLBACK_RECORD_BYTES;
    let direct_fixed_baseline_bytes = represented_fact_count * DIRECT_FIXED_RECORD_BYTES;
    LinuxResidualPacketSummary {
        format: "lrf-v1-schema32-residual32-fallback64-plus-cold-label-table",
        header_bytes: LRF_HEADER_BYTES,
        schema_record_bytes: SCHEMA_RECORD_BYTES,
        residual_record_bytes: RESIDUAL_RECORD_BYTES,
        fallback_record_bytes: FALLBACK_RECORD_BYTES,
        schema_record_count: schema_count,
        residual_record_count: residual_count,
        fallback_record_count: fallback_count,
        represented_fact_count,
        binary_hot_sections_bytes,
        direct_fixed_record_bytes: DIRECT_FIXED_RECORD_BYTES,
        direct_fixed_baseline_bytes,
        cold_label_count: label_count,
        cold_label_table_bytes,
        actual_file_bytes,
        hot_budget_bytes: SIX_MIB_BYTES,
        binary_hot_sections_fit_6m: binary_hot_sections_bytes <= SIX_MIB_BYTES,
        whole_file_fits_6m: actual_file_bytes <= SIX_MIB_BYTES,
        labels_are_cold_explain_data: true,
        json_used_in_hot_scan: false,
    }
}

fn proof_packet_summary(
    sections: &LinuxResidualSectionsImage,
    path: String,
) -> LinuxResidualProofPackSummary {
    let binary_hot_sections_bytes = sections.schemas.len() * SCHEMA_RECORD_BYTES
        + sections.residuals.len() * RESIDUAL_RECORD_BYTES
        + sections.fallbacks.len() * FALLBACK_RECORD_BYTES;
    let direct_fixed_baseline_bytes =
        sections.header.represented_fact_count * DIRECT_FIXED_RECORD_BYTES;
    let binary_hot_sections_fit_detected_l3 = sections
        .detected_l3_bytes
        .map(|l3_bytes| binary_hot_sections_bytes <= l3_bytes);
    LinuxResidualProofPackSummary {
        path,
        file_bytes: sections.actual_file_bytes,
        wave_dim: sections.header.wave_dim,
        represented_fact_count: sections.header.represented_fact_count,
        schema_record_count: sections.schemas.len(),
        residual_record_count: sections.residuals.len(),
        fallback_record_count: sections.fallbacks.len(),
        route_count: sections.header.route_count,
        corpus_hash64: sections.header.corpus_hash,
        promotion_threshold: sections.header.promotion_threshold,
        schema_record_bytes: sections.header.schema_record_bytes,
        residual_record_bytes: sections.header.residual_record_bytes,
        fallback_record_bytes: sections.header.fallback_record_bytes,
        binary_hot_sections_bytes,
        direct_fixed_baseline_bytes,
        cold_label_count: sections.header.label_count,
        cold_label_table_bytes: sections.cold_label_table_bytes,
        hot_budget_bytes: SIX_MIB_BYTES,
        detected_l3_bytes: sections.detected_l3_bytes,
        binary_hot_sections_fit_6m: binary_hot_sections_bytes <= SIX_MIB_BYTES,
        binary_hot_sections_fit_detected_l3,
        beats_direct_fixed64: binary_hot_sections_bytes < direct_fixed_baseline_bytes,
    }
}

fn economics_summary(
    promotion_threshold: usize,
    schema_count: usize,
    residual_count: usize,
    fallback_count: usize,
    represented_fact_count: usize,
    written: bool,
) -> LinuxResidualEconomics {
    let direct_fixed_baseline_bytes = represented_fact_count * DIRECT_FIXED_RECORD_BYTES;
    let binary_hot_sections_bytes = schema_count * SCHEMA_RECORD_BYTES
        + residual_count * RESIDUAL_RECORD_BYTES
        + fallback_count * FALLBACK_RECORD_BYTES;
    let residual_saving_bytes =
        direct_fixed_baseline_bytes as isize - binary_hot_sections_bytes as isize;
    LinuxResidualEconomics {
        schema_key: "route+relation+polarity",
        promotion_threshold,
        promoted_schema_count: schema_count,
        residual_fact_count: residual_count,
        full_fallback_fact_count: fallback_count,
        residual_saving_bytes,
        residual_saving_ratio: ratio_signed(residual_saving_bytes, direct_fixed_baseline_bytes),
        schema_reuse_ratio: ratio(residual_count, represented_fact_count),
        bytes_per_fact_direct_fixed64: if represented_fact_count == 0 {
            0.0
        } else {
            round4(direct_fixed_baseline_bytes as f32 / represented_fact_count as f32)
        },
        bytes_per_fact_schema_residual_hot: if represented_fact_count == 0 {
            0.0
        } else {
            round4(binary_hot_sections_bytes as f32 / represented_fact_count as f32)
        },
        binary_schema_residual_written: written,
        notes: vec![
            "promoted schemas are stored once as SchemaRecord32",
            "residual facts carry subject/object/evidence variation as ResidualRecord32",
            "one-off facts stay as FallbackRecord64 to avoid false schema compression",
        ],
    }
}

fn benchmark_linux_residual_sections(
    sections: &LinuxResidualSectionsImage,
    query: &LinuxResidualNumericQuery,
    iterations: usize,
    warmup_iterations: usize,
    samples: usize,
) -> LinuxResidualBenchmarkSummary {
    let mut last_scan = LinuxResidualNumericScan {
        top_score: 0,
        second_score: 0,
        top_kind: None,
        top_index: None,
        checksum: 0,
    };
    for _ in 0..warmup_iterations {
        last_scan = black_box(scan_linux_residual_sections_numeric(sections, query));
    }

    let mut sample_ns_per_scan = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        for _ in 0..iterations {
            last_scan = black_box(scan_linux_residual_sections_numeric(sections, query));
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
    let represented_facts_per_scan = sections.header.represented_fact_count;
    let ns_per_fact_p50 = if represented_facts_per_scan == 0 {
        0.0
    } else {
        p50 / represented_facts_per_scan as f64
    };
    let scans_per_second_p50 = if p50 <= 0.0 {
        0.0
    } else {
        1_000_000_000.0 / p50
    };
    let measured_scans = samples.saturating_mul(iterations);
    LinuxResidualBenchmarkSummary {
        warmup_iterations,
        measured_samples: samples,
        iterations_per_sample: iterations,
        measured_scans,
        represented_facts_per_scan,
        measured_fact_visits: measured_scans as u128 * represented_facts_per_scan as u128,
        ns_per_scan_min: round2(min),
        ns_per_scan_p50: round2(p50),
        ns_per_scan_p95: round2(p95),
        ns_per_scan_max: round2(max),
        ns_per_fact_p50: round4_f64(ns_per_fact_p50),
        scans_per_second_p50: round2(scans_per_second_p50),
        represented_facts_per_second_p50: round2(
            scans_per_second_p50 * represented_facts_per_scan as f64,
        ),
        top_score: last_scan.top_score,
        margin: last_scan.top_score - last_scan.second_score,
        top_kind: last_scan.top_kind,
        top_index: last_scan.top_index,
        checksum: last_scan.checksum,
        sample_ns_per_scan: sample_ns_per_scan.into_iter().map(round2).collect(),
    }
}

fn scan_linux_residual_sections_numeric(
    sections: &LinuxResidualSectionsImage,
    query: &LinuxResidualNumericQuery,
) -> LinuxResidualNumericScan {
    let mut top_score = i64::MIN;
    let mut second_score = i64::MIN;
    let mut top_kind = None;
    let mut top_index = None;
    let mut checksum = 0u64;

    for (index, residual) in sections.residuals.iter().enumerate() {
        let Some(schema) = sections.schemas.get(residual.schema_index as usize) else {
            continue;
        };
        let score = score_residual_numeric(schema, residual, query);
        checksum = checksum.rotate_left(7)
            ^ schema.schema_hash
            ^ residual.subject_hash
            ^ residual.object_hash
            ^ ((score as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        if score > top_score {
            second_score = top_score;
            top_score = score;
            top_kind = Some("residual");
            top_index = Some(index);
        } else if score > second_score {
            second_score = score;
        }
    }

    for (index, fallback) in sections.fallbacks.iter().enumerate() {
        let score = score_fallback_numeric(fallback, query);
        checksum = checksum.rotate_left(7)
            ^ fallback.fact_hash
            ^ fallback.route_hash
            ^ ((score as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
        if score > top_score {
            second_score = top_score;
            top_score = score;
            top_kind = Some("fallback");
            top_index = Some(index);
        } else if score > second_score {
            second_score = score;
        }
    }

    LinuxResidualNumericScan {
        top_score: top_score.max(0),
        second_score: second_score.max(0),
        top_kind,
        top_index,
        checksum,
    }
}

fn score_residual_numeric(
    schema: &LinuxSchemaRecord32,
    residual: &LinuxResidualRecord32,
    query: &LinuxResidualNumericQuery,
) -> i64 {
    let mut score = i64::from(residual.confidence) / 10;
    score += schema_hint_score(
        schema.route_hash,
        schema.relation_hash,
        schema.polarity_id,
        query,
    );
    if contains_hash(&query.token_hashes, residual.object_hash) {
        score += 95;
    }
    if contains_hash(&query.token_hashes, residual.subject_hash) {
        score += 75;
    }
    score
}

fn score_fallback_numeric(
    fallback: &LinuxFallbackRecord64,
    query: &LinuxResidualNumericQuery,
) -> i64 {
    let mut score = i64::from(fallback.confidence) / 10;
    score += schema_hint_score(
        fallback.route_hash,
        fallback.relation_hash,
        fallback.polarity_id,
        query,
    );
    if contains_hash(&query.token_hashes, fallback.object_hash) {
        score += 95;
    }
    if contains_hash(&query.token_hashes, fallback.subject_hash) {
        score += 75;
    }
    if contains_hash(&query.token_hashes, fallback.relation_hash) {
        score += 15;
    }
    score
}

fn schema_hint_score(
    route_hash: u64,
    relation_hash: u64,
    polarity_id_value: u8,
    query: &LinuxResidualNumericQuery,
) -> i64 {
    let mut score = 0;
    if query.boundary_intent && polarity_id_value == polarity_id("negative") {
        score += 55;
    }
    if query.positive_intent && polarity_id_value == polarity_id("positive") {
        score += 12;
    }
    if contains_hash(&query.route_hint_hashes, route_hash) {
        score += 35;
    }
    if contains_hash(&query.relation_hint_hashes, relation_hash) {
        score += 45;
    }
    score
}

fn run_linux_residual_eval(
    packet: &LinuxResidualPacketImage,
    top_k: usize,
) -> LinuxResidualEvalSummary {
    let cases = linux_residual_eval_cases()
        .into_iter()
        .map(|case| eval_linux_residual_case(packet, case, top_k))
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
    LinuxResidualEvalSummary {
        cases,
        metrics: LinuxResidualEvalMetrics {
            total,
            passed,
            pass_rate: ratio(passed, total),
            lexical_duel_total,
            lexical_duel_wins,
            lexical_duel_win_rate: ratio(lexical_duel_wins, lexical_duel_total),
            boundary_cases,
            boundary_cases_passed,
            false_positive_rate: ratio(false_positive_count, total),
        },
    }
}

fn scan_linux_residual_packet(
    packet: &LinuxResidualPacketImage,
    query: &str,
    top_k: usize,
) -> LinuxResidualAskField {
    let mut scored = Vec::new();
    for residual in &packet.residuals {
        let Some(schema) = packet.schemas.get(residual.schema_index as usize) else {
            continue;
        };
        let Some(route) = label(&packet.labels, u32::from(schema.route_id)) else {
            continue;
        };
        let Some(relation) = label(&packet.labels, u32::from(schema.relation_id)) else {
            continue;
        };
        let Some(subject) = label(&packet.labels, residual.subject_id) else {
            continue;
        };
        let Some(object) = label(&packet.labels, residual.object_id) else {
            continue;
        };
        let polarity = polarity_label(schema.polarity_id);
        let score = score_linux_fact_terms(LinuxFactScoreTerms {
            query,
            route,
            subject,
            relation,
            object,
            polarity,
            layer: "",
            confidence: residual.confidence,
        });
        if score > 0 {
            scored.push(LinuxResidualAskFact {
                score,
                route: route.to_string(),
                object: object.to_string(),
                polarity,
                memory_kind: "residual",
            });
        }
    }
    for fallback in &packet.fallbacks {
        let Some(route) = label(&packet.labels, fallback.route_id) else {
            continue;
        };
        let Some(subject) = label(&packet.labels, fallback.subject_id) else {
            continue;
        };
        let Some(relation) = label(&packet.labels, fallback.relation_id) else {
            continue;
        };
        let Some(object) = label(&packet.labels, fallback.object_id) else {
            continue;
        };
        let polarity = polarity_label(fallback.polarity_id);
        let score = score_linux_fact_terms(LinuxFactScoreTerms {
            query,
            route,
            subject,
            relation,
            object,
            polarity,
            layer: "",
            confidence: fallback.confidence,
        });
        if score > 0 {
            scored.push(LinuxResidualAskFact {
                score,
                route: route.to_string(),
                object: object.to_string(),
                polarity,
                memory_kind: "fallback",
            });
        }
    }
    scored.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.route.cmp(&right.route))
            .then_with(|| left.object.cmp(&right.object))
    });
    let top_score = scored.first().map(|fact| fact.score).unwrap_or(0);
    let top_facts = scored.into_iter().take(top_k).collect::<Vec<_>>();
    let state = linux_residual_state(top_score, top_facts.first().map(|fact| fact.polarity));
    LinuxResidualAskField {
        state,
        top_score,
        top_facts,
    }
}

fn eval_linux_residual_case(
    packet: &LinuxResidualPacketImage,
    case: BuiltinLinuxResidualEvalCase,
    top_k: usize,
) -> LinuxResidualEvalCaseReport {
    let field = scan_linux_residual_packet(packet, case.query, top_k);
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
    LinuxResidualEvalCaseReport {
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
        memory_kind: field_top.map(|fact| fact.memory_kind),
        lexical_top_route: lexical.as_ref().map(|fact| fact.route.clone()),
        lexical_top_object: lexical.as_ref().map(|fact| fact.object.clone()),
        lexical_top_score: lexical.as_ref().map(|fact| fact.score).unwrap_or(0),
        field_beats_lexical_baseline: passed && !lexical_passed,
        passed,
    }
}

fn lexical_baseline_top(
    packet: &LinuxResidualPacketImage,
    query: &str,
) -> Option<LinuxResidualAskFact> {
    let query_tokens = query
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() > 2)
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for residual in &packet.residuals {
        let Some(schema) = packet.schemas.get(residual.schema_index as usize) else {
            continue;
        };
        let Some(route) = label(&packet.labels, u32::from(schema.route_id)) else {
            continue;
        };
        let Some(relation) = label(&packet.labels, u32::from(schema.relation_id)) else {
            continue;
        };
        let Some(subject) = label(&packet.labels, residual.subject_id) else {
            continue;
        };
        let Some(object) = label(&packet.labels, residual.object_id) else {
            continue;
        };
        let haystack = format!("{route} {subject} {relation} {object}").to_ascii_lowercase();
        let mut score = i64::from(residual.confidence) / 10;
        for token in &query_tokens {
            if haystack.contains(token) {
                score += 10;
            }
        }
        if score > 0 {
            candidates.push(LinuxResidualAskFact {
                score,
                route: route.to_string(),
                object: object.to_string(),
                polarity: polarity_label(schema.polarity_id),
                memory_kind: "residual",
            });
        }
    }
    for fallback in &packet.fallbacks {
        let Some(route) = label(&packet.labels, fallback.route_id) else {
            continue;
        };
        let Some(subject) = label(&packet.labels, fallback.subject_id) else {
            continue;
        };
        let Some(relation) = label(&packet.labels, fallback.relation_id) else {
            continue;
        };
        let Some(object) = label(&packet.labels, fallback.object_id) else {
            continue;
        };
        let haystack = format!("{route} {subject} {relation} {object}").to_ascii_lowercase();
        let mut score = i64::from(fallback.confidence) / 10;
        for token in &query_tokens {
            if haystack.contains(token) {
                score += 10;
            }
        }
        if score > 0 {
            candidates.push(LinuxResidualAskFact {
                score,
                route: route.to_string(),
                object: object.to_string(),
                polarity: polarity_label(fallback.polarity_id),
                memory_kind: "fallback",
            });
        }
    }
    candidates.into_iter().max_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| right.route.cmp(&left.route))
    })
}

fn linux_residual_eval_cases() -> Vec<BuiltinLinuxResidualEvalCase> {
    vec![
        BuiltinLinuxResidualEvalCase {
            id: "command-provider-bash",
            query: "which package provides command bash",
            expected_state: "FOCUSED",
            expected_route: "linux.apt.command",
            expected_polarity: "positive",
            expected_object_match: "command_anchor",
            expected_object_contains: "bash",
        },
        BuiltinLinuxResidualEvalCase {
            id: "command-provider-systemctl",
            query: "which package provides command systemctl",
            expected_state: "FOCUSED",
            expected_route: "linux.package.binary",
            expected_polarity: "positive",
            expected_object_match: "command_anchor",
            expected_object_contains: "systemctl",
        },
        BuiltinLinuxResidualEvalCase {
            id: "negative-package-installed-not-running",
            query: "package installed does not prove binary is running",
            expected_state: "BOUNDARY_FOCUSED",
            expected_route: "linux.boundary.package",
            expected_polarity: "negative",
            expected_object_match: "contains",
            expected_object_contains: "binary is running",
        },
        BuiltinLinuxResidualEvalCase {
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

fn object_matches_case(case: BuiltinLinuxResidualEvalCase, object: &str) -> bool {
    match case.expected_object_match {
        "command_anchor" => {
            object == case.expected_object_contains
                || object.ends_with(&format!("/{}", case.expected_object_contains))
        }
        _ => object.contains(case.expected_object_contains),
    }
}

fn linux_residual_state(top_score: i64, polarity: Option<&str>) -> &'static str {
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

fn compile_linux_residual_numeric_query(query: &str) -> LinuxResidualNumericQuery {
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

    LinuxResidualNumericQuery {
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

fn write_lrf_header(writer: &mut impl Write, spec: LrfHeaderWriteSpec<'_>) -> Result<()> {
    writer.write_all(LRF_MAGIC)?;
    write_u32(writer, LRF_FORMAT_VERSION)?;
    write_u32(writer, WAVE_DIM as u32)?;
    write_u32(writer, spec.schema_count as u32)?;
    write_u32(writer, spec.residual_count as u32)?;
    write_u32(writer, spec.fallback_count as u32)?;
    write_u32(writer, spec.label_count as u32)?;
    write_u32(writer, spec.route_count as u32)?;
    write_u32(writer, SCHEMA_RECORD_BYTES as u32)?;
    write_u32(writer, RESIDUAL_RECORD_BYTES as u32)?;
    write_u32(writer, FALLBACK_RECORD_BYTES as u32)?;
    write_u64(writer, hash64(spec.corpus_hash))?;
    write_u32(writer, spec.represented_fact_count as u32)?;
    write_u32(writer, spec.promotion_threshold as u32)?;
    Ok(())
}

fn write_schema_record(writer: &mut impl Write, record: &LinuxSchemaRecord32) -> Result<()> {
    write_u64(writer, record.schema_hash)?;
    write_u64(writer, record.route_hash)?;
    write_u64(writer, record.relation_hash)?;
    write_u16(writer, record.route_id)?;
    write_u16(writer, record.relation_id)?;
    write_u8(writer, record.polarity_id)?;
    write_u8(writer, record.mean_confidence)?;
    write_u16(writer, record.fact_count_bucket)?;
    Ok(())
}

fn write_residual_record(writer: &mut impl Write, record: &LinuxResidualRecord32) -> Result<()> {
    write_u32(writer, record.schema_index)?;
    write_u64(writer, record.subject_hash)?;
    write_u64(writer, record.object_hash)?;
    write_u32(writer, record.subject_id)?;
    write_u32(writer, record.object_id)?;
    write_u8(writer, record.confidence)?;
    write_u16(writer, record.evidence_hash16)?;
    write_u8(writer, record.flags)?;
    Ok(())
}

fn write_fallback_record(writer: &mut impl Write, record: &LinuxFallbackRecord64) -> Result<()> {
    write_u64(writer, record.fact_hash)?;
    write_u64(writer, record.route_hash)?;
    write_u64(writer, record.subject_hash)?;
    write_u64(writer, record.relation_hash)?;
    write_u64(writer, record.object_hash)?;
    write_u32(writer, record.route_id)?;
    write_u32(writer, record.subject_id)?;
    write_u32(writer, record.relation_id)?;
    write_u32(writer, record.object_id)?;
    write_u8(writer, record.confidence)?;
    write_u8(writer, record.polarity_id)?;
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

fn parse_lrf_sections_only(path: &PathBuf) -> Result<LinuxResidualSectionsImage> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("open linux residual packet {}", path.display()))?;
    let actual_file_bytes = file
        .metadata()
        .with_context(|| format!("stat linux residual packet {}", path.display()))?
        .len() as usize;
    let mut header_bytes = [0u8; LRF_HEADER_BYTES];
    file.read_exact(&mut header_bytes)
        .with_context(|| format!("read linux residual header {}", path.display()))?;
    let header = parse_lrf_header(&header_bytes)?;
    let sections_bytes = lrf_binary_sections_bytes(header)?;
    if LRF_HEADER_BYTES + sections_bytes > actual_file_bytes {
        bail!("truncated linux residual binary sections");
    }
    let mut schemas = Vec::with_capacity(header.schema_count);
    let mut schema_bytes = [0u8; SCHEMA_RECORD_BYTES];
    for _ in 0..header.schema_count {
        file.read_exact(&mut schema_bytes)
            .with_context(|| format!("read linux schema record {}", path.display()))?;
        schemas.push(read_schema_record(&mut Cursor::new(&schema_bytes[..]))?);
    }
    let mut residuals = Vec::with_capacity(header.residual_count);
    let mut residual_bytes = [0u8; RESIDUAL_RECORD_BYTES];
    for _ in 0..header.residual_count {
        file.read_exact(&mut residual_bytes)
            .with_context(|| format!("read linux residual record {}", path.display()))?;
        residuals.push(read_residual_record(&mut Cursor::new(&residual_bytes[..]))?);
    }
    let mut fallbacks = Vec::with_capacity(header.fallback_count);
    let mut fallback_bytes = [0u8; FALLBACK_RECORD_BYTES];
    for _ in 0..header.fallback_count {
        file.read_exact(&mut fallback_bytes)
            .with_context(|| format!("read linux fallback record {}", path.display()))?;
        fallbacks.push(read_fallback_record(&mut Cursor::new(&fallback_bytes[..]))?);
    }
    Ok(LinuxResidualSectionsImage {
        header,
        schemas,
        residuals,
        fallbacks,
        actual_file_bytes,
        cold_label_table_bytes: actual_file_bytes.saturating_sub(LRF_HEADER_BYTES + sections_bytes),
        detected_l3_bytes: detect_l3_cache_bytes(),
    })
}

fn parse_lrf_packet(bytes: &[u8]) -> Result<LinuxResidualPacketImage> {
    if bytes.len() < LRF_HEADER_BYTES {
        bail!("truncated linux residual packet header");
    }
    let header = parse_lrf_header(&bytes[..LRF_HEADER_BYTES])?;
    let sections_bytes = lrf_binary_sections_bytes(header)?;
    if LRF_HEADER_BYTES + sections_bytes > bytes.len() {
        bail!("truncated linux residual packet records");
    }
    let mut cursor = Cursor::new(bytes);
    cursor.set_position(LRF_HEADER_BYTES as u64);
    let mut schemas = Vec::with_capacity(header.schema_count);
    for _ in 0..header.schema_count {
        schemas.push(read_schema_record(&mut cursor)?);
    }
    let mut residuals = Vec::with_capacity(header.residual_count);
    for _ in 0..header.residual_count {
        residuals.push(read_residual_record(&mut cursor)?);
    }
    let mut fallbacks = Vec::with_capacity(header.fallback_count);
    for _ in 0..header.fallback_count {
        fallbacks.push(read_fallback_record(&mut cursor)?);
    }
    let mut labels = vec![String::new(); header.label_count];
    for _ in 0..header.label_count {
        let id = read_u32(&mut cursor)? as usize;
        let len = read_u32(&mut cursor)? as usize;
        if id >= header.label_count {
            bail!("linux residual packet label id out of range");
        }
        let end = cursor
            .position()
            .checked_add(len as u64)
            .context("linux residual label cursor overflow")?;
        if end > bytes.len() as u64 {
            bail!("truncated linux residual label");
        }
        let start = cursor.position() as usize;
        let end_usize = end as usize;
        labels[id] = std::str::from_utf8(&bytes[start..end_usize])
            .context("linux residual label is not utf-8")?
            .to_string();
        cursor.set_position(end);
    }
    Ok(LinuxResidualPacketImage {
        schemas,
        residuals,
        fallbacks,
        labels,
    })
}

fn parse_lrf_header(bytes: &[u8]) -> Result<LrfHeaderFields> {
    let mut cursor = Cursor::new(bytes);
    let mut magic = [0u8; 8];
    cursor.read_exact(&mut magic)?;
    if &magic != LRF_MAGIC {
        bail!("invalid linux residual packet magic");
    }
    let version = read_u32(&mut cursor)?;
    if version != LRF_FORMAT_VERSION {
        bail!("unsupported linux residual packet version {version}");
    }
    let wave_dim = read_u32(&mut cursor)?;
    let schema_count = read_u32(&mut cursor)? as usize;
    let residual_count = read_u32(&mut cursor)? as usize;
    let fallback_count = read_u32(&mut cursor)? as usize;
    let label_count = read_u32(&mut cursor)? as usize;
    let route_count = read_u32(&mut cursor)? as usize;
    let schema_record_bytes = read_u32(&mut cursor)? as usize;
    let residual_record_bytes = read_u32(&mut cursor)? as usize;
    let fallback_record_bytes = read_u32(&mut cursor)? as usize;
    if schema_record_bytes != SCHEMA_RECORD_BYTES {
        bail!("unsupported linux schema record size {schema_record_bytes}");
    }
    if residual_record_bytes != RESIDUAL_RECORD_BYTES {
        bail!("unsupported linux residual record size {residual_record_bytes}");
    }
    if fallback_record_bytes != FALLBACK_RECORD_BYTES {
        bail!("unsupported linux fallback record size {fallback_record_bytes}");
    }
    let corpus_hash = read_u64(&mut cursor)?;
    let represented_fact_count = read_u32(&mut cursor)? as usize;
    let promotion_threshold = read_u32(&mut cursor)? as usize;
    Ok(LrfHeaderFields {
        wave_dim,
        schema_count,
        residual_count,
        fallback_count,
        label_count,
        route_count,
        schema_record_bytes,
        residual_record_bytes,
        fallback_record_bytes,
        corpus_hash,
        represented_fact_count,
        promotion_threshold,
    })
}

fn lrf_binary_sections_bytes(header: LrfHeaderFields) -> Result<usize> {
    let schema_bytes = header
        .schema_count
        .checked_mul(SCHEMA_RECORD_BYTES)
        .context("linux residual schema bytes overflow")?;
    let residual_bytes = header
        .residual_count
        .checked_mul(RESIDUAL_RECORD_BYTES)
        .context("linux residual residual bytes overflow")?;
    let fallback_bytes = header
        .fallback_count
        .checked_mul(FALLBACK_RECORD_BYTES)
        .context("linux residual fallback bytes overflow")?;
    schema_bytes
        .checked_add(residual_bytes)
        .and_then(|value| value.checked_add(fallback_bytes))
        .context("linux residual binary section bytes overflow")
}

fn read_schema_record(cursor: &mut Cursor<&[u8]>) -> Result<LinuxSchemaRecord32> {
    Ok(LinuxSchemaRecord32 {
        schema_hash: read_u64(cursor)?,
        route_hash: read_u64(cursor)?,
        relation_hash: read_u64(cursor)?,
        route_id: read_u16(cursor)?,
        relation_id: read_u16(cursor)?,
        polarity_id: read_u8(cursor)?,
        mean_confidence: read_u8(cursor)?,
        fact_count_bucket: read_u16(cursor)?,
    })
}

fn read_residual_record(cursor: &mut Cursor<&[u8]>) -> Result<LinuxResidualRecord32> {
    Ok(LinuxResidualRecord32 {
        schema_index: read_u32(cursor)?,
        subject_hash: read_u64(cursor)?,
        object_hash: read_u64(cursor)?,
        subject_id: read_u32(cursor)?,
        object_id: read_u32(cursor)?,
        confidence: read_u8(cursor)?,
        evidence_hash16: read_u16(cursor)?,
        flags: read_u8(cursor)?,
    })
}

fn read_fallback_record(cursor: &mut Cursor<&[u8]>) -> Result<LinuxFallbackRecord64> {
    Ok(LinuxFallbackRecord64 {
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

fn schema_hash(key: &SchemaKey) -> u64 {
    hash64(&format!("{}|{}|{}", key.route, key.relation, key.polarity))
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

fn hash16(value: &str) -> u16 {
    let digest = Sha256::digest(value.as_bytes());
    u16::from_le_bytes([digest[0], digest[1]])
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        round4(part as f32 / total as f32)
    }
}

fn ratio_signed(part: isize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        round4(part as f32 / total as f32)
    }
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

fn write_u8(writer: &mut impl Write, value: u8) -> Result<()> {
    writer.write_all(&[value])?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::super::linux_atlas::LinuxAtlasEvidence;
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn linux_residual_packet_roundtrip_proves_schema_memory() {
        let root = fixture_root("linux-residual-roundtrip");
        let out = root.join("linux.lrf");
        let pack = build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 7,
            promotion_threshold: 2,
            out: out.clone(),
        })
        .unwrap();
        assert_eq!(pack.verdict, "LINUX_SCHEMA_RESIDUAL_PACKET_READY_NOT_PROOF");
        assert_eq!(pack.packet.schema_record_count, 1);
        assert_eq!(pack.packet.residual_record_count, 2);
        assert_eq!(pack.packet.fallback_record_count, 5);
        assert!(pack.economics.residual_saving_bytes > 0);
        assert!(!pack.claim_boundary.nonlinear_memory_proven);

        let proof = build_linux_residual_proof_report(LinuxResidualProofConfig {
            residual_pack: out,
            query: "which package provides command bash".to_string(),
            top_k: 3,
            iterations: 2,
            warmup_iterations: 1,
            samples: 2,
        })
        .unwrap();
        assert_eq!(proof.verdict, "LINUX_SCHEMA_RESIDUAL_MEMORY_PROVEN");
        assert!(proof.claim_boundary.nonlinear_memory_proven);
        assert!(proof.claim_boundary.linux_profile_nonlinear_memory_proven);
        assert!(!proof.claim_boundary.broad_chat_llm_ready);
        assert!(!proof.claim_boundary.exposure_layer_ready);
        assert!(
            proof
                .runtime_contract
                .schema_residual_binary_sections_scanned
        );
        assert!(!proof.runtime_contract.json_used_in_hot_loop);
        assert_eq!(proof.eval.metrics.passed, proof.eval.metrics.total);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn linux_residual_packet_does_not_prove_without_schema_reuse() {
        let root = unique_tmp_dir("linux-residual-no-reuse");
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
        let out = root.join("linux.lrf");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 2,
            promotion_threshold: 2,
            out: out.clone(),
        })
        .unwrap();
        let proof = build_linux_residual_proof_report(LinuxResidualProofConfig {
            residual_pack: out,
            query: "which package provides command bash".to_string(),
            top_k: 2,
            iterations: 1,
            warmup_iterations: 0,
            samples: 1,
        })
        .unwrap();
        assert_ne!(proof.verdict, "LINUX_SCHEMA_RESIDUAL_MEMORY_PROVEN");
        assert!(!proof.claim_boundary.nonlinear_memory_proven);
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_root(prefix: &str) -> PathBuf {
        let root = unique_tmp_dir(prefix);
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
            test_fact(
                "linux.package.binary",
                "systemd",
                "provides binary",
                "/usr/bin/systemctl",
                "positive",
            ),
            test_fact(
                "linux.apt.command.package-command",
                "which.debianutils",
                "provides command",
                "which.debianutils",
                "positive",
            ),
            test_fact(
                "linux.systemd.exec",
                "ssh.service",
                "execstart",
                "/usr/sbin/sshd",
                "positive",
            ),
            test_fact(
                "linux.boundary.package",
                "package installed",
                "does not prove",
                "binary is running",
                "negative",
            ),
            test_fact(
                "linux.boundary.socket",
                "port listening",
                "does not prove",
                "firewall allows external packets",
                "negative",
            ),
        ];
        let mut file = fs::File::create(&facts_path).unwrap();
        for fact in facts {
            serde_json::to_writer(&mut file, &fact).unwrap();
            file.write_all(b"\n").unwrap();
        }
        root
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
