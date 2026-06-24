//! Ablation hooks for multi-profile density suites.
//!
//! This is intentionally a suite-level hook: it does not rebuild corpora or
//! rerun field inference. It asks whether the already-built proof survives
//! removing each profile and reports the linear-baseline duel exposed by the
//! suite metrics.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub(crate) const DENSITY_ABLATION_VERSION: &str = "llmwave-big-v-next-density-ablation";

#[derive(Clone)]
pub(crate) struct DensityAblationConfig {
    pub suite: PathBuf,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityAblationReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub suite_path: String,
    pub baseline_duel: DensityBaselineDuel,
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
