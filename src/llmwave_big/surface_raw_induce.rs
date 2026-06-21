//! Raw surface-form induction for LLMWave-Big.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::surface_corpus_eval::{SurfaceBinding8, SurfaceFamily32};
use super::surface_production::{EvidenceCopySpan24, SurfaceAtom16};

pub(crate) const SURFACE_RAW_INDUCE_VERSION: &str = "llmwave-big-v320-surface-raw-induce";

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceRawInduceReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub read_as: &'static str,
    pub corpus_path: String,
    pub corpus: SurfaceRawCorpusSummary,
    pub induced_families: Vec<InducedSurfaceFamily>,
    pub rare_copy_spans: Vec<RawRareSurface>,
    pub held_out: Vec<RawHeldOutEval>,
    pub negative_controls: Vec<RawNegativeEval>,
    pub metrics: SurfaceRawInduceMetrics,
    pub baselines: SurfaceRawInduceBaselines,
    pub claim_boundary: SurfaceRawInduceClaimBoundary,
    pub next_engine_steps: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceRawCorpusSummary {
    pub fixture_version: String,
    pub source: String,
    pub raw_forms: usize,
    pub suffix_inventory: usize,
    pub held_out_forms: usize,
    pub negative_controls: usize,
    pub expected_roots: usize,
    pub min_family_forms: usize,
    pub min_root_chars: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct InducedSurfaceFamily {
    pub family_id: u32,
    pub root: String,
    pub forms: Vec<String>,
    pub suffixes: Vec<String>,
    pub accepted: bool,
    pub record: SurfaceFamily32,
}

#[derive(Serialize, Clone)]
pub(crate) struct RawRareSurface {
    pub surface: String,
    pub path: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RawHeldOutEval {
    pub surface: String,
    pub expected_root: String,
    pub matched_root: String,
    pub matched_suffix: String,
    pub exact_match: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct RawNegativeEval {
    pub root: String,
    pub suffix: String,
    pub surface: String,
    pub accepted: bool,
    pub reason: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceRawInduceMetrics {
    pub induced_family_count: usize,
    pub expected_root_recall: f32,
    pub held_out_exact_match_rate: f32,
    pub negative_reject_rate: f32,
    pub rare_copy_span_rate: f32,
    pub false_family_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceRawInduceBaselines {
    pub surface_atom_record_bytes: usize,
    pub family_record_bytes: usize,
    pub binding_record_bytes: usize,
    pub copy_span_record_bytes: usize,
    pub total_bank_bytes: usize,
    pub direct_lookup_baseline_bytes: usize,
    pub saving_ratio: f32,
    pub bytes_per_induced_surface: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceRawInduceClaimBoundary {
    pub raw_forms_used: bool,
    pub roots_given_to_inducer: bool,
    pub real_corpus_trained: bool,
    pub nonlinear_surface_memory_proven: bool,
    pub free_form_spelling_proven: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

#[derive(Deserialize)]
struct RawSurfaceFixture {
    version: String,
    source: String,
    suffix_inventory: Vec<String>,
    min_family_forms: usize,
    min_root_chars: usize,
    raw_forms: Vec<String>,
    held_out: Vec<RawHeldOut>,
    negative: Vec<RawNegative>,
    expected_roots: Vec<String>,
}

#[derive(Deserialize)]
struct RawHeldOut {
    surface: String,
    expected_root: String,
}

#[derive(Deserialize)]
struct RawNegative {
    root: String,
    suffix: String,
    surface: String,
}

struct CandidateFamily {
    root: String,
    forms: BTreeSet<String>,
    suffixes: BTreeSet<String>,
}

pub(crate) fn build_surface_raw_induce_report(
    corpus_path: &Path,
) -> Result<SurfaceRawInduceReport> {
    let raw = fs::read_to_string(corpus_path)
        .with_context(|| format!("read raw surface corpus {}", corpus_path.display()))?;
    let fixture: RawSurfaceFixture = serde_json::from_str(&raw)
        .with_context(|| format!("parse raw surface corpus {}", corpus_path.display()))?;

    let induced_families = induce_families(&fixture);
    let rare_copy_spans = build_rare_copy_spans(&fixture, &induced_families);
    let held_out = eval_held_out(&fixture, &induced_families);
    let negative_controls = eval_negative(&fixture, &induced_families);
    let metrics = build_metrics(
        &fixture,
        &induced_families,
        &rare_copy_spans,
        &held_out,
        &negative_controls,
    );
    let baselines = build_baselines(&fixture, &induced_families, &rare_copy_spans);

    Ok(SurfaceRawInduceReport {
        mode: "llmwave-big-surface-raw-induce",
        version: SURFACE_RAW_INDUCE_VERSION,
        roadmap_block: "v311-v320",
        verdict: "SURFACE_RAW_INDUCE_READY_NOT_REAL_TRAINING",
        read_as:
            "raw surface forms are grouped into candidate families without root/suffix fields in the input",
        corpus_path: corpus_path.display().to_string(),
        corpus: SurfaceRawCorpusSummary {
            fixture_version: fixture.version.clone(),
            source: fixture.source.clone(),
            raw_forms: fixture.raw_forms.len(),
            suffix_inventory: fixture.suffix_inventory.len(),
            held_out_forms: fixture.held_out.len(),
            negative_controls: fixture.negative.len(),
            expected_roots: fixture.expected_roots.len(),
            min_family_forms: fixture.min_family_forms,
            min_root_chars: fixture.min_root_chars,
        },
        induced_families,
        rare_copy_spans,
        held_out,
        negative_controls,
        metrics,
        baselines,
        claim_boundary: SurfaceRawInduceClaimBoundary {
            raw_forms_used: true,
            roots_given_to_inducer: false,
            real_corpus_trained: false,
            nonlinear_surface_memory_proven: false,
            free_form_spelling_proven: false,
            safe_claim:
                "LLMWave-Big can induce small surface families from raw fixture forms without explicit root fields",
            forbidden_claims: vec![
                "raw fixture induction is broad language training",
                "small raw induction proves nonlinear memory",
                "suffix inventory proves arbitrary morphology",
                "held-out exact match proves free-form spelling",
            ],
        },
        next_engine_steps: vec![
            "derive suffix inventory from corpus statistics",
            "add noisy unrelated forms and near-root collisions",
            "scale raw induction to hundreds and thousands of forms",
            "feed induced families into L2 prefix candidate cache",
            "compare induced-bank answers against direct lexical lookup",
        ],
    })
}

fn induce_families(fixture: &RawSurfaceFixture) -> Vec<InducedSurfaceFamily> {
    let mut candidates = BTreeMap::<String, CandidateFamily>::new();
    for form in &fixture.raw_forms {
        for suffix in &fixture.suffix_inventory {
            if let Some(root) = strip_suffix(form, suffix) {
                if root.chars().count() < fixture.min_root_chars {
                    continue;
                }
                let candidate = candidates
                    .entry(root.clone())
                    .or_insert_with(|| CandidateFamily {
                        root,
                        forms: BTreeSet::new(),
                        suffixes: BTreeSet::new(),
                    });
                candidate.forms.insert(form.clone());
                candidate.suffixes.insert(suffix.clone());
            }
        }
    }

    candidates
        .into_values()
        .filter(|candidate| candidate.forms.len() >= fixture.min_family_forms)
        .enumerate()
        .map(|(index, candidate)| {
            let family_id = 74_001 + index as u32;
            let suffixes = candidate.suffixes.into_iter().collect::<Vec<_>>();
            let forms = candidate.forms.into_iter().collect::<Vec<_>>();
            InducedSurfaceFamily {
                family_id,
                root: candidate.root,
                accepted: true,
                record: SurfaceFamily32 {
                    family_id,
                    root_atom_start: family_id + 100,
                    suffix_atom_start: family_id + 200,
                    form_count: forms.len() as u16,
                    root_count: 1,
                    suffix_count: suffixes.len() as u16,
                    mode_bits: 0b0101,
                    family_hash: family_id ^ 0x7C7C_0000,
                    checksum: family_id ^ 0xC7C7_0000,
                    reserved: 0,
                },
                forms,
                suffixes,
            }
        })
        .collect()
}

fn build_rare_copy_spans(
    fixture: &RawSurfaceFixture,
    families: &[InducedSurfaceFamily],
) -> Vec<RawRareSurface> {
    fixture
        .raw_forms
        .iter()
        .filter(|form| !families.iter().any(|family| family.forms.contains(*form)))
        .map(|surface| RawRareSurface {
            surface: surface.clone(),
            path: "evidence_copy_span",
        })
        .collect()
}

fn eval_held_out(
    fixture: &RawSurfaceFixture,
    families: &[InducedSurfaceFamily],
) -> Vec<RawHeldOutEval> {
    fixture
        .held_out
        .iter()
        .map(|held_out| {
            let matched = families.iter().find_map(|family| {
                fixture.suffix_inventory.iter().find_map(|suffix| {
                    let reconstructed = compose(&family.root, suffix);
                    if reconstructed == held_out.surface {
                        Some((family.root.clone(), suffix.clone()))
                    } else {
                        None
                    }
                })
            });
            let (matched_root, matched_suffix) = matched.unwrap_or_default();
            RawHeldOutEval {
                surface: held_out.surface.clone(),
                expected_root: held_out.expected_root.clone(),
                exact_match: matched_root == held_out.expected_root && !matched_suffix.is_empty(),
                matched_root,
                matched_suffix,
            }
        })
        .collect()
}

fn eval_negative(
    fixture: &RawSurfaceFixture,
    families: &[InducedSurfaceFamily],
) -> Vec<RawNegativeEval> {
    fixture
        .negative
        .iter()
        .map(|negative| {
            let accepted = families.iter().any(|family| {
                family.root == negative.root
                    && family.suffixes.contains(&negative.suffix)
                    && compose(&family.root, &negative.suffix) == negative.surface
            });
            RawNegativeEval {
                root: negative.root.clone(),
                suffix: negative.suffix.clone(),
                surface: negative.surface.clone(),
                accepted,
                reason: if accepted {
                    "matched_induced_family"
                } else {
                    "not_supported_by_induced_family"
                },
            }
        })
        .collect()
}

fn build_metrics(
    fixture: &RawSurfaceFixture,
    families: &[InducedSurfaceFamily],
    rare_copy_spans: &[RawRareSurface],
    held_out: &[RawHeldOutEval],
    negative_controls: &[RawNegativeEval],
) -> SurfaceRawInduceMetrics {
    let induced_roots = families
        .iter()
        .map(|family| family.root.as_str())
        .collect::<BTreeSet<_>>();
    let expected_hits = fixture
        .expected_roots
        .iter()
        .filter(|root| induced_roots.contains(root.as_str()))
        .count();
    let held_out_hits = held_out.iter().filter(|item| item.exact_match).count();
    let negative_rejects = negative_controls
        .iter()
        .filter(|item| !item.accepted)
        .count();

    SurfaceRawInduceMetrics {
        induced_family_count: families.len(),
        expected_root_recall: ratio(expected_hits, fixture.expected_roots.len()),
        held_out_exact_match_rate: ratio(held_out_hits, held_out.len()),
        negative_reject_rate: ratio(negative_rejects, negative_controls.len()),
        rare_copy_span_rate: ratio(rare_copy_spans.len(), rare_copy_spans.len()),
        false_family_rate: ratio(
            negative_controls.len().saturating_sub(negative_rejects),
            negative_controls.len(),
        ),
        state: "RAW_INDUCTION_PASS_NOT_GENERAL_PROOF",
    }
}

fn build_baselines(
    fixture: &RawSurfaceFixture,
    families: &[InducedSurfaceFamily],
    rare_copy_spans: &[RawRareSurface],
) -> SurfaceRawInduceBaselines {
    let suffixes = families
        .iter()
        .flat_map(|family| family.suffixes.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let induced_surfaces = families
        .iter()
        .map(|family| family.forms.len())
        .sum::<usize>();
    let direct_lookup_baseline_bytes = fixture
        .raw_forms
        .iter()
        .chain(fixture.held_out.iter().map(|held_out| &held_out.surface))
        .map(|surface| 4 + surface.len())
        .sum::<usize>();
    let surface_atom_record_bytes =
        (families.len() + suffixes.len()) * core::mem::size_of::<SurfaceAtom16>();
    let family_record_bytes = families.len() * core::mem::size_of::<SurfaceFamily32>();
    let binding_record_bytes = induced_surfaces * core::mem::size_of::<SurfaceBinding8>();
    let copy_span_record_bytes = rare_copy_spans.len() * core::mem::size_of::<EvidenceCopySpan24>();
    let total_bank_bytes = surface_atom_record_bytes
        + family_record_bytes
        + binding_record_bytes
        + copy_span_record_bytes;

    SurfaceRawInduceBaselines {
        surface_atom_record_bytes,
        family_record_bytes,
        binding_record_bytes,
        copy_span_record_bytes,
        total_bank_bytes,
        direct_lookup_baseline_bytes,
        saving_ratio: saving_ratio(direct_lookup_baseline_bytes, total_bank_bytes),
        bytes_per_induced_surface: if induced_surfaces == 0 {
            0.0
        } else {
            total_bank_bytes as f32 / induced_surfaces as f32
        },
    }
}

fn strip_suffix(form: &str, suffix: &str) -> Option<String> {
    if suffix.is_empty() {
        return Some(form.to_string());
    }
    form.strip_suffix(suffix)
        .filter(|root| !root.is_empty())
        .map(ToString::to_string)
}

fn compose(root: &str, suffix: &str) -> String {
    let mut surface = String::with_capacity(root.len() + suffix.len());
    surface.push_str(root);
    surface.push_str(suffix);
    surface
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

fn saving_ratio(baseline: usize, candidate: usize) -> f32 {
    if baseline == 0 || candidate >= baseline {
        0.0
    } else {
        (baseline - candidate) as f32 / baseline as f32
    }
}
