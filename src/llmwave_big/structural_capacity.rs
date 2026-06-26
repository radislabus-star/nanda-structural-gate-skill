//! Fixed 1024 Pattern16 structural-capacity gate for LLMWave Big.
//!
//! This is the bridge from the old robust 128-pattern checkpoint to the current
//! active/schema-residual core. It deliberately exposes no smaller pattern
//! count and no smaller cell shape: the gate is 1024 Pattern16 or it is not the
//! gate.

use serde::Serialize;

use crate::field_core::{
    run_field_pass, FieldAntiWaveLane, FieldClaimBoundary, FieldFamily, FieldLensKind,
    FieldLensOperation, FieldPassInput, FieldPassReport, FieldRecord, FieldRecordKind,
    FIELD_CORE_VERSION, FIELD_PASS_VERSION,
};

pub(crate) const STRUCTURAL_CAPACITY_VERSION: &str =
    "llmwave-big-v-next-structural-capacity-1024-pattern16";

pub(crate) const STRUCTURAL_PATTERN_CAPACITY: usize = 1024;

const DEFAULT_HOT_BUDGET_BYTES: usize = 6 * 1024 * 1024;
const EDGES_PER_PATTERN: usize = 16;
const DIRECT_FIXED64_BYTES: usize = 64;
const SCHEMA_RECORD_BYTES: usize = 32;
const RESIDUAL_RECORD_BYTES: usize = 24;
const PATTERN_INDEX_RECORD_BYTES: usize = 16;
const ANTI_LANE_RECORD_BYTES: usize = 16;
const OLD_ROBUST_BASELINE_PATTERNS: usize = 128;
const OLD_TURBO_CELLS: usize = 256;
const SKILL_ADMISSION_MIN_SEEDS: usize = 8;
const SKILL_ADMISSION_NOISE_EDGES: usize = EDGES_PER_PATTERN;
const SINGLE_PEAK_NOISE_MARGIN_MIN: i64 = 1_000;

