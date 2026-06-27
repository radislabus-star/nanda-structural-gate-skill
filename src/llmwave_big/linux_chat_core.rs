//! Unified Linux ChatCore memory/cache layer.
//!
//! The cache is a compiled runtime view over source memory (`.lrf` plus `.lwm`
//! overlays). It is never the source of truth: every gate recomputes source
//! hashes and refuses answer authority when the compiled manifest is stale.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::linux_profile::{
    build_linux_profile_claim_gate_report, build_linux_query_wave_report,
    build_linux_reason_report_from_decoded_facts, LinuxEvidenceStep, LinuxProfileClaimGateConfig,
    LinuxProfileClaimGateReport, LinuxQueryWave, LinuxQueryWaveConfig, LinuxReasonReport,
};
use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
};
use super::persistent_wave_memory::{
    append_delta, build_delta_record, load_memory, PersistentWaveDeltaRecord,
    PersistentWaveDeltaSpec, PersistentWaveMemorySummary, DELTA_NEGATIVE, DELTA_POSITIVE,
    DELTA_WATCH_TRACE,
};

pub(crate) const LINUX_CHAT_CORE_VERSION: &str =
    "llmwave-big-v-next-linux-chat-core-hot-v3-learning-loop";
pub(crate) const DEFAULT_LINUX_CHAT_CORE_PROFILE: &str = "examples/linux-chat-core.profile.json";
const HOT_MAGIC: &[u8] = b"LLMWCHATCORE3\0";
const HOT_FORMAT_VERSION: u32 = 3;
const HOT_FORMAT: &str = "interned-packed-readout-v3";
const HOT_TARGET_BYTES: u64 = 6 * 1024 * 1024;
const PACKED_FACT_RECORD_BYTES: u64 = 37;

