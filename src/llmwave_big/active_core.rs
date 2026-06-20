//! Hot active-packet runtime boundary for future LLMWave-Big work.

use serde::Serialize;

use super::nanda_6m;

pub(crate) const ACTIVE_CORE_VERSION: &str = "llmwave-big-v180-active-core-contract";
pub(crate) const ACTIVE_HEADER_BYTES: usize = 262_144;
pub(crate) const ACTIVE_SYMBOL_BYTES: usize = 524_288;
pub(crate) const ACTIVE_OPERATOR_BYTES: usize = 262_144;
pub(crate) const ACTIVE_SCHEMA_BYTES: usize = 1_310_720;
pub(crate) const ACTIVE_RESIDUAL_BYTES: usize = 1_310_720;
pub(crate) const ACTIVE_CENTROID_BYTES: usize = 1_048_576;
pub(crate) const ACTIVE_LANE_BYTES: usize = 524_288;
pub(crate) const ACTIVE_WORKSPACE_BYTES: usize = 1_048_576;
pub(crate) const ACTIVE_TOTAL_BYTES: usize = ACTIVE_HEADER_BYTES
    + ACTIVE_SYMBOL_BYTES
    + ACTIVE_OPERATOR_BYTES
    + ACTIVE_SCHEMA_BYTES
    + ACTIVE_RESIDUAL_BYTES
    + ACTIVE_CENTROID_BYTES
    + ACTIVE_LANE_BYTES
    + ACTIVE_WORKSPACE_BYTES;

