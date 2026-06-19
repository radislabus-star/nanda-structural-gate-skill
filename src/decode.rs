use crate::*;
use serde::Deserialize;

pub(crate) fn decode_cmd(args: DecodeArgs) -> Result<u8> {
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let memory = normalize_ids(packet.triads.clone(), "m");
    let mut query_packet = if let Some(query_file) = &args.query_file {
        load_packet_auto(
            query_file,
            &args.query_format,
            &args.task_id,
            &args.domain,
            &args.query,
            args.normalize_paths,
        )?
    } else {
        packet.clone()
    };
    inherit_aliases_if_needed(&mut query_packet, &packet);
    let query_text = if !args.query.trim().is_empty() {
        args.query.clone()
    } else if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let out = recurrent_decode_report(
        &packet,
        &memory,
        &query,
        query_source,
        &args,
        args.steps.clamp(1, 16),
    );
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_decode_text(&out),
        OutputFormat::Md => print_decode_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn decode_eval_cmd(args: DecodeEvalArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.suite)
        .with_context(|| format!("read {}", args.suite.display()))?;
    let suite: DecodeEvalSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", args.suite.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow!(
            "nanda decode-eval requires a suite with at least one case"
        ));
    }
    let base = args.suite.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    for case in suite.cases {
        let path = resolve_suite_path(base, &case.path);
        let mut packet = load_packet_auto(
            &path,
            &args.input_format,
            "decode-eval",
            "general",
            &case.query,
            args.normalize_paths,
        )?;
        let memory = normalize_ids(packet.triads.clone(), "m");
        let mut query_packet = if let Some(query_file) = &case.query_file {
            load_packet_auto(
                &resolve_suite_path(base, query_file),
                &args.input_format,
                "decode-eval",
                "general",
                &case.query,
                args.normalize_paths,
            )?
        } else {
            packet.clone()
        };
        inherit_aliases_if_needed(&mut query_packet, &packet);
        let query_text = if !case.query.trim().is_empty() {
            case.query.clone()
        } else if !query_packet.query.trim().is_empty() {
            query_packet.query.clone()
        } else {
            packet.query.clone()
        };
        packet.query = query_text.clone();
        let (query, query_source) = search_query_triads(&query_packet, &query_text);
        let decode_args = DecodeArgs {
            input: path.clone(),
            input_format: args.input_format.clone(),
            task_id: "decode-eval".to_string(),
            domain: "general".to_string(),
            query: query_text,
            query_file: None,
            query_format: args.input_format.clone(),
            top_k: case.top_k.unwrap_or(args.top_k),
            steps: case.steps.unwrap_or(args.steps).clamp(1, 16),
            search_top_k: args.search_top_k,
            route_cap: args.route_cap,
            route_triad_cap: args.route_triad_cap,
            group_by: args.group_by.clone(),
            format: OutputFormat::Json,
            normalize_paths: args.normalize_paths,
        };
        let result = recurrent_decode_report(
            &packet,
            &memory,
            &query,
            query_source,
            &decode_args,
            decode_args.steps,
        );
        let actual_state = result["decoder_state"].as_str().unwrap_or("").to_string();
        let actual_top_pattern = result["top_pattern"].as_str().unwrap_or("").to_string();
        let actual_final_state = result["recurrent"]["final_decoder_state"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let completed_steps = result["recurrent"]["completed_steps"].as_u64().unwrap_or(0) as usize;
        let state_ok =
            case.expected_decoder_state.is_empty() || actual_state == case.expected_decoder_state;
        let pattern_ok =
            case.expected_top_pattern.is_empty() || actual_top_pattern == case.expected_top_pattern;
        let final_ok = case.expected_final_decoder_state.is_empty()
            || actual_final_state == case.expected_final_decoder_state;
        let steps_ok = completed_steps >= case.min_completed_steps.unwrap_or(1);
        let ok = state_ok && pattern_ok && final_ok && steps_ok;
        if ok {
            passed += 1;
        }
        rows.push(json!({
            "id": if case.id.is_empty() { path.display().to_string() } else { case.id },
            "case": path.display().to_string(),
            "expected_decoder_state": case.expected_decoder_state,
            "actual_decoder_state": actual_state,
            "expected_top_pattern": case.expected_top_pattern,
            "actual_top_pattern": actual_top_pattern,
            "expected_final_decoder_state": case.expected_final_decoder_state,
            "actual_final_decoder_state": actual_final_state,
            "completed_steps": completed_steps,
            "min_completed_steps": case.min_completed_steps.unwrap_or(1),
            "ok": ok
        }));
    }
    let total = rows.len();
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "decode-eval-suite",
        "suite": if suite.name.is_empty() { args.suite.display().to_string() } else { suite.name },
        "passed": passed,
        "total": total,
        "accuracy": round4(passed as f64 / total.max(1) as f64),
        "cases": rows,
        "interpretation": "Decode eval checks whether the wave decoder produces expected structural continuations and honest recurrent stop states."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_decode_eval_text(&out),
        OutputFormat::Md => print_decode_eval_md(&out),
    }
    if passed == total {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

fn resolve_suite_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

pub(crate) fn recurrent_decode_report(
    packet: &Packet,
    memory: &[Triad],
    query: &[Triad],
    query_source: &str,
    args: &DecodeArgs,
    steps: usize,
) -> Value {
    let mut current_query = query.to_vec();
    let mut step_reports = vec![];
    for step_idx in 0..steps {
        let focus =
            route_balanced_focus(memory, &current_query, args.route_cap, args.route_triad_cap);
        let search = interference_search(
            packet,
            &focus.memory,
            &current_query,
            args.search_top_k.max(args.top_k),
            &args.group_by,
            query_source,
            focus.metadata,
        );
        let mut step =
            decode_step_report(packet, &current_query, &search, args.top_k, step_idx + 1);
        if let Some(selected) = select_recurrent_pattern(&step["patterns"], &current_query) {
            let next = pattern_to_query_triad(&selected, step_idx + 1);
            if let Some(object) = step.as_object_mut() {
                object.insert("selected_pattern".to_string(), selected);
                object.insert("selected_query_triad".to_string(), triad_json(&next));
            }
            current_query.push(next);
        } else {
            if let Some(object) = step.as_object_mut() {
                object.insert("decoder_state".to_string(), json!("PATTERN_SATURATED"));
                object.insert("safe_to_generate".to_string(), json!(false));
                object.insert("selected_pattern".to_string(), json!(null));
                object.insert("selected_query_triad".to_string(), json!(null));
            }
            step_reports.push(step);
            break;
        }
        step_reports.push(step);
    }
    let first = step_reports.first().cloned().unwrap_or_else(|| {
        json!({
            "decoder_state": "NO_PATTERN",
            "safe_to_generate": false,
            "top_pattern": "",
            "patterns": []
        })
    });
    let final_step = step_reports
        .last()
        .cloned()
        .unwrap_or_else(|| first.clone());
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "wave-pattern-decoder",
        "decoder_version": if steps > 1 { "v31-recurrent-wave-decoder" } else { "v30-pattern-store-wave-decoder" },
        "task_id": packet.task_id,
        "domain": packet.domain,
        "query": first["query"],
        "source_search": first["source_search"],
        "decoder_state": first["decoder_state"],
        "safe_to_generate": first["safe_to_generate"],
        "continuation_training": first["continuation_training"],
        "top_pattern": first["top_pattern"],
        "patterns": first["patterns"],
        "recurrent": {
            "enabled": steps > 1,
            "requested_steps": steps,
            "completed_steps": step_reports.len(),
            "final_decoder_state": final_step["decoder_state"],
            "final_top_pattern": final_step["top_pattern"],
            "final_context": current_query.iter().map(triad_json).collect::<Vec<_>>(),
            "steps": step_reports,
            "read_as": "Each recurrent step decodes a next structural pattern, feeds it back as query context, and re-runs the field."
        },
        "read_as": if steps > 1 {
            "This is a recurrent LLMWave bridge: field peak -> next structural pattern -> updated field context."
        } else {
            "This is the first LLMWave bridge: it decodes the interference field into ranked next structural patterns, not natural-language text."
        }
    })
}

