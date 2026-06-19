use crate::*;
use serde::Deserialize;
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

#[derive(Clone, Debug, Deserialize)]
struct PatternEvalSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<PatternEvalCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct PatternEvalCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    #[serde(default)]
    query_file: Option<PathBuf>,
    #[serde(default)]
    query: String,
    #[serde(default = "default_pattern_eval_decision")]
    decision: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    expected_baseline_top: String,
    #[serde(default)]
    expected_trained_top: String,
    #[serde(default)]
    expected_action: String,
    #[serde(default)]
    min_abs_delta: Option<f64>,
    #[serde(default)]
    top_k: Option<usize>,
    #[serde(default)]
    steps: Option<usize>,
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

pub(crate) fn pattern_eval_cmd(args: PatternEvalArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.suite)
        .with_context(|| format!("read {}", args.suite.display()))?;
    let suite: PatternEvalSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", args.suite.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow!(
            "nanda pattern-eval requires a suite with at least one case"
        ));
    }
    let base = args.suite.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    let mut changed_top = 0usize;
    let mut reinforced_same_top = 0usize;

    for case in suite.cases {
        let row = run_pattern_eval_case(&args, base, case)?;
        if row["ok"].as_bool().unwrap_or(false) {
            passed += 1;
        }
        if row["learning_changed_top"].as_bool().unwrap_or(false) {
            changed_top += 1;
        }
        if row["learning_reinforced_same_top"]
            .as_bool()
            .unwrap_or(false)
        {
            reinforced_same_top += 1;
        }
        rows.push(row);
    }

    let total = rows.len();
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "pattern-learning-eval-suite",
        "version": "v41-learning-effect-eval",
        "suite": if suite.name.is_empty() { args.suite.display().to_string() } else { suite.name },
        "passed": passed,
        "total": total,
        "accuracy": round4(passed as f64 / total.max(1) as f64),
        "learning_effect": {
            "changed_top": changed_top,
            "reinforced_same_top": reinforced_same_top,
            "changed_or_reinforced": changed_top + reinforced_same_top
        },
        "cases": rows,
        "read_as": "Pattern eval measures whether continuation feedback changes the next-pattern field: reject should suppress a local false continuation; accept should reinforce the selected one."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_pattern_eval_text(&out),
        OutputFormat::Md => print_pattern_eval_md(&out),
    }
    if passed == total {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
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
        "version": "v39-encode-decode-train-loop",
        "text": text,
        "tokens": tokens,
        "encoded_query_triads": query.iter().map(triad_json).collect::<Vec<_>>(),
        "pattern_store": compact_pattern_store_report(&packet, 3),
        "decode": decode,
        "feedback_preview": feedback_preview,
        "read_as": "Mini-loop: raw text -> token wave query -> structural decode -> optional continuation feedback preview."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_llmwave_text(&out),
        OutputFormat::Md => print_llmwave_md(&out),
    }
    Ok(EXIT_PASS)
}

