//! Quality doctor for multi-profile density proof artifacts.
//!
//! The multi-profile suite answers "did the configured gates pass?". This
//! doctor answers a different question: "is the evidence broad and strong
//! enough to trust the public nonlinear-memory claim?". A passing suite can
//! still be weak when profiles are tiny, too similar, or only fixture-scale.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub(crate) const DENSITY_PROOF_DOCTOR_VERSION: &str = "llmwave-big-v-next-density-proof-doctor";

#[derive(Clone)]
pub(crate) struct DensityProofDoctorConfig {
    pub suite: PathBuf,
    pub out: Option<PathBuf>,
    pub medium_profile_min: usize,
    pub strong_profile_min: usize,
    pub min_fact_count: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityProofDoctorReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub suite_path: String,
    pub profile_diversity: ProfileDiversityReport,
    pub proof_quality: ProofQualityReport,
    pub gates: DensityProofDoctorGates,
    pub output: DensityProofDoctorOutput,
    pub claim_boundary: DensityProofDoctorClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProfileDiversityReport {
    pub profile_count: usize,
    pub unique_source_signatures: usize,
    pub unique_source_names: usize,
    pub unique_source_kinds: usize,
    pub smallest_profile_fact_count: Option<usize>,
    pub small_profile_count: usize,
    pub profiles: Vec<DoctorProfileSummary>,
}

#[derive(Serialize, Clone)]
pub(crate) struct DoctorProfileSummary {
    pub profile: String,
    pub source_name: Option<String>,
    pub source_kind: Option<String>,
    pub source_fact_count: Option<usize>,
    pub profile_density_proven: bool,
    pub density_win_ratio: f64,
    pub heldout_pass_rate: f64,
    pub false_shortcut_rejection_rate: f64,
    pub collision_pressure: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProofQualityReport {
    pub state: &'static str,
    pub quality_score: f64,
    pub weak_spots: Vec<&'static str>,
    pub next_profile_needed: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityProofDoctorGates {
    pub suite_general_nonlinear_memory_proven: bool,
    pub independent_profile_sources: bool,
    pub enough_profiles_for_medium: bool,
    pub enough_profiles_for_strong: bool,
    pub profile_sources_diverse: bool,
    pub profile_facts_large_enough: bool,
    pub density_margin_strong: bool,
    pub heldout_quality_strong: bool,
    pub false_shortcuts_strong: bool,
    pub collision_pressure_low: bool,
    pub density_proof_doctor_medium_or_better: bool,
    pub density_proof_doctor_strong: bool,
    pub llm_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityProofDoctorOutput {
    pub evidence_written: bool,
    pub evidence_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct DensityProofDoctorClaimBoundary {
    pub density_proof_doctor_implemented: bool,
    pub medium_or_better_profile_evidence: bool,
    pub strong_profile_evidence: bool,
    pub general_nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Deserialize)]
struct SuiteArtifact {
    suite: SuiteSummary,
    profiles: Vec<SuiteProfile>,
    aggregate: SuiteAggregate,
    gates: SuiteGates,
}

#[derive(Deserialize)]
struct SuiteSummary {
    profile_count: usize,
}

#[derive(Deserialize)]
struct SuiteProfile {
    profile: String,
    #[serde(default)]
    source_signature: Option<String>,
    #[serde(default)]
    source_name: Option<String>,
    #[serde(default)]
    source_kind: Option<String>,
    #[serde(default)]
    source_fact_count: Option<usize>,
    profile_density_proven: bool,
    density_win_ratio: f64,
    heldout_pass_rate: f64,
    false_shortcut_rejection_rate: f64,
    collision_pressure: f64,
}

#[derive(Deserialize)]
struct SuiteAggregate {
    min_density_win_ratio: f64,
    min_heldout_pass_rate: f64,
    max_collision_pressure: f64,
}

#[derive(Deserialize)]
struct SuiteGates {
    #[serde(default)]
    independent_profile_sources: bool,
    general_nonlinear_memory_proven: bool,
}

pub(crate) fn build_density_proof_doctor_report(
    config: DensityProofDoctorConfig,
) -> Result<DensityProofDoctorReport> {
    let raw = fs::read_to_string(&config.suite)
        .with_context(|| format!("read density proof suite {}", config.suite.display()))?;
    let suite: SuiteArtifact = serde_json::from_str(&raw)
        .with_context(|| format!("parse density proof suite {}", config.suite.display()))?;
    let medium_profile_min = config.medium_profile_min.max(2);
    let strong_profile_min = config.strong_profile_min.max(medium_profile_min);
    let min_fact_count = config.min_fact_count.max(1);
    let profile_count = suite.suite.profile_count;
    let unique_source_signatures = suite
        .profiles
        .iter()
        .filter_map(|profile| profile.source_signature.as_deref())
        .collect::<BTreeSet<_>>()
        .len();
    let unique_source_names = suite
        .profiles
        .iter()
        .filter_map(|profile| profile.source_name.as_deref())
        .collect::<BTreeSet<_>>()
        .len();
    let unique_source_kinds = suite
        .profiles
        .iter()
        .filter_map(|profile| profile.source_kind.as_deref())
        .collect::<BTreeSet<_>>()
        .len();
    let smallest_profile_fact_count = suite
        .profiles
        .iter()
        .filter_map(|profile| profile.source_fact_count)
        .min();
    let small_profile_count = suite
        .profiles
        .iter()
        .filter(|profile| profile.source_fact_count.unwrap_or(0) < min_fact_count)
        .count();
    let profile_sources_diverse = unique_source_signatures == profile_count
        && unique_source_names >= medium_profile_min.saturating_sub(1);
    let profile_facts_large_enough =
        profile_count > 0 && small_profile_count == 0 && smallest_profile_fact_count.is_some();
    let density_margin_strong = suite.aggregate.min_density_win_ratio >= 1.50;
    let heldout_quality_strong = suite.aggregate.min_heldout_pass_rate >= 0.95;
    let false_shortcuts_strong = suite
        .profiles
        .iter()
        .all(|profile| profile.false_shortcut_rejection_rate >= 1.0);
    let collision_pressure_low = suite.aggregate.max_collision_pressure <= 0.10;
    let enough_profiles_for_medium = profile_count >= medium_profile_min;
    let enough_profiles_for_strong = profile_count >= strong_profile_min;
    let mut score = 0.0;
    for passed in [
        suite.gates.general_nonlinear_memory_proven,
        suite.gates.independent_profile_sources,
        enough_profiles_for_medium,
        profile_sources_diverse,
        profile_facts_large_enough,
        density_margin_strong,
        heldout_quality_strong,
        false_shortcuts_strong,
        collision_pressure_low,
    ] {
        if passed {
            score += 1.0;
        }
    }
    let quality_score = round4(score / 9.0);
    let weak_spots = weak_spots(WeakSpotInputs {
        suite_general: suite.gates.general_nonlinear_memory_proven,
        independent_profile_sources: suite.gates.independent_profile_sources,
        enough_profiles_for_medium,
        enough_profiles_for_strong,
        profile_sources_diverse,
        profile_facts_large_enough,
        density_margin_strong,
        heldout_quality_strong,
        false_shortcuts_strong,
        collision_pressure_low,
    });
    let state = if !suite.gates.general_nonlinear_memory_proven {
        "DENSITY_PROOF_BLOCKED"
    } else if enough_profiles_for_strong
        && profile_sources_diverse
        && profile_facts_large_enough
        && density_margin_strong
        && heldout_quality_strong
        && false_shortcuts_strong
        && collision_pressure_low
    {
        "DENSITY_PROOF_STRONG"
    } else if enough_profiles_for_medium
        && suite.gates.independent_profile_sources
        && profile_sources_diverse
        && profile_facts_large_enough
        && density_margin_strong
        && heldout_quality_strong
        && false_shortcuts_strong
        && collision_pressure_low
    {
        "DENSITY_PROOF_MEDIUM"
    } else {
        "DENSITY_PROOF_WEAK"
    };
    let gates = DensityProofDoctorGates {
        suite_general_nonlinear_memory_proven: suite.gates.general_nonlinear_memory_proven,
        independent_profile_sources: suite.gates.independent_profile_sources,
        enough_profiles_for_medium,
        enough_profiles_for_strong,
        profile_sources_diverse,
        profile_facts_large_enough,
        density_margin_strong,
        heldout_quality_strong,
        false_shortcuts_strong,
        collision_pressure_low,
        density_proof_doctor_medium_or_better: matches!(
            state,
            "DENSITY_PROOF_MEDIUM" | "DENSITY_PROOF_STRONG"
        ),
        density_proof_doctor_strong: state == "DENSITY_PROOF_STRONG",
        llm_ready: false,
    };
    let diversity = ProfileDiversityReport {
        profile_count,
        unique_source_signatures,
        unique_source_names,
        unique_source_kinds,
        smallest_profile_fact_count,
        small_profile_count,
        profiles: suite
            .profiles
            .iter()
            .map(|profile| DoctorProfileSummary {
                profile: profile.profile.clone(),
                source_name: profile.source_name.clone(),
                source_kind: profile.source_kind.clone(),
                source_fact_count: profile.source_fact_count,
                profile_density_proven: profile.profile_density_proven,
                density_win_ratio: profile.density_win_ratio,
                heldout_pass_rate: profile.heldout_pass_rate,
                false_shortcut_rejection_rate: profile.false_shortcut_rejection_rate,
                collision_pressure: profile.collision_pressure,
            })
            .collect(),
    };
    let proof_quality = ProofQualityReport {
        state,
        quality_score,
        weak_spots: weak_spots.clone(),
        next_profile_needed: next_profile_needed(state, &weak_spots),
    };
    let claim_boundary = DensityProofDoctorClaimBoundary {
        density_proof_doctor_implemented: true,
        medium_or_better_profile_evidence: gates.density_proof_doctor_medium_or_better,
        strong_profile_evidence: gates.density_proof_doctor_strong,
        general_nonlinear_memory_proven: gates.density_proof_doctor_strong,
        llm_ready: false,
        safe_claim: if gates.density_proof_doctor_strong {
            "Density proof evidence is strong across independent profiles, but LLM readiness remains blocked."
        } else if gates.density_proof_doctor_medium_or_better {
            "Density proof evidence is medium quality; treat nonlinear memory as supported but not finally proven."
        } else {
            "Density proof evidence is weak or blocked; do not use it as a broad nonlinear-memory proof."
        },
        blocked_by: weak_spots.clone(),
    };
    let output = write_if_requested(
        &config.out,
        state,
        &config.suite.display().to_string(),
        &diversity,
        &proof_quality,
        &gates,
        &claim_boundary,
    )?;
    Ok(DensityProofDoctorReport {
        mode: "llmwave-big-density-proof-doctor",
        version: DENSITY_PROOF_DOCTOR_VERSION,
        verdict: state,
        suite_path: config.suite.display().to_string(),
        profile_diversity: diversity,
        proof_quality,
        gates,
        output,
        claim_boundary,
    })
}

struct WeakSpotInputs {
    suite_general: bool,
    independent_profile_sources: bool,
    enough_profiles_for_medium: bool,
    enough_profiles_for_strong: bool,
    profile_sources_diverse: bool,
    profile_facts_large_enough: bool,
    density_margin_strong: bool,
    heldout_quality_strong: bool,
    false_shortcuts_strong: bool,
    collision_pressure_low: bool,
}

fn weak_spots(input: WeakSpotInputs) -> Vec<&'static str> {
    let mut spots = Vec::new();
    if !input.suite_general {
        spots.push("multi_profile_suite_not_passing");
    }
    if !input.independent_profile_sources {
        spots.push("profile_sources_not_independent");
    }
    if !input.enough_profiles_for_medium {
        spots.push("not_enough_profiles_for_medium_quality");
    }
    if !input.enough_profiles_for_strong {
        spots.push("not_enough_profiles_for_strong_quality");
    }
    if !input.profile_sources_diverse {
        spots.push("profile_source_diversity_too_low");
    }
    if !input.profile_facts_large_enough {
        spots.push("profile_corpora_too_small_or_unknown");
    }
    if !input.density_margin_strong {
        spots.push("density_margin_too_low");
    }
    if !input.heldout_quality_strong {
        spots.push("heldout_quality_too_low");
    }
    if !input.false_shortcuts_strong {
        spots.push("false_shortcut_rejection_too_low");
    }
    if !input.collision_pressure_low {
        spots.push("collision_pressure_too_high");
    }
    spots
}

fn next_profile_needed(state: &str, weak_spots: &[&'static str]) -> &'static str {
    if state == "DENSITY_PROOF_STRONG" {
        "none"
    } else if weak_spots.contains(&"profile_corpora_too_small_or_unknown") {
        "larger-real-corpus-profile"
    } else if weak_spots.contains(&"profile_source_diversity_too_low") {
        "new-domain-profile"
    } else if weak_spots.contains(&"false_shortcut_rejection_too_low")
        || weak_spots.contains(&"collision_pressure_too_high")
    {
        "adversarial-noise-profile"
    } else {
        "natural-text-or-adversarial-profile"
    }
}

fn write_if_requested(
    out: &Option<PathBuf>,
    verdict: &'static str,
    suite_path: &str,
    diversity: &ProfileDiversityReport,
    proof_quality: &ProofQualityReport,
    gates: &DensityProofDoctorGates,
    claim_boundary: &DensityProofDoctorClaimBoundary,
) -> Result<DensityProofDoctorOutput> {
    let Some(path) = out else {
        return Ok(DensityProofDoctorOutput {
            evidence_written: false,
            evidence_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "create density proof doctor output dir {}",
                parent.display()
            )
        })?;
    }
    let payload = DensityProofDoctorWritePayload {
        mode: "llmwave-big-density-proof-doctor",
        version: DENSITY_PROOF_DOCTOR_VERSION,
        verdict,
        suite_path,
        profile_diversity: diversity,
        proof_quality,
        gates,
        claim_boundary,
    };
    let json = serde_json::to_vec_pretty(&payload).context("serialize density proof doctor")?;
    let mut file = fs::File::create(path)
        .with_context(|| format!("create density proof doctor {}", path.display()))?;
    file.write_all(&json)
        .with_context(|| format!("write density proof doctor {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish density proof doctor {}", path.display()))?;
    Ok(DensityProofDoctorOutput {
        evidence_written: true,
        evidence_path: Some(path.display().to_string()),
    })
}

#[derive(Serialize)]
struct DensityProofDoctorWritePayload<'a> {
    mode: &'static str,
    version: &'static str,
    verdict: &'static str,
    suite_path: &'a str,
    profile_diversity: &'a ProfileDiversityReport,
    proof_quality: &'a ProofQualityReport,
    gates: &'a DensityProofDoctorGates,
    claim_boundary: &'a DensityProofDoctorClaimBoundary,
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_profile_prefers_larger_corpus_for_tiny_fixtures() {
        let weak = vec!["profile_corpora_too_small_or_unknown"];
        assert_eq!(
            next_profile_needed("DENSITY_PROOF_WEAK", &weak),
            "larger-real-corpus-profile"
        );
    }
}