pub(crate) fn decode_step_report(
    packet: &Packet,
    query: &[Triad],
    search: &Value,
    top_k: usize,
    step: usize,
) -> Value {
    let mut candidates = decode_candidates(search, query);
    let continuation_training =
        apply_continuation_memory(&mut candidates, query, &packet.continuation_memory);
    candidates.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(top_k);
    let top_candidate = candidates
        .first()
        .map(|item| {
            format!(
                "{} -> {} -> {}",
                item["subject"].as_str().unwrap_or(""),
                item["relation"].as_str().unwrap_or(""),
                item["object"].as_str().unwrap_or("")
            )
        })
        .unwrap_or_default();
    let decoder_state = if candidates.is_empty() {
        "NO_PATTERN"
    } else if search["peak_decision"]["state"].as_str().unwrap_or("") == "FOCUSED"
        && search["resonant_field"]["waw_status"]
            .as_str()
            .unwrap_or("")
            == "WAW_RESONANCE"
    {
        "PATTERN_READY"
    } else if search["field_state"].as_str().unwrap_or("") == "FIELD_REVERSED" {
        "PATTERN_BLOCKED"
    } else {
        "PATTERN_REVIEW"
    };
    json!({
        "step": step,
        "query": search["query"],
        "source_search": {
            "verdict": search["verdict"],
            "field_state": search["field_state"],
            "safe_to_answer": search["safe_to_answer"],
            "top_peak": search["top_peak"],
            "peak_margin": search["peak_margin"],
            "resonance": search["resonant_field"],
            "resonance_memory": search["resonance_memory"]
        },
        "decoder_state": decoder_state,
        "safe_to_generate": decoder_state == "PATTERN_READY",
        "continuation_training": continuation_training,
        "top_pattern": top_candidate,
        "patterns": candidates
    })
}

