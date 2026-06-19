use crate::*;
use sha2::Digest;

pub(crate) const PACKED_PATTERN_BYTES: usize = 32;
pub(crate) const PATTERN_STORE_ARENA_BYTES: usize = 524_288;
pub(crate) const PATTERN_STORE_CAPACITY: usize = PATTERN_STORE_ARENA_BYTES / PACKED_PATTERN_BYTES;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PackedPattern32 {
    pub signature: u64,
    pub subject_hash: u32,
    pub relation_hash: u32,
    pub object_hash: u32,
    pub route_hash: u32,
    pub group_hash: u32,
    pub boost_i16: i16,
    pub penalty_i16: i16,
    pub accepted: u16,
    pub rejected: u16,
    pub flags: u16,
}

impl PackedPattern32 {
    pub(crate) fn from_memory(memory: &ContinuationMemory) -> Self {
        Self {
            signature: pattern_signature(
                &memory.subject,
                &memory.relation,
                &memory.object,
                &memory.route,
            ),
            subject_hash: hash32(&memory.subject),
            relation_hash: hash32(&memory.relation),
            object_hash: hash32(&memory.object),
            route_hash: hash32(&memory.route),
            group_hash: hash32(&memory.group),
            boost_i16: quantize_weight(memory.boost),
            penalty_i16: quantize_weight(memory.penalty),
            accepted: memory.accepted_count.min(u16::MAX as usize) as u16,
            rejected: memory.rejected_count.min(u16::MAX as usize) as u16,
            flags: match memory.decision.as_str() {
                "accept" => 1,
                "reject" => 2,
                _ => 4,
            },
        }
    }
}

pub(crate) fn pattern_store_cmd(args: PatternStoreArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let out = compact_pattern_store_report(&packet, args.sample);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_pattern_store_text(&out),
        OutputFormat::Md => print_pattern_store_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn pattern_capacity_cmd(args: PatternCapacityArgs) -> Result<u8> {
    let counts = if args.counts.is_empty() {
        vec![1024, 4096, 16_384, 65_536]
    } else {
        args.counts.clone()
    };
    let rows = counts
        .into_iter()
        .map(|count| pattern_capacity_row(count, args.query_patterns.max(1)))
        .collect::<Vec<_>>();
    let out = json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-pattern-capacity",
        "version": "v37-pattern-capacity",
        "packed_pattern_bytes": PACKED_PATTERN_BYTES,
        "pattern_store_arena_bytes": PATTERN_STORE_ARENA_BYTES,
        "pattern_store_capacity": PATTERN_STORE_CAPACITY,
        "rows": rows,
        "read_as": "Capacity estimates when learned continuation patterns stay compact enough for the NANDA-6M pattern arena."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_capacity_text(&out),
        OutputFormat::Md => print_capacity_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn llmwave_cmd(args: LlmwaveArgs) -> Result<u8> {
    let text = if let Some(path) = &args.text_file {
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
    } else {
        args.text.clone()
    };
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &text,
        args.normalize_paths,
    )?;
    let tokens = tokenize_pattern(&text);
    let query = tokens_to_query_triads(&tokens, &args.task_id, &args.domain);
    packet.query = text.clone();
    let memory = normalize_ids(packet.triads.clone(), "m");
    let decode_args = DecodeArgs {
        input: args.input.clone(),
        input_format: args.input_format.clone(),
        task_id: args.task_id.clone(),
        domain: args.domain.clone(),
        query: text.clone(),
        query_file: None,
        query_format: args.input_format.clone(),
        top_k: args.top_k,
        steps: args.steps.clamp(1, 16),
        beam_width: 1,
        adaptive_scoring: false,
        search_top_k: args.search_top_k,
        route_cap: args.route_cap,
        route_triad_cap: args.route_triad_cap,
        group_by: args.group_by.clone(),
        format: OutputFormat::Json,
        normalize_paths: args.normalize_paths,
    };
    let decode = recurrent_decode_report(
        &packet,
        &memory,
        &query,
        "llmwave-auto-query",
        &decode_args,
        decode_args.steps,
    );
    let hrr_binding = hrr_binding_report(&packet, &query, args.top_k);
    let cleanup_memory = cleanup_memory_report(&decode, &packet);
    let attractor_trace = llmwave_attractor_report(&decode);
    let superposition_capacity = superposition_capacity_report(&packet);
    let anti_wave_audit = shortcut_anti_wave_audit(&decode, &packet);
    let feedback_preview = if args.train {
        decode_feedback_preview(&decode, &args.decision, &args.note)
    } else {
        json!({
            "enabled": false,
            "reason": "pass --train to preview continuation feedback"
        })
    };
    let out = json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-mini-loop",
        "version": "v52-read-write-retrieve-loop",
        "text": text,
        "tokens": tokens,
        "encoded_query_triads": query.iter().map(triad_json).collect::<Vec<_>>(),
        "hrr_binding": hrr_binding,
        "cleanup_memory": cleanup_memory,
        "attractor_trace": attractor_trace,
        "superposition_capacity": superposition_capacity,
        "anti_wave_audit": anti_wave_audit,
        "pattern_store": compact_pattern_store_report(&packet, 3),
        "decode": decode,
        "feedback_preview": feedback_preview,
        "read_as": "LLMWave v52 loop: raw text -> wave write/query -> HRR-style binding probe -> structural decode -> cleanup/energy/capacity/anti-wave audit -> optional feedback replay."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_llmwave_text(&out),
        OutputFormat::Md => print_llmwave_md(&out),
    }
    Ok(EXIT_PASS)
}

