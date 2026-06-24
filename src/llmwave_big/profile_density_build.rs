//! Generic density-profile builder for multi-profile nonlinear-memory evidence.
//!
//! The Rust profile has its own structural pipeline. This module adapts the
//! existing corpus-driven nonlinear-memory evaluator into the same evidence
//! shape, so independent domains can be fed into the multi-profile suite
//! without copying Rust artifacts.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::nonlinear_memory_eval::{build_nonlinear_memory_eval_report, NonlinearProofPolicyKind};

pub(crate) const PROFILE_DENSITY_BUILD_VERSION: &str = "llmwave-big-v-next-profile-density-build";

#[derive(Clone)]
pub(crate) struct ProfileDensityBuildConfig {
    pub profile: String,
    pub corpus: PathBuf,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensityBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub profile: String,
    pub source: ProfileDensitySource,
    pub density: ProfileDensityMetrics,
    pub quality: ProfileDensityQuality,
    pub gates: ProfileDensityGates,
    pub output: ProfileDensityOutput,
    pub claim_boundary: ProfileDensityClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensitySource {
    pub source_kind: &'static str,
    pub corpus_path: String,
    pub corpus_hash: String,
    pub source_version: Option<String>,
    pub source_name: Option<String>,
    pub fact_count: usize,
    pub heldout_count: usize,
    pub negative_count: usize,
    pub noise_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensityMetrics {
    pub linear_baseline_bytes: usize,
    pub fixed_basis_standalone_bytes: usize,
    pub fixed_basis_amortized_bytes: usize,
    pub density_win_ratio: f64,
    pub standalone_density_win_ratio: f64,
    pub schema_reuse_ratio: f64,
    pub residual_saving_ratio: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensityQuality {
    pub heldout_pass_rate: f64,
    pub false_shortcut_rejection_rate: f64,
    pub noise_reject_rate: f64,
    pub collision_pressure: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensityGates {
    pub profile_density_proven: bool,
    pub rust_density_profile_proven: bool,
    pub general_nonlinear_memory_proven: bool,
    pub external_corpus_present: bool,
    pub heldout_quality_passed: bool,
    pub false_shortcut_rejection_passed: bool,
    pub noise_controls_passed: bool,
    pub collision_pressure_bounded: bool,
    pub amortized_wave_beats_linear: bool,
    pub standalone_wave_beats_linear: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensityOutput {
    pub evidence_written: bool,
    pub evidence_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDensityClaimBoundary {
    pub profile_density_build_implemented: bool,
    pub profile_density_proven: bool,
    pub general_nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

pub(crate) fn build_profile_density_report(
    config: ProfileDensityBuildConfig,
) -> Result<ProfileDensityBuildReport> {
    let profile = config.profile.trim();
    let profile = if profile.is_empty() {
        "unnamed-profile"
    } else {
        profile
    }
    .to_string();
    let corpus_hash = file_hash(&config.corpus)?;
    let eval = build_nonlinear_memory_eval_report(
        Some(&config.corpus),
        NonlinearProofPolicyKind::ScaleAmortized,
    )?;
    let memory = &eval.corpus_driven_memory;
    let external = &eval.external_corpus;
    let collision_pressure = round4(1.0 - external.noise_reject_rate);
    let density = ProfileDensityMetrics {
        linear_baseline_bytes: memory.linear_baseline.bytes_total,
        fixed_basis_standalone_bytes: memory.fixed_basis_standalone.bytes_total,
        fixed_basis_amortized_bytes: memory.fixed_basis_amortized.bytes_total,
        density_win_ratio: memory.delta.amortized_bytes_per_useful_fact_gain,
        standalone_density_win_ratio: memory.delta.standalone_bytes_per_useful_fact_gain,
        schema_reuse_ratio: memory.fixed_basis_amortized.schema_reuse_ratio,
        residual_saving_ratio: memory.fixed_basis_amortized.residual_saving_ratio,
    };
    let quality = ProfileDensityQuality {
        heldout_pass_rate: external.heldout_pass_rate,
        false_shortcut_rejection_rate: external.negative_reject_rate,
        noise_reject_rate: external.noise_reject_rate,
        collision_pressure,
    };
    let gates = ProfileDensityGates {
        profile_density_proven: memory.gates.corpus_driven_nonlinear_density_observed,
        rust_density_profile_proven: false,
        general_nonlinear_memory_proven: false,
        external_corpus_present: external.external_corpus_present,
        heldout_quality_passed: quality.heldout_pass_rate >= 0.90,
        false_shortcut_rejection_passed: quality.false_shortcut_rejection_rate >= 1.0,
        noise_controls_passed: quality.noise_reject_rate >= 1.0,
        collision_pressure_bounded: quality.collision_pressure <= 0.35,
        amortized_wave_beats_linear: memory.gates.amortized_wave_beats_linear,
        standalone_wave_beats_linear: memory.gates.standalone_wave_beats_linear,
    };
    let verdict = if gates.profile_density_proven {
        "PROFILE_DENSITY_PROVEN_NOT_GENERAL_LLM"
    } else {
        "PROFILE_DENSITY_REVIEW"
    };
    let source = ProfileDensitySource {
        source_kind: "external-relation-corpus",
        corpus_path: config.corpus.display().to_string(),
        corpus_hash,
        source_version: external.version.clone(),
        source_name: external.source.clone(),
        fact_count: external.fact_count,
        heldout_count: external.heldout_count,
        negative_count: external.negative_count,
        noise_count: external.noise_count,
    };
    let blocked_by = blocked_by(&gates);
    let claim_boundary = ProfileDensityClaimBoundary {
        profile_density_build_implemented: true,
        profile_density_proven: gates.profile_density_proven,
        general_nonlinear_memory_proven: false,
        llm_ready: false,
        safe_claim: if gates.profile_density_proven {
            "This independent profile passed local density, held-out, negative, and noise gates. It is profile evidence only, not a general nonlinear-memory or LLM proof."
        } else {
            "This profile did not pass the local density gates; use it as review evidence only."
        },
        blocked_by,
    };
    let write_input = ProfileDensityWriteInput {
        out: &config.out,
        verdict,
        profile: &profile,
        source: &source,
        density: &density,
        quality: &quality,
        gates: &gates,
        claim_boundary: &claim_boundary,
    };
    let output = write_profile_if_requested(write_input)?;

    Ok(ProfileDensityBuildReport {
        mode: "llmwave-big-profile-density-build",
        version: PROFILE_DENSITY_BUILD_VERSION,
        verdict,
        profile,
        source,
        density,
        quality,
        gates,
        output,
        claim_boundary,
    })
}

struct ProfileDensityWriteInput<'a> {
    out: &'a Option<PathBuf>,
    verdict: &'static str,
    profile: &'a str,
    source: &'a ProfileDensitySource,
    density: &'a ProfileDensityMetrics,
    quality: &'a ProfileDensityQuality,
    gates: &'a ProfileDensityGates,
    claim_boundary: &'a ProfileDensityClaimBoundary,
}

fn write_profile_if_requested(input: ProfileDensityWriteInput<'_>) -> Result<ProfileDensityOutput> {
    let Some(path) = input.out else {
        return Ok(ProfileDensityOutput {
            evidence_written: false,
            evidence_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create profile density output dir {}", parent.display()))?;
    }
    let payload = ProfileDensityWritePayload {
        mode: "llmwave-big-profile-density-build",
        version: PROFILE_DENSITY_BUILD_VERSION,
        verdict: input.verdict,
        profile: input.profile,
        source: input.source,
        density: input.density,
        quality: input.quality,
        gates: input.gates,
        claim_boundary: input.claim_boundary,
    };
    let json = serde_json::to_vec_pretty(&payload).context("serialize profile density artifact")?;
    let mut file = fs::File::create(path)
        .with_context(|| format!("create profile density artifact {}", path.display()))?;
    file.write_all(&json)
        .with_context(|| format!("write profile density artifact {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish profile density artifact {}", path.display()))?;
    Ok(ProfileDensityOutput {
        evidence_written: true,
        evidence_path: Some(path.display().to_string()),
    })
}

#[derive(Serialize)]
struct ProfileDensityWritePayload<'a> {
    mode: &'static str,
    version: &'static str,
    verdict: &'static str,
    profile: &'a str,
    source: &'a ProfileDensitySource,
    density: &'a ProfileDensityMetrics,
    quality: &'a ProfileDensityQuality,
    gates: &'a ProfileDensityGates,
    claim_boundary: &'a ProfileDensityClaimBoundary,
}

fn blocked_by(gates: &ProfileDensityGates) -> Vec<&'static str> {
    let mut blocked = Vec::new();
    if !gates.external_corpus_present {
        blocked.push("external_corpus_gate_missing");
    }
    if !gates.amortized_wave_beats_linear {
        blocked.push("amortized_density_win_missing");
    }
    if !gates.heldout_quality_passed {
        blocked.push("heldout_quality_below_threshold");
    }
    if !gates.false_shortcut_rejection_passed {
        blocked.push("false_shortcut_rejection_missing");
    }
    if !gates.noise_controls_passed {
        blocked.push("noise_controls_missing");
    }
    if !gates.collision_pressure_bounded {
        blocked.push("collision_pressure_too_high");
    }
    blocked
}

fn file_hash(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read corpus {}", path.display()))?;
    let mut hash = Sha256::new();
    hash.update(PROFILE_DENSITY_BUILD_VERSION.as_bytes());
    hash.update(&bytes);
    Ok(format!("{:x}", hash.finalize()))
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_by_requires_amortized_density_and_controls() {
        let gates = ProfileDensityGates {
            profile_density_proven: false,
            rust_density_profile_proven: false,
            general_nonlinear_memory_proven: false,
            external_corpus_present: true,
            heldout_quality_passed: true,
            false_shortcut_rejection_passed: true,
            noise_controls_passed: true,
            collision_pressure_bounded: true,
            amortized_wave_beats_linear: false,
            standalone_wave_beats_linear: false,
        };
        assert_eq!(blocked_by(&gates), vec!["amortized_density_win_missing"]);
    }
}
