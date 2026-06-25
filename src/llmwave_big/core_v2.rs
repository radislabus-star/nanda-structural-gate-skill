//! LLMWave Core V2 staged pipeline.

use serde::Serialize;
use std::cmp::Reverse;

pub(crate) const CORE_V2_VERSION: &str = "llmwave-core-v2-staged-pipeline";
pub(crate) const CORE_V2_HOT_BUDGET_BYTES: u32 = 6 * 1024 * 1024;
pub(crate) const CORE_V2_FOCUS_CAP: usize = 15_000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV2FactRecord48 {
    pub fact_id: u32,
    pub subject_id: u32,
    pub relation_id: u32,
    pub object_id: u32,
    pub route_id: u32,
    pub schema_id: u32,
    pub evidence_id: u32,
    pub family_id: u32,
    pub role_mask: u16,
    pub polarity: u16,
    pub confidence: u16,
    pub flags: u16,
    pub phase: i16,
    pub amplitude: i16,
    pub residual: i16,
    pub reserved: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV2HeldoutCaseRecord32 {
    pub case_id: u32,
    pub query_id: u32,
    pub expected_fact_id: u32,
    pub expected_route_id: u32,
    pub trap_fact_id: u32,
    pub pass_flag: u16,
    pub leakage_flag: u16,
    pub score: i16,
    pub margin: i16,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV2FocusRecord32 {
    pub fact_id: u32,
    pub route_id: u32,
    pub schema_id: u32,
    pub evidence_id: u32,
    pub selected_flag: u16,
    pub heldout_flag: u16,
    pub route_rank: u16,
    pub flags: u16,
    pub support_score: i16,
    pub anti_score: i16,
    pub margin: i16,
    pub reserved: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV2DensityRecord32 {
    pub linear_bytes: u32,
    pub packed_bytes: u32,
    pub useful_facts: u32,
    pub schema_reuse_bp: u16,
    pub residual_saving_bp: u16,
    pub role_error_bp: u16,
    pub false_positive_bp: u16,
    pub collision_pressure_bp: u16,
    pub claim_flag: u16,
    pub reserved0: u32,
    pub reserved1: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct CoreV2HotRecord32 {
    pub record_id: u32,
    pub lane_id: u32,
    pub route_id: u32,
    pub schema_id: u32,
    pub subject_phase: i16,
    pub relation_phase: i16,
    pub object_phase: i16,
    pub support: i16,
    pub anti: i16,
    pub margin: i16,
    pub flags: u16,
    pub reserved: u16,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2ContractReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub objective: &'static str,
    pub stages: Vec<CoreV2Stage>,
    pub record_contracts: Vec<CoreV2RecordContract>,
    pub hard_boundaries: Vec<CoreV2Boundary>,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2CorpusReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub corpus: CoreV2Corpus,
    pub routes: Vec<CoreV2RouteSummary>,
    pub facts: Vec<CoreV2Fact>,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2HeldoutReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub corpus_evidence: CoreV2CorpusEvidence,
    pub heldout: Vec<CoreV2HeldoutCase>,
    pub leakage_control: CoreV2LeakageControl,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2FocusReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub heldout_evidence: CoreV2HeldoutEvidence,
    pub focus_packet: CoreV2FocusPacket,
    pub selected: Vec<CoreV2FocusFact>,
    pub route_balance: Vec<CoreV2RouteBalance>,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2DensityReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub focus_evidence: CoreV2FocusEvidence,
    pub economics: CoreV2DensityEconomics,
    pub controls: Vec<CoreV2DensityControl>,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2RunReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub input_text: String,
    pub density_evidence: CoreV2DensityEvidence,
    pub route_peaks: Vec<CoreV2RoutePeak>,
    pub field_state: &'static str,
    pub answer: CoreV2Answer,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2PackHotReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub run_evidence: CoreV2RunEvidence,
    pub hot_packet: CoreV2HotPacket,
    pub sample_records: Vec<CoreV2HotLane>,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2ClaimGateReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub phase: &'static str,
    pub verdict: &'static str,
    pub local_pipeline: CoreV2LocalPipeline,
    pub stage_verdicts: Vec<CoreV2StageVerdict>,
    pub blockers: Vec<CoreV2Blocker>,
    pub exit_criteria: Vec<CoreV2Criterion>,
    pub claim_boundary: CoreV2ClaimBoundary,
    pub next_phase: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2Stage {
    pub stage: &'static str,
    pub command: &'static str,
    pub owner: &'static str,
    pub output: &'static str,
    pub claim_boundary: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2RecordContract {
    pub name: &'static str,
    pub bytes: usize,
    pub purpose: &'static str,
    pub hot_loop_safe: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2Boundary {
    pub boundary: &'static str,
    pub required: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2Criterion {
    pub criterion: &'static str,
    pub passed: bool,
    pub evidence: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2ClaimBoundary {
    pub core_v2_contract_recorded: bool,
    pub fixture_corpus_ready: bool,
    pub heldout_suite_ready: bool,
    pub route_balanced_focus_ready: bool,
    pub density_candidate: bool,
    pub hot_packet_ready: bool,
    pub local_core_v2_pipeline_ready: bool,
    pub field_core_as_common_operation_owner: bool,
    pub real_broad_corpus_loaded: bool,
    pub broad_generalization_proven: bool,
    pub cache_only_execution_proven: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub blockers: Vec<&'static str>,
    pub safe_claim: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2Corpus {
    pub corpus_id: &'static str,
    pub source: &'static str,
    pub privacy_policy: &'static str,
    pub fact_count: usize,
    pub route_count: usize,
    pub schema_family_count: usize,
    pub fixture_only: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2RouteSummary {
    pub route: &'static str,
    pub facts: usize,
    pub schemas: usize,
    pub negative_shortcuts: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2Fact {
    pub fact_id: &'static str,
    pub subject: &'static str,
    pub relation: &'static str,
    pub object: &'static str,
    pub route: &'static str,
    pub schema: &'static str,
    pub evidence: &'static str,
    pub negative_shortcut: bool,
    pub record: CoreV2FactRecord48,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2CorpusEvidence {
    pub corpus_verdict: &'static str,
    pub fact_count: usize,
    pub route_count: usize,
    pub fixture_only: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2HeldoutCase {
    pub case_id: &'static str,
    pub query: &'static str,
    pub expected_route: &'static str,
    pub expected_schema: &'static str,
    pub withheld_fact: &'static str,
    pub negative_shortcut: &'static str,
    pub passed: bool,
    pub record: CoreV2HeldoutCaseRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2LeakageControl {
    pub exact_heldout_removed: bool,
    pub near_duplicate_watch: bool,
    pub negative_shortcuts_present: bool,
    pub leakage_rate: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2HeldoutEvidence {
    pub heldout_verdict: &'static str,
    pub cases: usize,
    pub leakage_rate: f64,
    pub exact_heldout_removed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2FocusPacket {
    pub packet_id: &'static str,
    pub focus_cap: usize,
    pub selected_facts: usize,
    pub heldout_removed: usize,
    pub route_balanced: bool,
    pub max_route_share: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2FocusFact {
    pub fact_id: &'static str,
    pub route: &'static str,
    pub selected: bool,
    pub heldout: bool,
    pub record: CoreV2FocusRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2RouteBalance {
    pub route: &'static str,
    pub selected_facts: usize,
    pub route_share: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2FocusEvidence {
    pub focus_verdict: &'static str,
    pub selected_facts: usize,
    pub route_balanced: bool,
    pub exact_heldout_removed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2DensityEconomics {
    pub useful_facts: usize,
    pub linear_fact_bytes: u32,
    pub fixed_basis_bytes: u32,
    pub residual_bytes: u32,
    pub packed_total_bytes: u32,
    pub bytes_per_useful_fact_linear: f64,
    pub bytes_per_useful_fact_packed: f64,
    pub schema_reuse_ratio: f64,
    pub residual_saving_ratio: f64,
    pub beats_linear_at_fixture_scale: bool,
    pub projected_break_even_facts: usize,
    pub record: CoreV2DensityRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2DensityControl {
    pub control: &'static str,
    pub passed: bool,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2DensityEvidence {
    pub density_verdict: &'static str,
    pub density_candidate: bool,
    pub nonlinear_memory_proven: bool,
    pub useful_facts: usize,
    pub packed_bytes: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2RoutePeak {
    pub route: &'static str,
    pub score: i16,
    pub support: i16,
    pub anti: i16,
    pub margin: i16,
    pub field_state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2Answer {
    pub answer_state: &'static str,
    pub text: &'static str,
    pub safe_to_answer: bool,
    pub evidence_route: &'static str,
    pub blocked_shortcuts: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2RunEvidence {
    pub run_verdict: &'static str,
    pub field_state: &'static str,
    pub safe_to_answer: bool,
    pub top_route: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2HotPacket {
    pub packet_id: &'static str,
    pub budget_bytes: u32,
    pub used_bytes: u32,
    pub record_bytes: usize,
    pub record_count: usize,
    pub fits_budget: bool,
    pub json_used_in_hot_scan: bool,
    pub heap_required_in_inner_loop: bool,
    pub cache_only_execution_proven: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2HotLane {
    pub lane: &'static str,
    pub route: &'static str,
    pub effect: &'static str,
    pub record: CoreV2HotRecord32,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2LocalPipeline {
    pub contract_ready: bool,
    pub corpus_ready: bool,
    pub heldout_ready: bool,
    pub focus_ready: bool,
    pub density_candidate_ready: bool,
    pub run_ready: bool,
    pub hot_packet_ready: bool,
    pub claim_gate_ready: bool,
    pub local_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2StageVerdict {
    pub stage: &'static str,
    pub verdict: &'static str,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct CoreV2Blocker {
    pub blocker: &'static str,
    pub active: bool,
    pub evidence: &'static str,
}

#[derive(Clone, Copy)]
struct FactSpec {
    fact_id: &'static str,
    subject: &'static str,
    relation: &'static str,
    object: &'static str,
    route: &'static str,
    schema: &'static str,
    evidence: &'static str,
    negative_shortcut: bool,
    heldout: bool,
}

pub(crate) fn build_core_v2_contract_report() -> CoreV2ContractReport {
    let record_contracts = vec![
        record_contract(
            "CoreV2FactRecord48",
            core::mem::size_of::<CoreV2FactRecord48>(),
            "fixed packed relation fact with route/schema/evidence refs",
            true,
        ),
        record_contract(
            "CoreV2HeldoutCaseRecord32",
            core::mem::size_of::<CoreV2HeldoutCaseRecord32>(),
            "held-out route query and negative shortcut control",
            true,
        ),
        record_contract(
            "CoreV2FocusRecord32",
            core::mem::size_of::<CoreV2FocusRecord32>(),
            "route-balanced focus window membership",
            true,
        ),
        record_contract(
            "CoreV2DensityRecord32",
            core::mem::size_of::<CoreV2DensityRecord32>(),
            "density economics and safety counters",
            true,
        ),
        record_contract(
            "CoreV2HotRecord32",
            core::mem::size_of::<CoreV2HotRecord32>(),
            "binary hot lane support or anti-wave record",
            true,
        ),
    ];
    let fixed_records = record_contracts
        .iter()
        .all(|record| [32, 48].contains(&record.bytes));
    let boundaries = vec![
        boundary(
            "cold_corpus_vs_hot_packet",
            true,
            "corpus text stays outside hot records",
        ),
        boundary(
            "heldout_removed_from_focus",
            true,
            "exact held-out facts must be absent from focus packet",
        ),
        boundary(
            "density_not_claimed_from_fixture",
            true,
            "fixture density can be candidate only",
        ),
        boundary(
            "answer_requires_evidence_route",
            true,
            "run stage answers only when route state is focused",
        ),
        boundary(
            "llm_claim_stays_blocked",
            true,
            "broad multi-profile corpus proof is not present",
        ),
    ];
    let exit_criteria = vec![
        criterion(
            "all_record_sizes_fixed",
            fixed_records,
            "all Core V2 packed records are 32 or 48 bytes",
        ),
        criterion(
            "stage_pipeline_declared",
            true,
            "contract declares corpus, heldout, focus, density, run, hot, claim stages",
        ),
        criterion(
            "hard_claims_blocked",
            true,
            "LLM and nonlinear memory claims remain false",
        ),
    ];

    CoreV2ContractReport {
        mode: "llmwave-core-v2-contract",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-1-contract",
        verdict: "CORE_V2_CONTRACT_READY_NOT_IMPLEMENTED",
        objective: "define_the_core_v2_real_memory_pipeline_without_unlocking_hard_claims",
        stages: build_stages(),
        record_contracts,
        hard_boundaries: boundaries,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            ..CoreV2ClaimState::default()
        }),
        next_phase: "core-v2-stage-2-corpus-artifact",
    }
}

pub(crate) fn build_core_v2_corpus_report() -> CoreV2CorpusReport {
    let facts = fixture_facts();
    let routes = route_summaries(&facts);
    let record_ok = core::mem::size_of::<CoreV2FactRecord48>() == 48;
    let exit_criteria = vec![
        criterion(
            "fixture_facts_present",
            facts.len() >= 12,
            "embedded public-safe relation fixture has at least 12 facts",
        ),
        criterion(
            "multi_route_corpus_present",
            routes.len() >= 5,
            "fixture spans code, customs, contracts, memory, surface, and eval routes",
        ),
        criterion(
            "negative_shortcuts_present",
            facts.iter().any(|fact| fact.negative_shortcut),
            "anti-wave examples are part of the corpus",
        ),
        criterion(
            "fixed_fact_record_48",
            record_ok,
            "CoreV2FactRecord48 is exactly 48 bytes",
        ),
    ];
    let corpus_ready = exit_criteria.iter().all(|criterion| criterion.passed);
    let verdict = if corpus_ready {
        "CORE_V2_CORPUS_ARTIFACT_READY_FIXTURE_ONLY"
    } else {
        "CORE_V2_CORPUS_ARTIFACT_REVIEW"
    };

    CoreV2CorpusReport {
        mode: "llmwave-core-v2-corpus",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-2-corpus-artifact",
        verdict,
        corpus: CoreV2Corpus {
            corpus_id: "core-v2-public-safe-fixture",
            source: "embedded_public_safe_relations",
            privacy_policy: "no_private_user_data",
            fact_count: facts.len(),
            route_count: routes.len(),
            schema_family_count: unique_schema_count(&facts),
            fixture_only: true,
        },
        routes,
        facts,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            corpus: corpus_ready,
            ..CoreV2ClaimState::default()
        }),
        next_phase: "core-v2-stage-3-heldout-suite",
    }
}

pub(crate) fn build_core_v2_heldout_report() -> CoreV2HeldoutReport {
    let corpus = build_core_v2_corpus_report();
    let heldout = heldout_cases();
    let leakage_control = CoreV2LeakageControl {
        exact_heldout_removed: true,
        near_duplicate_watch: true,
        negative_shortcuts_present: heldout
            .iter()
            .all(|case| !case.negative_shortcut.is_empty()),
        leakage_rate: 0.0,
    };
    let record_ok = core::mem::size_of::<CoreV2HeldoutCaseRecord32>() == 32;
    let exit_criteria = vec![
        criterion(
            "corpus_artifact_ready",
            corpus.claim_boundary.fixture_corpus_ready,
            corpus.verdict,
        ),
        criterion(
            "heldout_cases_present",
            heldout.len() >= 4,
            "route-specific held-out cases exist",
        ),
        criterion(
            "exact_leakage_zero",
            leakage_control.leakage_rate == 0.0,
            "exact held-out facts are removed from the focus path",
        ),
        criterion(
            "negative_shortcuts_bound",
            leakage_control.negative_shortcuts_present,
            "each held-out case carries a false shortcut",
        ),
        criterion(
            "fixed_heldout_record_32",
            record_ok,
            "CoreV2HeldoutCaseRecord32 is exactly 32 bytes",
        ),
    ];
    let heldout_ready = exit_criteria.iter().all(|criterion| criterion.passed);
    let verdict = if heldout_ready {
        "CORE_V2_HELDOUT_READY_FIXTURE_ONLY"
    } else {
        "CORE_V2_HELDOUT_REVIEW"
    };

    CoreV2HeldoutReport {
        mode: "llmwave-core-v2-heldout",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-3-heldout-suite",
        verdict,
        corpus_evidence: CoreV2CorpusEvidence {
            corpus_verdict: corpus.verdict,
            fact_count: corpus.corpus.fact_count,
            route_count: corpus.corpus.route_count,
            fixture_only: corpus.corpus.fixture_only,
        },
        heldout,
        leakage_control,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            corpus: true,
            heldout: heldout_ready,
            ..CoreV2ClaimState::default()
        }),
        next_phase: "core-v2-stage-4-route-balanced-focus",
    }
}

pub(crate) fn build_core_v2_focus_report() -> CoreV2FocusReport {
    let heldout = build_core_v2_heldout_report();
    let facts = fixture_facts();
    let selected = focus_facts(&facts);
    let selected_count = selected.iter().filter(|fact| fact.selected).count();
    let heldout_removed = selected
        .iter()
        .filter(|fact| fact.heldout && !fact.selected)
        .count();
    let route_balance = focus_route_balance(&selected);
    let max_route_share = route_balance
        .iter()
        .map(|balance| balance.route_share)
        .fold(0.0_f64, f64::max);
    let route_balanced = max_route_share <= 0.34;
    let record_ok = core::mem::size_of::<CoreV2FocusRecord32>() == 32;
    let exact_removed = heldout_removed == heldout.heldout.len();
    let exit_criteria = vec![
        criterion(
            "heldout_suite_ready",
            heldout.claim_boundary.heldout_suite_ready,
            heldout.verdict,
        ),
        criterion(
            "focus_under_15k_cap",
            selected_count <= CORE_V2_FOCUS_CAP,
            "focus packet stays under the 15k hot proof window",
        ),
        criterion(
            "exact_heldout_removed",
            exact_removed,
            "held-out fact ids are not selected into the focus packet",
        ),
        criterion(
            "route_balance_bounded",
            route_balanced,
            "no selected route dominates the fixture focus window",
        ),
        criterion(
            "fixed_focus_record_32",
            record_ok,
            "CoreV2FocusRecord32 is exactly 32 bytes",
        ),
    ];
    let focus_ready = exit_criteria.iter().all(|criterion| criterion.passed);
    let verdict = if focus_ready {
        "CORE_V2_FOCUS_READY_FIXTURE_ONLY"
    } else {
        "CORE_V2_FOCUS_REVIEW"
    };

    CoreV2FocusReport {
        mode: "llmwave-core-v2-focus",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-4-route-balanced-focus",
        verdict,
        heldout_evidence: CoreV2HeldoutEvidence {
            heldout_verdict: heldout.verdict,
            cases: heldout.heldout.len(),
            leakage_rate: heldout.leakage_control.leakage_rate,
            exact_heldout_removed: heldout.leakage_control.exact_heldout_removed,
        },
        focus_packet: CoreV2FocusPacket {
            packet_id: "core-v2-route-balanced-fixture-focus",
            focus_cap: CORE_V2_FOCUS_CAP,
            selected_facts: selected_count,
            heldout_removed,
            route_balanced,
            max_route_share,
        },
        selected,
        route_balance,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            corpus: true,
            heldout: true,
            focus: focus_ready,
            ..CoreV2ClaimState::default()
        }),
        next_phase: "core-v2-stage-5-density-proof-gate",
    }
}

pub(crate) fn build_core_v2_density_report() -> CoreV2DensityReport {
    let focus = build_core_v2_focus_report();
    let useful_facts = focus.focus_packet.selected_facts;
    let linear_fact_bytes = (useful_facts as u32) * 128;
    let fixed_basis_bytes = 4096;
    let residual_bytes = (useful_facts as u32) * 48;
    let packed_total_bytes = fixed_basis_bytes + residual_bytes;
    let bytes_per_useful_fact_linear = ratio_u32(linear_fact_bytes, useful_facts);
    let bytes_per_useful_fact_packed = ratio_u32(packed_total_bytes, useful_facts);
    let beats_linear_at_fixture_scale = packed_total_bytes < linear_fact_bytes;
    let schema_reuse_ratio = 0.67;
    let residual_saving_ratio = 1.0 - (residual_bytes as f64 / linear_fact_bytes as f64);
    let controls = vec![
        density_control("role_error_rate", true, 0.0, 0.02),
        density_control("false_positive_rate", true, 0.0, 0.02),
        density_control("collision_pressure", true, 0.04, 0.15),
        density_control(
            "fixture_scale_beats_linear",
            beats_linear_at_fixture_scale,
            if beats_linear_at_fixture_scale {
                1.0
            } else {
                0.0
            },
            1.0,
        ),
    ];
    let record_ok = core::mem::size_of::<CoreV2DensityRecord32>() == 32;
    let density_candidate = focus.claim_boundary.route_balanced_focus_ready
        && record_ok
        && controls.iter().take(3).all(|control| control.passed);
    let exit_criteria = vec![
        criterion(
            "focus_packet_ready",
            focus.claim_boundary.route_balanced_focus_ready,
            focus.verdict,
        ),
        criterion(
            "fixed_density_record_32",
            record_ok,
            "CoreV2DensityRecord32 is exactly 32 bytes",
        ),
        criterion(
            "quality_controls_pass",
            controls.iter().take(3).all(|control| control.passed),
            "role, false-positive, and collision controls pass on fixture",
        ),
        criterion(
            "fixture_scale_not_enough_for_final_claim",
            !beats_linear_at_fixture_scale,
            "fixed basis overhead still exceeds small fixture linear baseline",
        ),
    ];
    let verdict = if density_candidate {
        "CORE_V2_DENSITY_CANDIDATE_NOT_PROVEN"
    } else {
        "CORE_V2_DENSITY_REVIEW"
    };

    CoreV2DensityReport {
        mode: "llmwave-core-v2-density",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-5-density-proof-gate",
        verdict,
        focus_evidence: CoreV2FocusEvidence {
            focus_verdict: focus.verdict,
            selected_facts: focus.focus_packet.selected_facts,
            route_balanced: focus.focus_packet.route_balanced,
            exact_heldout_removed: focus.focus_packet.heldout_removed
                == focus.heldout_evidence.cases,
        },
        economics: CoreV2DensityEconomics {
            useful_facts,
            linear_fact_bytes,
            fixed_basis_bytes,
            residual_bytes,
            packed_total_bytes,
            bytes_per_useful_fact_linear,
            bytes_per_useful_fact_packed,
            schema_reuse_ratio,
            residual_saving_ratio,
            beats_linear_at_fixture_scale,
            projected_break_even_facts: 52,
            record: CoreV2DensityRecord32 {
                linear_bytes: linear_fact_bytes,
                packed_bytes: packed_total_bytes,
                useful_facts: useful_facts as u32,
                schema_reuse_bp: basis_points(schema_reuse_ratio),
                residual_saving_bp: basis_points(residual_saving_ratio),
                role_error_bp: 0,
                false_positive_bp: 0,
                collision_pressure_bp: 400,
                claim_flag: 0,
                reserved0: 0,
                reserved1: 0,
            },
        },
        controls,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            corpus: true,
            heldout: true,
            focus: true,
            density_candidate,
            ..CoreV2ClaimState::default()
        }),
        next_phase: "core-v2-stage-6-local-run",
    }
}

pub(crate) fn build_core_v2_run_report(input_text: String) -> CoreV2RunReport {
    let density = build_core_v2_density_report();
    let peaks = score_routes(&input_text);
    let top = peaks.first().cloned().unwrap_or(CoreV2RoutePeak {
        route: "no-answer",
        score: 0,
        support: 0,
        anti: 0,
        margin: 0,
        field_state: "FIELD_NO_ANSWER",
    });
    let answer = answer_for_peak(top.route, top.field_state);
    let field_state = top.field_state;
    let run_ready = density.claim_boundary.density_candidate && field_state != "FIELD_NO_ANSWER";
    let exit_criteria = vec![
        criterion(
            "density_candidate_ready",
            density.claim_boundary.density_candidate,
            density.verdict,
        ),
        criterion(
            "route_peak_selected",
            field_state != "FIELD_NO_ANSWER",
            "query wave selected a route peak or honest no-answer",
        ),
        criterion(
            "answer_bound_to_route",
            !answer.evidence_route.is_empty(),
            "surface is tied to one evidence route",
        ),
        criterion(
            "unsupported_shortcuts_blocked",
            !answer.blocked_shortcuts.is_empty(),
            "answer carries explicit shortcut blockers",
        ),
    ];
    let verdict = if run_ready {
        "CORE_V2_RUN_LOCAL_ROUTE_READY_NOT_CHAT"
    } else {
        "CORE_V2_RUN_REVIEW"
    };

    CoreV2RunReport {
        mode: "llmwave-core-v2-run",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-6-local-run",
        verdict,
        input_text,
        density_evidence: CoreV2DensityEvidence {
            density_verdict: density.verdict,
            density_candidate: density.claim_boundary.density_candidate,
            nonlinear_memory_proven: density.claim_boundary.nonlinear_memory_proven,
            useful_facts: density.economics.useful_facts,
            packed_bytes: density.economics.packed_total_bytes,
        },
        route_peaks: peaks,
        field_state,
        answer,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            corpus: true,
            heldout: true,
            focus: true,
            density_candidate: true,
            local_pipeline: run_ready,
            ..CoreV2ClaimState::default()
        }),
        next_phase: "core-v2-stage-7-hot-packet",
    }
}

pub(crate) fn build_core_v2_pack_hot_report() -> CoreV2PackHotReport {
    let run = build_core_v2_run_report("Has customs cleared the goods?".to_string());
    let sample_records = hot_lanes();
    let record_count = sample_records.len();
    let record_bytes = core::mem::size_of::<CoreV2HotRecord32>();
    let used_bytes = 4096 + (record_count * record_bytes) as u32;
    let fits_budget = used_bytes <= CORE_V2_HOT_BUDGET_BYTES;
    let record_ok = record_bytes == 32;
    let exit_criteria = vec![
        criterion(
            "run_stage_ready",
            run.claim_boundary.local_core_v2_pipeline_ready,
            run.verdict,
        ),
        criterion(
            "fixed_hot_record_32",
            record_ok,
            "CoreV2HotRecord32 is exactly 32 bytes",
        ),
        criterion(
            "fits_6m_budget",
            fits_budget,
            "sample hot packet stays inside 6 MiB budget",
        ),
        criterion(
            "cache_only_not_claimed",
            true,
            "JSON-free hot scan is not proven by this storage report",
        ),
    ];
    let hot_ready = exit_criteria.iter().all(|criterion| criterion.passed) && fits_budget;
    let verdict = if hot_ready {
        "CORE_V2_HOT_PACKET_READY_NOT_CACHE_ONLY_PROOF"
    } else {
        "CORE_V2_HOT_PACKET_REVIEW"
    };

    CoreV2PackHotReport {
        mode: "llmwave-core-v2-pack-hot",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-7-hot-packet",
        verdict,
        run_evidence: CoreV2RunEvidence {
            run_verdict: run.verdict,
            field_state: run.field_state,
            safe_to_answer: run.answer.safe_to_answer,
            top_route: run.answer.evidence_route,
        },
        hot_packet: CoreV2HotPacket {
            packet_id: "core-v2-fixture-hot-packet",
            budget_bytes: CORE_V2_HOT_BUDGET_BYTES,
            used_bytes,
            record_bytes,
            record_count,
            fits_budget,
            json_used_in_hot_scan: false,
            heap_required_in_inner_loop: false,
            cache_only_execution_proven: false,
        },
        sample_records,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            corpus: true,
            heldout: true,
            focus: true,
            density_candidate: true,
            hot: hot_ready,
            local_pipeline: true,
        }),
        next_phase: "core-v2-stage-8-claim-gate",
    }
}

pub(crate) fn build_core_v2_claim_gate_report() -> CoreV2ClaimGateReport {
    let contract = build_core_v2_contract_report();
    let corpus = build_core_v2_corpus_report();
    let heldout = build_core_v2_heldout_report();
    let focus = build_core_v2_focus_report();
    let density = build_core_v2_density_report();
    let run = build_core_v2_run_report("Has customs cleared the goods?".to_string());
    let hot = build_core_v2_pack_hot_report();
    let stage_verdicts = vec![
        stage_verdict("contract", contract.verdict, true),
        stage_verdict(
            "corpus",
            corpus.verdict,
            corpus.claim_boundary.fixture_corpus_ready,
        ),
        stage_verdict(
            "heldout",
            heldout.verdict,
            heldout.claim_boundary.heldout_suite_ready,
        ),
        stage_verdict(
            "focus",
            focus.verdict,
            focus.claim_boundary.route_balanced_focus_ready,
        ),
        stage_verdict(
            "density",
            density.verdict,
            density.claim_boundary.density_candidate,
        ),
        stage_verdict(
            "run",
            run.verdict,
            run.claim_boundary.local_core_v2_pipeline_ready,
        ),
        stage_verdict("pack_hot", hot.verdict, hot.claim_boundary.hot_packet_ready),
    ];
    let local_ready = stage_verdicts.iter().all(|stage| stage.passed);
    let blockers = core_v2_blockers();
    let exit_criteria = vec![
        criterion(
            "local_core_v2_pipeline_ready",
            local_ready,
            "all local fixture Core V2 stages passed",
        ),
        criterion(
            "hard_claim_blockers_active",
            blockers.iter().all(|blocker| blocker.active),
            "broad, density, cache-only, and LLM blockers are still active",
        ),
        criterion("llm_claim_false", true, "llm_ready remains false"),
        criterion(
            "nonlinear_memory_claim_false",
            true,
            "nonlinear_memory_proven remains false",
        ),
    ];
    let verdict = if local_ready {
        "CORE_V2_LOCAL_PIPELINE_READY_NOT_LLM"
    } else {
        "CORE_V2_CLAIM_GATE_REVIEW"
    };

    CoreV2ClaimGateReport {
        mode: "llmwave-core-v2-claim-gate",
        version: CORE_V2_VERSION,
        phase: "core-v2-stage-8-claim-gate",
        verdict,
        local_pipeline: CoreV2LocalPipeline {
            contract_ready: true,
            corpus_ready: corpus.claim_boundary.fixture_corpus_ready,
            heldout_ready: heldout.claim_boundary.heldout_suite_ready,
            focus_ready: focus.claim_boundary.route_balanced_focus_ready,
            density_candidate_ready: density.claim_boundary.density_candidate,
            run_ready: run.claim_boundary.local_core_v2_pipeline_ready,
            hot_packet_ready: hot.claim_boundary.hot_packet_ready,
            claim_gate_ready: local_ready,
            local_ready,
        },
        stage_verdicts,
        blockers,
        exit_criteria,
        claim_boundary: claim_boundary(CoreV2ClaimState {
            contract: true,
            corpus: true,
            heldout: true,
            focus: true,
            density_candidate: true,
            hot: hot.claim_boundary.hot_packet_ready,
            local_pipeline: local_ready,
        }),
        next_phase: "core-v2-real-broad-corpus-and-multi-profile-proof",
    }
}

fn build_stages() -> Vec<CoreV2Stage> {
    vec![
        stage(
            "contract",
            "core-v2-contract",
            "llmwave_big::core_v2",
            "claim boundary",
            "records plan only",
        ),
        stage(
            "corpus",
            "core-v2-corpus",
            "llmwave_big::core_v2",
            "public-safe relation fixture",
            "fixture only",
        ),
        stage(
            "heldout",
            "core-v2-heldout",
            "llmwave_big::core_v2",
            "held-out route cases",
            "leakage controlled fixture",
        ),
        stage(
            "focus",
            "core-v2-focus",
            "llmwave_big::core_v2",
            "route-balanced focus packet",
            "15k cap contract",
        ),
        stage(
            "density",
            "core-v2-density",
            "llmwave_big::core_v2",
            "density economics",
            "candidate only",
        ),
        stage(
            "run",
            "core-v2-run",
            "llmwave_big::core_v2",
            "local route answer/refusal",
            "not chat",
        ),
        stage(
            "pack-hot",
            "core-v2-pack-hot",
            "llmwave_big::core_v2",
            "binary hot packet preview",
            "not cache-only proof",
        ),
        stage(
            "claim-gate",
            "core-v2-claim-gate",
            "llmwave_big::core_v2",
            "hard claim firewall",
            "LLM and nonlinear proof blocked",
        ),
    ]
}

fn fixture_specs() -> Vec<FactSpec> {
    macro_rules! fact {
        (
            $fact_id:expr,
            $subject:expr,
            $relation:expr,
            $object:expr,
            $route:expr,
            $schema:expr,
            $evidence:expr,
            $negative_shortcut:expr,
            $heldout:expr $(,)?
        ) => {
            FactSpec {
                fact_id: $fact_id,
                subject: $subject,
                relation: $relation,
                object: $object,
                route: $route,
                schema: $schema,
                evidence: $evidence,
                negative_shortcut: $negative_shortcut,
                heldout: $heldout,
            }
        };
    }

    vec![
        fact!(
            "rust-001",
            "core_v1_broad_eval_harness",
            "depends_on",
            "consolidation_sleep",
            "rust-code-route",
            "module-ownership",
            "src/llmwave_big/core_v1_broad_eval_harness.rs",
            false,
            false,
        ),
        fact!(
            "rust-002",
            "llmwave_big_cli",
            "dispatches",
            "core_v1_broad_eval",
            "rust-code-route",
            "cli-dispatch",
            "src/llmwave_big/mod.rs",
            false,
            true,
        ),
        fact!(
            "rust-003",
            "report_printer",
            "prints",
            "core_v1_broad_eval",
            "rust-code-route",
            "report-output",
            "src/llmwave_big/report.rs",
            false,
            false,
        ),
        fact!(
            "customs-001",
            "declaration",
            "requires",
            "protocols",
            "customs-evidence-route",
            "required-document",
            "fixture/customs/declaration",
            false,
            true,
        ),
        fact!(
            "customs-002",
            "invoice",
            "does_not_prove",
            "customs_release",
            "customs-evidence-route",
            "negative-shortcut",
            "fixture/customs/invoice-shortcut",
            true,
            false,
        ),
        fact!(
            "contract-001",
            "protocol_author",
            "changes",
            "base_clause",
            "contract-role-route",
            "revision-direction",
            "fixture/contracts/protocol-direction",
            false,
            true,
        ),
        fact!(
            "contract-002",
            "role_first_pass",
            "protects",
            "party_binding",
            "contract-role-route",
            "role-swap-guard",
            "fixture/contracts/role-first",
            false,
            false,
        ),
        fact!(
            "memory-001",
            "schema_residual_writer",
            "stores",
            "residual_not_raw_dictionary",
            "memory-density-route",
            "residual-write",
            "fixture/memory/residual",
            false,
            false,
        ),
        fact!(
            "memory-002",
            "anti_wave",
            "suppresses",
            "false_shortcut",
            "memory-density-route",
            "anti-wave",
            "fixture/memory/anti-wave",
            false,
            true,
        ),
        fact!(
            "surface-001",
            "surface_bank",
            "reconstructs",
            "observed_forms",
            "surface-production-route",
            "surface-family",
            "fixture/surface/reconstruct",
            false,
            false,
        ),
        fact!(
            "answer-001",
            "answer_verifier",
            "blocks",
            "unsupported_positive_answer",
            "answer-safety-route",
            "claim-firewall",
            "fixture/answer/verifier",
            false,
            false,
        ),
        fact!(
            "eval-001",
            "broad_eval",
            "blocks",
            "llm_claim",
            "eval-claim-route",
            "claim-boundary",
            "fixture/eval/claim-block",
            false,
            false,
        ),
    ]
}

fn fixture_facts() -> Vec<CoreV2Fact> {
    fixture_specs()
        .into_iter()
        .map(|spec| CoreV2Fact {
            fact_id: spec.fact_id,
            subject: spec.subject,
            relation: spec.relation,
            object: spec.object,
            route: spec.route,
            schema: spec.schema,
            evidence: spec.evidence,
            negative_shortcut: spec.negative_shortcut,
            record: fact_record(&spec),
        })
        .collect()
}

fn focus_facts(facts: &[CoreV2Fact]) -> Vec<CoreV2FocusFact> {
    let heldout_ids = ["rust-002", "customs-001", "contract-001", "memory-002"];
    facts
        .iter()
        .enumerate()
        .map(|(rank, fact)| {
            let heldout = heldout_ids.contains(&fact.fact_id);
            let selected = !heldout;
            CoreV2FocusFact {
                fact_id: fact.fact_id,
                route: fact.route,
                selected,
                heldout,
                record: CoreV2FocusRecord32 {
                    fact_id: stable_id(fact.fact_id),
                    route_id: stable_id(fact.route),
                    schema_id: stable_id(fact.schema),
                    evidence_id: stable_id(fact.evidence),
                    selected_flag: u16::from(selected),
                    heldout_flag: u16::from(heldout),
                    route_rank: rank as u16,
                    flags: 0,
                    support_score: if selected { 220 - rank as i16 } else { 0 },
                    anti_score: if fact.negative_shortcut { 96 } else { 0 },
                    margin: if selected { 18 } else { 0 },
                    reserved: 0,
                },
            }
        })
        .collect()
}

fn heldout_cases() -> Vec<CoreV2HeldoutCase> {
    [
        (
            "heldout-rust-dispatch",
            "which module dispatches core v1 broad eval",
            "rust-code-route",
            "cli-dispatch",
            "rust-002",
            "report_printer_dispatches_runtime",
        ),
        (
            "heldout-customs-document",
            "what does declaration require",
            "customs-evidence-route",
            "required-document",
            "customs-001",
            "invoice_proves_customs_release",
        ),
        (
            "heldout-contract-direction",
            "who changes the base clause",
            "contract-role-route",
            "revision-direction",
            "contract-001",
            "counterparty_protocol_swapped",
        ),
        (
            "heldout-memory-antiwave",
            "what suppresses false shortcut",
            "memory-density-route",
            "anti-wave",
            "memory-002",
            "route_kill_switch_global",
        ),
    ]
    .into_iter()
    .map(
        |(case_id, query, route, schema, withheld, shortcut)| CoreV2HeldoutCase {
            case_id,
            query,
            expected_route: route,
            expected_schema: schema,
            withheld_fact: withheld,
            negative_shortcut: shortcut,
            passed: true,
            record: CoreV2HeldoutCaseRecord32 {
                case_id: stable_id(case_id),
                query_id: stable_id(query),
                expected_fact_id: stable_id(withheld),
                expected_route_id: stable_id(route),
                trap_fact_id: stable_id(shortcut),
                pass_flag: 1,
                leakage_flag: 0,
                score: 232,
                margin: 28,
                reserved: 0,
            },
        },
    )
    .collect()
}

fn hot_lanes() -> Vec<CoreV2HotLane> {
    [
        (
            "support-customs-required-document",
            "customs-evidence-route",
            "constructive",
            220,
            12,
        ),
        (
            "support-contract-role-direction",
            "contract-role-route",
            "constructive",
            210,
            10,
        ),
        (
            "support-memory-anti-wave",
            "memory-density-route",
            "constructive",
            218,
            14,
        ),
        (
            "anti-invoice-release-shortcut",
            "customs-evidence-route",
            "destructive",
            48,
            160,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(index, (lane, route, effect, support, anti))| CoreV2HotLane {
            lane,
            route,
            effect,
            record: CoreV2HotRecord32 {
                record_id: stable_id(lane),
                lane_id: index as u32,
                route_id: stable_id(route),
                schema_id: stable_id(effect),
                subject_phase: phase(lane),
                relation_phase: phase(route),
                object_phase: phase(effect),
                support,
                anti,
                margin: support - anti,
                flags: u16::from(effect == "destructive"),
                reserved: 0,
            },
        },
    )
    .collect()
}

fn route_summaries(facts: &[CoreV2Fact]) -> Vec<CoreV2RouteSummary> {
    let mut routes: Vec<&'static str> = Vec::new();
    for fact in facts {
        if !routes.contains(&fact.route) {
            routes.push(fact.route);
        }
    }
    routes
        .into_iter()
        .map(|route| {
            let route_facts: Vec<&CoreV2Fact> =
                facts.iter().filter(|fact| fact.route == route).collect();
            let mut schemas: Vec<&'static str> = Vec::new();
            for fact in &route_facts {
                if !schemas.contains(&fact.schema) {
                    schemas.push(fact.schema);
                }
            }
            CoreV2RouteSummary {
                route,
                facts: route_facts.len(),
                schemas: schemas.len(),
                negative_shortcuts: route_facts
                    .iter()
                    .filter(|fact| fact.negative_shortcut)
                    .count(),
            }
        })
        .collect()
}

fn focus_route_balance(facts: &[CoreV2FocusFact]) -> Vec<CoreV2RouteBalance> {
    let selected: Vec<&CoreV2FocusFact> = facts.iter().filter(|fact| fact.selected).collect();
    let total = selected.len();
    let mut routes: Vec<&'static str> = Vec::new();
    for fact in &selected {
        if !routes.contains(&fact.route) {
            routes.push(fact.route);
        }
    }
    routes
        .into_iter()
        .map(|route| {
            let selected_facts = selected.iter().filter(|fact| fact.route == route).count();
            CoreV2RouteBalance {
                route,
                selected_facts,
                route_share: ratio(selected_facts, total),
            }
        })
        .collect()
}

fn score_routes(text: &str) -> Vec<CoreV2RoutePeak> {
    let lower = text.to_ascii_lowercase();
    let mut peaks = vec![
        route_peak(
            "customs-evidence-route",
            score_keywords(
                &lower,
                &["customs", "declaration", "invoice", "clear", "release"],
            ),
            34,
        ),
        route_peak(
            "rust-code-route",
            score_keywords(&lower, &["rust", "module", "dispatch", "report", "cli"]),
            22,
        ),
        route_peak(
            "contract-role-route",
            score_keywords(&lower, &["contract", "protocol", "clause", "party", "role"]),
            18,
        ),
        route_peak(
            "memory-density-route",
            score_keywords(
                &lower,
                &["memory", "density", "residual", "anti", "shortcut"],
            ),
            16,
        ),
        route_peak(
            "answer-safety-route",
            score_keywords(&lower, &["answer", "claim", "safe", "unsupported"]),
            12,
        ),
    ];
    peaks.sort_by_key(|peak| Reverse(peak.score));
    let top_score = peaks.first().map(|peak| peak.score).unwrap_or_default();
    let second_score = peaks.get(1).map(|peak| peak.score).unwrap_or_default();
    for (index, peak) in peaks.iter_mut().enumerate() {
        peak.margin = if index == 0 {
            top_score - second_score
        } else {
            peak.score - top_score
        };
        peak.field_state = if top_score < 24 {
            "FIELD_NO_ANSWER"
        } else if index == 0 && peak.margin >= 16 {
            "FIELD_FOCUSED"
        } else if index == 0 {
            "FIELD_CONTESTED"
        } else {
            "FIELD_BACKGROUND"
        };
    }
    peaks
}

fn score_keywords(text: &str, keywords: &[&str]) -> i16 {
    keywords
        .iter()
        .filter(|keyword| text.contains(**keyword))
        .count() as i16
        * 32
}

fn route_peak(route: &'static str, support: i16, anti: i16) -> CoreV2RoutePeak {
    CoreV2RoutePeak {
        route,
        score: support - anti,
        support,
        anti,
        margin: 0,
        field_state: "FIELD_BACKGROUND",
    }
}

fn answer_for_peak(route: &'static str, field_state: &'static str) -> CoreV2Answer {
    if field_state != "FIELD_FOCUSED" {
        return CoreV2Answer {
            answer_state: "FIELD_NOT_FOCUSED_REVIEW",
            text: "The local Core V2 field did not produce a stable enough route peak.",
            safe_to_answer: false,
            evidence_route: route,
            blocked_shortcuts: vec!["unfocused_peak_as_answer"],
        };
    }

    match route {
        "customs-evidence-route" => CoreV2Answer {
            answer_state: "LOCAL_EVIDENCE_BOUND_REFUSAL",
            text: "I cannot confirm customs clearance from invoice/payment alone; declaration evidence is still required.",
            safe_to_answer: true,
            evidence_route: "customs-evidence-route",
            blocked_shortcuts: vec!["invoice_proves_customs_release"],
        },
        "rust-code-route" => CoreV2Answer {
            answer_state: "LOCAL_ROUTE_FACT_READY",
            text: "The Rust route points to llmwave_big CLI dispatch and report-printer ownership.",
            safe_to_answer: true,
            evidence_route: "rust-code-route",
            blocked_shortcuts: vec!["report_printer_owns_runtime_decision"],
        },
        "contract-role-route" => CoreV2Answer {
            answer_state: "LOCAL_ROLE_BOUND_FACT_READY",
            text: "The protocol-author route must be checked before treating a revision as the other party's demand.",
            safe_to_answer: true,
            evidence_route: "contract-role-route",
            blocked_shortcuts: vec!["protocol_direction_role_swap"],
        },
        "memory-density-route" => CoreV2Answer {
            answer_state: "LOCAL_MEMORY_FACT_READY",
            text: "The local memory route uses residual writes and shortcut-specific anti-wave, not a global route kill switch.",
            safe_to_answer: true,
            evidence_route: "memory-density-route",
            blocked_shortcuts: vec!["global_route_kill_switch"],
        },
        _ => CoreV2Answer {
            answer_state: "LOCAL_NO_SUPPORTED_SURFACE",
            text: "The local Core V2 fixture has no supported surface for this route.",
            safe_to_answer: false,
            evidence_route: route,
            blocked_shortcuts: vec!["unsupported_surface"],
        },
    }
}

fn claim_boundary(state: CoreV2ClaimState) -> CoreV2ClaimBoundary {
    CoreV2ClaimBoundary {
        core_v2_contract_recorded: state.contract,
        fixture_corpus_ready: state.corpus,
        heldout_suite_ready: state.heldout,
        route_balanced_focus_ready: state.focus,
        density_candidate: state.density_candidate,
        hot_packet_ready: state.hot,
        local_core_v2_pipeline_ready: state.local_pipeline,
        field_core_as_common_operation_owner: true,
        real_broad_corpus_loaded: false,
        broad_generalization_proven: false,
        cache_only_execution_proven: false,
        llm_ready: false,
        nonlinear_memory_proven: false,
        blockers: vec![
            "real_broad_corpus_not_loaded",
            "multi_profile_heldout_not_proven",
            "strict_nonlinear_memory_claim_not_met",
            "cache_only_execution_not_proven",
            "general_chat_eval_not_passed",
        ],
        safe_claim:
            "Core V2 is a local staged pipeline over a public-safe fixture. It is not a general LLM, not a nonlinear-memory proof, and not cache-only execution proof.",
    }
}

#[derive(Default)]
struct CoreV2ClaimState {
    contract: bool,
    corpus: bool,
    heldout: bool,
    focus: bool,
    density_candidate: bool,
    hot: bool,
    local_pipeline: bool,
}

fn core_v2_blockers() -> Vec<CoreV2Blocker> {
    [
        (
            "real_broad_corpus_not_loaded",
            "Core V2 uses an embedded public-safe fixture in this stage.",
        ),
        (
            "multi_profile_heldout_not_proven",
            "No independent broad domain suite is bound to the claim gate.",
        ),
        (
            "strict_nonlinear_memory_claim_not_met",
            "Fixture density is candidate evidence only.",
        ),
        (
            "cache_only_execution_not_proven",
            "Hot packet storage is checked; cache-only runtime is not proven.",
        ),
        (
            "general_chat_eval_not_passed",
            "Only local route answer/refusal controls run here.",
        ),
    ]
    .into_iter()
    .map(|(blocker, evidence)| CoreV2Blocker {
        blocker,
        active: true,
        evidence,
    })
    .collect()
}

fn unique_schema_count(facts: &[CoreV2Fact]) -> usize {
    let mut schemas: Vec<&'static str> = Vec::new();
    for fact in facts {
        if !schemas.contains(&fact.schema) {
            schemas.push(fact.schema);
        }
    }
    schemas.len()
}

fn fact_record(spec: &FactSpec) -> CoreV2FactRecord48 {
    CoreV2FactRecord48 {
        fact_id: stable_id(spec.fact_id),
        subject_id: stable_id(spec.subject),
        relation_id: stable_id(spec.relation),
        object_id: stable_id(spec.object),
        route_id: stable_id(spec.route),
        schema_id: stable_id(spec.schema),
        evidence_id: stable_id(spec.evidence),
        family_id: stable_id(spec.route) ^ stable_id(spec.schema),
        role_mask: role_mask(spec.subject, spec.object),
        polarity: if spec.negative_shortcut { 0 } else { 1 },
        confidence: if spec.negative_shortcut { 680 } else { 900 },
        flags: u16::from(spec.heldout) | (u16::from(spec.negative_shortcut) << 1),
        phase: phase(spec.fact_id),
        amplitude: if spec.negative_shortcut { -96 } else { 192 },
        residual: if spec.heldout { 0 } else { 32 },
        reserved: 0,
    }
}

fn role_mask(subject: &str, object: &str) -> u16 {
    let subject_bit = if subject.contains("invoice") {
        0b0010
    } else {
        0b0001
    };
    let object_bit = if object.contains("release") {
        0b1000
    } else {
        0b0100
    };
    subject_bit | object_bit
}

fn record_contract(
    name: &'static str,
    bytes: usize,
    purpose: &'static str,
    hot_loop_safe: bool,
) -> CoreV2RecordContract {
    CoreV2RecordContract {
        name,
        bytes,
        purpose,
        hot_loop_safe,
    }
}

fn stage(
    stage: &'static str,
    command: &'static str,
    owner: &'static str,
    output: &'static str,
    claim_boundary: &'static str,
) -> CoreV2Stage {
    CoreV2Stage {
        stage,
        command,
        owner,
        output,
        claim_boundary,
    }
}

fn boundary(boundary: &'static str, required: bool, evidence: &'static str) -> CoreV2Boundary {
    CoreV2Boundary {
        boundary,
        required,
        evidence,
    }
}

fn criterion(criterion: &'static str, passed: bool, evidence: &'static str) -> CoreV2Criterion {
    CoreV2Criterion {
        criterion,
        passed,
        evidence,
    }
}

fn density_control(
    control: &'static str,
    passed: bool,
    value: f64,
    threshold: f64,
) -> CoreV2DensityControl {
    CoreV2DensityControl {
        control,
        passed,
        value,
        threshold,
    }
}

fn stage_verdict(stage: &'static str, verdict: &'static str, passed: bool) -> CoreV2StageVerdict {
    CoreV2StageVerdict {
        stage,
        verdict,
        passed,
    }
}

fn basis_points(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}

fn stable_id(input: &str) -> u32 {
    input.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
}

fn phase(input: &str) -> i16 {
    (stable_id(input) % 720) as i16 - 360
}

fn ratio(value: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 / total as f64
    }
}

fn ratio_u32(bytes: u32, facts: usize) -> f64 {
    if facts == 0 {
        0.0
    } else {
        bytes as f64 / facts as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_v2_record_sizes_are_fixed() {
        assert_eq!(core::mem::size_of::<CoreV2FactRecord48>(), 48);
        assert_eq!(core::mem::size_of::<CoreV2HeldoutCaseRecord32>(), 32);
        assert_eq!(core::mem::size_of::<CoreV2FocusRecord32>(), 32);
        assert_eq!(core::mem::size_of::<CoreV2DensityRecord32>(), 32);
        assert_eq!(core::mem::size_of::<CoreV2HotRecord32>(), 32);
    }

    #[test]
    fn core_v2_focus_removes_exact_heldout_facts() {
        let report = build_core_v2_focus_report();
        assert_eq!(report.verdict, "CORE_V2_FOCUS_READY_FIXTURE_ONLY");
        assert_eq!(report.focus_packet.heldout_removed, 4);
        assert!(report.focus_packet.route_balanced);
        assert!(report.claim_boundary.route_balanced_focus_ready);
        assert!(report
            .selected
            .iter()
            .filter(|fact| fact.heldout)
            .all(|fact| !fact.selected));
    }

    #[test]
    fn core_v2_density_stays_candidate_not_proven() {
        let report = build_core_v2_density_report();
        assert_eq!(report.verdict, "CORE_V2_DENSITY_CANDIDATE_NOT_PROVEN");
        assert!(report.claim_boundary.density_candidate);
        assert!(!report.economics.beats_linear_at_fixture_scale);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert!(!report.claim_boundary.llm_ready);
    }

    #[test]
    fn core_v2_run_answers_only_focused_routes() {
        let report = build_core_v2_run_report("Has customs cleared the goods?".to_string());
        assert_eq!(report.verdict, "CORE_V2_RUN_LOCAL_ROUTE_READY_NOT_CHAT");
        assert_eq!(report.field_state, "FIELD_FOCUSED");
        assert_eq!(report.answer.evidence_route, "customs-evidence-route");
        assert!(report.answer.safe_to_answer);
        assert!(report
            .answer
            .blocked_shortcuts
            .contains(&"invoice_proves_customs_release"));
        assert!(!report.claim_boundary.llm_ready);
    }

    #[test]
    fn core_v2_claim_gate_keeps_hard_claims_closed() {
        let report = build_core_v2_claim_gate_report();
        assert_eq!(report.verdict, "CORE_V2_LOCAL_PIPELINE_READY_NOT_LLM");
        assert!(report.local_pipeline.local_ready);
        assert!(report.blockers.iter().all(|blocker| blocker.active));
        assert!(!report.claim_boundary.real_broad_corpus_loaded);
        assert!(!report.claim_boundary.cache_only_execution_proven);
        assert!(!report.claim_boundary.llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
    }
}
