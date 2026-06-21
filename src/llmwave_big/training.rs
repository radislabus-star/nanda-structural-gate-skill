//! Corpus compiler for LLMWave-Big.
//!
//! This is cold training code. It may allocate, parse text, and write JSON
//! artifacts. Hot Active Core code consumes the compact records produced here.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::WAVE_DIM;

pub(crate) const TRAINING_VERSION: &str = "llmwave-big-v1901-corpus-training";
const DEFAULT_EXTENSIONS: &[&str] = &[
    "md", "txt", "rs", "json", "toml", "yaml", "yml", "py", "js", "ts", "html", "css",
];

#[derive(Serialize, Clone)]
pub(crate) struct TrainingCompileReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub read_as: &'static str,
    pub corpus: CorpusStats,
    pub field_budget: FieldBudget,
    pub active_core: ActiveCoreTrainingSummary,
    pub wave_atlas: WaveAtlasTrainingSummary,
    pub eval: TrainingEvalSummary,
    pub output: TrainingOutput,
    pub claim_boundary: TrainingClaimBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CorpusStats {
    pub input_roots: Vec<String>,
    pub files_seen: usize,
    pub files_used: usize,
    pub duplicate_files_skipped: usize,
    pub bytes_used: usize,
    pub lines_used: usize,
    pub tokens_seen: usize,
    pub unique_tokens: usize,
    pub unique_chunks: usize,
    pub chunk_tokens: usize,
    pub held_out_chunks: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FieldBudget {
    pub wave_dim: usize,
    pub hot_budget_bytes: usize,
    pub token_record_bytes: usize,
    pub transition_record_bytes: usize,
    pub chunk_record_bytes: usize,
    pub token_records: usize,
    pub transition_records: usize,
    pub chunk_records: usize,
    pub estimated_hot_bytes: usize,
    pub fits_hot_budget: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct ActiveCoreTrainingSummary {
    pub state: &'static str,
    pub top_token_records: usize,
    pub transition_records: usize,
    pub active_chunk_records: usize,
    pub schema_hint_records: usize,
    pub residual_only_write: bool,
    pub cold_text_kept_out_of_hot_loop: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct WaveAtlasTrainingSummary {
    pub artifact_schema: &'static str,
    pub corpus_hash: String,
    pub top_tokens: Vec<TokenRecord>,
    pub top_transitions: Vec<TransitionRecord>,
    pub schema_hints: Vec<SchemaHint>,
    pub sample_chunks: Vec<ChunkRecord>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TrainingEvalSummary {
    pub state: &'static str,
    pub next_token_accuracy: f32,
    pub held_out_cases: usize,
    pub held_out_passed: usize,
    pub avg_rank_margin: f32,
    pub notes: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TrainingOutput {
    pub artifact_written: bool,
    pub artifact_path: Option<String>,
    pub report_is_summary_only: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct TrainingClaimBoundary {
    pub real_corpus_loaded: bool,
    pub broad_training_pipeline: bool,
    pub chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactAskReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub query: String,
    pub artifact: ArtifactAskSummary,
    pub query_wave: QueryWaveSummary,
    pub field: ArtifactFieldSummary,
    pub answer: ArtifactAnswer,
    pub claim_boundary: ArtifactAskClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactAskSummary {
    pub path: String,
    pub corpus_tokens: usize,
    pub token_records: usize,
    pub transition_records: usize,
    pub chunk_records: usize,
    pub schema_hint_records: usize,
    pub estimated_hot_bytes: usize,
    pub fits_hot_budget: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct QueryWaveSummary {
    pub tokens: Vec<String>,
    pub token_count: usize,
    pub phase: u16,
    pub energy: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactFieldSummary {
    pub state: &'static str,
    pub top_chunk_peaks: Vec<ChunkPeak>,
    pub top_schema_peaks: Vec<SchemaPeak>,
    pub top_transition_peaks: Vec<TransitionPeak>,
    pub support_score: f32,
    pub margin: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct ChunkPeak {
    pub chunk_id: u32,
    pub score: f32,
    pub overlap: usize,
    pub phase_alignment: f32,
    pub text: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct SchemaPeak {
    pub schema_id: u32,
    pub score: f32,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub count: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct TransitionPeak {
    pub transition_id: u32,
    pub score: f32,
    pub from: String,
    pub to: String,
    pub count: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactAnswer {
    pub state: &'static str,
    pub safe_to_answer: bool,
    pub text: String,
    pub evidence_chunks: Vec<u32>,
    pub evidence_schemas: Vec<u32>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactAskClaimBoundary {
    pub artifact_loaded: bool,
    pub trained_field_used: bool,
    pub generated_from_training_artifact: bool,
    pub broad_chat_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactAskEvalReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub artifact: String,
    pub suite: String,
    pub cases: Vec<ArtifactAskEvalCaseReport>,
    pub metrics: ArtifactAskEvalMetrics,
    pub claim_boundary: ArtifactAskClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactAskEvalCaseReport {
    pub id: String,
    pub query: String,
    pub expected_contains: String,
    pub expected_safe_to_answer: bool,
    pub observed_state: &'static str,
    pub observed_safe_to_answer: bool,
    pub observed_answer: String,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct ArtifactAskEvalMetrics {
    pub total: usize,
    pub passed: usize,
    pub answer_accuracy: f32,
    pub false_positive_rate: f32,
    pub false_negative_rate: f32,
}

#[derive(Deserialize)]
struct ArtifactAskEvalSuite {
    cases: Vec<ArtifactAskEvalCase>,
}

#[derive(Deserialize)]
struct ArtifactAskEvalCase {
    id: String,
    query: String,
    #[serde(default)]
    expected_contains: String,
    #[serde(default)]
    expected_safe_to_answer: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct TrainingArtifact {
    pub mode: String,
    pub version: String,
    pub corpus: CorpusStats,
    pub field_budget: FieldBudget,
    pub records: TrainingRecords,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct TrainingRecords {
    pub tokens: Vec<TokenRecord>,
    pub transitions: Vec<TransitionRecord>,
    pub chunks: Vec<ChunkRecord>,
    pub schema_hints: Vec<SchemaHint>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct TokenRecord {
    pub id: u32,
    pub token: String,
    pub count: u32,
    pub phase: u16,
    pub polarity: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct TransitionRecord {
    pub id: u32,
    pub from: String,
    pub to: String,
    pub count: u32,
    pub phase_delta: i16,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ChunkRecord {
    pub id: u32,
    pub file_id: u32,
    pub token_count: usize,
    pub hash: String,
    pub centroid_phase: u16,
    pub energy: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct SchemaHint {
    pub id: u32,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub count: u32,
    pub route: String,
}

#[derive(Clone)]
pub(crate) struct TrainingConfig {
    pub inputs: Vec<PathBuf>,
    pub out: Option<PathBuf>,
    pub vocab_cap: usize,
    pub transition_cap: usize,
    pub active_chunk_cap: usize,
    pub chunk_tokens: usize,
    pub hot_budget_bytes: usize,
    pub max_file_bytes: usize,
    pub extensions: Vec<String>,
}

struct LoadedFile {
    id: u32,
    bytes: usize,
    lines: usize,
    text: String,
}

struct CorpusBuild {
    files_seen: usize,
    files: Vec<LoadedFile>,
    duplicate_files_skipped: usize,
    corpus_hash: String,
}

pub(crate) fn compile_training_corpus(config: TrainingConfig) -> Result<TrainingCompileReport> {
    if config.inputs.is_empty() {
        bail!("at least one input file or directory is required");
    }
    if config.chunk_tokens == 0 {
        bail!("--chunk-tokens must be greater than zero");
    }

    let corpus = load_corpus(&config)?;
    let mut token_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut transition_counts: BTreeMap<(String, String), u32> = BTreeMap::new();
    let mut trigram_counts: BTreeMap<(String, String, String), u32> = BTreeMap::new();
    let mut chunks = Vec::new();
    let mut held_out = Vec::new();
    let mut seen_chunks = HashSet::new();
    let mut tokens_seen = 0usize;
    let mut bytes_used = 0usize;
    let mut lines_used = 0usize;

    for file in &corpus.files {
        bytes_used += file.bytes;
        lines_used += file.lines;
        let tokens = tokenize(&file.text);
        tokens_seen += tokens.len();
        for token in &tokens {
            *token_counts.entry(token.clone()).or_insert(0) += 1;
        }
        for pair in tokens.windows(2) {
            if let [from, to] = pair {
                *transition_counts
                    .entry((from.clone(), to.clone()))
                    .or_insert(0) += 1;
            }
        }
        for triple in tokens.windows(3) {
            if let [subject, relation, object] = triple {
                *trigram_counts
                    .entry((subject.clone(), relation.clone(), object.clone()))
                    .or_insert(0) += 1;
            }
        }
        collect_chunks(
            file,
            &tokens,
            config.chunk_tokens,
            config.active_chunk_cap,
            &mut seen_chunks,
            &mut chunks,
            &mut held_out,
        );
    }

    let token_records = build_token_records(token_counts, config.vocab_cap);
    let transition_records =
        build_transition_records(&transition_counts, &token_records, config.transition_cap);
    let schema_hints = build_schema_hints(&trigram_counts, 512);
    let corpus_stats = CorpusStats {
        input_roots: config
            .inputs
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        files_seen: corpus.files_seen,
        files_used: corpus.files.len(),
        duplicate_files_skipped: corpus.duplicate_files_skipped,
        bytes_used,
        lines_used,
        tokens_seen,
        unique_tokens: token_records.len(),
        unique_chunks: chunks.len(),
        chunk_tokens: config.chunk_tokens,
        held_out_chunks: held_out.len(),
    };
    let field_budget = build_field_budget(
        config.hot_budget_bytes,
        token_records.len(),
        transition_records.len(),
        chunks.len(),
    );
    let eval = eval_next_token(&held_out, &transition_records);
    let artifact = TrainingArtifact {
        mode: "llmwave-big-training-artifact".to_string(),
        version: TRAINING_VERSION.to_string(),
        corpus: corpus_stats.clone(),
        field_budget: field_budget.clone(),
        records: TrainingRecords {
            tokens: token_records.clone(),
            transitions: transition_records.clone(),
            chunks: chunks.clone(),
            schema_hints: schema_hints.clone(),
        },
    };
    if let Some(out) = &config.out {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output directory {}", parent.display()))?;
        }
        fs::write(out, serde_json::to_string_pretty(&artifact)? + "\n")
            .with_context(|| format!("write training artifact {}", out.display()))?;
    }

    Ok(TrainingCompileReport {
        mode: "llmwave-big-train",
        version: TRAINING_VERSION,
        verdict: if eval.held_out_cases > 0 && field_budget.fits_hot_budget {
            "TRAINING_ARTIFACT_READY_NOT_LLM"
        } else {
            "TRAINING_ARTIFACT_REVIEW"
        },
        read_as: "large corpus compiler for Wave Atlas and Active Core records; not a claim of chat readiness",
        corpus: corpus_stats,
        field_budget,
        active_core: ActiveCoreTrainingSummary {
            state: "ACTIVE_CORE_TRAINING_RECORDS_READY",
            top_token_records: token_records.len(),
            transition_records: transition_records.len(),
            active_chunk_records: chunks.len(),
            schema_hint_records: schema_hints.len(),
            residual_only_write: true,
            cold_text_kept_out_of_hot_loop: true,
        },
        wave_atlas: WaveAtlasTrainingSummary {
            artifact_schema: "tokens + transitions + chunks + schema_hints",
            corpus_hash: corpus.corpus_hash,
            top_tokens: token_records.iter().take(32).cloned().collect(),
            top_transitions: transition_records.iter().take(32).cloned().collect(),
            schema_hints: schema_hints.iter().take(32).cloned().collect(),
            sample_chunks: chunks.iter().take(8).cloned().collect(),
        },
        eval,
        output: TrainingOutput {
            artifact_written: config.out.is_some(),
            artifact_path: config.out.as_ref().map(|path| path.display().to_string()),
            report_is_summary_only: true,
        },
        claim_boundary: TrainingClaimBoundary {
            real_corpus_loaded: true,
            broad_training_pipeline: true,
            chat_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim: "LLMWave-Big can compile a real corpus into compact field records and measure held-out next-token resonance.",
            forbidden_claims: vec![
                "this is already a general LLM",
                "this proves nonlinear memory",
                "this proves cache-only execution",
                "more words alone make the model intelligent",
            ],
        },
    })
}

pub(crate) fn load_training_artifact(path: &Path) -> Result<TrainingArtifact> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read training artifact {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parse training artifact {}", path.display()))
}

pub(crate) fn ask_training_artifact(
    artifact_path: &Path,
    query: String,
    top_k: usize,
) -> Result<ArtifactAskReport> {
    let artifact = load_training_artifact(artifact_path)?;
    let query_tokens = tokenize(&query)
        .into_iter()
        .filter(|token| is_word_token(token))
        .collect::<Vec<_>>();
    let query_terms = expand_query_terms(&query_tokens);
    let query_wave = QueryWaveSummary {
        phase: phase_for(&query_tokens.join(" ")),
        energy: query_tokens.len() as u32,
        token_count: query_tokens.len(),
        tokens: query_tokens.clone(),
    };
    let top_k = top_k.max(1);
    let chunk_peaks = score_chunks(&artifact, &query_terms, query_wave.phase, top_k);
    let schema_peaks = score_schemas(&artifact, &query_terms, top_k);
    let transition_peaks = score_transitions(&artifact, &query_terms, top_k);
    let support_score = chunk_peaks
        .first()
        .map(|peak| peak.score)
        .unwrap_or(0.0)
        .max(schema_peaks.first().map(|peak| peak.score).unwrap_or(0.0));
    let chunk_margin = chunk_peaks.first().map(|peak| peak.score).unwrap_or(0.0)
        - chunk_peaks.get(1).map(|peak| peak.score).unwrap_or(0.0);
    let schema_margin = schema_peaks.first().map(|peak| peak.score).unwrap_or(0.0)
        - schema_peaks.get(1).map(|peak| peak.score).unwrap_or(0.0);
    let _second_score = chunk_peaks
        .get(1)
        .map(|peak| peak.score)
        .unwrap_or(0.0)
        .max(schema_peaks.get(1).map(|peak| peak.score).unwrap_or(0.0));
    let margin = round4(chunk_margin.max(schema_margin));
    let focused_schema = schema_peaks
        .first()
        .is_some_and(|peak| peak.score >= 1.5 && schema_margin >= 0.1 && peak.count >= 2);
    let field_state = if focused_schema {
        "TRAINED_FIELD_FOCUSED"
    } else if chunk_peaks
        .first()
        .is_some_and(|peak| peak.score >= 1.5 && chunk_margin >= 0.1)
    {
        "TRAINED_FIELD_EVIDENCE_REVIEW"
    } else if support_score > 0.0 {
        "TRAINED_FIELD_THIN"
    } else {
        "TRAINED_FIELD_NO_MATCH"
    };
    let answer = build_artifact_answer(field_state, &chunk_peaks, &schema_peaks);

    Ok(ArtifactAskReport {
        mode: "llmwave-big-ask",
        version: "llmwave-big-v1902-artifact-ask",
        verdict: if answer.safe_to_answer {
            "ARTIFACT_FIELD_ANSWER_READY_NOT_GENERAL_LLM"
        } else {
            "ARTIFACT_FIELD_REVIEW"
        },
        query,
        artifact: ArtifactAskSummary {
            path: artifact_path.display().to_string(),
            corpus_tokens: artifact.corpus.tokens_seen,
            token_records: artifact.records.tokens.len(),
            transition_records: artifact.records.transitions.len(),
            chunk_records: artifact.records.chunks.len(),
            schema_hint_records: artifact.records.schema_hints.len(),
            estimated_hot_bytes: artifact.field_budget.estimated_hot_bytes,
            fits_hot_budget: artifact.field_budget.fits_hot_budget,
        },
        query_wave,
        field: ArtifactFieldSummary {
            state: field_state,
            top_chunk_peaks: chunk_peaks,
            top_schema_peaks: schema_peaks,
            top_transition_peaks: transition_peaks,
            support_score,
            margin,
        },
        answer,
        claim_boundary: ArtifactAskClaimBoundary {
            artifact_loaded: true,
            trained_field_used: true,
            generated_from_training_artifact: true,
            broad_chat_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "LLMWave-Big can answer narrowly by retrieving peaks from a compiled training artifact.",
        },
    })
}

pub(crate) fn eval_training_artifact(
    artifact_path: &Path,
    suite_path: &Path,
    top_k: usize,
) -> Result<ArtifactAskEvalReport> {
    let raw = fs::read_to_string(suite_path)
        .with_context(|| format!("read ask eval suite {}", suite_path.display()))?;
    let suite: ArtifactAskEvalSuite = serde_json::from_str(&raw)
        .with_context(|| format!("parse ask eval suite {}", suite_path.display()))?;
    let mut cases = Vec::new();
    let mut false_positive = 0usize;
    let mut false_negative = 0usize;
    for case in suite.cases {
        let observed = ask_training_artifact(artifact_path, case.query.clone(), top_k)?;
        let contains_ok = case.expected_contains.is_empty()
            || observed
                .answer
                .text
                .to_lowercase()
                .contains(&case.expected_contains.to_lowercase());
        let safe_ok = observed.answer.safe_to_answer == case.expected_safe_to_answer;
        if observed.answer.safe_to_answer && !case.expected_safe_to_answer {
            false_positive += 1;
        }
        if !observed.answer.safe_to_answer && case.expected_safe_to_answer {
            false_negative += 1;
        }
        cases.push(ArtifactAskEvalCaseReport {
            id: case.id,
            query: observed.query,
            expected_contains: case.expected_contains,
            expected_safe_to_answer: case.expected_safe_to_answer,
            observed_state: observed.field.state,
            observed_safe_to_answer: observed.answer.safe_to_answer,
            observed_answer: observed.answer.text,
            passed: contains_ok && safe_ok,
        });
    }
    let total = cases.len();
    let passed = cases.iter().filter(|case| case.passed).count();
    let verdict = if total > 0 && passed == total {
        "ARTIFACT_ASK_EVAL_PASS_NOT_GENERAL_LLM"
    } else {
        "ARTIFACT_ASK_EVAL_REVIEW"
    };
    Ok(ArtifactAskEvalReport {
        mode: "llmwave-big-ask-eval",
        version: "llmwave-big-v1903-artifact-ask-eval",
        verdict,
        artifact: artifact_path.display().to_string(),
        suite: suite_path.display().to_string(),
        cases,
        metrics: ArtifactAskEvalMetrics {
            total,
            passed,
            answer_accuracy: ratio(passed, total),
            false_positive_rate: ratio(false_positive, total),
            false_negative_rate: ratio(false_negative, total),
        },
        claim_boundary: ArtifactAskClaimBoundary {
            artifact_loaded: true,
            trained_field_used: true,
            generated_from_training_artifact: true,
            broad_chat_llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "LLMWave-Big can run an artifact-grounded ask eval suite without claiming broad LLM readiness.",
        },
    })
}

fn load_corpus(config: &TrainingConfig) -> Result<CorpusBuild> {
    let mut paths = Vec::new();
    for input in &config.inputs {
        collect_paths(input, &config.extensions, &mut paths)?;
    }
    paths.sort();

    let mut seen_hashes = HashSet::new();
    let mut files = Vec::new();
    let mut duplicate_files_skipped = 0usize;
    let mut corpus_hasher = Sha256::new();
    for path in &paths {
        let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
        if metadata.len() as usize > config.max_file_bytes {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        if text.trim().is_empty() {
            continue;
        }
        let hash = short_hash(text.as_bytes());
        if !seen_hashes.insert(hash.clone()) {
            duplicate_files_skipped += 1;
            continue;
        }
        corpus_hasher.update(hash.as_bytes());
        corpus_hasher.update(path.display().to_string().as_bytes());
        let id = files.len() as u32;
        files.push(LoadedFile {
            id,
            bytes: text.len(),
            lines: text.lines().count(),
            text,
        });
    }
    if files.is_empty() {
        bail!("no usable UTF-8 corpus files found");
    }

    Ok(CorpusBuild {
        files_seen: paths.len(),
        files,
        duplicate_files_skipped,
        corpus_hash: format!("{:x}", corpus_hasher.finalize()),
    })
}

fn collect_paths(path: &Path, extensions: &[String], out: &mut Vec<PathBuf>) -> Result<()> {
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if metadata.is_file() {
        if is_allowed_extension(path, extensions) {
            out.push(path.to_path_buf());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(path).with_context(|| format!("read dir {}", path.display()))? {
        let entry = entry.with_context(|| format!("read dir entry {}", path.display()))?;
        let child = entry.path();
        let name = child.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name == ".git" || name == "target" || name == "node_modules" || name == ".nanda" {
            continue;
        }
        collect_paths(&child, extensions, out)?;
    }
    Ok(())
}

fn is_allowed_extension(path: &Path, extensions: &[String]) -> bool {
    let Some(ext) = path.extension().and_then(|v| v.to_str()) else {
        return false;
    };
    extensions.iter().any(|allowed| allowed == ext)
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            current.extend(ch.to_lowercase());
        } else {
            flush_token(&mut current, &mut tokens);
            if matches!(
                ch,
                '.' | ',' | ';' | ':' | '?' | '!' | '(' | ')' | '[' | ']'
            ) {
                tokens.push(ch.to_string());
            }
        }
    }
    flush_token(&mut current, &mut tokens);
    tokens
}

fn flush_token(current: &mut String, tokens: &mut Vec<String>) {
    if current.chars().count() >= 2 {
        tokens.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

fn is_word_token(token: &str) -> bool {
    token.chars().any(char::is_alphanumeric)
}

fn collect_chunks(
    file: &LoadedFile,
    tokens: &[String],
    chunk_tokens: usize,
    active_chunk_cap: usize,
    seen_chunks: &mut HashSet<String>,
    chunks: &mut Vec<ChunkRecord>,
    held_out: &mut Vec<Vec<String>>,
) {
    for (idx, window) in tokens.chunks(chunk_tokens).enumerate() {
        if window.len() < 8 {
            continue;
        }
        let text = window.join(" ");
        let hash = short_hash(text.as_bytes());
        if !seen_chunks.insert(hash.clone()) {
            continue;
        }
        if idx % 11 == 0 {
            held_out.push(window.to_vec());
            continue;
        }
        if chunks.len() >= active_chunk_cap {
            continue;
        }
        let phase = phase_for(&hash);
        chunks.push(ChunkRecord {
            id: chunks.len() as u32,
            file_id: file.id,
            token_count: window.len(),
            hash,
            centroid_phase: phase,
            energy: window.len() as u32,
            text: text.chars().take(320).collect(),
        });
    }
}

fn build_token_records(counts: BTreeMap<String, u32>, cap: usize) -> Vec<TokenRecord> {
    let mut rows = counts
        .into_iter()
        .filter(|(token, _)| is_word_token(token))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter()
        .take(cap)
        .enumerate()
        .map(|(idx, (token, count))| {
            let phase = phase_for(&token);
            TokenRecord {
                id: idx as u32,
                polarity: if phase.is_multiple_of(2) { 1 } else { -1 },
                token,
                count,
                phase,
            }
        })
        .collect()
}

fn build_transition_records(
    counts: &BTreeMap<(String, String), u32>,
    tokens: &[TokenRecord],
    cap: usize,
) -> Vec<TransitionRecord> {
    let vocab = tokens
        .iter()
        .map(|record| record.token.as_str())
        .collect::<BTreeSet<_>>();
    let mut rows = counts
        .iter()
        .filter(|((from, to), _)| vocab.contains(from.as_str()) && vocab.contains(to.as_str()))
        .filter(|((from, to), _)| is_word_token(from) && is_word_token(to))
        .map(|((from, to), count)| (from.clone(), to.clone(), *count))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });
    rows.into_iter()
        .take(cap)
        .enumerate()
        .map(|(idx, (from, to, count))| {
            let from_phase = phase_for(&from) as i32;
            let to_phase = phase_for(&to) as i32;
            let mut delta = to_phase - from_phase;
            if delta > 32767 {
                delta -= 65536;
            }
            if delta < -32768 {
                delta += 65536;
            }
            TransitionRecord {
                id: idx as u32,
                from,
                to,
                count,
                phase_delta: delta as i16,
                score: round4((count as f32).ln_1p()),
            }
        })
        .collect()
}

fn build_schema_hints(
    counts: &BTreeMap<(String, String, String), u32>,
    cap: usize,
) -> Vec<SchemaHint> {
    let relation_words = [
        "is",
        "are",
        "was",
        "were",
        "имеет",
        "это",
        "requires",
        "uses",
        "builds",
        "writes",
        "reads",
        "checks",
        "rejects",
        "accepts",
        "должен",
        "может",
        "проверяет",
    ];
    let mut rows = counts
        .iter()
        .filter(|((subject, relation, object), _)| {
            is_word_token(subject)
                && is_word_token(relation)
                && is_word_token(object)
                && relation_words.contains(&relation.as_str())
        })
        .map(|((subject, relation, object), count)| {
            (subject.clone(), relation.clone(), object.clone(), *count)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.3.cmp(&a.3)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    let mut hints = Vec::new();
    for (subject, relation, object, count) in rows.into_iter().take(cap) {
        if !is_word_token(&subject)
            || !is_word_token(&relation)
            || !relation_words.contains(&relation.as_str())
        {
            continue;
        }
        if !is_word_token(&object) {
            continue;
        }
        hints.push(SchemaHint {
            id: hints.len() as u32,
            subject,
            relation,
            object,
            count,
            route: "corpus-induced".to_string(),
        });
        if hints.len() >= cap {
            return hints;
        }
    }
    hints
}

fn build_field_budget(
    hot_budget_bytes: usize,
    token_records: usize,
    transition_records: usize,
    chunk_records: usize,
) -> FieldBudget {
    let token_record_bytes = 16;
    let transition_record_bytes = 16;
    let chunk_record_bytes = 32;
    let estimated_hot_bytes = token_records * token_record_bytes
        + transition_records * transition_record_bytes
        + chunk_records * chunk_record_bytes;
    FieldBudget {
        wave_dim: WAVE_DIM,
        hot_budget_bytes,
        token_record_bytes,
        transition_record_bytes,
        chunk_record_bytes,
        token_records,
        transition_records,
        chunk_records,
        estimated_hot_bytes,
        fits_hot_budget: estimated_hot_bytes <= hot_budget_bytes,
    }
}

fn eval_next_token(
    held_out: &[Vec<String>],
    transitions: &[TransitionRecord],
) -> TrainingEvalSummary {
    let mut map: BTreeMap<&str, Vec<&TransitionRecord>> = BTreeMap::new();
    for transition in transitions {
        map.entry(&transition.from).or_default().push(transition);
    }
    let mut cases = 0usize;
    let mut passed = 0usize;
    let mut margin_sum = 0.0f32;
    for chunk in held_out.iter().take(2048) {
        for pair in chunk.windows(2).take(8) {
            let [from, expected] = pair else {
                continue;
            };
            let Some(candidates) = map.get(from.as_str()) else {
                continue;
            };
            cases += 1;
            let top = candidates
                .first()
                .map(|record| record.to.as_str())
                .unwrap_or("");
            if top == expected {
                passed += 1;
            }
            let top_score = candidates.first().map(|record| record.score).unwrap_or(0.0);
            let second_score = candidates.get(1).map(|record| record.score).unwrap_or(0.0);
            margin_sum += top_score - second_score;
        }
    }
    let next_token_accuracy = if cases == 0 {
        0.0
    } else {
        round4(passed as f32 / cases as f32)
    };
    let avg_rank_margin = if cases == 0 {
        0.0
    } else {
        round4(margin_sum / cases as f32)
    };
    TrainingEvalSummary {
        state: if cases > 0 {
            "HELD_OUT_EVAL_RAN"
        } else {
            "HELD_OUT_EVAL_EMPTY"
        },
        next_token_accuracy,
        held_out_cases: cases,
        held_out_passed: passed,
        avg_rank_margin,
        notes: vec![
            "next-token eval is a baseline resonance check, not chat quality",
            "large corpus quality is judged by held-out lift and field stability",
        ],
    }
}

fn score_chunks(
    artifact: &TrainingArtifact,
    query_set: &BTreeSet<String>,
    query_phase: u16,
    top_k: usize,
) -> Vec<ChunkPeak> {
    let mut peaks = artifact
        .records
        .chunks
        .iter()
        .filter_map(|chunk| {
            let chunk_tokens = tokenize(&chunk.text);
            let overlap = chunk_tokens
                .iter()
                .filter(|token| query_set.contains(token.as_str()))
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                .len();
            if overlap == 0 {
                return None;
            }
            let phase_alignment = phase_alignment(query_phase, chunk.centroid_phase);
            let score =
                round4(overlap as f32 + phase_alignment + (chunk.energy as f32).ln_1p() * 0.05);
            Some(ChunkPeak {
                chunk_id: chunk.id,
                score,
                overlap,
                phase_alignment,
                text: chunk.text.clone(),
            })
        })
        .collect::<Vec<_>>();
    peaks.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.overlap.cmp(&a.overlap))
            .then_with(|| a.chunk_id.cmp(&b.chunk_id))
    });
    peaks.truncate(top_k);
    peaks
}

fn score_schemas(
    artifact: &TrainingArtifact,
    query_set: &BTreeSet<String>,
    top_k: usize,
) -> Vec<SchemaPeak> {
    let mut peaks = artifact
        .records
        .schema_hints
        .iter()
        .filter_map(|schema| {
            let mut overlap = 0usize;
            for part in [&schema.subject, &schema.relation, &schema.object] {
                if query_set.contains(part.as_str()) {
                    overlap += 1;
                }
            }
            if overlap == 0 {
                return None;
            }
            let score = round4(overlap as f32 + (schema.count as f32).ln_1p() * 0.25);
            Some(SchemaPeak {
                schema_id: schema.id,
                score,
                subject: schema.subject.clone(),
                relation: schema.relation.clone(),
                object: schema.object.clone(),
                count: schema.count,
            })
        })
        .collect::<Vec<_>>();
    peaks.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.schema_id.cmp(&b.schema_id))
    });
    peaks.truncate(top_k);
    peaks
}

fn score_transitions(
    artifact: &TrainingArtifact,
    query_set: &BTreeSet<String>,
    top_k: usize,
) -> Vec<TransitionPeak> {
    let mut peaks = artifact
        .records
        .transitions
        .iter()
        .filter_map(|transition| {
            let overlap = usize::from(query_set.contains(transition.from.as_str()))
                + usize::from(query_set.contains(transition.to.as_str()));
            if overlap == 0 {
                return None;
            }
            let score = round4(overlap as f32 + transition.score * 0.2);
            Some(TransitionPeak {
                transition_id: transition.id,
                score,
                from: transition.from.clone(),
                to: transition.to.clone(),
                count: transition.count,
            })
        })
        .collect::<Vec<_>>();
    peaks.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.transition_id.cmp(&b.transition_id))
    });
    peaks.truncate(top_k);
    peaks
}

fn build_artifact_answer(
    field_state: &'static str,
    chunk_peaks: &[ChunkPeak],
    schema_peaks: &[SchemaPeak],
) -> ArtifactAnswer {
    let safe_to_answer = field_state == "TRAINED_FIELD_FOCUSED"
        && (!chunk_peaks.is_empty() || !schema_peaks.is_empty());
    if !safe_to_answer {
        return ArtifactAnswer {
            state: "ANSWER_REVIEW",
            safe_to_answer: false,
            text: "LLMWave-Big did not find a stable trained-artifact peak.".to_string(),
            evidence_chunks: chunk_peaks.iter().map(|peak| peak.chunk_id).collect(),
            evidence_schemas: schema_peaks.iter().map(|peak| peak.schema_id).collect(),
        };
    }

    let mut parts = Vec::new();
    if let Some(schema) = schema_peaks.first() {
        parts.push(format!(
            "{} {} {}",
            schema.subject, schema.relation, schema.object
        ));
    }
    if parts.is_empty() {
        if let Some(chunk) = chunk_peaks.first() {
            parts.push(chunk.text.clone());
        }
    } else if let Some(chunk) = chunk_peaks.first() {
        parts.push(format!("evidence chunk {}", chunk.chunk_id));
    }
    ArtifactAnswer {
        state: "ANSWER_FROM_TRAINED_ARTIFACT",
        safe_to_answer: true,
        text: parts.join(" | "),
        evidence_chunks: chunk_peaks.iter().map(|peak| peak.chunk_id).collect(),
        evidence_schemas: schema_peaks.iter().map(|peak| peak.schema_id).collect(),
    }
}

fn phase_alignment(a: u16, b: u16) -> f32 {
    let diff = a.abs_diff(b).min(65535 - a.abs_diff(b)) as f32;
    round4(1.0 - diff / 32767.0)
}

fn expand_query_terms(tokens: &[String]) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for token in tokens {
        terms.insert(token.clone());
        if token.ends_with('s') && token.len() > 3 {
            terms.insert(token.trim_end_matches('s').to_string());
        } else if token.len() > 3 {
            terms.insert(format!("{token}s"));
        }
        if token.ends_with("es") && token.len() > 4 {
            terms.insert(token.trim_end_matches("es").to_string());
        } else if token.len() > 3 {
            terms.insert(format!("{token}es"));
        }
    }
    terms
}

fn phase_for(input: &str) -> u16 {
    let digest = Sha256::digest(input.as_bytes());
    u16::from_le_bytes([digest[0], digest[1]])
}

fn short_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn round4(value: f32) -> f32 {
    (value * 10_000.0).round() / 10_000.0
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        round4(numerator as f32 / denominator as f32)
    }
}

pub(crate) fn default_extensions_csv() -> String {
    DEFAULT_EXTENSIONS.join(",")
}

pub(crate) fn parse_extensions(csv: &str) -> Vec<String> {
    csv.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