#[derive(Clone)]
pub(crate) struct LinuxChatCoreBuildConfig {
    pub profile: PathBuf,
    pub residual_pack: PathBuf,
    pub dialogue_overlay: PathBuf,
    pub centers_overlay: PathBuf,
    pub vpn_overlay: PathBuf,
    pub broad_eval: Option<PathBuf>,
    pub heldout_eval: Option<PathBuf>,
    pub cache_dir: PathBuf,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LinuxChatCoreGateConfig {
    pub profile: PathBuf,
    pub residual_pack: PathBuf,
    pub dialogue_overlay: PathBuf,
    pub centers_overlay: PathBuf,
    pub vpn_overlay: PathBuf,
    pub broad_eval: Option<PathBuf>,
    pub heldout_eval: Option<PathBuf>,
    pub center_learning_script: Option<PathBuf>,
    pub cache_dir: PathBuf,
    pub manifest: Option<PathBuf>,
    pub rebuild_cache: bool,
    pub max_facts: usize,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LinuxChatCoreAskConfig {
    pub text: String,
    pub residual_pack: PathBuf,
    pub dialogue_overlay: PathBuf,
    pub centers_overlay: PathBuf,
    pub vpn_overlay: PathBuf,
    pub cache_dir: PathBuf,
    pub manifest: Option<PathBuf>,
    pub max_facts: usize,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LinuxChatCoreLearnConfig {
    pub profile: PathBuf,
    pub residual_pack: PathBuf,
    pub dialogue_overlay: PathBuf,
    pub centers_overlay: PathBuf,
    pub vpn_overlay: PathBuf,
    pub broad_eval: Option<PathBuf>,
    pub heldout_eval: Option<PathBuf>,
    pub cache_dir: PathBuf,
    pub accept: Option<String>,
    pub reject: Option<String>,
    pub domain: Option<String>,
    pub overlay: Option<String>,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LinuxChatCoreLearnEvalConfig {
    pub profile: PathBuf,
    pub residual_pack: PathBuf,
    pub cache_dir: PathBuf,
    pub reset_scratch: bool,
    pub max_facts: usize,
    pub out: Option<PathBuf>,
}

struct ChatCoreSpecOverrides<'a> {
    residual_pack: &'a Path,
    dialogue_overlay: &'a Path,
    centers_overlay: &'a Path,
    vpn_overlay: &'a Path,
    broad_eval: &'a Option<PathBuf>,
    heldout_eval: &'a Option<PathBuf>,
    cache_dir: &'a Path,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreSpec {
    pub profile_id: String,
    pub source_memory: LinuxChatCoreSourceMemorySpec,
    pub overlays: Vec<LinuxChatCoreOverlaySpec>,
    pub domains: Vec<LinuxChatCoreDomainSpec>,
    pub evals: Vec<LinuxChatCoreEvalSpec>,
    pub cache: LinuxChatCoreCacheSpec,
    pub invariants: LinuxChatCoreInvariants,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreSourceMemorySpec {
    pub residual_pack: String,
    pub source_of_truth: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreOverlaySpec {
    pub overlay_id: String,
    pub overlay_kind: String,
    pub path: String,
    pub domain_scope: Vec<String>,
    pub required_for_profile_ready: bool,
    pub source_of_truth: bool,
    pub write_policy: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreDomainSpec {
    pub domain_id: String,
    pub routes: Vec<String>,
    pub negative_routes: Vec<String>,
    pub overlay_ids: Vec<String>,
    pub action_scope: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreEvalSpec {
    pub eval_id: String,
    pub path: Option<String>,
    pub required_for_chat_core_gate: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreCacheSpec {
    pub hot_path: String,
    pub index_path: String,
    pub manifest_path: String,
    pub cache_is_source_of_truth: bool,
    pub stale_requires_recompile: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreInvariants {
    pub cache_is_not_memory: bool,
    pub cache_must_match_source_hashes: bool,
    pub eval_uses_scratch_overlays: bool,
    pub stale_cache_blocks_answer_authority: bool,
    pub domains_are_registry_entries_not_top_level_gates: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreArtifactDigest {
    pub artifact_id: String,
    pub kind: String,
    pub path: String,
    pub required: bool,
    pub present: bool,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreCacheManifest {
    pub mode: String,
    pub version: String,
    pub profile_id: String,
    pub created_unix_seconds: u64,
    pub compiler_version: String,
    pub spec_hash: String,
    pub source_artifacts: Vec<LinuxChatCoreArtifactDigest>,
    pub domain_registry_hash: String,
    pub overlay_registry_hash: String,
    pub hot_path: String,
    pub hot_bytes: u64,
    pub hot_sha256: String,
    pub index_path: String,
    pub index_bytes: u64,
    pub index_sha256: String,
    pub cache_is_source_of_truth: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreCacheIndex {
    pub profile_id: String,
    pub residual_summary: LinuxResidualDecodedSummary,
    pub represented_fact_count: usize,
    pub schema_record_count: usize,
    pub residual_record_count: usize,
    pub fallback_record_count: usize,
    pub route_index: Vec<LinuxChatCoreRouteIndexEntry>,
    pub readout_facts: Vec<LinuxChatCoreFactPreview>,
    pub domains: Vec<LinuxChatCoreDomainSpec>,
    pub overlays: Vec<LinuxChatCoreOverlaySpec>,
    pub source_artifacts: Vec<LinuxChatCoreArtifactDigest>,
    pub cache_contract: LinuxChatCoreCacheContract,
}

#[derive(Clone)]
struct LinuxChatCoreHotCache {
    residual_summary: LinuxResidualDecodedSummary,
    route_index: Vec<LinuxChatCoreRouteIndexEntry>,
    readout_facts: Vec<LinuxChatCoreFactPreview>,
    domains: Vec<LinuxChatCoreDomainSpec>,
}

struct LinuxChatCoreEncodedHotCache {
    bytes: Vec<u8>,
    stats: LinuxChatCoreHotStorageStats,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreHotStorageStats {
    pub hot_format: &'static str,
    pub hot_target_bytes: u64,
    pub hot_fits_6m_budget: bool,
    pub hot_bytes: u64,
    pub interned_string_count: usize,
    pub intern_table_bytes: u64,
    pub packed_fact_record_count: usize,
    pub packed_fact_record_bytes: u64,
    pub bytes_per_fact: f32,
    pub bytes_per_answerable_fact: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreFactPreview {
    pub route: String,
    pub subject: String,
    pub subject_role: String,
    pub relation: String,
    pub object: String,
    pub object_role: String,
    pub polarity: String,
    pub evidence_kind: String,
    pub confidence: u8,
    pub memory_kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreRouteIndexEntry {
    pub route: String,
    pub fact_count: usize,
    pub relations: Vec<String>,
    pub memory_kinds: Vec<String>,
    pub polarities: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxChatCoreCacheContract {
    pub compiled_from_source_hashes: bool,
    pub no_secret_scan: bool,
    pub hot_cache_has_no_authority_without_gate: bool,
    pub hot_cache_contains_binary_readout_records: bool,
    pub json_index_required_for_answer_authority: bool,
    pub stale_detection_required: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreDomainRuntime {
    pub interface: &'static str,
    pub suite_count: usize,
    pub suites: Vec<LinuxChatCoreDomainSuiteReport>,
    pub json_index_used_for_domain_runtime: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreDomainSuiteReport {
    pub domain_id: String,
    pub route_count: usize,
    pub negative_route_count: usize,
    pub overlay_count: usize,
    pub action_scope: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub spec: LinuxChatCoreSpec,
    pub manifest: LinuxChatCoreCacheManifest,
    pub source_status: LinuxChatCoreSourceStatus,
    pub cache: LinuxChatCoreCompiledCache,
    pub domain_runtime: LinuxChatCoreDomainRuntime,
    pub token_economics: LinuxChatCoreCacheTokenEconomics,
    pub claim_boundary: LinuxChatCoreClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreGateReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub spec: LinuxChatCoreSpec,
    pub manifest_path: String,
    pub cache_status: LinuxChatCoreCacheStatus,
    pub source_status: LinuxChatCoreSourceStatus,
    pub profile_gate: Option<LinuxProfileClaimGateReport>,
    pub domain_runtime: LinuxChatCoreDomainRuntime,
    pub chat_core: LinuxChatCoreAuthority,
    pub token_economics: LinuxChatCoreCacheTokenEconomics,
    pub claim_boundary: LinuxChatCoreClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreAskReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub text: String,
    pub cache_status: LinuxChatCoreCacheStatus,
    pub domain_runtime: Option<LinuxChatCoreDomainRuntime>,
    pub grounded_packet: LinuxChatCoreGroundedPacket,
    pub token_economics: LinuxChatCoreAskTokenEconomics,
    pub claim_boundary: LinuxChatCoreClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearnReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub selected_overlay: Option<LinuxChatCoreSelectedOverlay>,
    pub learned_delta: Option<LinuxChatCoreLearnedDelta>,
    pub overlay_summary: Option<PersistentWaveMemorySummary>,
    pub feedback_safety: LinuxChatCoreFeedbackSafety,
    pub admission: LinuxChatCoreLearningAdmission,
    pub conflict: LinuxChatCoreLearningConflict,
    pub cache_status_before: LinuxChatCoreCacheStatus,
    pub cache_status_after: LinuxChatCoreCacheStatus,
    pub learning_update: LinuxChatCoreLearningUpdate,
    pub claim_boundary: LinuxChatCoreLearningClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearnEvalReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub scratch_root: String,
    pub learning_loop: LinuxChatCoreLearningLoop,
    pub before: LinuxChatCoreAskSummary,
    pub stale_after_learn: LinuxChatCoreAskSummary,
    pub after: LinuxChatCoreAskSummary,
    pub anti_wave: LinuxChatCoreAskSummary,
    pub safety: LinuxChatCoreLearningSafetyChecks,
    pub regression: LinuxChatCoreLearningRegression,
    pub claim_boundary: LinuxChatCoreLearningClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreSelectedOverlay {
    pub overlay_id: String,
    pub overlay_kind: String,
    pub path: String,
    pub domain_scope: Vec<String>,
    pub source_of_truth: bool,
    pub write_policy: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearnedDelta {
    pub delta_state: String,
    pub source_prompt: String,
    pub source_prompt_hash: String,
    pub source_prompt_redacted: bool,
    pub intent: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub reason: String,
    pub hot_memory_kind_after_rebuild: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreFeedbackSafety {
    pub secret_detected: bool,
    pub secret_refused: bool,
    pub redacted_source_prompt: String,
    pub source_prompt_hash: String,
    pub raw_secret_written: bool,
    pub detector_reasons: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearningAdmission {
    pub write_allowed: bool,
    pub rejected: bool,
    pub quarantined: bool,
    pub duplicate_feedback: bool,
    pub route_known: bool,
    pub domain_id: Option<String>,
    pub overlay_allowed_for_domain: bool,
    pub overlay_scope_matches_domain: bool,
    pub unknown_route_rejected: bool,
    pub foreign_overlay_rejected: bool,
    pub reasons: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearningConflict {
    pub conflict_detected: bool,
    pub exact_duplicate: bool,
    pub base_lrf_conflict: bool,
    pub overlay_conflict: bool,
    pub conflicting_facts: Vec<String>,
    pub conflict_policy: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearningUpdate {
    pub write_allowed: bool,
    pub overlay_written: bool,
    pub quarantine_written: bool,
    pub duplicate_feedback: bool,
    pub cache_marked_stale: bool,
    pub rebuild_required: bool,
    pub hot_not_mutated_directly: bool,
    pub base_lrf_not_mutated: bool,
    pub raw_secret_written: bool,
    pub hot_hash_before: String,
    pub hot_hash_after: String,
    pub base_lrf_hash_before: String,
    pub base_lrf_hash_after: String,
    pub write_policy: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearningLoop {
    pub before_blocked: bool,
    pub overlay_written: bool,
    pub cache_marked_stale: bool,
    pub ask_blocked_while_stale: bool,
    pub rebuild_required: bool,
    pub hot_rebuilt: bool,
    pub after_answer_changed: bool,
    pub answer_from_hot: bool,
    pub target_query_improved: bool,
    pub anti_center_replay_observed: bool,
    pub unrelated_route_preserved: bool,
    pub false_positive_regressed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearningRegression {
    pub bash_preserved: bool,
    pub systemctl_preserved: bool,
    pub unrelated_route_preserved: bool,
    pub false_positive_regressed: bool,
    pub mini_heldout_passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearningSafetyChecks {
    pub duplicate_feedback_no_write: bool,
    pub conflicting_feedback_quarantined: bool,
    pub unknown_route_rejected: bool,
    pub wrong_overlay_rejected: bool,
    pub secret_feedback_refused: bool,
    pub poison_feedback_quarantined: bool,
    pub raw_secret_written: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreAskSummary {
    pub query: String,
    pub verdict: String,
    pub cache_fresh: bool,
    pub answer_allowed: bool,
    pub state: String,
    pub readout_source: String,
    pub answer: String,
    pub evidence_memory_kinds: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreLearningClaimBoundary {
    pub linux_profile_dynamic_learning_ready: bool,
    pub hot_cache_learns_directly: bool,
    pub overlay_is_source_of_truth_for_learning: bool,
    pub cache_rebuild_required_after_overlay_write: bool,
    pub general_llm_ready: bool,
    pub global_nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreCacheTokenEconomics {
    pub estimate_method: &'static str,
    pub source_artifacts_bytes: u64,
    pub source_artifacts_estimated_tokens: u64,
    pub cache_hot_bytes: u64,
    pub cache_index_bytes: u64,
    pub cache_total_bytes: u64,
    pub cache_estimated_tokens: u64,
    pub cache_vs_source_bytes_ratio: f32,
    pub cache_is_runtime_index_not_prompt_payload: bool,
    pub note: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreAskTokenEconomics {
    pub estimate_method: &'static str,
    pub source_artifacts_size_bytes: u64,
    pub source_artifacts_bytes: u64,
    pub source_artifacts_estimated_tokens: u64,
    pub full_cache_index_size_bytes: u64,
    pub cache_index_bytes: u64,
    pub cache_index_estimated_tokens: u64,
    pub grounded_packet_size_bytes: u64,
    pub grounded_packet_bytes: u64,
    pub grounded_packet_estimated_tokens: u64,
    pub actual_answer_context_bytes: u64,
    pub actual_answer_context_estimated_tokens: u64,
    pub estimated_tokens_saved_vs_source: u64,
    pub estimated_tokens_saved_vs_cache_index: u64,
    pub source_to_packet_reduction_ratio: f32,
    pub cache_index_to_packet_reduction_ratio: f32,
    pub cache_is_runtime_index_not_prompt_payload: bool,
    pub note: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreSourceStatus {
    pub source_memory_loaded: bool,
    pub source_of_truth: Vec<String>,
    pub overlays_present: usize,
    pub overlays_missing: usize,
    pub required_missing: Vec<String>,
    pub eval_artifacts_present: usize,
    pub source_hash: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreCompiledCache {
    pub hot_format: &'static str,
    pub hot_path: String,
    pub hot_bytes: u64,
    pub hot_sha256: String,
    pub hot_target_bytes: u64,
    pub hot_fits_6m_budget: bool,
    pub index_path: String,
    pub index_bytes: u64,
    pub index_sha256: String,
    pub manifest_path: String,
    pub manifest_sha256: String,
    pub hot_readout_record_count: usize,
    pub hot_route_record_count: usize,
    pub hot_domain_record_count: usize,
    pub hot_interned_string_count: usize,
    pub hot_intern_table_bytes: u64,
    pub hot_packed_fact_record_bytes: u64,
    pub hot_bytes_per_fact: f32,
    pub hot_bytes_per_answerable_fact: f32,
    pub json_index_required_for_answer_authority: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreCacheStatus {
    pub manifest_present: bool,
    pub hot_present: bool,
    pub index_present: bool,
    pub cache_fresh: bool,
    pub stale_reasons: Vec<String>,
    pub current_source_hash: String,
    pub manifest_source_hash: String,
    pub cache_is_source_of_truth: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreAuthority {
    pub safe_to_use_cache: bool,
    pub safe_to_answer_from_cache: bool,
    pub compiled_runtime_cache_ready: bool,
    pub profile_gate_ready: bool,
    pub source_hash_matched: bool,
    pub stale_cache_blocks_answer_authority: bool,
    pub cache_is_source_of_truth: bool,
    pub compatibility_wrapper_for_linux_chat_profile_gate: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreGroundedPacket {
    pub cache_fresh: bool,
    pub answer_allowed: bool,
    pub readout_source: String,
    pub cache_is_runtime_index_not_prompt_payload: bool,
    pub domain_suites: Vec<String>,
    pub decision_state: String,
    pub answer: String,
    pub intent: String,
    pub route_priors: Vec<String>,
    pub evidence_count: usize,
    pub anti_wave_hits: Vec<String>,
    pub evidence: Vec<LinuxChatCoreGroundedEvidence>,
    pub compact_evidence: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreGroundedEvidence {
    pub route: String,
    pub role: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub memory_kind: String,
    pub confidence: u8,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreClaimBoundary {
    pub linux_chat_core_cache_ready: bool,
    pub compiled_cache_matches_source_memory: bool,
    pub cache_is_source_of_truth: bool,
    pub cache_is_runtime_index_not_prompt_payload: bool,
    pub source_memory_required_for_authority: bool,
    pub stale_cache_blocks_answer_authority: bool,
    pub profile_scoped_chat_core_ready: bool,
    pub general_llm_ready: bool,
    pub global_nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

pub(crate) fn build_linux_chat_core_cache_report(
    config: LinuxChatCoreBuildConfig,
) -> Result<LinuxChatCoreBuildReport> {
    let overrides = ChatCoreSpecOverrides {
        residual_pack: &config.residual_pack,
        dialogue_overlay: &config.dialogue_overlay,
        centers_overlay: &config.centers_overlay,
        vpn_overlay: &config.vpn_overlay,
        broad_eval: &config.broad_eval,
        heldout_eval: &config.heldout_eval,
        cache_dir: &config.cache_dir,
    };
    let spec = load_chat_core_spec(&config.profile, &overrides)?;
    let compiled = compile_chat_core_cache(&spec)?;
    let source_status = source_status(&compiled.manifest.source_artifacts);
    let token_economics =
        cache_token_economics(&compiled.manifest.source_artifacts, &compiled.manifest);
    let cache = compiled.cache;
    let manifest = compiled.manifest;
    let domain_runtime = domain_runtime_report(&spec.domains);
    let report = LinuxChatCoreBuildReport {
        mode: "llmwave-big-linux-chat-core-build",
        version: LINUX_CHAT_CORE_VERSION,
        verdict: "LINUX_CHAT_CORE_CACHE_READY_NOT_GENERAL_LLM",
        spec,
        manifest,
        source_status,
        cache,
        domain_runtime,
        token_economics,
        claim_boundary: claim_boundary(true, false, false),
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_chat_core_gate_report(
    config: LinuxChatCoreGateConfig,
) -> Result<LinuxChatCoreGateReport> {
    build_linux_chat_core_gate_report_inner(config, false)
}

pub(crate) fn build_linux_chat_core_profile_gate_report(
    config: LinuxChatCoreGateConfig,
) -> Result<LinuxChatCoreGateReport> {
    build_linux_chat_core_gate_report_inner(config, true)
}

fn build_linux_chat_core_gate_report_inner(
    config: LinuxChatCoreGateConfig,
    run_profile_gate: bool,
) -> Result<LinuxChatCoreGateReport> {
    let overrides = ChatCoreSpecOverrides {
        residual_pack: &config.residual_pack,
        dialogue_overlay: &config.dialogue_overlay,
        centers_overlay: &config.centers_overlay,
        vpn_overlay: &config.vpn_overlay,
        broad_eval: &config.broad_eval,
        heldout_eval: &config.heldout_eval,
        cache_dir: &config.cache_dir,
    };
    let spec = load_chat_core_spec(&config.profile, &overrides)?;
    if config.rebuild_cache {
        compile_chat_core_cache(&spec)?;
    }
    let manifest_path = manifest_path(&config);
    let cache_status = evaluate_cache(&spec, &manifest_path)?;
    let current_artifacts = source_artifacts(&spec)?;
    let source_status = source_status(&current_artifacts);
    let token_economics =
        cache_token_economics_from_manifest_path(&current_artifacts, &manifest_path)?;
    let profile_gate = if run_profile_gate && cache_status.cache_fresh {
        Some(build_linux_profile_claim_gate_report(
            LinuxProfileClaimGateConfig {
                residual_pack: config.residual_pack.clone(),
                broad_eval: config.broad_eval.clone(),
                heldout_eval: config.heldout_eval.clone(),
                run_chat_learning_eval: true,
                chat_learning_memory: config.cache_dir.join("eval/dialogue-check.lwm"),
                run_center_learning_eval: true,
                center_learning_memory: config.cache_dir.join("eval/centers-check.lwm"),
                center_learning_script: config.center_learning_script.clone(),
                run_vpn_training_eval: true,
                vpn_memory: config.cache_dir.join("eval/vpn-check.lwm"),
                max_facts: config.max_facts.max(1),
                out: None,
            },
        )?)
    } else {
        None
    };
    let profile_ready = profile_gate
        .as_ref()
        .map(|gate| gate.chat_target.ready)
        .unwrap_or(false);
    let chat_core_ready = if run_profile_gate {
        cache_status.cache_fresh && profile_ready
    } else {
        cache_status.cache_fresh
    };
    let domain_runtime = domain_runtime_report(&spec.domains);
    let report = LinuxChatCoreGateReport {
        mode: if run_profile_gate {
            "llmwave-big-linux-chat-core-profile-gate"
        } else {
            "llmwave-big-linux-chat-core-authority-gate"
        },
        version: LINUX_CHAT_CORE_VERSION,
        verdict: if chat_core_ready {
            if run_profile_gate {
                "LLMWAVE_LINUX_CHAT_CORE_PROFILE_READY_NOT_GENERAL_LLM"
            } else {
                "LLMWAVE_LINUX_CHAT_CORE_AUTHORITY_READY_NOT_GENERAL_LLM"
            }
        } else if !cache_status.cache_fresh {
            "LINUX_CHAT_CORE_CACHE_STALE"
        } else {
            "LINUX_CHAT_CORE_BLOCKED_BY_PROFILE_GATE"
        },
        spec,
        manifest_path: path_string(&manifest_path),
        cache_status: cache_status.clone(),
        source_status,
        profile_gate,
        domain_runtime,
        chat_core: LinuxChatCoreAuthority {
            safe_to_use_cache: cache_status.cache_fresh,
            safe_to_answer_from_cache: chat_core_ready,
            compiled_runtime_cache_ready: cache_status.cache_fresh,
            profile_gate_ready: run_profile_gate && profile_ready,
            source_hash_matched: cache_status.cache_fresh,
            stale_cache_blocks_answer_authority: true,
            cache_is_source_of_truth: false,
            compatibility_wrapper_for_linux_chat_profile_gate: run_profile_gate,
        },
        token_economics,
        claim_boundary: claim_boundary(cache_status.cache_fresh, true, chat_core_ready),
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_chat_core_ask_report(
    config: LinuxChatCoreAskConfig,
) -> Result<LinuxChatCoreAskReport> {
    let manifest = config
        .manifest
        .clone()
        .unwrap_or_else(|| config.cache_dir.join("chat-core.manifest.json"));
    let cache_status = evaluate_cache_from_manifest_for_ask(&manifest, &config)?;
    let hot_cache = if cache_status.cache_fresh {
        Some(load_hot_cache_from_manifest(&manifest)?)
    } else {
        None
    };
    let domain_runtime = hot_cache
        .as_ref()
        .map(|cache| domain_runtime_report(&cache.domains));
    let query_wave = build_linux_query_wave_report(LinuxQueryWaveConfig {
        text: config.text.clone(),
    })
    .query_wave;
    let active_domain_suites = hot_cache
        .as_ref()
        .map(|cache| select_domain_suites(&cache.domains, &query_wave))
        .unwrap_or_default();
    let reason = if let Some(cache) = &hot_cache {
        let facts = hot_facts_for_query(cache, &query_wave, &active_domain_suites);
        Some(build_linux_reason_report_from_decoded_facts(
            cache.residual_summary.clone(),
            &facts,
            &config.text,
            config.max_facts.max(1),
        ))
    } else {
        None
    };
    let learned_anti_wave_hits = hot_cache
        .as_ref()
        .map(|cache| learned_anti_wave_hits(cache, &config.text, &query_wave))
        .unwrap_or_default();
    let packet = match reason {
        Some(report) => {
            if !learned_anti_wave_hits.is_empty() {
                let evidence = learned_anti_wave_hits
                    .iter()
                    .map(grounded_evidence_from_fact_preview)
                    .collect::<Vec<_>>();
                let compact_evidence = evidence
                    .iter()
                    .map(compact_grounded_evidence)
                    .collect::<Vec<_>>();
                return build_chat_core_ask_report_with_packet(
                    config,
                    manifest,
                    cache_status,
                    domain_runtime,
                    LinuxChatCoreGroundedPacket {
                        cache_fresh: true,
                        answer_allowed: false,
                        readout_source: "compiled_chat_core_hot".to_string(),
                        cache_is_runtime_index_not_prompt_payload: true,
                        domain_suites: active_domain_suites
                            .iter()
                            .map(|domain| domain.domain_id.clone())
                            .collect(),
                        decision_state: "LEARNED_ANTI_WAVE_SUPPRESSED".to_string(),
                        answer: learned_anti_wave_answer(&learned_anti_wave_hits),
                        intent: query_wave.intent,
                        route_priors: query_wave.route_priors,
                        evidence_count: evidence.len(),
                        anti_wave_hits: learned_anti_wave_hits
                            .iter()
                            .map(|fact| format!("{} does not prove {}", fact.subject, fact.object))
                            .collect(),
                        evidence,
                        compact_evidence,
                    },
                );
            }
            let packet_evidence = selected_packet_evidence(&report);
            LinuxChatCoreGroundedPacket {
                cache_fresh: true,
                answer_allowed: report.decision.answer_allowed,
                readout_source: "compiled_chat_core_hot".to_string(),
                cache_is_runtime_index_not_prompt_payload: true,
                domain_suites: active_domain_suites
                    .iter()
                    .map(|domain| domain.domain_id.clone())
                    .collect(),
                decision_state: report.decision.state,
                answer: report.decision.answer,
                intent: report.query_wave.intent,
                route_priors: report.query_wave.route_priors,
                evidence_count: packet_evidence.len(),
                anti_wave_hits: report
                    .anti_wave_hits
                    .iter()
                    .map(|hit| {
                        format!("{} -> {}:{}", hit.shortcut, hit.replacement_peak, hit.reason)
                    })
                    .collect(),
                evidence: packet_evidence
                    .iter()
                    .map(LinuxChatCoreGroundedEvidence::from)
                    .collect(),
                compact_evidence: packet_evidence
                    .iter()
                    .map(compact_evidence_step)
                    .collect(),
            }
        }
        None => LinuxChatCoreGroundedPacket {
            cache_fresh: false,
            answer_allowed: false,
            readout_source: "none_cache_stale".to_string(),
            cache_is_runtime_index_not_prompt_payload: true,
            domain_suites: Vec::new(),
            decision_state: "CACHE_STALE_NO_AUTHORITY".to_string(),
            answer: "Cache is stale or missing; rebuild and gate ChatCore before using it as answer support.".to_string(),
            intent: "unknown".to_string(),
            route_priors: Vec::new(),
            evidence_count: 0,
            anti_wave_hits: cache_status.stale_reasons.clone(),
            evidence: Vec::new(),
            compact_evidence: Vec::new(),
        },
    };
    build_chat_core_ask_report_with_packet(config, manifest, cache_status, domain_runtime, packet)
}

fn build_chat_core_ask_report_with_packet(
    config: LinuxChatCoreAskConfig,
    manifest: PathBuf,
    cache_status: LinuxChatCoreCacheStatus,
    domain_runtime: Option<LinuxChatCoreDomainRuntime>,
    packet: LinuxChatCoreGroundedPacket,
) -> Result<LinuxChatCoreAskReport> {
    let token_economics = ask_token_economics(&manifest, &packet)?;
    let report = LinuxChatCoreAskReport {
        mode: "llmwave-big-linux-chat-core-ask",
        version: LINUX_CHAT_CORE_VERSION,
        verdict: if packet.cache_fresh {
            "LINUX_CHAT_CORE_PACKET_READY_NOT_GENERAL_LLM"
        } else {
            "LINUX_CHAT_CORE_CACHE_STALE"
        },
        text: config.text,
        cache_status: cache_status.clone(),
        domain_runtime,
        grounded_packet: packet,
        token_economics,
        claim_boundary: claim_boundary(cache_status.cache_fresh, true, false),
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_chat_core_learn_report(
    config: LinuxChatCoreLearnConfig,
) -> Result<LinuxChatCoreLearnReport> {
    let overrides = ChatCoreSpecOverrides {
        residual_pack: &config.residual_pack,
        dialogue_overlay: &config.dialogue_overlay,
        centers_overlay: &config.centers_overlay,
        vpn_overlay: &config.vpn_overlay,
        broad_eval: &config.broad_eval,
        heldout_eval: &config.heldout_eval,
        cache_dir: &config.cache_dir,
    };
    let spec = load_chat_core_spec(&config.profile, &overrides)?;
    let manifest_path = PathBuf::from(&spec.cache.manifest_path);
    let hot_path = PathBuf::from(&spec.cache.hot_path);
    let base_lrf_path = PathBuf::from(&spec.source_memory.residual_pack);
    let cache_status_before = evaluate_cache(&spec, &manifest_path)?;
    let hot_hash_before = optional_hash_file(&hot_path)?;
    let base_lrf_hash_before = optional_hash_file(&base_lrf_path)?;
    let raw_feedback = chat_core_raw_feedback(config.accept.as_deref(), config.reject.as_deref())?;
    let feedback_safety = sanitize_feedback(raw_feedback);
    if feedback_safety.secret_refused {
        let cache_status_after = evaluate_cache(&spec, &manifest_path)?;
        let hot_hash_after = optional_hash_file(&hot_path)?;
        let base_lrf_hash_after = optional_hash_file(&base_lrf_path)?;
        let report = LinuxChatCoreLearnReport {
            mode: "llmwave-big-linux-chat-core-learn",
            version: LINUX_CHAT_CORE_VERSION,
            verdict: "LLMWAVE_CHAT_CORE_LEARNING_WRITE_REJECTED",
            selected_overlay: None,
            learned_delta: None,
            overlay_summary: None,
            feedback_safety,
            admission: rejected_admission("secret_feedback_refused"),
            conflict: no_learning_conflict(),
            cache_status_before,
            cache_status_after,
            learning_update: LinuxChatCoreLearningUpdate {
                write_allowed: false,
                overlay_written: false,
                quarantine_written: false,
                duplicate_feedback: false,
                cache_marked_stale: false,
                rebuild_required: false,
                hot_not_mutated_directly: hot_hash_before == hot_hash_after,
                base_lrf_not_mutated: base_lrf_hash_before == base_lrf_hash_after,
                raw_secret_written: false,
                hot_hash_before,
                hot_hash_after,
                base_lrf_hash_before,
                base_lrf_hash_after,
                write_policy: "reject_secret_before_overlay_write",
            },
            claim_boundary: learning_claim_boundary(false),
        };
        write_json_if_requested(config.out.as_deref(), &report)?;
        return Ok(report);
    }
    let learn_request = parse_chat_core_learn_request(
        config.accept.as_deref(),
        config.reject.as_deref(),
        config.domain.as_deref(),
        config.overlay.as_deref(),
        feedback_safety.redacted_source_prompt.clone(),
    )?;
    let selected_overlay = select_learning_overlay(&spec, &learn_request)?;
    let mut admission = validate_learning_admission(&spec, &learn_request, selected_overlay);
    let mut conflict = no_learning_conflict();
    if admission.write_allowed {
        conflict = inspect_learning_conflict(&spec, &learn_request)?;
        if conflict.exact_duplicate {
            admission.write_allowed = false;
            admission.duplicate_feedback = true;
            admission
                .reasons
                .push("duplicate_feedback_no_write".to_string());
        } else if conflict.conflict_detected {
            admission.quarantined = true;
            admission
                .reasons
                .push("conflicting_feedback_quarantined".to_string());
        }
    }
    if !admission.write_allowed && !admission.quarantined {
        let cache_status_after = evaluate_cache(&spec, &manifest_path)?;
        let hot_hash_after = optional_hash_file(&hot_path)?;
        let base_lrf_hash_after = optional_hash_file(&base_lrf_path)?;
        let duplicate_feedback = admission.duplicate_feedback;
        let report = LinuxChatCoreLearnReport {
            mode: "llmwave-big-linux-chat-core-learn",
            version: LINUX_CHAT_CORE_VERSION,
            verdict: if admission.duplicate_feedback {
                "LLMWAVE_CHAT_CORE_DUPLICATE_FEEDBACK_NO_WRITE"
            } else {
                "LLMWAVE_CHAT_CORE_LEARNING_WRITE_REJECTED"
            },
            selected_overlay: Some(selected_overlay_report(selected_overlay)),
            learned_delta: None,
            overlay_summary: None,
            feedback_safety,
            admission,
            conflict,
            cache_status_before,
            cache_status_after,
            learning_update: LinuxChatCoreLearningUpdate {
                write_allowed: false,
                overlay_written: false,
                quarantine_written: false,
                duplicate_feedback,
                cache_marked_stale: false,
                rebuild_required: false,
                hot_not_mutated_directly: hot_hash_before == hot_hash_after,
                base_lrf_not_mutated: base_lrf_hash_before == base_lrf_hash_after,
                raw_secret_written: false,
                hot_hash_before,
                hot_hash_after,
                base_lrf_hash_before,
                base_lrf_hash_after,
                write_policy: "learning_write_rejected_before_overlay_append",
            },
            claim_boundary: learning_claim_boundary(false),
        };
        write_json_if_requested(config.out.as_deref(), &report)?;
        return Ok(report);
    }
    let delta_state = if admission.quarantined {
        DELTA_WATCH_TRACE.to_string()
    } else {
        learn_request.delta_state.clone()
    };
    let relation = if admission.quarantined {
        "candidate_quarantine".to_string()
    } else {
        learn_request.relation.clone()
    };
    let polarity = if admission.quarantined {
        "watch".to_string()
    } else {
        learn_request.polarity.clone()
    };
    let reason = if admission.quarantined {
        format!(
            "{}; quarantined before hot projection",
            learn_request.reason
        )
    } else {
        learn_request.reason.clone()
    };
    let record = build_delta_record(PersistentWaveDeltaSpec {
        delta_state: delta_state.clone(),
        source_prompt: learn_request.source_prompt.clone(),
        intent: learn_request.intent.clone(),
        route: learn_request.route.clone(),
        subject: learn_request.subject.clone(),
        relation: relation.clone(),
        object: learn_request.object.clone(),
        polarity: polarity.clone(),
        reason: reason.clone(),
        strength: learn_request.strength,
    });
    let overlay_path = PathBuf::from(&selected_overlay.path);
    let overlay_summary = append_delta(&overlay_path, record.clone())?;
    let cache_status_after = evaluate_cache(&spec, &manifest_path)?;
    let hot_hash_after = optional_hash_file(&hot_path)?;
    let base_lrf_hash_after = optional_hash_file(&base_lrf_path)?;
    let cache_marked_stale = !cache_status_after.cache_fresh
        && cache_status_after.stale_reasons.iter().any(|reason| {
            reason == "source_memory_hash_changed" || reason == "cache_manifest_missing"
        });
    let report = LinuxChatCoreLearnReport {
        mode: "llmwave-big-linux-chat-core-learn",
        version: LINUX_CHAT_CORE_VERSION,
        verdict: if admission.quarantined {
            "LLMWAVE_CHAT_CORE_FEEDBACK_QUARANTINED_CACHE_STALE"
        } else if cache_marked_stale {
            "LLMWAVE_CHAT_CORE_OVERLAY_WRITTEN_CACHE_STALE"
        } else {
            "LLMWAVE_CHAT_CORE_OVERLAY_WRITTEN_CACHE_REBUILD_REQUIRED"
        },
        selected_overlay: Some(selected_overlay_report(selected_overlay)),
        learned_delta: Some(LinuxChatCoreLearnedDelta {
            delta_state: record.delta_state,
            source_prompt: record.source_prompt,
            source_prompt_hash: feedback_safety.source_prompt_hash.clone(),
            source_prompt_redacted: feedback_safety.secret_detected,
            intent: record.intent,
            route: record.route,
            subject: record.subject,
            relation: record.relation,
            object: record.object,
            polarity: record.polarity,
            reason: record.reason,
            hot_memory_kind_after_rebuild: if admission.quarantined {
                "quarantined_feedback_not_projected"
            } else if learn_request.delta_state == DELTA_NEGATIVE {
                "learned_anti_wave"
            } else {
                "learned_overlay"
            },
        }),
        overlay_summary: Some(overlay_summary),
        feedback_safety,
        admission: admission.clone(),
        conflict,
        cache_status_before,
        cache_status_after,
        learning_update: LinuxChatCoreLearningUpdate {
            write_allowed: admission.write_allowed,
            overlay_written: true,
            quarantine_written: admission.quarantined,
            duplicate_feedback: false,
            cache_marked_stale,
            rebuild_required: true,
            hot_not_mutated_directly: hot_hash_before == hot_hash_after,
            base_lrf_not_mutated: base_lrf_hash_before == base_lrf_hash_after,
            raw_secret_written: false,
            hot_hash_before,
            hot_hash_after,
            base_lrf_hash_before,
            base_lrf_hash_after,
            write_policy: "append_overlay_then_recompile_cache",
        },
        claim_boundary: learning_claim_boundary(false),
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_chat_core_learn_eval_report(
    config: LinuxChatCoreLearnEvalConfig,
) -> Result<LinuxChatCoreLearnEvalReport> {
    let scratch_root = config.cache_dir.join("chat-core-learn-eval");
    if config.reset_scratch && scratch_root.exists() {
        fs::remove_dir_all(&scratch_root)
            .with_context(|| format!("remove {}", scratch_root.display()))?;
    }
    fs::create_dir_all(&scratch_root)
        .with_context(|| format!("create {}", scratch_root.display()))?;
    let scratch_cache = scratch_root.join("cache");
    let dialogue_overlay = scratch_root.join("dialogue.lwm");
    let centers_overlay = scratch_root.join("centers.lwm");
    let vpn_overlay = scratch_root.join("vpn.lwm");
    let no_eval: Option<PathBuf> = None;
    let paths = ChatCoreSpecOverrides {
        residual_pack: &config.residual_pack,
        dialogue_overlay: &dialogue_overlay,
        centers_overlay: &centers_overlay,
        vpn_overlay: &vpn_overlay,
        broad_eval: &no_eval,
        heldout_eval: &no_eval,
        cache_dir: &scratch_cache,
    };
    let spec = load_chat_core_spec(&config.profile, &paths)?;
    compile_chat_core_cache(&spec)?;

    let target_query = "which package provides command foocmd";
    let before = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
        text: target_query.to_string(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        cache_dir: scratch_cache.clone(),
        manifest: None,
        max_facts: config.max_facts.max(1),
        out: None,
    })?;
    let learn = build_linux_chat_core_learn_report(LinuxChatCoreLearnConfig {
        profile: config.profile.clone(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        broad_eval: None,
        heldout_eval: None,
        cache_dir: scratch_cache.clone(),
        accept: Some("foocmd | linux.apt.command.package-command | foopkg".to_string()),
        reject: None,
        domain: None,
        overlay: Some("dialogue".to_string()),
        out: None,
    })?;
    let stale_after_learn = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
        text: target_query.to_string(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        cache_dir: scratch_cache.clone(),
        manifest: None,
        max_facts: config.max_facts.max(1),
        out: None,
    })?;
    let hot_hash_before_rebuild = hash_file(&PathBuf::from(&spec.cache.hot_path))?;
    compile_chat_core_cache(&spec)?;
    let hot_hash_after_rebuild = hash_file(&PathBuf::from(&spec.cache.hot_path))?;
    let after = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
        text: target_query.to_string(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        cache_dir: scratch_cache.clone(),
        manifest: None,
        max_facts: config.max_facts.max(1),
        out: None,
    })?;
    let duplicate = build_linux_chat_core_learn_report(LinuxChatCoreLearnConfig {
        profile: config.profile.clone(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        broad_eval: None,
        heldout_eval: None,
        cache_dir: scratch_cache.clone(),
        accept: Some("foocmd | linux.apt.command.package-command | foopkg".to_string()),
        reject: None,
        domain: None,
        overlay: Some("dialogue".to_string()),
        out: None,
    })?;
    let conflict = build_linux_chat_core_learn_report(LinuxChatCoreLearnConfig {
        profile: config.profile.clone(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        broad_eval: None,
        heldout_eval: None,
        cache_dir: scratch_cache.clone(),
        accept: Some("foocmd | linux.apt.command.package-command | otherpkg".to_string()),
        reject: None,
        domain: None,
        overlay: Some("dialogue".to_string()),
        out: None,
    })?;
    let wrong_overlay = build_linux_chat_core_learn_report(LinuxChatCoreLearnConfig {
        profile: config.profile.clone(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        broad_eval: None,
        heldout_eval: None,
        cache_dir: scratch_cache.clone(),
        accept: None,
        reject: Some("package_installed implies vpn_running".to_string()),
        domain: Some("vpn".to_string()),
        overlay: Some("dialogue".to_string()),
        out: None,
    })?;
    let unknown_route = build_linux_chat_core_learn_report(LinuxChatCoreLearnConfig {
        profile: config.profile.clone(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        broad_eval: None,
        heldout_eval: None,
        cache_dir: scratch_cache.clone(),
        accept: Some("thing | linux.unknown.route | value".to_string()),
        reject: None,
        domain: None,
        overlay: Some("dialogue".to_string()),
        out: None,
    })?;
    let secret = build_linux_chat_core_learn_report(LinuxChatCoreLearnConfig {
        profile: config.profile.clone(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        broad_eval: None,
        heldout_eval: None,
        cache_dir: scratch_cache.clone(),
        accept: Some("foocmd | linux.apt.command.package-command | sk-secret-token".to_string()),
        reject: None,
        domain: None,
        overlay: Some("dialogue".to_string()),
        out: None,
    })?;
    let reject = build_linux_chat_core_learn_report(LinuxChatCoreLearnConfig {
        profile: config.profile.clone(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        broad_eval: None,
        heldout_eval: None,
        cache_dir: scratch_cache.clone(),
        accept: None,
        reject: Some("package_installed implies vpn_running".to_string()),
        domain: Some("vpn".to_string()),
        overlay: Some("vpn".to_string()),
        out: None,
    })?;
    compile_chat_core_cache(&spec)?;
    let after_conflict = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
        text: target_query.to_string(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        cache_dir: scratch_cache.clone(),
        manifest: None,
        max_facts: config.max_facts.max(1),
        out: None,
    })?;
    let bash = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
        text: "which package provides command bash".to_string(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        cache_dir: scratch_cache.clone(),
        manifest: None,
        max_facts: config.max_facts.max(1),
        out: None,
    })?;
    let systemctl = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
        text: "which package provides command systemctl".to_string(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay: dialogue_overlay.clone(),
        centers_overlay: centers_overlay.clone(),
        vpn_overlay: vpn_overlay.clone(),
        cache_dir: scratch_cache.clone(),
        manifest: None,
        max_facts: config.max_facts.max(1),
        out: None,
    })?;
    let anti_wave = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
        text: "does package installed prove vpn running".to_string(),
        residual_pack: config.residual_pack.clone(),
        dialogue_overlay,
        centers_overlay,
        vpn_overlay,
        cache_dir: scratch_cache.clone(),
        manifest: None,
        max_facts: config.max_facts.max(1),
        out: None,
    })?;

    let before_summary = ask_summary(target_query, &before);
    let stale_summary = ask_summary(target_query, &stale_after_learn);
    let after_summary = ask_summary(target_query, &after);
    let anti_summary = ask_summary("does package installed prove vpn running", &anti_wave);
    let bash_preserved = bash.grounded_packet.answer_allowed
        && bash
            .grounded_packet
            .answer
            .to_ascii_lowercase()
            .contains("bash");
    let systemctl_preserved = systemctl.grounded_packet.answer_allowed
        && systemctl
            .grounded_packet
            .answer
            .to_ascii_lowercase()
            .contains("systemd");
    let duplicate_feedback_no_write = duplicate.learning_update.duplicate_feedback
        && !duplicate.learning_update.overlay_written
        && !duplicate.learning_update.cache_marked_stale;
    let conflicting_feedback_quarantined = conflict.admission.quarantined
        && conflict.conflict.conflict_detected
        && conflict.learning_update.quarantine_written
        && conflict
            .learned_delta
            .as_ref()
            .map(|delta| {
                delta.hot_memory_kind_after_rebuild == "quarantined_feedback_not_projected"
            })
            .unwrap_or(false);
    let conflict_preserved_target = after_conflict.grounded_packet.answer_allowed
        && after_conflict
            .grounded_packet
            .answer
            .to_ascii_lowercase()
            .contains("foopkg")
        && !after_conflict
            .grounded_packet
            .answer
            .to_ascii_lowercase()
            .contains("otherpkg");
    let unknown_route_rejected = unknown_route.admission.unknown_route_rejected
        && !unknown_route.learning_update.overlay_written;
    let wrong_overlay_rejected = wrong_overlay.admission.foreign_overlay_rejected
        && !wrong_overlay.learning_update.overlay_written;
    let secret_feedback_refused = secret.feedback_safety.secret_refused
        && !secret.learning_update.overlay_written
        && !secret.learning_update.raw_secret_written;
    let poison_feedback_quarantined = conflicting_feedback_quarantined && conflict_preserved_target;
    let safety = LinuxChatCoreLearningSafetyChecks {
        duplicate_feedback_no_write,
        conflicting_feedback_quarantined,
        unknown_route_rejected,
        wrong_overlay_rejected,
        secret_feedback_refused,
        poison_feedback_quarantined,
        raw_secret_written: secret.learning_update.raw_secret_written,
    };
    let target_query_improved = !before.grounded_packet.answer_allowed
        && after.grounded_packet.answer_allowed
        && after
            .grounded_packet
            .answer
            .to_ascii_lowercase()
            .contains("foopkg")
        && after
            .grounded_packet
            .evidence
            .iter()
            .any(|evidence| evidence.memory_kind == "learned_overlay");
    let anti_center_replay_observed = !anti_wave.grounded_packet.answer_allowed
        && anti_wave.grounded_packet.decision_state == "LEARNED_ANTI_WAVE_SUPPRESSED";
    let unrelated_route_preserved = bash_preserved && systemctl_preserved;
    let false_positive_regressed = !anti_center_replay_observed;
    let loop_report = LinuxChatCoreLearningLoop {
        before_blocked: !before.grounded_packet.answer_allowed,
        overlay_written: learn.learning_update.overlay_written
            && reject.learning_update.overlay_written,
        cache_marked_stale: learn.learning_update.cache_marked_stale
            && reject.learning_update.cache_marked_stale,
        ask_blocked_while_stale: !stale_after_learn.grounded_packet.answer_allowed
            && stale_after_learn.grounded_packet.decision_state == "CACHE_STALE_NO_AUTHORITY",
        rebuild_required: true,
        hot_rebuilt: hot_hash_before_rebuild != hot_hash_after_rebuild,
        after_answer_changed: before.grounded_packet.answer != after.grounded_packet.answer,
        answer_from_hot: after.grounded_packet.readout_source == "compiled_chat_core_hot",
        target_query_improved,
        anti_center_replay_observed,
        unrelated_route_preserved,
        false_positive_regressed,
    };
    let regression = LinuxChatCoreLearningRegression {
        bash_preserved,
        systemctl_preserved,
        unrelated_route_preserved,
        false_positive_regressed,
        mini_heldout_passed: unrelated_route_preserved && !false_positive_regressed,
    };
    let ready = loop_report.before_blocked
        && loop_report.overlay_written
        && loop_report.cache_marked_stale
        && loop_report.ask_blocked_while_stale
        && loop_report.hot_rebuilt
        && loop_report.target_query_improved
        && loop_report.answer_from_hot
        && loop_report.anti_center_replay_observed
        && loop_report.unrelated_route_preserved
        && !loop_report.false_positive_regressed
        && safety.duplicate_feedback_no_write
        && safety.conflicting_feedback_quarantined
        && safety.unknown_route_rejected
        && safety.wrong_overlay_rejected
        && safety.secret_feedback_refused
        && safety.poison_feedback_quarantined
        && !safety.raw_secret_written;
    let report = LinuxChatCoreLearnEvalReport {
        mode: "llmwave-big-linux-chat-core-learn-eval",
        version: LINUX_CHAT_CORE_VERSION,
        verdict: if ready {
            "LLMWAVE_CHAT_CORE_LEARNING_READY_NOT_GENERAL_LLM"
        } else {
            "LLMWAVE_CHAT_CORE_LEARNING_BLOCKED"
        },
        scratch_root: path_string(&scratch_root),
        learning_loop: loop_report,
        before: before_summary,
        stale_after_learn: stale_summary,
        after: after_summary,
        anti_wave: anti_summary,
        safety,
        regression,
        claim_boundary: learning_claim_boundary(ready),
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

struct ChatCoreLearnRequest {
    delta_state: String,
    source_prompt: String,
    intent: String,
    route: String,
    subject: String,
    relation: String,
    object: String,
    polarity: String,
    reason: String,
    overlay_id: Option<String>,
    domain: Option<String>,
    strength: i16,
}

fn chat_core_raw_feedback<'a>(accept: Option<&'a str>, reject: Option<&'a str>) -> Result<&'a str> {
    match (accept, reject) {
        (Some(raw), None) | (None, Some(raw)) => Ok(raw),
        (Some(_), Some(_)) => anyhow::bail!("choose exactly one of --accept or --reject"),
        (None, None) => anyhow::bail!("missing --accept or --reject learning input"),
    }
}

fn sanitize_feedback(raw: &str) -> LinuxChatCoreFeedbackSafety {
    let (secret_detected, detector_reasons) = detect_secret_like(raw);
    let source_prompt_hash = hash_bytes(raw.as_bytes());
    let redacted_source_prompt = if secret_detected {
        format!("[REDACTED_SECRET_FEEDBACK sha256={source_prompt_hash}]")
    } else {
        raw.to_string()
    };
    LinuxChatCoreFeedbackSafety {
        secret_detected,
        secret_refused: secret_detected,
        redacted_source_prompt,
        source_prompt_hash,
        raw_secret_written: false,
        detector_reasons,
    }
}

fn detect_secret_like(raw: &str) -> (bool, Vec<String>) {
    let lower = raw.to_ascii_lowercase();
    let mut reasons = Vec::new();
    for marker in [
        "-----begin",
        "private key",
        "token=",
        "access_token",
        "api_key",
        "apikey",
        "secret=",
        "client_secret",
        "password=",
        "passwd=",
        "authorization:",
        "bearer ",
        "sk-",
        "ghp_",
        "github_pat_",
        "xoxb-",
        "xoxp-",
        "akia",
    ] {
        if lower.contains(marker) {
            reasons.push(format!("secret_marker:{marker}"));
        }
    }
    if raw.split('.').filter(|part| part.len() > 10).count() >= 3 {
        reasons.push("jwt_like_token".to_string());
    }
    if raw
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '+')
        .any(|part| part.len() >= 48 && high_entropyish(part))
    {
        reasons.push("long_high_entropy_token".to_string());
    }
    (!reasons.is_empty(), reasons)
}

fn high_entropyish(value: &str) -> bool {
    let has_upper = value.bytes().any(|byte| byte.is_ascii_uppercase());
    let has_lower = value.bytes().any(|byte| byte.is_ascii_lowercase());
    let has_digit = value.bytes().any(|byte| byte.is_ascii_digit());
    let has_symbol = value.contains(['_', '-', '+']);
    [has_upper, has_lower, has_digit, has_symbol]
        .into_iter()
        .filter(|present| *present)
        .count()
        >= 3
}

fn selected_overlay_report(overlay: &LinuxChatCoreOverlaySpec) -> LinuxChatCoreSelectedOverlay {
    LinuxChatCoreSelectedOverlay {
        overlay_id: overlay.overlay_id.clone(),
        overlay_kind: overlay.overlay_kind.clone(),
        path: overlay.path.clone(),
        domain_scope: overlay.domain_scope.clone(),
        source_of_truth: overlay.source_of_truth,
        write_policy: overlay.write_policy.clone(),
    }
}

fn rejected_admission(reason: &str) -> LinuxChatCoreLearningAdmission {
    LinuxChatCoreLearningAdmission {
        write_allowed: false,
        rejected: true,
        quarantined: false,
        duplicate_feedback: false,
        route_known: false,
        domain_id: None,
        overlay_allowed_for_domain: false,
        overlay_scope_matches_domain: false,
        unknown_route_rejected: reason == "unknown_route_rejected",
        foreign_overlay_rejected: reason == "foreign_overlay_rejected",
        reasons: vec![reason.to_string()],
    }
}

fn no_learning_conflict() -> LinuxChatCoreLearningConflict {
    LinuxChatCoreLearningConflict {
        conflict_detected: false,
        exact_duplicate: false,
        base_lrf_conflict: false,
        overlay_conflict: false,
        conflicting_facts: Vec::new(),
        conflict_policy: "no_conflict",
    }
}

fn validate_learning_admission(
    spec: &LinuxChatCoreSpec,
    request: &ChatCoreLearnRequest,
    overlay: &LinuxChatCoreOverlaySpec,
) -> LinuxChatCoreLearningAdmission {
    let mut reasons = Vec::new();
    let matching_domains = spec
        .domains
        .iter()
        .filter(|domain| {
            domain.routes.iter().any(|route| route == &request.route)
                || domain
                    .negative_routes
                    .iter()
                    .any(|route| route == &request.route)
        })
        .collect::<Vec<_>>();
    let requested_domain = request.domain.as_deref();
    let selected_domain = requested_domain
        .and_then(|domain_id| {
            matching_domains
                .iter()
                .copied()
                .find(|domain| domain.domain_id == domain_id)
        })
        .or_else(|| matching_domains.first().copied());
    let route_known = selected_domain.is_some();
    if !route_known {
        reasons.push("unknown_route_rejected".to_string());
    }
    if requested_domain.is_some() && selected_domain.is_none() {
        reasons.push("domain_route_mismatch".to_string());
    }
    let overlay_allowed_for_domain = selected_domain
        .map(|domain| {
            domain
                .overlay_ids
                .iter()
                .any(|overlay_id| overlay_id == &overlay.overlay_id)
        })
        .unwrap_or(false);
    if route_known && !overlay_allowed_for_domain {
        reasons.push("foreign_overlay_rejected".to_string());
    }
    let overlay_scope_matches_domain = selected_domain
        .map(|domain| {
            overlay
                .domain_scope
                .iter()
                .any(|scope| scope == &domain.domain_id)
        })
        .unwrap_or(false);
    if route_known && !overlay_scope_matches_domain {
        reasons.push("overlay_domain_scope_mismatch".to_string());
    }
    if !overlay.source_of_truth {
        reasons.push("overlay_not_source_of_truth".to_string());
    }
    if overlay.write_policy != "append_overlay_then_recompile_cache" {
        reasons.push("unsupported_overlay_write_policy".to_string());
    }
    let write_allowed = route_known
        && overlay_allowed_for_domain
        && overlay_scope_matches_domain
        && overlay.source_of_truth
        && overlay.write_policy == "append_overlay_then_recompile_cache";
    LinuxChatCoreLearningAdmission {
        write_allowed,
        rejected: !write_allowed,
        quarantined: false,
        duplicate_feedback: false,
        route_known,
        domain_id: selected_domain.map(|domain| domain.domain_id.clone()),
        overlay_allowed_for_domain,
        overlay_scope_matches_domain,
        unknown_route_rejected: !route_known,
        foreign_overlay_rejected: route_known && !overlay_allowed_for_domain,
        reasons,
    }
}

fn inspect_learning_conflict(
    spec: &LinuxChatCoreSpec,
    request: &ChatCoreLearnRequest,
) -> Result<LinuxChatCoreLearningConflict> {
    if request.delta_state == DELTA_NEGATIVE {
        return Ok(no_learning_conflict());
    }
    let decoded_packet =
        load_linux_residual_decoded_packet(&PathBuf::from(&spec.source_memory.residual_pack))?;
    let overlay_facts = load_overlay_facts(&spec.overlays)?;
    let mut conflict = no_learning_conflict();
    let mut inspect_fact = |fact: &LinuxResidualDecodedFact, source: &str| {
        if fact.route != request.route || fact.subject != request.subject {
            return;
        }
        let exact = fact.relation == request.relation
            && fact.object == request.object
            && fact.polarity == request.polarity;
        if exact {
            conflict.exact_duplicate = true;
            return;
        }
        if fact.object != request.object || fact.relation != request.relation {
            conflict.conflict_detected = true;
            if source == "base_lrf" {
                conflict.base_lrf_conflict = true;
            } else {
                conflict.overlay_conflict = true;
            }
            conflict.conflicting_facts.push(format!(
                "{source}: {} | {} | {} | {} | {}",
                fact.route, fact.subject, fact.relation, fact.object, fact.memory_kind
            ));
        }
    };
    for fact in &decoded_packet.facts {
        inspect_fact(fact, "base_lrf");
    }
    for fact in &overlay_facts {
        inspect_fact(fact, "overlay");
    }
    conflict.conflict_policy = if conflict.conflict_detected {
        "write_watch_trace_quarantine_not_projected_to_hot"
    } else if conflict.exact_duplicate {
        "duplicate_feedback_no_write"
    } else {
        "no_conflict"
    };
    Ok(conflict)
}

fn parse_chat_core_learn_request(
    accept: Option<&str>,
    reject: Option<&str>,
    domain: Option<&str>,
    overlay: Option<&str>,
    source_prompt: String,
) -> Result<ChatCoreLearnRequest> {
    match (accept, reject) {
        (Some(_), Some(_)) => anyhow::bail!("choose exactly one of --accept or --reject"),
        (None, None) => anyhow::bail!("missing --accept or --reject learning input"),
        (Some(raw), None) => {
            let fact = parse_pipe_fact(raw)?;
            let route = normalize_accept_route(&fact.route);
            let (relation, _, _) = role_contract_for_route(&route, false);
            let (subject, object) =
                normalize_accept_subject_object(&fact.route, &fact.subject, &fact.object);
            Ok(ChatCoreLearnRequest {
                delta_state: DELTA_POSITIVE.to_string(),
                source_prompt,
                intent: intent_for_route(&route),
                route,
                subject,
                relation,
                object,
                polarity: "positive".to_string(),
                reason: "accepted ChatCore feedback writes a source overlay delta".to_string(),
                overlay_id: overlay.map(str::to_string),
                domain: domain.map(str::to_string),
                strength: 24,
            })
        }
        (None, Some(raw)) => {
            let (subject, route, object) = if raw.contains('|') {
                let fact = parse_pipe_fact(raw)?;
                (fact.subject, fact.route, fact.object)
            } else if let Some((left, right)) = raw.split_once("implies") {
                (
                    normalize_token(left),
                    reject_route_for_domain(domain),
                    normalize_token(right),
                )
            } else {
                anyhow::bail!("--reject expects `subject | route | object` or `A implies B`");
            };
            let route = normalize_reject_route(&route, domain);
            Ok(ChatCoreLearnRequest {
                delta_state: DELTA_NEGATIVE.to_string(),
                source_prompt,
                intent: intent_for_route(&route),
                route,
                subject,
                relation: "does not prove".to_string(),
                object,
                polarity: "negative".to_string(),
                reason: "rejected ChatCore shortcut writes an anti-wave overlay delta".to_string(),
                overlay_id: overlay.map(str::to_string),
                domain: domain.map(str::to_string),
                strength: 28,
            })
        }
    }
}

struct ParsedPipeFact {
    subject: String,
    route: String,
    object: String,
}

fn parse_pipe_fact(raw: &str) -> Result<ParsedPipeFact> {
    let parts = raw.split('|').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        anyhow::bail!("expected `subject | route | object`, got `{raw}`");
    }
    Ok(ParsedPipeFact {
        subject: normalize_token(parts[0]),
        route: parts[1].to_string(),
        object: normalize_token(parts[2]),
    })
}

fn normalize_token(value: &str) -> String {
    value.trim().replace(' ', "_")
}

fn normalize_accept_route(route: &str) -> String {
    match route {
        "linux.apt.command.package-command" => "linux.apt.command.provider".to_string(),
        value => value.to_string(),
    }
}

fn normalize_reject_route(route: &str, domain: Option<&str>) -> String {
    if route.trim().is_empty() {
        reject_route_for_domain(domain)
    } else {
        route.to_string()
    }
}

fn normalize_accept_subject_object(route: &str, subject: &str, object: &str) -> (String, String) {
    match route {
        "linux.apt.command.package-command" => (subject.to_string(), object.to_string()),
        _ => (subject.to_string(), object.to_string()),
    }
}

fn reject_route_for_domain(domain: Option<&str>) -> String {
    match domain.unwrap_or("").trim() {
        "vpn" => "linux.vpn.status".to_string(),
        "exposure" | "socket" => "linux.boundary.socket".to_string(),
        _ => "linux.chatcore.anti_wave".to_string(),
    }
}

fn intent_for_route(route: &str) -> String {
    match route {
        "linux.apt.command.provider"
        | "linux.apt.command.package-command"
        | "linux.package.binary" => "command_provider",
        "linux.vpn.status" | "linux.vpn.config" | "linux.vpn.action" => "vpn_runtime",
        "linux.boundary.socket" | "linux.chatcore.anti_wave" => "boundary_rejection",
        _ => "linux_profile_fact",
    }
    .to_string()
}

fn role_contract_for_route(route: &str, negative: bool) -> (String, String, String) {
    match route {
        "linux.apt.command.provider" => (
            "provided by package".to_string(),
            "command".to_string(),
            "package".to_string(),
        ),
        "linux.apt.command.package-command" => (
            "provides command".to_string(),
            "package".to_string(),
            "command".to_string(),
        ),
        "linux.package.binary" => (
            "provides binary".to_string(),
            "package".to_string(),
            "binary".to_string(),
        ),
        "linux.vpn.status" if negative => (
            "does not prove".to_string(),
            "shortcut".to_string(),
            "runtime_state".to_string(),
        ),
        "linux.boundary.socket" | "linux.chatcore.anti_wave" if negative => (
            "does not prove".to_string(),
            "shortcut".to_string(),
            "boundary".to_string(),
        ),
        _ if negative => (
            "does not prove".to_string(),
            "shortcut".to_string(),
            "boundary".to_string(),
        ),
        _ => (
            "supports route".to_string(),
            "subject".to_string(),
            "object".to_string(),
        ),
    }
}

fn select_learning_overlay<'a>(
    spec: &'a LinuxChatCoreSpec,
    request: &ChatCoreLearnRequest,
) -> Result<&'a LinuxChatCoreOverlaySpec> {
    let inferred_domain = request.domain.as_deref().or_else(|| {
        spec.domains
            .iter()
            .find(|domain| {
                domain.routes.iter().any(|route| route == &request.route)
                    || domain
                        .negative_routes
                        .iter()
                        .any(|route| route == &request.route)
            })
            .map(|domain| domain.domain_id.as_str())
    });
    let preferred = request.overlay_id.as_deref().unwrap_or_else(|| {
        if inferred_domain == Some("vpn") {
            "vpn"
        } else {
            "dialogue"
        }
    });
    spec.overlays
        .iter()
        .find(|overlay| overlay.overlay_id == preferred)
        .with_context(|| format!("ChatCore overlay not found in profile: {preferred}"))
}

fn optional_hash_file(path: &Path) -> Result<String> {
    if path.exists() {
        hash_file(path)
    } else {
        Ok("missing".to_string())
    }
}

fn ask_summary(query: &str, report: &LinuxChatCoreAskReport) -> LinuxChatCoreAskSummary {
    LinuxChatCoreAskSummary {
        query: query.to_string(),
        verdict: report.verdict.to_string(),
        cache_fresh: report.cache_status.cache_fresh,
        answer_allowed: report.grounded_packet.answer_allowed,
        state: report.grounded_packet.decision_state.clone(),
        readout_source: report.grounded_packet.readout_source.clone(),
        answer: report.grounded_packet.answer.clone(),
        evidence_memory_kinds: report
            .grounded_packet
            .evidence
            .iter()
            .map(|evidence| evidence.memory_kind.clone())
            .collect(),
    }
}

fn learning_claim_boundary(ready: bool) -> LinuxChatCoreLearningClaimBoundary {
    LinuxChatCoreLearningClaimBoundary {
        linux_profile_dynamic_learning_ready: ready,
        hot_cache_learns_directly: false,
        overlay_is_source_of_truth_for_learning: true,
        cache_rebuild_required_after_overlay_write: true,
        general_llm_ready: false,
        global_nonlinear_memory_proven: false,
        safe_claim: "ChatCore learning writes source `.lwm` overlays, marks the compiled cache stale, and only changes answer behavior after rebuilding `.hot` from `.lrf` + overlays.",
        blocked_claims: vec!["general_llm_ready", "global_nonlinear_memory_proven"],
    }
}

fn selected_packet_evidence(report: &LinuxReasonReport) -> Vec<LinuxEvidenceStep> {
    if report.query_wave.intent != "command_provider" || !report.decision.answer_allowed {
        return report.evidence_chain.clone();
    }
    for anchor in &report.query_wave.anchors {
        let anchor = anchor.to_ascii_lowercase();
        let exact = report
            .evidence_chain
            .iter()
            .filter(|step| command_provider_step_matches_anchor(step, &anchor))
            .cloned()
            .collect::<Vec<_>>();
        if !exact.is_empty() {
            return exact;
        }
    }
    report.evidence_chain.iter().take(1).cloned().collect()
}

fn command_provider_step_matches_anchor(step: &LinuxEvidenceStep, anchor: &str) -> bool {
    let subject = step.subject.to_ascii_lowercase();
    let object = step.object.to_ascii_lowercase();
    match step.route.as_str() {
        "linux.apt.command.provider" => subject == anchor,
        "linux.apt.command.package-command" => object == anchor,
        "linux.package.binary" => {
            object == anchor || object.rsplit('/').next().is_some_and(|name| name == anchor)
        }
        _ => false,
    }
}

fn learned_anti_wave_hits(
    cache: &LinuxChatCoreHotCache,
    text: &str,
    query: &LinuxQueryWave,
) -> Vec<LinuxChatCoreFactPreview> {
    let normalized_text = normalize_match_text(text);
    let mut query_terms = query
        .anchors
        .iter()
        .chain(query.forbidden_shortcuts.iter())
        .map(|term| normalize_match_text(term))
        .collect::<Vec<_>>();
    query_terms.push(normalized_text.clone());
    cache
        .readout_facts
        .iter()
        .filter(|fact| fact.memory_kind == "learned_anti_wave")
        .filter(|fact| {
            let subject = normalize_match_text(&fact.subject);
            let object = normalize_match_text(&fact.object);
            let subject_hit = query_terms.iter().any(|term| {
                !subject.is_empty() && (term.contains(&subject) || subject.contains(term))
            });
            let object_hit = query_terms.iter().any(|term| {
                !object.is_empty() && (term.contains(&object) || object.contains(term))
            });
            (subject_hit && object_hit)
                || (!subject.is_empty()
                    && !object.is_empty()
                    && normalized_text.contains(&subject)
                    && normalized_text.contains(&object))
        })
        .take(4)
        .cloned()
        .collect()
}

fn normalize_match_text(text: &str) -> String {
    text.to_ascii_lowercase()
        .replace(['_', '-', '/', ':'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn grounded_evidence_from_fact_preview(
    fact: &LinuxChatCoreFactPreview,
) -> LinuxChatCoreGroundedEvidence {
    LinuxChatCoreGroundedEvidence {
        route: fact.route.clone(),
        role: format!("{}->{}", fact.subject_role, fact.object_role),
        subject: fact.subject.clone(),
        relation: fact.relation.clone(),
        object: fact.object.clone(),
        polarity: fact.polarity.clone(),
        memory_kind: fact.memory_kind.clone(),
        confidence: fact.confidence,
    }
}

fn compact_grounded_evidence(evidence: &LinuxChatCoreGroundedEvidence) -> String {
    format!(
        "{} | role={} | subject={} | relation={} | object={} | polarity={} | memory={} | confidence={}",
        evidence.route,
        evidence.role,
        evidence.subject,
        evidence.relation,
        evidence.object,
        evidence.polarity,
        evidence.memory_kind,
        evidence.confidence
    )
}

fn learned_anti_wave_answer(hits: &[LinuxChatCoreFactPreview]) -> String {
    hits.first()
        .map(|fact| {
            format!(
                "Learned anti-wave boundary: {} does not prove {}.",
                fact.subject, fact.object
            )
        })
        .unwrap_or_else(|| "Learned anti-wave boundary suppresses this shortcut.".to_string())
}

fn domain_runtime_report(domains: &[LinuxChatCoreDomainSpec]) -> LinuxChatCoreDomainRuntime {
    LinuxChatCoreDomainRuntime {
        interface: "DomainSuite::select_routes",
        suite_count: domains.len(),
        suites: domains
            .iter()
            .map(|domain| LinuxChatCoreDomainSuiteReport {
                domain_id: domain.domain_id.clone(),
                route_count: domain.routes.len(),
                negative_route_count: domain.negative_routes.len(),
                overlay_count: domain.overlay_ids.len(),
                action_scope: domain.action_scope.clone(),
            })
            .collect(),
        json_index_used_for_domain_runtime: false,
    }
}

fn select_domain_suites(
    domains: &[LinuxChatCoreDomainSpec],
    query: &LinuxQueryWave,
) -> Vec<LinuxChatCoreDomainSpec> {
    let query_routes = query_route_set(query);
    domains
        .iter()
        .filter(|domain| {
            domain
                .routes
                .iter()
                .chain(domain.negative_routes.iter())
                .any(|route| query_routes.contains(route))
        })
        .cloned()
        .collect()
}

fn query_route_set(query: &LinuxQueryWave) -> BTreeSet<String> {
    query
        .route_priors
        .iter()
        .chain(query.required_routes.iter())
        .chain(query.negative_boundaries.iter())
        .cloned()
        .collect()
}

fn hot_facts_for_query(
    cache: &LinuxChatCoreHotCache,
    query: &LinuxQueryWave,
    active_domains: &[LinuxChatCoreDomainSpec],
) -> Vec<LinuxResidualDecodedFact> {
    let mut routes = query_route_set(query);
    for domain in active_domains {
        routes.extend(domain.routes.iter().cloned());
        routes.extend(domain.negative_routes.iter().cloned());
    }
    let selected = cache
        .readout_facts
        .iter()
        .filter(|fact| routes.is_empty() || routes.contains(&fact.route))
        .map(LinuxChatCoreFactPreview::to_decoded_fact)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        cache
            .readout_facts
            .iter()
            .map(LinuxChatCoreFactPreview::to_decoded_fact)
            .collect()
    } else {
        selected
    }
}

fn compact_evidence_step(step: &LinuxEvidenceStep) -> String {
    format!(
        "{} | role={} | subject={} | relation={} | object={} | polarity={} | memory={} | confidence={}",
        step.route,
        step.role,
        step.subject,
        step.relation,
        step.object,
        step.polarity,
        step.memory_kind,
        step.confidence
    )
}

impl From<&LinuxEvidenceStep> for LinuxChatCoreGroundedEvidence {
    fn from(step: &LinuxEvidenceStep) -> Self {
        Self {
            route: step.route.clone(),
            role: step.role.clone(),
            subject: step.subject.clone(),
            relation: step.relation.clone(),
            object: step.object.clone(),
            polarity: step.polarity.clone(),
            memory_kind: step.memory_kind.clone(),
            confidence: step.confidence,
        }
    }
}

fn load_chat_core_spec(
    profile: &Path,
    overrides: &ChatCoreSpecOverrides<'_>,
) -> Result<LinuxChatCoreSpec> {
    let mut spec: LinuxChatCoreSpec = serde_json::from_slice(
        &fs::read(profile).with_context(|| format!("read {}", profile.display()))?,
    )
    .with_context(|| format!("parse {}", profile.display()))?;

    let hot_path = overrides.cache_dir.join("chat-core.hot");
    let index_path = overrides.cache_dir.join("chat-core.index.json");
    let manifest_path = overrides.cache_dir.join("chat-core.manifest.json");

    spec.source_memory.residual_pack = path_string(overrides.residual_pack);
    set_overlay_path(&mut spec, "dialogue", overrides.dialogue_overlay);
    set_overlay_path(&mut spec, "centers", overrides.centers_overlay);
    set_overlay_path(&mut spec, "vpn", overrides.vpn_overlay);
    if let Some(path) = overrides.broad_eval {
        set_eval_path(&mut spec, "broad", path, true);
    }
    if let Some(path) = overrides.heldout_eval {
        set_eval_path(&mut spec, "heldout", path, true);
    }
    spec.cache.hot_path = path_string(&hot_path);
    spec.cache.index_path = path_string(&index_path);
    spec.cache.manifest_path = path_string(&manifest_path);
    spec.cache.cache_is_source_of_truth = false;
    spec.cache.stale_requires_recompile = true;
    spec.invariants.cache_is_not_memory = true;
    spec.invariants.cache_must_match_source_hashes = true;
    spec.invariants.stale_cache_blocks_answer_authority = true;
    Ok(spec)
}

fn set_overlay_path(spec: &mut LinuxChatCoreSpec, overlay_id: &str, path: &Path) {
    if let Some(overlay) = spec
        .overlays
        .iter_mut()
        .find(|overlay| overlay.overlay_id == overlay_id)
    {
        overlay.path = path_string(path);
    }
}

fn set_eval_path(
    spec: &mut LinuxChatCoreSpec,
    eval_id: &str,
    path: &Path,
    required_for_chat_core_gate: bool,
) {
    if let Some(eval) = spec.evals.iter_mut().find(|eval| eval.eval_id == eval_id) {
        eval.path = Some(path_string(path));
        eval.required_for_chat_core_gate = required_for_chat_core_gate;
    } else {
        spec.evals.push(LinuxChatCoreEvalSpec {
            eval_id: eval_id.to_string(),
            path: Some(path_string(path)),
            required_for_chat_core_gate,
        });
    }
}

struct CompiledChatCore {
    manifest: LinuxChatCoreCacheManifest,
    cache: LinuxChatCoreCompiledCache,
}

fn compile_chat_core_cache(spec: &LinuxChatCoreSpec) -> Result<CompiledChatCore> {
    let source_artifacts = source_artifacts(spec)?;
    let decoded_packet =
        load_linux_residual_decoded_packet(&PathBuf::from(&spec.source_memory.residual_pack))?;
    let mut active_facts = decoded_packet.facts.clone();
    active_facts.extend(load_overlay_facts(&spec.overlays)?);
    let route_index = route_index(&active_facts);
    let domain_registry_hash = hash_json(&spec.domains)?;
    let overlay_registry_hash = hash_json(&spec.overlays)?;
    let spec_hash = hash_json(spec)?;
    let readout_facts = active_facts
        .iter()
        .map(LinuxChatCoreFactPreview::from_decoded_fact)
        .collect::<Vec<_>>();
    let domains = spec.domains.clone();
    let index = LinuxChatCoreCacheIndex {
        profile_id: spec.profile_id.clone(),
        residual_summary: decoded_packet.summary.clone(),
        represented_fact_count: active_facts.len(),
        schema_record_count: decoded_packet.summary.schema_record_count,
        residual_record_count: decoded_packet.summary.residual_record_count,
        fallback_record_count: decoded_packet.summary.fallback_record_count,
        route_index: route_index.clone(),
        readout_facts: readout_facts.clone(),
        domains: domains.clone(),
        overlays: spec.overlays.clone(),
        source_artifacts: source_artifacts.clone(),
        cache_contract: LinuxChatCoreCacheContract {
            compiled_from_source_hashes: true,
            no_secret_scan: true,
            hot_cache_has_no_authority_without_gate: true,
            hot_cache_contains_binary_readout_records: true,
            json_index_required_for_answer_authority: false,
            stale_detection_required: true,
        },
    };
    let index_bytes = serde_json::to_vec_pretty(&index)?;
    let hot_cache = LinuxChatCoreHotCache {
        residual_summary: decoded_packet.summary.clone(),
        route_index,
        readout_facts,
        domains,
    };
    let encoded_hot = encode_hot_cache(&hot_cache)?;
    let hot_hash = hash_bytes(&encoded_hot.bytes);
    let index_hash = hash_bytes(&index_bytes);
    let hot_path = PathBuf::from(&spec.cache.hot_path);
    let index_path = PathBuf::from(&spec.cache.index_path);
    let manifest_path = PathBuf::from(&spec.cache.manifest_path);
    write_bytes(&hot_path, &encoded_hot.bytes)?;
    write_bytes(&index_path, &index_bytes)?;
    let manifest = LinuxChatCoreCacheManifest {
        mode: "llmwave-big-linux-chat-core-cache-manifest".to_string(),
        version: LINUX_CHAT_CORE_VERSION.to_string(),
        profile_id: spec.profile_id.clone(),
        created_unix_seconds: now_unix_seconds(),
        compiler_version: LINUX_CHAT_CORE_VERSION.to_string(),
        spec_hash,
        source_artifacts,
        domain_registry_hash,
        overlay_registry_hash,
        hot_path: path_string(&hot_path),
        hot_bytes: encoded_hot.bytes.len() as u64,
        hot_sha256: hot_hash,
        index_path: path_string(&index_path),
        index_bytes: index_bytes.len() as u64,
        index_sha256: index_hash,
        cache_is_source_of_truth: false,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    write_bytes(&manifest_path, &manifest_bytes)?;
    let cache = LinuxChatCoreCompiledCache {
        hot_format: encoded_hot.stats.hot_format,
        hot_path: manifest.hot_path.clone(),
        hot_bytes: manifest.hot_bytes,
        hot_sha256: manifest.hot_sha256.clone(),
        hot_target_bytes: encoded_hot.stats.hot_target_bytes,
        hot_fits_6m_budget: encoded_hot.stats.hot_fits_6m_budget,
        index_path: manifest.index_path.clone(),
        index_bytes: manifest.index_bytes,
        index_sha256: manifest.index_sha256.clone(),
        manifest_path: path_string(&manifest_path),
        manifest_sha256: hash_bytes(&manifest_bytes),
        hot_readout_record_count: hot_cache.readout_facts.len(),
        hot_route_record_count: hot_cache.route_index.len(),
        hot_domain_record_count: hot_cache.domains.len(),
        hot_interned_string_count: encoded_hot.stats.interned_string_count,
        hot_intern_table_bytes: encoded_hot.stats.intern_table_bytes,
        hot_packed_fact_record_bytes: encoded_hot.stats.packed_fact_record_bytes,
        hot_bytes_per_fact: encoded_hot.stats.bytes_per_fact,
        hot_bytes_per_answerable_fact: encoded_hot.stats.bytes_per_answerable_fact,
        json_index_required_for_answer_authority: false,
    };
    Ok(CompiledChatCore { manifest, cache })
}

fn evaluate_cache(
    spec: &LinuxChatCoreSpec,
    manifest_path: &Path,
) -> Result<LinuxChatCoreCacheStatus> {
    if !manifest_path.exists() {
        let current_artifacts = source_artifacts(spec)?;
        return Ok(LinuxChatCoreCacheStatus {
            manifest_present: false,
            hot_present: false,
            index_present: false,
            cache_fresh: false,
            stale_reasons: vec!["cache_manifest_missing".to_string()],
            current_source_hash: combined_hash(
                current_artifacts
                    .iter()
                    .map(|artifact| artifact.sha256.as_str()),
            ),
            manifest_source_hash: "missing".to_string(),
            cache_is_source_of_truth: false,
        });
    }
    let manifest: LinuxChatCoreCacheManifest = serde_json::from_slice(
        &fs::read(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse {}", manifest_path.display()))?;
    let current_artifacts = source_artifacts(spec)?;
    let current_source_hash = combined_hash(
        current_artifacts
            .iter()
            .map(|artifact| artifact.sha256.as_str()),
    );
    let manifest_source_hash = combined_hash(
        manifest
            .source_artifacts
            .iter()
            .map(|artifact| artifact.sha256.as_str()),
    );
    let mut stale_reasons = Vec::new();
    if manifest.version != LINUX_CHAT_CORE_VERSION {
        stale_reasons.push("compiler_version_changed".to_string());
    }
    if manifest.spec_hash != hash_json(spec)? {
        stale_reasons.push("profile_spec_changed".to_string());
    }
    if current_source_hash != manifest_source_hash {
        stale_reasons.push("source_memory_hash_changed".to_string());
    }
    if manifest.cache_is_source_of_truth {
        stale_reasons.push("manifest_claims_cache_as_source_of_truth".to_string());
    }
    let hot_path = PathBuf::from(&manifest.hot_path);
    let index_path = PathBuf::from(&manifest.index_path);
    let hot_present = hot_path.exists();
    let index_present = index_path.exists();
    if !hot_present {
        stale_reasons.push("hot_cache_missing".to_string());
    } else if hash_file(&hot_path)? != manifest.hot_sha256 {
        stale_reasons.push("hot_cache_hash_changed".to_string());
    }
    Ok(LinuxChatCoreCacheStatus {
        manifest_present: true,
        hot_present,
        index_present,
        cache_fresh: stale_reasons.is_empty(),
        stale_reasons,
        current_source_hash,
        manifest_source_hash,
        cache_is_source_of_truth: false,
    })
}

fn evaluate_cache_from_manifest(manifest_path: &Path) -> Result<LinuxChatCoreCacheStatus> {
    if !manifest_path.exists() {
        return Ok(LinuxChatCoreCacheStatus {
            manifest_present: false,
            hot_present: false,
            index_present: false,
            cache_fresh: false,
            stale_reasons: vec!["cache_manifest_missing".to_string()],
            current_source_hash: "missing".to_string(),
            manifest_source_hash: "missing".to_string(),
            cache_is_source_of_truth: false,
        });
    }
    let manifest: LinuxChatCoreCacheManifest = serde_json::from_slice(
        &fs::read(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse {}", manifest_path.display()))?;
    let mut current_artifacts = Vec::new();
    for expected in &manifest.source_artifacts {
        current_artifacts.push(artifact(
            &expected.artifact_id,
            &expected.kind,
            Path::new(&expected.path),
            expected.required,
        )?);
    }
    let current_source_hash = combined_hash(
        current_artifacts
            .iter()
            .map(|artifact| artifact.sha256.as_str()),
    );
    let manifest_source_hash = combined_hash(
        manifest
            .source_artifacts
            .iter()
            .map(|artifact| artifact.sha256.as_str()),
    );
    let mut stale_reasons = Vec::new();
    if manifest.version != LINUX_CHAT_CORE_VERSION {
        stale_reasons.push("compiler_version_changed".to_string());
    }
    if current_source_hash != manifest_source_hash {
        stale_reasons.push("source_memory_hash_changed".to_string());
    }
    if manifest.cache_is_source_of_truth {
        stale_reasons.push("manifest_claims_cache_as_source_of_truth".to_string());
    }
    let hot_path = PathBuf::from(&manifest.hot_path);
    let index_path = PathBuf::from(&manifest.index_path);
    let hot_present = hot_path.exists();
    let index_present = index_path.exists();
    if !hot_present {
        stale_reasons.push("hot_cache_missing".to_string());
    } else if hash_file(&hot_path)? != manifest.hot_sha256 {
        stale_reasons.push("hot_cache_hash_changed".to_string());
    }
    Ok(LinuxChatCoreCacheStatus {
        manifest_present: true,
        hot_present,
        index_present,
        cache_fresh: stale_reasons.is_empty(),
        stale_reasons,
        current_source_hash,
        manifest_source_hash,
        cache_is_source_of_truth: false,
    })
}

fn evaluate_cache_from_manifest_for_ask(
    manifest_path: &Path,
    config: &LinuxChatCoreAskConfig,
) -> Result<LinuxChatCoreCacheStatus> {
    let mut status = evaluate_cache_from_manifest(manifest_path)?;
    if !manifest_path.exists() {
        return Ok(status);
    }
    let manifest: LinuxChatCoreCacheManifest = serde_json::from_slice(
        &fs::read(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse {}", manifest_path.display()))?;
    let overrides = [
        ("base-lrf", config.residual_pack.as_path()),
        ("dialogue", config.dialogue_overlay.as_path()),
        ("centers", config.centers_overlay.as_path()),
        ("vpn", config.vpn_overlay.as_path()),
    ];
    let mut ask_artifacts = Vec::new();
    let mut source_path_changed = false;
    for expected in &manifest.source_artifacts {
        if let Some((_, override_path)) = overrides
            .iter()
            .find(|(artifact_id, _)| *artifact_id == expected.artifact_id)
        {
            if path_string(override_path) != expected.path {
                source_path_changed = true;
            }
            ask_artifacts.push(artifact(
                &expected.artifact_id,
                &expected.kind,
                override_path,
                expected.required,
            )?);
        } else {
            ask_artifacts.push(artifact(
                &expected.artifact_id,
                &expected.kind,
                Path::new(&expected.path),
                expected.required,
            )?);
        }
    }
    let ask_source_hash = combined_hash(
        ask_artifacts
            .iter()
            .map(|artifact| artifact.sha256.as_str()),
    );
    status.current_source_hash = ask_source_hash;
    if status.current_source_hash != status.manifest_source_hash {
        push_unique_stale_reason(&mut status.stale_reasons, "source_memory_hash_changed");
    }
    if source_path_changed {
        push_unique_stale_reason(&mut status.stale_reasons, "source_memory_path_changed");
    }
    status.cache_fresh = status.stale_reasons.is_empty();
    Ok(status)
}

fn push_unique_stale_reason(reasons: &mut Vec<String>, reason: &str) {
    if !reasons.iter().any(|existing| existing == reason) {
        reasons.push(reason.to_string());
    }
}

fn source_artifacts(spec: &LinuxChatCoreSpec) -> Result<Vec<LinuxChatCoreArtifactDigest>> {
    let mut artifacts = vec![artifact(
        "base-lrf",
        "residual-pack",
        Path::new(&spec.source_memory.residual_pack),
        true,
    )?];
    for overlay in &spec.overlays {
        artifacts.push(artifact(
            &overlay.overlay_id,
            &overlay.overlay_kind,
            Path::new(&overlay.path),
            overlay.required_for_profile_ready,
        )?);
    }
    for eval in &spec.evals {
        if let Some(path) = &eval.path {
            artifacts.push(artifact(
                &eval.eval_id,
                "eval-artifact",
                Path::new(path),
                eval.required_for_chat_core_gate,
            )?);
        }
    }
    Ok(artifacts)
}

fn load_overlay_facts(
    overlays: &[LinuxChatCoreOverlaySpec],
) -> Result<Vec<LinuxResidualDecodedFact>> {
    let mut facts = Vec::new();
    for overlay in overlays {
        let path = PathBuf::from(&overlay.path);
        let Ok(memory) = load_memory(&path) else {
            continue;
        };
        for record in memory.records {
            if let Some(fact) = overlay_record_to_fact(&record) {
                facts.push(fact);
            }
        }
    }
    Ok(facts)
}

fn overlay_record_to_fact(record: &PersistentWaveDeltaRecord) -> Option<LinuxResidualDecodedFact> {
    let negative = record.delta_state == DELTA_NEGATIVE || record.polarity == "negative";
    if record.delta_state != DELTA_POSITIVE && !negative {
        return None;
    }
    let route = normalize_hot_route_from_record(record);
    let (default_relation, subject_role, object_role) = role_contract_for_route(&route, negative);
    let relation = if negative {
        "does not prove".to_string()
    } else if record.relation.trim().is_empty() {
        default_relation
    } else {
        normalize_relation_for_route(&route, &record.relation)
    };
    Some(LinuxResidualDecodedFact {
        route,
        subject: record.subject.clone(),
        subject_role,
        relation,
        object: record.object.clone(),
        object_role,
        polarity: if negative { "negative" } else { "positive" },
        evidence_kind: "chatcore_overlay".to_string(),
        confidence: if negative { 92 } else { 96 },
        memory_kind: if negative {
            "learned_anti_wave"
        } else {
            "learned_overlay"
        },
    })
}

fn normalize_hot_route_from_record(record: &PersistentWaveDeltaRecord) -> String {
    match record.route.as_str() {
        "linux.apt.command.package-command" if record.relation == "provided by package" => {
            "linux.apt.command.provider".to_string()
        }
        route => route.to_string(),
    }
}

fn normalize_relation_for_route(route: &str, relation: &str) -> String {
    match route {
        "linux.apt.command.provider" => "provided by package".to_string(),
        "linux.apt.command.package-command" => "provides command".to_string(),
        "linux.package.binary" => "provides binary".to_string(),
        _ => relation.to_string(),
    }
}

fn route_index(
    facts: &[super::linux_residual_memory::LinuxResidualDecodedFact],
) -> Vec<LinuxChatCoreRouteIndexEntry> {
    #[derive(Default)]
    struct Acc {
        fact_count: usize,
        relations: BTreeSet<String>,
        memory_kinds: BTreeSet<String>,
        polarities: BTreeSet<String>,
    }
    let mut by_route: BTreeMap<String, Acc> = BTreeMap::new();
    for fact in facts {
        let acc = by_route.entry(fact.route.clone()).or_default();
        acc.fact_count += 1;
        acc.relations.insert(fact.relation.clone());
        acc.memory_kinds.insert(fact.memory_kind.to_string());
        acc.polarities.insert(fact.polarity.to_string());
    }
    by_route
        .into_iter()
        .map(|(route, acc)| LinuxChatCoreRouteIndexEntry {
            route,
            fact_count: acc.fact_count,
            relations: acc.relations.into_iter().take(16).collect(),
            memory_kinds: acc.memory_kinds.into_iter().collect(),
            polarities: acc.polarities.into_iter().collect(),
        })
        .collect()
}

impl LinuxChatCoreFactPreview {
    fn from_decoded_fact(fact: &LinuxResidualDecodedFact) -> Self {
        Self {
            route: fact.route.clone(),
            subject: fact.subject.clone(),
            subject_role: fact.subject_role.clone(),
            relation: fact.relation.clone(),
            object: fact.object.clone(),
            object_role: fact.object_role.clone(),
            polarity: fact.polarity.to_string(),
            evidence_kind: fact.evidence_kind.clone(),
            confidence: fact.confidence,
            memory_kind: fact.memory_kind.to_string(),
        }
    }

    fn to_decoded_fact(&self) -> LinuxResidualDecodedFact {
        LinuxResidualDecodedFact {
            route: self.route.clone(),
            subject: self.subject.clone(),
            subject_role: self.subject_role.clone(),
            relation: self.relation.clone(),
            object: self.object.clone(),
            object_role: self.object_role.clone(),
            polarity: stable_polarity(&self.polarity),
            evidence_kind: self.evidence_kind.clone(),
            confidence: self.confidence,
            memory_kind: stable_memory_kind(&self.memory_kind),
        }
    }
}

fn stable_polarity(value: &str) -> &'static str {
    match value {
        "negative" => "negative",
        "positive" => "positive",
        _ => "unknown",
    }
}

fn stable_memory_kind(value: &str) -> &'static str {
    match value {
        "fallback" => "fallback",
        "learned_anti_wave" => "learned_anti_wave",
        "learned_overlay" => "learned_overlay",
        "residual" => "residual",
        "schema" => "schema",
        _ => "unknown",
    }
}

fn encode_hot_cache(cache: &LinuxChatCoreHotCache) -> Result<LinuxChatCoreEncodedHotCache> {
    let mut interner = HotStringInterner::default();
    collect_hot_strings(cache, &mut interner)?;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(HOT_MAGIC);
    write_u32(&mut bytes, HOT_FORMAT_VERSION);
    let intern_table_start = bytes.len();
    write_u64(&mut bytes, interner.values.len() as u64);
    for value in &interner.values {
        write_string(&mut bytes, value)?;
    }
    let intern_table_bytes = (bytes.len() - intern_table_start) as u64;

    encode_summary(&mut bytes, &cache.residual_summary, &interner)?;
    write_u64(&mut bytes, cache.route_index.len() as u64);
    for route in &cache.route_index {
        write_string_id(&mut bytes, &interner, &route.route)?;
        write_u64(&mut bytes, route.fact_count as u64);
        write_string_id_vec(&mut bytes, &interner, &route.relations)?;
        write_string_id_vec(&mut bytes, &interner, &route.memory_kinds)?;
        write_string_id_vec(&mut bytes, &interner, &route.polarities)?;
    }
    write_u64(&mut bytes, cache.domains.len() as u64);
    for domain in &cache.domains {
        write_string_id(&mut bytes, &interner, &domain.domain_id)?;
        write_string_id_vec(&mut bytes, &interner, &domain.routes)?;
        write_string_id_vec(&mut bytes, &interner, &domain.negative_routes)?;
        write_string_id_vec(&mut bytes, &interner, &domain.overlay_ids)?;
        write_string_id(&mut bytes, &interner, &domain.action_scope)?;
    }
    write_u64(&mut bytes, cache.readout_facts.len() as u64);
    for fact in &cache.readout_facts {
        write_string_id(&mut bytes, &interner, &fact.route)?;
        write_string_id(&mut bytes, &interner, &fact.subject)?;
        write_string_id(&mut bytes, &interner, &fact.subject_role)?;
        write_string_id(&mut bytes, &interner, &fact.relation)?;
        write_string_id(&mut bytes, &interner, &fact.object)?;
        write_string_id(&mut bytes, &interner, &fact.object_role)?;
        write_string_id(&mut bytes, &interner, &fact.polarity)?;
        write_string_id(&mut bytes, &interner, &fact.evidence_kind)?;
        write_u8(&mut bytes, fact.confidence);
        write_string_id(&mut bytes, &interner, &fact.memory_kind)?;
    }
    let hot_bytes = bytes.len() as u64;
    let packed_fact_record_count = cache.readout_facts.len();
    let packed_fact_record_bytes = packed_fact_record_count as u64 * PACKED_FACT_RECORD_BYTES;
    let stats = LinuxChatCoreHotStorageStats {
        hot_format: HOT_FORMAT,
        hot_target_bytes: HOT_TARGET_BYTES,
        hot_fits_6m_budget: hot_bytes <= HOT_TARGET_BYTES,
        hot_bytes,
        interned_string_count: interner.values.len(),
        intern_table_bytes,
        packed_fact_record_count,
        packed_fact_record_bytes,
        bytes_per_fact: ratio_f32(hot_bytes, packed_fact_record_count as u64),
        bytes_per_answerable_fact: ratio_f32(hot_bytes, packed_fact_record_count as u64),
    };
    Ok(LinuxChatCoreEncodedHotCache { bytes, stats })
}

fn load_hot_cache_from_manifest(manifest_path: &Path) -> Result<LinuxChatCoreHotCache> {
    let manifest: LinuxChatCoreCacheManifest = serde_json::from_slice(
        &fs::read(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse {}", manifest_path.display()))?;
    let hot_path = PathBuf::from(&manifest.hot_path);
    let bytes = fs::read(&hot_path).with_context(|| format!("read {}", hot_path.display()))?;
    if hash_bytes(&bytes) != manifest.hot_sha256 {
        anyhow::bail!("chat-core hot cache hash mismatch: {}", hot_path.display());
    }
    decode_hot_cache(&bytes).with_context(|| format!("decode {}", hot_path.display()))
}

fn decode_hot_cache(bytes: &[u8]) -> Result<LinuxChatCoreHotCache> {
    let mut pos = 0usize;
    let magic = read_exact(bytes, &mut pos, HOT_MAGIC.len())?;
    if magic != HOT_MAGIC {
        anyhow::bail!("invalid chat-core hot cache magic");
    }
    let version = read_u32(bytes, &mut pos)?;
    if version != HOT_FORMAT_VERSION {
        anyhow::bail!("unsupported chat-core hot cache format: {version}");
    }
    let string_count = read_len(bytes, &mut pos)?;
    let mut strings = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        strings.push(read_string(bytes, &mut pos)?);
    }

    let residual_summary = decode_summary(bytes, &mut pos, &strings)?;
    let route_count = read_len(bytes, &mut pos)?;
    let mut route_index = Vec::with_capacity(route_count);
    for _ in 0..route_count {
        route_index.push(LinuxChatCoreRouteIndexEntry {
            route: read_string_id(bytes, &mut pos, &strings)?,
            fact_count: read_usize(bytes, &mut pos)?,
            relations: read_string_id_vec(bytes, &mut pos, &strings)?,
            memory_kinds: read_string_id_vec(bytes, &mut pos, &strings)?,
            polarities: read_string_id_vec(bytes, &mut pos, &strings)?,
        });
    }
    let domain_count = read_len(bytes, &mut pos)?;
    let mut domains = Vec::with_capacity(domain_count);
    for _ in 0..domain_count {
        domains.push(LinuxChatCoreDomainSpec {
            domain_id: read_string_id(bytes, &mut pos, &strings)?,
            routes: read_string_id_vec(bytes, &mut pos, &strings)?,
            negative_routes: read_string_id_vec(bytes, &mut pos, &strings)?,
            overlay_ids: read_string_id_vec(bytes, &mut pos, &strings)?,
            action_scope: read_string_id(bytes, &mut pos, &strings)?,
        });
    }
    let fact_count = read_len(bytes, &mut pos)?;
    let mut readout_facts = Vec::with_capacity(fact_count);
    for _ in 0..fact_count {
        readout_facts.push(LinuxChatCoreFactPreview {
            route: read_string_id(bytes, &mut pos, &strings)?,
            subject: read_string_id(bytes, &mut pos, &strings)?,
            subject_role: read_string_id(bytes, &mut pos, &strings)?,
            relation: read_string_id(bytes, &mut pos, &strings)?,
            object: read_string_id(bytes, &mut pos, &strings)?,
            object_role: read_string_id(bytes, &mut pos, &strings)?,
            polarity: read_string_id(bytes, &mut pos, &strings)?,
            evidence_kind: read_string_id(bytes, &mut pos, &strings)?,
            confidence: read_u8(bytes, &mut pos)?,
            memory_kind: read_string_id(bytes, &mut pos, &strings)?,
        });
    }
    if pos != bytes.len() {
        anyhow::bail!("chat-core hot cache has trailing bytes");
    }
    Ok(LinuxChatCoreHotCache {
        residual_summary,
        route_index,
        readout_facts,
        domains,
    })
}

#[derive(Default)]
struct HotStringInterner {
    ids: BTreeMap<String, u32>,
    values: Vec<String>,
}

impl HotStringInterner {
    fn intern(&mut self, value: &str) -> Result<u32> {
        if let Some(id) = self.ids.get(value) {
            return Ok(*id);
        }
        let id: u32 = self.values.len().try_into()?;
        self.values.push(value.to_string());
        self.ids.insert(value.to_string(), id);
        Ok(id)
    }

    fn id(&self, value: &str) -> Result<u32> {
        self.ids
            .get(value)
            .copied()
            .with_context(|| format!("chat-core hot string was not interned: {value}"))
    }
}

fn collect_hot_strings(
    cache: &LinuxChatCoreHotCache,
    interner: &mut HotStringInterner,
) -> Result<()> {
    interner.intern(&cache.residual_summary.path)?;
    for route in &cache.route_index {
        interner.intern(&route.route)?;
        for value in route
            .relations
            .iter()
            .chain(route.memory_kinds.iter())
            .chain(route.polarities.iter())
        {
            interner.intern(value)?;
        }
    }
    for domain in &cache.domains {
        interner.intern(&domain.domain_id)?;
        for value in domain
            .routes
            .iter()
            .chain(domain.negative_routes.iter())
            .chain(domain.overlay_ids.iter())
        {
            interner.intern(value)?;
        }
        interner.intern(&domain.action_scope)?;
    }
    for fact in &cache.readout_facts {
        interner.intern(&fact.route)?;
        interner.intern(&fact.subject)?;
        interner.intern(&fact.subject_role)?;
        interner.intern(&fact.relation)?;
        interner.intern(&fact.object)?;
        interner.intern(&fact.object_role)?;
        interner.intern(&fact.polarity)?;
        interner.intern(&fact.evidence_kind)?;
        interner.intern(&fact.memory_kind)?;
    }
    Ok(())
}

fn encode_summary(
    bytes: &mut Vec<u8>,
    summary: &LinuxResidualDecodedSummary,
    interner: &HotStringInterner,
) -> Result<()> {
    write_string_id(bytes, interner, &summary.path)?;
    write_u64(bytes, summary.file_bytes as u64);
    write_u32(bytes, summary.wave_dim);
    write_u64(bytes, summary.represented_fact_count as u64);
    write_u64(bytes, summary.schema_record_count as u64);
    write_u64(bytes, summary.residual_record_count as u64);
    write_u64(bytes, summary.fallback_record_count as u64);
    write_u64(bytes, summary.route_count as u64);
    write_u64(bytes, summary.corpus_hash64);
    write_u64(bytes, summary.promotion_threshold as u64);
    write_u64(bytes, summary.binary_hot_sections_bytes as u64);
    write_u64(bytes, summary.direct_fixed_baseline_bytes as u64);
    write_u64(bytes, summary.cold_label_count as u64);
    write_u64(bytes, summary.cold_label_table_bytes as u64);
    write_bool(bytes, summary.binary_hot_sections_fit_6m);
    write_bool(bytes, summary.beats_direct_fixed64);
    Ok(())
}

fn decode_summary(
    bytes: &[u8],
    pos: &mut usize,
    strings: &[String],
) -> Result<LinuxResidualDecodedSummary> {
    Ok(LinuxResidualDecodedSummary {
        path: read_string_id(bytes, pos, strings)?,
        file_bytes: read_usize(bytes, pos)?,
        wave_dim: read_u32(bytes, pos)?,
        represented_fact_count: read_usize(bytes, pos)?,
        schema_record_count: read_usize(bytes, pos)?,
        residual_record_count: read_usize(bytes, pos)?,
        fallback_record_count: read_usize(bytes, pos)?,
        route_count: read_usize(bytes, pos)?,
        corpus_hash64: read_u64(bytes, pos)?,
        promotion_threshold: read_usize(bytes, pos)?,
        binary_hot_sections_bytes: read_usize(bytes, pos)?,
        direct_fixed_baseline_bytes: read_usize(bytes, pos)?,
        cold_label_count: read_usize(bytes, pos)?,
        cold_label_table_bytes: read_usize(bytes, pos)?,
        binary_hot_sections_fit_6m: read_bool(bytes, pos)?,
        beats_direct_fixed64: read_bool(bytes, pos)?,
    })
}

fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

fn write_bool(bytes: &mut Vec<u8>, value: bool) {
    write_u8(bytes, u8::from(value));
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_string(bytes: &mut Vec<u8>, value: &str) -> Result<()> {
    write_u32(bytes, value.len().try_into()?);
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn write_string_id(bytes: &mut Vec<u8>, interner: &HotStringInterner, value: &str) -> Result<()> {
    write_u32(bytes, interner.id(value)?);
    Ok(())
}

fn write_string_id_vec(
    bytes: &mut Vec<u8>,
    interner: &HotStringInterner,
    values: &[String],
) -> Result<()> {
    write_u64(bytes, values.len() as u64);
    for value in values {
        write_string_id(bytes, interner, value)?;
    }
    Ok(())
}

fn read_exact<'a>(bytes: &'a [u8], pos: &mut usize, len: usize) -> Result<&'a [u8]> {
    let end = pos
        .checked_add(len)
        .context("chat-core hot cache offset overflow")?;
    let slice = bytes
        .get(*pos..end)
        .context("truncated chat-core hot cache")?;
    *pos = end;
    Ok(slice)
}

fn read_u8(bytes: &[u8], pos: &mut usize) -> Result<u8> {
    Ok(*read_exact(bytes, pos, 1)?
        .first()
        .context("truncated chat-core u8")?)
}

fn read_bool(bytes: &[u8], pos: &mut usize) -> Result<bool> {
    match read_u8(bytes, pos)? {
        0 => Ok(false),
        1 => Ok(true),
        value => anyhow::bail!("invalid chat-core bool: {value}"),
    }
}

fn read_u32(bytes: &[u8], pos: &mut usize) -> Result<u32> {
    let mut raw = [0u8; 4];
    raw.copy_from_slice(read_exact(bytes, pos, 4)?);
    Ok(u32::from_le_bytes(raw))
}

fn read_u64(bytes: &[u8], pos: &mut usize) -> Result<u64> {
    let mut raw = [0u8; 8];
    raw.copy_from_slice(read_exact(bytes, pos, 8)?);
    Ok(u64::from_le_bytes(raw))
}

fn read_usize(bytes: &[u8], pos: &mut usize) -> Result<usize> {
    Ok(read_u64(bytes, pos)?.try_into()?)
}

fn read_len(bytes: &[u8], pos: &mut usize) -> Result<usize> {
    read_usize(bytes, pos)
}

fn read_string(bytes: &[u8], pos: &mut usize) -> Result<String> {
    let len = read_u32(bytes, pos)? as usize;
    let raw = read_exact(bytes, pos, len)?;
    Ok(std::str::from_utf8(raw)
        .context("chat-core hot cache string is not UTF-8")?
        .to_string())
}

fn read_string_id(bytes: &[u8], pos: &mut usize, strings: &[String]) -> Result<String> {
    let id = read_u32(bytes, pos)? as usize;
    strings
        .get(id)
        .cloned()
        .with_context(|| format!("chat-core hot string id out of range: {id}"))
}

fn read_string_id_vec(bytes: &[u8], pos: &mut usize, strings: &[String]) -> Result<Vec<String>> {
    let len = read_len(bytes, pos)?;
    let mut values = Vec::with_capacity(len);
    for _ in 0..len {
        values.push(read_string_id(bytes, pos, strings)?);
    }
    Ok(values)
}

fn source_status(artifacts: &[LinuxChatCoreArtifactDigest]) -> LinuxChatCoreSourceStatus {
    let source_of_truth = artifacts
        .iter()
        .filter(|artifact| artifact.kind == "residual-pack" || artifact.kind.contains("learning"))
        .map(|artifact| artifact.path.clone())
        .collect();
    let overlays_present = artifacts
        .iter()
        .filter(|artifact| artifact.kind.contains("learning") && artifact.present)
        .count();
    let overlays_missing = artifacts
        .iter()
        .filter(|artifact| artifact.kind.contains("learning") && !artifact.present)
        .count();
    let eval_artifacts_present = artifacts
        .iter()
        .filter(|artifact| artifact.kind == "eval-artifact" && artifact.present)
        .count();
    LinuxChatCoreSourceStatus {
        source_memory_loaded: artifacts
            .iter()
            .any(|artifact| artifact.kind == "residual-pack" && artifact.present),
        source_of_truth,
        overlays_present,
        overlays_missing,
        required_missing: artifacts
            .iter()
            .filter(|artifact| artifact.required && !artifact.present)
            .map(|artifact| artifact.path.clone())
            .collect(),
        eval_artifacts_present,
        source_hash: combined_hash(artifacts.iter().map(|artifact| artifact.sha256.as_str())),
    }
}

fn cache_token_economics(
    artifacts: &[LinuxChatCoreArtifactDigest],
    manifest: &LinuxChatCoreCacheManifest,
) -> LinuxChatCoreCacheTokenEconomics {
    let source_artifacts_bytes = artifacts.iter().map(|artifact| artifact.bytes).sum::<u64>();
    let cache_total_bytes = manifest.hot_bytes + manifest.index_bytes;
    LinuxChatCoreCacheTokenEconomics {
        estimate_method: "ceil(bytes / 4), conservative tokenizer-free estimate",
        source_artifacts_bytes,
        source_artifacts_estimated_tokens: estimate_tokens_from_bytes(source_artifacts_bytes),
        cache_hot_bytes: manifest.hot_bytes,
        cache_index_bytes: manifest.index_bytes,
        cache_total_bytes,
        cache_estimated_tokens: estimate_tokens_from_bytes(cache_total_bytes),
        cache_vs_source_bytes_ratio: ratio_f32(cache_total_bytes, source_artifacts_bytes),
        cache_is_runtime_index_not_prompt_payload: true,
        note: "This estimates prompt/context scale only. The binary hot cache is a runtime readout, and the JSON index is debug/explain only; neither is a prompt payload. It is not a model-specific BPE tokenizer count.",
    }
}

fn cache_token_economics_from_manifest_path(
    artifacts: &[LinuxChatCoreArtifactDigest],
    manifest_path: &Path,
) -> Result<LinuxChatCoreCacheTokenEconomics> {
    if manifest_path.exists() {
        let manifest: LinuxChatCoreCacheManifest = serde_json::from_slice(
            &fs::read(manifest_path)
                .with_context(|| format!("read {}", manifest_path.display()))?,
        )
        .with_context(|| format!("parse {}", manifest_path.display()))?;
        Ok(cache_token_economics(artifacts, &manifest))
    } else {
        let source_artifacts_bytes = artifacts.iter().map(|artifact| artifact.bytes).sum::<u64>();
        Ok(LinuxChatCoreCacheTokenEconomics {
            estimate_method: "ceil(bytes / 4), conservative tokenizer-free estimate",
            source_artifacts_bytes,
            source_artifacts_estimated_tokens: estimate_tokens_from_bytes(source_artifacts_bytes),
            cache_hot_bytes: 0,
            cache_index_bytes: 0,
            cache_total_bytes: 0,
            cache_estimated_tokens: 0,
            cache_vs_source_bytes_ratio: 0.0,
            cache_is_runtime_index_not_prompt_payload: true,
            note:
                "Cache manifest is missing; gate is read-only and does not build cache implicitly.",
        })
    }
}

fn ask_token_economics(
    manifest_path: &Path,
    packet: &LinuxChatCoreGroundedPacket,
) -> Result<LinuxChatCoreAskTokenEconomics> {
    let packet_bytes = serde_json::to_vec(packet)?.len() as u64;
    if !manifest_path.exists() {
        return Ok(LinuxChatCoreAskTokenEconomics {
            estimate_method: "ceil(bytes / 4), conservative tokenizer-free estimate",
            source_artifacts_size_bytes: 0,
            source_artifacts_bytes: 0,
            source_artifacts_estimated_tokens: 0,
            full_cache_index_size_bytes: 0,
            cache_index_bytes: 0,
            cache_index_estimated_tokens: 0,
            grounded_packet_size_bytes: packet_bytes,
            grounded_packet_bytes: packet_bytes,
            grounded_packet_estimated_tokens: estimate_tokens_from_bytes(packet_bytes),
            actual_answer_context_bytes: packet_bytes,
            actual_answer_context_estimated_tokens: estimate_tokens_from_bytes(packet_bytes),
            estimated_tokens_saved_vs_source: 0,
            estimated_tokens_saved_vs_cache_index: 0,
            source_to_packet_reduction_ratio: 0.0,
            cache_index_to_packet_reduction_ratio: 0.0,
            cache_is_runtime_index_not_prompt_payload: true,
            note: "Cache manifest is missing; savings cannot be estimated from source artifacts.",
        });
    }
    let manifest: LinuxChatCoreCacheManifest = serde_json::from_slice(
        &fs::read(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse {}", manifest_path.display()))?;
    let source_artifacts_bytes = manifest
        .source_artifacts
        .iter()
        .map(|artifact| artifact.bytes)
        .sum::<u64>();
    let source_tokens = estimate_tokens_from_bytes(source_artifacts_bytes);
    let cache_index_tokens = estimate_tokens_from_bytes(manifest.index_bytes);
    let packet_tokens = estimate_tokens_from_bytes(packet_bytes);
    Ok(LinuxChatCoreAskTokenEconomics {
        estimate_method: "ceil(bytes / 4), conservative tokenizer-free estimate",
        source_artifacts_size_bytes: source_artifacts_bytes,
        source_artifacts_bytes,
        source_artifacts_estimated_tokens: source_tokens,
        full_cache_index_size_bytes: manifest.index_bytes,
        cache_index_bytes: manifest.index_bytes,
        cache_index_estimated_tokens: cache_index_tokens,
        grounded_packet_size_bytes: packet_bytes,
        grounded_packet_bytes: packet_bytes,
        grounded_packet_estimated_tokens: packet_tokens,
        actual_answer_context_bytes: packet_bytes,
        actual_answer_context_estimated_tokens: packet_tokens,
        estimated_tokens_saved_vs_source: source_tokens.saturating_sub(packet_tokens),
        estimated_tokens_saved_vs_cache_index: cache_index_tokens.saturating_sub(packet_tokens),
        source_to_packet_reduction_ratio: ratio_f32(source_artifacts_bytes, packet_bytes),
        cache_index_to_packet_reduction_ratio: ratio_f32(manifest.index_bytes, packet_bytes),
        cache_is_runtime_index_not_prompt_payload: true,
        note: "Savings estimate compares sending the full source/cache-index context to sending only the grounded packet. The cache index is a runtime readout, not a prompt payload. This is not a model-specific BPE tokenizer count.",
    })
}

fn estimate_tokens_from_bytes(bytes: u64) -> u64 {
    bytes.div_ceil(4)
}

fn ratio_f32(numerator: u64, denominator: u64) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64 / denominator as f64) as f32
    }
}

fn artifact(
    artifact_id: &str,
    kind: &str,
    path: &Path,
    required: bool,
) -> Result<LinuxChatCoreArtifactDigest> {
    let present = path.exists();
    let (bytes, sha256) = if present {
        let metadata =
            fs::metadata(path).with_context(|| format!("read metadata {}", path.display()))?;
        (metadata.len(), hash_file(path)?)
    } else if required {
        anyhow::bail!("required ChatCore artifact missing: {}", path.display());
    } else {
        (0, "missing".to_string())
    };
    Ok(LinuxChatCoreArtifactDigest {
        artifact_id: artifact_id.to_string(),
        kind: kind.to_string(),
        path: path_string(path),
        required,
        present,
        bytes,
        sha256,
    })
}

fn claim_boundary(
    cache_ready: bool,
    gate_checked: bool,
    profile_ready: bool,
) -> LinuxChatCoreClaimBoundary {
    LinuxChatCoreClaimBoundary {
            linux_chat_core_cache_ready: cache_ready,
            compiled_cache_matches_source_memory: cache_ready && gate_checked,
            cache_is_source_of_truth: false,
            cache_is_runtime_index_not_prompt_payload: true,
            source_memory_required_for_authority: true,
            stale_cache_blocks_answer_authority: true,
        profile_scoped_chat_core_ready: profile_ready,
        general_llm_ready: false,
        global_nonlinear_memory_proven: false,
        safe_claim: "Linux ChatCore is a profile-scoped compiled cache over `.lrf` + `.lwm` source memory. The cache is not the source of truth and cannot grant answer authority when stale.",
        blocked_claims: vec!["general_llm_ready", "global_nonlinear_memory_proven"],
    }
}

fn manifest_path(config: &LinuxChatCoreGateConfig) -> PathBuf {
    config
        .manifest
        .clone()
        .unwrap_or_else(|| config.cache_dir.join("chat-core.manifest.json"))
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn write_json_if_requested<T: Serialize>(out: Option<&Path>, report: &T) -> Result<()> {
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(path, serde_json::to_string_pretty(report)?)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex(&digest)
}

fn hash_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(hash_bytes(&serde_json::to_vec(value)?))
}

fn combined_hash<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update(b"\n");
    }
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "nanda-chat-core-{name}-{}-{}",
            std::process::id(),
            now_unix_seconds()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_chat_core_spec(dir: &Path, residual_pack: &Path) -> LinuxChatCoreSpec {
        let none: Option<PathBuf> = None;
        load_chat_core_spec(
            Path::new(DEFAULT_LINUX_CHAT_CORE_PROFILE),
            &ChatCoreSpecOverrides {
                residual_pack,
                dialogue_overlay: &dir.join("dialogue.lwm"),
                centers_overlay: &dir.join("centers.lwm"),
                vpn_overlay: &dir.join("vpn.lwm"),
                broad_eval: &none,
                heldout_eval: &none,
                cache_dir: &dir.join("cache"),
            },
        )
        .unwrap()
    }

    #[test]
    fn chat_core_compact_evidence_keeps_subject_and_object() {
        let step = LinuxEvidenceStep {
            role: "provider".to_string(),
            route: "linux.apt.command.package-command".to_string(),
            subject: "bash".to_string(),
            relation: "provides command".to_string(),
            object: "bash".to_string(),
            polarity: "positive".to_string(),
            memory_kind: "residual".to_string(),
            confidence: 99,
        };
        let compact = compact_evidence_step(&step);
        assert!(compact.contains("subject=bash"));
        assert!(compact.contains("object=bash"));
        assert!(compact.contains("role=provider"));
        assert!(compact.contains("confidence=99"));
        let structured = LinuxChatCoreGroundedEvidence::from(&step);
        assert_eq!(structured.route, "linux.apt.command.package-command");
        assert_eq!(structured.role, "provider");
        assert_eq!(structured.subject, "bash");
        assert_eq!(structured.object, "bash");
        assert_eq!(structured.memory_kind, "residual");
    }

    #[test]
    fn chat_core_hot_cache_roundtrip_contains_binary_readout() {
        let cache = LinuxChatCoreHotCache {
            residual_summary: LinuxResidualDecodedSummary {
                path: "unit.lrf".to_string(),
                file_bytes: 128,
                wave_dim: 1024,
                represented_fact_count: 1,
                schema_record_count: 1,
                residual_record_count: 1,
                fallback_record_count: 0,
                route_count: 1,
                corpus_hash64: 42,
                promotion_threshold: 2,
                binary_hot_sections_bytes: 64,
                direct_fixed_baseline_bytes: 64,
                cold_label_count: 3,
                cold_label_table_bytes: 96,
                binary_hot_sections_fit_6m: true,
                beats_direct_fixed64: true,
            },
            route_index: vec![LinuxChatCoreRouteIndexEntry {
                route: "linux.apt.command.package-command".to_string(),
                fact_count: 1,
                relations: vec!["provides command".to_string()],
                memory_kinds: vec!["residual".to_string()],
                polarities: vec!["positive".to_string()],
            }],
            readout_facts: vec![LinuxChatCoreFactPreview {
                route: "linux.apt.command.package-command".to_string(),
                subject: "bash".to_string(),
                subject_role: "package".to_string(),
                relation: "provides command".to_string(),
                object: "bash".to_string(),
                object_role: "command".to_string(),
                polarity: "positive".to_string(),
                evidence_kind: "package_metadata".to_string(),
                confidence: 82,
                memory_kind: "residual".to_string(),
            }],
            domains: vec![LinuxChatCoreDomainSpec {
                domain_id: "packages".to_string(),
                routes: vec!["linux.apt.command.package-command".to_string()],
                negative_routes: vec![],
                overlay_ids: vec!["dialogue".to_string()],
                action_scope: "read_only_answer_support".to_string(),
            }],
        };
        let encoded = encode_hot_cache(&cache).unwrap();
        assert_eq!(encoded.stats.hot_format, HOT_FORMAT);
        assert_eq!(encoded.stats.packed_fact_record_count, 1);
        assert_eq!(
            encoded.stats.packed_fact_record_bytes,
            PACKED_FACT_RECORD_BYTES
        );
        let bytes = encoded.bytes;
        assert!(bytes.starts_with(HOT_MAGIC));
        assert!(bytes.len() > 128);
        let decoded = decode_hot_cache(&bytes).unwrap();
        assert_eq!(decoded.readout_facts.len(), 1);
        assert_eq!(decoded.route_index.len(), 1);
        assert_eq!(decoded.domains.len(), 1);
        assert_eq!(decoded.readout_facts[0].subject, "bash");
        assert_eq!(decoded.readout_facts[0].object, "bash");
    }

    #[test]
    fn chat_core_command_provider_evidence_matches_exact_anchor() {
        let bash = LinuxEvidenceStep {
            role: "provider".to_string(),
            route: "linux.apt.command.package-command".to_string(),
            subject: "bash".to_string(),
            relation: "provides command".to_string(),
            object: "bash".to_string(),
            polarity: "positive".to_string(),
            memory_kind: "residual".to_string(),
            confidence: 82,
        };
        let bashbug = LinuxEvidenceStep {
            object: "bashbug".to_string(),
            ..bash.clone()
        };
        let systemctl = LinuxEvidenceStep {
            role: "provider".to_string(),
            route: "linux.package.binary".to_string(),
            subject: "systemd".to_string(),
            relation: "provides binary".to_string(),
            object: "/usr/bin/systemctl".to_string(),
            polarity: "positive".to_string(),
            memory_kind: "residual".to_string(),
            confidence: 82,
        };
        assert!(command_provider_step_matches_anchor(&bash, "bash"));
        assert!(!command_provider_step_matches_anchor(&bashbug, "bash"));
        assert!(command_provider_step_matches_anchor(
            &systemctl,
            "systemctl"
        ));
    }

    #[test]
    fn chat_core_ask_blocks_every_source_override_mismatch() {
        for artifact_id in ["base-lrf", "dialogue", "centers", "vpn"] {
            assert_ask_blocks_source_override_mismatch(artifact_id);
        }
    }

    fn assert_ask_blocks_source_override_mismatch(mutated_artifact_id: &str) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "nanda-chat-core-ask-stale-{mutated_artifact_id}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let lrf = dir.join("base.lrf");
        let lrf_mutated = dir.join("base-mutated.lrf");
        let dialogue = dir.join("dialogue.lwm");
        let dialogue_mutated = dir.join("dialogue-mutated.lwm");
        let centers = dir.join("centers.lwm");
        let centers_mutated = dir.join("centers-mutated.lwm");
        let vpn = dir.join("vpn.lwm");
        let vpn_mutated = dir.join("vpn-mutated.lwm");
        fs::write(&lrf, b"fake-lrf").unwrap();
        fs::write(&lrf_mutated, b"fake-lrf changed").unwrap();
        fs::write(&dialogue, b"dialogue").unwrap();
        fs::write(&dialogue_mutated, b"dialogue changed").unwrap();
        fs::write(&centers, b"centers").unwrap();
        fs::write(&centers_mutated, b"centers changed").unwrap();
        fs::write(&vpn, b"vpn").unwrap();
        fs::write(&vpn_mutated, b"vpn changed").unwrap();
        let manifest = dir.join("chat-core.manifest.json");
        let hot = dir.join("chat-core.hot");
        let index = dir.join("chat-core.index.json");
        fs::write(&hot, b"hot").unwrap();
        fs::write(&index, b"index").unwrap();
        let artifacts = vec![
            artifact("base-lrf", "residual-pack", &lrf, true).unwrap(),
            artifact("dialogue", "dialogue-learning", &dialogue, false).unwrap(),
            artifact("centers", "dynamic-center-learning", &centers, false).unwrap(),
            artifact("vpn", "domain-learning", &vpn, false).unwrap(),
        ];
        let manifest_file = LinuxChatCoreCacheManifest {
            mode: "llmwave-big-linux-chat-core-cache-manifest".to_string(),
            version: LINUX_CHAT_CORE_VERSION.to_string(),
            profile_id: "linux-chat-core".to_string(),
            created_unix_seconds: now_unix_seconds(),
            compiler_version: LINUX_CHAT_CORE_VERSION.to_string(),
            spec_hash: "unit-test-spec".to_string(),
            source_artifacts: artifacts,
            domain_registry_hash: "unit-test-domains".to_string(),
            overlay_registry_hash: "unit-test-overlays".to_string(),
            hot_path: path_string(&hot),
            hot_bytes: 3,
            hot_sha256: hash_file(&hot).unwrap(),
            index_path: path_string(&index),
            index_bytes: 5,
            index_sha256: hash_file(&index).unwrap(),
            cache_is_source_of_truth: false,
        };
        fs::write(
            &manifest,
            serde_json::to_vec_pretty(&manifest_file).unwrap(),
        )
        .unwrap();
        let residual_pack = if mutated_artifact_id == "base-lrf" {
            lrf_mutated
        } else {
            lrf
        };
        let dialogue_overlay = if mutated_artifact_id == "dialogue" {
            dialogue_mutated
        } else {
            dialogue
        };
        let centers_overlay = if mutated_artifact_id == "centers" {
            centers_mutated
        } else {
            centers
        };
        let vpn_overlay = if mutated_artifact_id == "vpn" {
            vpn_mutated
        } else {
            vpn
        };
        let report = build_linux_chat_core_ask_report(LinuxChatCoreAskConfig {
            text: "which package provides command bash".to_string(),
            residual_pack,
            dialogue_overlay,
            centers_overlay,
            vpn_overlay,
            cache_dir: dir.clone(),
            manifest: Some(manifest),
            max_facts: 4,
            out: None,
        })
        .unwrap();
        assert_eq!(report.verdict, "LINUX_CHAT_CORE_CACHE_STALE");
        assert!(!report.cache_status.cache_fresh);
        assert!(!report.grounded_packet.answer_allowed);
        assert!(report
            .cache_status
            .stale_reasons
            .contains(&"source_memory_hash_changed".to_string()));
        assert!(report
            .cache_status
            .stale_reasons
            .contains(&"source_memory_path_changed".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn chat_core_cache_detects_stale_overlay() {
        let nonce = now_unix_seconds();
        let dir = std::env::temp_dir().join(format!(
            "nanda-chat-core-cache-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let lrf = dir.join("base.lrf");
        let dialogue = dir.join("dialogue.lwm");
        let centers = dir.join("centers.lwm");
        let vpn = dir.join("vpn.lwm");
        fs::write(&lrf, b"fake-lrf").unwrap();
        fs::write(&dialogue, b"dialogue").unwrap();
        fs::write(&centers, b"centers").unwrap();
        fs::write(&vpn, b"vpn").unwrap();
        let manifest = dir.join("chat-core.manifest.json");
        let hot = dir.join("chat-core.hot");
        let index = dir.join("chat-core.index.json");
        fs::write(&hot, b"hot").unwrap();
        fs::write(&index, b"index").unwrap();
        let artifacts = vec![
            artifact("base-lrf", "residual-pack", &lrf, true).unwrap(),
            artifact("dialogue", "dialogue-learning", &dialogue, false).unwrap(),
            artifact("centers", "dynamic-center-learning", &centers, false).unwrap(),
            artifact("vpn", "domain-learning", &vpn, false).unwrap(),
        ];
        let manifest_file = LinuxChatCoreCacheManifest {
            mode: "llmwave-big-linux-chat-core-cache-manifest".to_string(),
            version: LINUX_CHAT_CORE_VERSION.to_string(),
            profile_id: "linux-chat-core".to_string(),
            created_unix_seconds: now_unix_seconds(),
            compiler_version: LINUX_CHAT_CORE_VERSION.to_string(),
            spec_hash: "unit-test-spec".to_string(),
            source_artifacts: artifacts,
            domain_registry_hash: "unit-test-domains".to_string(),
            overlay_registry_hash: "unit-test-overlays".to_string(),
            hot_path: path_string(&hot),
            hot_bytes: 3,
            hot_sha256: hash_file(&hot).unwrap(),
            index_path: path_string(&index),
            index_bytes: 5,
            index_sha256: hash_file(&index).unwrap(),
            cache_is_source_of_truth: false,
        };
        fs::write(
            &manifest,
            serde_json::to_vec_pretty(&manifest_file).unwrap(),
        )
        .unwrap();
        let fresh = evaluate_cache_from_manifest(&manifest).unwrap();
        assert!(fresh.cache_fresh);
        fs::write(&centers, b"centers changed").unwrap();
        let stale = evaluate_cache_from_manifest(&manifest).unwrap();
        assert!(!stale.cache_fresh);
        assert!(stale
            .stale_reasons
            .contains(&"source_memory_hash_changed".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn chat_core_learn_accept_normalizes_command_provider() {
        let request = parse_chat_core_learn_request(
            Some("foocmd | linux.apt.command.package-command | foopkg"),
            None,
            None,
            None,
            "foocmd | linux.apt.command.package-command | foopkg".to_string(),
        )
        .unwrap();
        assert_eq!(request.delta_state, DELTA_POSITIVE);
        assert_eq!(request.route, "linux.apt.command.provider");
        assert_eq!(request.intent, "command_provider");
        assert_eq!(request.subject, "foocmd");
        assert_eq!(request.object, "foopkg");
        assert_eq!(request.relation, "provided by package");
    }

    #[test]
    fn chat_core_overlay_delta_projects_to_learned_fact() {
        let record = build_delta_record(PersistentWaveDeltaSpec {
            delta_state: DELTA_POSITIVE.to_string(),
            source_prompt: "learn accept".to_string(),
            intent: "command_provider".to_string(),
            route: "linux.apt.command.provider".to_string(),
            subject: "foocmd".to_string(),
            relation: "provided by package".to_string(),
            object: "foopkg".to_string(),
            polarity: "positive".to_string(),
            reason: "unit test".to_string(),
            strength: 24,
        });
        let fact = overlay_record_to_fact(&record).unwrap();
        assert_eq!(fact.route, "linux.apt.command.provider");
        assert_eq!(fact.subject_role, "command");
        assert_eq!(fact.object_role, "package");
        assert_eq!(fact.memory_kind, "learned_overlay");
    }

    #[test]
    fn chat_core_feedback_sanitizer_refuses_secret_like_input() {
        let safety =
            sanitize_feedback("foocmd | linux.apt.command.package-command | sk-live-secret-token");
        assert!(safety.secret_detected);
        assert!(safety.secret_refused);
        assert!(!safety
            .redacted_source_prompt
            .contains("sk-live-secret-token"));
        assert!(safety
            .redacted_source_prompt
            .starts_with("[REDACTED_SECRET_FEEDBACK sha256="));
    }

    #[test]
    fn chat_core_learning_admission_rejects_unknown_route() {
        let dir = test_temp_dir("admission-unknown");
        let lrf = dir.join("linux-active-65k.lrf");
        fs::write(&lrf, b"stub").unwrap();
        let spec = test_chat_core_spec(&dir, &lrf);
        let request = parse_chat_core_learn_request(
            Some("thing | linux.unknown.route | value"),
            None,
            None,
            Some("dialogue"),
            "thing | linux.unknown.route | value".to_string(),
        )
        .unwrap();
        let overlay = spec
            .overlays
            .iter()
            .find(|overlay| overlay.overlay_id == "dialogue")
            .unwrap();
        let admission = validate_learning_admission(&spec, &request, overlay);
        assert!(!admission.write_allowed);
        assert!(admission.unknown_route_rejected);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn chat_core_learning_admission_rejects_wrong_overlay() {
        let dir = test_temp_dir("admission-overlay");
        let lrf = dir.join("linux-active-65k.lrf");
        fs::write(&lrf, b"stub").unwrap();
        let spec = test_chat_core_spec(&dir, &lrf);
        let request = parse_chat_core_learn_request(
            None,
            Some("package_installed implies vpn_running"),
            Some("vpn"),
            Some("dialogue"),
            "package_installed implies vpn_running".to_string(),
        )
        .unwrap();
        let overlay = spec
            .overlays
            .iter()
            .find(|overlay| overlay.overlay_id == "dialogue")
            .unwrap();
        let admission = validate_learning_admission(&spec, &request, overlay);
        assert!(!admission.write_allowed);
        assert!(admission.foreign_overlay_rejected);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn chat_core_quarantine_delta_is_not_projected_to_hot_fact() {
        let record = build_delta_record(PersistentWaveDeltaSpec {
            delta_state: DELTA_WATCH_TRACE.to_string(),
            source_prompt: "quarantined conflict".to_string(),
            intent: "command_provider".to_string(),
            route: "linux.apt.command.provider".to_string(),
            subject: "foocmd".to_string(),
            relation: "candidate_quarantine".to_string(),
            object: "otherpkg".to_string(),
            polarity: "watch".to_string(),
            reason: "unit test".to_string(),
            strength: 8,
        });
        assert!(overlay_record_to_fact(&record).is_none());
    }

    #[test]
    fn chat_core_anti_wave_hits_require_learned_lane() {
        let fallback_negative = LinuxChatCoreFactPreview {
            route: "linux.boundary.socket".to_string(),
            subject: "package_installed".to_string(),
            subject_role: "shortcut".to_string(),
            relation: "does not prove".to_string(),
            object: "vpn_running".to_string(),
            object_role: "runtime_state".to_string(),
            polarity: "negative".to_string(),
            evidence_kind: "fixture".to_string(),
            confidence: 90,
            memory_kind: "fallback".to_string(),
        };
        let learned_negative = LinuxChatCoreFactPreview {
            memory_kind: "learned_anti_wave".to_string(),
            ..fallback_negative.clone()
        };
        let cache = LinuxChatCoreHotCache {
            residual_summary: LinuxResidualDecodedSummary {
                path: "unit-test.lrf".to_string(),
                file_bytes: 0,
                wave_dim: 1024,
                represented_fact_count: 0,
                schema_record_count: 0,
                residual_record_count: 0,
                fallback_record_count: 0,
                route_count: 0,
                corpus_hash64: 0,
                promotion_threshold: 0,
                binary_hot_sections_bytes: 0,
                direct_fixed_baseline_bytes: 0,
                cold_label_count: 0,
                cold_label_table_bytes: 0,
                binary_hot_sections_fit_6m: true,
                beats_direct_fixed64: true,
            },
            route_index: Vec::new(),
            readout_facts: vec![fallback_negative, learned_negative],
            domains: Vec::new(),
        };
        let query = LinuxQueryWave {
            intent: "boundary_rejection".to_string(),
            anchors: Vec::new(),
            route_priors: Vec::new(),
            required_routes: Vec::new(),
            negative_boundaries: Vec::new(),
            forbidden_shortcuts: Vec::new(),
            polarity: "question".to_string(),
            answer_policy: "deny_shortcut".to_string(),
        };
        let hits =
            learned_anti_wave_hits(&cache, "does package installed prove vpn running", &query);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].memory_kind, "learned_anti_wave");
    }
}
