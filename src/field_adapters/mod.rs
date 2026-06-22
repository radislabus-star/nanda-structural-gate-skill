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
        "overall_state": "FIELD_PASS_BRIDGE_ACTIVE_NOT_SOLE_ENGINE",
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
                "state": "REPORT_LAYER_FIELD_PASS",
                "remaining": ["move bounded structural scoring into field_core pass"]
            },
            {
                "family": "packed",
                "embedded_unified_field": true,
                "field_pass_present": true,
                "sole_engine": false,
                "state": "PROTECTED_HOT_CORE_EXCEPTION",
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
            "nonlinear_memory_proven": false,
            "llm_ready": false
        },
        "next_required_steps": [
            "semantic equivalence tests across structural/packed/cognitive",
            "route-scoped extraction of large report modules only after boundary audit"
        ]
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
