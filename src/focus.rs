use crate::*;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;

#[derive(Clone, Debug)]
pub(crate) struct FocusBuild {
    pub(crate) packet: Packet,
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug)]
struct RouteCandidate {
    route: String,
    original_triads: usize,
    items: Vec<(f64, Triad)>,
}

pub(crate) fn focus_cmd(args: FocusArgs) -> Result<u8> {
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let mut query_packet = if let Some(query_file) = &args.query_file {
        load_packet_auto(
            query_file,
            &args.query_format,
            &args.task_id,
            &args.domain,
            &args.query,
            args.normalize_paths,
        )?
    } else {
        packet.clone()
    };
    inherit_aliases_if_needed(&mut query_packet, &packet);
    let query_text = if !args.query.trim().is_empty() {
        args.query.clone()
    } else if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let memory = normalize_ids(packet.triads.clone(), "m");
    let build = build_focused_packet(
        &packet,
        &memory,
        &query,
        query_source,
        args.max_triads,
        args.route_cap,
        args.route_triad_cap,
    );
    if let Some(out) = &args.out {
        let text = serde_json::to_string_pretty(&build.packet)? + "\n";
        if let Some(parent) = out.parent().filter(|path| !path.as_os_str().is_empty()) {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(out, text).with_context(|| format!("write {}", out.display()))?;
    }
    if args.stdout {
        println!("{}", serde_json::to_string_pretty(&build.packet)?);
        return Ok(EXIT_PASS);
    }
    let mut report = build.metadata.clone();
    if let Some(out) = &args.out {
        report["output"] = json!(out.display().to_string());
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputFormat::Text => print_focus_text(&report),
        OutputFormat::Md => print_focus_md(&report),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn build_focused_packet(
    packet: &Packet,
    memory: &[Triad],
    query: &[Triad],
    query_source: &str,
    max_triads: usize,
    route_cap: usize,
    route_triad_cap: usize,
) -> FocusBuild {
    let focus = route_balanced_focus_exact(memory, query, max_triads, route_cap, route_triad_cap);
    let focused_packet = Packet {
        task_id: format!("{}:focus", packet.task_id),
        domain: packet.domain.clone(),
        query: packet.query.clone(),
        triads: focus.memory.clone(),
        candidate_triads: query.to_vec(),
        candidate_answer: packet.candidate_answer.clone(),
        aliases: packet.aliases.clone(),
        canonicalization: packet.canonicalization.clone(),
        negative_shortcuts: packet.negative_shortcuts.clone(),
        positive_shortcuts: packet.positive_shortcuts.clone(),
        resonance_memory: packet.resonance_memory.clone(),
        continuation_memory: packet.continuation_memory.clone(),
        failure_contract: packet.failure_contract.clone(),
    };
    let mut metadata = focus.metadata;
    metadata["mode"] = json!("focused-packet-builder");
    metadata["core_version"] = json!(CORE_VERSION);
    metadata["nanda_6m_version"] = json!(nanda_6m::VERSION);
    metadata["query_source"] = json!(query_source);
    metadata["query_triads"] = json!(query.iter().map(triad_json).collect::<Vec<_>>());
    metadata["packet_task_id"] = json!(focused_packet.task_id.clone());
    metadata["packet_triads"] = json!(focused_packet.triads.len());
    metadata["packet_candidate_triads"] = json!(focused_packet.candidate_triads.len());
    FocusBuild {
        packet: focused_packet,
        metadata,
    }
}

pub(crate) fn route_balanced_focus_exact(
    memory: &[Triad],
    query: &[Triad],
    max_triads: usize,
    route_cap: usize,
    route_triad_cap: usize,
) -> FocusedMemory {
    let max_triads = max_triads.clamp(1, nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY);
    let route_cap = route_cap.max(1);
    let route_triad_cap = route_triad_cap.max(1);
    if memory.len() <= max_triads {
        return FocusedMemory {
            memory: memory.to_vec(),
            metadata: json!({
                "enabled": false,
                "reason": "memory_size_within_max_triads",
                "max_triads": max_triads,
                "route_cap": route_cap,
                "route_triad_cap": route_triad_cap,
                "original_memory_size": memory.len(),
                "focused_memory_size": memory.len(),
                "runtime_contract": runtime_focus_contract(memory.len())
            }),
        };
    }

    let query_terms = query_term_set(query);
    let mut by_route: BTreeMap<String, Vec<(f64, Triad)>> = BTreeMap::new();
    for triad in memory {
        let relevance = (0.52 * symbolic_query_overlap(query, triad))
            + (0.28 * token_overlap(&query_terms, triad))
            + (0.20 * source_weight(triad));
        by_route
            .entry(route_name(triad, "memory-route"))
            .or_default()
            .push((round4(relevance), triad.clone()));
    }

    let mut routes = by_route
        .into_iter()
        .map(|(route, mut items)| {
            items.sort_by(|a, b| {
                b.0.partial_cmp(&a.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.1.id.cmp(&b.1.id))
            });
            RouteCandidate {
                route,
                original_triads: items.len(),
                items,
            }
        })
        .collect::<Vec<_>>();
    routes.sort_by(|a, b| {
        b.items
            .first()
            .map(|(score, _)| *score)
            .unwrap_or(0.0)
            .partial_cmp(&a.items.first().map(|(score, _)| *score).unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.route.cmp(&b.route))
    });
    let original_routes = routes.len();
    routes.truncate(route_cap);

    let mut selected_by_route = vec![Vec::<Triad>::new(); routes.len()];
    let mut focused = Vec::with_capacity(max_triads.min(memory.len()));
    for slot in 0..route_triad_cap {
        for (route_index, route) in routes.iter().enumerate() {
            if focused.len() >= max_triads {
                break;
            }
            let Some((_, triad)) = route.items.get(slot) else {
                continue;
            };
            selected_by_route[route_index].push(triad.clone());
            focused.push(triad.clone());
        }
        if focused.len() >= max_triads {
            break;
        }
    }

    let route_rows = routes
        .iter()
        .enumerate()
        .map(|(idx, route)| {
            let selected = &selected_by_route[idx];
            json!({
                "route": route.route,
                "original_triads": route.original_triads,
                "selected_triads": selected.len(),
                "omitted_triads": route.original_triads.saturating_sub(selected.len()),
                "top_relevance": route.items.first().map(|(score, _)| *score).unwrap_or(0.0),
                "selected_ids": selected.iter().take(12).map(|triad| triad.id.clone()).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    FocusedMemory {
        memory: focused,
        metadata: json!({
            "enabled": true,
            "reason": "route_balanced_exact_focus",
            "max_triads": max_triads,
            "route_cap": route_cap,
            "route_triad_cap": route_triad_cap,
            "original_memory_size": memory.len(),
            "original_routes": original_routes,
            "focused_memory_size": route_rows.iter().map(|row| row["selected_triads"].as_u64().unwrap_or(0)).sum::<u64>() as usize,
            "focused_routes": route_rows.iter().filter(|row| row["selected_triads"].as_u64().unwrap_or(0) > 0).count(),
            "dropped_routes": original_routes.saturating_sub(route_rows.len()),
            "runtime_contract": runtime_focus_contract(route_rows.iter().map(|row| row["selected_triads"].as_u64().unwrap_or(0)).sum::<u64>() as usize),
            "routes": route_rows
        }),
    }
}

fn runtime_focus_contract(memory_records: usize) -> Value {
    let usage = nanda_6m::validate_packed_runtime(nanda_6m::PackedRuntimeShape {
        memory_records,
        centroids: 18,
        resident_lanes: 0,
        field_requests: nanda_6m::RUNTIME_FOCUS_FIELD_REQUESTS,
    });
    json!({
        "state": usage.state.as_str(),
        "ready": usage.ready(),
        "focus_window_fits": usage.focus_window_fits,
        "workspace_fits": usage.workspace_fits,
        "workspace_required_bytes": usage.workspace_required_bytes,
        "workspace_budget_bytes": usage.workspace_budget_bytes,
        "max_memory_records_for_requests": usage.max_memory_records_for_requests
    })
}

fn print_focus_text(report: &Value) {
    println!("NANDA FOCUS PACKET");
    println!(
        "state: {}",
        report["runtime_contract"]["state"].as_str().unwrap_or("")
    );
    println!(
        "triads: {} -> {}",
        report["original_memory_size"].as_u64().unwrap_or(0),
        report["focused_memory_size"].as_u64().unwrap_or(0)
    );
    println!(
        "routes: {} -> {}",
        report["original_routes"].as_u64().unwrap_or(0),
        report["focused_routes"].as_u64().unwrap_or(0)
    );
    if let Some(output) = report["output"].as_str() {
        println!("output: {output}");
    }
}

fn print_focus_md(report: &Value) {
    println!("# NANDA Focus Packet\n");
    println!(
        "- state: `{}`",
        report["runtime_contract"]["state"].as_str().unwrap_or("")
    );
    println!(
        "- triads: `{}` -> `{}`",
        report["original_memory_size"].as_u64().unwrap_or(0),
        report["focused_memory_size"].as_u64().unwrap_or(0)
    );
    println!(
        "- routes: `{}` -> `{}`",
        report["original_routes"].as_u64().unwrap_or(0),
        report["focused_routes"].as_u64().unwrap_or(0)
    );
    if let Some(output) = report["output"].as_str() {
        println!("- output: `{output}`");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_balanced_exact_focus_holds_hot_cap() {
        let mut memory = vec![];
        for route in 0..4 {
            for idx in 0..10 {
                memory.push(triad(
                    &format!("r{route}-{idx}"),
                    &format!("subject-{route}-{idx}"),
                    "links",
                    &format!("object-{route}-{idx}"),
                    "current",
                    "subject",
                    "object",
                    &format!("route-{route}"),
                    "group",
                ));
            }
        }
        let query = vec![triad(
            "q1",
            "subject-1-1",
            "links",
            "object-1-1",
            "query",
            "subject",
            "object",
            "",
            "query",
        )];

        let focus = route_balanced_focus_exact(&memory, &query, 9, 4, 10);

        assert_eq!(focus.memory.len(), 9);
        assert_eq!(
            focus.metadata["runtime_contract"]["state"].as_str(),
            Some("PACKED_RUNTIME_READY")
        );
        let routes = count_by(
            focus
                .memory
                .iter()
                .map(|triad| route_name(triad, "memory-route")),
        );
        assert!(routes.len() >= 3);
        assert!(routes.values().all(|count| *count <= 3));
    }

    #[test]
    fn focused_packet_preserves_query_triads() {
        let packet = Packet {
            task_id: "focus-test".to_string(),
            domain: "test".to_string(),
            query: "who links object".to_string(),
            triads: vec![triad(
                "t1", "a", "links", "b", "current", "subject", "object", "r1", "g1",
            )],
            candidate_triads: vec![],
            candidate_answer: String::new(),
            aliases: vec![],
            canonicalization: CanonicalizationReport::default(),
            negative_shortcuts: vec![],
            positive_shortcuts: vec![],
            resonance_memory: vec![],
            continuation_memory: vec![],
            failure_contract: Value::Null,
        };
        let query = vec![triad(
            "q1", "a", "links", "b", "query", "subject", "object", "", "query",
        )];

        let build = build_focused_packet(
            &packet,
            &packet.triads,
            &query,
            "candidate_triads",
            15,
            8,
            8,
        );

        assert_eq!(build.packet.triads.len(), 1);
        assert_eq!(build.packet.candidate_triads.len(), 1);
        assert_eq!(
            build.metadata["packet_task_id"].as_str(),
            Some("focus-test:focus")
        );
    }
}
