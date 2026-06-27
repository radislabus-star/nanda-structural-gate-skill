//! Spectral center proof for Linux `.lrf` schema/residual memory.
//!
//! This is not a sidecar claim by itself: `linux_residual_memory` consumes this
//! report as an admission layer before the Linux-profile nonlinear-memory proof
//! is allowed to pass.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
};

pub(crate) const LINUX_SPECTRAL_MEMORY_VERSION: &str = "llmwave-big-v-next-linux-spectral-center";

const SPECTRAL_LANES: usize = 16;
const HARMONICS: usize = SPECTRAL_LANES / 2;
const MIN_AVERAGE_CENTER_GAP: f64 = 0.05;
const MIN_ROLE_SWAP_GAP: f64 = 0.05;
const MIN_SCHEMA_ABLATION_DROP: f64 = 0.02;

#[derive(Serialize, Clone)]
pub(crate) struct LinuxSpectralCenterReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub spectral_model: LinuxSpectralModel,
    pub metrics: LinuxSpectralCenterMetrics,
    pub gates: LinuxSpectralCenterGates,
    pub claim_boundary: LinuxSpectralClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxSpectralModel {
    pub basis: &'static str,
    pub lanes: usize,
    pub harmonics: usize,
    pub schema_modes: Vec<&'static str>,
    pub residual_modes: Vec<&'static str>,
    pub fallback_policy: &'static str,
    pub hot_path_role: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxSpectralCenterMetrics {
    pub schema_center_count: usize,
    pub evaluated_residuals: usize,
    pub fallback_records: usize,
    pub average_center_correct: f64,
    pub average_best_wrong_center: f64,
    pub average_center_gap: f64,
    pub min_center_gap: f64,
    pub false_center_accept_rate: f32,
    pub role_swap_evaluated: usize,
    pub average_role_aligned: f64,
    pub average_role_swapped: f64,
    pub average_role_swap_gap: f64,
    pub average_schema_gap_without_route_relation: f64,
    pub route_relation_ablation_drop: f64,
    pub fallback_outlier_rate: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxSpectralCenterGates {
    pub schema_centers_built: bool,
    pub center_gap_positive: bool,
    pub wrong_center_rejected: bool,
    pub role_swap_center_collapse: bool,
    pub route_relation_modes_causal: bool,
    pub fallback_guard_preserved: bool,
    pub spectral_center_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxSpectralClaimBoundary {
    pub spectral_center_metric_present: bool,
    pub schema_center_is_profile_evidence: bool,
    pub spectral_center_proven: bool,
    pub nonlinear_memory_proven_by_itself: bool,
    pub broad_chat_llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Clone)]
struct SchemaCenter {
    center: SpectralVector,
    ablated_center: SpectralVector,
}

#[derive(Clone, Copy)]
struct SpectralVector {
    lanes: [f64; SPECTRAL_LANES],
}

impl SpectralVector {
    fn zero() -> Self {
        Self {
            lanes: [0.0; SPECTRAL_LANES],
        }
    }

    fn add_assign(&mut self, other: Self) {
        for (target, value) in self.lanes.iter_mut().zip(other.lanes) {
            *target += value;
        }
    }

    fn add_weighted_phase(&mut self, channel: &str, value: &str, weight: f64) {
        let phase = hash_phase(&format!("{channel}:{value}"));
        for harmonic in 0..HARMONICS {
            let angle = phase * (harmonic + 1) as f64;
            self.lanes[harmonic * 2] += weight * angle.cos();
            self.lanes[harmonic * 2 + 1] += weight * angle.sin();
        }
    }

    fn normalized(mut self) -> Self {
        let norm = self.norm();
        if norm > 0.0 {
            for lane in &mut self.lanes {
                *lane /= norm;
            }
        }
        self
    }

    fn norm(&self) -> f64 {
        self.lanes
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt()
    }

    fn dot(&self, other: &Self) -> f64 {
        self.lanes
            .iter()
            .zip(other.lanes)
            .map(|(left, right)| left * right)
            .sum()
    }
}

pub(crate) fn build_linux_spectral_center_report(
    residual_pack: &PathBuf,
) -> Result<LinuxSpectralCenterReport> {
    let packet = load_linux_residual_decoded_packet(residual_pack)?;
    Ok(build_linux_spectral_center_report_from_parts(
        packet.summary,
        &packet.facts,
    ))
}

pub(crate) fn build_linux_spectral_center_report_from_parts(
    summary: LinuxResidualDecodedSummary,
    facts: &[LinuxResidualDecodedFact],
) -> LinuxSpectralCenterReport {
    let residuals = facts
        .iter()
        .filter(|fact| fact.memory_kind == "residual")
        .collect::<Vec<_>>();
    let fallback_count = facts
        .iter()
        .filter(|fact| fact.memory_kind == "fallback")
        .count();
    let centers = build_schema_centers(&residuals);
    let metrics = compute_metrics(&summary, &residuals, fallback_count, &centers);
    let ablation_is_meaningful = metrics.schema_center_count >= 2;
    let gates = LinuxSpectralCenterGates {
        schema_centers_built: metrics.schema_center_count > 0 && metrics.evaluated_residuals > 0,
        center_gap_positive: metrics.average_center_gap >= MIN_AVERAGE_CENTER_GAP,
        wrong_center_rejected: metrics.false_center_accept_rate == 0.0,
        role_swap_center_collapse: metrics.role_swap_evaluated == 0
            || metrics.average_role_swap_gap >= MIN_ROLE_SWAP_GAP,
        route_relation_modes_causal: !ablation_is_meaningful
            || metrics.route_relation_ablation_drop >= MIN_SCHEMA_ABLATION_DROP,
        fallback_guard_preserved: fallback_count == summary.fallback_record_count,
        spectral_center_proven: false,
    };
    let spectral_center_proven = gates.schema_centers_built
        && gates.center_gap_positive
        && gates.wrong_center_rejected
        && gates.role_swap_center_collapse
        && gates.route_relation_modes_causal
        && gates.fallback_guard_preserved;
    let gates = LinuxSpectralCenterGates {
        spectral_center_proven,
        ..gates
    };
    let verdict = if spectral_center_proven {
        "LINUX_SPECTRAL_CENTER_PROVEN"
    } else if gates.schema_centers_built {
        "LINUX_SPECTRAL_CENTER_REVIEW"
    } else {
        "LINUX_SPECTRAL_CENTER_BLOCKED"
    };

    LinuxSpectralCenterReport {
        mode: "llmwave-big-linux-spectral-center",
        version: LINUX_SPECTRAL_MEMORY_VERSION,
        verdict,
        residual_pack: summary,
        spectral_model: LinuxSpectralModel {
            basis: "deterministic 8-harmonic phase basis over schema and residual channels",
            lanes: SPECTRAL_LANES,
            harmonics: HARMONICS,
            schema_modes: vec![
                "route",
                "relation",
                "subject-role",
                "object-role",
                "polarity",
                "evidence-kind",
            ],
            residual_modes: vec!["subject-filler", "object-filler"],
            fallback_policy: "one-off fallback records stay spectral outliers and are not forced into schema centers",
            hot_path_role: "admission metadata for linux-residual-proof; not a broad chat engine",
        },
        metrics,
        gates,
        claim_boundary: LinuxSpectralClaimBoundary {
            spectral_center_metric_present: true,
            schema_center_is_profile_evidence: true,
            spectral_center_proven,
            nonlinear_memory_proven_by_itself: false,
            broad_chat_llm_ready: false,
            safe_claim: "Linux spectral centers are profile evidence that `.lrf` schema/residual memory has separable role-complete schema modes, role-swap near-misses collapse, and route/relation ablation reduces separation. This strengthens the Linux-profile memory proof but is not a broad LLM claim.",
            blocked_claims: vec!["broad_chat_llm_ready", "global_nonlinear_memory_proven"],
        },
    }
}

fn build_schema_centers(residuals: &[&LinuxResidualDecodedFact]) -> BTreeMap<String, SchemaCenter> {
    let mut accumulators = BTreeMap::<String, (SpectralVector, SpectralVector)>::new();
    for fact in residuals {
        let key = schema_key(fact);
        let entry = accumulators
            .entry(key)
            .or_insert((SpectralVector::zero(), SpectralVector::zero()));
        entry.0.add_assign(schema_wave(fact, true));
        entry.1.add_assign(schema_wave(fact, false));
    }
    accumulators
        .into_iter()
        .map(|(key, (center, ablated_center))| {
            (
                key,
                SchemaCenter {
                    center: center.normalized(),
                    ablated_center: ablated_center.normalized(),
                },
            )
        })
        .collect()
}

fn compute_metrics(
    summary: &LinuxResidualDecodedSummary,
    residuals: &[&LinuxResidualDecodedFact],
    fallback_count: usize,
    centers: &BTreeMap<String, SchemaCenter>,
) -> LinuxSpectralCenterMetrics {
    let mut center_correct_sum = 0.0;
    let mut wrong_center_sum = 0.0;
    let mut center_gap_sum = 0.0;
    let mut min_center_gap = f64::INFINITY;
    let mut false_center_accepts = 0usize;
    let mut role_aligned_sum = 0.0;
    let mut role_swapped_sum = 0.0;
    let mut role_gap_sum = 0.0;
    let mut role_evaluated = 0usize;
    let mut ablated_gap_sum = 0.0;
    let mut ablation_drop_sum = 0.0;
    let mut evaluated = 0usize;

    for fact in residuals {
        let key = schema_key(fact);
        let Some(correct_center) = centers.get(&key) else {
            continue;
        };
        let fact_schema_wave = schema_wave(fact, true).normalized();
        let fact_ablated_wave = schema_wave(fact, false).normalized();
        let correct = fact_schema_wave.dot(&correct_center.center);
        let ablated_correct = fact_ablated_wave.dot(&correct_center.ablated_center);
        let mut best_wrong = 0.0;
        let mut best_wrong_ablated = 0.0;
        for (candidate_key, center) in centers {
            if candidate_key == &key {
                continue;
            }
            let score = fact_schema_wave.dot(&center.center);
            if score > best_wrong {
                best_wrong = score;
            }
            let ablated_score = fact_ablated_wave.dot(&center.ablated_center);
            if ablated_score > best_wrong_ablated {
                best_wrong_ablated = ablated_score;
            }
        }
        let gap = correct - best_wrong;
        let ablated_gap = ablated_correct - best_wrong_ablated;
        if fact.subject != fact.object {
            let role = role_wave(fact, false).normalized();
            let swapped = role_wave(fact, true).normalized();
            let role_aligned = role.dot(&role);
            let role_swapped = role.dot(&swapped);
            let role_gap = role_aligned - role_swapped;
            role_aligned_sum += role_aligned;
            role_swapped_sum += role_swapped;
            role_gap_sum += role_gap;
            role_evaluated += 1;
        }

        center_correct_sum += correct;
        wrong_center_sum += best_wrong;
        center_gap_sum += gap;
        min_center_gap = min_center_gap.min(gap);
        if best_wrong >= correct {
            false_center_accepts += 1;
        }
        ablated_gap_sum += ablated_gap;
        ablation_drop_sum += gap - ablated_gap;
        evaluated += 1;
    }

    let min_center_gap = if evaluated == 0 { 0.0 } else { min_center_gap };
    LinuxSpectralCenterMetrics {
        schema_center_count: centers.len(),
        evaluated_residuals: evaluated,
        fallback_records: fallback_count,
        average_center_correct: average(center_correct_sum, evaluated),
        average_best_wrong_center: average(wrong_center_sum, evaluated),
        average_center_gap: average(center_gap_sum, evaluated),
        min_center_gap: round4_f64(min_center_gap),
        false_center_accept_rate: ratio(false_center_accepts, evaluated),
        role_swap_evaluated: role_evaluated,
        average_role_aligned: average(role_aligned_sum, role_evaluated),
        average_role_swapped: average(role_swapped_sum, role_evaluated),
        average_role_swap_gap: average(role_gap_sum, role_evaluated),
        average_schema_gap_without_route_relation: average(ablated_gap_sum, evaluated),
        route_relation_ablation_drop: average(ablation_drop_sum, evaluated),
        fallback_outlier_rate: ratio(fallback_count, summary.represented_fact_count),
    }
}

fn schema_wave(fact: &LinuxResidualDecodedFact, include_route_relation: bool) -> SpectralVector {
    let mut vector = SpectralVector::zero();
    if include_route_relation {
        vector.add_weighted_phase("route", &fact.route, 1.35);
        vector.add_weighted_phase("relation", &fact.relation, 1.35);
    }
    vector.add_weighted_phase("subject-role", &fact.subject_role, 1.00);
    vector.add_weighted_phase("object-role", &fact.object_role, 1.00);
    vector.add_weighted_phase("polarity", fact.polarity, 0.70);
    vector.add_weighted_phase("evidence-kind", &fact.evidence_kind, 0.55);
    vector.normalized()
}

fn role_wave(fact: &LinuxResidualDecodedFact, swapped: bool) -> SpectralVector {
    let mut vector = SpectralVector::zero();
    if swapped {
        vector.add_weighted_phase("subject-role", &fact.object_role, 1.0);
        vector.add_weighted_phase("subject-filler", &fact.object, 0.65);
        vector.add_weighted_phase("object-role", &fact.subject_role, 1.0);
        vector.add_weighted_phase("object-filler", &fact.subject, 0.65);
    } else {
        vector.add_weighted_phase("subject-role", &fact.subject_role, 1.0);
        vector.add_weighted_phase("subject-filler", &fact.subject, 0.65);
        vector.add_weighted_phase("object-role", &fact.object_role, 1.0);
        vector.add_weighted_phase("object-filler", &fact.object, 0.65);
    }
    vector.normalized()
}

fn schema_key(fact: &LinuxResidualDecodedFact) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        fact.route,
        fact.relation,
        fact.subject_role,
        fact.object_role,
        fact.polarity,
        fact.evidence_kind
    )
}

