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
    let llmwave_contract = llmwave_contract_report(
        &packet,
        &text,
        &tokens,
        &decode,
        &hrr_binding,
        &cleanup_memory,
        &cleanup_dictionary,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &proof_summary,
        &args.lens,
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
        "llmwave_contract": llmwave_contract,
        "public_demo": public_demo,
        "pattern_store": compact_pattern_store_report(&packet, 3),
        "decode": decode,
        "feedback_preview": feedback_preview,
        "read_as": "LLMWave v67 loop: raw text -> field excitation -> selected lens -> structural decode -> cleanup/energy/capacity/anti-wave audit -> packed readiness -> proof summary -> demo card."
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

pub(crate) fn llmwave_token_compact_report(packet: &Packet, text: &str, top_k: usize) -> Value {
    let args = LlmwaveEvalArgs {
        suite: PathBuf::from("<serve>"),
        input_format: InputFormat::Json,
        top_k,
        steps: 1,
        search_top_k: 8,
        route_cap: 256,
        route_triad_cap: 32,
        group_by: PeakGroupBy::Route,
        format: OutputFormat::Json,
        normalize_paths: false,
    };
    let report = llmwave_report_from_packet(packet, Path::new("<serve>"), text, &args);
    let token = &report["llmwave_contract"]["lenses"]["token"];
    json!({
        "mode": "llmwave-token-compact",
        "version": "v75-serve-compact-token-lens",
        "contract_state": report["llmwave_contract"]["state"],
        "proof_state": report["proof_summary"]["state"],
        "token_state": token["state"],
        "ready": token["ready"],
        "prefix": token["prefix"],
        "top_token": token["top_token"],
        "top_phrase": token["top_phrase"],
        "margin": token["margin"],
        "top_k": token["top_k"],
        "baseline_compare": token["baseline_compare"],
        "token_cleanup": token["token_cleanup"],
        "anti_wave": token["anti_wave"],
        "token_memory": token["token_memory"],
        "read_as": "Compact Token Lens response for agent next-token/phrase resonance."
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
            "version": "v62-demo-raw-text-adapter",
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

    let text = demo_text(&args)?;
    let (input, mut packet, text, raw_adapter) = load_demo_packet(&args, &text)?;
    let eval_args = demo_eval_args(&args);
    if args.train {
        let case = LlmwaveEvalCase {
            id: "demo-feedback".to_string(),
            path: input.to_path_buf(),
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
            expected_next_token: String::new(),
        };
        packet = inject_llmwave_feedback(packet, &text, &case, &eval_args)?;
    }
    let report = llmwave_report_from_packet(&packet, input, &text, &eval_args);
    let mut demo = demo_surface_report("single", input, &text, &report);
    if let Some(adapter) = raw_adapter {
        apply_raw_adapter_metadata(&mut demo, adapter);
    }
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
            expected_next_token: String::new(),
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
        "demo": case.expected_demo_state.is_empty() || report["public_demo"]["state"].as_str() == Some(case.expected_demo_state.as_str()),
        "llmwave_contract": if case.expected_next_token.is_empty() {
            report["llmwave_contract"]["state"].as_str() == Some("LLMWAVE_LENS_READY")
        } else {
            report["llmwave_contract"]["lenses"]["token"]["top_token"].as_str() == Some(case.expected_next_token.as_str())
        },
        "next_token": case.expected_next_token.is_empty() || report["llmwave_contract"]["lenses"]["token"]["top_token"].as_str() == Some(case.expected_next_token.as_str())
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
            "llmwave_contract": report["llmwave_contract"]["state"],
            "token_lens": report["llmwave_contract"]["lenses"]["token"]["state"],
            "demo": report["public_demo"]["state"]
        },
        "actual_next_token": report["llmwave_contract"]["lenses"]["token"]["top_token"],
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
    let llmwave_contract = llmwave_contract_report(
        &packet,
        text,
        &tokens,
        &decode,
        &hrr_binding,
        &cleanup_memory,
        &cleanup_dictionary,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &proof_summary,
        &LlmwaveLensKind::Pattern,
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
        "llmwave_contract": llmwave_contract,
        "public_demo": public_demo,
        "decode": decode
    })
}

fn load_demo_packet<'a>(
    args: &'a DemoArgs,
    text: &str,
) -> Result<(&'a Path, Packet, String, Option<Value>)> {
    if let Some(raw_path) = args.from_text.as_deref() {
        let raw =
            fs::read_to_string(raw_path).with_context(|| format!("read {}", raw_path.display()))?;
        let query = if text.trim().is_empty() {
            raw.clone()
        } else {
            text.to_string()
        };
        let (packet, adapter) =
            packet_from_demo_text(&raw, &args.task_id, &args.domain, &query, raw_path);
        return Ok((raw_path, packet, query, Some(adapter)));
    }
    let input = args
        .input
        .as_deref()
        .ok_or_else(|| anyhow!("nanda demo requires an input packet, --from-text, or --suite"))?;
    let packet = load_packet_auto(
        input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        text,
        args.normalize_paths,
    )?;
    Ok((input, packet, text.to_string(), None))
}

fn packet_from_demo_text(
    raw: &str,
    task_id: &str,
    domain: &str,
    query: &str,
    source: &Path,
) -> (Packet, Value) {
    let mut triads = parse_arrow_triads(raw, task_id, domain);
    let extraction_method = if triads.is_empty() {
        let tokens = tokenize_pattern(raw);
        triads = tokens_to_query_triads(&tokens, task_id, domain);
        "token-window-fallback"
    } else {
        "arrow-triads"
    };
    let quality = if extraction_method == "arrow-triads" {
        "RAW_ADAPTER_READY"
    } else {
        "RAW_ADAPTER_REVIEW"
    };
    let triad_count = triads.len();
    let packet = Packet {
        task_id: task_id.to_string(),
        domain: domain.to_string(),
        query: query.to_string(),
        triads,
        candidate_triads: vec![],
        candidate_answer: String::new(),
        aliases: vec![],
        canonicalization: CanonicalizationReport::default(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
        resonance_memory: vec![],
        continuation_memory: vec![],
    };
    (
        packet,
        json!({
            "input_mode": "raw-text",
            "source": source.display().to_string(),
            "extraction_method": extraction_method,
            "quality": quality,
            "triads": triad_count,
            "read_as": "Raw demo adapter accepts explicit `subject -> relation -> object [route=x group=y]` lines. Free text fallback is review-only."
        }),
    )
}

fn parse_arrow_triads(raw: &str, task_id: &str, domain: &str) -> Vec<Triad> {
    let mut triads = vec![];
    let mut section = "triads".to_string();
    for (line_idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            let lower = trimmed.trim_start_matches('#').trim().to_lowercase();
            if lower.contains("candidate") {
                section = "candidate_triads".to_string();
            } else if lower.contains("triad") {
                section = "triads".to_string();
            }
            continue;
        }
        let cleaned = trimmed
            .trim_start_matches('-')
            .trim_start_matches('*')
            .trim_start()
            .trim();
        if !cleaned.contains("->") {
            continue;
        }
        let (body, route, group) = split_arrow_metadata(cleaned, task_id, domain);
        let parts = body
            .split("->")
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.len() != 3 {
            continue;
        }
        triads.push(Triad {
            id: format!("raw{}", triads.len() + 1),
            subject: parts[0].to_string(),
            relation: parts[1].to_string(),
            object: parts[2].to_string(),
            evidence: format!("{}:{}", section, line_idx + 1),
            confidence: 0.82,
            subject_role: "subject".to_string(),
            object_role: "object".to_string(),
            route,
            group,
        });
    }
    normalize_ids(triads, "raw")
}

