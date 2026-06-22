use super::*;
use serde_json::Value;

pub(crate) fn adapt_value(input: &Value) -> UnifiedFieldReport {
    if looks_packed(input) {
        return packed_report(input);
    }
    if looks_cognitive(input) {
        return cognitive_report(input);
    }
    if looks_structural(input) {
        return structural_report(input);
    }
    unknown_report(input)
}

fn looks_structural(input: &Value) -> bool {
    input.get("peak_decision").is_some()
        || input.get("field_state_machine").is_some()
        || input.get("structural_map").is_some()
        || input.get("route_field").is_some()
        || input.get("field_interpretation").is_some()
}

fn looks_packed(input: &Value) -> bool {
    input.get("packed_support").is_some()
        || input.get("packed_lanes").is_some()
        || input.get("packed_runtime").is_some()
        || input.get("hot_pack").is_some()
        || input.get("pack6m").is_some()
        || input.get("packed_peak").is_some()
}

fn looks_cognitive(input: &Value) -> bool {
    input.get("roadmap_block").is_some()
        || input.get("claim_boundary").is_some()
        || input.get("l2_word_field").is_some()
        || input.get("l3_schema_field").is_some()
        || input.get("lens").is_some()
        || input.get("field_after_anti").is_some()
        || input.get("schema_peak").is_some()
}

