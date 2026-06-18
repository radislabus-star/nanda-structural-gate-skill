use crate::*;

pub(crate) fn dataset_doctor_cmd(args: DatasetDoctorArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let memory = normalize_ids(packet.triads.clone(), "m");
    let query = normalize_ids(packet.candidate_triads.clone(), "q");
    let out = corpus_diagnostics(&memory, &query, &packet.query, args.route_cap);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_dataset_doctor_text(&out),
        OutputFormat::Md => print_dataset_doctor_md(&out),
    }
    let verdict = out["verdict"].as_str().unwrap_or("WATCH");
    Ok(match verdict {
        "PASS" => EXIT_PASS,
        "VETO" => EXIT_VETO,
        "WATCH" => EXIT_WATCH,
        _ => EXIT_WATCH,
    })
}

pub(crate) fn corpus_diagnostics(
    memory: &[Triad],
    query: &[Triad],
    query_text: &str,
    route_cap: usize,
) -> Value {
    let triad_count = memory.len();
    let large_corpus = triad_count > route_cap;
    let route_counts = count_by(memory.iter().map(|triad| route_name(triad, "memory-route")));
    let group_counts = count_by(memory.iter().map(|triad| group_name(triad, "memory")));
    let relation_counts = count_by(memory.iter().map(|triad| norm(&triad.relation)));
    let mut entity_counts = BTreeMap::<String, usize>::new();
    for triad in memory {
        for entity in [norm(&triad.subject), norm(&triad.object)] {
            if !entity.is_empty() {
                *entity_counts.entry(entity).or_default() += 1;
            }
        }
    }
    let route_distribution = distribution_rows(&route_counts, triad_count, 8);
    let group_distribution = distribution_rows(&group_counts, triad_count, 8);
    let hub_nodes = hub_rows(&entity_counts, triad_count, 8);
    let top_route_share = route_distribution
        .first()
        .and_then(|row| row["share"].as_f64())
        .unwrap_or(0.0);
    let top_hub_share = hub_nodes
        .first()
        .and_then(|row| row["share"].as_f64())
        .unwrap_or(0.0);
    let duplicate_rows = duplicate_rows(memory, 8);
    let duplicate_count = duplicate_rows
        .iter()
        .map(|row| row["count"].as_u64().unwrap_or(0).saturating_sub(1))
        .sum::<u64>() as usize;
    let current_duplicates = duplicate_rows
        .iter()
        .filter(|row| {
            row["evidence_refs"].as_array().is_some_and(|refs| {
                refs.iter()
                    .any(|value| norm(value.as_str().unwrap_or("")).contains("current"))
            })
        })
        .count();
    let weak_query = query.is_empty() && !query_text.trim().is_empty() && large_corpus;
    let empty_query = query.is_empty() && query_text.trim().is_empty();
    let route_imbalance = large_corpus && top_route_share >= 0.45;
    let hub_dominance = large_corpus && top_hub_share >= 0.08;
    let duplicate_current = large_corpus && current_duplicates > 0;
    let mut warnings = vec![];
    let mut notices = vec![];
    if large_corpus && route_imbalance {
        warnings.push(json!({
            "kind": "large_unbalanced_corpus",
            "message": "Corpus exceeds direct-search route cap and is route-heavy; build a route-balanced focus packet before search."
        }));
    } else if large_corpus {
        notices.push(json!({
            "kind": "large_but_route_balanced_focus",
            "message": "Corpus exceeds direct-search route cap, but no single route dominates; continue with coarse-to-fine search or route caps."
        }));
    }
    if route_imbalance {
        warnings.push(json!({
            "kind": "route_imbalance",
            "message": "One route dominates the field; use stratified route sampling or coarse-to-fine search."
        }));
    }
    if hub_dominance {
        warnings.push(json!({
            "kind": "hub_dominance",
            "message": "A high-frequency entity behaves like a hub and may pin unrelated peaks."
        }));
    }
    if duplicate_current {
        warnings.push(json!({
            "kind": "duplicate_current",
            "message": "Current/canonical evidence appears duplicated with dated or archive copies."
        }));
    }
    if weak_query {
        warnings.push(json!({
            "kind": "weak_text_query",
            "message": "Text query without candidate_triads is weak on large corpora; provide query triads or extract lightweight query_triads."
        }));
    }
    if empty_query {
        warnings.push(json!({
            "kind": "empty_query",
            "message": "No candidate_triads or query text were provided; search activation will not be query-specific."
        }));
    }
    let verdict = if warnings.is_empty() { "PASS" } else { "WATCH" };
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "dataset-doctor",
        "verdict": verdict,
        "triad_count": triad_count,
        "route_count": route_counts.len(),
        "group_count": group_counts.len(),
        "relation_count": relation_counts.len(),
        "entity_count": entity_counts.len(),
        "route_cap": route_cap,
        "top_route_share": round4(top_route_share),
        "top_hub_share": round4(top_hub_share),
        "duplicate_structural_facts": duplicate_count,
        "current_duplicate_groups": current_duplicates,
        "query_triads": query.len(),
        "query_text_present": !query_text.trim().is_empty(),
        "warnings": warnings,
        "notices": notices,
        "route_distribution": route_distribution,
        "group_distribution": group_distribution,
        "hub_nodes": hub_nodes,
        "duplicate_examples": duplicate_rows,
        "recommended_pipeline": [
            "dataset-doctor",
            "route-balanced focus packet",
            "search with candidate_triads",
            "field_interpretation",
            "split by route if contested",
            "check candidate route",
            "feedback accept/reject/WATCH"
        ]
    })
}

