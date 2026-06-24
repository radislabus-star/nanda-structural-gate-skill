//! Ablation hooks for multi-profile density suites.
//!
//! This is intentionally a suite-level hook: it does not rebuild corpora or
//! rerun field inference. It asks whether the already-built proof survives
//! removing each profile and reports the linear-baseline duel exposed by the
//! suite metrics.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub(crate) const DENSITY_ABLATION_VERSION: &str = "llmwave-big-v-next-density-ablation";

#[derive(Clone)]
pub(crate) struct DensityAblationConfig {
    pub suite: PathBuf,
    pub out_hot_packet: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityAblationReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub suite_path: String,
    pub baseline_duel: DensityBaselineDuel,
    pub runtime_path: DensityRuntimePath,
    pub hot_artifact: DensityHotArtifact,
    pub ablations: Vec<ProfileAblation>,
    pub claim_boundary: DensityAblationClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityBaselineDuel {
    pub min_density_win_ratio: f64,
    pub median_density_win_ratio: f64,
    pub packed_beats_linear_baseline: bool,
    pub weakest_profile: Option<String>,
    pub strongest_profile: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityRuntimePath {
    pub runtime_path_kind: &'static str,
    pub l2_profile_surfaces: Vec<String>,
    pub l3_proof_axes: Vec<&'static str>,
    pub active_packet_records: usize,
    pub hot_loop_ready: bool,
    pub claim_boundary: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityHotArtifact {
    pub hot_packet_written: bool,
    pub hot_packet_path: Option<String>,
    pub hot_packet_bytes: usize,
    pub header_size_bytes: usize,
    pub record_size_bytes: usize,
    pub record_count: usize,
    pub contains_json: bool,
    pub hot_loop_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileAblation {
    pub removed_profile: String,
    pub remaining_profiles: usize,
    pub remaining_independent_sources: usize,
    pub suite_passes_without_profile: bool,
    pub impact: &'static str,
    pub reason: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityAblationClaimBoundary {
    pub density_ablation_implemented: bool,
    pub reruns_field_inference: bool,
    pub proves_nonlinear_memory: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
}

#[derive(Deserialize)]
struct SuiteArtifact {
    suite: SuiteSummary,
    profiles: Vec<SuiteProfile>,
    aggregate: SuiteAggregate,
}

#[derive(Deserialize)]
struct SuiteSummary {
    min_profiles_required: usize,
}

#[derive(Deserialize, Clone)]
struct SuiteProfile {
    profile: String,
    source_signature: String,
    profile_density_proven: bool,
    density_win_ratio: f64,
    heldout_pass_rate: f64,
    false_shortcut_rejection_rate: f64,
    collision_pressure: f64,
}

#[derive(Deserialize)]
struct SuiteAggregate {
    min_density_win_ratio: f64,
    median_density_win_ratio: f64,
}

pub(crate) fn build_density_ablation_report(
    config: DensityAblationConfig,
) -> Result<DensityAblationReport> {
    let raw = fs::read_to_string(&config.suite)
        .with_context(|| format!("read density suite {}", config.suite.display()))?;
    let suite: SuiteArtifact = serde_json::from_str(&raw)
        .with_context(|| format!("parse density suite {}", config.suite.display()))?;
    let weakest_profile = suite
        .profiles
        .iter()
        .min_by(|a, b| a.density_win_ratio.total_cmp(&b.density_win_ratio))
        .map(|profile| profile.profile.clone());
    let strongest_profile = suite
        .profiles
        .iter()
        .max_by(|a, b| a.density_win_ratio.total_cmp(&b.density_win_ratio))
        .map(|profile| profile.profile.clone());
    let baseline_duel = DensityBaselineDuel {
        min_density_win_ratio: suite.aggregate.min_density_win_ratio,
        median_density_win_ratio: suite.aggregate.median_density_win_ratio,
        packed_beats_linear_baseline: suite.aggregate.min_density_win_ratio > 1.0,
        weakest_profile,
        strongest_profile,
    };
    let runtime_path = DensityRuntimePath {
        runtime_path_kind: "density-suite-l2-l3-readonly-active-packet",
        l2_profile_surfaces: suite
            .profiles
            .iter()
            .map(|profile| profile.profile.clone())
            .collect(),
        l3_proof_axes: vec![
            "density_win",
            "heldout_quality",
            "anti_shortcut_rejection",
            "collision_pressure",
            "profile_independence",
        ],
        active_packet_records: suite.profiles.len(),
        hot_loop_ready: false,
        claim_boundary: "L2/L3 runtime path is represented as a read-only suite packet; it is not a hot-loop or cache-resident proof.",
    };
    let hot_artifact = write_hot_packet_if_requested(&config.out_hot_packet, &suite.profiles)?;
    let ablations = suite
        .profiles
        .iter()
        .map(|removed| {
            let remaining = suite
                .profiles
                .iter()
                .filter(|profile| profile.profile != removed.profile)
                .cloned()
                .collect::<Vec<_>>();
            let remaining_independent_sources = remaining
                .iter()
                .map(|profile| profile.source_signature.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            let suite_passes_without_profile =
                suite_passes(&remaining, suite.suite.min_profiles_required);
            let reason = if remaining.len() < suite.suite.min_profiles_required {
                "below_min_profile_count"
            } else if remaining_independent_sources != remaining.len() {
                "remaining_sources_not_independent"
            } else if !remaining
                .iter()
                .all(|profile| profile.profile_density_proven)
            {
                "remaining_profile_density_not_proven"
            } else if !remaining
                .iter()
                .all(|profile| profile.heldout_pass_rate >= 0.90)
            {
                "remaining_heldout_quality_low"
            } else if !remaining
                .iter()
                .all(|profile| profile.false_shortcut_rejection_rate >= 1.0)
            {
                "remaining_shortcut_rejection_low"
            } else if !remaining
                .iter()
                .all(|profile| profile.collision_pressure <= 0.35)
            {
                "remaining_collision_pressure_high"
            } else {
                "suite_still_passes"
            };
            ProfileAblation {
                removed_profile: removed.profile.clone(),
                remaining_profiles: remaining.len(),
                remaining_independent_sources,
                suite_passes_without_profile,
                impact: if suite_passes_without_profile {
                    "NON_CRITICAL"
                } else {
                    "CRITICAL"
                },
                reason,
            }
        })
        .collect::<Vec<_>>();
    let any_critical = ablations
        .iter()
        .any(|ablation| ablation.impact == "CRITICAL");
    let verdict = if any_critical {
        "DENSITY_ABLATION_HAS_CRITICAL_PROFILES"
    } else {
        "DENSITY_ABLATION_REDUNDANT_SUITE"
    };
    Ok(DensityAblationReport {
        mode: "llmwave-big-density-ablation",
        version: DENSITY_ABLATION_VERSION,
        verdict,
        suite_path: config.suite.display().to_string(),
        baseline_duel,
        runtime_path,
        hot_artifact,
        ablations,
        claim_boundary: DensityAblationClaimBoundary {
            density_ablation_implemented: true,
            reruns_field_inference: false,
            proves_nonlinear_memory: false,
            llm_ready: false,
            safe_claim: "Density ablation reports suite-level profile criticality and baseline-duel metrics only; it does not prove nonlinear memory or LLM readiness.",
        },
    })
}

fn suite_passes(profiles: &[SuiteProfile], min_profiles: usize) -> bool {
    if profiles.len() < min_profiles {
        return false;
    }
    let unique_sources = profiles
        .iter()
        .map(|profile| profile.source_signature.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    unique_sources == profiles.len()
        && profiles
            .iter()
            .all(|profile| profile.profile_density_proven)
        && profiles
            .iter()
            .all(|profile| profile.heldout_pass_rate >= 0.90)
        && profiles
            .iter()
            .all(|profile| profile.false_shortcut_rejection_rate >= 1.0)
        && profiles
            .iter()
            .all(|profile| profile.collision_pressure <= 0.35)
}

fn write_hot_packet_if_requested(
    out_hot_packet: &Option<PathBuf>,
    profiles: &[SuiteProfile],
) -> Result<DensityHotArtifact> {
    const HEADER_SIZE: usize = 16;
    const RECORD_SIZE: usize = 16;
    let Some(path) = out_hot_packet else {
        return Ok(DensityHotArtifact {
            hot_packet_written: false,
            hot_packet_path: None,
            hot_packet_bytes: 0,
            header_size_bytes: HEADER_SIZE,
            record_size_bytes: RECORD_SIZE,
            record_count: profiles.len(),
            contains_json: false,
            hot_loop_ready: false,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create density hot packet dir {}", parent.display()))?;
    }
    let mut bytes = Vec::with_capacity(HEADER_SIZE + profiles.len() * RECORD_SIZE);
    bytes.extend_from_slice(b"NDABLTN1");
    bytes.extend_from_slice(&(profiles.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(RECORD_SIZE as u32).to_le_bytes());
    for profile in profiles {
        bytes.extend_from_slice(&scale_metric(profile.density_win_ratio).to_le_bytes());
        bytes.extend_from_slice(&scale_metric(profile.heldout_pass_rate).to_le_bytes());
        bytes.extend_from_slice(&scale_metric(profile.false_shortcut_rejection_rate).to_le_bytes());
        bytes.extend_from_slice(&scale_metric(profile.collision_pressure).to_le_bytes());
    }
    let mut file = fs::File::create(path)
        .with_context(|| format!("create density hot packet {}", path.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("write density hot packet {}", path.display()))?;
    Ok(DensityHotArtifact {
        hot_packet_written: true,
        hot_packet_path: Some(path.display().to_string()),
        hot_packet_bytes: bytes.len(),
        header_size_bytes: HEADER_SIZE,
        record_size_bytes: RECORD_SIZE,
        record_count: profiles.len(),
        contains_json: false,
        hot_loop_ready: false,
    })
}

fn scale_metric(value: f64) -> u32 {
    (value.max(0.0) * 10_000.0).round().min(u32::MAX as f64) as u32
}
