use crate::*;

pub(crate) fn route_set(triads: &[Triad]) -> BTreeSet<String> {
    triads
        .iter()
        .filter(|x| !x.route.is_empty())
        .map(|x| norm(&x.route))
        .collect()
}

pub(crate) fn group_memories(triads: &[Triad], fallback: &str) -> BTreeMap<String, Vec<i32>> {
    let mut memories = BTreeMap::new();
    for triad in triads {
        let group = if triad.group.trim().is_empty() {
            fallback.to_string()
        } else {
            norm(&triad.group)
        };
        let memory = memories.entry(group).or_insert_with(|| vec![0; WAVE_DIM]);
        add_into(memory, &triad_wave(triad));
    }
    memories
}

pub(crate) fn route_memories(triads: &[Triad], fallback: &str) -> BTreeMap<String, Vec<i32>> {
    let mut memories = BTreeMap::new();
    for triad in triads {
        let route = route_name(triad, fallback);
        let memory = memories.entry(route).or_insert_with(|| vec![0; WAVE_DIM]);
        add_into(memory, &triad_wave(triad));
    }
    memories
}

pub(crate) fn route_coherence(source: &[Triad], candidates: &[Triad]) -> Value {
    if source.is_empty() || candidates.is_empty() {
        return json!({"scores": {}, "weak": [], "best_source_group": {}});
    }
    let source_groups = group_memories(source, "source");
    let candidate_groups = group_memories(candidates, "candidate");
    let mut scores = serde_json::Map::new();
    let mut best_source_group = serde_json::Map::new();
    let mut weak = vec![];
    for (candidate_group, candidate_memory) in candidate_groups {
        let mut best_name = String::new();
        let mut best_score = -1.0;
        for (source_group, source_memory) in &source_groups {
            let score = cosine(source_memory, &candidate_memory);
            if score > best_score {
                best_score = score;
                best_name = source_group.clone();
            }
        }
        scores.insert(candidate_group.clone(), json!(round4(best_score)));
        best_source_group.insert(candidate_group.clone(), json!(best_name));
        let candidate_size = candidates
            .iter()
            .filter(|triad| {
                let group = if triad.group.trim().is_empty() {
                    "candidate".to_string()
                } else {
                    norm(&triad.group)
                };
                group == candidate_group
            })
            .count();
        let exact_source_groups = exact_match_source_groups(source, candidates, &candidate_group);
        if candidate_size >= 2 && (best_score < 0.65 || exact_source_groups.len() > 1) {
            weak.push(candidate_group);
        }
    }
    json!({"scores": scores, "weak": weak, "best_source_group": best_source_group})
}

pub(crate) fn exact_match_source_groups(
    source: &[Triad],
    candidates: &[Triad],
    candidate_group: &str,
) -> BTreeSet<String> {
    let mut source_by_key: HashMap<(String, String, String), BTreeSet<String>> = HashMap::new();
    for triad in source {
        source_by_key
            .entry(structural_key(triad))
            .or_default()
            .insert(if triad.group.trim().is_empty() {
                "source".to_string()
            } else {
                norm(&triad.group)
            });
    }
    let mut groups = BTreeSet::new();
    for candidate in candidates {
        let group = if candidate.group.trim().is_empty() {
            "candidate".to_string()
        } else {
            norm(&candidate.group)
        };
        if group != candidate_group {
            continue;
        }
        if let Some(matches) = source_by_key.get(&structural_key(candidate)) {
            groups.extend(matches.iter().cloned());
        }
    }
    groups
}

