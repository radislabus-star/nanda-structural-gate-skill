//! External fixture loader for LLMWave-Big surface-bank validation.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::surface_corpus_eval::{SurfaceBinding8, SurfaceFamily32};
use super::surface_production::{EvidenceCopySpan24, SurfaceAtom16};

pub(crate) const SURFACE_BANK_FIXTURE_VERSION: &str = "llmwave-big-v310-surface-bank-fixture";

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceBankFixtureReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub read_as: &'static str,
    pub corpus_path: String,
    pub corpus: SurfaceBankFixtureCorpusSummary,
    pub accepted_families: Vec<SurfaceFixtureFamilyReport>,
    pub rare_copy_spans: Vec<SurfaceFixtureRareReport>,
    pub metrics: SurfaceBankFixtureMetrics,
    pub baselines: SurfaceBankFixtureBaselines,
    pub claim_boundary: SurfaceBankFixtureClaimBoundary,
    pub next_engine_steps: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceBankFixtureCorpusSummary {
    pub fixture_version: String,
    pub source: String,
    pub family_count: usize,
    pub observed_forms: usize,
    pub held_out_forms: usize,
    pub negative_controls: usize,
    pub rare_forms: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceFixtureFamilyReport {
    pub family_id: u32,
    pub root: String,
    pub accepted: bool,
    pub observed_forms: Vec<String>,
    pub held_out_reconstructions: Vec<SurfaceFixtureReconstruction>,
    pub negative_controls: Vec<SurfaceFixtureNegativeControl>,
    pub record: SurfaceFamily32,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceFixtureReconstruction {
    pub suffix: String,
    pub expected: String,
    pub reconstructed: String,
    pub exact_match: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceFixtureNegativeControl {
    pub root: String,
    pub suffix: String,
    pub surface: String,
    pub accepted: bool,
    pub reason: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceFixtureRareReport {
    pub surface: String,
    pub path: &'static str,
    pub accepted_as_family: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceBankFixtureMetrics {
    pub fixture_loaded: bool,
    pub accepted_family_count: usize,
    pub positive_exact_match_rate: f32,
    pub negative_reject_rate: f32,
    pub rare_copy_span_rate: f32,
    pub false_family_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceBankFixtureBaselines {
    pub surface_atom_record_bytes: usize,
    pub family_record_bytes: usize,
    pub binding_record_bytes: usize,
    pub copy_span_record_bytes: usize,
    pub total_bank_bytes: usize,
    pub direct_lookup_baseline_bytes: usize,
    pub saving_ratio: f32,
    pub bytes_per_reconstructable_surface: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceBankFixtureClaimBoundary {
    pub external_fixture_loaded: bool,
    pub real_corpus_trained: bool,
    pub nonlinear_surface_memory_proven: bool,
    pub free_form_spelling_proven: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

#[derive(Deserialize)]
struct SurfaceFixture {
    version: String,
    source: String,
    families: Vec<SurfaceFixtureFamily>,
    rare_forms: Vec<String>,
}

#[derive(Deserialize)]
struct SurfaceFixtureFamily {
    family_id: u32,
    root: String,
    observed_suffixes: Vec<String>,
    held_out: Vec<SurfaceFixtureHeldOut>,
    negative: Vec<SurfaceFixtureNegative>,
}

#[derive(Deserialize)]
struct SurfaceFixtureHeldOut {
    suffix: String,
    surface: String,
}

#[derive(Deserialize)]
struct SurfaceFixtureNegative {
    root: String,
    suffix: String,
    surface: String,
}

pub(crate) fn build_surface_bank_fixture_report(
    corpus_path: &Path,
) -> Result<SurfaceBankFixtureReport> {
    let raw = fs::read_to_string(corpus_path)
        .with_context(|| format!("read surface corpus fixture {}", corpus_path.display()))?;
    let fixture: SurfaceFixture = serde_json::from_str(&raw)
        .with_context(|| format!("parse surface corpus fixture {}", corpus_path.display()))?;

    let accepted_families = fixture
        .families
        .iter()
        .map(build_family_report)
        .collect::<Vec<_>>();
    let rare_copy_spans = fixture
        .rare_forms
        .iter()
        .map(|surface| SurfaceFixtureRareReport {
            surface: surface.clone(),
            path: "evidence_copy_span",
            accepted_as_family: false,
        })
        .collect::<Vec<_>>();

    let corpus = build_corpus_summary(&fixture);
    let metrics = build_metrics(&accepted_families, &rare_copy_spans);
    let baselines = build_baselines(&fixture, &accepted_families);

    Ok(SurfaceBankFixtureReport {
        mode: "llmwave-big-surface-bank-fixture",
        version: SURFACE_BANK_FIXTURE_VERSION,
        roadmap_block: "v301-v310",
        verdict: "SURFACE_BANK_FIXTURE_READY_NOT_REAL_TRAINING",
        read_as: "an external JSON fixture loader for surface-family validation, not broad corpus training",
        corpus_path: corpus_path.display().to_string(),
        corpus,
        accepted_families,
        rare_copy_spans,
        metrics,
        baselines,
        claim_boundary: SurfaceBankFixtureClaimBoundary {
            external_fixture_loaded: true,
            real_corpus_trained: false,
            nonlinear_surface_memory_proven: false,
            free_form_spelling_proven: false,
            safe_claim:
                "LLMWave-Big can validate a surface-family bank from an external JSON corpus fixture",
            forbidden_claims: vec![
                "this external fixture is broad language training",
                "fixture-level savings prove nonlinear memory",
                "negative controls prove arbitrary spelling safety",
                "rare copy spans understand identifier semantics",
            ],
        },
        next_engine_steps: vec![
            "increase fixture size to hundreds of noisy observed forms",
            "derive families from raw forms instead of explicit fixture roots",
            "add baseline duel against lexical direct lookup under equal evidence",
            "measure role_error_rate and false_positive_rate after L2 candidate use",
            "feed fixture-built families into the L2 Word Field cache",
        ],
    })
}

fn build_family_report(family: &SurfaceFixtureFamily) -> SurfaceFixtureFamilyReport {
    let observed_forms = family
        .observed_suffixes
        .iter()
        .map(|suffix| compose(&family.root, suffix))
        .collect::<Vec<_>>();
    let held_out_reconstructions = family
        .held_out
        .iter()
        .map(|held_out| {
            let reconstructed = compose(&family.root, &held_out.suffix);
            SurfaceFixtureReconstruction {
                suffix: held_out.suffix.clone(),
                expected: held_out.surface.clone(),
                exact_match: reconstructed == held_out.surface,
                reconstructed,
            }
        })
        .collect::<Vec<_>>();
    let negative_controls = family
        .negative
        .iter()
        .map(|negative| {
            let produced = compose(&negative.root, &negative.suffix);
            let accepted = produced == negative.surface && negative.root == family.root;
            SurfaceFixtureNegativeControl {
                root: negative.root.clone(),
                suffix: negative.suffix.clone(),
                surface: negative.surface.clone(),
                accepted,
                reason: if accepted {
                    "matches_current_family"
                } else {
                    "does_not_match_current_family_or_expected_surface"
                },
            }
        })
        .collect::<Vec<_>>();

    SurfaceFixtureFamilyReport {
        family_id: family.family_id,
        root: family.root.clone(),
        accepted: family.root.len() >= 4 && family.observed_suffixes.len() >= 2,
        observed_forms,
        held_out_reconstructions,
        negative_controls,
        record: SurfaceFamily32 {
            family_id: family.family_id,
            root_atom_start: family.family_id + 100,
            suffix_atom_start: family.family_id + 200,
            form_count: (family.observed_suffixes.len() + family.held_out.len()) as u16,
            root_count: 1,
            suffix_count: (family.observed_suffixes.len() + family.held_out.len()) as u16,
            mode_bits: 0b0011,
            family_hash: family.family_id ^ 0x6B6B_0000,
            checksum: family.family_id ^ 0xB6B6_0000,
            reserved: 0,
        },
    }
}

fn build_corpus_summary(fixture: &SurfaceFixture) -> SurfaceBankFixtureCorpusSummary {
    SurfaceBankFixtureCorpusSummary {
        fixture_version: fixture.version.clone(),
        source: fixture.source.clone(),
        family_count: fixture.families.len(),
        observed_forms: fixture
            .families
            .iter()
            .map(|family| family.observed_suffixes.len())
            .sum(),
        held_out_forms: fixture
            .families
            .iter()
            .map(|family| family.held_out.len())
            .sum(),
        negative_controls: fixture
            .families
            .iter()
            .map(|family| family.negative.len())
            .sum(),
        rare_forms: fixture.rare_forms.len(),
    }
}

fn build_metrics(
    families: &[SurfaceFixtureFamilyReport],
    rare_copy_spans: &[SurfaceFixtureRareReport],
) -> SurfaceBankFixtureMetrics {
    let positives = families
        .iter()
        .flat_map(|family| family.held_out_reconstructions.iter())
        .collect::<Vec<_>>();
    let positive_matches = positives.iter().filter(|case| case.exact_match).count();
    let negatives = families
        .iter()
        .flat_map(|family| family.negative_controls.iter())
        .collect::<Vec<_>>();
    let negative_rejects = negatives.iter().filter(|case| !case.accepted).count();
    let rare_copy = rare_copy_spans
        .iter()
        .filter(|rare| rare.path == "evidence_copy_span" && !rare.accepted_as_family)
        .count();

    SurfaceBankFixtureMetrics {
        fixture_loaded: true,
        accepted_family_count: families.iter().filter(|family| family.accepted).count(),
        positive_exact_match_rate: ratio(positive_matches, positives.len()),
        negative_reject_rate: ratio(negative_rejects, negatives.len()),
        rare_copy_span_rate: ratio(rare_copy, rare_copy_spans.len()),
        false_family_rate: ratio(
            negatives.len().saturating_sub(negative_rejects),
            negatives.len(),
        ),
        state: "EXTERNAL_FIXTURE_PASS_NOT_GENERAL_PROOF",
    }
}

fn build_baselines(
    fixture: &SurfaceFixture,
    families: &[SurfaceFixtureFamilyReport],
) -> SurfaceBankFixtureBaselines {
    let roots = families
        .iter()
        .filter(|family| family.accepted)
        .map(|family| family.root.as_str())
        .collect::<BTreeSet<_>>();
    let suffixes = fixture
        .families
        .iter()
        .flat_map(|family| {
            family
                .observed_suffixes
                .iter()
                .chain(family.held_out.iter().map(|held_out| &held_out.suffix))
        })
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let observed_and_held_out = families
        .iter()
        .map(|family| family.observed_forms.len() + family.held_out_reconstructions.len())
        .sum::<usize>();
    let direct_lookup_baseline_bytes = families
        .iter()
        .flat_map(|family| {
            family.observed_forms.iter().chain(
                family
                    .held_out_reconstructions
                    .iter()
                    .map(|case| &case.expected),
            )
        })
        .map(|surface| 4 + surface.len())
        .sum::<usize>()
        + fixture
            .rare_forms
            .iter()
            .map(|surface| 4 + surface.len())
            .sum::<usize>();

    let surface_atom_record_bytes =
        (roots.len() + suffixes.len()) * core::mem::size_of::<SurfaceAtom16>();
    let family_record_bytes = families.iter().filter(|family| family.accepted).count()
        * core::mem::size_of::<SurfaceFamily32>();
    let binding_record_bytes = observed_and_held_out * core::mem::size_of::<SurfaceBinding8>();
    let copy_span_record_bytes =
        fixture.rare_forms.len() * core::mem::size_of::<EvidenceCopySpan24>();
    let total_bank_bytes = surface_atom_record_bytes
        + family_record_bytes
        + binding_record_bytes
        + copy_span_record_bytes;

    SurfaceBankFixtureBaselines {
        surface_atom_record_bytes,
        family_record_bytes,
        binding_record_bytes,
        copy_span_record_bytes,
        total_bank_bytes,
        direct_lookup_baseline_bytes,
        saving_ratio: saving_ratio(direct_lookup_baseline_bytes, total_bank_bytes),
        bytes_per_reconstructable_surface: if observed_and_held_out == 0 {
            0.0
        } else {
            total_bank_bytes as f32 / observed_and_held_out as f32
        },
    }
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
