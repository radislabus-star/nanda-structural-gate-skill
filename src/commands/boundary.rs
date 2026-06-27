use crate::*;
use clap::Parser;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub(crate) struct BoundaryEconomicsArgs {
    #[arg(default_value = ".")]
    pub(crate) input: PathBuf,
    #[arg(long)]
    pub(crate) atlas: Option<PathBuf>,
    #[arg(long)]
    pub(crate) route: Option<String>,
    #[arg(long)]
    pub(crate) owner: Option<String>,
    #[arg(long)]
    pub(crate) find_refactors: bool,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

pub(crate) fn cmd(args: BoundaryEconomicsArgs) -> Result<u8> {
    let atlas = match args.atlas.as_deref() {
        Some(path) => Some(commands::guard::load_atlas(path)?),
        None => None,
    };
    let out = if args.find_refactors {
        find_refactors_with_atlas(
            &args.input,
            atlas.as_ref(),
            args.atlas.as_deref(),
            args.route.as_deref(),
            args.owner.as_deref(),
        )?
    } else {
        report_with_atlas(
            &args.input,
            atlas.as_ref(),
            args.atlas.as_deref(),
            args.route.as_deref(),
            args.owner.as_deref(),
        )?
    };
    print_boundary_output(&out, &args.format)?;
    Ok(boundary_exit_code(&out))
}

fn report_with_atlas(
    input: &Path,
    atlas: Option<&Value>,
    atlas_path: Option<&Path>,
    route: Option<&str>,
    owner: Option<&str>,
) -> Result<Value> {
    crate::field_core::boundary_report(
        input,
        atlas,
        atlas_path.map(|path| path.display().to_string()),
        route,
        owner,
        commands::dogfood::auto_route_for_path,
        commands::dogfood::auto_owner_for_path,
    )
}

fn find_refactors_with_atlas(
    input: &Path,
    atlas: Option<&Value>,
    atlas_path: Option<&Path>,
    route: Option<&str>,
    owner: Option<&str>,
) -> Result<Value> {
    crate::field_core::boundary_find_refactors(
        input,
        atlas,
        atlas_path.map(|path| path.display().to_string()),
        route,
        owner,
        commands::dogfood::auto_route_for_path,
        commands::dogfood::auto_owner_for_path,
    )
}

fn print_boundary_output(out: &Value, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(out)?),
        OutputFormat::Text => {
            if out["mode"].as_str() == Some("boundary-refactor-finder") {
                println!("mode: boundary-refactor-finder");
                println!("verdict: {}", out["verdict"].as_str().unwrap_or(""));
                println!(
                    "safe_to_edit: {}",
                    out["safe_to_edit"].as_bool().unwrap_or(false)
                );
                println!(
                    "candidates: {}",
                    out["candidate_summary"]["total"].as_u64().unwrap_or(0)
                );
                if let Some(top) = out["refactor_candidates"]
                    .as_array()
                    .and_then(|items| items.first())
                {
                    println!(
                        "top: {} {} {}",
                        top["rank"].as_u64().unwrap_or(0),
                        top["verdict"].as_str().unwrap_or("WATCH"),
                        top["subject"].as_str().unwrap_or("")
                    );
                }
                return Ok(());
            }
            let decision = &out["boundary_decision"];
            let selected = selected_boundary_verdict(out);
            println!("mode: boundary-economics");
            println!(
                "verdict: {}",
                decision["verdict"].as_str().unwrap_or("WATCH")
            );
            println!("field_selected_verdict: {selected}");
            println!("score: {}", decision["score"].as_i64().unwrap_or(0));
            println!("reason: {}", decision["reason"].as_str().unwrap_or(""));
        }
        OutputFormat::Md => {
            if out["mode"].as_str() == Some("boundary-refactor-finder") {
                println!("# NANDA Boundary Refactor Finder\n");
                println!("- verdict: `{}`", out["verdict"].as_str().unwrap_or(""));
                println!(
                    "- safe to edit: `{}`",
                    out["safe_to_edit"].as_bool().unwrap_or(false)
                );
                println!(
                    "- candidates: `{}`",
                    out["candidate_summary"]["total"].as_u64().unwrap_or(0)
                );
                if let Some(top) = out["refactor_candidates"]
                    .as_array()
                    .and_then(|items| items.first())
                {
                    println!(
                        "- top: `{} {}` {}",
                        top["rank"].as_u64().unwrap_or(0),
                        top["verdict"].as_str().unwrap_or("WATCH"),
                        top["subject"].as_str().unwrap_or("")
                    );
                }
                return Ok(());
            }
            let decision = &out["boundary_decision"];
            let selected = selected_boundary_verdict(out);
            println!("# NANDA Boundary Economics\n");
            println!(
                "- verdict: `{}`",
                decision["verdict"].as_str().unwrap_or("WATCH")
            );
            println!("- field selected verdict: `{selected}`");
            println!("- reason: {}", decision["reason"].as_str().unwrap_or(""));
        }
    }
    Ok(())
}

fn boundary_exit_code(out: &Value) -> u8 {
    if out["mode"].as_str() == Some("boundary-refactor-finder") {
        return EXIT_PASS;
    }
    match selected_boundary_verdict(out) {
        "SPLIT_STRONG" | "KEEP" => EXIT_PASS,
        "VETO" => EXIT_VETO,
        _ => EXIT_WATCH,
    }
}

fn selected_boundary_verdict(out: &Value) -> &str {
    out["boundary_field_engine"]["selected_verdict"]
        .as_str()
        .or_else(|| out["boundary_decision"]["verdict"].as_str())
        .unwrap_or("WATCH")
}
