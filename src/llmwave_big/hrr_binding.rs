//! HRR/VSA-style role-filler binding for LLMWave-Big.

use serde::Serialize;

pub(crate) const HRR_BINDING_VERSION: &str = "llmwave-big-v430-hrr-binding";
pub(crate) const HRR_DIM: usize = 1024;
const CLEANUP_MIN_MARGIN: i32 = 500;

#[derive(Clone, Copy)]
struct Wave1024 {
    values: [i16; HRR_DIM],
}

impl Wave1024 {
    fn zero() -> Self {
        Self {
            values: [0; HRR_DIM],
        }
    }

    fn symbol(label: &str) -> Self {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64 ^ label.len() as u64;
        for byte in label.bytes() {
            state ^= u64::from(byte);
            state = state.rotate_left(13).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        }

        let mut out = [0_i16; HRR_DIM];
        for value in &mut out {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
            *value = if state & 1 == 0 { -1 } else { 1 };
        }
        Self { values: out }
    }

    fn add_assign(&mut self, other: &Self) {
        for (left, right) in self.values.iter_mut().zip(other.values.iter()) {
            *left = left.saturating_add(*right);
        }
    }

    fn add_noise(&mut self, seed: u64, amplitude: i16) {
        let mut state = seed;
        for value in &mut self.values {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let sign = if state & 0x8000_0000 == 0 { -1 } else { 1 };
            *value = value.saturating_add(sign * amplitude);
        }
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrBindingReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub wave_dim: usize,
    pub operation: HrrOperationReport,
    pub bindings: Vec<HrrBindingCaseReport>,
    pub cleanup: Vec<HrrCleanupCaseReport>,
    pub noise_eval: HrrNoiseEvalReport,
    pub collision_eval: HrrCollisionEvalReport,
    pub metrics: HrrBindingMetrics,
    pub claim_boundary: HrrBindingClaimBoundary,
    pub next_engine_steps: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrOperationReport {
    pub bind: &'static str,
    pub unbind: &'static str,
    pub cleanup: &'static str,
    pub hot_loop_uses_json: bool,
    pub hot_loop_uses_strings: bool,
    pub hot_loop_uses_heap: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrBindingCaseReport {
    pub role: &'static str,
    pub filler: &'static str,
    pub recovered: &'static str,
    pub runner_up: &'static str,
    pub top_score: i32,
    pub runner_up_score: i32,
    pub margin: i32,
    pub exact: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrCleanupCaseReport {
    pub cue: &'static str,
    pub recovered: &'static str,
    pub expected: &'static str,
    pub runner_up: &'static str,
    pub margin: i32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrNoiseEvalReport {
    pub noise_amplitude: i16,
    pub exact_after_noise: usize,
    pub total: usize,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrCollisionEvalReport {
    pub trap_role: &'static str,
    pub expected_filler: &'static str,
    pub rejected_filler: &'static str,
    pub recovered: &'static str,
    pub rejected_score: i32,
    pub expected_score: i32,
    pub rejected: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrBindingMetrics {
    pub role_recall: f32,
    pub cleanup_top1: f32,
    pub noisy_role_recall: f32,
    pub collision_reject_rate: f32,
    pub ambiguous_cleanup_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct HrrBindingClaimBoundary {
    pub hrr_binding_implemented: bool,
    pub cleanup_memory_implemented: bool,
    pub real_corpus_trained: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
    pub forbidden_claims: Vec<&'static str>,
}

#[derive(Clone, Copy)]
struct BindingSpec {
    role: &'static str,
    filler: &'static str,
}

#[derive(Clone, Copy)]
struct CleanupSymbol {
    label: &'static str,
    wave: Wave1024,
}

pub(crate) fn build_hrr_binding_report() -> HrrBindingReport {
    let bindings = [
        BindingSpec {
            role: "supplier",
            filler: "Honglu",
        },
        BindingSpec {
            role: "buyer",
            filler: "Rustrade",
        },
        BindingSpec {
            role: "document",
            filler: "invoice",
        },
        BindingSpec {
            role: "route",
            filler: "Guangzhou",
        },
    ];
    let cleanup = cleanup_symbols();
    let schema = build_schema_wave(&bindings);
    let binding_reports = bindings
        .iter()
        .map(|binding| recover_binding(&schema, *binding, &cleanup))
        .collect::<Vec<_>>();

    let mut noisy_schema = schema;
    noisy_schema.add_noise(0x51A7_4300, 1);
    let noisy_exact = bindings
        .iter()
        .filter(|binding| recover_binding(&noisy_schema, **binding, &cleanup).exact)
        .count();
    let noise_eval = HrrNoiseEvalReport {
        noise_amplitude: 1,
        exact_after_noise: noisy_exact,
        total: bindings.len(),
        state: if noisy_exact == bindings.len() {
            "NOISY_BINDING_STABLE"
        } else {
            "NOISY_BINDING_REVIEW"
        },
    };

    let collision_eval = collision_eval(&schema, &cleanup);
    let cleanup_reports = binding_reports
        .iter()
        .map(|binding| HrrCleanupCaseReport {
            cue: binding.role,
            recovered: binding.recovered,
            expected: binding.filler,
            runner_up: binding.runner_up,
            margin: binding.margin,
            state: if binding.margin >= CLEANUP_MIN_MARGIN {
                "CLEANUP_STABLE"
            } else {
                "CLEANUP_AMBIGUOUS"
            },
        })
        .collect::<Vec<_>>();
    let exact_count = binding_reports.iter().filter(|item| item.exact).count();
    let stable_cleanup = cleanup_reports
        .iter()
        .filter(|item| item.state == "CLEANUP_STABLE")
        .count();
    let ambiguous = cleanup_reports.len().saturating_sub(stable_cleanup);
    let cleanup_report_count = cleanup_reports.len();
    let collision_reject_rate = if collision_eval.rejected { 1.0 } else { 0.0 };
    let state = if exact_count == bindings.len()
        && noisy_exact == bindings.len()
        && collision_eval.rejected
        && ambiguous == 0
    {
        "HRR_BINDING_READY_NOT_NONLINEAR_PROOF"
    } else {
        "HRR_BINDING_REVIEW"
    };

    HrrBindingReport {
        mode: "llmwave-big-hrr-binding",
        version: HRR_BINDING_VERSION,
        roadmap_block: "v391-v430",
        verdict: state,
        wave_dim: HRR_DIM,
        operation: HrrOperationReport {
            bind: "bipolar_vsa_elementwise_multiply(role_wave, filler_wave)",
            unbind: "elementwise_multiply(schema_wave, role_wave)",
            cleanup: "nearest_known_filler_with_margin_gate",
            hot_loop_uses_json: false,
            hot_loop_uses_strings: false,
            hot_loop_uses_heap: false,
        },
        bindings: binding_reports,
        cleanup: cleanup_reports,
        noise_eval,
        collision_eval,
        metrics: HrrBindingMetrics {
            role_recall: ratio(exact_count, bindings.len()),
            cleanup_top1: ratio(stable_cleanup, cleanup_report_count),
            noisy_role_recall: ratio(noisy_exact, bindings.len()),
            collision_reject_rate,
            ambiguous_cleanup_rate: ratio(ambiguous, cleanup_report_count),
            state,
        },
        claim_boundary: HrrBindingClaimBoundary {
            hrr_binding_implemented: true,
            cleanup_memory_implemented: true,
            real_corpus_trained: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim:
                "LLMWave-Big can bind role/filler waves and recover fixture fillers through cleanup memory under small noise",
            forbidden_claims: vec![
                "HRR fixture binding proves nonlinear memory",
                "cleanup on four fillers proves language understanding",
                "circular convolution replaces schema eval",
                "LLMWave-Big is chat-ready",
            ],
        },
        next_engine_steps: vec![
            "connect HRR role/filler bindings to L3 schema records",
            "run role-reversal traps with larger filler dictionaries",
            "compare circular convolution against sparse triad baseline",
            "add cleanup ambiguity no-answer state to generation core",
            "measure bytes_per_useful_bound_fact under growing dictionaries",
        ],
    }
}

fn build_schema_wave(bindings: &[BindingSpec]) -> Wave1024 {
    let mut schema = Wave1024::zero();
    for binding in bindings {
        schema.add_assign(&bind(
            &Wave1024::symbol(binding.role),
            &Wave1024::symbol(binding.filler),
        ));
    }
    schema
}

fn recover_binding(
    schema: &Wave1024,
    binding: BindingSpec,
    cleanup: &[CleanupSymbol],
) -> HrrBindingCaseReport {
    let cue = unbind(schema, &Wave1024::symbol(binding.role));
    let (recovered, runner_up, top_score, runner_up_score) = cleanup_nearest(&cue, cleanup);
    HrrBindingCaseReport {
        role: binding.role,
        filler: binding.filler,
        recovered,
        runner_up,
        top_score,
        runner_up_score,
        margin: top_score - runner_up_score,
        exact: recovered == binding.filler,
    }
}

fn collision_eval(schema: &Wave1024, cleanup: &[CleanupSymbol]) -> HrrCollisionEvalReport {
    let cue = unbind(schema, &Wave1024::symbol("supplier"));
    let (recovered, _, _, _) = cleanup_nearest(&cue, cleanup);
    let expected_score = score_label(&cue, cleanup, "Honglu");
    let rejected_score = score_label(&cue, cleanup, "Rustrade");
    HrrCollisionEvalReport {
        trap_role: "supplier",
        expected_filler: "Honglu",
        rejected_filler: "Rustrade",
        recovered,
        rejected_score,
        expected_score,
        rejected: recovered == "Honglu" && expected_score > rejected_score,
    }
}

fn cleanup_symbols() -> Vec<CleanupSymbol> {
    [
        "Honglu",
        "Rustrade",
        "invoice",
        "Guangzhou",
        "Monster",
        "certificate",
    ]
    .iter()
    .map(|label| CleanupSymbol {
        label,
        wave: Wave1024::symbol(label),
    })
    .collect()
}

fn cleanup_nearest(
    cue: &Wave1024,
    cleanup: &[CleanupSymbol],
) -> (&'static str, &'static str, i32, i32) {
    let mut best = ("", i32::MIN);
    let mut second = ("", i32::MIN);
    for symbol in cleanup {
        let score = dot(cue, &symbol.wave);
        if score > best.1 {
            second = best;
            best = (symbol.label, score);
        } else if score > second.1 {
            second = (symbol.label, score);
        }
    }
    (best.0, second.0, best.1, second.1)
}

fn score_label(cue: &Wave1024, cleanup: &[CleanupSymbol], label: &str) -> i32 {
    cleanup
        .iter()
        .find(|symbol| symbol.label == label)
        .map(|symbol| dot(cue, &symbol.wave))
        .unwrap_or(i32::MIN)
}

fn bind(role: &Wave1024, filler: &Wave1024) -> Wave1024 {
    let mut out = [0_i16; HRR_DIM];
    for ((value, role_value), filler_value) in out
        .iter_mut()
        .zip(role.values.iter())
        .zip(filler.values.iter())
    {
        *value = role_value.saturating_mul(*filler_value);
    }
    Wave1024 { values: out }
}

fn unbind(schema: &Wave1024, role: &Wave1024) -> Wave1024 {
    let mut out = [0_i16; HRR_DIM];
    for ((value, schema_value), role_value) in out
        .iter_mut()
        .zip(schema.values.iter())
        .zip(role.values.iter())
    {
        *value = schema_value.saturating_mul(*role_value);
    }
    Wave1024 { values: out }
}

fn dot(left: &Wave1024, right: &Wave1024) -> i32 {
    left.values
        .iter()
        .zip(right.values.iter())
        .map(|(a, b)| i32::from(*a) * i32::from(*b))
        .sum()
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}