pub(crate) fn structural_map(source: &[Triad], candidates: &[Triad]) -> Value {
    let source_groups = group_memories(source, "source");
    let candidate_groups = group_memories(candidates, "candidate");
    let source_routes = route_memories(source, "source-route");
    let candidate_routes = route_memories(candidates, "candidate-route");
    let mut source_group_sizes = serde_json::Map::new();
    let mut candidate_group_sizes = serde_json::Map::new();
    for triad in source {
        let group = group_name(triad, "source");
        let current = source_group_sizes
            .get(&group)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        source_group_sizes.insert(group, json!(current + 1));
    }
    for triad in candidates {
        let group = group_name(triad, "candidate");
        let current = candidate_group_sizes
            .get(&group)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        candidate_group_sizes.insert(group, json!(current + 1));
    }

    let mut interference_matrix = serde_json::Map::new();
    let mut dominant_source_group = serde_json::Map::new();
    let mut mixed_candidate_groups = vec![];
    let mut group_links = serde_json::Map::new();
    let mut stable_groups = BTreeSet::new();
    let mut repair_tasks = vec![];
    let mut foreign_pull = vec![];

    for (candidate_group, candidate_memory) in &candidate_groups {
        let mut row = serde_json::Map::new();
        let mut best_group = String::new();
        let mut best_score = -1.0;
        for (source_group, source_memory) in &source_groups {
            let score = round4(cosine(source_memory, candidate_memory));
            row.insert(source_group.clone(), json!(score));
            if score > best_score {
                best_score = score;
                best_group = source_group.clone();
            }
        }
        let exact_groups = exact_match_source_groups(source, candidates, candidate_group);
        let exact_groups_vec: Vec<String> = exact_groups.iter().cloned().collect();
        if exact_groups.len() > 1 {
            mixed_candidate_groups.push(candidate_group.clone());
            repair_tasks.push(json!({
                "candidate_group": candidate_group,
                "reason": "candidate group contains exact triads from multiple source groups",
                "source_groups": exact_groups_vec,
                "suggested_fix": "Split this candidate group by source route before finalizing."
            }));
        }
        if !best_group.is_empty() && best_score < 0.65 {
            repair_tasks.push(json!({
                "candidate_group": candidate_group,
                "reason": "candidate group has low interference coherence with every source group",
                "dominant_source_group": best_group,
                "score": best_score,
                "suggested_fix": "Check whether this candidate route is missing triads, using wrong roles, or mixing unrelated routes."
            }));
        }
        group_links.insert(
            candidate_group.clone(),
            json!({
                "dominant_source_group": best_group.clone(),
                "dominant_score": round4(best_score),
                "exact_source_groups": exact_groups_vec
            }),
        );
        if best_score >= 0.65 && exact_groups.len() <= 1 {
            stable_groups.insert(best_group.clone());
        }
        dominant_source_group.insert(candidate_group.clone(), json!(best_group));
        interference_matrix.insert(candidate_group.clone(), json!(row));
    }

    for candidate in candidates {
        let candidate_wave = triad_wave(candidate);
        let candidate_group = group_name(candidate, "candidate");
        let candidate_dominant = dominant_source_group
            .get(&candidate_group)
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let (best_source_group, best_source_score) =
            best_memory_match(&source_groups, &candidate_wave);
        let exact_groups = exact_match_source_groups_for_candidate(source, candidate);
        let exact_groups_vec: Vec<String> = exact_groups.iter().cloned().collect();
        let is_foreign = !candidate_dominant.is_empty()
            && !best_source_group.is_empty()
            && (best_source_group != candidate_dominant
                || exact_groups.len() > 1
                || (!exact_groups.is_empty() && !exact_groups.contains(candidate_dominant)));
        if is_foreign {
            foreign_pull.push(json!({
                "candidate_triad": candidate.id,
                "candidate_group": candidate_group,
                "dominant_source_group": candidate_dominant,
                "triad_best_source_group": best_source_group,
                "triad_best_score": round4(best_source_score),
                "exact_source_groups": exact_groups_vec,
                "relation": candidate.relation,
                "subject": candidate.subject,
                "object": candidate.object,
                "repair": "Move this triad to the matching route or split the candidate group."
            }));
        }
    }

    json!({
        "core_version": CORE_VERSION,
        "wave_dim": nanda_6m::WAVE_DIM,
        "source_group_sizes": source_group_sizes,
        "candidate_group_sizes": candidate_group_sizes,
        "group_centroids": {
            "source": centroid_summary(source, &source_groups, GroupAxis::Group),
            "candidate": centroid_summary(candidates, &candidate_groups, GroupAxis::Group)
        },
        "route_memory": {
            "source": centroid_summary(source, &source_routes, GroupAxis::Route),
            "candidate": centroid_summary(candidates, &candidate_routes, GroupAxis::Route),
            "interference_matrix": interference_matrix_for(&source_routes, &candidate_routes)
        },
        "candidate_superposition": memory_summary(candidates, &build_memory(candidates)),
        "interference_matrix": interference_matrix,
        "dominant_source_group": dominant_source_group,
        "group_links": group_links,
        "stable_groups": stable_groups.into_iter().collect::<Vec<_>>(),
        "mixed_candidate_groups": mixed_candidate_groups,
        "foreign_pull": foreign_pull,
        "repair_tasks": repair_tasks
    })
}

pub(crate) fn group_name(triad: &Triad, fallback: &str) -> String {
    if triad.group.trim().is_empty() {
        fallback.to_string()
    } else {
        norm(&triad.group)
    }
}

pub(crate) fn route_name(triad: &Triad, fallback: &str) -> String {
    if triad.route.trim().is_empty() {
        fallback.to_string()
    } else {
        norm(&triad.route)
    }
}

pub(crate) fn exact_match_source_groups_for_candidate(
    source: &[Triad],
    candidate: &Triad,
) -> BTreeSet<String> {
    let mut groups = BTreeSet::new();
    let candidate_key = structural_key(candidate);
    for triad in source {
        if structural_key(triad) == candidate_key {
            groups.insert(group_name(triad, "source"));
        }
    }
    groups
}

