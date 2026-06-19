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
    let packed_hrr_runtime = packed_hrr_runtime_report(&hrr_binding, &packet);
    let cleanup_dictionary = cleanup_dictionary_report(&cleanup_memory, &packet);
    let anti_wave_locality = anti_wave_locality_report(&anti_wave_audit, &decode);
    let capacity_curve = capacity_curve_report(&superposition_capacity, &packet);
    let packed_hot_cycle = llmwave_hot_cycle_report(
        &packed_hrr_runtime,
        &cleanup_dictionary,
        &capacity_curve,
        &anti_wave_locality,
    );
    let proof_summary = llmwave_proof_summary(
        &hrr_binding,
        &cleanup_memory,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &decode,
    );
    let public_demo = llmwave_public_demo_report(&proof_summary, &decode, &text);
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
        "version": "v60-public-demo-packet",
        "text": text,
        "tokens": tokens,
        "encoded_query_triads": query.iter().map(triad_json).collect::<Vec<_>>(),
        "hrr_binding": hrr_binding,
        "cleanup_memory": cleanup_memory,
        "attractor_trace": attractor_trace,
        "superposition_capacity": superposition_capacity,
        "anti_wave_audit": anti_wave_audit,
        "packed_hrr_runtime": packed_hrr_runtime,
        "cleanup_dictionary": cleanup_dictionary,
        "anti_wave_locality": anti_wave_locality,
        "capacity_curve": capacity_curve,
        "packed_hot_cycle": packed_hot_cycle,
        "proof_summary": proof_summary,
        "public_demo": public_demo,
        "pattern_store": compact_pattern_store_report(&packet, 3),
        "decode": decode,
        "feedback_preview": feedback_preview,
        "read_as": "LLMWave v60 loop: raw text -> wave write/query -> HRR binding -> structural decode -> cleanup/energy/capacity/anti-wave audit -> packed readiness -> proof summary -> demo card."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_llmwave_text(&out),
        OutputFormat::Md => print_llmwave_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn llmwave_eval_cmd(args: LlmwaveEvalArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.suite)
        .with_context(|| format!("read {}", args.suite.display()))?;
    let suite: LlmwaveEvalSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", args.suite.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow!("nanda llmwave-eval requires at least one case"));
    }
    let base = args.suite.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    for case in suite.cases {
        let row = run_llmwave_eval_case(&args, base, case)?;
        if row["ok"].as_bool().unwrap_or(false) {
            passed += 1;
        }
        rows.push(row);
    }
    let total = rows.len();
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "llmwave-eval-suite",
        "version": "v53-llmwave-proof-suite",
        "suite": if suite.name.is_empty() { args.suite.display().to_string() } else { suite.name },
        "passed": passed,
        "total": total,
        "accuracy": round4(passed as f64 / total.max(1) as f64),
        "cases": rows,
        "read_as": "LLMWave eval verifies the full v60 read/write/retrieve proof packet, not only the top decoded pattern."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_llmwave_eval_text(&out),
        OutputFormat::Md => print_llmwave_eval_md(&out),
    }
    Ok(if passed == total {
        EXIT_PASS
    } else {
        EXIT_VETO
    })
}

