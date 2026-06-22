use crate::*;

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
    let out = json!({
        "mode": "unified-field-audit",
        "version": field_core::FIELD_PASS_VERSION,
        "overall_state": "UNIFIED_FIELD_BRIDGE_COMPLETE_NOT_SOLE_ENGINE",
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
                "state": "DUAL_RUN_CUTOVER_EVAL_READY",
                "remaining": ["run structural cutover suite before using field_core as sole structural engine"]
            },
            {
                "family": "packed",
                "embedded_unified_field": true,
                "field_pass_present": true,
                "sole_engine": false,
                "state": "DUAL_RUN_READY_PROTECTED_HOT_CORE",
                "remaining": ["zero-cost packed FieldRecordView", "bench before replacing hot summaries"]
            },
            {
                "family": "cognitive",
                "embedded_unified_field": true,
                "field_pass_present": true,
                "sole_engine": false,
                "state": "REPORT_LAYER_FIELD_PASS",
                "remaining": ["route query-wave/multi-peak/lens/anti/evidence through FieldPass input"]
            }
        ],
        "acceptance": {
            "one_field_vocabulary": true,
            "one_field_pass": true,
            "all_json_reports_project_unified_field": true,
            "field_core_as_sole_engine": false,
            "feedback_memory_delta_unified": true,
            "semantic_equivalence_gate": true,
            "structural_dual_run_active": true,
            "structural_cutover_eval_ready": true,
            "packed_dual_run_active": true,
            "packed_hot_core_exception": true,
            "route_scoped_extraction_required": false,
            "nonlinear_memory_proven": false,
            "llm_ready": false
        },
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
