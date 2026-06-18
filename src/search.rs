use crate::*;

pub(crate) fn search_cmd(args: SearchArgs) -> Result<u8> {
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let memory = normalize_ids(packet.triads.clone(), "m");
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
    let focus = route_balanced_focus(&memory, &query, args.route_cap, args.route_triad_cap);
    let result = interference_search(
        &packet,
        &focus.memory,
        &query,
        args.top_k,
        &args.group_by,
        query_source,
        focus.metadata,
    );
    let mut result = result;
    result["canonicalization"] = json!(packet.canonicalization);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
        OutputFormat::Text => print_search_text(&result),
        OutputFormat::Md => print_search_md(&result),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn probe_search_summary(search: &Value) -> Value {
    json!({
        "verdict": search["verdict"],
        "field_state": search["field_state"],
        "safe_to_answer": search["safe_to_answer"],
        "top_peak": search["top_peak"],
        "top_score": round4(top_peak_score(search)),
        "peak_margin": search["peak_margin"],
        "lexical_baseline_top": search["lexical_baseline"]["top_peak"],
        "wins_over_lexical_baseline": search["wins_over_lexical_baseline"]
    })
}

pub(crate) fn top_peak_score(search: &Value) -> f64 {
    search["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["score"].as_f64())
        .unwrap_or(0.0)
}

pub(crate) fn positive_shortcut_from_search(
    search: &Value,
    peak_name: &str,
    note: &str,
    source_feedback: String,
) -> PositiveShortcut {
    let top_peak = search["peaks"].as_array().and_then(|peaks| peaks.first());
    let reinforce_route = top_peak
        .and_then(|peak| peak["center"]["route"].as_str())
        .unwrap_or("")
        .to_string();
    let reinforce_group = top_peak
        .and_then(|peak| peak["center"]["group"].as_str())
        .unwrap_or("")
        .to_string();
    let support_terms = top_peak
        .and_then(|peak| peak["supporting_triads"].as_array())
        .map(|items| support_terms_from_items(items))
        .unwrap_or_default();
    let terms = query_tokens_from_search(search)
        .into_iter()
        .take(16)
        .collect::<Vec<_>>();
    PositiveShortcut {
        id: format!("pos-{}", slug(&format!("{peak_name}-{note}"))),
        reinforce_peak: peak_name.to_string(),
        reinforce_route,
        reinforce_group,
        boost: default_positive_boost(),
        terms,
        support_terms,
        reason: if note.trim().is_empty() {
            "accepted interference peak".to_string()
        } else {
            note.to_string()
        },
        source_feedback,
        observations: 1,
        accepted_count: 1,
        rejected_count: 0,
    }
}

pub(crate) fn negative_shortcut_from_search(
    search: &Value,
    peak_name: &str,
    note: &str,
    source_feedback: String,
) -> NegativeShortcut {
    let group_by = search["group_by"].as_str().unwrap_or("route");
    let top_peak = search["peaks"].as_array().and_then(|peaks| peaks.first());
    let suppress_route = top_peak
        .and_then(|peak| peak["center"]["route"].as_str())
        .unwrap_or("")
        .to_string();
    let suppress_group = top_peak
        .and_then(|peak| peak["center"]["group"].as_str())
        .unwrap_or("")
        .to_string();
    let support_terms = top_peak
        .and_then(|peak| peak["supporting_triads"].as_array())
        .map(|items| support_terms_from_items(items))
        .unwrap_or_default();
    let prefer_item = search["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["anti_triads"].as_array())
        .and_then(|items| items.first());
    let prefer_peak = search["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["anti_triads"].as_array())
        .and_then(|items| items.first())
        .and_then(|item| match group_by {
            "group" => item["group"].as_str(),
            _ => item["route"].as_str(),
        })
        .unwrap_or("")
        .to_string();
    let prefer_route = prefer_item
        .and_then(|item| item["route"].as_str())
        .unwrap_or("")
        .to_string();
    let prefer_group = prefer_item
        .and_then(|item| item["group"].as_str())
        .unwrap_or("")
        .to_string();
    let terms = query_tokens_from_search(search)
        .into_iter()
        .take(16)
        .collect::<Vec<_>>();
    NegativeShortcut {
        id: format!("neg-{}", slug(&format!("{peak_name}-{prefer_peak}-{note}"))),
        suppress_peak: peak_name.to_string(),
        suppress_route,
        suppress_group,
        prefer_peak,
        prefer_route,
        prefer_group,
        penalty: default_negative_penalty(),
        terms,
        support_terms,
        reason: if note.trim().is_empty() {
            "rejected interference peak".to_string()
        } else {
            note.to_string()
        },
        source_feedback,
        observations: 1,
        rejected_count: 1,
        accepted_count: 0,
    }
}

pub(crate) fn query_tokens_from_search(search: &Value) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    if let Some(items) = search["query"]["triads"].as_array() {
        for item in items {
            for key in [
                "subject",
                "relation",
                "object",
                "subject_role",
                "object_role",
                "route",
                "group",
            ] {
                for token in norm(item[key].as_str().unwrap_or(""))
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .filter(|token| token.len() >= 2)
                {
                    tokens.insert(token.to_string());
                }
            }
        }
    }
    tokens
}

pub(crate) fn search_query_triads(packet: &Packet, query_text: &str) -> (Vec<Triad>, &'static str) {
    if !packet.candidate_triads.is_empty() {
        return (
            normalize_ids(packet.candidate_triads.clone(), "q"),
            "candidate_triads",
        );
    }
    let auto = auto_query_triads(query_text);
    if auto.is_empty() {
        (vec![], "empty")
    } else {
        (normalize_ids(auto, "q"), "auto_query_triads")
    }
}

pub(crate) fn auto_query_triads(query_text: &str) -> Vec<Triad> {
    let tokens = query_tokens_from_text(query_text);
    if tokens.is_empty() {
        return vec![];
    }
    let topic = tokens.iter().take(5).cloned().collect::<Vec<_>>().join(" ");
    let mut triads = vec![Triad {
        id: "q1".to_string(),
        subject: "query".to_string(),
        relation: "asks_about".to_string(),
        object: topic.clone(),
        evidence: "auto_query".to_string(),
        confidence: 0.72,
        subject_role: "query".to_string(),
        object_role: "topic".to_string(),
        route: String::new(),
        group: "auto-query".to_string(),
    }];
    if tokens.len() >= 2 {
        triads.push(Triad {
            id: "q2".to_string(),
            subject: tokens[0].clone(),
            relation: "co_occurs_with".to_string(),
            object: tokens[1].clone(),
            evidence: "auto_query".to_string(),
            confidence: 0.66,
            subject_role: "query_term".to_string(),
            object_role: "query_term".to_string(),
            route: String::new(),
            group: "auto-query".to_string(),
        });
    }
    if tokens.len() >= 3 {
        triads.push(Triad {
            id: "q3".to_string(),
            subject: tokens[1].clone(),
            relation: "co_occurs_with".to_string(),
            object: tokens[2].clone(),
            evidence: "auto_query".to_string(),
            confidence: 0.62,
            subject_role: "query_term".to_string(),
            object_role: "query_term".to_string(),
            route: String::new(),
            group: "auto-query".to_string(),
        });
    }
    triads
}

pub(crate) fn query_tokens_from_text(query_text: &str) -> Vec<String> {
    let stop = [
        "about", "after", "again", "against", "and", "are", "before", "between", "can", "could",
        "does", "for", "from", "how", "into", "need", "needs", "not", "our", "route", "should",
        "that", "the", "then", "this", "what", "when", "where", "which", "with", "без", "где",
        "для", "как", "или", "что", "это",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut out = vec![];
    for token in norm(query_text)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.chars().count() >= 2)
    {
        if stop.contains(token) || !seen.insert(token.to_string()) {
            continue;
        }
        out.push(token.to_string());
        if out.len() == 12 {
            break;
        }
    }
    out
}

pub(crate) fn interference_search(
    packet: &Packet,
    memory: &[Triad],
    query: &[Triad],
    top_k: usize,
    group_by: &PeakGroupBy,
    query_source: &str,
    route_balanced_focus: Value,
) -> Value {
    let query_wave = query_feature_wave(query);
    let mut scored = vec![];
    for triad in memory {
        let wave = triad_feature_wave(triad);
        let score = cosine(&query_wave, &wave);
        let symbolic = symbolic_query_overlap(query, triad);
        let weight = source_weight(triad);
        let combined = round4(((0.72 * score) + (0.28 * symbolic)) * weight);
        scored.push((combined, score, symbolic, triad));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut groups: BTreeMap<String, Vec<(f64, f64, f64, &Triad)>> = BTreeMap::new();
    for item in &scored {
        let key = match group_by {
            PeakGroupBy::Group => group_name(item.3, "memory"),
            PeakGroupBy::Route => route_name(item.3, "memory-route"),
        };
        groups.entry(key).or_default().push(*item);
    }

    let query_terms = query_term_set(query);
    let mut peaks = vec![];
    for (key, items) in groups {
        let mut group_wave = vec![0; WAVE_DIM];
        for (_, _, _, triad) in &items {
            add_into(&mut group_wave, &triad_feature_wave(triad));
        }
        let coherence = round4(cosine(&query_wave, &group_wave));
        let coverage = round4(route_term_coverage(&query_terms, &items));
        let chain = round4(chain_coherence(&items));
        let propagation = propagation_summary(&query_terms, &items);
        let propagation_score = propagation["score"].as_f64().unwrap_or(0.0);
        let component_score = propagation["component_score"].as_f64().unwrap_or(0.0);
        let top_score = items.first().map(|item| item.0).unwrap_or(0.0);
        let avg_top3 = if items.is_empty() {
            0.0
        } else {
            let take = items.len().min(3);
            items.iter().take(take).map(|item| item.0).sum::<f64>() / take as f64
        };
        let base_peak_score = round4(
            (0.26 * coherence)
                + (0.18 * coverage)
                + (0.15 * avg_top3)
                + (0.11 * chain)
                + (0.12 * propagation_score)
                + (0.18 * component_score),
        );
        let support = items
            .iter()
            .take(8)
            .map(|(combined, wave_score, symbolic, triad)| {
                json!({
                    "id": triad.id,
                    "score": combined,
                    "wave_score": round4(*wave_score),
                    "symbolic_overlap": round4(*symbolic),
                    "source_weight": source_weight(triad),
                    "polarity": triad_polarity(triad),
                    "subject": triad.subject,
                    "relation": triad.relation,
                    "object": triad.object,
                    "route": triad.route,
                    "group": triad.group,
                    "evidence": triad.evidence
                })
            })
            .collect::<Vec<_>>();
        let anti = scored
            .iter()
            .filter(|(_, _, _, triad)| {
                let triad_key = match group_by {
                    PeakGroupBy::Group => group_name(triad, "memory"),
                    PeakGroupBy::Route => route_name(triad, "memory-route"),
                };
                triad_key != key
            })
            .take(5)
            .map(|(combined, wave_score, symbolic, triad)| {
                json!({
                    "id": triad.id,
                    "score": combined,
                    "wave_score": round4(*wave_score),
                    "symbolic_overlap": round4(*symbolic),
                    "source_weight": source_weight(triad),
                    "polarity": triad_polarity(triad),
                    "subject": triad.subject,
                    "relation": triad.relation,
                    "object": triad.object,
                    "route": triad.route,
                    "group": triad.group,
                    "reason": "similar query features but outside this peak route/group"
                })
            })
            .collect::<Vec<_>>();
        let center = peak_center(&items);
        let polarization = polarization_summary(query, &items);
        let polarization_penalty = polarization_penalty(&polarization);
        let peak_score = round4((base_peak_score - polarization_penalty).max(0.0));
        peaks.push(json!({
            "peak": key,
            "score": peak_score,
            "raw_score": base_peak_score,
            "polarization_penalty": polarization_penalty,
            "coherence": coherence,
            "coverage": coverage,
            "chain_coherence": chain,
            "propagation": propagation,
            "top_triad_score": round4(top_score),
            "symbolic_baseline": symbolic_peak_baseline(&items),
            "center": center,
            "polarization": polarization,
            "supporting_triads": support,
            "anti_triads": anti,
            "missing_edges": missing_edges(&query_terms, &items),
            "answer_projection": answer_projection(&center, &items)
        }));
    }
    let destructive_interference =
        apply_negative_lanes(&mut peaks, query, &packet.negative_shortcuts);
    let constructive_interference =
        apply_positive_lanes(&mut peaks, query, &packet.positive_shortcuts);
    peaks.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let peak_margin = if peaks.len() >= 2 {
        round4(
            peaks[0]["score"].as_f64().unwrap_or(0.0) - peaks[1]["score"].as_f64().unwrap_or(0.0),
        )
    } else {
        0.0
    };
    let lexical_baseline = lexical_baseline(&scored, query, group_by);
    let corpus_interpretation = corpus_diagnostics(memory, query, &packet.query, 256);
    let top_peak = peaks
        .first()
        .and_then(|peak| peak["peak"].as_str())
        .unwrap_or("")
        .to_string();
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let peak_decision = peak_decision(&peaks, peak_margin, lexical_peak);
    let mut field_interpretation = field_interpretation(&peaks, peak_margin, &lexical_baseline);
    if let Some(object) = field_interpretation.as_object_mut() {
        object.insert("corpus".to_string(), corpus_interpretation.clone());
    }
    let coarse_to_fine = coarse_to_fine_trace(&peaks, &query_terms);
    let field_state_machine = field_state_machine(
        &peaks,
        peak_margin,
        &lexical_baseline,
        &corpus_interpretation,
        &route_balanced_focus,
        &coarse_to_fine,
    );
    let field_state = field_state_machine["state"].as_str().unwrap_or("NO_FIELD");
    let safe_to_answer = field_state_machine["safe_to_answer"]
        .as_bool()
        .unwrap_or(false);
    let verdict = search_verdict(field_state, safe_to_answer);
    let mut output_peaks = peaks;
    output_peaks.truncate(top_k);

    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "interference-retrieval",
        "verdict": verdict,
        "field_state": field_state,
        "safe_to_answer": safe_to_answer,
        "top_peak": top_peak,
        "task_id": packet.task_id,
        "domain": packet.domain,
        "query": {
            "text": packet.query,
            "source": query_source,
            "triads": query.iter().map(triad_json).collect::<Vec<_>>()
        },
        "memory_size": memory.len(),
        "route_balanced_focus": route_balanced_focus,
        "group_by": match group_by {
            PeakGroupBy::Group => "group",
            PeakGroupBy::Route => "route",
        },
        "peak_margin": peak_margin,
        "lexical_baseline": lexical_baseline,
        "wins_over_lexical_baseline": !lexical_peak.is_empty() && top_peak.as_str() != lexical_peak,
        "peak_decision": peak_decision,
        "destructive_interference": destructive_interference,
        "constructive_interference": constructive_interference,
        "source_weighting": {
            "enabled": true,
            "policy": "confidence multiplied by evidence authority: current/canon > latest/frontier > historical/archive > archive_noise"
        },
        "coarse_to_fine": coarse_to_fine,
        "field_state_machine": field_state_machine,
        "field_interpretation": field_interpretation,
        "peaks": output_peaks,
        "interpretation": "A peak is a route/group whose triads resonate together with the partial query. Read support as the focused structure and anti_triads as similar-but-foreign pulls."
    })
}

pub(crate) fn coarse_to_fine_trace(peaks: &[Value], query_terms: &BTreeSet<String>) -> Value {
    let Some(top) = peaks.first() else {
        return json!({
            "enabled": true,
            "state": "NO_PEAK",
            "coarse_peak": "",
            "local_path": []
        });
    };
    let coarse_peak = top["peak"].as_str().unwrap_or("");
    let support = top["supporting_triads"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let local_path = support
        .iter()
        .take(5)
        .map(|item| {
            let subject = item["subject"].as_str().unwrap_or("");
            let relation = item["relation"].as_str().unwrap_or("");
            let object = item["object"].as_str().unwrap_or("");
            let mut hits = vec![];
            for term in query_terms {
                let haystack = norm(&format!("{subject} {relation} {object}"));
                if haystack.contains(term) {
                    hits.push(term.clone());
                }
            }
            json!({
                "triad": item["id"].as_str().unwrap_or(""),
                "edge": format!("{subject} -> {relation} -> {object}"),
                "score": item["score"].as_f64().unwrap_or(0.0),
                "polarity": item["polarity"].as_str().unwrap_or(""),
                "query_hits": hits
            })
        })
        .collect::<Vec<_>>();
    json!({
        "enabled": true,
        "state": if local_path.is_empty() { "THIN" } else { "LOCALIZED" },
        "coarse_peak": coarse_peak,
        "local_memory_size": support.len(),
        "local_path": local_path,
        "read_as": "Coarse route first, then inspect the local supporting path inside that route."
    })
}

pub(crate) fn search_verdict(field_state: &str, safe_to_answer: bool) -> &'static str {
    match field_state {
        "FIELD_REVERSED" => "VETO",
        "FIELD_FOCUSED" | "FIELD_SAFE" | "FIELD_ROUTE_BALANCED" if safe_to_answer => "PASS",
        _ => "WATCH",
    }
}

pub(crate) fn peak_decision(peaks: &[Value], margin: f64, lexical_peak: &str) -> Value {
    if peaks.is_empty() {
        return json!({
            "state": "NO_PEAK",
            "safe_to_answer": false,
            "reason": "No route/group peak was produced."
        });
    }
    let top = &peaks[0];
    let second = peaks.get(1);
    let top_name = top["peak"].as_str().unwrap_or("");
    let top_polarization = top["polarization"]["state"].as_str().unwrap_or("");
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let wins_lexical = !lexical_peak.is_empty() && top_name != lexical_peak;
    let (state, safe_to_answer, reason) = if top_polarization == "REVERSED" {
        (
            "POLARITY_REVERSED",
            false,
            "Top peak has reversed role-direction polarity relative to the query.",
        )
    } else if margin >= 0.055 && component_gap >= 0.12 {
        (
            "FOCUSED",
            true,
            "Top peak has enough margin and stronger connected component than the nearest rival.",
        )
    } else if margin < 0.04 {
        (
            "WATCH",
            false,
            "Top peak is close to the nearest rival; use as retrieval hint, not final structure.",
        )
    } else if component_gap < 0.0 {
        (
            "AMBIGUOUS",
            false,
            "Nearest rival has stronger connected component; inspect support and anti-triads.",
        )
    } else {
        (
            "WATCH",
            false,
            "Top peak is plausible but not focused enough for a confident structural answer.",
        )
    };
    json!({
        "state": state,
        "safe_to_answer": safe_to_answer,
        "top_peak": top_name,
        "lexical_baseline_top": lexical_peak,
        "wins_over_lexical_baseline": wins_lexical,
        "top_polarization": top_polarization,
        "margin": round4(margin),
        "top_component_score": round4(top_component),
        "second_component_score": round4(second_component),
        "component_gap": component_gap,
        "reason": reason
    })
}

pub(crate) fn symbolic_peak_baseline(items: &[(f64, f64, f64, &Triad)]) -> Value {
    let max_symbolic = items
        .iter()
        .map(|(_, _, symbolic, _)| *symbolic)
        .fold(0.0, f64::max);
    let avg_top3 = if items.is_empty() {
        0.0
    } else {
        let take = items.len().min(3);
        items
            .iter()
            .take(take)
            .map(|(_, _, symbolic, _)| *symbolic)
            .sum::<f64>()
            / take as f64
    };
    json!({
        "max_symbolic_overlap": round4(max_symbolic),
        "avg_top3_symbolic_overlap": round4(avg_top3)
    })
}

pub(crate) fn query_token_set(query: &[Triad]) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    for triad in query {
        tokens.extend(triad_tokens(triad));
    }
    tokens
}

pub(crate) fn propagation_summary(
    query_terms: &BTreeSet<String>,
    items: &[(f64, f64, f64, &Triad)],
) -> Value {
    if items.is_empty() {
        return json!({"score": 0.0, "component_score": 0.0, "links": [], "components": [], "activated_triads": []});
    }
    let mut activation: Vec<f64> = items
        .iter()
        .map(|(combined, _, _, _)| combined.max(0.0))
        .collect();
    let links = propagation_links(items);
    for _ in 0..2 {
        let mut next = activation.clone();
        for idx in 0..items.len() {
            let neighbors = links
                .iter()
                .filter_map(|(left, right)| {
                    if *left == idx {
                        Some(*right)
                    } else if *right == idx {
                        Some(*left)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if neighbors.is_empty() {
                continue;
            }
            let neighbor_energy = neighbors
                .iter()
                .map(|neighbor| activation[*neighbor])
                .sum::<f64>()
                / neighbors.len() as f64;
            next[idx] = (0.72 * activation[idx]) + (0.28 * neighbor_energy);
        }
        activation = next;
    }
    let mut ranked = activation
        .iter()
        .enumerate()
        .map(|(idx, value)| (idx, *value))
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let take = ranked.len().min(3);
    let score = if take == 0 {
        0.0
    } else {
        ranked
            .iter()
            .take(take)
            .map(|(_, value)| *value)
            .sum::<f64>()
            / take as f64
    };
    let components = connected_components(items.len(), &links);
    let mut component_rows = vec![];
    let mut component_score = 0.0_f64;
    for component in components {
        let mut terms = BTreeSet::new();
        let mut energy = 0.0;
        for idx in &component {
            terms.extend(triad_term_set(items[*idx].3));
            energy += activation[*idx];
        }
        let coverage = if query_terms.is_empty() {
            0.0
        } else {
            query_terms.intersection(&terms).count() as f64 / query_terms.len() as f64
        };
        let size_ratio = component.len() as f64 / items.len() as f64;
        let avg_energy = if component.is_empty() {
            0.0
        } else {
            energy / component.len() as f64
        };
        let score = (0.62 * coverage) + (0.23 * size_ratio) + (0.15 * avg_energy);
        component_score = component_score.max(score);
        component_rows.push(json!({
            "score": round4(score),
            "coverage": round4(coverage),
            "size": component.len(),
            "triads": component.iter().map(|idx| items[*idx].3.id.clone()).collect::<Vec<_>>()
        }));
    }
    component_rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    json!({
        "score": round4(score),
        "component_score": round4(component_score),
        "links": links.iter().map(|(left, right)| {
            json!({
                "from": items[*left].3.id,
                "to": items[*right].3.id,
                "via": shared_endpoint(items[*left].3, items[*right].3)
            })
        }).collect::<Vec<_>>(),
        "components": component_rows,
        "activated_triads": ranked.iter().take(5).map(|(idx, value)| {
            json!({
                "id": items[*idx].3.id,
                "activation": round4(*value),
                "base_score": round4(items[*idx].0)
            })
        }).collect::<Vec<_>>()
    })
}

pub(crate) fn propagation_links(items: &[(f64, f64, f64, &Triad)]) -> Vec<(usize, usize)> {
    let mut links = vec![];
    for left in 0..items.len() {
        for right in (left + 1)..items.len() {
            if !shared_endpoint(items[left].3, items[right].3).is_empty() {
                links.push((left, right));
            }
        }
    }
    links
}

pub(crate) fn triad_polarity(triad: &Triad) -> String {
    format!(
        "{}->{}->{}",
        role_family(&triad.subject_role),
        relation_family(&triad.relation),
        role_family(&triad.object_role)
    )
}

pub(crate) fn reversed_polarity(triad: &Triad) -> String {
    format!(
        "{}->{}->{}",
        role_family(&triad.object_role),
        relation_family(&triad.relation),
        role_family(&triad.subject_role)
    )
}

pub(crate) fn query_feature_wave(query: &[Triad]) -> Vec<i32> {
    let mut wave = vec![0; WAVE_DIM];
    for triad in query {
        add_into(&mut wave, &partial_triad_feature_wave(triad));
    }
    wave
}

pub(crate) fn symbolic_query_overlap(query: &[Triad], triad: &Triad) -> f64 {
    let q = query_term_set(query);
    if q.is_empty() {
        return 0.0;
    }
    let t = triad_term_set(triad);
    let hits = q.intersection(&t).count() as f64;
    hits / q.len() as f64
}

pub(crate) fn query_term_set(query: &[Triad]) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for triad in query {
        terms.extend(triad_term_set(triad));
    }
    terms
}

pub(crate) fn peak_center(items: &[(f64, f64, f64, &Triad)]) -> Value {
    json!({
        "entity": weighted_center(items.iter().flat_map(|(score, _, _, triad)| [(*score, norm(&triad.subject)), (*score, norm(&triad.object))])),
        "relation": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, norm(&triad.relation)))),
        "route": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, route_name(triad, "memory-route")))),
        "group": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, group_name(triad, "memory")))),
        "subject_role": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, norm(&triad.subject_role)))),
        "object_role": weighted_center(items.iter().map(|(score, _, _, triad)| (*score, norm(&triad.object_role))))
    })
}

