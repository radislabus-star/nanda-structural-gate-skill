use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldReadoutInput {
    pub has_peak: bool,
    pub top_peak: String,
    pub second_peak: String,
    pub lexical_baseline_top: String,
    pub top_polarization: String,
    pub margin: f64,
    pub top_component_score: f64,
    pub second_component_score: f64,
    pub top_center: Value,
    pub second_center: Option<Value>,
    pub nearest_foreign_pull: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldReadoutResult {
    pub state: String,
    pub top_peak: String,
    pub second_peak: String,
    pub margin: f64,
    pub component_gap: f64,
    pub lexical_baseline_top: String,
    pub lexical_trap_detected: bool,
    pub top_polarization: String,
    pub centroid_drift: Value,
    pub nearest_foreign_pull: Value,
    pub read_as: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldLocalPathInput {
    pub peak: String,
    pub support: Vec<Value>,
    pub query_terms: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldLocalPathResult {
    pub enabled: bool,
    pub state: String,
    pub coarse_peak: String,
    pub local_memory_size: usize,
    pub local_path: Vec<Value>,
    pub read_as: String,
}

pub(crate) fn structural_field_readout(input: FieldReadoutInput) -> FieldReadoutResult {
    if !input.has_peak {
        return FieldReadoutResult {
            state: "NO_FIELD".to_string(),
            top_peak: String::new(),
            second_peak: String::new(),
            margin: round4(input.margin),
            component_gap: 0.0,
            lexical_baseline_top: input.lexical_baseline_top,
            lexical_trap_detected: false,
            top_polarization: input.top_polarization,
            centroid_drift: json!({}),
            nearest_foreign_pull: Value::Null,
            read_as: "No resonance field was produced.".to_string(),
        };
    }

    let component_gap = round4(input.top_component_score - input.second_component_score);
    let stability = if input.top_polarization == "REVERSED" {
        "polarity_reversed"
    } else if input.margin >= 0.055 && component_gap >= 0.12 {
        "stable"
    } else if input.margin < 0.04 {
        "contested"
    } else {
        "thin"
    };
    let lexical_trap =
        !input.lexical_baseline_top.is_empty() && input.lexical_baseline_top != input.top_peak;
    let centroid_drift = json!({
        "from_second_peak": input.second_peak,
        "route": center_pair(input.second_center.as_ref(), &input.top_center, "route"),
        "relation": center_pair(input.second_center.as_ref(), &input.top_center, "relation"),
        "entity": center_pair(input.second_center.as_ref(), &input.top_center, "entity"),
        "subject_role": center_pair(input.second_center.as_ref(), &input.top_center, "subject_role"),
        "object_role": center_pair(input.second_center.as_ref(), &input.top_center, "object_role")
    });
    let read_as = if lexical_trap {
        "The structural field beats the lexical baseline; inspect support and anti-triads before final prose."
    } else if stability == "polarity_reversed" {
        "The top route has reversed role-direction polarity; do not use it as an answer route."
    } else if stability == "stable" {
        "The top route has a stable connected peak."
    } else {
        "The field is useful as retrieval context but is not a final answer skeleton."
    };

    FieldReadoutResult {
        state: stability.to_string(),
        top_peak: input.top_peak,
        second_peak: input.second_peak,
        margin: round4(input.margin),
        component_gap,
        lexical_baseline_top: input.lexical_baseline_top,
        lexical_trap_detected: lexical_trap,
        top_polarization: input.top_polarization,
        centroid_drift,
        nearest_foreign_pull: input.nearest_foreign_pull.unwrap_or(Value::Null),
        read_as: read_as.to_string(),
    }
}

pub(crate) fn structural_local_path(input: FieldLocalPathInput) -> FieldLocalPathResult {
    let local_path = input
        .support
        .iter()
        .take(5)
        .map(|item| {
            let subject = item["subject"].as_str().unwrap_or("");
            let relation = item["relation"].as_str().unwrap_or("");
            let object = item["object"].as_str().unwrap_or("");
            let haystack = normalize(&format!("{subject} {relation} {object}"));
            let hits = input
                .query_terms
                .iter()
                .filter(|term| haystack.contains(term.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            json!({
                "triad": item["id"].as_str().unwrap_or(""),
                "edge": format!("{subject} -> {relation} -> {object}"),
                "score": item["score"].as_f64().unwrap_or(0.0),
                "polarity": item["polarity"].as_str().unwrap_or(""),
                "query_hits": hits
            })
        })
        .collect::<Vec<_>>();
    FieldLocalPathResult {
        enabled: true,
        state: if local_path.is_empty() {
            "THIN".to_string()
        } else {
            "LOCALIZED".to_string()
        },
        coarse_peak: input.peak,
        local_memory_size: input.support.len(),
        local_path,
        read_as: "Coarse route first, then inspect the local supporting path inside that route."
            .to_string(),
    }
}

pub(crate) fn readout_value(result: FieldReadoutResult) -> Value {
    serde_json::to_value(result).unwrap_or_else(|_| {
        json!({
            "state": "FIELD_READOUT_FAILED",
            "read_as": "field_core readout serialization failed"
        })
    })
}

pub(crate) fn local_path_value(result: FieldLocalPathResult) -> Value {
    serde_json::to_value(result).unwrap_or_else(|_| {
        json!({
            "enabled": true,
            "state": "LOCAL_PATH_FAILED",
            "coarse_peak": "",
            "local_path": []
        })
    })
}

fn center_pair(second_center: Option<&Value>, top_center: &Value, key: &str) -> Value {
    json!({
        "from": second_center
            .and_then(|center| center[key].as_str())
            .unwrap_or(""),
        "to": top_center[key].as_str().unwrap_or(""),
        "changed": second_center
            .and_then(|center| center[key].as_str())
            .unwrap_or("") != top_center[key].as_str().unwrap_or("")
    })
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}
