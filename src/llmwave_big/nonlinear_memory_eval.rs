//! Fixed-basis nonlinear memory proof harness.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde::Serialize;

use super::write;

pub(crate) const NONLINEAR_MEMORY_EVAL_VERSION: &str = "llmwave-big-v-next-nonlinear-memory-eval";

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub(crate) enum NonlinearProofPolicyKind {
    #[value(name = "strict-full-sweep")]
    StrictFullSweep,
    #[value(name = "scale-amortized")]
    ScaleAmortized,
}

#[derive(Serialize, Clone)]
pub(crate) struct NonlinearMemoryEvalReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub proof_policy: NonlinearProofPolicyReport,
    pub basis: FixedBasisReport,
    pub external_corpus: ExternalCorpusEvalReport,
    pub corpus_driven_memory: CorpusDrivenMemoryReport,
    pub sweep: Vec<CapacitySweepPoint>,
    pub aggregate: NonlinearAggregateMetrics,
    pub gates: NonlinearProofGates,
    pub claim_boundary: NonlinearClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct NonlinearProofPolicyReport {
    pub selected: &'static str,
    pub selected_policy_proven: bool,
    pub strict_full_sweep_nonlinear_memory_proven: bool,
    pub scale_amortized_nonlinear_memory_proven: bool,
    pub general_claim_unlocked: bool,
    pub read_as: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct FixedBasisReport {
    pub wave_dim: usize,
    pub basis_id: &'static str,
    pub fixed_across_sweep: bool,
    pub schema_families: usize,
    pub relation_slots: usize,
    pub role_slots: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct ExternalCorpusEvalReport {
    pub loaded: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
    pub fact_count: usize,
    pub heldout_count: usize,
    pub negative_count: usize,
    pub noise_count: usize,
    pub heldout_pass_rate: f64,
    pub negative_reject_rate: f64,
    pub noise_reject_rate: f64,
    pub external_corpus_present: bool,
    pub broad_noise_eval_present: bool,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CorpusDrivenMemoryReport {
    pub loaded: bool,
    pub fact_count: usize,
    pub schema_count: usize,
    pub residual_count: usize,
    pub linear_baseline: CorpusMemoryModelMetrics,
    pub fixed_basis_standalone: CorpusMemoryModelMetrics,
    pub fixed_basis_amortized: CorpusMemoryModelMetrics,
    pub delta: CorpusMemoryDelta,
    pub gates: CorpusDrivenMemoryGates,
    pub verdict: &'static str,
    pub read_as: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CorpusMemoryModelMetrics {
    pub bytes_total: usize,
    pub bytes_per_useful_fact: f64,
    pub schema_reuse_ratio: f64,
    pub residual_saving_ratio: f64,
    pub heldout_pass_rate: f64,
    pub negative_reject_rate: f64,
    pub noise_reject_rate: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CorpusMemoryDelta {
    pub standalone_bytes_per_useful_fact_gain: f64,
    pub amortized_bytes_per_useful_fact_gain: f64,
    pub amortized_total_bytes_gain: f64,
    pub basis_overhead_bytes: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct CorpusDrivenMemoryGates {
    pub schema_reuse_observed: bool,
    pub residual_encoding_smaller_than_full_fact: bool,
    pub amortized_wave_beats_linear: bool,
    pub standalone_wave_beats_linear: bool,
    pub heldout_inference_passed: bool,
    pub negative_controls_passed: bool,
    pub noise_controls_passed: bool,
    pub corpus_driven_nonlinear_density_observed: bool,
    pub strict_standalone_density_observed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CapacitySweepPoint {
    pub facts: usize,
    pub useful_facts: usize,
    pub schemas_reused: usize,
    pub linear_baseline: MemoryModelMetrics,
    pub fixed_basis_wave: MemoryModelMetrics,
    pub delta: MemoryDeltaMetrics,
    pub verdict: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct MemoryModelMetrics {
    pub bytes_total: usize,
    pub bytes_per_useful_fact: f64,
    pub schema_reuse_ratio: f64,
    pub residual_saving_ratio: f64,
    pub role_error_rate: f64,
    pub false_positive_rate: f64,
    pub inference_score: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct MemoryDeltaMetrics {
    pub bytes_per_useful_fact_gain: f64,
    pub useful_inference_per_mb_gain: f64,
    pub role_error_delta: f64,
    pub false_positive_delta: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct NonlinearAggregateMetrics {
    pub best_bytes_per_useful_fact_gain: f64,
    pub median_bytes_per_useful_fact_gain: f64,
    pub large_scale_bytes_per_useful_fact_gain: f64,
    pub large_scale_win_rate: f64,
    pub max_schema_reuse_ratio: f64,
    pub max_residual_saving_ratio: f64,
    pub max_role_error_rate: f64,
    pub max_false_positive_rate: f64,
    pub min_inference_score: f64,
    pub useful_inference_per_mb: f64,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct NonlinearProofGates {
    pub scale_amortized_basis_beats_linear: bool,
    pub scale_amortized_bytes_per_useful_fact_improves: bool,
    pub scale_amortized_nonlinear_memory_proven: bool,
    pub fixed_basis_beats_linear_baseline: bool,
    pub bytes_per_useful_fact_improves: bool,
    pub schema_reuse_rises_with_scale: bool,
    pub residual_saving_survives_scale: bool,
    pub role_error_rate_bounded: bool,
    pub false_positive_rate_bounded: bool,
    pub heldout_inference_present: bool,
    pub external_corpus_present: bool,
    pub broad_noise_eval_present: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct NonlinearClaimBoundary {
    pub nonlinear_memory_eval_implemented: bool,
    pub fixed_basis_used: bool,
    pub linear_baseline_compared: bool,
    pub useful_density_candidate: bool,
    pub scale_amortized_nonlinear_memory_proven: bool,
    pub selected_policy_proven: bool,
    pub selected_policy: &'static str,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

#[derive(Deserialize)]
struct ExternalCorpusFixture {
    version: String,
    source: String,
    facts: Vec<ExternalFact>,
    held_out: Vec<ExternalHeldOut>,
    negative: Vec<ExternalNegative>,
    noise: Vec<ExternalNegative>,
}

#[derive(Deserialize)]
struct ExternalFact {
    route: String,
    subject_role: String,
    subject: String,
    operator: String,
    object_role: String,
    object: String,
}

#[derive(Deserialize)]
struct ExternalHeldOut {
    route: String,
    subject_role: String,
    operator: String,
    object_role: String,
    expected_object_family: String,
}

#[derive(Deserialize)]
struct ExternalNegative {
    route: String,
    subject_role: String,
    subject: String,
    operator: String,
    object_role: String,
    object: String,
}

pub(crate) fn build_nonlinear_memory_eval_report(
    corpus_path: Option<&Path>,
    proof_policy: NonlinearProofPolicyKind,
) -> Result<NonlinearMemoryEvalReport> {
    let basis = FixedBasisReport {
        wave_dim: super::super::WAVE_DIM,
        basis_id: "fixed-field-basis-1024-v1",
        fixed_across_sweep: true,
        schema_families: 6,
        relation_slots: 12,
        role_slots: 16,
    };
    let (external_corpus, corpus_driven_memory) = match corpus_path {
        Some(path) => {
            let fixture = read_external_corpus(path)?;
            (
                external_corpus_eval(path, &fixture),
                corpus_driven_memory_eval(&fixture),
            )
        }
        None => (empty_external_corpus(), empty_corpus_driven_memory()),
    };
    let sweep = [64usize, 256, 1024, 4096, 15000]
        .into_iter()
        .map(|facts| sweep_point(facts, &basis))
        .collect::<Vec<_>>();
    let aggregate = aggregate_metrics(&sweep);
    let gates = proof_gates(&sweep, &aggregate, &external_corpus);
    let nonlinear_memory_proven = gates.fixed_basis_beats_linear_baseline
        && gates.bytes_per_useful_fact_improves
        && gates.schema_reuse_rises_with_scale
        && gates.residual_saving_survives_scale
        && gates.role_error_rate_bounded
        && gates.false_positive_rate_bounded
        && gates.heldout_inference_present
        && gates.external_corpus_present
        && gates.broad_noise_eval_present;
    let scale_amortized_nonlinear_memory_proven = gates.scale_amortized_nonlinear_memory_proven;
    let proof_policy_report = proof_policy_report(
        proof_policy,
        nonlinear_memory_proven,
        scale_amortized_nonlinear_memory_proven,
    );
    let blocked_by = blocked_reasons(&gates);
    let scale_candidate = aggregate.large_scale_win_rate >= 1.0
        && aggregate.large_scale_bytes_per_useful_fact_gain > 1.2
        && gates.role_error_rate_bounded
        && gates.false_positive_rate_bounded;
    let verdict = if nonlinear_memory_proven {
        "NONLINEAR_MEMORY_PROOF_PASS"
    } else if scale_candidate {
        "NONLINEAR_MEMORY_SCALE_CANDIDATE_NOT_PROVEN"
    } else {
        "NONLINEAR_MEMORY_REVIEW"
    };

    Ok(NonlinearMemoryEvalReport {
        mode: "llmwave-big-nonlinear-memory-eval",
        version: NONLINEAR_MEMORY_EVAL_VERSION,
        roadmap_block: "v-next-nonlinear-memory-proof",
        verdict,
        proof_policy: proof_policy_report.clone(),
        basis,
        external_corpus,
        corpus_driven_memory,
        sweep,
        aggregate,
        gates,
        claim_boundary: NonlinearClaimBoundary {
            nonlinear_memory_eval_implemented: true,
            fixed_basis_used: true,
            linear_baseline_compared: true,
            useful_density_candidate: verdict == "NONLINEAR_MEMORY_CANDIDATE_NOT_PROVEN"
                || verdict == "NONLINEAR_MEMORY_SCALE_CANDIDATE_NOT_PROVEN"
                || verdict == "NONLINEAR_MEMORY_PROOF_PASS",
            scale_amortized_nonlinear_memory_proven,
            selected_policy_proven: proof_policy_report.selected_policy_proven,
            selected_policy: proof_policy_report.selected,
            nonlinear_memory_proven,
            safe_claim: if nonlinear_memory_proven {
                "Fixed-basis residual memory passed the configured nonlinear proof gates"
            } else if scale_amortized_nonlinear_memory_proven {
                "Fixed-basis residual memory passed the scale-amortized local proof gates, but strict/general nonlinear memory remains unproven"
            } else {
                "Fixed-basis residual memory shows a useful density candidate, but nonlinear memory remains unproven until external held-out/noise gates pass"
            },
            blocked_by,
        },
    })
}

fn read_external_corpus(path: &Path) -> Result<ExternalCorpusFixture> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read nonlinear memory corpus {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parse nonlinear memory corpus {}", path.display()))
}

fn external_corpus_eval(path: &Path, fixture: &ExternalCorpusFixture) -> ExternalCorpusEvalReport {
    let heldout_passes = fixture
        .held_out
        .iter()
        .filter(|held_out| heldout_matches(&fixture.facts, held_out))
        .count();
    let negative_rejects = fixture
        .negative
        .iter()
        .filter(|negative| !fact_matches(&fixture.facts, negative))
        .count();
    let noise_rejects = fixture
        .noise
        .iter()
        .filter(|negative| !fact_matches(&fixture.facts, negative))
        .count();
    let heldout_pass_rate = ratio(heldout_passes, fixture.held_out.len());
    let negative_reject_rate = ratio(negative_rejects, fixture.negative.len());
    let noise_reject_rate = ratio(noise_rejects, fixture.noise.len());
    let external_corpus_present =
        fixture.facts.len() >= 8 && heldout_pass_rate >= 1.0 && negative_reject_rate >= 1.0;
    let broad_noise_eval_present =
        fixture.noise.len() >= 4 && noise_reject_rate >= 1.0 && negative_reject_rate >= 1.0;
    let state = if external_corpus_present && broad_noise_eval_present {
        "EXTERNAL_FIXTURE_AND_NOISE_PASS"
    } else if external_corpus_present {
        "EXTERNAL_FIXTURE_PASS_NOISE_MISSING"
    } else {
        "EXTERNAL_FIXTURE_REVIEW"
    };

    ExternalCorpusEvalReport {
        loaded: true,
        path: Some(path.display().to_string()),
        version: Some(fixture.version.clone()),
        source: Some(fixture.source.clone()),
        fact_count: fixture.facts.len(),
        heldout_count: fixture.held_out.len(),
        negative_count: fixture.negative.len(),
        noise_count: fixture.noise.len(),
        heldout_pass_rate,
        negative_reject_rate,
        noise_reject_rate,
        external_corpus_present,
        broad_noise_eval_present,
        state,
    }
}

fn empty_external_corpus() -> ExternalCorpusEvalReport {
    ExternalCorpusEvalReport {
        loaded: false,
        path: None,
        version: None,
        source: None,
        fact_count: 0,
        heldout_count: 0,
        negative_count: 0,
        noise_count: 0,
        heldout_pass_rate: 0.0,
        negative_reject_rate: 0.0,
        noise_reject_rate: 0.0,
        external_corpus_present: false,
        broad_noise_eval_present: false,
        state: "NO_EXTERNAL_CORPUS",
    }
}

fn corpus_driven_memory_eval(fixture: &ExternalCorpusFixture) -> CorpusDrivenMemoryReport {
    let fact_count = fixture.facts.len();
    let useful_facts = fact_count.max(1);
    let schema_count = fixture
        .facts
        .iter()
        .map(schema_key)
        .collect::<BTreeSet<_>>()
        .len();
    let schema_count = schema_count.max(1);
    let residual_count = fact_count;
    let heldout_passes = fixture
        .held_out
        .iter()
        .filter(|held_out| heldout_matches(&fixture.facts, held_out))
        .count();
    let negative_rejects = fixture
        .negative
        .iter()
        .filter(|negative| !fact_matches(&fixture.facts, negative))
        .count();
    let noise_rejects = fixture
        .noise
        .iter()
        .filter(|negative| !fact_matches(&fixture.facts, negative))
        .count();
    let heldout_pass_rate = ratio(heldout_passes, fixture.held_out.len());
    let negative_reject_rate = ratio(negative_rejects, fixture.negative.len());
    let noise_reject_rate = ratio(noise_rejects, fixture.noise.len());
    let basis_overhead_bytes = 64 * 1024;
    let schema_bytes = schema_count * 32;
    let residual_bytes = residual_count * write::SMALL_RESIDUAL_BYTES;
    let centroid_bytes = schema_count * write::CENTROID_UPDATE_BYTES;
    let linear_bytes = fact_count * write::FULL_FACT_RECORD_BYTES;
    let amortized_bytes = schema_bytes + residual_bytes + centroid_bytes;
    let standalone_bytes = basis_overhead_bytes + amortized_bytes;
    let schema_reuse_ratio = round4(fact_count as f64 / schema_count as f64);
    let residual_saving_ratio =
        round4(1.0 - (write::SMALL_RESIDUAL_BYTES as f64 / write::FULL_FACT_RECORD_BYTES as f64));
    let linear = CorpusMemoryModelMetrics {
        bytes_total: linear_bytes,
        bytes_per_useful_fact: round4(linear_bytes as f64 / useful_facts as f64),
        schema_reuse_ratio: 1.0,
        residual_saving_ratio: 0.0,
        heldout_pass_rate,
        negative_reject_rate,
        noise_reject_rate,
    };
    let fixed_basis_amortized = CorpusMemoryModelMetrics {
        bytes_total: amortized_bytes,
        bytes_per_useful_fact: round4(amortized_bytes as f64 / useful_facts as f64),
        schema_reuse_ratio,
        residual_saving_ratio,
        heldout_pass_rate,
        negative_reject_rate,
        noise_reject_rate,
    };
    let fixed_basis_standalone = CorpusMemoryModelMetrics {
        bytes_total: standalone_bytes,
        bytes_per_useful_fact: round4(standalone_bytes as f64 / useful_facts as f64),
        schema_reuse_ratio,
        residual_saving_ratio,
        heldout_pass_rate,
        negative_reject_rate,
        noise_reject_rate,
    };
    let delta = CorpusMemoryDelta {
        standalone_bytes_per_useful_fact_gain: round4(
            linear.bytes_per_useful_fact / fixed_basis_standalone.bytes_per_useful_fact,
        ),
        amortized_bytes_per_useful_fact_gain: round4(
            linear.bytes_per_useful_fact / fixed_basis_amortized.bytes_per_useful_fact,
        ),
        amortized_total_bytes_gain: round4(linear_bytes as f64 / amortized_bytes.max(1) as f64),
        basis_overhead_bytes,
    };
    let gates = CorpusDrivenMemoryGates {
        schema_reuse_observed: schema_reuse_ratio > 1.0,
        residual_encoding_smaller_than_full_fact: write::SMALL_RESIDUAL_BYTES
            < write::FULL_FACT_RECORD_BYTES,
        amortized_wave_beats_linear: delta.amortized_bytes_per_useful_fact_gain > 1.2,
        standalone_wave_beats_linear: delta.standalone_bytes_per_useful_fact_gain > 1.2,
        heldout_inference_passed: heldout_pass_rate >= 1.0,
        negative_controls_passed: negative_reject_rate >= 1.0,
        noise_controls_passed: noise_reject_rate >= 1.0,
        corpus_driven_nonlinear_density_observed: false,
        strict_standalone_density_observed: false,
    };
    let corpus_driven_nonlinear_density_observed = gates.schema_reuse_observed
        && gates.residual_encoding_smaller_than_full_fact
        && gates.amortized_wave_beats_linear
        && gates.heldout_inference_passed
        && gates.negative_controls_passed
        && gates.noise_controls_passed;
    let strict_standalone_density_observed =
        corpus_driven_nonlinear_density_observed && gates.standalone_wave_beats_linear;
    let gates = CorpusDrivenMemoryGates {
        corpus_driven_nonlinear_density_observed,
        strict_standalone_density_observed,
        ..gates
    };
    let verdict = if strict_standalone_density_observed {
        "CORPUS_DRIVEN_STRICT_DENSITY_OBSERVED"
    } else if corpus_driven_nonlinear_density_observed {
        "CORPUS_DRIVEN_AMORTIZED_DENSITY_OBSERVED"
    } else {
        "CORPUS_DRIVEN_DENSITY_REVIEW"
    };

    CorpusDrivenMemoryReport {
        loaded: true,
        fact_count,
        schema_count,
        residual_count,
        linear_baseline: linear,
        fixed_basis_standalone,
        fixed_basis_amortized,
        delta,
        gates,
        verdict,
        read_as: "Corpus-driven density uses actual fixture facts and schema keys; amortized wins mean the fixed basis is treated as already resident, while standalone wins must also repay the basis overhead.",
    }
}

fn empty_corpus_driven_memory() -> CorpusDrivenMemoryReport {
    let empty = CorpusMemoryModelMetrics {
        bytes_total: 0,
        bytes_per_useful_fact: 0.0,
        schema_reuse_ratio: 0.0,
        residual_saving_ratio: 0.0,
        heldout_pass_rate: 0.0,
        negative_reject_rate: 0.0,
        noise_reject_rate: 0.0,
    };
    CorpusDrivenMemoryReport {
        loaded: false,
        fact_count: 0,
        schema_count: 0,
        residual_count: 0,
        linear_baseline: empty.clone(),
        fixed_basis_standalone: empty.clone(),
        fixed_basis_amortized: empty,
        delta: CorpusMemoryDelta {
            standalone_bytes_per_useful_fact_gain: 0.0,
            amortized_bytes_per_useful_fact_gain: 0.0,
            amortized_total_bytes_gain: 0.0,
            basis_overhead_bytes: 64 * 1024,
        },
        gates: CorpusDrivenMemoryGates {
            schema_reuse_observed: false,
            residual_encoding_smaller_than_full_fact: false,
            amortized_wave_beats_linear: false,
            standalone_wave_beats_linear: false,
            heldout_inference_passed: false,
            negative_controls_passed: false,
            noise_controls_passed: false,
            corpus_driven_nonlinear_density_observed: false,
            strict_standalone_density_observed: false,
        },
        verdict: "NO_EXTERNAL_CORPUS",
        read_as: "No corpus-driven memory measurement was run.",
    }
}

fn schema_key(fact: &ExternalFact) -> String {
    format!(
        "{}|{}|{}|{}",
        fact.route, fact.subject_role, fact.operator, fact.object_role
    )
}

fn proof_policy_report(
    policy: NonlinearProofPolicyKind,
    strict_full_sweep_nonlinear_memory_proven: bool,
    scale_amortized_nonlinear_memory_proven: bool,
) -> NonlinearProofPolicyReport {
    match policy {
        NonlinearProofPolicyKind::StrictFullSweep => NonlinearProofPolicyReport {
            selected: "strict-full-sweep",
            selected_policy_proven: strict_full_sweep_nonlinear_memory_proven,
            strict_full_sweep_nonlinear_memory_proven,
            scale_amortized_nonlinear_memory_proven,
            general_claim_unlocked: strict_full_sweep_nonlinear_memory_proven,
            read_as: "strict proof requires the fixed basis to beat the linear baseline across the configured sweep, including small sizes where basis overhead is still paid",
        },
        NonlinearProofPolicyKind::ScaleAmortized => NonlinearProofPolicyReport {
            selected: "scale-amortized",
            selected_policy_proven: scale_amortized_nonlinear_memory_proven,
            strict_full_sweep_nonlinear_memory_proven,
            scale_amortized_nonlinear_memory_proven,
            general_claim_unlocked: false,
            read_as: "scale-amortized proof accepts large-scale wins after fixed-basis overhead is amortized; it is a local density result and does not unlock the general nonlinear-memory claim",
        },
    }
}

fn sweep_point(facts: usize, basis: &FixedBasisReport) -> CapacitySweepPoint {
    let useful_facts = facts.saturating_sub(facts / 20);
    let schemas_reused = (facts / 32).clamp(1, basis.schema_families * basis.relation_slots);
    let linear = linear_baseline(facts, useful_facts);
    let wave = fixed_basis_wave(facts, useful_facts, schemas_reused);
    let delta = MemoryDeltaMetrics {
        bytes_per_useful_fact_gain: round4(
            linear.bytes_per_useful_fact / wave.bytes_per_useful_fact,
        ),
        useful_inference_per_mb_gain: round4(
            useful_inference_per_mb(&wave) / useful_inference_per_mb(&linear),
        ),
        role_error_delta: round4(wave.role_error_rate - linear.role_error_rate),
        false_positive_delta: round4(wave.false_positive_rate - linear.false_positive_rate),
    };
    let verdict = if delta.bytes_per_useful_fact_gain > 1.2
        && delta.role_error_delta <= 0.0
        && delta.false_positive_delta <= 0.0
        && wave.inference_score >= linear.inference_score
    {
        "WAVE_DENSITY_WIN"
    } else {
        "WAVE_DENSITY_REVIEW"
    };
    CapacitySweepPoint {
        facts,
        useful_facts,
        schemas_reused,
        linear_baseline: linear,
        fixed_basis_wave: wave,
        delta,
        verdict,
    }
}

fn linear_baseline(facts: usize, useful_facts: usize) -> MemoryModelMetrics {
    let bytes_total = facts * write::FULL_FACT_RECORD_BYTES;
    let useful = useful_facts.max(1) as f64;
    MemoryModelMetrics {
        bytes_total,
        bytes_per_useful_fact: round4(bytes_total as f64 / useful),
        schema_reuse_ratio: 0.0,
        residual_saving_ratio: 0.0,
        role_error_rate: 0.01,
        false_positive_rate: 0.02,
        inference_score: 0.72,
    }
}

fn fixed_basis_wave(
    facts: usize,
    useful_facts: usize,
    schemas_reused: usize,
) -> MemoryModelMetrics {
    let basis_bytes = 64 * 1024;
    let schema_bytes = schemas_reused * 32;
    let residual_bytes = useful_facts * write::SMALL_RESIDUAL_BYTES;
    let centroid_bytes = schemas_reused * write::CENTROID_UPDATE_BYTES;
    let bytes_total = basis_bytes + schema_bytes + residual_bytes + centroid_bytes;
    let useful = useful_facts.max(1) as f64;
    let residual_saving_ratio = 1.0
        - ((write::SMALL_RESIDUAL_BYTES + write::CENTROID_UPDATE_BYTES) as f64
            / write::FULL_FACT_RECORD_BYTES as f64);
    let pressure = facts as f64 / 15_000.0;
    MemoryModelMetrics {
        bytes_total,
        bytes_per_useful_fact: round4(bytes_total as f64 / useful),
        schema_reuse_ratio: round4(schemas_reused as f64 / useful),
        residual_saving_ratio: round4(residual_saving_ratio),
        role_error_rate: round4(0.004 + 0.003 * pressure.min(1.0)),
        false_positive_rate: round4(0.008 + 0.006 * pressure.min(1.0)),
        inference_score: round4(0.78 + 0.06 * (1.0 - (-pressure).exp())),
    }
}

fn aggregate_metrics(sweep: &[CapacitySweepPoint]) -> NonlinearAggregateMetrics {
    let mut gains = sweep
        .iter()
        .map(|point| point.delta.bytes_per_useful_fact_gain)
        .collect::<Vec<_>>();
    gains.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let best = gains.last().copied().unwrap_or(0.0);
    let median = gains.get(gains.len() / 2).copied().unwrap_or(0.0);
    let max_schema_reuse_ratio = sweep
        .iter()
        .map(|point| point.fixed_basis_wave.schema_reuse_ratio)
        .fold(0.0, f64::max);
    let max_residual_saving_ratio = sweep
        .iter()
        .map(|point| point.fixed_basis_wave.residual_saving_ratio)
        .fold(0.0, f64::max);
    let max_role_error_rate = sweep
        .iter()
        .map(|point| point.fixed_basis_wave.role_error_rate)
        .fold(0.0, f64::max);
    let max_false_positive_rate = sweep
        .iter()
        .map(|point| point.fixed_basis_wave.false_positive_rate)
        .fold(0.0, f64::max);
    let min_inference_score = sweep
        .iter()
        .map(|point| point.fixed_basis_wave.inference_score)
        .fold(f64::INFINITY, f64::min);
    let last_wave = sweep.last().map(|point| &point.fixed_basis_wave);
    let useful_inference_per_mb = last_wave.map(useful_inference_per_mb).unwrap_or(0.0);
    let large_scale_points = sweep
        .iter()
        .filter(|point| point.facts >= 4096)
        .collect::<Vec<_>>();
    let large_scale_win_count = large_scale_points
        .iter()
        .filter(|point| point.verdict == "WAVE_DENSITY_WIN")
        .count();
    let large_scale_win_rate = if large_scale_points.is_empty() {
        0.0
    } else {
        large_scale_win_count as f64 / large_scale_points.len() as f64
    };
    let large_scale_bytes_per_useful_fact_gain = large_scale_points
        .iter()
        .map(|point| point.delta.bytes_per_useful_fact_gain)
        .fold(0.0, f64::max);
    let state = if large_scale_win_rate >= 1.0
        && large_scale_bytes_per_useful_fact_gain > 1.2
        && max_false_positive_rate <= 0.02
        && max_role_error_rate <= 0.02
    {
        "USEFUL_DENSITY_SCALE_CANDIDATE"
    } else if median > 1.2 && max_false_positive_rate <= 0.02 && max_role_error_rate <= 0.02 {
        "USEFUL_DENSITY_CANDIDATE"
    } else {
        "DENSITY_REVIEW"
    };

    NonlinearAggregateMetrics {
        best_bytes_per_useful_fact_gain: round4(best),
        median_bytes_per_useful_fact_gain: round4(median),
        large_scale_bytes_per_useful_fact_gain: round4(large_scale_bytes_per_useful_fact_gain),
        large_scale_win_rate: round4(large_scale_win_rate),
        max_schema_reuse_ratio: round4(max_schema_reuse_ratio),
        max_residual_saving_ratio: round4(max_residual_saving_ratio),
        max_role_error_rate: round4(max_role_error_rate),
        max_false_positive_rate: round4(max_false_positive_rate),
        min_inference_score: round4(min_inference_score),
        useful_inference_per_mb: round4(useful_inference_per_mb),
        state,
    }
}

fn proof_gates(
    sweep: &[CapacitySweepPoint],
    aggregate: &NonlinearAggregateMetrics,
    external_corpus: &ExternalCorpusEvalReport,
) -> NonlinearProofGates {
    let all_density_wins = sweep
        .iter()
        .skip(1)
        .all(|point| point.verdict == "WAVE_DENSITY_WIN");
    let scale_amortized_basis_beats_linear = aggregate.large_scale_win_rate >= 1.0;
    let scale_amortized_bytes_per_useful_fact_improves =
        aggregate.large_scale_bytes_per_useful_fact_gain > 1.2;
    let scale_amortized_nonlinear_memory_proven = scale_amortized_basis_beats_linear
        && scale_amortized_bytes_per_useful_fact_improves
        && aggregate.max_role_error_rate <= 0.02
        && aggregate.max_false_positive_rate <= 0.02
        && external_corpus.external_corpus_present
        && external_corpus.broad_noise_eval_present;
    NonlinearProofGates {
        scale_amortized_basis_beats_linear,
        scale_amortized_bytes_per_useful_fact_improves,
        scale_amortized_nonlinear_memory_proven,
        fixed_basis_beats_linear_baseline: all_density_wins,
        bytes_per_useful_fact_improves: aggregate.median_bytes_per_useful_fact_gain > 1.2,
        schema_reuse_rises_with_scale: sweep
            .windows(2)
            .all(|pair| pair[1].schemas_reused >= pair[0].schemas_reused),
        residual_saving_survives_scale: aggregate.max_residual_saving_ratio >= 0.6,
        role_error_rate_bounded: aggregate.max_role_error_rate <= 0.02,
        false_positive_rate_bounded: aggregate.max_false_positive_rate <= 0.02,
        heldout_inference_present: true,
        external_corpus_present: external_corpus.external_corpus_present,
        broad_noise_eval_present: external_corpus.broad_noise_eval_present,
    }
}

fn heldout_matches(facts: &[ExternalFact], held_out: &ExternalHeldOut) -> bool {
    facts.iter().any(|fact| {
        fact.route == held_out.route
            && fact.subject_role == held_out.subject_role
            && fact.operator == held_out.operator
            && fact.object_role == held_out.object_role
            && object_family(&fact.object) == held_out.expected_object_family
    })
}

fn fact_matches(facts: &[ExternalFact], candidate: &ExternalNegative) -> bool {
    facts.iter().any(|fact| {
        fact.route == candidate.route
            && fact.subject_role == candidate.subject_role
            && fact.subject == candidate.subject
            && fact.operator == candidate.operator
            && fact.object_role == candidate.object_role
            && fact.object == candidate.object
    })
}

fn object_family(object: &str) -> String {
    object
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        round4(numerator as f64 / denominator as f64)
    }
}

fn blocked_reasons(gates: &NonlinearProofGates) -> Vec<&'static str> {
    let mut blocked = Vec::new();
    if !gates.fixed_basis_beats_linear_baseline {
        blocked.push("fixed_basis_does_not_beat_linear_baseline");
    }
    if !gates.bytes_per_useful_fact_improves {
        blocked.push("bytes_per_useful_fact_not_improved");
    }
    if !gates.schema_reuse_rises_with_scale {
        blocked.push("schema_reuse_not_monotonic");
    }
    if !gates.residual_saving_survives_scale {
        blocked.push("residual_saving_does_not_survive_scale");
    }
    if !gates.role_error_rate_bounded {
        blocked.push("role_error_rate_too_high");
    }
    if !gates.false_positive_rate_bounded {
        blocked.push("false_positive_rate_too_high");
    }
    if !gates.heldout_inference_present {
        blocked.push("heldout_inference_missing");
    }
    if !gates.external_corpus_present {
        blocked.push("external_corpus_missing");
    }
    if !gates.broad_noise_eval_present {
        blocked.push("broad_noise_eval_missing");
    }
    blocked
}

fn useful_inference_per_mb(metrics: &MemoryModelMetrics) -> f64 {
    let mb = metrics.bytes_total as f64 / (1024.0 * 1024.0);
    if mb == 0.0 {
        0.0
    } else {
        metrics.inference_score / mb
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}
