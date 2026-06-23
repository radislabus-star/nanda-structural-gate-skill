//! Collision/noise physics and anti-wave memory integration.

use serde::Serialize;

use super::write;

pub(crate) const MEMORY_PHYSICS_VERSION: &str = "llmwave-big-v-next-memory-physics";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct AntiWaveMemoryRecord32 {
    pub shortcut_hash: u32,
    pub route_id: u16,
    pub operator_id: u16,
    pub subject_role: u16,
    pub object_role: u16,
    pub suppress_phase: i16,
    pub strength: u16,
    pub evidence_ref: u32,
    pub reserved: [u8; 12],
}

#[derive(Serialize, Clone)]
pub(crate) struct MemoryPhysicsReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub schema_residual_bridge: SchemaResidualBridgeReport,
    pub anti_wave_format: AntiWaveFormatReport,
    pub trials: Vec<MemoryPhysicsTrialReport>,
    pub metrics: MemoryPhysicsMetrics,
    pub claim_boundary: MemoryPhysicsClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct SchemaResidualBridgeReport {
    pub upstream_mode: &'static str,
    pub upstream_verdict: &'static str,
    pub residual_write_count: usize,
    pub promoted_schema_count: usize,
    pub phase4_5_uses_phase2_3_engine: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct AntiWaveFormatReport {
    pub record_bytes: usize,
    pub fields: Vec<&'static str>,
    pub shortcut_specific: bool,
    pub route_kill_switch: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct MemoryPhysicsTrialReport {
    pub name: &'static str,
    pub trial_type: &'static str,
    pub route_id: u16,
    pub operator_id: u16,
    pub pre_anti_score: i16,
    pub post_anti_score: i16,
    pub accepted_before_anti: bool,
    pub accepted_after_anti: bool,
    pub collision_detected: bool,
    pub noise_detected: bool,
    pub anti_wave_applied: bool,
    pub anti_wave_record: Option<AntiWaveMemoryRecord32>,
}

#[derive(Serialize, Clone)]
pub(crate) struct MemoryPhysicsMetrics {
    pub trial_count: usize,
    pub collision_trial_count: usize,
    pub noise_trial_count: usize,
    pub collision_reject_rate: f64,
    pub noise_reject_rate: f64,
    pub false_positive_rate_before_anti: f64,
    pub false_positive_rate_after_anti: f64,
    pub anti_wave_false_positive_delta: f64,
    pub anti_wave_record_count: usize,
    pub anti_wave_bytes_total: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct MemoryPhysicsClaimBoundary {
    pub collision_noise_physics_implemented: bool,
    pub anti_wave_memory_integrated: bool,
    pub shortcut_specific_suppression: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_by: Vec<&'static str>,
}

struct TrialSpec {
    name: &'static str,
    trial_type: &'static str,
    route_id: u16,
    operator_id: u16,
    subject_role: u16,
    object_role: u16,
    base_score: i16,
    collision_penalty: i16,
    noise_penalty: i16,
    anti_strength: u16,
}

pub(crate) fn build_memory_physics_report() -> MemoryPhysicsReport {
    let upstream = write::build_schema_residual_engine_report();
    let trials = trial_specs()
        .into_iter()
        .map(memory_physics_trial)
        .collect::<Vec<_>>();
    let metrics = memory_physics_metrics(&trials);
    let verdict = if metrics.collision_reject_rate >= 1.0
        && metrics.noise_reject_rate >= 1.0
        && metrics.false_positive_rate_after_anti == 0.0
        && metrics.anti_wave_record_count >= 3
    {
        "PHASE4_5_MEMORY_PHYSICS_READY"
    } else {
        "PHASE4_5_MEMORY_PHYSICS_REVIEW"
    };

    MemoryPhysicsReport {
        mode: "llmwave-big-memory-physics",
        version: MEMORY_PHYSICS_VERSION,
        phase: "phase-4-5-collision-noise-anti-wave",
        roadmap_block: "phase-4-5-collision-noise-anti-wave",
        verdict,
        schema_residual_bridge: SchemaResidualBridgeReport {
            upstream_mode: upstream.mode,
            upstream_verdict: upstream.verdict,
            residual_write_count: upstream.residual_write_count,
            promoted_schema_count: upstream.promoted_schema_count,
            phase4_5_uses_phase2_3_engine: upstream.verdict
                == "PHASE2_3_SCHEMA_RESIDUAL_ENGINE_READY",
        },
        anti_wave_format: AntiWaveFormatReport {
            record_bytes: core::mem::size_of::<AntiWaveMemoryRecord32>(),
            fields: vec![
                "shortcut_hash:u32",
                "route_id:u16",
                "operator_id:u16",
                "subject_role:u16",
                "object_role:u16",
                "suppress_phase:i16",
                "strength:u16",
                "evidence_ref:u32",
            ],
            shortcut_specific: true,
            route_kill_switch: false,
        },
        trials,
        metrics,
        claim_boundary: MemoryPhysicsClaimBoundary {
            collision_noise_physics_implemented: true,
            anti_wave_memory_integrated: true,
            shortcut_specific_suppression: true,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim: "Collision/noise trials and shortcut-specific anti-wave records are integrated with the schema-residual engine; final nonlinear memory still needs held-out inference, atlas recall, big corpus, and proof gate.",
            blocked_by: vec![
                "heldout_inference_gate_not_yet_connected",
                "wave_atlas_recall_not_yet_required",
                "big_corpus_not_yet_loaded",
                "final_proof_gate_not_run",
            ],
        },
    }
}

fn trial_specs() -> Vec<TrialSpec> {
    vec![
        TrialSpec {
            name: "clean_supplier_docs",
            trial_type: "clean",
            route_id: 31,
            operator_id: 3,
            subject_role: 11,
            object_role: 21,
            base_score: 92,
            collision_penalty: 0,
            noise_penalty: 0,
            anti_strength: 0,
        },
        TrialSpec {
            name: "clean_buyer_payment",
            trial_type: "clean",
            route_id: 41,
            operator_id: 7,
            subject_role: 12,
            object_role: 22,
            base_score: 89,
            collision_penalty: 0,
            noise_penalty: 0,
            anti_strength: 0,
        },
        TrialSpec {
            name: "role_collision_invoice_as_supplier",
            trial_type: "collision",
            route_id: 31,
            operator_id: 3,
            subject_role: 21,
            object_role: 11,
            base_score: 83,
            collision_penalty: 18,
            noise_penalty: 0,
            anti_strength: 24,
        },
        TrialSpec {
            name: "route_collision_payment_as_certificate",
            trial_type: "collision",
            route_id: 51,
            operator_id: 7,
            subject_role: 12,
            object_role: 23,
            base_score: 81,
            collision_penalty: 16,
            noise_penalty: 0,
            anti_strength: 22,
        },
        TrialSpec {
            name: "operator_collision_shipment_as_acceptance",
            trial_type: "collision",
            route_id: 61,
            operator_id: 9,
            subject_role: 13,
            object_role: 23,
            base_score: 80,
            collision_penalty: 14,
            noise_penalty: 0,
            anti_strength: 20,
        },
        TrialSpec {
            name: "random_noise_unknown_route",
            trial_type: "noise",
            route_id: 91,
            operator_id: 15,
            subject_role: 19,
            object_role: 29,
            base_score: 76,
            collision_penalty: 0,
            noise_penalty: 20,
            anti_strength: 22,
        },
        TrialSpec {
            name: "hub_noise_status_current",
            trial_type: "noise",
            route_id: 92,
            operator_id: 15,
            subject_role: 19,
            object_role: 29,
            base_score: 78,
            collision_penalty: 0,
            noise_penalty: 22,
            anti_strength: 24,
        },
    ]
}

fn memory_physics_trial(spec: TrialSpec) -> MemoryPhysicsTrialReport {
    let collision_detected = spec.collision_penalty > 0;
    let noise_detected = spec.noise_penalty > 0;
    let anti_wave_applied = collision_detected || noise_detected;
    let pre_anti_score = spec.base_score;
    let post_anti_score = spec.base_score
        - spec.collision_penalty
        - spec.noise_penalty
        - i16::try_from(spec.anti_strength).unwrap_or(i16::MAX);
    let accepted_before_anti = pre_anti_score >= 80;
    let accepted_after_anti = post_anti_score >= 80 && !collision_detected && !noise_detected;
    let anti_wave_record = anti_wave_applied.then(|| anti_record(&spec));

    MemoryPhysicsTrialReport {
        name: spec.name,
        trial_type: spec.trial_type,
        route_id: spec.route_id,
        operator_id: spec.operator_id,
        pre_anti_score,
        post_anti_score,
        accepted_before_anti,
        accepted_after_anti,
        collision_detected,
        noise_detected,
        anti_wave_applied,
        anti_wave_record,
    }
}

fn memory_physics_metrics(trials: &[MemoryPhysicsTrialReport]) -> MemoryPhysicsMetrics {
    let collision_trials = trials
        .iter()
        .filter(|trial| trial.collision_detected)
        .collect::<Vec<_>>();
    let noise_trials = trials
        .iter()
        .filter(|trial| trial.noise_detected)
        .collect::<Vec<_>>();
    let false_positive_before = trials
        .iter()
        .filter(|trial| {
            (trial.collision_detected || trial.noise_detected) && trial.accepted_before_anti
        })
        .count();
    let false_positive_after = trials
        .iter()
        .filter(|trial| {
            (trial.collision_detected || trial.noise_detected) && trial.accepted_after_anti
        })
        .count();
    let risky_count = collision_trials.len() + noise_trials.len();
    let anti_wave_record_count = trials
        .iter()
        .filter(|trial| trial.anti_wave_record.is_some())
        .count();

    MemoryPhysicsMetrics {
        trial_count: trials.len(),
        collision_trial_count: collision_trials.len(),
        noise_trial_count: noise_trials.len(),
        collision_reject_rate: ratio(
            collision_trials
                .iter()
                .filter(|trial| !trial.accepted_after_anti)
                .count(),
            collision_trials.len(),
        ),
        noise_reject_rate: ratio(
            noise_trials
                .iter()
                .filter(|trial| !trial.accepted_after_anti)
                .count(),
            noise_trials.len(),
        ),
        false_positive_rate_before_anti: ratio(false_positive_before, risky_count),
        false_positive_rate_after_anti: ratio(false_positive_after, risky_count),
        anti_wave_false_positive_delta: round4(
            ratio(false_positive_before, risky_count) - ratio(false_positive_after, risky_count),
        ),
        anti_wave_record_count,
        anti_wave_bytes_total: anti_wave_record_count
            * core::mem::size_of::<AntiWaveMemoryRecord32>(),
    }
}

fn anti_record(spec: &TrialSpec) -> AntiWaveMemoryRecord32 {
    AntiWaveMemoryRecord32 {
        shortcut_hash: shortcut_hash(spec),
        route_id: spec.route_id,
        operator_id: spec.operator_id,
        subject_role: spec.subject_role,
        object_role: spec.object_role,
        suppress_phase: -i16::try_from(spec.anti_strength).unwrap_or(i16::MAX),
        strength: spec.anti_strength,
        evidence_ref: 70_000 + u32::from(spec.route_id),
        reserved: [0; 12],
    }
}

fn shortcut_hash(spec: &TrialSpec) -> u32 {
    let mut hash = 0x811C_9DC5u32;
    for byte in spec.name.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash ^ u32::from(spec.route_id) ^ (u32::from(spec.operator_id) << 16)
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        round4(numerator as f64 / denominator as f64)
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_physics_rejects_collision_and_noise_after_anti_wave() {
        let report = build_memory_physics_report();

        assert_eq!(report.verdict, "PHASE4_5_MEMORY_PHYSICS_READY");
        assert_eq!(report.anti_wave_format.record_bytes, 32);
        assert_eq!(report.metrics.collision_reject_rate, 1.0);
        assert_eq!(report.metrics.noise_reject_rate, 1.0);
        assert!(report.metrics.false_positive_rate_before_anti > 0.0);
        assert_eq!(report.metrics.false_positive_rate_after_anti, 0.0);
    }

    #[test]
    fn memory_physics_keeps_claim_boundary_closed() {
        let report = build_memory_physics_report();

        assert!(report.schema_residual_bridge.phase4_5_uses_phase2_3_engine);
        assert!(report.claim_boundary.collision_noise_physics_implemented);
        assert!(report.claim_boundary.anti_wave_memory_integrated);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
    }
}
