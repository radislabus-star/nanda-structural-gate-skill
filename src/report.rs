use crate::*;
use std::io;

pub(crate) fn make_report(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
) -> Result<Report> {
    let complexity = complexity_score(source, candidates);
    let gaps = evidence_gaps(source, candidates);
    let weak_conf = low_confidence(source, candidates);
    let conflicts = symbolic_conflicts(source, candidates);
    let limits = limit_warnings(source, candidates, packet);
    let wave = score_candidates(source, candidates);
    let routes = route_coherence(source, candidates);
    let structural_map = structural_map(source, candidates);
    let baselines = baseline_summary(source, candidates);

    let has_foreign_pull = structural_map["foreign_pull"]
        .as_array()
        .is_some_and(|items| !items.is_empty());

    let alias_watch = packet.canonicalization.conflict_count > 0
        || packet
            .canonicalization
            .warnings
            .iter()
            .any(|item| item.contains("low confidence") || item.contains("empty canonical"));

    let has_weak_route = routes["weak"].as_array().is_some_and(|x| !x.is_empty());
    let has_weak_wave =
        !candidates.is_empty() && wave["weak"].as_array().is_some_and(|x| !x.is_empty());
    let has_candidate_watch = alias_watch || !gaps.is_empty() || !weak_conf.is_empty();

    let verdict = if limits.iter().any(|x| x.contains("hard limit")) {
        "WATCH"
    } else if !conflicts.is_empty() || has_foreign_pull {
        "VETO"
    } else if has_candidate_watch {
        "WATCH"
    } else if has_weak_route || has_weak_wave {
        "VETO"
    } else if (complexity < MANDATORY_COMPLEXITY && candidates.is_empty()) || source.is_empty() {
        "WATCH"
    } else {
        "PASS"
    }
    .to_string();

    let mut weak_ids: BTreeSet<String> = BTreeSet::new();
    for value in wave["weak"].as_array().into_iter().flatten() {
        if let Some(id) = value.as_str() {
            weak_ids.insert(id.to_string());
        }
    }
    for item in gaps.iter().chain(weak_conf.iter()) {
        weak_ids.insert(item.clone());
    }
    let stable: Vec<String> = if wave["stable"].as_array().is_some_and(|x| !x.is_empty()) {
        wave["stable"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect()
    } else {
        source
            .iter()
            .filter(|triad| !gaps.contains(&triad.id))
            .map(|triad| triad.id.clone())
            .collect()
    };

    let mut report = Report {
        verdict,
        engine: ENGINE_ID.to_string(),
        core_version: CORE_VERSION.to_string(),
        wave_dim: WAVE_DIM,
        task_id: packet.task_id.clone(),
        domain: packet.domain.clone(),
        complexity_score: complexity,
        mandatory_gate: complexity >= MANDATORY_COMPLEXITY,
        limits,
        stable_triads: stable,
        weak_triads: weak_ids.into_iter().collect(),
        conflicts,
        evidence_gaps: gaps,
        canonicalization: packet.canonicalization.clone(),
        baseline_summary: baselines,
        wave_summary: wave,
        route_coherence: routes,
        structural_map,
        explanation: vec![],
        repair_prompt: String::new(),
        trace_path: String::new(),
    };
    report.explanation = build_explanation(&report);
    report.repair_prompt = build_repair_prompt(&report);
    report.trace_path = write_trace(&report)?;
    Ok(report)
}

pub(crate) fn print_report(report: &Report, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => {
            println!("verdict: {}", report.verdict);
            println!("engine: {}", report.engine);
            println!("task_id: {}", report.task_id);
            println!("complexity_score: {}", report.complexity_score);
            println!("mandatory_gate: {}", report.mandatory_gate);
            if report.canonicalization.enabled {
                println!(
                    "canonicalization: applied={} conflicts={} warnings={}",
                    report.canonicalization.applied_count,
                    report.canonicalization.conflict_count,
                    report.canonicalization.watch_count
                );
                for item in &report.canonicalization.applied {
                    println!(
                        "  - {} {}: {} -> {}",
                        item.triad_id, item.field, item.from, item.to
                    );
                }
            }
            for (label, items) in [
                ("conflicts", &report.conflicts),
                ("evidence_gaps", &report.evidence_gaps),
                ("weak_triads", &report.weak_triads),
                ("explanation", &report.explanation),
            ] {
                if !items.is_empty() {
                    println!("{label}:");
                    for item in items {
                        println!("  - {item}");
                    }
                }
            }
            if report.verdict != "PASS" {
                println!("repair:");
                for line in report.repair_prompt.lines() {
                    println!("  {line}");
                }
            }
            println!("trace_path: {}", report.trace_path);
        }
        OutputFormat::Md => {
            println!("# NANDA Report\n");
            println!("- verdict: `{}`", report.verdict);
            println!("- action: `{}`", action_for_report(report));
            println!("- complexity: `{}`", report.complexity_score);
            if report.canonicalization.enabled {
                println!(
                    "- canonicalization: `applied={} conflicts={} warnings={}`",
                    report.canonicalization.applied_count,
                    report.canonicalization.conflict_count,
                    report.canonicalization.watch_count
                );
            }
            println!("- trace: `{}`", report.trace_path);
        }
    }
    Ok(())
}