fn hrr_binding_report(packet: &Packet, query: &[Triad], top_k: usize) -> Value {
    let mut rows = packet
        .triads
        .iter()
        .map(|triad| {
            let subject_bound = bind(
                &vector(&format!("role:{}", norm(&triad.subject_role))),
                &vector(&format!("entity:{}", norm(&triad.subject))),
            );
            let object_bound = bind(
                &vector(&format!("role:{}", norm(&triad.object_role))),
                &vector(&format!("entity:{}", norm(&triad.object))),
            );
            let relation = vector(&format!("relation:{}", norm(&triad.relation)));
            let bound = bind(&bind(&subject_bound, &relation), &object_bound);
            let query_score = if query.is_empty() {
                0.0
            } else {
                query
                    .iter()
                    .map(|q| cosine(&bound, &triad_wave(q)))
                    .fold(f64::NEG_INFINITY, f64::max)
                    .max(0.0)
            };
            let recovered_subject = bind(
                &subject_bound,
                &vector(&format!("role:{}", norm(&triad.subject_role))),
            );
            let recovered_object = bind(
                &object_bound,
                &vector(&format!("role:{}", norm(&triad.object_role))),
            );
            let subject_recovery = cosine(
                &recovered_subject,
                &vector(&format!("entity:{}", norm(&triad.subject))),
            );
            let object_recovery = cosine(
                &recovered_object,
                &vector(&format!("entity:{}", norm(&triad.object))),
            );
            json!({
                "triad": triad.id,
                "subject": triad.subject,
                "relation": triad.relation,
                "object": triad.object,
                "route": triad.route,
                "role_filler_binding": true,
                "query_score": round4(query_score),
                "subject_unbind_cosine": round4(subject_recovery),
                "object_unbind_cosine": round4(object_recovery),
                "state": if subject_recovery > 0.98 && object_recovery > 0.98 { "BINDING_RECOVERED" } else { "BINDING_AMBIGUOUS" }
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["query_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["query_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(top_k.max(1));
    let recovered = rows
        .iter()
        .filter(|row| row["state"].as_str() == Some("BINDING_RECOVERED"))
        .count();
    json!({
        "version": "v47-hrr-binding-sandbox",
        "operation": "role_filler_bind_unbind",
        "records_checked": packet.triads.len(),
        "sample": rows,
        "recovered_in_sample": recovered,
        "state": if recovered > 0 { "HRR_BINDING_VISIBLE" } else { "HRR_BINDING_REVIEW" },
        "read_as": "Experimental HRR-style binding probe: role * filler lanes are bound and then unbound back to subject/object vectors. This is a sandbox signal, not proof by itself."
    })
}

fn cleanup_memory_report(decode: &Value, packet: &Packet) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let mut cleanup_rows = vec![];
    for pattern in patterns.iter().take(5) {
        let mut best = None::<(f64, &Triad)>;
        let p_wave = triad_wave(&Triad {
            id: String::new(),
            subject: pattern["subject"].as_str().unwrap_or("").to_string(),
            relation: pattern["relation"].as_str().unwrap_or("").to_string(),
            object: pattern["object"].as_str().unwrap_or("").to_string(),
            evidence: String::new(),
            confidence: 1.0,
            subject_role: pattern["subject_role"]
                .as_str()
                .unwrap_or("subject")
                .to_string(),
            object_role: pattern["object_role"]
                .as_str()
                .unwrap_or("object")
                .to_string(),
            route: pattern["route"].as_str().unwrap_or("").to_string(),
            group: pattern["group"].as_str().unwrap_or("").to_string(),
        });
        for triad in &packet.triads {
            let score = cosine(&p_wave, &triad_wave(triad));
            if best.as_ref().is_none_or(|(old, _)| score > *old) {
                best = Some((score, triad));
            }
        }
        if let Some((score, triad)) = best {
            let state = if score >= 0.92 {
                "CLEANUP_EXACT"
            } else if score >= 0.45 {
                "CLEANUP_NEAR"
            } else {
                "CLEANUP_AMBIGUOUS"
            };
            cleanup_rows.push(json!({
                "raw_pattern": pattern_label_value(pattern),
                "nearest_memory_triad": triad.id,
                "nearest_pattern": format!("{} -> {} -> {}", triad.subject, triad.relation, triad.object),
                "route": triad.route,
                "score": round4(score),
                "state": state
            }));
        }
    }
    let ambiguous = cleanup_rows
        .iter()
        .filter(|row| row["state"].as_str() == Some("CLEANUP_AMBIGUOUS"))
        .count();
    json!({
        "version": "v48-cleanup-memory",
        "operation": "decoded_pattern_cleanup",
        "items": cleanup_rows,
        "state": if ambiguous == 0 { "CLEANUP_READY" } else { "CLEANUP_WATCH" },
        "read_as": "Cleanup memory maps raw decoded structural patterns back to the nearest known triad and keeps ambiguity visible instead of forcing a clean answer."
    })
}

fn llmwave_attractor_report(decode: &Value) -> Value {
    let steps = decode["recurrent"]["steps"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut last_energy = None::<f64>;
    let mut route_jumps = 0usize;
    let mut last_route = String::new();
    let mut rows = vec![];
    for step in steps {
        let route = step["source_search"]["top_peak"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if !last_route.is_empty() && !route.is_empty() && route != last_route {
            route_jumps += 1;
        }
        if !route.is_empty() {
            last_route = route.clone();
        }
        let margin = step["source_search"]["peak_margin"].as_f64().unwrap_or(0.0);
        let top_score = step["patterns"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item["score"].as_f64())
            .unwrap_or(0.0);
        let anti = step["source_search"]["resonance"]["energy_accounting"]["anti_energy"]
            .as_f64()
            .unwrap_or(0.0);
        let energy = round4((top_score + margin - anti).clamp(-1.0, 2.0));
        let trend = match last_energy {
            None => "START",
            Some(prev) if energy > prev + 0.005 => "IMPROVING",
            Some(prev) if energy < prev - 0.005 => "DROPPING",
            Some(_) => "SATURATING",
        };
        last_energy = Some(energy);
        rows.push(json!({
            "step": step["step"],
            "route_basin": route,
            "top_pattern": step["top_pattern"],
            "energy": energy,
            "trend": trend,
            "decoder_state": step["decoder_state"]
        }));
    }
    let final_state = if rows.is_empty() {
        "NO_ATTRACTOR_TRACE"
    } else if route_jumps == 0
        && rows
            .iter()
            .all(|row| row["trend"].as_str().is_some_and(|x| x != "DROPPING"))
    {
        "ATTRACTOR_STABLE"
    } else if route_jumps > 0 {
        "ATTRACTOR_ROUTE_JUMP"
    } else {
        "ATTRACTOR_REVIEW"
    };
    json!({
        "version": "v49-attractor-energy-trace",
        "route_jumps": route_jumps,
        "steps": rows,
        "state": final_state,
        "read_as": "Energy trace treats recurrent decode as an attractor candidate: stable routes should improve or saturate; route jumps and drops stay review-only."
    })
}

fn superposition_capacity_report(packet: &Packet) -> Value {
    let active_patterns = packet.triads.len() + packet.continuation_memory.len();
    let dimensions = WAVE_DIM;
    let route_count = packet
        .triads
        .iter()
        .filter_map(|triad| (!triad.route.is_empty()).then_some(norm(&triad.route)))
        .collect::<BTreeSet<_>>()
        .len()
        .max(1);
    let load = active_patterns as f64 / dimensions as f64;
    let route_pressure = route_count as f64 / 64.0;
    let crosstalk = round4((load * load + route_pressure * 0.25).min(1.0));
    let state = if active_patterns <= 1024 && crosstalk < 0.35 {
        "CAPACITY_HEALTHY"
    } else if active_patterns <= PATTERN_STORE_CAPACITY && crosstalk < 0.75 {
        "CAPACITY_WATCH"
    } else {
        "FOCUS_REQUIRED"
    };
    json!({
        "version": "v50-superposition-capacity-curve",
        "wave_dim": dimensions,
        "active_patterns": active_patterns,
        "routes": route_count,
        "pattern_store_capacity": PATTERN_STORE_CAPACITY,
        "load_factor": round4(load),
        "estimated_crosstalk": crosstalk,
        "state": state,
        "read_as": "Capacity is treated as a curve, not a slogan: more patterns increase useful compression and harmful crosstalk at the same time."
    })
}

fn shortcut_anti_wave_audit(decode: &Value, packet: &Packet) -> Value {
    let applications = decode["recurrent"]["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|step| {
            step["continuation_training"]["applications"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    let suppressions = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("suppress"))
        .count();
    let reinforcements = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("reinforce"))
        .count();
    let negative_records = packet
        .continuation_memory
        .iter()
        .filter(|item| item.decision == "reject")
        .count();
    let state = if suppressions > 0 {
        "ANTI_WAVE_APPLIED"
    } else if negative_records > 0 {
        "ANTI_WAVE_AVAILABLE_NOT_MATCHED"
    } else {
        "NO_ANTI_WAVE_MEMORY"
    };
    json!({
        "version": "v51-shortcut-specific-anti-wave-audit",
        "negative_records": negative_records,
        "suppressions": suppressions,
        "reinforcements": reinforcements,
        "applications": applications,
        "state": state,
        "read_as": "Anti-wave is accepted only when it is shortcut-specific: rejected local pattern signatures are suppressed without killing the whole topic."
    })
}

fn pattern_label_value(pattern: &Value) -> String {
    format!(
        "{} -> {} -> {}",
        pattern["subject"].as_str().unwrap_or(""),
        pattern["relation"].as_str().unwrap_or(""),
        pattern["object"].as_str().unwrap_or("")
    )
}

pub(crate) fn compact_pattern_store_report(packet: &Packet, sample: usize) -> Value {
    let records = packet
        .continuation_memory
        .iter()
        .map(PackedPattern32::from_memory)
        .collect::<Vec<_>>();
    let used_bytes = records.len() * PACKED_PATTERN_BYTES;
    let capacity = PATTERN_STORE_CAPACITY;
    let accepted = packet
        .continuation_memory
        .iter()
        .filter(|item| item.decision == "accept")
        .count();
    let rejected = packet
        .continuation_memory
        .iter()
        .filter(|item| item.decision == "reject")
        .count();
    json!({
        "core_version": CORE_VERSION,
        "mode": "compact-pattern-store",
        "version": "v35-compact-pattern-store",
        "runtime_version": "v40-6m-pattern-runtime-contract",
        "packed_pattern_bytes": PACKED_PATTERN_BYTES,
        "arena_bytes": PATTERN_STORE_ARENA_BYTES,
        "capacity": capacity,
        "records": records.len(),
        "accepted": accepted,
        "rejected": rejected,
        "used_bytes": used_bytes,
        "remaining_bytes": PATTERN_STORE_ARENA_BYTES.saturating_sub(used_bytes),
        "fits_pattern_arena": records.len() <= capacity,
        "records_sample": packet.continuation_memory.iter().zip(records.iter()).take(sample).map(|(memory, record)| packed_pattern_json(memory, record)).collect::<Vec<_>>()
    })
}

pub(crate) fn apply_compact_pattern_store(
    candidates: &mut [Value],
    query: &[Triad],
    memories: &[ContinuationMemory],
) -> Value {
    let query_terms = query_term_set(query);
    let mut applications = vec![];
    let store_records = memories.len();
    if candidates.is_empty() || memories.is_empty() {
        return json!({
            "version": "v36-pattern-replay",
            "applied": false,
            "store_records": store_records,
            "applications": applications
        });
    }
    for memory in memories {
        let query_ratio = continuation_query_ratio(&query_terms, memory);
        if query_ratio <= 0.0 {
            continue;
        }
        let packed = PackedPattern32::from_memory(memory);
        for candidate in candidates.iter_mut() {
            let candidate_signature = pattern_signature(
                candidate["subject"].as_str().unwrap_or(""),
                candidate["relation"].as_str().unwrap_or(""),
                candidate["object"].as_str().unwrap_or(""),
                candidate["route"].as_str().unwrap_or(""),
            );
            if candidate_signature != packed.signature {
                continue;
            }
            let support_ratio = continuation_candidate_support_ratio(memory, candidate);
            let match_ratio = round4(query_ratio * support_ratio);
            if match_ratio <= 0.0 {
                continue;
            }
            let old_score = candidate["score"].as_f64().unwrap_or(0.0);
            let (delta, action, version) = match memory.decision.as_str() {
                "accept" => (
                    round4(
                        dequantize_weight(packed.boost_i16)
                            * learned_multiplier(memory.accepted_count)
                            * match_ratio,
                    ),
                    "reinforce",
                    "v36-pattern-replay",
                ),
                "reject" => (
                    -round4(
                        dequantize_weight(packed.penalty_i16)
                            * learned_multiplier(memory.rejected_count)
                            * match_ratio,
                    ),
                    "suppress",
                    "v38-negative-continuation-lane",
                ),
                _ => (0.0, "watch", "v36-pattern-replay"),
            };
            if delta == 0.0 {
                continue;
            }
            let new_score = round4((old_score + delta).clamp(-1.0, 1.5));
            if let Some(object) = candidate.as_object_mut() {
                object.insert("score".to_string(), json!(new_score));
                object.insert("raw_decode_score".to_string(), json!(round4(old_score)));
                object.insert("compact_pattern_delta".to_string(), json!(delta));
                object.insert("continuation_memory_delta".to_string(), json!(delta));
                object.insert(
                    "compact_pattern_signature".to_string(),
                    json!(format!("{:016x}", packed.signature)),
                );
            }
            applications.push(json!({
                "version": version,
                "memory": memory.id,
                "action": action,
                "pattern": format!("{} -> {} -> {}", memory.subject, memory.relation, memory.object),
                "signature": format!("{:016x}", packed.signature),
                "delta": delta,
                "match_ratio": match_ratio,
                "query_match_ratio": round4(query_ratio),
                "support_match_ratio": round4(support_ratio),
                "accepted_count": memory.accepted_count,
                "rejected_count": memory.rejected_count,
                "locality": if memory.decision == "reject" { "shortcut-specific-negative-lane" } else { "pattern-specific-positive-replay" },
                "reason": memory.reason
            }));
        }
    }
    json!({
        "version": "v36-pattern-replay",
        "negative_lane_version": "v38-negative-continuation-lane",
        "applied": !applications.is_empty(),
        "store_records": store_records,
        "applications": applications,
        "read_as": "Compact pattern store replays accepted continuations and suppresses rejected local pattern signatures before recurrent selection."
    })
}

fn decode_feedback_preview(decode: &Value, decision: &FeedbackDecision, note: &str) -> Value {
    let decision_label = feedback_decision_label(decision);
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let Some(pattern) = patterns.first() else {
        return json!({
            "enabled": true,
            "decision": decision_label,
            "continuation_memory": []
        });
    };
    let subject = pattern["subject"].as_str().unwrap_or("").to_string();
    let relation = pattern["relation"].as_str().unwrap_or("").to_string();
    let object = pattern["object"].as_str().unwrap_or("").to_string();
    let memory = ContinuationMemory {
        id: format!(
            "cont-{}",
            slug(&format!("{decision_label}-{subject}-{relation}-{object}"))
        ),
        decision: decision_label.to_string(),
        pattern_id: pattern["pattern_id"].as_str().unwrap_or("").to_string(),
        subject,
        relation,
        object,
        route: pattern["route"].as_str().unwrap_or("").to_string(),
        group: pattern["group"].as_str().unwrap_or("").to_string(),
        peak: pattern["peak"].as_str().unwrap_or("").to_string(),
        boost: default_positive_boost(),
        penalty: default_negative_penalty(),
        terms: vec![],
        support_terms: vec![],
        reason: if note.trim().is_empty() {
            "llmwave mini-loop preview".to_string()
        } else {
            note.to_string()
        },
        source_feedback: "llmwave-preview".to_string(),
        observations: 1,
        accepted_count: usize::from(decision_label == "accept"),
        rejected_count: usize::from(decision_label == "reject"),
    };
    json!({
        "enabled": true,
        "decision": decision_label,
        "continuation_memory": [memory],
        "read_as": "Preview only; use nanda-feedback on decode output to persist this memory."
    })
}

fn pattern_capacity_row(count: usize, query_patterns: usize) -> Value {
    let used_bytes = count * PACKED_PATTERN_BYTES;
    let load_factor = count as f64 / PATTERN_STORE_CAPACITY as f64;
    let estimated_collision = (load_factor * load_factor * query_patterns as f64).min(1.0);
    let state = if count <= PATTERN_STORE_CAPACITY {
        "FITS_PATTERN_ARENA"
    } else {
        "FOCUS_OR_SPLIT_PATTERN_STORE"
    };
    json!({
        "patterns": count,
        "used_bytes": used_bytes,
        "fits_pattern_arena": count <= PATTERN_STORE_CAPACITY,
        "state": state,
        "load_factor": round4(load_factor),
        "estimated_collision_pressure": round4(estimated_collision),
        "repair": if count <= PATTERN_STORE_CAPACITY { "none" } else { "keep active continuation focus <= 16k or split by route/group" }
    })
}

fn packed_pattern_json(memory: &ContinuationMemory, record: &PackedPattern32) -> Value {
    json!({
        "id": memory.id,
        "decision": memory.decision,
        "pattern": format!("{} -> {} -> {}", memory.subject, memory.relation, memory.object),
        "route": memory.route,
        "group": memory.group,
        "signature": format!("{:016x}", record.signature),
        "boost_i16": record.boost_i16,
        "penalty_i16": record.penalty_i16,
        "accepted": record.accepted,
        "rejected": record.rejected,
        "flags": record.flags
    })
}

fn continuation_query_ratio(query_terms: &BTreeSet<String>, memory: &ContinuationMemory) -> f64 {
    if memory.terms.is_empty() || query_terms.is_empty() {
        return 1.0;
    }
    let terms = normalized_shortcut_terms(&memory.terms);
    if terms.is_empty() {
        return 1.0;
    }
    terms.intersection(query_terms).count() as f64 / terms.len() as f64
}

fn continuation_candidate_support_ratio(memory: &ContinuationMemory, candidate: &Value) -> f64 {
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

fn pattern_signature(subject: &str, relation: &str, object: &str, route: &str) -> u64 {
    hash64(&format!(
        "{}|{}|{}|{}",
        norm(subject),
        norm(relation),
        norm(object),
        norm(route)
    ))
}

fn hash32(value: &str) -> u32 {
    let digest = Sha256::digest(norm(value).as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

fn hash64(value: &str) -> u64 {
    let digest = Sha256::digest(value.as_bytes());
    u64::from_le_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ])
}

fn quantize_weight(value: f64) -> i16 {
    (value.clamp(-1.0, 1.0) * 10_000.0).round() as i16
}

fn dequantize_weight(value: i16) -> f64 {
    value as f64 / 10_000.0
}

fn learned_multiplier(count: usize) -> f64 {
    (1.0 + count.saturating_sub(1) as f64 * 0.25).min(3.0)
}

fn print_pattern_store_text(out: &Value) {
    println!("mode: compact-pattern-store");
    println!("records: {}", out["records"].as_u64().unwrap_or(0));
    println!("capacity: {}", out["capacity"].as_u64().unwrap_or(0));
    println!(
        "fits: {}",
        out["fits_pattern_arena"].as_bool().unwrap_or(false)
    );
}

fn print_pattern_store_md(out: &Value) {
    println!("# NANDA Pattern Store\n");
    println!("- records: `{}`", out["records"]);
    println!("- capacity: `{}`", out["capacity"]);
    println!("- fits_pattern_arena: `{}`", out["fits_pattern_arena"]);
}

fn print_capacity_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    if let Some(rows) = out["rows"].as_array() {
        for row in rows {
            println!(
                "{} patterns: {}",
                row["patterns"].as_u64().unwrap_or(0),
                row["state"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_capacity_md(out: &Value) {
    println!("# NANDA Pattern Capacity\n");
    if let Some(rows) = out["rows"].as_array() {
        for row in rows {
            println!(
                "- `{}` patterns: `{}`",
                row["patterns"].as_u64().unwrap_or(0),
                row["state"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_llmwave_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "top_pattern: {}",
        out["decode"]["top_pattern"].as_str().unwrap_or("")
    );
    println!(
        "decoder_state: {}",
        out["decode"]["decoder_state"].as_str().unwrap_or("")
    );
}

fn print_llmwave_md(out: &Value) {
    println!("# NANDA LLMWave Mini Loop\n");
    println!("- mode: `{}`", out["mode"].as_str().unwrap_or(""));
    println!(
        "- top_pattern: `{}`",
        out["decode"]["top_pattern"].as_str().unwrap_or("")
    );
    println!(
        "- decoder_state: `{}`",
        out["decode"]["decoder_state"].as_str().unwrap_or("")
    );
}
