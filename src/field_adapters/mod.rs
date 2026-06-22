use crate::*;

#[derive(Parser)]
pub(crate) struct FieldReportArgs {
    #[arg(long = "from")]
    pub(crate) from: PathBuf,
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
