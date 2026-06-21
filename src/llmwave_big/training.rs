//! Corpus compiler for LLMWave-Big.
//!
//! This is cold training code. It may allocate, parse text, and write JSON
//! artifacts. Hot Active Core code consumes the compact records produced here.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::io::{BufWriter, Cursor, Read, Write};
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
    #[serde(default = "default_schema_record_bytes")]
    pub schema_record_bytes: usize,
    pub token_records: usize,
    pub transition_records: usize,
    pub chunk_records: usize,
    #[serde(default)]
    pub schema_records: usize,
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

#[derive(Serialize, Clone)]
pub(crate) struct HotPackReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub artifact: String,
    pub out: String,
    pub record_counts: HotPackRecordCounts,
    pub bytes: HotPackBytes,
    pub claim_boundary: HotPackClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotPackRecordCounts {
    pub tokens: usize,
    pub transitions: usize,
    pub chunks: usize,
    pub schema_hints: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotPackBytes {
    pub actual_file_bytes: usize,
    pub estimated_hot_bytes: usize,
    pub hot_budget_bytes: usize,
    pub fits_hot_budget: bool,
    pub header_bytes: usize,
    pub record_bytes: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotPackClaimBoundary {
    pub binary_hot_pack_written: bool,
    pub strings_excluded_from_hot_pack: bool,
    pub json_excluded_from_hot_pack: bool,
    pub cache_only_execution_proven: bool,
    pub broad_chat_llm_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotAskReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub query: String,
    pub hot_pack: HotAskPackSummary,
    pub learning: HotAskLearningSummary,
    pub field: HotAskField,
    pub answer: HotAskAnswer,
    pub claim_boundary: HotAskClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotAskPackSummary {
    pub path: String,
    pub artifact_path: String,
    pub bytes_scanned: usize,
    pub tokens: usize,
    pub transitions: usize,
    pub chunks: usize,
    pub schema_hints: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotAskLearningSummary {
    pub memory_loaded: bool,
    pub memory_path: Option<String>,
    pub learned_records: usize,
    pub accepted_records: usize,
    pub rejected_records: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotAskField {
    pub state: &'static str,
    pub query_hashes: Vec<u32>,
    pub top_hot_schema_peaks: Vec<HotSchemaPeak>,
    pub top_hot_transition_peaks: Vec<HotTransitionPeak>,
    pub margin: f32,
    pub polarity_state: &'static str,
    pub polarity_penalty: f32,
    pub anti_polarity_energy: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotSchemaPeak {
    pub score: f32,
    pub raw_score: f32,
    pub matched_terms: usize,
    pub subject_matched: bool,
    pub relation_matched: bool,
    pub object_matched: bool,
    pub polarization: HotSchemaPolarization,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub count: u32,
    pub source: String,
    pub learned: bool,
    pub accepted_count: u32,
    pub rejected_count: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotSchemaPolarization {
    pub state: &'static str,
    pub role_order: &'static str,
    pub subject_position: Option<usize>,
    pub object_position: Option<usize>,
    pub penalty: f32,
    pub anti_energy: f32,
    pub hard_stop: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotTransitionPeak {
    pub score: f32,
    pub matched_terms: usize,
    pub from: String,
    pub to: String,
    pub count: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotAskAnswer {
    pub state: &'static str,
    pub safe_to_answer: bool,
    pub text: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotAskClaimBoundary {
    pub binary_hot_pack_loaded: bool,
    pub hot_records_scanned: bool,
    pub polarity_lens_applied: bool,
    pub reversed_polarity_hard_stop: bool,
    pub hot_memory_loaded: bool,
    pub learning_overlay_applied: bool,
    pub cold_artifact_used_for_labels: bool,
    pub json_used_in_hot_scan: bool,
    pub online_gradient_training: bool,
    pub cache_only_execution_proven: bool,
    pub broad_chat_llm_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotLearnReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub feedback: String,
    pub out: String,
    pub memory: HotLearnMemorySummary,
    pub claim_boundary: HotLearnClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotLearnMemorySummary {
    pub records_written: usize,
    pub accepted_records: usize,
    pub rejected_records: usize,
    pub authority_avg: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct HotLearnClaimBoundary {
    pub batch_feedback_ingested: bool,
    pub persistent_hot_memory_written: bool,
    pub can_change_next_hot_ask: bool,
    pub online_gradient_training: bool,
    pub broad_chat_llm_ready: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct HotMemoryFile {
    mode: String,
    version: String,
    records: Vec<HotMemoryRecord>,
}

#[derive(Serialize, Deserialize, Clone)]
struct HotMemoryRecord {
    subject: String,
    relation: String,
    object: String,
    decision: String,
    authority: f32,
    accepted_count: u32,
    rejected_count: u32,
    source: String,
}

#[derive(Deserialize)]
struct HotFeedbackBatch {
    events: Vec<HotFeedbackEvent>,
}

#[derive(Deserialize)]
struct HotFeedbackEvent {
    decision: String,
    subject: String,
    relation: String,
    object: String,
    #[serde(default = "default_feedback_authority")]
    authority: f32,
    #[serde(default = "default_feedback_source")]
    source: String,
}

struct HotPackImage {
    counts: HotPackRecordCounts,
    transitions: Vec<HotTransitionRecord>,
    schema_hints: Vec<HotSchemaRecord>,
}

struct HotTransitionRecord {
    from_hash: u32,
    to_hash: u32,
    count: u32,
}

struct HotSchemaRecord {
    subject_hash: u32,
    relation_hash: u32,
    object_hash: u32,
    count: u32,
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
        schema_hints.len(),
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

pub(crate) fn pack_hot_artifact(artifact_path: &Path, out: &Path) -> Result<HotPackReport> {
    let artifact = load_training_artifact(artifact_path)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create hot pack directory {}", parent.display()))?;
    }

    let mut writer = BufWriter::new(
        fs::File::create(out).with_context(|| format!("create hot pack {}", out.display()))?,
    );
    let counts = HotPackRecordCounts {
        tokens: artifact.records.tokens.len(),
        transitions: artifact.records.transitions.len(),
        chunks: artifact.records.chunks.len(),
        schema_hints: artifact.records.schema_hints.len(),
    };
    write_hot_header(&mut writer, &counts)?;
    for record in &artifact.records.tokens {
        write_u32(&mut writer, hash32(&record.token))?;
        write_u32(&mut writer, record.count)?;
        write_u16(&mut writer, record.phase)?;
        writer.write_all(&record.polarity.to_le_bytes())?;
        writer.write_all(&[0; 5])?;
    }
    for record in &artifact.records.transitions {
        write_u32(&mut writer, hash32(&record.from))?;
        write_u32(&mut writer, hash32(&record.to))?;
        write_u32(&mut writer, record.count)?;
        write_i16(&mut writer, record.phase_delta)?;
        writer.write_all(&[0; 2])?;
    }
    for record in &artifact.records.chunks {
        write_u64(&mut writer, chunk_hash64(&record.hash))?;
        write_u16(&mut writer, record.centroid_phase)?;
        write_u32(&mut writer, record.energy)?;
        write_u16(
            &mut writer,
            record.token_count.min(u16::MAX as usize) as u16,
        )?;
        write_u32(&mut writer, record.file_id)?;
        write_u32(&mut writer, record.id)?;
        writer.write_all(&[0; 8])?;
    }
    for record in &artifact.records.schema_hints {
        write_u32(&mut writer, hash32(&record.subject))?;
        write_u32(&mut writer, hash32(&record.relation))?;
        write_u32(&mut writer, hash32(&record.object))?;
        write_u32(&mut writer, record.count)?;
    }
    writer.flush()?;

    let actual_file_bytes = fs::metadata(out)
        .with_context(|| format!("stat hot pack {}", out.display()))?
        .len() as usize;
    let header_bytes = hot_header_bytes();
    let record_bytes = counts.tokens * 16
        + counts.transitions * 16
        + counts.chunks * 32
        + counts.schema_hints * 16;
    Ok(HotPackReport {
        mode: "llmwave-big-pack-hot",
        version: "llmwave-big-v1904-hot-pack",
        verdict: if actual_file_bytes <= artifact.field_budget.hot_budget_bytes {
            "HOT_PACK_READY_NOT_CACHE_ONLY_PROOF"
        } else {
            "HOT_PACK_EXCEEDS_HOT_BUDGET"
        },
        artifact: artifact_path.display().to_string(),
        out: out.display().to_string(),
        record_counts: counts,
        bytes: HotPackBytes {
            actual_file_bytes,
            estimated_hot_bytes: record_bytes,
            hot_budget_bytes: artifact.field_budget.hot_budget_bytes,
            fits_hot_budget: actual_file_bytes <= artifact.field_budget.hot_budget_bytes,
            header_bytes,
            record_bytes,
        },
        claim_boundary: HotPackClaimBoundary {
            binary_hot_pack_written: true,
            strings_excluded_from_hot_pack: true,
            json_excluded_from_hot_pack: true,
            cache_only_execution_proven: false,
            broad_chat_llm_ready: false,
        },
    })
}

pub(crate) fn ask_hot_pack(
    hot_pack_path: &Path,
    artifact_path: &Path,
    query: String,
    top_k: usize,
    memory_path: Option<&Path>,
) -> Result<HotAskReport> {
    let hot_bytes = fs::read(hot_pack_path)
        .with_context(|| format!("read hot pack {}", hot_pack_path.display()))?;
    let hot = parse_hot_pack(&hot_bytes)?;
    let artifact = load_training_artifact(artifact_path)?;
    let memory = match memory_path {
        Some(path) => Some(load_hot_memory(path)?),
        None => None,
    };
    let query_tokens = tokenize(&query)
        .into_iter()
        .filter(|token| is_word_token(token))
        .collect::<Vec<_>>();
    let query_hashes = query_tokens
        .iter()
        .map(|token| hash32(token))
        .collect::<Vec<_>>();
    let query_hash_set = query_hashes.iter().copied().collect::<BTreeSet<_>>();
    let query_positions = build_query_hash_positions(&query_hashes);
    let top_k = top_k.max(1);
    let mut schema_peaks = score_hot_schemas(&hot, &artifact, &query_positions, top_k);
    if let Some(memory) = &memory {
        schema_peaks.extend(score_hot_memory_schemas(memory, &query_positions, top_k));
        sort_hot_schema_peaks(&mut schema_peaks);
        schema_peaks.truncate(top_k);
    }
    let transition_peaks = score_hot_transitions(&hot, &artifact, &query_hash_set, top_k);
    let top_score = schema_peaks
        .first()
        .map(|peak| peak.score)
        .unwrap_or(0.0)
        .max(
            transition_peaks
                .first()
                .map(|peak| peak.score)
                .unwrap_or(0.0),
        );
    let second_score = schema_peaks
        .get(1)
        .map(|peak| peak.score)
        .unwrap_or(0.0)
        .max(
            transition_peaks
                .get(1)
                .map(|peak| peak.score)
                .unwrap_or(0.0),
        );
    let margin = round4(top_score - second_score);
    let focused_schema = schema_peaks.first().is_some_and(|peak| {
        peak.subject_matched
            && peak.matched_terms >= 2
            && peak.score >= 2.0
            && margin >= 0.1
            && !peak.polarization.hard_stop
    });
    let polarity_peak = if focused_schema {
        schema_peaks.first()
    } else {
        schema_peaks
            .iter()
            .find(|peak| peak.polarization.state == "REVERSED")
            .or_else(|| {
                schema_peaks
                    .iter()
                    .find(|peak| peak.polarization.state == "OBJECT_FOREIGN_PULL")
            })
            .or_else(|| schema_peaks.first())
    };
    let top_polarization = polarity_peak
        .map(|peak| peak.polarization.state)
        .unwrap_or("NONE");
    let polarity_penalty = polarity_peak
        .map(|peak| peak.polarization.penalty)
        .unwrap_or(0.0);
    let anti_polarity_energy = polarity_peak
        .map(|peak| peak.polarization.anti_energy)
        .unwrap_or(0.0);
    let field_state = if focused_schema {
        "HOT_FIELD_SCHEMA_FOCUSED"
    } else if top_polarization == "REVERSED" {
        "HOT_FIELD_POLARITY_REVERSED"
    } else if top_polarization == "OBJECT_FOREIGN_PULL" {
        "HOT_FIELD_POLARITY_FOREIGN_PULL"
    } else if top_score > 0.0 {
        "HOT_FIELD_REVIEW"
    } else {
        "HOT_FIELD_NO_MATCH"
    };
    let answer = build_hot_answer(field_state, &schema_peaks, &transition_peaks);
    Ok(HotAskReport {
        mode: "llmwave-big-ask-hot",
        version: "llmwave-big-v1905-hot-ask",
        verdict: if answer.safe_to_answer {
            "HOT_FIELD_ANSWER_READY_NOT_GENERAL_LLM"
        } else {
            "HOT_FIELD_REVIEW"
        },
        query,
        hot_pack: HotAskPackSummary {
            path: hot_pack_path.display().to_string(),
            artifact_path: artifact_path.display().to_string(),
            bytes_scanned: hot_bytes.len(),
            tokens: hot.counts.tokens,
            transitions: hot.counts.transitions,
            chunks: hot.counts.chunks,
            schema_hints: hot.counts.schema_hints,
        },
        learning: hot_learning_summary(memory_path, memory.as_ref()),
        field: HotAskField {
            state: field_state,
            query_hashes,
            top_hot_schema_peaks: schema_peaks,
            top_hot_transition_peaks: transition_peaks,
            margin,
            polarity_state: top_polarization,
            polarity_penalty,
            anti_polarity_energy,
        },
        answer,
        claim_boundary: HotAskClaimBoundary {
            binary_hot_pack_loaded: true,
            hot_records_scanned: true,
            polarity_lens_applied: true,
            reversed_polarity_hard_stop: true,
            hot_memory_loaded: memory.is_some(),
            learning_overlay_applied: memory.is_some(),
            cold_artifact_used_for_labels: true,
            json_used_in_hot_scan: false,
            online_gradient_training: false,
            cache_only_execution_proven: false,
            broad_chat_llm_ready: false,
        },
    })
}

pub(crate) fn learn_hot_memory(feedback_path: &Path, out: &Path) -> Result<HotLearnReport> {
    let raw = fs::read_to_string(feedback_path)
        .with_context(|| format!("read hot feedback {}", feedback_path.display()))?;
    let batch: HotFeedbackBatch = serde_json::from_str(&raw)
        .with_context(|| format!("parse hot feedback {}", feedback_path.display()))?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create hot memory directory {}", parent.display()))?;
    }
    let mut merged = BTreeMap::<(String, String, String, String), HotMemoryRecord>::new();
    for event in batch.events {
        let decision = normalize_decision(&event.decision)?;
        let subject = normalize_feedback_text(&event.subject);
        let relation = normalize_feedback_text(&event.relation);
        let object = normalize_feedback_text(&event.object);
        let key = (
            subject.clone(),
            relation.clone(),
            object.clone(),
            decision.clone(),
        );
        let entry = merged.entry(key).or_insert_with(|| HotMemoryRecord {
            subject,
            relation,
            object,
            decision,
            authority: event.authority.clamp(0.0, 1.0),
            accepted_count: 0,
            rejected_count: 0,
            source: event.source,
        });
        entry.authority = entry.authority.max(event.authority.clamp(0.0, 1.0));
        if entry.decision == "accept" {
            entry.accepted_count += 1;
        } else {
            entry.rejected_count += 1;
        }
    }
    let records = merged.into_values().collect::<Vec<_>>();
    let memory = HotMemoryFile {
        mode: "llmwave-big-hot-memory".to_string(),
        version: "llmwave-big-v1906-hot-learning-memory".to_string(),
        records: records.clone(),
    };
    fs::write(out, serde_json::to_string_pretty(&memory)? + "\n")
        .with_context(|| format!("write hot memory {}", out.display()))?;
    let accepted_records = records
        .iter()
        .filter(|record| record.decision == "accept")
        .count();
    let rejected_records = records
        .iter()
        .filter(|record| record.decision == "reject")
        .count();
    let authority_avg = if records.is_empty() {
        0.0
    } else {
        round4(records.iter().map(|record| record.authority).sum::<f32>() / records.len() as f32)
    };
    Ok(HotLearnReport {
        mode: "llmwave-big-learn-hot",
        version: "llmwave-big-v1906-hot-learning-memory",
        verdict: if records.is_empty() {
            "HOT_LEARNING_MEMORY_EMPTY"
        } else {
            "HOT_LEARNING_MEMORY_WRITTEN_NOT_GRADIENT_TRAINING"
        },
        feedback: feedback_path.display().to_string(),
        out: out.display().to_string(),
        memory: HotLearnMemorySummary {
            records_written: records.len(),
            accepted_records,
            rejected_records,
            authority_avg,
        },
        claim_boundary: HotLearnClaimBoundary {
            batch_feedback_ingested: true,
            persistent_hot_memory_written: true,
            can_change_next_hot_ask: !records.is_empty(),
            online_gradient_training: false,
            broad_chat_llm_ready: false,
        },
    })
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
    schema_records: usize,
) -> FieldBudget {
    let token_record_bytes = 16;
    let transition_record_bytes = 16;
    let chunk_record_bytes = 32;
    let schema_record_bytes = default_schema_record_bytes();
    let estimated_hot_bytes = token_records * token_record_bytes
        + transition_records * transition_record_bytes
        + chunk_records * chunk_record_bytes
        + schema_records * schema_record_bytes;
    FieldBudget {
        wave_dim: WAVE_DIM,
        hot_budget_bytes,
        token_record_bytes,
        transition_record_bytes,
        chunk_record_bytes,
        schema_record_bytes,
        token_records,
        transition_records,
        chunk_records,
        schema_records,
        estimated_hot_bytes,
        fits_hot_budget: estimated_hot_bytes <= hot_budget_bytes,
    }
}

fn default_schema_record_bytes() -> usize {
    16
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

fn build_hot_answer(
    field_state: &'static str,
    schema_peaks: &[HotSchemaPeak],
    transition_peaks: &[HotTransitionPeak],
) -> HotAskAnswer {
    if field_state == "HOT_FIELD_SCHEMA_FOCUSED" {
        let peak = &schema_peaks[0];
        return HotAskAnswer {
            state: "HOT_SCHEMA_BOUND_ANSWER",
            safe_to_answer: true,
            text: format!(
                "{} {} {} | hot schema peak count {}",
                peak.subject, peak.relation, peak.object, peak.count
            ),
        };
    }
    if field_state == "HOT_FIELD_POLARITY_REVERSED" {
        let peak = schema_peaks
            .iter()
            .find(|peak| peak.polarization.state == "REVERSED")
            .unwrap_or(&schema_peaks[0]);
        return HotAskAnswer {
            state: "HOT_POLARITY_REVERSED_STOP",
            safe_to_answer: false,
            text: format!(
                "Reversed polarity: query order conflicts with {} -> {} -> {}",
                peak.subject, peak.relation, peak.object
            ),
        };
    }
    if field_state == "HOT_FIELD_POLARITY_FOREIGN_PULL" {
        let peak = schema_peaks
            .iter()
            .find(|peak| peak.polarization.state == "OBJECT_FOREIGN_PULL")
            .unwrap_or(&schema_peaks[0]);
        return HotAskAnswer {
            state: "HOT_POLARITY_FOREIGN_PULL_REVIEW",
            safe_to_answer: false,
            text: format!(
                "Review only: query pulls relation/object from {} -> {} -> {} without matching subject",
                peak.subject, peak.relation, peak.object
            ),
        };
    }
    if let Some(peak) = transition_peaks.first() {
        return HotAskAnswer {
            state: "HOT_TRANSITION_REVIEW_ONLY",
            safe_to_answer: false,
            text: format!(
                "Review hot transition: {} -> {} | count {}",
                peak.from, peak.to, peak.count
            ),
        };
    }
    HotAskAnswer {
        state: "HOT_NO_ANSWER",
        safe_to_answer: false,
        text: "No focused hot-pack peak.".to_string(),
    }
}

fn build_query_hash_positions(query_hashes: &[u32]) -> BTreeMap<u32, usize> {
    let mut positions = BTreeMap::new();
    for (idx, hash) in query_hashes.iter().copied().enumerate() {
        positions.entry(hash).or_insert(idx);
    }
    positions
}

fn hot_learning_summary(
    memory_path: Option<&Path>,
    memory: Option<&HotMemoryFile>,
) -> HotAskLearningSummary {
    let learned_records = memory.map(|memory| memory.records.len()).unwrap_or(0);
    let accepted_records = memory
        .map(|memory| {
            memory
                .records
                .iter()
                .filter(|record| record.decision == "accept")
                .count()
        })
        .unwrap_or(0);
    let rejected_records = memory
        .map(|memory| {
            memory
                .records
                .iter()
                .filter(|record| record.decision == "reject")
                .count()
        })
        .unwrap_or(0);
    HotAskLearningSummary {
        memory_loaded: memory.is_some(),
        memory_path: memory_path.map(|path| path.display().to_string()),
        learned_records,
        accepted_records,
        rejected_records,
    }
}

fn load_hot_memory(path: &Path) -> Result<HotMemoryFile> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("read hot memory {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse hot memory {}", path.display()))
}

fn normalize_decision(decision: &str) -> Result<String> {
    match decision.trim().to_lowercase().as_str() {
        "accept" | "accepted" | "reinforce" | "positive" => Ok("accept".to_string()),
        "reject" | "rejected" | "suppress" | "negative" => Ok("reject".to_string()),
        other => bail!("unsupported hot learning decision {other:?}"),
    }
}

fn normalize_feedback_text(value: &str) -> String {
    tokenize(value)
        .into_iter()
        .filter(|token| is_word_token(token))
        .collect::<Vec<_>>()
        .join("-")
}

fn default_feedback_authority() -> f32 {
    1.0
}

fn default_feedback_source() -> String {
    "batch".to_string()
}

fn hot_schema_polarization(
    record: &HotSchemaRecord,
    query_positions: &BTreeMap<u32, usize>,
) -> HotSchemaPolarization {
    let subject_position = query_positions.get(&record.subject_hash).copied();
    let object_position = query_positions.get(&record.object_hash).copied();
    let relation_matched = query_positions.contains_key(&record.relation_hash);
    let (state, role_order, penalty, anti_energy, hard_stop) =
        match (subject_position, object_position) {
            (Some(subject), Some(object)) if subject < object => {
                ("ALIGNED", "subject_before_object", 0.0, 0.0, false)
            }
            (Some(subject), Some(object)) if subject > object => {
                ("REVERSED", "object_before_subject", 2.0, 1.0, true)
            }
            (Some(_), Some(_)) => ("NEUTRAL", "same_position", 0.25, 0.1, false),
            (Some(_), None) => (
                "SUBJECT_ALIGNED_PARTIAL",
                "subject_present_object_absent",
                0.0,
                0.0,
                false,
            ),
            (None, Some(_)) if relation_matched => (
                "OBJECT_FOREIGN_PULL",
                "object_present_subject_absent",
                0.75,
                0.5,
                true,
            ),
            (None, Some(_)) => (
                "OBJECT_ONLY_REVIEW",
                "object_present_subject_absent",
                0.5,
                0.25,
                false,
            ),
            (None, None) => ("NEUTRAL", "roles_absent", 0.0, 0.0, false),
        };
    HotSchemaPolarization {
        state,
        role_order,
        subject_position,
        object_position,
        penalty,
        anti_energy,
        hard_stop,
    }
}

fn score_hot_schemas(
    hot: &HotPackImage,
    artifact: &TrainingArtifact,
    query_positions: &BTreeMap<u32, usize>,
    top_k: usize,
) -> Vec<HotSchemaPeak> {
    let labels = artifact
        .records
        .schema_hints
        .iter()
        .map(|record| {
            (
                (
                    hash32(&record.subject),
                    hash32(&record.relation),
                    hash32(&record.object),
                ),
                record,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut peaks = hot
        .schema_hints
        .iter()
        .filter_map(|record| {
            let matched_terms = [
                record.subject_hash,
                record.relation_hash,
                record.object_hash,
            ]
            .into_iter()
            .filter(|hash| query_positions.contains_key(hash))
            .count();
            if matched_terms == 0 {
                return None;
            }
            let subject_matched = query_positions.contains_key(&record.subject_hash);
            let relation_matched = query_positions.contains_key(&record.relation_hash);
            let object_matched = query_positions.contains_key(&record.object_hash);
            let polarization = hot_schema_polarization(record, query_positions);
            let label = labels.get(&(
                record.subject_hash,
                record.relation_hash,
                record.object_hash,
            ))?;
            let raw_score = round4(
                matched_terms as f32
                    + if subject_matched { 0.75 } else { 0.0 }
                    + (record.count as f32).ln_1p() * 0.2,
            );
            Some(HotSchemaPeak {
                score: round4((raw_score - polarization.penalty).max(0.0)),
                raw_score,
                matched_terms,
                subject_matched,
                relation_matched,
                object_matched,
                polarization,
                subject: label.subject.clone(),
                relation: label.relation.clone(),
                object: label.object.clone(),
                count: record.count,
                source: "hot_pack".to_string(),
                learned: false,
                accepted_count: 0,
                rejected_count: 0,
            })
        })
        .collect::<Vec<_>>();
    sort_hot_schema_peaks(&mut peaks);
    peaks.truncate(top_k);
    peaks
}

fn score_hot_memory_schemas(
    memory: &HotMemoryFile,
    query_positions: &BTreeMap<u32, usize>,
    top_k: usize,
) -> Vec<HotSchemaPeak> {
    let mut peaks = memory
        .records
        .iter()
        .filter_map(|record| {
            let schema = HotSchemaRecord {
                subject_hash: hash32(&record.subject),
                relation_hash: hash32(&record.relation),
                object_hash: hash32(&record.object),
                count: record.accepted_count.max(record.rejected_count).max(1),
            };
            let matched_terms = [
                schema.subject_hash,
                schema.relation_hash,
                schema.object_hash,
            ]
            .into_iter()
            .filter(|hash| query_positions.contains_key(hash))
            .count();
            if matched_terms == 0 {
                return None;
            }
            let subject_matched = query_positions.contains_key(&schema.subject_hash);
            let relation_matched = query_positions.contains_key(&schema.relation_hash);
            let object_matched = query_positions.contains_key(&schema.object_hash);
            let polarization = hot_schema_polarization(&schema, query_positions);
            let signed_memory = if record.decision == "accept" {
                record.authority * (record.accepted_count.max(1) as f32) * 1.5
            } else {
                -(record.authority * (record.rejected_count.max(1) as f32) * 1.5)
            };
            let raw_score = round4(
                matched_terms as f32 + if subject_matched { 0.75 } else { 0.0 } + signed_memory,
            );
            Some(HotSchemaPeak {
                score: round4((raw_score - polarization.penalty).max(0.0)),
                raw_score,
                matched_terms,
                subject_matched,
                relation_matched,
                object_matched,
                polarization,
                subject: record.subject.clone(),
                relation: record.relation.clone(),
                object: record.object.clone(),
                count: record.accepted_count.max(record.rejected_count).max(1),
                source: format!("hot_memory:{}", record.source),
                learned: true,
                accepted_count: record.accepted_count,
                rejected_count: record.rejected_count,
            })
        })
        .collect::<Vec<_>>();
    sort_hot_schema_peaks(&mut peaks);
    peaks.truncate(top_k);
    peaks
}

fn sort_hot_schema_peaks(peaks: &mut [HotSchemaPeak]) {
    peaks.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.learned.cmp(&a.learned))
            .then_with(|| b.matched_terms.cmp(&a.matched_terms))
            .then_with(|| a.subject.cmp(&b.subject))
    });
}

fn score_hot_transitions(
    hot: &HotPackImage,
    artifact: &TrainingArtifact,
    query_hashes: &BTreeSet<u32>,
    top_k: usize,
) -> Vec<HotTransitionPeak> {
    let labels = artifact
        .records
        .transitions
        .iter()
        .map(|record| ((hash32(&record.from), hash32(&record.to)), record))
        .collect::<BTreeMap<_, _>>();
    let mut peaks = hot
        .transitions
        .iter()
        .filter_map(|record| {
            let matched_terms = [record.from_hash, record.to_hash]
                .into_iter()
                .filter(|hash| query_hashes.contains(hash))
                .count();
            if matched_terms == 0 {
                return None;
            }
            let label = labels.get(&(record.from_hash, record.to_hash))?;
            Some(HotTransitionPeak {
                score: round4(matched_terms as f32 + (record.count as f32).ln_1p() * 0.1),
                matched_terms,
                from: label.from.clone(),
                to: label.to.clone(),
                count: record.count,
            })
        })
        .collect::<Vec<_>>();
    peaks.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.matched_terms.cmp(&a.matched_terms))
            .then_with(|| a.from.cmp(&b.from))
    });
    peaks.truncate(top_k);
    peaks
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
        let text = if let Some(chunk) = chunk_peaks.first() {
            format!(
                "Review evidence from chunk {}: {}",
                chunk.chunk_id, chunk.text
            )
        } else {
            "LLMWave-Big did not find a stable trained-artifact peak.".to_string()
        };
        return ArtifactAnswer {
            state: "ANSWER_REVIEW",
            safe_to_answer: false,
            text,
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

fn hash32(input: &str) -> u32 {
    let digest = Sha256::digest(input.as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

fn chunk_hash64(hash: &str) -> u64 {
    u64::from_str_radix(hash.get(..16).unwrap_or(hash), 16).unwrap_or_else(|_| {
        let digest = Sha256::digest(hash.as_bytes());
        u64::from_le_bytes([
            digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
        ])
    })
}

fn hot_header_bytes() -> usize {
    32
}

fn write_hot_header(writer: &mut impl Write, counts: &HotPackRecordCounts) -> Result<()> {
    writer.write_all(b"LLMWBHOT")?;
    write_u32(writer, 1)?;
    write_u32(writer, WAVE_DIM as u32)?;
    write_u32(writer, counts.tokens as u32)?;
    write_u32(writer, counts.transitions as u32)?;
    write_u32(writer, counts.chunks as u32)?;
    write_u32(writer, counts.schema_hints as u32)?;
    Ok(())
}

fn write_u16(writer: &mut impl Write, value: u16) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_i16(writer: &mut impl Write, value: i16) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_u32(writer: &mut impl Write, value: u32) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_u64(writer: &mut impl Write, value: u64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn parse_hot_pack(bytes: &[u8]) -> Result<HotPackImage> {
    let mut cursor = Cursor::new(bytes);
    let mut magic = [0u8; 8];
    cursor.read_exact(&mut magic)?;
    if &magic != b"LLMWBHOT" {
        bail!("invalid hot pack magic");
    }
    let version = read_u32(&mut cursor)?;
    if version != 1 {
        bail!("unsupported hot pack version {version}");
    }
    let _wave_dim = read_u32(&mut cursor)?;
    let counts = HotPackRecordCounts {
        tokens: read_u32(&mut cursor)? as usize,
        transitions: read_u32(&mut cursor)? as usize,
        chunks: read_u32(&mut cursor)? as usize,
        schema_hints: read_u32(&mut cursor)? as usize,
    };
    skip_bytes(&mut cursor, counts.tokens * 16)?;
    let mut transitions = Vec::with_capacity(counts.transitions);
    for _ in 0..counts.transitions {
        let from_hash = read_u32(&mut cursor)?;
        let to_hash = read_u32(&mut cursor)?;
        let count = read_u32(&mut cursor)?;
        let _phase_delta = read_i16(&mut cursor)?;
        skip_bytes(&mut cursor, 2)?;
        transitions.push(HotTransitionRecord {
            from_hash,
            to_hash,
            count,
        });
    }
    skip_bytes(&mut cursor, counts.chunks * 32)?;
    let mut schema_hints = Vec::with_capacity(counts.schema_hints);
    for _ in 0..counts.schema_hints {
        schema_hints.push(HotSchemaRecord {
            subject_hash: read_u32(&mut cursor)?,
            relation_hash: read_u32(&mut cursor)?,
            object_hash: read_u32(&mut cursor)?,
            count: read_u32(&mut cursor)?,
        });
    }
    Ok(HotPackImage {
        counts,
        transitions,
        schema_hints,
    })
}

fn skip_bytes(cursor: &mut Cursor<&[u8]>, count: usize) -> Result<()> {
    let new_position = cursor.position() + count as u64;
    if new_position > cursor.get_ref().len() as u64 {
        bail!("truncated hot pack");
    }
    cursor.set_position(new_position);
    Ok(())
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut bytes = [0u8; 4];
    cursor.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_i16(cursor: &mut Cursor<&[u8]>) -> Result<i16> {
    let mut bytes = [0u8; 2];
    cursor.read_exact(&mut bytes)?;
    Ok(i16::from_le_bytes(bytes))
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
