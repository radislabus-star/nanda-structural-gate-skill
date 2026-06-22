use serde::{Deserialize, Serialize};

use super::{FieldPeakSummary, FieldVector1024};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldCoherenceSummary {
    pub coherence: f64,
    pub energy: f64,
    pub foreign_pull: usize,
    pub field_state: String,
    pub field_action: String,
}

impl Default for FieldCoherenceSummary {
    fn default() -> Self {
        Self {
            coherence: 0.0,
            energy: 0.0,
            foreign_pull: 0,
            field_state: "FIELD_UNKNOWN".to_string(),
            field_action: "review".to_string(),
        }
    }
}

pub(crate) fn summarize_field_coherence(
    field: &FieldVector1024,
    reference: Option<&FieldVector1024>,
    peak: &FieldPeakSummary,
    foreign_pull: usize,
) -> FieldCoherenceSummary {
    let coherence = reference
        .map(|reference| field.cosine(reference))
        .unwrap_or(0.0);
    let field_state = if foreign_pull > 0 {
        "FIELD_CONTESTED"
    } else if peak.safe_to_answer {
        "FIELD_FOCUSED"
    } else if peak.state == "FIELD_WATCH" {
        "FIELD_WATCH"
    } else {
        "FIELD_DIFFUSE"
    };
    let field_action = match field_state {
        "FIELD_FOCUSED" => "answer_or_apply",
        "FIELD_WATCH" => "split_or_request_evidence",
        "FIELD_CONTESTED" => "repair_foreign_pull",
        _ => "collect_more_signal",
    };

    FieldCoherenceSummary {
        coherence: round4(coherence),
        energy: round4(field.energy()),
        foreign_pull,
        field_state: field_state.to_string(),
        field_action: field_action.to_string(),
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}