const ACTIVE_CORE_MIN_PEAK: i64 = 2_000;
const ACTIVE_CORE_MIN_MARGIN: i64 = 128;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ActivePacketHeader {
    pub symbol_count: u16,
    pub operator_count: u16,
    pub schema_count: u16,
    pub residual_count: u16,
    pub lane_count: u16,
    pub evidence_ref_count: u16,
    pub flags: u16,
    pub reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ActiveSymbol16 {
    pub id: u32,
    pub wave_seed: u32,
    pub kind: u8,
    pub projection_channel: u8,
    pub flags: u16,
    pub phase: u16,
    pub alias_root: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ActiveOperator16 {
    pub id: u16,
    pub phase: u16,
    pub inverse_id: u16,
    pub polarity_rules: u16,
    pub allowed_subject_mask: u16,
    pub allowed_object_mask: u16,
    pub anti_rule_mask: u16,
    pub flags: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ActiveSchema32 {
    pub id: u32,
    pub centroid_id: u32,
    pub wave_seed: u32,
    pub evidence_mask: u32,
    pub operator_id: u16,
    pub subject_role: u16,
    pub object_role: u16,
    pub route_id: u16,
    pub phase: u16,
    pub polarity: i16,
    pub confidence: u8,
    pub flags: u8,
    pub reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ActiveResidual32 {
    pub schema_id: u32,
    pub subject_id: u32,
    pub object_id: u32,
    pub evidence_ref: u32,
    pub subject_delta: i16,
    pub object_delta: i16,
    pub phase_delta: i16,
    pub evidence_bias: i16,
    pub lane_hint: u32,
    pub confidence: u8,
    pub flags: u8,
    pub reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ActiveLane64 {
    pub masks: [u64; 8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActiveWave1024 {
    pub values: [i16; nanda_6m::WAVE_DIM],
}

impl Default for ActiveWave1024 {
    fn default() -> Self {
        Self {
            values: [0; nanda_6m::WAVE_DIM],
        }
    }
}

impl ActiveWave1024 {
    pub(crate) fn add_seed(&mut self, seed: u32, weight: i16) {
        let mut state = seed;
        for value in &mut self.values {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let bit = ((state >> 31) as i16 * 2) - 1;
            *value = value.saturating_add(bit.saturating_mul(weight));
        }
    }

    pub(crate) fn dot(&self, other: &Self) -> i64 {
        let mut out = 0_i64;
        for (left, right) in self.values.iter().zip(other.values.iter()) {
            out += i64::from(*left) * i64::from(*right);
        }
        out
    }

    pub(crate) fn energy(&self) -> i64 {
        self.dot(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ActiveCoreCycle {
    pub verdict: &'static str,
    pub safe_to_answer: bool,
    pub top_schema_id: u32,
    pub top_score: i64,
    pub runner_up_score: i64,
    pub margin: i64,
    pub schema_energy: i64,
    pub residual_energy: i64,
    pub anti_veto_triggered: bool,
    pub reconstructed_evidence_ref: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct ActiveCoreReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub roadmap_block: &'static str,
    pub verdict: &'static str,
    pub hot_core_rules: Vec<&'static str>,
    pub packet_format: PacketFormatReport,
    pub budget: ActiveCoreBudgetReport,
    pub projections: ProjectionReport,
    pub loader_eval: LoaderEvalReport,
    pub focus_competition: Vec<&'static str>,
    pub runtime_ops: Vec<&'static str>,
    pub benchmark_command: &'static str,
    pub cycle: ActiveCoreCycle,
}

#[derive(Serialize, Clone)]
pub(crate) struct PacketFormatReport {
    pub header_bytes: usize,
    pub symbol_record_bytes: usize,
    pub operator_record_bytes: usize,
    pub schema_record_bytes: usize,
    pub residual_record_bytes: usize,
    pub lane_record_bytes: usize,
    pub requirements: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ActiveCoreBudgetReport {
    pub total_bytes: usize,
    pub nanda_6m_budget_bytes: usize,
    pub fits_nanda_6m_budget: bool,
    pub segments: Vec<BudgetSegment>,
}

#[derive(Serialize, Clone)]
pub(crate) struct BudgetSegment {
    pub name: &'static str,
    pub bytes: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct ProjectionReport {
    pub schema_wave: &'static str,
    pub residual_wave: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LoaderEvalReport {
    pub query: &'static str,
    pub must_lift: Vec<&'static str>,
    pub sample_lifted_operator: u16,
    pub sample_lifted_schema: u32,
    pub sample_lifted_residuals: Vec<u32>,
}

pub(crate) fn build_active_core_report() -> ActiveCoreReport {
    let cycle = run_sample_cycle();
    ActiveCoreReport {
        mode: "llmwave-big-active-core-contract",
        version: ACTIVE_CORE_VERSION,
        roadmap_block: "v171-v180",
        verdict: cycle.verdict,
        hot_core_rules: vec![
            "no_json_in_inner_loop",
            "no_strings_in_inner_loop",
            "no_heap_or_hashmap_in_inner_loop",
            "fixed_size_records",
            "explicit_active_packet_budget",
        ],
        packet_format: PacketFormatReport {
            header_bytes: core::mem::size_of::<ActivePacketHeader>(),
            symbol_record_bytes: core::mem::size_of::<ActiveSymbol16>(),
            operator_record_bytes: core::mem::size_of::<ActiveOperator16>(),
            schema_record_bytes: core::mem::size_of::<ActiveSchema32>(),
            residual_record_bytes: core::mem::size_of::<ActiveResidual32>(),
            lane_record_bytes: core::mem::size_of::<ActiveLane64>(),
            requirements: vec!["fits_l3", "fast_scan", "no_strings", "no_json"],
        },
        budget: ActiveCoreBudgetReport {
            total_bytes: ACTIVE_TOTAL_BYTES,
            nanda_6m_budget_bytes: nanda_6m::BUDGET_BYTES,
            fits_nanda_6m_budget: ACTIVE_TOTAL_BYTES <= nanda_6m::BUDGET_BYTES,
            segments: vec![
                segment("header", ACTIVE_HEADER_BYTES),
                segment("symbols", ACTIVE_SYMBOL_BYTES),
                segment("operators", ACTIVE_OPERATOR_BYTES),
                segment("schemas", ACTIVE_SCHEMA_BYTES),
                segment("residuals", ACTIVE_RESIDUAL_BYTES),
                segment("centroids", ACTIVE_CENTROID_BYTES),
                segment("lanes", ACTIVE_LANE_BYTES),
                segment("workspace", ACTIVE_WORKSPACE_BYTES),
            ],
        },
        projections: ProjectionReport {
            schema_wave:
                "operator_wave+subject_role_wave+object_role_wave+route_wave+phase+polarity",
            residual_wave: "schema_wave+subject_entity_delta+object_entity_delta+evidence_bias",
        },
        loader_eval: LoaderEvalReport {
            query: "who issued the invoice?",
            must_lift: vec![
                "issues_operator",
                "supplier_role",
                "invoice_schema",
                "relevant_residuals",
            ],
            sample_lifted_operator: 3,
            sample_lifted_schema: 101,
            sample_lifted_residuals: vec![1_001, 1_004],
        },
        focus_competition: vec![
            "schema_competition",
            "route_competition",
            "evidence_competition",
            "multi_axis_top_k_not_pure_similarity",
        ],
        runtime_ops: vec![
            "excite(query)",
            "settle()",
            "peak_detect()",
            "anti_veto()",
            "reconstruct()",
            "answer_plan()",
        ],
        benchmark_command: "nanda bench6m --mode active-core",
        cycle,
    }
}

pub(crate) fn run_sample_cycle() -> ActiveCoreCycle {
    let query = active_query_wave(3, 11, 21, 31);
    let schema_a = sample_schema(101, 3, 11, 21, 31, 0xC001_A101, 0);
    let schema_b = sample_schema(102, 4, 12, 22, 31, 0xC001_A102, 0);
    let residual = ActiveResidual32 {
        schema_id: 101,
        subject_id: 2_001,
        object_id: 3_001,
        evidence_ref: 10_001,
        subject_delta: 7,
        object_delta: -3,
        phase_delta: 17,
        evidence_bias: 5,
        lane_hint: 9_001,
        confidence: 94,
        flags: 0,
        reserved: 0,
    };
    let schema_wave_a = project_schema(schema_a);
    let schema_wave_b = project_schema(schema_b);
    let residual_wave = project_residual(&schema_wave_a, residual);
    let top_score = query.dot(&schema_wave_a);
    let runner_up_score = query.dot(&schema_wave_b);
    let margin = top_score - runner_up_score;
    let anti_veto_triggered = residual.phase_delta < -64;
    let verdict = if top_score >= ACTIVE_CORE_MIN_PEAK
        && margin >= ACTIVE_CORE_MIN_MARGIN
        && !anti_veto_triggered
    {
        "ACTIVE_CORE_READY"
    } else if anti_veto_triggered {
        "ACTIVE_CORE_CONTESTED"
    } else {
        "ACTIVE_CORE_THIN"
    };
    ActiveCoreCycle {
        verdict,
        safe_to_answer: verdict == "ACTIVE_CORE_READY",
        top_schema_id: schema_a.id,
        top_score,
        runner_up_score,
        margin,
        schema_energy: schema_wave_a.energy(),
        residual_energy: residual_wave.energy(),
        anti_veto_triggered,
        reconstructed_evidence_ref: residual.evidence_ref,
    }
}

pub(crate) fn bench_active_core(iterations: u64) -> ActiveCoreBench {
    let iterations = iterations.max(1);
    let started = std::time::Instant::now();
    let mut checksum = 0_i64;
    for _ in 0..iterations {
        let cycle = std::hint::black_box(run_sample_cycle());
        checksum = checksum
            .wrapping_add(cycle.top_score)
            .wrapping_add(cycle.margin);
    }
    let elapsed = started.elapsed();
    let elapsed_ns = elapsed.as_nanos() as f64;
    ActiveCoreBench {
        iterations,
        total_ns: elapsed.as_nanos(),
        ns_per_query: elapsed_ns / iterations as f64,
        queries_per_sec: (iterations as f64) * 1_000_000_000.0 / elapsed_ns.max(1.0),
        checksum,
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct ActiveCoreBench {
    pub iterations: u64,
    pub total_ns: u128,
    pub ns_per_query: f64,
    pub queries_per_sec: f64,
    pub checksum: i64,
}

fn sample_schema(
    id: u32,
    operator_id: u16,
    subject_role: u16,
    object_role: u16,
    route_id: u16,
    wave_seed: u32,
    polarity: i16,
) -> ActiveSchema32 {
    ActiveSchema32 {
        id,
        centroid_id: id + 10_000,
        wave_seed,
        evidence_mask: 1,
        operator_id,
        subject_role,
        object_role,
        route_id,
        phase: 0x1000 + operator_id,
        polarity,
        confidence: 94,
        flags: 0,
        reserved: 0,
    }
}

fn active_query_wave(
    operator_id: u16,
    subject_role: u16,
    object_role: u16,
    route_id: u16,
) -> ActiveWave1024 {
    let schema = sample_schema(
        900,
        operator_id,
        subject_role,
        object_role,
        route_id,
        0xC001_A101,
        0,
    );
    project_schema(schema)
}

fn project_schema(schema: ActiveSchema32) -> ActiveWave1024 {
    let mut wave = ActiveWave1024::default();
    wave.add_seed(u32::from(schema.operator_id) | 0x1000_0000, 5);
    wave.add_seed(u32::from(schema.subject_role) | 0x2000_0000, 3);
    wave.add_seed(u32::from(schema.object_role) | 0x3000_0000, 3);
    wave.add_seed(u32::from(schema.route_id) | 0x4000_0000, 2);
    wave.add_seed(u32::from(schema.phase) | 0x5000_0000, 1);
    wave.add_seed(schema.wave_seed, 1 + schema.polarity.signum());
    wave
}

fn project_residual(schema_wave: &ActiveWave1024, residual: ActiveResidual32) -> ActiveWave1024 {
    let mut wave = *schema_wave;
    wave.add_seed(residual.subject_id ^ 0xA000_0000, residual.subject_delta);
    wave.add_seed(residual.object_id ^ 0xB000_0000, residual.object_delta);
    wave.add_seed(residual.evidence_ref ^ 0xE000_0000, residual.evidence_bias);
    wave.add_seed(
        (i32::from(residual.phase_delta).unsigned_abs()) ^ 0xD000_0000,
        residual.phase_delta.signum(),
    );
    wave
}

fn segment(name: &'static str, bytes: usize) -> BudgetSegment {
    BudgetSegment { name, bytes }
}