fn split_arrow_metadata(line: &str, task_id: &str, domain: &str) -> (String, String, String) {
    let default_route = if domain.trim().is_empty() {
        task_id.to_string()
    } else {
        domain.to_string()
    };
    let default_group = format!("{task_id}:raw");
    let Some(start) = line.rfind('[') else {
        return (line.to_string(), default_route, default_group);
    };
    let Some(end_rel) = line[start..].find(']') else {
        return (line.to_string(), default_route, default_group);
    };
    let end = start + end_rel;
    let mut route = default_route;
    let mut group = default_group;
    let meta = &line[start + 1..end];
    for item in meta.split_whitespace() {
        if let Some(value) = item.strip_prefix("route=") {
            route = value.trim_matches('"').trim_matches('\'').to_string();
        } else if let Some(value) = item.strip_prefix("group=") {
            group = value.trim_matches('"').trim_matches('\'').to_string();
        }
    }
    (line[..start].trim().to_string(), route, group)
}

fn apply_raw_adapter_metadata(demo: &mut Value, adapter: Value) {
    if let Some(obj) = demo.as_object_mut() {
        obj.insert("input_mode".to_string(), adapter["input_mode"].clone());
        obj.insert("raw_adapter".to_string(), adapter.clone());
    }
    if adapter["quality"].as_str() == Some("RAW_ADAPTER_REVIEW") {
        if let Some(items) = demo["weak_spots"].as_array_mut() {
            items.push(json!({
                "signal": "raw_adapter",
                "state": adapter["quality"],
                "expected": ["RAW_ADAPTER_READY"],
                "reason": "Raw input had no explicit arrow triads, so the token-window fallback is review-only."
            }));
        }
        if let Some(obj) = demo.as_object_mut() {
            obj.insert("state".to_string(), json!("PUBLIC_DEMO_REVIEW"));
            obj.insert(
                "safe_claim".to_string(),
                json!("LLMWave produced a reviewable structural continuation from weak raw-text extraction; write explicit arrow triads before relying on it."),
            );
        }
    }
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

#[allow(clippy::too_many_arguments)]
fn llmwave_contract_report(
    packet: &Packet,
    text: &str,
    tokens: &[String],
    decode: &Value,
    hrr: &Value,
    cleanup: &Value,
    cleanup_dictionary: &Value,
    attractor: &Value,
    capacity: &Value,
    anti_wave: &Value,
    hot_cycle: &Value,
    proof: &Value,
    selected: &LlmwaveLensKind,
) -> Value {
    let pattern = llmwave_pattern_lens(decode, proof, cleanup_dictionary);
    let polarity = llmwave_polarity_lens(decode);
    let cleanup_lens = llmwave_cleanup_lens(cleanup, cleanup_dictionary);
    let token = llmwave_token_lens(packet, tokens, proof, cleanup_dictionary);
    let field = llmwave_field_report(packet, text, tokens, decode, hrr, attractor, capacity);
    let field_snapshot = llmwave_field_snapshot(packet, text, tokens, decode, anti_wave);
    let convex = llmwave_convex_lens(decode, &field_snapshot);
    let concave = llmwave_concave_lens(decode);
    let prism = llmwave_prism_lens(decode, &convex, anti_wave);
    let role = llmwave_role_lens(decode);
    let temporal = llmwave_temporal_lens(decode);
    let evidence = llmwave_evidence_lens(packet, decode);
    let energy = llmwave_energy_lens(attractor, &field_snapshot, &concave);
    let anti = llmwave_anti_lens(anti_wave, decode, &field_snapshot);
    let selected_name = llmwave_lens_name(selected);
    let selected_lens = match selected {
        LlmwaveLensKind::Pattern => pattern.clone(),
        LlmwaveLensKind::Polarity => polarity.clone(),
        LlmwaveLensKind::Cleanup => cleanup_lens.clone(),
        LlmwaveLensKind::Token => token.clone(),
        LlmwaveLensKind::Convex => convex.clone(),
        LlmwaveLensKind::Concave => concave.clone(),
        LlmwaveLensKind::Prism => prism.clone(),
        LlmwaveLensKind::Role => role.clone(),
        LlmwaveLensKind::Temporal => temporal.clone(),
        LlmwaveLensKind::Evidence => evidence.clone(),
        LlmwaveLensKind::Energy => energy.clone(),
        LlmwaveLensKind::Anti => anti.clone(),
    };
    let selected_ready = selected_lens["ready"].as_bool().unwrap_or(false);
    let hot_budget = llmwave_hot_budget_report(packet, capacity, hot_cycle);
    let baseline = llmwave_baseline_compare(tokens, decode);
    let state = if selected_ready && proof["state"].as_str() == Some("LLMWAVE_PROOF_READY") {
        "LLMWAVE_LENS_READY"
    } else if selected_lens["state"]
        .as_str()
        .is_some_and(|state| state.contains("REVERSED") || state.contains("AMBIGUOUS"))
    {
        "LLMWAVE_LENS_WATCH"
    } else {
        "LLMWAVE_LENS_REVIEW"
    };
    json!({
        "version": "v67-field-lens-contract",
        "state": state,
        "selected": selected_name,
        "selected_lens": selected_lens,
        "field": field,
        "field_snapshot": field_snapshot,
        "lenses": {
            "pattern": pattern,
            "polarity": polarity,
            "cleanup": cleanup_lens,
            "token": token,
            "convex": convex,
            "concave": concave,
            "prism": prism,
            "role": role,
            "temporal": temporal,
            "evidence": evidence,
            "energy": energy,
            "anti": anti
        },
        "lens_taxonomy": llmwave_lens_taxonomy(),
        "baseline_compare": baseline,
        "hot_budget": hot_budget,
        "proof_state": proof["state"],
        "answer_ready": state == "LLMWAVE_LENS_READY",
        "anti_wave_state": anti_wave["state"],
        "read_as": "v80 separates the wave field from lens optics. Convex gathers peaks, concave splits contested peaks, and prism explains peak contributions."
    })
}

fn llmwave_lens_name(kind: &LlmwaveLensKind) -> &'static str {
    match kind {
        LlmwaveLensKind::Pattern => "pattern",
        LlmwaveLensKind::Polarity => "polarity",
        LlmwaveLensKind::Cleanup => "cleanup",
        LlmwaveLensKind::Token => "token",
        LlmwaveLensKind::Convex => "convex",
        LlmwaveLensKind::Concave => "concave",
        LlmwaveLensKind::Prism => "prism",
        LlmwaveLensKind::Role => "role",
        LlmwaveLensKind::Temporal => "temporal",
        LlmwaveLensKind::Evidence => "evidence",
        LlmwaveLensKind::Energy => "energy",
        LlmwaveLensKind::Anti => "anti",
    }
}

