use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod aliases;
mod bench6m;
mod commands;
mod dataset_doctor;
mod eval;
mod feedback;
mod focus;
mod io;
mod map_gate;
mod model;
mod nanda_6m;
mod pack6m;
mod proof;
mod report;
mod search;

use aliases::{canonicalize_packet, inherit_aliases_if_needed, AliasRule, CanonicalizationReport};
pub(crate) use dataset_doctor::*;
pub(crate) use eval::*;
pub(crate) use feedback::*;
pub(crate) use io::*;
pub(crate) use map_gate::*;
pub(crate) use model::*;
pub(crate) use report::*;
pub(crate) use search::*;

const WAVE_DIM: usize = 1024;
const CORE_VERSION: &str = "sparse-triad-v3.4-resonance-memory";
const ENGINE_ID: &str = "nanda-check sparse-triad-v3.4-rust";
const MANDATORY_COMPLEXITY: i64 = 12;
const EXIT_PASS: u8 = 0;
const EXIT_VETO: u8 = 1;
const EXIT_ERROR: u8 = 2;
const EXIT_WATCH: u8 = 3;

#[derive(Parser)]
#[command(name = "nanda", about = "NANDA structural gate CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Check(CheckArgs),
    Gate(CheckArgs),
    InitTask(InitTaskArgs),
    InitMd(InitMdArgs),
    PackFromMd(PackArgs),
    GateMd(GateMdArgs),
    Split(SplitPacketArgs),
    SplitMd(SplitArgs),
    Map(MapArgs),
    Hgate(HgateArgs),
    Comb(CombArgs),
    Extract(ExtractArgs),
    Index(IndexArgs),
    Search(SearchArgs),
    Focus(FocusArgs),
    Proof(ProofArgs),
    Probe(ProbeArgs),
    DatasetDoctor(DatasetDoctorArgs),
    Aliases(AliasesArgs),
    Budget(BudgetArgs),
    Pack6m(Pack6mArgs),
    Feedback(FeedbackArgs),
    Eval(EvalArgs),
    Waw(WawArgs),
    Serve(ServeArgs),
    Doctor(DoctorArgs),
    Dogfood(DogfoodArgs),
    MapCode(commands::code_map::MapCodeArgs),
    Report(ReportArgs),
    SelfCheck,
    Bench6m(bench6m::Bench6mArgs),
    Benchmark,
}

#[derive(Parser, Clone)]
struct CheckArgs {
    #[arg(long)]
    triads: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Parser)]
struct InitTaskArgs {
    #[arg(long, default_value = "task")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    stdout: bool,
}

#[derive(Parser)]
struct InitMdArgs {
    #[arg(long, default_value = "task")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, value_enum, default_value = "generic")]
    template: TemplateKind,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    stdout: bool,
}

#[derive(Parser)]
struct PackArgs {
    input: PathBuf,
    #[arg(long, default_value = "md-task")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    stdout: bool,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct GateMdArgs {
    input: PathBuf,
    #[arg(long, default_value = "md-task")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct SplitArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "group")]
    by: SplitBy,
    #[arg(long, default_value = ".")]
    out_dir: PathBuf,
    #[arg(long)]
    prefix: Option<String>,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct SplitPacketArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, value_enum, default_value = "linked-group")]
    by: SplitBy,
    #[arg(long, value_enum, default_value = "json")]
    format: SplitOutputFormat,
    #[arg(long, default_value = ".")]
    out_dir: PathBuf,
    #[arg(long)]
    prefix: Option<String>,
    #[arg(long, default_value = "split")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct MapArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "map")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct HgateArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, value_enum, default_value = "linked-group")]
    by: SplitBy,
    #[arg(long, default_value_t = 64)]
    max_branches: usize,
    #[arg(long, default_value = "hgate")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct CombArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value_t = 1)]
    depth: usize,
    #[arg(long, default_value = "linked-group,subject-relation")]
    branch_by: String,
    #[arg(long, default_value = "foreign_pull,invariant_violation,size")]
    stop_on: String,
    #[arg(long, default_value_t = 64)]
    max_branches: usize,
    #[arg(long, default_value = "comb")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    out_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct SearchArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "search")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    query_file: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    query_format: InputFormat,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, default_value_t = 256)]
    route_cap: usize,
    #[arg(long, default_value_t = 32)]
    route_triad_cap: usize,
    #[arg(long, value_enum, default_value = "route")]
    group_by: PeakGroupBy,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct FocusArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "focus")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    query_file: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    query_format: InputFormat,
    #[arg(long, default_value_t = nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY)]
    max_triads: usize,
    #[arg(long, default_value_t = 256)]
    route_cap: usize,
    #[arg(long, default_value_t = 64)]
    route_triad_cap: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    stdout: bool,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct ProofArgs {
    input: Option<PathBuf>,
    #[arg(long)]
    suite: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "proof")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    query_file: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    query_format: InputFormat,
    #[arg(long, default_value_t = nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY)]
    max_triads: usize,
    #[arg(long, default_value_t = 256)]
    route_cap: usize,
    #[arg(long, default_value_t = 64)]
    route_triad_cap: usize,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "route")]
    group_by: PeakGroupBy,
    #[arg(long, default_value_t = 8)]
    sample: usize,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    focus_out: Option<PathBuf>,
    #[arg(long)]
    include_focused_packet: bool,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct ProbeArgs {
    input: Option<PathBuf>,
    #[arg(long)]
    suite: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long = "negative")]
    negative_inputs: Vec<PathBuf>,
    #[arg(long, default_value = "probe")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long)]
    query_file: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    query_format: InputFormat,
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, default_value_t = 256)]
    route_cap: usize,
    #[arg(long, default_value_t = 32)]
    route_triad_cap: usize,
    #[arg(long, value_enum, default_value = "route")]
    group_by: PeakGroupBy,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct DatasetDoctorArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "dataset-doctor")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long, default_value_t = 256)]
    route_cap: usize,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct AliasesArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "aliases")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct BudgetArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "budget")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct Pack6mArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "pack6m")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "")]
    query: String,
    #[arg(long, default_value_t = 8)]
    sample: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct IndexArgs {
    inputs: Vec<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value = "nanda-index")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "NANDA memory index")]
    query: String,
    #[arg(long, default_value = ".nanda/index.json")]
    out: PathBuf,
    #[arg(long)]
    include_candidates: bool,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct ExtractArgs {
    input: PathBuf,
    #[arg(long, default_value = "extracted")]
    task_id: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long, default_value = "Extracted triad packet")]
    query: String,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    stdout: bool,
}