pub(crate) fn print_map_text(map: &Value) {
    println!(
        "core_version: {}",
        map["core_version"].as_str().unwrap_or("")
    );
    println!("wave_dim: {}", map["wave_dim"].as_u64().unwrap_or(0));
    println!("mixed_candidate_groups:");
    if let Some(groups) = map["mixed_candidate_groups"].as_array() {
        for group in groups {
            println!("  - {}", group.as_str().unwrap_or(""));
        }
    }
    println!("repair_tasks:");
    if let Some(tasks) = map["repair_tasks"].as_array() {
        for task in tasks {
            println!(
                "  - {}: {}",
                task["candidate_group"].as_str().unwrap_or(""),
                task["reason"].as_str().unwrap_or("")
            );
        }
    }
}

pub(crate) fn print_map_md(map: &Value) {
    println!("# NANDA Structural Map\n");
    println!(
        "- core_version: `{}`",
        map["core_version"].as_str().unwrap_or("")
    );
    println!("- wave_dim: `{}`", map["wave_dim"].as_u64().unwrap_or(0));
    if let Some(groups) = map["mixed_candidate_groups"].as_array() {
        if !groups.is_empty() {
            println!("\n## Mixed Candidate Groups\n");
            for group in groups {
                println!("- `{}`", group.as_str().unwrap_or(""));
            }
        }
    }
    if let Some(tasks) = map["repair_tasks"].as_array() {
        if !tasks.is_empty() {
            println!("\n## Repair Tasks\n");
            for task in tasks {
                println!(
                    "- `{}`: {}",
                    task["candidate_group"].as_str().unwrap_or(""),
                    task["reason"].as_str().unwrap_or("")
                );
            }
        }
    }
}

pub(crate) fn split_md(args: SplitArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.input)?;
    let (triads, candidates) = parse_markdown_tables(&text, args.normalize_paths);
    if matches!(args.by, SplitBy::LinkedGroup) {
        return split_md_linked_group(args, triads, candidates);
    }
    let key_of = |row: &Triad| -> String {
        let raw = match args.by {
            SplitBy::Group => &row.group,
            SplitBy::Route => &row.route,
            SplitBy::LinkedGroup => unreachable!("linked-group split is handled before raw split"),
        };
        if raw.trim().is_empty() {
            "ungrouped".to_string()
        } else {
            raw.clone()
        }
    };
    let keys: BTreeSet<String> = triads.iter().chain(candidates.iter()).map(key_of).collect();
    fs::create_dir_all(&args.out_dir)?;
    let prefix = args.prefix.unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let mut written = vec![];
    for key in keys {
        let src: Vec<Triad> = triads
            .iter()
            .filter(|row| key_of(row) == key)
            .cloned()
            .collect();
        let cand: Vec<Triad> = candidates
            .iter()
            .filter(|row| key_of(row) == key)
            .cloned()
            .collect();
        let path = args.out_dir.join(format!(
            "{}-{}-{}.md",
            prefix,
            split_label(&args.by),
            slug(&key)
        ));
        fs::write(
            &path,
            split_worksheet(&args.input, split_label(&args.by), &key, &src, &cand),
        )?;
        written.push(path.display().to_string());
    }
    println!("{}", serde_json::to_string_pretty(&written)?);
    Ok(EXIT_PASS)
}

pub(crate) fn split_packet(args: SplitPacketArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let triads = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    fs::create_dir_all(&args.out_dir)?;
    let prefix = args.prefix.unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let splits = match args.by {
        SplitBy::LinkedGroup => linked_group_splits(&triads, &candidates),
        SplitBy::Group | SplitBy::Route => raw_splits(&triads, &candidates, &args.by),
    };
    let mut written = vec![];
    for split in &splits.items {
        let stem = format!("{}-{}-{}", prefix, split_label(&args.by), slug(&split.key));
        let path = match args.format {
            SplitOutputFormat::Json => args.out_dir.join(format!("{stem}.json")),
            SplitOutputFormat::Md => args.out_dir.join(format!("{stem}.md")),
        };
        match args.format {
            SplitOutputFormat::Json => {
                let split_packet = Packet {
                    task_id: format!("{}:{}", packet.task_id, split.key),
                    domain: packet.domain.clone(),
                    query: packet.query.clone(),
                    triads: split.triads.clone(),
                    candidate_triads: split.candidates.clone(),
                    candidate_answer: packet.candidate_answer.clone(),
                    aliases: packet.aliases.clone(),
                    canonicalization: packet.canonicalization.clone(),
                    negative_shortcuts: packet.negative_shortcuts.clone(),
                    positive_shortcuts: packet.positive_shortcuts.clone(),
                    resonance_memory: packet.resonance_memory.clone(),
                    continuation_memory: packet.continuation_memory.clone(),
                };
                fs::write(&path, serde_json::to_string_pretty(&split_packet)? + "\n")?;
            }
            SplitOutputFormat::Md => {
                fs::write(
                    &path,
                    split_worksheet(
                        &args.input,
                        split_label(&args.by),
                        &split.key,
                        &split.triads,
                        &split.candidates,
                    ),
                )?;
            }
        }
        written.push(path.display().to_string());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "mode": split_label(&args.by),
            "format": split_output_label(&args.format),
            "written": written,
            "warnings": splits.warnings
        }))?
    );
    Ok(EXIT_PASS)
}

