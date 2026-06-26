//! Phase 18: Linux Atlas -> 6 MiB cognitive projection gate.
//!
//! The Atlas can be very large because it stores append-only facts, labels,
//! provenance, and snapshots. This gate checks whether the useful active
//! projection is represented by the cache-budget `.laf` hot packet and the
//! schema/residual `.lrf` memory packet, without claiming lossless Atlas storage.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::linux_hot_packet::{
    build_linux_cache_proof_report, LinuxCacheProofConfig, LinuxCacheProofReport,
};
use super::linux_residual_memory::{
    build_linux_residual_proof_report, LinuxResidualProofConfig, LinuxResidualProofReport,
};

pub(crate) const LINUX_ATLAS_PROJECTION_VERSION: &str =
    "llmwave-big-v-next-linux-atlas-6mb-projection";

const SIX_MIB_BYTES: usize = 6 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct LinuxAtlasProjectionConfig {
    pub atlas_dir: PathBuf,
    pub hot_pack: PathBuf,
    pub residual_pack: PathBuf,
    pub query: String,
    pub top_k: usize,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub samples: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxAtlasProjectionReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub atlas: LinuxAtlasProjectionAtlasAudit,
    pub projection: LinuxAtlasProjectionSummary,
    pub runtime: LinuxAtlasProjectionRuntimeSummary,
    pub gates: LinuxAtlasProjectionGates,
    pub claim_boundary: LinuxAtlasProjectionClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxAtlasProjectionAtlasAudit {
    pub atlas_dir: String,
    pub total_bytes: usize,
    pub facts_bytes: usize,
    pub indexes_bytes: usize,
    pub manifest_bytes: usize,
    pub fact_file_count: usize,
    pub index_file_count: usize,
    pub manifest: LinuxAtlasProjectionManifestSummary,
    pub read_strategy: &'static str,
}

#[derive(Serialize, Clone, Default)]
pub(crate) struct LinuxAtlasProjectionManifestSummary {
    pub found: bool,
    pub append_only: Option<bool>,
    pub fact_count: Option<usize>,
    pub known_fact_count: Option<usize>,
    pub route_count: Option<usize>,
    pub latest_pack_kind: Option<String>,
    pub latest_facts_path: Option<String>,
    pub current_path: Option<String>,
    pub source_fact_total: Option<usize>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxAtlasProjectionSummary {
    pub goal: &'static str,
    pub active_fact_count: usize,
    pub source_known_fact_count: Option<usize>,
    pub active_to_known_ratio: Option<f32>,
    pub atlas_to_laf_file_ratio: Option<f32>,
    pub atlas_to_lrf_file_ratio: Option<f32>,
    pub atlas_to_lrf_hot_section_ratio: Option<f32>,
    pub lossless_atlas_storage_in_6m: bool,
    pub useful_cognitive_projection_in_6m: bool,
    pub projection_read_as: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxAtlasProjectionRuntimeSummary {
    pub laf_file_bytes: usize,
    pub laf_hot_loop_bytes: usize,
    pub laf_fixed_record_count: usize,
    pub laf_whole_file_fits_6m: bool,
    pub laf_hot_loop_fits_6m: bool,
    pub laf_cache_only_execution_proven: bool,
    pub laf_ns_per_scan_p50: f64,
    pub lrf_file_bytes: usize,
    pub lrf_hot_sections_bytes: usize,
    pub lrf_represented_fact_count: usize,
    pub lrf_schema_record_count: usize,
    pub lrf_residual_record_count: usize,
    pub lrf_fallback_record_count: usize,
    pub lrf_hot_sections_fit_6m: bool,
    pub lrf_beats_fixed64: bool,
    pub lrf_residual_saving_bytes: isize,
    pub lrf_residual_saving_ratio: f32,
    pub lrf_schema_reuse_ratio: f32,
    pub lrf_nonlinear_memory_proven: bool,
    pub lrf_ns_per_scan_p50: f64,
    pub labels_lazy_decode_required: bool,
    pub pmu_cache_miss_rate_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxAtlasProjectionGates {
    pub atlas_audited: bool,
    pub hot_packet_cache_budget_passed: bool,
    pub residual_memory_budget_passed: bool,
    pub residual_memory_beats_fixed64: bool,
    pub cache_only_execution_proven: bool,
    pub linux_profile_nonlinear_memory_proven: bool,
    pub active_fact_counts_match: bool,
    pub false_positive_rate_zero: bool,
    pub hardware_pmu_evidence_recorded_or_blocked: bool,
    pub final_gate_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxAtlasProjectionClaimBoundary {
    pub atlas_audited: bool,
    pub atlas_lossless_storage_in_6m: bool,
    pub useful_atlas_projection_ready: bool,
    pub cache_only_execution_proven: bool,
    pub linux_profile_nonlinear_memory_proven: bool,
    pub hardware_cache_residency_counter_proven: bool,
    pub global_nonlinear_memory_proven: bool,
    pub general_llm_ready: bool,
    pub open_domain_chat_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Deserialize, Default)]
struct LinuxAtlasManifestFile {
    append_only: Option<bool>,
    fact_count: Option<usize>,
    known_fact_count: Option<usize>,
    route_count: Option<usize>,
    latest_pack_kind: Option<String>,
    latest_facts_path: Option<String>,
    current_path: Option<String>,
    source_stats: Option<std::collections::BTreeMap<String, usize>>,
}

pub(crate) fn build_linux_atlas_projection_report(
    config: LinuxAtlasProjectionConfig,
) -> Result<LinuxAtlasProjectionReport> {
    let atlas = audit_atlas_dir(&config.atlas_dir)?;
    let cache_proof = build_linux_cache_proof_report(LinuxCacheProofConfig {
        hot_pack: config.hot_pack,
        query: config.query.clone(),
        iterations: config.iterations,
        warmup_iterations: config.warmup_iterations,
        samples: config.samples,
    })?;
    let residual_proof = build_linux_residual_proof_report(LinuxResidualProofConfig {
        residual_pack: config.residual_pack,
        query: config.query,
        top_k: config.top_k,
        iterations: config.iterations,
        warmup_iterations: config.warmup_iterations,
        samples: config.samples,
    })?;

    let runtime = runtime_summary(&cache_proof, &residual_proof);
    let source_known_fact_count = atlas
        .manifest
        .known_fact_count
        .or(atlas.manifest.fact_count)
        .filter(|count| *count > 0);
    let active_fact_count = runtime
        .lrf_represented_fact_count
        .max(runtime.laf_fixed_record_count);
    let projection = LinuxAtlasProjectionSummary {
        goal: "reuse useful Atlas cognition in a 6 MiB active runtime, not losslessly store the whole Atlas",
        active_fact_count,
        source_known_fact_count,
        active_to_known_ratio: source_known_fact_count
            .map(|known| ratio(active_fact_count, known)),
        atlas_to_laf_file_ratio: ratio_option(atlas.total_bytes, runtime.laf_file_bytes),
        atlas_to_lrf_file_ratio: ratio_option(atlas.total_bytes, runtime.lrf_file_bytes),
        atlas_to_lrf_hot_section_ratio: ratio_option(
            atlas.total_bytes,
            runtime.lrf_hot_sections_bytes,
        ),
        lossless_atlas_storage_in_6m: atlas.total_bytes <= SIX_MIB_BYTES,
        useful_cognitive_projection_in_6m: runtime.laf_whole_file_fits_6m
            && runtime.lrf_hot_sections_fit_6m
            && runtime.lrf_nonlinear_memory_proven,
        projection_read_as: "Atlas bytes are cold provenance/history; the 6 MiB runtime is a schema/residual/current projection.",
    };

    let gates = gates(&atlas, &runtime, &residual_proof);
    let verdict = if gates.final_gate_ready {
        "LINUX_ATLAS_6MB_COGNITIVE_PROJECTION_READY"
    } else if gates.hot_packet_cache_budget_passed && gates.residual_memory_budget_passed {
        "LINUX_ATLAS_6MB_COGNITIVE_PROJECTION_REVIEW"
    } else {
        "LINUX_ATLAS_6MB_COGNITIVE_PROJECTION_BLOCKED"
    };

    Ok(LinuxAtlasProjectionReport {
        mode: "llmwave-big-linux-atlas-6mb-projection",
        version: LINUX_ATLAS_PROJECTION_VERSION,
        verdict,
        atlas,
        projection,
        runtime,
        gates,
        claim_boundary: claim_boundary(verdict),
    })
}

fn runtime_summary(
    cache_proof: &LinuxCacheProofReport,
    residual_proof: &LinuxResidualProofReport,
) -> LinuxAtlasProjectionRuntimeSummary {
    LinuxAtlasProjectionRuntimeSummary {
        laf_file_bytes: cache_proof.hot_pack.file_bytes,
        laf_hot_loop_bytes: cache_proof.hot_pack.hot_loop_record_bytes,
        laf_fixed_record_count: cache_proof.hot_pack.fixed_record_count,
        laf_whole_file_fits_6m: cache_proof.hot_pack.whole_file_fits_6m,
        laf_hot_loop_fits_6m: cache_proof.hot_pack.fixed_records_fit_6m,
        laf_cache_only_execution_proven: cache_proof.claim_boundary.cache_only_execution_proven,
        laf_ns_per_scan_p50: cache_proof.benchmark.ns_per_scan_p50,
        lrf_file_bytes: residual_proof.residual_pack.file_bytes,
        lrf_hot_sections_bytes: residual_proof.residual_pack.binary_hot_sections_bytes,
        lrf_represented_fact_count: residual_proof.residual_pack.represented_fact_count,
        lrf_schema_record_count: residual_proof.residual_pack.schema_record_count,
        lrf_residual_record_count: residual_proof.residual_pack.residual_record_count,
        lrf_fallback_record_count: residual_proof.residual_pack.fallback_record_count,
        lrf_hot_sections_fit_6m: residual_proof.residual_pack.binary_hot_sections_fit_6m,
        lrf_beats_fixed64: residual_proof.residual_pack.beats_direct_fixed64,
        lrf_residual_saving_bytes: residual_proof.economics.residual_saving_bytes,
        lrf_residual_saving_ratio: residual_proof.economics.residual_saving_ratio,
        lrf_schema_reuse_ratio: residual_proof.economics.schema_reuse_ratio,
        lrf_nonlinear_memory_proven: residual_proof.claim_boundary.nonlinear_memory_proven,
        lrf_ns_per_scan_p50: residual_proof.benchmark.ns_per_scan_p50,
        labels_lazy_decode_required: cache_proof.runtime_contract.cold_label_table_excluded
            && residual_proof
                .runtime_contract
                .cold_label_table_excluded_from_hot_loop,
        pmu_cache_miss_rate_proven: cache_proof
            .claim_boundary
            .hardware_cache_residency_counter_proven,
    }
}

fn gates(
    atlas: &LinuxAtlasProjectionAtlasAudit,
    runtime: &LinuxAtlasProjectionRuntimeSummary,
    residual_proof: &LinuxResidualProofReport,
) -> LinuxAtlasProjectionGates {
    let active_fact_counts_match =
        runtime.laf_fixed_record_count == runtime.lrf_represented_fact_count;
    let false_positive_rate_zero = residual_proof.eval.metrics.false_positive_rate == 0.0;
    let hardware_pmu_evidence_recorded_or_blocked = !runtime.pmu_cache_miss_rate_proven;
    let final_gate_ready = atlas.total_bytes > 0
        && runtime.laf_hot_loop_fits_6m
        && runtime.laf_cache_only_execution_proven
        && runtime.lrf_hot_sections_fit_6m
        && runtime.lrf_beats_fixed64
        && runtime.lrf_nonlinear_memory_proven
        && active_fact_counts_match
        && false_positive_rate_zero
        && hardware_pmu_evidence_recorded_or_blocked;

    LinuxAtlasProjectionGates {
        atlas_audited: atlas.total_bytes > 0,
        hot_packet_cache_budget_passed: runtime.laf_hot_loop_fits_6m,
        residual_memory_budget_passed: runtime.lrf_hot_sections_fit_6m,
        residual_memory_beats_fixed64: runtime.lrf_beats_fixed64,
        cache_only_execution_proven: runtime.laf_cache_only_execution_proven,
        linux_profile_nonlinear_memory_proven: runtime.lrf_nonlinear_memory_proven,
        active_fact_counts_match,
        false_positive_rate_zero,
        hardware_pmu_evidence_recorded_or_blocked,
        final_gate_ready,
    }
}

fn claim_boundary(verdict: &'static str) -> LinuxAtlasProjectionClaimBoundary {
    let useful_atlas_projection_ready = verdict == "LINUX_ATLAS_6MB_COGNITIVE_PROJECTION_READY";
    LinuxAtlasProjectionClaimBoundary {
        atlas_audited: true,
        atlas_lossless_storage_in_6m: false,
        useful_atlas_projection_ready,
        cache_only_execution_proven: useful_atlas_projection_ready,
        linux_profile_nonlinear_memory_proven: useful_atlas_projection_ready,
        hardware_cache_residency_counter_proven: false,
        global_nonlinear_memory_proven: false,
        general_llm_ready: false,
        open_domain_chat_ready: false,
        safe_claim: "A large Linux Atlas can be reused as a 6 MiB active cognitive projection when the .laf cache proof and .lrf schema/residual proof both pass. This is not lossless Atlas storage, global nonlinear memory proof, or general LLM readiness.",
        blocked_claims: vec![
            "lossless_full_atlas_storage_in_6m",
            "hardware_pmu_cache_residency_proven",
            "global_nonlinear_memory_proven",
            "general_llm_ready",
            "open_domain_chat_ready",
        ],
    }
}

fn audit_atlas_dir(atlas_dir: &Path) -> Result<LinuxAtlasProjectionAtlasAudit> {
    let total_bytes = recursive_file_bytes(atlas_dir)?;
    let facts_dir = atlas_dir.join("facts");
    let indexes_dir = atlas_dir.join("indexes");
    let facts_bytes = recursive_file_bytes_if_exists(&facts_dir)?;
    let indexes_bytes = recursive_file_bytes_if_exists(&indexes_dir)?;
    let manifest_path = atlas_dir.join("manifest.json");
    let manifest_bytes = fs::metadata(&manifest_path)
        .map(|meta| meta.len() as usize)
        .unwrap_or(0);
    let manifest = read_manifest_summary(&manifest_path)?;

    Ok(LinuxAtlasProjectionAtlasAudit {
        atlas_dir: atlas_dir.display().to_string(),
        total_bytes,
        facts_bytes,
        indexes_bytes,
        manifest_bytes,
        fact_file_count: count_files_if_exists(&facts_dir)?,
        index_file_count: count_files_if_exists(&indexes_dir)?,
        manifest,
        read_strategy: "metadata-and-manifest-only; fact JSONL bodies are cold provenance and are not scanned for this gate",
    })
}

fn read_manifest_summary(path: &Path) -> Result<LinuxAtlasProjectionManifestSummary> {
    if !path.exists() {
        return Ok(LinuxAtlasProjectionManifestSummary::default());
    }
    let bytes =
        fs::read(path).with_context(|| format!("read atlas manifest {}", path.display()))?;
    let parsed: LinuxAtlasManifestFile = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse atlas manifest {}", path.display()))?;
    Ok(LinuxAtlasProjectionManifestSummary {
        found: true,
        append_only: parsed.append_only,
        fact_count: parsed.fact_count,
        known_fact_count: parsed.known_fact_count,
        route_count: parsed.route_count,
        latest_pack_kind: parsed.latest_pack_kind,
        latest_facts_path: parsed.latest_facts_path,
        current_path: parsed.current_path,
        source_fact_total: parsed
            .source_stats
            .map(|stats| stats.values().copied().sum::<usize>()),
    })
}

fn recursive_file_bytes_if_exists(path: &Path) -> Result<usize> {
    if path.exists() {
        recursive_file_bytes(path)
    } else {
        Ok(0)
    }
}

fn recursive_file_bytes(path: &Path) -> Result<usize> {
    let mut total = 0usize;
    if !path.exists() {
        return Ok(0);
    }
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        let metadata = fs::symlink_metadata(&current)
            .with_context(|| format!("stat {}", current.display()))?;
        if metadata.is_file() {
            total = total.saturating_add(metadata.len() as usize);
        } else if metadata.is_dir() {
            for entry in fs::read_dir(&current)
                .with_context(|| format!("read directory {}", current.display()))?
            {
                stack.push(entry?.path());
            }
        }
    }
    Ok(total)
}

fn count_files_if_exists(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let mut count = 0usize;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        let metadata = fs::symlink_metadata(&current)
            .with_context(|| format!("stat {}", current.display()))?;
        if metadata.is_file() {
            count += 1;
        } else if metadata.is_dir() {
            for entry in fs::read_dir(&current)
                .with_context(|| format!("read directory {}", current.display()))?
            {
                stack.push(entry?.path());
            }
        }
    }
    Ok(count)
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        round4(numerator as f32 / denominator as f32)
    }
}

fn ratio_option(numerator: usize, denominator: usize) -> Option<f32> {
    if denominator == 0 {
        None
    } else {
        Some(ratio(numerator, denominator))
    }
}

fn round4(value: f32) -> f32 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_rounds_and_blocks_zero_denominator() {
        assert_eq!(ratio(1, 3), 0.3333);
        assert_eq!(ratio(1, 0), 0.0);
        assert_eq!(ratio_option(10, 0), None);
    }
}
