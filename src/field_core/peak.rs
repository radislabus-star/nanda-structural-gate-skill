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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldPeakInput {
    pub has_peak: bool,
    pub top_peak: String,
    pub lexical_baseline_top: String,
    pub top_polarization: String,
    pub margin: f64,
    pub top_component_score: f64,
    pub second_component_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldPeakResult {
    pub state: String,
    pub safe_to_answer: bool,
    pub top_peak: String,
    pub lexical_baseline_top: String,
    pub wins_over_lexical_baseline: bool,
    pub top_polarization: String,
    pub margin: f64,
    pub top_component_score: f64,
    pub second_component_score: f64,
    pub component_gap: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub(crate) struct FieldCandidatePeak {
    pub target: String,
    pub vector: FieldVector1024,
    pub support_count: usize,
    pub anti_support_count: usize,
}

pub(crate) fn evaluate_structural_peak(input: FieldPeakInput) -> FieldPeakResult {
    if !input.has_peak {
        return FieldPeakResult {
            state: "NO_PEAK".to_string(),
            safe_to_answer: false,
            top_peak: String::new(),
            lexical_baseline_top: input.lexical_baseline_top,
            wins_over_lexical_baseline: false,
            top_polarization: input.top_polarization,
            margin: round4(input.margin),
            top_component_score: round4(input.top_component_score),
            second_component_score: round4(input.second_component_score),
            component_gap: round4(input.top_component_score - input.second_component_score),
            reason: "No route/group peak was produced.".to_string(),
        };
    }
    let component_gap = round4(input.top_component_score - input.second_component_score);
    let wins_lexical =
        !input.lexical_baseline_top.is_empty() && input.top_peak != input.lexical_baseline_top;
    let (state, safe_to_answer, reason) = if input.top_polarization == "REVERSED" {
        (
            "POLARITY_REVERSED",
            false,
            "Top peak has reversed role-direction polarity relative to the query.",
        )
    } else if input.margin >= 0.055 && component_gap >= 0.12 {
        (
            "FOCUSED",
            true,
            "Top peak has enough margin and stronger connected component than the nearest rival.",
        )
    } else if input.margin < 0.04 {
        (
            "WATCH",
            false,
            "Top peak is close to the nearest rival; use as retrieval hint, not final structure.",
        )
    } else if component_gap < 0.0 {
        (
            "AMBIGUOUS",
            false,
            "Nearest rival has stronger connected component; inspect support and anti-triads.",
        )
    } else {
        (
            "WATCH",
            false,
            "Top peak is plausible but not focused enough for a confident structural answer.",
        )
    };
    FieldPeakResult {
        state: state.to_string(),
        safe_to_answer,
        top_peak: input.top_peak,
        lexical_baseline_top: input.lexical_baseline_top,
        wins_over_lexical_baseline: wins_lexical,
        top_polarization: input.top_polarization,
        margin: round4(input.margin),
        top_component_score: round4(input.top_component_score),
        second_component_score: round4(input.second_component_score),
        component_gap,
        reason: reason.to_string(),
    }
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