pub(crate) fn distribution_rows(
    counts: &BTreeMap<String, usize>,
    total: usize,
    limit: usize,
) -> Vec<Value> {
    let mut rows = counts
        .iter()
        .map(|(name, count)| (name.clone(), *count))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter()
        .take(limit)
        .map(|(name, count)| {
            json!({
                "name": name,
                "count": count,
                "share": round4(count as f64 / total.max(1) as f64)
            })
        })
        .collect()
}

pub(crate) fn hub_rows(
    counts: &BTreeMap<String, usize>,
    triad_count: usize,
    limit: usize,
) -> Vec<Value> {
    let total_slots = triad_count.max(1) * 2;
    let mut rows = counts
        .iter()
        .map(|(name, count)| (name.clone(), *count))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter()
        .take(limit)
        .map(|(name, count)| {
            json!({
                "entity": name,
                "count": count,
                "share": round4(count as f64 / total_slots as f64)
            })
        })
        .collect()
}

pub(crate) fn duplicate_rows(memory: &[Triad], limit: usize) -> Vec<Value> {
    let mut groups: BTreeMap<(String, String, String), Vec<&Triad>> = BTreeMap::new();
    for triad in memory {
        groups.entry(structural_key(triad)).or_default().push(triad);
    }
    let mut rows = groups
        .into_iter()
        .filter(|(_, items)| items.len() > 1)
        .map(|((subject, relation, object), items)| {
            let evidence_refs = items
                .iter()
                .map(|triad| triad.evidence.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            json!({
                "subject": subject,
                "relation": relation,
                "object": object,
                "count": items.len(),
                "routes": items.iter().map(|triad| route_name(triad, "memory-route")).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>(),
                "evidence_refs": evidence_refs
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["count"].as_u64().unwrap_or(0))
            .then_with(|| {
                a["subject"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["subject"].as_str().unwrap_or(""))
            })
    });
    rows.truncate(limit);
    rows
}

pub(crate) fn print_dataset_doctor_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("verdict: {}", out["verdict"].as_str().unwrap_or(""));
    println!("triads: {}", out["triad_count"]);
    println!("routes: {}", out["route_count"]);
    println!("top_route_share: {}", out["top_route_share"]);
    println!("top_hub_share: {}", out["top_hub_share"]);
    println!(
        "duplicate_structural_facts: {}",
        out["duplicate_structural_facts"]
    );
    if let Some(warnings) = out["warnings"].as_array() {
        for warning in warnings {
            println!(
                "warning: {} - {}",
                warning["kind"].as_str().unwrap_or("warning"),
                warning["message"].as_str().unwrap_or("")
            );
        }
    }
}

pub(crate) fn print_dataset_doctor_md(out: &Value) {
    println!("# NANDA Dataset Doctor\n");
    println!("- core: `{}`", out["core_version"].as_str().unwrap_or(""));
    println!("- verdict: `{}`", out["verdict"].as_str().unwrap_or(""));
    println!("- triads: `{}`", out["triad_count"]);
    println!("- routes: `{}`", out["route_count"]);
    println!("- top_route_share: `{}`", out["top_route_share"]);
    println!("- top_hub_share: `{}`", out["top_hub_share"]);
    println!(
        "- duplicate_structural_facts: `{}`",
        out["duplicate_structural_facts"]
    );
    if let Some(warnings) = out["warnings"].as_array() {
        if !warnings.is_empty() {
            println!("\n## Warnings\n");
            for warning in warnings {
                println!(
                    "- `{}`: {}",
                    warning["kind"].as_str().unwrap_or("warning"),
                    warning["message"].as_str().unwrap_or("")
                );
            }
        }
    }
}