pub(crate) fn print_search_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "mode: {}",
        out["mode"].as_str().unwrap_or("interference-retrieval")
    );
    println!("memory_size: {}", out["memory_size"].as_u64().unwrap_or(0));
    println!(
        "peak_state: {}",
        out["peak_decision"]["state"].as_str().unwrap_or("WATCH")
    );
    println!(
        "safe_to_answer: {}",
        out["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "peak_margin: {}",
        out["peak_margin"].as_f64().unwrap_or(0.0)
    );
    println!(
        "lexical_baseline_top: {}",
        out["lexical_baseline"]["top_peak"].as_str().unwrap_or("")
    );
    if let Some(peaks) = out["peaks"].as_array() {
        for (idx, peak) in peaks.iter().enumerate() {
            println!(
                "{}. peak={} score={} route={} group={}",
                idx + 1,
                peak["peak"].as_str().unwrap_or(""),
                peak["score"].as_f64().unwrap_or(0.0),
                peak["center"]["route"].as_str().unwrap_or(""),
                peak["center"]["group"].as_str().unwrap_or("")
            );
            if let Some(support) = peak["supporting_triads"].as_array() {
                for item in support.iter().take(3) {
                    println!(
                        "   + {}: {} -> {} -> {}",
                        item["id"].as_str().unwrap_or(""),
                        item["subject"].as_str().unwrap_or(""),
                        item["relation"].as_str().unwrap_or(""),
                        item["object"].as_str().unwrap_or("")
                    );
                }
            }
            if let Some(anti) = peak["anti_triads"].as_array() {
                for item in anti.iter().take(2) {
                    println!(
                        "   ~ foreign {}: {} -> {} -> {}",
                        item["id"].as_str().unwrap_or(""),
                        item["subject"].as_str().unwrap_or(""),
                        item["relation"].as_str().unwrap_or(""),
                        item["object"].as_str().unwrap_or("")
                    );
                }
            }
        }
    }
}

