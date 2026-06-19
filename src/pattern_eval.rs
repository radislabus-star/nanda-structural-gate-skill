use crate::*;
use serde::Deserialize;

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
