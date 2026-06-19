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
            beam_width: case.beam_width.unwrap_or(args.beam_width),
            adaptive_scoring: args.adaptive_scoring,
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
        let top_beam = result["beam_decode"]["trajectories"]
            .as_array()
            .and_then(|items| items.first())
            .cloned()
            .unwrap_or_else(|| json!({}));
        let actual_beam_route = top_beam["route_center"].as_str().unwrap_or("").to_string();
        let actual_beam_length = top_beam["length"].as_u64().unwrap_or(0) as usize;
        let actual_beam_saturated = top_beam["saturated"].as_bool().unwrap_or(false);
        let beam_chain = top_beam["chain"].as_array().cloned().unwrap_or_default();
        let forbidden_seen = !case.forbidden_beam_pattern.is_empty()
            && beam_chain
                .iter()
                .any(|item| item.as_str() == Some(case.forbidden_beam_pattern.as_str()));
        let completed_steps = result["recurrent"]["completed_steps"].as_u64().unwrap_or(0) as usize;
        let state_ok =
            case.expected_decoder_state.is_empty() || actual_state == case.expected_decoder_state;
        let pattern_ok =
            case.expected_top_pattern.is_empty() || actual_top_pattern == case.expected_top_pattern;
        let final_ok = case.expected_final_decoder_state.is_empty()
            || actual_final_state == case.expected_final_decoder_state;
        let steps_ok = completed_steps >= case.min_completed_steps.unwrap_or(1);
        let beam_route_ok =
            case.expected_beam_route.is_empty() || actual_beam_route == case.expected_beam_route;
        let beam_length_ok = actual_beam_length >= case.min_beam_length.unwrap_or(0);
        let beam_saturated_ok = case
            .expected_beam_saturated
            .is_none_or(|expected| actual_beam_saturated == expected);
        let forbidden_ok = !forbidden_seen;
        let ok = state_ok
            && pattern_ok
            && final_ok
            && steps_ok
            && beam_route_ok
            && beam_length_ok
            && beam_saturated_ok
            && forbidden_ok;
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
            "expected_beam_route": case.expected_beam_route,
            "actual_beam_route": actual_beam_route,
            "min_beam_length": case.min_beam_length.unwrap_or(0),
            "actual_beam_length": actual_beam_length,
            "expected_beam_saturated": case.expected_beam_saturated,
            "actual_beam_saturated": actual_beam_saturated,
            "forbidden_beam_pattern": case.forbidden_beam_pattern,
            "forbidden_seen": forbidden_seen,
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
        let mut step = decode_step_report(
            packet,
            &current_query,
            &search,
            args.top_k,
            step_idx + 1,
            args.adaptive_scoring,
        );
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
    let beam_decode = beam_decode_report(packet, memory, query, query_source, args, steps);
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "wave-pattern-decoder",
        "decoder_version": if args.beam_width > 1 { "v42-beam-wave-decoder" } else if steps > 1 { "v31-recurrent-wave-decoder" } else { "v30-pattern-store-wave-decoder" },
        "task_id": packet.task_id,
        "domain": packet.domain,
        "query": first["query"],
        "source_search": first["source_search"],
        "decoder_state": first["decoder_state"],
        "safe_to_generate": first["safe_to_generate"],
        "continuation_training": first["continuation_training"],
        "compact_pattern_store": first["compact_pattern_store"],
        "top_pattern": first["top_pattern"],
        "patterns": first["patterns"],
        "beam_decode": beam_decode,
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
        "read_as": if args.beam_width > 1 {
            "This is a beam LLMWave bridge: it keeps several structural continuations in superposition and ranks stable trajectories."
        } else if steps > 1 {
            "This is a recurrent LLMWave bridge: field peak -> next structural pattern -> updated field context."
        } else {
            "This is the first LLMWave bridge: it decodes the interference field into ranked next structural patterns, not natural-language text."
        }
    })
}

#[derive(Clone)]
struct DecodeBeam {
    context: Vec<Triad>,
    patterns: Vec<Value>,
    score_sum: f64,
    saturated: bool,
}

