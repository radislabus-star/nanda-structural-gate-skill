//! Literature-grounded word birth mechanism for LLMWave-Big.

use serde::Serialize;

pub(crate) const LEXICAL_BIRTH_VERSION: &str = "llmwave-big-v246-lexical-birth";
const ACCEPT_SCORE: i16 = 96;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct LexicalBirthCandidate32 {
    pub provisional_symbol_id: u32,
    pub surface_hash: u32,
    pub context_centroid_id: u32,
    pub segmentation_score: u16,
    pub fast_mapping_score: u16,
    pub cross_situational_score: u16,
    pub usage_score: u16,
    pub grammar_score: u16,
    pub attractor_margin: i16,
    pub anti_confusion_penalty: i16,
    pub state: u8,
    pub flags: u8,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct LexicalBindingRecord32 {
    pub symbol_id: u32,
    pub surface_hash: u32,
    pub lemma_id: u32,
    pub concept_centroid_id: u32,
    pub context_centroid_id: u32,
    pub cleanup_target_id: u32,
    pub root_id: u16,
    pub morpheme_id: u16,
    pub syntactic_frame_id: u16,
    pub evidence_refs: u16,
}

#[derive(Serialize, Clone)]
pub(crate) struct LexicalBirthReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub read_as: &'static str,
    pub word_definition: &'static str,
    pub literature_scope: Vec<LiteratureAnchor>,
    pub storage_laws: Vec<StorageLaw>,
    pub birth_stages: Vec<BirthStage>,
    pub record_formats: Vec<RecordFormat>,
    pub sample: LexicalBirthSample,
    pub rejection_control: LexicalBirthSample,
    pub claim_boundary: LexicalBirthClaimBoundary,
    pub next_engine_steps: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LiteratureAnchor {
    pub line: &'static str,
    pub source: &'static str,
    pub imported_mechanism: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct StorageLaw {
    pub source_family: &'static str,
    pub stores_word_as: &'static str,
    pub key_mechanism: &'static str,
    pub engine_boundary: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct BirthStage {
    pub stage: &'static str,
    pub literature_line: &'static str,
    pub input_signal: &'static str,
    pub memory_operation: &'static str,
    pub gate: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct RecordFormat {
    pub name: &'static str,
    pub bytes: usize,
    pub role: &'static str,
    pub hot_core_visibility: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LexicalBirthSample {
    pub surface: &'static str,
    pub meaning_hint: &'static str,
    pub candidate_record: LexicalBirthCandidate32,
    pub binding_record: Option<LexicalBindingRecord32>,
    pub gate: LexicalBirthGate,
    pub timeline: Vec<TraceStep>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LexicalBirthGate {
    pub segmentation_ready: bool,
    pub fast_mapping_ready: bool,
    pub cross_situational_ready: bool,
    pub usage_ready: bool,
    pub grammar_ready: bool,
    pub attractor_ready: bool,
    pub anti_confusion_clear: bool,
    pub total_score: i16,
    pub accepted: bool,
    pub verdict: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct TraceStep {
    pub step: &'static str,
    pub observation: &'static str,
    pub effect: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LexicalBirthClaimBoundary {
    pub corpus_proven: bool,
    pub generator_ready: bool,
    pub nonlinear_density_proven: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

pub(crate) fn build_lexical_birth_report() -> LexicalBirthReport {
    let sample_candidate = LexicalBirthCandidate32 {
        provisional_symbol_id: 70_001,
        surface_hash: 0xB117_0001,
        context_centroid_id: 2_101,
        segmentation_score: 18,
        fast_mapping_score: 14,
        cross_situational_score: 24,
        usage_score: 18,
        grammar_score: 17,
        attractor_margin: 16,
        anti_confusion_penalty: 4,
        state: 2,
        flags: 0,
        reserved: 0,
    };
    let rejection_candidate = LexicalBirthCandidate32 {
        provisional_symbol_id: 70_002,
        surface_hash: 0xB117_0002,
        context_centroid_id: 2_199,
        segmentation_score: 16,
        fast_mapping_score: 9,
        cross_situational_score: 7,
        usage_score: 5,
        grammar_score: 4,
        attractor_margin: 3,
        anti_confusion_penalty: 24,
        state: 1,
        flags: 0,
        reserved: 0,
    };
    let sample_gate = evaluate_birth(sample_candidate);
    let rejection_gate = evaluate_birth(rejection_candidate);

    LexicalBirthReport {
        mode: "llmwave-big-lexical-birth",
        version: LEXICAL_BIRTH_VERSION,
        roadmap_block: "v246-v252",
        verdict: "LEXICAL_BIRTH_MECHANISM_READY",
        read_as: "a literature-grounded mechanism contract, not proof that a real corpus has learned new words",
        word_definition:
            "stable binding of surface form, meaning context, syntactic behavior, usage memory, and cleanup attractor",
        literature_scope: literature_scope(),
        storage_laws: storage_laws(),
        birth_stages: birth_stages(),
        record_formats: vec![
            RecordFormat {
                name: "LexicalBirthCandidate32",
                bytes: core::mem::size_of::<LexicalBirthCandidate32>(),
                role: "provisional word under lexicalization test",
                hot_core_visibility: "compact_ids_scores_centroid_and_penalty",
            },
            RecordFormat {
                name: "LexicalBindingRecord32",
                bytes: core::mem::size_of::<LexicalBindingRecord32>(),
                role: "accepted word binding for surface_to_lemma_to_concept cleanup",
                hot_core_visibility: "symbol_id_surface_hash_centroids_cleanup_target",
            },
        ],
        sample: LexicalBirthSample {
            surface: "novel_business_term",
            meaning_hint: "a repeated document-role term observed across contexts",
            candidate_record: sample_candidate,
            binding_record: if sample_gate.accepted {
                Some(sample_binding_record())
            } else {
                None
            },
            gate: sample_gate,
            timeline: accepted_timeline(),
        },
        rejection_control: LexicalBirthSample {
            surface: "ambiguous_noise_piece",
            meaning_hint: "surface fragment with weak cross-situational support and high confusion",
            candidate_record: rejection_candidate,
            binding_record: None,
            gate: rejection_gate,
            timeline: rejected_timeline(),
        },
        claim_boundary: LexicalBirthClaimBoundary {
            corpus_proven: false,
            generator_ready: false,
            nonlinear_density_proven: false,
            safe_claim:
                "LLMWave-Big now has a staged word-birth contract derived from language-acquisition and lexical-memory literature",
            forbidden_claims: vec![
                "the model can invent reliable new words from hashes alone",
                "a real corpus has proven lexical growth",
                "new words are generated without an invertible surface dictionary",
                "this proves nonlinear memory density",
            ],
        },
        next_engine_steps: vec![
            "add physical surface dictionary token_id_to_utf8",
            "collect corpus observations into LexicalBirthCandidate32 records",
            "run cross_situational_birth_eval against lexical and random baselines",
            "promote only accepted candidates into LexicalBindingRecord32",
            "connect accepted bindings to L2 candidate cache and L3 schema bias",
        ],
    }
}

fn evaluate_birth(candidate: LexicalBirthCandidate32) -> LexicalBirthGate {
    let segmentation_ready = candidate.segmentation_score >= 14;
    let fast_mapping_ready = candidate.fast_mapping_score >= 10;
    let cross_situational_ready = candidate.cross_situational_score >= 18;
    let usage_ready = candidate.usage_score >= 12;
    let grammar_ready = candidate.grammar_score >= 12;
    let attractor_ready = candidate.attractor_margin >= 12;
    let anti_confusion_clear = candidate.anti_confusion_penalty <= 8;
    let total_score = candidate
        .segmentation_score
        .saturating_add(candidate.fast_mapping_score)
        .saturating_add(candidate.cross_situational_score)
        .saturating_add(candidate.usage_score)
        .saturating_add(candidate.grammar_score) as i16
        + candidate.attractor_margin
        - candidate.anti_confusion_penalty;
    let accepted = total_score >= ACCEPT_SCORE
        && segmentation_ready
        && fast_mapping_ready
        && cross_situational_ready
        && usage_ready
        && grammar_ready
        && attractor_ready
        && anti_confusion_clear;
    LexicalBirthGate {
        segmentation_ready,
        fast_mapping_ready,
        cross_situational_ready,
        usage_ready,
        grammar_ready,
        attractor_ready,
        anti_confusion_clear,
        total_score,
        accepted,
        verdict: if accepted {
            "WORD_ACCEPTED"
        } else {
            "WORD_REJECTED_OR_WAITING"
        },
    }
}

fn sample_binding_record() -> LexicalBindingRecord32 {
    LexicalBindingRecord32 {
        symbol_id: 70_001,
        surface_hash: 0xB117_0001,
        lemma_id: 8_101,
        concept_centroid_id: 12_201,
        context_centroid_id: 2_101,
        cleanup_target_id: 70_001,
        root_id: 501,
        morpheme_id: 1_501,
        syntactic_frame_id: 41,
        evidence_refs: 7,
    }
}

fn literature_scope() -> Vec<LiteratureAnchor> {
    vec![
        LiteratureAnchor {
            line: "mental_lexicon",
            source: "Levelt lexical access: concept -> lemma -> word form",
            imported_mechanism: "word is layered, not a raw string",
        },
        LiteratureAnchor {
            line: "triangle_connectionist",
            source: "Plaut/McClelland/Seidenberg/Patterson orthography-phonology-semantics",
            imported_mechanism:
                "word is a distributed stable pattern across form and meaning fields",
        },
        LiteratureAnchor {
            line: "usage_exemplar",
            source: "Bybee and Pierrehumbert usage/exemplar lexicon",
            imported_mechanism: "frequency and repeated traces strengthen storage",
        },
        LiteratureAnchor {
            line: "distributional_semantics",
            source: "Harris/Firth distributional tradition",
            imported_mechanism: "meaning grows from recurring contexts",
        },
        LiteratureAnchor {
            line: "self_organizing_lexicon",
            source: "DevLex / Li, Farkas, MacWhinney",
            imported_mechanism: "surface and meaning maps bind through learned association",
        },
        LiteratureAnchor {
            line: "statistical_segmentation",
            source: "Saffran, Aslin, Newport statistical word segmentation",
            imported_mechanism: "candidate surfaces first emerge from boundary statistics",
        },
        LiteratureAnchor {
            line: "cross_situational_learning",
            source: "Smith and Yu cross-situational word learning",
            imported_mechanism: "ambiguous observations converge on a common referent/context",
        },
        LiteratureAnchor {
            line: "attractor_memory",
            source: "Hopfield-style associative memory",
            imported_mechanism: "partial/noisy cues must settle to the same word basin",
        },
    ]
}

fn storage_laws() -> Vec<StorageLaw> {
    vec![
        StorageLaw {
            source_family: "mental_lexicon",
            stores_word_as: "concept_plus_lemma_plus_form",
            key_mechanism: "separate access stages for meaning, syntax, and surface",
            engine_boundary: "do not collapse word into a single token hash",
        },
        StorageLaw {
            source_family: "connectionist_triangle",
            stores_word_as: "orthography_phonology_semantics_pattern",
            key_mechanism: "distributed activation across form and meaning",
            engine_boundary: "L2 surface and L3 concept/schema must bind without sharing storage",
        },
        StorageLaw {
            source_family: "usage_exemplar",
            stores_word_as: "many weighted traces",
            key_mechanism: "frequency, recency, and similarity strengthen access",
            engine_boundary: "lexicalization needs repeated evidence, not one pretty peak",
        },
        StorageLaw {
            source_family: "distributional",
            stores_word_as: "context_centroid",
            key_mechanism: "stable neighborhood across observations",
            engine_boundary: "birth candidate must accumulate context support",
        },
        StorageLaw {
            source_family: "attractor_cleanup",
            stores_word_as: "recoverable cleanup target",
            key_mechanism: "partial cues settle to one stable basin",
            engine_boundary: "accepted words need margin and anti-confusion checks",
        },
    ]
}

fn birth_stages() -> Vec<BirthStage> {
    vec![
        BirthStage {
            stage: "segmentation",
            literature_line: "statistical_word_segmentation",
            input_signal: "surface stream boundary and transition drop",
            memory_operation: "open provisional surface candidate",
            gate: "segmentation_score >= 14",
        },
        BirthStage {
            stage: "fast_mapping",
            literature_line: "early_word_learning",
            input_signal: "novel surface co-occurs with a plausible referent or role",
            memory_operation: "attach weak provisional symbol_id",
            gate: "fast_mapping_score >= 10",
        },
        BirthStage {
            stage: "cross_situational_convergence",
            literature_line: "cross_situational_learning",
            input_signal: "same surface appears across different contexts with one common center",
            memory_operation: "raise context_centroid stability",
            gate: "cross_situational_score >= 18",
        },
        BirthStage {
            stage: "usage_exemplar_strengthening",
            literature_line: "usage_based_exemplar_lexicon",
            input_signal: "repeated successful traces",
            memory_operation: "increase usage_score and evidence_refs",
            gate: "usage_score >= 12",
        },
        BirthStage {
            stage: "grammar_integration",
            literature_line: "lexicon_grammar_coupling",
            input_signal: "candidate behaves consistently in syntactic frames",
            memory_operation: "bind lemma and syntactic_frame_id",
            gate: "grammar_score >= 12",
        },
        BirthStage {
            stage: "attractor_cleanup",
            literature_line: "associative_memory",
            input_signal: "partial cues retrieve same candidate",
            memory_operation: "assign cleanup_target_id",
            gate: "attractor_margin >= 12",
        },
        BirthStage {
            stage: "anti_confusion",
            literature_line: "superposition_crosstalk_guard",
            input_signal: "neighbor words and false readings compete",
            memory_operation: "reject or delay if candidate steals another basin",
            gate: "anti_confusion_penalty <= 8",
        },
    ]
}

fn accepted_timeline() -> Vec<TraceStep> {
    vec![
        TraceStep {
            step: "surface_boundary",
            observation: "the same surface chunk recurs with clear boundaries",
            effect: "provisional candidate opens",
        },
        TraceStep {
            step: "meaning_hint",
            observation: "contexts point toward a document-role meaning",
            effect: "weak symbol_id receives a context centroid",
        },
        TraceStep {
            step: "context_convergence",
            observation: "different observations keep one shared center",
            effect: "cross-situational score crosses threshold",
        },
        TraceStep {
            step: "grammar_frame",
            observation: "candidate appears in one stable syntactic role",
            effect: "lemma and syntactic frame are bound",
        },
        TraceStep {
            step: "cleanup",
            observation: "partial cues recover the same candidate with margin",
            effect: "LexicalBindingRecord32 is created",
        },
    ]
}

fn rejected_timeline() -> Vec<TraceStep> {
    vec![
        TraceStep {
            step: "surface_boundary",
            observation: "surface fragment appears but contexts disagree",
            effect: "candidate stays provisional",
        },
        TraceStep {
            step: "confusion_check",
            observation: "fragment overlaps a stronger existing basin",
            effect: "anti-confusion penalty blocks promotion",
        },
    ]
}