fn structural_report(input: &Value) -> UnifiedFieldReport {
    let peak = input["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .cloned()
        .unwrap_or(Value::Null);
    let field_state = &input["field_state_machine"];
    let peak_decision = &input["peak_decision"];
    let structural_map = &input["structural_map"];
    let source_mode = mode(input, "structural_result");
    let peak_target = peak["peak"]
        .as_str()
        .or_else(|| input["top_peak"].as_str())
        .unwrap_or("")
        .to_string();
    let query_text = input["query"]["text"]
        .as_str()
        .or_else(|| input["query"].as_str())
        .map(str::to_string);
    UnifiedFieldReport {
        version: FIELD_CORE_VERSION,
        family: FieldFamily::Structural,
        source_mode: source_mode.clone(),
        basis: FieldBasis::dynamic_1024(),
        record: FieldRecordSummary {
            records: count_array(input, "triads") + count_array(input, "candidate_triads"),
            routes: object_len(&structural_map["route_field"]["routes"])
                .max(array_len(&input["route_balanced_focus"]["routes"])),
            groups: object_len(&structural_map["group_centroids"]["source"])
                + object_len(&structural_map["group_centroids"]["candidate"]),
            schemas: 0,
            surfaces: 0,
            evidence_refs: count_array(input, "supporting_triads"),
        },
        query: FieldQuerySummary {
            source: query_source(input),
            text: query_text.clone(),
            requested_axes: vec![
                "route".to_string(),
                "group".to_string(),
                "polarity".to_string(),
            ],
            signature: Some(adapter_field_signature(
                FieldFamily::Structural,
                &source_mode,
                &peak_target,
                query_text.as_deref(),
            )),
        },
        peak: FieldPeakSummary {
            target: peak_target,
            score: number(&peak["score"])
                .or_else(|| number(&input["top_score"]))
                .unwrap_or(0.0),
            margin: number(&input["peak_margin"]).unwrap_or(0.0),
            state: peak_decision["state"]
                .as_str()
                .unwrap_or("WATCH")
                .to_string(),
            safe_to_answer: peak_decision["safe_to_answer"].as_bool().unwrap_or(false),
            support_count: array_len(&input["supporting_triads"]),
            anti_support_count: array_len(&input["destructive_interference"]["suppressed"]),
        },
        lens: FieldLensSummary {
            lenses: vec![
                "route".to_string(),
                "group".to_string(),
                "polarity".to_string(),
            ],
            amplified: string_array(&input["constructive_interference"]["reinforced_routes"]),
            suppressed: string_array(&input["destructive_interference"]["suppressed_routes"]),
            explanation: "structural route/group/polarity field projection".to_string(),
        },
        anti_wave: AntiWaveSummary {
            active: input.get("destructive_interference").is_some(),
            lanes: array_len(&input["destructive_interference"]["lanes"])
                .max(array_len(&input["negative_shortcuts"])),
            suppressed_target: input["destructive_interference"]["suppressed_peak"]
                .as_str()
                .map(str::to_string),
            suppression_energy: number(&input["destructive_interference"]["effective_penalty"])
                .unwrap_or(0.0),
            local_only: true,
        },
        coherence: FieldCoherenceSummary {
            coherence: number(&structural_map["structural_energy"]["route_coherence"])
                .or_else(|| number(&input["field_interpretation"]["coherence"]))
                .unwrap_or(0.0),
            energy: number(&input["field_interpretation"]["energy"]).unwrap_or(0.0),
            foreign_pull: array_len(&structural_map["foreign_pull"])
                .max(array_len(&input["foreign_pull"])),
            field_state: field_state["state"]
                .as_str()
                .unwrap_or("FIELD_UNKNOWN")
                .to_string(),
            field_action: field_state["action"]
                .as_str()
                .unwrap_or("review")
                .to_string(),
        },
        feedback: feedback_summary(input),
        compatibility: FieldCompatibility {
            old_fields_required: strings(&["peak_decision", "field_state_machine"]),
            ..FieldCompatibility::default()
        },
        claim_boundary: FieldClaimBoundary::default(),
    }
}

fn packed_report(input: &Value) -> UnifiedFieldReport {
    let packed_support = &input["packed_support"];
    let packed_runtime = &input["packed_runtime"];
    let peak_decision = &input["peak_decision"];
    let source_mode = mode(input, "packed_result");
    let peak_target = input["top_peak"]
        .as_str()
        .or_else(|| input["packed_peak"]["target"].as_str())
        .unwrap_or("")
        .to_string();
    let query_text = input["query"].as_str().map(str::to_string);
    UnifiedFieldReport {
        version: FIELD_CORE_VERSION,
        family: FieldFamily::Packed,
        source_mode: source_mode.clone(),
        basis: FieldBasis::packed_1024(),
        record: FieldRecordSummary {
            records: usize_number(&input["records"])
                .or_else(|| usize_number(&input["memory_records"]))
                .or_else(|| usize_number(&packed_runtime["triads"]))
                .unwrap_or(0),
            routes: usize_number(&packed_runtime["routes"]).unwrap_or(0),
            groups: usize_number(&packed_runtime["groups"]).unwrap_or(0),
            schemas: 0,
            surfaces: 0,
            evidence_refs: usize_number(&packed_support["support_count"]).unwrap_or(0),
        },
        query: FieldQuerySummary {
            source: "packed_query_wave".to_string(),
            text: query_text.clone(),
            requested_axes: vec!["route_id".to_string(), "group_id".to_string()],
            signature: input["query_wave"]["signature_hex"]
                .as_str()
                .map(str::to_string)
                .or_else(|| {
                    Some(adapter_field_signature(
                        FieldFamily::Packed,
                        &source_mode,
                        &peak_target,
                        query_text.as_deref(),
                    ))
                }),
        },
        peak: FieldPeakSummary {
            target: peak_target,
            score: number(&input["peak_score"])
                .or_else(|| number(&input["packed_peak"]["score"]))
                .unwrap_or(0.0),
            margin: number(&input["peak_margin"]).unwrap_or(0.0),
            state: peak_decision["state"]
                .as_str()
                .or_else(|| input["packed_decision"]["state"].as_str())
                .unwrap_or("WATCH")
                .to_string(),
            safe_to_answer: peak_decision["safe_to_answer"].as_bool().unwrap_or(false),
            support_count: usize_number(&packed_support["support_count"]).unwrap_or(0),
            anti_support_count: usize_number(&packed_support["anti_count"]).unwrap_or(0),
        },
        lens: FieldLensSummary {
            lenses: vec!["packed_axis".to_string(), "support_field".to_string()],
            amplified: vec![],
            suppressed: vec![],
            explanation: "packed fixed-record field projection".to_string(),
        },
        anti_wave: AntiWaveSummary {
            active: input.get("packed_lanes").is_some(),
            lanes: usize_number(&input["packed_lanes"]["count"])
                .or_else(|| array_len_opt(&input["packed_lanes"]["lanes"]))
                .unwrap_or(0),
            suppressed_target: input["packed_lanes"]["suppressed_peak"]
                .as_str()
                .map(str::to_string),
            suppression_energy: number(&input["packed_lanes"]["delta"]).unwrap_or(0.0),
            local_only: true,
        },
        coherence: FieldCoherenceSummary {
            coherence: number(&input["coherence"])
                .or_else(|| number(&input["packed_peak"]["cosine"]))
                .unwrap_or(0.0),
            energy: number(&packed_support["net_dot"]).unwrap_or(0.0),
            foreign_pull: usize_number(&input["foreign_pull"]).unwrap_or(0),
            field_state: input["field_state"]
                .as_str()
                .or_else(|| input["packed_state"].as_str())
                .unwrap_or("PACKED_FIELD")
                .to_string(),
            field_action: input["field_action"]
                .as_str()
                .unwrap_or("review")
                .to_string(),
        },
        feedback: feedback_summary(input),
        compatibility: FieldCompatibility {
            old_fields_required: strings(&["packed_support", "packed_lanes"]),
            ..FieldCompatibility::default()
        },
        claim_boundary: FieldClaimBoundary::default(),
    }
}

fn cognitive_report(input: &Value) -> UnifiedFieldReport {
    let claim = &input["claim_boundary"];
    let metrics = &input["metrics"];
    let source_mode = mode(input, "llmwave_big_result");
    let peak_target = input["top_peak"]
        .as_str()
        .or_else(|| input["schema_peak"]["schema"].as_str())
        .or_else(|| input["answer_state"].as_str())
        .unwrap_or("")
        .to_string();
    let query_text = input["input_text"]
        .as_str()
        .or_else(|| input["text"].as_str())
        .map(str::to_string);
    UnifiedFieldReport {
        version: FIELD_CORE_VERSION,
        family: FieldFamily::Cognitive,
        source_mode: source_mode.clone(),
        basis: FieldBasis::cognitive_1024(),
        record: FieldRecordSummary {
            records: usize_number(&metrics["record_count"])
                .or_else(|| usize_number(&input["store"]["record_count"]))
                .or_else(|| usize_number(&input["memory"]["records_written"]))
                .unwrap_or(0),
            routes: usize_number(&metrics["route_count"]).unwrap_or(0),
            groups: usize_number(&metrics["group_count"]).unwrap_or(0),
            schemas: usize_number(&metrics["schema_count"])
                .or_else(|| usize_number(&input["schema_count"]))
                .unwrap_or(0),
            surfaces: usize_number(&metrics["surface_count"])
                .or_else(|| usize_number(&input["family_reuse"]["forms"]))
                .unwrap_or(0),
            evidence_refs: usize_number(&metrics["evidence_count"]).unwrap_or(0),
        },
        query: FieldQuerySummary {
            source: "llmwave_big_query_or_fixture".to_string(),
            text: query_text.clone(),
            requested_axes: vec!["l2_surface".to_string(), "l3_schema".to_string()],
            signature: input["query_wave"]["signature_hex"]
                .as_str()
                .map(str::to_string)
                .or_else(|| {
                    Some(adapter_field_signature(
                        FieldFamily::Cognitive,
                        &source_mode,
                        &peak_target,
                        query_text.as_deref(),
                    ))
                }),
        },
        peak: FieldPeakSummary {
            target: peak_target,
            score: number(&input["top_score"])
                .or_else(|| number(&metrics["adjusted_top_score"]))
                .or_else(|| number(&metrics["baseline_top_score"]))
                .unwrap_or(0.0),
            margin: number(&input["peak_margin"]).unwrap_or(0.0),
            state: input["verdict"].as_str().unwrap_or("WATCH").to_string(),
            safe_to_answer: input["safe_to_answer"]
                .as_bool()
                .or_else(|| claim["safe_to_answer"].as_bool())
                .unwrap_or(false),
            support_count: array_len(&input["support"]),
            anti_support_count: array_len(&input["anti_support"]),
        },
        lens: FieldLensSummary {
            lenses: string_array(&input["lenses"])
                .if_empty(vec!["l2".to_string(), "l3".to_string()]),
            amplified: string_array(&input["constructive"]),
            suppressed: string_array(&input["suppressed"]),
            explanation: "LLMWave Big L2/L3/schema/surface projection".to_string(),
        },
        anti_wave: AntiWaveSummary {
            active: input.get("field_after_anti").is_some()
                || input.get("applied_anti_memory").is_some()
                || input["claim_boundary"]["suppresses_false_route"]
                    .as_bool()
                    .unwrap_or(false),
            lanes: array_len(&input["anti_lanes"])
                .max(usize_number(&input["negative_controls"]).unwrap_or(0)),
            suppressed_target: input["suppressed_peak"].as_str().map(str::to_string),
            suppression_energy: number(&input["suppression_energy"]).unwrap_or(0.0),
            local_only: input["claim_boundary"]["local_suppression_only"]
                .as_bool()
                .unwrap_or(true),
        },
        coherence: FieldCoherenceSummary {
            coherence: number(&metrics["coherence"])
                .or_else(|| number(&input["coherence"]))
                .unwrap_or(0.0),
            energy: number(&metrics["energy"])
                .or_else(|| number(&input["energy"]))
                .unwrap_or(0.0),
            foreign_pull: usize_number(&metrics["foreign_pull"]).unwrap_or(0),
            field_state: input["field_state"]
                .as_str()
                .or_else(|| input["final_state"].as_str())
                .unwrap_or("LLMWAVE_FIELD")
                .to_string(),
            field_action: input["field_action"]
                .as_str()
                .unwrap_or("review")
                .to_string(),
        },
        feedback: feedback_summary(input),
        compatibility: FieldCompatibility {
            old_fields_required: strings(&["roadmap_block", "claim_boundary"]),
            ..FieldCompatibility::default()
        },
        claim_boundary: FieldClaimBoundary {
            not_llm_ready: !claim["llm_ready"].as_bool().unwrap_or(false),
            not_nonlinear_memory_proof: !claim["nonlinear_memory_proven"]
                .as_bool()
                .unwrap_or(false),
            not_cache_only_proof: !claim["cache_only_execution_proven"]
                .as_bool()
                .unwrap_or(false),
            ..FieldClaimBoundary::default()
        },
    }
}

fn unknown_report(input: &Value) -> UnifiedFieldReport {
    UnifiedFieldReport {
        version: FIELD_CORE_VERSION,
        family: FieldFamily::Unknown,
        source_mode: mode(input, "unknown_result"),
        basis: FieldBasis::unknown(),
        record: FieldRecordSummary::default(),
        query: FieldQuerySummary::default(),
        peak: FieldPeakSummary::default(),
        lens: FieldLensSummary {
            explanation: "input did not match structural, packed, or cognitive field shape"
                .to_string(),
            ..FieldLensSummary::default()
        },
        anti_wave: AntiWaveSummary::default(),
        coherence: FieldCoherenceSummary::default(),
        feedback: feedback_summary(input),
        compatibility: FieldCompatibility::default(),
        claim_boundary: FieldClaimBoundary::default(),
    }
}

fn feedback_summary(input: &Value) -> FeedbackSummary {
    FeedbackSummary {
        feedback_present: input.get("feedback").is_some()
            || input.get("feedback_memory").is_some()
            || input.get("negative_shortcuts").is_some()
            || input.get("positive_shortcuts").is_some(),
        accepted: array_len(&input["positive_shortcuts"]) + array_len(&input["accepted"]),
        rejected: array_len(&input["negative_shortcuts"]) + array_len(&input["rejected"]),
        watched: array_len(&input["watch"]) + array_len(&input["watched"]),
        replayable: input.get("resonance_memory").is_some()
            || input.get("feedback_memory").is_some()
            || input.get("continuation_memory").is_some(),
    }
}

fn adapter_field_signature(
    family: FieldFamily,
    source_mode: &str,
    peak_target: &str,
    query_text: Option<&str>,
) -> String {
    let vector = FieldVector1024::project_record(&FieldTriadProjection {
        subject: format!("adapter:{}", family.as_str()),
        relation: source_mode.to_string(),
        object: query_text.unwrap_or("no_query_text").to_string(),
        route: Some(peak_target.to_string()),
        group: Some("field_adapter".to_string()),
    });
    vector.signature_hex()
}

fn mode(input: &Value, default: &str) -> String {
    input["mode"]
        .as_str()
        .or_else(|| input["version"].as_str())
        .unwrap_or(default)
        .to_string()
}

fn query_source(input: &Value) -> String {
    input["query"]["source"]
        .as_str()
        .unwrap_or("structural_packet_or_search_result")
        .to_string()
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|v| v as f64))
        .or_else(|| value.as_u64().map(|v| v as f64))
}

fn usize_number(value: &Value) -> Option<usize> {
    value.as_u64().and_then(|value| usize::try_from(value).ok())
}

fn count_array(input: &Value, key: &str) -> usize {
    array_len(&input[key])
}

fn array_len(value: &Value) -> usize {
    value.as_array().map(Vec::len).unwrap_or(0)
}

fn array_len_opt(value: &Value) -> Option<usize> {
    value.as_array().map(Vec::len)
}

fn object_len(value: &Value) -> usize {
    value.as_object().map(serde_json::Map::len).unwrap_or(0)
}

fn string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

trait IfEmpty {
    fn if_empty(self, fallback: Vec<String>) -> Vec<String>;
}

impl IfEmpty for Vec<String> {
    fn if_empty(self, fallback: Vec<String>) -> Vec<String> {
        if self.is_empty() {
            fallback
        } else {
            self
        }
    }
}
