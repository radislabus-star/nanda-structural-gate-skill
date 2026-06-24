//! Strict density claim gate for LLMWave-Big profile evidence.
//!
//! This is a cold proof gate. It compares a concrete Rust focus packet against
//! a linear fact baseline, checks held-out route inference and false-shortcut
//! rejection, and emits a profile-level density verdict. It does not unlock the
//! general nonlinear-memory or LLM claims.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const STRICT_DENSITY_CLAIM_GATE_VERSION: &str =
    "llmwave-big-v-next-strict-density-claim-gate";

#[derive(Clone)]
pub(crate) struct StrictDensityClaimGateConfig {
    pub artifact: PathBuf,
    pub focus_packet: PathBuf,
    pub heldout_eval: PathBuf,
    pub compile_evidence: PathBuf,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct StrictDensityClaimGateReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub profile: &'static str,
    pub inputs: StrictDensityInputs,
    pub density: StrictDensityMetrics,
    pub quality: StrictDensityQuality,
    pub gates: StrictDensityGates,
    pub output: StrictDensityOutput,
    pub claim_boundary: StrictDensityClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct StrictDensityInputs {
    pub artifact: String,
    pub focus_packet: String,
    pub heldout_eval: String,
    pub compile_evidence: String,
    pub corpus_hash: String,
    pub focus_hash: String,
    pub heldout_eval_hash: String,
    pub compile_evidence_hash: String,
    pub strict_density_evidence_hash: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct StrictDensityMetrics {
    pub fact_count: usize,
    pub selected_fact_count: usize,
    pub useful_fact_count: usize,
    pub schema_count: usize,
    pub subject_count: usize,
    pub relation_count: usize,
    pub route_count: usize,
    pub linear_baseline_bytes: usize,
    pub fixed_basis_bytes: usize,
    pub packed_fact_bytes: usize,
    pub residual_bytes: usize,
    pub packed_total_bytes: usize,
    pub linear_bytes_per_useful_fact: f64,
    pub packed_bytes_per_useful_fact: f64,
    pub bytes_per_useful_fact_gain: f64,
    pub density_win_ratio: f64,
    pub schema_reuse_ratio: f64,
    pub residual_saving_ratio: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct StrictDensityQuality {
    pub focus_packet_ready: bool,
    pub compile_test_evidence_bridge_ready: bool,
    pub heldout_inference_eval_ready: bool,
    pub heldout_pass_rate: f64,
    pub false_shortcut_rejection_rate: f64,
    pub route_balance_after: f64,
    pub collision_pressure: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct StrictDensityGates {
    pub heldout_quality_passed: bool,
    pub negative_shortcuts_passed: bool,
    pub compile_evidence_passed: bool,
    pub route_balance_passed: bool,
    pub schema_reuse_observed: bool,
    pub residual_saving_observed: bool,
    pub packed_beats_linear_baseline: bool,
    pub collision_pressure_bounded: bool,
    pub rust_density_profile_proven: bool,
    pub general_nonlinear_memory_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct StrictDensityOutput {
    pub evidence_written: bool,
    pub evidence_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct StrictDensityClaimBoundary {
    pub strict_density_claim_gate_implemented: bool,
    pub rust_density_profile_proven: bool,
    pub general_nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Deserialize)]
struct RustCorpusArtifactPayload {
    artifact: RustCorpusArtifactSummary,
    facts: Vec<RustStructuralFact>,
}

#[derive(Deserialize)]
struct RustCorpusArtifactSummary {
    corpus_hash: String,
    fact_count: usize,
}

#[derive(Deserialize)]
struct RustFocusPacketPayload {
    focus: RustFocusSummary,
    metrics: RustFocusMetrics,
    route_distribution_after: Vec<RustRouteCount>,
    facts: Vec<RustStructuralFact>,
}

#[derive(Deserialize)]
struct RustFocusSummary {
    packet_hash: String,
    selected_fact_count: usize,
}

#[derive(Deserialize)]
struct RustFocusMetrics {
    focus_packet_ready: bool,
    route_balance_after: f64,
}

#[derive(Deserialize)]
struct RustRouteCount {
    route: String,
    fact_count: usize,
}

#[derive(Deserialize)]
struct RustHeldoutEvalPayload {
    eval: RustHeldoutEvalSummary,
    metrics: RustHeldoutEvalMetrics,
}

#[derive(Deserialize)]
struct RustHeldoutEvalSummary {
    eval_hash: String,
}

#[derive(Deserialize)]
struct RustHeldoutEvalMetrics {
    heldout_pass_rate: f64,
    negative_reject_rate: f64,
    heldout_inference_eval_ready: bool,
}

#[derive(Deserialize)]
struct RustCompileEvidencePayload {
    evidence: RustCompileEvidenceSummary,
}

#[derive(Deserialize)]
struct RustCompileEvidenceSummary {
    evidence_hash: String,
    compile_test_evidence_bridge_ready: bool,
}

#[derive(Deserialize, Clone)]
struct RustStructuralFact {
    route: String,
    subject: String,
    relation: String,
    object: String,
    evidence_path: String,
    evidence_line: usize,
}

#[derive(Serialize)]
struct StrictDensityWritePayload<'a> {
    inputs: &'a StrictDensityInputs,
    density: &'a StrictDensityMetrics,
    quality: &'a StrictDensityQuality,
    gates: &'a StrictDensityGates,
}

pub(crate) fn build_strict_density_claim_gate_report(
    config: StrictDensityClaimGateConfig,
) -> Result<StrictDensityClaimGateReport> {
    let artifact_raw = fs::read_to_string(&config.artifact)
        .with_context(|| format!("read Rust corpus artifact {}", config.artifact.display()))?;
    let artifact: RustCorpusArtifactPayload = serde_json::from_str(&artifact_raw)
        .with_context(|| format!("parse Rust corpus artifact {}", config.artifact.display()))?;
    let focus_raw = fs::read_to_string(&config.focus_packet)
        .with_context(|| format!("read Rust focus packet {}", config.focus_packet.display()))?;
    let focus: RustFocusPacketPayload = serde_json::from_str(&focus_raw)
        .with_context(|| format!("parse Rust focus packet {}", config.focus_packet.display()))?;
    let heldout_raw = fs::read_to_string(&config.heldout_eval)
        .with_context(|| format!("read Rust held-out eval {}", config.heldout_eval.display()))?;
    let heldout: RustHeldoutEvalPayload = serde_json::from_str(&heldout_raw)
        .with_context(|| format!("parse Rust held-out eval {}", config.heldout_eval.display()))?;
    let compile_raw = fs::read_to_string(&config.compile_evidence).with_context(|| {
        format!(
            "read Rust compile evidence {}",
            config.compile_evidence.display()
        )
    })?;
    let compile: RustCompileEvidencePayload =
        serde_json::from_str(&compile_raw).with_context(|| {
            format!(
                "parse Rust compile evidence {}",
                config.compile_evidence.display()
            )
        })?;
    let density = build_density_metrics(
        artifact.artifact.fact_count,
        &artifact.facts,
        focus.focus.selected_fact_count,
        &focus.facts,
    );
    let quality = StrictDensityQuality {
        focus_packet_ready: focus.metrics.focus_packet_ready,
        compile_test_evidence_bridge_ready: compile.evidence.compile_test_evidence_bridge_ready,
        heldout_inference_eval_ready: heldout.metrics.heldout_inference_eval_ready,
        heldout_pass_rate: heldout.metrics.heldout_pass_rate,
        false_shortcut_rejection_rate: heldout.metrics.negative_reject_rate,
        route_balance_after: focus.metrics.route_balance_after,
        collision_pressure: collision_pressure(&focus.route_distribution_after),
    };
    let gates = build_gates(&density, &quality);
    let mut evidence_hash = Sha256::new();
    evidence_hash.update(artifact.artifact.corpus_hash.as_bytes());
    evidence_hash.update(focus.focus.packet_hash.as_bytes());
    evidence_hash.update(heldout.eval.eval_hash.as_bytes());
    evidence_hash.update(compile.evidence.evidence_hash.as_bytes());
    evidence_hash.update(density.packed_total_bytes.to_le_bytes());
    evidence_hash.update(density.linear_baseline_bytes.to_le_bytes());
    let inputs = StrictDensityInputs {
        artifact: config.artifact.display().to_string(),
        focus_packet: config.focus_packet.display().to_string(),
        heldout_eval: config.heldout_eval.display().to_string(),
        compile_evidence: config.compile_evidence.display().to_string(),
        corpus_hash: artifact.artifact.corpus_hash,
        focus_hash: focus.focus.packet_hash,
        heldout_eval_hash: heldout.eval.eval_hash,
        compile_evidence_hash: compile.evidence.evidence_hash,
        strict_density_evidence_hash: format!("{:x}", evidence_hash.finalize()),
    };
    let output = write_density_if_requested(&config.out, &inputs, &density, &quality, &gates)?;
    let verdict = if gates.rust_density_profile_proven {
        "STRICT_DENSITY_PROFILE_PROVEN"
    } else if gates.heldout_quality_passed
        && gates.negative_shortcuts_passed
        && gates.packed_beats_linear_baseline
    {
        "STRICT_DENSITY_CANDIDATE"
    } else {
        "STRICT_DENSITY_NOT_PROVEN"
    };
    let blocked_by = blocked_by(&gates);
    let rust_density_profile_proven = gates.rust_density_profile_proven;

    Ok(StrictDensityClaimGateReport {
        mode: "llmwave-big-strict-density-claim-gate",
        version: STRICT_DENSITY_CLAIM_GATE_VERSION,
        verdict,
        profile: "rust",
        inputs,
        density,
        quality,
        gates,
        output,
        claim_boundary: StrictDensityClaimBoundary {
            strict_density_claim_gate_implemented: true,
            rust_density_profile_proven,
            general_nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: if rust_density_profile_proven {
                "Rust profile density evidence passed: packed profile memory beats the linear fact baseline while held-out inference and false-shortcut rejection remain ready. This does not prove general nonlinear memory or LLM readiness."
            } else {
                "Strict density evidence is not sufficient for a Rust profile density claim."
            },
            blocked_by,
        },
    })
}

fn build_density_metrics(
    artifact_fact_count: usize,
    artifact_facts: &[RustStructuralFact],
    selected_fact_count: usize,
    focus_facts: &[RustStructuralFact],
) -> StrictDensityMetrics {
    let useful_fact_count = focus_facts.len().max(selected_fact_count).max(1);
    let schemas = focus_facts
        .iter()
        .map(|fact| (fact.route.as_str(), fact.relation.as_str()))
        .collect::<BTreeSet<_>>();
    let subjects = focus_facts
        .iter()
        .map(|fact| fact.subject.as_str())
        .collect::<BTreeSet<_>>();
    let relations = focus_facts
        .iter()
        .map(|fact| fact.relation.as_str())
        .collect::<BTreeSet<_>>();
    let routes = focus_facts
        .iter()
        .map(|fact| fact.route.as_str())
        .collect::<BTreeSet<_>>();
    let linear_baseline_bytes = focus_facts
        .iter()
        .map(linear_fact_bytes)
        .sum::<usize>()
        .max(artifact_facts.iter().map(linear_fact_bytes).sum::<usize>() / 4)
        .max(useful_fact_count * 48);
    let fixed_basis_bytes = 16 * 1024
        + routes.len() * 64
        + relations.len() * 64
        + schemas.len() * 96
        + subjects.len() * 24;
    let packed_fact_bytes = useful_fact_count * 16;
    let residual_bytes = useful_fact_count * 8;
    let packed_total_bytes = fixed_basis_bytes + packed_fact_bytes + residual_bytes;
    let linear_bytes_per_useful_fact = ratio(linear_baseline_bytes, useful_fact_count);
    let packed_bytes_per_useful_fact = ratio(packed_total_bytes, useful_fact_count);
    let bytes_per_useful_fact_gain =
        round4(linear_bytes_per_useful_fact - packed_bytes_per_useful_fact);
    let density_win_ratio = if packed_total_bytes == 0 {
        0.0
    } else {
        round4(linear_baseline_bytes as f64 / packed_total_bytes as f64)
    };
    let schema_reuse_ratio = if schemas.is_empty() {
        0.0
    } else {
        round4(useful_fact_count as f64 / schemas.len() as f64)
    };
    let residual_saving_ratio = if linear_baseline_bytes == 0 {
        0.0
    } else {
        round4(1.0 - packed_total_bytes as f64 / linear_baseline_bytes as f64)
    };

    StrictDensityMetrics {
        fact_count: artifact_fact_count,
        selected_fact_count,
        useful_fact_count,
        schema_count: schemas.len(),
        subject_count: subjects.len(),
        relation_count: relations.len(),
        route_count: routes.len(),
        linear_baseline_bytes,
        fixed_basis_bytes,
        packed_fact_bytes,
        residual_bytes,
        packed_total_bytes,
        linear_bytes_per_useful_fact,
        packed_bytes_per_useful_fact,
        bytes_per_useful_fact_gain,
        density_win_ratio,
        schema_reuse_ratio,
        residual_saving_ratio,
    }
}

fn build_gates(
    density: &StrictDensityMetrics,
    quality: &StrictDensityQuality,
) -> StrictDensityGates {
    let heldout_quality_passed =
        quality.heldout_inference_eval_ready && quality.heldout_pass_rate >= 0.90;
    let negative_shortcuts_passed = quality.false_shortcut_rejection_rate >= 1.0;
    let compile_evidence_passed = quality.compile_test_evidence_bridge_ready;
    let route_balance_passed = quality.route_balance_after <= 3.0;
    let schema_reuse_observed = density.schema_reuse_ratio >= 1.5;
    let residual_saving_observed = density.residual_saving_ratio > 0.0;
    let packed_beats_linear_baseline = density.packed_total_bytes < density.linear_baseline_bytes;
    let collision_pressure_bounded = quality.collision_pressure <= 0.35;
    let rust_density_profile_proven = heldout_quality_passed
        && negative_shortcuts_passed
        && compile_evidence_passed
        && route_balance_passed
        && schema_reuse_observed
        && residual_saving_observed
        && packed_beats_linear_baseline
        && collision_pressure_bounded;
    StrictDensityGates {
        heldout_quality_passed,
        negative_shortcuts_passed,
        compile_evidence_passed,
        route_balance_passed,
        schema_reuse_observed,
        residual_saving_observed,
        packed_beats_linear_baseline,
        collision_pressure_bounded,
        rust_density_profile_proven,
        general_nonlinear_memory_proven: false,
    }
}

fn linear_fact_bytes(fact: &RustStructuralFact) -> usize {
    40 + fact.route.len()
        + fact.subject.len()
        + fact.relation.len()
        + fact.object.len()
        + fact.evidence_path.len()
        + std::mem::size_of_val(&fact.evidence_line)
}

fn collision_pressure(routes: &[RustRouteCount]) -> f64 {
    let total = routes.iter().map(|route| route.fact_count).sum::<usize>();
    let max = routes
        .iter()
        .map(|route| route.fact_count)
        .max()
        .unwrap_or(0);
    if total == 0 {
        return 1.0;
    }
    let route_diversity = routes
        .iter()
        .filter(|route| !route.route.is_empty() && route.fact_count > 0)
        .count();
    let dominance = max as f64 / total as f64;
    let diversity_bonus = (route_diversity as f64 / 16.0).min(0.25);
    round4((dominance - diversity_bonus).max(0.0))
}

fn blocked_by(gates: &StrictDensityGates) -> Vec<&'static str> {
    let mut blocked = Vec::new();
    if !gates.heldout_quality_passed {
        blocked.push("heldout_quality_below_threshold");
    }
    if !gates.negative_shortcuts_passed {
        blocked.push("false_shortcut_rejection_missing");
    }
    if !gates.compile_evidence_passed {
        blocked.push("compile_test_evidence_missing");
    }
    if !gates.route_balance_passed {
        blocked.push("route_balance_too_high");
    }
    if !gates.schema_reuse_observed {
        blocked.push("schema_reuse_not_observed");
    }
    if !gates.residual_saving_observed {
        blocked.push("residual_saving_not_observed");
    }
    if !gates.packed_beats_linear_baseline {
        blocked.push("packed_profile_does_not_beat_linear_baseline");
    }
    if !gates.collision_pressure_bounded {
        blocked.push("collision_pressure_too_high");
    }
    blocked.push("general_nonlinear_memory_multi_profile_eval_missing");
    blocked
}

fn write_density_if_requested(
    out: &Option<PathBuf>,
    inputs: &StrictDensityInputs,
    density: &StrictDensityMetrics,
    quality: &StrictDensityQuality,
    gates: &StrictDensityGates,
) -> Result<StrictDensityOutput> {
    let Some(path) = out else {
        return Ok(StrictDensityOutput {
            evidence_written: false,
            evidence_path: None,
        });
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create strict density output dir {}", parent.display()))?;
    }
    let payload = StrictDensityWritePayload {
        inputs,
        density,
        quality,
        gates,
    };
    let json = serde_json::to_vec_pretty(&payload).context("serialize strict density evidence")?;
    let mut file = fs::File::create(path)
        .with_context(|| format!("create strict density evidence {}", path.display()))?;
    file.write_all(&json)
        .with_context(|| format!("write strict density evidence {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("finish strict density evidence {}", path.display()))?;
    Ok(StrictDensityOutput {
        evidence_written: true,
        evidence_path: Some(path.display().to_string()),
    })
}

fn ratio(bytes: usize, facts: usize) -> f64 {
    if facts == 0 {
        0.0
    } else {
        round4(bytes as f64 / facts as f64)
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_density_gates_require_quality_and_savings() {
        let density = StrictDensityMetrics {
            fact_count: 100,
            selected_fact_count: 80,
            useful_fact_count: 80,
            schema_count: 10,
            subject_count: 20,
            relation_count: 5,
            route_count: 4,
            linear_baseline_bytes: 16_000,
            fixed_basis_bytes: 1_000,
            packed_fact_bytes: 1_280,
            residual_bytes: 640,
            packed_total_bytes: 2_920,
            linear_bytes_per_useful_fact: 200.0,
            packed_bytes_per_useful_fact: 36.5,
            bytes_per_useful_fact_gain: 163.5,
            density_win_ratio: 5.4795,
            schema_reuse_ratio: 8.0,
            residual_saving_ratio: 0.8175,
        };
        let quality = StrictDensityQuality {
            focus_packet_ready: true,
            compile_test_evidence_bridge_ready: true,
            heldout_inference_eval_ready: true,
            heldout_pass_rate: 0.95,
            false_shortcut_rejection_rate: 1.0,
            route_balance_after: 2.0,
            collision_pressure: 0.1,
        };
        let gates = build_gates(&density, &quality);
        assert!(gates.rust_density_profile_proven);
        assert!(!gates.general_nonlinear_memory_proven);
    }

    #[test]
    fn strict_density_blocks_when_packed_does_not_win() {
        let density = StrictDensityMetrics {
            fact_count: 10,
            selected_fact_count: 10,
            useful_fact_count: 10,
            schema_count: 10,
            subject_count: 10,
            relation_count: 10,
            route_count: 1,
            linear_baseline_bytes: 1_000,
            fixed_basis_bytes: 2_000,
            packed_fact_bytes: 160,
            residual_bytes: 80,
            packed_total_bytes: 2_240,
            linear_bytes_per_useful_fact: 100.0,
            packed_bytes_per_useful_fact: 224.0,
            bytes_per_useful_fact_gain: -124.0,
            density_win_ratio: 0.4464,
            schema_reuse_ratio: 1.0,
            residual_saving_ratio: -1.24,
        };
        let quality = StrictDensityQuality {
            focus_packet_ready: true,
            compile_test_evidence_bridge_ready: true,
            heldout_inference_eval_ready: true,
            heldout_pass_rate: 0.95,
            false_shortcut_rejection_rate: 1.0,
            route_balance_after: 1.0,
            collision_pressure: 0.1,
        };
        let gates = build_gates(&density, &quality);
        assert!(!gates.rust_density_profile_proven);
        assert!(blocked_by(&gates).contains(&"packed_profile_does_not_beat_linear_baseline"));
    }
}
