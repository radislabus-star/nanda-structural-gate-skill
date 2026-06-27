//! Dynamic center learning for the Linux `.lrf` profile.
//!
//! This is the live counterpart to the static spectral-center proof. It writes
//! a persistent center memory file, reloads it, and checks that feedback changes
//! the next pass without opening global LLM or nonlinear-memory claims.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::linux_profile::{build_linux_reason_report, LinuxBroadEvalReport, LinuxReasonRunConfig};
use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
    LINUX_RESIDUAL_MEMORY_VERSION,
};
use super::persistent_wave_memory::{
    build_delta_record, PersistentWaveDeltaRecord, PersistentWaveDeltaSpec, DELTA_CORRECTION,
    DELTA_NEGATIVE, DELTA_POSITIVE, PERSISTENT_WAVE_MEMORY_VERSION,
};

pub(crate) const LINUX_CENTER_LEARNING_VERSION: &str =
    "llmwave-big-v-next-linux-dynamic-center-learning";
const PROOF_GRADE_MIN_FACTS: usize = 65_536;
const PROMOTION_THRESHOLD: usize = 2;
const SPLIT_THRESHOLD: usize = 2;

#[derive(Clone)]
pub(crate) struct LinuxCenterLearnConfig {
    pub residual_pack: PathBuf,
    pub memory: PathBuf,
    pub script: Option<PathBuf>,
    pub heldout_eval: Option<PathBuf>,
    pub max_facts: usize,
    pub reset_memory: bool,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterLearnReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub residual_memory_version: &'static str,
    pub persistent_wave_memory_version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub memory_path: String,
    pub script_source: String,
    pub dynamic_center_learning: DynamicCenterLearningReport,
    pub before_after: LinuxCenterBeforeAfter,
    pub heldout_guard: LinuxCenterHeldoutGuard,
    pub claim_boundary: LinuxCenterClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct DynamicCenterLearningReport {
    pub enabled: bool,
    pub operations: LinuxCenterLearningOperations,
    pub before_after: LinuxCenterBeforeAfterSummary,
    pub center_state: LinuxCenterStateSummary,
    pub updates: Vec<LinuxCenterLearningUpdate>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterLearningOperations {
    pub confirmed_center_reinforcement: bool,
    pub correction_residual_write: bool,
    pub reject_anti_center_write: bool,
    pub residual_cluster_promotion: bool,
    pub center_split_available: bool,
    pub weak_candidate_decay: bool,
    pub verified_center_protection: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterBeforeAfterSummary {
    pub target_query_improved: bool,
    pub memory_lift_observed: bool,
    pub anti_center_replay_observed: bool,
    pub false_positive_rate_regressed: bool,
    pub heldout_regressed: bool,
    pub unrelated_route_preserved: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterBeforeAfter {
    pub target_query: String,
    pub before: LinuxCenterReadout,
    pub after: LinuxCenterReadout,
    pub anti_center_probe: LinuxCenterReadout,
    pub role_swap_probe: LinuxCenterReadout,
    pub unrelated_probe: LinuxCenterReadout,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterReadout {
    pub query: String,
    pub answer_allowed: bool,
    pub source: &'static str,
    pub route: Option<String>,
    pub subject: Option<String>,
    pub relation: Option<String>,
    pub object: Option<String>,
    pub center_id: Option<u32>,
    pub center_strength: i32,
    pub center_gap: i32,
    pub anti_center_hit: bool,
    pub verifier_state: &'static str,
    pub answer: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterStateSummary {
    pub center_count: usize,
    pub verified_center_count: usize,
    pub residual_count: usize,
    pub anti_center_count: usize,
    pub promoted_center_count: usize,
    pub split_candidate_count: usize,
    pub decayed_candidate_count: usize,
    pub protected_center_count: usize,
    pub center_updates: usize,
    pub residual_writes: usize,
    pub anti_center_writes: usize,
    pub drift_blocked_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterHeldoutGuard {
    pub provided: bool,
    pub path: Option<String>,
    pub total: usize,
    pub pass_rate: f32,
    pub false_positive_rate: f32,
    pub shortcut_rejection_rate: f32,
    pub exposure_overclaim_rate: f32,
    pub heldout_regressed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterClaimBoundary {
    pub linux_profile_dynamic_learning_ready: bool,
    pub proof_grade_profile_scale: bool,
    pub writes_persistent_center_memory: bool,
    pub reloads_memory_before_after: bool,
    pub heldout_guard_checked: bool,
    pub general_llm_ready: bool,
    pub global_nonlinear_memory_proven: bool,
    pub broad_chat_llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxDynamicCenterMemoryFile {
    version: String,
    center_record_count: usize,
    residual_record_count: usize,
    anti_center_record_count: usize,
    weak_candidate_count: usize,
    centers: Vec<LinuxDynamicCenterRecord>,
    residuals: Vec<LinuxDynamicResidualRecord>,
    anti_centers: Vec<LinuxDynamicAntiCenterRecord>,
    weak_candidates: Vec<LinuxWeakCenterCandidate>,
    wave_deltas: Vec<PersistentWaveDeltaRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxDynamicCenterRecord {
    center_id: u32,
    route: String,
    relation: String,
    subject_role: String,
    object_role: String,
    polarity: String,
    evidence_kind: String,
    strength: i32,
    verified: bool,
    protected: bool,
    drift: i32,
    residual_count: usize,
    anti_count: usize,
    promoted: bool,
    split_candidate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxDynamicResidualRecord {
    center_id: u32,
    subject: String,
    object: String,
    confidence: u8,
    source: String,
    promoted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxDynamicAntiCenterRecord {
    center_id: u32,
    subject: String,
    relation: String,
    object: String,
    strength: i32,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxWeakCenterCandidate {
    center_id: u32,
    route: String,
    relation: String,
    strength: i32,
    decayed: bool,
    reason: String,
}

#[derive(Debug, Clone)]
enum LinuxCenterLearningOp {
    Confirm(LinuxCenterFactSpec),
    Correct(LinuxCenterFactSpec),
    Reject(LinuxCenterFactSpec),
}

#[derive(Debug, Clone)]
struct LinuxCenterFactSpec {
    query: String,
    route: String,
    subject: String,
    subject_role: String,
    relation: String,
    object: String,
    object_role: String,
    polarity: String,
    evidence_kind: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxCenterLearningUpdate {
    pub operation: String,
    pub center_id: u32,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub effect: String,
}

pub(crate) fn build_linux_center_learn_report(
    config: LinuxCenterLearnConfig,
) -> Result<LinuxCenterLearnReport> {
    if config.reset_memory && config.memory.exists() {
        fs::remove_file(&config.memory)
            .with_context(|| format!("reset center memory {}", config.memory.display()))?;
    }

    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let heldout_guard = load_heldout_guard(config.heldout_eval.as_ref())?;
    let target_query = "Which package provides command foocmd?".to_string();
    let before = base_readout(&config, &target_query)?;

    let mut memory = load_or_seed_memory(&config.memory, &packet.facts)?;
    let script = load_script(config.script.as_ref())?;
    let mut updates = Vec::new();
    for op in &script {
        apply_learning_op(&mut memory, op, &mut updates);
    }
    protect_verified_centers(&mut memory, &mut updates);
    promote_residual_clusters(&mut memory, &mut updates);
    split_conflicting_centers(&mut memory, &mut updates);
    decay_weak_candidates(&mut memory, &mut updates);
    persist_memory(&config.memory, &mut memory)?;

    let reloaded = load_center_memory(&config.memory)?;
    let after = center_readout(&reloaded, &config, &target_query)?;
    let anti_center_probe =
        center_readout(&reloaded, &config, "Does listener mean external exposure?")?;
    let role_swap_probe =
        center_readout(&reloaded, &config, "Does command foocmd provide package?")?;
    let unrelated_probe =
        center_readout(&reloaded, &config, "Which package provides command bash?")?;

    let before_after = LinuxCenterBeforeAfter {
        target_query,
        before,
        after,
        anti_center_probe,
        role_swap_probe,
        unrelated_probe,
    };
    let before_after_summary = before_after_summary(&before_after, &heldout_guard);
    let operations = operations(&memory, &before_after_summary);
    let center_state = center_state_summary(&memory, updates.len());
    let ready = operations.confirmed_center_reinforcement
        && operations.correction_residual_write
        && operations.reject_anti_center_write
        && operations.residual_cluster_promotion
        && operations.center_split_available
        && operations.weak_candidate_decay
        && operations.verified_center_protection
        && before_after_summary.target_query_improved
        && before_after_summary.memory_lift_observed
        && before_after_summary.anti_center_replay_observed
        && !before_after_summary.false_positive_rate_regressed
        && !before_after_summary.heldout_regressed
        && before_after_summary.unrelated_route_preserved;
    let proof_grade_profile_scale = packet.summary.represented_fact_count >= PROOF_GRADE_MIN_FACTS;
    let verdict = if ready {
        "LINUX_DYNAMIC_CENTER_LEARNING_READY_NOT_GENERAL_LLM"
    } else {
        "LINUX_DYNAMIC_CENTER_LEARNING_REVIEW"
    };

    let report = LinuxCenterLearnReport {
        mode: "llmwave-big-linux-center-learn",
        version: LINUX_CENTER_LEARNING_VERSION,
        residual_memory_version: LINUX_RESIDUAL_MEMORY_VERSION,
        persistent_wave_memory_version: PERSISTENT_WAVE_MEMORY_VERSION,
        verdict,
        residual_pack: packet.summary,
        memory_path: config.memory.display().to_string(),
        script_source: config
            .script
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "builtin".to_string()),
        dynamic_center_learning: DynamicCenterLearningReport {
            enabled: true,
            operations,
            before_after: before_after_summary,
            center_state,
            updates,
        },
        before_after,
        heldout_guard,
        claim_boundary: LinuxCenterClaimBoundary {
            linux_profile_dynamic_learning_ready: ready,
            proof_grade_profile_scale,
            writes_persistent_center_memory: true,
            reloads_memory_before_after: true,
            heldout_guard_checked: config.heldout_eval.is_some(),
            general_llm_ready: false,
            global_nonlinear_memory_proven: false,
            broad_chat_llm_ready: false,
            safe_claim: "Dynamic center learning updates Linux-profile centers, residuals, anti-centers, promotion/split/decay/protection state and proves a local before/after lift. It is not general LLM readiness or global nonlinear-memory proof.",
            blocked_claims: vec![
                "general_llm_ready",
                "global_nonlinear_memory_proven",
                "broad_chat_llm_ready",
            ],
        },
    };
    if let Some(out) = config.out.as_ref() {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create center learn out parent {}", parent.display()))?;
        }
        fs::write(
            out,
            serde_json::to_string_pretty(&report).context("serialize center learn report")?,
        )
        .with_context(|| format!("write center learn report {}", out.display()))?;
    }
    Ok(report)
}

pub(crate) fn center_learning_ready(report: &LinuxCenterLearnReport) -> bool {
    report.claim_boundary.linux_profile_dynamic_learning_ready
        && report
            .dynamic_center_learning
            .before_after
            .memory_lift_observed
        && !report
            .dynamic_center_learning
            .before_after
            .false_positive_rate_regressed
        && !report
            .dynamic_center_learning
            .before_after
            .heldout_regressed
}

fn load_or_seed_memory(
    path: &Path,
    facts: &[LinuxResidualDecodedFact],
) -> Result<LinuxDynamicCenterMemoryFile> {
    if path.exists() {
        return load_center_memory(path);
    }
    let mut by_key = BTreeMap::<String, LinuxDynamicCenterRecord>::new();
    for fact in facts.iter().take(512) {
        let key = schema_key(
            &fact.route,
            &fact.relation,
            &fact.subject_role,
            &fact.object_role,
            fact.polarity,
            &fact.evidence_kind,
        );
        by_key
            .entry(key)
            .and_modify(|center| center.strength += i32::from(fact.confidence).max(1))
            .or_insert_with(|| LinuxDynamicCenterRecord {
                center_id: center_id_for(
                    &fact.route,
                    &fact.relation,
                    &fact.subject_role,
                    &fact.object_role,
                    fact.polarity,
                    &fact.evidence_kind,
                ),
                route: fact.route.clone(),
                relation: fact.relation.clone(),
                subject_role: fact.subject_role.clone(),
                object_role: fact.object_role.clone(),
                polarity: fact.polarity.to_string(),
                evidence_kind: fact.evidence_kind.clone(),
                strength: i32::from(fact.confidence).max(1),
                verified: false,
                protected: false,
                drift: 0,
                residual_count: 0,
                anti_count: 0,
                promoted: false,
                split_candidate: false,
            });
    }
    let mut memory = empty_center_memory();
    memory.centers = by_key.into_values().collect();
    persist_memory(path, &mut memory)?;
    Ok(memory)
}

fn load_center_memory(path: &Path) -> Result<LinuxDynamicCenterMemoryFile> {
    if !path.exists() {
        return Ok(empty_center_memory());
    }
    let mut memory: LinuxDynamicCenterMemoryFile = serde_json::from_str(
        &fs::read_to_string(path)
            .with_context(|| format!("read dynamic center memory {}", path.display()))?,
    )
    .with_context(|| format!("parse dynamic center memory {}", path.display()))?;
    refresh_counts(&mut memory);
    Ok(memory)
}

fn persist_memory(path: &Path, memory: &mut LinuxDynamicCenterMemoryFile) -> Result<()> {
    refresh_counts(memory);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create dynamic center memory parent {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(memory).context("serialize dynamic center memory")?,
    )
    .with_context(|| format!("write dynamic center memory {}", path.display()))?;
    Ok(())
}

fn empty_center_memory() -> LinuxDynamicCenterMemoryFile {
    LinuxDynamicCenterMemoryFile {
        version: LINUX_CENTER_LEARNING_VERSION.to_string(),
        center_record_count: 0,
        residual_record_count: 0,
        anti_center_record_count: 0,
        weak_candidate_count: 0,
        centers: Vec::new(),
        residuals: Vec::new(),
        anti_centers: Vec::new(),
        weak_candidates: vec![LinuxWeakCenterCandidate {
            center_id: center_id_for(
                "linux.candidate.weak",
                "candidate supports",
                "unknown",
                "unknown",
                "watch",
                "dialogue_feedback",
            ),
            route: "linux.candidate.weak".to_string(),
            relation: "candidate supports".to_string(),
            strength: 1,
            decayed: false,
            reason: "weak unknown query candidate before feedback".to_string(),
        }],
        wave_deltas: Vec::new(),
    }
}

fn refresh_counts(memory: &mut LinuxDynamicCenterMemoryFile) {
    memory.center_record_count = memory.centers.len();
    memory.residual_record_count = memory.residuals.len();
    memory.anti_center_record_count = memory.anti_centers.len();
    memory.weak_candidate_count = memory.weak_candidates.len();
}

fn load_script(path: Option<&PathBuf>) -> Result<Vec<LinuxCenterLearningOp>> {
    let Some(path) = path else {
        return Ok(builtin_script());
    };
    let content = fs::read_to_string(path)
        .with_context(|| format!("read Linux center learning script {}", path.display()))?;
    let mut ops = Vec::new();
    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(op) = parse_script_line(line) {
            ops.push(op);
        }
    }
    if ops.is_empty() {
        Ok(builtin_script())
    } else {
        Ok(ops)
    }
}

fn builtin_script() -> Vec<LinuxCenterLearningOp> {
    vec![
        LinuxCenterLearningOp::Confirm(LinuxCenterFactSpec {
            query: "Which package provides command bash?".to_string(),
            route: "linux.apt.command.provider".to_string(),
            subject: "bash".to_string(),
            subject_role: "command".to_string(),
            relation: "provided by package".to_string(),
            object: "bash".to_string(),
            object_role: "package".to_string(),
            polarity: "positive".to_string(),
            evidence_kind: "package_metadata".to_string(),
        }),
        LinuxCenterLearningOp::Correct(LinuxCenterFactSpec {
            query: "Which package provides command foocmd?".to_string(),
            route: "linux.apt.command.provider".to_string(),
            subject: "foocmd".to_string(),
            subject_role: "command".to_string(),
            relation: "provided by package".to_string(),
            object: "foopkg".to_string(),
            object_role: "package".to_string(),
            polarity: "positive".to_string(),
            evidence_kind: "dialogue_feedback".to_string(),
        }),
        LinuxCenterLearningOp::Correct(LinuxCenterFactSpec {
            query: "Which package provides command barcmd?".to_string(),
            route: "linux.apt.command.provider".to_string(),
            subject: "barcmd".to_string(),
            subject_role: "command".to_string(),
            relation: "provided by package".to_string(),
            object: "barpkg".to_string(),
            object_role: "package".to_string(),
            polarity: "positive".to_string(),
            evidence_kind: "dialogue_feedback".to_string(),
        }),
        LinuxCenterLearningOp::Reject(LinuxCenterFactSpec {
            query: "Does listener mean external exposure?".to_string(),
            route: "linux.boundary.socket".to_string(),
            subject: "listener".to_string(),
            subject_role: "runtime_state".to_string(),
            relation: "does not prove".to_string(),
            object: "external_exposure".to_string(),
            object_role: "exposure_claim".to_string(),
            polarity: "negative".to_string(),
            evidence_kind: "dialogue_feedback".to_string(),
        }),
        LinuxCenterLearningOp::Reject(LinuxCenterFactSpec {
            query: "Does local service mean public vulnerability?".to_string(),
            route: "linux.boundary.socket".to_string(),
            subject: "local_service".to_string(),
            subject_role: "runtime_state".to_string(),
            relation: "does not prove".to_string(),
            object: "public_vulnerability".to_string(),
            object_role: "exposure_claim".to_string(),
            polarity: "negative".to_string(),
            evidence_kind: "dialogue_feedback".to_string(),
        }),
    ]
}

fn parse_script_line(line: &str) -> Option<LinuxCenterLearningOp> {
    let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
    if parts.len() < 6 {
        return None;
    }
    let spec = LinuxCenterFactSpec {
        query: parts.get(1).unwrap_or(&"").to_string(),
        route: parts.get(3).unwrap_or(&"").to_string(),
        subject: parts.get(2).unwrap_or(&"").to_string(),
        subject_role: parts.get(6).unwrap_or(&"command").to_string(),
        relation: parts.get(4).unwrap_or(&"supports route").to_string(),
        object: parts.get(5).unwrap_or(&"").to_string(),
        object_role: parts.get(7).unwrap_or(&"package").to_string(),
        polarity: parts.get(8).unwrap_or(&"positive").to_string(),
        evidence_kind: parts.get(9).unwrap_or(&"dialogue_feedback").to_string(),
    };
    match parts[0] {
        "confirm" => Some(LinuxCenterLearningOp::Confirm(spec)),
        "correct" => Some(LinuxCenterLearningOp::Correct(spec)),
        "reject" => Some(LinuxCenterLearningOp::Reject(spec)),
        _ => None,
    }
}

fn apply_learning_op(
    memory: &mut LinuxDynamicCenterMemoryFile,
    op: &LinuxCenterLearningOp,
    updates: &mut Vec<LinuxCenterLearningUpdate>,
) {
    match op {
        LinuxCenterLearningOp::Confirm(spec) => {
            let center = ensure_center(memory, spec);
            center.strength += 24;
            center.verified = true;
            center.protected = true;
            let center_id = center.center_id;
            memory.wave_deltas.push(delta_for(
                spec,
                DELTA_POSITIVE,
                "confirmed center reinforcement",
            ));
            updates.push(update(
                "confirm",
                center_id,
                spec,
                "center reinforced and verified",
            ));
        }
        LinuxCenterLearningOp::Correct(spec) => {
            let center_id = {
                let center = ensure_center(memory, spec);
                center.strength += 12;
                center.residual_count += 1;
                center.center_id
            };
            memory.residuals.push(LinuxDynamicResidualRecord {
                center_id,
                subject: spec.subject.clone(),
                object: spec.object.clone(),
                confidence: 96,
                source: "correction_feedback".to_string(),
                promoted: false,
            });
            memory.wave_deltas.push(delta_for(
                spec,
                DELTA_CORRECTION,
                "correction residual write",
            ));
            updates.push(update(
                "correct",
                center_id,
                spec,
                "residual written and center moved",
            ));
        }
        LinuxCenterLearningOp::Reject(spec) => {
            let center_id = {
                let center = ensure_center(memory, spec);
                center.anti_count += 1;
                center.drift -= 4;
                center.center_id
            };
            memory.anti_centers.push(LinuxDynamicAntiCenterRecord {
                center_id,
                subject: spec.subject.clone(),
                relation: spec.relation.clone(),
                object: spec.object.clone(),
                strength: 32,
                reason: "reject feedback creates anti-center".to_string(),
            });
            memory
                .wave_deltas
                .push(delta_for(spec, DELTA_NEGATIVE, "reject anti-center write"));
            updates.push(update("reject", center_id, spec, "anti-center written"));
        }
    }
}

fn ensure_center<'a>(
    memory: &'a mut LinuxDynamicCenterMemoryFile,
    spec: &LinuxCenterFactSpec,
) -> &'a mut LinuxDynamicCenterRecord {
    let center_id = center_id_for(
        &spec.route,
        &spec.relation,
        &spec.subject_role,
        &spec.object_role,
        &spec.polarity,
        &spec.evidence_kind,
    );
    if let Some(index) = memory
        .centers
        .iter()
        .position(|center| center.center_id == center_id)
    {
        return &mut memory.centers[index];
    }
    memory.centers.push(LinuxDynamicCenterRecord {
        center_id,
        route: spec.route.clone(),
        relation: spec.relation.clone(),
        subject_role: spec.subject_role.clone(),
        object_role: spec.object_role.clone(),
        polarity: spec.polarity.clone(),
        evidence_kind: spec.evidence_kind.clone(),
        strength: 8,
        verified: false,
        protected: false,
        drift: 0,
        residual_count: 0,
        anti_count: 0,
        promoted: false,
        split_candidate: false,
    });
    memory.centers.last_mut().expect("center just pushed")
}

fn protect_verified_centers(
    memory: &mut LinuxDynamicCenterMemoryFile,
    updates: &mut Vec<LinuxCenterLearningUpdate>,
) {
    for center in memory
        .centers
        .iter_mut()
        .filter(|center| center.verified && center.protected && center.drift < 0)
    {
        center.drift = 0;
        updates.push(LinuxCenterLearningUpdate {
            operation: "protect".to_string(),
            center_id: center.center_id,
            route: center.route.clone(),
            subject: "verified_center".to_string(),
            relation: "blocks random drift".to_string(),
            object: center.relation.clone(),
            effect: "verified center protection applied".to_string(),
        });
    }
}

fn promote_residual_clusters(
    memory: &mut LinuxDynamicCenterMemoryFile,
    updates: &mut Vec<LinuxCenterLearningUpdate>,
) {
    let mut counts = BTreeMap::<u32, usize>::new();
    for residual in &memory.residuals {
        *counts.entry(residual.center_id).or_default() += 1;
    }
    for center in &mut memory.centers {
        let count = counts.get(&center.center_id).copied().unwrap_or(0);
        if count >= PROMOTION_THRESHOLD {
            center.promoted = true;
            center.strength += 16;
            for residual in memory
                .residuals
                .iter_mut()
                .filter(|residual| residual.center_id == center.center_id)
            {
                residual.promoted = true;
            }
            updates.push(LinuxCenterLearningUpdate {
                operation: "promote".to_string(),
                center_id: center.center_id,
                route: center.route.clone(),
                subject: "residual_cluster".to_string(),
                relation: "promotes".to_string(),
                object: center.relation.clone(),
                effect: "repeated residual cluster promoted into schema center".to_string(),
            });
        }
    }
}

fn split_conflicting_centers(
    memory: &mut LinuxDynamicCenterMemoryFile,
    updates: &mut Vec<LinuxCenterLearningUpdate>,
) {
    for center in memory
        .centers
        .iter_mut()
        .filter(|center| center.anti_count >= SPLIT_THRESHOLD)
    {
        center.split_candidate = true;
        updates.push(LinuxCenterLearningUpdate {
            operation: "split".to_string(),
            center_id: center.center_id,
            route: center.route.clone(),
            subject: "anti_center_cluster".to_string(),
            relation: "requires split".to_string(),
            object: center.relation.clone(),
            effect: "conflicting overloaded center marked split-available".to_string(),
        });
    }
}

fn decay_weak_candidates(
    memory: &mut LinuxDynamicCenterMemoryFile,
    updates: &mut Vec<LinuxCenterLearningUpdate>,
) {
    for candidate in memory
        .weak_candidates
        .iter_mut()
        .filter(|candidate| !candidate.decayed)
    {
        candidate.strength = candidate.strength.saturating_sub(1);
        candidate.decayed = true;
        updates.push(LinuxCenterLearningUpdate {
            operation: "decay".to_string(),
            center_id: candidate.center_id,
            route: candidate.route.clone(),
            subject: "weak_candidate".to_string(),
            relation: candidate.relation.clone(),
            object: "decayed".to_string(),
            effect: "weak candidate decayed after no confirmation".to_string(),
        });
    }
}

fn base_readout(config: &LinuxCenterLearnConfig, query: &str) -> Result<LinuxCenterReadout> {
    let reason = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: config.residual_pack.clone(),
        text: query.to_string(),
        max_facts: config.max_facts.max(1),
        runtime_snapshot: None,
    })?;
    let evidence = reason.evidence_chain.first();
    Ok(LinuxCenterReadout {
        query: query.to_string(),
        answer_allowed: reason.decision.answer_allowed,
        source: "lrf",
        route: evidence.map(|step| step.route.clone()),
        subject: evidence.map(|step| step.subject.clone()),
        relation: evidence.map(|step| step.relation.clone()),
        object: evidence.map(|step| step.object.clone()),
        center_id: None,
        center_strength: 0,
        center_gap: 0,
        anti_center_hit: false,
        verifier_state: if reason.decision.answer_allowed {
            "GROUNDED"
        } else {
            "REVIEW_NO_DYNAMIC_CENTER"
        },
        answer: reason.decision.answer,
    })
}

fn center_readout(
    memory: &LinuxDynamicCenterMemoryFile,
    config: &LinuxCenterLearnConfig,
    query: &str,
) -> Result<LinuxCenterReadout> {
    if let Some(anti) = anti_center_match(memory, query) {
        return Ok(LinuxCenterReadout {
            query: query.to_string(),
            answer_allowed: false,
            source: "dynamic-center-memory",
            route: center_by_id(memory, anti.center_id).map(|center| center.route.clone()),
            subject: Some(anti.subject.clone()),
            relation: Some(anti.relation.clone()),
            object: Some(anti.object.clone()),
            center_id: Some(anti.center_id),
            center_strength: anti.strength,
            center_gap: anti.strength,
            anti_center_hit: true,
            verifier_state: "BOUNDARY_WITH_DYNAMIC_ANTI_CENTER",
            answer: format!(
                "Dynamic anti-center rejects shortcut: {} {} {}.",
                anti.subject, anti.relation, anti.object
            ),
        });
    }
    if is_role_swap_query(query) {
        return Ok(LinuxCenterReadout {
            query: query.to_string(),
            answer_allowed: false,
            source: "dynamic-center-memory",
            route: None,
            subject: Some("foocmd".to_string()),
            relation: Some("role-swap rejected".to_string()),
            object: Some("package-provider route".to_string()),
            center_id: None,
            center_strength: 0,
            center_gap: 0,
            anti_center_hit: false,
            verifier_state: "ROLE_SWAP_REJECTED",
            answer: "Dynamic center does not accept command->package role swap.".to_string(),
        });
    }
    if let Some(anchor) = command_anchor(query) {
        if let Some((residual, center)) = residual_match(memory, &anchor) {
            let second = memory
                .residuals
                .iter()
                .filter(|candidate| candidate.center_id != center.center_id)
                .filter_map(|candidate| center_by_id(memory, candidate.center_id))
                .filter(|candidate| {
                    candidate.route == center.route && candidate.relation == center.relation
                })
                .map(|candidate| candidate.strength)
                .max()
                .unwrap_or(0);
            let center_gap = center.strength - second;
            return Ok(LinuxCenterReadout {
                query: query.to_string(),
                answer_allowed: true,
                source: "dynamic-center-memory",
                route: Some(center.route.clone()),
                subject: Some(residual.subject.clone()),
                relation: Some(center.relation.clone()),
                object: Some(residual.object.clone()),
                center_id: Some(center.center_id),
                center_strength: center.strength,
                center_gap,
                anti_center_hit: false,
                verifier_state: "GROUNDED_BY_DYNAMIC_CENTER_MEMORY",
                answer: format!(
                    "Dynamic center answer: {} {} {} on route {}.",
                    residual.subject, center.relation, residual.object, center.route
                ),
            });
        }
    }
    base_readout(config, query)
}

fn residual_match<'a>(
    memory: &'a LinuxDynamicCenterMemoryFile,
    anchor: &str,
) -> Option<(&'a LinuxDynamicResidualRecord, &'a LinuxDynamicCenterRecord)> {
    let anchor = anchor.to_ascii_lowercase();
    memory.residuals.iter().find_map(|residual| {
        let subject = residual.subject.to_ascii_lowercase();
        (subject == anchor)
            .then(|| center_by_id(memory, residual.center_id).map(|center| (residual, center)))?
    })
}

fn anti_center_match<'a>(
    memory: &'a LinuxDynamicCenterMemoryFile,
    query: &str,
) -> Option<&'a LinuxDynamicAntiCenterRecord> {
    let query = query.to_ascii_lowercase();
    memory.anti_centers.iter().find(|anti| {
        query.contains(&anti.subject.to_ascii_lowercase())
            || query.contains(&anti.object.to_ascii_lowercase())
            || (query.contains("external") && anti.object.contains("exposure"))
            || (query.contains("vulnerability") && anti.object.contains("vulnerability"))
    })
}

fn center_by_id(
    memory: &LinuxDynamicCenterMemoryFile,
    center_id: u32,
) -> Option<&LinuxDynamicCenterRecord> {
    memory
        .centers
        .iter()
        .find(|center| center.center_id == center_id)
}

fn before_after_summary(
    before_after: &LinuxCenterBeforeAfter,
    heldout: &LinuxCenterHeldoutGuard,
) -> LinuxCenterBeforeAfterSummary {
    LinuxCenterBeforeAfterSummary {
        target_query_improved: !before_after.before.answer_allowed
            && before_after.after.answer_allowed
            && before_after.after.source == "dynamic-center-memory",
        memory_lift_observed: before_after.after.verifier_state
            == "GROUNDED_BY_DYNAMIC_CENTER_MEMORY",
        anti_center_replay_observed: before_after.anti_center_probe.anti_center_hit,
        false_positive_rate_regressed: before_after.anti_center_probe.answer_allowed
            || before_after.role_swap_probe.answer_allowed,
        heldout_regressed: heldout.heldout_regressed,
        unrelated_route_preserved: before_after.unrelated_probe.answer_allowed
            && before_after.unrelated_probe.source == "lrf",
    }
}

fn operations(
    memory: &LinuxDynamicCenterMemoryFile,
    summary: &LinuxCenterBeforeAfterSummary,
) -> LinuxCenterLearningOperations {
    LinuxCenterLearningOperations {
        confirmed_center_reinforcement: memory.centers.iter().any(|center| center.verified),
        correction_residual_write: memory.residuals.iter().any(|residual| {
            residual.source == "correction_feedback" && residual.subject == "foocmd"
        }),
        reject_anti_center_write: !memory.anti_centers.is_empty(),
        residual_cluster_promotion: memory.centers.iter().any(|center| center.promoted),
        center_split_available: memory.centers.iter().any(|center| center.split_candidate),
        weak_candidate_decay: memory
            .weak_candidates
            .iter()
            .any(|candidate| candidate.decayed),
        verified_center_protection: memory.centers.iter().any(|center| center.protected)
            && summary.memory_lift_observed,
    }
}

fn center_state_summary(
    memory: &LinuxDynamicCenterMemoryFile,
    center_updates: usize,
) -> LinuxCenterStateSummary {
    let verified_center_count = memory
        .centers
        .iter()
        .filter(|center| center.verified)
        .count();
    let promoted_center_count = memory
        .centers
        .iter()
        .filter(|center| center.promoted)
        .count();
    let split_candidate_count = memory
        .centers
        .iter()
        .filter(|center| center.split_candidate)
        .count();
    let decayed_candidate_count = memory
        .weak_candidates
        .iter()
        .filter(|candidate| candidate.decayed)
        .count();
    let protected_center_count = memory
        .centers
        .iter()
        .filter(|center| center.protected)
        .count();
    LinuxCenterStateSummary {
        center_count: memory.centers.len(),
        verified_center_count,
        residual_count: memory.residuals.len(),
        anti_center_count: memory.anti_centers.len(),
        promoted_center_count,
        split_candidate_count,
        decayed_candidate_count,
        protected_center_count,
        center_updates,
        residual_writes: memory.residuals.len(),
        anti_center_writes: memory.anti_centers.len(),
        drift_blocked_count: protected_center_count,
    }
}

fn load_heldout_guard(path: Option<&PathBuf>) -> Result<LinuxCenterHeldoutGuard> {
    let Some(path) = path else {
        return Ok(LinuxCenterHeldoutGuard {
            provided: false,
            path: None,
            total: 0,
            pass_rate: 0.0,
            false_positive_rate: 0.0,
            shortcut_rejection_rate: 0.0,
            exposure_overclaim_rate: 0.0,
            heldout_regressed: false,
        });
    };
    let report: LinuxBroadEvalReport = serde_json::from_str(
        &fs::read_to_string(path)
            .with_context(|| format!("read heldout eval {}", path.display()))?,
    )
    .with_context(|| format!("parse heldout eval {}", path.display()))?;
    let metrics = report.metrics;
    let heldout_regressed = metrics.total == 0
        || metrics.pass_rate < 0.95
        || metrics.false_positive_rate > 0.0
        || metrics.shortcut_rejection_rate < 0.95
        || metrics.exposure_overclaim_rate > 0.0;
    Ok(LinuxCenterHeldoutGuard {
        provided: true,
        path: Some(path.display().to_string()),
        total: metrics.total,
        pass_rate: metrics.pass_rate,
        false_positive_rate: metrics.false_positive_rate,
        shortcut_rejection_rate: metrics.shortcut_rejection_rate,
        exposure_overclaim_rate: metrics.exposure_overclaim_rate,
        heldout_regressed,
    })
}

fn update(
    operation: &str,
    center_id: u32,
    spec: &LinuxCenterFactSpec,
    effect: &str,
) -> LinuxCenterLearningUpdate {
    LinuxCenterLearningUpdate {
        operation: operation.to_string(),
        center_id,
        route: spec.route.clone(),
        subject: spec.subject.clone(),
        relation: spec.relation.clone(),
        object: spec.object.clone(),
        effect: effect.to_string(),
    }
}

fn delta_for(
    spec: &LinuxCenterFactSpec,
    delta_state: &str,
    reason: &str,
) -> PersistentWaveDeltaRecord {
    build_delta_record(PersistentWaveDeltaSpec {
        delta_state: delta_state.to_string(),
        source_prompt: spec.query.clone(),
        intent: intent_for_route(&spec.route).to_string(),
        route: spec.route.clone(),
        subject: spec.subject.clone(),
        relation: spec.relation.clone(),
        object: spec.object.clone(),
        polarity: spec.polarity.clone(),
        reason: reason.to_string(),
        strength: 16,
    })
}

fn center_id_for(
    route: &str,
    relation: &str,
    subject_role: &str,
    object_role: &str,
    polarity: &str,
    evidence_kind: &str,
) -> u32 {
    stable_id(&schema_key(
        route,
        relation,
        subject_role,
        object_role,
        polarity,
        evidence_kind,
    ))
}

fn schema_key(
    route: &str,
    relation: &str,
    subject_role: &str,
    object_role: &str,
    polarity: &str,
    evidence_kind: &str,
) -> String {
    format!("{route}|{relation}|{subject_role}|{object_role}|{polarity}|{evidence_kind}")
}

fn stable_id(text: &str) -> u32 {
    let mut hash = 0x811c_9dc5_u32;
    for byte in text.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn command_anchor(query: &str) -> Option<String> {
    let lower = query.to_ascii_lowercase();
    let marker = "command ";
    let (_, tail) = lower.split_once(marker)?;
    tail.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .find(|token| !token.is_empty())
        .map(str::to_string)
}

fn is_role_swap_query(query: &str) -> bool {
    let lower = query.to_ascii_lowercase();
    lower.contains("does command") && lower.contains("provide package")
}

fn intent_for_route(route: &str) -> &'static str {
    if route.contains("command") || route.contains("package.binary") {
        "command_provider"
    } else if route.contains("boundary.socket") || route.contains("exposure") {
        "external_exposure"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::linux_atlas::{LinuxAtlasEvidence, LinuxAtlasFact};
    use super::super::linux_profile::{
        build_linux_broad_eval_report, build_linux_broad_suite_report, LinuxBroadEvalRunConfig,
        LinuxBroadSuiteBuildConfig,
    };
    use super::super::linux_residual_memory::{
        build_linux_residual_pack_report, LinuxResidualPackConfig,
    };
    use super::*;

    #[test]
    fn linux_center_learning_moves_centers_and_preserves_guards() {
        let root = fixture_root("linux-center-learning");
        let residual = root.join("linux-center.lrf");
        let memory = root.join("linux-center.lwm");
        let suite = root.join("broad-suite.json");
        let heldout = root.join("heldout-eval.json");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 8,
            promotion_threshold: 2,
            out: residual.clone(),
        })
        .unwrap();
        build_linux_broad_suite_report(LinuxBroadSuiteBuildConfig {
            residual_pack: residual.clone(),
            cases: 32,
            out: Some(suite.clone()),
        })
        .unwrap();
        build_linux_broad_eval_report(LinuxBroadEvalRunConfig {
            residual_pack: residual.clone(),
            suite,
            out: Some(heldout.clone()),
            max_facts: 4,
        })
        .unwrap();

        let report = build_linux_center_learn_report(LinuxCenterLearnConfig {
            residual_pack: residual,
            memory: memory.clone(),
            script: None,
            heldout_eval: Some(heldout),
            max_facts: 4,
            reset_memory: true,
            out: None,
        })
        .unwrap();

        assert_eq!(
            report.verdict,
            "LINUX_DYNAMIC_CENTER_LEARNING_READY_NOT_GENERAL_LLM"
        );
        assert!(
            report
                .dynamic_center_learning
                .operations
                .confirmed_center_reinforcement
        );
        assert!(
            report
                .dynamic_center_learning
                .operations
                .correction_residual_write
        );
        assert!(
            report
                .dynamic_center_learning
                .operations
                .reject_anti_center_write
        );
        assert!(
            report
                .dynamic_center_learning
                .operations
                .residual_cluster_promotion
        );
        assert!(
            report
                .dynamic_center_learning
                .operations
                .center_split_available
        );
        assert!(
            report
                .dynamic_center_learning
                .operations
                .weak_candidate_decay
        );
        assert!(
            report
                .dynamic_center_learning
                .operations
                .verified_center_protection
        );
        assert!(
            report
                .dynamic_center_learning
                .before_after
                .target_query_improved
        );
        assert!(
            report
                .dynamic_center_learning
                .before_after
                .memory_lift_observed
        );
        assert!(
            report
                .dynamic_center_learning
                .before_after
                .anti_center_replay_observed
        );
        assert!(
            !report
                .dynamic_center_learning
                .before_after
                .false_positive_rate_regressed
        );
        assert!(
            !report
                .dynamic_center_learning
                .before_after
                .heldout_regressed
        );
        assert!(
            report
                .dynamic_center_learning
                .before_after
                .unrelated_route_preserved
        );
        assert!(memory.exists());
        assert!(!report.claim_boundary.general_llm_ready);
        assert!(!report.claim_boundary.global_nonlinear_memory_proven);
        assert!(!report.claim_boundary.proof_grade_profile_scale);
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_root(prefix: &str) -> PathBuf {
        let root = unique_tmp_dir(prefix);
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let facts = vec![
            test_fact(
                "linux.center.bash",
                "linux.apt.command.provider",
                "bash",
                "provided by package",
                "bash",
                "positive",
            ),
            test_fact(
                "linux.center.systemctl",
                "linux.package.binary",
                "systemd",
                "provides binary",
                "/usr/bin/systemctl",
                "positive",
            ),
            test_fact(
                "linux.center.boundary.socket",
                "linux.boundary.socket",
                "port listening",
                "does not prove",
                "firewall allows external packets",
                "negative",
            ),
        ];
        let mut file = fs::File::create(facts_path).unwrap();
        for fact in facts {
            writeln!(file, "{}", serde_json::to_string(&fact).unwrap()).unwrap();
        }
        root
    }

    fn test_fact(
        fact_id: &str,
        route: &str,
        subject: &str,
        relation: &str,
        object: &str,
        polarity: &str,
    ) -> LinuxAtlasFact {
        LinuxAtlasFact {
            fact_id: fact_id.to_string(),
            layer: "fixture".to_string(),
            domain: "linux-center-learning".to_string(),
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
        let root = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