pub(crate) fn split_md_linked_group(
    args: SplitArgs,
    triads: Vec<Triad>,
    candidates: Vec<Triad>,
) -> Result<u8> {
    let splits = linked_group_splits(&triads, &candidates);
    fs::create_dir_all(&args.out_dir)?;
    let prefix = args.prefix.unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let mut written = vec![];
    for split in &splits.items {
        let path = args
            .out_dir
            .join(format!("{}-linked-group-{}.md", prefix, slug(&split.key)));
        fs::write(
            &path,
            split_worksheet(
                &args.input,
                "linked-group",
                &split.key,
                &split.triads,
                &split.candidates,
            ),
        )?;
        written.push(path.display().to_string());
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "mode": "linked-group",
            "written": written,
            "warnings": splits.warnings
        }))?
    );
    Ok(EXIT_PASS)
}

pub(crate) fn raw_splits(triads: &[Triad], candidates: &[Triad], by: &SplitBy) -> SplitResult {
    let key_of = |row: &Triad| -> String {
        let raw = match by {
            SplitBy::Group => &row.group,
            SplitBy::Route => &row.route,
            SplitBy::LinkedGroup => unreachable!("linked-group has a separate split mode"),
        };
        if raw.trim().is_empty() {
            "ungrouped".to_string()
        } else {
            raw.clone()
        }
    };
    let keys: BTreeSet<String> = triads.iter().chain(candidates.iter()).map(key_of).collect();
    let items = keys
        .into_iter()
        .map(|key| SplitItem {
            triads: triads
                .iter()
                .filter(|row| key_of(row) == key)
                .cloned()
                .collect(),
            candidates: candidates
                .iter()
                .filter(|row| key_of(row) == key)
                .cloned()
                .collect(),
            key,
        })
        .collect();
    SplitResult {
        items,
        warnings: vec![],
    }
}

