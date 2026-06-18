use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod nanda_6m;

const WAVE_DIM: usize = 1024;
const CORE_VERSION: &str = "sparse-triad-v2.8-packed-replay";
const ENGINE_ID: &str = "nanda-check sparse-triad-v2.8-rust";
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
    Probe(ProbeArgs),
    DatasetDoctor(DatasetDoctorArgs),
    Budget(BudgetArgs),
    Pack6m(Pack6mArgs),
    Feedback(FeedbackArgs),
    Eval(EvalArgs),
    Waw(WawArgs),
    Serve(ServeArgs),
    Doctor(DoctorArgs),
    Dogfood(DogfoodArgs),
    Report(ReportArgs),
    SelfCheck,
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

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
    Md,
}

#[derive(Clone, ValueEnum)]
enum SplitOutputFormat {
    Json,
    Md,
}

#[derive(Clone, ValueEnum)]
enum InputFormat {
    Auto,
    Json,
    Md,
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
enum TemplateKind {
    Generic,
    Code,
    Skill,
    Project,
}

#[derive(Clone, ValueEnum)]
enum SplitBy {
    Group,
    Route,
    LinkedGroup,
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
enum PeakGroupBy {
    Group,
    Route,
}

#[derive(Clone, ValueEnum)]
enum ServeFormat {
    Jsonl,
}

#[derive(Clone, ValueEnum)]
enum FeedbackDecision {
    Accept,
    Reject,
    Watch,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Triad {
    #[serde(default)]
    id: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    relation: String,
    #[serde(default)]
    object: String,
    #[serde(default)]
    evidence: String,
    #[serde(default = "default_confidence")]
    confidence: f64,
    #[serde(default = "default_subject_role")]
    subject_role: String,
    #[serde(default = "default_object_role")]
    object_role: String,
    #[serde(default)]
    route: String,
    #[serde(default)]
    group: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Packet {
    #[serde(default = "default_task_id")]
    task_id: String,
    #[serde(default = "default_domain")]
    domain: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    triads: Vec<Triad>,
    #[serde(default)]
    candidate_triads: Vec<Triad>,
    #[serde(default)]
    candidate_answer: String,
    #[serde(default)]
    negative_shortcuts: Vec<NegativeShortcut>,
    #[serde(default)]
    positive_shortcuts: Vec<PositiveShortcut>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct NegativeShortcut {
    #[serde(default)]
    id: String,
    #[serde(default)]
    suppress_peak: String,
    #[serde(default)]
    suppress_route: String,
    #[serde(default)]
    suppress_group: String,
    #[serde(default)]
    prefer_peak: String,
    #[serde(default)]
    prefer_route: String,
    #[serde(default)]
    prefer_group: String,
    #[serde(default = "default_negative_penalty")]
    penalty: f64,
    #[serde(default)]
    terms: Vec<String>,
    #[serde(default)]
    support_terms: Vec<String>,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    source_feedback: String,
    #[serde(default = "default_one_usize")]
    observations: usize,
    #[serde(default)]
    rejected_count: usize,
    #[serde(default)]
    accepted_count: usize,
}

fn default_negative_penalty() -> f64 {
    0.18
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PositiveShortcut {
    #[serde(default)]
    id: String,
    #[serde(default)]
    reinforce_peak: String,
    #[serde(default)]
    reinforce_route: String,
    #[serde(default)]
    reinforce_group: String,
    #[serde(default = "default_positive_boost")]
    boost: f64,
    #[serde(default)]
    terms: Vec<String>,
    #[serde(default)]
    support_terms: Vec<String>,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    source_feedback: String,
    #[serde(default = "default_one_usize")]
    observations: usize,
    #[serde(default)]
    accepted_count: usize,
    #[serde(default)]
    rejected_count: usize,
}

fn default_positive_boost() -> f64 {
    0.08
}

fn default_one_usize() -> usize {
    1
}

#[derive(Clone, Debug, Deserialize)]
struct EvalSuite {
    #[serde(default)]
    cases: Vec<EvalSuiteCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct EvalSuiteCase {
    path: PathBuf,
    expected_peak: String,
    expected_state: String,
}

#[derive(Clone, Debug, Deserialize)]
struct WawSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<WawSuiteCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct ProbeSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<ProbeSuiteCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct ProbeSuiteCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    #[serde(default)]
    negative: Vec<PathBuf>,
    #[serde(default)]
    expected_decision: String,
    #[serde(default)]
    expected_plain_peak: String,
    #[serde(default)]
    expected_negative_peak: String,
    #[serde(default)]
    group_by: String,
}

#[derive(Clone, Debug, Deserialize)]
struct WawSuiteCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    expected_peak: String,
    expected_state: String,
    #[serde(default)]
    expected_lexical_peak: String,
    #[serde(default = "default_true")]
    require_lexical_trap: bool,
    #[serde(default = "default_true")]
    require_safe_to_answer: bool,
}

fn default_true() -> bool {
    true
}

fn default_confidence() -> f64 {
    1.0
}
fn default_subject_role() -> String {
    "subject".to_string()
}
fn default_object_role() -> String {
    "object".to_string()
}
fn default_task_id() -> String {
    "task".to_string()
}
fn default_domain() -> String {
    "general".to_string()
}

#[derive(Serialize)]
struct Report {
    verdict: String,
    engine: String,
    core_version: String,
    wave_dim: usize,
    task_id: String,
    domain: String,
    complexity_score: i64,
    mandatory_gate: bool,
    limits: Vec<String>,
    stable_triads: Vec<String>,
    weak_triads: Vec<String>,
    conflicts: Vec<String>,
    evidence_gaps: Vec<String>,
    baseline_summary: Value,
    wave_summary: Value,
    route_coherence: Value,
    structural_map: Value,
    explanation: Vec<String>,
    repair_prompt: String,
    trace_path: String,
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
        Command::Probe(args) => probe_cmd(args),
        Command::DatasetDoctor(args) => dataset_doctor_cmd(args),
        Command::Budget(args) => budget_cmd(args),
        Command::Pack6m(args) => pack6m_cmd(args),
        Command::Feedback(args) => feedback_cmd(args),
        Command::Eval(args) => eval_cmd(args),
        Command::Waw(args) => waw_cmd(args),
        Command::Serve(args) => serve_cmd(args),
        Command::Doctor(args) => doctor_cmd(args),
        Command::Dogfood(args) => dogfood_cmd(args),
        Command::Report(args) => report_cmd(args),
        Command::SelfCheck => self_check(),
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

fn load_packet(path: Option<&Path>) -> Result<Packet> {
    match path {
        Some(path) => {
            let text =
                fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
            Ok(serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?)
        }
        None => Ok(Packet {
            task_id: "stdin-empty".to_string(),
            domain: "general".to_string(),
            query: String::new(),
            triads: vec![],
            candidate_triads: vec![],
            candidate_answer: String::new(),
            negative_shortcuts: vec![],
            positive_shortcuts: vec![],
        }),
    }
}

fn load_packet_auto(
    path: &Path,
    input_format: &InputFormat,
    task_id: &str,
    domain: &str,
    query: &str,
    normalize_paths: bool,
) -> Result<Packet> {
    let is_json = match input_format {
        InputFormat::Json => true,
        InputFormat::Md => false,
        InputFormat::Auto => path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json")),
    };
    if is_json {
        load_packet(Some(path))
    } else {
        packet_from_markdown(path, task_id, domain, query, normalize_paths)
    }
}

fn normalize_ids(mut triads: Vec<Triad>, prefix: &str) -> Vec<Triad> {
    for (idx, triad) in triads.iter_mut().enumerate() {
        if triad.id.is_empty() {
            triad.id = format!("{prefix}{}", idx + 1);
        }
        if triad.subject_role.is_empty() {
            triad.subject_role = "subject".to_string();
        }
        if triad.object_role.is_empty() {
            triad.object_role = "object".to_string();
        }
        if triad.confidence == 0.0 {
            triad.confidence = 1.0;
        }
    }
    triads
}

fn norm(value: &str) -> String {
    value.trim().to_lowercase()
}

fn structural_key(triad: &Triad) -> (String, String, String) {
    (
        norm(&triad.subject),
        norm(&triad.relation),
        norm(&triad.object),
    )
}

fn reversed_structural_key(triad: &Triad) -> (String, String, String) {
    (
        norm(&triad.object),
        norm(&triad.relation),
        norm(&triad.subject),
    )
}

fn full_key(triad: &Triad) -> (String, String, String, String, String) {
    (
        norm(&triad.subject_role),
        norm(&triad.subject),
        norm(&triad.relation),
        norm(&triad.object_role),
        norm(&triad.object),
    )
}

fn vector(label: &str) -> Vec<i32> {
    let mut out = Vec::with_capacity(WAVE_DIM);
    let mut counter = 0u64;
    while out.len() < WAVE_DIM {
        let mut hasher = Sha256::new();
        hasher.update(format!("{label}|{counter}").as_bytes());
        let digest = hasher.finalize();
        for byte in digest {
            for bit in 0..8 {
                out.push(if ((byte >> bit) & 1) == 1 { 1 } else { -1 });
                if out.len() == WAVE_DIM {
                    break;
                }
            }
            if out.len() == WAVE_DIM {
                break;
            }
        }
        counter += 1;
    }
    out
}

fn bind(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().zip(b).map(|(x, y)| x * y).collect()
}

fn rotate(value: &[i32], amount: usize) -> Vec<i32> {
    if value.is_empty() {
        return vec![];
    }
    let amount = amount % value.len();
    value[amount..]
        .iter()
        .chain(value[..amount].iter())
        .copied()
        .collect()
}

fn add_into(acc: &mut [i32], value: &[i32]) {
    for (dst, src) in acc.iter_mut().zip(value) {
        *dst += src;
    }
}

fn cosine(a: &[i32], b: &[i32]) -> f64 {
    let dot: i64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| (*x as i64) * (*y as i64))
        .sum();
    let na: f64 = a
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    let nb: f64 = b
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot as f64 / (na * nb)
    }
}

fn triad_wave(triad: &Triad) -> Vec<i32> {
    let subject_binding = bind(
        &vector(&format!("role:{}", norm(&triad.subject_role))),
        &bind(
            &vector("position:subject"),
            &rotate(&vector(&format!("entity:{}", norm(&triad.subject))), 17),
        ),
    );
    let object_binding = bind(
        &vector(&format!("role:{}", norm(&triad.object_role))),
        &bind(
            &vector("position:object"),
            &rotate(&vector(&format!("entity:{}", norm(&triad.object))), 73),
        ),
    );
    let relation_mode = vector(&format!("relation:{}", norm(&triad.relation)));
    bind(&bind(&subject_binding, &relation_mode), &object_binding)
}

fn build_memory(source: &[Triad]) -> Vec<i32> {
    let mut memory = vec![0; WAVE_DIM];
    for triad in source {
        add_into(&mut memory, &triad_wave(triad));
    }
    memory
}

fn make_report(packet: &Packet, source: &[Triad], candidates: &[Triad]) -> Result<Report> {
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

    let verdict = if limits.iter().any(|x| x.contains("hard limit")) {
        "WATCH"
    } else if !conflicts.is_empty() {
        "VETO"
    } else if has_foreign_pull {
        "VETO"
    } else if !gaps.is_empty() || !weak_conf.is_empty() {
        "WATCH"
    } else if routes["weak"].as_array().is_some_and(|x| !x.is_empty()) {
        "VETO"
    } else if !candidates.is_empty() && wave["weak"].as_array().is_some_and(|x| !x.is_empty()) {
        "VETO"
    } else if complexity < MANDATORY_COMPLEXITY && candidates.is_empty() {
        "WATCH"
    } else if source.is_empty() {
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

fn entity_set(triads: &[Triad]) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for triad in triads {
        if !triad.subject.is_empty() {
            values.insert(norm(&triad.subject));
        }
        if !triad.object.is_empty() {
            values.insert(norm(&triad.object));
        }
    }
    values
}

fn role_set(triads: &[Triad]) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for triad in triads {
        values.insert(norm(&triad.subject_role));
        values.insert(norm(&triad.object_role));
    }
    values
}

fn relation_set(triads: &[Triad]) -> BTreeSet<String> {
    triads
        .iter()
        .filter(|x| !x.relation.is_empty())
        .map(|x| norm(&x.relation))
        .collect()
}

fn route_set(triads: &[Triad]) -> BTreeSet<String> {
    triads
        .iter()
        .filter(|x| !x.route.is_empty())
        .map(|x| norm(&x.route))
        .collect()
}

fn evidence_set(triads: &[Triad]) -> BTreeSet<String> {
    triads
        .iter()
        .filter(|x| !x.evidence.is_empty())
        .map(|x| norm(&x.evidence))
        .collect()
}

fn count_conflicting_sources(triads: &[Triad]) -> i64 {
    let mut by_evidence: HashMap<String, HashSet<(String, String, String)>> = HashMap::new();
    for triad in triads {
        if triad.evidence.is_empty() {
            continue;
        }
        by_evidence
            .entry(norm(&triad.evidence))
            .or_default()
            .insert(structural_key(triad));
    }
    by_evidence.values().filter(|keys| keys.len() > 1).count() as i64
}

fn complexity_score(source: &[Triad], candidates: &[Triad]) -> i64 {
    let all: Vec<Triad> = source.iter().chain(candidates).cloned().collect();
    entity_set(&all).len() as i64
        + all.len() as i64
        + 2 * route_set(&all).len() as i64
        + 2 * count_conflicting_sources(&all)
        + 3 * high_risk_role_swaps(source, candidates) as i64
}

fn high_risk_role_swaps(source: &[Triad], candidates: &[Triad]) -> usize {
    let source_keys: HashSet<_> = source.iter().map(full_key).collect();
    candidates
        .iter()
        .filter(|candidate| {
            let swapped = (
                norm(&candidate.object_role),
                norm(&candidate.object),
                norm(&candidate.relation),
                norm(&candidate.subject_role),
                norm(&candidate.subject),
            );
            source_keys.contains(&swapped)
        })
        .count()
}

fn limit_warnings(source: &[Triad], candidates: &[Triad], packet: &Packet) -> Vec<String> {
    let all: Vec<Triad> = source.iter().chain(candidates).cloned().collect();
    let counts = [
        ("entities", entity_set(&all).len(), 16, 32),
        ("roles", role_set(&all).len(), 8, 16),
        ("relations", relation_set(&all).len(), 16, 32),
        ("triads", all.len(), 32, 64),
        ("routes", route_set(&all).len(), 4, 8),
        ("evidence_refs", evidence_set(&all).len(), 32, 64),
        (
            "candidate_answers",
            if packet.candidate_answer.is_empty() {
                0
            } else {
                1
            },
            2,
            4,
        ),
    ];
    let mut out = vec![];
    for (name, value, target, hard) in counts {
        if value > hard {
            out.push(format!("{name} hard limit exceeded: {value}>{hard}"));
        } else if value > target {
            out.push(format!("{name} target limit exceeded: {value}>{target}"));
        }
    }
    out
}

fn evidence_gaps(source: &[Triad], candidates: &[Triad]) -> Vec<String> {
    source
        .iter()
        .chain(candidates)
        .filter(|triad| triad.evidence.trim().is_empty())
        .map(|triad| triad.id.clone())
        .collect()
}

fn low_confidence(source: &[Triad], candidates: &[Triad]) -> Vec<String> {
    source
        .iter()
        .chain(candidates)
        .filter(|triad| triad.confidence < 0.7)
        .map(|triad| triad.id.clone())
        .collect()
}

fn symbolic_conflicts(source: &[Triad], candidates: &[Triad]) -> Vec<String> {
    let source_structural: HashSet<_> = source.iter().map(structural_key).collect();
    let source_reversed: HashSet<_> = source.iter().map(reversed_structural_key).collect();
    let mut conflicts = evidence_conflicts(source);
    for candidate in candidates {
        if source_reversed.contains(&structural_key(candidate))
            && !symmetric_relation(&candidate.relation)
        {
            conflicts.push(format!(
                "{} reverses a non-symmetric source relation",
                candidate.id
            ));
        }
        if functional_relation(&candidate.relation) {
            for source_tri in source {
                if norm(&source_tri.relation) == norm(&candidate.relation)
                    && norm(&source_tri.subject) == norm(&candidate.subject)
                    && norm(&source_tri.object) != norm(&candidate.object)
                {
                    conflicts.push(format!(
                        "{} changes functional object for {}",
                        candidate.id, candidate.subject
                    ));
                }
            }
        }
        if !source_structural.contains(&structural_key(candidate)) {
            for source_tri in source {
                if norm(&source_tri.subject) == norm(&candidate.object)
                    && norm(&source_tri.object) == norm(&candidate.subject)
                    && norm(&source_tri.relation) == norm(&candidate.relation)
                    && norm(&source_tri.subject_role) == norm(&candidate.object_role)
                    && norm(&source_tri.object_role) == norm(&candidate.subject_role)
                {
                    conflicts.push(format!(
                        "{} swaps roles for relation {}",
                        candidate.id, candidate.relation
                    ));
                }
            }
        }
    }
    conflicts.sort();
    conflicts.dedup();
    conflicts
}

fn evidence_conflicts(triads: &[Triad]) -> Vec<String> {
    let mut by_evidence: HashMap<String, Vec<&Triad>> = HashMap::new();
    for triad in triads {
        if triad.evidence.trim().is_empty() {
            continue;
        }
        by_evidence
            .entry(norm(&triad.evidence))
            .or_default()
            .push(triad);
    }
    let mut conflicts = vec![];
    for (evidence, items) in by_evidence {
        let mut subject_slot: HashMap<(String, String, String), BTreeSet<String>> = HashMap::new();
        let mut object_slot: HashMap<(String, String, String), BTreeSet<String>> = HashMap::new();
        for triad in &items {
            subject_slot
                .entry((
                    norm(&triad.relation),
                    norm(&triad.object_role),
                    norm(&triad.object),
                ))
                .or_default()
                .insert(norm(&triad.subject));
            object_slot
                .entry((
                    norm(&triad.subject_role),
                    norm(&triad.subject),
                    norm(&triad.relation),
                ))
                .or_default()
                .insert(norm(&triad.object));
        }
        if subject_slot.values().any(|values| values.len() > 1)
            || object_slot.values().any(|values| values.len() > 1)
        {
            let ids = items
                .iter()
                .map(|triad| triad.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            conflicts.push(format!(
                "evidence {evidence} supports incompatible role fillers: {ids}"
            ));
        }
    }
    conflicts
}

fn symmetric_relation(relation: &str) -> bool {
    matches!(
        norm(relation).as_str(),
        "equals" | "same_as" | "related_to" | "matches" | "connected_to"
    )
}

fn functional_relation(relation: &str) -> bool {
    matches!(
        norm(relation).as_str(),
        "pays"
            | "pays_to"
            | "supplies"
            | "buys"
            | "owns"
            | "imports"
            | "exports"
            | "certifies"
            | "applies_for"
            | "manufactures"
            | "delivers_to"
    )
}

fn group_memories(triads: &[Triad], fallback: &str) -> BTreeMap<String, Vec<i32>> {
    let mut memories = BTreeMap::new();
    for triad in triads {
        let group = if triad.group.trim().is_empty() {
            fallback.to_string()
        } else {
            norm(&triad.group)
        };
        let memory = memories.entry(group).or_insert_with(|| vec![0; WAVE_DIM]);
        add_into(memory, &triad_wave(triad));
    }
    memories
}

fn route_memories(triads: &[Triad], fallback: &str) -> BTreeMap<String, Vec<i32>> {
    let mut memories = BTreeMap::new();
    for triad in triads {
        let route = route_name(triad, fallback);
        let memory = memories.entry(route).or_insert_with(|| vec![0; WAVE_DIM]);
        add_into(memory, &triad_wave(triad));
    }
    memories
}

fn route_coherence(source: &[Triad], candidates: &[Triad]) -> Value {
    if source.is_empty() || candidates.is_empty() {
        return json!({"scores": {}, "weak": [], "best_source_group": {}});
    }
    let source_groups = group_memories(source, "source");
    let candidate_groups = group_memories(candidates, "candidate");
    let mut scores = serde_json::Map::new();
    let mut best_source_group = serde_json::Map::new();
    let mut weak = vec![];
    for (candidate_group, candidate_memory) in candidate_groups {
        let mut best_name = String::new();
        let mut best_score = -1.0;
        for (source_group, source_memory) in &source_groups {
            let score = cosine(source_memory, &candidate_memory);
            if score > best_score {
                best_score = score;
                best_name = source_group.clone();
            }
        }
        scores.insert(candidate_group.clone(), json!(round4(best_score)));
        best_source_group.insert(candidate_group.clone(), json!(best_name));
        let candidate_size = candidates
            .iter()
            .filter(|triad| {
                let group = if triad.group.trim().is_empty() {
                    "candidate".to_string()
                } else {
                    norm(&triad.group)
                };
                group == candidate_group
            })
            .count();
        let exact_source_groups = exact_match_source_groups(source, candidates, &candidate_group);
        if candidate_size >= 2 && (best_score < 0.65 || exact_source_groups.len() > 1) {
            weak.push(candidate_group);
        }
    }
    json!({"scores": scores, "weak": weak, "best_source_group": best_source_group})
}

fn exact_match_source_groups(
    source: &[Triad],
    candidates: &[Triad],
    candidate_group: &str,
) -> BTreeSet<String> {
    let mut source_by_key: HashMap<(String, String, String), BTreeSet<String>> = HashMap::new();
    for triad in source {
        source_by_key
            .entry(structural_key(triad))
            .or_default()
            .insert(if triad.group.trim().is_empty() {
                "source".to_string()
            } else {
                norm(&triad.group)
            });
    }
    let mut groups = BTreeSet::new();
    for candidate in candidates {
        let group = if candidate.group.trim().is_empty() {
            "candidate".to_string()
        } else {
            norm(&candidate.group)
        };
        if group != candidate_group {
            continue;
        }
        if let Some(matches) = source_by_key.get(&structural_key(candidate)) {
            groups.extend(matches.iter().cloned());
        }
    }
    groups
}

fn structural_map(source: &[Triad], candidates: &[Triad]) -> Value {
    let source_groups = group_memories(source, "source");
    let candidate_groups = group_memories(candidates, "candidate");
    let source_routes = route_memories(source, "source-route");
    let candidate_routes = route_memories(candidates, "candidate-route");
    let mut source_group_sizes = serde_json::Map::new();
    let mut candidate_group_sizes = serde_json::Map::new();
    for triad in source {
        let group = group_name(triad, "source");
        let current = source_group_sizes
            .get(&group)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        source_group_sizes.insert(group, json!(current + 1));
    }
    for triad in candidates {
        let group = group_name(triad, "candidate");
        let current = candidate_group_sizes
            .get(&group)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        candidate_group_sizes.insert(group, json!(current + 1));
    }

    let mut interference_matrix = serde_json::Map::new();
    let mut dominant_source_group = serde_json::Map::new();
    let mut mixed_candidate_groups = vec![];
    let mut group_links = serde_json::Map::new();
    let mut stable_groups = BTreeSet::new();
    let mut repair_tasks = vec![];
    let mut foreign_pull = vec![];

    for (candidate_group, candidate_memory) in &candidate_groups {
        let mut row = serde_json::Map::new();
        let mut best_group = String::new();
        let mut best_score = -1.0;
        for (source_group, source_memory) in &source_groups {
            let score = round4(cosine(source_memory, candidate_memory));
            row.insert(source_group.clone(), json!(score));
            if score > best_score {
                best_score = score;
                best_group = source_group.clone();
            }
        }
        let exact_groups = exact_match_source_groups(source, candidates, candidate_group);
        let exact_groups_vec: Vec<String> = exact_groups.iter().cloned().collect();
        if exact_groups.len() > 1 {
            mixed_candidate_groups.push(candidate_group.clone());
            repair_tasks.push(json!({
                "candidate_group": candidate_group,
                "reason": "candidate group contains exact triads from multiple source groups",
                "source_groups": exact_groups_vec,
                "suggested_fix": "Split this candidate group by source route before finalizing."
            }));
        }
        if !best_group.is_empty() && best_score < 0.65 {
            repair_tasks.push(json!({
                "candidate_group": candidate_group,
                "reason": "candidate group has low interference coherence with every source group",
                "dominant_source_group": best_group,
                "score": best_score,
                "suggested_fix": "Check whether this candidate route is missing triads, using wrong roles, or mixing unrelated routes."
            }));
        }
        group_links.insert(
            candidate_group.clone(),
            json!({
                "dominant_source_group": best_group.clone(),
                "dominant_score": round4(best_score),
                "exact_source_groups": exact_groups_vec
            }),
        );
        if best_score >= 0.65 && exact_groups.len() <= 1 {
            stable_groups.insert(best_group.clone());
        }
        dominant_source_group.insert(candidate_group.clone(), json!(best_group));
        interference_matrix.insert(candidate_group.clone(), json!(row));
    }

    for candidate in candidates {
        let candidate_wave = triad_wave(candidate);
        let candidate_group = group_name(candidate, "candidate");
        let candidate_dominant = dominant_source_group
            .get(&candidate_group)
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let (best_source_group, best_source_score) =
            best_memory_match(&source_groups, &candidate_wave);
        let exact_groups = exact_match_source_groups_for_candidate(source, candidate);
        let exact_groups_vec: Vec<String> = exact_groups.iter().cloned().collect();
        let is_foreign = !candidate_dominant.is_empty()
            && !best_source_group.is_empty()
            && (best_source_group != candidate_dominant
                || exact_groups.len() > 1
                || (!exact_groups.is_empty() && !exact_groups.contains(candidate_dominant)));
        if is_foreign {
            foreign_pull.push(json!({
                "candidate_triad": candidate.id,
                "candidate_group": candidate_group,
                "dominant_source_group": candidate_dominant,
                "triad_best_source_group": best_source_group,
                "triad_best_score": round4(best_source_score),
                "exact_source_groups": exact_groups_vec,
                "relation": candidate.relation,
                "subject": candidate.subject,
                "object": candidate.object,
                "repair": "Move this triad to the matching route or split the candidate group."
            }));
        }
    }

    json!({
        "core_version": CORE_VERSION,
        "wave_dim": nanda_6m::WAVE_DIM,
        "source_group_sizes": source_group_sizes,
        "candidate_group_sizes": candidate_group_sizes,
        "group_centroids": {
            "source": centroid_summary(source, &source_groups, GroupAxis::Group),
            "candidate": centroid_summary(candidates, &candidate_groups, GroupAxis::Group)
        },
        "route_memory": {
            "source": centroid_summary(source, &source_routes, GroupAxis::Route),
            "candidate": centroid_summary(candidates, &candidate_routes, GroupAxis::Route),
            "interference_matrix": interference_matrix_for(&source_routes, &candidate_routes)
        },
        "candidate_superposition": memory_summary(candidates, &build_memory(candidates)),
        "interference_matrix": interference_matrix,
        "dominant_source_group": dominant_source_group,
        "group_links": group_links,
        "stable_groups": stable_groups.into_iter().collect::<Vec<_>>(),
        "mixed_candidate_groups": mixed_candidate_groups,
        "foreign_pull": foreign_pull,
        "repair_tasks": repair_tasks
    })
}

fn group_name(triad: &Triad, fallback: &str) -> String {
    if triad.group.trim().is_empty() {
        fallback.to_string()
    } else {
        norm(&triad.group)
    }
}

fn route_name(triad: &Triad, fallback: &str) -> String {
    if triad.route.trim().is_empty() {
        fallback.to_string()
    } else {
        norm(&triad.route)
    }
}

enum GroupAxis {
    Group,
    Route,
}

fn centroid_summary(
    triads: &[Triad],
    memories: &BTreeMap<String, Vec<i32>>,
    axis: GroupAxis,
) -> Value {
    let mut out = serde_json::Map::new();
    for (name, memory) in memories {
        let members: Vec<&Triad> = triads
            .iter()
            .filter(|triad| match axis {
                GroupAxis::Group => {
                    group_name(triad, "source") == *name || group_name(triad, "candidate") == *name
                }
                GroupAxis::Route => {
                    route_name(triad, "source-route") == *name
                        || route_name(triad, "candidate-route") == *name
                }
            })
            .collect();
        let coherence = if members.is_empty() {
            0.0
        } else {
            members
                .iter()
                .map(|triad| cosine(memory, &triad_wave(triad)))
                .sum::<f64>()
                / members.len() as f64
        };
        out.insert(
            name.clone(),
            json!({
                "triads": members.len(),
                "norm": round4(norm2_i32(memory)),
                "self_coherence": round4(coherence)
            }),
        );
    }
    json!(out)
}

fn memory_summary(triads: &[Triad], memory: &[i32]) -> Value {
    let scores: Vec<f64> = triads
        .iter()
        .map(|triad| cosine(memory, &triad_wave(triad)))
        .collect();
    let avg = if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    };
    json!({
        "triads": triads.len(),
        "norm": round4(norm2_i32(memory)),
        "avg_triad_coherence": round4(avg)
    })
}

fn interference_matrix_for(
    source_memories: &BTreeMap<String, Vec<i32>>,
    candidate_memories: &BTreeMap<String, Vec<i32>>,
) -> Value {
    let mut out = serde_json::Map::new();
    for (candidate_name, candidate_memory) in candidate_memories {
        let mut row = serde_json::Map::new();
        for (source_name, source_memory) in source_memories {
            row.insert(
                source_name.clone(),
                json!(round4(cosine(source_memory, candidate_memory))),
            );
        }
        out.insert(candidate_name.clone(), json!(row));
    }
    json!(out)
}

fn best_memory_match(memories: &BTreeMap<String, Vec<i32>>, wave: &[i32]) -> (String, f64) {
    let mut best_name = String::new();
    let mut best_score = -1.0;
    for (name, memory) in memories {
        let score = cosine(memory, wave);
        if score > best_score {
            best_score = score;
            best_name = name.clone();
        }
    }
    (best_name, best_score)
}

fn exact_match_source_groups_for_candidate(
    source: &[Triad],
    candidate: &Triad,
) -> BTreeSet<String> {
    let mut groups = BTreeSet::new();
    let candidate_key = structural_key(candidate);
    for triad in source {
        if structural_key(triad) == candidate_key {
            groups.insert(group_name(triad, "source"));
        }
    }
    groups
}

fn norm2_i32(a: &[i32]) -> f64 {
    a.iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt()
}

fn score_candidates(source: &[Triad], candidates: &[Triad]) -> Value {
    if source.is_empty() {
        return json!({"stable": [], "weak": [], "scores": {}, "weak_details": {}, "source_self_score": 0.0});
    }
    let memory = build_memory(source);
    let mut stable = vec![];
    let mut weak = vec![];
    let mut scores = serde_json::Map::new();
    let mut weak_details = serde_json::Map::new();
    for candidate in candidates {
        let candidate_wave = triad_wave(candidate);
        let score = cosine(&memory, &candidate_wave);
        scores.insert(candidate.id.clone(), json!(round4(score)));
        if score >= 0.28 {
            stable.push(candidate.id.clone());
        } else {
            weak.push(candidate.id.clone());
            let (nearest_id, nearest_score) = nearest_source(source, &candidate_wave);
            weak_details.insert(
                candidate.id.clone(),
                json!({
                    "score": round4(score),
                    "nearest_source": nearest_id,
                    "nearest_source_score": round4(nearest_score),
                    "why_weak": "candidate triad does not resonate with the composite source memory above the stability threshold",
                    "suggested_fix": "Check subject/object roles, relation name, group, route, and evidence for this candidate triad."
                }),
            );
        }
    }
    let source_scores: Vec<f64> = source
        .iter()
        .map(|triad| cosine(&memory, &triad_wave(triad)))
        .collect();
    let source_self_score = if source_scores.is_empty() {
        0.0
    } else {
        round4(source_scores.iter().sum::<f64>() / source_scores.len() as f64)
    };
    json!({"stable": stable, "weak": weak, "scores": scores, "weak_details": weak_details, "source_self_score": source_self_score})
}

fn nearest_source(source: &[Triad], candidate_wave: &[i32]) -> (String, f64) {
    let mut best_id = String::new();
    let mut best_score = -1.0;
    for triad in source {
        let score = cosine(&triad_wave(triad), candidate_wave);
        if score > best_score {
            best_score = score;
            best_id = triad.id.clone();
        }
    }
    (best_id, best_score)
}

fn baseline_summary(source: &[Triad], candidates: &[Triad]) -> Value {
    let source_tokens: HashSet<String> = source
        .iter()
        .flat_map(|t| [norm(&t.subject), norm(&t.relation), norm(&t.object)])
        .collect();
    let source_structural: HashSet<_> = source.iter().map(structural_key).collect();
    let source_reversed: HashSet<_> = source.iter().map(reversed_structural_key).collect();
    let mut exact = vec![];
    let mut reversed = vec![];
    let mut overlap = serde_json::Map::new();
    for candidate in candidates {
        if source_structural.contains(&structural_key(candidate)) {
            exact.push(candidate.id.clone());
        }
        if source_reversed.contains(&structural_key(candidate)) {
            reversed.push(candidate.id.clone());
        }
        let candidate_tokens: HashSet<String> = [
            norm(&candidate.subject),
            norm(&candidate.relation),
            norm(&candidate.object),
        ]
        .into_iter()
        .collect();
        let union = source_tokens.union(&candidate_tokens).count().max(1);
        let inter = source_tokens.intersection(&candidate_tokens).count();
        overlap.insert(
            candidate.id.clone(),
            json!(round4(inter as f64 / union as f64)),
        );
    }
    json!({"exact_matches": exact, "reversed_hits": reversed, "token_overlap": overlap})
}

fn build_explanation(report: &Report) -> Vec<String> {
    let mut notes = vec![];
    if report.verdict == "PASS" {
        notes.push("Candidate structure is coherent with source triads.".to_string());
    }
    if !report.conflicts.is_empty() {
        notes.push("Structural conflicts were detected.".to_string());
    }
    if report.route_coherence["weak"]
        .as_array()
        .is_some_and(|x| !x.is_empty())
    {
        notes.push("At least one candidate group has weak route coherence.".to_string());
    }
    if report.structural_map["foreign_pull"]
        .as_array()
        .is_some_and(|x| !x.is_empty())
    {
        notes.push("Route splice suspected: at least one candidate triad pulls toward a different source group.".to_string());
    }
    if report.wave_summary["weak"]
        .as_array()
        .is_some_and(|x| !x.is_empty())
    {
        notes.push("At least one candidate triad has weak composite-mode support.".to_string());
    }
    if !report.evidence_gaps.is_empty() {
        notes.push("Evidence is missing for one or more triads.".to_string());
    }
    if !report.limits.is_empty() {
        notes.push("Task exceeds target or hard limits and should be split.".to_string());
    }
    if notes.is_empty() {
        notes.push("No decisive structural signal was found.".to_string());
    }
    notes
}

fn build_repair_prompt(report: &Report) -> String {
    if report.verdict == "PASS" {
        return "No repair needed. Preserve the checked source/candidate bindings.".to_string();
    }
    let mut lines = vec![
        "Repair the candidate answer using only one coherent structural route.".to_string(),
        format!("NANDA verdict: {}.", report.verdict),
    ];
    if !report.conflicts.is_empty() {
        lines.push("Fix these conflicts:".to_string());
        for item in &report.conflicts {
            lines.push(format!("- {item}"));
        }
    }
    if let Some(weak) = report.route_coherence["weak"].as_array() {
        if !weak.is_empty() {
            lines.push(
                "Do not splice triads from different source groups into one candidate group."
                    .to_string(),
            );
            for group in weak {
                if let Some(group) = group.as_str() {
                    let best = report.route_coherence["best_source_group"][group]
                        .as_str()
                        .unwrap_or("");
                    let score = &report.route_coherence["scores"][group];
                    lines.push(format!(
                        "- candidate group {group} best matches {best} only weakly: {score}"
                    ));
                }
            }
        }
    }
    if let Some(pulls) = report.structural_map["foreign_pull"].as_array() {
        if !pulls.is_empty() {
            lines.push("Route splice suspected from foreign_pull:".to_string());
            for pull in pulls {
                lines.push(format!(
                    "- {} in {} pulls from {} toward {}; repair: {}",
                    pull["candidate_triad"].as_str().unwrap_or(""),
                    pull["candidate_group"].as_str().unwrap_or(""),
                    pull["dominant_source_group"].as_str().unwrap_or(""),
                    pull["triad_best_source_group"].as_str().unwrap_or(""),
                    pull["repair"]
                        .as_str()
                        .unwrap_or("split or repair this candidate triad")
                ));
            }
        }
    }
    if !report.weak_triads.is_empty() {
        lines.push("Recheck or remove weak candidate triads:".to_string());
        for item in &report.weak_triads {
            if let Some(detail) = report.wave_summary["weak_details"].get(item) {
                lines.push(format!(
                    "- {item}: score={}, nearest_source={}, why={}",
                    detail["score"],
                    detail["nearest_source"].as_str().unwrap_or(""),
                    detail["why_weak"].as_str().unwrap_or("")
                ));
            } else {
                lines.push(format!("- {item}"));
            }
        }
    }
    if !report.evidence_gaps.is_empty() {
        lines.push("Add evidence before finalizing:".to_string());
        for item in &report.evidence_gaps {
            lines.push(format!("- {item}"));
        }
    }
    if !report.limits.is_empty() {
        lines.push("Split the task before retrying:".to_string());
        for item in &report.limits {
            lines.push(format!("- {item}"));
        }
    }
    lines.join("\n")
}

fn write_trace(report: &Report) -> Result<String> {
    let dir = std::env::temp_dir().join("nanda-structural-gate");
    fs::create_dir_all(&dir)?;
    let task_id = report.task_id.replace('/', "_");
    let path = dir.join(format!("{task_id}.trace.json"));
    fs::write(&path, serde_json::to_string_pretty(report)? + "\n")?;
    Ok(path.display().to_string())
}

fn print_report(report: &Report, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => {
            println!("verdict: {}", report.verdict);
            println!("engine: {}", report.engine);
            println!("task_id: {}", report.task_id);
            println!("complexity_score: {}", report.complexity_score);
            println!("mandatory_gate: {}", report.mandatory_gate);
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
            println!("- trace: `{}`", report.trace_path);
        }
    }
    Ok(())
}

fn print_map_text(map: &Value) {
    println!(
        "core_version: {}",
        map["core_version"].as_str().unwrap_or("")
    );
    println!("wave_dim: {}", map["wave_dim"].as_u64().unwrap_or(0));
    println!("mixed_candidate_groups:");
    if let Some(groups) = map["mixed_candidate_groups"].as_array() {
        for group in groups {
            println!("  - {}", group.as_str().unwrap_or(""));
        }
    }
    println!("repair_tasks:");
    if let Some(tasks) = map["repair_tasks"].as_array() {
        for task in tasks {
            println!(
                "  - {}: {}",
                task["candidate_group"].as_str().unwrap_or(""),
                task["reason"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_map_md(map: &Value) {
    println!("# NANDA Structural Map\n");
    println!(
        "- core_version: `{}`",
        map["core_version"].as_str().unwrap_or("")
    );
    println!("- wave_dim: `{}`", map["wave_dim"].as_u64().unwrap_or(0));
    if let Some(groups) = map["mixed_candidate_groups"].as_array() {
        if !groups.is_empty() {
            println!("\n## Mixed Candidate Groups\n");
            for group in groups {
                println!("- `{}`", group.as_str().unwrap_or(""));
            }
        }
    }
    if let Some(tasks) = map["repair_tasks"].as_array() {
        if !tasks.is_empty() {
            println!("\n## Repair Tasks\n");
            for task in tasks {
                println!(
                    "- `{}`: {}",
                    task["candidate_group"].as_str().unwrap_or(""),
                    task["reason"].as_str().unwrap_or("")
                );
            }
        }
    }
}

fn verdict_code(verdict: &str) -> u8 {
    match verdict {
        "PASS" => EXIT_PASS,
        "VETO" => EXIT_VETO,
        "WATCH" => EXIT_WATCH,
        _ => EXIT_ERROR,
    }
}

fn init_task(args: InitTaskArgs) -> Result<u8> {
    let packet = Packet {
        task_id: args.task_id.clone(),
        domain: args.domain,
        query: args.query,
        triads: vec![Triad {
            id: "t1".to_string(),
            subject: String::new(),
            relation: String::new(),
            object: String::new(),
            evidence: String::new(),
            confidence: 0.9,
            subject_role: String::new(),
            object_role: String::new(),
            route: String::new(),
            group: String::new(),
        }],
        candidate_triads: vec![],
        candidate_answer: String::new(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
    };
    let output = serde_json::to_string_pretty(&packet)? + "\n";
    write_or_print(
        args.out
            .unwrap_or_else(|| PathBuf::from(format!("nanda-task-{}.json", slug(&args.task_id)))),
        args.stdout,
        output,
    )?;
    Ok(EXIT_PASS)
}

fn init_md(args: InitMdArgs) -> Result<u8> {
    let text = template_text(&args.task_id, &args.domain, &args.query, &args.template);
    write_or_print(
        args.out
            .unwrap_or_else(|| PathBuf::from(format!("nanda-task-{}.md", slug(&args.task_id)))),
        args.stdout,
        text,
    )?;
    Ok(EXIT_PASS)
}

fn pack_from_md(args: PackArgs) -> Result<u8> {
    let packet = packet_from_markdown(
        &args.input,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let output = serde_json::to_string_pretty(&packet)? + "\n";
    let out = args
        .out
        .unwrap_or_else(|| args.input.with_extension("nanda.json"));
    write_or_print(out, args.stdout, output)?;
    Ok(EXIT_PASS)
}

fn gate_md(args: GateMdArgs) -> Result<u8> {
    let packet = packet_from_markdown(
        &args.input,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let report = make_report(&packet, &source, &candidates)?;
    print_report(&report, &args.format)?;
    Ok(verdict_code(&report.verdict))
}

fn split_md(args: SplitArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.input)?;
    let (triads, candidates) = parse_markdown_tables(&text, args.normalize_paths);
    if matches!(args.by, SplitBy::LinkedGroup) {
        return split_md_linked_group(args, triads, candidates);
    }
    let key_of = |row: &Triad| -> String {
        let raw = match args.by {
            SplitBy::Group => &row.group,
            SplitBy::Route => &row.route,
            SplitBy::LinkedGroup => unreachable!("linked-group split is handled before raw split"),
        };
        if raw.trim().is_empty() {
            "ungrouped".to_string()
        } else {
            raw.clone()
        }
    };
    let keys: BTreeSet<String> = triads.iter().chain(candidates.iter()).map(key_of).collect();
    fs::create_dir_all(&args.out_dir)?;
    let prefix = args.prefix.unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let mut written = vec![];
    for key in keys {
        let src: Vec<Triad> = triads
            .iter()
            .filter(|row| key_of(row) == key)
            .cloned()
            .collect();
        let cand: Vec<Triad> = candidates
            .iter()
            .filter(|row| key_of(row) == key)
            .cloned()
            .collect();
        let path = args.out_dir.join(format!(
            "{}-{}-{}.md",
            prefix,
            split_label(&args.by),
            slug(&key)
        ));
        fs::write(
            &path,
            split_worksheet(&args.input, split_label(&args.by), &key, &src, &cand),
        )?;
        written.push(path.display().to_string());
    }
    println!("{}", serde_json::to_string_pretty(&written)?);
    Ok(EXIT_PASS)
}

fn split_packet(args: SplitPacketArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let triads = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    fs::create_dir_all(&args.out_dir)?;
    let prefix = args.prefix.unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let splits = match args.by {
        SplitBy::LinkedGroup => linked_group_splits(&triads, &candidates),
        SplitBy::Group | SplitBy::Route => raw_splits(&triads, &candidates, &args.by),
    };
    let mut written = vec![];
    for split in &splits.items {
        let stem = format!("{}-{}-{}", prefix, split_label(&args.by), slug(&split.key));
        let path = match args.format {
            SplitOutputFormat::Json => args.out_dir.join(format!("{stem}.json")),
            SplitOutputFormat::Md => args.out_dir.join(format!("{stem}.md")),
        };
        match args.format {
            SplitOutputFormat::Json => {
                let split_packet = Packet {
                    task_id: format!("{}:{}", packet.task_id, split.key),
                    domain: packet.domain.clone(),
                    query: packet.query.clone(),
                    triads: split.triads.clone(),
                    candidate_triads: split.candidates.clone(),
                    candidate_answer: packet.candidate_answer.clone(),
                    negative_shortcuts: packet.negative_shortcuts.clone(),
                    positive_shortcuts: packet.positive_shortcuts.clone(),
                };
                fs::write(&path, serde_json::to_string_pretty(&split_packet)? + "\n")?;
            }
            SplitOutputFormat::Md => {
                fs::write(
                    &path,
                    split_worksheet(
                        &args.input,
                        split_label(&args.by),
                        &split.key,
                        &split.triads,
                        &split.candidates,
                    ),
                )?;
            }
        }
        written.push(path.display().to_string());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "mode": split_label(&args.by),
            "format": split_output_label(&args.format),
            "written": written,
            "warnings": splits.warnings
        }))?
    );
    Ok(EXIT_PASS)
}

fn split_md_linked_group(
    args: SplitArgs,
    triads: Vec<Triad>,
    candidates: Vec<Triad>,
) -> Result<u8> {
    let splits = linked_group_splits(&triads, &candidates);
    fs::create_dir_all(&args.out_dir)?;
    let prefix = args.prefix.unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let mut written = vec![];
    for split in &splits.items {
        let path = args
            .out_dir
            .join(format!("{}-linked-group-{}.md", prefix, slug(&split.key)));
        fs::write(
            &path,
            split_worksheet(
                &args.input,
                "linked-group",
                &split.key,
                &split.triads,
                &split.candidates,
            ),
        )?;
        written.push(path.display().to_string());
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "mode": "linked-group",
            "written": written,
            "warnings": splits.warnings
        }))?
    );
    Ok(EXIT_PASS)
}

struct SplitItem {
    key: String,
    triads: Vec<Triad>,
    candidates: Vec<Triad>,
}

struct SplitResult {
    items: Vec<SplitItem>,
    warnings: Vec<String>,
}

fn raw_splits(triads: &[Triad], candidates: &[Triad], by: &SplitBy) -> SplitResult {
    let key_of = |row: &Triad| -> String {
        let raw = match by {
            SplitBy::Group => &row.group,
            SplitBy::Route => &row.route,
            SplitBy::LinkedGroup => unreachable!("linked-group has a separate split mode"),
        };
        if raw.trim().is_empty() {
            "ungrouped".to_string()
        } else {
            raw.clone()
        }
    };
    let keys: BTreeSet<String> = triads.iter().chain(candidates.iter()).map(key_of).collect();
    let items = keys
        .into_iter()
        .map(|key| SplitItem {
            triads: triads
                .iter()
                .filter(|row| key_of(row) == key)
                .cloned()
                .collect(),
            candidates: candidates
                .iter()
                .filter(|row| key_of(row) == key)
                .cloned()
                .collect(),
            key,
        })
        .collect();
    SplitResult {
        items,
        warnings: vec![],
    }
}

fn linked_group_splits(triads: &[Triad], candidates: &[Triad]) -> SplitResult {
    let map = structural_map(triads, candidates);
    let mut candidate_to_sources: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if let Some(group_links) = map["group_links"].as_object() {
        for (candidate_group, link) in group_links {
            let entry = candidate_to_sources
                .entry(candidate_group.clone())
                .or_default();
            if let Some(source_group) = link["dominant_source_group"].as_str() {
                if !source_group.is_empty() {
                    entry.insert(source_group.to_string());
                }
            }
            if let Some(exact_groups) = link["exact_source_groups"].as_array() {
                for source_group in exact_groups {
                    if let Some(source_group) = source_group.as_str() {
                        if !source_group.is_empty() {
                            entry.insert(source_group.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut source_to_candidates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (candidate_group, source_groups) in &candidate_to_sources {
        for source_group in source_groups {
            source_to_candidates
                .entry(source_group.clone())
                .or_default()
                .insert(candidate_group.clone());
        }
    }

    let source_groups: BTreeSet<String> = triads
        .iter()
        .map(|triad| group_name(triad, "source"))
        .collect();
    let mut items = vec![];
    let mut warnings = vec![];
    for source_group in source_groups {
        let source_rows: Vec<Triad> = triads
            .iter()
            .filter(|row| group_name(row, "source") == source_group)
            .cloned()
            .collect();
        let linked_candidate_groups = source_to_candidates
            .get(&source_group)
            .cloned()
            .unwrap_or_default();
        let candidate_rows: Vec<Triad> = candidates
            .iter()
            .filter(|row| {
                let exact_groups = exact_match_source_groups_for_candidate(triads, row);
                if !exact_groups.is_empty() {
                    return exact_groups.contains(&source_group);
                }
                linked_candidate_groups.contains(&group_name(row, "candidate"))
            })
            .cloned()
            .collect();
        if candidate_rows.is_empty() {
            warnings.push(format!(
                "source group {source_group} has no linked candidate group"
            ));
        }
        items.push(SplitItem {
            key: source_group,
            triads: source_rows,
            candidates: candidate_rows,
        });
    }

    let linked_candidates: BTreeSet<String> = candidate_to_sources.keys().cloned().collect();
    let all_candidate_groups: BTreeSet<String> = candidates
        .iter()
        .map(|triad| group_name(triad, "candidate"))
        .collect();
    for candidate_group in all_candidate_groups.difference(&linked_candidates) {
        warnings.push(format!(
            "candidate group {candidate_group} has no linked source group"
        ));
    }

    SplitResult { items, warnings }
}

fn map_cmd(args: MapArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let map = structural_map(&source, &candidates);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&map)?),
        OutputFormat::Text => print_map_text(&map),
        OutputFormat::Md => print_map_md(&map),
    }
    Ok(EXIT_PASS)
}

fn hgate_cmd(args: HgateArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let global_report = make_report(&packet, &source, &candidates)?;
    let global_map = structural_map(&source, &candidates);
    let splits = match args.by {
        SplitBy::LinkedGroup => linked_group_splits(&source, &candidates),
        SplitBy::Group | SplitBy::Route => raw_splits(&source, &candidates, &args.by),
    };
    let mut branches = vec![];
    for split in splits.items.iter().take(args.max_branches) {
        let branch_packet = Packet {
            task_id: format!("{}:{}", packet.task_id, split.key),
            domain: packet.domain.clone(),
            query: packet.query.clone(),
            triads: split.triads.clone(),
            candidate_triads: split.candidates.clone(),
            candidate_answer: packet.candidate_answer.clone(),
            negative_shortcuts: packet.negative_shortcuts.clone(),
            positive_shortcuts: packet.positive_shortcuts.clone(),
        };
        let report = make_report(&branch_packet, &split.triads, &split.candidates)?;
        branches.push(json!({
            "key": split.key,
            "source_triads": split.triads.len(),
            "candidate_triads": split.candidates.len(),
            "verdict": report.verdict,
            "limits": report.limits,
            "conflicts": report.conflicts,
            "evidence_gaps": report.evidence_gaps,
            "weak_triads": report.weak_triads,
            "foreign_pull": report.structural_map["foreign_pull"],
            "repair_prompt": report.repair_prompt,
            "trace_path": report.trace_path
        }));
    }
    let truncated = splits.items.len().saturating_sub(branches.len());
    let decision = hierarchical_decision(&global_report, &global_map, &branches, truncated);
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "hierarchical-gate",
        "input": args.input,
        "split_by": split_label(&args.by),
        "global": {
            "verdict": global_report.verdict,
            "complexity_score": global_report.complexity_score,
            "limits": global_report.limits,
            "conflicts": global_report.conflicts,
            "evidence_gaps": global_report.evidence_gaps,
            "weak_triads": global_report.weak_triads,
            "foreign_pull": global_map["foreign_pull"],
            "mixed_candidate_groups": global_map["mixed_candidate_groups"],
            "repair_tasks": global_map["repair_tasks"],
            "trace_path": global_report.trace_path
        },
        "branches": branches,
        "split_warnings": splits.warnings,
        "truncated_branches": truncated,
        "hierarchical_decision": decision,
        "interpretation": "Global WATCH caused only by size can be accepted when every linked branch passes. Global foreign_pull, conflicts, or any local VETO remain repair blockers."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_hgate_text(&out),
        OutputFormat::Md => print_hgate_md(&out),
    }
    Ok(
        match out["hierarchical_decision"]["action"]
            .as_str()
            .unwrap_or("REVIEW_REQUIRED")
        {
            "STRUCTURALLY_ACCEPTED" => EXIT_PASS,
            "REPAIR_REQUIRED" => EXIT_VETO,
            _ => EXIT_WATCH,
        },
    )
}

fn hierarchical_decision(
    global_report: &Report,
    global_map: &Value,
    branches: &[Value],
    truncated: usize,
) -> Value {
    let branch_count = branches.len();
    let local_pass = branches
        .iter()
        .filter(|branch| branch["verdict"].as_str() == Some("PASS"))
        .count();
    let local_watch = branches
        .iter()
        .filter(|branch| branch["verdict"].as_str() == Some("WATCH"))
        .count();
    let local_veto = branches
        .iter()
        .filter(|branch| branch["verdict"].as_str() == Some("VETO"))
        .count();
    let global_foreign_pull = global_map["foreign_pull"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let global_size_only = global_report.verdict == "WATCH"
        && !global_report.limits.is_empty()
        && global_report.conflicts.is_empty()
        && global_report.evidence_gaps.is_empty()
        && global_foreign_pull == 0;
    let all_local_pass = branch_count > 0 && local_pass == branch_count && truncated == 0;
    let (action, accepted, next) = if global_report.verdict == "VETO"
        || global_foreign_pull > 0
        || local_veto > 0
    {
        (
            "REPAIR_REQUIRED",
            false,
            "Repair global foreign pull, conflicts, or vetoed branches before accepting the structure.",
        )
    } else if all_local_pass && (global_size_only || global_report.verdict == "PASS") {
        (
            "STRUCTURALLY_ACCEPTED",
            true,
            "Use the local branch PASS results as the trusted acceptance surface.",
        )
    } else if local_watch > 0 || truncated > 0 || global_report.verdict == "WATCH" {
        (
            "SPLIT_REQUIRED",
            false,
            "Narrow unresolved WATCH branches or increase max branches before finalizing.",
        )
    } else {
        (
            "REVIEW_REQUIRED",
            false,
            "Review hierarchical gate output before trusting the structure.",
        )
    };
    json!({
        "action": action,
        "structurally_accepted": accepted,
        "global_verdict": global_report.verdict,
        "global_size_only": global_size_only,
        "branches": branch_count,
        "local_pass": local_pass,
        "local_watch": local_watch,
        "local_veto": local_veto,
        "global_foreign_pull": global_foreign_pull,
        "truncated_branches": truncated,
        "next": next
    })
}

fn comb_cmd(args: CombArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let triads = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let branch_by = parse_csv(&args.branch_by);
    let stop_on = parse_csv(&args.stop_on);
    let topology = topology(&triads, &candidates);
    let comb_tree = comb_node(
        "root",
        0,
        args.depth,
        &branch_by,
        &stop_on,
        args.max_branches,
        &packet,
        &triads,
        &candidates,
    )?;
    let summary = comb_summary(&comb_tree);
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "depth": args.depth,
        "branch_by": branch_by,
        "stop_on": stop_on,
        "topology": topology,
        "comb_tree": comb_tree,
        "summary": summary
    });
    if let Some(out_dir) = args.out_dir {
        fs::create_dir_all(&out_dir)?;
        fs::write(
            out_dir.join("comb.json"),
            serde_json::to_string_pretty(&out)? + "\n",
        )?;
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_comb_text(&out),
        OutputFormat::Md => print_comb_md(&out),
    }
    Ok(EXIT_PASS)
}

fn search_cmd(args: SearchArgs) -> Result<u8> {
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let memory = normalize_ids(packet.triads.clone(), "m");
    let query_packet = if let Some(query_file) = &args.query_file {
        load_packet_auto(
            query_file,
            &args.query_format,
            &args.task_id,
            &args.domain,
            &args.query,
            args.normalize_paths,
        )?
    } else {
        packet.clone()
    };
    let query_text = if !args.query.trim().is_empty() {
        args.query.clone()
    } else if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let focus = route_balanced_focus(&memory, &query, args.route_cap, args.route_triad_cap);
    let result = interference_search(
        &packet,
        &focus.memory,
        &query,
        args.top_k,
        &args.group_by,
        query_source,
        focus.metadata,
    );
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
        OutputFormat::Text => print_search_text(&result),
        OutputFormat::Md => print_search_md(&result),
    }
    Ok(EXIT_PASS)
}

fn probe_cmd(args: ProbeArgs) -> Result<u8> {
    if let Some(suite_path) = &args.suite {
        return probe_suite_cmd(&args, suite_path);
    }
    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow!("nanda probe requires an input path or --suite"))?;
    let out = run_probe_once(
        input,
        &args.input_format,
        &args.negative_inputs,
        &args.task_id,
        &args.domain,
        &args.query,
        args.query_file.as_ref(),
        &args.query_format,
        args.top_k,
        args.route_cap,
        args.route_triad_cap,
        &args.group_by,
        args.normalize_paths,
    )?;
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_probe_text(&out),
        OutputFormat::Md => print_probe_md(&out),
    }
    Ok(EXIT_PASS)
}

fn run_probe_once(
    input: &Path,
    input_format: &InputFormat,
    negative_inputs: &[PathBuf],
    task_id: &str,
    domain: &str,
    query_arg: &str,
    query_file: Option<&PathBuf>,
    query_format: &InputFormat,
    top_k: usize,
    route_cap: usize,
    route_triad_cap: usize,
    group_by: &PeakGroupBy,
    normalize_paths: bool,
) -> Result<Value> {
    let mut packet = load_packet_auto(
        input,
        input_format,
        task_id,
        domain,
        query_arg,
        normalize_paths,
    )?;
    let mut negative_shortcuts = if negative_inputs.is_empty() {
        packet.negative_shortcuts.clone()
    } else {
        vec![]
    };
    for negative_input in negative_inputs {
        if let Some(shortcuts) = load_feedback_negative_shortcuts(negative_input)? {
            negative_shortcuts.extend(shortcuts);
            continue;
        }
        let negative_packet = load_packet_auto(
            negative_input,
            input_format,
            task_id,
            domain,
            query_arg,
            normalize_paths,
        )?;
        negative_shortcuts.extend(negative_packet.negative_shortcuts);
    }
    negative_shortcuts = merge_negative_shortcuts(negative_shortcuts);

    let query_packet = if let Some(query_file) = query_file {
        load_packet_auto(
            query_file,
            query_format,
            task_id,
            domain,
            query_arg,
            normalize_paths,
        )?
    } else {
        packet.clone()
    };
    let query_text = if !query_arg.trim().is_empty() {
        query_arg.to_string()
    } else if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let memory = normalize_ids(packet.triads.clone(), "m");
    let focus = route_balanced_focus(&memory, &query, route_cap, route_triad_cap);

    let mut plain_packet = packet.clone();
    plain_packet.negative_shortcuts.clear();
    let plain = interference_search(
        &plain_packet,
        &focus.memory,
        &query,
        top_k,
        group_by,
        query_source,
        focus.metadata.clone(),
    );

    let mut negative_packet = packet;
    negative_packet.negative_shortcuts = negative_shortcuts.clone();
    let negative = interference_search(
        &negative_packet,
        &focus.memory,
        &query,
        top_k,
        group_by,
        query_source,
        focus.metadata,
    );

    Ok(probe_report(&plain, &negative, negative_shortcuts.len()))
}

fn probe_suite_cmd(args: &ProbeArgs, suite_path: &Path) -> Result<u8> {
    let text =
        fs::read_to_string(suite_path).with_context(|| format!("read {}", suite_path.display()))?;
    let suite: ProbeSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", suite_path.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow!("nanda probe --suite requires at least one case"));
    }
    let base = suite_path.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    for case in &suite.cases {
        let path = resolve_suite_path(base, &case.path);
        let negative_inputs = case
            .negative
            .iter()
            .map(|path| resolve_suite_path(base, path))
            .collect::<Vec<_>>();
        let group_by = match case.group_by.as_str() {
            "" => args.group_by.clone(),
            "group" => PeakGroupBy::Group,
            "route" => PeakGroupBy::Route,
            other => return Err(anyhow!("unsupported probe suite group_by: {other}")),
        };
        let result = run_probe_once(
            &path,
            &args.input_format,
            &negative_inputs,
            &args.task_id,
            &args.domain,
            &args.query,
            args.query_file.as_ref(),
            &args.query_format,
            args.top_k,
            args.route_cap,
            args.route_triad_cap,
            &group_by,
            args.normalize_paths,
        )?;
        let decision_ok = case.expected_decision.is_empty()
            || result["decision"].as_str() == Some(&case.expected_decision);
        let plain_ok = case.expected_plain_peak.is_empty()
            || result["plain"]["top_peak"].as_str() == Some(&case.expected_plain_peak);
        let negative_ok = case.expected_negative_peak.is_empty()
            || result["negative"]["top_peak"].as_str() == Some(&case.expected_negative_peak);
        let ok = decision_ok && plain_ok && negative_ok;
        if ok {
            passed += 1;
        }
        rows.push(json!({
            "id": if case.id.is_empty() { path.display().to_string() } else { case.id.clone() },
            "path": path.display().to_string(),
            "expected_decision": case.expected_decision,
            "actual_decision": result["decision"],
            "expected_plain_peak": case.expected_plain_peak,
            "actual_plain_peak": result["plain"]["top_peak"],
            "expected_negative_peak": case.expected_negative_peak,
            "actual_negative_peak": result["negative"]["top_peak"],
            "ok": ok,
            "delta": result["delta"],
            "recommended_action": result["recommended_action"]
        }));
    }
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "probe-suite",
        "name": suite.name,
        "passed": passed,
        "total": rows.len(),
        "accuracy": round4(passed as f64 / rows.len().max(1) as f64),
        "cases": rows
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_probe_suite_text(&out),
        OutputFormat::Md => print_probe_suite_md(&out),
    }
    if passed == out["total"].as_u64().unwrap_or(0) as usize {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

fn resolve_suite_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn dataset_doctor_cmd(args: DatasetDoctorArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let memory = normalize_ids(packet.triads.clone(), "m");
    let query = normalize_ids(packet.candidate_triads.clone(), "q");
    let out = corpus_diagnostics(&memory, &query, &packet.query, args.route_cap);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_dataset_doctor_text(&out),
        OutputFormat::Md => print_dataset_doctor_md(&out),
    }
    let verdict = out["verdict"].as_str().unwrap_or("WATCH");
    Ok(match verdict {
        "PASS" => EXIT_PASS,
        "VETO" => EXIT_VETO,
        "WATCH" => EXIT_WATCH,
        _ => EXIT_WATCH,
    })
}

fn budget_cmd(args: BudgetArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let out = nanda_6m_budget_report(&packet, &source, &candidates);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_budget_text(&out),
        OutputFormat::Md => print_budget_md(&out),
    }
    Ok(if out["fits_l3"].as_bool().unwrap_or(false) {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

fn pack6m_cmd(args: Pack6mArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let out = nanda_6m_pack_report(&packet, &source, &candidates, args.sample);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_pack6m_text(&out),
        OutputFormat::Md => print_pack6m_md(&out),
    }
    Ok(if out["packed_ok"].as_bool().unwrap_or(false) {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

fn nanda_6m_budget_report(packet: &Packet, source: &[Triad], candidates: &[Triad]) -> Value {
    let active_triads = source.len() + candidates.len();
    let active_lanes = packet.negative_shortcuts.len() + packet.positive_shortcuts.len();
    let mut entities = BTreeSet::<String>::new();
    let mut relations = BTreeSet::<String>::new();
    let mut routes = BTreeSet::<String>::new();
    let mut groups = BTreeSet::<String>::new();
    let mut evidence_refs = BTreeSet::<String>::new();
    let mut cold_labels = BTreeSet::<String>::new();

    for triad in source.iter().chain(candidates.iter()) {
        insert_label(&mut entities, &triad.subject);
        insert_label(&mut entities, &triad.object);
        insert_label(&mut relations, &triad.relation);
        insert_label(&mut evidence_refs, &triad.evidence);
        insert_label(&mut cold_labels, &triad.id);
        insert_label(&mut cold_labels, &triad.subject);
        insert_label(&mut cold_labels, &triad.relation);
        insert_label(&mut cold_labels, &triad.object);
        insert_label(&mut cold_labels, &triad.evidence);
        insert_label(&mut cold_labels, &triad.subject_role);
        insert_label(&mut cold_labels, &triad.object_role);

        let route = if triad.route.trim().is_empty() {
            "__route_default"
        } else {
            triad.route.trim()
        };
        let group = if triad.group.trim().is_empty() {
            "__group_default"
        } else {
            triad.group.trim()
        };
        insert_label(&mut routes, route);
        insert_label(&mut groups, group);
        insert_label(&mut cold_labels, route);
        insert_label(&mut cold_labels, group);
    }

    for lane in &packet.negative_shortcuts {
        insert_label(&mut cold_labels, &lane.id);
        insert_label(&mut cold_labels, &lane.suppress_peak);
        insert_label(&mut cold_labels, &lane.suppress_route);
        insert_label(&mut cold_labels, &lane.suppress_group);
        insert_label(&mut cold_labels, &lane.prefer_peak);
        insert_label(&mut cold_labels, &lane.prefer_route);
        insert_label(&mut cold_labels, &lane.prefer_group);
        insert_label(&mut cold_labels, &lane.reason);
        insert_label(&mut cold_labels, &lane.source_feedback);
        insert_label(&mut routes, &lane.suppress_route);
        insert_label(&mut routes, &lane.prefer_route);
        insert_label(&mut groups, &lane.suppress_group);
        insert_label(&mut groups, &lane.prefer_group);
        for term in lane.terms.iter().chain(lane.support_terms.iter()) {
            insert_label(&mut cold_labels, term);
        }
    }
    for lane in &packet.positive_shortcuts {
        insert_label(&mut cold_labels, &lane.id);
        insert_label(&mut cold_labels, &lane.reinforce_peak);
        insert_label(&mut cold_labels, &lane.reinforce_route);
        insert_label(&mut cold_labels, &lane.reinforce_group);
        insert_label(&mut cold_labels, &lane.reason);
        insert_label(&mut cold_labels, &lane.source_feedback);
        insert_label(&mut routes, &lane.reinforce_route);
        insert_label(&mut groups, &lane.reinforce_group);
        for term in lane.terms.iter().chain(lane.support_terms.iter()) {
            insert_label(&mut cold_labels, term);
        }
    }

    let centroid_count = routes.len() + groups.len();
    let budget_usage = nanda_6m::BudgetUsage {
        active_triads,
        centroids: centroid_count,
        lanes: active_lanes,
    };
    let estimated_hot_bytes = budget_usage.estimated_hot_bytes();
    let reserved_core_bytes = nanda_6m::RESERVED_CORE_BYTES;
    let cold_dictionary_bytes = cold_labels
        .iter()
        .map(|label| label.len() + 8)
        .sum::<usize>();
    let triads_ok = active_triads <= nanda_6m::TRIAD_CAPACITY;
    let centroids_ok = centroid_count <= nanda_6m::CENTROID_CAPACITY;
    let lanes_ok = active_lanes <= nanda_6m::LANE_CAPACITY;
    let hot_bytes_ok = estimated_hot_bytes <= nanda_6m::BUDGET_BYTES;
    let fits_l3 = budget_usage.fits();
    let mut blockers = vec![];
    if !triads_ok {
        blockers.push(json!({
            "state": "TOO_MANY_TRIADS",
            "count": active_triads,
            "capacity": nanda_6m::TRIAD_CAPACITY,
            "repair": "build a route-balanced focus packet or split by linked group"
        }));
    }
    if !centroids_ok {
        blockers.push(json!({
            "state": "TOO_MANY_CENTROIDS",
            "count": centroid_count,
            "capacity": nanda_6m::CENTROID_CAPACITY,
            "repair": "merge aliases, normalize routes/groups, or split topology"
        }));
    }
    if !lanes_ok {
        blockers.push(json!({
            "state": "TOO_MANY_LANES",
            "count": active_lanes,
            "capacity": nanda_6m::LANE_CAPACITY,
            "repair": "keep only active positive/negative lanes for this focus packet"
        }));
    }
    if !hot_bytes_ok {
        blockers.push(json!({
            "state": "SPILL_REQUIRED",
            "estimated_hot_bytes": estimated_hot_bytes,
            "budget_bytes": nanda_6m::BUDGET_BYTES,
            "repair": "reduce active triads, centroids, or lanes before hot execution"
        }));
    }
    let state = if fits_l3 {
        "FITS_L3"
    } else if !hot_bytes_ok {
        "SPILL_REQUIRED"
    } else if !centroids_ok {
        "SPLIT_REQUIRED"
    } else {
        "FOCUS_REQUIRED"
    };
    json!({
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "mode": "nanda-6m-budget-planner",
        "state": state,
        "verdict": if fits_l3 { "PASS" } else { "WATCH" },
        "fits_l3": fits_l3,
        "safe_for_hot_core": fits_l3,
        "hard_budget_bytes": nanda_6m::BUDGET_BYTES,
        "reserved_core_bytes": reserved_core_bytes,
        "estimated_hot_bytes": estimated_hot_bytes,
        "remaining_hot_bytes": nanda_6m::BUDGET_BYTES.saturating_sub(estimated_hot_bytes),
        "cold_dictionary_bytes": cold_dictionary_bytes,
        "cold_dictionary_note": "String labels, evidence text, JSON, and source snippets stay outside the NANDA-6M hot core.",
        "wave_dim": WAVE_DIM,
        "record_sizes": {
            "packed_triad_bytes": nanda_6m::TRIAD_BYTES,
            "centroid_bytes": nanda_6m::CENTROID_BYTES,
            "lane_bytes": nanda_6m::LANE_BYTES
        },
        "capacity": {
            "triads": nanda_6m::TRIAD_CAPACITY,
            "centroids": nanda_6m::CENTROID_CAPACITY,
            "lanes": nanda_6m::LANE_CAPACITY
        },
        "counts": {
            "source_triads": source.len(),
            "candidate_triads": candidates.len(),
            "active_triads": active_triads,
            "entities": entities.len(),
            "relations": relations.len(),
            "routes": routes.len(),
            "groups": groups.len(),
            "centroids": centroid_count,
            "evidence_refs": evidence_refs.len(),
            "negative_lanes": packet.negative_shortcuts.len(),
            "positive_lanes": packet.positive_shortcuts.len(),
            "active_lanes": active_lanes
        },
        "usage": {
            "triad_arena": usage_row(active_triads, nanda_6m::TRIAD_CAPACITY, active_triads * nanda_6m::TRIAD_BYTES, nanda_6m::TRIAD_ARENA_BYTES),
            "centroid_arena": usage_row(centroid_count, nanda_6m::CENTROID_CAPACITY, centroid_count * nanda_6m::CENTROID_BYTES, nanda_6m::CENTROID_ARENA_BYTES),
            "lane_arena": usage_row(active_lanes, nanda_6m::LANE_CAPACITY, active_lanes * nanda_6m::LANE_BYTES, nanda_6m::LANE_ARENA_BYTES),
            "workspace": usage_row(1, 1, nanda_6m::WORKSPACE_BYTES, nanda_6m::WORKSPACE_BYTES),
            "index_stats": usage_row(1, 1, nanda_6m::INDEX_STATS_BYTES, nanda_6m::INDEX_STATS_BYTES)
        },
        "blockers": blockers,
        "next": if fits_l3 {
            "Packet can enter the future NANDA-6M packed hot core."
        } else {
            "Do not run as one hot packet; focus, split, or reduce lanes before NANDA-6M execution."
        }
    })
}

fn nanda_6m_pack_report(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
    sample: usize,
) -> Value {
    let budget = nanda_6m_budget_report(packet, source, candidates);
    let mut entities = IdDictionary::default();
    let mut relations = IdDictionary::default();
    let mut routes = IdDictionary::default();
    let mut groups = IdDictionary::default();
    let mut evidences = IdDictionary::default();
    let mut roles = IdDictionary::default();
    let mut memory_packed = Vec::with_capacity(source.len());
    let mut query_packed = Vec::with_capacity(candidates.len());
    let mut blockers = vec![];

    for triad in source {
        match pack_triad6m(
            triad,
            0x0001,
            &mut entities,
            &mut relations,
            &mut routes,
            &mut groups,
            &mut evidences,
            &mut roles,
        ) {
            Ok(record) => memory_packed.push(record),
            Err(err) => blockers.push(err),
        }
    }
    for triad in candidates {
        match pack_triad6m(
            triad,
            0x0002,
            &mut entities,
            &mut relations,
            &mut routes,
            &mut groups,
            &mut evidences,
            &mut roles,
        ) {
            Ok(record) => query_packed.push(record),
            Err(err) => blockers.push(err),
        }
    }

    let query_records = if query_packed.is_empty() {
        &memory_packed
    } else {
        &query_packed
    };
    let projection_source = if query_packed.is_empty() {
        "memory_fallback"
    } else {
        "candidate_triads"
    };
    let packed_count = memory_packed.len() + query_packed.len();
    let sample_records = memory_packed
        .iter()
        .chain(query_packed.iter())
        .take(sample)
        .map(packed_triad_json)
        .collect::<Vec<_>>();
    let projection = nanda_6m::project_triads(query_records);
    let projection_summary = projection.summary();
    let route_centroids =
        packed_centroid_report(&memory_packed, &projection, CentroidAxis6m::Route, sample);
    let group_centroids =
        packed_centroid_report(&memory_packed, &projection, CentroidAxis6m::Group, sample);
    let route_peak = packed_peak_summary(&route_centroids);
    let group_peak = packed_peak_summary(&group_centroids);
    let packed_support = packed_support_report(
        &memory_packed,
        &projection,
        &route_peak,
        &group_peak,
        sample,
    );
    let packed_lane_keys = packed_lane_keys_report(&packed_support);
    let packed_lanes = packed_lane_preview(&packed_support, &packed_lane_keys);
    let packed_lane_store = packed_lane_store_report(&packed_lane_keys, &packed_lanes);
    let peak_decision = packed_field_decision(
        &route_peak,
        &group_peak,
        projection_summary.energy,
        memory_packed.len(),
        query_packed.len(),
    );
    let packed_lane_application =
        packed_lane_application_report(&packed_support, &packed_lanes, &peak_decision);
    let packed_lane_replay = packed_lane_replay_report(
        packet,
        source,
        candidates,
        &packed_lane_store,
        &routes,
        &groups,
    );
    let dictionary_ok = blockers.is_empty()
        && relations.len() <= u16::MAX as usize
        && routes.len() <= u16::MAX as usize
        && groups.len() <= u16::MAX as usize
        && roles.len() <= u8::MAX as usize;
    let packed_ok = budget["fits_l3"].as_bool().unwrap_or(false) && dictionary_ok;
    json!({
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "mode": "nanda-6m-pack-skeleton",
        "state": if packed_ok { "PACKED_FITS_L3" } else { "PACK_REVIEW_REQUIRED" },
        "verdict": if packed_ok { "PASS" } else { "WATCH" },
        "packed_ok": packed_ok,
        "budget": budget,
        "packed_records": {
            "count": packed_count,
            "memory_count": memory_packed.len(),
            "query_count": query_packed.len(),
            "bytes": packed_count * nanda_6m::TRIAD_BYTES,
            "record_bytes": nanda_6m::TRIAD_BYTES,
            "sample": sample_records
        },
        "projection": {
            "mode": "packed-triad-signed-hash",
            "source": projection_source,
            "records": query_records.len(),
            "wave_dim": nanda_6m::WAVE_DIM,
            "bytes": nanda_6m::QUERY_WAVE_BYTES,
            "summary": {
                "l1": projection_summary.l1,
                "energy": projection_summary.energy,
                "nonzero": projection_summary.nonzero,
                "max_abs": projection_summary.max_abs
            },
            "sample": projection.values.iter().take(8).copied().collect::<Vec<_>>()
        },
        "centroids": {
            "source": "memory_triads",
            "record_bytes": nanda_6m::CENTROID_BYTES,
            "route_count": route_centroids.len(),
            "group_count": group_centroids.len(),
            "total_count": route_centroids.len() + group_centroids.len(),
            "route": route_centroids,
            "group": group_centroids
        },
        "peaks": {
            "mode": "packed-candidate-query-vs-memory-centroid-cosine",
            "route": route_peak,
            "group": group_peak
        },
        "packed_support": packed_support,
        "packed_lane_keys": packed_lane_keys,
        "packed_lanes": packed_lanes,
        "packed_lane_store": packed_lane_store,
        "packed_lane_application": packed_lane_application,
        "packed_lane_replay": packed_lane_replay,
        "peak_decision": peak_decision,
        "dictionaries": {
            "entities": dictionary_summary(&entities, u32::MAX as usize),
            "relations": dictionary_summary(&relations, u16::MAX as usize),
            "routes": dictionary_summary(&routes, u16::MAX as usize),
            "groups": dictionary_summary(&groups, u16::MAX as usize),
            "evidences": dictionary_summary(&evidences, u32::MAX as usize),
            "roles": dictionary_summary(&roles, u8::MAX as usize)
        },
        "blockers": blockers,
        "hot_core_note": "This command still runs in the cold layer. It now separates memory/source centroids from the candidate/query projection wave, proving the first honest packed peak path before full packed interference search."
    })
}

#[derive(Clone, Copy)]
enum CentroidAxis6m {
    Route,
    Group,
}

fn packed_centroid_report(
    packed: &[nanda_6m::PackedTriad32],
    query: &nanda_6m::PackedWave1024,
    axis: CentroidAxis6m,
    sample: usize,
) -> Vec<Value> {
    let mut by_id: BTreeMap<u16, Vec<nanda_6m::PackedTriad32>> = BTreeMap::new();
    for triad in packed {
        let id = match axis {
            CentroidAxis6m::Route => triad.route_id,
            CentroidAxis6m::Group => triad.group_id,
        };
        by_id.entry(id).or_default().push(*triad);
    }
    let mut rows = by_id
        .iter()
        .map(|(id, triads)| {
            let centroid = nanda_6m::centroid_from_triads(triads);
            let summary = centroid.summary();
            let score = nanda_6m::score_centroid(query, &centroid);
            json!({
                "id": id,
                "triads": triads.len(),
                "score": {
                    "cosine": round4(score.cosine),
                    "dot": score.dot,
                    "query_energy": score.query_energy,
                    "centroid_energy": score.centroid_energy
                },
                "summary": {
                    "l1": summary.l1,
                    "energy": summary.energy,
                    "nonzero": summary.nonzero,
                    "max_abs": summary.max_abs
                },
                "sample": centroid.values.iter().take(8).copied().collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]["cosine"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"]["cosine"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(sample);
    rows
}

fn packed_peak_summary(rows: &[Value]) -> Value {
    let top = rows.first();
    let second = rows.get(1);
    let top_score = top
        .and_then(|item| item["score"]["cosine"].as_f64())
        .unwrap_or(0.0);
    let second_score = second
        .and_then(|item| item["score"]["cosine"].as_f64())
        .unwrap_or(0.0);
    let margin = top_score - second_score;
    let state = if top.is_none() || top_score <= 0.0 {
        "NO_PEAK"
    } else if top_score < 0.01 {
        "PEAK_THIN"
    } else if margin < 0.003 {
        "PEAK_CONTESTED"
    } else {
        "PEAK_FOUND"
    };
    json!({
        "top_id": top.and_then(|item| item["id"].as_u64()).unwrap_or(0),
        "top_score": round4(top_score),
        "second_id": second.and_then(|item| item["id"].as_u64()).unwrap_or(0),
        "second_score": round4(second_score),
        "margin": round4(margin),
        "state": state
    })
}

fn packed_field_decision(
    route_peak: &Value,
    group_peak: &Value,
    query_energy: u64,
    memory_count: usize,
    query_count: usize,
) -> Value {
    let route_state = route_peak["state"].as_str().unwrap_or("NO_PEAK");
    let group_state = group_peak["state"].as_str().unwrap_or("NO_PEAK");
    let route_id = route_peak["top_id"].as_u64().unwrap_or(0);
    let group_id = group_peak["top_id"].as_u64().unwrap_or(0);
    let route_score = route_peak["top_score"].as_f64().unwrap_or(0.0);
    let group_score = group_peak["top_score"].as_f64().unwrap_or(0.0);
    let route_margin = route_peak["margin"].as_f64().unwrap_or(0.0);
    let group_margin = group_peak["margin"].as_f64().unwrap_or(0.0);

    let (state, verdict, safe_to_answer, reason) = if memory_count == 0 {
        (
            "PACKED_EMPTY_MEMORY",
            "WATCH",
            false,
            "No memory/source triads were packed, so centroids cannot be trusted.",
        )
    } else if query_count == 0 {
        (
            "PACKED_MEMORY_FALLBACK",
            "WATCH",
            false,
            "No candidate/query triads were packed; projection uses memory fallback for diagnostics only.",
        )
    } else if query_energy == 0 {
        (
            "PACKED_EMPTY_QUERY",
            "WATCH",
            false,
            "Candidate/query projection has zero energy.",
        )
    } else if route_state == "NO_PEAK" || group_state == "NO_PEAK" {
        (
            "PACKED_NO_PEAK",
            "WATCH",
            false,
            "At least one centroid axis has no positive peak.",
        )
    } else if route_state == "PEAK_THIN" || group_state == "PEAK_THIN" {
        (
            "PACKED_THIN",
            "WATCH",
            false,
            "A peak exists, but cosine strength is below the packed focus threshold.",
        )
    } else if route_state == "PEAK_CONTESTED" || group_state == "PEAK_CONTESTED" {
        (
            "PACKED_CONTESTED",
            "WATCH",
            false,
            "Top centroid is too close to the runner-up.",
        )
    } else {
        (
            "PACKED_FOCUSED",
            "PASS",
            true,
            "Route and group axes both expose strong packed peaks.",
        )
    };

    json!({
        "state": state,
        "verdict": verdict,
        "safe_to_answer": safe_to_answer,
        "reason": reason,
        "thresholds": {
            "min_focus_score": 0.01,
            "min_focus_margin": 0.003
        },
        "query_energy": query_energy,
        "memory_records": memory_count,
        "query_records": query_count,
        "route": {
            "top_id": route_id,
            "state": route_state,
            "top_score": round4(route_score),
            "margin": round4(route_margin)
        },
        "group": {
            "top_id": group_id,
            "state": group_state,
            "top_score": round4(group_score),
            "margin": round4(group_margin)
        }
    })
}

fn packed_support_report(
    memory: &[nanda_6m::PackedTriad32],
    query: &nanda_6m::PackedWave1024,
    route_peak: &Value,
    group_peak: &Value,
    sample: usize,
) -> Value {
    json!({
        "mode": "query-vs-memory-triad-contributors",
        "route": packed_axis_support(
            memory,
            query,
            CentroidAxis6m::Route,
            route_peak["top_id"].as_u64().unwrap_or(0) as u16,
            sample
        ),
        "group": packed_axis_support(
            memory,
            query,
            CentroidAxis6m::Group,
            group_peak["top_id"].as_u64().unwrap_or(0) as u16,
            sample
        )
    })
}

fn packed_axis_support(
    memory: &[nanda_6m::PackedTriad32],
    query: &nanda_6m::PackedWave1024,
    axis: CentroidAxis6m,
    top_id: u16,
    sample: usize,
) -> Value {
    let mut rows = memory
        .iter()
        .enumerate()
        .filter_map(|(index, triad)| {
            let id = match axis {
                CentroidAxis6m::Route => triad.route_id,
                CentroidAxis6m::Group => triad.group_id,
            };
            if id != top_id || top_id == 0 {
                return None;
            }
            let centroid = nanda_6m::centroid_from_triads(std::slice::from_ref(triad));
            let score = nanda_6m::score_centroid(query, &centroid);
            Some(json!({
                "record_index": index,
                "route_id": triad.route_id,
                "group_id": triad.group_id,
                "relation_id": triad.relation_id,
                "subject_id": triad.subject_id,
                "object_id": triad.object_id,
                "evidence_ref": triad.evidence_ref,
                "dot": score.dot,
                "cosine": round4(score.cosine),
                "polarity": triad.polarity,
                "confidence": triad.confidence,
                "wave_seed": triad.wave_seed,
                "check": triad.check
            }))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["cosine"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["cosine"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let considered = rows.len();
    let mut positive_dot: i64 = 0;
    let mut negative_dot: i64 = 0;
    let mut support_count = 0usize;
    let mut anti_count = 0usize;
    for row in &rows {
        let dot = row["dot"].as_i64().unwrap_or(0);
        if dot > 0 {
            positive_dot += dot;
            support_count += 1;
        } else if dot < 0 {
            negative_dot += dot;
            anti_count += 1;
        }
    }

    let support = rows
        .iter()
        .filter(|row| row["dot"].as_i64().unwrap_or(0) > 0)
        .take(sample)
        .cloned()
        .collect::<Vec<_>>();
    let anti = rows
        .iter()
        .rev()
        .filter(|row| row["dot"].as_i64().unwrap_or(0) < 0)
        .take(sample)
        .cloned()
        .collect::<Vec<_>>();

    json!({
        "top_id": top_id,
        "considered": considered,
        "support_count": support_count,
        "anti_count": anti_count,
        "positive_dot": positive_dot,
        "negative_dot": negative_dot,
        "net_dot": positive_dot + negative_dot,
        "support": support,
        "anti": anti
    })
}

fn packed_lane_keys_report(packed_support: &Value) -> Value {
    json!({
        "mode": "stable-lane-keys",
        "storage": "cold-stable-signature",
        "hot_compilation": "record masks are rebuilt for the current focused packet",
        "lane_bytes": nanda_6m::LANE_BYTES,
        "route": packed_axis_lane_key(&packed_support["route"], "route"),
        "group": packed_axis_lane_key(&packed_support["group"], "group")
    })
}

fn packed_axis_lane_key(axis_support: &Value, axis: &str) -> Value {
    let top_id = axis_support["top_id"].as_u64().unwrap_or(0);
    let support_signature = packed_items_signature(&axis_support["support"]);
    let anti_signature = packed_items_signature(&axis_support["anti"]);
    let key_material = format!("{axis}|{top_id}|{support_signature}|{anti_signature}");
    let key_hash = stable_hash32(&key_material);
    json!({
        "axis": axis,
        "target_id": top_id,
        "key_hash": key_hash,
        "support_signature": support_signature,
        "anti_signature": anti_signature,
        "support_count": axis_support["support_count"].as_u64().unwrap_or(0),
        "anti_count": axis_support["anti_count"].as_u64().unwrap_or(0),
        "compile_hint": {
            "record_mask_a": packed_support_mask(&axis_support["anti"]).0,
            "record_mask_b": packed_support_mask(&axis_support["anti"]).1,
            "protected_support_mask_a": packed_support_mask(&axis_support["support"]).0,
            "protected_support_mask_b": packed_support_mask(&axis_support["support"]).1
        }
    })
}

fn packed_items_signature(items: &Value) -> String {
    let mut parts = items
        .as_array()
        .into_iter()
        .flat_map(|items| items.iter())
        .map(|item| {
            format!(
                "{}:{}:{}",
                item["wave_seed"].as_u64().unwrap_or(0),
                item["polarity"].as_u64().unwrap_or(0),
                item["confidence"].as_u64().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts.join("|")
}

fn packed_lane_preview(packed_support: &Value, packed_lane_keys: &Value) -> Value {
    json!({
        "mode": "packed-lane-preview",
        "lane_bytes": nanda_6m::LANE_BYTES,
        "route": packed_axis_lane_preview(&packed_support["route"], &packed_lane_keys["route"]),
        "group": packed_axis_lane_preview(&packed_support["group"], &packed_lane_keys["group"])
    })
}

fn packed_lane_store_report(packed_lane_keys: &Value, packed_lanes: &Value) -> Value {
    let mut lanes = vec![];
    if let Some(lane) =
        packed_lane_store_item(&packed_lane_keys["route"], &packed_lanes["route"], "route")
    {
        lanes.push(lane);
    }
    if let Some(lane) =
        packed_lane_store_item(&packed_lane_keys["group"], &packed_lanes["group"], "group")
    {
        lanes.push(lane);
    }
    let count = lanes.len();
    json!({
        "mode": "packed-lane-store",
        "storage": "hot-compiled-lane-arena",
        "source": "cold-stable-lane-keys",
        "capacity": nanda_6m::LANE_CAPACITY,
        "count": count,
        "bytes": count * nanda_6m::LANE_BYTES,
        "arena_bytes": nanda_6m::LANE_ARENA_BYTES,
        "record_bytes": nanda_6m::LANE_BYTES,
        "replay_ready": count > 0,
        "sample": lanes
    })
}

fn packed_lane_store_item(axis_key: &Value, axis_lane: &Value, axis: &str) -> Option<Value> {
    if axis_lane["state"].as_str() != Some("LANE_PREVIEW_READY") {
        return None;
    }
    Some(json!({
        "axis": axis,
        "key_hash": axis_key["key_hash"].as_u64().unwrap_or(0),
        "target_id": axis_key["target_id"].as_u64().unwrap_or(0),
        "action": axis_lane["action"].as_str().unwrap_or("none"),
        "record_mask_a": axis_lane["record_mask_a"].as_u64().unwrap_or(0),
        "record_mask_b": axis_lane["record_mask_b"].as_u64().unwrap_or(0),
        "protected_support_mask_a": axis_lane["protected_support_mask_a"].as_u64().unwrap_or(0),
        "protected_support_mask_b": axis_lane["protected_support_mask_b"].as_u64().unwrap_or(0),
        "strength": axis_lane["strength"].as_u64().unwrap_or(0),
        "before_net_dot": axis_lane["before_net_dot"].as_i64().unwrap_or(0),
        "after_net_dot": axis_lane["after_net_dot"].as_i64().unwrap_or(0),
        "delta_dot": axis_lane["delta_dot"].as_i64().unwrap_or(0),
        "key_storage": axis_lane["key_storage"].as_str().unwrap_or("cold-stable-signature"),
        "compiled_storage": axis_lane["compiled_storage"].as_str().unwrap_or("hot-packed-lane64")
    }))
}

fn packed_axis_lane_preview(axis_support: &Value, axis_key: &Value) -> Value {
    let top_id = axis_support["top_id"].as_u64().unwrap_or(0) as u16;
    let positive_dot = axis_support["positive_dot"].as_i64().unwrap_or(0);
    let negative_dot = axis_support["negative_dot"].as_i64().unwrap_or(0);
    let before_net = axis_support["net_dot"].as_i64().unwrap_or(0);
    let anti_mask = packed_support_mask(&axis_support["anti"]);
    let support_mask = packed_support_mask(&axis_support["support"]);
    let has_lane = negative_dot < 0 && (anti_mask.0 != 0 || anti_mask.1 != 0);
    let lane = nanda_6m::PackedLane64 {
        support_mask_a: anti_mask.0,
        support_mask_b: anti_mask.1,
        anti_mask_a: support_mask.0,
        anti_mask_b: support_mask.1,
        lane_id: axis_key["key_hash"].as_u64().unwrap_or(u64::from(top_id)) as u32,
        target_route: top_id,
        target_group: top_id,
        target_relation: 0,
        accepted_count: 0,
        rejected_count: if has_lane { 1 } else { 0 },
        margin_hint: before_net.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16,
        action: if has_lane { 1 } else { 0 },
        strength: if has_lane { 255 } else { 0 },
        reserved: [0; 14],
    };
    let after_net = if has_lane { positive_dot } else { before_net };
    json!({
        "state": if has_lane { "LANE_PREVIEW_READY" } else { "NO_ANTI_LANE" },
        "action": if has_lane { "suppress_anti_support" } else { "none" },
        "key_hash": lane.lane_id,
        "key_storage": "cold-stable-signature",
        "compiled_storage": "hot-packed-lane64",
        "target_id": top_id,
        "record_mask_a": lane.support_mask_a,
        "record_mask_b": lane.support_mask_b,
        "protected_support_mask_a": lane.anti_mask_a,
        "protected_support_mask_b": lane.anti_mask_b,
        "strength": lane.strength,
        "before_net_dot": before_net,
        "suppressed_negative_dot": if has_lane { negative_dot } else { 0 },
        "after_net_dot": after_net,
        "delta_dot": after_net - before_net,
        "interpretation": if has_lane {
            "Preview only: suppressing the current anti-support records would remove the destructive contribution without changing positive support."
        } else {
            "No negative contribution was found for this packed axis."
        }
    })
}

fn packed_support_mask(items: &Value) -> (u64, u64) {
    let mut mask_a = 0u64;
    let mut mask_b = 0u64;
    if let Some(items) = items.as_array() {
        for item in items {
            let Some(index) = item["record_index"].as_u64() else {
                continue;
            };
            if index < 64 {
                mask_a |= 1u64 << index;
            } else if index < 128 {
                mask_b |= 1u64 << (index - 64);
            }
        }
    }
    (mask_a, mask_b)
}

fn packed_lane_application_report(
    packed_support: &Value,
    packed_lanes: &Value,
    raw_decision: &Value,
) -> Value {
    let route =
        packed_axis_lane_application(&packed_support["route"], &packed_lanes["route"], "route");
    let group =
        packed_axis_lane_application(&packed_support["group"], &packed_lanes["group"], "group");
    let route_state = route["state"].as_str().unwrap_or("LANE_NO_EFFECT");
    let group_state = group["state"].as_str().unwrap_or("LANE_NO_EFFECT");
    let improved = route["improved"].as_bool().unwrap_or(false)
        || group["improved"].as_bool().unwrap_or(false);
    let focused_candidate = route_state == "LANE_AXIS_FOCUSED_CANDIDATE"
        && group_state == "LANE_AXIS_FOCUSED_CANDIDATE";
    let state = if focused_candidate {
        "PACKED_LANE_FOCUSED_CANDIDATE"
    } else if improved {
        "PACKED_LANE_IMPROVED"
    } else {
        "PACKED_LANE_NO_EFFECT"
    };

    json!({
        "mode": "single-pass-suppress-anti-support",
        "state": state,
        "verdict": if focused_candidate { "WATCH" } else { raw_decision["verdict"].as_str().unwrap_or("WATCH") },
        "safe_to_answer": false,
        "ready_for_hot_loop": focused_candidate,
        "raw_state": raw_decision["state"].as_str().unwrap_or("PACKED_REVIEW_REQUIRED"),
        "reason": if focused_candidate {
            "Single-pass lane application turns the support-map net into a focused candidate, but persistent learned lanes are still required before answering from the hot core."
        } else if improved {
            "Single-pass lane application improves the support-map net, but the adjusted field is not focused enough."
        } else {
            "No useful packed lane application was available."
        },
        "thresholds": {
            "min_focused_net_dot": 128,
            "min_delta_dot": 64
        },
        "route": route,
        "group": group
    })
}

fn packed_axis_lane_application(axis_support: &Value, axis_lane: &Value, axis: &str) -> Value {
    let before_net = axis_lane["before_net_dot"]
        .as_i64()
        .unwrap_or_else(|| axis_support["net_dot"].as_i64().unwrap_or(0));
    let after_net = axis_lane["after_net_dot"].as_i64().unwrap_or(before_net);
    let delta = axis_lane["delta_dot"]
        .as_i64()
        .unwrap_or(after_net - before_net);
    let lane_ready = axis_lane["state"].as_str() == Some("LANE_PREVIEW_READY");
    let focused = lane_ready && after_net >= 128 && delta >= 64;
    let improved = lane_ready && delta > 0 && after_net > before_net;
    let state = if focused {
        "LANE_AXIS_FOCUSED_CANDIDATE"
    } else if improved {
        "LANE_AXIS_IMPROVED"
    } else {
        "LANE_NO_EFFECT"
    };
    json!({
        "axis": axis,
        "state": state,
        "improved": improved,
        "before_net_dot": before_net,
        "after_net_dot": after_net,
        "delta_dot": delta,
        "suppressed_negative_dot": axis_lane["suppressed_negative_dot"].as_i64().unwrap_or(0),
        "support_count": axis_support["support_count"].as_u64().unwrap_or(0),
        "anti_count": axis_support["anti_count"].as_u64().unwrap_or(0),
        "record_mask_a": axis_lane["record_mask_a"].as_u64().unwrap_or(0),
        "record_mask_b": axis_lane["record_mask_b"].as_u64().unwrap_or(0),
        "protected_support_mask_a": axis_lane["protected_support_mask_a"].as_u64().unwrap_or(0),
        "protected_support_mask_b": axis_lane["protected_support_mask_b"].as_u64().unwrap_or(0)
    })
}

fn packed_lane_replay_report(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
    packed_lane_store: &Value,
    routes: &IdDictionary,
    groups: &IdDictionary,
) -> Value {
    let query_tokens = packed_replay_tokens(packet, source, candidates);
    let mut replayed = vec![];
    if let Some(items) = packed_lane_store["sample"].as_array() {
        for item in items {
            let axis = item["axis"].as_str().unwrap_or("");
            let target_id = item["target_id"].as_u64().unwrap_or(0) as u32;
            let target_label = match axis {
                "route" => routes.label(target_id).unwrap_or("__route_default"),
                "group" => groups.label(target_id).unwrap_or("__group_default"),
                _ => "__unknown",
            };
            for shortcut in &packet.negative_shortcuts {
                let query_ratio = negative_lane_match_ratio(&query_tokens, shortcut);
                if query_ratio <= 0.0 || !negative_shortcut_matches_target(shortcut, target_label) {
                    continue;
                }
                replayed.push(packed_lane_replay_item(
                    item,
                    axis,
                    target_label,
                    "negative_shortcuts",
                    &shortcut.id,
                    query_ratio,
                ));
            }
            for shortcut in &packet.positive_shortcuts {
                let query_ratio = positive_lane_match_ratio(&query_tokens, shortcut);
                if query_ratio <= 0.0 || !positive_shortcut_matches_target(shortcut, target_label) {
                    continue;
                }
                replayed.push(packed_lane_replay_item(
                    item,
                    axis,
                    target_label,
                    "positive_shortcuts",
                    &shortcut.id,
                    query_ratio,
                ));
            }
        }
    }

    let compiled_lanes = replayed.len();
    let before_net_dot: i64 = replayed
        .iter()
        .map(|item| item["before_net_dot"].as_i64().unwrap_or(0))
        .sum();
    let after_net_dot: i64 = replayed
        .iter()
        .map(|item| item["after_net_dot"].as_i64().unwrap_or(0))
        .sum();
    let delta_dot = after_net_dot - before_net_dot;
    let focused = compiled_lanes > 0 && after_net_dot >= 128 && delta_dot >= 64;
    let stability_sweep = packed_lane_replay_stability_sweep(before_net_dot, delta_dot);
    let stability_state = packed_lane_replay_stability_state(&stability_sweep, compiled_lanes);
    json!({
        "mode": "feedback-lane-replay",
        "source": "negative_shortcuts+positive_shortcuts",
        "touch_policy": {
            "mode": "observer-to-compute-sweep",
            "default_strength": 0.0,
            "soft_strength": 0.25,
            "full_strength": 1.0,
            "safe_to_answer_grant": false,
            "interpretation": "Replay is measured at observer/soft/medium/full strengths. It may shape the packed field, but it never grants final answer permission by itself."
        },
        "state": if focused { "PACKED_LANE_REPLAY_FOCUSED" } else if compiled_lanes > 0 { "PACKED_LANE_REPLAY_PARTIAL" } else { "PACKED_LANE_REPLAY_NONE" },
        "stability_state": stability_state,
        "safe_to_answer": false,
        "matched_keys": compiled_lanes,
        "compiled_lanes": compiled_lanes,
        "before_net_dot": before_net_dot,
        "after_net_dot": after_net_dot,
        "delta_dot": delta_dot,
        "replay_ready": focused,
        "computational_effect": {
            "state": if focused { "REPLAY_COMPUTE_READY" } else if compiled_lanes > 0 { "REPLAY_COMPUTE_WEAK" } else { "REPLAY_COMPUTE_NONE" },
            "applied_strength": if focused { 1.0 } else { 0.0 },
            "field_before": before_net_dot,
            "field_after": if focused { after_net_dot } else { before_net_dot },
            "delta_dot": if focused { delta_dot } else { 0 },
            "safe_to_answer": false,
            "reason": if focused {
                "Matched feedback lanes can be applied as a packed compute pass, but the structural gate must still decide trust."
            } else if compiled_lanes > 0 {
                "Matched feedback lanes exist, but the replay field is not focused enough for compute application."
            } else {
                "No feedback lane matched the current packed lane keys."
            }
        },
        "stability_sweep": stability_sweep,
        "sample": replayed
    })
}

fn packed_lane_replay_stability_sweep(before_net_dot: i64, delta_dot: i64) -> Vec<Value> {
    [
        ("observer", 0u32),
        ("soft_touch", 250u32),
        ("medium_touch", 500u32),
        ("full_touch", 1000u32),
    ]
    .into_iter()
    .map(|(label, permille)| {
        let applied_delta = delta_dot * i64::from(permille) / 1000;
        let after = before_net_dot + applied_delta;
        json!({
            "label": label,
            "strength": round4(f64::from(permille) / 1000.0),
            "before_net_dot": before_net_dot,
            "after_net_dot": after,
            "delta_dot": applied_delta,
            "field_state": packed_lane_replay_field_state(after, applied_delta)
        })
    })
    .collect()
}

fn packed_lane_replay_stability_state(sweep: &[Value], compiled_lanes: usize) -> &'static str {
    if compiled_lanes == 0 {
        return "NO_REPLAY_FIELD";
    }
    let soft = sweep
        .iter()
        .find(|item| item["label"].as_str() == Some("soft_touch"));
    let full = sweep
        .iter()
        .find(|item| item["label"].as_str() == Some("full_touch"));
    let soft_after = soft
        .and_then(|item| item["after_net_dot"].as_i64())
        .unwrap_or(0);
    let soft_delta = soft
        .and_then(|item| item["delta_dot"].as_i64())
        .unwrap_or(0);
    let full_after = full
        .and_then(|item| item["after_net_dot"].as_i64())
        .unwrap_or(0);
    let full_delta = full
        .and_then(|item| item["delta_dot"].as_i64())
        .unwrap_or(0);
    if soft_after >= 128 && soft_delta >= 64 {
        "STABLE_UNDER_SOFT_TOUCH"
    } else if full_after >= 128 && full_delta >= 64 {
        "FULL_TOUCH_REQUIRED"
    } else if full_delta > 0 {
        "WEAK_CONSTRUCTIVE_REPLAY"
    } else if full_delta < 0 {
        "DESTABILIZING_REPLAY"
    } else {
        "NO_REPLAY_SHIFT"
    }
}

fn packed_lane_replay_field_state(after_net_dot: i64, delta_dot: i64) -> &'static str {
    if after_net_dot >= 128 && delta_dot >= 64 {
        "FIELD_FOCUSED_BY_REPLAY"
    } else if delta_dot > 0 {
        "FIELD_IMPROVED_BY_REPLAY"
    } else if delta_dot < 0 {
        "FIELD_WEAKENED_BY_REPLAY"
    } else {
        "FIELD_OBSERVED"
    }
}

fn packed_lane_replay_item(
    item: &Value,
    axis: &str,
    target_label: &str,
    source: &str,
    shortcut_id: &str,
    query_ratio: f64,
) -> Value {
    json!({
        "axis": axis,
        "target_label": target_label,
        "source": source,
        "shortcut_id": shortcut_id,
        "query_match_ratio": round4(query_ratio),
        "key_hash": item["key_hash"].as_u64().unwrap_or(0),
        "record_mask_a": item["record_mask_a"].as_u64().unwrap_or(0),
        "record_mask_b": item["record_mask_b"].as_u64().unwrap_or(0),
        "protected_support_mask_a": item["protected_support_mask_a"].as_u64().unwrap_or(0),
        "protected_support_mask_b": item["protected_support_mask_b"].as_u64().unwrap_or(0),
        "before_net_dot": item["before_net_dot"].as_i64().unwrap_or(0),
        "after_net_dot": item["after_net_dot"].as_i64().unwrap_or(0),
        "delta_dot": item["delta_dot"].as_i64().unwrap_or(0),
        "compiled_storage": item["compiled_storage"].as_str().unwrap_or("hot-packed-lane64")
    })
}

fn packed_replay_tokens(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    tokens.extend(normalized_shortcut_terms(std::slice::from_ref(
        &packet.query,
    )));
    for triad in source.iter().chain(candidates.iter()) {
        for value in [
            &triad.subject,
            &triad.relation,
            &triad.object,
            &triad.evidence,
            &triad.route,
            &triad.group,
            &triad.subject_role,
            &triad.object_role,
        ] {
            tokens.extend(normalized_shortcut_terms(std::slice::from_ref(value)));
        }
    }
    tokens
}

fn negative_shortcut_matches_target(shortcut: &NegativeShortcut, target_label: &str) -> bool {
    shortcut_label_matches(
        target_label,
        [
            shortcut.prefer_peak.as_str(),
            shortcut.prefer_route.as_str(),
            shortcut.prefer_group.as_str(),
            shortcut.suppress_peak.as_str(),
            shortcut.suppress_route.as_str(),
            shortcut.suppress_group.as_str(),
        ],
    )
}

fn positive_shortcut_matches_target(shortcut: &PositiveShortcut, target_label: &str) -> bool {
    shortcut_label_matches(
        target_label,
        [
            shortcut.reinforce_peak.as_str(),
            shortcut.reinforce_route.as_str(),
            shortcut.reinforce_group.as_str(),
            "",
            "",
            "",
        ],
    )
}

fn shortcut_label_matches<'a>(
    target_label: &str,
    labels: impl IntoIterator<Item = &'a str>,
) -> bool {
    let target = norm(target_label);
    if target.is_empty() {
        return false;
    }
    labels.into_iter().any(|label| {
        let label = norm(label);
        !label.is_empty() && (target == label || target.contains(&label) || label.contains(&target))
    })
}

#[derive(Default)]
struct IdDictionary {
    items: BTreeMap<String, u32>,
}

impl IdDictionary {
    fn id(&mut self, value: &str) -> u32 {
        let key = if value.trim().is_empty() {
            "__default"
        } else {
            value.trim()
        };
        if let Some(id) = self.items.get(key) {
            return *id;
        }
        let id = self.items.len() as u32 + 1;
        self.items.insert(key.to_string(), id);
        id
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn label(&self, id: u32) -> Option<&str> {
        self.items
            .iter()
            .find_map(|(label, item_id)| (*item_id == id).then_some(label.as_str()))
    }
}

fn pack_triad6m(
    triad: &Triad,
    flags: u16,
    entities: &mut IdDictionary,
    relations: &mut IdDictionary,
    routes: &mut IdDictionary,
    groups: &mut IdDictionary,
    evidences: &mut IdDictionary,
    roles: &mut IdDictionary,
) -> std::result::Result<nanda_6m::PackedTriad32, Value> {
    let relation_id = checked_u16(relations.id(&triad.relation), "relation", &triad.relation)?;
    let route_id = checked_u16(
        routes.id(defaulted(&triad.route, "__route_default")),
        "route",
        &triad.route,
    )?;
    let group_id = checked_u16(
        groups.id(defaulted(&triad.group, "__group_default")),
        "group",
        &triad.group,
    )?;
    let subject_role = checked_u8(
        roles.id(defaulted(&triad.subject_role, "subject")),
        "subject_role",
        &triad.subject_role,
    )?;
    let object_role = checked_u8(
        roles.id(defaulted(&triad.object_role, "object")),
        "object_role",
        &triad.object_role,
    )?;
    let role_pack = u16::from(subject_role) | (u16::from(object_role) << 8);
    let subject_id = entities.id(&triad.subject);
    let object_id = entities.id(&triad.object);
    let evidence_ref = evidences.id(&triad.evidence);
    let wave_seed = stable_hash32(&format!(
        "{}|{}|{}|{}|{}|{}",
        triad.subject,
        triad.relation,
        triad.object,
        triad.route,
        triad.group,
        triad_polarity(triad)
    ));
    let confidence = (triad.confidence.clamp(0.0, 1.0) * 255.0).round() as u8;
    let polarity = stable_hash8(&triad_polarity(triad));
    let check = stable_hash16(&format!(
        "{subject_id}|{object_id}|{evidence_ref}|{wave_seed}|{relation_id}|{route_id}|{group_id}|{role_pack}|{flags}|{confidence}|{polarity}"
    ));
    Ok(nanda_6m::PackedTriad32::new(nanda_6m::PackedTriadInput {
        subject_id,
        object_id,
        evidence_ref,
        wave_seed,
        relation_id,
        route_id,
        group_id,
        role_pack,
        flags,
        lane_hint: 0,
        check,
        confidence,
        polarity,
    }))
}

fn defaulted<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn checked_u16(value: u32, field: &str, label: &str) -> std::result::Result<u16, Value> {
    u16::try_from(value).map_err(|_| {
        json!({
            "state": "PACK_FIELD_OVERFLOW",
            "field": field,
            "label": label,
            "id": value,
            "capacity": u16::MAX
        })
    })
}

fn checked_u8(value: u32, field: &str, label: &str) -> std::result::Result<u8, Value> {
    u8::try_from(value).map_err(|_| {
        json!({
            "state": "PACK_FIELD_OVERFLOW",
            "field": field,
            "label": label,
            "id": value,
            "capacity": u8::MAX
        })
    })
}

fn stable_hash32(value: &str) -> u32 {
    let digest = Sha256::digest(value.as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

fn stable_hash16(value: &str) -> u16 {
    let digest = Sha256::digest(value.as_bytes());
    u16::from_le_bytes([digest[0], digest[1]])
}

fn stable_hash8(value: &str) -> u8 {
    Sha256::digest(value.as_bytes())[0]
}

fn dictionary_summary(dictionary: &IdDictionary, capacity: usize) -> Value {
    json!({
        "count": dictionary.len(),
        "capacity": capacity,
        "fits": dictionary.len() <= capacity,
        "sample": dictionary
            .items
            .iter()
            .take(8)
            .map(|(label, id)| json!({ "id": id, "label": label }))
            .collect::<Vec<_>>()
    })
}

fn packed_triad_json(record: &nanda_6m::PackedTriad32) -> Value {
    json!({
        "subject_id": record.subject_id,
        "object_id": record.object_id,
        "evidence_ref": record.evidence_ref,
        "wave_seed": record.wave_seed,
        "relation_id": record.relation_id,
        "route_id": record.route_id,
        "group_id": record.group_id,
        "role_pack": record.role_pack,
        "flags": record.flags,
        "lane_hint": record.lane_hint,
        "check": record.check,
        "confidence": record.confidence,
        "polarity": record.polarity
    })
}

fn print_pack6m_text(out: &Value) {
    println!("NANDA-6M PACK");
    println!(
        "state: {}",
        out["state"].as_str().unwrap_or("PACK_REVIEW_REQUIRED")
    );
    println!("packed_ok: {}", out["packed_ok"].as_bool().unwrap_or(false));
    println!(
        "records: {} / memory {} / query {} / bytes {}",
        out["packed_records"]["count"].as_u64().unwrap_or(0),
        out["packed_records"]["memory_count"].as_u64().unwrap_or(0),
        out["packed_records"]["query_count"].as_u64().unwrap_or(0),
        out["packed_records"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "projection: {} / records {} / dim {} / energy {}",
        out["projection"]["source"].as_str().unwrap_or("unknown"),
        out["projection"]["records"].as_u64().unwrap_or(0),
        out["projection"]["wave_dim"].as_u64().unwrap_or(0),
        out["projection"]["summary"]["energy"].as_u64().unwrap_or(0)
    );
    println!(
        "centroids: route {} / group {}",
        out["centroids"]["route_count"].as_u64().unwrap_or(0),
        out["centroids"]["group_count"].as_u64().unwrap_or(0)
    );
    println!(
        "peaks: route {} score {} / group {} score {}",
        out["peaks"]["route"]["top_id"].as_u64().unwrap_or(0),
        out["peaks"]["route"]["top_score"].as_f64().unwrap_or(0.0),
        out["peaks"]["group"]["top_id"].as_u64().unwrap_or(0),
        out["peaks"]["group"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "decision: {} / safe_to_answer {}",
        out["peak_decision"]["state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED"),
        out["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "support: route +{} / -{} / net {}",
        out["packed_support"]["route"]["support_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["anti_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "lane preview: route net {} -> {}",
        out["packed_lanes"]["route"]["before_net_dot"]
            .as_i64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["after_net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "lane key: route {} / hot mask {}",
        out["packed_lane_keys"]["route"]["key_hash"]
            .as_u64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["record_mask_a"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "lane store: {} / {} / bytes {}",
        out["packed_lane_store"]["count"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["capacity"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "lane replay: {} / matched {}",
        out["packed_lane_replay"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_REPLAY_NONE"),
        out["packed_lane_replay"]["matched_keys"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "replay stability: {} / compute {}",
        out["packed_lane_replay"]["stability_state"]
            .as_str()
            .unwrap_or("NO_REPLAY_FIELD"),
        out["packed_lane_replay"]["computational_effect"]["state"]
            .as_str()
            .unwrap_or("REPLAY_COMPUTE_NONE")
    );
    println!(
        "lane applied: {} -> {}",
        out["packed_lane_application"]["raw_state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED"),
        out["packed_lane_application"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_NO_EFFECT")
    );
    println!(
        "budget: {} / {}",
        out["budget"]["estimated_hot_bytes"].as_u64().unwrap_or(0),
        out["budget"]["hard_budget_bytes"].as_u64().unwrap_or(0)
    );
}

fn print_pack6m_md(out: &Value) {
    println!("# NANDA-6M Pack\n");
    println!(
        "- state: `{}`",
        out["state"].as_str().unwrap_or("PACK_REVIEW_REQUIRED")
    );
    println!("- packed_ok: `{}`", out["packed_ok"]);
    println!(
        "- records: `{}`",
        out["packed_records"]["count"].as_u64().unwrap_or(0)
    );
    println!(
        "- memory_records: `{}`",
        out["packed_records"]["memory_count"].as_u64().unwrap_or(0)
    );
    println!(
        "- query_records: `{}`",
        out["packed_records"]["query_count"].as_u64().unwrap_or(0)
    );
    println!(
        "- bytes: `{}`",
        out["packed_records"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "- projection_source: `{}`",
        out["projection"]["source"].as_str().unwrap_or("unknown")
    );
    println!(
        "- projection_energy: `{}`",
        out["projection"]["summary"]["energy"].as_u64().unwrap_or(0)
    );
    println!(
        "- centroids: route `{}` / group `{}`",
        out["centroids"]["route_count"].as_u64().unwrap_or(0),
        out["centroids"]["group_count"].as_u64().unwrap_or(0)
    );
    println!(
        "- route_peak: `{}` / score `{}`",
        out["peaks"]["route"]["top_id"].as_u64().unwrap_or(0),
        out["peaks"]["route"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "- peak_decision: `{}`",
        out["peak_decision"]["state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED")
    );
    println!(
        "- safe_to_answer: `{}`",
        out["peak_decision"]["safe_to_answer"]
    );
    println!(
        "- route_support: `+{} / -{} / net {}`",
        out["packed_support"]["route"]["support_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["anti_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "- lane_preview: `route net {} -> {}`",
        out["packed_lanes"]["route"]["before_net_dot"]
            .as_i64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["after_net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "- lane_key: `route {} / hot mask {}`",
        out["packed_lane_keys"]["route"]["key_hash"]
            .as_u64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["record_mask_a"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "- lane_store: `{}/{}` / bytes `{}`",
        out["packed_lane_store"]["count"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["capacity"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "- lane_replay: `{}` / matched `{}`",
        out["packed_lane_replay"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_REPLAY_NONE"),
        out["packed_lane_replay"]["matched_keys"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "- replay_stability: `{}` / compute `{}`",
        out["packed_lane_replay"]["stability_state"]
            .as_str()
            .unwrap_or("NO_REPLAY_FIELD"),
        out["packed_lane_replay"]["computational_effect"]["state"]
            .as_str()
            .unwrap_or("REPLAY_COMPUTE_NONE")
    );
    println!(
        "- lane_applied: `{} -> {}`",
        out["packed_lane_application"]["raw_state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED"),
        out["packed_lane_application"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_NO_EFFECT")
    );
    println!(
        "- budget: `{}/{}`",
        out["budget"]["estimated_hot_bytes"], out["budget"]["hard_budget_bytes"]
    );
}

fn insert_label(set: &mut BTreeSet<String>, value: &str) {
    let trimmed = value.trim();
    if !trimmed.is_empty() {
        set.insert(trimmed.to_string());
    }
}

fn usage_row(count: usize, capacity: usize, bytes: usize, arena_bytes: usize) -> Value {
    json!({
        "count": count,
        "capacity": capacity,
        "bytes": bytes,
        "arena_bytes": arena_bytes,
        "percent": round4((count as f64 / capacity.max(1) as f64) * 100.0)
    })
}

fn print_budget_text(out: &Value) {
    println!("NANDA-6M BUDGET");
    println!(
        "state: {}",
        out["state"].as_str().unwrap_or("FOCUS_REQUIRED")
    );
    println!("fits_l3: {}", out["fits_l3"].as_bool().unwrap_or(false));
    println!(
        "estimated_hot_bytes: {}/{}",
        out["estimated_hot_bytes"].as_u64().unwrap_or(0),
        out["hard_budget_bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "triads: {}/{}",
        out["counts"]["active_triads"].as_u64().unwrap_or(0),
        out["capacity"]["triads"].as_u64().unwrap_or(0)
    );
    println!(
        "centroids: {}/{}",
        out["counts"]["centroids"].as_u64().unwrap_or(0),
        out["capacity"]["centroids"].as_u64().unwrap_or(0)
    );
    println!(
        "lanes: {}/{}",
        out["counts"]["active_lanes"].as_u64().unwrap_or(0),
        out["capacity"]["lanes"].as_u64().unwrap_or(0)
    );
    println!("next: {}", out["next"].as_str().unwrap_or(""));
}

fn print_budget_md(out: &Value) {
    println!("# NANDA-6M Budget\n");
    println!(
        "- state: `{}`",
        out["state"].as_str().unwrap_or("FOCUS_REQUIRED")
    );
    println!("- fits_l3: `{}`", out["fits_l3"]);
    println!(
        "- estimated_hot_bytes: `{}/{}`",
        out["estimated_hot_bytes"], out["hard_budget_bytes"]
    );
    println!(
        "- triads: `{}/{}`",
        out["counts"]["active_triads"], out["capacity"]["triads"]
    );
    println!(
        "- centroids: `{}/{}`",
        out["counts"]["centroids"], out["capacity"]["centroids"]
    );
    println!(
        "- lanes: `{}/{}`",
        out["counts"]["active_lanes"], out["capacity"]["lanes"]
    );
    println!("- next: {}", out["next"].as_str().unwrap_or(""));
}

fn probe_report(plain: &Value, negative: &Value, negative_lanes: usize) -> Value {
    let plain_top = plain["top_peak"].as_str().unwrap_or("");
    let negative_top = negative["top_peak"].as_str().unwrap_or("");
    let plain_score = top_peak_score(plain);
    let negative_score = top_peak_score(negative);
    let plain_safe = plain["safe_to_answer"].as_bool().unwrap_or(false);
    let negative_safe = negative["safe_to_answer"].as_bool().unwrap_or(false);
    let suppression_count = negative["destructive_interference"]["suppressions"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let top_changed = plain_top != negative_top;
    let became_safer = !plain_safe && negative_safe;
    let used_negative_lane = suppression_count > 0;
    let (decision, recommended_action) =
        if used_negative_lane && (became_safer || (top_changed && negative_safe)) {
            ("IMPROVED", "USE_NEGATIVE_RESULT")
        } else if used_negative_lane && top_changed {
            (
                "SHIFTED_TO_REVIEW",
                "INSPECT_NEGATIVE_SUPPORT_BEFORE_ANSWER",
            )
        } else if used_negative_lane {
            ("SUPPRESSED_WITHOUT_TOP_CHANGE", "COMPARE_SUPPORT_AND_SCORE")
        } else if !used_negative_lane && top_changed {
            ("CHANGED_WITHOUT_SUPPRESSION", "CHECK_INPUTS_OR_ROUTE_FOCUS")
        } else if plain_safe && !negative_safe {
            ("REGRESSED", "DO_NOT_USE_NEGATIVE_RESULT")
        } else {
            ("UNCHANGED", "NO_PROVEN_NEGATIVE_LANE_BENEFIT")
        };
    let legacy_improved = if used_negative_lane && (top_changed || became_safer) {
        "IMPROVED"
    } else {
        "UNCHANGED"
    };
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "probe-report",
        "decision": decision,
        "legacy_decision": legacy_improved,
        "recommended_action": recommended_action,
        "negative_lanes": negative_lanes,
        "plain": probe_search_summary(plain),
        "negative": probe_search_summary(negative),
        "delta": {
            "top_changed": top_changed,
            "verdict_changed": plain["verdict"] != negative["verdict"],
            "field_state_changed": plain["field_state"] != negative["field_state"],
            "safe_to_answer_changed": plain["safe_to_answer"] != negative["safe_to_answer"],
            "score_delta": round4(negative_score - plain_score),
            "suppression_count": suppression_count,
            "suppressed_peaks": negative["destructive_interference"]["suppressions"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item["suppressed_peak"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        },
        "destructive_interference": negative["destructive_interference"].clone(),
        "read_as": "Probe compares the same search before and after negative lanes. Treat IMPROVED as evidence that destructive interference repaired a shortcut; treat UNCHANGED as no proof of benefit."
    })
}

fn probe_search_summary(search: &Value) -> Value {
    json!({
        "verdict": search["verdict"],
        "field_state": search["field_state"],
        "safe_to_answer": search["safe_to_answer"],
        "top_peak": search["top_peak"],
        "top_score": round4(top_peak_score(search)),
        "peak_margin": search["peak_margin"],
        "lexical_baseline_top": search["lexical_baseline"]["top_peak"],
        "wins_over_lexical_baseline": search["wins_over_lexical_baseline"]
    })
}

fn top_peak_score(search: &Value) -> f64 {
    search["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["score"].as_f64())
        .unwrap_or(0.0)
}

fn index_cmd(args: IndexArgs) -> Result<u8> {
    if args.inputs.is_empty() {
        return Err(anyhow!(
            "nanda index requires at least one input packet or worksheet"
        ));
    }
    let mut triads = vec![];
    let mut negative_shortcuts = vec![];
    let mut positive_shortcuts = vec![];
    for input in &args.inputs {
        if let Some((negative, positive)) = load_feedback_lanes(input)? {
            negative_shortcuts.extend(negative);
            positive_shortcuts.extend(positive);
            continue;
        }
        let packet = load_packet_auto(
            input,
            &args.input_format,
            &args.task_id,
            &args.domain,
            &args.query,
            args.normalize_paths,
        )?;
        triads.extend(packet.triads);
        if args.include_candidates {
            triads.extend(packet.candidate_triads);
        }
        negative_shortcuts.extend(packet.negative_shortcuts);
        positive_shortcuts.extend(packet.positive_shortcuts);
    }
    let triads = dedup_triads(triads);
    let negative_shortcuts = merge_negative_shortcuts(negative_shortcuts);
    let positive_shortcuts = merge_positive_shortcuts(positive_shortcuts);
    let packet = json!({
        "task_id": args.task_id,
        "domain": args.domain,
        "query": args.query,
        "triads": triads,
        "candidate_triads": [],
        "candidate_answer": "",
        "negative_shortcuts": negative_shortcuts,
        "positive_shortcuts": positive_shortcuts,
        "index": {
            "core_version": CORE_VERSION,
            "wave_dim": WAVE_DIM,
            "source_files": args.inputs.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()
        }
    });
    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(&args.out, serde_json::to_string_pretty(&packet)? + "\n")?;
    println!("{}", args.out.display());
    Ok(EXIT_PASS)
}

fn load_feedback_negative_shortcuts(path: &Path) -> Result<Option<Vec<NegativeShortcut>>> {
    Ok(load_feedback_lanes(path)?.map(|(negative, _)| negative))
}

fn load_feedback_lanes(
    path: &Path,
) -> Result<Option<(Vec<NegativeShortcut>, Vec<PositiveShortcut>)>> {
    if !path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        return Ok(None);
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if value["mode"].as_str() != Some("feedback-memory") {
        return Ok(None);
    }
    let negative = value["negative_shortcuts"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<Vec<NegativeShortcut>, _>>()?;
    let positive = value["positive_shortcuts"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<Vec<PositiveShortcut>, _>>()?;
    Ok(Some((negative, positive)))
}

fn merge_negative_shortcuts(shortcuts: Vec<NegativeShortcut>) -> Vec<NegativeShortcut> {
    let mut merged: BTreeMap<
        (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ),
        NegativeShortcut,
    > = BTreeMap::new();
    for mut shortcut in shortcuts {
        if shortcut.observations == 0 {
            shortcut.observations = 1;
        }
        if shortcut.rejected_count == 0 && shortcut.accepted_count == 0 {
            shortcut.rejected_count = shortcut.observations;
        }
        shortcut.terms = normalized_shortcut_terms(&shortcut.terms)
            .into_iter()
            .collect::<Vec<_>>();
        shortcut.support_terms = normalized_shortcut_terms(&shortcut.support_terms)
            .into_iter()
            .collect::<Vec<_>>();
        let key = (
            norm(&shortcut.suppress_peak),
            norm(&shortcut.suppress_route),
            norm(&shortcut.suppress_group),
            norm(&shortcut.prefer_peak),
            norm(&shortcut.prefer_route),
            norm(&shortcut.prefer_group),
            shortcut.terms.join("|"),
            shortcut.support_terms.join("|"),
        );
        merged
            .entry(key)
            .and_modify(|existing| {
                existing.observations += shortcut.observations;
                existing.rejected_count += shortcut.rejected_count;
                existing.accepted_count += shortcut.accepted_count;
                existing.penalty = existing.penalty.max(shortcut.penalty);
                if existing.reason.is_empty() {
                    existing.reason = shortcut.reason.clone();
                }
                if existing.suppress_route.is_empty() {
                    existing.suppress_route = shortcut.suppress_route.clone();
                }
                if existing.suppress_group.is_empty() {
                    existing.suppress_group = shortcut.suppress_group.clone();
                }
                if existing.prefer_route.is_empty() {
                    existing.prefer_route = shortcut.prefer_route.clone();
                }
                if existing.prefer_group.is_empty() {
                    existing.prefer_group = shortcut.prefer_group.clone();
                }
                if !shortcut.source_feedback.is_empty() {
                    if existing.source_feedback.is_empty() {
                        existing.source_feedback = shortcut.source_feedback.clone();
                    } else if !existing
                        .source_feedback
                        .split(';')
                        .any(|item| item == shortcut.source_feedback)
                    {
                        existing.source_feedback.push(';');
                        existing.source_feedback.push_str(&shortcut.source_feedback);
                    }
                }
            })
            .or_insert(shortcut);
    }
    merged.into_values().collect()
}

fn normalized_shortcut_terms(terms: &[String]) -> BTreeSet<String> {
    terms
        .iter()
        .flat_map(|term| {
            norm(term)
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|token| token.len() >= 2)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn merge_positive_shortcuts(shortcuts: Vec<PositiveShortcut>) -> Vec<PositiveShortcut> {
    let mut merged: BTreeMap<(String, String, String, String, String), PositiveShortcut> =
        BTreeMap::new();
    for mut shortcut in shortcuts {
        if shortcut.observations == 0 {
            shortcut.observations = 1;
        }
        if shortcut.accepted_count == 0 && shortcut.rejected_count == 0 {
            shortcut.accepted_count = shortcut.observations;
        }
        shortcut.terms = normalized_shortcut_terms(&shortcut.terms)
            .into_iter()
            .collect::<Vec<_>>();
        shortcut.support_terms = normalized_shortcut_terms(&shortcut.support_terms)
            .into_iter()
            .collect::<Vec<_>>();
        let key = (
            norm(&shortcut.reinforce_peak),
            norm(&shortcut.reinforce_route),
            norm(&shortcut.reinforce_group),
            shortcut.terms.join("|"),
            shortcut.support_terms.join("|"),
        );
        merged
            .entry(key)
            .and_modify(|existing| {
                existing.observations += shortcut.observations;
                existing.accepted_count += shortcut.accepted_count;
                existing.rejected_count += shortcut.rejected_count;
                existing.boost = existing.boost.max(shortcut.boost);
                if existing.reason.is_empty() {
                    existing.reason = shortcut.reason.clone();
                }
                if existing.reinforce_route.is_empty() {
                    existing.reinforce_route = shortcut.reinforce_route.clone();
                }
                if existing.reinforce_group.is_empty() {
                    existing.reinforce_group = shortcut.reinforce_group.clone();
                }
                if !shortcut.source_feedback.is_empty() {
                    if existing.source_feedback.is_empty() {
                        existing.source_feedback = shortcut.source_feedback.clone();
                    } else if !existing
                        .source_feedback
                        .split(';')
                        .any(|item| item == shortcut.source_feedback)
                    {
                        existing.source_feedback.push(';');
                        existing.source_feedback.push_str(&shortcut.source_feedback);
                    }
                }
            })
            .or_insert(shortcut);
    }
    merged.into_values().collect()
}

fn extract_cmd(args: ExtractArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.input)
        .with_context(|| format!("read {}", args.input.display()))?;
    let packet = extract_packet_from_text(&text, &args.task_id, &args.domain, &args.query);
    let output = serde_json::to_string_pretty(&packet)? + "\n";
    if args.stdout {
        print!("{output}");
    } else {
        let out = args
            .out
            .unwrap_or_else(|| args.input.with_extension("nanda.json"));
        if let Some(parent) = out.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&out, output)?;
        println!("{}", out.display());
    }
    Ok(EXIT_PASS)
}

fn feedback_cmd(args: FeedbackArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.input)
        .with_context(|| format!("read {}", args.input.display()))?;
    let search: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", args.input.display()))?;
    let top = search["peaks"].as_array().and_then(|peaks| peaks.first());
    let peak_name = top
        .and_then(|peak| peak["peak"].as_str())
        .unwrap_or("")
        .to_string();
    let support_ids = top
        .and_then(|peak| peak["supporting_triads"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["id"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let anti_ids = top
        .and_then(|peak| peak["anti_triads"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["id"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let negative_shortcuts = if matches!(args.decision, FeedbackDecision::Reject) {
        vec![negative_shortcut_from_search(
            &search,
            &peak_name,
            &args.note,
            args.input.display().to_string(),
        )]
    } else {
        vec![]
    };
    let positive_shortcuts = if matches!(args.decision, FeedbackDecision::Accept) {
        vec![positive_shortcut_from_search(
            &search,
            &peak_name,
            &args.note,
            args.input.display().to_string(),
        )]
    } else {
        vec![]
    };
    let decision = feedback_decision_label(&args.decision);
    let reinforcement = match args.decision {
        FeedbackDecision::Accept => json!({
            "reinforce_peak": peak_name,
            "reinforce_support": support_ids,
            "suppress_foreign": anti_ids,
            "positive_shortcuts": positive_shortcuts
        }),
        FeedbackDecision::Reject => json!({
            "reject_peak": peak_name,
            "suppress_support": support_ids,
            "inspect_alternatives": anti_ids,
            "negative_shortcuts": negative_shortcuts
        }),
        FeedbackDecision::Watch => json!({
            "watch_peak": peak_name,
            "needs_evidence": support_ids,
            "possible_foreign_pull": anti_ids
        }),
    };
    let feedback = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "feedback-memory",
        "source_search": args.input.display().to_string(),
        "decision": decision,
        "note": args.note,
        "peak": peak_name,
        "peak_score": top.and_then(|peak| peak["score"].as_f64()).unwrap_or(0.0),
        "peak_margin": search["peak_margin"].as_f64().unwrap_or(0.0),
        "peak_decision": search["peak_decision"].clone(),
        "lexical_baseline_top": search["lexical_baseline"]["top_peak"].as_str().unwrap_or(""),
        "wins_over_lexical_baseline": search["wins_over_lexical_baseline"].as_bool().unwrap_or(false),
        "support_ids": support_ids,
        "anti_ids": anti_ids,
        "negative_shortcuts": negative_shortcuts,
        "positive_shortcuts": positive_shortcuts,
        "memory_patch": reinforcement,
        "interpretation": "Feedback is a compact trace for later memory tuning. Reject feedback creates a negative shortcut; accept feedback creates a positive shortcut that can boost the same supported route in later search."
    });
    let output = serde_json::to_string_pretty(&feedback)? + "\n";
    write_or_print(args.out, args.stdout, output)?;
    Ok(EXIT_PASS)
}

fn positive_shortcut_from_search(
    search: &Value,
    peak_name: &str,
    note: &str,
    source_feedback: String,
) -> PositiveShortcut {
    let top_peak = search["peaks"].as_array().and_then(|peaks| peaks.first());
    let reinforce_route = top_peak
        .and_then(|peak| peak["center"]["route"].as_str())
        .unwrap_or("")
        .to_string();
    let reinforce_group = top_peak
        .and_then(|peak| peak["center"]["group"].as_str())
        .unwrap_or("")
        .to_string();
    let support_terms = top_peak
        .and_then(|peak| peak["supporting_triads"].as_array())
        .map(|items| support_terms_from_items(items))
        .unwrap_or_default();
    let terms = query_tokens_from_search(search)
        .into_iter()
        .take(16)
        .collect::<Vec<_>>();
    PositiveShortcut {
        id: format!("pos-{}", slug(&format!("{peak_name}-{note}"))),
        reinforce_peak: peak_name.to_string(),
        reinforce_route,
        reinforce_group,
        boost: default_positive_boost(),
        terms,
        support_terms,
        reason: if note.trim().is_empty() {
            "accepted interference peak".to_string()
        } else {
            note.to_string()
        },
        source_feedback,
        observations: 1,
        accepted_count: 1,
        rejected_count: 0,
    }
}

fn negative_shortcut_from_search(
    search: &Value,
    peak_name: &str,
    note: &str,
    source_feedback: String,
) -> NegativeShortcut {
    let group_by = search["group_by"].as_str().unwrap_or("route");
    let top_peak = search["peaks"].as_array().and_then(|peaks| peaks.first());
    let suppress_route = top_peak
        .and_then(|peak| peak["center"]["route"].as_str())
        .unwrap_or("")
        .to_string();
    let suppress_group = top_peak
        .and_then(|peak| peak["center"]["group"].as_str())
        .unwrap_or("")
        .to_string();
    let support_terms = top_peak
        .and_then(|peak| peak["supporting_triads"].as_array())
        .map(|items| support_terms_from_items(items))
        .unwrap_or_default();
    let prefer_item = search["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["anti_triads"].as_array())
        .and_then(|items| items.first());
    let prefer_peak = search["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["anti_triads"].as_array())
        .and_then(|items| items.first())
        .and_then(|item| match group_by {
            "group" => item["group"].as_str(),
            _ => item["route"].as_str(),
        })
        .unwrap_or("")
        .to_string();
    let prefer_route = prefer_item
        .and_then(|item| item["route"].as_str())
        .unwrap_or("")
        .to_string();
    let prefer_group = prefer_item
        .and_then(|item| item["group"].as_str())
        .unwrap_or("")
        .to_string();
    let terms = query_tokens_from_search(search)
        .into_iter()
        .take(16)
        .collect::<Vec<_>>();
    NegativeShortcut {
        id: format!("neg-{}", slug(&format!("{peak_name}-{prefer_peak}-{note}"))),
        suppress_peak: peak_name.to_string(),
        suppress_route,
        suppress_group,
        prefer_peak,
        prefer_route,
        prefer_group,
        penalty: default_negative_penalty(),
        terms,
        support_terms,
        reason: if note.trim().is_empty() {
            "rejected interference peak".to_string()
        } else {
            note.to_string()
        },
        source_feedback,
        observations: 1,
        rejected_count: 1,
        accepted_count: 0,
    }
}

fn support_terms_from_items(items: &[Value]) -> Vec<String> {
    let mut terms = BTreeSet::new();
    for item in items.iter().take(3) {
        for key in ["subject", "relation", "object", "route", "group"] {
            if let Some(value) = item[key].as_str() {
                terms.extend(normalized_shortcut_terms(&[value.to_string()]));
            }
        }
    }
    terms.into_iter().take(24).collect()
}

fn query_tokens_from_search(search: &Value) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    if let Some(items) = search["query"]["triads"].as_array() {
        for item in items {
            for key in [
                "subject",
                "relation",
                "object",
                "subject_role",
                "object_role",
                "route",
                "group",
            ] {
                for token in norm(item[key].as_str().unwrap_or(""))
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .filter(|token| token.len() >= 2)
                {
                    tokens.insert(token.to_string());
                }
            }
        }
    }
    tokens
}

fn feedback_decision_label(decision: &FeedbackDecision) -> &'static str {
    match decision {
        FeedbackDecision::Accept => "accept",
        FeedbackDecision::Reject => "reject",
        FeedbackDecision::Watch => "watch",
    }
}

fn eval_cmd(args: EvalArgs) -> Result<u8> {
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

fn eval_cases_from_args(args: &EvalArgs) -> Result<Vec<(PathBuf, String, String)>> {
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

fn parse_eval_case(raw: &str) -> Result<(PathBuf, String, String)> {
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

fn waw_cmd(args: WawArgs) -> Result<u8> {
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

fn print_waw_text(out: &Value) {
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

fn print_waw_md(out: &Value) {
    println!("# NANDA WAW Benchmark\n");
    println!("- core: `{}`", out["core_version"].as_str().unwrap_or(""));
    println!("- suite: `{}`", out["suite"].as_str().unwrap_or(""));
    println!("- passed: `{}/{}`", out["passed"], out["total"]);
    println!("- waw_score: `{}`", out["waw_score"]);
    println!("- structural_wins: `{}`", out["structural_wins"]);
    println!("- lexical_traps: `{}`", out["lexical_traps"]);
    println!("- explainable_drifts: `{}`", out["explainable_drifts"]);
}

fn print_eval_text(out: &Value) {
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

fn print_eval_md(out: &Value) {
    println!("# NANDA Eval\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- passed: `{}/{}`", out["passed"], out["total"]);
    println!("- accuracy: `{}`", out["accuracy"]);
}

fn serve_cmd(args: ServeArgs) -> Result<u8> {
    match args.format {
        ServeFormat::Jsonl => serve_jsonl(),
    }
}

fn serve_jsonl() -> Result<u8> {
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

fn handle_serve_request(line: &str) -> Result<Value> {
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

fn doctor_cmd(args: DoctorArgs) -> Result<u8> {
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

fn doctor_value() -> Value {
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

fn print_doctor_text(out: &Value) {
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

fn print_doctor_md(out: &Value) {
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

fn builtin_route_trap_packet(noisy: bool) -> Packet {
    let triads = if noisy {
        vec![
            doctor_triad(
                "m1",
                "Monster",
                "has_route",
                "certification",
                "product",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m2",
                "certification payment",
                "pays_for",
                "TR CU declaration",
                "payment",
                "document",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m3",
                "TR CU declaration",
                "requires",
                "test protocols",
                "document",
                "evidence",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m4",
                "Maria Elena payment",
                "belongs_to",
                "certification route",
                "payment",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m5",
                "Monster",
                "produced_by",
                "Guangzhou 998",
                "product",
                "factory",
                "production",
                "production-route",
            ),
            doctor_triad(
                "m6",
                "Monster",
                "packed_at",
                "Guangzhou 998",
                "product",
                "factory",
                "production",
                "production-route",
            ),
            doctor_triad(
                "m7",
                "Guangzhou 998",
                "ships",
                "Monster",
                "factory",
                "product",
                "production",
                "production-route",
            ),
            doctor_triad(
                "m8",
                "importer",
                "files",
                "customs declaration",
                "company",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m9",
                "customs declaration",
                "requires",
                "payment confirmation",
                "document",
                "evidence",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m10",
                "Maria Elena payment",
                "not_pays_for",
                "customs declaration",
                "payment",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m11",
                "customs declaration",
                "not_same_as",
                "TR CU declaration",
                "document",
                "document",
                "customs",
                "customs-route",
            ),
        ]
    } else {
        vec![
            doctor_triad(
                "m1",
                "Monster",
                "has_route",
                "certification",
                "product",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m2",
                "certification payment",
                "pays_for",
                "declaration",
                "payment",
                "document",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m3",
                "declaration",
                "requires",
                "protocols",
                "document",
                "evidence",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m4",
                "protocols",
                "support",
                "certification",
                "evidence",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m5",
                "Monster",
                "pays_for",
                "declaration",
                "product",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m6",
                "customs declaration",
                "requires",
                "payment confirmation",
                "document",
                "evidence",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m7",
                "importer",
                "files",
                "customs declaration",
                "company",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m8",
                "Monster",
                "produced_by",
                "Guangzhou 998",
                "product",
                "factory",
                "production",
                "production-route",
            ),
        ]
    };
    let candidate_triads = if noisy {
        vec![
            doctor_triad("q1", "Monster", "", "", "product", "", "", "query"),
            doctor_triad(
                "q2", "payment", "pays_for", "document", "payment", "document", "", "query",
            ),
            doctor_triad(
                "q3", "document", "requires", "evidence", "document", "evidence", "", "query",
            ),
        ]
    } else {
        vec![
            doctor_triad(
                "q1",
                "Monster",
                "pays_for",
                "declaration",
                "product",
                "document",
                "",
                "query",
            ),
            doctor_triad(
                "q2",
                "declaration",
                "requires",
                "protocols",
                "document",
                "evidence",
                "",
                "query",
            ),
        ]
    };
    Packet {
        task_id: "doctor-route-trap".to_string(),
        domain: "doctor".to_string(),
        query: "builtin doctor route trap".to_string(),
        triads,
        candidate_triads,
        candidate_answer: String::new(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
    }
}

fn doctor_triad(
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

fn extract_packet_from_text(text: &str, task_id: &str, domain: &str, query: &str) -> Packet {
    let mut triads = vec![];
    let mut candidates = vec![];
    let mut target_candidates = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') && !trimmed.starts_with("##") {
            continue;
        }
        let lower = norm(trimmed);
        if lower.starts_with("## candidate") || lower.starts_with("[candidate") {
            target_candidates = true;
            continue;
        }
        if lower.starts_with("## triads")
            || lower.starts_with("[triads")
            || lower.starts_with("## memory")
        {
            target_candidates = false;
            continue;
        }
        if let Some(mut triad) = parse_arrow_triad(trimmed) {
            let prefix = if target_candidates { "q" } else { "m" };
            let idx = if target_candidates {
                candidates.len() + 1
            } else {
                triads.len() + 1
            };
            if triad.id.is_empty() {
                triad.id = format!("{prefix}{idx}");
            }
            if target_candidates {
                candidates.push(triad);
            } else {
                triads.push(triad);
            }
        }
    }
    Packet {
        task_id: task_id.to_string(),
        domain: domain.to_string(),
        query: query.to_string(),
        triads,
        candidate_triads: candidates,
        candidate_answer: String::new(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
    }
}

fn parse_arrow_triad(line: &str) -> Option<Triad> {
    let (body, attrs) = if let Some(start) = line.find('[') {
        let end = line.rfind(']').unwrap_or(line.len());
        (&line[..start], parse_attrs(&line[start + 1..end]))
    } else {
        (line, BTreeMap::new())
    };
    let parts = body
        .split("->")
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }
    Some(Triad {
        id: attrs.get("id").cloned().unwrap_or_default(),
        subject: parts[0].to_string(),
        relation: parts[1].to_string(),
        object: parts[2..].join(" -> "),
        evidence: attrs.get("evidence").cloned().unwrap_or_default(),
        confidence: attrs
            .get("confidence")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1.0),
        subject_role: attrs
            .get("subject_role")
            .cloned()
            .unwrap_or_else(|| "subject".to_string()),
        object_role: attrs
            .get("object_role")
            .cloned()
            .unwrap_or_else(|| "object".to_string()),
        route: attrs.get("route").cloned().unwrap_or_default(),
        group: attrs.get("group").cloned().unwrap_or_default(),
    })
}

fn parse_attrs(value: &str) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    for item in value.split_whitespace() {
        if let Some((key, value)) = item.split_once('=') {
            attrs.insert(norm(key), value.trim_matches('"').to_string());
        }
    }
    attrs
}

fn dedup_triads(triads: Vec<Triad>) -> Vec<Triad> {
    let mut seen = BTreeSet::new();
    let mut out = vec![];
    for triad in triads {
        let key = (
            norm(&triad.subject),
            norm(&triad.relation),
            norm(&triad.object),
            norm(&triad.subject_role),
            norm(&triad.object_role),
            norm(&triad.route),
            norm(&triad.group),
        );
        if seen.insert(key) {
            out.push(triad);
        }
    }
    out
}

fn search_query_triads(packet: &Packet, query_text: &str) -> (Vec<Triad>, &'static str) {
    if !packet.candidate_triads.is_empty() {
        return (
            normalize_ids(packet.candidate_triads.clone(), "q"),
            "candidate_triads",
        );
    }
    let auto = auto_query_triads(query_text);
    if auto.is_empty() {
        (vec![], "empty")
    } else {
        (normalize_ids(auto, "q"), "auto_query_triads")
    }
}

fn auto_query_triads(query_text: &str) -> Vec<Triad> {
    let tokens = query_tokens_from_text(query_text);
    if tokens.is_empty() {
        return vec![];
    }
    let topic = tokens.iter().take(5).cloned().collect::<Vec<_>>().join(" ");
    let mut triads = vec![Triad {
        id: "q1".to_string(),
        subject: "query".to_string(),
        relation: "asks_about".to_string(),
        object: topic.clone(),
        evidence: "auto_query".to_string(),
        confidence: 0.72,
        subject_role: "query".to_string(),
        object_role: "topic".to_string(),
        route: String::new(),
        group: "auto-query".to_string(),
    }];
    if tokens.len() >= 2 {
        triads.push(Triad {
            id: "q2".to_string(),
            subject: tokens[0].clone(),
            relation: "co_occurs_with".to_string(),
            object: tokens[1].clone(),
            evidence: "auto_query".to_string(),
            confidence: 0.66,
            subject_role: "query_term".to_string(),
            object_role: "query_term".to_string(),
            route: String::new(),
            group: "auto-query".to_string(),
        });
    }
    if tokens.len() >= 3 {
        triads.push(Triad {
            id: "q3".to_string(),
            subject: tokens[1].clone(),
            relation: "co_occurs_with".to_string(),
            object: tokens[2].clone(),
            evidence: "auto_query".to_string(),
            confidence: 0.62,
            subject_role: "query_term".to_string(),
            object_role: "query_term".to_string(),
            route: String::new(),
            group: "auto-query".to_string(),
        });
    }
    triads
}

fn query_tokens_from_text(query_text: &str) -> Vec<String> {
    let stop = [
        "about", "after", "again", "against", "and", "are", "before", "between", "can", "could",
        "does", "for", "from", "how", "into", "need", "needs", "not", "our", "route", "should",
        "that", "the", "then", "this", "what", "when", "where", "which", "with", "без", "где",
        "для", "как", "или", "что", "это",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut out = vec![];
    for token in norm(query_text)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.chars().count() >= 2)
    {
        if stop.contains(token) || !seen.insert(token.to_string()) {
            continue;
        }
        out.push(token.to_string());
        if out.len() == 12 {
            break;
        }
    }
    out
}

fn source_weight(triad: &Triad) -> f64 {
    let text = norm(&format!(
        "{} {} {} {}",
        triad.evidence, triad.route, triad.group, triad.object_role
    ));
    let authority = if text.contains("archive_noise") || text.contains("noise") {
        0.35
    } else if text.contains("historical") || text.contains("archive") || text.contains("old") {
        0.65
    } else if text.contains("current")
        || text.contains("canon")
        || text.contains("canonical")
        || text.contains("source")
    {
        1.12
    } else if text.contains("latest")
        || text.contains("frontier")
        || text.contains("w-chain")
        || text.contains("w_")
        || text.contains("w-")
    {
        0.95
    } else {
        0.86
    };
    round4((triad.confidence.clamp(0.05, 1.0) * authority).clamp(0.05, 1.2))
}

struct FocusedMemory {
    memory: Vec<Triad>,
    metadata: Value,
}

fn route_balanced_focus(
    memory: &[Triad],
    query: &[Triad],
    route_cap: usize,
    route_triad_cap: usize,
) -> FocusedMemory {
    let route_cap = route_cap.max(1);
    let route_triad_cap = route_triad_cap.max(1);
    if memory.len() <= route_cap {
        return FocusedMemory {
            memory: memory.to_vec(),
            metadata: json!({
                "enabled": false,
                "reason": "memory_size_within_route_cap",
                "route_cap": route_cap,
                "route_triad_cap": route_triad_cap,
                "original_memory_size": memory.len(),
                "focused_memory_size": memory.len()
            }),
        };
    }
    let query_terms = query_term_set(query);
    let mut by_route: BTreeMap<String, Vec<(f64, &Triad)>> = BTreeMap::new();
    for triad in memory {
        let relevance = (0.52 * symbolic_query_overlap(query, triad))
            + (0.28 * token_overlap(&query_terms, triad))
            + (0.20 * source_weight(triad));
        by_route
            .entry(route_name(triad, "memory-route"))
            .or_default()
            .push((round4(relevance), triad));
    }
    let mut focused = vec![];
    let mut route_rows = vec![];
    for (route, mut items) in by_route {
        items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let before = items.len();
        let selected = items
            .iter()
            .take(route_triad_cap)
            .map(|(_, triad)| (*triad).clone())
            .collect::<Vec<_>>();
        route_rows.push(json!({
            "route": route,
            "original_triads": before,
            "selected_triads": selected.len(),
            "top_relevance": items.first().map(|(score, _)| *score).unwrap_or(0.0)
        }));
        focused.extend(selected);
    }
    route_rows.sort_by(|a, b| {
        b["top_relevance"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["top_relevance"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    FocusedMemory {
        memory: focused,
        metadata: json!({
            "enabled": true,
            "reason": "memory_size_exceeded_route_cap",
            "route_cap": route_cap,
            "route_triad_cap": route_triad_cap,
            "original_memory_size": memory.len(),
            "focused_memory_size": route_rows.iter().map(|row| row["selected_triads"].as_u64().unwrap_or(0)).sum::<u64>() as usize,
            "routes": route_rows
        }),
    }
}

fn no_focus_metadata(memory_size: usize) -> Value {
    json!({
        "enabled": false,
        "reason": "not_requested_by_internal_caller",
        "original_memory_size": memory_size,
        "focused_memory_size": memory_size
    })
}

fn interference_search(
    packet: &Packet,
    memory: &[Triad],
    query: &[Triad],
    top_k: usize,
    group_by: &PeakGroupBy,
    query_source: &str,
    route_balanced_focus: Value,
) -> Value {
    let query_wave = query_feature_wave(query);
    let mut scored = vec![];
    for triad in memory {
        let wave = triad_feature_wave(triad);
        let score = cosine(&query_wave, &wave);
        let symbolic = symbolic_query_overlap(query, triad);
        let weight = source_weight(triad);
        let combined = round4(((0.72 * score) + (0.28 * symbolic)) * weight);
        scored.push((combined, score, symbolic, triad));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut groups: BTreeMap<String, Vec<(f64, f64, f64, &Triad)>> = BTreeMap::new();
    for item in &scored {
        let key = match group_by {
            PeakGroupBy::Group => group_name(item.3, "memory"),
            PeakGroupBy::Route => route_name(item.3, "memory-route"),
        };
        groups.entry(key).or_default().push(*item);
    }

    let query_terms = query_term_set(query);
    let mut peaks = vec![];
    for (key, items) in groups {
        let mut group_wave = vec![0; WAVE_DIM];
        for (_, _, _, triad) in &items {
            add_into(&mut group_wave, &triad_feature_wave(triad));
        }
        let coherence = round4(cosine(&query_wave, &group_wave));
        let coverage = round4(route_term_coverage(&query_terms, &items));
        let chain = round4(chain_coherence(&items));
        let propagation = propagation_summary(&query_terms, &items);
        let propagation_score = propagation["score"].as_f64().unwrap_or(0.0);
        let component_score = propagation["component_score"].as_f64().unwrap_or(0.0);
        let top_score = items.first().map(|item| item.0).unwrap_or(0.0);
        let avg_top3 = if items.is_empty() {
            0.0
        } else {
            let take = items.len().min(3);
            items.iter().take(take).map(|item| item.0).sum::<f64>() / take as f64
        };
        let base_peak_score = round4(
            (0.26 * coherence)
                + (0.18 * coverage)
                + (0.15 * avg_top3)
                + (0.11 * chain)
                + (0.12 * propagation_score)
                + (0.18 * component_score),
        );
        let support = items
            .iter()
            .take(8)
            .map(|(combined, wave_score, symbolic, triad)| {
                json!({
                    "id": triad.id,
                    "score": combined,
                    "wave_score": round4(*wave_score),
                    "symbolic_overlap": round4(*symbolic),
                    "source_weight": source_weight(triad),
                    "polarity": triad_polarity(triad),
                    "subject": triad.subject,
                    "relation": triad.relation,
                    "object": triad.object,
                    "route": triad.route,
                    "group": triad.group,
                    "evidence": triad.evidence
                })
            })
            .collect::<Vec<_>>();
        let anti = scored
            .iter()
            .filter(|(_, _, _, triad)| {
                let triad_key = match group_by {
                    PeakGroupBy::Group => group_name(triad, "memory"),
                    PeakGroupBy::Route => route_name(triad, "memory-route"),
                };
                triad_key != key
            })
            .take(5)
            .map(|(combined, wave_score, symbolic, triad)| {
                json!({
                    "id": triad.id,
                    "score": combined,
                    "wave_score": round4(*wave_score),
                    "symbolic_overlap": round4(*symbolic),
                    "source_weight": source_weight(triad),
                    "polarity": triad_polarity(triad),
                    "subject": triad.subject,
                    "relation": triad.relation,
                    "object": triad.object,
                    "route": triad.route,
                    "group": triad.group,
                    "reason": "similar query features but outside this peak route/group"
                })
            })
            .collect::<Vec<_>>();
        let center = peak_center(&items);
        let polarization = polarization_summary(query, &items);
        let polarization_penalty = polarization_penalty(&polarization);
        let peak_score = round4((base_peak_score - polarization_penalty).max(0.0));
        peaks.push(json!({
            "peak": key,
            "score": peak_score,
            "raw_score": base_peak_score,
            "polarization_penalty": polarization_penalty,
            "coherence": coherence,
            "coverage": coverage,
            "chain_coherence": chain,
            "propagation": propagation,
            "top_triad_score": round4(top_score),
            "symbolic_baseline": symbolic_peak_baseline(&items),
            "center": center,
            "polarization": polarization,
            "supporting_triads": support,
            "anti_triads": anti,
            "missing_edges": missing_edges(&query_terms, &items),
            "answer_projection": answer_projection(&center, &items)
        }));
    }
    let destructive_interference =
        apply_negative_lanes(&mut peaks, query, &packet.negative_shortcuts);
    let constructive_interference =
        apply_positive_lanes(&mut peaks, query, &packet.positive_shortcuts);
    peaks.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let peak_margin = if peaks.len() >= 2 {
        round4(
            peaks[0]["score"].as_f64().unwrap_or(0.0) - peaks[1]["score"].as_f64().unwrap_or(0.0),
        )
    } else {
        0.0
    };
    let lexical_baseline = lexical_baseline(&scored, query, group_by);
    let corpus_interpretation = corpus_diagnostics(memory, query, &packet.query, 256);
    let top_peak = peaks
        .first()
        .and_then(|peak| peak["peak"].as_str())
        .unwrap_or("")
        .to_string();
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let peak_decision = peak_decision(&peaks, peak_margin, lexical_peak);
    let mut field_interpretation = field_interpretation(&peaks, peak_margin, &lexical_baseline);
    if let Some(object) = field_interpretation.as_object_mut() {
        object.insert("corpus".to_string(), corpus_interpretation.clone());
    }
    let coarse_to_fine = coarse_to_fine_trace(&peaks, &query_terms);
    let field_state_machine = field_state_machine(
        &peaks,
        peak_margin,
        &lexical_baseline,
        &corpus_interpretation,
        &route_balanced_focus,
        &coarse_to_fine,
    );
    let field_state = field_state_machine["state"].as_str().unwrap_or("NO_FIELD");
    let safe_to_answer = field_state_machine["safe_to_answer"]
        .as_bool()
        .unwrap_or(false);
    let verdict = search_verdict(field_state, safe_to_answer);
    let mut output_peaks = peaks;
    output_peaks.truncate(top_k);

    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "interference-retrieval",
        "verdict": verdict,
        "field_state": field_state,
        "safe_to_answer": safe_to_answer,
        "top_peak": top_peak,
        "task_id": packet.task_id,
        "domain": packet.domain,
        "query": {
            "text": packet.query,
            "source": query_source,
            "triads": query.iter().map(triad_json).collect::<Vec<_>>()
        },
        "memory_size": memory.len(),
        "route_balanced_focus": route_balanced_focus,
        "group_by": match group_by {
            PeakGroupBy::Group => "group",
            PeakGroupBy::Route => "route",
        },
        "peak_margin": peak_margin,
        "lexical_baseline": lexical_baseline,
        "wins_over_lexical_baseline": !lexical_peak.is_empty() && top_peak.as_str() != lexical_peak,
        "peak_decision": peak_decision,
        "destructive_interference": destructive_interference,
        "constructive_interference": constructive_interference,
        "source_weighting": {
            "enabled": true,
            "policy": "confidence multiplied by evidence authority: current/canon > latest/frontier > historical/archive > archive_noise"
        },
        "coarse_to_fine": coarse_to_fine,
        "field_state_machine": field_state_machine,
        "field_interpretation": field_interpretation,
        "peaks": output_peaks,
        "interpretation": "A peak is a route/group whose triads resonate together with the partial query. Read support as the focused structure and anti_triads as similar-but-foreign pulls."
    })
}

fn apply_positive_lanes(
    peaks: &mut [Value],
    query: &[Triad],
    shortcuts: &[PositiveShortcut],
) -> Value {
    let query_tokens = query_token_set(query);
    let mut reinforcements = vec![];
    if shortcuts.is_empty() || peaks.is_empty() {
        return json!({
            "applied": false,
            "positive_lanes": shortcuts.len(),
            "reinforcements": reinforcements
        });
    }
    for shortcut in shortcuts {
        let query_ratio = positive_lane_match_ratio(&query_tokens, shortcut);
        if query_ratio <= 0.0 {
            continue;
        }
        let accepted_count = shortcut_accepted_count(shortcut);
        let learned_boost =
            (shortcut.boost.max(0.0) + (accepted_count.saturating_sub(1) as f64 * 0.025)).min(0.25);
        for peak in peaks.iter_mut() {
            let Some(peak_name) = peak["peak"].as_str().map(str::to_string) else {
                continue;
            };
            if !positive_lane_matches_reinforce(peak, shortcut) {
                continue;
            }
            let support_ratio = positive_lane_support_ratio(peak, shortcut);
            let lane_ratio = round4(query_ratio * support_ratio);
            if lane_ratio <= 0.0 {
                continue;
            }
            let boost = round4(learned_boost * lane_ratio);
            let old_score = peak["score"].as_f64().unwrap_or(0.0);
            let new_score = round4((old_score + boost).min(1.5));
            if let Some(object) = peak.as_object_mut() {
                object.insert("score".to_string(), json!(new_score));
                object.insert("raw_score".to_string(), json!(round4(old_score)));
                object.insert("positive_lane_boost".to_string(), json!(boost));
            }
            reinforcements.push(json!({
                "shortcut": shortcut.id,
                "reinforce_peak": peak_name,
                "reinforce_route": shortcut.reinforce_route,
                "reinforce_group": shortcut.reinforce_group,
                "boost": boost,
                "effective_boost": round4(learned_boost),
                "match_ratio": lane_ratio,
                "query_match_ratio": round4(query_ratio),
                "support_match_ratio": round4(support_ratio),
                "observations": shortcut.observations,
                "accepted_count": accepted_count,
                "reason": shortcut.reason
            }));
        }
    }
    json!({
        "applied": !reinforcements.is_empty(),
        "positive_lanes": shortcuts.len(),
        "reinforcements": reinforcements
    })
}

fn apply_negative_lanes(
    peaks: &mut [Value],
    query: &[Triad],
    shortcuts: &[NegativeShortcut],
) -> Value {
    let query_tokens = query_token_set(query);
    let mut suppressions = vec![];
    if shortcuts.is_empty() || peaks.is_empty() {
        return json!({
            "applied": false,
            "negative_lanes": shortcuts.len(),
            "suppressions": suppressions
        });
    }
    for shortcut in shortcuts {
        let query_ratio = negative_lane_match_ratio(&query_tokens, shortcut);
        if query_ratio <= 0.0 {
            continue;
        }
        let rejected_count = shortcut_rejected_count(shortcut);
        let learned_penalty = (shortcut.penalty.max(0.0)
            + (rejected_count.saturating_sub(1) as f64 * 0.04))
            .min(0.45);
        for peak in peaks.iter_mut() {
            let Some(peak_name) = peak["peak"].as_str().map(str::to_string) else {
                continue;
            };
            let support_ratio = negative_lane_support_ratio(peak, shortcut);
            let lane_ratio = round4(query_ratio * support_ratio);
            if lane_ratio <= 0.0 {
                continue;
            }
            let penalty = round4(learned_penalty * lane_ratio);
            let boost = round4(penalty * 0.35);
            if negative_lane_matches_suppress(peak, shortcut) {
                let old_score = peak["score"].as_f64().unwrap_or(0.0);
                let new_score = round4((old_score - penalty).max(0.0));
                if let Some(object) = peak.as_object_mut() {
                    object.insert("score".to_string(), json!(new_score));
                    object.insert("raw_score".to_string(), json!(round4(old_score)));
                    object.insert("negative_lane_penalty".to_string(), json!(penalty));
                }
                suppressions.push(json!({
                    "shortcut": shortcut.id,
                    "suppress_peak": peak_name,
                    "suppressed_peak": peak_name,
                    "suppress_route": shortcut.suppress_route,
                    "suppress_group": shortcut.suppress_group,
                    "penalty": penalty,
                    "effective_penalty": round4(learned_penalty),
                    "match_ratio": lane_ratio,
                    "query_match_ratio": round4(query_ratio),
                    "support_match_ratio": round4(support_ratio),
                    "observations": shortcut.observations,
                    "rejected_count": rejected_count,
                    "prefer_peak": shortcut.prefer_peak,
                    "prefer_route": shortcut.prefer_route,
                    "prefer_group": shortcut.prefer_group,
                    "reason": shortcut.reason
                }));
            } else if (!shortcut.prefer_peak.is_empty()
                || !shortcut.prefer_route.is_empty()
                || !shortcut.prefer_group.is_empty())
                && negative_lane_matches_prefer(peak, shortcut)
            {
                let old_score = peak["score"].as_f64().unwrap_or(0.0);
                let new_score = round4(old_score + boost);
                if let Some(object) = peak.as_object_mut() {
                    object.insert("score".to_string(), json!(new_score));
                    object.insert("raw_score".to_string(), json!(round4(old_score)));
                    object.insert("negative_lane_boost".to_string(), json!(boost));
                }
            }
        }
    }
    json!({
        "applied": !suppressions.is_empty(),
        "negative_lanes": shortcuts.len(),
        "suppressions": suppressions
    })
}

fn negative_lane_matches_suppress(peak: &Value, shortcut: &NegativeShortcut) -> bool {
    negative_lane_matches_labels(
        peak,
        &[
            shortcut.suppress_peak.as_str(),
            shortcut.suppress_route.as_str(),
            shortcut.suppress_group.as_str(),
        ],
    )
}

fn negative_lane_matches_prefer(peak: &Value, shortcut: &NegativeShortcut) -> bool {
    negative_lane_matches_labels(
        peak,
        &[
            shortcut.prefer_peak.as_str(),
            shortcut.prefer_route.as_str(),
            shortcut.prefer_group.as_str(),
        ],
    )
}

fn positive_lane_matches_reinforce(peak: &Value, shortcut: &PositiveShortcut) -> bool {
    negative_lane_matches_labels(
        peak,
        &[
            shortcut.reinforce_peak.as_str(),
            shortcut.reinforce_route.as_str(),
            shortcut.reinforce_group.as_str(),
        ],
    )
}

fn negative_lane_matches_labels(peak: &Value, labels: &[&str]) -> bool {
    let peak_name = peak["peak"].as_str().unwrap_or("");
    let route = peak["center"]["route"].as_str().unwrap_or("");
    let group = peak["center"]["group"].as_str().unwrap_or("");
    let composite = format!("{route}:{group}");
    labels.iter().any(|hint| {
        let hint = norm(hint);
        !hint.is_empty()
            && (hint == norm(peak_name)
                || hint == norm(route)
                || hint == norm(group)
                || hint == norm(&composite))
    })
}

fn positive_lane_support_ratio(peak: &Value, shortcut: &PositiveShortcut) -> f64 {
    let terms = normalized_shortcut_terms(&shortcut.support_terms);
    if terms.is_empty() {
        return 1.0;
    }
    let mut peak_terms = BTreeSet::new();
    if let Some(items) = peak["supporting_triads"].as_array() {
        for item in items.iter().take(8) {
            for key in ["subject", "relation", "object", "route", "group"] {
                if let Some(value) = item[key].as_str() {
                    peak_terms.extend(normalized_shortcut_terms(&[value.to_string()]));
                }
            }
        }
    }
    if peak_terms.is_empty() {
        return 0.0;
    }
    let hits = terms.intersection(&peak_terms).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.35 {
        ratio
    } else {
        0.0
    }
}

fn negative_lane_support_ratio(peak: &Value, shortcut: &NegativeShortcut) -> f64 {
    let terms = normalized_shortcut_terms(&shortcut.support_terms);
    if terms.is_empty() {
        return 1.0;
    }
    let mut peak_terms = BTreeSet::new();
    if let Some(items) = peak["supporting_triads"].as_array() {
        for item in items.iter().take(8) {
            for key in ["subject", "relation", "object", "route", "group"] {
                if let Some(value) = item[key].as_str() {
                    peak_terms.extend(normalized_shortcut_terms(&[value.to_string()]));
                }
            }
        }
    }
    if peak_terms.is_empty() {
        return 0.0;
    }
    let hits = terms.intersection(&peak_terms).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.35 {
        ratio
    } else {
        0.0
    }
}

fn shortcut_accepted_count(shortcut: &PositiveShortcut) -> usize {
    if shortcut.accepted_count > 0 {
        shortcut.accepted_count
    } else {
        shortcut.observations.max(1)
    }
}

fn shortcut_rejected_count(shortcut: &NegativeShortcut) -> usize {
    if shortcut.rejected_count > 0 {
        shortcut.rejected_count
    } else {
        shortcut.observations.max(1)
    }
}

fn positive_lane_match_ratio(query_tokens: &BTreeSet<String>, shortcut: &PositiveShortcut) -> f64 {
    if shortcut.reinforce_peak.trim().is_empty() {
        return 0.0;
    }
    if shortcut.terms.is_empty() {
        return 1.0;
    }
    if query_tokens.is_empty() {
        return 0.0;
    }
    let terms = normalized_shortcut_terms(&shortcut.terms);
    if terms.is_empty() {
        return 1.0;
    }
    let hits = terms.intersection(query_tokens).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.5 {
        ratio
    } else {
        0.0
    }
}

fn negative_lane_match_ratio(query_tokens: &BTreeSet<String>, shortcut: &NegativeShortcut) -> f64 {
    if shortcut.suppress_peak.trim().is_empty() {
        return 0.0;
    }
    if shortcut.terms.is_empty() {
        return 1.0;
    }
    if query_tokens.is_empty() {
        return 0.0;
    }
    let terms = normalized_shortcut_terms(&shortcut.terms);
    if terms.is_empty() {
        return 1.0;
    }
    let hits = terms.intersection(query_tokens).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.5 {
        ratio
    } else {
        0.0
    }
}

fn corpus_diagnostics(
    memory: &[Triad],
    query: &[Triad],
    query_text: &str,
    route_cap: usize,
) -> Value {
    let triad_count = memory.len();
    let large_corpus = triad_count > route_cap;
    let route_counts = count_by(memory.iter().map(|triad| route_name(triad, "memory-route")));
    let group_counts = count_by(memory.iter().map(|triad| group_name(triad, "memory")));
    let relation_counts = count_by(memory.iter().map(|triad| norm(&triad.relation)));
    let mut entity_counts = BTreeMap::<String, usize>::new();
    for triad in memory {
        for entity in [norm(&triad.subject), norm(&triad.object)] {
            if !entity.is_empty() {
                *entity_counts.entry(entity).or_default() += 1;
            }
        }
    }
    let route_distribution = distribution_rows(&route_counts, triad_count, 8);
    let group_distribution = distribution_rows(&group_counts, triad_count, 8);
    let hub_nodes = hub_rows(&entity_counts, triad_count, 8);
    let top_route_share = route_distribution
        .first()
        .and_then(|row| row["share"].as_f64())
        .unwrap_or(0.0);
    let top_hub_share = hub_nodes
        .first()
        .and_then(|row| row["share"].as_f64())
        .unwrap_or(0.0);
    let duplicate_rows = duplicate_rows(memory, 8);
    let duplicate_count = duplicate_rows
        .iter()
        .map(|row| row["count"].as_u64().unwrap_or(0).saturating_sub(1))
        .sum::<u64>() as usize;
    let current_duplicates = duplicate_rows
        .iter()
        .filter(|row| {
            row["evidence_refs"].as_array().is_some_and(|refs| {
                refs.iter()
                    .any(|value| norm(value.as_str().unwrap_or("")).contains("current"))
            })
        })
        .count();
    let weak_query = query.is_empty() && !query_text.trim().is_empty() && large_corpus;
    let empty_query = query.is_empty() && query_text.trim().is_empty();
    let route_imbalance = large_corpus && top_route_share >= 0.45;
    let hub_dominance = large_corpus && top_hub_share >= 0.08;
    let duplicate_current = large_corpus && current_duplicates > 0;
    let mut warnings = vec![];
    let mut notices = vec![];
    if large_corpus && route_imbalance {
        warnings.push(json!({
            "kind": "large_unbalanced_corpus",
            "message": "Corpus exceeds direct-search route cap and is route-heavy; build a route-balanced focus packet before search."
        }));
    } else if large_corpus {
        notices.push(json!({
            "kind": "large_but_route_balanced_focus",
            "message": "Corpus exceeds direct-search route cap, but no single route dominates; continue with coarse-to-fine search or route caps."
        }));
    }
    if route_imbalance {
        warnings.push(json!({
            "kind": "route_imbalance",
            "message": "One route dominates the field; use stratified route sampling or coarse-to-fine search."
        }));
    }
    if hub_dominance {
        warnings.push(json!({
            "kind": "hub_dominance",
            "message": "A high-frequency entity behaves like a hub and may pin unrelated peaks."
        }));
    }
    if duplicate_current {
        warnings.push(json!({
            "kind": "duplicate_current",
            "message": "Current/canonical evidence appears duplicated with dated or archive copies."
        }));
    }
    if weak_query {
        warnings.push(json!({
            "kind": "weak_text_query",
            "message": "Text query without candidate_triads is weak on large corpora; provide query triads or extract lightweight query_triads."
        }));
    }
    if empty_query {
        warnings.push(json!({
            "kind": "empty_query",
            "message": "No candidate_triads or query text were provided; search activation will not be query-specific."
        }));
    }
    let verdict = if warnings.is_empty() { "PASS" } else { "WATCH" };
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "dataset-doctor",
        "verdict": verdict,
        "triad_count": triad_count,
        "route_count": route_counts.len(),
        "group_count": group_counts.len(),
        "relation_count": relation_counts.len(),
        "entity_count": entity_counts.len(),
        "route_cap": route_cap,
        "top_route_share": round4(top_route_share),
        "top_hub_share": round4(top_hub_share),
        "duplicate_structural_facts": duplicate_count,
        "current_duplicate_groups": current_duplicates,
        "query_triads": query.len(),
        "query_text_present": !query_text.trim().is_empty(),
        "warnings": warnings,
        "notices": notices,
        "route_distribution": route_distribution,
        "group_distribution": group_distribution,
        "hub_nodes": hub_nodes,
        "duplicate_examples": duplicate_rows,
        "recommended_pipeline": [
            "dataset-doctor",
            "route-balanced focus packet",
            "search with candidate_triads",
            "field_interpretation",
            "split by route if contested",
            "check candidate route",
            "feedback accept/reject/WATCH"
        ]
    })
}

fn count_by<I>(values: I) -> BTreeMap<String, usize>
where
    I: IntoIterator<Item = String>,
{
    let mut counts = BTreeMap::new();
    for value in values {
        if value.is_empty() {
            continue;
        }
        *counts.entry(value).or_default() += 1;
    }
    counts
}

fn distribution_rows(counts: &BTreeMap<String, usize>, total: usize, limit: usize) -> Vec<Value> {
    let mut rows = counts
        .iter()
        .map(|(name, count)| (name.clone(), *count))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter()
        .take(limit)
        .map(|(name, count)| {
            json!({
                "name": name,
                "count": count,
                "share": round4(count as f64 / total.max(1) as f64)
            })
        })
        .collect()
}

fn hub_rows(counts: &BTreeMap<String, usize>, triad_count: usize, limit: usize) -> Vec<Value> {
    let total_slots = triad_count.max(1) * 2;
    let mut rows = counts
        .iter()
        .map(|(name, count)| (name.clone(), *count))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter()
        .take(limit)
        .map(|(name, count)| {
            json!({
                "entity": name,
                "count": count,
                "share": round4(count as f64 / total_slots as f64)
            })
        })
        .collect()
}

fn duplicate_rows(memory: &[Triad], limit: usize) -> Vec<Value> {
    let mut groups: BTreeMap<(String, String, String), Vec<&Triad>> = BTreeMap::new();
    for triad in memory {
        groups.entry(structural_key(triad)).or_default().push(triad);
    }
    let mut rows = groups
        .into_iter()
        .filter(|(_, items)| items.len() > 1)
        .map(|((subject, relation, object), items)| {
            let evidence_refs = items
                .iter()
                .map(|triad| triad.evidence.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            json!({
                "subject": subject,
                "relation": relation,
                "object": object,
                "count": items.len(),
                "routes": items.iter().map(|triad| route_name(triad, "memory-route")).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>(),
                "evidence_refs": evidence_refs
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["count"].as_u64().unwrap_or(0))
            .then_with(|| {
                a["subject"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["subject"].as_str().unwrap_or(""))
            })
    });
    rows.truncate(limit);
    rows
}

fn print_dataset_doctor_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("verdict: {}", out["verdict"].as_str().unwrap_or(""));
    println!("triads: {}", out["triad_count"]);
    println!("routes: {}", out["route_count"]);
    println!("top_route_share: {}", out["top_route_share"]);
    println!("top_hub_share: {}", out["top_hub_share"]);
    println!(
        "duplicate_structural_facts: {}",
        out["duplicate_structural_facts"]
    );
    if let Some(warnings) = out["warnings"].as_array() {
        for warning in warnings {
            println!(
                "warning: {} - {}",
                warning["kind"].as_str().unwrap_or("warning"),
                warning["message"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_dataset_doctor_md(out: &Value) {
    println!("# NANDA Dataset Doctor\n");
    println!("- core: `{}`", out["core_version"].as_str().unwrap_or(""));
    println!("- verdict: `{}`", out["verdict"].as_str().unwrap_or(""));
    println!("- triads: `{}`", out["triad_count"]);
    println!("- routes: `{}`", out["route_count"]);
    println!("- top_route_share: `{}`", out["top_route_share"]);
    println!("- top_hub_share: `{}`", out["top_hub_share"]);
    println!(
        "- duplicate_structural_facts: `{}`",
        out["duplicate_structural_facts"]
    );
    if let Some(warnings) = out["warnings"].as_array() {
        if !warnings.is_empty() {
            println!("\n## Warnings\n");
            for warning in warnings {
                println!(
                    "- `{}`: {}",
                    warning["kind"].as_str().unwrap_or("warning"),
                    warning["message"].as_str().unwrap_or("")
                );
            }
        }
    }
}

fn field_interpretation(peaks: &[Value], margin: f64, lexical_baseline: &Value) -> Value {
    if peaks.is_empty() {
        return json!({
            "state": "NO_FIELD",
            "read_as": "No resonance field was produced."
        });
    }
    let top = &peaks[0];
    let second = peaks.get(1);
    let top_name = top["peak"].as_str().unwrap_or("");
    let second_name = second.and_then(|peak| peak["peak"].as_str()).unwrap_or("");
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let top_polarization = top["polarization"]["state"].as_str().unwrap_or("");
    let stability = if top_polarization == "REVERSED" {
        "polarity_reversed"
    } else if margin >= 0.055 && component_gap >= 0.12 {
        "stable"
    } else if margin < 0.04 {
        "contested"
    } else {
        "thin"
    };
    let top_center = &top["center"];
    let second_center = second.map(|peak| &peak["center"]);
    let centroid_drift = json!({
        "from_second_peak": second_name,
        "route": center_pair(second_center, top_center, "route"),
        "relation": center_pair(second_center, top_center, "relation"),
        "entity": center_pair(second_center, top_center, "entity"),
        "subject_role": center_pair(second_center, top_center, "subject_role"),
        "object_role": center_pair(second_center, top_center, "object_role")
    });
    let nearest_foreign_pull = top["anti_triads"]
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or_else(|| json!(null));
    let lexical_trap = !lexical_peak.is_empty() && lexical_peak != top_name;
    json!({
        "state": stability,
        "top_peak": top_name,
        "second_peak": second_name,
        "margin": round4(margin),
        "component_gap": component_gap,
        "lexical_baseline_top": lexical_peak,
        "lexical_trap_detected": lexical_trap,
        "top_polarization": top_polarization,
        "centroid_drift": centroid_drift,
        "nearest_foreign_pull": nearest_foreign_pull,
        "read_as": if lexical_trap {
            "The structural field beats the lexical baseline; inspect support and anti-triads before final prose."
        } else if stability == "polarity_reversed" {
            "The top route has reversed role-direction polarity; do not use it as an answer route."
        } else if stability == "stable" {
            "The top route has a stable connected peak."
        } else {
            "The field is useful as retrieval context but is not a final answer skeleton."
        }
    })
}

fn coarse_to_fine_trace(peaks: &[Value], query_terms: &BTreeSet<String>) -> Value {
    let Some(top) = peaks.first() else {
        return json!({
            "enabled": true,
            "state": "NO_PEAK",
            "coarse_peak": "",
            "local_path": []
        });
    };
    let coarse_peak = top["peak"].as_str().unwrap_or("");
    let support = top["supporting_triads"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let local_path = support
        .iter()
        .take(5)
        .map(|item| {
            let subject = item["subject"].as_str().unwrap_or("");
            let relation = item["relation"].as_str().unwrap_or("");
            let object = item["object"].as_str().unwrap_or("");
            let mut hits = vec![];
            for term in query_terms {
                let haystack = norm(&format!("{subject} {relation} {object}"));
                if haystack.contains(term) {
                    hits.push(term.clone());
                }
            }
            json!({
                "triad": item["id"].as_str().unwrap_or(""),
                "edge": format!("{subject} -> {relation} -> {object}"),
                "score": item["score"].as_f64().unwrap_or(0.0),
                "polarity": item["polarity"].as_str().unwrap_or(""),
                "query_hits": hits
            })
        })
        .collect::<Vec<_>>();
    json!({
        "enabled": true,
        "state": if local_path.is_empty() { "THIN" } else { "LOCALIZED" },
        "coarse_peak": coarse_peak,
        "local_memory_size": support.len(),
        "local_path": local_path,
        "read_as": "Coarse route first, then inspect the local supporting path inside that route."
    })
}

fn center_pair(second_center: Option<&Value>, top_center: &Value, key: &str) -> Value {
    json!({
        "from": second_center
            .and_then(|center| center[key].as_str())
            .unwrap_or(""),
        "to": top_center[key].as_str().unwrap_or(""),
        "changed": second_center
            .and_then(|center| center[key].as_str())
            .unwrap_or("") != top_center[key].as_str().unwrap_or("")
    })
}

fn field_state_machine(
    peaks: &[Value],
    margin: f64,
    lexical_baseline: &Value,
    corpus: &Value,
    route_balanced_focus: &Value,
    coarse_to_fine: &Value,
) -> Value {
    if peaks.is_empty() {
        return json!({
            "state": "NO_FIELD",
            "safe_to_answer": false,
            "action": "NO_ANSWER",
            "blocking": ["no_peak"],
            "signals": {
                "margin": round4(margin),
                "component_gap": 0.0,
                "top_polarization": "",
                "corpus_verdict": corpus["verdict"].as_str().unwrap_or(""),
                "route_balanced": route_balanced_focus["enabled"].as_bool().unwrap_or(false),
                "coarse_to_fine": coarse_to_fine["state"].as_str().unwrap_or("")
            },
            "read_as": "No resonance field was produced."
        });
    }

    let top = &peaks[0];
    let second = peaks.get(1);
    let top_name = top["peak"].as_str().unwrap_or("");
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let top_polarization = top["polarization"]["state"].as_str().unwrap_or("");
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let route_balanced = route_balanced_focus["enabled"].as_bool().unwrap_or(false);
    let ctf_state = coarse_to_fine["state"].as_str().unwrap_or("");
    let corpus_verdict = corpus["verdict"].as_str().unwrap_or("");
    let warnings = corpus["warnings"].as_array().cloned().unwrap_or_default();
    let warning_kinds = warnings
        .iter()
        .filter_map(|warning| warning["kind"].as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let noisy_warning_count = warning_kinds
        .iter()
        .filter(|kind| {
            matches!(
                kind.as_str(),
                "large_unbalanced_corpus"
                    | "route_imbalance"
                    | "hub_dominance"
                    | "duplicate_current"
            )
        })
        .count();
    let weak_text_query = warning_kinds.iter().any(|kind| kind == "weak_text_query");
    let corpus_noisy = corpus_verdict == "WATCH" && (noisy_warning_count > 0 || weak_text_query);
    let focused = margin >= 0.055 && component_gap >= 0.12 && ctf_state == "LOCALIZED";
    let lexical_trap = !lexical_peak.is_empty() && lexical_peak != top_name;

    let mut blocking: Vec<String> = vec![];
    let (state, safe_to_answer, action, read_as) = if top_polarization == "REVERSED" {
        blocking.push("polarity_reversed".to_string());
        (
            "FIELD_REVERSED",
            false,
            "STOP_REPAIR_POLARITY",
            "The top peak is role-direction reversed; do not read it as the answer route.",
        )
    } else if corpus_noisy && !route_balanced {
        blocking.extend(warning_kinds.iter().cloned());
        (
            "FIELD_NOISY",
            false,
            "FOCUS_CORPUS",
            "The corpus field is noisy; run dataset-doctor or route-balanced focus before trusting the peak.",
        )
    } else if margin < 0.04 {
        blocking.push("low_margin".to_string());
        (
            "FIELD_CONTESTED",
            false,
            "SPLIT_OR_QUERY",
            "The top peaks are too close; use the result as retrieval context and split or sharpen the query.",
        )
    } else if !focused {
        if component_gap < 0.12 {
            blocking.push("weak_component_gap".to_string());
        }
        if ctf_state != "LOCALIZED" {
            blocking.push("not_localized".to_string());
        }
        (
            "FIELD_THIN",
            false,
            "USE_AS_HINT",
            "The peak is plausible but not connected/localized enough to become an answer skeleton.",
        )
    } else if route_balanced {
        (
            "FIELD_ROUTE_BALANCED",
            true,
            "ANSWER_WITH_BALANCED_SUPPORT",
            "The peak is focused after route-balanced filtering; answer from support and mention the focused packet.",
        )
    } else if lexical_trap {
        (
            "FIELD_FOCUSED",
            true,
            "ANSWER_WITH_SUPPORT",
            "The structural field beats the lexical baseline and is focused enough to draft from support.",
        )
    } else {
        (
            "FIELD_SAFE",
            true,
            "ANSWER_WITH_SUPPORT",
            "The field is focused, localized, and not blocked by corpus or polarity warnings.",
        )
    };

    blocking.sort();
    blocking.dedup();

    json!({
        "state": state,
        "safe_to_answer": safe_to_answer,
        "action": action,
        "top_peak": top_name,
        "blocking": blocking,
        "signals": {
            "margin": round4(margin),
            "component_gap": component_gap,
            "top_polarization": top_polarization,
            "corpus_verdict": corpus_verdict,
            "corpus_warnings": warning_kinds,
            "route_balanced": route_balanced,
            "coarse_to_fine": ctf_state,
            "lexical_baseline_top": lexical_peak,
            "lexical_trap_detected": lexical_trap
        },
        "read_as": read_as
    })
}

fn search_verdict(field_state: &str, safe_to_answer: bool) -> &'static str {
    match field_state {
        "FIELD_REVERSED" => "VETO",
        "FIELD_FOCUSED" | "FIELD_SAFE" | "FIELD_ROUTE_BALANCED" if safe_to_answer => "PASS",
        _ => "WATCH",
    }
}

fn peak_decision(peaks: &[Value], margin: f64, lexical_peak: &str) -> Value {
    if peaks.is_empty() {
        return json!({
            "state": "NO_PEAK",
            "safe_to_answer": false,
            "reason": "No route/group peak was produced."
        });
    }
    let top = &peaks[0];
    let second = peaks.get(1);
    let top_name = top["peak"].as_str().unwrap_or("");
    let top_polarization = top["polarization"]["state"].as_str().unwrap_or("");
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let wins_lexical = !lexical_peak.is_empty() && top_name != lexical_peak;
    let (state, safe_to_answer, reason) = if top_polarization == "REVERSED" {
        (
            "POLARITY_REVERSED",
            false,
            "Top peak has reversed role-direction polarity relative to the query.",
        )
    } else if margin >= 0.055 && component_gap >= 0.12 {
        (
            "FOCUSED",
            true,
            "Top peak has enough margin and stronger connected component than the nearest rival.",
        )
    } else if margin < 0.04 {
        (
            "WATCH",
            false,
            "Top peak is close to the nearest rival; use as retrieval hint, not final structure.",
        )
    } else if component_gap < 0.0 {
        (
            "AMBIGUOUS",
            false,
            "Nearest rival has stronger connected component; inspect support and anti-triads.",
        )
    } else {
        (
            "WATCH",
            false,
            "Top peak is plausible but not focused enough for a confident structural answer.",
        )
    };
    json!({
        "state": state,
        "safe_to_answer": safe_to_answer,
        "top_peak": top_name,
        "lexical_baseline_top": lexical_peak,
        "wins_over_lexical_baseline": wins_lexical,
        "top_polarization": top_polarization,
        "margin": round4(margin),
        "top_component_score": round4(top_component),
        "second_component_score": round4(second_component),
        "component_gap": component_gap,
        "reason": reason
    })
}

fn symbolic_peak_baseline(items: &[(f64, f64, f64, &Triad)]) -> Value {
    let max_symbolic = items
        .iter()
        .map(|(_, _, symbolic, _)| *symbolic)
        .fold(0.0, f64::max);
    let avg_top3 = if items.is_empty() {
        0.0
    } else {
        let take = items.len().min(3);
        items
            .iter()
            .take(take)
            .map(|(_, _, symbolic, _)| *symbolic)
            .sum::<f64>()
            / take as f64
    };
    json!({
        "max_symbolic_overlap": round4(max_symbolic),
        "avg_top3_symbolic_overlap": round4(avg_top3)
    })
}

fn lexical_baseline(
    scored: &[(f64, f64, f64, &Triad)],
    query: &[Triad],
    group_by: &PeakGroupBy,
) -> Value {
    let query_tokens = query_token_set(query);
    let mut by_group: BTreeMap<String, Vec<(f64, &Triad)>> = BTreeMap::new();
    for (_, _, _, triad) in scored {
        let key = match group_by {
            PeakGroupBy::Group => group_name(triad, "memory"),
            PeakGroupBy::Route => route_name(triad, "memory-route"),
        };
        by_group
            .entry(key)
            .or_default()
            .push((token_overlap(&query_tokens, triad), *triad));
    }
    let mut rows = vec![];
    for (key, mut items) in by_group {
        items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let max = items.first().map(|item| item.0).unwrap_or(0.0);
        let take = items.len().min(3);
        let avg_top3 = if take == 0 {
            0.0
        } else {
            items.iter().take(take).map(|item| item.0).sum::<f64>() / take as f64
        };
        rows.push(json!({
            "peak": key,
            "score": round4(max),
            "max_symbolic_overlap": round4(max),
            "avg_top3_symbolic_overlap": round4(avg_top3),
            "top_triads": items.iter().take(3).map(|(_, triad)| triad.id.clone()).collect::<Vec<_>>()
        }));
    }
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    json!({
        "top_peak": rows.first().and_then(|row| row["peak"].as_str()).unwrap_or(""),
        "scores": rows
    })
}

fn query_token_set(query: &[Triad]) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    for triad in query {
        tokens.extend(triad_tokens(triad));
    }
    tokens
}

fn triad_tokens(triad: &Triad) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    for value in [
        &triad.subject,
        &triad.relation,
        &triad.object,
        &triad.subject_role,
        &triad.object_role,
        &triad.route,
        &triad.group,
    ] {
        for token in norm(value)
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|token| token.len() >= 2)
        {
            if token != "query" {
                tokens.insert(token.to_string());
            }
        }
    }
    tokens
}

fn token_overlap(query_tokens: &BTreeSet<String>, triad: &Triad) -> f64 {
    if query_tokens.is_empty() {
        return 0.0;
    }
    let triad_tokens = triad_tokens(triad);
    query_tokens.intersection(&triad_tokens).count() as f64 / query_tokens.len() as f64
}

fn route_term_coverage(query_terms: &BTreeSet<String>, items: &[(f64, f64, f64, &Triad)]) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let mut peak_terms = BTreeSet::new();
    for (_, _, _, triad) in items {
        peak_terms.extend(triad_term_set(triad));
    }
    query_terms.intersection(&peak_terms).count() as f64 / query_terms.len() as f64
}

fn chain_coherence(items: &[(f64, f64, f64, &Triad)]) -> f64 {
    if items.len() < 2 {
        return 0.0;
    }
    let mut links = 0usize;
    let mut possible = 0usize;
    for (_, _, _, left) in items {
        for (_, _, _, right) in items {
            if left.id == right.id {
                continue;
            }
            possible += 1;
            let left_subject = norm(&left.subject);
            let left_object = norm(&left.object);
            let right_subject = norm(&right.subject);
            let right_object = norm(&right.object);
            if (!left_object.is_empty() && left_object == right_subject)
                || (!left_subject.is_empty() && left_subject == right_object)
            {
                links += 1;
            }
        }
    }
    if possible == 0 {
        0.0
    } else {
        (links as f64 / possible as f64).min(1.0)
    }
}

fn propagation_summary(query_terms: &BTreeSet<String>, items: &[(f64, f64, f64, &Triad)]) -> Value {
    if items.is_empty() {
        return json!({"score": 0.0, "component_score": 0.0, "links": [], "components": [], "activated_triads": []});
    }
    let mut activation: Vec<f64> = items
        .iter()
        .map(|(combined, _, _, _)| combined.max(0.0))
        .collect();
    let links = propagation_links(items);
    for _ in 0..2 {
        let mut next = activation.clone();
        for idx in 0..items.len() {
            let neighbors = links
                .iter()
                .filter_map(|(left, right)| {
                    if *left == idx {
                        Some(*right)
                    } else if *right == idx {
                        Some(*left)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if neighbors.is_empty() {
                continue;
            }
            let neighbor_energy = neighbors
                .iter()
                .map(|neighbor| activation[*neighbor])
                .sum::<f64>()
                / neighbors.len() as f64;
            next[idx] = (0.72 * activation[idx]) + (0.28 * neighbor_energy);
        }
        activation = next;
    }
    let mut ranked = activation
        .iter()
        .enumerate()
        .map(|(idx, value)| (idx, *value))
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let take = ranked.len().min(3);
    let score = if take == 0 {
        0.0
    } else {
        ranked
            .iter()
            .take(take)
            .map(|(_, value)| *value)
            .sum::<f64>()
            / take as f64
    };
    let components = connected_components(items.len(), &links);
    let mut component_rows = vec![];
    let mut component_score = 0.0_f64;
    for component in components {
        let mut terms = BTreeSet::new();
        let mut energy = 0.0;
        for idx in &component {
            terms.extend(triad_term_set(items[*idx].3));
            energy += activation[*idx];
        }
        let coverage = if query_terms.is_empty() {
            0.0
        } else {
            query_terms.intersection(&terms).count() as f64 / query_terms.len() as f64
        };
        let size_ratio = component.len() as f64 / items.len() as f64;
        let avg_energy = if component.is_empty() {
            0.0
        } else {
            energy / component.len() as f64
        };
        let score = (0.62 * coverage) + (0.23 * size_ratio) + (0.15 * avg_energy);
        component_score = component_score.max(score);
        component_rows.push(json!({
            "score": round4(score),
            "coverage": round4(coverage),
            "size": component.len(),
            "triads": component.iter().map(|idx| items[*idx].3.id.clone()).collect::<Vec<_>>()
        }));
    }
    component_rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    json!({
        "score": round4(score),
        "component_score": round4(component_score),
        "links": links.iter().map(|(left, right)| {
            json!({
                "from": items[*left].3.id,
                "to": items[*right].3.id,
                "via": shared_endpoint(items[*left].3, items[*right].3)
            })
        }).collect::<Vec<_>>(),
        "components": component_rows,
        "activated_triads": ranked.iter().take(5).map(|(idx, value)| {
            json!({
                "id": items[*idx].3.id,
                "activation": round4(*value),
                "base_score": round4(items[*idx].0)
            })
        }).collect::<Vec<_>>()
    })
}

fn connected_components(size: usize, links: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::<usize>::new(); size];
    for (left, right) in links {
        adjacency[*left].push(*right);
        adjacency[*right].push(*left);
    }
    let mut seen = vec![false; size];
    let mut components = vec![];
    for start in 0..size {
        if seen[start] {
            continue;
        }
        let mut stack = vec![start];
        let mut component = vec![];
        seen[start] = true;
        while let Some(idx) = stack.pop() {
            component.push(idx);
            for next in &adjacency[idx] {
                if !seen[*next] {
                    seen[*next] = true;
                    stack.push(*next);
                }
            }
        }
        components.push(component);
    }
    components
}

fn propagation_links(items: &[(f64, f64, f64, &Triad)]) -> Vec<(usize, usize)> {
    let mut links = vec![];
    for left in 0..items.len() {
        for right in (left + 1)..items.len() {
            if !shared_endpoint(items[left].3, items[right].3).is_empty() {
                links.push((left, right));
            }
        }
    }
    links
}

fn shared_endpoint(left: &Triad, right: &Triad) -> String {
    let left_values = [norm(&left.subject), norm(&left.object)];
    let right_values = [norm(&right.subject), norm(&right.object)];
    for left_value in &left_values {
        if left_value.is_empty() {
            continue;
        }
        for right_value in &right_values {
            if left_value == right_value {
                return left_value.clone();
            }
        }
    }
    String::new()
}

fn triad_polarity(triad: &Triad) -> String {
    format!(
        "{}->{}->{}",
        role_family(&triad.subject_role),
        relation_family(&triad.relation),
        role_family(&triad.object_role)
    )
}

fn reversed_polarity(triad: &Triad) -> String {
    format!(
        "{}->{}->{}",
        role_family(&triad.object_role),
        relation_family(&triad.relation),
        role_family(&triad.subject_role)
    )
}

fn role_family(role: &str) -> String {
    let role = norm(role);
    if role.contains("payer") || role.contains("buyer") || role.contains("customer") {
        "payer".to_string()
    } else if role.contains("supplier") || role.contains("seller") || role.contains("factory") {
        "supplier".to_string()
    } else if role.contains("document") || role.contains("certificate") || role.contains("doc") {
        "document".to_string()
    } else if role.contains("route") || role.contains("path") {
        "route".to_string()
    } else if role.contains("owner") || role.contains("holder") {
        "owner".to_string()
    } else if role.contains("asset") || role.contains("goods") || role.contains("product") {
        "asset".to_string()
    } else if role.is_empty() {
        "role".to_string()
    } else {
        role
    }
}

fn relation_family(relation: &str) -> String {
    let relation = norm(relation);
    if relation.contains("pay") || relation.contains("fund") || relation.contains("owe") {
        "payment".to_string()
    } else if relation.contains("own") || relation.contains("hold") {
        "ownership".to_string()
    } else if relation.contains("supply")
        || relation.contains("deliver")
        || relation.contains("ship")
    {
        "flow".to_string()
    } else if relation.contains("require")
        || relation.contains("confirm")
        || relation.contains("cert")
    {
        "evidence".to_string()
    } else if relation.is_empty() {
        "relation".to_string()
    } else {
        relation
    }
}

fn polarization_summary(query: &[Triad], items: &[(f64, f64, f64, &Triad)]) -> Value {
    let query_polarities = query
        .iter()
        .map(triad_polarity)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let query_reversed = query
        .iter()
        .map(reversed_polarity)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if query_polarities.is_empty() || items.is_empty() {
        return json!({
            "state": "NO_QUERY_POLARITY",
            "aligned": 0,
            "reversed": 0,
            "dominant": ""
        });
    }
    let mut counts = BTreeMap::<String, usize>::new();
    let mut aligned = 0usize;
    let mut reversed = 0usize;
    for (_, _, _, triad) in items {
        let polarity = triad_polarity(triad);
        if query_polarities.contains(&polarity) {
            aligned += 1;
        }
        if query_reversed.contains(&polarity) && !query_polarities.contains(&polarity) {
            reversed += 1;
        }
        *counts.entry(polarity).or_default() += 1;
    }
    let dominant = counts
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(polarity, _)| polarity.clone())
        .unwrap_or_default();
    let state = if aligned > 0 && reversed == 0 {
        "ALIGNED"
    } else if reversed > aligned {
        "REVERSED"
    } else if aligned > 0 {
        "MIXED"
    } else {
        "UNALIGNED"
    };
    json!({
        "state": state,
        "aligned": aligned,
        "reversed": reversed,
        "dominant": dominant,
        "query": query_polarities.into_iter().collect::<Vec<_>>()
    })
}

fn polarization_penalty(polarization: &Value) -> f64 {
    match polarization["state"].as_str().unwrap_or("") {
        "REVERSED" => 0.18,
        "MIXED" => 0.04,
        "UNALIGNED" => 0.02,
        _ => 0.0,
    }
}

fn query_feature_wave(query: &[Triad]) -> Vec<i32> {
    let mut wave = vec![0; WAVE_DIM];
    for triad in query {
        add_into(&mut wave, &partial_triad_feature_wave(triad));
    }
    wave
}

fn triad_feature_wave(triad: &Triad) -> Vec<i32> {
    let mut wave = partial_triad_feature_wave(triad);
    add_feature(&mut wave, "group", &triad.group);
    add_feature(&mut wave, "route", &triad.route);
    add_feature(&mut wave, "subject_role", &triad.subject_role);
    add_feature(&mut wave, "object_role", &triad.object_role);
    add_feature(&mut wave, "polarity", &triad_polarity(triad));
    wave
}

fn partial_triad_feature_wave(triad: &Triad) -> Vec<i32> {
    let mut wave = vec![0; WAVE_DIM];
    add_feature(&mut wave, "subject", &triad.subject);
    add_feature(&mut wave, "relation", &triad.relation);
    add_feature(&mut wave, "object", &triad.object);
    add_feature(&mut wave, "subject_role", &triad.subject_role);
    add_feature(&mut wave, "object_role", &triad.object_role);
    add_feature(&mut wave, "route", &triad.route);
    add_feature(&mut wave, "group", &triad.group);
    add_feature(&mut wave, "polarity", &triad_polarity(triad));
    wave
}

fn add_feature(wave: &mut [i32], slot: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    let feature = bind(
        &vector(&format!("slot:{}", norm(slot))),
        &vector(&format!("value:{}", norm(value))),
    );
    add_into(wave, &feature);
}

fn symbolic_query_overlap(query: &[Triad], triad: &Triad) -> f64 {
    let q = query_term_set(query);
    if q.is_empty() {
        return 0.0;
    }
    let t = triad_term_set(triad);
    let hits = q.intersection(&t).count() as f64;
    hits / q.len() as f64
}

fn query_term_set(query: &[Triad]) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for triad in query {
        terms.extend(triad_term_set(triad));
    }
    terms
}

fn triad_term_set(triad: &Triad) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for value in [
        &triad.subject,
        &triad.relation,
        &triad.object,
        &triad.subject_role,
        &triad.object_role,
        &triad.route,
        &triad.group,
    ] {
        let normalized = norm(value);
        if !normalized.is_empty() && normalized != "subject" && normalized != "object" {
            terms.insert(normalized);
        }
    }
    terms
}

fn peak_center(items: &[(f64, f64, f64, &Triad)]) -> Value {
    json!({
        "entity": weighted_center(items.iter().flat_map(|(score, _, _, triad)| [(*score, norm(&triad.subject)), (*score, norm(&triad.object))])),
        "relation": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, norm(&triad.relation)))),
        "route": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, route_name(triad, "memory-route")))),
        "group": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, group_name(triad, "memory")))),
        "subject_role": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, norm(&triad.subject_role)))),
        "object_role": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, norm(&triad.object_role))))
    })
}

fn weighted_center<I>(values: I) -> String
where
    I: IntoIterator<Item = (f64, String)>,
{
    let mut weights: BTreeMap<String, f64> = BTreeMap::new();
    for (score, value) in values {
        if value.is_empty() {
            continue;
        }
        *weights.entry(value).or_default() += score.max(0.0);
    }
    weights
        .into_iter()
        .max_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.cmp(&a.0))
        })
        .map(|(value, _)| value)
        .unwrap_or_default()
}

fn missing_edges(query_terms: &BTreeSet<String>, items: &[(f64, f64, f64, &Triad)]) -> Vec<Value> {
    let mut peak_terms = BTreeSet::new();
    for (_, _, _, triad) in items {
        peak_terms.extend(triad_term_set(triad));
    }
    query_terms
        .difference(&peak_terms)
        .map(|term| {
            json!({
                "term": term,
                "suggested_fix": "Add or retrieve evidence that binds this query term to the peak route."
            })
        })
        .collect()
}

fn answer_projection(center: &Value, items: &[(f64, f64, f64, &Triad)]) -> Value {
    let top = items
        .iter()
        .take(5)
        .map(|(_, _, _, triad)| {
            format!(
                "{} -> {} -> {}",
                triad.subject, triad.relation, triad.object
            )
        })
        .collect::<Vec<_>>();
    json!({
        "dominant_route": center["route"],
        "dominant_group": center["group"],
        "dominant_relation": center["relation"],
        "read_as": "Use this peak as a candidate structural route, not as proof by itself.",
        "top_structure": top
    })
}

fn triad_json(triad: &Triad) -> Value {
    json!({
        "id": triad.id,
        "subject": triad.subject,
        "relation": triad.relation,
        "object": triad.object,
        "confidence": triad.confidence,
        "subject_role": triad.subject_role,
        "object_role": triad.object_role,
        "route": triad.route,
        "group": triad.group,
        "evidence": triad.evidence
    })
}

fn dogfood_cmd(args: DogfoodArgs) -> Result<u8> {
    let input = resolve_dogfood_input(&args.input)?;
    let packet = load_packet_auto(
        &input,
        &args.input_format,
        "dogfood",
        "code",
        "Self-check repository structure.",
        args.normalize_paths,
    )?;
    let triads = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let branch_by = parse_csv(&args.branch_by);
    let stop_on = parse_csv(&args.stop_on);
    let topology = topology(&triads, &candidates);
    let comb_tree = comb_node(
        "root",
        0,
        args.depth,
        &branch_by,
        &stop_on,
        args.max_branches,
        &packet,
        &triads,
        &candidates,
    )?;
    let summary = comb_summary(&comb_tree);
    let decision = dogfood_decision(&comb_tree, &summary);
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "dogfood",
        "input": input,
        "depth": args.depth,
        "branch_by": branch_by,
        "stop_on": stop_on,
        "topology": topology,
        "comb_tree": comb_tree,
        "summary": summary,
        "agent_decision": decision
    });
    if let Some(out_dir) = args.out_dir {
        fs::create_dir_all(&out_dir)?;
        fs::write(
            out_dir.join("dogfood.json"),
            serde_json::to_string_pretty(&out)? + "\n",
        )?;
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_dogfood_text(&out),
        OutputFormat::Md => print_dogfood_md(&out),
    }
    match out["agent_decision"]["action"].as_str().unwrap_or("") {
        "SAFE_TO_EDIT" => Ok(EXIT_PASS),
        "SPLIT_REQUIRED" | "REVIEW_REQUIRED" => Ok(EXIT_WATCH),
        "REPAIR_REQUIRED" => Ok(EXIT_VETO),
        _ => Ok(EXIT_ERROR),
    }
}

fn resolve_dogfood_input(input: &Path) -> Result<PathBuf> {
    if input.is_file() {
        return Ok(input.to_path_buf());
    }
    if input.is_dir() {
        for candidate in [
            input.join("examples/self-dogfood.nanda.json"),
            input.join("self-dogfood.nanda.json"),
            input.join(".nanda/self-dogfood.nanda.json"),
        ] {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(anyhow!(
        "dogfood input not found: pass a triad packet or a repo containing examples/self-dogfood.nanda.json"
    ))
}

fn dogfood_decision(tree: &Value, summary: &Value) -> Value {
    let root_verdict = tree["verdict"].as_str().unwrap_or("WATCH");
    let children = tree["children"].as_array().cloned().unwrap_or_default();
    let child_count = children.len();
    let local_pass = children
        .iter()
        .filter(|child| child["verdict"].as_str() == Some("PASS"))
        .count();
    let local_watch = children
        .iter()
        .filter(|child| child["verdict"].as_str() == Some("WATCH"))
        .count();
    let local_veto = children
        .iter()
        .filter(|child| child["verdict"].as_str() == Some("VETO"))
        .count();
    let foreign_pull = summary["foreign_pull"].as_u64().unwrap_or(0);
    let invariant_violation = summary["invariant_violation"].as_u64().unwrap_or(0);
    let root_stop = tree["stop_reasons"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let root_size_only = root_stop.iter().all(|item| item == "size")
        && !tree["limits"]
            .as_array()
            .map(|items| items.is_empty())
            .unwrap_or(true);

    let (action, next) = if foreign_pull > 0 || invariant_violation > 0 || local_veto > 0 {
        (
            "REPAIR_REQUIRED",
            "Repair foreign pull, invariant drift, or vetoed branch before editing.",
        )
    } else if child_count > 0
        && local_pass == child_count
        && root_verdict == "WATCH"
        && root_size_only
    {
        (
            "SAFE_TO_EDIT",
            "Root is size-only WATCH; use linked branch PASS results as acceptance.",
        )
    } else if child_count > 0 && local_pass == child_count && root_verdict == "PASS" {
        (
            "SAFE_TO_EDIT",
            "Root and linked branches are structurally clean.",
        )
    } else if root_verdict == "WATCH" {
        (
            "SPLIT_REQUIRED",
            "Split or narrow unresolved WATCH branches before finalizing.",
        )
    } else {
        (
            "REVIEW_REQUIRED",
            "Review the comb tree before trusting the structure.",
        )
    };

    json!({
        "action": action,
        "root_verdict": root_verdict,
        "root_size_only": root_size_only,
        "safe_to_edit": action == "SAFE_TO_EDIT",
        "local_branches": child_count,
        "local_pass": local_pass,
        "local_watch": local_watch,
        "local_veto": local_veto,
        "foreign_pull": foreign_pull,
        "invariant_violation": invariant_violation,
        "next": next
    })
}

fn parse_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn topology(source: &[Triad], candidates: &[Triad]) -> Value {
    let mut nodes = BTreeSet::new();
    let mut edges = vec![];
    let mut groups = BTreeSet::new();
    let mut routes = BTreeSet::new();
    for (kind, triads) in [("source", source), ("candidate", candidates)] {
        for triad in triads {
            nodes.insert(triad.subject.clone());
            nodes.insert(triad.object.clone());
            groups.insert(group_name(triad, kind));
            routes.insert(route_name(triad, &format!("{kind}-route")));
            edges.push(json!({
                "id": triad.id,
                "kind": kind,
                "subject": triad.subject,
                "relation": triad.relation,
                "object": triad.object,
                "subject_role": triad.subject_role,
                "object_role": triad.object_role,
                "route": triad.route,
                "group": triad.group,
                "evidence": triad.evidence
            }));
        }
    }
    json!({
        "nodes": nodes.into_iter().collect::<Vec<_>>(),
        "edges": edges,
        "groups": groups.into_iter().collect::<Vec<_>>(),
        "routes": routes.into_iter().collect::<Vec<_>>()
    })
}

fn comb_node(
    path: &str,
    depth: usize,
    max_depth: usize,
    branch_by: &[String],
    stop_on: &[String],
    max_branches: usize,
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
) -> Result<Value> {
    let local_packet = Packet {
        task_id: format!("{}:{path}", packet.task_id),
        domain: packet.domain.clone(),
        query: packet.query.clone(),
        triads: source.to_vec(),
        candidate_triads: candidates.to_vec(),
        candidate_answer: packet.candidate_answer.clone(),
        negative_shortcuts: packet.negative_shortcuts.clone(),
        positive_shortcuts: packet.positive_shortcuts.clone(),
    };
    let report = make_report(&local_packet, source, candidates)?;
    let map = structural_map(source, candidates);
    let invariants = if depth >= 1 && branch_by.iter().any(|item| item == "subject-relation") {
        invariant_scan(source, candidates)
    } else {
        json!({"violations": [], "checked": []})
    };
    let stop_reasons = stop_reasons(&report, &map, &invariants, stop_on);
    let mut children = vec![];
    if depth == 0 && depth < max_depth && !stop_reasons.iter().any(|item| item == "foreign_pull") {
        if branch_by.iter().any(|item| item == "linked-group") {
            let splits = linked_group_splits(source, candidates);
            for split in splits.items.into_iter().take(max_branches) {
                children.push(comb_node(
                    &format!("{path}/linked-group:{}", split.key),
                    depth + 1,
                    max_depth,
                    branch_by,
                    stop_on,
                    max_branches,
                    packet,
                    &split.triads,
                    &split.candidates,
                )?);
            }
        } else if branch_by.iter().any(|item| item == "route") {
            let splits = raw_splits(source, candidates, &SplitBy::Route);
            for split in splits.items.into_iter().take(max_branches) {
                children.push(comb_node(
                    &format!("{path}/route:{}", split.key),
                    depth + 1,
                    max_depth,
                    branch_by,
                    stop_on,
                    max_branches,
                    packet,
                    &split.triads,
                    &split.candidates,
                )?);
            }
        }
    }
    Ok(json!({
        "path": path,
        "depth": depth,
        "verdict": report.verdict,
        "complexity_score": report.complexity_score,
        "limits": report.limits,
        "map": map,
        "invariants": invariants,
        "stop_reasons": stop_reasons,
        "children": children
    }))
}

fn invariant_scan(source: &[Triad], candidates: &[Triad]) -> Value {
    let mut by_key: BTreeMap<(String, String, String), Vec<&Triad>> = BTreeMap::new();
    let map = structural_map(source, candidates);
    let mut candidate_scope: BTreeMap<String, String> = BTreeMap::new();
    if let Some(group_links) = map["group_links"].as_object() {
        for (candidate_group, link) in group_links {
            if let Some(source_group) = link["dominant_source_group"].as_str() {
                if !source_group.is_empty() {
                    candidate_scope.insert(candidate_group.clone(), source_group.to_string());
                }
            }
        }
    }
    for triad in source {
        if !is_invariant_candidate(triad) {
            continue;
        }
        by_key
            .entry((
                group_name(triad, "source"),
                norm(&triad.subject),
                norm(&triad.relation),
            ))
            .or_default()
            .push(triad);
    }
    for triad in candidates {
        if !is_invariant_candidate(triad) {
            continue;
        }
        let candidate_group = group_name(triad, "candidate");
        let scope = candidate_scope
            .get(&candidate_group)
            .cloned()
            .unwrap_or(candidate_group);
        by_key
            .entry((scope, norm(&triad.subject), norm(&triad.relation)))
            .or_default()
            .push(triad);
    }
    let mut checked = vec![];
    let mut violations = vec![];
    for ((group, subject, relation), triads) in by_key {
        if triads.len() < 2 {
            continue;
        }
        let values: BTreeSet<String> = triads.iter().map(|triad| norm(&triad.object)).collect();
        checked.push(json!({
            "selector": {
                "group": group,
                "subject": subject,
                "relation": relation
            },
            "values": values.iter().cloned().collect::<Vec<_>>(),
            "count": triads.len()
        }));
        if values.len() > 1 {
            violations.push(json!({
                "kind": "same_value",
                "selector": {
                    "group": group,
                    "subject": subject,
                    "relation": relation
                },
                "values": values.iter().cloned().collect::<Vec<_>>(),
                "evidence": triads.iter().map(|triad| triad.evidence.clone()).collect::<Vec<_>>(),
                "triads": triads.iter().map(|triad| triad.id.clone()).collect::<Vec<_>>(),
                "message": "same group+subject+relation has multiple object values"
            }));
        }
    }
    json!({"checked": checked, "violations": violations})
}

fn is_invariant_candidate(triad: &Triad) -> bool {
    let relation = norm(&triad.relation);
    let object_role = norm(&triad.object_role);
    matches!(
        relation.as_str(),
        "default_value"
            | "value"
            | "type"
            | "schema"
            | "unit"
            | "formula"
            | "normalizes_to"
            | "version"
            | "rate"
            | "currency"
            | "owner"
            | "required"
    ) || matches!(
        object_role.as_str(),
        "value" | "type" | "schema" | "unit" | "formula" | "version" | "rate" | "currency"
    )
}

fn stop_reasons(
    report: &Report,
    map: &Value,
    invariants: &Value,
    stop_on: &[String],
) -> Vec<String> {
    let mut reasons = vec![];
    if stop_on.iter().any(|item| item == "foreign_pull")
        && map["foreign_pull"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    {
        reasons.push("foreign_pull".to_string());
    }
    if stop_on.iter().any(|item| item == "invariant_violation")
        && invariants["violations"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    {
        reasons.push("invariant_violation".to_string());
    }
    if stop_on.iter().any(|item| item == "size") && !report.limits.is_empty() {
        reasons.push("size".to_string());
    }
    reasons
}

fn comb_summary(tree: &Value) -> Value {
    let mut summary = BTreeMap::from([
        ("pass".to_string(), 0usize),
        ("watch".to_string(), 0usize),
        ("veto".to_string(), 0usize),
        ("invariant_violation".to_string(), 0usize),
        ("foreign_pull".to_string(), 0usize),
    ]);
    accumulate_comb_summary(tree, &mut summary);
    json!(summary)
}

fn accumulate_comb_summary(node: &Value, summary: &mut BTreeMap<String, usize>) {
    match node["verdict"].as_str().unwrap_or("") {
        "PASS" => *summary.entry("pass".to_string()).or_default() += 1,
        "WATCH" => *summary.entry("watch".to_string()).or_default() += 1,
        "VETO" => *summary.entry("veto".to_string()).or_default() += 1,
        _ => {}
    }
    if node["invariants"]["violations"]
        .as_array()
        .is_some_and(|items| !items.is_empty())
    {
        *summary
            .entry("invariant_violation".to_string())
            .or_default() += 1;
    }
    if node["map"]["foreign_pull"]
        .as_array()
        .is_some_and(|items| !items.is_empty())
    {
        *summary.entry("foreign_pull".to_string()).or_default() += 1;
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            accumulate_comb_summary(child, summary);
        }
    }
}

fn print_comb_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("depth: {}", out["depth"].as_u64().unwrap_or(0));
    println!("summary: {}", out["summary"]);
}

fn print_hgate_text(out: &Value) {
    let decision = &out["hierarchical_decision"];
    println!("NANDA HGATE");
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "ACTION: {}",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "GLOBAL: {}{}",
        decision["global_verdict"].as_str().unwrap_or("WATCH"),
        if decision["global_size_only"].as_bool().unwrap_or(false) {
            " size-only"
        } else {
            ""
        }
    );
    println!(
        "BRANCHES: {}/{} PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["branches"].as_u64().unwrap_or(0)
    );
    println!(
        "BLOCKERS: local_veto={} local_watch={} foreign_pull={} truncated={}",
        decision["local_veto"].as_u64().unwrap_or(0),
        decision["local_watch"].as_u64().unwrap_or(0),
        decision["global_foreign_pull"].as_u64().unwrap_or(0),
        decision["truncated_branches"].as_u64().unwrap_or(0)
    );
    println!(
        "ACCEPTED: {}",
        decision["structurally_accepted"].as_bool().unwrap_or(false)
    );
    println!("NEXT: {}", decision["next"].as_str().unwrap_or(""));
}

fn print_hgate_md(out: &Value) {
    let decision = &out["hierarchical_decision"];
    println!("# NANDA Hierarchical Gate\n");
    println!(
        "- action: `{}`",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "- global: `{}`",
        decision["global_verdict"].as_str().unwrap_or("WATCH")
    );
    println!(
        "- global_size_only: `{}`",
        decision["global_size_only"].as_bool().unwrap_or(false)
    );
    println!(
        "- branches: `{}/{}` PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["branches"].as_u64().unwrap_or(0)
    );
    println!(
        "- blockers: `local_veto={} local_watch={} foreign_pull={} truncated={}`",
        decision["local_veto"].as_u64().unwrap_or(0),
        decision["local_watch"].as_u64().unwrap_or(0),
        decision["global_foreign_pull"].as_u64().unwrap_or(0),
        decision["truncated_branches"].as_u64().unwrap_or(0)
    );
    println!(
        "- structurally_accepted: `{}`",
        decision["structurally_accepted"].as_bool().unwrap_or(false)
    );
    println!("- next: {}", decision["next"].as_str().unwrap_or(""));
}

fn print_comb_md(out: &Value) {
    println!("# NANDA Comb\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- depth: `{}`", out["depth"].as_u64().unwrap_or(0));
    println!("- summary: `{}`", out["summary"]);
}

fn print_search_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "mode: {}",
        out["mode"].as_str().unwrap_or("interference-retrieval")
    );
    println!("memory_size: {}", out["memory_size"].as_u64().unwrap_or(0));
    println!(
        "peak_state: {}",
        out["peak_decision"]["state"].as_str().unwrap_or("WATCH")
    );
    println!(
        "safe_to_answer: {}",
        out["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "peak_margin: {}",
        out["peak_margin"].as_f64().unwrap_or(0.0)
    );
    println!(
        "lexical_baseline_top: {}",
        out["lexical_baseline"]["top_peak"].as_str().unwrap_or("")
    );
    if let Some(peaks) = out["peaks"].as_array() {
        for (idx, peak) in peaks.iter().enumerate() {
            println!(
                "{}. peak={} score={} route={} group={}",
                idx + 1,
                peak["peak"].as_str().unwrap_or(""),
                peak["score"].as_f64().unwrap_or(0.0),
                peak["center"]["route"].as_str().unwrap_or(""),
                peak["center"]["group"].as_str().unwrap_or("")
            );
            if let Some(support) = peak["supporting_triads"].as_array() {
                for item in support.iter().take(3) {
                    println!(
                        "   + {}: {} -> {} -> {}",
                        item["id"].as_str().unwrap_or(""),
                        item["subject"].as_str().unwrap_or(""),
                        item["relation"].as_str().unwrap_or(""),
                        item["object"].as_str().unwrap_or("")
                    );
                }
            }
            if let Some(anti) = peak["anti_triads"].as_array() {
                for item in anti.iter().take(2) {
                    println!(
                        "   ~ foreign {}: {} -> {} -> {}",
                        item["id"].as_str().unwrap_or(""),
                        item["subject"].as_str().unwrap_or(""),
                        item["relation"].as_str().unwrap_or(""),
                        item["object"].as_str().unwrap_or("")
                    );
                }
            }
        }
    }
}

fn print_probe_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("decision: {}", out["decision"].as_str().unwrap_or(""));
    println!(
        "plain: top={} verdict={} field={} safe={} score={}",
        out["plain"]["top_peak"].as_str().unwrap_or(""),
        out["plain"]["verdict"].as_str().unwrap_or(""),
        out["plain"]["field_state"].as_str().unwrap_or(""),
        out["plain"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["plain"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "negative: top={} verdict={} field={} safe={} score={}",
        out["negative"]["top_peak"].as_str().unwrap_or(""),
        out["negative"]["verdict"].as_str().unwrap_or(""),
        out["negative"]["field_state"].as_str().unwrap_or(""),
        out["negative"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["negative"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "delta: top_changed={} score_delta={} suppression_count={}",
        out["delta"]["top_changed"].as_bool().unwrap_or(false),
        out["delta"]["score_delta"].as_f64().unwrap_or(0.0),
        out["delta"]["suppression_count"].as_u64().unwrap_or(0)
    );
}

fn print_probe_suite_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("mode: {}", out["mode"].as_str().unwrap_or("probe-suite"));
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {}: {} plain={} negative={}",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_plain_peak"].as_str().unwrap_or(""),
                case["actual_negative_peak"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_search_md(out: &Value) {
    println!("# NANDA Interference Search\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- memory_size: `{}`", out["memory_size"]);
    println!("- peak_state: `{}`", out["peak_decision"]["state"]);
    println!(
        "- safe_to_answer: `{}`",
        out["peak_decision"]["safe_to_answer"]
    );
    println!("- peak_margin: `{}`", out["peak_margin"]);
    println!(
        "- lexical_baseline_top: `{}`",
        out["lexical_baseline"]["top_peak"].as_str().unwrap_or("")
    );
    if let Some(peaks) = out["peaks"].as_array() {
        for (idx, peak) in peaks.iter().enumerate() {
            println!(
                "\n## Peak {}: `{}`\n",
                idx + 1,
                peak["peak"].as_str().unwrap_or("")
            );
            println!("- score: `{}`", peak["score"]);
            println!("- route: `{}`", peak["center"]["route"]);
            println!("- group: `{}`", peak["center"]["group"]);
            println!("- support:");
            if let Some(support) = peak["supporting_triads"].as_array() {
                for item in support.iter().take(5) {
                    println!(
                        "  - `{}`: {} -> {} -> {}",
                        item["id"].as_str().unwrap_or(""),
                        item["subject"].as_str().unwrap_or(""),
                        item["relation"].as_str().unwrap_or(""),
                        item["object"].as_str().unwrap_or("")
                    );
                }
            }
        }
    }
}

fn print_probe_md(out: &Value) {
    println!("# NANDA Probe\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- decision: `{}`", out["decision"].as_str().unwrap_or(""));
    println!(
        "- plain: `{}` / `{}` / safe `{}` / score `{}`",
        out["plain"]["top_peak"].as_str().unwrap_or(""),
        out["plain"]["field_state"].as_str().unwrap_or(""),
        out["plain"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["plain"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "- negative: `{}` / `{}` / safe `{}` / score `{}`",
        out["negative"]["top_peak"].as_str().unwrap_or(""),
        out["negative"]["field_state"].as_str().unwrap_or(""),
        out["negative"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["negative"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "- delta: top_changed `{}` / score_delta `{}` / suppressions `{}`",
        out["delta"]["top_changed"].as_bool().unwrap_or(false),
        out["delta"]["score_delta"].as_f64().unwrap_or(0.0),
        out["delta"]["suppression_count"].as_u64().unwrap_or(0)
    );
}

fn print_probe_suite_md(out: &Value) {
    println!("# NANDA Probe Suite\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- `{}`: `{}` plain `{}` negative `{}`",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_plain_peak"].as_str().unwrap_or(""),
                case["actual_negative_peak"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_dogfood_text(out: &Value) {
    let decision = &out["agent_decision"];
    println!("NANDA DOGFOOD");
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "ACTION: {}",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "ROOT: {}{}",
        decision["root_verdict"].as_str().unwrap_or("WATCH"),
        if decision["root_size_only"].as_bool().unwrap_or(false) {
            " size-only"
        } else {
            ""
        }
    );
    println!(
        "STRUCTURE: foreign_pull={} invariant_violation={}",
        decision["foreign_pull"].as_u64().unwrap_or(0),
        decision["invariant_violation"].as_u64().unwrap_or(0)
    );
    println!(
        "BRANCHES: {}/{} PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["local_branches"].as_u64().unwrap_or(0)
    );
    println!(
        "SAFE_TO_EDIT: {}",
        decision["safe_to_edit"].as_bool().unwrap_or(false)
    );
    println!("NEXT: {}", decision["next"].as_str().unwrap_or(""));
}

fn print_dogfood_md(out: &Value) {
    let decision = &out["agent_decision"];
    println!("# NANDA Dogfood\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "- action: `{}`",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "- root: `{}`",
        decision["root_verdict"].as_str().unwrap_or("WATCH")
    );
    println!("- root_size_only: `{}`", decision["root_size_only"]);
    println!(
        "- branches: `{}/{}` PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["local_branches"].as_u64().unwrap_or(0)
    );
    println!("- foreign_pull: `{}`", decision["foreign_pull"]);
    println!(
        "- invariant_violation: `{}`",
        decision["invariant_violation"]
    );
    println!("- safe_to_edit: `{}`", decision["safe_to_edit"]);
}

fn report_cmd(args: ReportArgs) -> Result<u8> {
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

fn worst_status(a: u8, b: u8) -> u8 {
    if a == EXIT_VETO || b == EXIT_VETO {
        EXIT_VETO
    } else if a == EXIT_WATCH || b == EXIT_WATCH {
        EXIT_WATCH
    } else {
        EXIT_PASS
    }
}

fn self_check() -> Result<u8> {
    let packet = example_packet(false);
    let report = make_report(
        &packet,
        &normalize_ids(packet.triads.clone(), "t"),
        &normalize_ids(packet.candidate_triads.clone(), "c"),
    )?;
    if report.verdict != "PASS" {
        println!("verdict: VETO");
        return Ok(EXIT_VETO);
    }
    println!("verdict: PASS");
    Ok(EXIT_PASS)
}

fn benchmark() -> Result<u8> {
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

fn verdict_for(packet: &Packet) -> Result<String> {
    Ok(make_report(
        packet,
        &normalize_ids(packet.triads.clone(), "t"),
        &normalize_ids(packet.candidate_triads.clone(), "c"),
    )?
    .verdict)
}

fn exact_baseline_accepts(packet: &Packet) -> bool {
    let source: HashSet<_> = packet.triads.iter().map(structural_key).collect();
    packet
        .candidate_triads
        .iter()
        .any(|candidate| source.contains(&structural_key(candidate)))
}

fn write_or_print(path: PathBuf, stdout: bool, output: String) -> Result<()> {
    if stdout {
        print!("{output}");
    } else {
        fs::write(&path, output)?;
        println!("{}", path.display());
    }
    Ok(())
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn packet_from_markdown(
    path: &Path,
    task_id: &str,
    domain: &str,
    query: &str,
    normalize_paths: bool,
) -> Result<Packet> {
    let text = fs::read_to_string(path)?;
    let (triads, candidate_triads) = parse_markdown_tables(&text, normalize_paths);
    Ok(Packet {
        task_id: task_id.to_string(),
        domain: domain.to_string(),
        query: query.to_string(),
        triads,
        candidate_triads,
        candidate_answer: String::new(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
    })
}

fn parse_markdown_tables(text: &str, normalize_paths: bool) -> (Vec<Triad>, Vec<Triad>) {
    let mut mode = "triads";
    let mut triads = vec![];
    let mut candidates = vec![];
    let mut header: Vec<String> = vec![];
    for raw in text.lines() {
        let line = raw.trim();
        let heading = line
            .trim_start_matches('#')
            .trim()
            .replace('-', "_")
            .to_lowercase();
        if matches!(heading.as_str(), "triads" | "source_triads" | "source") {
            mode = "triads";
            header.clear();
            continue;
        }
        if matches!(
            heading.as_str(),
            "candidate_triads" | "candidates" | "candidate"
        ) {
            mode = "candidate_triads";
            header.clear();
            continue;
        }
        let cells = parse_row(line);
        if cells.is_empty()
            || cells
                .iter()
                .all(|c| c.trim_matches(':').chars().all(|ch| ch == '-'))
        {
            continue;
        }
        if header.is_empty() {
            header = cells.iter().map(|c| normalize_header(c)).collect();
            continue;
        }
        let mut row = HashMap::new();
        for (key, value) in header.iter().zip(cells.iter()) {
            row.insert(key.as_str(), value.as_str());
        }
        let mut triad = Triad {
            id: row.get("id").unwrap_or(&"").to_string(),
            subject: row.get("subject").unwrap_or(&"").to_string(),
            relation: row.get("relation").unwrap_or(&"").to_string(),
            object: row.get("object").unwrap_or(&"").to_string(),
            evidence: row.get("evidence").unwrap_or(&"").to_string(),
            confidence: row
                .get("confidence")
                .and_then(|x| x.parse().ok())
                .unwrap_or(0.0),
            subject_role: row.get("subject_role").unwrap_or(&"subject").to_string(),
            object_role: row.get("object_role").unwrap_or(&"object").to_string(),
            route: row.get("route").unwrap_or(&"").to_string(),
            group: row.get("group").unwrap_or(&"").to_string(),
        };
        if normalize_paths {
            triad.subject = normalize_code_entity(&triad.subject);
            triad.object = normalize_code_entity(&triad.object);
        }
        if mode == "triads" {
            triads.push(triad);
        } else {
            candidates.push(triad);
        }
    }
    (normalize_ids(triads, "t"), normalize_ids(candidates, "c"))
}

fn parse_row(line: &str) -> Vec<String> {
    if !line.starts_with('|') || !line.ends_with('|') {
        return vec![];
    }
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().trim_matches('`').trim().to_string())
        .collect()
}

fn normalize_header(value: &str) -> String {
    let key = value
        .trim()
        .to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");
    match key.trim_matches('_') {
        "subj" => "subject".to_string(),
        "rel" => "relation".to_string(),
        "obj" => "object".to_string(),
        "subj_role" => "subject_role".to_string(),
        "obj_role" => "object_role".to_string(),
        other => other.to_string(),
    }
}

fn normalize_code_entity(value: &str) -> String {
    let mut text = value.trim().trim_start_matches("./").to_string();
    if let Some(idx) = text.find("/projects/") {
        let rest = &text[idx + "/projects/".len()..];
        if let Some(pos) = rest.find('/') {
            text = rest[pos + 1..].to_string();
        }
    }
    if text.starts_with("src/bin/") && text.ends_with(".rs") {
        return format!(
            "bin::{}",
            Path::new(&text)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        );
    }
    if text.starts_with("src/") && text.ends_with(".rs") {
        let mut parts: Vec<&str> = text
            .trim_start_matches("src/")
            .trim_end_matches(".rs")
            .split('/')
            .collect();
        if parts.last() == Some(&"mod") {
            parts.pop();
        }
        return parts.join("::");
    }
    if text.ends_with(".rs") && text.contains('/') {
        return text
            .trim_end_matches(".rs")
            .split('/')
            .collect::<Vec<_>>()
            .join("::");
    }
    text
}

fn split_label(by: &SplitBy) -> &'static str {
    match by {
        SplitBy::Group => "group",
        SplitBy::Route => "route",
        SplitBy::LinkedGroup => "linked-group",
    }
}

fn split_output_label(format: &SplitOutputFormat) -> &'static str {
    match format {
        SplitOutputFormat::Json => "json",
        SplitOutputFormat::Md => "md",
    }
}

fn split_worksheet(
    source: &Path,
    by: &str,
    key: &str,
    triads: &[Triad],
    candidates: &[Triad],
) -> String {
    format!(
        "# NANDA Split Worksheet\n\nsplit_by: {by}\nsplit_key: {key}\nsource: {}\n\n{}\n{}",
        source.display(),
        table("triads", triads),
        table("candidate_triads", candidates)
    )
}

fn table(title: &str, rows: &[Triad]) -> String {
    let headers = [
        "id",
        "subject",
        "relation",
        "object",
        "evidence",
        "confidence",
        "subject_role",
        "object_role",
        "route",
        "group",
    ];
    let mut out = format!(
        "## {title}\n\n| {} |\n|{}|\n",
        headers.join(" | "),
        vec!["----"; headers.len()].join("|")
    );
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.3} | {} | {} | {} | {} |\n",
            row.id,
            row.subject,
            row.relation,
            row.object,
            row.evidence,
            row.confidence,
            row.subject_role,
            row.object_role,
            row.route,
            row.group
        ));
    }
    out.push('\n');
    out
}

fn template_text(task_id: &str, domain: &str, query: &str, kind: &TemplateKind) -> String {
    let rows = match kind {
        TemplateKind::Code => vec![
            (
                "t1",
                "source module",
                "installs_to",
                "runtime module",
                "README.md:1",
                "source",
                "runtime",
                "deploy",
                "source-runtime",
            ),
            (
                "t2",
                "runtime module",
                "exposes",
                "CLI command",
                "README.md:2",
                "runtime",
                "cli",
                "deploy",
                "source-runtime",
            ),
            (
                "t3",
                "CLI command",
                "calls",
                "checker core",
                "README.md:3",
                "cli",
                "core",
                "execution",
                "source-runtime",
            ),
        ],
        TemplateKind::Skill => vec![
            (
                "t1",
                "source skill",
                "syncs_to",
                "runtime skill",
                "scripts/install-local.sh:10",
                "source",
                "runtime",
                "deploy",
                "skill-flow",
            ),
            (
                "t2",
                "runtime skill",
                "provides",
                "trigger rule",
                "SKILL.md:2",
                "runtime",
                "trigger",
                "agent",
                "skill-flow",
            ),
            (
                "t3",
                "CLI command",
                "returns",
                "gate verdict",
                "scripts/nanda-check:1",
                "cli",
                "verdict",
                "execution",
                "skill-flow",
            ),
        ],
        TemplateKind::Project => vec![
            (
                "t1",
                "repository",
                "contains",
                "source files",
                "README.md:1",
                "repo",
                "source",
                "project",
                "project-flow",
            ),
            (
                "t2",
                "repository",
                "documents",
                "architecture",
                "ARCHITECTURE.md:1",
                "repo",
                "docs",
                "project",
                "project-flow",
            ),
            (
                "t3",
                "test suite",
                "validates",
                "runtime behavior",
                "scripts/test-local.sh:1",
                "tests",
                "runtime",
                "validation",
                "project-flow",
            ),
        ],
        TemplateKind::Generic => vec![("t1", "", "", "", "", "", "", "", "")],
    };
    let mut out = format!(
        "# NANDA Triad Worksheet\n\ntask_id: {task_id}\ndomain: {domain}\nquery: {query}\n\n"
    );
    out.push_str("## triads\n\n| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |\n|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|\n");
    for row in &rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | 0.9 | {} | {} | {} | {} |\n",
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8
        ));
    }
    out.push_str("\n## candidate_triads\n\n| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |\n|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|\n");
    for row in rows {
        out.push_str(&format!(
            "| c{} | {} | {} | {} | candidate_answer | 0.9 | {} | {} | {} | candidate-{} |\n",
            &row.0[1..],
            row.1,
            row.2,
            row.3,
            row.5,
            row.6,
            row.7,
            row.8
        ));
    }
    out.push_str("\n## notes\n\n- Fill `triads` from source evidence.\n- Fill `candidate_triads` from the answer being checked.\n- Keep one coherent `group` per route, case, or local structure.\n");
    out
}

fn action_for_report(report: &Report) -> &'static str {
    match report.verdict.as_str() {
        "PASS" => "SEND_OK",
        "WATCH" => "DRAFT_OK_REVIEW_REQUIRED",
        _ => "REPAIR_REQUIRED",
    }
}

fn example_packet(swapped: bool) -> Packet {
    let triads = vec![
        triad(
            "t1", "supplier", "supplies", "buyer", "doc:1", "seller", "buyer", "route-a", "deal-a",
        ),
        triad(
            "t2", "buyer", "pays_to", "supplier", "doc:2", "payer", "payee", "route-a", "deal-a",
        ),
    ];
    let candidate_triads = if swapped {
        vec![triad(
            "c1", "buyer", "supplies", "supplier", "answer", "buyer", "seller", "route-a", "deal-a",
        )]
    } else {
        vec![
            triad(
                "c1", "supplier", "supplies", "buyer", "answer", "seller", "buyer", "route-a",
                "deal-a",
            ),
            triad(
                "c2", "buyer", "pays_to", "supplier", "answer", "payer", "payee", "route-a",
                "deal-a",
            ),
        ]
    };
    Packet {
        task_id: "example".to_string(),
        domain: "general".to_string(),
        query: String::new(),
        triads,
        candidate_triads,
        candidate_answer: String::new(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
    }
}

fn synthetic_packet(idx: usize, kind: &str) -> Packet {
    let a = format!("supplier-{idx}");
    let b = format!("buyer-{idx}");
    let c = format!("carrier-{idx}");
    let d = format!("warehouse-{idx}");
    let triads = vec![
        triad(
            "t1",
            &a,
            "supplies",
            &b,
            "doc:invoice",
            "supplier",
            "buyer",
            "trade",
            "trade",
        ),
        triad(
            "t2",
            &b,
            "pays_to",
            &a,
            "doc:payment",
            "payer",
            "payee",
            "payment",
            "trade",
        ),
        triad(
            "t3",
            &a,
            "delivers_to",
            &c,
            "doc:logistics",
            "shipper",
            "carrier",
            "delivery",
            "logistics",
        ),
        triad(
            "t4",
            &c,
            "delivers_to",
            &d,
            "doc:warehouse",
            "carrier",
            "warehouse",
            "delivery",
            "logistics",
        ),
    ];
    let candidate_triads = match kind {
        "swap" => vec![triad(
            "c1",
            &b,
            "supplies",
            &a,
            "answer",
            "buyer",
            "supplier",
            "trade",
            "candidate",
        )],
        "splice" => vec![
            triad(
                "c1",
                &a,
                "supplies",
                &b,
                "answer",
                "supplier",
                "buyer",
                "trade",
                "candidate",
            ),
            triad(
                "c2",
                &c,
                "delivers_to",
                &d,
                "answer",
                "carrier",
                "warehouse",
                "delivery",
                "candidate",
            ),
        ],
        _ => vec![
            triad(
                "c1", &a, "supplies", &b, "answer", "supplier", "buyer", "trade", "trade",
            ),
            triad(
                "c2", &b, "pays_to", &a, "answer", "payer", "payee", "payment", "trade",
            ),
        ],
    };
    Packet {
        task_id: format!("synthetic-{idx}"),
        domain: "general".to_string(),
        query: String::new(),
        triads,
        candidate_triads,
        candidate_answer: String::new(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
    }
}

fn triad(
    id: &str,
    subject: &str,
    relation: &str,
    object: &str,
    evidence: &str,
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
        evidence: evidence.to_string(),
        confidence: 0.9,
        subject_role: subject_role.to_string(),
        object_role: object_role.to_string(),
        route: route.to_string(),
        group: group.to_string(),
    }
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
