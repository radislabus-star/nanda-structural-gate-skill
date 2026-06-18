use crate::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

use super::{
    nanda_6m, negative_lane_match_ratio, norm, normalized_shortcut_terms,
    positive_lane_match_ratio, round4, triad_polarity, NegativeShortcut, Packet, PositiveShortcut,
    Triad, CORE_VERSION, WAVE_DIM,
};

pub(crate) fn budget_cmd(args: BudgetArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let out = budget_report(&packet, &source, &candidates);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_budget_text(&out),
        OutputFormat::Md => print_budget_md(&out),
    }
    Ok(if out["fits_l3"].as_bool().unwrap_or(false) {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

pub(crate) fn pack6m_cmd(args: Pack6mArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let out = pack_report(&packet, &source, &candidates, args.sample);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_pack6m_text(&out),
        OutputFormat::Md => print_pack6m_md(&out),
    }
    Ok(if out["packed_ok"].as_bool().unwrap_or(false) {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

pub(super) fn budget_report(packet: &Packet, source: &[Triad], candidates: &[Triad]) -> Value {
    let active_triads = source.len() + candidates.len();
    let active_lanes = packet.negative_shortcuts.len() + packet.positive_shortcuts.len();
    let mut entities = BTreeSet::<String>::new();
    let mut relations = BTreeSet::<String>::new();
    let mut routes = BTreeSet::<String>::new();
    let mut groups = BTreeSet::<String>::new();
    let mut evidence_refs = BTreeSet::<String>::new();
    let mut cold_labels = BTreeSet::<String>::new();

    for triad in source.iter().chain(candidates.iter()) {
        insert_label(&mut entities, &triad.subject);
        insert_label(&mut entities, &triad.object);
        insert_label(&mut relations, &triad.relation);
        insert_label(&mut evidence_refs, &triad.evidence);
        insert_label(&mut cold_labels, &triad.id);
        insert_label(&mut cold_labels, &triad.subject);
        insert_label(&mut cold_labels, &triad.relation);
        insert_label(&mut cold_labels, &triad.object);
        insert_label(&mut cold_labels, &triad.evidence);
        insert_label(&mut cold_labels, &triad.subject_role);
        insert_label(&mut cold_labels, &triad.object_role);

        let route = if triad.route.trim().is_empty() {
            "__route_default"
        } else {
            triad.route.trim()
        };
        let group = if triad.group.trim().is_empty() {
            "__group_default"
        } else {
            triad.group.trim()
        };
        insert_label(&mut routes, route);
        insert_label(&mut groups, group);
        insert_label(&mut cold_labels, route);
        insert_label(&mut cold_labels, group);
    }

    for lane in &packet.negative_shortcuts {
        insert_label(&mut cold_labels, &lane.id);
        insert_label(&mut cold_labels, &lane.suppress_peak);
        insert_label(&mut cold_labels, &lane.suppress_route);
        insert_label(&mut cold_labels, &lane.suppress_group);
        insert_label(&mut cold_labels, &lane.prefer_peak);
        insert_label(&mut cold_labels, &lane.prefer_route);
        insert_label(&mut cold_labels, &lane.prefer_group);
        insert_label(&mut cold_labels, &lane.reason);
        insert_label(&mut cold_labels, &lane.source_feedback);
        insert_label(&mut routes, &lane.suppress_route);
        insert_label(&mut routes, &lane.prefer_route);
        insert_label(&mut groups, &lane.suppress_group);
        insert_label(&mut groups, &lane.prefer_group);
        for term in lane.terms.iter().chain(lane.support_terms.iter()) {
            insert_label(&mut cold_labels, term);
        }
    }
    for lane in &packet.positive_shortcuts {
        insert_label(&mut cold_labels, &lane.id);
        insert_label(&mut cold_labels, &lane.reinforce_peak);
        insert_label(&mut cold_labels, &lane.reinforce_route);
        insert_label(&mut cold_labels, &lane.reinforce_group);
        insert_label(&mut cold_labels, &lane.reason);
        insert_label(&mut cold_labels, &lane.source_feedback);
        insert_label(&mut routes, &lane.reinforce_route);
        insert_label(&mut groups, &lane.reinforce_group);
        for term in lane.terms.iter().chain(lane.support_terms.iter()) {
            insert_label(&mut cold_labels, term);
        }
    }

    let centroid_count = routes.len() + groups.len();
    let budget_usage = nanda_6m::BudgetUsage {
        active_triads,
        centroids: centroid_count,
        lanes: active_lanes,
    };
    let estimated_hot_bytes = budget_usage.estimated_hot_bytes();
    let reserved_core_bytes = nanda_6m::RESERVED_CORE_BYTES;
    let cold_dictionary_bytes = cold_labels
        .iter()
        .map(|label| label.len() + 8)
        .sum::<usize>();
    let triads_ok = active_triads <= nanda_6m::TRIAD_CAPACITY;
    let centroids_ok = centroid_count <= nanda_6m::CENTROID_CAPACITY;
    let lanes_ok = active_lanes <= nanda_6m::LANE_CAPACITY;
    let hot_bytes_ok = estimated_hot_bytes <= nanda_6m::BUDGET_BYTES;
    let fits_l3 = budget_usage.fits();
    let mut blockers = vec![];
    if !triads_ok {
        blockers.push(json!({
            "state": "TOO_MANY_TRIADS",
            "count": active_triads,
            "capacity": nanda_6m::TRIAD_CAPACITY,
            "repair": "build a route-balanced focus packet or split by linked group"
        }));
    }
    if !centroids_ok {
        blockers.push(json!({
            "state": "TOO_MANY_CENTROIDS",
            "count": centroid_count,
            "capacity": nanda_6m::CENTROID_CAPACITY,
            "repair": "merge aliases, normalize routes/groups, or split topology"
        }));
    }
    if !lanes_ok {
        blockers.push(json!({
            "state": "TOO_MANY_LANES",
            "count": active_lanes,
            "capacity": nanda_6m::LANE_CAPACITY,
            "repair": "keep only active positive/negative lanes for this focus packet"
        }));
    }
    if !hot_bytes_ok {
        blockers.push(json!({
            "state": "SPILL_REQUIRED",
            "estimated_hot_bytes": estimated_hot_bytes,
            "budget_bytes": nanda_6m::BUDGET_BYTES,
            "repair": "reduce active triads, centroids, or lanes before hot execution"
        }));
    }
    let state = if fits_l3 {
        "FITS_L3"
    } else if !hot_bytes_ok {
        "SPILL_REQUIRED"
    } else if !centroids_ok {
        "SPLIT_REQUIRED"
    } else {
        "FOCUS_REQUIRED"
    };
    json!({
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "mode": "nanda-6m-budget-planner",
        "state": state,
        "verdict": if fits_l3 { "PASS" } else { "WATCH" },
        "canonicalization": packet.canonicalization,
        "fits_l3": fits_l3,
        "safe_for_hot_core": fits_l3,
        "hard_budget_bytes": nanda_6m::BUDGET_BYTES,
        "reserved_core_bytes": reserved_core_bytes,
        "estimated_hot_bytes": estimated_hot_bytes,
        "remaining_hot_bytes": nanda_6m::BUDGET_BYTES.saturating_sub(estimated_hot_bytes),
        "cold_dictionary_bytes": cold_dictionary_bytes,
        "cold_dictionary_note": "String labels, evidence text, JSON, and source snippets stay outside the NANDA-6M hot core.",
        "wave_dim": WAVE_DIM,
        "record_sizes": {
            "packed_triad_bytes": nanda_6m::TRIAD_BYTES,
            "centroid_bytes": nanda_6m::CENTROID_BYTES,
            "lane_bytes": nanda_6m::LANE_BYTES
        },
        "capacity": {
            "triads": nanda_6m::TRIAD_CAPACITY,
            "centroids": nanda_6m::CENTROID_CAPACITY,
            "lanes": nanda_6m::LANE_CAPACITY
        },
        "counts": {
            "source_triads": source.len(),
            "candidate_triads": candidates.len(),
            "active_triads": active_triads,
            "entities": entities.len(),
            "relations": relations.len(),
            "routes": routes.len(),
            "groups": groups.len(),
            "centroids": centroid_count,
            "evidence_refs": evidence_refs.len(),
            "negative_lanes": packet.negative_shortcuts.len(),
            "positive_lanes": packet.positive_shortcuts.len(),
            "active_lanes": active_lanes
        },
        "usage": {
            "triad_arena": usage_row(active_triads, nanda_6m::TRIAD_CAPACITY, active_triads * nanda_6m::TRIAD_BYTES, nanda_6m::TRIAD_ARENA_BYTES),
            "centroid_arena": usage_row(centroid_count, nanda_6m::CENTROID_CAPACITY, centroid_count * nanda_6m::CENTROID_BYTES, nanda_6m::CENTROID_ARENA_BYTES),
            "lane_arena": usage_row(active_lanes, nanda_6m::LANE_CAPACITY, active_lanes * nanda_6m::LANE_BYTES, nanda_6m::LANE_ARENA_BYTES),
            "workspace": usage_row(1, 1, nanda_6m::WORKSPACE_BYTES, nanda_6m::WORKSPACE_BYTES),
            "index_stats": usage_row(1, 1, nanda_6m::INDEX_STATS_BYTES, nanda_6m::INDEX_STATS_BYTES)
        },
        "blockers": blockers,
        "next": if fits_l3 {
            "Packet can enter the future NANDA-6M packed hot core."
        } else {
            "Do not run as one hot packet; focus, split, or reduce lanes before NANDA-6M execution."
        }
    })
}

pub(super) fn pack_report(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
    sample: usize,
) -> Value {
    let budget = budget_report(packet, source, candidates);
    let mut entities = IdDictionary::default();
    let mut relations = IdDictionary::default();
    let mut routes = IdDictionary::default();
    let mut groups = IdDictionary::default();
    let mut evidences = IdDictionary::default();
    let mut roles = IdDictionary::default();
    let mut memory_packed = Vec::with_capacity(source.len());
    let mut query_packed = Vec::with_capacity(candidates.len());
    let mut blockers = vec![];

    for triad in source {
        match pack_triad6m(
            triad,
            0x0001,
            &mut entities,
            &mut relations,
            &mut routes,
            &mut groups,
            &mut evidences,
            &mut roles,
        ) {
            Ok(record) => memory_packed.push(record),
            Err(err) => blockers.push(err),
        }
    }
    for triad in candidates {
        match pack_triad6m(
            triad,
            0x0002,
            &mut entities,
            &mut relations,
            &mut routes,
            &mut groups,
            &mut evidences,
            &mut roles,
        ) {
            Ok(record) => query_packed.push(record),
            Err(err) => blockers.push(err),
        }
    }

    let query_records = if query_packed.is_empty() {
        &memory_packed
    } else {
        &query_packed
    };
    let projection_source = if query_packed.is_empty() {
        "memory_fallback"
    } else {
        "candidate_triads"
    };
    let packed_count = memory_packed.len() + query_packed.len();
    let sample_records = memory_packed
        .iter()
        .chain(query_packed.iter())
        .take(sample)
        .map(packed_triad_json)
        .collect::<Vec<_>>();
    let projection = nanda_6m::project_triads(query_records);
    let projection_summary = projection.summary();
    let route_centroids =
        packed_centroid_report(&memory_packed, &projection, CentroidAxis6m::Route, sample);
    let group_centroids =
        packed_centroid_report(&memory_packed, &projection, CentroidAxis6m::Group, sample);
    let route_peak = packed_peak_summary(&route_centroids);
    let group_peak = packed_peak_summary(&group_centroids);
    let packed_support = packed_support_report(
        &memory_packed,
        &projection,
        &route_peak,
        &group_peak,
        sample,
    );
    let packed_lane_keys = packed_lane_keys_report(&packed_support);
    let packed_lanes = packed_lane_preview(&packed_support, &packed_lane_keys);
    let packed_lane_store = packed_lane_store_report(&packed_lane_keys, &packed_lanes);
    let peak_decision = packed_field_decision(
        &route_peak,
        &group_peak,
        projection_summary.energy,
        memory_packed.len(),
        query_packed.len(),
    );
    let packed_lane_application =
        packed_lane_application_report(&packed_support, &packed_lanes, &peak_decision);
    let packed_lane_replay = packed_lane_replay_report(
        packet,
        source,
        candidates,
        &packed_lane_store,
        &routes,
        &groups,
    );
    let packed_replay_decision = packed_replay_decision_report(&peak_decision, &packed_lane_replay);
    let dictionary_ok = blockers.is_empty()
        && relations.len() <= u16::MAX as usize
        && routes.len() <= u16::MAX as usize
        && groups.len() <= u16::MAX as usize
        && roles.len() <= u8::MAX as usize;
    let packed_ok = budget["fits_l3"].as_bool().unwrap_or(false) && dictionary_ok;
    json!({
        "core_version": CORE_VERSION,
        "nanda_6m_version": nanda_6m::VERSION,
        "mode": "nanda-6m-pack-skeleton",
        "state": if packed_ok { "PACKED_FITS_L3" } else { "PACK_REVIEW_REQUIRED" },
        "verdict": if packed_ok { "PASS" } else { "WATCH" },
        "packed_ok": packed_ok,
        "canonicalization": packet.canonicalization,
        "budget": budget,
        "packed_records": {
            "count": packed_count,
            "memory_count": memory_packed.len(),
            "query_count": query_packed.len(),
            "bytes": packed_count * nanda_6m::TRIAD_BYTES,
            "record_bytes": nanda_6m::TRIAD_BYTES,
            "sample": sample_records
        },
        "projection": {
            "mode": "packed-triad-signed-hash",
            "source": projection_source,
            "records": query_records.len(),
            "wave_dim": nanda_6m::WAVE_DIM,
            "bytes": nanda_6m::QUERY_WAVE_BYTES,
            "summary": {
                "l1": projection_summary.l1,
                "energy": projection_summary.energy,
                "nonzero": projection_summary.nonzero,
                "max_abs": projection_summary.max_abs
            },
            "sample": projection.values.iter().take(8).copied().collect::<Vec<_>>()
        },
        "centroids": {
            "source": "memory_triads",
            "record_bytes": nanda_6m::CENTROID_BYTES,
            "route_count": route_centroids.len(),
            "group_count": group_centroids.len(),
            "total_count": route_centroids.len() + group_centroids.len(),
            "route": route_centroids,
            "group": group_centroids
        },
        "peaks": {
            "mode": "packed-candidate-query-vs-memory-centroid-cosine",
            "route": route_peak,
            "group": group_peak
        },
        "packed_support": packed_support,
        "packed_lane_keys": packed_lane_keys,
        "packed_lanes": packed_lanes,
        "packed_lane_store": packed_lane_store,
        "packed_lane_application": packed_lane_application,
        "packed_lane_replay": packed_lane_replay,
        "packed_replay_decision": packed_replay_decision,
        "peak_decision": peak_decision,
        "dictionaries": {
            "entities": dictionary_summary(&entities, u32::MAX as usize),
            "relations": dictionary_summary(&relations, u16::MAX as usize),
            "routes": dictionary_summary(&routes, u16::MAX as usize),
            "groups": dictionary_summary(&groups, u16::MAX as usize),
            "evidences": dictionary_summary(&evidences, u32::MAX as usize),
            "roles": dictionary_summary(&roles, u8::MAX as usize)
        },
        "blockers": blockers,
        "hot_core_note": "This command still runs in the cold layer. It now separates memory/source centroids from the candidate/query projection wave, proving the first honest packed peak path before full packed interference search."
    })
}

#[derive(Clone, Copy)]
enum CentroidAxis6m {
    Route,
    Group,
}

fn packed_centroid_report(
    packed: &[nanda_6m::PackedTriad32],
    query: &nanda_6m::PackedWave1024,
    axis: CentroidAxis6m,
    sample: usize,
) -> Vec<Value> {
    let mut by_id: BTreeMap<u16, Vec<nanda_6m::PackedTriad32>> = BTreeMap::new();
    for triad in packed {
        let id = match axis {
            CentroidAxis6m::Route => triad.route_id,
            CentroidAxis6m::Group => triad.group_id,
        };
        by_id.entry(id).or_default().push(*triad);
    }
    let mut rows = by_id
        .iter()
        .map(|(id, triads)| {
            let centroid = nanda_6m::centroid_from_triads(triads);
            let summary = centroid.summary();
            let score = nanda_6m::score_centroid(query, &centroid);
            json!({
                "id": id,
                "triads": triads.len(),
                "score": {
                    "cosine": round4(score.cosine),
                    "dot": score.dot,
                    "query_energy": score.query_energy,
                    "centroid_energy": score.centroid_energy
                },
                "summary": {
                    "l1": summary.l1,
                    "energy": summary.energy,
                    "nonzero": summary.nonzero,
                    "max_abs": summary.max_abs
                },
                "sample": centroid.values.iter().take(8).copied().collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]["cosine"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"]["cosine"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(sample);
    rows
}

fn packed_peak_summary(rows: &[Value]) -> Value {
    let top = rows.first();
    let second = rows.get(1);
    let top_score = top
        .and_then(|item| item["score"]["cosine"].as_f64())
        .unwrap_or(0.0);
    let second_score = second
        .and_then(|item| item["score"]["cosine"].as_f64())
        .unwrap_or(0.0);
    let margin = top_score - second_score;
    let state = if top.is_none() || top_score <= 0.0 {
        "NO_PEAK"
    } else if top_score < 0.01 {
        "PEAK_THIN"
    } else if margin < 0.003 {
        "PEAK_CONTESTED"
    } else {
        "PEAK_FOUND"
    };
    json!({
        "top_id": top.and_then(|item| item["id"].as_u64()).unwrap_or(0),
        "top_score": round4(top_score),
        "second_id": second.and_then(|item| item["id"].as_u64()).unwrap_or(0),
        "second_score": round4(second_score),
        "margin": round4(margin),
        "state": state
    })
}

fn packed_field_decision(
    route_peak: &Value,
    group_peak: &Value,
    query_energy: u64,
    memory_count: usize,
    query_count: usize,
) -> Value {
    let route_state = route_peak["state"].as_str().unwrap_or("NO_PEAK");
    let group_state = group_peak["state"].as_str().unwrap_or("NO_PEAK");
    let route_id = route_peak["top_id"].as_u64().unwrap_or(0);
    let group_id = group_peak["top_id"].as_u64().unwrap_or(0);
    let route_score = route_peak["top_score"].as_f64().unwrap_or(0.0);
    let group_score = group_peak["top_score"].as_f64().unwrap_or(0.0);
    let route_margin = route_peak["margin"].as_f64().unwrap_or(0.0);
    let group_margin = group_peak["margin"].as_f64().unwrap_or(0.0);

    let (state, verdict, safe_to_answer, reason) = if memory_count == 0 {
        (
            "PACKED_EMPTY_MEMORY",
            "WATCH",
            false,
            "No memory/source triads were packed, so centroids cannot be trusted.",
        )
    } else if query_count == 0 {
        (
            "PACKED_MEMORY_FALLBACK",
            "WATCH",
            false,
            "No candidate/query triads were packed; projection uses memory fallback for diagnostics only.",
        )
    } else if query_energy == 0 {
        (
            "PACKED_EMPTY_QUERY",
            "WATCH",
            false,
            "Candidate/query projection has zero energy.",
        )
    } else if route_state == "NO_PEAK" || group_state == "NO_PEAK" {
        (
            "PACKED_NO_PEAK",
            "WATCH",
            false,
            "At least one centroid axis has no positive peak.",
        )
    } else if route_state == "PEAK_THIN" || group_state == "PEAK_THIN" {
        (
            "PACKED_THIN",
            "WATCH",
            false,
            "A peak exists, but cosine strength is below the packed focus threshold.",
        )
    } else if route_state == "PEAK_CONTESTED" || group_state == "PEAK_CONTESTED" {
        (
            "PACKED_CONTESTED",
            "WATCH",
            false,
            "Top centroid is too close to the runner-up.",
        )
    } else {
        (
            "PACKED_FOCUSED",
            "PASS",
            true,
            "Route and group axes both expose strong packed peaks.",
        )
    };

    json!({
        "state": state,
        "verdict": verdict,
        "safe_to_answer": safe_to_answer,
        "reason": reason,
        "thresholds": {
            "min_focus_score": 0.01,
            "min_focus_margin": 0.003
        },
        "query_energy": query_energy,
        "memory_records": memory_count,
        "query_records": query_count,
        "route": {
            "top_id": route_id,
            "state": route_state,
            "top_score": round4(route_score),
            "margin": round4(route_margin)
        },
        "group": {
            "top_id": group_id,
            "state": group_state,
            "top_score": round4(group_score),
            "margin": round4(group_margin)
        }
    })
}

fn packed_support_report(
    memory: &[nanda_6m::PackedTriad32],
    query: &nanda_6m::PackedWave1024,
    route_peak: &Value,
    group_peak: &Value,
    sample: usize,
) -> Value {
    json!({
        "mode": "query-vs-memory-triad-contributors",
        "route": packed_axis_support(
            memory,
            query,
            CentroidAxis6m::Route,
            route_peak["top_id"].as_u64().unwrap_or(0) as u16,
            sample
        ),
        "group": packed_axis_support(
            memory,
            query,
            CentroidAxis6m::Group,
            group_peak["top_id"].as_u64().unwrap_or(0) as u16,
            sample
        )
    })
}

fn packed_axis_support(
    memory: &[nanda_6m::PackedTriad32],
    query: &nanda_6m::PackedWave1024,
    axis: CentroidAxis6m,
    top_id: u16,
    sample: usize,
) -> Value {
    let mut rows = memory
        .iter()
        .enumerate()
        .filter_map(|(index, triad)| {
            let id = match axis {
                CentroidAxis6m::Route => triad.route_id,
                CentroidAxis6m::Group => triad.group_id,
            };
            if id != top_id || top_id == 0 {
                return None;
            }
            let centroid = nanda_6m::centroid_from_triads(std::slice::from_ref(triad));
            let score = nanda_6m::score_centroid(query, &centroid);
            Some(json!({
                "record_index": index,
                "route_id": triad.route_id,
                "group_id": triad.group_id,
                "relation_id": triad.relation_id,
                "subject_id": triad.subject_id,
                "object_id": triad.object_id,
                "evidence_ref": triad.evidence_ref,
                "dot": score.dot,
                "cosine": round4(score.cosine),
                "polarity": triad.polarity,
                "confidence": triad.confidence,
                "wave_seed": triad.wave_seed,
                "check": triad.check
            }))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["cosine"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["cosine"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let considered = rows.len();
    let mut positive_dot: i64 = 0;
    let mut negative_dot: i64 = 0;
    let mut support_count = 0usize;
    let mut anti_count = 0usize;
    for row in &rows {
        let dot = row["dot"].as_i64().unwrap_or(0);
        if dot > 0 {
            positive_dot += dot;
            support_count += 1;
        } else if dot < 0 {
            negative_dot += dot;
            anti_count += 1;
        }
    }

    let support = rows
        .iter()
        .filter(|row| row["dot"].as_i64().unwrap_or(0) > 0)
        .take(sample)
        .cloned()
        .collect::<Vec<_>>();
    let anti = rows
        .iter()
        .rev()
        .filter(|row| row["dot"].as_i64().unwrap_or(0) < 0)
        .take(sample)
        .cloned()
        .collect::<Vec<_>>();

    json!({
        "top_id": top_id,
        "considered": considered,
        "support_count": support_count,
        "anti_count": anti_count,
        "positive_dot": positive_dot,
        "negative_dot": negative_dot,
        "net_dot": positive_dot + negative_dot,
        "support": support,
        "anti": anti
    })
}

fn packed_lane_keys_report(packed_support: &Value) -> Value {
    json!({
        "mode": "stable-lane-keys",
        "storage": "cold-stable-signature",
        "hot_compilation": "record masks are rebuilt for the current focused packet",
        "lane_bytes": nanda_6m::LANE_BYTES,
        "route": packed_axis_lane_key(&packed_support["route"], "route"),
        "group": packed_axis_lane_key(&packed_support["group"], "group")
    })
}

fn packed_axis_lane_key(axis_support: &Value, axis: &str) -> Value {
    let top_id = axis_support["top_id"].as_u64().unwrap_or(0);
    let support_signature = packed_items_signature(&axis_support["support"]);
    let anti_signature = packed_items_signature(&axis_support["anti"]);
    let key_material = format!("{axis}|{top_id}|{support_signature}|{anti_signature}");
    let key_hash = stable_hash32(&key_material);
    json!({
        "axis": axis,
        "target_id": top_id,
        "key_hash": key_hash,
        "support_signature": support_signature,
        "anti_signature": anti_signature,
        "support_count": axis_support["support_count"].as_u64().unwrap_or(0),
        "anti_count": axis_support["anti_count"].as_u64().unwrap_or(0),
        "compile_hint": {
            "record_mask_a": packed_support_mask(&axis_support["anti"]).0,
            "record_mask_b": packed_support_mask(&axis_support["anti"]).1,
            "protected_support_mask_a": packed_support_mask(&axis_support["support"]).0,
            "protected_support_mask_b": packed_support_mask(&axis_support["support"]).1
        }
    })
}

fn packed_items_signature(items: &Value) -> String {
    let mut parts = items
        .as_array()
        .into_iter()
        .flat_map(|items| items.iter())
        .map(|item| {
            format!(
                "{}:{}:{}",
                item["wave_seed"].as_u64().unwrap_or(0),
                item["polarity"].as_u64().unwrap_or(0),
                item["confidence"].as_u64().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts.join("|")
}

fn packed_lane_preview(packed_support: &Value, packed_lane_keys: &Value) -> Value {
    json!({
        "mode": "packed-lane-preview",
        "lane_bytes": nanda_6m::LANE_BYTES,
        "route": packed_axis_lane_preview(&packed_support["route"], &packed_lane_keys["route"]),
        "group": packed_axis_lane_preview(&packed_support["group"], &packed_lane_keys["group"])
    })
}

fn packed_lane_store_report(packed_lane_keys: &Value, packed_lanes: &Value) -> Value {
    let mut lanes = vec![];
    if let Some(lane) =
        packed_lane_store_item(&packed_lane_keys["route"], &packed_lanes["route"], "route")
    {
        lanes.push(lane);
    }
    if let Some(lane) =
        packed_lane_store_item(&packed_lane_keys["group"], &packed_lanes["group"], "group")
    {
        lanes.push(lane);
    }
    let count = lanes.len();
    json!({
        "mode": "packed-lane-store",
        "storage": "hot-compiled-lane-arena",
        "source": "cold-stable-lane-keys",
        "capacity": nanda_6m::LANE_CAPACITY,
        "count": count,
        "bytes": count * nanda_6m::LANE_BYTES,
        "arena_bytes": nanda_6m::LANE_ARENA_BYTES,
        "record_bytes": nanda_6m::LANE_BYTES,
        "replay_ready": count > 0,
        "sample": lanes
    })
}

fn packed_lane_store_item(axis_key: &Value, axis_lane: &Value, axis: &str) -> Option<Value> {
    if axis_lane["state"].as_str() != Some("LANE_PREVIEW_READY") {
        return None;
    }
    Some(json!({
        "axis": axis,
        "key_hash": axis_key["key_hash"].as_u64().unwrap_or(0),
        "target_id": axis_key["target_id"].as_u64().unwrap_or(0),
        "action": axis_lane["action"].as_str().unwrap_or("none"),
        "record_mask_a": axis_lane["record_mask_a"].as_u64().unwrap_or(0),
        "record_mask_b": axis_lane["record_mask_b"].as_u64().unwrap_or(0),
        "protected_support_mask_a": axis_lane["protected_support_mask_a"].as_u64().unwrap_or(0),
        "protected_support_mask_b": axis_lane["protected_support_mask_b"].as_u64().unwrap_or(0),
        "strength": axis_lane["strength"].as_u64().unwrap_or(0),
        "before_net_dot": axis_lane["before_net_dot"].as_i64().unwrap_or(0),
        "after_net_dot": axis_lane["after_net_dot"].as_i64().unwrap_or(0),
        "delta_dot": axis_lane["delta_dot"].as_i64().unwrap_or(0),
        "key_storage": axis_lane["key_storage"].as_str().unwrap_or("cold-stable-signature"),
        "compiled_storage": axis_lane["compiled_storage"].as_str().unwrap_or("hot-packed-lane64")
    }))
}

fn packed_axis_support_field(axis_support: &Value, key_hash: u32) -> nanda_6m::PackedSupportField {
    let top_id = axis_support["top_id"].as_u64().unwrap_or(0) as u16;
    let positive_dot = axis_support["positive_dot"].as_i64().unwrap_or(0);
    let negative_dot = axis_support["negative_dot"].as_i64().unwrap_or(0);
    let support_mask = packed_support_mask(&axis_support["support"]);
    let anti_mask = packed_support_mask(&axis_support["anti"]);
    nanda_6m::PackedSupportField {
        top_id,
        key_hash,
        positive_dot,
        negative_dot,
        support_mask_a: support_mask.0,
        support_mask_b: support_mask.1,
        anti_mask_a: anti_mask.0,
        anti_mask_b: anti_mask.1,
    }
}

fn packed_lane_from_preview(
    axis_lane: &Value,
    field: nanda_6m::PackedSupportField,
) -> nanda_6m::PackedLane64 {
    nanda_6m::PackedLane64 {
        support_mask_a: axis_lane["record_mask_a"].as_u64().unwrap_or(0),
        support_mask_b: axis_lane["record_mask_b"].as_u64().unwrap_or(0),
        anti_mask_a: axis_lane["protected_support_mask_a"].as_u64().unwrap_or(0),
        anti_mask_b: axis_lane["protected_support_mask_b"].as_u64().unwrap_or(0),
        lane_id: axis_lane["key_hash"]
            .as_u64()
            .unwrap_or(u64::from(field.key_hash)) as u32,
        target_route: axis_lane["target_id"]
            .as_u64()
            .unwrap_or(u64::from(field.top_id)) as u16,
        target_group: axis_lane["target_id"]
            .as_u64()
            .unwrap_or(u64::from(field.top_id)) as u16,
        target_relation: 0,
        accepted_count: 0,
        rejected_count: if axis_lane["state"].as_str() == Some("LANE_PREVIEW_READY") {
            1
        } else {
            0
        },
        margin_hint: field
            .before_net_dot()
            .clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16,
        action: if axis_lane["state"].as_str() == Some("LANE_PREVIEW_READY") {
            1
        } else {
            0
        },
        strength: axis_lane["strength"].as_u64().unwrap_or(0) as u8,
        reserved: [0; 14],
    }
}

fn packed_axis_lane_preview(axis_support: &Value, axis_key: &Value) -> Value {
    let key_hash = axis_key["key_hash"]
        .as_u64()
        .unwrap_or_else(|| axis_support["top_id"].as_u64().unwrap_or(0)) as u32;
    let field = packed_axis_support_field(axis_support, key_hash);
    let applied = nanda_6m::compile_and_apply_suppress_anti_lane(field);
    let lane = applied.lane;
    json!({
        "state": if applied.lane_ready { "LANE_PREVIEW_READY" } else { "NO_ANTI_LANE" },
        "action": if applied.lane_ready { "suppress_anti_support" } else { "none" },
        "key_hash": lane.lane_id,
        "key_storage": "cold-stable-signature",
        "compiled_storage": "hot-packed-lane64",
        "application_core": "nanda_6m::compile_and_apply_suppress_anti_lane",
        "target_id": field.top_id,
        "record_mask_a": lane.support_mask_a,
        "record_mask_b": lane.support_mask_b,
        "protected_support_mask_a": lane.anti_mask_a,
        "protected_support_mask_b": lane.anti_mask_b,
        "strength": lane.strength,
        "before_net_dot": applied.before_net_dot,
        "suppressed_negative_dot": applied.suppressed_negative_dot,
        "after_net_dot": applied.after_net_dot,
        "delta_dot": applied.delta_dot,
        "interpretation": if applied.lane_ready {
            "Preview only: suppressing the current anti-support records would remove the destructive contribution without changing positive support."
        } else {
            "No negative contribution was found for this packed axis."
        }
    })
}

fn packed_support_mask(items: &Value) -> (u64, u64) {
    let mut mask_a = 0u64;
    let mut mask_b = 0u64;
    if let Some(items) = items.as_array() {
        for item in items {
            let Some(index) = item["record_index"].as_u64() else {
                continue;
            };
            if index < 64 {
                mask_a |= 1u64 << index;
            } else if index < 128 {
                mask_b |= 1u64 << (index - 64);
            }
        }
    }
    (mask_a, mask_b)
}

fn packed_lane_application_report(
    packed_support: &Value,
    packed_lanes: &Value,
    raw_decision: &Value,
) -> Value {
    let route =
        packed_axis_lane_application(&packed_support["route"], &packed_lanes["route"], "route");
    let group =
        packed_axis_lane_application(&packed_support["group"], &packed_lanes["group"], "group");
    let route_state = route["state"].as_str().unwrap_or("LANE_NO_EFFECT");
    let group_state = group["state"].as_str().unwrap_or("LANE_NO_EFFECT");
    let improved = route["improved"].as_bool().unwrap_or(false)
        || group["improved"].as_bool().unwrap_or(false);
    let focused_candidate = route_state == "LANE_AXIS_FOCUSED_CANDIDATE"
        && group_state == "LANE_AXIS_FOCUSED_CANDIDATE";
    let state = if focused_candidate {
        "PACKED_LANE_FOCUSED_CANDIDATE"
    } else if improved {
        "PACKED_LANE_IMPROVED"
    } else {
        "PACKED_LANE_NO_EFFECT"
    };

    json!({
        "mode": "single-pass-suppress-anti-support",
        "state": state,
        "verdict": if focused_candidate { "WATCH" } else { raw_decision["verdict"].as_str().unwrap_or("WATCH") },
        "safe_to_answer": false,
        "ready_for_hot_loop": focused_candidate,
        "raw_state": raw_decision["state"].as_str().unwrap_or("PACKED_REVIEW_REQUIRED"),
        "reason": if focused_candidate {
            "Single-pass lane application turns the support-map net into a focused candidate, but persistent learned lanes are still required before answering from the hot core."
        } else if improved {
            "Single-pass lane application improves the support-map net, but the adjusted field is not focused enough."
        } else {
            "No useful packed lane application was available."
        },
        "thresholds": {
            "min_focused_net_dot": 128,
            "min_delta_dot": 64
        },
        "route": route,
        "group": group
    })
}

fn packed_axis_lane_application(axis_support: &Value, axis_lane: &Value, axis: &str) -> Value {
    let key_hash = axis_lane["key_hash"]
        .as_u64()
        .unwrap_or_else(|| axis_support["top_id"].as_u64().unwrap_or(0)) as u32;
    let field = packed_axis_support_field(axis_support, key_hash);
    let lane = packed_lane_from_preview(axis_lane, field);
    let applied = nanda_6m::apply_suppress_anti_lane(field, lane);
    let state = if applied.focused_candidate {
        "LANE_AXIS_FOCUSED_CANDIDATE"
    } else if applied.improved {
        "LANE_AXIS_IMPROVED"
    } else {
        "LANE_NO_EFFECT"
    };
    json!({
        "axis": axis,
        "state": state,
        "application_core": "nanda_6m::apply_suppress_anti_lane",
        "improved": applied.improved,
        "before_net_dot": applied.before_net_dot,
        "after_net_dot": applied.after_net_dot,
        "delta_dot": applied.delta_dot,
        "suppressed_negative_dot": applied.suppressed_negative_dot,
        "support_count": axis_support["support_count"].as_u64().unwrap_or(0),
        "anti_count": axis_support["anti_count"].as_u64().unwrap_or(0),
        "record_mask_a": applied.lane.support_mask_a,
        "record_mask_b": applied.lane.support_mask_b,
        "protected_support_mask_a": applied.lane.anti_mask_a,
        "protected_support_mask_b": applied.lane.anti_mask_b
    })
}

fn packed_lane_replay_report(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
    packed_lane_store: &Value,
    routes: &IdDictionary,
    groups: &IdDictionary,
) -> Value {
    let query_tokens = packed_replay_tokens(packet, source, candidates);
    let mut replayed = vec![];
    if let Some(items) = packed_lane_store["sample"].as_array() {
        for item in items {
            let axis = item["axis"].as_str().unwrap_or("");
            let target_id = item["target_id"].as_u64().unwrap_or(0) as u32;
            let target_label = match axis {
                "route" => routes.label(target_id).unwrap_or("__route_default"),
                "group" => groups.label(target_id).unwrap_or("__group_default"),
                _ => "__unknown",
            };
            for shortcut in &packet.negative_shortcuts {
                let query_ratio = negative_lane_match_ratio(&query_tokens, shortcut);
                if query_ratio <= 0.0 || !negative_shortcut_matches_target(shortcut, target_label) {
                    continue;
                }
                replayed.push(packed_lane_replay_item(
                    item,
                    axis,
                    target_label,
                    "negative_shortcuts",
                    &shortcut.id,
                    query_ratio,
                ));
            }
            for shortcut in &packet.positive_shortcuts {
                let query_ratio = positive_lane_match_ratio(&query_tokens, shortcut);
                if query_ratio <= 0.0 || !positive_shortcut_matches_target(shortcut, target_label) {
                    continue;
                }
                replayed.push(packed_lane_replay_item(
                    item,
                    axis,
                    target_label,
                    "positive_shortcuts",
                    &shortcut.id,
                    query_ratio,
                ));
            }
        }
    }

    let compiled_lanes = replayed.len();
    let before_net_dot: i64 = replayed
        .iter()
        .map(|item| item["before_net_dot"].as_i64().unwrap_or(0))
        .sum();
    let after_net_dot: i64 = replayed
        .iter()
        .map(|item| item["after_net_dot"].as_i64().unwrap_or(0))
        .sum();
    let delta_dot = after_net_dot - before_net_dot;
    let focused = compiled_lanes > 0 && after_net_dot >= 128 && delta_dot >= 64;
    let stability_sweep = packed_lane_replay_stability_sweep(before_net_dot, delta_dot);
    let stability_state = packed_lane_replay_stability_state(&stability_sweep, compiled_lanes);
    json!({
        "mode": "feedback-lane-replay",
        "source": "negative_shortcuts+positive_shortcuts",
        "touch_policy": {
            "mode": "observer-to-compute-sweep",
            "default_strength": 0.0,
            "soft_strength": 0.25,
            "full_strength": 1.0,
            "safe_to_answer_grant": false,
            "interpretation": "Replay is measured at observer/soft/medium/full strengths. It may shape the packed field, but it never grants final answer permission by itself."
        },
        "state": if focused { "PACKED_LANE_REPLAY_FOCUSED" } else if compiled_lanes > 0 { "PACKED_LANE_REPLAY_PARTIAL" } else { "PACKED_LANE_REPLAY_NONE" },
        "stability_state": stability_state,
        "safe_to_answer": false,
        "matched_keys": compiled_lanes,
        "compiled_lanes": compiled_lanes,
        "before_net_dot": before_net_dot,
        "after_net_dot": after_net_dot,
        "delta_dot": delta_dot,
        "replay_ready": focused,
        "computational_effect": {
            "state": if focused { "REPLAY_COMPUTE_READY" } else if compiled_lanes > 0 { "REPLAY_COMPUTE_WEAK" } else { "REPLAY_COMPUTE_NONE" },
            "applied_strength": if focused { 1.0 } else { 0.0 },
            "field_before": before_net_dot,
            "field_after": if focused { after_net_dot } else { before_net_dot },
            "delta_dot": if focused { delta_dot } else { 0 },
            "safe_to_answer": false,
            "reason": if focused {
                "Matched feedback lanes can be applied as a packed compute pass, but the structural gate must still decide trust."
            } else if compiled_lanes > 0 {
                "Matched feedback lanes exist, but the replay field is not focused enough for compute application."
            } else {
                "No feedback lane matched the current packed lane keys."
            }
        },
        "stability_sweep": stability_sweep,
        "sample": replayed
    })
}

fn packed_lane_replay_stability_sweep(before_net_dot: i64, delta_dot: i64) -> Vec<Value> {
    [
        ("observer", 0u32),
        ("soft_touch", 250u32),
        ("medium_touch", 500u32),
        ("full_touch", 1000u32),
    ]
    .into_iter()
    .map(|(label, permille)| {
        let applied_delta = delta_dot * i64::from(permille) / 1000;
        let after = before_net_dot + applied_delta;
        json!({
            "label": label,
            "strength": round4(f64::from(permille) / 1000.0),
            "before_net_dot": before_net_dot,
            "after_net_dot": after,
            "delta_dot": applied_delta,
            "field_state": packed_lane_replay_field_state(after, applied_delta)
        })
    })
    .collect()
}

fn packed_lane_replay_stability_state(sweep: &[Value], compiled_lanes: usize) -> &'static str {
    if compiled_lanes == 0 {
        return "NO_REPLAY_FIELD";
    }
    let soft = sweep
        .iter()
        .find(|item| item["label"].as_str() == Some("soft_touch"));
    let full = sweep
        .iter()
        .find(|item| item["label"].as_str() == Some("full_touch"));
    let soft_after = soft
        .and_then(|item| item["after_net_dot"].as_i64())
        .unwrap_or(0);
    let soft_delta = soft
        .and_then(|item| item["delta_dot"].as_i64())
        .unwrap_or(0);
    let full_after = full
        .and_then(|item| item["after_net_dot"].as_i64())
        .unwrap_or(0);
    let full_delta = full
        .and_then(|item| item["delta_dot"].as_i64())
        .unwrap_or(0);
    if soft_after >= 128 && soft_delta >= 64 {
        "STABLE_UNDER_SOFT_TOUCH"
    } else if full_after >= 128 && full_delta >= 64 {
        "FULL_TOUCH_REQUIRED"
    } else if full_delta > 0 {
        "WEAK_CONSTRUCTIVE_REPLAY"
    } else if full_delta < 0 {
        "DESTABILIZING_REPLAY"
    } else {
        "NO_REPLAY_SHIFT"
    }
}

fn packed_lane_replay_field_state(after_net_dot: i64, delta_dot: i64) -> &'static str {
    if after_net_dot >= 128 && delta_dot >= 64 {
        "FIELD_FOCUSED_BY_REPLAY"
    } else if delta_dot > 0 {
        "FIELD_IMPROVED_BY_REPLAY"
    } else if delta_dot < 0 {
        "FIELD_WEAKENED_BY_REPLAY"
    } else {
        "FIELD_OBSERVED"
    }
}

fn packed_replay_decision_report(raw_decision: &Value, replay: &Value) -> Value {
    let raw_state = raw_decision["state"]
        .as_str()
        .unwrap_or("PACKED_REVIEW_REQUIRED");
    let raw_safe = raw_decision["safe_to_answer"].as_bool().unwrap_or(false);
    let replay_state = replay["state"]
        .as_str()
        .unwrap_or("PACKED_LANE_REPLAY_NONE");
    let stability_state = replay["stability_state"]
        .as_str()
        .unwrap_or("NO_REPLAY_FIELD");
    let compute_state = replay["computational_effect"]["state"]
        .as_str()
        .unwrap_or("REPLAY_COMPUTE_NONE");
    let matched_keys = replay["matched_keys"].as_u64().unwrap_or(0);
    let soft = replay_touch(replay, "soft_touch");
    let full = replay_touch(replay, "full_touch");
    let typed = nanda_6m::evaluate_replay(nanda_6m::ReplayDecisionInput {
        raw_state: parse_raw_peak_state(raw_state),
        raw_safe_to_answer: raw_safe,
        raw_verdict_pass: raw_decision["verdict"].as_str() == Some("PASS"),
        matched_keys,
        observer_net_dot: replay["before_net_dot"].as_i64().unwrap_or(0),
        full_delta_dot: replay["delta_dot"].as_i64().unwrap_or(0),
        soft: parse_replay_touch(&soft),
        full: parse_replay_touch(&full),
        stability_state: parse_replay_stability_state(stability_state),
        compute_state: parse_replay_compute_state(compute_state),
    });
    let stability_verdict = typed.verdict.as_str();
    let verdict = if typed.output_veto {
        "VETO"
    } else if typed.output_pass {
        "PASS"
    } else if typed.verdict == nanda_6m::ReplayVerdict::NoReplayEvidence {
        raw_decision["verdict"].as_str().unwrap_or("WATCH")
    } else {
        "WATCH"
    };

    json!({
        "mode": "replay-adjusted-peak-firewall",
        "core": "nanda_6m::evaluate_replay",
        "hot_compatible": true,
        "raw_state": raw_state,
        "raw_safe_to_answer": raw_safe,
        "replay_state": replay_state,
        "compute_state": compute_state,
        "stability_state": stability_state,
        "stability_verdict": stability_verdict,
        "verdict": verdict,
        "action": typed.action.as_str(),
        "safe_to_answer": typed.safe_to_answer,
        "firewall": {
            "rule": "replay may shape or rescue the packed field, but cannot grant final answer permission",
            "blocks_direct_pass": true,
            "requires_structural_gate": true
        },
        "adjusted_field": {
            "observer_net_dot": typed.observer_net_dot,
            "soft_touch_net_dot": typed.soft_touch_net_dot,
            "full_touch_net_dot": typed.full_touch_net_dot,
            "full_delta_dot": typed.full_delta_dot,
            "matched_keys": typed.matched_keys
        },
        "reason": packed_replay_decision_reason(typed.verdict, raw_state, raw_safe)
    })
}

fn parse_raw_peak_state(value: &str) -> nanda_6m::RawPeakState {
    match value {
        "PACKED_FOCUSED" => nanda_6m::RawPeakState::Focused,
        "PACKED_THIN" => nanda_6m::RawPeakState::Thin,
        "PACKED_CONTESTED" => nanda_6m::RawPeakState::Contested,
        "PACKED_NO_PEAK" | "PACKED_EMPTY_MEMORY" | "PACKED_EMPTY_QUERY" => {
            nanda_6m::RawPeakState::NoPeak
        }
        _ => nanda_6m::RawPeakState::Review,
    }
}

fn parse_replay_compute_state(value: &str) -> nanda_6m::ReplayComputeState {
    match value {
        "REPLAY_COMPUTE_READY" => nanda_6m::ReplayComputeState::Ready,
        "REPLAY_COMPUTE_WEAK" => nanda_6m::ReplayComputeState::Weak,
        _ => nanda_6m::ReplayComputeState::None,
    }
}

fn parse_replay_field_state(value: &str) -> nanda_6m::ReplayFieldState {
    match value {
        "FIELD_FOCUSED_BY_REPLAY" => nanda_6m::ReplayFieldState::Focused,
        "FIELD_IMPROVED_BY_REPLAY" => nanda_6m::ReplayFieldState::Improved,
        "FIELD_WEAKENED_BY_REPLAY" => nanda_6m::ReplayFieldState::Weakened,
        _ => nanda_6m::ReplayFieldState::Observed,
    }
}

fn parse_replay_stability_state(value: &str) -> nanda_6m::ReplayStabilityState {
    match value {
        "STABLE_UNDER_SOFT_TOUCH" => nanda_6m::ReplayStabilityState::StableUnderSoftTouch,
        "FULL_TOUCH_REQUIRED" => nanda_6m::ReplayStabilityState::FullTouchRequired,
        "WEAK_CONSTRUCTIVE_REPLAY" => nanda_6m::ReplayStabilityState::WeakConstructive,
        "DESTABILIZING_REPLAY" => nanda_6m::ReplayStabilityState::Destabilizing,
        "NO_REPLAY_SHIFT" => nanda_6m::ReplayStabilityState::NoShift,
        _ => nanda_6m::ReplayStabilityState::NoReplayField,
    }
}

fn parse_replay_touch(value: &Value) -> nanda_6m::ReplayTouch {
    nanda_6m::ReplayTouch {
        after_net_dot: value["after_net_dot"].as_i64().unwrap_or(0),
        delta_dot: value["delta_dot"].as_i64().unwrap_or(0),
        field_state: parse_replay_field_state(
            value["field_state"].as_str().unwrap_or("FIELD_OBSERVED"),
        ),
    }
}

fn replay_touch(replay: &Value, label: &str) -> Value {
    replay["stability_sweep"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["label"].as_str() == Some(label))
        })
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "label": label,
                "strength": 0.0,
                "before_net_dot": 0,
                "after_net_dot": 0,
                "delta_dot": 0,
                "field_state": "FIELD_OBSERVED"
            })
        })
}

fn packed_replay_decision_reason(
    stability_verdict: nanda_6m::ReplayVerdict,
    raw_state: &str,
    raw_safe: bool,
) -> &'static str {
    match stability_verdict {
        nanda_6m::ReplayVerdict::StableWithReplay => {
            "Raw packed field was already acceptable and replay remains stable under soft touch."
        }
        nanda_6m::ReplayVerdict::ReplayRescuedThinField => {
            "Raw packed field was thin, but matched feedback lanes focus it under soft touch; keep under WATCH for structural review."
        }
        nanda_6m::ReplayVerdict::ReplayDestabilizedField => {
            "Replay weakens or destabilizes the packed field; do not trust the raw peak."
        }
        nanda_6m::ReplayVerdict::ReplayTooStrongRequired => {
            "Replay only focuses the field under full-strength intervention; treat the peak as intervention-dependent."
        }
        nanda_6m::ReplayVerdict::NoReplayEvidence if raw_safe => {
            "No feedback lane matched; rely on the raw packed decision."
        }
        nanda_6m::ReplayVerdict::NoReplayEvidence => {
            "No feedback lane matched; replay provides no extra evidence for the raw packed decision."
        }
        _ if raw_state == "PACKED_THIN" => {
            "Replay changed the field, but not enough to rescue the thin packed decision."
        }
        _ => "Replay effect requires review before it can influence downstream trust.",
    }
}

fn packed_lane_replay_item(
    item: &Value,
    axis: &str,
    target_label: &str,
    source: &str,
    shortcut_id: &str,
    query_ratio: f64,
) -> Value {
    json!({
        "axis": axis,
        "target_label": target_label,
        "source": source,
        "shortcut_id": shortcut_id,
        "query_match_ratio": round4(query_ratio),
        "key_hash": item["key_hash"].as_u64().unwrap_or(0),
        "record_mask_a": item["record_mask_a"].as_u64().unwrap_or(0),
        "record_mask_b": item["record_mask_b"].as_u64().unwrap_or(0),
        "protected_support_mask_a": item["protected_support_mask_a"].as_u64().unwrap_or(0),
        "protected_support_mask_b": item["protected_support_mask_b"].as_u64().unwrap_or(0),
        "before_net_dot": item["before_net_dot"].as_i64().unwrap_or(0),
        "after_net_dot": item["after_net_dot"].as_i64().unwrap_or(0),
        "delta_dot": item["delta_dot"].as_i64().unwrap_or(0),
        "compiled_storage": item["compiled_storage"].as_str().unwrap_or("hot-packed-lane64")
    })
}

fn packed_replay_tokens(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    tokens.extend(normalized_shortcut_terms(std::slice::from_ref(
        &packet.query,
    )));
    for triad in source.iter().chain(candidates.iter()) {
        for value in [
            &triad.subject,
            &triad.relation,
            &triad.object,
            &triad.evidence,
            &triad.route,
            &triad.group,
            &triad.subject_role,
            &triad.object_role,
        ] {
            tokens.extend(normalized_shortcut_terms(std::slice::from_ref(value)));
        }
    }
    tokens
}

fn negative_shortcut_matches_target(shortcut: &NegativeShortcut, target_label: &str) -> bool {
    shortcut_label_matches(
        target_label,
        [
            shortcut.prefer_peak.as_str(),
            shortcut.prefer_route.as_str(),
            shortcut.prefer_group.as_str(),
            shortcut.suppress_peak.as_str(),
            shortcut.suppress_route.as_str(),
            shortcut.suppress_group.as_str(),
        ],
    )
}

fn positive_shortcut_matches_target(shortcut: &PositiveShortcut, target_label: &str) -> bool {
    shortcut_label_matches(
        target_label,
        [
            shortcut.reinforce_peak.as_str(),
            shortcut.reinforce_route.as_str(),
            shortcut.reinforce_group.as_str(),
            "",
            "",
            "",
        ],
    )
}

fn shortcut_label_matches<'a>(
    target_label: &str,
    labels: impl IntoIterator<Item = &'a str>,
) -> bool {
    let target = norm(target_label);
    if target.is_empty() {
        return false;
    }
    labels.into_iter().any(|label| {
        let label = norm(label);
        !label.is_empty() && (target == label || target.contains(&label) || label.contains(&target))
    })
}

pub(super) fn print_budget_text(out: &Value) {
    println!("NANDA-6M BUDGET");
    println!(
        "state: {}",
        out["state"].as_str().unwrap_or("FOCUS_REQUIRED")
    );
    println!("fits_l3: {}", out["fits_l3"].as_bool().unwrap_or(false));
    println!(
        "estimated_hot_bytes: {}/{}",
        out["estimated_hot_bytes"].as_u64().unwrap_or(0),
        out["hard_budget_bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "triads: {}/{}",
        out["counts"]["active_triads"].as_u64().unwrap_or(0),
        out["capacity"]["triads"].as_u64().unwrap_or(0)
    );
    println!(
        "centroids: {}/{}",
        out["counts"]["centroids"].as_u64().unwrap_or(0),
        out["capacity"]["centroids"].as_u64().unwrap_or(0)
    );
    println!(
        "lanes: {}/{}",
        out["counts"]["active_lanes"].as_u64().unwrap_or(0),
        out["capacity"]["lanes"].as_u64().unwrap_or(0)
    );
    println!("next: {}", out["next"].as_str().unwrap_or(""));
}

pub(super) fn print_budget_md(out: &Value) {
    println!("# NANDA-6M Budget\n");
    println!(
        "- state: `{}`",
        out["state"].as_str().unwrap_or("FOCUS_REQUIRED")
    );
    println!("- fits_l3: `{}`", out["fits_l3"]);
    println!(
        "- estimated_hot_bytes: `{}/{}`",
        out["estimated_hot_bytes"], out["hard_budget_bytes"]
    );
    println!(
        "- triads: `{}/{}`",
        out["counts"]["active_triads"], out["capacity"]["triads"]
    );
    println!(
        "- centroids: `{}/{}`",
        out["counts"]["centroids"], out["capacity"]["centroids"]
    );
    println!(
        "- lanes: `{}/{}`",
        out["counts"]["active_lanes"], out["capacity"]["lanes"]
    );
    println!("- next: {}", out["next"].as_str().unwrap_or(""));
}

pub(super) fn print_pack6m_text(out: &Value) {
    println!("NANDA-6M PACK");
    println!(
        "state: {}",
        out["state"].as_str().unwrap_or("PACK_REVIEW_REQUIRED")
    );
    println!("packed_ok: {}", out["packed_ok"].as_bool().unwrap_or(false));
    println!(
        "records: {} / memory {} / query {} / bytes {}",
        out["packed_records"]["count"].as_u64().unwrap_or(0),
        out["packed_records"]["memory_count"].as_u64().unwrap_or(0),
        out["packed_records"]["query_count"].as_u64().unwrap_or(0),
        out["packed_records"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "projection: {} / records {} / dim {} / energy {}",
        out["projection"]["source"].as_str().unwrap_or("unknown"),
        out["projection"]["records"].as_u64().unwrap_or(0),
        out["projection"]["wave_dim"].as_u64().unwrap_or(0),
        out["projection"]["summary"]["energy"].as_u64().unwrap_or(0)
    );
    println!(
        "centroids: route {} / group {}",
        out["centroids"]["route_count"].as_u64().unwrap_or(0),
        out["centroids"]["group_count"].as_u64().unwrap_or(0)
    );
    println!(
        "peaks: route {} score {} / group {} score {}",
        out["peaks"]["route"]["top_id"].as_u64().unwrap_or(0),
        out["peaks"]["route"]["top_score"].as_f64().unwrap_or(0.0),
        out["peaks"]["group"]["top_id"].as_u64().unwrap_or(0),
        out["peaks"]["group"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "decision: {} / safe_to_answer {}",
        out["peak_decision"]["state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED"),
        out["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "support: route +{} / -{} / net {}",
        out["packed_support"]["route"]["support_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["anti_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "lane preview: route net {} -> {}",
        out["packed_lanes"]["route"]["before_net_dot"]
            .as_i64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["after_net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "lane key: route {} / hot mask {}",
        out["packed_lane_keys"]["route"]["key_hash"]
            .as_u64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["record_mask_a"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "lane store: {} / {} / bytes {}",
        out["packed_lane_store"]["count"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["capacity"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "lane replay: {} / matched {}",
        out["packed_lane_replay"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_REPLAY_NONE"),
        out["packed_lane_replay"]["matched_keys"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "replay stability: {} / compute {}",
        out["packed_lane_replay"]["stability_state"]
            .as_str()
            .unwrap_or("NO_REPLAY_FIELD"),
        out["packed_lane_replay"]["computational_effect"]["state"]
            .as_str()
            .unwrap_or("REPLAY_COMPUTE_NONE")
    );
    println!(
        "replay decision: {} / action {}",
        out["packed_replay_decision"]["stability_verdict"]
            .as_str()
            .unwrap_or("NO_REPLAY_EVIDENCE"),
        out["packed_replay_decision"]["action"]
            .as_str()
            .unwrap_or("USE_RAW_DECISION")
    );
    println!(
        "lane applied: {} -> {}",
        out["packed_lane_application"]["raw_state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED"),
        out["packed_lane_application"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_NO_EFFECT")
    );
    println!(
        "budget: {} / {}",
        out["budget"]["estimated_hot_bytes"].as_u64().unwrap_or(0),
        out["budget"]["hard_budget_bytes"].as_u64().unwrap_or(0)
    );
}

pub(super) fn print_pack6m_md(out: &Value) {
    println!("# NANDA-6M Pack\n");
    println!(
        "- state: `{}`",
        out["state"].as_str().unwrap_or("PACK_REVIEW_REQUIRED")
    );
    println!("- packed_ok: `{}`", out["packed_ok"]);
    println!(
        "- records: `{}`",
        out["packed_records"]["count"].as_u64().unwrap_or(0)
    );
    println!(
        "- memory_records: `{}`",
        out["packed_records"]["memory_count"].as_u64().unwrap_or(0)
    );
    println!(
        "- query_records: `{}`",
        out["packed_records"]["query_count"].as_u64().unwrap_or(0)
    );
    println!(
        "- bytes: `{}`",
        out["packed_records"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "- projection_source: `{}`",
        out["projection"]["source"].as_str().unwrap_or("unknown")
    );
    println!(
        "- projection_energy: `{}`",
        out["projection"]["summary"]["energy"].as_u64().unwrap_or(0)
    );
    println!(
        "- centroids: route `{}` / group `{}`",
        out["centroids"]["route_count"].as_u64().unwrap_or(0),
        out["centroids"]["group_count"].as_u64().unwrap_or(0)
    );
    println!(
        "- route_peak: `{}` / score `{}`",
        out["peaks"]["route"]["top_id"].as_u64().unwrap_or(0),
        out["peaks"]["route"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "- peak_decision: `{}`",
        out["peak_decision"]["state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED")
    );
    println!(
        "- safe_to_answer: `{}`",
        out["peak_decision"]["safe_to_answer"]
    );
    println!(
        "- route_support: `+{} / -{} / net {}`",
        out["packed_support"]["route"]["support_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["anti_count"]
            .as_u64()
            .unwrap_or(0),
        out["packed_support"]["route"]["net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "- lane_preview: `route net {} -> {}`",
        out["packed_lanes"]["route"]["before_net_dot"]
            .as_i64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["after_net_dot"]
            .as_i64()
            .unwrap_or(0)
    );
    println!(
        "- lane_key: `route {} / hot mask {}`",
        out["packed_lane_keys"]["route"]["key_hash"]
            .as_u64()
            .unwrap_or(0),
        out["packed_lanes"]["route"]["record_mask_a"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "- lane_store: `{}/{}` / bytes `{}`",
        out["packed_lane_store"]["count"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["capacity"].as_u64().unwrap_or(0),
        out["packed_lane_store"]["bytes"].as_u64().unwrap_or(0)
    );
    println!(
        "- lane_replay: `{}` / matched `{}`",
        out["packed_lane_replay"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_REPLAY_NONE"),
        out["packed_lane_replay"]["matched_keys"]
            .as_u64()
            .unwrap_or(0)
    );
    println!(
        "- replay_stability: `{}` / compute `{}`",
        out["packed_lane_replay"]["stability_state"]
            .as_str()
            .unwrap_or("NO_REPLAY_FIELD"),
        out["packed_lane_replay"]["computational_effect"]["state"]
            .as_str()
            .unwrap_or("REPLAY_COMPUTE_NONE")
    );
    println!(
        "- replay_decision: `{}` / action `{}`",
        out["packed_replay_decision"]["stability_verdict"]
            .as_str()
            .unwrap_or("NO_REPLAY_EVIDENCE"),
        out["packed_replay_decision"]["action"]
            .as_str()
            .unwrap_or("USE_RAW_DECISION")
    );
    println!(
        "- lane_applied: `{} -> {}`",
        out["packed_lane_application"]["raw_state"]
            .as_str()
            .unwrap_or("PACKED_REVIEW_REQUIRED"),
        out["packed_lane_application"]["state"]
            .as_str()
            .unwrap_or("PACKED_LANE_NO_EFFECT")
    );
    println!(
        "- budget: `{}/{}`",
        out["budget"]["estimated_hot_bytes"], out["budget"]["hard_budget_bytes"]
    );
}

fn insert_label(set: &mut BTreeSet<String>, value: &str) {
    let trimmed = value.trim();
    if !trimmed.is_empty() {
        set.insert(trimmed.to_string());
    }
}

fn usage_row(count: usize, capacity: usize, bytes: usize, arena_bytes: usize) -> Value {
    json!({
        "count": count,
        "capacity": capacity,
        "bytes": bytes,
        "arena_bytes": arena_bytes,
        "percent": round4((count as f64 / capacity.max(1) as f64) * 100.0)
    })
}

#[derive(Default)]
pub(super) struct IdDictionary {
    items: BTreeMap<String, u32>,
}

impl IdDictionary {
    pub(super) fn id(&mut self, value: &str) -> u32 {
        let key = if value.trim().is_empty() {
            "__default"
        } else {
            value.trim()
        };
        if let Some(id) = self.items.get(key) {
            return *id;
        }
        let id = self.items.len() as u32 + 1;
        self.items.insert(key.to_string(), id);
        id
    }

    pub(super) fn len(&self) -> usize {
        self.items.len()
    }

    pub(super) fn label(&self, id: u32) -> Option<&str> {
        self.items
            .iter()
            .find_map(|(label, item_id)| (*item_id == id).then_some(label.as_str()))
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn pack_triad6m(
    triad: &Triad,
    flags: u16,
    entities: &mut IdDictionary,
    relations: &mut IdDictionary,
    routes: &mut IdDictionary,
    groups: &mut IdDictionary,
    evidences: &mut IdDictionary,
    roles: &mut IdDictionary,
) -> std::result::Result<nanda_6m::PackedTriad32, Value> {
    let relation_id = checked_u16(relations.id(&triad.relation), "relation", &triad.relation)?;
    let route_id = checked_u16(
        routes.id(defaulted(&triad.route, "__route_default")),
        "route",
        &triad.route,
    )?;
    let group_id = checked_u16(
        groups.id(defaulted(&triad.group, "__group_default")),
        "group",
        &triad.group,
    )?;
    let subject_role = checked_u8(
        roles.id(defaulted(&triad.subject_role, "subject")),
        "subject_role",
        &triad.subject_role,
    )?;
    let object_role = checked_u8(
        roles.id(defaulted(&triad.object_role, "object")),
        "object_role",
        &triad.object_role,
    )?;
    let role_pack = u16::from(subject_role) | (u16::from(object_role) << 8);
    let subject_id = entities.id(&triad.subject);
    let object_id = entities.id(&triad.object);
    let evidence_ref = evidences.id(&triad.evidence);
    let wave_seed = stable_hash32(&format!(
        "{}|{}|{}|{}|{}|{}",
        triad.subject,
        triad.relation,
        triad.object,
        triad.route,
        triad.group,
        triad_polarity(triad)
    ));
    let confidence = (triad.confidence.clamp(0.0, 1.0) * 255.0).round() as u8;
    let polarity = stable_hash8(&triad_polarity(triad));
    let check = stable_hash16(&format!(
        "{subject_id}|{object_id}|{evidence_ref}|{wave_seed}|{relation_id}|{route_id}|{group_id}|{role_pack}|{flags}|{confidence}|{polarity}"
    ));
    Ok(nanda_6m::PackedTriad32::new(nanda_6m::PackedTriadInput {
        subject_id,
        object_id,
        evidence_ref,
        wave_seed,
        relation_id,
        route_id,
        group_id,
        role_pack,
        flags,
        lane_hint: 0,
        check,
        confidence,
        polarity,
    }))
}

fn defaulted<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn checked_u16(value: u32, field: &str, label: &str) -> std::result::Result<u16, Value> {
    u16::try_from(value).map_err(|_| {
        json!({
            "state": "PACK_FIELD_OVERFLOW",
            "field": field,
            "label": label,
            "id": value,
            "capacity": u16::MAX
        })
    })
}

fn checked_u8(value: u32, field: &str, label: &str) -> std::result::Result<u8, Value> {
    u8::try_from(value).map_err(|_| {
        json!({
            "state": "PACK_FIELD_OVERFLOW",
            "field": field,
            "label": label,
            "id": value,
            "capacity": u8::MAX
        })
    })
}

pub(super) fn stable_hash32(value: &str) -> u32 {
    let digest = Sha256::digest(value.as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

fn stable_hash16(value: &str) -> u16 {
    let digest = Sha256::digest(value.as_bytes());
    u16::from_le_bytes([digest[0], digest[1]])
}

fn stable_hash8(value: &str) -> u8 {
    Sha256::digest(value.as_bytes())[0]
}

pub(super) fn dictionary_summary(dictionary: &IdDictionary, capacity: usize) -> Value {
    json!({
        "count": dictionary.len(),
        "capacity": capacity,
        "fits": dictionary.len() <= capacity,
        "sample": dictionary
            .items
            .iter()
            .take(8)
            .map(|(label, id)| json!({ "id": id, "label": label }))
            .collect::<Vec<_>>()
    })
}

pub(super) fn packed_triad_json(record: &nanda_6m::PackedTriad32) -> Value {
    json!({
        "subject_id": record.subject_id,
        "object_id": record.object_id,
        "evidence_ref": record.evidence_ref,
        "wave_seed": record.wave_seed,
        "relation_id": record.relation_id,
        "route_id": record.route_id,
        "group_id": record.group_id,
        "role_pack": record.role_pack,
        "flags": record.flags,
        "lane_hint": record.lane_hint,
        "check": record.check,
        "confidence": record.confidence,
        "polarity": record.polarity
    })
}