pub(crate) fn linked_group_splits(triads: &[Triad], candidates: &[Triad]) -> SplitResult {
    let map = structural_map(triads, candidates);
    let mut candidate_to_sources: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if let Some(group_links) = map["group_links"].as_object() {
        for (candidate_group, link) in group_links {
            let entry = candidate_to_sources
                .entry(candidate_group.clone())
                .or_default();
            if let Some(source_group) = link["dominant_source_group"].as_str() {
                if !source_group.is_empty() {
                    entry.insert(source_group.to_string());
                }
            }
            if let Some(exact_groups) = link["exact_source_groups"].as_array() {
                for source_group in exact_groups {
                    if let Some(source_group) = source_group.as_str() {
                        if !source_group.is_empty() {
                            entry.insert(source_group.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut source_to_candidates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (candidate_group, source_groups) in &candidate_to_sources {
        for source_group in source_groups {
            source_to_candidates
                .entry(source_group.clone())
                .or_default()
                .insert(candidate_group.clone());
        }
    }

    let source_groups: BTreeSet<String> = triads
        .iter()
        .map(|triad| group_name(triad, "source"))
        .collect();
    let mut items = vec![];
    let mut warnings = vec![];
    for source_group in source_groups {
        let source_rows: Vec<Triad> = triads
            .iter()
            .filter(|row| group_name(row, "source") == source_group)
            .cloned()
            .collect();
        let linked_candidate_groups = source_to_candidates
            .get(&source_group)
            .cloned()
            .unwrap_or_default();
        let candidate_rows: Vec<Triad> = candidates
            .iter()
            .filter(|row| {
                let exact_groups = exact_match_source_groups_for_candidate(triads, row);
                if !exact_groups.is_empty() {
                    return exact_groups.contains(&source_group);
                }
                linked_candidate_groups.contains(&group_name(row, "candidate"))
            })
            .cloned()
            .collect();
        if candidate_rows.is_empty() {
            warnings.push(format!(
                "source group {source_group} has no linked candidate group"
            ));
        }
        items.push(SplitItem {
            key: source_group,
            triads: source_rows,
            candidates: candidate_rows,
        });
    }

    let linked_candidates: BTreeSet<String> = candidate_to_sources.keys().cloned().collect();
    let all_candidate_groups: BTreeSet<String> = candidates
        .iter()
        .map(|triad| group_name(triad, "candidate"))
        .collect();
    for candidate_group in all_candidate_groups.difference(&linked_candidates) {
        warnings.push(format!(
            "candidate group {candidate_group} has no linked source group"
        ));
    }

    SplitResult { items, warnings }
}

pub(crate) fn map_cmd(args: MapArgs) -> Result<u8> {
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
    let mut map = structural_map(&source, &candidates);
    map["canonicalization"] = json!(packet.canonicalization);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&map)?),
        OutputFormat::Text => print_map_text(&map),
        OutputFormat::Md => print_map_md(&map),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn hgate_cmd(args: HgateArgs) -> Result<u8> {
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
    let global_report = make_report(&packet, &source, &candidates)?;
    let global_map = structural_map(&source, &candidates);
    let splits = match args.by {
        SplitBy::LinkedGroup => linked_group_splits(&source, &candidates),
        SplitBy::Group | SplitBy::Route => raw_splits(&source, &candidates, &args.by),
    };
    let mut branches = vec![];
    for split in splits.items.iter().take(args.max_branches) {
        let branch_packet = Packet {
            task_id: format!("{}:{}", packet.task_id, split.key),
            domain: packet.domain.clone(),
            query: packet.query.clone(),
            triads: split.triads.clone(),
            candidate_triads: split.candidates.clone(),
            candidate_answer: packet.candidate_answer.clone(),
            aliases: packet.aliases.clone(),
            canonicalization: packet.canonicalization.clone(),
            negative_shortcuts: packet.negative_shortcuts.clone(),
            positive_shortcuts: packet.positive_shortcuts.clone(),
            resonance_memory: packet.resonance_memory.clone(),
            continuation_memory: packet.continuation_memory.clone(),
        };
        let report = make_report(&branch_packet, &split.triads, &split.candidates)?;
        branches.push(json!({
            "key": split.key,
            "source_triads": split.triads.len(),
            "candidate_triads": split.candidates.len(),
            "verdict": report.verdict,
            "limits": report.limits,
            "conflicts": report.conflicts,
            "evidence_gaps": report.evidence_gaps,
            "weak_triads": report.weak_triads,
            "canonicalization": report.canonicalization,
            "foreign_pull": report.structural_map["foreign_pull"],
            "repair_prompt": report.repair_prompt,
            "trace_path": report.trace_path
        }));
    }
    let truncated = splits.items.len().saturating_sub(branches.len());
    let decision = hierarchical_decision(&global_report, &global_map, &branches, truncated);
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "hierarchical-gate",
        "input": args.input,
        "split_by": split_label(&args.by),
        "global": {
            "verdict": global_report.verdict,
            "complexity_score": global_report.complexity_score,
            "limits": global_report.limits,
            "conflicts": global_report.conflicts,
            "evidence_gaps": global_report.evidence_gaps,
            "weak_triads": global_report.weak_triads,
            "foreign_pull": global_map["foreign_pull"],
            "mixed_candidate_groups": global_map["mixed_candidate_groups"],
            "repair_tasks": global_map["repair_tasks"],
            "trace_path": global_report.trace_path
        },
        "canonicalization": packet.canonicalization,
        "branches": branches,
        "split_warnings": splits.warnings,
        "truncated_branches": truncated,
        "hierarchical_decision": decision,
        "interpretation": "Global WATCH caused only by size can be accepted when every linked branch passes. Global foreign_pull, conflicts, or any local VETO remain repair blockers."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_hgate_text(&out),
        OutputFormat::Md => print_hgate_md(&out),
    }
    Ok(
        match out["hierarchical_decision"]["action"]
            .as_str()
            .unwrap_or("REVIEW_REQUIRED")
        {
            "STRUCTURALLY_ACCEPTED" => EXIT_PASS,
            "REPAIR_REQUIRED" => EXIT_VETO,
            _ => EXIT_WATCH,
        },
    )
}

pub(crate) fn comb_cmd(args: CombArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let triads = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let branch_by = parse_csv(&args.branch_by);
    let stop_on = parse_csv(&args.stop_on);
    let topology = topology(&triads, &candidates);
    let comb_tree = comb_node(
        "root",
        0,
        args.depth,
        &branch_by,
        &stop_on,
        args.max_branches,
        &packet,
        &triads,
        &candidates,
    )?;
    let summary = comb_summary(&comb_tree);
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "depth": args.depth,
        "branch_by": branch_by,
        "stop_on": stop_on,
        "topology": topology,
        "comb_tree": comb_tree,
        "summary": summary
    });
    if let Some(out_dir) = args.out_dir {
        fs::create_dir_all(&out_dir)?;
        fs::write(
            out_dir.join("comb.json"),
            serde_json::to_string_pretty(&out)? + "\n",
        )?;
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_comb_text(&out),
        OutputFormat::Md => print_comb_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn builtin_route_trap_packet(noisy: bool) -> Packet {
    let triads = if noisy {
        vec![
            doctor_triad(
                "m1",
                "Monster",
                "has_route",
                "certification",
                "product",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m2",
                "certification payment",
                "pays_for",
                "TR CU declaration",
                "payment",
                "document",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m3",
                "TR CU declaration",
                "requires",
                "test protocols",
                "document",
                "evidence",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m4",
                "Maria Elena payment",
                "belongs_to",
                "certification route",
                "payment",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m5",
                "Monster",
                "produced_by",
                "Guangzhou 998",
                "product",
                "factory",
                "production",
                "production-route",
            ),
            doctor_triad(
                "m6",
                "Monster",
                "packed_at",
                "Guangzhou 998",
                "product",
                "factory",
                "production",
                "production-route",
            ),
            doctor_triad(
                "m7",
                "Guangzhou 998",
                "ships",
                "Monster",
                "factory",
                "product",
                "production",
                "production-route",
            ),
            doctor_triad(
                "m8",
                "importer",
                "files",
                "customs declaration",
                "company",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m9",
                "customs declaration",
                "requires",
                "payment confirmation",
                "document",
                "evidence",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m10",
                "Maria Elena payment",
                "not_pays_for",
                "customs declaration",
                "payment",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m11",
                "customs declaration",
                "not_same_as",
                "TR CU declaration",
                "document",
                "document",
                "customs",
                "customs-route",
            ),
        ]
    } else {
        vec![
            doctor_triad(
                "m1",
                "Monster",
                "has_route",
                "certification",
                "product",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m2",
                "certification payment",
                "pays_for",
                "declaration",
                "payment",
                "document",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m3",
                "declaration",
                "requires",
                "protocols",
                "document",
                "evidence",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m4",
                "protocols",
                "support",
                "certification",
                "evidence",
                "route",
                "certification",
                "certification-route",
            ),
            doctor_triad(
                "m5",
                "Monster",
                "pays_for",
                "declaration",
                "product",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m6",
                "customs declaration",
                "requires",
                "payment confirmation",
                "document",
                "evidence",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m7",
                "importer",
                "files",
                "customs declaration",
                "company",
                "document",
                "customs",
                "customs-route",
            ),
            doctor_triad(
                "m8",
                "Monster",
                "produced_by",
                "Guangzhou 998",
                "product",
                "factory",
                "production",
                "production-route",
            ),
        ]
    };
    let candidate_triads = if noisy {
        vec![
            doctor_triad("q1", "Monster", "", "", "product", "", "", "query"),
            doctor_triad(
                "q2", "payment", "pays_for", "document", "payment", "document", "", "query",
            ),
            doctor_triad(
                "q3", "document", "requires", "evidence", "document", "evidence", "", "query",
            ),
        ]
    } else {
        vec![
            doctor_triad(
                "q1",
                "Monster",
                "pays_for",
                "declaration",
                "product",
                "document",
                "",
                "query",
            ),
            doctor_triad(
                "q2",
                "declaration",
                "requires",
                "protocols",
                "document",
                "evidence",
                "",
                "query",
            ),
        ]
    };
    Packet {
        task_id: "doctor-route-trap".to_string(),
        domain: "doctor".to_string(),
        query: "builtin doctor route trap".to_string(),
        triads,
        candidate_triads,
        candidate_answer: String::new(),
        aliases: vec![],
        canonicalization: CanonicalizationReport::default(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
        resonance_memory: vec![],
        continuation_memory: vec![],
    }
}

pub(crate) fn route_balanced_focus(
    memory: &[Triad],
    query: &[Triad],
    route_cap: usize,
    route_triad_cap: usize,
) -> FocusedMemory {
    let route_cap = route_cap.max(1);
    let route_triad_cap = route_triad_cap.max(1);
    if memory.len() <= route_cap {
        return FocusedMemory {
            memory: memory.to_vec(),
            metadata: json!({
                "enabled": false,
                "reason": "memory_size_within_route_cap",
                "route_cap": route_cap,
                "route_triad_cap": route_triad_cap,
                "original_memory_size": memory.len(),
                "focused_memory_size": memory.len()
            }),
        };
    }
    let query_terms = query_term_set(query);
    let mut by_route: BTreeMap<String, Vec<(f64, &Triad)>> = BTreeMap::new();
    for triad in memory {
        let relevance = (0.52 * symbolic_query_overlap(query, triad))
            + (0.28 * token_overlap(&query_terms, triad))
            + (0.20 * source_weight(triad));
        by_route
            .entry(route_name(triad, "memory-route"))
            .or_default()
            .push((round4(relevance), triad));
    }
    let mut focused = vec![];
    let mut route_rows = vec![];
    for (route, mut items) in by_route {
        items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let before = items.len();
        let selected = items
            .iter()
            .take(route_triad_cap)
            .map(|(_, triad)| (*triad).clone())
            .collect::<Vec<_>>();
        route_rows.push(json!({
            "route": route,
            "original_triads": before,
            "selected_triads": selected.len(),
            "top_relevance": items.first().map(|(score, _)| *score).unwrap_or(0.0)
        }));
        focused.extend(selected);
    }
    route_rows.sort_by(|a, b| {
        b["top_relevance"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["top_relevance"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    FocusedMemory {
        memory: focused,
        metadata: json!({
            "enabled": true,
            "reason": "memory_size_exceeded_route_cap",
            "route_cap": route_cap,
            "route_triad_cap": route_triad_cap,
            "original_memory_size": memory.len(),
            "focused_memory_size": route_rows.iter().map(|row| row["selected_triads"].as_u64().unwrap_or(0)).sum::<u64>() as usize,
            "routes": route_rows
        }),
    }
}

pub(crate) fn route_term_coverage(
    query_terms: &BTreeSet<String>,
    items: &[(f64, f64, f64, &Triad)],
) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let mut peak_terms = BTreeSet::new();
    for (_, _, _, triad) in items {
        peak_terms.extend(triad_term_set(triad));
    }
    query_terms.intersection(&peak_terms).count() as f64 / query_terms.len() as f64
}

pub(crate) fn topology(source: &[Triad], candidates: &[Triad]) -> Value {
    let mut nodes = BTreeSet::new();
    let mut edges = vec![];
    let mut groups = BTreeSet::new();
    let mut routes = BTreeSet::new();
    for (kind, triads) in [("source", source), ("candidate", candidates)] {
        for triad in triads {
            nodes.insert(triad.subject.clone());
            nodes.insert(triad.object.clone());
            groups.insert(group_name(triad, kind));
            routes.insert(route_name(triad, &format!("{kind}-route")));
            edges.push(json!({
                "id": triad.id,
                "kind": kind,
                "subject": triad.subject,
                "relation": triad.relation,
                "object": triad.object,
                "subject_role": triad.subject_role,
                "object_role": triad.object_role,
                "route": triad.route,
                "group": triad.group,
                "evidence": triad.evidence
            }));
        }
    }
    json!({
        "nodes": nodes.into_iter().collect::<Vec<_>>(),
        "edges": edges,
        "groups": groups.into_iter().collect::<Vec<_>>(),
        "routes": routes.into_iter().collect::<Vec<_>>()
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn comb_node(
    path: &str,
    depth: usize,
    max_depth: usize,
    branch_by: &[String],
    stop_on: &[String],
    max_branches: usize,
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
) -> Result<Value> {
    let local_packet = Packet {
        task_id: format!("{}:{path}", packet.task_id),
        domain: packet.domain.clone(),
        query: packet.query.clone(),
        triads: source.to_vec(),
        candidate_triads: candidates.to_vec(),
        candidate_answer: packet.candidate_answer.clone(),
        aliases: packet.aliases.clone(),
        canonicalization: packet.canonicalization.clone(),
        negative_shortcuts: packet.negative_shortcuts.clone(),
        positive_shortcuts: packet.positive_shortcuts.clone(),
        resonance_memory: packet.resonance_memory.clone(),
        continuation_memory: packet.continuation_memory.clone(),
    };
    let report = make_report(&local_packet, source, candidates)?;
    let map = structural_map(source, candidates);
    let invariants = if depth >= 1 && branch_by.iter().any(|item| item == "subject-relation") {
        invariant_scan(source, candidates)
    } else {
        json!({"violations": [], "checked": []})
    };
    let stop_reasons = stop_reasons(&report, &map, &invariants, stop_on);
    let mut children = vec![];
    if depth == 0 && depth < max_depth && !stop_reasons.iter().any(|item| item == "foreign_pull") {
        if branch_by.iter().any(|item| item == "linked-group") {
            let splits = linked_group_splits(source, candidates);
            for split in splits.items.into_iter().take(max_branches) {
                children.push(comb_node(
                    &format!("{path}/linked-group:{}", split.key),
                    depth + 1,
                    max_depth,
                    branch_by,
                    stop_on,
                    max_branches,
                    packet,
                    &split.triads,
                    &split.candidates,
                )?);
            }
        } else if branch_by.iter().any(|item| item == "route") {
            let splits = raw_splits(source, candidates, &SplitBy::Route);
            for split in splits.items.into_iter().take(max_branches) {
                children.push(comb_node(
                    &format!("{path}/route:{}", split.key),
                    depth + 1,
                    max_depth,
                    branch_by,
                    stop_on,
                    max_branches,
                    packet,
                    &split.triads,
                    &split.candidates,
                )?);
            }
        }
    }
    Ok(json!({
        "path": path,
        "depth": depth,
        "verdict": report.verdict,
        "complexity_score": report.complexity_score,
        "limits": report.limits,
        "map": map,
        "invariants": invariants,
        "stop_reasons": stop_reasons,
        "children": children
    }))
}

pub(crate) fn comb_summary(tree: &Value) -> Value {
    let mut summary = BTreeMap::from([
        ("pass".to_string(), 0usize),
        ("watch".to_string(), 0usize),
        ("veto".to_string(), 0usize),
        ("invariant_violation".to_string(), 0usize),
        ("foreign_pull".to_string(), 0usize),
    ]);
    accumulate_comb_summary(tree, &mut summary);
    json!(summary)
}

pub(crate) fn accumulate_comb_summary(node: &Value, summary: &mut BTreeMap<String, usize>) {
    match node["verdict"].as_str().unwrap_or("") {
        "PASS" => *summary.entry("pass".to_string()).or_default() += 1,
        "WATCH" => *summary.entry("watch".to_string()).or_default() += 1,
        "VETO" => *summary.entry("veto".to_string()).or_default() += 1,
        _ => {}
    }
    if node["invariants"]["violations"]
        .as_array()
        .is_some_and(|items| !items.is_empty())
    {
        *summary
            .entry("invariant_violation".to_string())
            .or_default() += 1;
    }
    if node["map"]["foreign_pull"]
        .as_array()
        .is_some_and(|items| !items.is_empty())
    {
        *summary.entry("foreign_pull".to_string()).or_default() += 1;
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            accumulate_comb_summary(child, summary);
        }
    }
}

pub(crate) fn print_comb_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("depth: {}", out["depth"].as_u64().unwrap_or(0));
    println!("summary: {}", out["summary"]);
}

pub(crate) fn print_hgate_text(out: &Value) {
    let decision = &out["hierarchical_decision"];
    println!("NANDA HGATE");
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "ACTION: {}",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "GLOBAL: {}{}",
        decision["global_verdict"].as_str().unwrap_or("WATCH"),
        if decision["global_size_only"].as_bool().unwrap_or(false) {
            " size-only"
        } else {
            ""
        }
    );
    println!(
        "BRANCHES: {}/{} PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["branches"].as_u64().unwrap_or(0)
    );
    println!(
        "BLOCKERS: local_veto={} local_watch={} foreign_pull={} truncated={}",
        decision["local_veto"].as_u64().unwrap_or(0),
        decision["local_watch"].as_u64().unwrap_or(0),
        decision["global_foreign_pull"].as_u64().unwrap_or(0),
        decision["truncated_branches"].as_u64().unwrap_or(0)
    );
    println!(
        "ACCEPTED: {}",
        decision["structurally_accepted"].as_bool().unwrap_or(false)
    );
    println!("NEXT: {}", decision["next"].as_str().unwrap_or(""));
}

pub(crate) fn print_hgate_md(out: &Value) {
    let decision = &out["hierarchical_decision"];
    println!("# NANDA Hierarchical Gate\n");
    println!(
        "- action: `{}`",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "- global: `{}`",
        decision["global_verdict"].as_str().unwrap_or("WATCH")
    );
    println!(
        "- global_size_only: `{}`",
        decision["global_size_only"].as_bool().unwrap_or(false)
    );
    println!(
        "- branches: `{}/{}` PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["branches"].as_u64().unwrap_or(0)
    );
    println!(
        "- blockers: `local_veto={} local_watch={} foreign_pull={} truncated={}`",
        decision["local_veto"].as_u64().unwrap_or(0),
        decision["local_watch"].as_u64().unwrap_or(0),
        decision["global_foreign_pull"].as_u64().unwrap_or(0),
        decision["truncated_branches"].as_u64().unwrap_or(0)
    );
    println!(
        "- structurally_accepted: `{}`",
        decision["structurally_accepted"].as_bool().unwrap_or(false)
    );
    println!("- next: {}", decision["next"].as_str().unwrap_or(""));
}

pub(crate) fn print_comb_md(out: &Value) {
    println!("# NANDA Comb\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- depth: `{}`", out["depth"].as_u64().unwrap_or(0));
    println!("- summary: `{}`", out["summary"]);
}

pub(crate) fn split_label(by: &SplitBy) -> &'static str {
    match by {
        SplitBy::Group => "group",
        SplitBy::Route => "route",
        SplitBy::LinkedGroup => "linked-group",
    }
}

pub(crate) fn split_output_label(format: &SplitOutputFormat) -> &'static str {
    match format {
        SplitOutputFormat::Json => "json",
        SplitOutputFormat::Md => "md",
    }
}

pub(crate) fn split_worksheet(
    source: &Path,
    by: &str,
    key: &str,
    triads: &[Triad],
    candidates: &[Triad],
) -> String {
    format!(
        "# NANDA Split Worksheet\n\nsplit_by: {by}\nsplit_key: {key}\nsource: {}\n\n{}\n{}",
        source.display(),
        table("triads", triads),
        table("candidate_triads", candidates)
    )
}
