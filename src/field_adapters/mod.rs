use crate::*;
use clap::ValueEnum;

#[derive(Parser)]
pub(crate) struct FieldReportArgs {
    #[arg(long = "from")]
    pub(crate) from: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Parser)]
pub(crate) struct FieldAuditArgs {
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Parser)]
pub(crate) struct FieldEquivalenceArgs {
    #[arg(long = "structural-from")]
    pub(crate) structural_from: PathBuf,
    #[arg(long = "packed-from")]
    pub(crate) packed_from: PathBuf,
    #[arg(long = "cognitive-from")]
    pub(crate) cognitive_from: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Parser)]
pub(crate) struct FieldCutoverArgs {
    #[arg(long = "structural-case")]
    pub(crate) structural_cases: Vec<PathBuf>,
    #[arg(long, value_enum)]
    pub(crate) suite: Option<FieldCutoverSuite>,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum FieldCutoverSuite {
    StructuralStandard,
}

pub(crate) fn field_report_cmd(args: FieldReportArgs) -> Result<u8> {
    let input = serde_json::from_str::<Value>(
        &fs::read_to_string(&args.from)
            .with_context(|| format!("read field report input {}", args.from.display()))?,
    )
    .with_context(|| format!("parse JSON field report input {}", args.from.display()))?;
    let report = field_core::adapters::adapt_value(&input);
    print_unified_field_report(&report, &args.format)?;
    Ok(EXIT_PASS)
}

pub(crate) fn field_audit_cmd(args: FieldAuditArgs) -> Result<u8> {
    let structural_cutover_suite =
        field_cutover_report(structural_standard_cutover_cases()?, "structural-standard");
    let structural_cutover_suite_pass = structural_cutover_suite["acceptance"]
        ["structural_cutover_suite_pass"]
        .as_bool()
        .unwrap_or(false);
    let out = json!({
        "mode": "unified-field-audit",
        "version": field_core::FIELD_PASS_VERSION,
        "overall_state": "UNIFIED_FIELD_RUNTIME_DUAL_RUN_ACTIVE_NOT_SOLE_ENGINE",
        "field_core": {
            "vector": "FieldVector1024",
            "record": "FieldRecord",
            "pass": "FieldPass",
            "verdict": "FieldPassReport",
            "feedback_delta": "FieldMemoryDelta",
            "memory_delta": "FieldMemoryDeltaSummary"
        },
        "families": [
            {
                "family": "structural",
                "embedded_unified_field": true,
                "field_pass_present": true,
                "sole_engine": false,
                "engine_guard": "structural-field-engine-v1",
                "opt_in_cutover_available": structural_cutover_suite_pass,
                "state": if structural_cutover_suite_pass {
                    "DUAL_RUN_CUTOVER_SUITE_PASS"
                } else {
                    "DUAL_RUN_CUTOVER_SUITE_WATCH"
                },
                "remaining": ["explicit follow-up cutover is still required before structural sole-engine mode"]
            },
            {
                "family": "packed",
                "embedded_unified_field": true,
                "field_pass_present": true,
                "sole_engine": false,
                "engine_guard": "packed-field-engine-guard-v1",
                "opt_in_cutover_available": false,
                "state": "DUAL_RUN_READY_PROTECTED_HOT_CORE",
                "remaining": ["bench before replacing hot summaries"]
            },
            {
                "family": "cognitive",
                "embedded_unified_field": true,
                "field_pass_present": true,
                "sole_engine": false,
                "engine_guard": "cognitive-field-engine-guard-v1",
                "opt_in_cutover_available": false,
                "state": "DUAL_RUN_ACTIVE_NOT_LLM",
                "remaining": ["route deeper query-wave/multi-peak/lens/anti/evidence records through FieldPass input"]
            }
        ],
        "field_engine_contract": {
            "version": "unified-field-engine-contract-v1",
            "policy_owner": "field_core::engine::FieldEngineDecision",
            "families_checked": 3,
            "structural": {
                "engine_guard": "structural-field-engine-v1",
                "cutover_mode": "opt-in",
                "cutover_allowed": structural_cutover_suite_pass,
                "global_sole_engine": false
            },
            "packed": {
                "engine_guard": "packed-field-engine-guard-v1",
                "cutover_mode": "blocked",
                "cutover_allowed": false,
                "blocked_by": "packed_hot_core_exception",
                "global_sole_engine": false
            },
            "cognitive": {
                "engine_guard": "cognitive-field-engine-guard-v1",
                "cutover_mode": "blocked",
                "cutover_allowed": false,
                "blocked_by": "claim_boundary_not_llm_ready",
                "chat_engine": false,
                "llm_ready": false,
                "global_sole_engine": false
            }
        },
        "acceptance": {
            "one_field_vocabulary": true,
            "one_field_pass": true,
            "all_json_reports_project_unified_field": true,
            "three_family_engine_contract": true,
            "field_engine_policy_in_field_core": true,
            "field_core_as_sole_engine": false,
            "field_core_as_semantic_engine": true,
            "feedback_memory_delta_unified": true,
            "semantic_equivalence_gate": true,
            "structural_dual_run_active": true,
            "structural_cutover_eval_ready": true,
            "structural_cutover_suite_available": true,
            "structural_cutover_suite_pass": structural_cutover_suite_pass,
            "structural_cutover_mode_available": structural_cutover_suite_pass,
            "packed_dual_run_active": true,
            "packed_field_engine_guard": true,
            "packed_cutover_blocked_by_hot_guard": true,
            "packed_hot_core_exception": true,
            "packed_field_record_view": true,
            "cognitive_dual_run_active": true,
            "cognitive_field_engine_guard": true,
            "cognitive_cutover_blocked_by_claim_guard": true,
            "unified_lens_contract": true,
            "unified_anti_wave_contract": true,
            "unified_memory_delta_store": true,
            "route_scoped_extraction_required": false,
            "nonlinear_memory_proven": false,
            "llm_ready": false
        },
        "structural_cutover_suite": structural_cutover_suite,
        "boundary_economics": {
            "report_module_extraction": "KEEP",
            "reason": "route-scoped extraction is not required until boundary evidence shows reduced confusion"
        },
        "next_required_steps": []
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => {
            println!("mode: unified-field-audit");
            println!("version: {}", out["version"].as_str().unwrap_or(""));
            println!("state: {}", out["overall_state"].as_str().unwrap_or(""));
            println!("field_core_as_sole_engine: false");
        }
        OutputFormat::Md => {
            println!("# Unified Field Audit\n");
            println!("- version: `{}`", out["version"].as_str().unwrap_or(""));
            println!("- state: `{}`", out["overall_state"].as_str().unwrap_or(""));
            println!("- field_core_as_sole_engine: `false`");
        }
    }
    Ok(EXIT_PASS)
}

pub(crate) fn field_equivalence_cmd(args: FieldEquivalenceArgs) -> Result<u8> {
    let cases = [
        ("structural", &args.structural_from),
        ("packed", &args.packed_from),
        ("cognitive", &args.cognitive_from),
    ];
    let mut families = vec![];
    for (expected_family, path) in cases {
        let input = serde_json::from_str::<Value>(
            &fs::read_to_string(path)
                .with_context(|| format!("read field equivalence input {}", path.display()))?,
        )
        .with_context(|| format!("parse JSON field equivalence input {}", path.display()))?;
        let report = field_core::adapters::adapt_value(&input);
        let value = report.to_value();
        let field_pass = &value["field_pass"];
        let compute_probe = &value["compute_probe"];
        let memory_delta = &value["memory_delta"];
        let claim_boundary = &value["claim_boundary"];
        let actual_family = value["family"].as_str().unwrap_or("unknown");
        families.push(json!({
            "expected_family": expected_family,
            "actual_family": actual_family,
            "input": path.display().to_string(),
            "family_matches": actual_family == expected_family,
            "compute_probe_version": compute_probe["version"].as_str().unwrap_or(""),
            "field_pass_version": field_pass["version"].as_str().unwrap_or(""),
            "field_pass_family": field_pass["family"].as_str().unwrap_or(""),
            "field_pass_present": field_pass["version"].as_str() == Some(field_core::FIELD_PASS_VERSION),
            "memory_delta_present": memory_delta["replayable_into_next_pass"].is_boolean(),
            "claim_boundary_preserved": claim_boundary["not_llm_ready"].as_bool().unwrap_or(false)
                && claim_boundary["not_nonlinear_memory_proof"].as_bool().unwrap_or(false),
            "verdict": field_pass["verdict"].as_str().unwrap_or("UNKNOWN"),
            "safe_to_answer": field_pass["safe_to_answer"].as_bool().unwrap_or(false)
        }));
    }

    let families_checked = families.len();
    let all_family_matches = families
        .iter()
        .all(|case| case["family_matches"].as_bool().unwrap_or(false));
    let all_have_compute_probe = families.iter().all(|case| {
        case["compute_probe_version"].as_str() == Some(field_core::FIELD_COMPUTE_VERSION)
    });
    let all_have_field_pass = families.iter().all(|case| {
        case["field_pass_present"].as_bool().unwrap_or(false)
            && case["field_pass_version"].as_str() == Some(field_core::FIELD_PASS_VERSION)
    });
    let all_field_pass_families_match = families
        .iter()
        .all(|case| case["field_pass_family"].as_str() == case["expected_family"].as_str());
    let all_have_memory_delta = families
        .iter()
        .all(|case| case["memory_delta_present"].as_bool().unwrap_or(false));
    let all_preserve_claim_boundary = families
        .iter()
        .all(|case| case["claim_boundary_preserved"].as_bool().unwrap_or(false));
    let equivalent_contract = families_checked == 3
        && all_family_matches
        && all_have_compute_probe
        && all_have_field_pass
        && all_field_pass_families_match
        && all_have_memory_delta
        && all_preserve_claim_boundary;
    let state = if equivalent_contract {
        "FIELD_EQUIVALENCE_PASS"
    } else {
        "FIELD_EQUIVALENCE_WATCH"
    };
    let out = json!({
        "mode": "unified-field-equivalence",
        "version": field_core::FIELD_PASS_VERSION,
        "state": state,
        "families": families,
        "acceptance": {
            "families_checked": families_checked,
            "all_family_matches": all_family_matches,
            "all_have_compute_probe": all_have_compute_probe,
            "all_have_field_pass": all_have_field_pass,
            "all_field_pass_families_match": all_field_pass_families_match,
            "all_have_memory_delta": all_have_memory_delta,
            "all_preserve_claim_boundary": all_preserve_claim_boundary,
            "equivalent_contract": equivalent_contract,
            "field_core_as_sole_engine": false,
            "llm_ready": false,
            "nonlinear_memory_proven": false
        }
    });

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => {
            println!("mode: unified-field-equivalence");
            println!("version: {}", out["version"].as_str().unwrap_or(""));
            println!("state: {state}");
            println!("equivalent_contract: {equivalent_contract}");
        }
        OutputFormat::Md => {
            println!("# Unified Field Equivalence\n");
            println!("- version: `{}`", out["version"].as_str().unwrap_or(""));
            println!("- state: `{state}`");
            println!("- equivalent_contract: `{equivalent_contract}`");
        }
    }

    Ok(if equivalent_contract {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

pub(crate) fn field_cutover_cmd(args: FieldCutoverArgs) -> Result<u8> {
    let mut cases = vec![];
    for path in &args.structural_cases {
        let input = serde_json::from_str::<Value>(
            &fs::read_to_string(path)
                .with_context(|| format!("read field cutover input {}", path.display()))?,
        )
        .with_context(|| format!("parse JSON field cutover input {}", path.display()))?;
        let dual_run = field_core::structural_dual_run_from_search(&input);
        cases.push(field_cutover_case(
            "manual",
            &path.display().to_string(),
            dual_run,
        )?);
    }
    if let Some(suite) = &args.suite {
        for case in field_cutover_suite_cases(suite)? {
            cases.push(case);
        }
    }
    let suite_label = match args.suite {
        Some(FieldCutoverSuite::StructuralStandard) => "structural-standard",
        None => "manual",
    };
    let out = field_cutover_report(cases, suite_label);

    let state = out["state"]
        .as_str()
        .unwrap_or("STRUCTURAL_FIELD_CUTOVER_SUITE_WATCH");
    let cases_checked = out["acceptance"]["cases_checked"].as_u64().unwrap_or(0);
    let structural_cutover_suite_pass = out["acceptance"]["structural_cutover_suite_pass"]
        .as_bool()
        .unwrap_or(false);

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => {
            println!("mode: unified-field-cutover-suite");
            println!("version: {}", field_core::FIELD_RUNTIME_VERSION);
            println!("family: structural");
            println!("state: {state}");
            println!("cases_checked: {cases_checked}");
            println!("structural_cutover_suite_pass: {structural_cutover_suite_pass}");
            println!("field_core_as_sole_engine_allowed: false");
        }
        OutputFormat::Md => {
            println!("# Unified Field Cutover Suite\n");
            println!("- version: `{}`", field_core::FIELD_RUNTIME_VERSION);
            println!("- family: `structural`");
            println!("- state: `{state}`");
            println!("- cases_checked: `{cases_checked}`");
            println!("- structural_cutover_suite_pass: `{structural_cutover_suite_pass}`");
            println!("- field_core_as_sole_engine_allowed: `false`");
        }
    }

    Ok(if structural_cutover_suite_pass {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

fn field_cutover_report(cases: Vec<Value>, suite_label: &str) -> Value {
    let cases_checked = cases.len();
    let all_peak_match = nonempty_all(&cases, "peak_matches");
    let all_state_family_match = nonempty_all(&cases, "state_family_matches");
    let all_not_more_permissive = nonempty_all(&cases, "field_not_more_permissive");
    let all_cutover_ready = nonempty_all(&cases, "cutover_ready");
    let structural_cutover_suite_pass = cases_checked > 0
        && all_peak_match
        && all_state_family_match
        && all_not_more_permissive
        && all_cutover_ready;
    let state = if structural_cutover_suite_pass {
        "STRUCTURAL_FIELD_CUTOVER_SUITE_PASS"
    } else if cases_checked == 0 {
        "STRUCTURAL_FIELD_CUTOVER_SUITE_NO_CASES"
    } else {
        "STRUCTURAL_FIELD_CUTOVER_SUITE_WATCH"
    };
    json!({
        "mode": "unified-field-cutover-suite",
        "version": field_core::FIELD_RUNTIME_VERSION,
        "family": "structural",
        "suite": suite_label,
        "state": state,
        "cases": cases,
        "acceptance": {
            "cases_checked": cases_checked,
            "all_peak_match": all_peak_match,
            "all_state_family_match": all_state_family_match,
            "all_not_more_permissive": all_not_more_permissive,
            "all_cutover_ready": all_cutover_ready,
            "structural_cutover_suite_pass": structural_cutover_suite_pass,
            "field_core_as_structural_engine_candidate": structural_cutover_suite_pass,
            "field_core_as_sole_engine_allowed": false,
            "packed_hot_core_exception": true,
            "llm_ready": false,
            "nonlinear_memory_proven": false
        },
        "claim_boundary": {
            "global_sole_engine": false,
            "structural_only_candidate": structural_cutover_suite_pass,
            "packed_hot_core_exception": true,
            "cognitive_not_llm": true,
            "requires_explicit_follow_up_cutover": true
        }
    })
}

fn field_cutover_case(
    source: &str,
    input: &str,
    dual_run: field_core::FieldRuntimeDualRun,
) -> Result<Value> {
    let field_runtime = serde_json::to_value(&dual_run)?;
    Ok(json!({
        "family": "structural",
        "source": source,
        "input": input,
        "old_peak": dual_run.old_peak,
        "field_peak": dual_run.field_peak,
        "old_verdict": dual_run.old_verdict,
        "field_verdict": dual_run.field_verdict,
        "old_field_state": dual_run.old_field_state,
        "field_state": dual_run.field_state,
        "old_safe_to_answer": dual_run.old_safe_to_answer,
        "field_safe_to_answer": dual_run.field_safe_to_answer,
        "peak_matches": dual_run.peak_matches,
        "state_family_matches": dual_run.state_family_matches,
        "field_not_more_permissive": dual_run.field_not_more_permissive,
        "cutover_ready": dual_run.cutover_ready,
        "mismatch_reason": dual_run.mismatch_reason,
        "field_runtime": field_runtime
    }))
}

fn field_cutover_suite_cases(suite: &FieldCutoverSuite) -> Result<Vec<Value>> {
    match suite {
        FieldCutoverSuite::StructuralStandard => structural_standard_cutover_cases(),
    }
}

fn structural_standard_cutover_cases() -> Result<Vec<Value>> {
    let specs = [
        (
            "focused-route-trap",
            "examples/triad-packet.interference-search-route-trap.json",
        ),
        (
            "contested-noisy",
            "examples/triad-packet.interference-search-noisy.json",
        ),
        (
            "reversed-polarity",
            "examples/triad-packet.polarization-reversed-stop.json",
        ),
        (
            "thin-negative-lane",
            "examples/triad-packet.negative-shortcut-lanes.json",
        ),
    ];
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cases = vec![];
    for (label, rel_path) in specs {
        let path = root.join(rel_path);
        let input = structural_suite_search_value(&path)?;
        let dual_run = field_core::structural_dual_run_from_search(&input);
        cases.push(field_cutover_case(
            "structural-standard",
            &format!("{label}:{rel_path}"),
            dual_run,
        )?);
    }
    Ok(cases)
}

fn structural_suite_search_value(path: &Path) -> Result<Value> {
    let mut packet = load_packet_auto(
        path,
        &InputFormat::Json,
        "field-cutover",
        "structural",
        "",
        false,
    )?;
    let memory = normalize_ids(packet.triads.clone(), "m");
    let query_packet = packet.clone();
    let query_text = if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let focus = route_balanced_focus(&memory, &query, 256, 32);
    let mut result = interference_search(
        &packet,
        &focus.memory,
        &query,
        3,
        &PeakGroupBy::Route,
        query_source,
        focus.metadata,
    );
    result["canonicalization"] = json!(packet.canonicalization);
    result["unified_field"] = field_core::adapters::adapt_value(&result).to_value();
    result["field_runtime"] = field_core::structural_dual_run_value(&result);
    Ok(result)
}

fn nonempty_all(cases: &[Value], key: &str) -> bool {
    !cases.is_empty()
        && cases
            .iter()
            .all(|case| case[key].as_bool().unwrap_or(false))
}

fn print_unified_field_report(
    report: &field_core::UnifiedFieldReport,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report.to_value())?),
        OutputFormat::Text => {
            println!("mode: unified-field-report");
            println!("version: {}", report.version);
            println!("family: {}", report.family.as_str());
            println!("source_mode: {}", report.source_mode);
            println!("basis: {}", report.basis.basis_kind);
            println!("dimension: {}", report.basis.dimension);
            println!("peak: {}", report.peak.target);
            println!("state: {}", report.peak.state);
            println!("safe_to_answer: {}", report.peak.safe_to_answer);
            println!("field_state: {}", report.coherence.field_state);
            println!("anti_wave_active: {}", report.anti_wave.active);
        }
        OutputFormat::Md => {
            println!("# Unified Field Report\n");
            println!("- version: `{}`", report.version);
            println!("- family: `{}`", report.family.as_str());
            println!("- source_mode: `{}`", report.source_mode);
            println!("- basis: `{}`", report.basis.basis_kind);
            println!("- dimension: `{}`", report.basis.dimension);
            println!("- peak: `{}`", report.peak.target);
            println!("- state: `{}`", report.peak.state);
            println!("- safe_to_answer: `{}`", report.peak.safe_to_answer);
            println!("- field_state: `{}`", report.coherence.field_state);
            println!("- anti_wave_active: `{}`", report.anti_wave.active);
        }
    }
    Ok(())
}