#[derive(Parser)]
struct FeedbackArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value = "accept")]
    decision: FeedbackDecision,
    #[arg(long, default_value = "")]
    note: String,
    #[arg(long, default_value = ".nanda/feedback.json")]
    out: PathBuf,
    #[arg(long)]
    stdout: bool,
}

#[derive(Parser)]
struct EvalArgs {
    #[arg(long = "case")]
    cases: Vec<String>,
    #[arg(long)]
    suite: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value_t = 3)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "route")]
    group_by: PeakGroupBy,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct WawArgs {
    #[arg(long)]
    suite: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
    #[arg(long, default_value_t = 3)]
    top_k: usize,
    #[arg(long, value_enum, default_value = "route")]
    group_by: PeakGroupBy,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long)]
    normalize_paths: bool,
}

#[derive(Parser)]
struct ServeArgs {
    #[arg(long, value_enum, default_value = "jsonl")]
    format: ServeFormat,
}

#[derive(Parser)]
struct DoctorArgs {
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct DogfoodArgs {
    #[arg(default_value = ".")]
    input: PathBuf,
    #[arg(long, value_enum, default_value = "auto")]
    input_format: InputFormat,
    #[arg(long, default_value_t = 2)]
    depth: usize,
    #[arg(long, default_value = "linked-group,subject-relation")]
    branch_by: String,
    #[arg(long, default_value = "foreign_pull,invariant_violation,size")]
    stop_on: String,
    #[arg(long, default_value_t = 64)]
    max_branches: usize,
    #[arg(long)]
    out_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
    #[arg(long)]
    normalize_paths: bool,
    #[arg(long)]
    refactor_plan: bool,
}

#[derive(Parser)]
struct ReportArgs {
    #[arg(long, default_value = "NANDA Report")]
    title: String,
    #[arg(long, default_value = "general")]
    domain: String,
    #[arg(long)]
    overall: PathBuf,
    #[arg(long = "route")]
    routes: Vec<String>,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn run() -> Result<u8> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check(args) => run_check(args, false),
        Command::Gate(args) => run_check(args, true),
        Command::InitTask(args) => init_task(args),
        Command::InitMd(args) => init_md(args),
        Command::PackFromMd(args) => pack_from_md(args),
        Command::GateMd(args) => gate_md(args),
        Command::Split(args) => split_packet(args),
        Command::SplitMd(args) => split_md(args),
        Command::Map(args) => map_cmd(args),
        Command::Hgate(args) => hgate_cmd(args),
        Command::Comb(args) => comb_cmd(args),
        Command::Extract(args) => extract_cmd(args),
        Command::Index(args) => index_cmd(args),
        Command::Search(args) => search_cmd(args),
        Command::Focus(args) => focus::focus_cmd(args),
        Command::Proof(args) => proof::proof_cmd(args),
        Command::Probe(args) => probe_cmd(args),
        Command::DatasetDoctor(args) => dataset_doctor_cmd(args),
        Command::Aliases(args) => aliases_cmd(args),
        Command::Budget(args) => pack6m::budget_cmd(args),
        Command::Pack6m(args) => pack6m::pack6m_cmd(args),
        Command::Feedback(args) => feedback_cmd(args),
        Command::Eval(args) => eval_cmd(args),
        Command::Waw(args) => waw_cmd(args),
        Command::Serve(args) => serve_cmd(args),
        Command::Doctor(args) => doctor_cmd(args),
        Command::Dogfood(args) => commands::dogfood::dogfood_cmd(args),
        Command::MapCode(args) => commands::code_map::cmd(args),
        Command::Report(args) => report_cmd(args),
        Command::SelfCheck => self_check(),
        Command::Bench6m(args) => bench6m::cmd(args),
        Command::Benchmark => benchmark(),
    }
}

fn run_check(args: CheckArgs, strict: bool) -> Result<u8> {
    let packet = load_packet(args.triads.as_deref())?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let report = make_report(&packet, &source, &candidates)?;
    print_report(&report, &args.format)?;
    if strict && report.verdict != "PASS" {
        return Ok(verdict_code(&report.verdict));
    }
    Ok(verdict_code(&report.verdict))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_case_parser_accepts_relative_paths() {
        let (path, peak, state) =
            parse_eval_case("examples/route.json:certification:FOCUSED").unwrap();
        assert_eq!(path, PathBuf::from("examples/route.json"));
        assert_eq!(peak, "certification");
        assert_eq!(state, "FOCUSED");
    }

    #[test]
    fn eval_case_parser_accepts_windows_drive_paths() {
        let (path, peak, state) =
            parse_eval_case(r"C:\repo\nanda\route.json:certification:WATCH").unwrap();
        assert_eq!(path, PathBuf::from(r"C:\repo\nanda\route.json"));
        assert_eq!(peak, "certification");
        assert_eq!(state, "WATCH");
    }
}
