use crate::*;

pub(crate) fn eval_cmd(args: EvalArgs) -> Result<u8> {
    let cases = eval_cases_from_args(&args)?;
    if cases.is_empty() {
        return Err(anyhow!(
            "nanda eval requires at least one --case path:expected_peak:expected_state or --suite file.json"
        ));
    }
    let mut rows = vec![];
    let mut passed = 0usize;
    for (path, expected_peak, expected_state) in cases {
        let packet = load_packet_auto(
            &path,
            &args.input_format,
            "eval",
            "general",
            "",
            args.normalize_paths,
        )?;
        let memory = normalize_ids(packet.triads.clone(), "m");
        let query = normalize_ids(packet.candidate_triads.clone(), "q");
        let result = interference_search(
            &packet,
            &memory,
            &query,
            args.top_k,
            &args.group_by,
            "candidate_triads",
            no_focus_metadata(memory.len()),
        );
        let actual_peak = result["peaks"]
            .as_array()
            .and_then(|peaks| peaks.first())
            .and_then(|peak| peak["peak"].as_str())
            .unwrap_or("")
            .to_string();
        let actual_state = result["peak_decision"]["state"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let ok = actual_peak == expected_peak && actual_state == expected_state;
        if ok {
            passed += 1;
        }
        rows.push(json!({
            "case": path.display().to_string(),
            "expected_peak": expected_peak,
            "actual_peak": actual_peak,
            "expected_state": expected_state,
            "actual_state": actual_state,
            "ok": ok,
            "peak_margin": result["peak_margin"],
            "safe_to_answer": result["peak_decision"]["safe_to_answer"],
            "wins_over_lexical_baseline": result["wins_over_lexical_baseline"]
        }));
    }
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "eval-suite",
        "passed": passed,
        "total": rows.len(),
        "accuracy": round4(passed as f64 / rows.len().max(1) as f64),
        "cases": rows
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_eval_text(&out),
        OutputFormat::Md => print_eval_md(&out),
    }
    if passed == out["total"].as_u64().unwrap_or(0) as usize {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

pub(crate) fn eval_cases_from_args(args: &EvalArgs) -> Result<Vec<(PathBuf, String, String)>> {
    let mut cases = vec![];
    for raw_case in &args.cases {
        cases.push(parse_eval_case(raw_case)?);
    }
    if let Some(suite_path) = &args.suite {
        let text = fs::read_to_string(suite_path)
            .with_context(|| format!("read {}", suite_path.display()))?;
        let suite: EvalSuite = serde_json::from_str(&text)
            .with_context(|| format!("parse {}", suite_path.display()))?;
        let base = suite_path.parent().unwrap_or_else(|| Path::new("."));
        for case in suite.cases {
            let path = if case.path.is_absolute() {
                case.path
            } else {
                base.join(case.path)
            };
            cases.push((path, case.expected_peak, case.expected_state));
        }
    }
    Ok(cases)
}

pub(crate) fn parse_eval_case(raw: &str) -> Result<(PathBuf, String, String)> {
    let mut parts = raw.rsplitn(3, ':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(anyhow!(
            "--case must be path:expected_peak:expected_state, got {raw}"
        ));
    }
    parts.reverse();
    Ok((
        PathBuf::from(parts[0]),
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

pub(crate) fn waw_cmd(args: WawArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.suite)
        .with_context(|| format!("read {}", args.suite.display()))?;
    let suite: WawSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", args.suite.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow!("nanda waw requires a suite with at least one case"));
    }
    let base = args.suite.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    let mut structural_wins = 0usize;
    let mut lexical_traps = 0usize;
    let mut safe_answers = 0usize;
    let mut explainable_drifts = 0usize;
    for case in suite.cases {
        let path = if case.path.is_absolute() {
            case.path.clone()
        } else {
            base.join(&case.path)
        };
        let packet = load_packet_auto(
            &path,
            &args.input_format,
            "waw",
            "general",
            "",
            args.normalize_paths,
        )?;
        let memory = normalize_ids(packet.triads.clone(), "m");
        let query = normalize_ids(packet.candidate_triads.clone(), "q");
        let result = interference_search(
            &packet,
            &memory,
            &query,
            args.top_k,
            &args.group_by,
            "candidate_triads",
            no_focus_metadata(memory.len()),
        );
        let actual_peak = result["peaks"]
            .as_array()
            .and_then(|peaks| peaks.first())
            .and_then(|peak| peak["peak"].as_str())
            .unwrap_or("")
            .to_string();
        let actual_state = result["peak_decision"]["state"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let lexical_peak = result["lexical_baseline"]["top_peak"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let wins = result["wins_over_lexical_baseline"]
            .as_bool()
            .unwrap_or(false);
        let trap = result["field_interpretation"]["lexical_trap_detected"]
            .as_bool()
            .unwrap_or(false);
        let safe = result["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(false);
        let route_drift = result["field_interpretation"]["centroid_drift"]["route"]["changed"]
            .as_bool()
            .unwrap_or(false);
        let relation_drift = result["field_interpretation"]["centroid_drift"]["relation"]
            ["changed"]
            .as_bool()
            .unwrap_or(false);
        let explainable = trap && (route_drift || relation_drift);
        if wins {
            structural_wins += 1;
        }
        if trap {
            lexical_traps += 1;
        }
        if safe {
            safe_answers += 1;
        }
        if explainable {
            explainable_drifts += 1;
        }
        let lexical_ok =
            case.expected_lexical_peak.is_empty() || lexical_peak == case.expected_lexical_peak;
        let trap_ok = !case.require_lexical_trap || trap;
        let safe_ok = !case.require_safe_to_answer || safe;
        let ok = actual_peak == case.expected_peak
            && actual_state == case.expected_state
            && lexical_ok
            && wins
            && trap_ok
            && safe_ok
            && explainable;
        if ok {
            passed += 1;
        }
        rows.push(json!({
            "id": if case.id.is_empty() { path.display().to_string() } else { case.id },
            "case": path.display().to_string(),
            "expected_peak": case.expected_peak,
            "actual_peak": actual_peak,
            "expected_state": case.expected_state,
            "actual_state": actual_state,
            "expected_lexical_peak": case.expected_lexical_peak,
            "actual_lexical_peak": lexical_peak,
            "wins_over_lexical_baseline": wins,
            "lexical_trap_detected": trap,
            "safe_to_answer": safe,
            "explainable_drift": explainable,
            "route_drift": route_drift,
            "relation_drift": relation_drift,
            "peak_margin": result["peak_margin"],
            "field_state": result["field_interpretation"]["state"],
            "nearest_foreign_pull": result["field_interpretation"]["nearest_foreign_pull"],
            "ok": ok
        }));
    }
    let total = rows.len();
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "waw-benchmark",
        "suite": if suite.name.is_empty() { args.suite.display().to_string() } else { suite.name },
        "passed": passed,
        "total": total,
        "waw_score": round4(passed as f64 / total.max(1) as f64),
        "structural_wins": structural_wins,
        "lexical_traps": lexical_traps,
        "safe_answers": safe_answers,
        "explainable_drifts": explainable_drifts,
        "cases": rows,
        "interpretation": "A WAW pass means the structural interference peak beat the lexical baseline on a trap case and the field explains the drift."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_waw_text(&out),
        OutputFormat::Md => print_waw_md(&out),
    }
    if passed == total {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

pub(crate) fn benchmark() -> Result<u8> {
    let mut clean_pass = 0;
    let mut swap_veto = 0;
    let mut splice_veto = 0;
    let mut exact_false_accept = 0;
    for idx in 0..50 {
        let clean = synthetic_packet(idx, "clean");
        if verdict_for(&clean)? == "PASS" {
            clean_pass += 1;
        }
        let swap = synthetic_packet(idx, "swap");
        if verdict_for(&swap)? == "VETO" {
            swap_veto += 1;
        }
        let splice = synthetic_packet(idx, "splice");
        if verdict_for(&splice)? == "VETO" {
            splice_veto += 1;
        }
        if exact_baseline_accepts(&splice) {
            exact_false_accept += 1;
        }
    }
    println!("clean_pass:                         {clean_pass}/50");
    println!("swap_veto:                          {swap_veto}/50");
    println!("splice_veto:                        {splice_veto}/50");
    println!("splice_exact_baseline_false_accept: {exact_false_accept}/50");
    if clean_pass == 50 && swap_veto == 50 && splice_veto == 50 {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}