fn apply_continuation_memory(
    candidates: &mut [Value],
    query: &[Triad],
    memories: &[ContinuationMemory],
) -> Value {
    let query_terms = query_term_set(query);
    let mut applications = vec![];
    if memories.is_empty() || candidates.is_empty() {
        return json!({
            "applied": false,
            "continuation_memory": memories.len(),
            "applications": applications
        });
    }
    for memory in memories {
        let query_ratio = continuation_query_match_ratio(&query_terms, memory);
        if query_ratio <= 0.0 {
            continue;
        }
        for candidate in candidates.iter_mut() {
            if !continuation_matches_candidate(memory, candidate) {
                continue;
            }
            let support_ratio = continuation_support_ratio(memory, candidate);
            let match_ratio = round4(query_ratio * support_ratio);
            if match_ratio <= 0.0 {
                continue;
            }
            let old_score = candidate["score"].as_f64().unwrap_or(0.0);
            let decision = memory.decision.as_str();
            let accepted_count = memory.accepted_count.max(usize::from(decision == "accept"));
            let rejected_count = memory.rejected_count.max(usize::from(decision == "reject"));
            let (delta, action) = match decision {
                "accept" => (
                    round4(
                        (memory.boost.max(0.0) + accepted_count.saturating_sub(1) as f64 * 0.025)
                            .min(0.25)
                            * match_ratio,
                    ),
                    "reinforce",
                ),
                "reject" => (
                    -round4(
                        (memory.penalty.max(0.0) + rejected_count.saturating_sub(1) as f64 * 0.035)
                            .min(0.35)
                            * match_ratio,
                    ),
                    "suppress",
                ),
                _ => (0.0, "watch"),
            };
            if delta == 0.0 {
                continue;
            }
            let new_score = round4((old_score + delta).clamp(-1.0, 1.5));
            if let Some(object) = candidate.as_object_mut() {
                object.insert("score".to_string(), json!(new_score));
                object.insert("raw_decode_score".to_string(), json!(round4(old_score)));
                object.insert("continuation_memory_delta".to_string(), json!(delta));
            }
            applications.push(json!({
                "memory": memory.id,
                "action": action,
                "pattern": format!("{} -> {} -> {}", memory.subject, memory.relation, memory.object),
                "delta": delta,
                "match_ratio": match_ratio,
                "query_match_ratio": round4(query_ratio),
                "support_match_ratio": round4(support_ratio),
                "observations": memory.observations,
                "accepted_count": accepted_count,
                "rejected_count": rejected_count,
                "reason": memory.reason
            }));
        }
    }
    json!({
        "applied": !applications.is_empty(),
        "continuation_memory": memories.len(),
        "applications": applications,
        "read_as": "Continuation memory softly reinforces accepted decoded patterns and suppresses rejected decoded patterns before recurrent selection."
    })
}

fn continuation_matches_candidate(memory: &ContinuationMemory, candidate: &Value) -> bool {
    if norm(&memory.subject) != norm(candidate["subject"].as_str().unwrap_or("")) {
        return false;
    }
    if norm(&memory.relation) != norm(candidate["relation"].as_str().unwrap_or("")) {
        return false;
    }
    if norm(&memory.object) != norm(candidate["object"].as_str().unwrap_or("")) {
        return false;
    }
    if !memory.route.is_empty()
        && norm(&memory.route) != norm(candidate["route"].as_str().unwrap_or(""))
    {
        return false;
    }
    true
}

