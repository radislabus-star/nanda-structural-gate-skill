use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde_json::{json, Value};
use std::hint::black_box;
use std::time::Instant;

use super::{nanda_6m, OutputFormat, CORE_VERSION, EXIT_PASS};

#[derive(Parser)]
pub(super) struct Bench6mArgs {
    #[arg(long, value_enum, default_value = "all")]
    mode: Bench6mMode,
    #[arg(long, default_value_t = 1_000_000)]
    replay_iterations: u64,
    #[arg(long, default_value_t = 10_000)]
    projection_iterations: u64,
    #[arg(long, default_value_t = 1_000_000)]
    lane_iterations: u64,
    #[arg(long, default_value_t = 100_000)]
    lane_sweep_iterations: u64,
    #[arg(long, default_value_t = 100)]
    support_build_iterations: u64,
    #[arg(long, default_value_t = 64)]
    lane_sweep_width: usize,
    #[arg(long, default_value_t = 64)]
    triads: usize,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Clone, ValueEnum)]
enum Bench6mMode {
    Replay,
    Projection,
    Lane,
    LaneSweep,
    AlignedLaneSweep,
    AlignedCompileSweep,
    SupportBuild,
    SupportBuildCompileSweep,
    SupportScoreBuild,
    SupportScoreBuildCompileSweep,
    SupportBucketBuild,
    SupportBucketBuildCompileSweep,
    HotCycle,
    All,
}

