use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{adapters, FieldFamily, FieldPassReport, UnifiedFieldReport, FIELD_PASS_VERSION};

pub(crate) const FIELD_RUNTIME_VERSION: &str = "unified-field-runtime-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldRuntimeContract {
    pub version: &'static str,
    pub family: FieldFamily,
    pub input_contract: &'static str,
    pub output_contract: &'static str,
    pub runtime_role: &'static str,
    pub field_core_as_sole_engine: bool,
    pub domain_engine_preserved: bool,
    pub claim_boundary_preserved: bool,
    pub hot_loop_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldRuntimeDualRun {
    pub version: &'static str,
    pub family: FieldFamily,
    pub mode: &'static str,
    pub old_peak: String,
    pub field_peak: String,
    pub old_verdict: String,
    pub field_verdict: String,
    pub old_field_state: String,
    pub field_state: String,
    pub old_safe_to_answer: bool,
    pub field_safe_to_answer: bool,
    pub peak_matches: bool,
    pub state_family_matches: bool,
    pub field_not_more_permissive: bool,
    pub cutover_ready: bool,
    pub mismatch_reason: Vec<String>,
}

pub(crate) fn runtime_contract_from_report(
    report: &UnifiedFieldReport,
    pass: &FieldPassReport,
) -> FieldRuntimeContract {
    FieldRuntimeContract {
        version: FIELD_RUNTIME_VERSION,
        family: report.family.clone(),
        input_contract: "FieldPassInput",
        output_contract: "FieldPassReport",
        runtime_role: "shared-semantic-pass",
        field_core_as_sole_engine: false,
        domain_engine_preserved: true,
        claim_boundary_preserved: pass.claim_boundary.not_llm_ready
            && pass.claim_boundary.not_nonlinear_memory_proof,
        hot_loop_safe: match report.family {
            FieldFamily::Packed => report.compatibility.hot_loop_unchanged,
            _ => true,
        },
    }
}

