//! Cold surface materializer and reconstruction eval for LLMWave-Big.

use serde::Serialize;

use super::surface_production::{EvidenceCopySpan24, SurfaceAtom16, SurfaceProgram32};

pub(crate) const SURFACE_RECONSTRUCT_VERSION: &str = "llmwave-big-v270-surface-reconstruct";

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceReconstructReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub read_as: &'static str,
    pub materializer_contract: MaterializerContract,
    pub bank_summary: SurfaceBankSummary,
    pub cases: Vec<ReconstructionCase>,
    pub eval: SurfaceReconstructionEval,
    pub claim_boundary: SurfaceReconstructClaimBoundary,
    pub next_engine_steps: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct MaterializerContract {
    pub input_records: Vec<&'static str>,
    pub output: &'static str,
    pub hot_core_boundary: &'static str,
    pub composition_rule: &'static str,
    pub exact_copy_rule: &'static str,
    pub fallback_rule: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceBankSummary {
    pub atom_records: usize,
    pub program_records: usize,
    pub copy_span_records: usize,
    pub program_atom_index_entries: usize,
    pub atom_record_bytes: usize,
    pub program_record_bytes: usize,
    pub copy_span_record_bytes: usize,
    pub program_atom_index_bytes: usize,
    pub total_surface_memory_bytes: usize,
    pub evidence_bytes_borrowed_from_corpus: usize,
    pub hot_core_contains_utf8: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct ReconstructionCase {
    pub case_id: &'static str,
    pub expected: &'static str,
    pub reconstructed: String,
    pub path: &'static str,
    pub program_id: u32,
    pub exact_match: bool,
    pub model_bytes_charged: usize,
    pub direct_lookup_baseline_bytes: usize,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceReconstructionEval {
    pub cases: usize,
    pub exact_matches: usize,
    pub exact_match_rate: f32,
    pub copy_error_rate: f32,
    pub fallback_rate: f32,
    pub false_surface_rate: f32,
    pub program_reuse_ratio: f32,
    pub bytes_per_reconstructable_surface: f32,
    pub direct_lookup_baseline_bytes: usize,
    pub surface_memory_bytes: usize,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceReconstructClaimBoundary {
    pub real_corpus_trained: bool,
    pub free_form_spelling_proven: bool,
    pub nonlinear_surface_memory_proven: bool,
    pub hot_core_utf8_free: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

#[derive(Clone, Copy)]
struct MaterialAtom {
    record: SurfaceAtom16,
    text: &'static str,
}

struct SurfaceBanks {
    atoms: [MaterialAtom; 6],
    programs: [SurfaceProgram32; 4],
    copy_spans: [EvidenceCopySpan24; 1],
    program_atom_index: [u32; 7],
    evidence: &'static [u8],
}

pub(crate) fn build_surface_reconstruct_report() -> SurfaceReconstructReport {
    let banks = sample_banks();
    let cases = reconstruct_cases(&banks);
    let eval = evaluate_cases(&banks, &cases);

    SurfaceReconstructReport {
        mode: "llmwave-big-surface-reconstruct",
        version: SURFACE_RECONSTRUCT_VERSION,
        roadmap_block: "v261-v270",
        verdict: "SURFACE_RECONSTRUCT_READY",
        read_as: "a cold materializer and toy reconstruction eval, not trained free-form spelling",
        materializer_contract: MaterializerContract {
            input_records: vec![
                "SurfaceAtom16",
                "SurfaceProgram32",
                "EvidenceCopySpan24",
                "program_atom_index:u32",
                "external_evidence_bytes",
            ],
            output: "visible UTF-8 surface outside the hot core",
            hot_core_boundary:
                "hot core selects program ids, atom ranges, copy refs, and scores; it does not store UTF-8 strings",
            composition_rule:
                "surface_program walks program_atom_index and expands atom ids through the cold atom materializer",
            exact_copy_rule:
                "evidence_copy_span slices observed corpus bytes for rare codes and names",
            fallback_rule:
                "byte_fallback emits bytes from atom payload only when no stable atom program or copy span wins",
        },
        bank_summary: bank_summary(&banks),
        cases,
        eval,
        claim_boundary: SurfaceReconstructClaimBoundary {
            real_corpus_trained: false,
            free_form_spelling_proven: false,
            nonlinear_surface_memory_proven: false,
            hot_core_utf8_free: true,
            safe_claim:
                "LLMWave-Big can now materialize toy surface programs outside the hot core and measure reconstruction quality",
            forbidden_claims: vec![
                "this proves arbitrary word spelling",
                "this toy eval proves nonlinear surface memory",
                "the hot core contains word strings",
                "copy-span recovery is semantic understanding",
            ],
        },
        next_engine_steps: vec![
            "load surface atoms and programs from a real corpus-derived bank",
            "add a reconstruction suite with held-out common forms, rare codes, and noisy evidence",
            "compare against direct lookup, byte-only fallback, and lexical bag baselines",
            "track exact_copy_error_rate and false_surface_rate under noise",
            "connect L2 candidate cache output to SurfaceProgram32 ids",
        ],
    }
}

fn reconstruct_cases(banks: &SurfaceBanks) -> Vec<ReconstructionCase> {
    let specs = [
        ("common_invoice", "invoice", 61_001, "surface_program"),
        ("common_invoicing", "invoicing", 61_004, "surface_program"),
        (
            "rare_invoice_code",
            "PI-HL-RLTG-GZ-20260611-03",
            61_002,
            "evidence_copy_span",
        ),
        ("unknown_byte_fallback", "zxq", 61_003, "byte_fallback"),
    ];

    specs
        .into_iter()
        .map(|(case_id, expected, program_id, path)| {
            let reconstructed = reconstruct_program(banks, program_id);
            let exact_match = reconstructed == expected;
            ReconstructionCase {
                case_id,
                expected,
                reconstructed,
                path,
                program_id,
                exact_match,
                model_bytes_charged: charged_model_bytes(path),
                direct_lookup_baseline_bytes: 4 + expected.len(),
                state: if exact_match {
                    "RECONSTRUCTED"
                } else {
                    "RECONSTRUCTION_ERROR"
                },
            }
        })
        .collect()
}

fn reconstruct_program(banks: &SurfaceBanks, program_id: u32) -> String {
    let Some(program) = find_program(banks, program_id) else {
        return String::new();
    };
    if program.copy_span_count > 0 {
        return reconstruct_copy_span(banks, program);
    }
    reconstruct_atom_program(banks, program)
}

fn reconstruct_atom_program(banks: &SurfaceBanks, program: SurfaceProgram32) -> String {
    let start = program.atom_start as usize;
    let end = start.saturating_add(program.atom_count as usize);
    let mut out = String::new();
    for atom_id in banks.program_atom_index.get(start..end).unwrap_or(&[]) {
        if let Some(atom) = find_atom(banks, *atom_id) {
            if atom.record.kind == 9 {
                if let Some(ch) = char::from_u32(atom.record.payload) {
                    out.push(ch);
                }
            } else {
                out.push_str(atom.text);
            }
        }
    }
    out
}

fn reconstruct_copy_span(banks: &SurfaceBanks, program: SurfaceProgram32) -> String {
    let index = program.copy_span_start as usize;
    let Some(span) = banks.copy_spans.get(index).copied() else {
        return String::new();
    };
    let start = span.byte_start as usize;
    let end = start.saturating_add(span.byte_len as usize);
    let Some(bytes) = banks.evidence.get(start..end) else {
        return String::new();
    };
    String::from_utf8_lossy(bytes).into_owned()
}

fn find_program(banks: &SurfaceBanks, program_id: u32) -> Option<SurfaceProgram32> {
    banks
        .programs
        .iter()
        .copied()
        .find(|program| program.program_id == program_id)
}

fn find_atom(banks: &SurfaceBanks, atom_id: u32) -> Option<MaterialAtom> {
    banks
        .atoms
        .iter()
        .copied()
        .find(|atom| atom.record.atom_id == atom_id)
}

fn charged_model_bytes(path: &str) -> usize {
    match path {
        "surface_program" => {
            core::mem::size_of::<SurfaceProgram32>() + 2 * core::mem::size_of::<SurfaceAtom16>()
        }
        "evidence_copy_span" => {
            core::mem::size_of::<SurfaceProgram32>() + core::mem::size_of::<EvidenceCopySpan24>()
        }
        "byte_fallback" => {
            core::mem::size_of::<SurfaceProgram32>() + 3 * core::mem::size_of::<SurfaceAtom16>()
        }
        _ => 0,
    }
}

fn evaluate_cases(banks: &SurfaceBanks, cases: &[ReconstructionCase]) -> SurfaceReconstructionEval {
    let exact_matches = cases.iter().filter(|case| case.exact_match).count();
    let copy_cases = cases
        .iter()
        .filter(|case| case.path == "evidence_copy_span")
        .count();
    let copy_errors = cases
        .iter()
        .filter(|case| case.path == "evidence_copy_span" && !case.exact_match)
        .count();
    let fallback_cases = cases
        .iter()
        .filter(|case| case.path == "byte_fallback")
        .count();
    let false_surfaces = cases.iter().filter(|case| !case.exact_match).count();
    let direct_lookup_baseline_bytes = cases
        .iter()
        .map(|case| case.direct_lookup_baseline_bytes)
        .sum();
    let surface_memory_bytes = bank_summary(banks).total_surface_memory_bytes;

    SurfaceReconstructionEval {
        cases: cases.len(),
        exact_matches,
        exact_match_rate: ratio(exact_matches, cases.len()),
        copy_error_rate: ratio(copy_errors, copy_cases),
        fallback_rate: ratio(fallback_cases, cases.len()),
        false_surface_rate: ratio(false_surfaces, cases.len()),
        program_reuse_ratio: program_reuse_ratio(banks),
        bytes_per_reconstructable_surface: surface_memory_bytes as f32
            / exact_matches.max(1) as f32,
        direct_lookup_baseline_bytes,
        surface_memory_bytes,
        state: "TOY_RECONSTRUCTION_PASS_NOT_DENSITY_PROOF",
    }
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

fn program_reuse_ratio(banks: &SurfaceBanks) -> f32 {
    let used = banks.program_atom_index.len();
    let mut unique: [u32; 7] = [0; 7];
    let mut unique_count = 0usize;
    for atom_id in banks.program_atom_index {
        if !unique[..unique_count].contains(&atom_id) {
            unique[unique_count] = atom_id;
            unique_count += 1;
        }
    }
    ratio(used, unique_count)
}

fn bank_summary(banks: &SurfaceBanks) -> SurfaceBankSummary {
    let atom_record_bytes = banks.atoms.len() * core::mem::size_of::<SurfaceAtom16>();
    let program_record_bytes = banks.programs.len() * core::mem::size_of::<SurfaceProgram32>();
    let copy_span_record_bytes =
        banks.copy_spans.len() * core::mem::size_of::<EvidenceCopySpan24>();
    let program_atom_index_bytes = banks.program_atom_index.len() * core::mem::size_of::<u32>();
    SurfaceBankSummary {
        atom_records: banks.atoms.len(),
        program_records: banks.programs.len(),
        copy_span_records: banks.copy_spans.len(),
        program_atom_index_entries: banks.program_atom_index.len(),
        atom_record_bytes,
        program_record_bytes,
        copy_span_record_bytes,
        program_atom_index_bytes,
        total_surface_memory_bytes: atom_record_bytes
            + program_record_bytes
            + copy_span_record_bytes
            + program_atom_index_bytes,
        evidence_bytes_borrowed_from_corpus: banks.evidence.len(),
        hot_core_contains_utf8: false,
    }
}

fn sample_banks() -> SurfaceBanks {
    SurfaceBanks {
        atoms: [
            MaterialAtom {
                record: SurfaceAtom16 {
                    atom_id: 42_101,
                    value_hash: 0x2000_0A11,
                    payload: 0,
                    kind: 3,
                    script: 1,
                    flags: 0,
                },
                text: "invoic",
            },
            MaterialAtom {
                record: SurfaceAtom16 {
                    atom_id: 42_201,
                    value_hash: 0x2000_0E01,
                    payload: 0,
                    kind: 4,
                    script: 1,
                    flags: 0,
                },
                text: "e",
            },
            MaterialAtom {
                record: SurfaceAtom16 {
                    atom_id: 42_202,
                    value_hash: 0x2000_1A19,
                    payload: 0,
                    kind: 4,
                    script: 1,
                    flags: 0,
                },
                text: "ing",
            },
            MaterialAtom {
                record: SurfaceAtom16 {
                    atom_id: 49_122,
                    value_hash: 0xFF00_007A,
                    payload: b'z' as u32,
                    kind: 9,
                    script: 0,
                    flags: 1,
                },
                text: "",
            },
            MaterialAtom {
                record: SurfaceAtom16 {
                    atom_id: 49_120,
                    value_hash: 0xFF00_0078,
                    payload: b'x' as u32,
                    kind: 9,
                    script: 0,
                    flags: 1,
                },
                text: "",
            },
            MaterialAtom {
                record: SurfaceAtom16 {
                    atom_id: 49_113,
                    value_hash: 0xFF00_0071,
                    payload: b'q' as u32,
                    kind: 9,
                    script: 0,
                    flags: 1,
                },
                text: "",
            },
        ],
        programs: [
            SurfaceProgram32 {
                program_id: 61_001,
                atom_start: 0,
                root_atom_id: 42_101,
                morpheme_mask: 0b11,
                surface_hash: 0xB117_0001,
                checksum: 0xA001_A001,
                atom_count: 2,
                copy_span_start: 0,
                copy_span_count: 0,
                mode_bits: 0b0001,
            },
            SurfaceProgram32 {
                program_id: 61_004,
                atom_start: 2,
                root_atom_id: 42_101,
                morpheme_mask: 0b101,
                surface_hash: 0xB117_0004,
                checksum: 0xA004_A004,
                atom_count: 2,
                copy_span_start: 0,
                copy_span_count: 0,
                mode_bits: 0b0001,
            },
            SurfaceProgram32 {
                program_id: 61_002,
                atom_start: 0,
                root_atom_id: 0,
                morpheme_mask: 0,
                surface_hash: 0xC0DE_0603,
                checksum: 0xA002_A002,
                atom_count: 0,
                copy_span_start: 0,
                copy_span_count: 1,
                mode_bits: 0b0010,
            },
            SurfaceProgram32 {
                program_id: 61_003,
                atom_start: 4,
                root_atom_id: 49_122,
                morpheme_mask: 0,
                surface_hash: 0xFFFF_0001,
                checksum: 0xA003_A003,
                atom_count: 3,
                copy_span_start: 0,
                copy_span_count: 0,
                mode_bits: 0b0100,
            },
        ],
        copy_spans: [EvidenceCopySpan24 {
            evidence_ref: 10_001,
            byte_start: 8,
            surface_hash: 0xC0DE_0603,
            program_id: 61_002,
            checksum: 0xCC00_0603,
            byte_len: 25,
            mode_bits: 0b0010,
        }],
        program_atom_index: [42_101, 42_201, 42_101, 42_202, 49_122, 49_120, 49_113],
        evidence: b"prefix::PI-HL-RLTG-GZ-20260611-03::suffix",
    }
}
