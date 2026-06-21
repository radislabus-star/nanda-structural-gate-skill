//! L2 surface-language field boundary: token, root, morpheme, and word candidates.

use serde::Serialize;
use std::cmp::Reverse;

use super::l3_schema_field::L3ToL2Bias;
use super::nanda_6m;

pub(crate) const L2_WORD_FIELD_VERSION: &str = "llmwave-big-v390-l2-wave-runtime";
pub(crate) const L2_MIN_READY_MARGIN: i16 = 12;
pub(crate) const L2_CANDIDATE_CACHE_MIN: usize = 128;
pub(crate) const L2_CANDIDATE_CACHE_MAX: usize = 4096;
pub(crate) const L2_RUNTIME_DIM: usize = nanda_6m::WAVE_DIM;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct L2Candidate32 {
    pub token_id: u32,
    pub surface_hash: u32,
    pub root_id: u32,
    pub morpheme_id: u32,
    pub language: u8,
    pub form_kind: u8,
    pub style: u8,
    pub flags: u8,
    pub prefix_score: i16,
    pub local_score: i16,
    pub l3_bias: i16,
    pub anti_score: i16,
    pub final_score: i16,
    pub reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ActiveL2Slice {
    pub language: u8,
    pub domain: u8,
    pub style: u8,
    pub flags: u8,
    pub prefix_hash: u32,
    pub candidate_start: u32,
    pub candidate_count: u16,
    pub reserved: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrefixWave {
    pub values: [i16; nanda_6m::WAVE_DIM],
}

impl Default for PrefixWave {
    fn default() -> Self {
        Self {
            values: [0; nanda_6m::WAVE_DIM],
        }
    }
}

impl PrefixWave {
    pub(crate) fn update_byte(&mut self, byte: u8, position: u16) {
        let mut state = u32::from(byte) | (u32::from(position) << 16) | 0x51A7_0000;
        for value in &mut self.values {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let bit = ((state >> 31) as i16 * 2) - 1;
            *value = value.saturating_add(bit);
        }
    }

    pub(crate) fn energy(&self) -> i64 {
        let mut out = 0_i64;
        for value in self.values {
            out += i64::from(value) * i64::from(value);
        }
        out
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct L2WordFieldReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub owns: Vec<&'static str>,
    pub active_slice: ActiveL2Slice,
    pub prefix_wave: PrefixWaveReport,
    pub candidate_cache: CandidateCacheReport,
    pub runtime_field: L2WaveRuntimeReport,
    pub l3_bias: L3BiasReport,
    pub anti_wave: AntiWaveReport,
    pub sync_policy: SyncPolicyReport,
    pub multilingual_surface: Vec<SurfaceBankReport>,
    pub eval_metrics: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct PrefixWaveReport {
    pub update: &'static str,
    pub prefix: &'static str,
    pub energy: i64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CandidateCacheReport {
    pub min_candidates: usize,
    pub max_candidates: usize,
    pub record_bytes: usize,
    pub sample: Vec<CandidateReport>,
    pub top_token_label: &'static str,
    pub margin: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CandidateReport {
    pub label: &'static str,
    pub token_id: u32,
    pub prefix_resonance: i16,
    pub family_resonance: i16,
    pub suffix_resonance: i16,
    pub l3_phase_bias: i16,
    pub final_score: i16,
    pub anti_score: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct L2WaveRuntimeReport {
    pub version: &'static str,
    pub input_prefix: &'static str,
    pub runtime_dim: usize,
    pub cycle: &'static str,
    pub top_surface: &'static str,
    pub top_family: &'static str,
    pub margin: i16,
    pub anti_wave_suppressed: &'static str,
    pub field_state: &'static str,
    pub candidates: Vec<L2RuntimeCandidateReport>,
    pub claim_boundary: L2RuntimeClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct L2RuntimeCandidateReport {
    pub surface: &'static str,
    pub family: &'static str,
    pub suffix: &'static str,
    pub prefix_resonance: i16,
    pub family_resonance: i16,
    pub suffix_resonance: i16,
    pub l3_phase_bias: i16,
    pub anti_wave: i16,
    pub final_score: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct L2RuntimeClaimBoundary {
    pub hot_loop_uses_json: bool,
    pub hot_loop_uses_heap: bool,
    pub l2_l3_storage_mixed: bool,
    pub chat_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct L3BiasReport {
    pub route: &'static str,
    pub operator: &'static str,
    pub role_expectation: &'static str,
    pub style: &'static str,
    pub bias_record: L3ToL2Bias,
}

#[derive(Serialize, Clone)]
pub(crate) struct AntiWaveReport {
    pub rule: &'static str,
    pub suppressed_candidate: &'static str,
    pub suppression_score: i16,
}

#[derive(Serialize, Clone)]
pub(crate) struct SyncPolicyReport {
    pub l2_update: &'static str,
    pub l3_update: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct SurfaceBankReport {
    pub language: &'static str,
    pub surface: &'static str,
}

pub(crate) fn build_l2_word_field_report(bias: L3ToL2Bias) -> L2WordFieldReport {
    let mut prefix_wave = PrefixWave::default();
    prefix_wave.update_byte(b'i', 0);
    prefix_wave.update_byte(b'n', 1);
    let active_slice = ActiveL2Slice {
        language: 2,
        domain: 4,
        style: 2,
        flags: 0,
        prefix_hash: 0x1A2B_0001,
        candidate_start: 0,
        candidate_count: 4,
        reserved: 0,
    };
    let candidates = score_sample_candidates(bias);
    let runtime_field = run_l2_wave_runtime(bias);
    let (top, runner_up) = top_two(&candidates);
    let margin = top.final_score - runner_up.final_score;
    let verdict = if margin >= L2_MIN_READY_MARGIN {
        "L2_READY"
    } else if bias.strength <= 0 {
        "L2_NEEDS_L3"
    } else {
        "L2_AMBIGUOUS"
    };
    L2WordFieldReport {
        mode: "llmwave-big-l2-word-field",
        version: L2_WORD_FIELD_VERSION,
        roadmap_block: "v361-v390",
        verdict,
        owns: vec![
            "tokens",
            "roots",
            "morphemes",
            "forms",
            "synonyms",
            "language_variants",
            "prefix_continuations",
        ],
        active_slice,
        prefix_wave: PrefixWaveReport {
            update: "each_character_updates_prefix_wave_local_context_wave_candidate_wave",
            prefix: "in",
            energy: prefix_wave.energy(),
        },
        candidate_cache: CandidateCacheReport {
            min_candidates: L2_CANDIDATE_CACHE_MIN,
            max_candidates: L2_CANDIDATE_CACHE_MAX,
            record_bytes: core::mem::size_of::<L2Candidate32>(),
            sample: candidates
                .iter()
                .map(|candidate| CandidateReport {
                    label: candidate_label(candidate.token_id),
                    token_id: candidate.token_id,
                    prefix_resonance: candidate.prefix_score,
                    family_resonance: candidate.local_score,
                    suffix_resonance: 0,
                    l3_phase_bias: candidate.l3_bias,
                    final_score: candidate.final_score,
                    anti_score: candidate.anti_score,
                })
                .collect(),
            top_token_label: candidate_label(top.token_id),
            margin,
        },
        runtime_field,
        l3_bias: L3BiasReport {
            route: "business_doc",
            operator: "issues",
            role_expectation: "supplier/document",
            style: "formal_ru",
            bias_record: bias,
        },
        anti_wave: AntiWaveReport {
            rule: "suppress_prefix_match_that_breaks_active_schema",
            suppressed_candidate: "inventory",
            suppression_score: 38,
        },
        sync_policy: SyncPolicyReport {
            l2_update: "per_keystroke",
            l3_update: "word_boundary_punctuation_semantic_shift",
        },
        multilingual_surface: vec![
            surface("ru", "postavshchik_vystavlyaet_invois"),
            surface("en", "supplier_issues_invoice"),
            surface("cn", "supplier_issues_invoice_cn"),
        ],
        eval_metrics: vec![
            "prefix_accuracy",
            "semantic_consistency",
            "role_safety",
            "language_switch",
        ],
    }
}

#[derive(Clone, Copy)]
struct L2RuntimeCandidate {
    surface: &'static str,
    family: &'static str,
    suffix: &'static str,
    l3_match: i16,
    collision_risk: i16,
}

fn run_l2_wave_runtime(bias: L3ToL2Bias) -> L2WaveRuntimeReport {
    let input_prefix = "счет";
    let mut prefix_wave = PrefixWave::default();
    for (position, byte) in input_prefix.bytes().enumerate() {
        prefix_wave.update_byte(byte, position as u16);
    }
    let candidates = [
        L2RuntimeCandidate {
            surface: "счет",
            family: "счет",
            suffix: "",
            l3_match: 14,
            collision_risk: 0,
        },
        L2RuntimeCandidate {
            surface: "счета",
            family: "счет",
            suffix: "а",
            l3_match: 18,
            collision_risk: 0,
        },
        L2RuntimeCandidate {
            surface: "счете",
            family: "счет",
            suffix: "е",
            l3_match: 22,
            collision_risk: 0,
        },
        L2RuntimeCandidate {
            surface: "счетчик",
            family: "счетчик",
            suffix: "",
            l3_match: -8,
            collision_risk: 42,
        },
        L2RuntimeCandidate {
            surface: "договор",
            family: "договор",
            suffix: "",
            l3_match: 8,
            collision_risk: 0,
        },
    ];
    let mut scored = [
        score_runtime_candidate(input_prefix, &prefix_wave, candidates[0], bias),
        score_runtime_candidate(input_prefix, &prefix_wave, candidates[1], bias),
        score_runtime_candidate(input_prefix, &prefix_wave, candidates[2], bias),
        score_runtime_candidate(input_prefix, &prefix_wave, candidates[3], bias),
        score_runtime_candidate(input_prefix, &prefix_wave, candidates[4], bias),
    ];
    scored.sort_by_key(|candidate| Reverse(candidate.final_score));
    let top = scored[0].clone();
    let runner_up = scored[1].clone();
    let margin = top.final_score.saturating_sub(runner_up.final_score);
    let field_state = if top.family == "счет" && margin >= L2_MIN_READY_MARGIN {
        "L2_WAVE_RUNTIME_READY_NOT_CHAT"
    } else {
        "L2_WAVE_RUNTIME_REVIEW"
    };

    L2WaveRuntimeReport {
        version: "v361-v390",
        input_prefix,
        runtime_dim: L2_RUNTIME_DIM,
        cycle: "prefix_wave_dot_surface_wave_plus_family_suffix_l3_bias_minus_anti_wave",
        top_surface: top.surface,
        top_family: top.family,
        margin,
        anti_wave_suppressed: "счетчик",
        field_state,
        candidates: scored.to_vec(),
        claim_boundary: L2RuntimeClaimBoundary {
            hot_loop_uses_json: false,
            hot_loop_uses_heap: false,
            l2_l3_storage_mixed: false,
            chat_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "L2 runtime field can rank surface candidates with prefix resonance, L3 phase bias, and local anti-wave suppression",
        },
    }
}

fn score_runtime_candidate(
    input_prefix: &str,
    prefix_wave: &PrefixWave,
    candidate: L2RuntimeCandidate,
    bias: L3ToL2Bias,
) -> L2RuntimeCandidateReport {
    let surface_wave = surface_wave(candidate.surface);
    let wave_resonance = dot_prefix_surface(prefix_wave, &surface_wave) / 128;
    let prefix_contour = if candidate.surface.starts_with(input_prefix) {
        24 + (candidate
            .surface
            .len()
            .saturating_sub(input_prefix.len())
            .min(8) as i16)
    } else {
        -18
    };
    let prefix_resonance = clamp_i64_to_i16(wave_resonance + i64::from(prefix_contour));
    let family_resonance = if candidate.family == "счет" {
        28
    } else {
        -12
    };
    let suffix_resonance = match candidate.suffix {
        "е" => 22,
        "а" => 10,
        "" => 6,
        _ => 2,
    };
    let l3_phase_bias = clamp_i64_to_i16(i64::from(candidate.l3_match + bias.strength / 4));
    let anti_wave = candidate.collision_risk;
    let final_score = prefix_resonance
        .saturating_add(family_resonance)
        .saturating_add(suffix_resonance)
        .saturating_add(l3_phase_bias)
        .saturating_sub(anti_wave);

    L2RuntimeCandidateReport {
        surface: candidate.surface,
        family: candidate.family,
        suffix: candidate.suffix,
        prefix_resonance,
        family_resonance,
        suffix_resonance,
        l3_phase_bias,
        anti_wave,
        final_score,
    }
}

fn surface_wave(surface: &str) -> [i8; L2_RUNTIME_DIM] {
    let mut out = [0_i8; L2_RUNTIME_DIM];
    let mut state = 0xC0DE_5102_u32 ^ surface.len() as u32;
    for (position, byte) in surface.bytes().enumerate() {
        state ^= u32::from(byte) << ((position % 4) * 8);
        state = state
            .rotate_left(5)
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
    }
    for value in &mut out {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        *value = if state & 0x8000_0000 == 0 { -1 } else { 1 };
    }
    out
}

fn dot_prefix_surface(prefix_wave: &PrefixWave, surface_wave: &[i8; L2_RUNTIME_DIM]) -> i64 {
    prefix_wave
        .values
        .iter()
        .zip(surface_wave.iter())
        .map(|(left, right)| i64::from(*left) * i64::from(*right))
        .sum()
}

fn clamp_i64_to_i16(value: i64) -> i16 {
    value.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16
}

fn score_sample_candidates(bias: L3ToL2Bias) -> [L2Candidate32; 4] {
    let mut candidates = [
        candidate(L2CandidateInput {
            token_id: 10,
            surface_hash: 0xAA00_0010,
            root_id: 100,
            morpheme_id: 1_000,
            prefix_score: 34,
            local_score: 22,
            anti_score: 0,
        }),
        candidate(L2CandidateInput {
            token_id: 11,
            surface_hash: 0xAA00_0011,
            root_id: 101,
            morpheme_id: 1_001,
            prefix_score: 31,
            local_score: 18,
            anti_score: 0,
        }),
        candidate(L2CandidateInput {
            token_id: 12,
            surface_hash: 0xAA00_0012,
            root_id: 102,
            morpheme_id: 1_002,
            prefix_score: 26,
            local_score: 17,
            anti_score: 0,
        }),
        candidate(L2CandidateInput {
            token_id: 13,
            surface_hash: 0xAA00_0013,
            root_id: 103,
            morpheme_id: 1_003,
            prefix_score: 33,
            local_score: 10,
            anti_score: 38,
        }),
    ];
    for candidate in &mut candidates {
        candidate.l3_bias = l3_bias_for_candidate(candidate.token_id, bias);
        candidate.final_score = candidate
            .prefix_score
            .saturating_add(candidate.local_score)
            .saturating_add(candidate.l3_bias)
            .saturating_sub(candidate.anti_score);
    }
    candidates
}

struct L2CandidateInput {
    token_id: u32,
    surface_hash: u32,
    root_id: u32,
    morpheme_id: u32,
    prefix_score: i16,
    local_score: i16,
    anti_score: i16,
}

fn candidate(input: L2CandidateInput) -> L2Candidate32 {
    L2Candidate32 {
        token_id: input.token_id,
        surface_hash: input.surface_hash,
        root_id: input.root_id,
        morpheme_id: input.morpheme_id,
        language: 2,
        form_kind: 1,
        style: 2,
        flags: 0,
        prefix_score: input.prefix_score,
        local_score: input.local_score,
        l3_bias: 0,
        anti_score: input.anti_score,
        final_score: 0,
        reserved: 0,
    }
}

fn l3_bias_for_candidate(token_id: u32, bias: L3ToL2Bias) -> i16 {
    match token_id {
        10 => bias.strength,
        11 => bias.strength - 10,
        12 => bias.strength - 14,
        13 => -bias.strength,
        _ => 0,
    }
}

fn top_two(candidates: &[L2Candidate32; 4]) -> (L2Candidate32, L2Candidate32) {
    let mut first = candidates[0];
    let mut second = candidates[1];
    if second.final_score > first.final_score {
        core::mem::swap(&mut first, &mut second);
    }
    for candidate in &candidates[2..] {
        if candidate.final_score > first.final_score {
            second = first;
            first = *candidate;
        } else if candidate.final_score > second.final_score {
            second = *candidate;
        }
    }
    (first, second)
}

fn candidate_label(token_id: u32) -> &'static str {
    match token_id {
        10 => "invoice",
        11 => "PI",
        12 => "schet",
        13 => "inventory",
        _ => "unknown",
    }
}

fn surface(language: &'static str, surface: &'static str) -> SurfaceBankReport {
    SurfaceBankReport { language, surface }
}