fn continuation_query_match_ratio(
    query_terms: &BTreeSet<String>,
    memory: &ContinuationMemory,
) -> f64 {
    if memory.terms.is_empty() || query_terms.is_empty() {
        return 1.0;
    }
    let terms = normalized_shortcut_terms(&memory.terms);
    if terms.is_empty() {
        return 1.0;
    }
    terms.intersection(query_terms).count() as f64 / terms.len() as f64
}

fn continuation_support_ratio(memory: &ContinuationMemory, candidate: &Value) -> f64 {
    if memory.support_terms.is_empty() {
        return 1.0;
    }
    let candidate_terms = ["subject", "relation", "object", "route", "group", "peak"]
        .into_iter()
        .filter_map(|key| candidate[key].as_str())
        .flat_map(|value| {
            norm(value)
                .split(|ch: char| !ch.is_ascii_alphanumeric())
                .filter(|token| token.len() >= 2)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();
    let support_terms = normalized_shortcut_terms(&memory.support_terms);
    if support_terms.is_empty() || candidate_terms.is_empty() {
        return 1.0;
    }
    support_terms.intersection(&candidate_terms).count() as f64 / support_terms.len() as f64
}

fn decode_candidates(search: &Value, query: &[Triad]) -> Vec<Value> {
    let query_terms = query_term_set(query);
    let mut by_key: BTreeMap<(String, String, String), Value> = BTreeMap::new();
    let Some(peaks) = search["peaks"].as_array() else {
        return vec![];
    };
    for (peak_rank, peak) in peaks.iter().enumerate() {
        let peak_score = peak["score"].as_f64().unwrap_or(0.0);
        let peak_name = peak["peak"].as_str().unwrap_or("");
        let peak_state = if peak_rank == 0 {
            search["peak_decision"]["state"].as_str().unwrap_or("")
        } else {
            "RIVAL"
        };
        if let Some(support) = peak["supporting_triads"].as_array() {
            for item in support.iter().take(8) {
                let subject = item["subject"].as_str().unwrap_or("").to_string();
                let relation = item["relation"].as_str().unwrap_or("").to_string();
                let object = item["object"].as_str().unwrap_or("").to_string();
                if subject.is_empty() || relation.is_empty() || object.is_empty() {
                    continue;
                }
                let key = (norm(&subject), norm(&relation), norm(&object));
                let triad_score = item["score"].as_f64().unwrap_or(0.0);
                let source_weight = item["source_weight"].as_f64().unwrap_or(1.0);
                let continuity = pattern_continuity(&query_terms, item);
                let resonance_bonus = if search["resonant_field"]["waw_status"]
                    .as_str()
                    .unwrap_or("")
                    == "WAW_RESONANCE"
                    && peak_rank == 0
                {
                    0.06
                } else {
                    0.0
                };
                let score = round4(
                    (0.44 * peak_score)
                        + (0.32 * triad_score)
                        + (0.12 * source_weight.min(1.0))
                        + (0.12 * continuity)
                        + resonance_bonus,
                );
                let candidate = json!({
                    "pattern_id": format!("pat-{}", slug(&format!("{subject}-{relation}-{object}"))),
                    "score": score,
                    "subject": subject,
                    "relation": relation,
                    "object": object,
                    "subject_role": item["subject_role"].as_str().unwrap_or(""),
                    "object_role": item["object_role"].as_str().unwrap_or(""),
                    "route": item["route"].as_str().unwrap_or(""),
                    "group": item["group"].as_str().unwrap_or(""),
                    "peak": peak_name,
                    "peak_rank": peak_rank + 1,
                    "peak_state": peak_state,
                    "triad": item["id"].as_str().unwrap_or(""),
                    "triad_score": round4(triad_score),
                    "continuity": round4(continuity),
                    "source_weight": round4(source_weight),
                    "polarity": item["polarity"].as_str().unwrap_or(""),
                    "decode_as": "next_structural_pattern"
                });
                by_key
                    .entry(key)
                    .and_modify(|existing| {
                        if candidate["score"].as_f64().unwrap_or(0.0)
                            > existing["score"].as_f64().unwrap_or(0.0)
                        {
                            *existing = candidate.clone();
                        }
                    })
                    .or_insert(candidate);
            }
        }
    }
    by_key.into_values().collect()
}

fn select_recurrent_pattern(patterns: &Value, current_query: &[Triad]) -> Option<Value> {
    let items = patterns.as_array()?;
    let current = current_query
        .iter()
        .map(|triad| {
            (
                norm(&triad.subject),
                norm(&triad.relation),
                norm(&triad.object),
            )
        })
        .collect::<BTreeSet<_>>();
    items
        .iter()
        .find(|item| {
            let key = (
                norm(item["subject"].as_str().unwrap_or("")),
                norm(item["relation"].as_str().unwrap_or("")),
                norm(item["object"].as_str().unwrap_or("")),
            );
            !current.contains(&key)
        })
        .cloned()
}

fn pattern_to_query_triad(pattern: &Value, step: usize) -> Triad {
    Triad {
        id: format!("qd{step}"),
        subject: pattern["subject"].as_str().unwrap_or("").to_string(),
        relation: pattern["relation"].as_str().unwrap_or("").to_string(),
        object: pattern["object"].as_str().unwrap_or("").to_string(),
        evidence: "recurrent_decode".to_string(),
        confidence: pattern["score"].as_f64().unwrap_or(0.72).clamp(0.1, 1.0),
        subject_role: pattern["subject_role"].as_str().unwrap_or("").to_string(),
        object_role: pattern["object_role"].as_str().unwrap_or("").to_string(),
        route: pattern["route"].as_str().unwrap_or("").to_string(),
        group: "decoded-context".to_string(),
    }
}

fn pattern_continuity(query_terms: &BTreeSet<String>, item: &Value) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let mut terms = BTreeSet::new();
    for key in ["subject", "relation", "object", "route", "group"] {
        if let Some(value) = item[key].as_str() {
            terms.extend(
                norm(value)
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|token| token.chars().count() >= 2)
                    .map(str::to_string),
            );
        }
    }
    if terms.is_empty() {
        return 0.0;
    }
    query_terms.intersection(&terms).count() as f64 / query_terms.len() as f64
}