pub(crate) fn demo_cmd(args: DemoArgs) -> Result<u8> {
    if let Some(suite_path) = &args.suite {
        let text = fs::read_to_string(suite_path)
            .with_context(|| format!("read {}", suite_path.display()))?;
        let suite: DemoSuite = serde_json::from_str(&text)
            .with_context(|| format!("parse {}", suite_path.display()))?;
        if suite.cases.is_empty() {
            return Err(anyhow!("nanda demo --suite requires at least one case"));
        }
        let base = suite_path.parent().unwrap_or_else(|| Path::new("."));
        let mut rows = vec![];
        let mut passed = 0usize;
        for case in suite.cases {
            let row = run_demo_case(&args, base, case)?;
            if row["ok"].as_bool().unwrap_or(false) {
                passed += 1;
            }
            rows.push(row);
        }
        let total = rows.len();
        let out = json!({
            "core_version": CORE_VERSION,
            "mode": "llmwave-demo-suite",
            "version": "v61-demo-weak-spot-surface",
            "suite": if suite.name.is_empty() { suite_path.display().to_string() } else { suite.name },
            "passed": passed,
            "total": total,
            "accuracy": round4(passed as f64 / total.max(1) as f64),
            "cases": rows,
            "read_as": "Demo suite checks the human/agent-facing LLMWave surface: ready, anti-wave, and review cases must all be legible."
        });
        match args.format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
            OutputFormat::Text => print_demo_suite_text(&out),
            OutputFormat::Md => print_demo_suite_md(&out),
        }
        return Ok(if passed == total {
            EXIT_PASS
        } else {
            EXIT_VETO
        });
    }

    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow!("nanda demo requires an input packet or --suite"))?;
    let text = demo_text(&args)?;
    let mut packet = load_packet_auto(
        input,
        &args.input_format,
        "demo",
        "general",
        &text,
        args.normalize_paths,
    )?;
    let eval_args = demo_eval_args(&args);
    if args.train {
        let case = LlmwaveEvalCase {
            id: "demo-feedback".to_string(),
            path: input.clone(),
            text: text.clone(),
            feedback_decision: feedback_decision_label(&args.decision).to_string(),
            note: "demo feedback preview".to_string(),
            expected_top_pattern: String::new(),
            expected_hrr_state: String::new(),
            expected_cleanup_state: String::new(),
            expected_attractor_state: String::new(),
            expected_capacity_state: String::new(),
            expected_anti_wave_state: String::new(),
            expected_proof_state: String::new(),
            expected_demo_state: String::new(),
        };
        packet = inject_llmwave_feedback(packet, &text, &case, &eval_args)?;
    }
    let report = llmwave_report_from_packet(&packet, input, &text, &eval_args);
    let demo = demo_surface_report("single", input, &text, &report);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&demo)?),
        OutputFormat::Text => print_demo_text(&demo),
        OutputFormat::Md => print_demo_md(&demo),
    }
    Ok(if demo["state"].as_str() == Some("PUBLIC_DEMO_READY") {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

fn run_demo_case(args: &DemoArgs, base: &Path, case: DemoCase) -> Result<Value> {
    let path = resolve_llmwave_suite_path(base, &case.path);
    let text = if case.text.trim().is_empty() {
        args.text.clone()
    } else {
        case.text.clone()
    };
    let mut packet = load_packet_auto(
        &path,
        &args.input_format,
        "demo-suite",
        "general",
        &text,
        args.normalize_paths,
    )?;
    let eval_args = demo_eval_args(args);
    if !case.feedback_decision.trim().is_empty() {
        let eval_case = LlmwaveEvalCase {
            id: case.id.clone(),
            path: path.clone(),
            text: text.clone(),
            feedback_decision: case.feedback_decision.clone(),
            note: case.note.clone(),
            expected_top_pattern: String::new(),
            expected_hrr_state: String::new(),
            expected_cleanup_state: String::new(),
            expected_attractor_state: String::new(),
            expected_capacity_state: String::new(),
            expected_anti_wave_state: String::new(),
            expected_proof_state: String::new(),
            expected_demo_state: String::new(),
        };
        packet = inject_llmwave_feedback(packet, &text, &eval_case, &eval_args)?;
    }
    let report = llmwave_report_from_packet(&packet, &path, &text, &eval_args);
    let demo = demo_surface_report(&case.id, &path, &text, &report);
    let expected_state_ok = case.expected_state.is_empty()
        || demo["state"].as_str() == Some(case.expected_state.as_str());
    let expected_weak_ok = case
        .expected_weak_spots
        .is_none_or(|expected| demo["weak_spots"].as_array().map_or(0, Vec::len) == expected);
    let expected_anti_ok = case.expected_anti_wave_state.is_empty()
        || demo["signals"]["anti_wave"].as_str() == Some(case.expected_anti_wave_state.as_str());
    let ok = expected_state_ok && expected_weak_ok && expected_anti_ok;
    Ok(json!({
        "id": if case.id.is_empty() { path.display().to_string() } else { case.id },
        "case": path.display().to_string(),
        "state": demo["state"],
        "top_pattern": demo["top_pattern"],
        "weak_spots": demo["weak_spots"],
        "signals": demo["signals"],
        "expected_state": case.expected_state,
        "checks": {
            "state": expected_state_ok,
            "weak_spots": expected_weak_ok,
            "anti_wave": expected_anti_ok
        },
        "ok": ok
    }))
}

fn run_llmwave_eval_case(
    args: &LlmwaveEvalArgs,
    base: &Path,
    case: LlmwaveEvalCase,
) -> Result<Value> {
    let path = resolve_llmwave_suite_path(base, &case.path);
    let mut packet = load_packet_auto(
        &path,
        &args.input_format,
        "llmwave-eval",
        "general",
        &case.text,
        args.normalize_paths,
    )?;
    let text = if case.text.trim().is_empty() {
        packet.query.clone()
    } else {
        case.text.clone()
    };
    if !case.feedback_decision.trim().is_empty() {
        packet = inject_llmwave_feedback(packet, &text, &case, args)?;
    }
    let report = llmwave_report_from_packet(&packet, &path, &text, args);
    let checks = json!({
        "top_pattern": case.expected_top_pattern.is_empty() || report["decode"]["top_pattern"].as_str() == Some(case.expected_top_pattern.as_str()),
        "hrr": case.expected_hrr_state.is_empty() || report["hrr_binding"]["state"].as_str() == Some(case.expected_hrr_state.as_str()),
        "cleanup": case.expected_cleanup_state.is_empty() || report["cleanup_memory"]["state"].as_str() == Some(case.expected_cleanup_state.as_str()),
        "attractor": case.expected_attractor_state.is_empty() || report["attractor_trace"]["state"].as_str() == Some(case.expected_attractor_state.as_str()),
        "capacity": case.expected_capacity_state.is_empty() || report["superposition_capacity"]["state"].as_str() == Some(case.expected_capacity_state.as_str()),
        "anti_wave": case.expected_anti_wave_state.is_empty() || report["anti_wave_audit"]["state"].as_str() == Some(case.expected_anti_wave_state.as_str()),
        "proof": case.expected_proof_state.is_empty() || report["proof_summary"]["state"].as_str() == Some(case.expected_proof_state.as_str()),
        "demo": case.expected_demo_state.is_empty() || report["public_demo"]["state"].as_str() == Some(case.expected_demo_state.as_str())
    });
    let ok = checks
        .as_object()
        .is_some_and(|items| items.values().all(|value| value.as_bool().unwrap_or(false)));
    Ok(json!({
        "id": if case.id.is_empty() { path.display().to_string() } else { case.id },
        "case": path.display().to_string(),
        "text": text,
        "feedback_decision": case.feedback_decision,
        "expected_top_pattern": case.expected_top_pattern,
        "actual_top_pattern": report["decode"]["top_pattern"],
        "states": {
            "hrr": report["hrr_binding"]["state"],
            "cleanup": report["cleanup_memory"]["state"],
            "attractor": report["attractor_trace"]["state"],
            "capacity": report["superposition_capacity"]["state"],
            "anti_wave": report["anti_wave_audit"]["state"],
            "packed_hrr": report["packed_hrr_runtime"]["state"],
            "cleanup_dictionary": report["cleanup_dictionary"]["state"],
            "anti_wave_locality": report["anti_wave_locality"]["state"],
            "capacity_curve": report["capacity_curve"]["state"],
            "hot_cycle": report["packed_hot_cycle"]["state"],
            "proof": report["proof_summary"]["state"],
            "demo": report["public_demo"]["state"]
        },
        "checks": checks,
        "ok": ok
    }))
}

fn llmwave_report_from_packet(
    packet: &Packet,
    input: &Path,
    text: &str,
    args: &LlmwaveEvalArgs,
) -> Value {
    let mut packet = packet.clone();
    let tokens = tokenize_pattern(text);
    let query = tokens_to_query_triads(&tokens, "llmwave-eval", "general");
    packet.query = text.to_string();
    let memory = normalize_ids(packet.triads.clone(), "m");
    let decode_args = DecodeArgs {
        input: input.to_path_buf(),
        input_format: args.input_format.clone(),
        task_id: "llmwave-eval".to_string(),
        domain: "general".to_string(),
        query: text.to_string(),
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
        "llmwave-eval-query",
        &decode_args,
        decode_args.steps,
    );
    let hrr_binding = hrr_binding_report(&packet, &query, args.top_k);
    let cleanup_memory = cleanup_memory_report(&decode, &packet);
    let attractor_trace = llmwave_attractor_report(&decode);
    let superposition_capacity = superposition_capacity_report(&packet);
    let anti_wave_audit = shortcut_anti_wave_audit(&decode, &packet);
    let packed_hrr_runtime = packed_hrr_runtime_report(&hrr_binding, &packet);
    let cleanup_dictionary = cleanup_dictionary_report(&cleanup_memory, &packet);
    let anti_wave_locality = anti_wave_locality_report(&anti_wave_audit, &decode);
    let capacity_curve = capacity_curve_report(&superposition_capacity, &packet);
    let packed_hot_cycle = llmwave_hot_cycle_report(
        &packed_hrr_runtime,
        &cleanup_dictionary,
        &capacity_curve,
        &anti_wave_locality,
    );
    let proof_summary = llmwave_proof_summary(
        &hrr_binding,
        &cleanup_memory,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &decode,
    );
    let public_demo = llmwave_public_demo_report(&proof_summary, &decode, text);
    json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-eval-case-report",
        "version": "v60-public-demo-packet",
        "hrr_binding": hrr_binding,
        "cleanup_memory": cleanup_memory,
        "attractor_trace": attractor_trace,
        "superposition_capacity": superposition_capacity,
        "anti_wave_audit": anti_wave_audit,
        "packed_hrr_runtime": packed_hrr_runtime,
        "cleanup_dictionary": cleanup_dictionary,
        "anti_wave_locality": anti_wave_locality,
        "capacity_curve": capacity_curve,
        "packed_hot_cycle": packed_hot_cycle,
        "proof_summary": proof_summary,
        "public_demo": public_demo,
        "decode": decode
    })
}

