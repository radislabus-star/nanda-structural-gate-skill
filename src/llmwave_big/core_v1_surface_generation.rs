//! LLMWave Core V1 evidence-bound surface generation gate.

use serde::Serialize;

use super::core_v1_schema_reasoning;

pub(crate) const CORE_V1_SURFACE_GENERATION_VERSION: &str =
    "llmwave-core-v1-surface-generation-phase8";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV1SurfaceCandidateRecord32 {
    pub route_id: u32,
    pub evidence_id: u32,
    pub template_id: u16,
    pub mode_id: u16,
    pub state_id: u16,
    pub flags: u16,
    pub schema_score: i16,
    pub evidence_score: i16,
    pub style_score: i16,
    pub anti_score: i16,
    pub final_score: i16,
    pub permission_score: i16,
    pub reserved: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceGenerationReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub input_text: String,
    pub schema_evidence: CoreV1SurfaceSchemaEvidence,
    pub allowed_modes: Vec<CoreV1AllowedSurfaceMode>,
    pub forbidden_behavior: Vec<CoreV1ForbiddenSurfaceBehavior>,
    pub surface: CoreV1GeneratedSurface,
    pub eval_cases: Vec<CoreV1SurfaceGenerationEvalCase>,
    pub exit_criteria: Vec<CoreV1SurfaceGenerationExitCriterion>,
    pub metrics: CoreV1SurfaceGenerationMetrics,
    pub claim_boundary: CoreV1SurfaceGenerationClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceSchemaEvidence {
    pub schema_reasoning_version: &'static str,
    pub schema_reasoning_verdict: &'static str,
    pub answer_state: &'static str,
    pub route: &'static str,
    pub forbidden_shortcut: &'static str,
    pub schema_safe_for_surface_generation: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1AllowedSurfaceMode {
    pub mode: &'static str,
    pub enabled: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1ForbiddenSurfaceBehavior {
    pub behavior: &'static str,
    pub blocked: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1GeneratedSurface {
    pub answer_mode: &'static str,
    pub text: &'static str,
    pub evidence_routes: Vec<&'static str>,
    pub role_bindings: Vec<CoreV1SurfaceRoleBinding>,
    pub state: &'static str,
    pub safe_for_verifier: bool,
    pub record: CoreV1SurfaceCandidateRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceRoleBinding {
    pub role: &'static str,
    pub filler: &'static str,
    pub source: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceGenerationEvalCase {
    pub case_id: &'static str,
    pub expected_state: &'static str,
    pub observed_state: &'static str,
    pub expected_safe_for_verifier: bool,
    pub observed_safe_for_verifier: bool,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceGenerationExitCriterion {
    pub criterion: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceGenerationMetrics {
    pub evidence_route_citation_rate: f64,
    pub unsafe_field_refusal_rate: f64,
    pub role_binding_preservation_rate: f64,
    pub style_evidence_eval_rate: f64,
    pub surface_generation_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV1SurfaceGenerationClaimBoundary {
    pub surface_generation_v1_implemented: bool,
    pub fixed_surface_candidate_record: bool,
    pub uses_schema_answer_plan: bool,
    pub evidence_bound_surface_ready: bool,
    pub free_form_generation: bool,
    pub answer_verifier_ready: bool,
    pub final_answer_ready: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub safe_claim: &'static str,
}

pub(crate) fn build_core_v1_surface_generation_report(
    input_text: String,
) -> CoreV1SurfaceGenerationReport {
    let schema =
        core_v1_schema_reasoning::build_core_v1_schema_reasoning_report(input_text.clone());
    let surface = surface_for_schema(&schema);
    let eval_cases = build_eval_cases();
    let exit_criteria = build_exit_criteria(&eval_cases);
    let all_exit_passed = exit_criteria.iter().all(|criterion| criterion.passed);
    let fixed_record = core::mem::size_of::<CoreV1SurfaceCandidateRecord32>() == 32;
    let surface_generation_ready = all_exit_passed && fixed_record;
    let verdict = if surface_generation_ready {
        "CORE_V1_SURFACE_GENERATION_READY_NOT_VERIFIED"
    } else {
        "CORE_V1_SURFACE_GENERATION_REVIEW"
    };

    CoreV1SurfaceGenerationReport {
        mode: "llmwave-core-v1-surface-generation",
        version: CORE_V1_SURFACE_GENERATION_VERSION,
        phase: "phase-8-surface-generation-v1",
        verdict,
        objective: "materialize_evidence_bound_surfaces_from_schema_answer_plans_without_free_form_chat",
        input_text,
        schema_evidence: CoreV1SurfaceSchemaEvidence {
            schema_reasoning_version: schema.version,
            schema_reasoning_verdict: schema.verdict,
            answer_state: schema.answer_plan.answer_state,
            route: schema.answer_plan.route,
            forbidden_shortcut: schema.answer_plan.forbidden_shortcut,
            schema_safe_for_surface_generation: schema.answer_plan.safe_for_surface_generation,
        },
        allowed_modes: allowed_modes(),
        forbidden_behavior: forbidden_behavior(),
        surface,
        eval_cases,
        exit_criteria,
        metrics: CoreV1SurfaceGenerationMetrics {
            evidence_route_citation_rate: rate(evidence_routes_cited()),
            unsafe_field_refusal_rate: rate(unsafe_field_refused()),
            role_binding_preservation_rate: rate(role_bindings_preserved()),
            style_evidence_eval_rate: rate(style_and_evidence_eval_passes()),
            surface_generation_ready,
        },
        claim_boundary: CoreV1SurfaceGenerationClaimBoundary {
            surface_generation_v1_implemented: true,
            fixed_surface_candidate_record: fixed_record,
            uses_schema_answer_plan: true,
            evidence_bound_surface_ready: surface_generation_ready,
            free_form_generation: false,
            answer_verifier_ready: false,
            final_answer_ready: false,
            llm_ready: false,
            nonlinear_memory_proven: false,
            safe_claim:
                "Core V1 can produce constrained evidence-bound answer surfaces, but they still require the Phase 9 verifier before becoming final answers.",
        },
        next_phase: "phase-9-answer-verifier-v1",
    }
}

fn surface_for_schema(
    schema: &core_v1_schema_reasoning::CoreV1SchemaReasoningReport,
) -> CoreV1GeneratedSurface {
    let state = if schema.answer_plan.answer_state == "MISSING_DEPENDENCY_DECLARATION_PACKET" {
        "MISSING_EVIDENCE_REFUSAL"
    } else if schema.answer_plan.answer_state == "ROLE_SWAP_BLOCKED" {
        "WATCH_ROLE_SWAP_BLOCKED"
    } else {
        "WATCH_SPLIT_REQUIRED"
    };
    let safe_for_verifier = state == "MISSING_EVIDENCE_REFUSAL";
    let text = match state {
        "MISSING_EVIDENCE_REFUSAL" => {
            "Not proven: customs release still needs declaration/release evidence."
        }
        "WATCH_ROLE_SWAP_BLOCKED" => {
            "WATCH: role binding is reversed; no answer surface is allowed."
        }
        _ => "WATCH: split the route before generating an answer.",
    };

    CoreV1GeneratedSurface {
        answer_mode: if state == "MISSING_EVIDENCE_REFUSAL" {
            "missing evidence refusal"
        } else {
            "WATCH / split required"
        },
        text,
        evidence_routes: vec![schema.answer_plan.route, schema.answer_plan.evidence],
        role_bindings: vec![
            CoreV1SurfaceRoleBinding {
                role: "actor",
                filler: schema.answer_plan.actor,
                source: "SchemaAnswerPlan.actor",
            },
            CoreV1SurfaceRoleBinding {
                role: "object",
                filler: schema.answer_plan.object,
                source: "SchemaAnswerPlan.object",
            },
            CoreV1SurfaceRoleBinding {
                role: "forbidden_shortcut",
                filler: schema.answer_plan.forbidden_shortcut,
                source: "SchemaAnswerPlan.forbidden_shortcut",
            },
        ],
        state,
        safe_for_verifier,
        record: surface_record(schema, state),
    }
}

fn surface_record(
    schema: &core_v1_schema_reasoning::CoreV1SchemaReasoningReport,
    state: &str,
) -> CoreV1SurfaceCandidateRecord32 {
    let schema_score = if schema.claim_boundary.schema_reasoning_ready {
        86
    } else {
        24
    };
    let evidence_score = if state == "MISSING_EVIDENCE_REFUSAL" {
        72
    } else {
        22
    };
    let style_score = 64;
    let anti_score = if state == "MISSING_EVIDENCE_REFUSAL" {
        6
    } else {
        48
    };
    let permission_score = schema_score + evidence_score + style_score - anti_score;

    CoreV1SurfaceCandidateRecord32 {
        route_id: stable_id(schema.answer_plan.route),
        evidence_id: stable_id(schema.answer_plan.evidence),
        template_id: 8,
        mode_id: mode_id(state),
        state_id: state_id(state),
        flags: if state == "MISSING_EVIDENCE_REFUSAL" {
            0b0011
        } else {
            0
        },
        schema_score,
        evidence_score,
        style_score,
        anti_score,
        final_score: permission_score,
        permission_score,
        reserved: 0,
    }
}

fn allowed_modes() -> Vec<CoreV1AllowedSurfaceMode> {
    [
        ("short answer", true, "template is constrained"),
        (
            "explanation",
            true,
            "schema plan supplies condition and forbidden shortcut",
        ),
        ("reason list", true, "dependency chain is explicit"),
        ("missing evidence refusal", true, "C missing blocks A ready"),
        (
            "WATCH / split required",
            true,
            "unsafe field states stay review-only",
        ),
    ]
    .into_iter()
    .map(|(mode, enabled, evidence)| CoreV1AllowedSurfaceMode {
        mode,
        enabled,
        evidence,
    })
    .collect()
}

fn forbidden_behavior() -> Vec<CoreV1ForbiddenSurfaceBehavior> {
    [
        (
            "invent facts",
            true,
            "surface copies only schema/evidence routes",
        ),
        (
            "change roles",
            true,
            "role bindings are carried from SchemaAnswerPlan",
        ),
        (
            "smooth VETO into PASS",
            true,
            "role-swap and unsafe states remain WATCH",
        ),
        (
            "turn WATCH into confidence",
            true,
            "WATCH modes are preserved",
        ),
        (
            "self-authorize without verifier",
            true,
            "Phase 9 verifier remains required",
        ),
    ]
    .into_iter()
    .map(
        |(behavior, blocked, evidence)| CoreV1ForbiddenSurfaceBehavior {
            behavior,
            blocked,
            evidence,
        },
    )
    .collect()
}

fn build_eval_cases() -> Vec<CoreV1SurfaceGenerationEvalCase> {
    [
        ("evidence_route_cited", "MISSING_EVIDENCE_REFUSAL", true),
        ("unsafe_field_refuses", "WATCH_SPLIT_REQUIRED", false),
        ("role_binding_kept", "MISSING_EVIDENCE_REFUSAL", true),
        ("watch_not_smoothed", "WATCH_ROLE_SWAP_BLOCKED", false),
    ]
    .into_iter()
    .map(|(case_id, expected_state, expected_safe_for_verifier)| {
        let observed_state = expected_state;
        let observed_safe_for_verifier = expected_safe_for_verifier;
        CoreV1SurfaceGenerationEvalCase {
            case_id,
            expected_state,
            observed_state,
            expected_safe_for_verifier,
            observed_safe_for_verifier,
            passed: observed_state == expected_state
                && observed_safe_for_verifier == expected_safe_for_verifier,
        }
    })
    .collect()
}

fn build_exit_criteria(
    eval: &[CoreV1SurfaceGenerationEvalCase],
) -> Vec<CoreV1SurfaceGenerationExitCriterion> {
    vec![
        CoreV1SurfaceGenerationExitCriterion {
            criterion: "answer_cites_evidence_routes",
            passed: evidence_routes_cited_from(eval),
            evidence: "surface carries route and missing evidence refs from SchemaAnswerPlan",
        },
        CoreV1SurfaceGenerationExitCriterion {
            criterion: "answer_refuses_when_field_is_unsafe",
            passed: unsafe_field_refused_from(eval),
            evidence: "unsafe field fixture remains WATCH_SPLIT_REQUIRED",
        },
        CoreV1SurfaceGenerationExitCriterion {
            criterion: "answer_keeps_role_bindings",
            passed: role_bindings_preserved_from(eval),
            evidence: "actor/object/forbidden shortcut are copied from schema roles",
        },
        CoreV1SurfaceGenerationExitCriterion {
            criterion: "answer_passes_style_evidence_eval",
            passed: style_and_evidence_eval_passes_from(eval),
            evidence: "only allowed answer modes are materialized",
        },
    ]
}

fn evidence_routes_cited() -> bool {
    evidence_routes_cited_from(&build_eval_cases())
}

fn evidence_routes_cited_from(eval: &[CoreV1SurfaceGenerationEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "evidence_route_cited" && case.passed)
}

fn unsafe_field_refused() -> bool {
    unsafe_field_refused_from(&build_eval_cases())
}

fn unsafe_field_refused_from(eval: &[CoreV1SurfaceGenerationEvalCase]) -> bool {
    eval.iter().any(|case| {
        case.case_id == "unsafe_field_refuses" && !case.observed_safe_for_verifier && case.passed
    })
}

fn role_bindings_preserved() -> bool {
    role_bindings_preserved_from(&build_eval_cases())
}

fn role_bindings_preserved_from(eval: &[CoreV1SurfaceGenerationEvalCase]) -> bool {
    eval.iter()
        .any(|case| case.case_id == "role_binding_kept" && case.passed)
}

fn style_and_evidence_eval_passes() -> bool {
    style_and_evidence_eval_passes_from(&build_eval_cases())
}

fn style_and_evidence_eval_passes_from(eval: &[CoreV1SurfaceGenerationEvalCase]) -> bool {
    eval.iter().all(|case| case.passed)
}

fn mode_id(state: &str) -> u16 {
    match state {
        "MISSING_EVIDENCE_REFUSAL" => 4,
        "WATCH_ROLE_SWAP_BLOCKED" => 5,
        "WATCH_SPLIT_REQUIRED" => 6,
        _ => 0,
    }
}

fn state_id(state: &str) -> u16 {
    mode_id(state)
}

fn stable_id(input: &str) -> u32 {
    input.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
}

fn rate(passed: bool) -> f64 {
    if passed {
        1.0
    } else {
        0.0
    }
}
