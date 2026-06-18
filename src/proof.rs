use crate::*;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;

pub(crate) fn proof_cmd(args: ProofArgs) -> Result<u8> {
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
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
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

    let memory = normalize_ids(packet.triads.clone(), "m");
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let corpus = corpus_diagnostics(&memory, &query, &packet.query, args.route_cap);
    let focus_build = focus::build_focused_packet(
        &packet,
        &memory,
        &query,
        query_source,
        args.max_triads,
        args.route_cap,
        args.route_triad_cap,
    );
    let focused_source = normalize_ids(focus_build.packet.triads.clone(), "m");
    let focused_query = normalize_ids(focus_build.packet.candidate_triads.clone(), "q");
    let budget = pack6m::budget_report(&focus_build.packet, &focused_source, &focused_query);
    let pack = pack6m::pack_report(
        &focus_build.packet,
        &focused_source,
        &focused_query,
        args.sample,
    );
    let search = interference_search(
        &focus_build.packet,
        &focused_source,
        &focused_query,
        args.top_k,
        &args.group_by,
        query_source,
        focus_build.metadata.clone(),
    );
    Ok(proof_report(
        &packet,
        &focus_build.packet,
        corpus,
        focus_build.metadata,
        budget,
        pack,
        search,
        args.include_focused_packet || args.focus_out.is_some(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn proof_report(
    packet: &Packet,
    focused_packet: &Packet,
    corpus: Value,
    focus: Value,
    budget: Value,
    pack: Value,
    search: Value,
    include_focused_packet: bool,
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
    let repair = proof_repair(proof_state, &corpus, &focus, &budget, &pack, &search);
    let mut out = json!({
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "mode": "proof-from-focus",
        "proof_version": "v26-hot-proof-report",
        "proof_state": proof_state,
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
            "field_interpretation": search["field_interpretation"],
            "peaks": search["peaks"]
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

fn proof_repair(
    proof_state: &str,
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
    if let Some(tasks) = out["repair"]["tasks"].as_array() {
        println!("\n## Repair\n");
        for task in tasks {
            println!("- {}", task.as_str().unwrap_or(""));
        }
    }
}