pub(crate) fn beam_decode_report(
    packet: &Packet,
    memory: &[Triad],
    query: &[Triad],
    query_source: &str,
    args: &DecodeArgs,
    steps: usize,
) -> Value {
    let width = args.beam_width.clamp(1, 8);
    if width <= 1 {
        return json!({
            "enabled": false,
            "beam_width": width,
            "version": "v42-beam-wave-decoder"
        });
    }
    let mut beams = vec![DecodeBeam {
        context: query.to_vec(),
        patterns: vec![],
        score_sum: 0.0,
        saturated: false,
    }];
    for step_idx in 0..steps.clamp(1, 16) {
        let mut next_beams = vec![];
        for beam in &beams {
            if beam.saturated {
                next_beams.push(beam.clone());
                continue;
            }
            let focus =
                route_balanced_focus(memory, &beam.context, args.route_cap, args.route_triad_cap);
            let search = interference_search(
                packet,
                &focus.memory,
                &beam.context,
                args.search_top_k.max(args.top_k).max(width),
                &args.group_by,
                query_source,
                focus.metadata,
            );
            let step = decode_step_report(
                packet,
                &beam.context,
                &search,
                args.top_k.max(width),
                step_idx + 1,
                args.adaptive_scoring,
            );
            let mut expanded = 0usize;
            for pattern in step["patterns"].as_array().into_iter().flatten() {
                if pattern_seen(&beam.context, pattern) {
                    continue;
                }
                let mut candidate = beam.clone();
                let mut selected = pattern.clone();
                if let Some(object) = selected.as_object_mut() {
                    object.insert("beam_step".to_string(), json!(step_idx + 1));
                }
                candidate
                    .context
                    .push(pattern_to_query_triad(&selected, step_idx + 1));
                candidate.score_sum += selected["score"].as_f64().unwrap_or(0.0);
                candidate.patterns.push(selected);
                next_beams.push(candidate);
                expanded += 1;
                if expanded >= width {
                    break;
                }
            }
            if expanded == 0 {
                let mut saturated = beam.clone();
                saturated.saturated = true;
                next_beams.push(saturated);
            }
        }
        next_beams.sort_by(|a, b| {
            beam_score(b)
                .partial_cmp(&beam_score(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        next_beams.truncate(width);
        beams = next_beams;
    }
    let trajectories = beams
        .iter()
        .enumerate()
        .map(|(idx, beam)| {
            let final_pattern = beam.patterns.last().cloned().unwrap_or(Value::Null);
            let route = weighted_label(beam.patterns.iter().filter_map(|pattern| {
                pattern["route"]
                    .as_str()
                    .map(|route| (pattern["score"].as_f64().unwrap_or(0.0), route.to_string()))
            }));
            let chain = beam.patterns.iter().map(pattern_label).collect::<Vec<_>>();
            json!({
                "rank": idx + 1,
                "score": round4(beam_score(beam)),
                "length": beam.patterns.len(),
                "saturated": beam.saturated,
                "route_center": route,
                "final_pattern": final_pattern,
                "chain": chain,
                "patterns": beam.patterns,
                "context": beam.context.iter().map(triad_json).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    json!({
        "enabled": true,
        "version": "v42-beam-wave-decoder",
        "beam_width": width,
        "steps": steps.clamp(1, 16),
        "trajectories": trajectories,
        "read_as": "Beam decode keeps several structural continuations in superposition and ranks the most stable pattern trajectory."
    })
}

pub(crate) fn decode_step_report(
    packet: &Packet,
    query: &[Triad],
    search: &Value,
    top_k: usize,
    step: usize,
    adaptive_scoring: bool,
) -> Value {
    let scoring_policy = pattern_scoring_policy(search, adaptive_scoring);
    let mut candidates = decode_candidates(search, query, &scoring_policy);
    let raw_candidates = candidates.clone();
    let raw_top_pattern = raw_candidates
        .iter()
        .max_by(|a, b| {
            a["score"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&b["score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(pattern_label)
        .unwrap_or_default();
    let continuation_training = if packet.continuation_memory.is_empty() {
        apply_continuation_memory(&mut candidates, query, &packet.continuation_memory)
    } else {
        apply_compact_pattern_store(&mut candidates, query, &packet.continuation_memory)
    };
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
    let early_pattern_replay = early_pattern_replay_report(
        packet,
        &continuation_training,
        raw_candidates.len(),
        &raw_top_pattern,
        &top_candidate,
    );
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
        "adaptive_pattern_scoring": scoring_policy,
        "continuation_training": continuation_training,
        "early_pattern_replay": early_pattern_replay,
        "compact_pattern_store": compact_pattern_store_report(packet, 3),
        "top_pattern": top_candidate,
        "patterns": candidates
    })
}

fn early_pattern_replay_report(
    packet: &Packet,
    replay: &Value,
    candidates_before: usize,
    raw_top_pattern: &str,
    adjusted_top_pattern: &str,
) -> Value {
    let applications = replay["applications"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let suppressions = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("suppress"))
        .count();
    let reinforcements = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("reinforce"))
        .count();
    json!({
        "version": "v44-pre-ranking-pattern-replay",
        "stage": "before_decode_ranking",
        "applied": replay["applied"].as_bool().unwrap_or(false),
        "pattern_records": packet.continuation_memory.len(),
        "packed_pattern_bytes": PACKED_PATTERN_BYTES,
        "candidates_before": candidates_before,
        "raw_top_pattern": raw_top_pattern,
        "adjusted_top_pattern": adjusted_top_pattern,
        "top_changed": !raw_top_pattern.is_empty() && raw_top_pattern != adjusted_top_pattern,
        "applications": applications.len(),
        "suppressions": suppressions,
        "reinforcements": reinforcements,
        "hot_bridge": "candidate-pattern-signatures are adjusted before final decode ranking; this exposes the cold-to-hot replay boundary for NANDA-6M."
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

fn pattern_scoring_policy(search: &Value, adaptive: bool) -> Value {
    let field_state = search["field_state"].as_str().unwrap_or("");
    let waw = search["resonant_field"]["waw_status"]
        .as_str()
        .unwrap_or("");
    let (peak, triad, source, continuity, resonance, source_name) = if !adaptive {
        (0.44, 0.32, 0.12, 0.12, 0.06, "static-v30")
    } else if field_state == "FIELD_FOCUSED" && waw == "WAW_RESONANCE" {
        (
            0.48,
            0.30,
            0.10,
            0.12,
            0.08,
            "adaptive-v45-focused-resonance",
        )
    } else if matches!(
        field_state,
        "FIELD_CONTESTED" | "FIELD_THIN" | "FIELD_NOISY"
    ) {
        (
            0.36,
            0.30,
            0.16,
            0.18,
            0.04,
            "adaptive-v45-contested-support",
        )
    } else {
        (0.42, 0.32, 0.13, 0.13, 0.05, "adaptive-v45-balanced")
    };
    json!({
        "version": "v45-adaptive-pattern-scoring",
        "enabled": adaptive,
        "source": source_name,
        "field_state": field_state,
        "waw_status": waw,
        "weights": {
            "peak": peak,
            "triad": triad,
            "source": source,
            "continuity": continuity,
            "resonance_bonus": resonance
        },
        "read_as": "Adaptive scoring shifts decode weights from fixed constants toward field-state-aware pattern ranking while keeping static scoring as the default."
    })
}

fn decode_candidates(search: &Value, query: &[Triad], policy: &Value) -> Vec<Value> {
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
                let weights = &policy["weights"];
                let score = round4(
                    (weights["peak"].as_f64().unwrap_or(0.44) * peak_score)
                        + (weights["triad"].as_f64().unwrap_or(0.32) * triad_score)
                        + (weights["source"].as_f64().unwrap_or(0.12) * source_weight.min(1.0))
                        + (weights["continuity"].as_f64().unwrap_or(0.12) * continuity)
                        + if resonance_bonus > 0.0 {
                            weights["resonance_bonus"].as_f64().unwrap_or(0.06)
                        } else {
                            0.0
                        },
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
                    "scoring_policy": policy["source"],
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

fn pattern_seen(current_query: &[Triad], pattern: &Value) -> bool {
    let key = (
        norm(pattern["subject"].as_str().unwrap_or("")),
        norm(pattern["relation"].as_str().unwrap_or("")),
        norm(pattern["object"].as_str().unwrap_or("")),
    );
    current_query.iter().any(|triad| {
        (
            norm(&triad.subject),
            norm(&triad.relation),
            norm(&triad.object),
        ) == key
    })
}

fn beam_score(beam: &DecodeBeam) -> f64 {
    if beam.patterns.is_empty() {
        return 0.0;
    }
    let length = beam.patterns.len() as f64;
    let average = beam.score_sum / length;
    let depth_bonus = (length * 0.025).min(0.12);
    let saturation_penalty = if beam.saturated { 0.04 } else { 0.0 };
    round4((average + depth_bonus - saturation_penalty).clamp(-1.0, 1.5))
}

fn weighted_label(items: impl Iterator<Item = (f64, String)>) -> String {
    let mut scores = BTreeMap::<String, f64>::new();
    for (score, label) in items {
        if label.trim().is_empty() {
            continue;
        }
        *scores.entry(label).or_default() += score.max(0.0);
    }
    scores
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(label, _)| label)
        .unwrap_or_default()
}

fn pattern_label(pattern: &Value) -> String {
    format!(
        "{} -> {} -> {}",
        pattern["subject"].as_str().unwrap_or(""),
        pattern["relation"].as_str().unwrap_or(""),
        pattern["object"].as_str().unwrap_or("")
    )
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
    expected_beam_route: String,
    #[serde(default)]
    min_beam_length: Option<usize>,
    #[serde(default)]
    expected_beam_saturated: Option<bool>,
    #[serde(default)]
    forbidden_beam_pattern: String,
    #[serde(default)]
    beam_width: Option<usize>,
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
