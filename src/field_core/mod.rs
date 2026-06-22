use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub(crate) mod adapters;
pub(crate) mod anti_wave;
pub(crate) mod basis;
pub(crate) mod coherence;
pub(crate) mod feedback;
pub(crate) mod lens;
pub(crate) mod peak;
pub(crate) mod record;
pub(crate) mod vector;

pub(crate) use anti_wave::*;
pub(crate) use basis::*;
pub(crate) use coherence::*;
pub(crate) use feedback::*;
pub(crate) use lens::*;
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
            object.insert(
                "compute_probe".to_string(),
                serde_json::to_value(FieldComputeProbe::from_report(self))
                    .unwrap_or_else(|_| json!({"state": "FIELD_PROBE_FAILED"})),
            );
        }
        value
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
}
