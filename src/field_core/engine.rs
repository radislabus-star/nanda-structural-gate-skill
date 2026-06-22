use serde::Serialize;
use serde_json::{json, Value};

use crate::FieldEngineMode;

#[derive(Debug, Clone, Serialize)]
struct FieldEngineDecision {
    version: &'static str,
    mode: &'static str,
    field_participates: bool,
    selected_engine: &'static str,
    selected: Value,
    legacy: Value,
    field_candidate: Value,
    candidate_allowed: bool,
    cutover_applied: bool,
    top_level_behavior_changed: bool,
    field_core_as_sole_engine: bool,
}

pub(crate) fn structural_field_engine_decision(search: &Value, mode: &FieldEngineMode) -> Value {
    let legacy = engine_snapshot(
        "structural-domain",
        search["top_peak"].as_str().unwrap_or(""),
        search["verdict"].as_str().unwrap_or("WATCH"),
        search["field_state"].as_str().unwrap_or("FIELD_UNKNOWN"),
        search["safe_to_answer"].as_bool().unwrap_or(false),
    );
    let runtime = &search["field_runtime"];
    let field_candidate = field_candidate_snapshot(runtime);
    let candidate_requested = matches!(mode, FieldEngineMode::Candidate | FieldEngineMode::Cutover);
    let candidate_allowed = candidate_requested
        && runtime["cutover_ready"].as_bool().unwrap_or(false)
        && runtime["field_not_more_permissive"]
            .as_bool()
            .unwrap_or(false);
    let cutover_applied = matches!(mode, FieldEngineMode::Cutover) && candidate_allowed;
    let selected_engine = if cutover_applied {
        "field-core-cutover"
    } else if candidate_allowed {
        "field-core-candidate"
    } else {
        "structural-domain"
    };
    let selected = if candidate_allowed {
        field_candidate.clone()
    } else {
        legacy.clone()
    };
    decision_value(
        FieldEngineDecision {
            version: "structural-field-engine-v1",
            mode: field_engine_mode_label(mode),
            field_participates: !matches!(mode, FieldEngineMode::Legacy),
            selected_engine,
            selected,
            legacy,
            field_candidate,
            candidate_allowed,
            cutover_applied,
            top_level_behavior_changed: cutover_applied,
            field_core_as_sole_engine: cutover_applied,
        },
        json!({
        "field_core_as_structural_sole_engine": cutover_applied,
        "field_core_as_sole_engine_scope": if cutover_applied { "structural-only" } else { "none" },
        "cutover_policy": {
            "requires_cutover_ready": true,
            "requires_not_more_permissive": true,
            "safe_to_answer_policy": "field_may_be_less_permissive_than_legacy",
            "global_sole_engine": false,
            "packed_hot_core_exception": true,
            "cognitive_not_llm": true
        },
        "claim_boundary": {
            "structural_candidate_only": matches!(mode, FieldEngineMode::Candidate),
            "structural_cutover_only": cutover_applied,
            "requires_cutover_suite": true,
            "packed_hot_core_exception": true,
            "llm_ready": false,
            "nonlinear_memory_proven": false
        }
        }),
    )
}

pub(crate) fn apply_structural_field_cutover(search: &mut Value, field_engine: &Value) {
    if !field_engine["cutover_applied"].as_bool().unwrap_or(false) {
        return;
    }
    let selected = &field_engine["selected"];
    let old_top_peak = search["top_peak"].clone();
    let old_verdict = search["verdict"].clone();
    let old_field_state = search["field_state"].clone();
    let old_safe_to_answer = search["safe_to_answer"].clone();
    search["top_peak"] = selected["peak"].clone();
    search["verdict"] = selected["verdict"].clone();
    search["field_state"] = selected["field_state"].clone();
    search["safe_to_answer"] = selected["safe_to_answer"].clone();
    search["field_cutover"] = json!({
        "version": "structural-field-cutover-v1",
        "applied": true,
        "scope": "structural-only",
        "old_top_peak": old_top_peak,
        "new_top_peak": search["top_peak"],
        "old_verdict": old_verdict,
        "new_verdict": search["verdict"],
        "old_field_state": old_field_state,
        "new_field_state": search["field_state"],
        "old_safe_to_answer": old_safe_to_answer,
        "new_safe_to_answer": search["safe_to_answer"],
        "policy": field_engine["cutover_policy"].clone()
    });
}

