use serde::{Deserialize, Serialize};

use super::FieldVector1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldPeakSummary {
    pub target: String,
    pub score: f64,
    pub margin: f64,
    pub state: String,
    pub safe_to_answer: bool,
    pub support_count: usize,
    pub anti_support_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct FieldCandidatePeak {
    pub target: String,
    pub vector: FieldVector1024,
    pub support_count: usize,
    pub anti_support_count: usize,
}

pub(crate) fn detect_field_peak(
    query: &FieldVector1024,
    candidates: &[FieldCandidatePeak],
) -> FieldPeakSummary {
    let mut scored = candidates
        .iter()
        .map(|candidate| {
            (
                candidate,
                query.cosine(&candidate.vector),
                candidate.support_count,
                candidate.anti_support_count,
            )
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let Some((top, score, support_count, anti_support_count)) = scored.first() else {
        return FieldPeakSummary::default();
    };
    let runner_up = scored.get(1).map(|(_, score, _, _)| *score).unwrap_or(0.0);
    let margin = score - runner_up;
    let state = if *score >= 0.16 && margin >= 0.04 && *anti_support_count == 0 {
        "FIELD_FOCUSED"
    } else if *score >= 0.08 && margin >= 0.02 {
        "FIELD_WATCH"
    } else {
        "FIELD_DIFFUSE"
    };

    FieldPeakSummary {
        target: top.target.clone(),
        score: round4(*score),
        margin: round4(margin),
        state: state.to_string(),
        safe_to_answer: state == "FIELD_FOCUSED",
        support_count: *support_count,
        anti_support_count: *anti_support_count,
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

impl Default for FieldPeakSummary {
    fn default() -> Self {
        Self {
            target: String::new(),
            score: 0.0,
            margin: 0.0,
            state: "WATCH".to_string(),
            safe_to_answer: false,
            support_count: 0,
            anti_support_count: 0,
        }
    }
}