fn inject_llmwave_feedback(
    mut packet: Packet,
    text: &str,
    case: &LlmwaveEvalCase,
    args: &LlmwaveEvalArgs,
) -> Result<Packet> {
    let report = llmwave_report_from_packet(&packet, &case.path, text, args);
    let decision = normalize_llmwave_feedback_decision(&case.feedback_decision)?;
    let note = if case.note.trim().is_empty() {
        format!("llmwave eval {decision}")
    } else {
        case.note.clone()
    };
    let mut memories =
        continuation_memory_from_decode(&report["decode"], &decision, &note, "llmwave-eval".into());
    packet.continuation_memory.append(&mut memories);
    packet.continuation_memory = merge_continuation_memory(packet.continuation_memory);
    Ok(packet)
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

fn packed_hrr_runtime_report(hrr: &Value, packet: &Packet) -> Value {
    let records = hrr["sample"].as_array().map_or(0, Vec::len);
    let bytes_per_lane = 64usize;
    let bytes = records * bytes_per_lane;
    let recovered = hrr["recovered_in_sample"].as_u64().unwrap_or(0);
    json!({
        "version": "v54-packed-hrr-lanes",
        "state": if recovered > 0 { "PACKED_HRR_READY" } else { "PACKED_HRR_REVIEW" },
        "records": records,
        "source_triads": packet.triads.len(),
        "bytes_per_lane": bytes_per_lane,
        "bytes": bytes,
        "fits_pattern_arena": bytes <= PATTERN_STORE_ARENA_BYTES,
        "contract": "role/filler binding probes are representable as fixed 64-byte hot lanes",
        "read_as": "v54 moves HRR binding from a visual report toward a packed-lane runtime contract."
    })
}

fn cleanup_dictionary_report(cleanup: &Value, packet: &Packet) -> Value {
    let items = cleanup["items"].as_array().cloned().unwrap_or_default();
    let exact = items
        .iter()
        .filter(|item| item["state"].as_str() == Some("CLEANUP_EXACT"))
        .count();
    let near = items
        .iter()
        .filter(|item| item["state"].as_str() == Some("CLEANUP_NEAR"))
        .count();
    let ambiguous = items
        .iter()
        .filter(|item| item["state"].as_str() == Some("CLEANUP_AMBIGUOUS"))
        .count();
    let state = if ambiguous > 0 {
        "CLEANUP_DICTIONARY_WATCH"
    } else if exact + near > 0 {
        "CLEANUP_DICTIONARY_READY"
    } else {
        "CLEANUP_DICTIONARY_EMPTY"
    };
    json!({
        "version": "v55-cleanup-dictionary-thresholds",
        "state": state,
        "memory_triads": packet.triads.len(),
        "exact": exact,
        "near": near,
        "ambiguous": ambiguous,
        "exact_threshold": 0.92,
        "near_threshold": 0.45,
        "read_as": "v55 makes cleanup explicit: exact and near matches can support proof; ambiguous cleanup keeps the result under WATCH."
    })
}

fn anti_wave_locality_report(anti_wave: &Value, decode: &Value) -> Value {
    let applications = anti_wave["applications"]
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
    let top_after = decode["top_pattern"].as_str().unwrap_or("");
    let state = if suppressions > 0 && !top_after.is_empty() {
        "ANTI_WAVE_LOCAL"
    } else if anti_wave["negative_records"].as_u64().unwrap_or(0) > 0 {
        "ANTI_WAVE_NOT_TRIGGERED"
    } else {
        "ANTI_WAVE_NO_MEMORY"
    };
    json!({
        "version": "v56-anti-wave-locality-fixture",
        "state": state,
        "suppressions": suppressions,
        "reinforcements": reinforcements,
        "top_after": top_after,
        "kept_decode_alive": !top_after.is_empty(),
        "read_as": "v56 checks that anti-wave suppresses a shortcut-shaped continuation without destroying the whole decode surface."
    })
}

fn capacity_curve_report(capacity: &Value, packet: &Packet) -> Value {
    let active = capacity["active_patterns"].as_u64().unwrap_or(0) as usize;
    let direct_table_bytes = active * 96;
    let packed_wave_bytes = active * PACKED_PATTERN_BYTES;
    let compression_ratio = if packed_wave_bytes == 0 {
        0.0
    } else {
        direct_table_bytes as f64 / packed_wave_bytes as f64
    };
    json!({
        "version": "v57-superposition-capacity-baseline",
        "state": capacity["state"],
        "active_patterns": active,
        "routes": capacity["routes"],
        "direct_table_bytes_estimate": direct_table_bytes,
        "packed_wave_bytes_estimate": packed_wave_bytes,
        "compression_ratio_vs_direct_table": round4(compression_ratio),
        "continuation_records": packet.continuation_memory.len(),
        "read_as": "v57 compares the packed wave representation with a simple direct-table estimate before making capacity claims."
    })
}

fn llmwave_hot_cycle_report(
    packed_hrr: &Value,
    cleanup_dictionary: &Value,
    capacity_curve: &Value,
    anti_wave_locality: &Value,
) -> Value {
    let ready = packed_hrr["state"].as_str() == Some("PACKED_HRR_READY")
        && matches!(
            cleanup_dictionary["state"].as_str().unwrap_or(""),
            "CLEANUP_DICTIONARY_READY" | "CLEANUP_DICTIONARY_EMPTY"
        )
        && capacity_curve["state"].as_str() != Some("FOCUS_REQUIRED")
        && anti_wave_locality["kept_decode_alive"]
            .as_bool()
            .unwrap_or(true);
    json!({
        "version": "v58-packed-hot-cycle-bridge",
        "state": if ready { "LLMWAVE_HOT_READY" } else { "LLMWAVE_HOT_WATCH" },
        "packed_hrr_state": packed_hrr["state"],
        "cleanup_dictionary_state": cleanup_dictionary["state"],
        "capacity_state": capacity_curve["state"],
        "anti_wave_locality_state": anti_wave_locality["state"],
        "read_as": "v58 is the cold-to-hot bridge: v52 reports are collapsed into a single packed runtime readiness state."
    })
}

fn llmwave_proof_summary(
    hrr: &Value,
    cleanup: &Value,
    attractor: &Value,
    capacity: &Value,
    anti_wave: &Value,
    hot_cycle: &Value,
    decode: &Value,
) -> Value {
    let mut blockers = vec![];
    if hrr["state"].as_str() != Some("HRR_BINDING_VISIBLE") {
        blockers.push("hrr_binding");
    }
    if cleanup["state"].as_str() == Some("CLEANUP_WATCH") {
        blockers.push("cleanup_memory");
    }
    if !matches!(
        attractor["state"].as_str().unwrap_or(""),
        "ATTRACTOR_STABLE" | "NO_ATTRACTOR_TRACE"
    ) {
        blockers.push("attractor_trace");
    }
    if capacity["state"].as_str() == Some("FOCUS_REQUIRED") {
        blockers.push("capacity");
    }
    if hot_cycle["state"].as_str() != Some("LLMWAVE_HOT_READY") {
        blockers.push("hot_cycle");
    }
    let top_pattern = decode["top_pattern"].as_str().unwrap_or("");
    if top_pattern.is_empty() {
        blockers.push("decode");
    }
    let state = if blockers.is_empty() {
        "LLMWAVE_PROOF_READY"
    } else {
        "LLMWAVE_PROOF_WATCH"
    };
    json!({
        "version": "v59-llmwave-proof-command-contract",
        "state": state,
        "answer_ready": state == "LLMWAVE_PROOF_READY",
        "top_pattern": top_pattern,
        "anti_wave_state": anti_wave["state"],
        "blockers": blockers,
        "read_as": "v59 is the proof contract: a decoded pattern is useful only when binding, cleanup, attractor, capacity, and hot readiness agree."
    })
}

fn llmwave_public_demo_report(proof: &Value, decode: &Value, text: &str) -> Value {
    let top = decode["top_pattern"].as_str().unwrap_or("");
    let state = if proof["state"].as_str() == Some("LLMWAVE_PROOF_READY") {
        "PUBLIC_DEMO_READY"
    } else {
        "PUBLIC_DEMO_REVIEW"
    };
    json!({
        "version": "v60-public-demo-packet",
        "state": state,
        "input": text,
        "demo_claim": if state == "PUBLIC_DEMO_READY" { "LLMWave found a stable structural continuation and exposed its proof signals." } else { "LLMWave produced a reviewable structural continuation, but proof blockers remain." },
        "top_pattern": top,
        "safe_claim": "This is structural wave retrieval/proof, not standalone natural-language understanding.",
        "read_as": "v60 is the public demo surface: one compact packet that can be shown without hiding proof state."
    })
}

#[derive(Clone, Debug, Deserialize)]
struct LlmwaveEvalSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<LlmwaveEvalCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct LlmwaveEvalCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    #[serde(default)]
    text: String,
    #[serde(default)]
    feedback_decision: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    expected_top_pattern: String,
    #[serde(default)]
    expected_hrr_state: String,
    #[serde(default)]
    expected_cleanup_state: String,
    #[serde(default)]
    expected_attractor_state: String,
    #[serde(default)]
    expected_capacity_state: String,
    #[serde(default)]
    expected_anti_wave_state: String,
    #[serde(default)]
    expected_proof_state: String,
    #[serde(default)]
    expected_demo_state: String,
}

