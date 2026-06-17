use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const WAVE_DIM: usize = 1024;
const CORE_VERSION: &str = "sparse-triad-v1.0-release";
const ENGINE_ID: &str = "nanda-check sparse-triad-v1.0-rust";
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
    Comb(CombArgs),
    Extract(ExtractArgs),
    Index(IndexArgs),
    Search(SearchArgs),
    Feedback(FeedbackArgs),
    Eval(EvalArgs),
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
    #[arg(long, value_enum, default_value = "route")]
    group_by: PeakGroupBy,
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
        Command::Comb(args) => comb_cmd(args),
        Command::Extract(args) => extract_cmd(args),
        Command::Index(args) => index_cmd(args),
        Command::Search(args) => search_cmd(args),
        Command::Feedback(args) => feedback_cmd(args),
        Command::Eval(args) => eval_cmd(args),
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
        "wave_dim": WAVE_DIM,
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
    let packet = load_packet_auto(
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
    let query = normalize_ids(query_packet.candidate_triads.clone(), "q");
    let result = interference_search(&packet, &memory, &query, args.top_k, &args.group_by);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
        OutputFormat::Text => print_search_text(&result),
        OutputFormat::Md => print_search_md(&result),
    }
    Ok(EXIT_PASS)
}

fn index_cmd(args: IndexArgs) -> Result<u8> {
    if args.inputs.is_empty() {
        return Err(anyhow!(
            "nanda index requires at least one input packet or worksheet"
        ));
    }
    let mut triads = vec![];
    for input in &args.inputs {
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
    }
    let triads = dedup_triads(triads);
    let packet = json!({
        "task_id": args.task_id,
        "domain": args.domain,
        "query": args.query,
        "triads": triads,
        "candidate_triads": [],
        "candidate_answer": "",
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
    let decision = feedback_decision_label(&args.decision);
    let reinforcement = match args.decision {
        FeedbackDecision::Accept => json!({
            "reinforce_peak": peak_name,
            "reinforce_support": support_ids,
            "suppress_foreign": anti_ids
        }),
        FeedbackDecision::Reject => json!({
            "reject_peak": peak_name,
            "suppress_support": support_ids,
            "inspect_alternatives": anti_ids
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
        "memory_patch": reinforcement,
        "interpretation": "Feedback is a compact trace for later memory tuning. It records whether a focused interference peak was accepted, rejected, or kept under WATCH."
    });
    let output = serde_json::to_string_pretty(&feedback)? + "\n";
    write_or_print(args.out, args.stdout, output)?;
    Ok(EXIT_PASS)
}

fn feedback_decision_label(decision: &FeedbackDecision) -> &'static str {
    match decision {
        FeedbackDecision::Accept => "accept",
        FeedbackDecision::Reject => "reject",
        FeedbackDecision::Watch => "watch",
    }
}

fn eval_cmd(args: EvalArgs) -> Result<u8> {
    if args.cases.is_empty() {
        return Err(anyhow!(
            "nanda eval requires at least one --case path:expected_peak:expected_state"
        ));
    }
    let mut rows = vec![];
    let mut passed = 0usize;
    for raw_case in &args.cases {
        let (path, expected_peak, expected_state) = parse_eval_case(raw_case)?;
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
        let result = interference_search(&packet, &memory, &query, args.top_k, &args.group_by);
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

fn doctor_cmd(args: DoctorArgs) -> Result<u8> {
    let route_trap = builtin_route_trap_packet(false);
    let trap_result = interference_search(
        &route_trap,
        &normalize_ids(route_trap.triads.clone(), "m"),
        &normalize_ids(route_trap.candidate_triads.clone(), "q"),
        3,
        &PeakGroupBy::Route,
    );
    let noisy = builtin_route_trap_packet(true);
    let noisy_result = interference_search(
        &noisy,
        &normalize_ids(noisy.triads.clone(), "m"),
        &normalize_ids(noisy.candidate_triads.clone(), "q"),
        3,
        &PeakGroupBy::Route,
    );
    let trap_ok = trap_result["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["peak"].as_str())
        == Some("certification")
        && trap_result["peak_decision"]["state"].as_str() == Some("FOCUSED")
        && trap_result["wins_over_lexical_baseline"]
            .as_bool()
            .unwrap_or(false);
    let noisy_ok = noisy_result["peak_decision"]["state"].as_str() == Some("WATCH")
        && !noisy_result["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(true);
    let healthy = trap_ok && noisy_ok;
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "doctor",
        "healthy": healthy,
        "checks": {
            "route_trap_focused": trap_ok,
            "noisy_query_watch": noisy_ok
        },
        "route_trap": {
            "top": trap_result["peaks"].as_array().and_then(|peaks| peaks.first()).and_then(|peak| peak["peak"].as_str()).unwrap_or(""),
            "state": trap_result["peak_decision"]["state"],
            "safe_to_answer": trap_result["peak_decision"]["safe_to_answer"],
            "lexical_baseline_top": trap_result["lexical_baseline"]["top_peak"],
            "wins_over_lexical_baseline": trap_result["wins_over_lexical_baseline"],
            "peak_margin": trap_result["peak_margin"]
        },
        "noisy": {
            "top": noisy_result["peaks"].as_array().and_then(|peaks| peaks.first()).and_then(|peak| peak["peak"].as_str()).unwrap_or(""),
            "state": noisy_result["peak_decision"]["state"],
            "safe_to_answer": noisy_result["peak_decision"]["safe_to_answer"],
            "peak_margin": noisy_result["peak_margin"]
        }
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_doctor_text(&out),
        OutputFormat::Md => print_doctor_md(&out),
    }
    if healthy {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

fn print_doctor_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("healthy: {}", out["healthy"].as_bool().unwrap_or(false));
    println!(
        "route_trap: top={} state={} lexical={} wins={}",
        out["route_trap"]["top"].as_str().unwrap_or(""),
        out["route_trap"]["state"].as_str().unwrap_or(""),
        out["route_trap"]["lexical_baseline_top"]
            .as_str()
            .unwrap_or(""),
        out["route_trap"]["wins_over_lexical_baseline"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "noisy: top={} state={} safe={}",
        out["noisy"]["top"].as_str().unwrap_or(""),
        out["noisy"]["state"].as_str().unwrap_or(""),
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
        "- route_trap: `{}` / `{}`",
        out["route_trap"]["top"], out["route_trap"]["state"]
    );
    println!(
        "- noisy: `{}` / `{}`",
        out["noisy"]["top"], out["noisy"]["state"]
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

fn interference_search(
    packet: &Packet,
    memory: &[Triad],
    query: &[Triad],
    top_k: usize,
    group_by: &PeakGroupBy,
) -> Value {
    let query_wave = query_feature_wave(query);
    let mut scored = vec![];
    for triad in memory {
        let wave = triad_feature_wave(triad);
        let score = cosine(&query_wave, &wave);
        let symbolic = symbolic_query_overlap(query, triad);
        let combined = round4((0.72 * score) + (0.28 * symbolic));
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
        let peak_score = round4(
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
        peaks.push(json!({
            "peak": key,
            "score": peak_score,
            "coherence": coherence,
            "coverage": coverage,
            "chain_coherence": chain,
            "propagation": propagation,
            "top_triad_score": round4(top_score),
            "symbolic_baseline": symbolic_peak_baseline(&items),
            "center": center,
            "supporting_triads": support,
            "anti_triads": anti,
            "missing_edges": missing_edges(&query_terms, &items),
            "answer_projection": answer_projection(&center, &items)
        }));
    }
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
    let top_peak = peaks
        .first()
        .and_then(|peak| peak["peak"].as_str())
        .unwrap_or("")
        .to_string();
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let peak_decision = peak_decision(&peaks, peak_margin, lexical_peak);
    let mut output_peaks = peaks;
    output_peaks.truncate(top_k);

    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "interference-retrieval",
        "task_id": packet.task_id,
        "domain": packet.domain,
        "query": {
            "text": packet.query,
            "triads": query.iter().map(triad_json).collect::<Vec<_>>()
        },
        "memory_size": memory.len(),
        "group_by": match group_by {
            PeakGroupBy::Group => "group",
            PeakGroupBy::Route => "route",
        },
        "peak_margin": peak_margin,
        "lexical_baseline": lexical_baseline,
        "wins_over_lexical_baseline": !lexical_peak.is_empty() && top_peak.as_str() != lexical_peak,
        "peak_decision": peak_decision,
        "peaks": output_peaks,
        "interpretation": "A peak is a route/group whose triads resonate together with the partial query. Read support as the focused structure and anti_triads as similar-but-foreign pulls."
    })
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
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let wins_lexical = !lexical_peak.is_empty() && top_name != lexical_peak;
    let (state, safe_to_answer, reason) = if margin >= 0.06 && component_gap >= 0.12 {
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