pub(crate) fn print_search_md(out: &Value) {
    println!("# NANDA Interference Search\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- memory_size: `{}`", out["memory_size"]);
    println!("- peak_state: `{}`", out["peak_decision"]["state"]);
    println!(
        "- safe_to_answer: `{}`",
        out["peak_decision"]["safe_to_answer"]
    );
    println!("- peak_margin: `{}`", out["peak_margin"]);
    println!(
        "- lexical_baseline_top: `{}`",
        out["lexical_baseline"]["top_peak"].as_str().unwrap_or("")
    );
    if let Some(peaks) = out["peaks"].as_array() {
        for (idx, peak) in peaks.iter().enumerate() {
            println!(
                "\n## Peak {}: `{}`\n",
                idx + 1,
                peak["peak"].as_str().unwrap_or("")
            );
            println!("- score: `{}`", peak["score"]);
            println!("- route: `{}`", peak["center"]["route"]);
            println!("- group: `{}`", peak["center"]["group"]);
            println!("- support:");
            if let Some(support) = peak["supporting_triads"].as_array() {
                for item in support.iter().take(5) {
                    println!(
                        "  - `{}`: {} -> {} -> {}",
                        item["id"].as_str().unwrap_or(""),
                        item["subject"].as_str().unwrap_or(""),
                        item["relation"].as_str().unwrap_or(""),
                        item["object"].as_str().unwrap_or("")
                    );
                }
            }
        }
    }
}