fn resolve_llmwave_suite_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn normalize_llmwave_feedback_decision(decision: &str) -> Result<String> {
    match norm(decision).as_str() {
        "" => Ok(String::new()),
        "accept" | "accepted" => Ok("accept".to_string()),
        "reject" | "rejected" => Ok("reject".to_string()),
        "watch" => Ok("watch".to_string()),
        other => Err(anyhow!("unsupported llmwave feedback decision: {other}")),
    }
}

#[derive(Clone, Debug, Deserialize)]
struct DemoSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<DemoCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct DemoCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    #[serde(default)]
    text: String,
    #[serde(default)]
    feedback_decision: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    expected_state: String,
    #[serde(default)]
    expected_weak_spots: Option<usize>,
    #[serde(default)]
    expected_anti_wave_state: String,
}

fn demo_text(args: &DemoArgs) -> Result<String> {
    if let Some(path) = &args.text_file {
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
    } else {
        Ok(args.text.clone())
    }
}

fn demo_eval_args(args: &DemoArgs) -> LlmwaveEvalArgs {
    LlmwaveEvalArgs {
        suite: PathBuf::new(),
        input_format: args.input_format.clone(),
        top_k: args.top_k,
        steps: args.steps,
        search_top_k: args.search_top_k,
        route_cap: args.route_cap,
        route_triad_cap: args.route_triad_cap,
        group_by: args.group_by.clone(),
        format: OutputFormat::Json,
        normalize_paths: args.normalize_paths,
    }
}

