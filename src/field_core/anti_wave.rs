use serde::{Deserialize, Serialize};

use super::FieldVector1024;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct AntiWaveSummary {
    pub active: bool,
    pub lanes: usize,
    pub suppressed_target: Option<String>,
    pub suppression_energy: f64,
    pub local_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AntiWaveApplication {
    pub before_alignment: f64,
    pub after_alignment: f64,
    pub delta: f64,
    pub suppressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct FieldAntiWaveEffect {
    pub active: bool,
    pub lanes: usize,
    pub suppressed_target: Option<String>,
    pub suppression_energy: f64,
    pub local_only: bool,
    pub decision_effect: String,
}

pub(crate) fn summarize_anti_wave_effect(summary: &AntiWaveSummary) -> FieldAntiWaveEffect {
    let decision_effect = if !summary.active || summary.lanes == 0 {
        "NO_ANTI_WAVE"
    } else if summary.local_only {
        "LOCAL_SUPPRESSION"
    } else {
        "GLOBAL_SUPPRESSION_REVIEW"
    };
    FieldAntiWaveEffect {
        active: summary.active,
        lanes: summary.lanes,
        suppressed_target: summary.suppressed_target.clone(),
        suppression_energy: summary.suppression_energy,
        local_only: summary.local_only,
        decision_effect: decision_effect.to_string(),
    }
}

pub(crate) fn apply_anti_wave(
    field: &FieldVector1024,
    anti_wave: &FieldVector1024,
    strength: i32,
) -> (FieldVector1024, AntiWaveApplication) {
    let before_alignment = field.cosine(anti_wave);
    let mut reduced = field.clone();
    reduced.subtract_scaled(anti_wave, strength.max(1));
    let after_alignment = reduced.cosine(anti_wave);
    let delta = before_alignment - after_alignment;
    (
        reduced,
        AntiWaveApplication {
            before_alignment: round4(before_alignment),
            after_alignment: round4(after_alignment),
            delta: round4(delta),
            suppressed: delta > 0.0,
        },
    )
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}