pub(crate) fn packed_field_engine_decision(pack: &Value, mode: &FieldEngineMode) -> Value {
    let runtime = &pack["field_runtime"];
    let legacy = engine_snapshot(
        "packed-hot-core",
        runtime["old_peak"].as_str().unwrap_or(""),
        runtime["old_verdict"].as_str().unwrap_or("WATCH"),
        runtime["old_field_state"]
            .as_str()
            .unwrap_or("PACKED_UNKNOWN"),
        runtime["old_safe_to_answer"].as_bool().unwrap_or(false),
    );
    let field_candidate = field_candidate_snapshot(runtime);
    let candidate_requested = matches!(mode, FieldEngineMode::Candidate | FieldEngineMode::Cutover);
    let candidate_allowed = candidate_requested
        && runtime["cutover_ready"].as_bool().unwrap_or(false)
        && runtime["field_not_more_permissive"]
            .as_bool()
            .unwrap_or(false);
    let cutover_requested = matches!(mode, FieldEngineMode::Cutover);
    let selected = legacy.clone();
    let legacy_again = engine_snapshot(
        "packed-hot-core",
        runtime["old_peak"].as_str().unwrap_or(""),
        runtime["old_verdict"].as_str().unwrap_or("WATCH"),
        runtime["old_field_state"]
            .as_str()
            .unwrap_or("PACKED_UNKNOWN"),
        runtime["old_safe_to_answer"].as_bool().unwrap_or(false),
    );
    decision_value(
        FieldEngineDecision {
            version: "packed-field-engine-guard-v1",
            mode: field_engine_mode_label(mode),
            field_participates: !matches!(mode, FieldEngineMode::Legacy),
            selected_engine: "packed-hot-core",
            selected,
            legacy: legacy_again,
            field_candidate,
            candidate_allowed,
            cutover_applied: false,
            top_level_behavior_changed: false,
            field_core_as_sole_engine: false,
        },
        json!({
        "cutover_requested": cutover_requested,
        "cutover_applied": false,
        "top_level_behavior_changed": false,
        "field_core_as_sole_engine": false,
        "field_core_as_packed_engine_candidate": candidate_allowed,
        "field_core_as_packed_hot_engine": false,
        "hot_core_guard": {
            "packed_hot_core_exception": true,
            "requires_zero_cost_view": true,
            "requires_hot_bench_guard": true,
            "requires_explicit_follow_up": true,
            "no_json_string_heap_hashmap_inner_loop": true
        },
        "cutover_blocked_reason": if cutover_requested {
            json!(["packed_hot_core_exception", "packed_cutover_requires_explicit_hot_loop_follow_up"])
        } else {
            json!([])
        },
        "claim_boundary": {
            "packed_candidate_only": matches!(mode, FieldEngineMode::Candidate),
            "packed_cutover_requested": cutover_requested,
            "global_sole_engine": false,
            "packed_hot_core_exception": true,
            "llm_ready": false,
            "nonlinear_memory_proven": false
        }
        }),
    )
}