fn demo_surface_report(id: &str, path: &Path, text: &str, report: &Value) -> Value {
    let proof_state = report["proof_summary"]["state"].as_str().unwrap_or("");
    let demo_state = report["public_demo"]["state"].as_str().unwrap_or("");
    let top_pattern = report["decode"]["top_pattern"].as_str().unwrap_or("");
    let signals = json!({
        "hrr": report["hrr_binding"]["state"],
        "cleanup": report["cleanup_memory"]["state"],
        "attractor": report["attractor_trace"]["state"],
        "capacity": report["superposition_capacity"]["state"],
        "anti_wave": report["anti_wave_audit"]["state"],
        "packed_hrr": report["packed_hrr_runtime"]["state"],
        "cleanup_dictionary": report["cleanup_dictionary"]["state"],
        "anti_wave_locality": report["anti_wave_locality"]["state"],
        "capacity_curve": report["capacity_curve"]["state"],
        "hot_cycle": report["packed_hot_cycle"]["state"],
        "proof": proof_state,
        "demo": demo_state
    });
    let weak_spots = demo_weak_spots(report);
    let state = if demo_state == "PUBLIC_DEMO_READY" && weak_spots.is_empty() {
        "PUBLIC_DEMO_READY"
    } else {
        "PUBLIC_DEMO_REVIEW"
    };
    json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-demo",
        "version": "v61-demo-weak-spot-surface",
        "id": id,
        "input_packet": path.display().to_string(),
        "text": text,
        "state": state,
        "top_pattern": top_pattern,
        "proof_state": proof_state,
        "signals": signals,
        "weak_spots": weak_spots,
        "safe_claim": if state == "PUBLIC_DEMO_READY" {
            "LLMWave found a stable structural continuation and exposed its proof signals."
        } else {
            "LLMWave produced a reviewable structural continuation, but weak spots remain."
        },
        "boundary": "This is structural wave retrieval/proof, not standalone natural-language understanding.",
        "raw_public_demo": report["public_demo"],
        "read_as": "Demo mode is the weak-spot surface: it compresses v60 JSON into a human/agent readable proof dashboard."
    })
}