pub(crate) fn print_waw_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("suite: {}", out["suite"].as_str().unwrap_or(""));
    println!("passed: {}/{}", out["passed"], out["total"]);
    println!("waw_score: {}", out["waw_score"]);
    println!("structural_wins: {}", out["structural_wins"]);
    println!("lexical_traps: {}", out["lexical_traps"]);
    println!("explainable_drifts: {}", out["explainable_drifts"]);
}

pub(crate) fn print_waw_md(out: &Value) {
    println!("# NANDA WAW Benchmark\n");
    println!("- core: `{}`", out["core_version"].as_str().unwrap_or(""));
    println!("- suite: `{}`", out["suite"].as_str().unwrap_or(""));
    println!("- passed: `{}/{}`", out["passed"], out["total"]);
    println!("- waw_score: `{}`", out["waw_score"]);
    println!("- structural_wins: `{}`", out["structural_wins"]);
    println!("- lexical_traps: `{}`", out["lexical_traps"]);
    println!("- explainable_drifts: `{}`", out["explainable_drifts"]);
}

pub(crate) fn print_eval_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    println!("accuracy: {}", out["accuracy"].as_f64().unwrap_or(0.0));
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {} ok={} peak={}/{} state={}/{}",
                case["case"].as_str().unwrap_or(""),
                case["ok"].as_bool().unwrap_or(false),
                case["actual_peak"].as_str().unwrap_or(""),
                case["expected_peak"].as_str().unwrap_or(""),
                case["actual_state"].as_str().unwrap_or(""),
                case["expected_state"].as_str().unwrap_or("")
            );
        }
    }
}

pub(crate) fn print_eval_md(out: &Value) {
    println!("# NANDA Eval\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- passed: `{}/{}`", out["passed"], out["total"]);
    println!("- accuracy: `{}`", out["accuracy"]);
}

pub(crate) fn serve_cmd(args: ServeArgs) -> Result<u8> {
    match args.format {
        ServeFormat::Jsonl => serve_jsonl(),
    }
}

pub(crate) fn serve_jsonl() -> Result<u8> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match handle_serve_request(&line) {
            Ok(result) => json!({"ok": true, "result": result}),
            Err(err) => json!({"ok": false, "error": format!("{err:#}")}),
        };
        println!("{}", serde_json::to_string(&response)?);
    }
    Ok(EXIT_PASS)
}

pub(crate) fn handle_serve_request(line: &str) -> Result<Value> {
    let request: Value = serde_json::from_str(line).context("parse serve request JSON")?;
    let command = request["command"]
        .as_str()
        .ok_or_else(|| anyhow!("serve request requires string field command"))?;
    match command {
        "doctor" => Ok(doctor_value()),
        "search" => {
            let packet_value = request
                .get("packet")
                .ok_or_else(|| anyhow!("search request requires packet"))?;
            let packet: Packet = serde_json::from_value(packet_value.clone())?;
            let top_k = request["top_k"].as_u64().unwrap_or(5) as usize;
            let group_by = match request["group_by"].as_str().unwrap_or("route") {
                "group" => PeakGroupBy::Group,
                "route" => PeakGroupBy::Route,
                other => return Err(anyhow!("unsupported group_by: {other}")),
            };
            let (query, query_source) = search_query_triads(&packet, &packet.query);
            Ok(interference_search(
                &packet,
                &normalize_ids(packet.triads.clone(), "m"),
                &query,
                top_k,
                &group_by,
                query_source,
                no_focus_metadata(packet.triads.len()),
            ))
        }
        "check" => {
            let packet_value = request
                .get("packet")
                .ok_or_else(|| anyhow!("check request requires packet"))?;
            let packet: Packet = serde_json::from_value(packet_value.clone())?;
            let source = normalize_ids(packet.triads.clone(), "t");
            let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
            Ok(serde_json::to_value(make_report(
                &packet,
                &source,
                &candidates,
            )?)?)
        }
        "dataset-doctor" | "dataset_doctor" => {
            let packet_value = request
                .get("packet")
                .ok_or_else(|| anyhow!("dataset-doctor request requires packet"))?;
            let packet: Packet = serde_json::from_value(packet_value.clone())?;
            let route_cap = request["route_cap"].as_u64().unwrap_or(256) as usize;
            Ok(corpus_diagnostics(
                &normalize_ids(packet.triads.clone(), "m"),
                &normalize_ids(packet.candidate_triads.clone(), "q"),
                &packet.query,
                route_cap,
            ))
        }
        other => Err(anyhow!("unsupported serve command: {other}")),
    }
}

