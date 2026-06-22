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
    result["unified_field"] = field_core::adapters::adapt_value(&result).to_value();
    result["field_runtime"] = field_core::structural_dual_run_value(&result);
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
        layer: "adapter".to_string(),
        owner: "auto-query".to_string(),
        entrypoint: "nanda-search --query".to_string(),
        output: "query-triads".to_string(),
        evidence_path: String::new(),
        scope: "query".to_string(),
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
            layer: "adapter".to_string(),
            owner: "auto-query".to_string(),
            entrypoint: "nanda-search --query".to_string(),
            output: "query-triads".to_string(),
            evidence_path: String::new(),
            scope: "query".to_string(),
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
            layer: "adapter".to_string(),
            owner: "auto-query".to_string(),
            entrypoint: "nanda-search --query".to_string(),
            output: "query-triads".to_string(),
            evidence_path: String::new(),
            scope: "query".to_string(),
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
                    "subject_role": triad.subject_role,
                    "object_role": triad.object_role,
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
                    "subject_role": triad.subject_role,
                    "object_role": triad.object_role,
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
    let resonance_memory = apply_resonance_memory(&mut peaks, &packet.resonance_memory);
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
    let resonant_field = resonant_field_report(
        &peaks,
        peak_margin,
        &lexical_baseline,
        &corpus_interpretation,
        &route_balanced_focus,
        &coarse_to_fine,
        &destructive_interference,
        &constructive_interference,
        packet,
    );
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
        "resonance_memory": resonance_memory,
        "source_weighting": {
            "enabled": true,
            "policy": "confidence multiplied by evidence authority: current/canon > latest/frontier > historical/archive > archive_noise"
        },
        "coarse_to_fine": coarse_to_fine,
        "resonant_field": resonant_field,
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn resonant_field_report(
    peaks: &[Value],
    margin: f64,
    lexical_baseline: &Value,
    corpus: &Value,
    route_balanced_focus: &Value,
    coarse_to_fine: &Value,
    destructive_interference: &Value,
    constructive_interference: &Value,
    packet: &Packet,
) -> Value {
    if peaks.is_empty() {
        return json!({
            "version": "v28-resonant-field-core",
            "state": "NO_RESONANCE",
            "waw_status": "NO_PEAK",
            "safe_to_answer": false,
            "read_as": "No field was produced."
        });
    }
    let top = &peaks[0];
    let second = peaks.get(1);
    let top_peak = top["peak"].as_str().unwrap_or("");
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let top_score = top["score"].as_f64().unwrap_or(0.0);
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let support = top["supporting_triads"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let anti = top["anti_triads"].as_array().cloned().unwrap_or_default();
    let support_energy = round4(
        support
            .iter()
            .map(|item| item["score"].as_f64().unwrap_or(0.0).max(0.0))
            .sum(),
    );
    let anti_energy = round4(
        anti.iter()
            .map(|item| item["score"].as_f64().unwrap_or(0.0).max(0.0))
            .sum(),
    );
    let total_peak_energy = peaks
        .iter()
        .map(|peak| peak["score"].as_f64().unwrap_or(0.0).max(0.0))
        .sum::<f64>();
    let leakage_energy = round4((total_peak_energy - top_score.max(0.0)).max(0.0));
    let energy_total = round4(support_energy + anti_energy + leakage_energy);
    let leakage_ratio = if energy_total > 0.0 {
        round4(leakage_energy / energy_total)
    } else {
        0.0
    };
    let anti_ratio = if support_energy + anti_energy > 0.0 {
        round4(anti_energy / (support_energy + anti_energy))
    } else {
        0.0
    };
    let polarity = top["polarization"]["state"].as_str().unwrap_or("");
    let phase_score = match polarity {
        "ALIGNED" => round4(
            (0.45
                + top["coherence"].as_f64().unwrap_or(0.0)
                + (0.2 * top["coverage"].as_f64().unwrap_or(0.0)))
            .min(1.0),
        ),
        "UNALIGNED" => round4((0.22 + (0.5 * top["coherence"].as_f64().unwrap_or(0.0))).min(0.72)),
        "REVERSED" => 0.0,
        _ => round4((0.12 + (0.4 * top["coherence"].as_f64().unwrap_or(0.0))).min(0.55)),
    };
    let reflected_margin =
        round4((margin + (0.18 * component_gap.max(0.0)) - (0.06 * anti_ratio)).max(0.0));
    let standing_iterations =
        standing_wave_iterations(top_peak, top_score, reflected_margin, anti_ratio, polarity);
    let standing_stable = standing_iterations
        .last()
        .map(|item| {
            item["peak"].as_str().unwrap_or("") == top_peak
                && item["state"].as_str().unwrap_or("") != "COLLAPSED"
        })
        .unwrap_or(false)
        && reflected_margin >= 0.055;
    let reflection = round4((anti_ratio * 0.62 + leakage_ratio * 0.38).min(1.0));
    let boundary_state = if polarity == "REVERSED" {
        "BOUNDARY_REVERSED"
    } else if leakage_ratio <= 0.34 && anti_ratio <= 0.48 {
        "BOUNDARY_CONTAINED"
    } else if leakage_ratio <= 0.52 {
        "BOUNDARY_LEAKING"
    } else {
        "BOUNDARY_DIFFUSE"
    };
    let destructive_state = if destructive_interference["applied"]
        .as_bool()
        .unwrap_or(false)
    {
        "SHORTCUT_SUPPRESSED"
    } else if anti_ratio <= 0.35 {
        "ANTI_LOW"
    } else {
        "ANTI_READY"
    };
    let multiscale_score = round4(
        (top["top_triad_score"].as_f64().unwrap_or(0.0).min(1.0)
            + top["coherence"].as_f64().unwrap_or(0.0).max(0.0)
            + top["coverage"].as_f64().unwrap_or(0.0).max(0.0)
            + top_component.clamp(0.0, 1.0))
            / 4.0,
    );
    let scan = resonance_scan(top, peaks, lexical_peak, constructive_interference);
    let temporal_phase = temporal_phase_report(&support, packet);
    let memory = coherence_memory_report(packet, top_peak);
    let lexical_trap = !lexical_peak.is_empty() && lexical_peak != top_peak;
    let corpus_clean = corpus["verdict"].as_str().unwrap_or("") != "WATCH";
    let focused_or_balanced = route_balanced_focus["enabled"].as_bool().unwrap_or(false)
        || coarse_to_fine["state"].as_str().unwrap_or("") == "LOCALIZED";
    let waw_ready = polarity != "REVERSED"
        && phase_score >= 0.75
        && standing_stable
        && reflected_margin >= 0.055
        && component_gap >= 0.12
        && leakage_ratio <= 0.5
        && anti_ratio <= 0.62
        && multiscale_score >= 0.36
        && focused_or_balanced
        && corpus_clean;
    let state = if waw_ready {
        "WAW_RESONANCE"
    } else if polarity == "REVERSED" {
        "RESONANCE_REVERSED"
    } else if leakage_ratio > 0.58 {
        "FIELD_LEAKING"
    } else if anti_ratio > 0.65 {
        "FIELD_ANTI_DOMINATED"
    } else if reflected_margin < 0.04 {
        "FIELD_DIFFUSE"
    } else {
        "RESONANCE_REVIEW"
    };
    json!({
        "version": "v28-resonant-field-core",
        "state": state,
        "waw_status": if waw_ready { "WAW_RESONANCE" } else { "NO_WAW_RESONANCE" },
        "safe_to_answer": waw_ready,
        "top_peak": top_peak,
        "lexical_trap_detected": lexical_trap,
        "phase_lock": {
            "state": if phase_score >= 0.75 { "PHASE_LOCKED" } else if phase_score > 0.0 { "PHASE_PARTIAL" } else { "PHASE_REVERSED" },
            "score": phase_score,
            "polarity": polarity,
            "read_as": "Phase lock combines role polarity, wave coherence, and query coverage."
        },
        "standing_wave": {
            "state": if standing_stable { "STANDING_STABLE" } else { "STANDING_UNSTABLE" },
            "reflected_margin": reflected_margin,
            "iterations": standing_iterations,
            "read_as": "The peak is reflected through the field several times; stable peaks keep identity and do not collapse."
        },
        "route_boundary": {
            "state": boundary_state,
            "reflection": reflection,
            "leakage_ratio": leakage_ratio,
            "anti_ratio": anti_ratio,
            "read_as": "Routes/groups act as membranes: foreign pulls reflect, leak, or diffuse the field."
        },
        "destructive_locality": {
            "state": destructive_state,
            "applied": destructive_interference["applied"],
            "suppression_count": destructive_interference["suppressions"].as_array().map(|items| items.len()).unwrap_or(0),
            "read_as": "Anti-waves should suppress a rejected shortcut path, not erase the whole topic."
        },
        "multiscale": {
            "state": if multiscale_score >= 0.5 { "MULTISCALE_ALIGNED" } else if multiscale_score >= 0.36 { "MULTISCALE_PARTIAL" } else { "MULTISCALE_THIN" },
            "score": multiscale_score,
            "triad": top["top_triad_score"],
            "route": top["coherence"],
            "coverage": top["coverage"],
            "component": top_component,
            "read_as": "A strong field agrees across triad, route, coverage, and propagation scales."
        },
        "energy": {
            "state": if leakage_ratio <= 0.35 { "ENERGY_CONTAINED" } else if leakage_ratio <= 0.55 { "ENERGY_LEAKING" } else { "ENERGY_DIFFUSE" },
            "support": support_energy,
            "anti": anti_energy,
            "leakage": leakage_energy,
            "total": energy_total,
            "leakage_ratio": leakage_ratio,
            "anti_ratio": anti_ratio,
            "read_as": "Query energy is accounted as support, anti-support, and leakage into rival peaks."
        },
        "resonance_scan": scan,
        "temporal_phase": temporal_phase,
        "coherence_memory": memory,
        "waw_threshold": {
            "requires": [
                "phase_score >= 0.75",
                "standing_wave stable",
                "component_gap >= 0.12",
                "reflected_margin >= 0.055",
                "leakage_ratio <= 0.50",
                "multiscale_score >= 0.36",
                "corpus not WATCH"
            ],
            "passed": waw_ready
        },
        "read_as": if waw_ready {
            "The peak survived phase, reflection, energy, multiscale, and corpus checks. This is a WAW resonance, not just a high score."
        } else {
            "The field exposes useful physics, but it did not satisfy the full WAW resonance threshold."
        }
    })
}

fn standing_wave_iterations(
    top_peak: &str,
    top_score: f64,
    reflected_margin: f64,
    anti_ratio: f64,
    polarity: &str,
) -> Vec<Value> {
    let mut wave = top_score.max(0.0);
    let mut out = vec![];
    for idx in 0..4 {
        let damping = 0.04 * idx as f64;
        let gain = reflected_margin * (1.0 - damping);
        let loss = anti_ratio * 0.035;
        wave = round4((wave + gain - loss).max(0.0));
        let state = if polarity == "REVERSED" {
            "COLLAPSED"
        } else if wave >= top_score.max(0.0) * 0.92 {
            "PEAK_HELD"
        } else {
            "WEAKENED"
        };
        out.push(json!({
            "iteration": idx + 1,
            "peak": if state == "COLLAPSED" { "" } else { top_peak },
            "relative_energy": wave,
            "state": state
        }));
    }
    out
}

fn resonance_scan(
    top: &Value,
    peaks: &[Value],
    lexical_peak: &str,
    constructive_interference: &Value,
) -> Value {
    let center = &top["center"];
    let route_peak = top["peak"].as_str().unwrap_or("");
    let relation_peak = center["relation"].as_str().unwrap_or("");
    let role_peak = format!(
        "{}->{}",
        center["subject_role"].as_str().unwrap_or(""),
        center["object_role"].as_str().unwrap_or("")
    );
    let nearest_rival = peaks
        .get(1)
        .and_then(|peak| peak["peak"].as_str())
        .unwrap_or("");
    json!({
        "modes": [
            {
                "mode": "route",
                "peak": route_peak,
                "state": "ACTIVE"
            },
            {
                "mode": "relation",
                "peak": relation_peak,
                "state": if relation_peak.is_empty() { "NO_MODE" } else { "ACTIVE" }
            },
            {
                "mode": "role",
                "peak": role_peak,
                "state": "ACTIVE"
            },
            {
                "mode": "lexical",
                "peak": lexical_peak,
                "state": if !lexical_peak.is_empty() && lexical_peak != route_peak { "LEXICAL_TRAP" } else { "ALIGNED" }
            },
            {
                "mode": "constructive_memory",
                "peak": constructive_interference["reinforcements"].as_array().and_then(|items| items.first()).and_then(|item| item["reinforce_peak"].as_str()).unwrap_or(""),
                "state": if constructive_interference["applied"].as_bool().unwrap_or(false) { "REINFORCED" } else { "NO_REINFORCEMENT" }
            }
        ],
        "nearest_rival": nearest_rival,
        "read_as": "Different field modes expose whether the same peak wins by route, relation, role, lexical, and learned-memory frequencies."
    })
}

fn temporal_phase_report(support: &[Value], packet: &Packet) -> Value {
    let text = support
        .iter()
        .map(|item| {
            format!(
                "{} {} {} {}",
                item["subject"].as_str().unwrap_or(""),
                item["relation"].as_str().unwrap_or(""),
                item["object"].as_str().unwrap_or(""),
                item["evidence"].as_str().unwrap_or("")
            )
        })
        .chain(std::iter::once(packet.query.clone()))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let past = ["past", "previous", "archive", "was", "2024", "2025"]
        .iter()
        .filter(|term| text.contains(**term))
        .count();
    let current = ["current", "today", "now", "live", "2026-06-18", "snapshot"]
        .iter()
        .filter(|term| text.contains(**term))
        .count();
    let future = [
        "future",
        "forecast",
        "expect",
        "scheduled",
        "tomorrow",
        "2026-06-19",
    ]
    .iter()
    .filter(|term| text.contains(**term))
    .count();
    let total_temporal_hits = past + current + future;
    let active = if total_temporal_hits == 0 {
        "none"
    } else {
        [("past", past), ("current", current), ("future", future)]
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(label, _)| label)
            .unwrap_or("none")
    };
    let contested = [past, current, future]
        .iter()
        .filter(|count| **count > 0)
        .count()
        > 1;
    json!({
        "state": if contested { "TEMPORAL_CONTESTED" } else if active == "none" { "TEMPORAL_UNSPECIFIED" } else { "TEMPORAL_PHASE_LOCKED" },
        "dominant_phase": active,
        "counts": {
            "past": past,
            "current": current,
            "future": future
        },
        "read_as": "Past facts, current facts, and future forecasts are treated as different phase bands."
    })
}