#[derive(Clone)]
pub(crate) struct StructuralCapacityConfig {
    pub seed: u64,
    pub seeds: usize,
    pub noise_edges: usize,
    pub hot_budget_bytes: usize,
    pub noise_profile: StructuralCapacityNoiseProfile,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum StructuralCapacityNoiseProfile {
    Default,
    SkillAdmission,
}

impl StructuralCapacityNoiseProfile {
    fn as_str(self) -> &'static str {
        match self {
            StructuralCapacityNoiseProfile::Default => "default",
            StructuralCapacityNoiseProfile::SkillAdmission => "skill-admission",
        }
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub old_baseline: StructuralCapacityOldBaseline,
    pub workload: StructuralCapacityWorkload,
    pub memory: StructuralCapacityMemory,
    pub metrics: StructuralCapacityMetrics,
    pub gates: StructuralCapacityGates,
    pub lens_admission: StructuralCapacityLensAdmission,
    pub claim_boundary: StructuralCapacityClaimBoundary,
    pub seed_results: Vec<StructuralCapacitySeedReport>,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityOldBaseline {
    pub source: &'static str,
    pub robust_patterns: usize,
    pub turbo_cells: usize,
    pub required_new_patterns: usize,
    pub required_lift_factor: f32,
    pub read_as: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityWorkload {
    pub noise_profile: &'static str,
    pub noise_pressure: &'static str,
    pub anti_wave_trap_policy: &'static str,
    pub pattern_shape: &'static str,
    pub patterns: usize,
    pub fixed_pattern_count: bool,
    pub fixed_pattern_shape: bool,
    pub smaller_pattern_modes_available: bool,
    pub smaller_pattern_shapes_available: bool,
    pub requested_seeds: usize,
    pub requested_noise_edges_per_noisy_case: usize,
    pub seeds: usize,
    pub seed_start: u64,
    pub edges_per_pattern: usize,
    pub active_facts: usize,
    pub clean_cases: usize,
    pub noisy_cases: usize,
    pub cold_cases: usize,
    pub role_swap_cases: usize,
    pub route_splice_cases: usize,
    pub conflict_cases: usize,
    pub missing_edge_cases: usize,
    pub noise_edges_per_noisy_case: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityMemory {
    pub schema_key: &'static str,
    pub schema_records: usize,
    pub residual_records: usize,
    pub fallback_records: usize,
    pub anti_lane_records: usize,
    pub direct_fixed64_bytes: usize,
    pub schema_bytes: usize,
    pub residual_bytes: usize,
    pub pattern_index_bytes: usize,
    pub anti_lane_bytes: usize,
    pub hot_bytes: usize,
    pub hot_budget_bytes: usize,
    pub fits_hot_budget: bool,
    pub residual_saving_bytes: isize,
    pub residual_saving_ratio: f32,
    pub schema_reuse_ratio: f32,
    pub bytes_per_useful_pattern: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityMetrics {
    pub total_cases: usize,
    pub positive_cases: usize,
    pub negative_cases: usize,
    pub clean_retrieval_pass_rate: f32,
    pub noisy_retrieval_pass_rate: f32,
    pub cold_rejection_pass_rate: f32,
    pub role_swap_rejection_pass_rate: f32,
    pub route_splice_rejection_pass_rate: f32,
    pub conflict_rejection_pass_rate: f32,
    pub missing_edge_rejection_pass_rate: f32,
    pub false_accept_rate: f32,
    pub false_negative_rate: f32,
    pub min_clean_margin: i64,
    pub min_noisy_margin: i64,
    pub max_negative_score: i64,
    pub seed_robustness_pass_rate: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityGates {
    pub fixed_1024_only: bool,
    pub skill_admission_noise_profile: bool,
    pub skill_admission_noise_pressure: bool,
    pub single_peak_under_noise: bool,
    pub field_core_lens_admission: bool,
    pub clean_retrieval: bool,
    pub noisy_retrieval: bool,
    pub cold_rejection: bool,
    pub role_swap_rejection: bool,
    pub route_splice_rejection: bool,
    pub conflict_rejection: bool,
    pub missing_edge_rejection: bool,
    pub anti_wave_traps_reject_false_peaks: bool,
    pub false_accept_rate_zero: bool,
    pub false_negative_rate_zero: bool,
    pub schema_residual_reuse: bool,
    pub hot_budget: bool,
    pub seed_robust: bool,
    pub old_baseline_beaten: bool,
    pub pattern16_macro_cell: bool,
    pub final_gate_passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityLensAdmission {
    pub version: &'static str,
    pub field_core_version: &'static str,
    pub field_pass_version: &'static str,
    pub uses_existing_field_core_lens: bool,
    pub uses_existing_field_core_anti_wave: bool,
    pub lens_chain: Vec<&'static str>,
    pub anti_wave_lanes: Vec<&'static str>,
    pub field_pass_verdict: String,
    pub field_pass_peak_target: String,
    pub field_pass_peak_state: String,
    pub field_pass_coherence_state: String,
    pub field_pass_safe_to_answer: bool,
    pub claim_boundary_preserved: bool,
    pub single_peak_confirmed: bool,
    pub anti_wave_local_false_peak_blockers: bool,
    pub accepted_for_skill_admission: bool,
    pub read_as: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacityClaimBoundary {
    pub structural_capacity_1024_ready: bool,
    pub old_baseline_beaten: bool,
    pub synthetic_structural_patterns_only: bool,
    pub pattern16_macro_cells: bool,
    pub smaller_pattern_modes_available: bool,
    pub smaller_pattern_shapes_available: bool,
    pub broad_chat_llm_ready: bool,
    pub global_nonlinear_memory_proven: bool,
    pub hardware_cache_residency_counter_proven: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct StructuralCapacitySeedReport {
    pub seed: u64,
    pub patterns: usize,
    pub clean_passed: usize,
    pub noisy_passed: usize,
    pub cold_rejected: usize,
    pub role_swap_rejected: usize,
    pub route_splice_rejected: usize,
    pub conflict_rejected: usize,
    pub missing_edge_rejected: usize,
    pub false_accepts: usize,
    pub false_negatives: usize,
    pub min_clean_margin: i64,
    pub min_noisy_margin: i64,
    pub max_negative_score: i64,
    pub passed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct CapacityEdge {
    slot: u8,
    subject: u64,
    relation: u64,
    object: u64,
}

#[derive(Clone)]
struct CapacityPattern {
    edges: [CapacityEdge; EDGES_PER_PATTERN],
}

#[derive(Clone)]
struct CapacityField {
    patterns: Vec<CapacityPattern>,
}

#[derive(Clone, Copy)]
struct PatternScore {
    index: usize,
    score: i64,
    correct_edges: usize,
    reverse_edges: usize,
    conflicts: usize,
}

#[derive(Clone, Copy)]
struct FieldDecision {
    accepted: bool,
    pattern_index: Option<usize>,
    top_score: i64,
    margin: i64,
}

#[derive(Default)]
struct SeedAccumulator {
    clean_passed: usize,
    noisy_passed: usize,
    cold_rejected: usize,
    role_swap_rejected: usize,
    route_splice_rejected: usize,
    conflict_rejected: usize,
    missing_edge_rejected: usize,
    false_accepts: usize,
    false_negatives: usize,
    min_clean_margin: Option<i64>,
    min_noisy_margin: Option<i64>,
    max_negative_score: Option<i64>,
}

pub(crate) fn build_structural_capacity_report(
    config: StructuralCapacityConfig,
) -> StructuralCapacityReport {
    let requested_seeds = config.seeds.max(1);
    let requested_noise_edges = config.noise_edges;
    let seeds = match config.noise_profile {
        StructuralCapacityNoiseProfile::Default => requested_seeds,
        StructuralCapacityNoiseProfile::SkillAdmission => {
            requested_seeds.max(SKILL_ADMISSION_MIN_SEEDS)
        }
    };
    let noise_edges = match config.noise_profile {
        StructuralCapacityNoiseProfile::Default => requested_noise_edges,
        StructuralCapacityNoiseProfile::SkillAdmission => {
            requested_noise_edges.max(SKILL_ADMISSION_NOISE_EDGES)
        }
    };
    let hot_budget_bytes = if config.hot_budget_bytes == 0 {
        DEFAULT_HOT_BUDGET_BYTES
    } else {
        config.hot_budget_bytes
    };
    let seed_results = (0..seeds)
        .map(|offset| {
            evaluate_seed(
                STRUCTURAL_PATTERN_CAPACITY,
                config.seed.wrapping_add(offset as u64),
                noise_edges,
            )
        })
        .collect::<Vec<_>>();
    let workload = workload(
        config.noise_profile,
        requested_seeds,
        requested_noise_edges,
        seeds,
        config.seed,
        noise_edges,
    );
    let memory = memory(hot_budget_bytes);
    let metrics = aggregate_metrics(&seed_results, &workload);
    let lens_admission = lens_admission(&metrics);
    let gates = gates(&workload, &metrics, &memory, &lens_admission);
    let verdict = if gates.final_gate_passed {
        "STRUCTURAL_CAPACITY_1024_PATTERN16_BASELINE_BEATEN"
    } else {
        "STRUCTURAL_CAPACITY_1024_PATTERN16_REVIEW"
    };

    StructuralCapacityReport {
        mode: "llmwave-big-structural-capacity-1024-pattern16",
        version: STRUCTURAL_CAPACITY_VERSION,
        verdict,
        old_baseline: old_baseline(),
        workload,
        memory,
        metrics,
        gates: gates.clone(),
        lens_admission,
        claim_boundary: claim_boundary(verdict),
        seed_results,
    }
}

fn evaluate_seed(patterns: usize, seed: u64, noise_edges: usize) -> StructuralCapacitySeedReport {
    debug_assert_eq!(patterns, STRUCTURAL_PATTERN_CAPACITY);
    let field = CapacityField {
        patterns: (0..patterns)
            .map(|index| pattern(seed, index))
            .collect::<Vec<_>>(),
    };
    let mut acc = SeedAccumulator::default();
    for index in 0..patterns {
        let clean = decide(&field, &field.patterns[index].edges);
        if clean.accepted && clean.pattern_index == Some(index) {
            acc.clean_passed += 1;
        } else {
            acc.false_negatives += 1;
        }
        acc.min_clean_margin = Some(
            acc.min_clean_margin
                .map_or(clean.margin, |value| value.min(clean.margin)),
        );

        let noisy_edges = noisy_query_edges(&field, index, noise_edges);
        let noisy = decide(&field, &noisy_edges);
        if noisy.accepted && noisy.pattern_index == Some(index) {
            acc.noisy_passed += 1;
        } else {
            acc.false_negatives += 1;
        }
        acc.min_noisy_margin = Some(
            acc.min_noisy_margin
                .map_or(noisy.margin, |value| value.min(noisy.margin)),
        );

        let cold_edges = pattern(seed, patterns + index + 97).edges;
        add_negative_result(&mut acc, decide(&field, &cold_edges), |acc| {
            acc.cold_rejected += 1;
        });

        let role_swap_edges = role_swap_query_edges(&field.patterns[index]);
        add_negative_result(&mut acc, decide(&field, &role_swap_edges), |acc| {
            acc.role_swap_rejected += 1;
        });

        let route_splice_edges = route_splice_query_edges(&field, index);
        add_negative_result(&mut acc, decide(&field, &route_splice_edges), |acc| {
            acc.route_splice_rejected += 1;
        });

        let conflict_edges = conflict_query_edges(&field, seed, index);
        add_negative_result(&mut acc, decide(&field, &conflict_edges), |acc| {
            acc.conflict_rejected += 1;
        });

        let missing_edges = missing_edge_query_edges(&field, index);
        add_negative_result(&mut acc, decide(&field, &missing_edges), |acc| {
            acc.missing_edge_rejected += 1;
        });
    }
    let passed = acc.clean_passed == patterns
        && acc.noisy_passed == patterns
        && acc.cold_rejected == patterns
        && acc.role_swap_rejected == patterns
        && acc.route_splice_rejected == patterns
        && acc.conflict_rejected == patterns
        && acc.missing_edge_rejected == patterns
        && acc.false_accepts == 0
        && acc.false_negatives == 0;

    StructuralCapacitySeedReport {
        seed,
        patterns,
        clean_passed: acc.clean_passed,
        noisy_passed: acc.noisy_passed,
        cold_rejected: acc.cold_rejected,
        role_swap_rejected: acc.role_swap_rejected,
        route_splice_rejected: acc.route_splice_rejected,
        conflict_rejected: acc.conflict_rejected,
        missing_edge_rejected: acc.missing_edge_rejected,
        false_accepts: acc.false_accepts,
        false_negatives: acc.false_negatives,
        min_clean_margin: acc.min_clean_margin.unwrap_or(0),
        min_noisy_margin: acc.min_noisy_margin.unwrap_or(0),
        max_negative_score: acc.max_negative_score.unwrap_or(0),
        passed,
    }
}

fn add_negative_result(
    acc: &mut SeedAccumulator,
    decision: FieldDecision,
    pass_counter: impl FnOnce(&mut SeedAccumulator),
) {
    acc.max_negative_score = Some(
        acc.max_negative_score
            .map_or(decision.top_score, |value| value.max(decision.top_score)),
    );
    if decision.accepted {
        acc.false_accepts += 1;
    } else {
        pass_counter(acc);
    }
}

fn decide(field: &CapacityField, query_edges: &[CapacityEdge]) -> FieldDecision {
    let mut top = PatternScore {
        index: 0,
        score: i64::MIN,
        correct_edges: 0,
        reverse_edges: 0,
        conflicts: 0,
    };
    let mut second_score = i64::MIN;
    for (index, pattern) in field.patterns.iter().enumerate() {
        let score = score_pattern(index, pattern, query_edges);
        if score.score > top.score {
            second_score = top.score;
            top = score;
        } else if score.score > second_score {
            second_score = score.score;
        }
    }
    let second_score = second_score.max(0);
    let top_score = top.score.max(0);
    let margin = top_score - second_score;
    let accepted = top.correct_edges == EDGES_PER_PATTERN
        && top.reverse_edges == 0
        && top.conflicts == 0
        && top_score >= (EDGES_PER_PATTERN as i64 * 100)
        && margin >= 100;
    FieldDecision {
        accepted,
        pattern_index: accepted.then_some(top.index),
        top_score,
        margin,
    }
}

fn score_pattern(
    index: usize,
    pattern: &CapacityPattern,
    query_edges: &[CapacityEdge],
) -> PatternScore {
    let mut correct_edges = 0usize;
    let mut reverse_edges = 0usize;
    let mut conflicts = 0usize;
    for query in query_edges {
        let slot = usize::from(query.slot);
        if slot >= EDGES_PER_PATTERN {
            continue;
        }
        let edge = pattern.edges[slot];
        if *query == edge {
            correct_edges += 1;
        } else if query.subject == edge.object
            && query.relation == edge.relation
            && query.object == edge.subject
        {
            reverse_edges += 1;
        } else if query.relation == edge.relation
            && ((query.subject == edge.subject && query.object != edge.object)
                || (query.object == edge.object && query.subject != edge.subject))
        {
            conflicts += 1;
        }
    }
    PatternScore {
        index,
        score: correct_edges as i64 * 100 - reverse_edges as i64 * 120 - conflicts as i64 * 80,
        correct_edges,
        reverse_edges,
        conflicts,
    }
}

fn noisy_query_edges(field: &CapacityField, index: usize, noise_edges: usize) -> Vec<CapacityEdge> {
    let mut edges = field.patterns[index].edges.to_vec();
    for offset in 0..noise_edges {
        let foreign = (index + offset + 1) % field.patterns.len();
        edges.push(field.patterns[foreign].edges[(offset + 1) % EDGES_PER_PATTERN]);
    }
    edges
}

fn role_swap_query_edges(pattern: &CapacityPattern) -> Vec<CapacityEdge> {
    pattern
        .edges
        .iter()
        .map(|edge| CapacityEdge {
            slot: edge.slot,
            subject: edge.object,
            relation: edge.relation,
            object: edge.subject,
        })
        .collect()
}

fn route_splice_query_edges(field: &CapacityField, index: usize) -> Vec<CapacityEdge> {
    let foreign = (index + 1) % field.patterns.len();
    let mut edges = Vec::with_capacity(EDGES_PER_PATTERN);
    edges.extend_from_slice(&field.patterns[index].edges[..EDGES_PER_PATTERN / 2]);
    edges.extend_from_slice(&field.patterns[foreign].edges[EDGES_PER_PATTERN / 2..]);
    edges
}

fn conflict_query_edges(field: &CapacityField, seed: u64, index: usize) -> Vec<CapacityEdge> {
    let mut edges = field.patterns[index].edges.to_vec();
    edges.push(CapacityEdge {
        slot: 0,
        subject: entity_hash(seed, "supplier_conflict", index + 11),
        relation: relation_hash("issues_invoice"),
        object: field.patterns[index].edges[0].object,
    });
    edges
}

fn missing_edge_query_edges(field: &CapacityField, index: usize) -> Vec<CapacityEdge> {
    field.patterns[index].edges[..EDGES_PER_PATTERN - 1].to_vec()
}

fn pattern(seed: u64, index: usize) -> CapacityPattern {
    let supplier = entity_hash(seed, "supplier", index);
    let invoice = entity_hash(seed, "invoice", index);
    let product = entity_hash(seed, "product", index);
    let buyer = entity_hash(seed, "buyer", index);
    let certificate = entity_hash(seed, "certificate", index);
    let manufacturer = entity_hash(seed, "manufacturer", index);
    let site = entity_hash(seed, "site", index);
    let shipment = entity_hash(seed, "shipment", index);
    let carrier = entity_hash(seed, "carrier", index);
    let warehouse = entity_hash(seed, "warehouse", index);
    let payment = entity_hash(seed, "payment", index);
    let contract = entity_hash(seed, "contract", index);
    let obligation = entity_hash(seed, "obligation", index);
    let evidence = entity_hash(seed, "evidence", index);
    CapacityPattern {
        edges: [
            edge(0, supplier, "issues_invoice", invoice),
            edge(1, invoice, "covers_product", product),
            edge(2, product, "ships_to_buyer", buyer),
            edge(3, certificate, "belongs_to_manufacturer", manufacturer),
            edge(4, manufacturer, "operates_site", site),
            edge(5, site, "authorized_for_product", product),
            edge(6, carrier, "moves_shipment", shipment),
            edge(7, shipment, "contains_product", product),
            edge(8, shipment, "arrives_at_warehouse", warehouse),
            edge(9, warehouse, "releases_to_buyer", buyer),
            edge(10, buyer, "pays_invoice_with", payment),
            edge(11, payment, "settles_invoice", invoice),
            edge(12, contract, "governs_invoice", invoice),
            edge(13, contract, "defines_obligation", obligation),
            edge(14, obligation, "assigns_responsibility_to", supplier),
            edge(15, evidence, "supports_certificate", certificate),
        ],
    }
}

fn edge(slot: u8, subject: u64, relation: &str, object: u64) -> CapacityEdge {
    CapacityEdge {
        slot,
        subject,
        relation: relation_hash(relation),
        object,
    }
}

fn workload(
    noise_profile: StructuralCapacityNoiseProfile,
    requested_seeds: usize,
    requested_noise_edges: usize,
    seeds: usize,
    seed_start: u64,
    noise_edges: usize,
) -> StructuralCapacityWorkload {
    StructuralCapacityWorkload {
        noise_profile: noise_profile.as_str(),
        noise_pressure: "positive noisy queries keep the full Pattern16 and add foreign edges as interference pressure",
        anti_wave_trap_policy: "cold, role-swap, 8/8 route-splice, conflict, and missing-edge traps must stay rejected",
        pattern_shape: "Pattern16 macro-cell",
        patterns: STRUCTURAL_PATTERN_CAPACITY,
        fixed_pattern_count: true,
        fixed_pattern_shape: true,
        smaller_pattern_modes_available: false,
        smaller_pattern_shapes_available: false,
        requested_seeds,
        requested_noise_edges_per_noisy_case: requested_noise_edges,
        seeds,
        seed_start,
        edges_per_pattern: EDGES_PER_PATTERN,
        active_facts: STRUCTURAL_PATTERN_CAPACITY * EDGES_PER_PATTERN,
        clean_cases: STRUCTURAL_PATTERN_CAPACITY * seeds,
        noisy_cases: STRUCTURAL_PATTERN_CAPACITY * seeds,
        cold_cases: STRUCTURAL_PATTERN_CAPACITY * seeds,
        role_swap_cases: STRUCTURAL_PATTERN_CAPACITY * seeds,
        route_splice_cases: STRUCTURAL_PATTERN_CAPACITY * seeds,
        conflict_cases: STRUCTURAL_PATTERN_CAPACITY * seeds,
        missing_edge_cases: STRUCTURAL_PATTERN_CAPACITY * seeds,
        noise_edges_per_noisy_case: noise_edges,
    }
}

fn memory(hot_budget_bytes: usize) -> StructuralCapacityMemory {
    let active_facts = STRUCTURAL_PATTERN_CAPACITY * EDGES_PER_PATTERN;
    let schema_records = EDGES_PER_PATTERN;
    let residual_records = active_facts;
    let fallback_records = 0usize;
    let anti_lane_records = STRUCTURAL_PATTERN_CAPACITY * 3;
    let direct_fixed64_bytes = active_facts * DIRECT_FIXED64_BYTES;
    let schema_bytes = schema_records * SCHEMA_RECORD_BYTES;
    let residual_bytes = residual_records * RESIDUAL_RECORD_BYTES;
    let pattern_index_bytes = STRUCTURAL_PATTERN_CAPACITY * PATTERN_INDEX_RECORD_BYTES;
    let anti_lane_bytes = anti_lane_records * ANTI_LANE_RECORD_BYTES;
    let hot_bytes = schema_bytes + residual_bytes + pattern_index_bytes + anti_lane_bytes;
    let residual_saving_bytes = direct_fixed64_bytes as isize - hot_bytes as isize;
    StructuralCapacityMemory {
        schema_key: "edge-role-schema + per-pattern residual entity hashes",
        schema_records,
        residual_records,
        fallback_records,
        anti_lane_records,
        direct_fixed64_bytes,
        schema_bytes,
        residual_bytes,
        pattern_index_bytes,
        anti_lane_bytes,
        hot_bytes,
        hot_budget_bytes,
        fits_hot_budget: hot_bytes <= hot_budget_bytes,
        residual_saving_bytes,
        residual_saving_ratio: ratio(residual_saving_bytes.max(0) as usize, direct_fixed64_bytes),
        schema_reuse_ratio: ratio(residual_records, active_facts),
        bytes_per_useful_pattern: round4(hot_bytes as f32 / STRUCTURAL_PATTERN_CAPACITY as f32),
    }
}

fn aggregate_metrics(
    seed_results: &[StructuralCapacitySeedReport],
    workload: &StructuralCapacityWorkload,
) -> StructuralCapacityMetrics {
    let clean_passed = seed_results
        .iter()
        .map(|seed| seed.clean_passed)
        .sum::<usize>();
    let noisy_passed = seed_results
        .iter()
        .map(|seed| seed.noisy_passed)
        .sum::<usize>();
    let cold_rejected = seed_results
        .iter()
        .map(|seed| seed.cold_rejected)
        .sum::<usize>();
    let role_swap_rejected = seed_results
        .iter()
        .map(|seed| seed.role_swap_rejected)
        .sum::<usize>();
    let route_splice_rejected = seed_results
        .iter()
        .map(|seed| seed.route_splice_rejected)
        .sum::<usize>();
    let conflict_rejected = seed_results
        .iter()
        .map(|seed| seed.conflict_rejected)
        .sum::<usize>();
    let missing_edge_rejected = seed_results
        .iter()
        .map(|seed| seed.missing_edge_rejected)
        .sum::<usize>();
    let false_accepts = seed_results
        .iter()
        .map(|seed| seed.false_accepts)
        .sum::<usize>();
    let false_negatives = seed_results
        .iter()
        .map(|seed| seed.false_negatives)
        .sum::<usize>();
    let negative_cases = workload.cold_cases
        + workload.role_swap_cases
        + workload.route_splice_cases
        + workload.conflict_cases
        + workload.missing_edge_cases;
    let positive_cases = workload.clean_cases + workload.noisy_cases;
    let seed_passed = seed_results.iter().filter(|seed| seed.passed).count();
    StructuralCapacityMetrics {
        total_cases: positive_cases + negative_cases,
        positive_cases,
        negative_cases,
        clean_retrieval_pass_rate: ratio(clean_passed, workload.clean_cases),
        noisy_retrieval_pass_rate: ratio(noisy_passed, workload.noisy_cases),
        cold_rejection_pass_rate: ratio(cold_rejected, workload.cold_cases),
        role_swap_rejection_pass_rate: ratio(role_swap_rejected, workload.role_swap_cases),
        route_splice_rejection_pass_rate: ratio(route_splice_rejected, workload.route_splice_cases),
        conflict_rejection_pass_rate: ratio(conflict_rejected, workload.conflict_cases),
        missing_edge_rejection_pass_rate: ratio(missing_edge_rejected, workload.missing_edge_cases),
        false_accept_rate: ratio(false_accepts, negative_cases),
        false_negative_rate: ratio(false_negatives, positive_cases),
        min_clean_margin: seed_results
            .iter()
            .map(|seed| seed.min_clean_margin)
            .min()
            .unwrap_or(0),
        min_noisy_margin: seed_results
            .iter()
            .map(|seed| seed.min_noisy_margin)
            .min()
            .unwrap_or(0),
        max_negative_score: seed_results
            .iter()
            .map(|seed| seed.max_negative_score)
            .max()
            .unwrap_or(0),
        seed_robustness_pass_rate: ratio(seed_passed, seed_results.len()),
    }
}

fn gates(
    workload: &StructuralCapacityWorkload,
    metrics: &StructuralCapacityMetrics,
    memory: &StructuralCapacityMemory,
    lens_admission: &StructuralCapacityLensAdmission,
) -> StructuralCapacityGates {
    let skill_admission_noise_profile = workload.noise_profile == "skill-admission";
    let skill_admission_noise_pressure = !skill_admission_noise_profile
        || (workload.seeds >= SKILL_ADMISSION_MIN_SEEDS
            && workload.noise_edges_per_noisy_case >= SKILL_ADMISSION_NOISE_EDGES);
    let single_peak_under_noise = metrics.min_noisy_margin >= SINGLE_PEAK_NOISE_MARGIN_MIN;
    let clean_retrieval = metrics.clean_retrieval_pass_rate == 1.0;
    let noisy_retrieval = metrics.noisy_retrieval_pass_rate == 1.0;
    let cold_rejection = metrics.cold_rejection_pass_rate == 1.0;
    let role_swap_rejection = metrics.role_swap_rejection_pass_rate == 1.0;
    let route_splice_rejection = metrics.route_splice_rejection_pass_rate == 1.0;
    let conflict_rejection = metrics.conflict_rejection_pass_rate == 1.0;
    let missing_edge_rejection = metrics.missing_edge_rejection_pass_rate == 1.0;
    let false_accept_rate_zero = metrics.false_accept_rate == 0.0;
    let anti_wave_traps_reject_false_peaks = cold_rejection
        && role_swap_rejection
        && route_splice_rejection
        && conflict_rejection
        && missing_edge_rejection
        && false_accept_rate_zero;
    let false_negative_rate_zero = metrics.false_negative_rate == 0.0;
    let schema_residual_reuse =
        memory.residual_saving_bytes > 0 && memory.schema_reuse_ratio == 1.0;
    let seed_robust = metrics.seed_robustness_pass_rate == 1.0;
    let old_baseline_beaten = STRUCTURAL_PATTERN_CAPACITY > OLD_ROBUST_BASELINE_PATTERNS;
    let pattern16_macro_cell = EDGES_PER_PATTERN == 16;
    let field_core_lens_admission = lens_admission.accepted_for_skill_admission;
    let final_gate_passed = clean_retrieval
        && noisy_retrieval
        && skill_admission_noise_pressure
        && single_peak_under_noise
        && field_core_lens_admission
        && cold_rejection
        && role_swap_rejection
        && route_splice_rejection
        && conflict_rejection
        && missing_edge_rejection
        && anti_wave_traps_reject_false_peaks
        && false_accept_rate_zero
        && false_negative_rate_zero
        && schema_residual_reuse
        && memory.fits_hot_budget
        && seed_robust
        && old_baseline_beaten
        && pattern16_macro_cell;
    StructuralCapacityGates {
        fixed_1024_only: true,
        skill_admission_noise_profile,
        skill_admission_noise_pressure,
        single_peak_under_noise,
        field_core_lens_admission,
        clean_retrieval,
        noisy_retrieval,
        cold_rejection,
        role_swap_rejection,
        route_splice_rejection,
        conflict_rejection,
        missing_edge_rejection,
        anti_wave_traps_reject_false_peaks,
        false_accept_rate_zero,
        false_negative_rate_zero,
        schema_residual_reuse,
        hot_budget: memory.fits_hot_budget,
        seed_robust,
        old_baseline_beaten,
        pattern16_macro_cell,
        final_gate_passed,
    }
}

fn lens_admission(metrics: &StructuralCapacityMetrics) -> StructuralCapacityLensAdmission {
    let lens_chain = vec!["route", "role", "evidence"];
    let anti_wave_lanes = vec![
        "role-swap-false-peak",
        "route-splice-false-peak",
        "conflict-false-peak",
        "missing-edge-false-peak",
        "cold-false-peak",
    ];
    let pass = run_pattern16_field_pass(&lens_chain, &anti_wave_lanes);
    let claim_boundary_preserved = !pass.safe_to_answer
        && pass.claim_boundary.read_only_projection
        && pass.claim_boundary.no_behavior_change
        && pass.claim_boundary.not_llm_ready
        && pass.claim_boundary.not_nonlinear_memory_proof;
    let single_peak_confirmed =
        pass.peak.target == "pattern16-structural-capacity" && pass.peak.margin >= 0.04;
    let anti_wave_local_false_peak_blockers =
        pass.anti_wave_count == anti_wave_lanes.len() && pass.verdict != "VETO";
    let accepted_for_skill_admission = single_peak_confirmed
        && anti_wave_local_false_peak_blockers
        && claim_boundary_preserved
        && metrics.min_noisy_margin >= SINGLE_PEAK_NOISE_MARGIN_MIN;

    StructuralCapacityLensAdmission {
        version: "pattern16-field-core-lens-admission-v1",
        field_core_version: FIELD_CORE_VERSION,
        field_pass_version: FIELD_PASS_VERSION,
        uses_existing_field_core_lens: true,
        uses_existing_field_core_anti_wave: true,
        lens_chain,
        anti_wave_lanes,
        field_pass_verdict: pass.verdict,
        field_pass_peak_target: pass.peak.target,
        field_pass_peak_state: pass.peak.state,
        field_pass_coherence_state: pass.coherence_state,
        field_pass_safe_to_answer: pass.safe_to_answer,
        claim_boundary_preserved,
        single_peak_confirmed,
        anti_wave_local_false_peak_blockers,
        accepted_for_skill_admission,
        read_as: "Pattern16 admission reuses field_core lens -> anti-wave -> peak pass. It is an admission readout only and preserves the non-LLM/nonlinear claim boundary.",
    }
}

fn run_pattern16_field_pass(
    lens_chain: &[&'static str],
    anti_wave_lanes: &[&'static str],
) -> FieldPassReport {
    let lenses = lens_chain
        .iter()
        .map(|name| FieldLensOperation {
            kind: match *name {
                "route" => FieldLensKind::Route,
                "role" => FieldLensKind::Role,
                "evidence" => FieldLensKind::Evidence,
                _ => FieldLensKind::Group,
            },
            label: format!("pattern16-{name}"),
            strength: 1,
        })
        .collect::<Vec<_>>();
    let anti_waves = anti_wave_lanes
        .iter()
        .map(|lane| FieldAntiWaveLane {
            id: (*lane).to_string(),
            target: (*lane).to_string(),
            subject: "pattern16".to_string(),
            relation: "suppresses_false_peak".to_string(),
            object: (*lane).to_string(),
            route: Some((*lane).to_string()),
            group: Some("pattern16-admission".to_string()),
            strength: 2,
        })
        .collect::<Vec<_>>();
    let query = FieldRecord::synthetic(
        "pattern16-query",
        FieldRecordKind::StructuralTriad,
        "pattern16",
        "admits",
        "skill-core",
        Some("pattern16-structural-capacity".to_string()),
        Some("pattern16-admission".to_string()),
    );
    let records = vec![
        FieldRecord::synthetic(
            "pattern16-true-peak",
            FieldRecordKind::StructuralTriad,
            "pattern16",
            "admits",
            "skill-core",
            Some("pattern16-structural-capacity".to_string()),
            Some("pattern16-admission".to_string()),
        ),
        FieldRecord::synthetic(
            "role-swap-false-peak",
            FieldRecordKind::StructuralTriad,
            "skill-core",
            "admits",
            "pattern16",
            Some("role-swap-false-peak".to_string()),
            Some("pattern16-admission".to_string()),
        ),
        FieldRecord::synthetic(
            "route-splice-false-peak",
            FieldRecordKind::StructuralTriad,
            "pattern16",
            "splices",
            "foreign-route",
            Some("route-splice-false-peak".to_string()),
            Some("pattern16-admission".to_string()),
        ),
        FieldRecord::synthetic(
            "missing-edge-false-peak",
            FieldRecordKind::StructuralTriad,
            "pattern16",
            "missing_edge",
            "partial-cell",
            Some("missing-edge-false-peak".to_string()),
            Some("pattern16-admission".to_string()),
        ),
    ];
    run_field_pass(&FieldPassInput {
        family: FieldFamily::Structural,
        query,
        records,
        lenses,
        anti_waves,
        state_hint: Some("FIELD_THIN".to_string()),
        claim_boundary: FieldClaimBoundary::default(),
    })
}

fn claim_boundary(verdict: &'static str) -> StructuralCapacityClaimBoundary {
    StructuralCapacityClaimBoundary {
        structural_capacity_1024_ready: verdict
            == "STRUCTURAL_CAPACITY_1024_PATTERN16_BASELINE_BEATEN",
        old_baseline_beaten: verdict == "STRUCTURAL_CAPACITY_1024_PATTERN16_BASELINE_BEATEN",
        synthetic_structural_patterns_only: true,
        pattern16_macro_cells: true,
        smaller_pattern_modes_available: false,
        smaller_pattern_shapes_available: false,
        broad_chat_llm_ready: false,
        global_nonlinear_memory_proven: false,
        hardware_cache_residency_counter_proven: false,
        safe_claim: "LLMWave Big now has a fixed 1024 Pattern16 structural-capacity gate: each macro-cell has 16 directed edges, clean/noisy retrieval must pass, and cold, role-swap, route-splice, conflict, and missing-edge traps must reject with zero false accepts. This is synthetic structural-capacity evidence, not broad chat or global nonlinear-memory proof.",
        blocked_claims: vec![
            "broad_chat_llm_ready",
            "global_nonlinear_memory_proven",
            "real_corpus_structural_capacity_proven",
            "hardware_cache_residency_counter_proven",
        ],
    }
}

fn old_baseline() -> StructuralCapacityOldBaseline {
    StructuralCapacityOldBaseline {
        source: "nando-wave CAPACITY-1 robust-pattern checkpoint",
        robust_patterns: OLD_ROBUST_BASELINE_PATTERNS,
        turbo_cells: OLD_TURBO_CELLS,
        required_new_patterns: STRUCTURAL_PATTERN_CAPACITY,
        required_lift_factor: round4(
            STRUCTURAL_PATTERN_CAPACITY as f32 / OLD_ROBUST_BASELINE_PATTERNS as f32,
        ),
        read_as: "The old accepted checkpoint was 128 robust wave patterns. This gate has no smaller runnable mode or smaller cell shape; it must prove 1024 Pattern16 structural macro-cells.",
    }
}

fn entity_hash(seed: u64, role: &str, index: usize) -> u64 {
    mix64(seed ^ relation_hash(role) ^ (index as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93))
}

fn relation_hash(value: &str) -> u64 {
    let mut hash = 0xCBF2_9CE4_8422_2325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    mix64(hash)
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        round4(part as f32 / total as f32)
    }
}

fn round4(value: f32) -> f32 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_capacity_is_fixed_to_1024_and_beats_old_baseline() {
        let report = build_structural_capacity_report(StructuralCapacityConfig {
            seed: 13,
            seeds: 2,
            noise_edges: 4,
            hot_budget_bytes: DEFAULT_HOT_BUDGET_BYTES,
            noise_profile: StructuralCapacityNoiseProfile::Default,
        });
        assert_eq!(report.workload.patterns, STRUCTURAL_PATTERN_CAPACITY);
        assert_eq!(report.workload.pattern_shape, "Pattern16 macro-cell");
        assert_eq!(report.workload.edges_per_pattern, 16);
        assert_eq!(report.workload.active_facts, 16_384);
        assert!(report.workload.fixed_pattern_count);
        assert!(report.workload.fixed_pattern_shape);
        assert!(!report.workload.smaller_pattern_modes_available);
        assert!(!report.workload.smaller_pattern_shapes_available);
        assert_eq!(
            report.verdict,
            "STRUCTURAL_CAPACITY_1024_PATTERN16_BASELINE_BEATEN"
        );
        assert!(report.gates.final_gate_passed);
        assert!(report.gates.old_baseline_beaten);
        assert!(report.gates.pattern16_macro_cell);
        assert!(report.gates.missing_edge_rejection);
        assert!(report.gates.field_core_lens_admission);
        assert!(report.lens_admission.uses_existing_field_core_lens);
        assert!(report.lens_admission.uses_existing_field_core_anti_wave);
        assert_eq!(
            report.lens_admission.field_pass_peak_target,
            "pattern16-structural-capacity"
        );
        assert_eq!(report.lens_admission.field_pass_verdict, "WATCH");
        assert!(!report.lens_admission.field_pass_safe_to_answer);
        assert!(report.lens_admission.claim_boundary_preserved);
        assert_eq!(report.metrics.false_accept_rate, 0.0);
        assert_eq!(report.metrics.false_negative_rate, 0.0);
        assert!(report.memory.residual_saving_bytes > 0);
        assert!(report.memory.fits_hot_budget);
    }

    #[test]
    fn skill_admission_profile_forces_noise_pressure_and_single_peak() {
        let report = build_structural_capacity_report(StructuralCapacityConfig {
            seed: 13,
            seeds: 2,
            noise_edges: 4,
            hot_budget_bytes: DEFAULT_HOT_BUDGET_BYTES,
            noise_profile: StructuralCapacityNoiseProfile::SkillAdmission,
        });
        assert_eq!(report.workload.noise_profile, "skill-admission");
        assert_eq!(report.workload.requested_seeds, 2);
        assert_eq!(report.workload.requested_noise_edges_per_noisy_case, 4);
        assert_eq!(report.workload.seeds, 8);
        assert_eq!(report.workload.noise_edges_per_noisy_case, 16);
        assert!(report.gates.skill_admission_noise_profile);
        assert!(report.gates.skill_admission_noise_pressure);
        assert!(report.gates.single_peak_under_noise);
        assert!(report.gates.anti_wave_traps_reject_false_peaks);
        assert!(report.gates.field_core_lens_admission);
        assert!(report.lens_admission.accepted_for_skill_admission);
        assert!(report.lens_admission.single_peak_confirmed);
        assert!(report.lens_admission.anti_wave_local_false_peak_blockers);
        assert_eq!(
            report.verdict,
            "STRUCTURAL_CAPACITY_1024_PATTERN16_BASELINE_BEATEN"
        );
    }
}
