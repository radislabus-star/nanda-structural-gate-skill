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
    #[arg(long, default_value_t = 64)]
    triads: usize,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Clone, ValueEnum)]
enum Bench6mMode {
    Replay,
    Projection,
    All,
}

pub(super) fn cmd(args: Bench6mArgs) -> Result<u8> {
    let include_replay = matches!(args.mode, Bench6mMode::Replay | Bench6mMode::All);
    let include_projection = matches!(args.mode, Bench6mMode::Projection | Bench6mMode::All);
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
    let out = json!({
        "mode": "nanda-6m-hot-benchmark",
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "budget_bytes": nanda_6m::BUDGET_BYTES,
        "wave_dim": nanda_6m::WAVE_DIM,
        "benchmarks": {
            "replay": replay,
            "projection": projection
        },
        "interpretation": {
            "replay": "Pure typed replay firewall; no JSON, no file IO, no process spawn.",
            "projection": "Packed 1024-dimensional projection plus centroid scoring over an in-memory triad window.",
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
    println!("- scope: no JSON, no file IO, no process spawn");
}