pub(crate) fn cognitive_field_engine_decision(
    report: &Value,
    unified_field: &Value,
    runtime: &Value,
) -> Value {
    let legacy = engine_snapshot(
        "llmwave-big-domain-report",
        runtime["old_peak"].as_str().unwrap_or("report-peak"),
        runtime["old_verdict"].as_str().unwrap_or("WATCH"),
        runtime["old_field_state"]
            .as_str()
            .unwrap_or("COGNITIVE_UNKNOWN"),
        runtime["old_safe_to_answer"].as_bool().unwrap_or(false),
    );
    let field_candidate = field_candidate_snapshot(runtime);
    let candidate_allowed = runtime["cutover_ready"].as_bool().unwrap_or(false)
        && runtime["field_not_more_permissive"]
            .as_bool()
            .unwrap_or(false);
    let claim_boundary = &unified_field["claim_boundary"];
    let not_llm_ready = claim_boundary["not_llm_ready"].as_bool().unwrap_or(true)
        || !report["claim_boundary"]["chat_ready"]
            .as_bool()
            .unwrap_or(false);
    let not_nonlinear_memory_proof = claim_boundary["not_nonlinear_memory_proof"]
        .as_bool()
        .unwrap_or(true)
        || !report["claim_boundary"]["nonlinear_memory_proven"]
            .as_bool()
            .unwrap_or(false);
    let blockers = [
        (not_llm_ready, "claim_boundary_not_llm_ready"),
        (not_nonlinear_memory_proof, "nonlinear_memory_not_proven"),
        (
            !report["claim_boundary"]["full_field_mature"]
                .as_bool()
                .unwrap_or(false),
            "full_field_mature_not_proven",
        ),
    ]
    .into_iter()
    .filter_map(|(blocked, reason)| blocked.then_some(reason))
    .collect::<Vec<_>>();
    let selected = legacy.clone();
    let legacy_again = engine_snapshot(
        "llmwave-big-domain-report",
        runtime["old_peak"].as_str().unwrap_or("report-peak"),
        runtime["old_verdict"].as_str().unwrap_or("WATCH"),
        runtime["old_field_state"]
            .as_str()
            .unwrap_or("COGNITIVE_UNKNOWN"),
        runtime["old_safe_to_answer"].as_bool().unwrap_or(false),
    );
    decision_value(
        FieldEngineDecision {
            version: "cognitive-field-engine-guard-v1",
            mode: "cognitive-guard",
            field_participates: true,
            selected_engine: "llmwave-big-domain-report",
            selected,
            legacy: legacy_again,
            field_candidate,
            candidate_allowed,
            cutover_applied: false,
            top_level_behavior_changed: false,
            field_core_as_sole_engine: false,
        },
        json!({
        "field_core_as_semantic_engine": true,
        "field_core_as_sole_engine": false,
        "field_core_as_chat_engine": false,
        "field_core_as_llm": false,
        "guard": {
            "not_llm_ready": not_llm_ready,
            "not_nonlinear_memory_proof": not_nonlinear_memory_proof,
            "requires_big_cognition_eval": true,
            "requires_external_corpus_eval": true,
            "requires_chat_safety_eval": true
        },
        "cutover_blocked_reason": blockers,
        "claim_boundary": {
            "chat_ready": false,
            "llm_ready": false,
            "nonlinear_memory_proven": false,
            "global_sole_engine": false
        }
        }),
    )
}

fn decision_value(decision: FieldEngineDecision, extensions: Value) -> Value {
    let mut value = serde_json::to_value(decision).unwrap_or_else(|_| {
        json!({
            "version": "field-engine-decision-serialization-failed",
            "field_core_as_sole_engine": false
        })
    });
    if let (Some(base), Some(extra)) = (value.as_object_mut(), extensions.as_object()) {
        for (key, item) in extra {
            base.insert(key.clone(), item.clone());
        }
    }
    value
}

fn field_engine_mode_label(mode: &FieldEngineMode) -> &'static str {
    match mode {
        FieldEngineMode::Legacy => "legacy",
        FieldEngineMode::Shadow => "shadow",
        FieldEngineMode::Candidate => "candidate",
        FieldEngineMode::Cutover => "cutover",
    }
}

fn engine_snapshot(
    engine: &'static str,
    peak: &str,
    verdict: &str,
    field_state: &str,
    safe_to_answer: bool,
) -> Value {
    json!({
        "engine": engine,
        "peak": peak,
        "verdict": verdict,
        "field_state": field_state,
        "safe_to_answer": safe_to_answer
    })
}

fn field_candidate_snapshot(runtime: &Value) -> Value {
    json!({
        "engine": "field-core",
        "peak": runtime["field_peak"].as_str().unwrap_or(""),
        "verdict": runtime["field_verdict"].as_str().unwrap_or("WATCH"),
        "field_state": runtime["field_state"].as_str().unwrap_or("FIELD_UNKNOWN"),
        "safe_to_answer": runtime["field_safe_to_answer"].as_bool().unwrap_or(false),
        "cutover_ready": runtime["cutover_ready"].as_bool().unwrap_or(false),
        "peak_matches": runtime["peak_matches"].as_bool().unwrap_or(false),
        "state_family_matches": runtime["state_family_matches"].as_bool().unwrap_or(false),
        "field_not_more_permissive": runtime["field_not_more_permissive"].as_bool().unwrap_or(false),
        "mismatch_reason": runtime["mismatch_reason"].clone()
    })
}
