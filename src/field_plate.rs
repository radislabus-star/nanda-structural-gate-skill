use crate::{nanda_6m, OutputFormat, CORE_VERSION, EXIT_PASS, EXIT_VETO};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

const FIELD_PLATE_VERSION: &str = "nanda-field-plate-v1";
const REFERENCE_KERNEL: &str = "classic-1024-reference";
const EXACT_PROJECTION: &str = "classic-1024-exact";
const DEFAULT_QUERY_TRIADS: usize = 8;
const HEATMAP_BINS: usize = 64;

#[derive(Parser)]
pub(crate) struct FieldPlateArgs {
    #[command(subcommand)]
    command: FieldPlateCommand,
}

#[derive(Subcommand)]
enum FieldPlateCommand {
    Build(FieldPlateBuildArgs),
    Check(FieldPlateCheckArgs),
    Render(FieldPlateRenderArgs),
}

#[derive(Parser)]
struct FieldPlateBuildArgs {
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, default_value_t = nanda_6m::ACTIVE_FIELD_RECORDS)]
    records: usize,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct FieldPlateCheckArgs {
    #[arg(long)]
    plate: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Parser)]
struct FieldPlateRenderArgs {
    #[arg(long)]
    plate: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    format: OutputFormat,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct FieldPlateSnapshot {
    pub snapshot_version: String,
    pub mode: String,
    pub kernel: String,
    pub projection: String,
    pub core_version: String,
    pub nanda_6m_version: String,
    pub wave_dim: usize,
    pub records: usize,
    pub query_triads: usize,
    pub memory_hash: String,
    pub query_hash: String,
    pub dot_hash: String,
    pub route_accumulator_hash: String,
    pub group_accumulator_hash: String,
    pub proof_hash: String,
    pub field_signature: String,
    pub runtime_contract: PlateRuntimeContract,
    pub route_peak: PlatePeak,
    pub group_peak: PlatePeak,
    pub top_record: PlateTopRecord,
    pub proof: PlateProof,
    pub heatmap: Vec<PlateHeatBin>,
    pub claim_boundary: PlateClaimBoundary,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PlateRuntimeContract {
    pub full_active_scan: bool,
    pub proof_rescan: bool,
    pub no_per_record_score_arrays: bool,
    pub json_used_in_hot_loop: bool,
    pub heap_allocations_in_hot_loop: u8,
    pub workspace_required_bytes: usize,
    pub workspace_budget_bytes: usize,
    pub fits_l3: bool,
    pub workspace_fits: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct PlatePeak {
    pub top_id: u16,
    pub top_score: f64,
    pub top_score_bits: u64,
    pub second_id: u16,
    pub second_score: f64,
    pub second_score_bits: u64,
    pub margin: f64,
    pub margin_bits: u64,
    pub state: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PlateTopRecord {
    pub record_index: u16,
    pub route_id: u16,
    pub group_id: u16,
    pub dot: i64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PlateProof {
    pub records_scanned: u32,
    pub dot_records_scored: u32,
    pub dot_records_skipped: u32,
    pub route_considered: u16,
    pub route_support: u16,
    pub route_anti: u16,
    pub route_positive_dot: i64,
    pub route_negative_dot: i64,
    pub group_considered: u16,
    pub group_support: u16,
    pub group_anti: u16,
    pub group_positive_dot: i64,
    pub group_negative_dot: i64,
    pub lane_applied: usize,
    pub lane_improved: usize,
    pub lane_focused: usize,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PlateHeatBin {
    pub bin: usize,
    pub records: u32,
    pub positive_dot: i64,
    pub negative_dot: i64,
    pub net_dot: i64,
    pub max_dot: i64,
    pub min_dot: i64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PlateClaimBoundary {
    pub exact_kernel_snapshot: bool,
    pub visual_render_is_not_authority: bool,
    pub discovery_is_not_answer_authority: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub hardware_cache_residency_counter_proven: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FieldPlateComparison {
    pub mode: String,
    pub verdict: String,
    pub safe_to_use_kernel: bool,
    pub expected_signature: String,
    pub current_signature: String,
    pub exact_match: bool,
    pub mismatches: Vec<String>,
    pub checked_fields: Vec<String>,
    pub claim_boundary: PlateClaimBoundary,
}

#[derive(Clone, Copy, Debug, Default)]
struct PlateAccumulator {
    positive_dot: i64,
    negative_dot: i64,
    net_dot: i64,
    top_dot: i64,
    considered: u32,
    support_count: u32,
    anti_count: u32,
    id: u16,
    top_record: u16,
}

pub(crate) fn cmd(args: FieldPlateArgs) -> Result<u8> {
    match args.command {
        FieldPlateCommand::Build(args) => {
            let plate = build_plate(args.records)?;
            if let Some(path) = args.out.as_ref() {
                write_plate(path, &plate)?;
            }
            let out = json!({
                "mode": "field-plate-build",
                "verdict": "FIELD_PLATE_BUILT",
                "safe_to_use_kernel": true,
                "out": args.out.as_ref().map(|path| path.display().to_string()),
                "plate": plate
            });
            print_value(&out, &args.format)?;
            Ok(EXIT_PASS)
        }
        FieldPlateCommand::Check(args) => {
            let expected = read_plate(&args.plate)?;
            let current = build_plate(expected.records)?;
            let comparison = compare_plates(&expected, &current);
            let out = json!({
                "mode": "field-plate-check",
                "plate": args.plate.display().to_string(),
                "comparison": comparison,
                "expected": compact_plate_view(&expected),
                "current": compact_plate_view(&current)
            });
            let pass = comparison.exact_match;
            print_value(&out, &args.format)?;
            Ok(if pass { EXIT_PASS } else { EXIT_VETO })
        }
        FieldPlateCommand::Render(args) => {
            let plate = read_plate(&args.plate)?;
            let svg = render_plate_svg(&plate);
            fs::write(&args.out, svg)
                .with_context(|| format!("write field plate svg {}", args.out.display()))?;
            let out = json!({
                "mode": "field-plate-render",
                "verdict": "FIELD_PLATE_RENDERED",
                "plate": args.plate.display().to_string(),
                "out": args.out.display().to_string(),
                "visual_render_is_not_authority": true,
                "field_signature": plate.field_signature
            });
            print_value(&out, &args.format)?;
            Ok(EXIT_PASS)
        }
    }
}

pub(crate) fn build_plate(records: usize) -> Result<FieldPlateSnapshot> {
    let records = records.clamp(1, nanda_6m::ACTIVE_FIELD_RECORDS);
    let memory = plate_triads(records);
    let query_triads = memory.len().min(DEFAULT_QUERY_TRIADS);
    let query = nanda_6m::project_triads(&memory[..query_triads]);
    let mut arena = nanda_6m::PackedActive65kArena::new();
    let run = nanda_6m::run_packed_active65k_cycle(&memory, &query, &mut arena, 18, 0);
    let usage = nanda_6m::validate_active65k_runtime(memory.len(), 18, 0);
    let (memory_hash, query_hash, dot_hash, route_hash, group_hash, heatmap) =
        field_hashes(&memory, &query);
    let proof_hash = proof_hash(&run.proof);
    let route_peak = plate_peak(run.discovery.route_peak);
    let group_peak = plate_peak(run.discovery.group_peak);
    let top_record = PlateTopRecord {
        record_index: run.discovery.top_record.record_index,
        route_id: run.discovery.top_record.route_id,
        group_id: run.discovery.top_record.group_id,
        dot: run.discovery.top_record.dot,
    };
    let proof = PlateProof {
        records_scanned: run.proof.records_scanned,
        dot_records_scored: run.proof.dot_records_scored,
        dot_records_skipped: run
            .proof
            .records_scanned
            .saturating_sub(run.proof.dot_records_scored),
        route_considered: run.proof.route_summary.considered,
        route_support: run.proof.route_summary.support_count,
        route_anti: run.proof.route_summary.anti_count,
        route_positive_dot: run.proof.route_summary.field.positive_dot,
        route_negative_dot: run.proof.route_summary.field.negative_dot,
        group_considered: run.proof.group_summary.considered,
        group_support: run.proof.group_summary.support_count,
        group_anti: run.proof.group_summary.anti_count,
        group_positive_dot: run.proof.group_summary.field.positive_dot,
        group_negative_dot: run.proof.group_summary.field.negative_dot,
        lane_applied: run.proof.lane_sweep.applied,
        lane_improved: run.proof.lane_sweep.improved,
        lane_focused: run.proof.lane_sweep.focused,
        checksum: run.proof.checksum,
    };
    let runtime_contract = PlateRuntimeContract {
        full_active_scan: usage.full_active_scan,
        proof_rescan: usage.proof_rescan,
        no_per_record_score_arrays: true,
        json_used_in_hot_loop: false,
        heap_allocations_in_hot_loop: 0,
        workspace_required_bytes: usage.workspace_required_bytes,
        workspace_budget_bytes: usage.workspace_budget_bytes,
        fits_l3: usage.fits_l3,
        workspace_fits: usage.workspace_fits,
    };
    let claim_boundary = PlateClaimBoundary {
        exact_kernel_snapshot: true,
        visual_render_is_not_authority: true,
        discovery_is_not_answer_authority: true,
        llm_ready: false,
        nonlinear_memory_proven: false,
        hardware_cache_residency_counter_proven: false,
    };
    let field_signature = field_signature(FieldSignatureInput {
        memory_hash: &memory_hash,
        query_hash: &query_hash,
        dot_hash: &dot_hash,
        route_hash: &route_hash,
        group_hash: &group_hash,
        proof_hash: &proof_hash,
        route_peak: &route_peak,
        group_peak: &group_peak,
        top_record,
        proof,
        runtime_contract,
    });
    Ok(FieldPlateSnapshot {
        snapshot_version: FIELD_PLATE_VERSION.to_string(),
        mode: "field-plate-snapshot".to_string(),
        kernel: REFERENCE_KERNEL.to_string(),
        projection: EXACT_PROJECTION.to_string(),
        core_version: CORE_VERSION.to_string(),
        nanda_6m_version: nanda_6m::VERSION.to_string(),
        wave_dim: nanda_6m::WAVE_DIM,
        records: memory.len(),
        query_triads,
        memory_hash,
        query_hash,
        dot_hash,
        route_accumulator_hash: route_hash,
        group_accumulator_hash: group_hash,
        proof_hash,
        field_signature,
        runtime_contract,
        route_peak,
        group_peak,
        top_record,
        proof,
        heatmap,
        claim_boundary,
    })
}

pub(crate) fn compare_plates(
    expected: &FieldPlateSnapshot,
    current: &FieldPlateSnapshot,
) -> FieldPlateComparison {
    let checks = [
        (
            "snapshot_version",
            expected.snapshot_version == current.snapshot_version,
        ),
        ("kernel", expected.kernel == current.kernel),
        ("projection", expected.projection == current.projection),
        ("wave_dim", expected.wave_dim == current.wave_dim),
        ("records", expected.records == current.records),
        (
            "query_triads",
            expected.query_triads == current.query_triads,
        ),
        ("memory_hash", expected.memory_hash == current.memory_hash),
        ("query_hash", expected.query_hash == current.query_hash),
        ("dot_hash", expected.dot_hash == current.dot_hash),
        (
            "route_accumulator_hash",
            expected.route_accumulator_hash == current.route_accumulator_hash,
        ),
        (
            "group_accumulator_hash",
            expected.group_accumulator_hash == current.group_accumulator_hash,
        ),
        ("proof_hash", expected.proof_hash == current.proof_hash),
        (
            "field_signature",
            expected.field_signature == current.field_signature,
        ),
    ];
    let checked_fields = checks
        .iter()
        .map(|(field, _)| (*field).to_string())
        .collect::<Vec<_>>();
    let mismatches = checks
        .iter()
        .filter(|(_, pass)| !*pass)
        .map(|(field, _)| (*field).to_string())
        .collect::<Vec<_>>();
    let exact_match = mismatches.is_empty();
    FieldPlateComparison {
        mode: "exact-field-plate-comparison".to_string(),
        verdict: if exact_match {
            "FIELD_PLATE_MATCH"
        } else {
            "FIELD_PLATE_VETO"
        }
        .to_string(),
        safe_to_use_kernel: exact_match,
        expected_signature: expected.field_signature.clone(),
        current_signature: current.field_signature.clone(),
        exact_match,
        mismatches,
        checked_fields,
        claim_boundary: current.claim_boundary,
    }
}

pub(crate) fn render_plate_svg(plate: &FieldPlateSnapshot) -> String {
    let width = 980.0f64;
    let height = 420.0f64;
    let base_y = 300.0f64;
    let bar_width = 12.0f64;
    let gap = 2.0f64;
    let max_abs = plate
        .heatmap
        .iter()
        .flat_map(|bin| [bin.positive_dot.abs(), bin.negative_dot.abs()])
        .max()
        .unwrap_or(1)
        .max(1) as f64;
    let mut svg = String::new();
    let _ = writeln!(
        svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"##
    );
    let _ = writeln!(
        svg,
        r##"<rect width="100%" height="100%" fill="#101418"/>"##
    );
    let _ = writeln!(
        svg,
        r##"<text x="24" y="34" fill="#e8eef2" font-family="monospace" font-size="18">NANDA Field Plate</text>"##
    );
    let _ = writeln!(
        svg,
        r##"<text x="24" y="58" fill="#a8b3bd" font-family="monospace" font-size="12">signature {}</text>"##,
        &plate.field_signature[..plate.field_signature.len().min(24)]
    );
    let _ = writeln!(
        svg,
        r##"<text x="24" y="82" fill="#a8b3bd" font-family="monospace" font-size="12">records {} wave_dim {} route {} group {} proof_dot {}/{}</text>"##,
        plate.records,
        plate.wave_dim,
        plate.route_peak.top_id,
        plate.group_peak.top_id,
        plate.proof.dot_records_scored,
        plate.proof.records_scanned
    );
    let _ = writeln!(
        svg,
        r##"<line x1="24" y1="{base_y}" x2="956" y2="{base_y}" stroke="#45515c" stroke-width="1"/>"##
    );
    for bin in &plate.heatmap {
        let x = 24.0 + bin.bin as f64 * (bar_width + gap);
        let positive_h = (bin.positive_dot.max(0) as f64 / max_abs * 170.0).max(1.0);
        let negative_h = (bin.negative_dot.abs() as f64 / max_abs * 120.0).max(1.0);
        let net_color = if bin.net_dot >= 0 {
            "#62d26f"
        } else {
            "#ff6b6b"
        };
        let _ = writeln!(
            svg,
            r#"<rect x="{x:.1}" y="{:.1}" width="{bar_width}" height="{positive_h:.1}" fill="{net_color}" opacity="0.85"/>"#,
            base_y - positive_h
        );
        let _ = writeln!(
            svg,
            r##"<rect x="{x:.1}" y="{base_y:.1}" width="{bar_width}" height="{negative_h:.1}" fill="#d94848" opacity="0.55"/>"##
        );
    }
    let _ = writeln!(
        svg,
        r##"<text x="24" y="392" fill="#a8b3bd" font-family="monospace" font-size="12">green = constructive support, red = anti-wave pressure, SVG is visual only; JSON signature is authority.</text>"##
    );
    let _ = writeln!(svg, "</svg>");
    svg
}

fn field_hashes(
    memory: &[nanda_6m::PackedTriad32],
    query: &nanda_6m::PackedWave1024,
) -> (String, String, String, String, String, Vec<PlateHeatBin>) {
    let mut memory_hasher = Sha256::new();
    for triad in memory {
        hash_triad(&mut memory_hasher, triad);
    }
    let mut query_hasher = Sha256::new();
    for value in query.values {
        query_hasher.update(value.to_le_bytes());
    }
    let mut dot_hasher = Sha256::new();
    let mut routes = vec![PlateAccumulator::default(); nanda_6m::SCORE_BUCKET_CAPACITY];
    let mut groups = vec![PlateAccumulator::default(); nanda_6m::SCORE_BUCKET_CAPACITY];
    let mut heatmap = (0..HEATMAP_BINS)
        .map(|bin| PlateHeatBin {
            bin,
            min_dot: i64::MAX,
            ..PlateHeatBin::default()
        })
        .collect::<Vec<_>>();
    for (index, triad) in memory.iter().copied().enumerate() {
        let dot = nanda_6m::score_packed_triad_projection_dot(query, &triad);
        let record_index = index.min(usize::from(u16::MAX)) as u16;
        hash_u16(&mut dot_hasher, record_index);
        hash_i64(&mut dot_hasher, dot);
        hash_u16(&mut dot_hasher, triad.route_id);
        hash_u16(&mut dot_hasher, triad.group_id);
        accumulate_plate_axis(&mut routes, triad.route_id, record_index, dot);
        accumulate_plate_axis(&mut groups, triad.group_id, record_index, dot);
        let bin = index * HEATMAP_BINS / memory.len().max(1);
        if let Some(heat) = heatmap.get_mut(bin.min(HEATMAP_BINS - 1)) {
            heat.records = heat.records.saturating_add(1);
            heat.net_dot = heat.net_dot.saturating_add(dot);
            if dot > 0 {
                heat.positive_dot = heat.positive_dot.saturating_add(dot);
            } else if dot < 0 {
                heat.negative_dot = heat.negative_dot.saturating_add(dot);
            }
            heat.max_dot = heat.max_dot.max(dot);
            heat.min_dot = heat.min_dot.min(dot);
        }
    }
    for heat in &mut heatmap {
        if heat.records == 0 {
            heat.min_dot = 0;
        }
    }
    let mut route_hasher = Sha256::new();
    for accumulator in routes {
        hash_accumulator(&mut route_hasher, accumulator);
    }
    let mut group_hasher = Sha256::new();
    for accumulator in groups {
        hash_accumulator(&mut group_hasher, accumulator);
    }
    (
        hex(memory_hasher),
        hex(query_hasher),
        hex(dot_hasher),
        hex(route_hasher),
        hex(group_hasher),
        heatmap,
    )
}

fn plate_triads(count: usize) -> Vec<nanda_6m::PackedTriad32> {
    (0..count)
        .map(|idx| {
            let idx = idx as u32;
            nanda_6m::PackedTriad32::new(nanda_6m::PackedTriadInput {
                subject_id: 1_000 + idx,
                object_id: 10_000 + idx.wrapping_mul(3),
                evidence_ref: 100_000 + idx.wrapping_mul(7),
                wave_seed: 0x9e37_0000u32.wrapping_add(idx.wrapping_mul(97)),
                relation_id: (10 + (idx % 31)) as u16,
                route_id: (1 + (idx % 7)) as u16,
                group_id: (1 + (idx % 11)) as u16,
                role_pack: (((idx % 16) as u16) << 8) | ((idx % 13) as u16),
                flags: (idx % 5) as u16,
                lane_hint: (idx % 17) as u16,
                check: 0x55aa ^ (idx as u16).wrapping_mul(13),
                confidence: 160 + (idx % 80) as u8,
                polarity: (idx % 2) as u8,
            })
        })
        .collect()
}

fn accumulate_plate_axis(
    accumulators: &mut [PlateAccumulator],
    id: u16,
    record_index: u16,
    dot: i64,
) {
    let Some(accumulator) = accumulators.get_mut(usize::from(id)) else {
        return;
    };
    accumulator.id = id;
    let was_empty = accumulator.considered == 0;
    accumulator.considered = accumulator.considered.saturating_add(1);
    accumulator.net_dot = accumulator.net_dot.saturating_add(dot);
    if dot > 0 {
        accumulator.positive_dot = accumulator.positive_dot.saturating_add(dot);
        accumulator.support_count = accumulator.support_count.saturating_add(1);
    } else if dot < 0 {
        accumulator.negative_dot = accumulator.negative_dot.saturating_add(dot);
        accumulator.anti_count = accumulator.anti_count.saturating_add(1);
    }
    if was_empty || dot > accumulator.top_dot {
        accumulator.top_dot = dot;
        accumulator.top_record = record_index;
    }
}

fn plate_peak(peak: nanda_6m::PackedAxisPeak) -> PlatePeak {
    PlatePeak {
        top_id: peak.top_id,
        top_score: peak.top_score,
        top_score_bits: peak.top_score.to_bits(),
        second_id: peak.second_id,
        second_score: peak.second_score,
        second_score_bits: peak.second_score.to_bits(),
        margin: peak.margin,
        margin_bits: peak.margin.to_bits(),
        state: peak.state.as_str().to_string(),
    }
}

struct FieldSignatureInput<'a> {
    memory_hash: &'a str,
    query_hash: &'a str,
    dot_hash: &'a str,
    route_hash: &'a str,
    group_hash: &'a str,
    proof_hash: &'a str,
    route_peak: &'a PlatePeak,
    group_peak: &'a PlatePeak,
    top_record: PlateTopRecord,
    proof: PlateProof,
    runtime_contract: PlateRuntimeContract,
}

fn field_signature(input: FieldSignatureInput<'_>) -> String {
    let mut hasher = Sha256::new();
    hash_str(&mut hasher, FIELD_PLATE_VERSION);
    hash_str(&mut hasher, input.memory_hash);
    hash_str(&mut hasher, input.query_hash);
    hash_str(&mut hasher, input.dot_hash);
    hash_str(&mut hasher, input.route_hash);
    hash_str(&mut hasher, input.group_hash);
    hash_str(&mut hasher, input.proof_hash);
    hash_peak(&mut hasher, input.route_peak);
    hash_peak(&mut hasher, input.group_peak);
    hash_top_record(&mut hasher, input.top_record);
    hash_proof(&mut hasher, input.proof);
    hash_u64(
        &mut hasher,
        input.runtime_contract.workspace_required_bytes as u64,
    );
    hash_u64(
        &mut hasher,
        input.runtime_contract.workspace_budget_bytes as u64,
    );
    hex(hasher)
}

fn proof_hash(proof: &nanda_6m::PackedActive65kProof) -> String {
    let mut hasher = Sha256::new();
    hash_u32(&mut hasher, proof.records_scanned);
    hash_u32(&mut hasher, proof.dot_records_scored);
    hash_support_summary(&mut hasher, proof.route_summary);
    hash_support_summary(&mut hasher, proof.group_summary);
    hash_u16(&mut hasher, proof.fields_built);
    hash_u16(&mut hasher, proof.lanes_compiled);
    hash_u64(&mut hasher, proof.lane_sweep.applied as u64);
    hash_u64(&mut hasher, proof.lane_sweep.improved as u64);
    hash_u64(&mut hasher, proof.lane_sweep.focused as u64);
    hash_u64(&mut hasher, proof.lane_sweep.checksum);
    hash_u64(&mut hasher, proof.checksum);
    hex(hasher)
}

fn hash_support_summary(hasher: &mut Sha256, summary: nanda_6m::PackedSupportSummary) {
    hash_u16(hasher, summary.considered);
    hash_u16(hasher, summary.support_count);
    hash_u16(hasher, summary.anti_count);
    hash_i64(hasher, summary.field.positive_dot);
    hash_i64(hasher, summary.field.negative_dot);
    hash_u16(hasher, summary.field.top_id);
    hash_u32(hasher, summary.field.key_hash);
    hash_u64(hasher, summary.field.support_mask_a);
    hash_u64(hasher, summary.field.support_mask_b);
    hash_u64(hasher, summary.field.anti_mask_a);
    hash_u64(hasher, summary.field.anti_mask_b);
}

fn hash_triad(hasher: &mut Sha256, triad: &nanda_6m::PackedTriad32) {
    hash_u32(hasher, triad.subject_id);
    hash_u32(hasher, triad.object_id);
    hash_u32(hasher, triad.evidence_ref);
    hash_u32(hasher, triad.wave_seed);
    hash_u16(hasher, triad.relation_id);
    hash_u16(hasher, triad.route_id);
    hash_u16(hasher, triad.group_id);
    hash_u16(hasher, triad.role_pack);
    hash_u16(hasher, triad.flags);
    hash_u16(hasher, triad.lane_hint);
    hash_u16(hasher, triad.check);
    hasher.update([triad.confidence, triad.polarity]);
}

fn hash_accumulator(hasher: &mut Sha256, accumulator: PlateAccumulator) {
    hash_u16(hasher, accumulator.id);
    hash_u32(hasher, accumulator.considered);
    hash_u32(hasher, accumulator.support_count);
    hash_u32(hasher, accumulator.anti_count);
    hash_i64(hasher, accumulator.positive_dot);
    hash_i64(hasher, accumulator.negative_dot);
    hash_i64(hasher, accumulator.net_dot);
    hash_i64(hasher, accumulator.top_dot);
    hash_u16(hasher, accumulator.top_record);
}

fn hash_peak(hasher: &mut Sha256, peak: &PlatePeak) {
    hash_u16(hasher, peak.top_id);
    hash_u64(hasher, peak.top_score_bits);
    hash_u16(hasher, peak.second_id);
    hash_u64(hasher, peak.second_score_bits);
    hash_u64(hasher, peak.margin_bits);
    hash_str(hasher, &peak.state);
}

fn hash_top_record(hasher: &mut Sha256, record: PlateTopRecord) {
    hash_u16(hasher, record.record_index);
    hash_u16(hasher, record.route_id);
    hash_u16(hasher, record.group_id);
    hash_i64(hasher, record.dot);
}

fn hash_proof(hasher: &mut Sha256, proof: PlateProof) {
    hash_u32(hasher, proof.records_scanned);
    hash_u32(hasher, proof.dot_records_scored);
    hash_u32(hasher, proof.dot_records_skipped);
    hash_u16(hasher, proof.route_considered);
    hash_u16(hasher, proof.route_support);
    hash_u16(hasher, proof.route_anti);
    hash_i64(hasher, proof.route_positive_dot);
    hash_i64(hasher, proof.route_negative_dot);
    hash_u16(hasher, proof.group_considered);
    hash_u16(hasher, proof.group_support);
    hash_u16(hasher, proof.group_anti);
    hash_i64(hasher, proof.group_positive_dot);
    hash_i64(hasher, proof.group_negative_dot);
    hash_u64(hasher, proof.lane_applied as u64);
    hash_u64(hasher, proof.lane_improved as u64);
    hash_u64(hasher, proof.lane_focused as u64);
    hash_u64(hasher, proof.checksum);
}

fn hash_str(hasher: &mut Sha256, value: &str) {
    hash_u64(hasher, value.len() as u64);
    hasher.update(value.as_bytes());
}

fn hash_u16(hasher: &mut Sha256, value: u16) {
    hasher.update(value.to_le_bytes());
}

fn hash_u32(hasher: &mut Sha256, value: u32) {
    hasher.update(value.to_le_bytes());
}

fn hash_u64(hasher: &mut Sha256, value: u64) {
    hasher.update(value.to_le_bytes());
}

fn hash_i64(hasher: &mut Sha256, value: i64) {
    hasher.update(value.to_le_bytes());
}

fn hex(hasher: Sha256) -> String {
    format!("{:x}", hasher.finalize())
}

fn write_plate(path: &PathBuf, plate: &FieldPlateSnapshot) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create field plate directory {}", parent.display()))?;
        }
    }
    fs::write(path, serde_json::to_vec_pretty(plate)?)
        .with_context(|| format!("write field plate {}", path.display()))
}

fn read_plate(path: &PathBuf) -> Result<FieldPlateSnapshot> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read field plate {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse field plate {}", path.display()))
}

fn compact_plate_view(plate: &FieldPlateSnapshot) -> Value {
    json!({
        "snapshot_version": plate.snapshot_version,
        "kernel": plate.kernel,
        "projection": plate.projection,
        "records": plate.records,
        "wave_dim": plate.wave_dim,
        "field_signature": plate.field_signature,
        "memory_hash": plate.memory_hash,
        "query_hash": plate.query_hash,
        "dot_hash": plate.dot_hash,
        "route_accumulator_hash": plate.route_accumulator_hash,
        "group_accumulator_hash": plate.group_accumulator_hash,
        "proof_hash": plate.proof_hash,
        "route_peak": plate.route_peak,
        "group_peak": plate.group_peak,
        "top_record": plate.top_record,
        "proof": plate.proof
    })
}

fn print_value(out: &Value, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(out)?),
        OutputFormat::Text => {
            println!(
                "{}",
                out["comparison"]["verdict"]
                    .as_str()
                    .or_else(|| out["verdict"].as_str())
                    .unwrap_or("FIELD_PLATE")
            );
            if let Some(signature) = out["plate"]["field_signature"]
                .as_str()
                .or_else(|| out["field_signature"].as_str())
                .or_else(|| out["comparison"]["current_signature"].as_str())
            {
                println!("field_signature: {signature}");
            }
            if let Some(out_path) = out["out"].as_str() {
                println!("out: {out_path}");
            }
        }
        OutputFormat::Md => {
            println!(
                "# {}\n",
                out["comparison"]["verdict"]
                    .as_str()
                    .or_else(|| out["verdict"].as_str())
                    .unwrap_or("FIELD_PLATE")
            );
            if let Some(signature) = out["plate"]["field_signature"]
                .as_str()
                .or_else(|| out["field_signature"].as_str())
                .or_else(|| out["comparison"]["current_signature"].as_str())
            {
                println!("- field_signature: `{signature}`");
            }
            if let Some(out_path) = out["out"].as_str() {
                println!("- out: `{out_path}`");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_plate_roundtrip_matches_exact_signature() {
        let plate = build_plate(nanda_6m::ACTIVE_FIELD_RECORDS).expect("build field plate");
        let comparison = compare_plates(&plate, &plate);

        assert_eq!(comparison.verdict, "FIELD_PLATE_MATCH");
        assert!(comparison.safe_to_use_kernel);
        assert!(comparison.mismatches.is_empty());
        assert_eq!(plate.records, nanda_6m::ACTIVE_FIELD_RECORDS);
        assert_eq!(plate.wave_dim, nanda_6m::WAVE_DIM);
        assert!(plate.runtime_contract.full_active_scan);
        assert!(plate.runtime_contract.proof_rescan);
    }

    #[test]
    fn field_plate_vetoes_drift() {
        let plate = build_plate(nanda_6m::ACTIVE_FIELD_RECORDS).expect("build field plate");
        let mut drifted = plate.clone();
        drifted.dot_hash = "drift".to_string();
        let comparison = compare_plates(&plate, &drifted);

        assert_eq!(comparison.verdict, "FIELD_PLATE_VETO");
        assert!(!comparison.safe_to_use_kernel);
        assert!(comparison.mismatches.contains(&"dot_hash".to_string()));
    }

    #[test]
    fn field_plate_svg_is_visual_not_authority() {
        let plate = build_plate(nanda_6m::ACTIVE_FIELD_RECORDS).expect("build field plate");
        let svg = render_plate_svg(&plate);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("NANDA Field Plate"));
        assert!(svg.contains("visual only"));
    }
}