pub(crate) fn doctor_cmd(args: DoctorArgs) -> Result<u8> {
    let out = doctor_value();
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_doctor_text(&out),
        OutputFormat::Md => print_doctor_md(&out),
    }
    if out["healthy"].as_bool().unwrap_or(false) {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

pub(crate) fn doctor_value() -> Value {
    let route_trap = builtin_route_trap_packet(false);
    let trap_result = interference_search(
        &route_trap,
        &normalize_ids(route_trap.triads.clone(), "m"),
        &normalize_ids(route_trap.candidate_triads.clone(), "q"),
        3,
        &PeakGroupBy::Route,
        "candidate_triads",
        no_focus_metadata(route_trap.triads.len()),
    );
    let noisy = builtin_route_trap_packet(true);
    let noisy_result = interference_search(
        &noisy,
        &normalize_ids(noisy.triads.clone(), "m"),
        &normalize_ids(noisy.candidate_triads.clone(), "q"),
        3,
        &PeakGroupBy::Route,
        "candidate_triads",
        no_focus_metadata(noisy.triads.len()),
    );
    let trap_field_state = trap_result["field_state_machine"]["state"]
        .as_str()
        .unwrap_or("");
    let noisy_field_state = noisy_result["field_state_machine"]["state"]
        .as_str()
        .unwrap_or("");
    let trap_ok = trap_result["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["peak"].as_str())
        == Some("certification")
        && trap_result["peak_decision"]["state"].as_str() == Some("FOCUSED")
        && trap_field_state == "FIELD_FOCUSED"
        && trap_result["wins_over_lexical_baseline"]
            .as_bool()
            .unwrap_or(false);
    let noisy_ok = noisy_result["peak_decision"]["state"].as_str() == Some("WATCH")
        && noisy_field_state == "FIELD_CONTESTED"
        && !noisy_result["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(true);
    let healthy = trap_ok && noisy_ok;
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "doctor",
        "healthy": healthy,
        "checks": {
            "route_trap_focused": trap_ok,
            "noisy_query_watch": noisy_ok,
            "field_state_machine": trap_field_state == "FIELD_FOCUSED" && noisy_field_state == "FIELD_CONTESTED"
        },
        "route_trap": {
            "top": trap_result["peaks"].as_array().and_then(|peaks| peaks.first()).and_then(|peak| peak["peak"].as_str()).unwrap_or(""),
            "state": trap_result["peak_decision"]["state"],
            "field_state": trap_result["field_state_machine"]["state"],
            "field_action": trap_result["field_state_machine"]["action"],
            "safe_to_answer": trap_result["peak_decision"]["safe_to_answer"],
            "field_safe_to_answer": trap_result["field_state_machine"]["safe_to_answer"],
            "lexical_baseline_top": trap_result["lexical_baseline"]["top_peak"],
            "wins_over_lexical_baseline": trap_result["wins_over_lexical_baseline"],
            "peak_margin": trap_result["peak_margin"]
        },
        "noisy": {
            "top": noisy_result["peaks"].as_array().and_then(|peaks| peaks.first()).and_then(|peak| peak["peak"].as_str()).unwrap_or(""),
            "state": noisy_result["peak_decision"]["state"],
            "field_state": noisy_result["field_state_machine"]["state"],
            "field_action": noisy_result["field_state_machine"]["action"],
            "safe_to_answer": noisy_result["peak_decision"]["safe_to_answer"],
            "field_safe_to_answer": noisy_result["field_state_machine"]["safe_to_answer"],
            "peak_margin": noisy_result["peak_margin"]
        }
    })
}

pub(crate) fn print_doctor_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("healthy: {}", out["healthy"].as_bool().unwrap_or(false));
    println!(
        "route_trap: top={} state={} field={} lexical={} wins={}",
        out["route_trap"]["top"].as_str().unwrap_or(""),
        out["route_trap"]["state"].as_str().unwrap_or(""),
        out["route_trap"]["field_state"].as_str().unwrap_or(""),
        out["route_trap"]["lexical_baseline_top"]
            .as_str()
            .unwrap_or(""),
        out["route_trap"]["wins_over_lexical_baseline"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "noisy: top={} state={} field={} safe={}",
        out["noisy"]["top"].as_str().unwrap_or(""),
        out["noisy"]["state"].as_str().unwrap_or(""),
        out["noisy"]["field_state"].as_str().unwrap_or(""),
        out["noisy"]["safe_to_answer"].as_bool().unwrap_or(false)
    );
}

pub(crate) fn print_doctor_md(out: &Value) {
    println!("# NANDA Doctor\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- healthy: `{}`", out["healthy"]);
    println!(
        "- route_trap: `{}` / `{}` / `{}`",
        out["route_trap"]["top"], out["route_trap"]["state"], out["route_trap"]["field_state"]
    );
    println!(
        "- noisy: `{}` / `{}` / `{}`",
        out["noisy"]["top"], out["noisy"]["state"], out["noisy"]["field_state"]
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn doctor_triad(
    id: &str,
    subject: &str,
    relation: &str,
    object: &str,
    subject_role: &str,
    object_role: &str,
    route: &str,
    group: &str,
) -> Triad {
    Triad {
        id: id.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        evidence: "builtin-doctor".to_string(),
        confidence: 0.9,
        subject_role: subject_role.to_string(),
        object_role: object_role.to_string(),
        route: route.to_string(),
        group: group.to_string(),
    }
}

pub(crate) fn report_cmd(args: ReportArgs) -> Result<u8> {
    let overall_packet = packet_from_markdown(&args.overall, "overall", &args.domain, "", false)?;
    let overall = make_report(
        &overall_packet,
        &normalize_ids(overall_packet.triads.clone(), "t"),
        &normalize_ids(overall_packet.candidate_triads.clone(), "c"),
    )?;
    let mut route_reports = serde_json::Map::new();
    let mut worst = verdict_code(&overall.verdict);
    for route in args.routes {
        let (name, path) = route
            .split_once(':')
            .ok_or_else(|| anyhow!("--route must be name:path"))?;
        let packet = packet_from_markdown(Path::new(path), name, &args.domain, "", false)?;
        let report = make_report(
            &packet,
            &normalize_ids(packet.triads.clone(), "t"),
            &normalize_ids(packet.candidate_triads.clone(), "c"),
        )?;
        worst = worst_status(worst, verdict_code(&report.verdict));
        route_reports.insert(name.to_string(), serde_json::to_value(&report)?);
    }
    let action =
        if route_reports.values().any(|v| v["verdict"] == "VETO") || overall.verdict == "VETO" {
            "REPAIR_REQUIRED"
        } else if overall.verdict == "WATCH" {
            "DRAFT_OK_REVIEW_REQUIRED"
        } else {
            "SEND_OK"
        };
    let packet = json!({
        "title": args.title,
        "action": action,
        "safe_to_draft": action != "REPAIR_REQUIRED",
        "safe_to_send": action == "SEND_OK",
        "blocking": action == "REPAIR_REQUIRED",
        "review_required": action != "SEND_OK",
        "overall": overall,
        "routes": route_reports,
        "repair_prompts": [],
        "next_prompt": if action == "SEND_OK" { "Finalize with checked structure." } else { "Repair or split unresolved routes before final send." }
    });
    match args.format {
        OutputFormat::Json | OutputFormat::Text => {
            println!("{}", serde_json::to_string_pretty(&packet)?)
        }
        OutputFormat::Md => {
            println!("# {}\n", packet["title"].as_str().unwrap_or("NANDA Report"));
            println!("- action: `{}`", action);
            println!("- safe_to_draft: `{}`", packet["safe_to_draft"]);
            println!("- safe_to_send: `{}`", packet["safe_to_send"]);
        }
    }
    if action == "REPAIR_REQUIRED" {
        Ok(EXIT_VETO)
    } else if action == "DRAFT_OK_REVIEW_REQUIRED" {
        Ok(EXIT_WATCH)
    } else {
        Ok(worst)
    }
}

pub(crate) fn write_or_print(path: PathBuf, stdout: bool, output: String) -> Result<()> {
    if stdout {
        print!("{output}");
    } else {
        fs::write(&path, output)?;
        println!("{}", path.display());
    }
    Ok(())
}

pub(crate) fn action_for_report(report: &Report) -> &'static str {
    match report.verdict.as_str() {
        "PASS" => "SEND_OK",
        "WATCH" => "DRAFT_OK_REVIEW_REQUIRED",
        _ => "REPAIR_REQUIRED",
    }
}