fn llmwave_lens_taxonomy() -> Value {
    json!({
        "version": "v76-lens-taxonomy",
        "active": ["pattern", "polarity", "cleanup", "token", "convex", "concave", "prism", "role", "temporal", "evidence", "energy", "anti"],
        "planned": ["microscope", "telescope"],
        "convex": "gather weak aligned signals into a stable peak",
        "concave": "separate a mixed or contested peak into rival branches",
        "prism": "explain a peak by route, relation, role path, and polarity contribution",
        "role": "read actor/action/target role binding and role-swap risk",
        "temporal": "read event order and sequence gaps",
        "evidence": "read whether the peak is backed by evidence or only resembles a fact",
        "energy": "read basin stability, margin, attractor trace, and perturbation risk",
        "anti": "explain destructive interference and what changed after suppression"
    })
}

fn llmwave_field_report(
    packet: &Packet,
    text: &str,
    tokens: &[String],
    decode: &Value,
    hrr: &Value,
    attractor: &Value,
    capacity: &Value,
) -> Value {
    json!({
        "input_kind": if text.trim().is_empty() { "structural-prefix" } else { "text-prefix" },
        "tokens": tokens.len(),
        "memory_triads": packet.triads.len(),
        "continuation_memory": packet.continuation_memory.len(),
        "query_triads": decode["recurrent"]["steps"]
            .as_array()
            .and_then(|steps| steps.first())
            .and_then(|step| step["query"].as_array())
            .map_or(0, Vec::len),
        "field_state": decode["source_search"]["field_state"],
        "top_peak": decode["source_search"]["top_peak"],
        "hrr_state": hrr["state"],
        "attractor_state": attractor["state"],
        "capacity_state": capacity["state"],
        "estimated_crosstalk": capacity["estimated_crosstalk"],
        "read_as": "Shared LLMWave field before choosing a readout lens."
    })
}

fn llmwave_field_snapshot(
    packet: &Packet,
    text: &str,
    tokens: &[String],
    decode: &Value,
    anti_wave: &Value,
) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let energy = patterns
        .iter()
        .map(|item| item["score"].as_f64().unwrap_or(0.0).max(0.0))
        .sum::<f64>();
    let top_score = patterns
        .first()
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0);
    let second_score = patterns
        .get(1)
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0);
    let fingerprint_source = json!({
        "text": text,
        "tokens": tokens,
        "memory_triads": packet.triads.len(),
        "continuation_memory": packet.continuation_memory.len(),
        "top_peak": decode["source_search"]["top_peak"],
        "top_pattern": decode["top_pattern"],
        "top_score": round4(top_score),
        "patterns": patterns.iter().take(8).map(pattern_label_value).collect::<Vec<_>>(),
        "anti_wave_state": anti_wave["state"]
    });
    let digest = Sha256::digest(fingerprint_source.to_string().as_bytes());
    json!({
        "version": "v77-field-snapshot",
        "snapshot_id": format!("{:x}", digest)[..16].to_string(),
        "input_kind": if text.trim().is_empty() { "structural-prefix" } else { "text-prefix" },
        "token_count": tokens.len(),
        "pattern_count": patterns.len(),
        "energy": round4(energy),
        "top_score": round4(top_score),
        "second_score": round4(second_score),
        "margin": round4((top_score - second_score).max(0.0)),
        "top_peak": decode["source_search"]["top_peak"],
        "top_pattern": decode["top_pattern"],
        "anti_wave_state": anti_wave["state"],
        "read_as": "Field Snapshot is the cold repeatable fingerprint of the wave before lens optics read it."
    })
}

fn llmwave_pattern_lens(decode: &Value, proof: &Value, cleanup_dictionary: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let top_pattern = decode["top_pattern"].as_str().unwrap_or("");
    let cleanup_ready = matches!(
        cleanup_dictionary["state"].as_str().unwrap_or(""),
        "CLEANUP_DICTIONARY_READY" | "CLEANUP_DICTIONARY_EMPTY"
    );
    let proof_ready = proof["state"].as_str() == Some("LLMWAVE_PROOF_READY");
    let ready = !top_pattern.is_empty() && cleanup_ready && proof_ready;
    let candidates = patterns
        .iter()
        .take(5)
        .map(|item| {
            json!({
                "pattern": pattern_label_value(item),
                "score": item["score"],
                "route": item["route"],
                "polarity": item["polarity"],
                "continuity": item["continuity"],
                "peak": item["peak"],
                "safe_to_use": ready && item["polarity"].as_str().unwrap_or("") != "REVERSED"
            })
        })
        .collect::<Vec<_>>();
    json!({
        "kind": "pattern",
        "state": if ready { "PATTERN_LENS_READY" } else if top_pattern.is_empty() { "PATTERN_LENS_EMPTY" } else { "PATTERN_LENS_REVIEW" },
        "ready": ready,
        "top_pattern": top_pattern,
        "candidates": candidates,
        "cleanup_state": cleanup_dictionary["state"],
        "proof_state": proof["state"],
        "read_as": "Pattern Lens reads the field as ranked next structural continuations."
    })
}

fn llmwave_polarity_lens(decode: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let mut directional = 0usize;
    let mut reversed = 0usize;
    let mut unknown = 0usize;
    for item in &patterns {
        let polarity = item["polarity"].as_str().unwrap_or("");
        if polarity.contains("REVERSED") {
            reversed += 1;
        } else if polarity.contains("->") || polarity == "ALIGNED" {
            directional += 1;
        } else {
            unknown += 1;
        }
    }
    let top_polarity = patterns
        .first()
        .and_then(|item| item["polarity"].as_str())
        .unwrap_or("");
    let state = if top_polarity.contains("REVERSED") {
        "POLARITY_LENS_REVERSED"
    } else if top_polarity.contains("->") || top_polarity == "ALIGNED" {
        "POLARITY_LENS_DIRECTIONAL"
    } else if top_polarity.is_empty() {
        "POLARITY_LENS_EMPTY"
    } else {
        "POLARITY_LENS_AMBIGUOUS"
    };
    json!({
        "kind": "polarity",
        "state": state,
        "ready": state == "POLARITY_LENS_DIRECTIONAL",
        "top_polarity": top_polarity,
        "directional": directional,
        "reversed": reversed,
        "unknown": unknown,
        "stop_signal": state == "POLARITY_LENS_REVERSED",
        "read_as": "Polarity Lens reads direction, role reversal, and negation risk before using a continuation."
    })
}

