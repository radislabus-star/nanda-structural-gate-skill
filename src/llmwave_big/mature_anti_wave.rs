//! Mature anti-wave layer built from blocking field lenses.

use serde::Serialize;

use super::lens_scan;
use crate::field_core;

pub(crate) const MATURE_ANTI_WAVE_VERSION: &str = "llmwave-big-v1210-mature-anti-wave";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct AntiLaneRecord32 {
    pub route_id: u32,
    pub lens_id: u16,
    pub lane_id: u16,
    pub suppress_score: i16,
    pub support_preserved: i16,
    pub locality_score: i16,
    pub residual_delta: i16,
    pub phase: u16,
    pub flags: u16,
    pub reserved: u32,
    pub reserved2: u32,
    pub reserved3: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct MatureAntiWaveReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub lens_bridge_verdict: &'static str,
    pub top_route: &'static str,
    pub anti_lanes: Vec<AntiLane>,
    pub field_after_anti: FieldAfterAnti,
    pub field_core_admission: field_core::FieldPassReport,
    pub metrics: MatureAntiWaveMetrics,
    pub claim_boundary: MatureAntiWaveClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct AntiLane {
    pub lens: &'static str,
    pub action: &'static str,
    pub reason: &'static str,
    pub record: AntiLaneRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct FieldAfterAnti {
    pub raw_field_state: &'static str,
    pub anti_field_state: &'static str,
    pub answer_decision: &'static str,
    pub suppress_total: i16,
    pub support_preserved_total: i16,
    pub locality_floor: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct MatureAntiWaveMetrics {
    pub lane_count: usize,
    pub suppress_total: i16,
    pub support_preserved_total: i16,
    pub locality_floor: i16,
    pub evidence_lane_rate: f32,
    pub causal_lane_rate: f32,
    pub answer_lane_rate: f32,
    pub unsafe_answer_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct MatureAntiWaveClaimBoundary {
    pub mature_anti_wave_implemented: bool,
    pub fixed_anti_lane_records: bool,
    pub uses_lens_scan_bridge: bool,
    pub local_suppression_only: bool,
    pub safe_to_answer: bool,
    pub full_field_mature: bool,
    pub chat_ready: bool,
    pub external_corpus_loaded: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_mature_anti_wave_report(input_text: String) -> MatureAntiWaveReport {
    let lens_report = lens_scan::build_lens_scan_report(input_text.clone());
    let anti_lanes = compile_anti_lanes(&lens_report);
    let suppress_total = anti_lanes
        .iter()
        .map(|lane| lane.record.suppress_score)
        .sum::<i16>();
    let support_preserved_total = anti_lanes
        .iter()
        .map(|lane| lane.record.support_preserved)
        .sum::<i16>();
    let locality_floor = anti_lanes
        .iter()
        .map(|lane| lane.record.locality_score)
        .min()
        .unwrap_or(0);
    let has_answer_lane = anti_lanes.iter().any(|lane| lane.lens == "answer");
    let field_core_admission = field_core_admission(lens_report.top_route, &anti_lanes);
    let verdict = if has_answer_lane && locality_floor >= 70 {
        "MATURE_ANTI_WAVE_READY_NOT_ANSWER"
    } else {
        "MATURE_ANTI_WAVE_REVIEW"
    };

    MatureAntiWaveReport {
        mode: "llmwave-big-mature-anti-wave",
        version: MATURE_ANTI_WAVE_VERSION,
        roadmap_block: "v1141-v1210",
        verdict,
        input_text,
        lens_bridge_verdict: lens_report.verdict,
        top_route: lens_report.top_route,
        field_after_anti: FieldAfterAnti {
            raw_field_state: lens_report.field_bridge_state,
            anti_field_state: if has_answer_lane {
                "SUPPRESSED_UNSUPPORTED_ANSWER"
            } else {
                "ANTI_WAVE_REVIEW"
            },
            answer_decision: "ANSWER_BLOCKED_BY_ANTI_WAVE",
            suppress_total,
            support_preserved_total,
            locality_floor,
        },
        field_core_admission,
        metrics: MatureAntiWaveMetrics {
            lane_count: anti_lanes.len(),
            suppress_total,
            support_preserved_total,
            locality_floor,
            evidence_lane_rate: rate(has_lane(&anti_lanes, "evidence")),
            causal_lane_rate: rate(has_lane(&anti_lanes, "causal")),
            answer_lane_rate: rate(has_answer_lane),
            unsafe_answer_rate: 0.0,
            state: verdict,
        },
        anti_lanes,
        claim_boundary: MatureAntiWaveClaimBoundary {
            mature_anti_wave_implemented: true,
            fixed_anti_lane_records: core::mem::size_of::<AntiLaneRecord32>() == 32,
            uses_lens_scan_bridge: true,
            local_suppression_only: true,
            safe_to_answer: false,
            full_field_mature: false,
            chat_ready: false,
            external_corpus_loaded: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Blocking lenses can compile local anti-wave lanes that suppress an unsupported answer without deleting the route peak",
        },
    }
}

fn field_core_admission(
    top_route: &'static str,
    anti_lanes: &[AntiLane],
) -> field_core::FieldPassReport {
    let query = field_core::FieldRecord::synthetic(
        "mature-anti-wave-query",
        field_core::FieldRecordKind::AntiWaveLane,
        "mature_anti_wave",
        "suppresses_false_answer_on",
        top_route,
        Some(top_route.to_string()),
        Some("mature-anti-wave".to_string()),
    );
    let records = vec![field_core::FieldRecord::synthetic(
        "mature-anti-wave-route",
        field_core::FieldRecordKind::L3Schema,
        "route_peak",
        "preserved_under_anti_wave",
        top_route,
        Some(top_route.to_string()),
        Some("mature-anti-wave".to_string()),
    )];
    let anti_waves = anti_lanes
        .iter()
        .map(|lane| field_core::FieldAntiWaveLane {
            id: format!("anti-lane-{}", lane.lens),
            target: lane.lens.to_string(),
            subject: "mature_anti_wave".to_string(),
            relation: lane.action.to_string(),
            object: lane.reason.to_string(),
            route: Some(top_route.to_string()),
            group: Some(lane.lens.to_string()),
            strength: (lane.record.suppress_score / 12).max(1) as i32,
        })
        .collect::<Vec<_>>();

    field_core::run_field_pass(&field_core::FieldPassInput {
        family: field_core::FieldFamily::Cognitive,
        query,
        records,
        lenses: vec![
            field_core::FieldLensOperation {
                kind: field_core::FieldLensKind::Route,
                label: top_route.to_string(),
                strength: 1,
            },
            field_core::FieldLensOperation {
                kind: field_core::FieldLensKind::Evidence,
                label: "anti-wave-locality".to_string(),
                strength: 1,
            },
        ],
        anti_waves,
        state_hint: Some("FIELD_THIN".to_string()),
        claim_boundary: field_core::FieldClaimBoundary::default(),
    })
}

fn compile_anti_lanes(report: &lens_scan::LensScanReport) -> Vec<AntiLane> {
    report
        .lenses
        .iter()
        .filter(|lens| lens.state == "WATCH" || lens.state == "BLOCK")
        .enumerate()
        .map(|(index, lens)| anti_lane(index as u16 + 1, lens))
        .collect()
}

fn anti_lane(lane_id: u16, lens: &lens_scan::LensResult) -> AntiLane {
    let (action, suppress, preserved, locality, residual_delta, phase) = match lens.lens {
        "evidence" => ("suppress_missing_evidence_claim", 44, 24, 82, -18, 0x20),
        "causal" => ("suppress_causal_shortcut", 38, 30, 78, -14, 0x40),
        "answer" => ("suppress_answer_permission", 72, 18, 86, -28, 0x80),
        _ => ("suppress_unstable_lens", 24, 20, 70, -8, 0x10),
    };

    AntiLane {
        lens: lens.lens,
        action,
        reason: lens.reason,
        record: AntiLaneRecord32 {
            route_id: lens.record.peak_route_id,
            lens_id: lens.record.lens_id,
            lane_id,
            suppress_score: suppress,
            support_preserved: preserved,
            locality_score: locality,
            residual_delta,
            phase,
            flags: lane_flags(lens.state),
            reserved: 0,
            reserved2: 0,
            reserved3: 0,
        },
    }
}

fn has_lane(lanes: &[AntiLane], lens_name: &str) -> bool {
    lanes.iter().any(|lane| lane.lens == lens_name)
}

fn lane_flags(state: &str) -> u16 {
    match state {
        "WATCH" => 0b01,
        "BLOCK" => 0b11,
        _ => 0,
    }
}

fn rate(passed: bool) -> f32 {
    if passed {
        1.0
    } else {
        0.0
    }
}
