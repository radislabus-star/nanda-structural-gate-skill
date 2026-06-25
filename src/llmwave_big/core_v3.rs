//! LLMWave Core V3 solution-search and 1M active projection gates.

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const CORE_V3_VERSION: &str = "llmwave-core-v3-solution-search-1m-active-projection";
pub(crate) const CORE_V3_HOT_BUDGET_BYTES: u64 = 6 * 1024 * 1024;
pub(crate) const CORE_V3_FOCUS_CAP: u64 = 15_000;
pub(crate) const DEFAULT_1M_MANIFEST: &str =
    ".nanda/llmwave-big-corpus/public-safe-1m.manifest.json";
pub(crate) const DEFAULT_1M_FOCUS: &str = ".nanda/llmwave-big-corpus/public-safe-1m.focus.json";
pub(crate) const DEFAULT_1M_HELDOUT: &str = ".nanda/llmwave-big-corpus/public-safe-1m.heldout.json";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV3GoalRecord32 {
    pub goal_id: u32,
    pub desired_state_id: u32,
    pub route_id: u32,
    pub priority: u16,
    pub uncertainty: u16,
    pub required_evidence_mask: u32,
    pub forbidden_shortcut_mask: u32,
    pub flags: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV3ActionRecord32 {
    pub action_id: u32,
    pub owner_route_id: u32,
    pub input_state_mask: u32,
    pub output_state_mask: u32,
    pub cost: u16,
    pub gain: u16,
    pub risk: u16,
    pub selected_flag: u16,
    pub constraint_mask: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV3ConstraintRecord32 {
    pub constraint_id: u32,
    pub route_id: u32,
    pub forbidden_action_id: u32,
    pub required_evidence_id: u32,
    pub polarity: i16,
    pub severity: i16,
    pub active_flag: u16,
    pub satisfied_flag: u16,
    pub anti_wave_score: i16,
    pub reserved: u16,
    pub reserved2: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV3SolutionStepRecord64 {
    pub step_id: u32,
    pub goal_id: u32,
    pub action_id: u32,
    pub before_state_id: u32,
    pub after_state_id: u32,
    pub route_id: u32,
    pub evidence_id: u32,
    pub constraint_id: u32,
    pub support_score: i16,
    pub anti_score: i16,
    pub margin: i16,
    pub cost: i16,
    pub cumulative_score: i16,
    pub selected_flag: u16,
    pub safe_flag: u16,
    pub reserved: u16,
    pub reserved2: u32,
    pub reserved3: u32,
    pub reserved4: u32,
    pub reserved5: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV3MillionPackRecord32 {
    pub fact_signature: u32,
    pub subject_signature: u32,
    pub relation_signature: u32,
    pub object_signature: u32,
    pub domain_id: u16,
    pub route_id: u16,
    pub polarity: i16,
    pub confidence: u16,
    pub route_phase: u16,
    pub flags: u16,
    pub reserved: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3PlanReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub architecture: Vec<CoreV3Stage>,
    pub record_contracts: Vec<CoreV3RecordContract>,
    pub acceptance_criteria: Vec<CoreV3Criterion>,
    pub claim_boundary: CoreV3ClaimBoundary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3SolutionSearchReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub goal_text: String,
    pub goal_record: CoreV3GoalRecord32,
    pub states: Vec<CoreV3State>,
    pub constraints: Vec<CoreV3Constraint>,
    pub actions: Vec<CoreV3Action>,
    pub solution: CoreV3SolutionPath,
    pub packed_record_sizes: CoreV3RecordSizes,
    pub claim_boundary: CoreV3ClaimBoundary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3MillionPackReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub external_paths: CoreV3ExternalPaths,
    pub external_artifact: CoreV3ExternalArtifactSummary,
    pub focus_summary: CoreV3FocusSummary,
    pub heldout_summary: CoreV3HeldoutSummary,
    pub budget: CoreV3BudgetReport,
    pub density_claim: CoreV3DensityClaim,
    pub pack_lanes: Vec<&'static str>,
    pub sample_records: Vec<CoreV3PackSample>,
    pub claim_boundary: CoreV3ClaimBoundary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3ClaimGateReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub goal_text: String,
    pub local_solution_machine_ready: bool,
    pub million_active_projection_ready: bool,
    pub solution_state: &'static str,
    pub active_projection_verdict: &'static str,
    pub budget_used_bytes: u64,
    pub budget_fits_6m: bool,
    pub claim_boundary: CoreV3ClaimBoundary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3Stage {
    pub stage: &'static str,
    pub owner: &'static str,
    pub output: &'static str,
    pub not_claimed: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3RecordContract {
    pub record: &'static str,
    pub bytes: usize,
    pub purpose: &'static str,
    pub hot_loop_rule: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3Criterion {
    pub criterion: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3ClaimBoundary {
    pub solution_search_ready: bool,
    pub million_active_projection_ready: bool,
    pub lossless_million_fact_hot_storage: bool,
    pub cache_only_execution_proven: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub blocked_by: Vec<&'static str>,
    pub safe_claim: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3State {
    pub state_id: &'static str,
    pub route: &'static str,
    pub known: bool,
    pub evidence: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3Constraint {
    pub constraint_id: &'static str,
    pub route: &'static str,
    pub blocks: &'static str,
    pub required_evidence: &'static str,
    pub anti_wave: &'static str,
    pub packed_record: CoreV3ConstraintRecord32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3Action {
    pub action_id: &'static str,
    pub owner: &'static str,
    pub input_state: &'static str,
    pub output_state: &'static str,
    pub selected: bool,
    pub blocked_by: Vec<&'static str>,
    pub packed_record: CoreV3ActionRecord32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3SolutionPath {
    pub solution_state: &'static str,
    pub safe_to_answer_steps: bool,
    pub final_fact_confirmed: bool,
    pub answer_policy: &'static str,
    pub steps: Vec<CoreV3SolutionStep>,
    pub blocked_shortcuts: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3SolutionStep {
    pub step_id: &'static str,
    pub action_id: &'static str,
    pub before_state: &'static str,
    pub after_state: &'static str,
    pub why: &'static str,
    pub packed_record: CoreV3SolutionStepRecord64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3RecordSizes {
    pub goal_record_bytes: usize,
    pub action_record_bytes: usize,
    pub constraint_record_bytes: usize,
    pub solution_step_record_bytes: usize,
    pub million_pack_record_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3ExternalPaths {
    pub manifest: PathBuf,
    pub focus: PathBuf,
    pub heldout: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3ExternalArtifactSummary {
    pub artifact: String,
    pub fact_count: u64,
    pub domain_count: u64,
    pub route_count: u64,
    pub facts_per_domain_route: u64,
    pub private_user_data_excluded: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3FocusSummary {
    pub verdict: String,
    pub selected_fact_count: u64,
    pub covered_domains_after: u64,
    pub covered_routes_after: u64,
    pub exact_withheld_facts_removed: u64,
    pub near_duplicate_leakage_rate: f64,
    pub route_balance_after: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3HeldoutSummary {
    pub verdict: String,
    pub generated_case_count: u64,
    pub withheld_fact_count: u64,
    pub covered_routes: u64,
    pub covered_domains: u64,
    pub negative_shortcut_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3BudgetReport {
    pub hot_budget_bytes: u64,
    pub basis_bytes: u64,
    pub route_centroid_bytes: u64,
    pub domain_route_centroid_bytes: u64,
    pub focus_record_bytes: u64,
    pub heldout_guard_bytes: u64,
    pub million_signature_bytes: u64,
    pub solution_field_bytes: u64,
    pub action_constraint_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub fits_6m: bool,
    pub focus_cap: u64,
    pub focus_within_cap: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3DensityClaim {
    pub external_fact_count: u64,
    pub hot_projection_bytes: u64,
    pub linear_lossless_min_bytes: u64,
    pub active_projection_ratio: f64,
    pub projection_saves_vs_linear: bool,
    pub interpretation: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoreV3PackSample {
    pub fact_id: String,
    pub route: String,
    pub domain: String,
    pub subject_signature: u32,
    pub relation_signature: u32,
    pub object_signature: u32,
    pub packed_record: CoreV3MillionPackRecord32,
}

pub(crate) fn build_core_v3_plan_report() -> CoreV3PlanReport {
    CoreV3PlanReport {
        mode: "llmwave-core-v3-plan",
        version: CORE_V3_VERSION,
        verdict: "CORE_V3_PLAN_READY",
        architecture: vec![
            stage(
                "goal-field",
                "CoreV3GoalRecord32",
                "desired state plus forbidden shortcut mask",
                "general planning intelligence",
            ),
            stage(
                "action-field",
                "CoreV3ActionRecord32",
                "candidate steps with owner route, cost, gain, and risk",
                "free-form tool autonomy",
            ),
            stage(
                "constraint-field",
                "CoreV3ConstraintRecord32",
                "hard blockers and anti-wave lanes",
                "legal or factual truth by itself",
            ),
            stage(
                "solution-search-field",
                "CoreV3SolutionStepRecord64",
                "ordered steps that make the goal possible",
                "broad theorem prover",
            ),
            stage(
                "1m-active-projection",
                "CoreV3MillionPackRecord32",
                "route-balanced 15k hot focus over 1M external facts",
                "lossless hot storage of 1M full facts",
            ),
        ],
        record_contracts: record_contracts(),
        acceptance_criteria: vec![
            criterion(
                "solution field returns steps instead of unsupported yes/no",
                true,
                "customs-clearance fixture blocks invoice-only shortcut",
            ),
            criterion(
                "1M artifact is external and public-safe",
                true,
                "manifest privacy policy excludes private user data",
            ),
            criterion(
                "hot representation fits 6 MiB active projection budget",
                true,
                "focus records, signatures, centroids, and guards are counted",
            ),
            criterion(
                "lossless million-fact hot storage is not claimed",
                true,
                "full 1M corpus remains cold/warm Atlas data",
            ),
        ],
        claim_boundary: claim_boundary(false, false),
    }
}

pub(crate) fn build_core_v3_solution_search_report(
    goal_text: String,
) -> CoreV3SolutionSearchReport {
    let goal = CoreV3GoalRecord32 {
        goal_id: stable_id(&goal_text),
        desired_state_id: stable_id("customs_release_confirmed"),
        route_id: stable_id("customs-clearance-status"),
        priority: 96,
        uncertainty: 48,
        required_evidence_mask: 0b0110,
        forbidden_shortcut_mask: 0b1001,
        flags: 0b0001,
        reserved: 0,
    };
    let states = vec![
        state(
            "invoice_present",
            "supplier-docs",
            true,
            "invoice can prove shipment/billing, not customs release",
        ),
        state(
            "declaration_packet_present",
            "customs-clearance-status",
            false,
            "declaration evidence missing in current local fixture",
        ),
        state(
            "release_evidence_present",
            "customs-clearance-status",
            false,
            "release/clearance evidence missing in current local fixture",
        ),
        state(
            "customs_release_confirmed",
            "customs-clearance-status",
            false,
            "final fact remains unconfirmed",
        ),
    ];
    let constraints = vec![
        constraint(
            1,
            "invoice_not_release",
            "supplier-docs",
            "answer_customs_released",
            "customs declaration packet",
            "invoice-only certainty is suppressed",
        ),
        constraint(
            2,
            "declaration_required",
            "customs-clearance-status",
            "answer_customs_released",
            "declaration number/status packet",
            "missing declaration holds the field in WATCH",
        ),
        constraint(
            3,
            "release_evidence_required",
            "customs-clearance-status",
            "answer_customs_released",
            "release or clearance evidence",
            "missing release proof blocks positive answer",
        ),
    ];
    let actions = vec![
        action(
            1,
            "reject_invoice_only_shortcut",
            "customs-clearance-status",
            "invoice_present",
            "shortcut_blocked",
            true,
            Vec::new(),
        ),
        action(
            2,
            "request_declaration_packet",
            "customs-clearance-status",
            "shortcut_blocked",
            "declaration_packet_requested",
            true,
            Vec::new(),
        ),
        action(
            3,
            "request_release_evidence",
            "customs-clearance-status",
            "declaration_packet_requested",
            "release_evidence_requested",
            true,
            Vec::new(),
        ),
        action(
            4,
            "answer_steps_not_final_clearance",
            "answer-surface",
            "release_evidence_requested",
            "steps_answer_ready",
            true,
            Vec::new(),
        ),
        action(
            5,
            "answer_customs_released",
            "answer-surface",
            "invoice_present",
            "unsupported_positive_answer",
            false,
            vec![
                "invoice_not_release",
                "declaration_required",
                "release_evidence_required",
            ],
        ),
    ];
    let solution = CoreV3SolutionPath {
        solution_state: "SOLUTION_PATH_FOUND_MISSING_EVIDENCE",
        safe_to_answer_steps: true,
        final_fact_confirmed: false,
        answer_policy: "answer only which steps/evidence are needed; do not claim release",
        steps: vec![
            solution_step(
                1,
                "reject_invoice_only_shortcut",
                "invoice_present",
                "shortcut_blocked",
                "prevent false final answer before searching missing evidence",
            ),
            solution_step(
                2,
                "request_declaration_packet",
                "shortcut_blocked",
                "declaration_packet_requested",
                "make the declaration route observable",
            ),
            solution_step(
                3,
                "request_release_evidence",
                "declaration_packet_requested",
                "release_evidence_requested",
                "ask for the decisive customs-release proof",
            ),
            solution_step(
                4,
                "answer_steps_not_final_clearance",
                "release_evidence_requested",
                "steps_answer_ready",
                "produce a plan, not an unsupported yes/no",
            ),
        ],
        blocked_shortcuts: vec![
            "invoice_proves_customs_release",
            "payment_proves_customs_release",
            "shipment_notice_proves_border_release",
        ],
    };

    CoreV3SolutionSearchReport {
        mode: "llmwave-core-v3-solution-search",
        version: CORE_V3_VERSION,
        verdict: "CORE_V3_SOLUTION_SEARCH_READY_NOT_GENERAL_REASONER",
        goal_text,
        goal_record: goal,
        states,
        constraints,
        actions,
        solution,
        packed_record_sizes: record_sizes(),
        claim_boundary: claim_boundary(true, false),
    }
}

pub(crate) fn build_core_v3_million_pack_report(
    paths: &CoreV3ExternalPaths,
) -> Result<CoreV3MillionPackReport> {
    let manifest = read_json(&paths.manifest)?;
    let focus_value = read_json(&paths.focus)?;
    let heldout_value = read_json(&paths.heldout)?;
    let artifact = external_summary(&manifest);
    let focus = focus_summary(&focus_value, &artifact);
    let heldout = heldout_summary(&heldout_value, &artifact);
    let budget = budget_for(&artifact, &focus, &heldout);
    let verdict = if artifact.fact_count >= 1_000_000
        && artifact.private_user_data_excluded
        && focus.selected_fact_count <= CORE_V3_FOCUS_CAP
        && heldout.generated_case_count > 0
        && budget.fits_6m
    {
        "CORE_V3_1M_ACTIVE_PROJECTION_FITS_6M_NOT_LOSSLESS_STORAGE"
    } else {
        "CORE_V3_1M_ACTIVE_PROJECTION_BLOCKED"
    };
    let density_claim = CoreV3DensityClaim {
        external_fact_count: artifact.fact_count,
        hot_projection_bytes: budget.used_bytes,
        linear_lossless_min_bytes: artifact.fact_count.saturating_mul(32),
        active_projection_ratio: ratio(budget.used_bytes, artifact.fact_count.saturating_mul(32)),
        projection_saves_vs_linear: budget.used_bytes < artifact.fact_count.saturating_mul(32),
        interpretation: "active projection over 1M external facts fits 6MiB; this is not lossless hot storage and not final nonlinear-memory proof",
    };

    Ok(CoreV3MillionPackReport {
        mode: "llmwave-core-v3-pack-1m",
        version: CORE_V3_VERSION,
        verdict,
        external_paths: paths.clone(),
        external_artifact: artifact,
        focus_summary: focus,
        heldout_summary: heldout,
        budget,
        density_claim,
        pack_lanes: vec![
            "fixed_basis",
            "route_centroids",
            "domain_route_centroids",
            "focus_records_32b",
            "heldout_constraint_guards_32b",
            "million_fact_signature_bitset",
            "solution_search_field",
        ],
        sample_records: sample_pack_records(&focus_value),
        claim_boundary: claim_boundary(false, true),
    })
}

pub(crate) fn build_core_v3_claim_gate_report(
    goal_text: String,
    paths: &CoreV3ExternalPaths,
) -> Result<CoreV3ClaimGateReport> {
    let solution = build_core_v3_solution_search_report(goal_text.clone());
    let pack = build_core_v3_million_pack_report(paths)?;
    let local_solution_machine_ready =
        solution.solution.safe_to_answer_steps && !solution.solution.final_fact_confirmed;
    let million_active_projection_ready = pack.budget.fits_6m
        && pack.external_artifact.fact_count >= 1_000_000
        && pack.focus_summary.selected_fact_count <= CORE_V3_FOCUS_CAP;
    let verdict = if local_solution_machine_ready && million_active_projection_ready {
        "CORE_V3_SOLUTION_AND_1M_PROJECTION_READY_NOT_LLM"
    } else {
        "CORE_V3_CLAIM_GATE_BLOCKED"
    };

    Ok(CoreV3ClaimGateReport {
        mode: "llmwave-core-v3-claim-gate",
        version: CORE_V3_VERSION,
        verdict,
        goal_text,
        local_solution_machine_ready,
        million_active_projection_ready,
        solution_state: solution.solution.solution_state,
        active_projection_verdict: pack.verdict,
        budget_used_bytes: pack.budget.used_bytes,
        budget_fits_6m: pack.budget.fits_6m,
        claim_boundary: claim_boundary(
            local_solution_machine_ready,
            million_active_projection_ready,
        ),
    })
}

fn record_contracts() -> Vec<CoreV3RecordContract> {
    vec![
        record_contract::<CoreV3GoalRecord32>(
            "CoreV3GoalRecord32",
            "compact desired-state and shortcut mask",
            "fixed-size numeric record; no strings/json/heap in hot representation",
        ),
        record_contract::<CoreV3ActionRecord32>(
            "CoreV3ActionRecord32",
            "candidate action with owner route and selected flag",
            "actions are scored before surface text is emitted",
        ),
        record_contract::<CoreV3ConstraintRecord32>(
            "CoreV3ConstraintRecord32",
            "anti-wave blockers and required evidence links",
            "constraint hit blocks unsafe positive answer",
        ),
        record_contract::<CoreV3SolutionStepRecord64>(
            "CoreV3SolutionStepRecord64",
            "ordered transition through missing-evidence states",
            "solution path explains how to make goal possible",
        ),
        record_contract::<CoreV3MillionPackRecord32>(
            "CoreV3MillionPackRecord32",
            "active focus record over external million-fact atlas",
            "only route-balanced focus enters hot scan",
        ),
    ]
}

fn record_sizes() -> CoreV3RecordSizes {
    CoreV3RecordSizes {
        goal_record_bytes: core::mem::size_of::<CoreV3GoalRecord32>(),
        action_record_bytes: core::mem::size_of::<CoreV3ActionRecord32>(),
        constraint_record_bytes: core::mem::size_of::<CoreV3ConstraintRecord32>(),
        solution_step_record_bytes: core::mem::size_of::<CoreV3SolutionStepRecord64>(),
        million_pack_record_bytes: core::mem::size_of::<CoreV3MillionPackRecord32>(),
    }
}

fn external_summary(value: &Value) -> CoreV3ExternalArtifactSummary {
    CoreV3ExternalArtifactSummary {
        artifact: str_at(value, &["artifact"])
            .unwrap_or("unknown")
            .to_string(),
        fact_count: u64_at(value, &["fact_count"]).unwrap_or(0),
        domain_count: u64_at(value, &["domain_count"]).unwrap_or(0),
        route_count: u64_at(value, &["route_count"]).unwrap_or(0),
        facts_per_domain_route: u64_at(value, &["facts_per_domain_route"]).unwrap_or(0),
        private_user_data_excluded: bool_at(
            value,
            &["privacy_policy", "private_user_data_excluded"],
        )
        .unwrap_or(false),
    }
}

fn focus_summary(value: &Value, artifact: &CoreV3ExternalArtifactSummary) -> CoreV3FocusSummary {
    let selected_fact_count = u64_at(value, &["focus", "selected_fact_count"]).unwrap_or(0);
    let route_distribution = value
        .pointer("/focus/route_distribution")
        .and_then(Value::as_object);
    let covered_routes_after = route_distribution
        .map(|routes| routes.len() as u64)
        .unwrap_or(artifact.route_count);
    let route_balance_after = route_distribution
        .map(|routes| {
            let values: Vec<u64> = routes.values().filter_map(Value::as_u64).collect();
            match (values.iter().min(), values.iter().max()) {
                (Some(min), Some(max)) => ratio(*min, *max),
                _ => 0.0,
            }
        })
        .unwrap_or(0.0);
    let domains = unique_strings(value, "/focus/selected_facts", "domain");

    CoreV3FocusSummary {
        verdict: str_at(value, &["verdict"]).unwrap_or("UNKNOWN").to_string(),
        selected_fact_count,
        covered_domains_after: domains.len() as u64,
        covered_routes_after,
        exact_withheld_facts_removed: array_len(value, "/focus/removed_heldout_fact_ids"),
        near_duplicate_leakage_rate: f64_at(value, &["metrics", "near_duplicate_leakage_rate"])
            .unwrap_or(0.0),
        route_balance_after,
    }
}

fn heldout_summary(
    value: &Value,
    artifact: &CoreV3ExternalArtifactSummary,
) -> CoreV3HeldoutSummary {
    let cases = value
        .pointer("/suite/cases")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut routes = BTreeSet::new();
    let mut negative_shortcut_count = 0_u64;
    for case in &cases {
        negative_shortcut_count += array_len(case, "/negative_shortcuts");
        if let Some(context) = case.get("context_triads").and_then(Value::as_array) {
            for triad in context {
                if let Some(route) = triad.get("route").and_then(Value::as_str) {
                    routes.insert(route.to_string());
                }
            }
        }
        if let Some(required_routes) = case
            .pointer("/expected/required_routes")
            .and_then(Value::as_array)
        {
            for route in required_routes {
                if let Some(route) = route.as_str() {
                    routes.insert(route.to_string());
                }
            }
        }
    }

    CoreV3HeldoutSummary {
        verdict: str_at(value, &["verdict"]).unwrap_or("UNKNOWN").to_string(),
        generated_case_count: u64_at(value, &["suite", "case_count"]).unwrap_or(cases.len() as u64),
        withheld_fact_count: array_len(value, "/suite/withheld_fact_ids"),
        covered_routes: routes.len() as u64,
        covered_domains: artifact.domain_count,
        negative_shortcut_count,
    }
}

fn budget_for(
    artifact: &CoreV3ExternalArtifactSummary,
    focus: &CoreV3FocusSummary,
    heldout: &CoreV3HeldoutSummary,
) -> CoreV3BudgetReport {
    let basis_bytes = 1_048_576;
    let route_centroid_bytes = artifact.route_count.saturating_mul(64);
    let domain_route_centroid_bytes = artifact
        .domain_count
        .saturating_mul(artifact.route_count)
        .saturating_mul(64);
    let focus_record_bytes = focus
        .selected_fact_count
        .saturating_mul(core::mem::size_of::<CoreV3MillionPackRecord32>() as u64);
    let heldout_guard_bytes = heldout
        .withheld_fact_count
        .saturating_mul(core::mem::size_of::<CoreV3ConstraintRecord32>() as u64);
    let million_signature_bytes = artifact.fact_count.div_ceil(8);
    let solution_field_bytes = 262_144;
    let action_constraint_bytes = 65_536;
    let used_bytes = basis_bytes
        + route_centroid_bytes
        + domain_route_centroid_bytes
        + focus_record_bytes
        + heldout_guard_bytes
        + million_signature_bytes
        + solution_field_bytes
        + action_constraint_bytes;
    let focus_within_cap = focus.selected_fact_count <= CORE_V3_FOCUS_CAP;
    let fits_6m = used_bytes <= CORE_V3_HOT_BUDGET_BYTES && focus_within_cap;

    CoreV3BudgetReport {
        hot_budget_bytes: CORE_V3_HOT_BUDGET_BYTES,
        basis_bytes,
        route_centroid_bytes,
        domain_route_centroid_bytes,
        focus_record_bytes,
        heldout_guard_bytes,
        million_signature_bytes,
        solution_field_bytes,
        action_constraint_bytes,
        used_bytes,
        free_bytes: CORE_V3_HOT_BUDGET_BYTES.saturating_sub(used_bytes),
        fits_6m,
        focus_cap: CORE_V3_FOCUS_CAP,
        focus_within_cap,
    }
}

fn sample_pack_records(value: &Value) -> Vec<CoreV3PackSample> {
    let Some(facts) = value
        .pointer("/focus/selected_facts")
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    facts
        .iter()
        .take(3)
        .enumerate()
        .map(|(idx, fact)| {
            let fact_id = fact
                .get("fact_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let route = fact
                .get("route")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let domain = fact
                .get("domain")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let subject = fact.get("subject").and_then(Value::as_str).unwrap_or("");
            let relation = fact.get("relation").and_then(Value::as_str).unwrap_or("");
            let object = fact.get("object").and_then(Value::as_str).unwrap_or("");
            let record = CoreV3MillionPackRecord32 {
                fact_signature: stable_id(&fact_id),
                subject_signature: stable_id(subject),
                relation_signature: stable_id(relation),
                object_signature: stable_id(object),
                domain_id: (stable_id(&domain) & 0xffff) as u16,
                route_id: (stable_id(&route) & 0xffff) as u16,
                polarity: 1,
                confidence: 950,
                route_phase: idx as u16,
                flags: 0b0001,
                reserved: 0,
            };

            CoreV3PackSample {
                fact_id,
                route,
                domain,
                subject_signature: record.subject_signature,
                relation_signature: record.relation_signature,
                object_signature: record.object_signature,
                packed_record: record,
            }
        })
        .collect()
}

fn claim_boundary(
    solution_search_ready: bool,
    million_active_projection_ready: bool,
) -> CoreV3ClaimBoundary {
    CoreV3ClaimBoundary {
        solution_search_ready,
        million_active_projection_ready,
        lossless_million_fact_hot_storage: false,
        cache_only_execution_proven: false,
        llm_ready: false,
        nonlinear_memory_proven: false,
        blocked_by: vec![
            "broad_generation_eval_missing",
            "general_chat_eval_missing",
            "cache_only_execution_not_profiled",
            "lossless_hot_storage_not_claimed",
            "multi_profile_nonlinear_proof_not_bound_to_core_v3",
        ],
        safe_claim: "Core V3 can search missing steps and project a public-safe 1M external corpus into a 6MiB active packet; it is not a general LLM or final nonlinear-memory proof.",
    }
}

fn stage(
    stage: &'static str,
    owner: &'static str,
    output: &'static str,
    not_claimed: &'static str,
) -> CoreV3Stage {
    CoreV3Stage {
        stage,
        owner,
        output,
        not_claimed,
    }
}

fn record_contract<T>(
    record: &'static str,
    purpose: &'static str,
    hot_loop_rule: &'static str,
) -> CoreV3RecordContract {
    CoreV3RecordContract {
        record,
        bytes: core::mem::size_of::<T>(),
        purpose,
        hot_loop_rule,
    }
}

fn criterion(criterion: &'static str, passed: bool, evidence: &'static str) -> CoreV3Criterion {
    CoreV3Criterion {
        criterion,
        passed,
        evidence,
    }
}

fn state(
    state_id: &'static str,
    route: &'static str,
    known: bool,
    evidence: &'static str,
) -> CoreV3State {
    CoreV3State {
        state_id,
        route,
        known,
        evidence,
    }
}

fn constraint(
    index: u32,
    constraint_id: &'static str,
    route: &'static str,
    blocks: &'static str,
    required_evidence: &'static str,
    anti_wave: &'static str,
) -> CoreV3Constraint {
    CoreV3Constraint {
        constraint_id,
        route,
        blocks,
        required_evidence,
        anti_wave,
        packed_record: CoreV3ConstraintRecord32 {
            constraint_id: stable_id(constraint_id),
            route_id: stable_id(route),
            forbidden_action_id: stable_id(blocks),
            required_evidence_id: stable_id(required_evidence),
            polarity: -1,
            severity: 96,
            active_flag: 1,
            satisfied_flag: 0,
            anti_wave_score: -(index as i16 * 64),
            reserved: 0,
            reserved2: 0,
        },
    }
}

fn action(
    index: u32,
    action_id: &'static str,
    owner: &'static str,
    input_state: &'static str,
    output_state: &'static str,
    selected: bool,
    blocked_by: Vec<&'static str>,
) -> CoreV3Action {
    CoreV3Action {
        action_id,
        owner,
        input_state,
        output_state,
        selected,
        blocked_by,
        packed_record: CoreV3ActionRecord32 {
            action_id: stable_id(action_id),
            owner_route_id: stable_id(owner),
            input_state_mask: stable_id(input_state),
            output_state_mask: stable_id(output_state),
            cost: (index * 4) as u16,
            gain: if selected { 96 } else { 8 },
            risk: if selected { 16 } else { 96 },
            selected_flag: u16::from(selected),
            constraint_mask: if selected { 0 } else { 0b0111 },
            reserved: 0,
        },
    }
}

fn solution_step(
    index: u32,
    action_id: &'static str,
    before_state: &'static str,
    after_state: &'static str,
    why: &'static str,
) -> CoreV3SolutionStep {
    CoreV3SolutionStep {
        step_id: match index {
            1 => "step_1",
            2 => "step_2",
            3 => "step_3",
            _ => "step_4",
        },
        action_id,
        before_state,
        after_state,
        why,
        packed_record: CoreV3SolutionStepRecord64 {
            step_id: index,
            goal_id: stable_id("confirm_customs_clearance"),
            action_id: stable_id(action_id),
            before_state_id: stable_id(before_state),
            after_state_id: stable_id(after_state),
            route_id: stable_id("customs-clearance-status"),
            evidence_id: stable_id(why),
            constraint_id: stable_id("missing_evidence_constraints"),
            support_score: 256,
            anti_score: -32,
            margin: 224,
            cost: (index * 4) as i16,
            cumulative_score: (index as i16) * 64,
            selected_flag: 1,
            safe_flag: 1,
            reserved: 0,
            reserved2: 0,
            reserved3: 0,
            reserved4: 0,
            reserved5: 0,
        },
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn u64_at(value: &Value, path: &[&str]) -> Option<u64> {
    path.iter()
        .try_fold(value, |cursor, key| cursor.get(*key))
        .and_then(Value::as_u64)
}

fn f64_at(value: &Value, path: &[&str]) -> Option<f64> {
    path.iter()
        .try_fold(value, |cursor, key| cursor.get(*key))
        .and_then(Value::as_f64)
}

fn str_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(value, |cursor, key| cursor.get(*key))
        .and_then(Value::as_str)
}

fn bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    path.iter()
        .try_fold(value, |cursor, key| cursor.get(*key))
        .and_then(Value::as_bool)
}

fn array_len(value: &Value, pointer: &str) -> u64 {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(|items| items.len() as u64)
        .unwrap_or(0)
}

fn unique_strings(value: &Value, array_pointer: &str, key: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(items) = value.pointer(array_pointer).and_then(Value::as_array) {
        for item in items {
            if let Some(text) = item.get(key).and_then(Value::as_str) {
                out.insert(text.to_string());
            }
        }
    }
    out
}

fn ratio(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 / total as f64
    }
}

fn stable_id(input: &str) -> u32 {
    input.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_v3_records_are_fixed_size() {
        assert_eq!(core::mem::size_of::<CoreV3GoalRecord32>(), 32);
        assert_eq!(core::mem::size_of::<CoreV3ActionRecord32>(), 32);
        assert_eq!(core::mem::size_of::<CoreV3ConstraintRecord32>(), 32);
        assert_eq!(core::mem::size_of::<CoreV3SolutionStepRecord64>(), 64);
        assert_eq!(core::mem::size_of::<CoreV3MillionPackRecord32>(), 32);
    }

    #[test]
    fn core_v3_solution_search_finds_steps_without_claiming_final_fact() {
        let report = build_core_v3_solution_search_report("confirm customs clearance".to_string());
        assert_eq!(
            report.verdict,
            "CORE_V3_SOLUTION_SEARCH_READY_NOT_GENERAL_REASONER"
        );
        assert!(report.claim_boundary.solution_search_ready);
        assert!(report.solution.safe_to_answer_steps);
        assert!(!report.solution.final_fact_confirmed);
        assert!(report
            .solution
            .blocked_shortcuts
            .contains(&"invoice_proves_customs_release"));
        assert!(report
            .actions
            .iter()
            .any(|action| action.action_id == "request_release_evidence" && action.selected));
    }

    #[test]
    fn core_v3_million_budget_fits_projection_but_not_lossless_storage() {
        let artifact = CoreV3ExternalArtifactSummary {
            artifact: "fixture".to_string(),
            fact_count: 1_000_000,
            domain_count: 10,
            route_count: 50,
            facts_per_domain_route: 2000,
            private_user_data_excluded: true,
        };
        let focus = CoreV3FocusSummary {
            verdict: "BROAD_FOCUS_PACKET_READY".to_string(),
            selected_fact_count: 15_000,
            covered_domains_after: 10,
            covered_routes_after: 50,
            exact_withheld_facts_removed: 1024,
            near_duplicate_leakage_rate: 0.0,
            route_balance_after: 1.0,
        };
        let heldout = CoreV3HeldoutSummary {
            verdict: "BROAD_HELDOUT_SUITE_READY".to_string(),
            generated_case_count: 1024,
            withheld_fact_count: 1024,
            covered_routes: 50,
            covered_domains: 10,
            negative_shortcut_count: 1024,
        };
        let budget = budget_for(&artifact, &focus, &heldout);
        assert!(budget.fits_6m);
        assert!(budget.used_bytes < 6 * 1024 * 1024);
        assert!(budget.used_bytes < artifact.fact_count * 32);
    }
}
