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
    build_linux_profile_claim_gate_report, build_linux_reason_report_from_decoded_facts,
    LinuxProfileClaimGateConfig, LinuxProfileClaimGateReport,
};
use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
};

pub(crate) const LINUX_CHAT_CORE_VERSION: &str = "llmwave-big-v-next-linux-chat-core";
pub(crate) const DEFAULT_LINUX_CHAT_CORE_PROFILE: &str = "examples/linux-chat-core.profile.json";
const HOT_MAGIC: &[u8] = b"LLMWCHATCORE1\n";

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
    pub stale_detection_required: bool,
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
    pub note: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreAskTokenEconomics {
    pub estimate_method: &'static str,
    pub source_artifacts_bytes: u64,
    pub source_artifacts_estimated_tokens: u64,
    pub cache_index_bytes: u64,
    pub cache_index_estimated_tokens: u64,
    pub grounded_packet_bytes: u64,
    pub grounded_packet_estimated_tokens: u64,
    pub estimated_tokens_saved_vs_source: u64,
    pub estimated_tokens_saved_vs_cache_index: u64,
    pub source_to_packet_reduction_ratio: f32,
    pub cache_index_to_packet_reduction_ratio: f32,
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
    pub hot_path: String,
    pub hot_bytes: u64,
    pub hot_sha256: String,
    pub index_path: String,
    pub index_bytes: u64,
    pub index_sha256: String,
    pub manifest_path: String,
    pub manifest_sha256: String,
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
    pub decision_state: String,
    pub answer: String,
    pub intent: String,
    pub route_priors: Vec<String>,
    pub evidence_count: usize,
    pub anti_wave_hits: Vec<String>,
    pub compact_evidence: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatCoreClaimBoundary {
    pub linux_chat_core_cache_ready: bool,
    pub compiled_cache_matches_source_memory: bool,
    pub cache_is_source_of_truth: bool,
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
    let report = LinuxChatCoreBuildReport {
        mode: "llmwave-big-linux-chat-core-build",
        version: LINUX_CHAT_CORE_VERSION,
        verdict: "LINUX_CHAT_CORE_CACHE_READY_NOT_GENERAL_LLM",
        spec,
        manifest,
        source_status,
        cache,
        token_economics,
        claim_boundary: claim_boundary(true, false, false),
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
}

pub(crate) fn build_linux_chat_core_gate_report(
    config: LinuxChatCoreGateConfig,
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
    let profile_gate = if cache_status.cache_fresh {
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
    let chat_core_ready = cache_status.cache_fresh && profile_ready;
    let report = LinuxChatCoreGateReport {
        mode: "llmwave-big-linux-chat-core-gate",
        version: LINUX_CHAT_CORE_VERSION,
        verdict: if chat_core_ready {
            "LLMWAVE_LINUX_CHAT_CORE_READY_NOT_GENERAL_LLM"
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
        chat_core: LinuxChatCoreAuthority {
            safe_to_use_cache: cache_status.cache_fresh,
            safe_to_answer_from_cache: chat_core_ready,
            compiled_runtime_cache_ready: cache_status.cache_fresh,
            profile_gate_ready: profile_ready,
            source_hash_matched: cache_status.cache_fresh,
            stale_cache_blocks_answer_authority: true,
            cache_is_source_of_truth: false,
            compatibility_wrapper_for_linux_chat_profile_gate: true,
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
    let _source_overlay_args_preserved_for_cli_symmetry = (
        &config.residual_pack,
        &config.dialogue_overlay,
        &config.centers_overlay,
        &config.vpn_overlay,
    );
    let manifest = config
        .manifest
        .clone()
        .unwrap_or_else(|| config.cache_dir.join("chat-core.manifest.json"));
    let cache_status = evaluate_cache_from_manifest(&manifest)?;
    let reason = if cache_status.cache_fresh {
        let cache_index = load_cache_index_from_manifest(&manifest)?;
        let facts = cache_index
            .readout_facts
            .iter()
            .map(LinuxChatCoreFactPreview::to_decoded_fact)
            .collect::<Vec<_>>();
        Some(build_linux_reason_report_from_decoded_facts(
            cache_index.residual_summary,
            &facts,
            &config.text,
            config.max_facts.max(1),
        ))
    } else {
        None
    };
    let packet = match reason {
        Some(report) => LinuxChatCoreGroundedPacket {
            cache_fresh: true,
            answer_allowed: report.decision.answer_allowed,
            readout_source: "compiled_chat_core_index".to_string(),
            decision_state: report.decision.state,
            answer: report.decision.answer,
            intent: report.query_wave.intent,
            route_priors: report.query_wave.route_priors,
            evidence_count: report.evidence_chain.len(),
            anti_wave_hits: report
                .anti_wave_hits
                .iter()
                .map(|hit| format!("{} -> {}:{}", hit.shortcut, hit.replacement_peak, hit.reason))
                .collect(),
            compact_evidence: report
                .evidence_chain
                .iter()
                .map(|step| format!("{} | {} | {}", step.route, step.relation, step.object))
                .collect(),
        },
        None => LinuxChatCoreGroundedPacket {
            cache_fresh: false,
            answer_allowed: false,
            readout_source: "none_cache_stale".to_string(),
            decision_state: "CACHE_STALE_NO_AUTHORITY".to_string(),
            answer: "Cache is stale or missing; rebuild and gate ChatCore before using it as answer support.".to_string(),
            intent: "unknown".to_string(),
            route_priors: Vec::new(),
            evidence_count: 0,
            anti_wave_hits: cache_status.stale_reasons.clone(),
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
        grounded_packet: packet,
        token_economics,
        claim_boundary: claim_boundary(cache_status.cache_fresh, true, false),
    };
    write_json_if_requested(config.out.as_deref(), &report)?;
    Ok(report)
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
    let source_hash = combined_hash(
        source_artifacts
            .iter()
            .map(|artifact| artifact.sha256.as_str()),
    );
    let route_index_hash = hash_json(&route_index)?;
    let domain_registry_hash = hash_json(&spec.domains)?;
    let overlay_registry_hash = hash_json(&spec.overlays)?;
    let spec_hash = hash_json(spec)?;
    let index = LinuxChatCoreCacheIndex {
        profile_id: spec.profile_id.clone(),
        residual_summary: decoded_packet.summary.clone(),
        represented_fact_count: decoded_packet.summary.represented_fact_count,
        schema_record_count: decoded_packet.summary.schema_record_count,
        residual_record_count: decoded_packet.summary.residual_record_count,
        fallback_record_count: decoded_packet.summary.fallback_record_count,
        route_index,
        readout_facts: decoded_packet
            .facts
            .iter()
            .map(LinuxChatCoreFactPreview::from_decoded_fact)
            .collect(),
        domains: spec.domains.clone(),
        overlays: spec.overlays.clone(),
        source_artifacts: source_artifacts.clone(),
        cache_contract: LinuxChatCoreCacheContract {
            compiled_from_source_hashes: true,
            no_secret_scan: true,
            hot_cache_has_no_authority_without_gate: true,
            stale_detection_required: true,
        },
    };
    let index_bytes = serde_json::to_vec_pretty(&index)?;
    let mut hot = Vec::new();
    hot.extend_from_slice(HOT_MAGIC);
    hot.extend_from_slice(spec.profile_id.as_bytes());
    hot.extend_from_slice(b"\nsource:");
    hot.extend_from_slice(source_hash.as_bytes());
    hot.extend_from_slice(b"\ndomains:");
    hot.extend_from_slice(domain_registry_hash.as_bytes());
    hot.extend_from_slice(b"\noverlays:");
    hot.extend_from_slice(overlay_registry_hash.as_bytes());
    hot.extend_from_slice(b"\nroute-index:");
    hot.extend_from_slice(route_index_hash.as_bytes());
    hot.extend_from_slice(b"\nfacts:");
    hot.extend_from_slice(
        decoded_packet
            .summary
            .represented_fact_count
            .to_string()
            .as_bytes(),
    );
    hot.extend_from_slice(b"\nschemas:");
    hot.extend_from_slice(
        decoded_packet
            .summary
            .schema_record_count
            .to_string()
            .as_bytes(),
    );
    hot.extend_from_slice(b"\nresiduals:");
    hot.extend_from_slice(
        decoded_packet
            .summary
            .residual_record_count
            .to_string()
            .as_bytes(),
    );
    hot.extend_from_slice(b"\n");
    for domain in &spec.domains {
        hot.extend_from_slice(domain.domain_id.as_bytes());
        hot.extend_from_slice(b"\0");
    }
    let hot_hash = hash_bytes(&hot);
    let index_hash = hash_bytes(&index_bytes);
    let hot_path = PathBuf::from(&spec.cache.hot_path);
    let index_path = PathBuf::from(&spec.cache.index_path);
    let manifest_path = PathBuf::from(&spec.cache.manifest_path);
    write_bytes(&hot_path, &hot)?;
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
        hot_bytes: hot.len() as u64,
        hot_sha256: hot_hash,
        index_path: path_string(&index_path),
        index_bytes: index_bytes.len() as u64,
        index_sha256: index_hash,
        cache_is_source_of_truth: false,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    write_bytes(&manifest_path, &manifest_bytes)?;
    let cache = LinuxChatCoreCompiledCache {
        hot_path: manifest.hot_path.clone(),
        hot_bytes: manifest.hot_bytes,
        hot_sha256: manifest.hot_sha256.clone(),
        index_path: manifest.index_path.clone(),
        index_bytes: manifest.index_bytes,
        index_sha256: manifest.index_sha256.clone(),
        manifest_path: path_string(&manifest_path),
        manifest_sha256: hash_bytes(&manifest_bytes),
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
    if !index_present {
        stale_reasons.push("cache_index_missing".to_string());
    } else if hash_file(&index_path)? != manifest.index_sha256 {
        stale_reasons.push("cache_index_hash_changed".to_string());
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
    if !index_present {
        stale_reasons.push("cache_index_missing".to_string());
    } else if hash_file(&index_path)? != manifest.index_sha256 {
        stale_reasons.push("cache_index_hash_changed".to_string());
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

fn load_cache_index_from_manifest(manifest_path: &Path) -> Result<LinuxChatCoreCacheIndex> {
    let manifest: LinuxChatCoreCacheManifest = serde_json::from_slice(
        &fs::read(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse {}", manifest_path.display()))?;
    let index_path = PathBuf::from(&manifest.index_path);
    serde_json::from_slice(
        &fs::read(&index_path).with_context(|| format!("read {}", index_path.display()))?,
    )
    .with_context(|| format!("parse {}", index_path.display()))
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
        note: "This estimates prompt/context scale only. It is not a model-specific BPE tokenizer count.",
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
            source_artifacts_bytes: 0,
            source_artifacts_estimated_tokens: 0,
            cache_index_bytes: 0,
            cache_index_estimated_tokens: 0,
            grounded_packet_bytes: packet_bytes,
            grounded_packet_estimated_tokens: estimate_tokens_from_bytes(packet_bytes),
            estimated_tokens_saved_vs_source: 0,
            estimated_tokens_saved_vs_cache_index: 0,
            source_to_packet_reduction_ratio: 0.0,
            cache_index_to_packet_reduction_ratio: 0.0,
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
        source_artifacts_bytes,
        source_artifacts_estimated_tokens: source_tokens,
        cache_index_bytes: manifest.index_bytes,
        cache_index_estimated_tokens: cache_index_tokens,
        grounded_packet_bytes: packet_bytes,
        grounded_packet_estimated_tokens: packet_tokens,
        estimated_tokens_saved_vs_source: source_tokens.saturating_sub(packet_tokens),
        estimated_tokens_saved_vs_cache_index: cache_index_tokens.saturating_sub(packet_tokens),
        source_to_packet_reduction_ratio: ratio_f32(source_artifacts_bytes, packet_bytes),
        cache_index_to_packet_reduction_ratio: ratio_f32(manifest.index_bytes, packet_bytes),
        note: "Savings estimate compares sending the full source/cache-index context to sending only the grounded packet. It is not a model-specific BPE tokenizer count.",
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