fn demo_weak_spots(report: &Value) -> Vec<Value> {
    let mut weak = vec![];
    push_weak(
        &mut weak,
        "hrr",
        report["hrr_binding"]["state"].as_str().unwrap_or(""),
        &["HRR_BINDING_VISIBLE"],
        "HRR binding did not visibly recover role/filler lanes.",
    );
    push_weak(
        &mut weak,
        "cleanup",
        report["cleanup_memory"]["state"].as_str().unwrap_or(""),
        &["CLEANUP_READY"],
        "Cleanup memory is ambiguous or empty for decoded patterns.",
    );
    push_weak(
        &mut weak,
        "attractor",
        report["attractor_trace"]["state"].as_str().unwrap_or(""),
        &["ATTRACTOR_STABLE", "NO_ATTRACTOR_TRACE"],
        "Decode trajectory jumps route or loses energy.",
    );
    push_weak(
        &mut weak,
        "capacity",
        report["superposition_capacity"]["state"]
            .as_str()
            .unwrap_or(""),
        &["CAPACITY_HEALTHY", "CAPACITY_WATCH"],
        "Superposition pressure requires focus or split.",
    );
    push_weak(
        &mut weak,
        "hot_cycle",
        report["packed_hot_cycle"]["state"].as_str().unwrap_or(""),
        &["LLMWAVE_HOT_READY"],
        "Packed hot-cycle readiness is not proven.",
    );
    push_weak(
        &mut weak,
        "proof",
        report["proof_summary"]["state"].as_str().unwrap_or(""),
        &["LLMWAVE_PROOF_READY"],
        "Proof summary is not answer-ready.",
    );
    if report["decode"]["top_pattern"]
        .as_str()
        .unwrap_or("")
        .is_empty()
    {
        weak.push(json!({
            "signal": "decode",
            "state": "NO_PATTERN",
            "reason": "No structural continuation was decoded."
        }));
    }
    weak
}

