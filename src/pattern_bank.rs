use crate::*;

pub(crate) fn pattern_bank_cmd(args: PatternBankArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let out = pattern_bank_report(&packet, &args);
    let output = match args.format {
        OutputFormat::Json => serde_json::to_string_pretty(&out)? + "\n",
        OutputFormat::Text => pattern_bank_text(&out),
        OutputFormat::Md => pattern_bank_md(&out),
    };
    if let Some(path) = args.out {
        write_or_print(path, false, output)?;
    } else {
        print!("{output}");
    }
    Ok(if out["fits_pattern_arena"].as_bool().unwrap_or(false) {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

pub(crate) fn pattern_bank_report(packet: &Packet, args: &PatternBankArgs) -> Value {
    let store = compact_pattern_store_report(packet, args.sample);
    let mode = match args.mode {
        PatternBankMode::Build => "build",
        PatternBankMode::Inspect => "inspect",
        PatternBankMode::Apply => "apply",
    };
    let apply_state = if matches!(args.mode, PatternBankMode::Apply) {
        if packet.continuation_memory.is_empty() {
            "NO_PATTERN_BANK"
        } else {
            "PATTERN_BANK_READY_FOR_DECODE"
        }
    } else {
        "NOT_APPLIED"
    };
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "pattern-bank",
        "version": "v48-cleanup-pattern-bank",
        "operation": mode,
        "records": store["records"],
        "accepted": store["accepted"],
        "rejected": store["rejected"],
        "packed_pattern_bytes": store["packed_pattern_bytes"],
        "arena_bytes": store["arena_bytes"],
        "capacity": store["capacity"],
        "used_bytes": store["used_bytes"],
        "remaining_bytes": store["remaining_bytes"],
        "fits_pattern_arena": store["fits_pattern_arena"],
        "apply_state": apply_state,
        "cleanup_memory": {
            "version": "v48-cleanup-memory",
            "state": if packet.continuation_memory.is_empty() { "NO_CLEANUP_RECORDS" } else { "CLEANUP_DICTIONARY_READY" },
            "records": packet.continuation_memory.len(),
            "accepted_records": store["accepted"],
            "rejected_records": store["rejected"],
            "contract": "raw decoded pattern -> nearest accepted/rejected continuation record -> cleanup verdict"
        },
        "records_sample": store["records_sample"],
        "read_as": "A Pattern Bank is now the cleanup-memory layer: compact 32-byte continuation records that can inspect, budget, replay, and clean up noisy decoded structural patterns."
    })
}

fn pattern_bank_text(out: &Value) -> String {
    format!(
        "mode: pattern-bank\noperation: {}\nrecords: {}\nfits_pattern_arena: {}\napply_state: {}\n",
        out["operation"].as_str().unwrap_or(""),
        out["records"].as_u64().unwrap_or(0),
        out["fits_pattern_arena"].as_bool().unwrap_or(false),
        out["apply_state"].as_str().unwrap_or("")
    )
}

fn pattern_bank_md(out: &Value) -> String {
    format!(
        "# NANDA Pattern Bank\n\n- operation: `{}`\n- records: `{}`\n- fits_pattern_arena: `{}`\n- apply_state: `{}`\n",
        out["operation"].as_str().unwrap_or(""),
        out["records"].as_u64().unwrap_or(0),
        out["fits_pattern_arena"].as_bool().unwrap_or(false),
        out["apply_state"].as_str().unwrap_or("")
    )
}
