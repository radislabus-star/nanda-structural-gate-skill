//! L3 schema binding over role/filler waves.

use serde::Serialize;

use super::schemas::{build_schema_atlas_report, SchemaRecord};

pub(crate) const L3_SCHEMA_BIND_VERSION: &str = "llmwave-big-v455-l3-schema-bind";
const WAVE_DIM: usize = 1024;

#[derive(Clone, Copy)]
struct Wave1024 {
    values: [i16; WAVE_DIM],
}

impl Wave1024 {
    fn zero() -> Self {
        Self {
            values: [0; WAVE_DIM],
        }
    }

    fn symbol(label: &str) -> Self {
        let mut state = 0xD1B5_AEED_4550_0001_u64 ^ label.len() as u64;
        for byte in label.bytes() {
            state ^= u64::from(byte);
            state = state.rotate_left(11).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
        let mut values = [0_i16; WAVE_DIM];
        for value in &mut values {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
            *value = if state & 1 == 0 { -1 } else { 1 };
        }
        Self { values }
    }

    fn add_assign(&mut self, other: &Self) {
        for (left, right) in self.values.iter_mut().zip(other.values.iter()) {
            *left = left.saturating_add(*right);
        }
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct L3SchemaBindReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub schema: L3SchemaBindingSchema,
    pub recovered_roles: Vec<L3RecoveredRole>,
    pub role_swap_trap: L3RoleSwapTrap,
    pub metrics: L3SchemaBindMetrics,
    pub claim_boundary: L3SchemaBindClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct L3SchemaBindingSchema {
    pub schema_id: u32,
    pub operator_id: u16,
    pub subject_role: u16,
    pub object_role: u16,
    pub route_id: u16,
    pub form: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct L3RecoveredRole {
    pub role: &'static str,
    pub expected: &'static str,
    pub recovered: &'static str,
    pub runner_up: &'static str,
    pub margin: i32,
    pub exact: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct L3RoleSwapTrap {
    pub trap: &'static str,
    pub wrong_claim: &'static str,
    pub recovered_subject: &'static str,
    pub recovered_object: &'static str,
    pub rejected: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct L3SchemaBindMetrics {
    pub schema_role_recall: f32,
    pub role_error_rate: f32,
    pub role_swap_reject_rate: f32,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct L3SchemaBindClaimBoundary {
    pub schema_binding_implemented: bool,
    pub uses_l3_schema_record: bool,
    pub real_corpus_trained: bool,
    pub nonlinear_memory_proven: bool,
    pub llm_ready: bool,
    pub safe_claim: &'static str,
}

#[derive(Clone, Copy)]
struct RoleFiller {
    role: &'static str,
    filler: &'static str,
}

#[derive(Clone, Copy)]
struct CleanupSymbol {
    label: &'static str,
    wave: Wave1024,
}

pub(crate) fn build_l3_schema_bind_report() -> L3SchemaBindReport {
    let schema_record = invoice_schema_record();
    let schema = L3SchemaBindingSchema {
        schema_id: schema_record.id,
        operator_id: schema_record.operator_id,
        subject_role: schema_record.subject_role,
        object_role: schema_record.object_role,
        route_id: schema_record.route_id,
        form: "supplier issues invoice",
    };
    let bindings = [
        RoleFiller {
            role: "subject:supplier",
            filler: "Honglu",
        },
        RoleFiller {
            role: "object:document",
            filler: "invoice",
        },
        RoleFiller {
            role: "route",
            filler: "business_docs",
        },
        RoleFiller {
            role: "operator",
            filler: "issues",
        },
    ];
    let cleanup = cleanup_symbols();
    let field = build_schema_field(&bindings);
    let recovered_roles = bindings
        .iter()
        .map(|binding| recover_role(&field, *binding, &cleanup))
        .collect::<Vec<_>>();
    let exact = recovered_roles.iter().filter(|role| role.exact).count();
    let role_error = recovered_roles.len().saturating_sub(exact);
    let role_swap_trap = role_swap_trap(&field, &cleanup);
    let role_swap_reject_rate = if role_swap_trap.rejected { 1.0 } else { 0.0 };
    let state = if role_error == 0 && role_swap_trap.rejected {
        "L3_SCHEMA_BIND_READY_NOT_LLM"
    } else {
        "L3_SCHEMA_BIND_REVIEW"
    };

    L3SchemaBindReport {
        mode: "llmwave-big-l3-schema-bind",
        version: L3_SCHEMA_BIND_VERSION,
        roadmap_block: "v431-v455",
        verdict: state,
        schema,
        recovered_roles,
        role_swap_trap,
        metrics: L3SchemaBindMetrics {
            schema_role_recall: ratio(exact, bindings.len()),
            role_error_rate: ratio(role_error, bindings.len()),
            role_swap_reject_rate,
            state,
        },
        claim_boundary: L3SchemaBindClaimBoundary {
            schema_binding_implemented: true,
            uses_l3_schema_record: true,
            real_corpus_trained: false,
            nonlinear_memory_proven: false,
            llm_ready: false,
            safe_claim:
                "L3 schema binding can recover fixture role fillers from a schema field and reject a role-swap trap",
        },
    }
}

fn invoice_schema_record() -> SchemaRecord {
    build_schema_atlas_report()
        .records
        .into_iter()
        .find(|record| record.id == 101)
        .expect("schema 101")
}

fn build_schema_field(bindings: &[RoleFiller]) -> Wave1024 {
    let mut field = Wave1024::zero();
    for binding in bindings {
        field.add_assign(&bind(
            &Wave1024::symbol(binding.role),
            &Wave1024::symbol(binding.filler),
        ));
    }
    field
}

fn recover_role(
    field: &Wave1024,
    binding: RoleFiller,
    cleanup: &[CleanupSymbol],
) -> L3RecoveredRole {
    let cue = unbind(field, &Wave1024::symbol(binding.role));
    let (recovered, runner_up, top_score, runner_up_score) = cleanup_nearest(&cue, cleanup);
    L3RecoveredRole {
        role: binding.role,
        expected: binding.filler,
        recovered,
        runner_up,
        margin: top_score - runner_up_score,
        exact: recovered == binding.filler,
    }
}

fn role_swap_trap(field: &Wave1024, cleanup: &[CleanupSymbol]) -> L3RoleSwapTrap {
    let subject = cleanup_nearest(
        &unbind(field, &Wave1024::symbol("subject:supplier")),
        cleanup,
    )
    .0;
    let object = cleanup_nearest(
        &unbind(field, &Wave1024::symbol("object:document")),
        cleanup,
    )
    .0;
    L3RoleSwapTrap {
        trap: "subject_object_swap",
        wrong_claim: "invoice issues Honglu",
        recovered_subject: subject,
        recovered_object: object,
        rejected: subject == "Honglu" && object == "invoice",
    }
}

fn cleanup_symbols() -> Vec<CleanupSymbol> {
    [
        "Honglu",
        "Rustrade",
        "invoice",
        "business_docs",
        "issues",
        "pays",
    ]
    .iter()
    .map(|label| CleanupSymbol {
        label,
        wave: Wave1024::symbol(label),
    })
    .collect()
}

fn bind(role: &Wave1024, filler: &Wave1024) -> Wave1024 {
    let mut values = [0_i16; WAVE_DIM];
    for ((value, role_value), filler_value) in values
        .iter_mut()
        .zip(role.values.iter())
        .zip(filler.values.iter())
    {
        *value = role_value.saturating_mul(*filler_value);
    }
    Wave1024 { values }
}

fn unbind(field: &Wave1024, role: &Wave1024) -> Wave1024 {
    let mut values = [0_i16; WAVE_DIM];
    for ((value, field_value), role_value) in values
        .iter_mut()
        .zip(field.values.iter())
        .zip(role.values.iter())
    {
        *value = field_value.saturating_mul(*role_value);
    }
    Wave1024 { values }
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
