//! Multi-profile density suite for the general nonlinear-memory claim.
//!
//! This command aggregates independent profile density artifacts. A single
//! Rust profile can prove only a Rust profile density claim; the general
//! nonlinear-memory claim needs multiple passing profiles with quality and
//! density wins.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const MULTI_PROFILE_DENSITY_SUITE_VERSION: &str =
    "llmwave-big-v-next-multi-profile-density-suite";

#[derive(Clone)]
pub(crate) struct MultiProfileDensitySuiteConfig {
    pub rust_density: Option<PathBuf>,
    pub profile_evidence: Vec<String>,
    pub out: Option<PathBuf>,
    pub min_profiles: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiProfileDensitySuiteReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub suite: MultiProfileDensitySuiteSummary,
    pub profiles: Vec<ProfileDensityResult>,
    pub aggregate: MultiProfileDensityAggregate,
    pub gates: MultiProfileDensityGates,
    pub output: MultiProfileDensityOutput,
    pub claim_boundary: MultiProfileDensityClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiProfileDensitySuiteSummary {
    pub suite_kind: &'static str,
    pub suite_hash: String,
    pub min_profiles_required: usize,
    pub profile_count: usize,
    pub passing_profile_count: usize,
    pub missing_profile_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensityResult {
    pub profile: String,
    pub evidence_path: Option<String>,
    pub present: bool,
    pub rust_density_profile_proven: bool,
    pub profile_density_proven: bool,
    pub density_win_ratio: f64,
    pub schema_reuse_ratio: f64,
    pub residual_saving_ratio: f64,
    pub heldout_pass_rate: f64,
    pub false_shortcut_rejection_rate: f64,
    pub collision_pressure: f64,
    pub verdict: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiProfileDensityAggregate {
    pub min_density_win_ratio: f64,
    pub median_density_win_ratio: f64,
    pub min_heldout_pass_rate: f64,
    pub max_collision_pressure: f64,
    pub average_schema_reuse_ratio: f64,
    pub average_residual_saving_ratio: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiProfileDensityGates {
    pub enough_profiles: bool,
    pub all_profiles_present: bool,
    pub all_profiles_density_proven: bool,
    pub heldout_quality_passed: bool,
    pub false_shortcuts_passed: bool,
    pub collision_pressure_bounded: bool,
    pub general_nonlinear_memory_proven: bool,
    pub llm_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiProfileDensityOutput {
    pub evidence_written: bool,
    pub evidence_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct MultiProfileDensityClaimBoundary {
    pub multi_profile_density_suite_implemented: bool,
    pub rust_profile_alone_is_not_general_proof: bool,
    pub general_nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Deserialize)]
struct ProfileEvidencePayload {
    density: ProfileDensityMetrics,
    quality: ProfileQualityMetrics,
    gates: ProfileGates,
}

#[derive(Deserialize)]
struct ProfileDensityMetrics {
    density_win_ratio: f64,
    schema_reuse_ratio: f64,
    residual_saving_ratio: f64,
}

#[derive(Deserialize)]
struct ProfileQualityMetrics {
    heldout_pass_rate: f64,
    false_shortcut_rejection_rate: f64,
    collision_pressure: f64,
}

#[derive(Deserialize)]
struct ProfileGates {
    #[serde(default)]
    rust_density_profile_proven: bool,
    #[serde(default)]
    profile_density_proven: bool,
    #[serde(default)]
    general_nonlinear_memory_proven: bool,
}

#[derive(Serialize)]
struct MultiProfileDensityWritePayload<'a> {
    mode: &'static str,
    version: &'static str,
    verdict: &'static str,
    suite: &'a MultiProfileDensitySuiteSummary,
    profiles: &'a [ProfileDensityResult],
    aggregate: &'a MultiProfileDensityAggregate,
    gates: &'a MultiProfileDensityGates,
    claim_boundary: &'a MultiProfileDensityClaimBoundary,
}

pub(crate) fn build_multi_profile_density_suite_report(
    config: MultiProfileDensitySuiteConfig,
) -> Result<MultiProfileDensitySuiteReport> {
    let min_profiles = config.min_profiles.max(2);
    let mut profiles = Vec::new();
    if let Some(path) = config.rust_density {
        profiles.push(read_profile("rust", &path)?);
    }
    for spec in config.profile_evidence {
        let (name, path) = parse_profile_spec(&spec)?;
        profiles.push(read_profile(&name, &path)?);
    }
    profiles.sort_by(|a, b| a.profile.cmp(&b.profile));
    let passing_profile_count = profiles
        .iter()
        .filter(|profile| profile.profile_density_proven)
        .count();
    let missing_profile_count = min_profiles.saturating_sub(profiles.len());
    let aggregate = aggregate_profiles(&profiles);
    let enough_profiles = profiles.len() >= min_profiles;
    let all_profiles_present = missing_profile_count == 0;
    let all_profiles_density_proven = enough_profiles
        && profiles
            .iter()
            .all(|profile| profile.profile_density_proven);
    let heldout_quality_passed = enough_profiles
        && profiles
            .iter()
            .all(|profile| profile.heldout_pass_rate >= 0.90);
    let false_shortcuts_passed = enough_profiles
        && profiles
            .iter()
            .all(|profile| profile.false_shortcut_rejection_rate >= 1.0);
    let collision_pressure_bounded = enough_profiles
        && profiles
            .iter()
            .all(|profile| profile.collision_pressure <= 0.35);
    let general_nonlinear_memory_proven = enough_profiles
        && all_profiles_present
        && all_profiles_density_proven
        && heldout_quality_passed
        && false_shortcuts_passed
        && collision_pressure_bounded;
    let gates = MultiProfileDensityGates {
        enough_profiles,
        all_profiles_present,
        all_profiles_density_proven,
        heldout_quality_passed,
        false_shortcuts_passed,
        collision_pressure_bounded,
        general_nonlinear_memory_proven,
        llm_ready: false,
    };
    let mut hash = Sha256::new();
    hash.update(MULTI_PROFILE_DENSITY_SUITE_VERSION.as_bytes());
    hash.update(min_profiles.to_le_bytes());
    for profile in &profiles {
        hash.update(profile.profile.as_bytes());
        hash.update(profile.density_win_ratio.to_le_bytes());
        hash.update(profile.heldout_pass_rate.to_le_bytes());
        hash.update(profile.profile_density_proven.to_string().as_bytes());
    }
    let suite = MultiProfileDensitySuiteSummary {
        suite_kind: "multi-profile-density-suite",
        suite_hash: format!("{:x}", hash.finalize()),
        min_profiles_required: min_profiles,
        profile_count: profiles.len(),
        passing_profile_count,
        missing_profile_count,
    };
    let verdict = if gates.general_nonlinear_memory_proven {
        "MULTI_PROFILE_NONLINEAR_MEMORY_PROVEN_NOT_LLM"
    } else if profiles.len() == 1 {
        "MULTI_PROFILE_DENSITY_BLOCKED_BY_SINGLE_PROFILE"
    } else {
        "MULTI_PROFILE_DENSITY_BLOCKED"
    };
    let blocked_by = blocked_by(&gates, &suite);
    let general_nonlinear_memory_proven = gates.general_nonlinear_memory_proven;
    let claim_boundary = MultiProfileDensityClaimBoundary {
        multi_profile_density_suite_implemented: true,
        rust_profile_alone_is_not_general_proof: true,
        general_nonlinear_memory_proven,
        llm_ready: false,
        safe_claim: if general_nonlinear_memory_proven {
            "Multiple independent density profiles passed. This supports the general nonlinear-memory claim, but not LLM readiness."
        } else {
            "Multi-profile density evidence is incomplete; a single passing Rust profile is not enough for the general nonlinear-memory claim."
        },
        blocked_by,
    };
    let output = write_suite_if_requested(
        &config.out,
        verdict,
        &suite,
        &profiles,
        &aggregate,
        &gates,
        &claim_boundary,
    )?;

    Ok(MultiProfileDensitySuiteReport {
        mode: "llmwave-big-multi-profile-density-suite",
        version: MULTI_PROFILE_DENSITY_SUITE_VERSION,
        verdict,
        suite,
        profiles,
        aggregate,
        gates,
        output,
        claim_boundary,
    })
}

fn read_profile(profile: &str, path: &PathBuf) -> Result<ProfileDensityResult> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read profile density evidence {}", path.display()))?;
    let payload: ProfileEvidencePayload = serde_json::from_str(&raw)
        .with_context(|| format!("parse profile density evidence {}", path.display()))?;
    let profile_density_proven = payload.gates.rust_density_profile_proven
        || payload.gates.profile_density_proven
        || payload.gates.general_nonlinear_memory_proven;
    let mut blocked_by = Vec::new();
    if !profile_density_proven {
        blocked_by.push("profile_density_not_proven");
    }
    if payload.quality.heldout_pass_rate < 0.90 {
        blocked_by.push("heldout_quality_below_threshold");
    }
    if payload.quality.false_shortcut_rejection_rate < 1.0 {
        blocked_by.push("false_shortcut_rejection_missing");
    }
    if payload.quality.collision_pressure > 0.35 {
        blocked_by.push("collision_pressure_too_high");
    }
    Ok(ProfileDensityResult {
        profile: profile.to_string(),
        evidence_path: Some(path.display().to_string()),
        present: true,
        rust_density_profile_proven: payload.gates.rust_density_profile_proven,
        profile_density_proven,
        density_win_ratio: payload.density.density_win_ratio,
        schema_reuse_ratio: payload.density.schema_reuse_ratio,
        residual_saving_ratio: payload.density.residual_saving_ratio,
        heldout_pass_rate: payload.quality.heldout_pass_rate,
        false_shortcut_rejection_rate: payload.quality.false_shortcut_rejection_rate,
        collision_pressure: payload.quality.collision_pressure,
        verdict: if profile_density_proven {
            "PROFILE_DENSITY_PROVEN"
        } else {
            "PROFILE_DENSITY_REVIEW"
        },
        blocked_by,
    })
}

fn parse_profile_spec(spec: &str) -> Result<(String, PathBuf)> {
    let Some((name, path)) = spec.split_once('=') else {
        bail!("profile evidence must use name=path, got {spec}");
    };
    let name = name.trim();
    let path = path.trim();
    if name.is_empty() || path.is_empty() {
        bail!("profile evidence must use non-empty name=path, got {spec}");
    }
    Ok((name.to_string(), PathBuf::from(path)))
}

fn aggregate_profiles(profiles: &[ProfileDensityResult]) -> MultiProfileDensityAggregate {
    let mut density = profiles
        .iter()
        .map(|profile| profile.density_win_ratio)
        .collect::<Vec<_>>();
    density.sort_by(f64::total_cmp);
    let min_density_win_ratio = density.first().copied().unwrap_or(0.0);
    let median_density_win_ratio = if density.is_empty() {
        0.0
    } else {
        density[density.len() / 2]
    };
    let min_heldout_pass_rate = profiles
        .iter()
        .map(|profile| profile.heldout_pass_rate)
        .min_by(f64::total_cmp)
        .unwrap_or(0.0);
    let max_collision_pressure = profiles
        .iter()
        .map(|profile| profile.collision_pressure)
        .max_by(f64::total_cmp)
        .unwrap_or(0.0);
    MultiProfileDensityAggregate {
        min_density_win_ratio: round4(min_density_win_ratio),
        median_density_win_ratio: round4(median_density_win_ratio),
        min_heldout_pass_rate: round4(min_heldout_pass_rate),
        max_collision_pressure: round4(max_collision_pressure),
        average_schema_reuse_ratio: average(
            profiles
                .iter()
                .map(|profile| profile.schema_reuse_ratio)
                .collect::<Vec<_>>()
                .as_slice(),
        ),
        average_residual_saving_ratio: average(
            profiles
                .iter()
                .map(|profile| profile.residual_saving_ratio)
                .collect::<Vec<_>>()
                .as_slice(),
        ),
    }
}

fn blocked_by(
    gates: &MultiProfileDensityGates,
    suite: &MultiProfileDensitySuiteSummary,
) -> Vec<&'static str> {
    let mut blocked = Vec::new();
    if !gates.enough_profiles {
        blocked.push("not_enough_independent_profiles");
    }
    if suite.missing_profile_count > 0 {
        blocked.push("profile_evidence_missing");
    }
    if !gates.all_profiles_density_proven {
        blocked.push("some_profiles_not_density_proven");
    }
    if !gates.heldout_quality_passed {
        blocked.push("multi_profile_heldout_quality_missing");
    }
    if !gates.false_shortcuts_passed {
        blocked.push("multi_profile_false_shortcut_rejection_missing");
    }
    if !gates.collision_pressure_bounded {
        blocked.push("multi_profile_collision_pressure_too_high");
    }
    blocked
}

fn write_suite_if_requested(
    out: &Option<PathBuf>,
    verdict: &'static str,
    suite: &MultiProfileDensitySuiteSummary,
    profiles: &[ProfileDensityResult],
    aggregate: &MultiProfileDensityAggregate,
    gates: &MultiProfileDensityGates,
    claim_boundary: &MultiProfileDensityClaimBoundary,
) -> Result<MultiProfileDensityOutput> {
    let Some(path) = out else {
        return Ok(MultiProfileDensityOutput {
            evidence_written: false,
            evidence_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "create multi-profile density output dir {}",
                parent.display()
            )
        })?;
    }
    let payload = MultiProfileDensityWritePayload {
        mode: "llmwave-big-multi-profile-density-suite",
        version: MULTI_PROFILE_DENSITY_SUITE_VERSION,
        verdict,
        suite,
        profiles,
        aggregate,
        gates,
        claim_boundary,
    };
    let json =
        serde_json::to_vec_pretty(&payload).context("serialize multi-profile density suite")?;
    let mut file = fs::File::create(path)
        .with_context(|| format!("create multi-profile density suite {}", path.display()))?;
    file.write_all(&json)
        .with_context(|| format!("write multi-profile density suite {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish multi-profile density suite {}", path.display()))?;
    Ok(MultiProfileDensityOutput {
        evidence_written: true,
        evidence_path: Some(path.display().to_string()),
    })
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        round4(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_profile_is_blocked() {
        let profiles = vec![ProfileDensityResult {
            profile: "rust".to_string(),
            evidence_path: None,
            present: true,
            rust_density_profile_proven: true,
            profile_density_proven: true,
            density_win_ratio: 3.0,
            schema_reuse_ratio: 10.0,
            residual_saving_ratio: 0.6,
            heldout_pass_rate: 0.98,
            false_shortcut_rejection_rate: 1.0,
            collision_pressure: 0.01,
            verdict: "PROFILE_DENSITY_PROVEN",
            blocked_by: Vec::new(),
        }];
        let aggregate = aggregate_profiles(&profiles);
        assert_eq!(aggregate.min_density_win_ratio, 3.0);
        let gates = MultiProfileDensityGates {
            enough_profiles: false,
            all_profiles_present: false,
            all_profiles_density_proven: false,
            heldout_quality_passed: false,
            false_shortcuts_passed: false,
            collision_pressure_bounded: false,
            general_nonlinear_memory_proven: false,
            llm_ready: false,
        };
        let suite = MultiProfileDensitySuiteSummary {
            suite_kind: "multi-profile-density-suite",
            suite_hash: "test".to_string(),
            min_profiles_required: 3,
            profile_count: 1,
            passing_profile_count: 1,
            missing_profile_count: 2,
        };
        assert!(blocked_by(&gates, &suite).contains(&"not_enough_independent_profiles"));
    }
}