fn llmwave_cleanup_lens(cleanup: &Value, cleanup_dictionary: &Value) -> Value {
    let items = cleanup["items"].as_array().cloned().unwrap_or_default();
    let state = if cleanup_dictionary["ambiguous"].as_u64().unwrap_or(0) > 0 {
        "CLEANUP_LENS_AMBIGUOUS"
    } else if cleanup_dictionary["exact"].as_u64().unwrap_or(0) > 0 {
        "CLEANUP_LENS_EXACT"
    } else if cleanup_dictionary["near"].as_u64().unwrap_or(0) > 0 {
        "CLEANUP_LENS_NEAR"
    } else {
        "CLEANUP_LENS_EMPTY"
    };
    let anchors = items
        .iter()
        .take(5)
        .map(|item| {
            json!({
                "raw_pattern": item["raw_pattern"],
                "nearest_pattern": item["nearest_pattern"],
                "score": item["score"],
                "state": item["state"]
            })
        })
        .collect::<Vec<_>>();
    json!({
        "kind": "cleanup",
        "state": state,
        "ready": matches!(state, "CLEANUP_LENS_EXACT" | "CLEANUP_LENS_NEAR"),
        "dictionary_state": cleanup_dictionary["state"],
        "anchors": anchors,
        "read_as": "Cleanup Lens maps noisy decoded peaks to known clean patterns and keeps ambiguity under WATCH."
    })
}

