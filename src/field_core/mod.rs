use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub(crate) mod adapters;
pub(crate) mod anti_wave;
pub(crate) mod basis;
pub(crate) mod coherence;
pub(crate) mod feedback;
pub(crate) mod lens;
pub(crate) mod pass;
pub(crate) mod peak;
pub(crate) mod record;
pub(crate) mod vector;

pub(crate) use anti_wave::*;
pub(crate) use basis::*;
pub(crate) use coherence::*;
pub(crate) use feedback::*;
pub(crate) use lens::*;
pub(crate) use pass::*;
pub(crate) use peak::*;
pub(crate) use record::*;
pub(crate) use vector::*;

pub(crate) const FIELD_CORE_VERSION: &str = "unified-field-v1-readonly";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FieldFamily {
    Structural,
    Packed,
    Cognitive,
    Unknown,
}

impl FieldFamily {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Structural => "structural",
            Self::Packed => "packed",
            Self::Cognitive => "cognitive",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UnifiedFieldReport {
    pub version: &'static str,
    pub family: FieldFamily,
    pub source_mode: String,
    pub basis: FieldBasis,
    pub record: FieldRecordSummary,
    pub query: FieldQuerySummary,
    pub peak: FieldPeakSummary,
    pub lens: FieldLensSummary,
    pub anti_wave: AntiWaveSummary,
    pub coherence: FieldCoherenceSummary,
    pub feedback: FeedbackSummary,
    pub compatibility: FieldCompatibility,
    pub claim_boundary: FieldClaimBoundary,
}

impl UnifiedFieldReport {
    pub(crate) fn to_value(&self) -> Value {
        let mut value = serde_json::to_value(self).unwrap_or_else(|_| {
            json!({
                "version": FIELD_CORE_VERSION,
                "family": "unknown",
                "verdict": "WATCH",
                "reason": "failed to serialize unified field report"
            })
        });
        if let Some(object) = value.as_object_mut() {
            let memory_delta = summarize_memory_deltas(&self.feedback, &self.peak.target);
            object.insert(
                "compute_probe".to_string(),
                serde_json::to_value(FieldComputeProbe::from_report(self))
                    .unwrap_or_else(|_| json!({"state": "FIELD_PROBE_FAILED"})),
            );
            object.insert(
                "memory_delta".to_string(),
                serde_json::to_value(&memory_delta)
                    .unwrap_or_else(|_| json!({"state": "FIELD_MEMORY_DELTA_FAILED"})),
            );
            object.insert(
                "field_pass".to_string(),
                serde_json::to_value(FieldPassProjection::from_report(self, &memory_delta.deltas))
                    .unwrap_or_else(|_| json!({"state": "FIELD_PASS_FAILED"})),
            );
        }
        value
    }
}

struct FieldPassProjection;

impl FieldPassProjection {
    fn from_report(report: &UnifiedFieldReport, deltas: &[FieldMemoryDelta]) -> FieldPassReport {
        let query = FieldRecord::synthetic(
            "report-query",
            match report.family {
                FieldFamily::Structural => FieldRecordKind::StructuralTriad,
                FieldFamily::Packed => FieldRecordKind::PackedRecord,
                FieldFamily::Cognitive => FieldRecordKind::L3Schema,
                FieldFamily::Unknown => FieldRecordKind::StructuralTriad,
            },
            "unified_field",
            "queries",
            report
                .query
                .text
                .as_deref()
                .unwrap_or(report.family.as_str()),
            Some(report.peak.target.clone()),
            Some(report.source_mode.clone()),
        );
        let records = vec![FieldRecord::synthetic(
            "report-peak",
            match report.family {
                FieldFamily::Structural => FieldRecordKind::StructuralTriad,
                FieldFamily::Packed => FieldRecordKind::PackedRecord,
                FieldFamily::Cognitive => FieldRecordKind::L3Schema,
                FieldFamily::Unknown => FieldRecordKind::StructuralTriad,
            },
            report.family.as_str(),
            "has_peak",
            report.peak.target.clone(),
            Some(report.peak.target.clone()),
            Some(report.source_mode.clone()),
        )];
        let anti_waves = if report.anti_wave.lanes > 0 || report.coherence.foreign_pull > 0 {
            vec![FieldAntiWaveLane {
                id: "report-anti-wave".to_string(),
                target: report
                    .anti_wave
                    .suppressed_target
                    .clone()
                    .unwrap_or_else(|| "report-noise".to_string()),
                subject: "unified_field".to_string(),
                relation: "suppresses".to_string(),
                object: report.peak.target.clone(),
                route: Some(report.peak.target.clone()),
                group: Some(report.source_mode.clone()),
                strength: report.anti_wave.lanes.max(1) as i32,
            }]
        } else {
            vec![]
        };

        let input = FieldPassInput {
            family: report.family.clone(),
            query,
            records,
            lenses: vec![FieldLensOperation {
                kind: FieldLensKind::Route,
                label: report.peak.target.clone(),
                strength: 1,
            }],
            anti_waves,
            claim_boundary: report.claim_boundary.clone(),
        };
        let input = apply_memory_deltas_to_pass(&input, deltas);
        run_field_pass(&input)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldComputeProbe {
    pub version: &'static str,
    pub dim: usize,
    pub seed: String,
    pub query_signature: String,
    pub lens_signature: String,
    pub peak_target: String,
    pub peak_state: String,
    pub anti_wave_delta: f64,
    pub coherence_state: String,
}

impl FieldComputeProbe {
    pub(crate) fn from_report(report: &UnifiedFieldReport) -> Self {
        let seed = format!("{}:{}", report.family.as_str(), report.source_mode);
        let query = FieldVector1024::project_record(&FieldTriadProjection {
            subject: "field_report".to_string(),
            relation: "probes".to_string(),
            object: report.family.as_str().to_string(),
            route: Some(report.peak.target.clone()),
            group: Some(report.source_mode.clone()),
        });
        let mut seeded = FieldVector1024::zero();
        seeded.bundle(&query);
        let seeded_stats = seeded.stats();
        let focused = apply_lens_chain(
            &seeded,
            &[
                FieldLensOperation {
                    kind: FieldLensKind::Route,
                    label: report.peak.target.clone(),
                    strength: 1,
                },
                FieldLensOperation {
                    kind: FieldLensKind::Group,
                    label: report.source_mode.clone(),
                    strength: 1,
                },
                FieldLensOperation {
                    kind: FieldLensKind::Role,
                    label: report.family.as_str().to_string(),
                    strength: 1,
                },
                FieldLensOperation {
                    kind: FieldLensKind::Polarity,
                    label: report.peak.state.clone(),
                    strength: 1,
                },
                FieldLensOperation {
                    kind: FieldLensKind::Evidence,
                    label: report.record.evidence_refs.to_string(),
                    strength: 1,
                },
            ],
        );
        let false_shortcut =
            FieldVector1024::from_label(&format!("anti:{}", report.anti_wave.lanes));
        let mut contested = focused.clone();
        contested.bundle_scaled(&false_shortcut, 2);
        let (reduced, anti) = apply_anti_wave(&contested, &false_shortcut, 2);
        let peak = detect_field_peak(
            &reduced,
            &[
                FieldCandidatePeak {
                    target: report.peak.target.clone(),
                    vector: focused.clone(),
                    support_count: report.peak.support_count,
                    anti_support_count: report.peak.anti_support_count,
                },
                FieldCandidatePeak {
                    target: "field_probe_noise".to_string(),
                    vector: false_shortcut,
                    support_count: 0,
                    anti_support_count: report.anti_wave.lanes,
                },
            ],
        );
        let coherence = summarize_field_coherence(
            &reduced,
            Some(&focused),
            &peak,
            report.coherence.foreign_pull,
        );

        Self {
            version: FIELD_COMPUTE_VERSION,
            dim: seeded_stats.dim,
            seed,
            query_signature: seeded.signature_hex(),
            lens_signature: focused.signature_hex(),
            peak_target: peak.target,
            peak_state: peak.state,
            anti_wave_delta: anti.delta,
            coherence_state: coherence.field_state,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldCompatibility {
    pub source_output_preserved: bool,
    pub adapter_only: bool,
    pub hot_loop_unchanged: bool,
    pub old_fields_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldClaimBoundary {
    pub read_only_projection: bool,
    pub no_behavior_change: bool,
    pub not_llm_ready: bool,
    pub not_nonlinear_memory_proof: bool,
    pub not_cache_only_proof: bool,
}

impl Default for FieldCompatibility {
    fn default() -> Self {
        Self {
            source_output_preserved: true,
            adapter_only: true,
            hot_loop_unchanged: true,
            old_fields_required: vec![],
        }
    }
}

impl Default for FieldClaimBoundary {
    fn default() -> Self {
        Self {
            read_only_projection: true,
            no_behavior_change: true,
            not_llm_ready: true,
            not_nonlinear_memory_proof: true,
            not_cache_only_proof: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_projection_is_deterministic() {
        let left = FieldVector1024::from_label("route:ime-display-flow");
        let right = FieldVector1024::from_label("route:ime-display-flow");
        let mut bundled = FieldVector1024::zero();
        bundled.bundle(&left);

        assert_eq!(FIELD_COMPUTE_VERSION, "unified-field-compute-v1");
        assert_eq!(left.signature_hex(), right.signature_hex());
        assert_eq!(bundled.signature_hex(), left.signature_hex());
        assert_eq!(left.stats().dim, FIELD_DIM);
        assert_eq!(left.stats().non_zero, FIELD_DIM);
        assert!((left.cosine(&right) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn triad_projection_keeps_route_coordinates() {
        let plain = FieldVector1024::project_triad("ime", "displays", "candidate");
        let routed = FieldVector1024::project_record(&FieldTriadProjection {
            subject: "ime".to_string(),
            relation: "displays".to_string(),
            object: "candidate".to_string(),
            route: Some("ime-display-flow".to_string()),
            group: Some("runtime".to_string()),
        });

        assert!(plain.cosine(&routed).abs() < 0.12);
        assert_ne!(plain.signature_hex(), routed.signature_hex());
    }

    #[test]
    fn peak_detector_prefers_matching_route() {
        let query = FieldVector1024::project_record(&FieldTriadProjection {
            subject: "ime".to_string(),
            relation: "displays".to_string(),
            object: "candidate".to_string(),
            route: Some("ime-display-flow".to_string()),
            group: Some("runtime".to_string()),
        });
        let matching = query.clone();
        let foreign = FieldVector1024::project_record(&FieldTriadProjection {
            subject: "nanda".to_string(),
            relation: "scores".to_string(),
            object: "candidate".to_string(),
            route: Some("nanda-field-flow".to_string()),
            group: Some("core".to_string()),
        });

        let peak = detect_field_peak(
            &query,
            &[
                FieldCandidatePeak {
                    target: "nanda-field-flow".to_string(),
                    vector: foreign,
                    support_count: 1,
                    anti_support_count: 0,
                },
                FieldCandidatePeak {
                    target: "ime-display-flow".to_string(),
                    vector: matching.clone(),
                    support_count: 3,
                    anti_support_count: 0,
                },
            ],
        );

        assert_eq!(peak.target, "ime-display-flow");
        assert_eq!(peak.state, "FIELD_FOCUSED");
        assert!(peak.safe_to_answer);

        let coherence = summarize_field_coherence(&query, Some(&matching), &peak, 0);
        assert_eq!(coherence.field_state, "FIELD_FOCUSED");
        assert_eq!(coherence.field_action, "answer_or_apply");
    }

    #[test]
    fn anti_wave_reduces_false_alignment() {
        let useful = FieldVector1024::from_label("useful:route");
        let false_shortcut = FieldVector1024::from_label("false:shortcut");
        let mut field = useful.clone();
        field.bundle_scaled(&false_shortcut, 4);

        let before_useful = field.cosine(&useful);
        let (reduced, application) = apply_anti_wave(&field, &false_shortcut, 4);
        let after_useful = reduced.cosine(&useful);

        assert!(application.suppressed);
        assert!(application.after_alignment < application.before_alignment);
        assert!(after_useful >= before_useful);
    }

    #[test]
    fn lens_chain_changes_signature_without_erasing_field() {
        let field = FieldVector1024::from_label("query:manual-trigger");
        let focused = apply_lens_chain(
            &field,
            &[FieldLensOperation {
                kind: FieldLensKind::Route,
                label: "manual-trigger-flow".to_string(),
                strength: 2,
            }],
        );

        assert_ne!(field.signature_hex(), focused.signature_hex());
        assert!(field.cosine(&focused) > 0.2);
    }

    #[test]
    fn field_pass_focuses_matching_record() {
        let report = run_field_pass(&FieldPassInput {
            family: FieldFamily::Structural,
            query: FieldRecord::synthetic(
                "q1",
                FieldRecordKind::StructuralTriad,
                "ime",
                "displays",
                "candidate",
                Some("ime-display-flow".to_string()),
                Some("runtime".to_string()),
            ),
            records: vec![
                FieldRecord::synthetic(
                    "r1",
                    FieldRecordKind::StructuralTriad,
                    "ime",
                    "displays",
                    "candidate",
                    Some("ime-display-flow".to_string()),
                    Some("runtime".to_string()),
                ),
                FieldRecord::synthetic(
                    "r2",
                    FieldRecordKind::StructuralTriad,
                    "nanda",
                    "scores",
                    "candidate",
                    Some("nanda-field-flow".to_string()),
                    Some("core".to_string()),
                ),
            ],
            lenses: vec![],
            anti_waves: vec![],
            claim_boundary: FieldClaimBoundary {
                not_llm_ready: false,
                not_nonlinear_memory_proof: false,
                ..FieldClaimBoundary::default()
            },
        });

        assert_eq!(report.version, FIELD_PASS_VERSION);
        assert_eq!(report.peak.target, "ime-display-flow");
        assert_eq!(report.verdict, "PASS");
        assert!(report.safe_to_answer);
    }

    #[test]
    fn unified_report_exports_field_pass() {
        let report = UnifiedFieldReport {
            version: FIELD_CORE_VERSION,
            family: FieldFamily::Structural,
            source_mode: "test".to_string(),
            basis: FieldBasis::dynamic_1024(),
            record: FieldRecordSummary::default(),
            query: FieldQuerySummary {
                source: "test".to_string(),
                text: Some("ime displays candidate".to_string()),
                requested_axes: vec!["route".to_string()],
                signature: None,
            },
            peak: FieldPeakSummary {
                target: "ime-display-flow".to_string(),
                score: 1.0,
                margin: 1.0,
                state: "FIELD_FOCUSED".to_string(),
                safe_to_answer: true,
                support_count: 1,
                anti_support_count: 0,
            },
            lens: FieldLensSummary::default(),
            anti_wave: AntiWaveSummary::default(),
            coherence: FieldCoherenceSummary::default(),
            feedback: FeedbackSummary::default(),
            compatibility: FieldCompatibility::default(),
            claim_boundary: FieldClaimBoundary::default(),
        };

        let value = report.to_value();
        assert_eq!(value["field_pass"]["version"], FIELD_PASS_VERSION);
        assert_eq!(value["field_pass"]["family"], "structural");
    }

    #[test]
    fn feedback_exports_memory_delta_into_next_pass() {
        let report = UnifiedFieldReport {
            version: FIELD_CORE_VERSION,
            family: FieldFamily::Structural,
            source_mode: "test".to_string(),
            basis: FieldBasis::dynamic_1024(),
            record: FieldRecordSummary::default(),
            query: FieldQuerySummary {
                source: "test".to_string(),
                text: Some("false shortcut".to_string()),
                requested_axes: vec!["route".to_string()],
                signature: None,
            },
            peak: FieldPeakSummary {
                target: "false-route".to_string(),
                score: 0.2,
                margin: 0.1,
                state: "FIELD_WATCH".to_string(),
                safe_to_answer: false,
                support_count: 1,
                anti_support_count: 0,
            },
            lens: FieldLensSummary::default(),
            anti_wave: AntiWaveSummary::default(),
            coherence: FieldCoherenceSummary::default(),
            feedback: FeedbackSummary {
                feedback_present: true,
                accepted: 0,
                rejected: 1,
                watched: 0,
                replayable: true,
            },
            compatibility: FieldCompatibility::default(),
            claim_boundary: FieldClaimBoundary::default(),
        };

        let value = report.to_value();
        assert_eq!(value["memory_delta"]["rejected"], 1);
        assert_eq!(value["field_pass"]["anti_wave_count"], 1);
        assert_eq!(value["field_pass"]["verdict"], "VETO");
    }
}
