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

pub(crate) const LINUX_CHAT_CORE_VERSION: &str = "llmwave-big-v-next-linux-chat-core-hot-v3";
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
    let packet = match reason {
        Some(report) => {
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
    let route_index = route_index(&decoded_packet.facts);
    let domain_registry_hash = hash_json(&spec.domains)?;
    let overlay_registry_hash = hash_json(&spec.overlays)?;
    let spec_hash = hash_json(spec)?;
    let readout_facts = decoded_packet
        .facts
        .iter()
        .map(LinuxChatCoreFactPreview::from_decoded_fact)
        .collect::<Vec<_>>();
    let domains = spec.domains.clone();
    let index = LinuxChatCoreCacheIndex {
        profile_id: spec.profile_id.clone(),
        residual_summary: decoded_packet.summary.clone(),
        represented_fact_count: decoded_packet.summary.represented_fact_count,
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
}