fn run_pattern_eval_case(
    args: &PatternEvalArgs,
    base: &Path,
    case: PatternEvalCase,
) -> Result<Value> {
    let path = resolve_pattern_suite_path(base, &case.path);
    let mut packet = load_packet_auto(
        &path,
        &args.input_format,
        "pattern-eval",
        "general",
        &case.query,
        args.normalize_paths,
    )?;
    let memory = normalize_ids(packet.triads.clone(), "m");
    let mut query_packet = if let Some(query_file) = &case.query_file {
        load_packet_auto(
            &resolve_pattern_suite_path(base, query_file),
            &args.input_format,
            "pattern-eval",
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
        task_id: "pattern-eval".to_string(),
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
    let baseline = recurrent_decode_report(
        &packet,
        &memory,
        &query,
        query_source,
        &decode_args,
        decode_args.steps,
    );
    let decision = normalize_pattern_eval_decision(&case.decision)?;
    let note = if case.note.trim().is_empty() {
        format!("pattern eval {decision}")
    } else {
        case.note.clone()
    };
    let source_feedback = format!("pattern-eval:{}", case.id);
    let mut trained_packet = packet.clone();
    let mut continuation_memory =
        continuation_memory_from_decode(&baseline, &decision, &note, source_feedback);
    trained_packet
        .continuation_memory
        .append(&mut continuation_memory);
    trained_packet.continuation_memory =
        merge_continuation_memory(trained_packet.continuation_memory);
    let trained = recurrent_decode_report(
        &trained_packet,
        &memory,
        &query,
        query_source,
        &decode_args,
        decode_args.steps,
    );

    let baseline_top = baseline["top_pattern"].as_str().unwrap_or("").to_string();
    let trained_top = trained["top_pattern"].as_str().unwrap_or("").to_string();
    let baseline_score = pattern_score(&baseline, &baseline_top);
    let trained_score = pattern_score(&trained, &baseline_top);
    let applications = trained["continuation_training"]["applications"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let first_application = applications.first().cloned().unwrap_or_else(|| json!({}));
    let actual_action = first_application["action"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let delta = first_application["delta"].as_f64().unwrap_or(0.0);
    let training_applied = trained["continuation_training"]["applied"]
        .as_bool()
        .unwrap_or(false);
    let learning_changed_top = !baseline_top.is_empty() && baseline_top != trained_top;
    let learning_reinforced_same_top = baseline_top == trained_top && delta > 0.0;
    let baseline_ok =
        case.expected_baseline_top.is_empty() || baseline_top == case.expected_baseline_top;
    let trained_ok =
        case.expected_trained_top.is_empty() || trained_top == case.expected_trained_top;
    let action_ok = case.expected_action.is_empty() || actual_action == case.expected_action;
    let delta_ok = delta.abs() >= case.min_abs_delta.unwrap_or(0.0001);
    let ok = training_applied && baseline_ok && trained_ok && action_ok && delta_ok;

    Ok(json!({
        "id": if case.id.is_empty() { path.display().to_string() } else { case.id },
        "case": path.display().to_string(),
        "decision": decision,
        "expected_baseline_top": case.expected_baseline_top,
        "actual_baseline_top": baseline_top,
        "baseline_top_score": round4(baseline_score),
        "expected_trained_top": case.expected_trained_top,
        "actual_trained_top": trained_top,
        "trained_baseline_pattern_score": round4(trained_score),
        "expected_action": case.expected_action,
        "actual_action": actual_action,
        "delta": round4(delta),
        "min_abs_delta": case.min_abs_delta.unwrap_or(0.0001),
        "training_applied": training_applied,
        "learning_changed_top": learning_changed_top,
        "learning_reinforced_same_top": learning_reinforced_same_top,
        "continuation_records": trained_packet.continuation_memory.len(),
        "first_application": first_application,
        "checks": {
            "baseline_top": baseline_ok,
            "trained_top": trained_ok,
            "action": action_ok,
            "delta": delta_ok
        },
        "ok": ok
    }))
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

fn default_pattern_eval_decision() -> String {
    "accept".to_string()
}

fn resolve_pattern_suite_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn normalize_pattern_eval_decision(decision: &str) -> Result<String> {
    match norm(decision).as_str() {
        "" | "accept" => Ok("accept".to_string()),
        "reject" => Ok("reject".to_string()),
        "watch" => Ok("watch".to_string()),
        other => Err(anyhow!(
            "unsupported pattern eval decision `{other}`; expected accept, reject, or watch"
        )),
    }
}

fn pattern_score(decode: &Value, pattern_name: &str) -> f64 {
    decode["patterns"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|pattern| {
            let name = format!(
                "{} -> {} -> {}",
                pattern["subject"].as_str().unwrap_or(""),
                pattern["relation"].as_str().unwrap_or(""),
                pattern["object"].as_str().unwrap_or("")
            );
            name == pattern_name
        })
        .and_then(|pattern| pattern["score"].as_f64())
        .unwrap_or(0.0)
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

fn print_pattern_eval_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "{}: {} {} -> {} delta={}",
                case["id"].as_str().unwrap_or("case"),
                if case["ok"].as_bool().unwrap_or(false) {
                    "PASS"
                } else {
                    "VETO"
                },
                case["actual_baseline_top"].as_str().unwrap_or(""),
                case["actual_trained_top"].as_str().unwrap_or(""),
                case["delta"].as_f64().unwrap_or(0.0)
            );
        }
    }
}

fn print_pattern_eval_md(out: &Value) {
    println!("# NANDA Pattern Learning Eval\n");
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- `{}`: `{}` `{}` -> `{}` delta `{}`",
                case["id"].as_str().unwrap_or("case"),
                if case["ok"].as_bool().unwrap_or(false) {
                    "PASS"
                } else {
                    "VETO"
                },
                case["actual_baseline_top"].as_str().unwrap_or(""),
                case["actual_trained_top"].as_str().unwrap_or(""),
                case["delta"].as_f64().unwrap_or(0.0)
            );
        }
    }
}
