use crate::*;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn proof_cmd(args: ProofArgs) -> Result<u8> {
    if let Some(suite_path) = &args.suite {
        return proof_suite_cmd(&args, suite_path);
    }
    let mut report = proof_report_from_args(&args)?;
    if let Some(focus_out) = &args.focus_out {
        let focus_packet = report
            .get("focused_packet")
            .ok_or_else(|| anyhow::anyhow!("proof report did not include focused_packet"))?;
        if let Some(parent) = focus_out
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(
            focus_out,
            serde_json::to_string_pretty(focus_packet)? + "\n",
        )
        .with_context(|| format!("write {}", focus_out.display()))?;
    }
    if !args.include_focused_packet {
        if let Some(object) = report.as_object_mut() {
            object.remove("focused_packet");
        }
    }
    if let Some(out) = &args.out {
        if let Some(parent) = out.parent().filter(|path| !path.as_os_str().is_empty()) {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(out, serde_json::to_string_pretty(&report)? + "\n")
            .with_context(|| format!("write {}", out.display()))?;
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputFormat::Text => print_proof_text(&report),
        OutputFormat::Md => print_proof_md(&report),
    }
    Ok(match report["proof_state"].as_str().unwrap_or("WATCH") {
        "ANSWER_READY" => EXIT_PASS,
        "VETO" => EXIT_VETO,
        _ => EXIT_WATCH,
    })
}

fn proof_report_from_args(args: &ProofArgs) -> Result<Value> {
    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("nanda proof requires an input path or --suite"))?;
    run_proof_once(
        input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.query_file.as_ref(),
        &args.query_format,
        args.max_triads,
        args.route_cap,
        args.route_triad_cap,
        args.top_k,
        &args.group_by,
        args.sample,
        args.normalize_paths,
        args.include_focused_packet || args.focus_out.is_some(),
        args.fast,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_proof_once(
    input: &Path,
    input_format: &InputFormat,
    task_id: &str,
    domain: &str,
    query_arg: &str,
    query_file: Option<&PathBuf>,
    query_format: &InputFormat,
    max_triads: usize,
    route_cap: usize,
    route_triad_cap: usize,
    top_k: usize,
    group_by: &PeakGroupBy,
    sample: usize,
    normalize_paths: bool,
    include_focused_packet: bool,
    fast: bool,
) -> Result<Value> {
    let mut packet = load_packet_auto(
        input,
        input_format,
        task_id,
        domain,
        query_arg,
        normalize_paths,
    )?;
    let mut query_packet = if let Some(query_file) = query_file {
        load_packet_auto(
            query_file,
            query_format,
            task_id,
            domain,
            query_arg,
            normalize_paths,
        )?
    } else {
        packet.clone()
    };
    inherit_aliases_if_needed(&mut query_packet, &packet);
    let query_text = if !query_arg.trim().is_empty() {
        query_arg.to_string()
    } else if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();

    let memory = normalize_ids(packet.triads.clone(), "m");
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let corpus = corpus_diagnostics(&memory, &query, &packet.query, route_cap);
    let focus_build = focus::build_focused_packet(
        &packet,
        &memory,
        &query,
        query_source,
        max_triads,
        route_cap,
        route_triad_cap,
    );
    let focused_source = normalize_ids(focus_build.packet.triads.clone(), "m");
    let focused_query = normalize_ids(focus_build.packet.candidate_triads.clone(), "q");
    let budget = pack6m::budget_report(&focus_build.packet, &focused_source, &focused_query);
    let pack = pack6m::pack_report(&focus_build.packet, &focused_source, &focused_query, sample);
    let search = interference_search(
        &focus_build.packet,
        &focused_source,
        &focused_query,
        top_k,
        group_by,
        query_source,
        focus_build.metadata.clone(),
    );
    let raw_search = if fast {
        skipped_raw_search_summary(memory.len(), &search)
    } else {
        interference_search(
            &packet,
            &memory,
            &query,
            top_k,
            group_by,
            query_source,
            no_focus_metadata(memory.len()),
        )
    };
    Ok(proof_report(
        &packet,
        &focus_build.packet,
        corpus,
        focus_build.metadata,
        budget,
        pack,
        raw_search,
        search,
        include_focused_packet,
        fast,
    ))
}

pub(crate) fn proof_suite_cmd(args: &ProofArgs, suite_path: &Path) -> Result<u8> {
    let text =
        fs::read_to_string(suite_path).with_context(|| format!("read {}", suite_path.display()))?;
    let suite: ProofSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", suite_path.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow::anyhow!(
            "nanda proof --suite requires at least one case"
        ));
    }
    let base = suite_path.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    for case in &suite.cases {
        let path = resolve_suite_path(base, &case.path);
        let case_query_file = case
            .query_file
            .as_ref()
            .map(|path| resolve_suite_path(base, path));
        let query_file = case_query_file.as_ref().or(args.query_file.as_ref());
        let query = if case.query.trim().is_empty() {
            args.query.as_str()
        } else {
            case.query.as_str()
        };
        let group_by = proof_suite_group_by(&case.group_by, &args.group_by)?;
        let result = run_proof_once(
            &path,
            &args.input_format,
            &args.task_id,
            &args.domain,
            query,
            query_file,
            &args.query_format,
            case.max_triads.unwrap_or(args.max_triads),
            case.route_cap.unwrap_or(args.route_cap),
            case.route_triad_cap.unwrap_or(args.route_triad_cap),
            args.top_k,
            &group_by,
            args.sample,
            args.normalize_paths,
            false,
            args.fast,
        )?;
        let actual_codes = value_string_array(&result["reason_codes"]);
        let state_ok = case.expected_proof_state.is_empty()
            || result["proof_state"].as_str() == Some(&case.expected_proof_state);
        let peak_ok = case.expected_top_peak.is_empty()
            || result["top_peak"].as_str() == Some(&case.expected_top_peak);
        let field_ok = case.expected_field_state.is_empty()
            || result["field_state"].as_str() == Some(&case.expected_field_state);
        let codes_ok = case
            .expected_reason_codes
            .iter()
            .all(|code| actual_codes.contains(code));
        let ok = state_ok && peak_ok && field_ok && codes_ok;
        if ok {
            passed += 1;
        }
        rows.push(json!({
            "id": if case.id.is_empty() { path.display().to_string() } else { case.id.clone() },
            "path": path.display().to_string(),
            "expected_proof_state": case.expected_proof_state,
            "actual_proof_state": result["proof_state"],
            "expected_top_peak": case.expected_top_peak,
            "actual_top_peak": result["top_peak"],
            "expected_field_state": case.expected_field_state,
            "actual_field_state": result["field_state"],
            "expected_reason_codes": case.expected_reason_codes,
            "actual_reason_codes": actual_codes,
            "proof_confidence": result["proof_confidence"],
            "proof_compare": result["proof_compare"]["state"],
            "ok": ok
        }));
    }
    let out = json!({
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "mode": "proof-suite",
        "proof_version": "v27-proof-reason-suite",
        "name": suite.name,
        "passed": passed,
        "total": rows.len(),
        "accuracy": round4(passed as f64 / rows.len().max(1) as f64),
        "cases": rows,
        "read_as": "Proof suite locks expected proof states, peaks, field states, and reason codes across representative packets."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_proof_suite_text(&out),
        OutputFormat::Md => print_proof_suite_md(&out),
    }
    if passed == out["total"].as_u64().unwrap_or(0) as usize {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

fn proof_suite_group_by(value: &str, fallback: &PeakGroupBy) -> Result<PeakGroupBy> {
    match value {
        "" => Ok(fallback.clone()),
        "group" => Ok(PeakGroupBy::Group),
        "route" => Ok(PeakGroupBy::Route),
        other => Err(anyhow::anyhow!("unsupported proof suite group_by: {other}")),
    }
}

fn value_string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|text| text.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
fn proof_report(
    packet: &Packet,
    focused_packet: &Packet,
    corpus: Value,
    focus: Value,
    budget: Value,
    pack: Value,
    raw_search: Value,
    search: Value,
    include_focused_packet: bool,
    fast: bool,
) -> Value {
    let runtime_ready = focus["runtime_contract"]["ready"]
        .as_bool()
        .unwrap_or(false)
        && budget["safe_for_hot_core"].as_bool().unwrap_or(false)
        && pack["packed_ok"].as_bool().unwrap_or(false);
    let search_safe = search["safe_to_answer"].as_bool().unwrap_or(false);
    let packed_safe = pack["peak_decision"]["safe_to_answer"]
        .as_bool()
        .unwrap_or(false);
    let search_verdict = search["verdict"].as_str().unwrap_or("WATCH");
    let field_state = search["field_state"].as_str().unwrap_or("NO_FIELD");
    let proof_state = if search_verdict == "VETO" || field_state == "FIELD_REVERSED" {
        "VETO"
    } else if runtime_ready && search_safe && packed_safe {
        "ANSWER_READY"
    } else {
        "WATCH"
    };
    let safe_to_answer = proof_state == "ANSWER_READY";
    let reason_codes = proof_reason_codes(
        proof_state,
        &corpus,
        &focus,
        &budget,
        &pack,
        &raw_search,
        &search,
    );
    let confidence = proof_confidence(&corpus, &focus, &budget, &pack, &search);
    let compare = proof_compare(&raw_search, &search, &pack);
    let repair = proof_repair(
        proof_state,
        &reason_codes,
        &corpus,
        &focus,
        &budget,
        &pack,
        &search,
    );
    let mut out = json!({
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "mode": "proof-from-focus",
        "proof_version": "v27-proof-reason-suite",
        "proof_mode": if fast { "fast-focused" } else { "full-compare" },
        "proof_state": proof_state,
        "reason_codes": reason_codes,
        "proof_confidence": confidence,
        "proof_compare": compare,
        "verdict": if proof_state == "ANSWER_READY" { "PASS" } else { proof_state },
        "safe_to_answer": safe_to_answer,
        "answer_ready": safe_to_answer,
        "task_id": packet.task_id,
        "domain": packet.domain,
        "query": packet.query,
        "input_memory_size": packet.triads.len(),
        "focused_memory_size": focused_packet.triads.len(),
        "focused_query_size": focused_packet.candidate_triads.len(),
        "top_peak": search["top_peak"],
        "field_state": field_state,
        "peak_margin": search["peak_margin"],
        "runtime_ready": runtime_ready,
        "runtime_contract": {
            "focus": focus["runtime_contract"].clone(),
            "budget": budget["runtime_focus"].clone(),
            "pack": pack["runtime_contract"].clone()
        },
        "hot_proof": {
            "budget_safe": budget["safe_for_hot_core"].as_bool().unwrap_or(false),
            "packed_ok": pack["packed_ok"].as_bool().unwrap_or(false),
            "packed_peak_state": pack["peak_decision"]["state"],
            "packed_peak_safe_to_answer": packed_safe,
            "search_safe_to_answer": search_safe,
            "replay_verdict": pack["packed_replay_decision"]["verdict"],
            "lane_replay_state": pack["packed_lane_replay"]["state"]
        },
        "focus": focus,
        "corpus": corpus,
        "search_summary": {
            "verdict": search["verdict"],
            "field_state": search["field_state"],
            "safe_to_answer": search["safe_to_answer"],
            "top_peak": search["top_peak"],
            "peak_margin": search["peak_margin"],
            "lexical_baseline": search["lexical_baseline"],
            "wins_over_lexical_baseline": search["wins_over_lexical_baseline"],
            "coarse_to_fine": search["coarse_to_fine"],
            "resonant_field": search["resonant_field"],
            "field_interpretation": search["field_interpretation"],
            "peaks": search["peaks"]
        },
        "raw_search_summary": {
            "skipped": raw_search["skipped"],
            "reason": raw_search["reason"],
            "verdict": raw_search["verdict"],
            "field_state": raw_search["field_state"],
            "safe_to_answer": raw_search["safe_to_answer"],
            "top_peak": raw_search["top_peak"],
            "peak_margin": raw_search["peak_margin"],
            "field_interpretation": raw_search["field_interpretation"]
        },
        "pack_summary": {
            "state": pack["state"],
            "packed_records": pack["packed_records"],
            "projection": pack["projection"],
            "peaks": pack["peaks"],
            "packed_support": pack["packed_support"],
            "packed_replay_decision": pack["packed_replay_decision"]
        },
        "budget_summary": budget,
        "repair": repair,
        "read_as": "Proof links corpus diagnostics, route-balanced focus, 6M runtime contract, interference retrieval, and packed peak checks. ANSWER_READY requires both the retrieval field and packed peak to be safe."
    });
    if include_focused_packet {
        out["focused_packet"] = json!(focused_packet);
    }
    out
}

fn skipped_raw_search_summary(memory_size: usize, focused_search: &Value) -> Value {
    json!({
        "skipped": true,
        "reason": "nanda-proof --fast skips full-corpus raw_search after corpus diagnostics and uses focused search plus packed proof as the answer gate.",
        "verdict": "SKIPPED",
        "field_state": "RAW_SEARCH_SKIPPED",
        "safe_to_answer": false,
        "top_peak": Value::Null,
        "peak_margin": Value::Null,
        "input_memory_size": memory_size,
        "focused_top_peak": focused_search["top_peak"].clone(),
        "focused_field_state": focused_search["field_state"].clone(),
        "field_interpretation": {
            "state": "raw_search_skipped",
            "read_as": "Fast proof did not run the unfocused full-corpus field; inspect focused and packed sections instead."
        }
    })
}

fn proof_reason_codes(
    proof_state: &str,
    corpus: &Value,
    focus: &Value,
    budget: &Value,
    pack: &Value,
    raw_search: &Value,
    search: &Value,
) -> Vec<String> {
    let mut codes = vec![];
    if proof_state == "ANSWER_READY" {
        codes.push("PROOF_READY".to_string());
    }
    if corpus["verdict"].as_str() == Some("WATCH") {
        codes.push("CORPUS_WATCH".to_string());
    }
    if !focus["runtime_contract"]["ready"]
        .as_bool()
        .unwrap_or(false)
    {
        codes.push("FOCUS_NOT_READY".to_string());
    }
    if !budget["safe_for_hot_core"].as_bool().unwrap_or(false) {
        codes.push("BUDGET_NOT_SAFE".to_string());
    }
    if !pack["packed_ok"].as_bool().unwrap_or(false) {
        codes.push("PACKED_NOT_OK".to_string());
    }
    if raw_search["skipped"].as_bool().unwrap_or(false) {
        codes.push("RAW_SEARCH_SKIPPED".to_string());
    }
    let raw_peak = raw_search["top_peak"].as_str().unwrap_or("");
    let focused_peak = search["top_peak"].as_str().unwrap_or("");
    if !raw_search["skipped"].as_bool().unwrap_or(false)
        && !raw_peak.is_empty()
        && !focused_peak.is_empty()
        && raw_peak != focused_peak
    {
        codes.push("FOCUS_SHIFTED_PEAK".to_string());
    }
    let field_state = search["field_state"].as_str().unwrap_or("NO_FIELD");
    match field_state {
        "FIELD_REVERSED" => codes.push("FIELD_REVERSED".to_string()),
        "FIELD_NOISY" => codes.push("FIELD_NOISY".to_string()),
        "FIELD_CONTESTED" => codes.push("FIELD_CONTESTED".to_string()),
        "FIELD_THIN" => codes.push("FIELD_THIN".to_string()),
        "FIELD_ROUTE_BALANCED" if !search["safe_to_answer"].as_bool().unwrap_or(false) => {
            codes.push("FIELD_ROUTE_BALANCED_REVIEW".to_string());
        }
        "FIELD_FOCUSED" | "FIELD_SAFE" => {}
        other => codes.push(format!("FIELD_{other}")),
    }
    if !search["safe_to_answer"].as_bool().unwrap_or(false) {
        codes.push("FIELD_NOT_SAFE".to_string());
    }
    match search["resonant_field"]["state"].as_str().unwrap_or("") {
        "WAW_RESONANCE" => codes.push("WAW_RESONANCE".to_string()),
        "RESONANCE_REVERSED" => codes.push("RESONANCE_REVERSED".to_string()),
        "FIELD_LEAKING" => codes.push("RESONANCE_FIELD_LEAKING".to_string()),
        "FIELD_ANTI_DOMINATED" => codes.push("RESONANCE_ANTI_DOMINATED".to_string()),
        "FIELD_DIFFUSE" => codes.push("RESONANCE_FIELD_DIFFUSE".to_string()),
        "RESONANCE_REVIEW" => codes.push("RESONANCE_NOT_READY".to_string()),
        _ => {}
    }
    let packed_state = pack["peak_decision"]["state"]
        .as_str()
        .unwrap_or("PACKED_REVIEW_REQUIRED");
    match packed_state {
        "PACKED_FOCUSED" => {}
        "PACKED_THIN" => codes.push("PACKED_PEAK_THIN".to_string()),
        "PACKED_CONTESTED" => codes.push("PACKED_PEAK_CONTESTED".to_string()),
        "PACKED_NO_PEAK" | "PACKED_EMPTY_MEMORY" | "PACKED_EMPTY_QUERY" => {
            codes.push("PACKED_PEAK_NO_PEAK".to_string());
        }
        other => codes.push(format!("PACKED_PEAK_{other}")),
    }
    if !pack["peak_decision"]["safe_to_answer"]
        .as_bool()
        .unwrap_or(false)
    {
        codes.push("PACKED_PEAK_NOT_SAFE".to_string());
    }
    if search["destructive_interference"]["suppressions"]
        .as_array()
        .map(|items| !items.is_empty())
        .unwrap_or(false)
    {
        codes.push("SHORTCUT_SUPPRESSED".to_string());
    }
    match pack["packed_lane_replay"]["state"].as_str().unwrap_or("") {
        "PACKED_LANE_REPLAY_FOCUSED" => codes.push("REPLAY_FOCUSED".to_string()),
        "PACKED_LANE_REPLAY_PARTIAL" => codes.push("REPLAY_PARTIAL".to_string()),
        _ => {}
    }
    codes.sort();
    codes.dedup();
    codes
}

fn proof_confidence(
    corpus: &Value,
    focus: &Value,
    budget: &Value,
    pack: &Value,
    search: &Value,
) -> Value {
    let corpus_score = if corpus["verdict"].as_str() == Some("WATCH") {
        0.55
    } else {
        1.0
    };
    let focus_score = if focus["runtime_contract"]["ready"]
        .as_bool()
        .unwrap_or(false)
    {
        1.0
    } else {
        0.0
    };
    let budget_score = if budget["safe_for_hot_core"].as_bool().unwrap_or(false) {
        1.0
    } else {
        0.0
    };
    let search_score = match search["field_state"].as_str().unwrap_or("NO_FIELD") {
        "FIELD_SAFE" | "FIELD_FOCUSED" => 1.0,
        "FIELD_ROUTE_BALANCED" => 0.8,
        "FIELD_THIN" => 0.45,
        "FIELD_CONTESTED" => 0.35,
        "FIELD_NOISY" => 0.25,
        "FIELD_REVERSED" => 0.0,
        _ => 0.2,
    };
    let packed_score = match pack["peak_decision"]["state"]
        .as_str()
        .unwrap_or("PACKED_REVIEW_REQUIRED")
    {
        "PACKED_FOCUSED" => 1.0,
        "PACKED_THIN" => 0.45,
        "PACKED_CONTESTED" => 0.35,
        "PACKED_NO_PEAK" | "PACKED_EMPTY_MEMORY" | "PACKED_EMPTY_QUERY" => 0.0,
        _ => 0.2,
    };
    let replay_score = match pack["packed_lane_replay"]["state"].as_str().unwrap_or("") {
        "PACKED_LANE_REPLAY_FOCUSED" => 0.75,
        "PACKED_LANE_REPLAY_PARTIAL" => 0.55,
        _ => 0.5,
    };
    let resonance_score = match search["resonant_field"]["state"].as_str().unwrap_or("") {
        "WAW_RESONANCE" => 1.0,
        "RESONANCE_REVIEW" => 0.62,
        "FIELD_LEAKING" => 0.42,
        "FIELD_DIFFUSE" | "FIELD_ANTI_DOMINATED" => 0.25,
        "RESONANCE_REVERSED" => 0.0,
        _ => 0.5,
    };
    let score = round4(
        (corpus_score
            + focus_score
            + budget_score
            + search_score
            + packed_score
            + replay_score
            + resonance_score)
            / 7.0,
    );
    json!({
        "score": score,
        "state": if score >= 0.82 { "HIGH" } else if score >= 0.62 { "MEDIUM" } else { "LOW" },
        "components": {
            "corpus": corpus_score,
            "focus": focus_score,
            "budget": budget_score,
            "search": search_score,
            "packed": packed_score,
            "replay": replay_score,
            "resonance": resonance_score
        },
        "read_as": "Structural confidence is a review aid, not a PASS condition. ANSWER_READY still requires safe search and safe packed peak."
    })
}

fn proof_compare(raw_search: &Value, focused_search: &Value, pack: &Value) -> Value {
    let raw_skipped = raw_search["skipped"].as_bool().unwrap_or(false);
    let raw_peak = raw_search["top_peak"].as_str().unwrap_or("");
    let focused_peak = focused_search["top_peak"].as_str().unwrap_or("");
    let raw_safe = raw_search["safe_to_answer"].as_bool().unwrap_or(false);
    let focused_safe = focused_search["safe_to_answer"].as_bool().unwrap_or(false);
    let packed_safe = pack["peak_decision"]["safe_to_answer"]
        .as_bool()
        .unwrap_or(false);
    let packed_state = pack["peak_decision"]["state"]
        .as_str()
        .unwrap_or("PACKED_REVIEW_REQUIRED");
    let state = if raw_skipped && focused_safe && packed_safe {
        "FOCUSED_PACKED_ALIGNED"
    } else if raw_skipped {
        "FOCUSED_ONLY_REVIEW"
    } else if raw_peak != focused_peak {
        "FOCUS_SHIFTED_PEAK"
    } else if focused_safe && !packed_safe {
        "PACKED_DISAGREES"
    } else if raw_safe == focused_safe && focused_safe == packed_safe {
        "ALIGNED"
    } else {
        "REVIEW"
    };
    json!({
        "state": state,
        "raw": {
            "skipped": raw_skipped,
            "reason": raw_search["reason"],
            "top_peak": raw_search["top_peak"],
            "field_state": raw_search["field_state"],
            "safe_to_answer": raw_safe,
            "peak_margin": raw_search["peak_margin"]
        },
        "focused": {
            "top_peak": focused_search["top_peak"],
            "field_state": focused_search["field_state"],
            "safe_to_answer": focused_safe,
            "peak_margin": focused_search["peak_margin"]
        },
        "packed": {
            "peak_state": packed_state,
            "safe_to_answer": packed_safe,
            "route": pack["peaks"]["route"].clone(),
            "group": pack["peaks"]["group"].clone()
        },
        "read_as": if raw_skipped {
            "Fast proof skipped the unfocused raw field; focused search shows the proof window and packed shows whether the 6M bridge agrees strongly enough."
        } else {
            "Raw search shows the unfocused corpus pull; focused search shows the proof window; packed shows whether the 6M bridge agrees strongly enough."
        }
    })
}

fn proof_repair(
    proof_state: &str,
    reason_codes: &[String],
    corpus: &Value,
    focus: &Value,
    budget: &Value,
    pack: &Value,
    search: &Value,
) -> Value {
    let mut tasks = vec![];
    if corpus["verdict"].as_str() == Some("WATCH") {
        tasks.push("Corpus doctor returned WATCH; inspect warnings before trusting the peak.");
    }
    if !focus["runtime_contract"]["ready"]
        .as_bool()
        .unwrap_or(false)
    {
        tasks.push(
            "Focused packet is not runtime-ready; reduce max triads, routes, or field width.",
        );
    }
    if !budget["safe_for_hot_core"].as_bool().unwrap_or(false) {
        tasks.push("NANDA-6M budget is not safe for hot core; split or focus further.");
    }
    if !pack["packed_ok"].as_bool().unwrap_or(false) {
        tasks
            .push("Packed bridge is not OK; inspect dictionaries, blockers, and runtime contract.");
    }
    if !search["safe_to_answer"].as_bool().unwrap_or(false) {
        tasks.push("Interference search did not mark the field safe; inspect support, anti-triads, and coarse-to-fine path.");
    }
    if !pack["peak_decision"]["safe_to_answer"]
        .as_bool()
        .unwrap_or(false)
    {
        tasks.push("Packed peak is not safe; treat the result as a retrieval hint, not an answer.");
    }
    json!({
        "required": proof_state != "ANSWER_READY",
        "reason_codes": reason_codes,
        "tasks": tasks,
        "next": if proof_state == "ANSWER_READY" {
            "Use the top peak with cited supporting triads."
        } else if proof_state == "VETO" {
            "Stop and repair route direction or conflicting structure."
        } else {
            "Answer only after local review, stronger query triads, or a narrower focus packet."
        }
    })
}

fn print_proof_text(out: &Value) {
    println!("NANDA PROOF");
    println!("state: {}", out["proof_state"].as_str().unwrap_or(""));
    println!("top_peak: {}", out["top_peak"].as_str().unwrap_or(""));
    println!("field: {}", out["field_state"].as_str().unwrap_or(""));
    println!(
        "confidence: {} ({})",
        out["proof_confidence"]["score"].as_f64().unwrap_or(0.0),
        out["proof_confidence"]["state"].as_str().unwrap_or("")
    );
    println!(
        "focus: {} -> {} triads",
        out["input_memory_size"].as_u64().unwrap_or(0),
        out["focused_memory_size"].as_u64().unwrap_or(0)
    );
    println!(
        "runtime_ready: {}",
        out["runtime_ready"].as_bool().unwrap_or(false)
    );
    println!(
        "safe_to_answer: {}",
        out["safe_to_answer"].as_bool().unwrap_or(false)
    );
    if let Some(codes) = out["reason_codes"].as_array() {
        println!(
            "reason_codes: {}",
            codes
                .iter()
                .filter_map(|code| code.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if let Some(tasks) = out["repair"]["tasks"].as_array() {
        for task in tasks {
            println!("- {}", task.as_str().unwrap_or(""));
        }
    }
}

fn print_proof_md(out: &Value) {
    println!("# NANDA Proof\n");
    println!("- state: `{}`", out["proof_state"].as_str().unwrap_or(""));
    println!("- top_peak: `{}`", out["top_peak"].as_str().unwrap_or(""));
    println!("- field: `{}`", out["field_state"].as_str().unwrap_or(""));
    println!(
        "- confidence: `{}` `{}`",
        out["proof_confidence"]["score"].as_f64().unwrap_or(0.0),
        out["proof_confidence"]["state"].as_str().unwrap_or("")
    );
    println!(
        "- focus: `{}` -> `{}` triads",
        out["input_memory_size"].as_u64().unwrap_or(0),
        out["focused_memory_size"].as_u64().unwrap_or(0)
    );
    println!(
        "- runtime_ready: `{}`",
        out["runtime_ready"].as_bool().unwrap_or(false)
    );
    println!(
        "- safe_to_answer: `{}`",
        out["safe_to_answer"].as_bool().unwrap_or(false)
    );
    if let Some(codes) = out["reason_codes"].as_array() {
        println!(
            "- reason_codes: `{}`",
            codes
                .iter()
                .filter_map(|code| code.as_str())
                .collect::<Vec<_>>()
                .join("`, `")
        );
    }
    if let Some(tasks) = out["repair"]["tasks"].as_array() {
        println!("\n## Repair\n");
        for task in tasks {
            println!("- {}", task.as_str().unwrap_or(""));
        }
    }
}

fn print_proof_suite_text(out: &Value) {
    println!("NANDA PROOF SUITE");
    println!("name: {}", out["name"].as_str().unwrap_or(""));
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {} ok={} state={} peak={} reasons={}",
                case["id"].as_str().unwrap_or(""),
                case["ok"].as_bool().unwrap_or(false),
                case["actual_proof_state"].as_str().unwrap_or(""),
                case["actual_top_peak"].as_str().unwrap_or(""),
                value_string_array(&case["actual_reason_codes"]).join(",")
            );
        }
    }
}

fn print_proof_suite_md(out: &Value) {
    println!("# NANDA Proof Suite\n");
    println!("- name: `{}`", out["name"].as_str().unwrap_or(""));
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        println!("\n## Cases\n");
        for case in cases {
            println!(
                "- `{}` ok=`{}` state=`{}` peak=`{}` reasons=`{}`",
                case["id"].as_str().unwrap_or(""),
                case["ok"].as_bool().unwrap_or(false),
                case["actual_proof_state"].as_str().unwrap_or(""),
                case["actual_top_peak"].as_str().unwrap_or(""),
                value_string_array(&case["actual_reason_codes"]).join("`, `")
            );
        }
    }
}
