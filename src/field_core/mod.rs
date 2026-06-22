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
        serde_json::to_value(self).unwrap_or_else(|_| {
            json!({
                "version": FIELD_CORE_VERSION,
                "family": "unknown",
                "verdict": "WATCH",
                "reason": "failed to serialize unified field report"
            })
        })
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