fn llmwave_convex_lens(decode: &Value, snapshot: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let mut basins: BTreeMap<String, Value> = BTreeMap::new();
    for item in &patterns {
        let key = item["route"]
            .as_str()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| item["peak"].as_str())
            .unwrap_or("unrouted")
            .to_string();
        let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
        let support_row = json!({
            "triad": item["triad"],
            "pattern": pattern_label_value(item),
            "score": round4(score),
            "relation": item["relation"],
            "polarity": item["polarity"]
        });
        basins
            .entry(key.clone())
            .and_modify(|basin| {
                let total = basin["total_score"].as_f64().unwrap_or(0.0) + score;
                let count = basin["support_count"].as_u64().unwrap_or(0) + 1;
                basin["total_score"] = json!(round4(total));
                basin["support_count"] = json!(count);
                if score > basin["top_score"].as_f64().unwrap_or(0.0) {
                    basin["top_score"] = json!(round4(score));
                    basin["top_pattern"] = json!(pattern_label_value(item));
                }
                if let Some(support) = basin["support"].as_array_mut() {
                    support.push(support_row.clone());
                    support.sort_by(|a, b| {
                        b["score"]
                            .as_f64()
                            .unwrap_or(0.0)
                            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    support.truncate(5);
                }
            })
            .or_insert_with(|| {
                json!({
                    "basin": key,
                    "total_score": round4(score),
                    "top_score": round4(score),
                    "support_count": 1,
                    "top_pattern": pattern_label_value(item),
                    "support": [support_row]
                })
            });
    }
    let mut rows = basins.into_values().collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["total_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["total_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for row in &mut rows {
        let total = row["total_score"].as_f64().unwrap_or(0.0);
        let top = row["top_score"].as_f64().unwrap_or(0.0);
        let count = row["support_count"].as_u64().unwrap_or(1).max(1) as f64;
        row["coherence"] = json!(round4((total / count).min(top).max(0.0)));
        row["gather_gain"] = json!(round4((total - top).max(0.0)));
    }
    let top_total = rows
        .first()
        .and_then(|row| row["total_score"].as_f64())
        .unwrap_or(0.0);
    let second_total = rows
        .get(1)
        .and_then(|row| row["total_score"].as_f64())
        .unwrap_or(0.0);
    let margin = round4((top_total - second_total).max(0.0));
    let support_count = rows
        .first()
        .and_then(|row| row["support_count"].as_u64())
        .unwrap_or(0);
    let ready = !rows.is_empty() && support_count >= 2 && margin >= 0.03;
    rows.truncate(5);
    json!({
        "kind": "convex",
        "version": "v78-convex-gathering-lens",
        "state": if ready { "CONVEX_LENS_READY" } else if rows.is_empty() { "CONVEX_LENS_EMPTY" } else { "CONVEX_LENS_REVIEW" },
        "ready": ready,
        "top_basin": rows.first().map(|row| row["basin"].clone()).unwrap_or(Value::Null),
        "top_pattern": rows.first().map(|row| row["top_pattern"].clone()).unwrap_or(Value::Null),
        "margin": margin,
        "basins": rows,
        "snapshot_id": snapshot["snapshot_id"],
        "read_as": "Convex Lens gathers aligned weak pattern waves into a route basin and reports whether one basin dominates."
    })
}

fn llmwave_concave_lens(decode: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let top_score = patterns
        .first()
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0)
        .max(0.0);
    let mut branches = patterns
        .iter()
        .take(8)
        .map(|item| {
            let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
            json!({
                "pattern": pattern_label_value(item),
                "score": round4(score),
                "delta_from_top": round4((top_score - score).max(0.0)),
                "route": item["route"],
                "group": item["group"],
                "relation": item["relation"],
                "polarity": item["polarity"],
                "competing": top_score > 0.0 && score >= top_score * 0.70
            })
        })
        .collect::<Vec<_>>();
    branches.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let competing = branches
        .iter()
        .filter(|branch| branch["competing"].as_bool().unwrap_or(false))
        .count();
    let second_score = branches
        .get(1)
        .and_then(|branch| branch["score"].as_f64())
        .unwrap_or(0.0);
    let separation = round4((top_score - second_score).max(0.0));
    let state = if branches.is_empty() {
        "CONCAVE_LENS_EMPTY"
    } else if competing >= 2 || separation < 0.04 {
        "CONCAVE_LENS_SPLIT"
    } else {
        "CONCAVE_LENS_SINGLE"
    };
    json!({
        "kind": "concave",
        "version": "v79-concave-separation-lens",
        "state": state,
        "ready": !branches.is_empty(),
        "competing_branches": competing,
        "separation": separation,
        "branches": branches,
        "read_as": "Concave Lens spreads a mixed peak into rival continuations instead of forcing one answer."
    })
}

fn llmwave_prism_lens(decode: &Value, convex: &Value, anti_wave: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let routes = prism_dimension(&patterns, "route");
    let relations = prism_dimension(&patterns, "relation");
    let polarities = prism_dimension(&patterns, "polarity");
    let role_paths = patterns
        .iter()
        .map(|item| {
            let key = format!(
                "{}->{}",
                item["subject_role"].as_str().unwrap_or("unknown"),
                item["object_role"].as_str().unwrap_or("unknown")
            );
            let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
            (key, score)
        })
        .collect::<Vec<_>>();
    let role_paths = aggregate_prism_pairs(role_paths);
    let top_pattern = patterns
        .first()
        .map(pattern_label_value)
        .map(Value::String)
        .unwrap_or(Value::Null);
    let ready = !patterns.is_empty();
    json!({
        "kind": "prism",
        "version": "v80-prism-explanation-lens",
        "state": if ready { "PRISM_LENS_READY" } else { "PRISM_LENS_EMPTY" },
        "ready": ready,
        "top_pattern": top_pattern,
        "dominant_basin": convex["top_basin"],
        "contributions": {
            "routes": routes,
            "relations": relations,
            "role_paths": role_paths,
            "polarities": polarities
        },
        "anti_wave_state": anti_wave["state"],
        "explain_peak": {
            "snapshot_route": decode["source_search"]["top_peak"],
            "field_state": decode["source_search"]["field_state"],
            "top_score": patterns.first().map(|item| item["score"].clone()).unwrap_or(Value::Null),
            "why_visible": "top pattern is explained by route basin, relation contribution, role path, polarity, and any anti-wave state"
        },
        "read_as": "Prism Lens decomposes one visible peak into the structural contributions that made it visible."
    })
}

fn prism_dimension(patterns: &[Value], field: &str) -> Vec<Value> {
    let pairs = patterns
        .iter()
        .map(|item| {
            let key = item[field]
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown")
                .to_string();
            let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
            (key, score)
        })
        .collect::<Vec<_>>();
    aggregate_prism_pairs(pairs)
}

fn aggregate_prism_pairs(pairs: Vec<(String, f64)>) -> Vec<Value> {
    let mut totals: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for (key, score) in pairs {
        let entry = totals.entry(key).or_insert((0.0, 0));
        entry.0 += score;
        entry.1 += 1;
    }
    let mut rows = totals
        .into_iter()
        .map(|(key, (score, count))| {
            json!({
                "key": key,
                "score": round4(score),
                "support_count": count
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(5);
    rows
}

fn llmwave_role_lens(decode: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let role_paths = patterns
        .iter()
        .take(8)
        .map(|item| {
            let subject_role = item["subject_role"].as_str().unwrap_or("");
            let object_role = item["object_role"].as_str().unwrap_or("");
            let relation = item["relation"].as_str().unwrap_or("");
            let path = format!(
                "{}->{}->{}",
                role_family(subject_role),
                relation_family(relation),
                role_family(object_role)
            );
            let polarity = item["polarity"].as_str().unwrap_or("");
            let risk = if polarity.contains("REVERSED") {
                "ROLE_SWAP_RISK"
            } else if role_family(subject_role) == role_family(object_role) {
                "ROLE_COLLAPSE_REVIEW"
            } else {
                "ROLE_ALIGNED"
            };
            json!({
                "actor": item["subject"],
                "action": item["relation"],
                "target": item["object"],
                "subject_role": subject_role,
                "object_role": object_role,
                "role_path": path,
                "polarity": polarity,
                "risk": risk,
                "score": item["score"]
            })
        })
        .collect::<Vec<_>>();
    let top_risk = role_paths
        .first()
        .and_then(|item| item["risk"].as_str())
        .unwrap_or("");
    let swap_risks = role_paths
        .iter()
        .filter(|item| item["risk"].as_str() == Some("ROLE_SWAP_RISK"))
        .count();
    let state = if role_paths.is_empty() {
        "ROLE_LENS_EMPTY"
    } else if top_risk == "ROLE_SWAP_RISK" {
        "ROLE_LENS_SWAP_RISK"
    } else if swap_risks > 0 {
        "ROLE_LENS_REVIEW"
    } else {
        "ROLE_LENS_READY"
    };
    json!({
        "kind": "role",
        "version": "v81-role-binding-lens",
        "state": state,
        "ready": state == "ROLE_LENS_READY",
        "top_role_path": role_paths.first().map(|item| item["role_path"].clone()).unwrap_or(Value::Null),
        "role_swap_risks": swap_risks,
        "bindings": role_paths,
        "read_as": "Role Lens reads actor/action/target binding and keeps role swaps visible."
    })
}

fn llmwave_temporal_lens(decode: &Value) -> Value {
    let steps = decode["recurrent"]["steps"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut route_jumps = 0usize;
    let mut last_route = String::new();
    let mut rows = vec![];
    for step in &steps {
        let route = step["source_search"]["top_peak"].as_str().unwrap_or("");
        if !last_route.is_empty() && !route.is_empty() && route != last_route {
            route_jumps += 1;
        }
        if !route.is_empty() {
            last_route = route.to_string();
        }
        rows.push(json!({
            "step": step["step"],
            "route": route,
            "top_pattern": step["top_pattern"],
            "decoder_state": step["decoder_state"],
            "field_state": step["source_search"]["field_state"]
        }));
    }
    let repeated_pattern = rows
        .windows(2)
        .any(|window| window[0]["top_pattern"] == window[1]["top_pattern"]);
    let state = if rows.is_empty() {
        "TEMPORAL_LENS_EMPTY"
    } else if route_jumps > 0 {
        "TEMPORAL_LENS_ROUTE_JUMP"
    } else if repeated_pattern {
        "TEMPORAL_LENS_STANDING"
    } else {
        "TEMPORAL_LENS_ORDERED"
    };
    json!({
        "kind": "temporal",
        "version": "v82-temporal-order-lens",
        "state": state,
        "ready": matches!(state, "TEMPORAL_LENS_ORDERED" | "TEMPORAL_LENS_STANDING"),
        "steps": rows,
        "route_jumps": route_jumps,
        "standing_pattern": repeated_pattern,
        "read_as": "Temporal Lens reads recurrent decode as event/order flow and flags route jumps."
    })
}

fn llmwave_evidence_lens(packet: &Packet, decode: &Value) -> Value {
    let evidence_by_triad = packet
        .triads
        .iter()
        .map(|triad| (triad.id.clone(), triad.evidence.clone()))
        .collect::<BTreeMap<_, _>>();
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let rows = patterns
        .iter()
        .take(8)
        .map(|item| {
            let triad_id = item["triad"].as_str().unwrap_or("");
            let evidence = evidence_by_triad.get(triad_id).cloned().unwrap_or_default();
            let has_evidence = !evidence.trim().is_empty();
            json!({
                "triad": triad_id,
                "pattern": pattern_label_value(item),
                "route": item["route"],
                "score": item["score"],
                "evidence": evidence,
                "has_evidence": has_evidence,
                "evidence_state": if has_evidence { "EVIDENCE_BOUND" } else { "EVIDENCE_MISSING" }
            })
        })
        .collect::<Vec<_>>();
    let missing = rows
        .iter()
        .filter(|row| !row["has_evidence"].as_bool().unwrap_or(false))
        .count();
    let conflicts = evidence_conflicts(&packet.triads);
    let top_bound = rows
        .first()
        .and_then(|row| row["has_evidence"].as_bool())
        .unwrap_or(false);
    let state = if rows.is_empty() {
        "EVIDENCE_LENS_EMPTY"
    } else if !conflicts.is_empty() {
        "EVIDENCE_LENS_CONFLICT"
    } else if !top_bound {
        "EVIDENCE_LENS_TOP_MISSING"
    } else if missing > 0 {
        "EVIDENCE_LENS_PARTIAL"
    } else {
        "EVIDENCE_LENS_READY"
    };
    json!({
        "kind": "evidence",
        "version": "v83-evidence-binding-lens",
        "state": state,
        "ready": matches!(state, "EVIDENCE_LENS_READY" | "EVIDENCE_LENS_PARTIAL"),
        "top_evidence_bound": top_bound,
        "missing": missing,
        "conflicts": conflicts,
        "bindings": rows,
        "read_as": "Evidence Lens separates an evidence-backed peak from a plausible but unsupported peak."
    })
}

fn llmwave_energy_lens(attractor: &Value, snapshot: &Value, concave: &Value) -> Value {
    let steps = attractor["steps"].as_array().cloned().unwrap_or_default();
    let final_energy = steps
        .last()
        .and_then(|step| step["energy"].as_f64())
        .unwrap_or_else(|| snapshot["energy"].as_f64().unwrap_or(0.0));
    let dropping = steps
        .iter()
        .any(|step| step["trend"].as_str() == Some("DROPPING"));
    let route_jumps = attractor["route_jumps"].as_u64().unwrap_or(0);
    let margin = snapshot["margin"].as_f64().unwrap_or(0.0);
    let contested = concave["state"].as_str() == Some("CONCAVE_LENS_SPLIT");
    let state = if steps.is_empty() {
        "ENERGY_LENS_SNAPSHOT_ONLY"
    } else if route_jumps > 0 || dropping {
        "ENERGY_LENS_UNSTABLE"
    } else if contested && margin < 0.08 {
        "ENERGY_LENS_CONTESTED"
    } else {
        "ENERGY_LENS_STABLE"
    };
    json!({
        "kind": "energy",
        "version": "v84-energy-stability-lens",
        "state": state,
        "ready": matches!(state, "ENERGY_LENS_STABLE" | "ENERGY_LENS_CONTESTED"),
        "final_energy": round4(final_energy),
        "margin": round4(margin),
        "route_jumps": route_jumps,
        "dropping": dropping,
        "attractor_state": attractor["state"],
        "contested": contested,
        "snapshot_energy": snapshot["energy"],
        "read_as": "Energy Lens reads basin stability: margin, attractor trend, route jumps, and contested split risk."
    })
}

fn llmwave_anti_lens(anti_wave: &Value, decode: &Value, snapshot: &Value) -> Value {
    let applications = anti_wave["applications"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let suppressions = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("suppress"))
        .cloned()
        .collect::<Vec<_>>();
    let reinforcements = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("reinforce"))
        .count();
    let top_after = decode["top_pattern"].as_str().unwrap_or("");
    let changed_field = !suppressions.is_empty() && !top_after.is_empty();
    let state = if !suppressions.is_empty() {
        "ANTI_LENS_SUPPRESSED_SHORTCUT"
    } else if anti_wave["negative_records"].as_u64().unwrap_or(0) > 0 {
        "ANTI_LENS_AVAILABLE_NOT_TRIGGERED"
    } else {
        "ANTI_LENS_NO_MEMORY"
    };
    json!({
        "kind": "anti",
        "version": "v85-anti-lens-destructive-report",
        "state": state,
        "ready": !suppressions.is_empty(),
        "negative_records": anti_wave["negative_records"],
        "suppressions": suppressions,
        "reinforcements": reinforcements,
        "top_after": top_after,
        "changed_field": changed_field,
        "snapshot_id": snapshot["snapshot_id"],
        "read_as": "Anti Lens explains destructive interference: which shortcut was suppressed, what stayed visible, and whether the field survived."
    })
}

#[derive(Clone, Debug)]
struct TokenPatternRecord {
    prefix_tokens: Vec<String>,
    next_token: String,
    next_phrase: String,
    pattern: String,
    triad_id: String,
    route: String,
    group: String,
    polarity: String,
    confidence: f64,
}

fn llmwave_token_lens(
    packet: &Packet,
    tokens: &[String],
    proof: &Value,
    cleanup_dictionary: &Value,
) -> Value {
    let records = token_pattern_records(packet);
    let anti = token_anti_wave_report(packet, tokens);
    let mut candidates = token_resonance_candidates(tokens, &records, &anti);
    candidates.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(5);
    let top_token = candidates
        .first()
        .and_then(|candidate| candidate["token"].as_str())
        .unwrap_or("");
    let top_score = candidates
        .first()
        .and_then(|candidate| candidate["score"].as_f64())
        .unwrap_or(0.0);
    let second_score = candidates
        .get(1)
        .and_then(|candidate| candidate["score"].as_f64())
        .unwrap_or(0.0);
    let margin = round4((top_score - second_score).max(0.0));
    let cleanup = token_cleanup_report(top_token, &candidates, &records, margin);
    let mut baseline = token_lens_baseline(tokens, &records);
    let baseline_score = baseline["top_score"].as_f64().unwrap_or(0.0);
    baseline["field_top_score"] = json!(round4(top_score));
    baseline["field_beats_baseline"] = json!(top_score > baseline_score + 0.03);
    let cleanup_ready = matches!(
        cleanup["state"].as_str().unwrap_or(""),
        "TOKEN_CLEANUP_EXACT" | "TOKEN_CLEANUP_NEAR"
    );
    let proof_ready = proof["state"].as_str() == Some("LLMWAVE_PROOF_READY");
    let ready = !top_token.is_empty() && cleanup_ready && proof_ready && margin >= 0.015;
    let state = if ready {
        "TOKEN_LENS_READY"
    } else if top_token.is_empty() {
        "TOKEN_LENS_EMPTY"
    } else if cleanup["state"].as_str() == Some("TOKEN_CLEANUP_AMBIGUOUS") || margin < 0.015 {
        "TOKEN_LENS_CONTESTED"
    } else if baseline["top_token"].as_str() == Some(top_token)
        && !baseline["field_beats_baseline"].as_bool().unwrap_or(false)
    {
        "TOKEN_LENS_BASELINE_TIE"
    } else {
        "TOKEN_LENS_REVIEW"
    };
    json!({
        "kind": "token",
        "version": "v75-token-lens-next-token-resonance",
        "state": state,
        "ready": ready,
        "prefix": tokens.join(" "),
        "top_token": top_token,
        "top_phrase": candidates.first().map(|candidate| candidate["phrase"].clone()).unwrap_or(Value::Null),
        "top_k": candidates,
        "margin": margin,
        "token_cleanup": cleanup,
        "structural_cleanup_state": cleanup_dictionary["state"],
        "anti_wave": anti,
        "baseline_compare": baseline,
        "token_memory": {
            "version": "v69-token-pattern-records",
            "records": records.len(),
            "packed_record_bytes": 32,
            "estimated_packed_bytes": records.len() * 32,
            "fits_pattern_arena": records.len() * 32 <= PATTERN_STORE_ARENA_BYTES
        },
        "encoder": {
            "version": "v70-token-position-phase",
            "position_weights": [1.0, 0.70, 0.45, 0.25, 0.15, 0.10],
            "read_as": "Prefix tokens are encoded with deterministic token waves plus relative position phase."
        },
        "read_as": "Token Lens reads the LLMWave field as next-token/phrase candidates, with cleanup, anti-wave, and cheap baselines visible."
    })
}

fn token_pattern_records(packet: &Packet) -> Vec<TokenPatternRecord> {
    let mut records = vec![];
    for triad in &packet.triads {
        let sequence = tokenize_pattern(&format!(
            "{} {} {}",
            triad.subject, triad.relation, triad.object
        ));
        if sequence.len() < 2 {
            continue;
        }
        for index in 1..sequence.len() {
            let start = index.saturating_sub(6);
            let prefix_tokens = sequence[start..index].to_vec();
            let next_token = sequence[index].clone();
            let next_phrase = sequence[index..].join(" ");
            records.push(TokenPatternRecord {
                prefix_tokens,
                next_token,
                next_phrase,
                pattern: format!(
                    "{} -> {} -> {}",
                    triad.subject, triad.relation, triad.object
                ),
                triad_id: triad.id.clone(),
                route: triad.route.clone(),
                group: triad.group.clone(),
                polarity: triad_polarity(triad),
                confidence: triad.confidence,
            });
        }
    }
    records
}

fn token_resonance_candidates(
    tokens: &[String],
    records: &[TokenPatternRecord],
    anti: &Value,
) -> Vec<Value> {
    let prefix_wave = token_prefix_wave(tokens);
    let query_terms = tokens
        .iter()
        .map(|token| norm(token))
        .collect::<BTreeSet<_>>();
    let mut by_token: BTreeMap<String, Value> = BTreeMap::new();
    for record in records {
        let wave_score = ((cosine(&prefix_wave, &token_prefix_wave(&record.prefix_tokens)) + 1.0)
            / 2.0)
            .clamp(0.0, 1.0);
        let suffix = token_suffix_match(tokens, &record.prefix_tokens);
        let length_fit = token_prefix_length_fit(tokens, &record.prefix_tokens);
        let route = token_route_match(&query_terms, record);
        let anti_penalty = token_anti_penalty(anti, record);
        let score = round4(
            (0.40 * wave_score)
                + (0.30 * suffix)
                + (0.10 * length_fit)
                + (0.12 * route)
                + (0.08 * record.confidence.min(1.0))
                - anti_penalty,
        )
        .max(0.0);
        let candidate = json!({
            "token": record.next_token,
            "phrase": record.next_phrase,
            "score": score,
            "wave_score": round4(wave_score),
            "suffix_score": round4(suffix),
            "length_fit": round4(length_fit),
            "route_score": round4(route),
            "anti_penalty": round4(anti_penalty),
            "pattern": record.pattern,
            "triad": record.triad_id,
            "route": record.route,
            "group": record.group,
            "polarity": record.polarity,
            "support": [{
                "triad": record.triad_id,
                "pattern": record.pattern,
                "prefix": record.prefix_tokens.join(" "),
                "next": record.next_token,
                "route": record.route,
                "score": score
            }]
        });
        by_token
            .entry(record.next_token.clone())
            .and_modify(|existing| {
                if score > existing["score"].as_f64().unwrap_or(0.0) {
                    *existing = candidate.clone();
                } else if let Some(support) = existing["support"].as_array_mut() {
                    support.push(candidate["support"][0].clone());
                }
            })
            .or_insert(candidate);
    }
    by_token.into_values().collect()
}

fn token_prefix_wave(tokens: &[String]) -> Vec<i32> {
    let mut out = vec![0i32; WAVE_DIM];
    let weights = [100, 70, 45, 25, 15, 10];
    for (offset, token) in tokens.iter().rev().take(weights.len()).enumerate() {
        let phase = vector(&format!("token:{}:pos:-{}", norm(token), offset + 1));
        let base = vector(&format!("token:{}", norm(token)));
        for idx in 0..WAVE_DIM {
            out[idx] += ((phase[idx] + base[idx]) * weights[offset]) / 100;
        }
    }
    out
}

fn token_suffix_match(query: &[String], prefix: &[String]) -> f64 {
    if query.is_empty() || prefix.is_empty() {
        return 0.0;
    }
    let max = query.len().min(prefix.len()).min(6);
    let mut weighted_hit = 0.0;
    let mut total = 0.0;
    for offset in 0..max {
        let weight = match offset {
            0 => 1.0,
            1 => 0.70,
            2 => 0.45,
            3 => 0.25,
            4 => 0.15,
            _ => 0.10,
        };
        total += weight;
        let q = query[query.len() - 1 - offset].as_str();
        let p = prefix[prefix.len() - 1 - offset].as_str();
        if norm(q) == norm(p) {
            weighted_hit += weight;
        }
    }
    if total == 0.0 {
        0.0
    } else {
        weighted_hit / total
    }
}

fn token_prefix_length_fit(query: &[String], prefix: &[String]) -> f64 {
    if query.is_empty() || prefix.is_empty() {
        return 0.0;
    }
    let delta = query.len().abs_diff(prefix.len()) as f64;
    (1.0 - (delta * 0.18)).clamp(0.0, 1.0)
}

fn token_route_match(query_terms: &BTreeSet<String>, record: &TokenPatternRecord) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let mut route_terms = BTreeSet::new();
    for value in [&record.route, &record.group] {
        for term in norm(value)
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|term| term.len() >= 2)
        {
            route_terms.insert(term.to_string());
        }
    }
    if route_terms.is_empty() {
        0.0
    } else {
        query_terms.intersection(&route_terms).count() as f64 / route_terms.len() as f64
    }
}

fn token_cleanup_report(
    top_token: &str,
    candidates: &[Value],
    records: &[TokenPatternRecord],
    margin: f64,
) -> Value {
    if top_token.is_empty() {
        return json!({
            "version": "v72-token-cleanup-dictionary",
            "state": "TOKEN_CLEANUP_EMPTY",
            "anchors": []
        });
    }
    let anchors = records
        .iter()
        .filter(|record| record.next_token == top_token)
        .take(5)
        .map(|record| {
            json!({
                "token": record.next_token,
                "phrase": record.next_phrase,
                "pattern": record.pattern,
                "triad": record.triad_id,
                "route": record.route
            })
        })
        .collect::<Vec<_>>();
    let state = if margin < 0.015 && candidates.len() > 1 {
        "TOKEN_CLEANUP_AMBIGUOUS"
    } else if !anchors.is_empty() {
        "TOKEN_CLEANUP_EXACT"
    } else {
        "TOKEN_CLEANUP_EMPTY"
    };
    json!({
        "version": "v72-token-cleanup-dictionary",
        "state": state,
        "token": top_token,
        "margin": margin,
        "anchors": anchors,
        "read_as": "Token cleanup maps a raw next-token peak back to known token-pattern records."
    })
}

fn token_anti_wave_report(packet: &Packet, tokens: &[String]) -> Value {
    let mut lanes = vec![];
    for memory in &packet.continuation_memory {
        if memory.decision != "reject" {
            continue;
        }
        let sequence = tokenize_pattern(&format!(
            "{} {} {}",
            memory.subject, memory.relation, memory.object
        ));
        for index in 1..sequence.len() {
            let prefix = sequence[index.saturating_sub(6)..index].to_vec();
            let next = sequence[index].clone();
            let match_score = token_suffix_match(tokens, &prefix);
            if match_score >= 0.65 {
                lanes.push(json!({
                    "prefix": prefix.join(" "),
                    "suppress_next": next,
                    "route": memory.route,
                    "match_score": round4(match_score),
                    "penalty": memory.penalty,
                    "source_feedback": memory.source_feedback
                }));
            }
        }
    }
    json!({
        "version": "v73-token-shortcut-anti-wave",
        "state": if lanes.is_empty() { "TOKEN_ANTI_WAVE_NONE" } else { "TOKEN_ANTI_WAVE_APPLIED" },
        "lanes": lanes,
        "read_as": "Token anti-wave suppresses prefix-specific false next tokens without killing the whole token topic."
    })
}

fn token_anti_penalty(anti: &Value, record: &TokenPatternRecord) -> f64 {
    anti["lanes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|lane| lane["suppress_next"].as_str() == Some(record.next_token.as_str()))
        .map(|lane| {
            lane["penalty"].as_f64().unwrap_or(0.18) * lane["match_score"].as_f64().unwrap_or(1.0)
        })
        .fold(0.0, f64::max)
}

fn token_lens_baseline(tokens: &[String], records: &[TokenPatternRecord]) -> Value {
    let mut by_token: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for record in records {
        let suffix = token_suffix_match(tokens, &record.prefix_tokens);
        let entry = by_token
            .entry(record.next_token.clone())
            .or_insert((0.0, 0));
        entry.0 = entry.0.max(suffix);
        entry.1 += 1;
    }
    let mut rows = by_token
        .into_iter()
        .map(|(token, (suffix, count))| {
            let frequency = count as f64 / records.len().max(1) as f64;
            json!({
                "token": token,
                "score": round4((0.75 * suffix) + (0.25 * frequency)),
                "suffix_score": round4(suffix),
                "frequency": round4(frequency)
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(5);
    let top = rows
        .first()
        .and_then(|row| row["token"].as_str())
        .unwrap_or("");
    let top_score = rows
        .first()
        .and_then(|row| row["score"].as_f64())
        .unwrap_or(0.0);
    json!({
        "version": "v74-token-baseline-compare",
        "top_token": top,
        "top_score": round4(top_score),
        "top_k": rows,
        "field_beats_baseline": false,
        "baselines": ["suffix-ngram", "frequency"],
        "read_as": "Cheap next-token baseline. Token Lens reports it explicitly before claiming resonance value."
    })
}

fn llmwave_hot_budget_report(packet: &Packet, capacity: &Value, hot_cycle: &Value) -> Value {
    let pattern_bytes =
        (packet.triads.len() + packet.continuation_memory.len()) * PACKED_PATTERN_BYTES;
    json!({
        "hot_budget_bytes": nanda_6m::BUDGET_BYTES,
        "pattern_arena_bytes": PATTERN_STORE_ARENA_BYTES,
        "pattern_record_bytes": PACKED_PATTERN_BYTES,
        "estimated_pattern_bytes": pattern_bytes,
        "active_patterns": capacity["active_patterns"],
        "fits_pattern_arena": pattern_bytes <= PATTERN_STORE_ARENA_BYTES,
        "hot_cycle_state": hot_cycle["state"],
        "rule": "JSON/text stay cold; packed records, lanes, centroids, and lens readout stay hot."
    })
}

fn llmwave_baseline_compare(tokens: &[String], decode: &Value) -> Value {
    let top_pattern = decode["top_pattern"].as_str().unwrap_or("");
    let lexical = lexical_overlap_with_pattern(tokens, top_pattern);
    let graph_hint = decode["patterns"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item["continuity"].as_f64())
        .unwrap_or(0.0);
    let field_score = decode["patterns"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0);
    json!({
        "version": "v67-lens-baseline-compare",
        "lexical_overlap": round4(lexical),
        "graph_next_edge_hint": round4(graph_hint),
        "field_top_score": round4(field_score),
        "field_beats_lexical_hint": field_score > lexical + 0.05,
        "baselines": ["token-overlap", "graph-continuity", "current-decode"],
        "read_as": "v67 keeps cheap baselines visible before calling a lens readout interesting."
    })
}

fn lexical_overlap_with_pattern(tokens: &[String], pattern: &str) -> f64 {
    let token_set = tokens
        .iter()
        .map(|token| norm(token))
        .filter(|token| token.len() >= 2)
        .collect::<BTreeSet<_>>();
    let pattern_terms = norm(pattern)
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 2)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if token_set.is_empty() || pattern_terms.is_empty() {
        return 0.0;
    }
    token_set.intersection(&pattern_terms).count() as f64 / pattern_terms.len() as f64
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
    #[serde(default)]
    expected_next_token: String,
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
        "version": "v62-demo-raw-text-adapter",
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
    if out.get("input_mode").is_some() {
        println!("input_mode: {}", out["input_mode"].as_str().unwrap_or(""));
    }
    if let Some(adapter) = out.get("raw_adapter") {
        println!(
            "raw_adapter: {} triads={} quality={}",
            adapter["extraction_method"].as_str().unwrap_or(""),
            adapter["triads"].as_u64().unwrap_or(0),
            adapter["quality"].as_str().unwrap_or("")
        );
    }
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
    if out.get("input_mode").is_some() {
        println!(
            "- input_mode: `{}`",
            out["input_mode"].as_str().unwrap_or("")
        );
    }
    if let Some(adapter) = out.get("raw_adapter") {
        println!(
            "- raw_adapter: `{}` triads=`{}` quality=`{}`",
            adapter["extraction_method"].as_str().unwrap_or(""),
            adapter["triads"].as_u64().unwrap_or(0),
            adapter["quality"].as_str().unwrap_or("")
        );
    }
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
