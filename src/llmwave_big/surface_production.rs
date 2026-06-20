//! Surface production memory for LLMWave-Big word forms.

use serde::Serialize;

pub(crate) const SURFACE_PRODUCTION_VERSION: &str = "llmwave-big-v260-surface-production";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct SurfaceAtom16 {
    pub atom_id: u32,
    pub value_hash: u32,
    pub payload: u32,
    pub kind: u8,
    pub script: u8,
    pub flags: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct SurfaceProgram32 {
    pub program_id: u32,
    pub atom_start: u32,
    pub root_atom_id: u32,
    pub morpheme_mask: u32,
    pub surface_hash: u32,
    pub checksum: u32,
    pub atom_count: u16,
    pub copy_span_start: u16,
    pub copy_span_count: u16,
    pub mode_bits: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct EvidenceCopySpan24 {
    pub evidence_ref: u32,
    pub byte_start: u32,
    pub surface_hash: u32,
    pub program_id: u32,
    pub checksum: u32,
    pub byte_len: u16,
    pub mode_bits: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct SurfaceProductionCandidate32 {
    pub candidate_id: u32,
    pub program_id: u32,
    pub meaning_centroid_id: u32,
    pub context_centroid_id: u32,
    pub morpheme_score: i16,
    pub copy_score: i16,
    pub byte_fallback_score: i16,
    pub grammar_score: i16,
    pub anti_confusion: i16,
    pub final_score: i16,
    pub route: u8,
    pub state: u8,
    pub flags: u16,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceProductionReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub read_as: &'static str,
    pub production_law: SurfaceProductionLaw,
    pub record_formats: Vec<RecordFormat>,
    pub atoms: Vec<SurfaceAtomView>,
    pub programs: Vec<SurfaceProgramView>,
    pub copy_spans: Vec<EvidenceCopySpanView>,
    pub candidates: Vec<SurfaceCandidateView>,
    pub selected: SelectedSurfaceProduction,
    pub claim_boundary: SurfaceProductionClaimBoundary,
    pub next_engine_steps: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceProductionLaw {
    pub primary_rule: &'static str,
    pub memory_shape: &'static str,
    pub hot_core_boundary: &'static str,
    pub exact_form_rule: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RecordFormat {
    pub name: &'static str,
    pub bytes: usize,
    pub role: &'static str,
    pub hot_core_visibility: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceAtomView {
    pub label: &'static str,
    pub layer: &'static str,
    pub role: &'static str,
    pub record: SurfaceAtom16,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceProgramView {
    pub label: &'static str,
    pub path: &'static str,
    pub role: &'static str,
    pub materializes: &'static str,
    pub materialization_scope: &'static str,
    pub record: SurfaceProgram32,
}

#[derive(Serialize, Clone)]
pub(crate) struct EvidenceCopySpanView {
    pub label: &'static str,
    pub role: &'static str,
    pub materializes: &'static str,
    pub materialization_scope: &'static str,
    pub record: EvidenceCopySpan24,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceCandidateView {
    pub label: &'static str,
    pub production_path: &'static str,
    pub interpretation: &'static str,
    pub record: SurfaceProductionCandidate32,
}

#[derive(Serialize, Clone)]
pub(crate) struct SelectedSurfaceProduction {
    pub selected_candidate_id: u32,
    pub program_id: u32,
    pub production_path: &'static str,
    pub final_score: i16,
    pub materialized_preview: &'static str,
    pub materialization_scope: &'static str,
    pub why_selected: &'static str,
    pub not_selected: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceProductionClaimBoundary {
    pub real_corpus_trained: bool,
    pub free_form_spelling_proven: bool,
    pub nonlinear_surface_memory_proven: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

pub(crate) fn build_surface_production_report() -> SurfaceProductionReport {
    let candidates = scored_candidates(sample_candidates());
    let selected = select_candidate(&candidates);

    SurfaceProductionReport {
        mode: "llmwave-big-surface-production",
        version: SURFACE_PRODUCTION_VERSION,
        roadmap_block: "v253-v260",
        verdict: "SURFACE_PRODUCTION_READY",
        read_as: "a typed surface-production memory contract, not proof of trained free-form generation",
        production_law: SurfaceProductionLaw {
            primary_rule:
                "do_not_store_words_as_token_id_to_utf8; produce visible forms from composable surface memory",
            memory_shape:
                "grapheme_or_byte_atoms + morpheme_atoms + surface_program + evidence_copy_span",
            hot_core_boundary:
                "hot_core_scores_program_ids_and_copy_span_refs; cold_materializer_expands_utf8",
            exact_form_rule:
                "common forms are composed; rare names/codes are copied from observed evidence spans",
        },
        record_formats: vec![
            RecordFormat {
                name: "SurfaceAtom16",
                bytes: core::mem::size_of::<SurfaceAtom16>(),
                role: "minimal grapheme byte or morpheme atom",
                hot_core_visibility: "atom_id_kind_script_hash_payload",
            },
            RecordFormat {
                name: "SurfaceProgram32",
                bytes: core::mem::size_of::<SurfaceProgram32>(),
                role: "ordered surface recipe over atoms and optional copy spans",
                hot_core_visibility: "program_id_atom_range_root_mask_hash",
            },
            RecordFormat {
                name: "EvidenceCopySpan24",
                bytes: core::mem::size_of::<EvidenceCopySpan24>(),
                role: "exact recovery path for rare names codes and one-off strings",
                hot_core_visibility: "evidence_ref_byte_span_hash_program_id",
            },
            RecordFormat {
                name: "SurfaceProductionCandidate32",
                bytes: core::mem::size_of::<SurfaceProductionCandidate32>(),
                role: "score record choosing compose copy or byte fallback path",
                hot_core_visibility: "candidate_program_centroids_scores_state",
            },
        ],
        atoms: surface_atoms(),
        programs: surface_programs(),
        copy_spans: evidence_copy_spans(),
        candidates: candidate_views(candidates),
        selected: SelectedSurfaceProduction {
            selected_candidate_id: selected.candidate_id,
            program_id: selected.program_id,
            production_path: "surface_program",
            final_score: selected.final_score,
            materialized_preview: "invoice",
            materialization_scope: "cold_report_preview_only_not_hot_storage",
            why_selected:
                "morpheme and grammar support beat copy and byte fallback without using a flat string table",
            not_selected: vec![
                "evidence_copy_span reserved for exact rare codes and names",
                "byte_fallback reserved for unknown surface recovery",
            ],
        },
        claim_boundary: SurfaceProductionClaimBoundary {
            real_corpus_trained: false,
            free_form_spelling_proven: false,
            nonlinear_surface_memory_proven: false,
            safe_claim:
                "LLMWave-Big now has fixed-size records for producing surfaces from composable form memory",
            forbidden_claims: vec![
                "this proves the model can spell arbitrary unseen words",
                "surface production proves nonlinear memory density",
                "the hot core stores UTF-8 strings",
                "a real corpus has trained this surface memory",
            ],
        },
        next_engine_steps: vec![
            "connect accepted LexicalBindingRecord32 values to SurfaceProgram32 ids",
            "add a cold materializer that expands atom programs into UTF-8 outside the hot loop",
            "run spelling reconstruction eval against direct lookup and byte fallback baselines",
            "measure bytes_per_reconstructable_surface and exact_copy_error_rate",
            "feed L2 candidate cache with program ids instead of flat output strings",
        ],
    }
}

fn sample_candidates() -> [SurfaceProductionCandidate32; 3] {
    [
        SurfaceProductionCandidate32 {
            candidate_id: 81_001,
            program_id: 61_001,
            meaning_centroid_id: 12_201,
            context_centroid_id: 2_101,
            morpheme_score: 54,
            copy_score: 0,
            byte_fallback_score: 3,
            grammar_score: 28,
            anti_confusion: 4,
            final_score: 0,
            route: 1,
            state: 0,
            flags: 0,
        },
        SurfaceProductionCandidate32 {
            candidate_id: 81_002,
            program_id: 61_002,
            meaning_centroid_id: 12_201,
            context_centroid_id: 2_101,
            morpheme_score: 8,
            copy_score: 63,
            byte_fallback_score: 0,
            grammar_score: 12,
            anti_confusion: 9,
            final_score: 0,
            route: 2,
            state: 0,
            flags: 0,
        },
        SurfaceProductionCandidate32 {
            candidate_id: 81_003,
            program_id: 61_003,
            meaning_centroid_id: 0,
            context_centroid_id: 2_101,
            morpheme_score: 0,
            copy_score: 0,
            byte_fallback_score: 38,
            grammar_score: 2,
            anti_confusion: 14,
            final_score: 0,
            route: 3,
            state: 0,
            flags: 0,
        },
    ]
}

fn scored_candidates(
    mut candidates: [SurfaceProductionCandidate32; 3],
) -> [SurfaceProductionCandidate32; 3] {
    for candidate in &mut candidates {
        candidate.final_score = score_candidate(*candidate);
        candidate.state = if candidate.final_score >= 72 { 2 } else { 1 };
    }
    candidates
}

fn score_candidate(candidate: SurfaceProductionCandidate32) -> i16 {
    candidate.morpheme_score
        + candidate.copy_score
        + candidate.byte_fallback_score
        + candidate.grammar_score
        - candidate.anti_confusion
}

fn select_candidate(
    candidates: &[SurfaceProductionCandidate32; 3],
) -> SurfaceProductionCandidate32 {
    let mut selected = candidates[0];
    for candidate in candidates.iter().skip(1) {
        if candidate.final_score > selected.final_score {
            selected = *candidate;
        }
    }
    selected
}

fn surface_atoms() -> Vec<SurfaceAtomView> {
    vec![
        SurfaceAtomView {
            label: "latin_i",
            layer: "grapheme_or_byte_atoms",
            role: "base grapheme",
            record: SurfaceAtom16 {
                atom_id: 41_001,
                value_hash: 0x1000_0009,
                payload: 0,
                kind: 1,
                script: 1,
                flags: 0,
            },
        },
        SurfaceAtomView {
            label: "root_invoic",
            layer: "morpheme_atoms",
            role: "productive root atom",
            record: SurfaceAtom16 {
                atom_id: 42_101,
                value_hash: 0x2000_0A11,
                payload: 0,
                kind: 3,
                script: 1,
                flags: 0,
            },
        },
        SurfaceAtomView {
            label: "suffix_e",
            layer: "morpheme_atoms",
            role: "surface ending atom",
            record: SurfaceAtom16 {
                atom_id: 42_201,
                value_hash: 0x2000_0E01,
                payload: 0,
                kind: 4,
                script: 1,
                flags: 0,
            },
        },
        SurfaceAtomView {
            label: "byte_escape",
            layer: "grapheme_or_byte_atoms",
            role: "unknown-byte fallback",
            record: SurfaceAtom16 {
                atom_id: 49_255,
                value_hash: 0xFF00_00FF,
                payload: 255,
                kind: 9,
                script: 0,
                flags: 1,
            },
        },
    ]
}

fn surface_programs() -> Vec<SurfaceProgramView> {
    vec![
        SurfaceProgramView {
            label: "compose_common_business_form",
            path: "surface_program",
            role: "common productive spelling",
            materializes: "invoice",
            materialization_scope: "cold_report_preview_only",
            record: SurfaceProgram32 {
                program_id: 61_001,
                atom_start: 1,
                root_atom_id: 42_101,
                morpheme_mask: 0b11,
                surface_hash: 0xB117_0001,
                checksum: 0xA001_A001,
                atom_count: 2,
                copy_span_start: 0,
                copy_span_count: 0,
                mode_bits: 0b0001,
            },
        },
        SurfaceProgramView {
            label: "copy_rare_invoice_code",
            path: "evidence_copy_span",
            role: "exact one-off recovery",
            materializes: "PI-HL-RLTG-GZ-20260611-03",
            materialization_scope: "cold_report_preview_only",
            record: SurfaceProgram32 {
                program_id: 61_002,
                atom_start: 0,
                root_atom_id: 0,
                morpheme_mask: 0,
                surface_hash: 0xC0DE_0603,
                checksum: 0xA002_A002,
                atom_count: 0,
                copy_span_start: 1,
                copy_span_count: 1,
                mode_bits: 0b0010,
            },
        },
        SurfaceProgramView {
            label: "unknown_byte_fallback",
            path: "byte_fallback",
            role: "lossless recovery when no atom program is accepted",
            materializes: "<bytes>",
            materialization_scope: "cold_report_preview_only",
            record: SurfaceProgram32 {
                program_id: 61_003,
                atom_start: 3,
                root_atom_id: 49_255,
                morpheme_mask: 0,
                surface_hash: 0xFFFF_0001,
                checksum: 0xA003_A003,
                atom_count: 1,
                copy_span_start: 0,
                copy_span_count: 0,
                mode_bits: 0b0100,
            },
        },
    ]
}

fn evidence_copy_spans() -> Vec<EvidenceCopySpanView> {
    vec![EvidenceCopySpanView {
        label: "observed_invoice_code_span",
        role: "exact rare form recovery",
        materializes: "PI-HL-RLTG-GZ-20260611-03",
        materialization_scope: "cold_report_preview_only_not_hot_storage",
        record: EvidenceCopySpan24 {
            evidence_ref: 10_001,
            byte_start: 128,
            surface_hash: 0xC0DE_0603,
            program_id: 61_002,
            checksum: 0xCC00_0603,
            byte_len: 27,
            mode_bits: 0b0010,
        },
    }]
}

fn candidate_views(candidates: [SurfaceProductionCandidate32; 3]) -> Vec<SurfaceCandidateView> {
    vec![
        SurfaceCandidateView {
            label: "compose_common_business_form",
            production_path: "surface_program",
            interpretation: "compose common form from morpheme atoms under grammar support",
            record: candidates[0],
        },
        SurfaceCandidateView {
            label: "copy_rare_invoice_code",
            production_path: "evidence_copy_span",
            interpretation: "copy exact observed code when copy support dominates",
            record: candidates[1],
        },
        SurfaceCandidateView {
            label: "unknown_byte_fallback",
            production_path: "byte_fallback",
            interpretation: "recover bytes only when no stable atom program is available",
            record: candidates[2],
        },
    ]
}