pub(crate) fn print_decode_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "decoder_state: {}",
        out["decoder_state"].as_str().unwrap_or("")
    );
    println!(
        "safe_to_generate: {}",
        out["safe_to_generate"].as_bool().unwrap_or(false)
    );
    println!("top_pattern: {}", out["top_pattern"].as_str().unwrap_or(""));
    if let Some(patterns) = out["patterns"].as_array() {
        for (idx, pattern) in patterns.iter().enumerate() {
            println!(
                "{}. score={} {} -> {} -> {}",
                idx + 1,
                pattern["score"].as_f64().unwrap_or(0.0),
                pattern["subject"].as_str().unwrap_or(""),
                pattern["relation"].as_str().unwrap_or(""),
                pattern["object"].as_str().unwrap_or("")
            );
        }
    }
}

pub(crate) fn print_decode_md(out: &Value) {
    println!("# NANDA Wave Decode\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "- decoder_state: `{}`",
        out["decoder_state"].as_str().unwrap_or("")
    );
    println!("- safe_to_generate: `{}`", out["safe_to_generate"]);
    println!(
        "- top_pattern: `{}`",
        out["top_pattern"].as_str().unwrap_or("")
    );
    if let Some(patterns) = out["patterns"].as_array() {
        for (idx, pattern) in patterns.iter().enumerate() {
            println!(
                "\n{}. `{}` -> `{}` -> `{}`",
                idx + 1,
                pattern["subject"].as_str().unwrap_or(""),
                pattern["relation"].as_str().unwrap_or(""),
                pattern["object"].as_str().unwrap_or("")
            );
            println!("- score: `{}`", pattern["score"]);
            println!("- peak: `{}`", pattern["peak"]);
            println!("- route: `{}`", pattern["route"]);
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct DecodeEvalSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<DecodeEvalCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct DecodeEvalCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    #[serde(default)]
    query_file: Option<PathBuf>,
    #[serde(default)]
    query: String,
    #[serde(default)]
    expected_decoder_state: String,
    #[serde(default)]
    expected_top_pattern: String,
    #[serde(default)]
    expected_final_decoder_state: String,
    #[serde(default)]
    min_completed_steps: Option<usize>,
    #[serde(default)]
    steps: Option<usize>,
    #[serde(default)]
    top_k: Option<usize>,
}

pub(crate) fn print_decode_eval_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {}: {} pattern={} final={}",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_top_pattern"].as_str().unwrap_or(""),
                case["actual_final_decoder_state"].as_str().unwrap_or("")
            );
        }
    }
}

pub(crate) fn print_decode_eval_md(out: &Value) {
    println!("# NANDA Decode Eval\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- passed: `{}/{}`", out["passed"], out["total"]);
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "\n- `{}`: `{}` top=`{}` final=`{}`",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_top_pattern"].as_str().unwrap_or(""),
                case["actual_final_decoder_state"].as_str().unwrap_or("")
            );
        }
    }
}