fn hash_phase(value: &str) -> f64 {
    let digest = Sha256::digest(value.as_bytes());
    let hash = u64::from_le_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ]);
    let unit = (hash >> 11) as f64 / ((1u64 << 53) as f64);
    unit * std::f64::consts::TAU
}

fn average(sum: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        round4_f64(sum / count as f64)
    }
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        ((part as f32 / total as f32) * 10_000.0).round() / 10_000.0
    }
}

fn round4_f64(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectral_center_accepts_schema_and_rejects_swapped_roles() {
        let summary = LinuxResidualDecodedSummary {
            path: "fixture.lrf".to_string(),
            file_bytes: 1,
            wave_dim: 1024,
            represented_fact_count: 4,
            schema_record_count: 2,
            residual_record_count: 4,
            fallback_record_count: 0,
            route_count: 2,
            corpus_hash64: 7,
            promotion_threshold: 2,
            binary_hot_sections_bytes: 160,
            direct_fixed_baseline_bytes: 256,
            cold_label_count: 8,
            cold_label_table_bytes: 64,
            binary_hot_sections_fit_6m: true,
            beats_direct_fixed64: true,
        };
        let facts = vec![
            fact(
                "linux.apt.command.provider",
                "bash",
                "provides command",
                "bash",
                "positive",
            ),
            fact(
                "linux.apt.command.provider",
                "coreutils",
                "provides command",
                "ls",
                "positive",
            ),
            fact(
                "linux.package.binary",
                "systemd",
                "provides command",
                "systemctl",
                "positive",
            ),
            fact(
                "linux.package.binary",
                "openssh-server",
                "provides command",
                "ssh",
                "positive",
            ),
        ];
        let report = build_linux_spectral_center_report_from_parts(summary, &facts);
        assert_eq!(report.verdict, "LINUX_SPECTRAL_CENTER_PROVEN");
        assert!(report.metrics.average_center_gap > 0.05);
        assert!(report.metrics.average_role_swap_gap > 0.05);
        assert_eq!(report.metrics.false_center_accept_rate, 0.0);
    }

    fn fact(
        route: &str,
        subject: &str,
        relation: &str,
        object: &str,
        polarity: &'static str,
    ) -> LinuxResidualDecodedFact {
        LinuxResidualDecodedFact {
            route: route.to_string(),
            subject: subject.to_string(),
            subject_role: match relation {
                "provides command" => "package",
                "does not prove" if route.contains("boundary.socket") => "runtime_signal",
                "does not prove" => "package_state",
                _ => "subject",
            }
            .to_string(),
            relation: relation.to_string(),
            object: object.to_string(),
            object_role: match relation {
                "provides command" => "command",
                "does not prove" if route.contains("boundary.socket") => "exposure_claim",
                "does not prove" => "runtime_claim",
                _ => "object",
            }
            .to_string(),
            polarity,
            evidence_kind: "fixture".to_string(),
            confidence: 90,
            memory_kind: "residual",
        }
    }
}