fn coherence_memory_report(packet: &Packet, top_peak: &str) -> Value {
    let positive_hits = packet
        .positive_shortcuts
        .iter()
        .filter(|shortcut| {
            shortcut.reinforce_peak == top_peak
                || shortcut.reinforce_route == top_peak
                || shortcut.reinforce_group == top_peak
        })
        .count();
    let negative_hits = packet
        .negative_shortcuts
        .iter()
        .filter(|shortcut| {
            shortcut.suppress_peak == top_peak
                || shortcut.suppress_route == top_peak
                || shortcut.suppress_group == top_peak
        })
        .count();
    let resonance_positive_hits = packet
        .resonance_memory
        .iter()
        .filter(|memory| {
            memory.decision == "accept" && (memory.peak == top_peak || memory.route == top_peak)
        })
        .count();
    let resonance_negative_hits = packet
        .resonance_memory
        .iter()
        .filter(|memory| {
            memory.decision == "reject" && (memory.peak == top_peak || memory.route == top_peak)
        })
        .count();
    let total_positive = positive_hits + resonance_positive_hits;
    let total_negative = negative_hits + resonance_negative_hits;
    json!({
        "state": if total_positive > total_negative { "MEMORY_REINFORCED" } else if total_negative > 0 { "MEMORY_SUPPRESSED" } else { "MEMORY_NEUTRAL" },
        "positive_hits": positive_hits,
        "negative_hits": negative_hits,
        "resonance_positive_hits": resonance_positive_hits,
        "resonance_negative_hits": resonance_negative_hits,
        "read_as": "Coherence memory stores accepted/rejected field shapes as lane feedback, not as final answers."
    })
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
        "resonance: {} / {}",
        out["resonant_field"]["state"].as_str().unwrap_or(""),
        out["resonant_field"]["waw_status"].as_str().unwrap_or("")
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
        "- resonance: `{}` / `{}`",
        out["resonant_field"]["state"].as_str().unwrap_or(""),
        out["resonant_field"]["waw_status"].as_str().unwrap_or("")
    );
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
