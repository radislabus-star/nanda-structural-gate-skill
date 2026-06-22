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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldCoherenceInput {
    pub has_peak: bool,
    pub top_peak: String,
    pub margin: f64,
    pub component_gap: f64,
    pub top_polarization: String,
    pub corpus_verdict: String,
    pub corpus_warnings: Vec<String>,
    pub route_balanced: bool,
    pub coarse_to_fine_state: String,
    pub lexical_baseline_top: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldCoherenceResult {
    pub state: String,
    pub safe_to_answer: bool,
    pub action: String,
    pub top_peak: String,
    pub blocking: Vec<String>,
    pub margin: f64,
    pub component_gap: f64,
    pub top_polarization: String,
    pub corpus_verdict: String,
    pub corpus_warnings: Vec<String>,
    pub route_balanced: bool,
    pub coarse_to_fine_state: String,
    pub lexical_baseline_top: String,
    pub lexical_trap_detected: bool,
    pub read_as: String,
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

pub(crate) fn evaluate_structural_coherence(input: FieldCoherenceInput) -> FieldCoherenceResult {
    if !input.has_peak {
        return FieldCoherenceResult {
            state: "NO_FIELD".to_string(),
            safe_to_answer: false,
            action: "NO_ANSWER".to_string(),
            top_peak: String::new(),
            blocking: vec!["no_peak".to_string()],
            margin: round4(input.margin),
            component_gap: round4(input.component_gap),
            top_polarization: input.top_polarization,
            corpus_verdict: input.corpus_verdict,
            corpus_warnings: input.corpus_warnings,
            route_balanced: input.route_balanced,
            coarse_to_fine_state: input.coarse_to_fine_state,
            lexical_baseline_top: input.lexical_baseline_top,
            lexical_trap_detected: false,
            read_as: "No resonance field was produced.".to_string(),
        };
    }

    let noisy_warning_count = input
        .corpus_warnings
        .iter()
        .filter(|kind| {
            matches!(
                kind.as_str(),
                "large_unbalanced_corpus"
                    | "route_imbalance"
                    | "hub_dominance"
                    | "duplicate_current"
            )
        })
        .count();
    let weak_text_query = input
        .corpus_warnings
        .iter()
        .any(|kind| kind == "weak_text_query");
    let corpus_noisy =
        input.corpus_verdict == "WATCH" && (noisy_warning_count > 0 || weak_text_query);
    let focused = input.margin >= 0.055
        && input.component_gap >= 0.12
        && input.coarse_to_fine_state == "LOCALIZED";
    let lexical_trap =
        !input.lexical_baseline_top.is_empty() && input.lexical_baseline_top != input.top_peak;

    let mut blocking: Vec<String> = vec![];
    let (state, safe_to_answer, action, read_as) = if input.top_polarization == "REVERSED" {
        blocking.push("polarity_reversed".to_string());
        (
            "FIELD_REVERSED",
            false,
            "STOP_REPAIR_POLARITY",
            "The top peak is role-direction reversed; do not read it as the answer route.",
        )
    } else if corpus_noisy && !input.route_balanced {
        blocking.extend(input.corpus_warnings.iter().cloned());
        (
            "FIELD_NOISY",
            false,
            "FOCUS_CORPUS",
            "The corpus field is noisy; run dataset-doctor or route-balanced focus before trusting the peak.",
        )
    } else if input.margin < 0.04 {
        blocking.push("low_margin".to_string());
        (
            "FIELD_CONTESTED",
            false,
            "SPLIT_OR_QUERY",
            "The top peaks are too close; use the result as retrieval context and split or sharpen the query.",
        )
    } else if !focused {
        if input.component_gap < 0.12 {
            blocking.push("weak_component_gap".to_string());
        }
        if input.coarse_to_fine_state != "LOCALIZED" {
            blocking.push("not_localized".to_string());
        }
        (
            "FIELD_THIN",
            false,
            "USE_AS_HINT",
            "The peak is plausible but not connected/localized enough to become an answer skeleton.",
        )
    } else if input.route_balanced {
        (
            "FIELD_ROUTE_BALANCED",
            true,
            "ANSWER_WITH_BALANCED_SUPPORT",
            "The peak is focused after route-balanced filtering; answer from support and mention the focused packet.",
        )
    } else if lexical_trap {
        (
            "FIELD_FOCUSED",
            true,
            "ANSWER_WITH_SUPPORT",
            "The structural field beats the lexical baseline and is focused enough to draft from support.",
        )
    } else {
        (
            "FIELD_SAFE",
            true,
            "ANSWER_WITH_SUPPORT",
            "The field is focused, localized, and not blocked by corpus or polarity warnings.",
        )
    };

    blocking.sort();
    blocking.dedup();

    FieldCoherenceResult {
        state: state.to_string(),
        safe_to_answer,
        action: action.to_string(),
        top_peak: input.top_peak,
        blocking,
        margin: round4(input.margin),
        component_gap: round4(input.component_gap),
        top_polarization: input.top_polarization,
        corpus_verdict: input.corpus_verdict,
        corpus_warnings: input.corpus_warnings,
        route_balanced: input.route_balanced,
        coarse_to_fine_state: input.coarse_to_fine_state,
        lexical_baseline_top: input.lexical_baseline_top,
        lexical_trap_detected: lexical_trap,
        read_as: read_as.to_string(),
    }
}

pub(crate) fn field_verdict_for_state(field_state: &str, safe_to_answer: bool) -> &'static str {
    match field_state {
        "FIELD_REVERSED" => "VETO",
        "FIELD_FOCUSED" | "FIELD_SAFE" | "FIELD_ROUTE_BALANCED" if safe_to_answer => "PASS",
        _ => "WATCH",
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