fn push_weak(weak: &mut Vec<Value>, signal: &str, state: &str, allowed: &[&str], reason: &str) {
    if !allowed.contains(&state) {
        weak.push(json!({
            "signal": signal,
            "state": state,
            "reason": reason
        }));
    }
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

fn print_llmwave_eval_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {}: {} top={}",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_top_pattern"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_llmwave_eval_md(out: &Value) {
    println!("# NANDA LLMWave Eval\n");
    println!("- mode: `{}`", out["mode"].as_str().unwrap_or(""));
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- `{}`: `{}` top=`{}`",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_top_pattern"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_demo_text(out: &Value) {
    println!("NANDA LLMWave Demo");
    println!();
    println!("state: {}", out["state"].as_str().unwrap_or(""));
    println!("input: {}", out["text"].as_str().unwrap_or(""));
    println!("top_pattern: {}", out["top_pattern"].as_str().unwrap_or(""));
    println!("proof: {}", out["proof_state"].as_str().unwrap_or(""));
    println!();
    println!("signals:");
    if let Some(signals) = out["signals"].as_object() {
        for (key, value) in signals {
            println!("  {key}: {}", value.as_str().unwrap_or(""));
        }
    }
    println!();
    println!("weak_spots:");
    if out["weak_spots"].as_array().is_none_or(Vec::is_empty) {
        println!("  none");
    } else if let Some(items) = out["weak_spots"].as_array() {
        for item in items {
            println!(
                "  - {} [{}]: {}",
                item["signal"].as_str().unwrap_or(""),
                item["state"].as_str().unwrap_or(""),
                item["reason"].as_str().unwrap_or("")
            );
        }
    }
    println!();
    println!("safe_claim: {}", out["safe_claim"].as_str().unwrap_or(""));
    println!("boundary: {}", out["boundary"].as_str().unwrap_or(""));
}

fn print_demo_md(out: &Value) {
    println!("# NANDA LLMWave Demo\n");
    println!("- state: `{}`", out["state"].as_str().unwrap_or(""));
    println!("- input: `{}`", out["text"].as_str().unwrap_or(""));
    println!(
        "- top_pattern: `{}`",
        out["top_pattern"].as_str().unwrap_or("")
    );
    println!("- proof: `{}`", out["proof_state"].as_str().unwrap_or(""));
    println!("\n## Signals\n");
    if let Some(signals) = out["signals"].as_object() {
        for (key, value) in signals {
            println!("- `{key}`: `{}`", value.as_str().unwrap_or(""));
        }
    }
    println!("\n## Weak Spots\n");
    if out["weak_spots"].as_array().is_none_or(Vec::is_empty) {
        println!("- none");
    } else if let Some(items) = out["weak_spots"].as_array() {
        for item in items {
            println!(
                "- `{}` / `{}`: {}",
                item["signal"].as_str().unwrap_or(""),
                item["state"].as_str().unwrap_or(""),
                item["reason"].as_str().unwrap_or("")
            );
        }
    }
    println!("\n## Claim\n");
    println!("{}", out["safe_claim"].as_str().unwrap_or(""));
    println!();
    println!("{}", out["boundary"].as_str().unwrap_or(""));
}

fn print_demo_suite_text(out: &Value) {
    println!("NANDA LLMWave Demo Suite");
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {}: {} state={} weak_spots={}",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["state"].as_str().unwrap_or(""),
                case["weak_spots"].as_array().map_or(0, Vec::len)
            );
        }
    }
}

fn print_demo_suite_md(out: &Value) {
    println!("# NANDA LLMWave Demo Suite\n");
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- `{}`: `{}` state=`{}` weak_spots=`{}`",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["state"].as_str().unwrap_or(""),
                case["weak_spots"].as_array().map_or(0, Vec::len)
            );
        }
    }
}