pub(super) fn cmd(args: Bench6mArgs) -> Result<u8> {
    let include_replay = matches!(args.mode, Bench6mMode::Replay | Bench6mMode::All);
    let include_projection = matches!(args.mode, Bench6mMode::Projection | Bench6mMode::All);
    let include_lane = matches!(args.mode, Bench6mMode::Lane | Bench6mMode::All);
    let include_lane_sweep = matches!(args.mode, Bench6mMode::LaneSweep | Bench6mMode::All);
    let include_aligned_lane_sweep =
        matches!(args.mode, Bench6mMode::AlignedLaneSweep | Bench6mMode::All);
    let include_aligned_compile_sweep = matches!(
        args.mode,
        Bench6mMode::AlignedCompileSweep | Bench6mMode::All
    );
    let include_support_build = matches!(args.mode, Bench6mMode::SupportBuild | Bench6mMode::All);
    let include_support_build_compile_sweep = matches!(
        args.mode,
        Bench6mMode::SupportBuildCompileSweep | Bench6mMode::All
    );
    let include_support_score_build =
        matches!(args.mode, Bench6mMode::SupportScoreBuild | Bench6mMode::All);
    let include_support_score_build_compile_sweep = matches!(
        args.mode,
        Bench6mMode::SupportScoreBuildCompileSweep | Bench6mMode::All
    );
    let include_support_bucket_build = matches!(
        args.mode,
        Bench6mMode::SupportBucketBuild | Bench6mMode::All
    );
    let include_support_bucket_build_compile_sweep = matches!(
        args.mode,
        Bench6mMode::SupportBucketBuildCompileSweep | Bench6mMode::All
    );
    let include_hot_cycle = matches!(args.mode, Bench6mMode::HotCycle | Bench6mMode::All);
    let replay = if include_replay {
        Some(bench6m_replay(args.replay_iterations))
    } else {
        None
    };
    let projection = if include_projection {
        Some(bench6m_projection(
            args.projection_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
        ))
    } else {
        None
    };
    let lane_application = if include_lane {
        Some(bench6m_lane_application(args.lane_iterations))
    } else {
        None
    };
    let lane_sweep = if include_lane_sweep {
        Some(bench6m_lane_sweep(
            args.lane_sweep_iterations,
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            LaneSweepKernel::Search,
        ))
    } else {
        None
    };
    let aligned_lane_sweep = if include_aligned_lane_sweep {
        Some(bench6m_lane_sweep(
            args.lane_sweep_iterations,
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            LaneSweepKernel::Aligned,
        ))
    } else {
        None
    };
    let aligned_compile_sweep = if include_aligned_compile_sweep {
        Some(bench6m_lane_sweep(
            args.lane_sweep_iterations,
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            LaneSweepKernel::AlignedCompile,
        ))
    } else {
        None
    };
    let support_build = if include_support_build {
        Some(bench6m_support_build(
            args.support_build_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            false,
        ))
    } else {
        None
    };
    let support_build_compile_sweep = if include_support_build_compile_sweep {
        Some(bench6m_support_build(
            args.support_build_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            true,
        ))
    } else {
        None
    };
    let support_score_build = if include_support_score_build {
        Some(bench6m_support_score_build(
            args.support_build_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            false,
        ))
    } else {
        None
    };
    let support_score_build_compile_sweep = if include_support_score_build_compile_sweep {
        Some(bench6m_support_score_build(
            args.support_build_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            true,
        ))
    } else {
        None
    };
    let support_bucket_build = if include_support_bucket_build {
        Some(bench6m_support_bucket_build(
            args.support_build_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            false,
        ))
    } else {
        None
    };
    let support_bucket_build_compile_sweep = if include_support_bucket_build_compile_sweep {
        Some(bench6m_support_bucket_build(
            args.support_build_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
            true,
        ))
    } else {
        None
    };
    let hot_cycle = if include_hot_cycle {
        Some(bench6m_hot_cycle(
            args.support_build_iterations,
            args.triads.clamp(1, nanda_6m::TRIAD_CAPACITY),
            args.lane_sweep_width.clamp(1, nanda_6m::LANE_CAPACITY),
        ))
    } else {
        None
    };
    let out = json!({
        "mode": "nanda-6m-hot-benchmark",
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "budget_bytes": nanda_6m::BUDGET_BYTES,
        "wave_dim": nanda_6m::WAVE_DIM,
        "benchmarks": {
            "replay": replay,
            "projection": projection,
            "lane_application": lane_application,
            "lane_sweep": lane_sweep,
            "aligned_lane_sweep": aligned_lane_sweep,
            "aligned_compile_sweep": aligned_compile_sweep,
            "support_build": support_build,
            "support_build_compile_sweep": support_build_compile_sweep,
            "support_score_build": support_score_build,
            "support_score_build_compile_sweep": support_score_build_compile_sweep,
            "support_bucket_build": support_bucket_build,
            "support_bucket_build_compile_sweep": support_bucket_build_compile_sweep,
            "hot_cycle": hot_cycle
        },
        "interpretation": {
            "replay": "Pure typed replay firewall; no JSON, no file IO, no process spawn.",
            "projection": "Packed 1024-dimensional projection plus centroid scoring over an in-memory triad window.",
            "lane_application": "Packed suppress-anti-support lane compilation/application over typed support fields.",
            "lane_sweep": "Batch packed suppress-anti-support lane sweep over typed support fields and compiled lane arena.",
            "aligned_lane_sweep": "Fast batch lane sweep for pre-aligned field/lane windows with no arena search.",
            "aligned_compile_sweep": "Fused batch lane compilation and aligned application over support fields.",
            "support_build": "Build typed support fields from packed memory triads and a query wave.",
            "support_build_compile_sweep": "Build typed support fields, compile aligned lanes, and apply the sweep.",
            "support_score_build": "Build per-triad support scores once, then assemble support fields from cached dots.",
            "support_score_build_compile_sweep": "Build cached support scores, assemble fields, compile aligned lanes, and apply the sweep.",
            "support_bucket_build": "Build cached support scores, bucket them by route/group, then assemble fields from bucket ranges.",
            "support_bucket_build_compile_sweep": "Build cached support score buckets, assemble fields, compile aligned lanes, and apply the sweep.",
            "hot_cycle": "Single typed hot-cycle call: score cache, route/group buckets, support fields, lane compilation, and aligned sweep.",
            "not_measured": "CLI startup, JSON parsing, dictionary packing, and report serialization are intentionally excluded."
        }
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_bench6m_text(&out),
        OutputFormat::Md => print_bench6m_md(&out),
    }
    Ok(EXIT_PASS)
}

fn bench6m_replay(iterations: u64) -> Value {
    let iterations = iterations.max(1);
    let inputs = [
        nanda_6m::ReplayDecisionInput {
            raw_state: nanda_6m::RawPeakState::Thin,
            raw_safe_to_answer: false,
            raw_verdict_pass: false,
            matched_keys: 2,
            observer_net_dot: 64,
            full_delta_dot: 512,
            soft: nanda_6m::ReplayTouch {
                after_net_dot: 192,
                delta_dot: 128,
                field_state: nanda_6m::ReplayFieldState::Focused,
            },
            full: nanda_6m::ReplayTouch {
                after_net_dot: 576,
                delta_dot: 512,
                field_state: nanda_6m::ReplayFieldState::Focused,
            },
            stability_state: nanda_6m::ReplayStabilityState::StableUnderSoftTouch,
            compute_state: nanda_6m::ReplayComputeState::Ready,
        },
        nanda_6m::ReplayDecisionInput {
            raw_state: nanda_6m::RawPeakState::Focused,
            raw_safe_to_answer: true,
            raw_verdict_pass: true,
            matched_keys: 2,
            observer_net_dot: 128,
            full_delta_dot: 384,
            soft: nanda_6m::ReplayTouch {
                after_net_dot: 256,
                delta_dot: 128,
                field_state: nanda_6m::ReplayFieldState::Focused,
            },
            full: nanda_6m::ReplayTouch {
                after_net_dot: 512,
                delta_dot: 384,
                field_state: nanda_6m::ReplayFieldState::Focused,
            },
            stability_state: nanda_6m::ReplayStabilityState::StableUnderSoftTouch,
            compute_state: nanda_6m::ReplayComputeState::Ready,
        },
        nanda_6m::ReplayDecisionInput {
            raw_state: nanda_6m::RawPeakState::Focused,
            raw_safe_to_answer: true,
            raw_verdict_pass: true,
            matched_keys: 1,
            observer_net_dot: 128,
            full_delta_dot: -96,
            soft: nanda_6m::ReplayTouch {
                after_net_dot: 96,
                delta_dot: -32,
                field_state: nanda_6m::ReplayFieldState::Weakened,
            },
            full: nanda_6m::ReplayTouch {
                after_net_dot: 32,
                delta_dot: -96,
                field_state: nanda_6m::ReplayFieldState::Weakened,
            },
            stability_state: nanda_6m::ReplayStabilityState::Destabilizing,
            compute_state: nanda_6m::ReplayComputeState::Weak,
        },
        nanda_6m::ReplayDecisionInput {
            raw_state: nanda_6m::RawPeakState::Thin,
            raw_safe_to_answer: false,
            raw_verdict_pass: false,
            matched_keys: 0,
            observer_net_dot: 32,
            full_delta_dot: 0,
            soft: nanda_6m::ReplayTouch::default(),
            full: nanda_6m::ReplayTouch::default(),
            stability_state: nanda_6m::ReplayStabilityState::NoReplayField,
            compute_state: nanda_6m::ReplayComputeState::None,
        },
    ];
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for idx in 0..iterations {
        let input = black_box(inputs[(idx as usize) & 3]);
        let decision = nanda_6m::evaluate_replay(input);
        checksum = checksum
            .wrapping_add(decision.matched_keys)
            .wrapping_add(decision.soft_touch_net_dot as u64)
            .wrapping_add(decision.verdict as u64);
        black_box(decision);
    }
    let elapsed = start.elapsed();
    bench_result_json(iterations, elapsed.as_nanos(), checksum, "evaluate_replay")
}

fn bench6m_lane_application(iterations: u64) -> Value {
    let iterations = iterations.max(1);
    let fields = [
        nanda_6m::PackedSupportField {
            top_id: 7,
            key_hash: 0x1234,
            positive_dot: 288,
            negative_dot: -256,
            support_mask_a: 0b0001_0000,
            support_mask_b: 0,
            anti_mask_a: 0b0110_0000,
            anti_mask_b: 0,
        },
        nanda_6m::PackedSupportField {
            top_id: 9,
            key_hash: 0x5678,
            positive_dot: 320,
            negative_dot: -64,
            support_mask_a: 0b0000_0011,
            support_mask_b: 0,
            anti_mask_a: 0b0000_1100,
            anti_mask_b: 0,
        },
        nanda_6m::PackedSupportField {
            top_id: 11,
            key_hash: 0x9abc,
            positive_dot: 256,
            negative_dot: 0,
            support_mask_a: 0b0000_1111,
            support_mask_b: 0,
            anti_mask_a: 0,
            anti_mask_b: 0,
        },
        nanda_6m::PackedSupportField {
            top_id: 13,
            key_hash: 0xdef0,
            positive_dot: 96,
            negative_dot: -32,
            support_mask_a: 0b0011_0000,
            support_mask_b: 0,
            anti_mask_a: 0b1100_0000,
            anti_mask_b: 0,
        },
    ];
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for idx in 0..iterations {
        let field = black_box(fields[(idx as usize) & 3]);
        let application = nanda_6m::compile_and_apply_suppress_anti_lane(field);
        checksum = checksum
            .wrapping_add(application.after_net_dot as u64)
            .wrapping_add(application.delta_dot as u64)
            .wrapping_add(application.lane.lane_id as u64)
            .wrapping_add(u64::from(application.focused_candidate));
        black_box(application);
    }
    let elapsed = start.elapsed();
    bench_result_json(
        iterations,
        elapsed.as_nanos(),
        checksum,
        "compile_and_apply_suppress_anti_lane",
    )
}

#[derive(Clone, Copy)]
enum LaneSweepKernel {
    Search,
    Aligned,
    AlignedCompile,
}

fn bench6m_lane_sweep(iterations: u64, width: usize, kernel: LaneSweepKernel) -> Value {
    let iterations = iterations.max(1);
    let fields = bench6m_support_fields(width);
    let mut lanes = vec![nanda_6m::PackedLane64::default(); fields.len()];
    let compiled = if matches!(kernel, LaneSweepKernel::AlignedCompile) {
        fields.len()
    } else {
        nanda_6m::compile_suppress_anti_lanes(&fields, &mut lanes)
    };
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for _ in 0..iterations {
        let sweep = match kernel {
            LaneSweepKernel::Search => nanda_6m::apply_suppress_anti_lane_sweep(
                black_box(fields.as_slice()),
                black_box(lanes.as_slice()),
            ),
            LaneSweepKernel::Aligned => nanda_6m::apply_aligned_suppress_anti_lane_sweep(
                black_box(fields.as_slice()),
                black_box(lanes.as_slice()),
            ),
            LaneSweepKernel::AlignedCompile => {
                nanda_6m::compile_and_apply_aligned_suppress_anti_lane_sweep(
                    black_box(fields.as_slice()),
                    black_box(lanes.as_mut_slice()),
                )
            }
        };
        checksum = checksum
            .wrapping_add(sweep.checksum)
            .wrapping_add(sweep.best_after_net_dot as u64)
            .wrapping_add(sweep.total_delta_dot as u64)
            .wrapping_add(sweep.focused as u64);
        black_box(sweep);
    }
    let elapsed = start.elapsed();
    let mut out = bench_result_json(
        iterations,
        elapsed.as_nanos(),
        checksum,
        match kernel {
            LaneSweepKernel::Search => "apply_suppress_anti_lane_sweep",
            LaneSweepKernel::Aligned => "apply_aligned_suppress_anti_lane_sweep",
            LaneSweepKernel::AlignedCompile => "compile_and_apply_aligned_suppress_anti_lane_sweep",
        },
    );
    out["fields"] = json!(fields.len());
    out["compiled_lanes"] = json!(compiled);
    out["ns_per_field"] = json!(out["ns_per_op"].as_f64().unwrap_or(0.0) / fields.len() as f64);
    out
}

fn bench6m_support_build(
    iterations: u64,
    triad_count: usize,
    field_count: usize,
    with_compile_sweep: bool,
) -> Value {
    let iterations = iterations.max(1);
    let memory = bench6m_triads(triad_count);
    let query_len = memory.len().clamp(1, 8);
    let query = nanda_6m::project_triads(&memory[..query_len]);
    let mut fields = vec![nanda_6m::PackedSupportField::default(); field_count];
    let mut lanes = vec![nanda_6m::PackedLane64::default(); field_count];
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for iter in 0..iterations {
        for (field_idx, field) in fields.iter_mut().enumerate() {
            let axis = if (field_idx + iter as usize).is_multiple_of(2) {
                nanda_6m::PackedAxis::Route
            } else {
                nanda_6m::PackedAxis::Group
            };
            let top_id = match axis {
                nanda_6m::PackedAxis::Route => 1 + ((field_idx as u16 + iter as u16) % 7),
                nanda_6m::PackedAxis::Group => 1 + ((field_idx as u16 + iter as u16) % 11),
            };
            let summary = nanda_6m::build_packed_support_field(
                black_box(memory.as_slice()),
                black_box(&query),
                axis,
                top_id,
                0x2000_0000u32.wrapping_add(field_idx as u32),
            );
            *field = summary.field;
            checksum = checksum
                .wrapping_add(summary.field.positive_dot as u64)
                .wrapping_add(summary.field.negative_dot as u64)
                .wrapping_add(u64::from(summary.considered))
                .wrapping_add(u64::from(summary.support_count))
                .wrapping_add(u64::from(summary.anti_count));
        }
        if with_compile_sweep {
            let sweep = nanda_6m::compile_and_apply_aligned_suppress_anti_lane_sweep(
                black_box(fields.as_slice()),
                black_box(lanes.as_mut_slice()),
            );
            checksum = checksum
                .wrapping_add(sweep.checksum)
                .wrapping_add(sweep.best_after_net_dot as u64)
                .wrapping_add(sweep.total_delta_dot as u64)
                .wrapping_add(sweep.focused as u64);
            black_box(sweep);
        }
    }
    let elapsed = start.elapsed();
    let mut out = bench_result_json(
        iterations,
        elapsed.as_nanos(),
        checksum,
        if with_compile_sweep {
            "build_support_fields_and_compile_sweep"
        } else {
            "build_support_fields"
        },
    );
    out["triads_in_memory"] = json!(memory.len());
    out["fields"] = json!(field_count);
    out["query_triads"] = json!(query_len);
    out["ns_per_field"] = json!(out["ns_per_op"].as_f64().unwrap_or(0.0) / field_count as f64);
    out
}

fn bench6m_support_score_build(
    iterations: u64,
    triad_count: usize,
    field_count: usize,
    with_compile_sweep: bool,
) -> Value {
    let iterations = iterations.max(1);
    let memory = bench6m_triads(triad_count);
    let query_len = memory.len().clamp(1, 8);
    let query = nanda_6m::project_triads(&memory[..query_len]);
    let mut scores = vec![nanda_6m::PackedTriadSupportScore::default(); memory.len()];
    let mut fields = vec![nanda_6m::PackedSupportField::default(); field_count];
    let mut lanes = vec![nanda_6m::PackedLane64::default(); field_count];
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for iter in 0..iterations {
        let score_count = nanda_6m::build_packed_triad_support_scores(
            black_box(memory.as_slice()),
            black_box(&query),
            black_box(scores.as_mut_slice()),
        );
        let score_slice = &scores[..score_count];
        for (field_idx, field) in fields.iter_mut().enumerate() {
            let axis = if (field_idx + iter as usize).is_multiple_of(2) {
                nanda_6m::PackedAxis::Route
            } else {
                nanda_6m::PackedAxis::Group
            };
            let top_id = match axis {
                nanda_6m::PackedAxis::Route => 1 + ((field_idx as u16 + iter as u16) % 7),
                nanda_6m::PackedAxis::Group => 1 + ((field_idx as u16 + iter as u16) % 11),
            };
            let summary = nanda_6m::build_packed_support_field_from_scores(
                black_box(score_slice),
                axis,
                top_id,
                0x3000_0000u32.wrapping_add(field_idx as u32),
            );
            *field = summary.field;
            checksum = checksum
                .wrapping_add(summary.field.positive_dot as u64)
                .wrapping_add(summary.field.negative_dot as u64)
                .wrapping_add(u64::from(summary.considered))
                .wrapping_add(u64::from(summary.support_count))
                .wrapping_add(u64::from(summary.anti_count));
        }
        if with_compile_sweep {
            let sweep = nanda_6m::compile_and_apply_aligned_suppress_anti_lane_sweep(
                black_box(fields.as_slice()),
                black_box(lanes.as_mut_slice()),
            );
            checksum = checksum
                .wrapping_add(sweep.checksum)
                .wrapping_add(sweep.best_after_net_dot as u64)
                .wrapping_add(sweep.total_delta_dot as u64)
                .wrapping_add(sweep.focused as u64);
            black_box(sweep);
        }
    }
    let elapsed = start.elapsed();
    let mut out = bench_result_json(
        iterations,
        elapsed.as_nanos(),
        checksum,
        if with_compile_sweep {
            "build_support_scores_fields_and_compile_sweep"
        } else {
            "build_support_scores_and_fields"
        },
    );
    out["triads_in_memory"] = json!(memory.len());
    out["fields"] = json!(field_count);
    out["query_triads"] = json!(query_len);
    out["ns_per_field"] = json!(out["ns_per_op"].as_f64().unwrap_or(0.0) / field_count as f64);
    out
}

fn bench6m_support_bucket_build(
    iterations: u64,
    triad_count: usize,
    field_count: usize,
    with_compile_sweep: bool,
) -> Value {
    let iterations = iterations.max(1);
    let memory = bench6m_triads(triad_count);
    let query_len = memory.len().clamp(1, 8);
    let query = nanda_6m::project_triads(&memory[..query_len]);
    let mut scores = vec![nanda_6m::PackedTriadSupportScore::default(); memory.len()];
    let mut route_sorted = vec![nanda_6m::PackedTriadSupportScore::default(); memory.len()];
    let mut group_sorted = vec![nanda_6m::PackedTriadSupportScore::default(); memory.len()];
    let mut route_offsets = vec![0u16; nanda_6m::SCORE_BUCKET_CAPACITY + 1];
    let mut group_offsets = vec![0u16; nanda_6m::SCORE_BUCKET_CAPACITY + 1];
    let mut cursors = vec![0u16; nanda_6m::SCORE_BUCKET_CAPACITY];
    let mut fields = vec![nanda_6m::PackedSupportField::default(); field_count];
    let mut lanes = vec![nanda_6m::PackedLane64::default(); field_count];
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for iter in 0..iterations {
        let score_count = nanda_6m::build_packed_triad_support_scores(
            black_box(memory.as_slice()),
            black_box(&query),
            black_box(scores.as_mut_slice()),
        );
        let score_slice = &scores[..score_count];
        let route_count = nanda_6m::bucket_packed_triad_support_scores(
            black_box(score_slice),
            nanda_6m::PackedAxis::Route,
            black_box(route_sorted.as_mut_slice()),
            black_box(route_offsets.as_mut_slice()),
            black_box(cursors.as_mut_slice()),
        );
        let group_count = nanda_6m::bucket_packed_triad_support_scores(
            black_box(score_slice),
            nanda_6m::PackedAxis::Group,
            black_box(group_sorted.as_mut_slice()),
            black_box(group_offsets.as_mut_slice()),
            black_box(cursors.as_mut_slice()),
        );
        for (field_idx, field) in fields.iter_mut().enumerate() {
            let axis = if (field_idx + iter as usize).is_multiple_of(2) {
                nanda_6m::PackedAxis::Route
            } else {
                nanda_6m::PackedAxis::Group
            };
            let top_id = match axis {
                nanda_6m::PackedAxis::Route => 1 + ((field_idx as u16 + iter as u16) % 7),
                nanda_6m::PackedAxis::Group => 1 + ((field_idx as u16 + iter as u16) % 11),
            };
            let summary = match axis {
                nanda_6m::PackedAxis::Route => {
                    nanda_6m::build_packed_support_field_from_score_bucket(
                        black_box(&route_sorted[..route_count]),
                        black_box(&route_offsets),
                        axis,
                        top_id,
                        0x4000_0000u32.wrapping_add(field_idx as u32),
                    )
                }
                nanda_6m::PackedAxis::Group => {
                    nanda_6m::build_packed_support_field_from_score_bucket(
                        black_box(&group_sorted[..group_count]),
                        black_box(&group_offsets),
                        axis,
                        top_id,
                        0x4000_0000u32.wrapping_add(field_idx as u32),
                    )
                }
            };
            *field = summary.field;
            checksum = checksum
                .wrapping_add(summary.field.positive_dot as u64)
                .wrapping_add(summary.field.negative_dot as u64)
                .wrapping_add(u64::from(summary.considered))
                .wrapping_add(u64::from(summary.support_count))
                .wrapping_add(u64::from(summary.anti_count));
        }
        if with_compile_sweep {
            let sweep = nanda_6m::compile_and_apply_aligned_suppress_anti_lane_sweep(
                black_box(fields.as_slice()),
                black_box(lanes.as_mut_slice()),
            );
            checksum = checksum
                .wrapping_add(sweep.checksum)
                .wrapping_add(sweep.best_after_net_dot as u64)
                .wrapping_add(sweep.total_delta_dot as u64)
                .wrapping_add(sweep.focused as u64);
            black_box(sweep);
        }
    }
    let elapsed = start.elapsed();
    let mut out = bench_result_json(
        iterations,
        elapsed.as_nanos(),
        checksum,
        if with_compile_sweep {
            "build_support_score_buckets_fields_and_compile_sweep"
        } else {
            "build_support_score_buckets_and_fields"
        },
    );
    out["triads_in_memory"] = json!(memory.len());
    out["fields"] = json!(field_count);
    out["query_triads"] = json!(query_len);
    out["ns_per_field"] = json!(out["ns_per_op"].as_f64().unwrap_or(0.0) / field_count as f64);
    out
}

fn bench6m_hot_cycle(iterations: u64, triad_count: usize, field_count: usize) -> Value {
    let iterations = iterations.max(1);
    let memory = bench6m_triads(triad_count);
    let query_len = memory.len().clamp(1, 8);
    let query = nanda_6m::project_triads(&memory[..query_len]);
    let requests = bench6m_field_requests(field_count);
    let mut scores = vec![nanda_6m::PackedTriadSupportScore::default(); memory.len()];
    let mut route_sorted = vec![nanda_6m::PackedTriadSupportScore::default(); memory.len()];
    let mut group_sorted = vec![nanda_6m::PackedTriadSupportScore::default(); memory.len()];
    let mut route_offsets = vec![0u16; nanda_6m::SCORE_BUCKET_CAPACITY + 1];
    let mut group_offsets = vec![0u16; nanda_6m::SCORE_BUCKET_CAPACITY + 1];
    let mut cursors = vec![0u16; nanda_6m::SCORE_BUCKET_CAPACITY];
    let mut fields = vec![nanda_6m::PackedSupportField::default(); field_count];
    let mut lanes = vec![nanda_6m::PackedLane64::default(); field_count];
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for _ in 0..iterations {
        let mut workspace = nanda_6m::PackedHotWorkspace {
            buckets: nanda_6m::PackedBucketWorkspace {
                scores_out: scores.as_mut_slice(),
                route_sorted_out: route_sorted.as_mut_slice(),
                route_offsets_out: route_offsets.as_mut_slice(),
                group_sorted_out: group_sorted.as_mut_slice(),
                group_offsets_out: group_offsets.as_mut_slice(),
                cursors_out: cursors.as_mut_slice(),
            },
            fields_out: fields.as_mut_slice(),
            lanes_out: lanes.as_mut_slice(),
        };
        let cycle = nanda_6m::run_packed_hot_cycle(
            black_box(memory.as_slice()),
            black_box(&query),
            black_box(requests.as_slice()),
            black_box(&mut workspace),
        );
        checksum = checksum
            .wrapping_add(cycle.checksum)
            .wrapping_add(u64::from(cycle.score_count))
            .wrapping_add(u64::from(cycle.route_count))
            .wrapping_add(u64::from(cycle.group_count))
            .wrapping_add(u64::from(cycle.fields_built))
            .wrapping_add(u64::from(cycle.lanes_compiled));
        black_box(cycle);
    }
    let elapsed = start.elapsed();
    let mut out = bench_result_json(
        iterations,
        elapsed.as_nanos(),
        checksum,
        "run_packed_hot_cycle",
    );
    out["triads_in_memory"] = json!(memory.len());
    out["fields"] = json!(field_count);
    out["query_triads"] = json!(query_len);
    out["ns_per_field"] = json!(out["ns_per_op"].as_f64().unwrap_or(0.0) / field_count as f64);
    out
}

fn bench6m_projection(iterations: u64, triad_count: usize) -> Value {
    let iterations = iterations.max(1);
    let triads = bench6m_triads(triad_count);
    let query_len = triads.len().clamp(1, 8);
    let mut checksum: u64 = 0;
    let start = Instant::now();
    for idx in 0..iterations {
        let rotate = (idx as usize) % triads.len();
        let query = nanda_6m::project_triads(black_box(&triads[..query_len]));
        let centroid = if rotate + query_len <= triads.len() {
            nanda_6m::centroid_from_triads(black_box(&triads[rotate..rotate + query_len]))
        } else {
            nanda_6m::centroid_from_triads(black_box(&triads[..query_len]))
        };
        let score = nanda_6m::score_centroid(&query, &centroid);
        checksum = checksum
            .wrapping_add(score.dot as u64)
            .wrapping_add(score.query_energy as u64)
            .wrapping_add(score.centroid_energy as u64);
        black_box(score);
    }
    let elapsed = start.elapsed();
    let mut out = bench_result_json(
        iterations,
        elapsed.as_nanos(),
        checksum,
        "project+centroid+score",
    );
    out["triads_in_window"] = json!(triad_count);
    out["query_triads"] = json!(query_len);
    out
}

fn bench6m_support_fields(count: usize) -> Vec<nanda_6m::PackedSupportField> {
    (0..count)
        .map(|idx| {
            let idx = idx as u32;
            let positive_dot = 96 + i64::from((idx % 8) * 32);
            let negative_dot = if idx.is_multiple_of(3) {
                -i64::from(64 + (idx % 5) * 32)
            } else if idx.is_multiple_of(5) {
                -32
            } else {
                0
            };
            let anti_mask = if negative_dot < 0 {
                1u64 << ((idx % 63) + 1)
            } else {
                0
            };
            nanda_6m::PackedSupportField {
                top_id: (1 + (idx % 127)) as u16,
                key_hash: 0x1000_0000u32.wrapping_add(idx.wrapping_mul(97)),
                positive_dot,
                negative_dot,
                support_mask_a: 1u64 << (idx % 64),
                support_mask_b: 0,
                anti_mask_a: anti_mask,
                anti_mask_b: 0,
            }
        })
        .collect()
}

fn bench6m_field_requests(count: usize) -> Vec<nanda_6m::PackedFieldRequest> {
    (0..count)
        .map(|idx| {
            let key_hash = 0x5000_0000u32.wrapping_add(idx as u32);
            if idx.is_multiple_of(2) {
                nanda_6m::PackedFieldRequest::route(1 + (idx as u16 % 7), key_hash)
            } else {
                nanda_6m::PackedFieldRequest::group(1 + (idx as u16 % 11), key_hash)
            }
        })
        .collect()
}

fn bench_result_json(iterations: u64, elapsed_ns: u128, checksum: u64, kernel: &str) -> Value {
    let ns_per_op = elapsed_ns as f64 / iterations as f64;
    json!({
        "kernel": kernel,
        "iterations": iterations,
        "elapsed_ns": elapsed_ns,
        "ns_per_op": ns_per_op,
        "ops_per_second": 1_000_000_000.0 / ns_per_op.max(0.000001),
        "checksum": checksum
    })
}

fn bench6m_triads(count: usize) -> Vec<nanda_6m::PackedTriad32> {
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

fn print_bench6m_text(out: &Value) {
    println!("NANDA-6M HOT BENCH");
    println!(
        "core: {} / {}",
        out["core_version"].as_str().unwrap_or(""),
        out["nanda_6m_version"].as_str().unwrap_or("")
    );
    if !out["benchmarks"]["replay"].is_null() {
        let replay = &out["benchmarks"]["replay"];
        println!(
            "replay: {} iters / {:.2} ns/op / {:.0} ops/s",
            replay["iterations"].as_u64().unwrap_or(0),
            replay["ns_per_op"].as_f64().unwrap_or(0.0),
            replay["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["projection"].is_null() {
        let projection = &out["benchmarks"]["projection"];
        println!(
            "projection: {} iters / {} triads / {:.2} ns/op / {:.0} ops/s",
            projection["iterations"].as_u64().unwrap_or(0),
            projection["triads_in_window"].as_u64().unwrap_or(0),
            projection["ns_per_op"].as_f64().unwrap_or(0.0),
            projection["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["lane_application"].is_null() {
        let lane = &out["benchmarks"]["lane_application"];
        println!(
            "lane: {} iters / {:.2} ns/op / {:.0} ops/s",
            lane["iterations"].as_u64().unwrap_or(0),
            lane["ns_per_op"].as_f64().unwrap_or(0.0),
            lane["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["lane_sweep"].is_null() {
        let sweep = &out["benchmarks"]["lane_sweep"];
        println!(
            "lane-sweep: {} iters / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            sweep["iterations"].as_u64().unwrap_or(0),
            sweep["fields"].as_u64().unwrap_or(0),
            sweep["ns_per_op"].as_f64().unwrap_or(0.0),
            sweep["ns_per_field"].as_f64().unwrap_or(0.0),
            sweep["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["aligned_lane_sweep"].is_null() {
        let sweep = &out["benchmarks"]["aligned_lane_sweep"];
        println!(
            "aligned-lane-sweep: {} iters / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            sweep["iterations"].as_u64().unwrap_or(0),
            sweep["fields"].as_u64().unwrap_or(0),
            sweep["ns_per_op"].as_f64().unwrap_or(0.0),
            sweep["ns_per_field"].as_f64().unwrap_or(0.0),
            sweep["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["aligned_compile_sweep"].is_null() {
        let sweep = &out["benchmarks"]["aligned_compile_sweep"];
        println!(
            "aligned-compile-sweep: {} iters / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            sweep["iterations"].as_u64().unwrap_or(0),
            sweep["fields"].as_u64().unwrap_or(0),
            sweep["ns_per_op"].as_f64().unwrap_or(0.0),
            sweep["ns_per_field"].as_f64().unwrap_or(0.0),
            sweep["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_build"].is_null() {
        let support = &out["benchmarks"]["support_build"];
        println!(
            "support-build: {} iters / {} triads / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0),
            support["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_build_compile_sweep"].is_null() {
        let support = &out["benchmarks"]["support_build_compile_sweep"];
        println!(
            "support-build+compile-sweep: {} iters / {} triads / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0),
            support["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_score_build"].is_null() {
        let support = &out["benchmarks"]["support_score_build"];
        println!(
            "support-score-build: {} iters / {} triads / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0),
            support["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_score_build_compile_sweep"].is_null() {
        let support = &out["benchmarks"]["support_score_build_compile_sweep"];
        println!(
            "support-score-build+compile-sweep: {} iters / {} triads / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0),
            support["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_bucket_build"].is_null() {
        let support = &out["benchmarks"]["support_bucket_build"];
        println!(
            "support-bucket-build: {} iters / {} triads / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0),
            support["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_bucket_build_compile_sweep"].is_null() {
        let support = &out["benchmarks"]["support_bucket_build_compile_sweep"];
        println!(
            "support-bucket-build+compile-sweep: {} iters / {} triads / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0),
            support["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["hot_cycle"].is_null() {
        let support = &out["benchmarks"]["hot_cycle"];
        println!(
            "hot-cycle: {} iters / {} triads / {} fields / {:.2} ns/op / {:.2} ns/field / {:.0} ops/s",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0),
            support["ops_per_second"].as_f64().unwrap_or(0.0)
        );
    }
    println!("scope: no JSON, no file IO, no process spawn");
}

fn print_bench6m_md(out: &Value) {
    println!("# NANDA-6M Hot Bench\n");
    println!("- core: `{}`", out["core_version"].as_str().unwrap_or(""));
    println!(
        "- nanda_6m: `{}`",
        out["nanda_6m_version"].as_str().unwrap_or("")
    );
    if !out["benchmarks"]["replay"].is_null() {
        let replay = &out["benchmarks"]["replay"];
        println!(
            "- replay: `{}` iterations, `{:.2}` ns/op",
            replay["iterations"].as_u64().unwrap_or(0),
            replay["ns_per_op"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["projection"].is_null() {
        let projection = &out["benchmarks"]["projection"];
        println!(
            "- projection: `{}` iterations, `{}` triads, `{:.2}` ns/op",
            projection["iterations"].as_u64().unwrap_or(0),
            projection["triads_in_window"].as_u64().unwrap_or(0),
            projection["ns_per_op"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["lane_application"].is_null() {
        let lane = &out["benchmarks"]["lane_application"];
        println!(
            "- lane: `{}` iterations, `{:.2}` ns/op",
            lane["iterations"].as_u64().unwrap_or(0),
            lane["ns_per_op"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["lane_sweep"].is_null() {
        let sweep = &out["benchmarks"]["lane_sweep"];
        println!(
            "- lane-sweep: `{}` iterations, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            sweep["iterations"].as_u64().unwrap_or(0),
            sweep["fields"].as_u64().unwrap_or(0),
            sweep["ns_per_op"].as_f64().unwrap_or(0.0),
            sweep["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["aligned_lane_sweep"].is_null() {
        let sweep = &out["benchmarks"]["aligned_lane_sweep"];
        println!(
            "- aligned-lane-sweep: `{}` iterations, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            sweep["iterations"].as_u64().unwrap_or(0),
            sweep["fields"].as_u64().unwrap_or(0),
            sweep["ns_per_op"].as_f64().unwrap_or(0.0),
            sweep["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["aligned_compile_sweep"].is_null() {
        let sweep = &out["benchmarks"]["aligned_compile_sweep"];
        println!(
            "- aligned-compile-sweep: `{}` iterations, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            sweep["iterations"].as_u64().unwrap_or(0),
            sweep["fields"].as_u64().unwrap_or(0),
            sweep["ns_per_op"].as_f64().unwrap_or(0.0),
            sweep["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_build"].is_null() {
        let support = &out["benchmarks"]["support_build"];
        println!(
            "- support-build: `{}` iterations, `{}` triads, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_build_compile_sweep"].is_null() {
        let support = &out["benchmarks"]["support_build_compile_sweep"];
        println!(
            "- support-build+compile-sweep: `{}` iterations, `{}` triads, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_score_build"].is_null() {
        let support = &out["benchmarks"]["support_score_build"];
        println!(
            "- support-score-build: `{}` iterations, `{}` triads, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_score_build_compile_sweep"].is_null() {
        let support = &out["benchmarks"]["support_score_build_compile_sweep"];
        println!(
            "- support-score-build+compile-sweep: `{}` iterations, `{}` triads, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_bucket_build"].is_null() {
        let support = &out["benchmarks"]["support_bucket_build"];
        println!(
            "- support-bucket-build: `{}` iterations, `{}` triads, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["support_bucket_build_compile_sweep"].is_null() {
        let support = &out["benchmarks"]["support_bucket_build_compile_sweep"];
        println!(
            "- support-bucket-build+compile-sweep: `{}` iterations, `{}` triads, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    if !out["benchmarks"]["hot_cycle"].is_null() {
        let support = &out["benchmarks"]["hot_cycle"];
        println!(
            "- hot-cycle: `{}` iterations, `{}` triads, `{}` fields, `{:.2}` ns/op, `{:.2}` ns/field",
            support["iterations"].as_u64().unwrap_or(0),
            support["triads_in_memory"].as_u64().unwrap_or(0),
            support["fields"].as_u64().unwrap_or(0),
            support["ns_per_op"].as_f64().unwrap_or(0.0),
            support["ns_per_field"].as_f64().unwrap_or(0.0)
        );
    }
    println!("- scope: no JSON, no file IO, no process spawn");
}