pub(crate) fn structural_dual_run_from_search(search: &Value) -> FieldRuntimeDualRun {
    let report = adapters::adapt_value(search);
    let unified = report.to_value();
    let field_pass = &unified["field_pass"];
    let old_peak = search["top_peak"]
        .as_str()
        .or_else(|| {
            search["peaks"]
                .as_array()
                .and_then(|peaks| peaks.first())
                .and_then(|peak| peak["peak"].as_str())
        })
        .unwrap_or("")
        .to_string();
    let field_peak = field_pass["peak"]["target"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let old_verdict = search["verdict"].as_str().unwrap_or("WATCH").to_string();
    let field_verdict = field_pass["verdict"]
        .as_str()
        .unwrap_or("WATCH")
        .to_string();
    let old_field_state = search["field_state"]
        .as_str()
        .or_else(|| search["field_state_machine"]["state"].as_str())
        .or_else(|| search["peak_decision"]["state"].as_str())
        .unwrap_or("FIELD_UNKNOWN")
        .to_string();
    let field_state = field_pass["coherence_state"]
        .as_str()
        .or_else(|| field_pass["peak"]["state"].as_str())
        .unwrap_or("FIELD_UNKNOWN")
        .to_string();
    let old_safe_to_answer = search["safe_to_answer"]
        .as_bool()
        .or_else(|| search["field_state_machine"]["safe_to_answer"].as_bool())
        .or_else(|| search["peak_decision"]["safe_to_answer"].as_bool())
        .unwrap_or(false);
    let field_safe_to_answer = field_pass["safe_to_answer"].as_bool().unwrap_or(false);
    let peak_matches = !old_peak.is_empty() && old_peak == field_peak;
    let state_family_matches = state_family(&old_field_state) == state_family(&field_state);
    let field_not_more_permissive = !field_safe_to_answer || old_safe_to_answer;
    let mut mismatch_reason = vec![];
    if !peak_matches {
        mismatch_reason.push("peak_mismatch".to_string());
    }
    if !state_family_matches {
        mismatch_reason.push("state_family_mismatch".to_string());
    }
    if !field_not_more_permissive {
        mismatch_reason.push("field_more_permissive_than_domain_engine".to_string());
    }
    if field_pass["version"].as_str() != Some(FIELD_PASS_VERSION) {
        mismatch_reason.push("field_pass_version_mismatch".to_string());
    }
    let cutover_ready = peak_matches
        && state_family_matches
        && field_not_more_permissive
        && mismatch_reason.is_empty();

    FieldRuntimeDualRun {
        version: FIELD_RUNTIME_VERSION,
        family: FieldFamily::Structural,
        mode: "structural-dual-run",
        old_peak,
        field_peak,
        old_verdict,
        field_verdict,
        old_field_state,
        field_state,
        old_safe_to_answer,
        field_safe_to_answer,
        peak_matches,
        state_family_matches,
        field_not_more_permissive,
        cutover_ready,
        mismatch_reason,
    }
}

pub(crate) fn structural_dual_run_value(search: &Value) -> Value {
    serde_json::to_value(structural_dual_run_from_search(search)).unwrap_or_else(|_| {
        json!({
            "version": FIELD_RUNTIME_VERSION,
            "family": "structural",
            "mode": "structural-dual-run",
            "cutover_ready": false,
            "mismatch_reason": ["serialization_failed"]
        })
    })
}

pub(crate) fn packed_dual_run_from_pack(pack: &Value) -> FieldRuntimeDualRun {
    let report = adapters::adapt_value(pack);
    let unified = report.to_value();
    let field_pass = &unified["field_pass"];
    let old_peak = pack["top_peak"]
        .as_str()
        .map(str::to_string)
        .or_else(|| {
            pack["peak_decision"]["route"]["top_id"]
                .as_u64()
                .map(|id| format!("route:{id}"))
        })
        .unwrap_or_default();
    let field_peak = field_pass["peak"]["target"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let old_verdict = pack["peak_decision"]["verdict"]
        .as_str()
        .or_else(|| pack["verdict"].as_str())
        .unwrap_or("WATCH")
        .to_string();
    let field_verdict = field_pass["verdict"]
        .as_str()
        .unwrap_or("WATCH")
        .to_string();
    let old_field_state = pack["peak_decision"]["state"]
        .as_str()
        .unwrap_or("PACKED_UNKNOWN")
        .to_string();
    let field_state = field_pass["coherence_state"]
        .as_str()
        .or_else(|| field_pass["peak"]["state"].as_str())
        .unwrap_or("FIELD_UNKNOWN")
        .to_string();
    let old_safe_to_answer = pack["peak_decision"]["safe_to_answer"]
        .as_bool()
        .unwrap_or(false);
    let field_safe_to_answer = field_pass["safe_to_answer"].as_bool().unwrap_or(false);
    let peak_matches = !old_peak.is_empty() && old_peak == field_peak;
    let state_family_matches = state_family(&old_field_state) == state_family(&field_state);
    let field_not_more_permissive = !field_safe_to_answer || old_safe_to_answer;
    let mut mismatch_reason = vec![];
    if !peak_matches {
        mismatch_reason.push("peak_mismatch".to_string());
    }
    if !state_family_matches {
        mismatch_reason.push("state_family_mismatch".to_string());
    }
    if !field_not_more_permissive {
        mismatch_reason.push("field_more_permissive_than_packed_engine".to_string());
    }
    if field_pass["version"].as_str() != Some(FIELD_PASS_VERSION) {
        mismatch_reason.push("field_pass_version_mismatch".to_string());
    }
    let cutover_ready = peak_matches
        && state_family_matches
        && field_not_more_permissive
        && mismatch_reason.is_empty();

    FieldRuntimeDualRun {
        version: FIELD_RUNTIME_VERSION,
        family: FieldFamily::Packed,
        mode: "packed-dual-run",
        old_peak,
        field_peak,
        old_verdict,
        field_verdict,
        old_field_state,
        field_state,
        old_safe_to_answer,
        field_safe_to_answer,
        peak_matches,
        state_family_matches,
        field_not_more_permissive,
        cutover_ready,
        mismatch_reason,
    }
}

pub(crate) fn packed_dual_run_value(pack: &Value) -> Value {
    serde_json::to_value(packed_dual_run_from_pack(pack)).unwrap_or_else(|_| {
        json!({
            "version": FIELD_RUNTIME_VERSION,
            "family": "packed",
            "mode": "packed-dual-run",
            "cutover_ready": false,
            "mismatch_reason": ["serialization_failed"]
        })
    })
}

fn state_family(state: &str) -> &'static str {
    match state {
        "PASS" | "FOCUSED" | "FIELD_FOCUSED" | "FIELD_SAFE" | "PACKED_FOCUSED" => "focused",
        "VETO" | "FIELD_REVERSED" | "POLARITY_REVERSED" => "veto",
        "FIELD_CONTESTED" => "contested",
        "FIELD_THIN" | "PACKED_THIN" => "thin",
        _ => "watch",
    }
}
