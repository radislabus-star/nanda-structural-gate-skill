use crate::*;

pub(crate) fn probe_cmd(args: ProbeArgs) -> Result<u8> {
    if let Some(suite_path) = &args.suite {
        return probe_suite_cmd(&args, suite_path);
    }
    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow!("nanda probe requires an input path or --suite"))?;
    let out = run_probe_once(
        input,
        &args.input_format,
        &args.negative_inputs,
        &args.task_id,
        &args.domain,
        &args.query,
        args.query_file.as_ref(),
        &args.query_format,
        args.top_k,
        args.route_cap,
        args.route_triad_cap,
        &args.group_by,
        args.normalize_paths,
    )?;
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_probe_text(&out),
        OutputFormat::Md => print_probe_md(&out),
    }
    Ok(EXIT_PASS)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_probe_once(
    input: &Path,
    input_format: &InputFormat,
    negative_inputs: &[PathBuf],
    task_id: &str,
    domain: &str,
    query_arg: &str,
    query_file: Option<&PathBuf>,
    query_format: &InputFormat,
    top_k: usize,
    route_cap: usize,
    route_triad_cap: usize,
    group_by: &PeakGroupBy,
    normalize_paths: bool,
) -> Result<Value> {
    let mut packet = load_packet_auto(
        input,
        input_format,
        task_id,
        domain,
        query_arg,
        normalize_paths,
    )?;
    let mut negative_shortcuts = if negative_inputs.is_empty() {
        packet.negative_shortcuts.clone()
    } else {
        vec![]
    };
    for negative_input in negative_inputs {
        if let Some(shortcuts) = load_feedback_negative_shortcuts(negative_input)? {
            negative_shortcuts.extend(shortcuts);
            continue;
        }
        let negative_packet = load_packet_auto(
            negative_input,
            input_format,
            task_id,
            domain,
            query_arg,
            normalize_paths,
        )?;
        negative_shortcuts.extend(negative_packet.negative_shortcuts);
    }
    negative_shortcuts = merge_negative_shortcuts(negative_shortcuts);

    let mut query_packet = if let Some(query_file) = query_file {
        load_packet_auto(
            query_file,
            query_format,
            task_id,
            domain,
            query_arg,
            normalize_paths,
        )?
    } else {
        packet.clone()
    };
    inherit_aliases_if_needed(&mut query_packet, &packet);
    let query_text = if !query_arg.trim().is_empty() {
        query_arg.to_string()
    } else if !query_packet.query.trim().is_empty() {
        query_packet.query.clone()
    } else {
        packet.query.clone()
    };
    packet.query = query_text.clone();
    let (query, query_source) = search_query_triads(&query_packet, &query_text);
    let memory = normalize_ids(packet.triads.clone(), "m");
    let focus = route_balanced_focus(&memory, &query, route_cap, route_triad_cap);

    let mut plain_packet = packet.clone();
    plain_packet.negative_shortcuts.clear();
    let plain = interference_search(
        &plain_packet,
        &focus.memory,
        &query,
        top_k,
        group_by,
        query_source,
        focus.metadata.clone(),
    );

    let mut negative_packet = packet;
    negative_packet.negative_shortcuts = negative_shortcuts.clone();
    let negative = interference_search(
        &negative_packet,
        &focus.memory,
        &query,
        top_k,
        group_by,
        query_source,
        focus.metadata,
    );

    Ok(probe_report(&plain, &negative, negative_shortcuts.len()))
}

pub(crate) fn probe_suite_cmd(args: &ProbeArgs, suite_path: &Path) -> Result<u8> {
    let text =
        fs::read_to_string(suite_path).with_context(|| format!("read {}", suite_path.display()))?;
    let suite: ProbeSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", suite_path.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow!("nanda probe --suite requires at least one case"));
    }
    let base = suite_path.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    for case in &suite.cases {
        let path = resolve_suite_path(base, &case.path);
        let negative_inputs = case
            .negative
            .iter()
            .map(|path| resolve_suite_path(base, path))
            .collect::<Vec<_>>();
        let group_by = match case.group_by.as_str() {
            "" => args.group_by.clone(),
            "group" => PeakGroupBy::Group,
            "route" => PeakGroupBy::Route,
            other => return Err(anyhow!("unsupported probe suite group_by: {other}")),
        };
        let result = run_probe_once(
            &path,
            &args.input_format,
            &negative_inputs,
            &args.task_id,
            &args.domain,
            &args.query,
            args.query_file.as_ref(),
            &args.query_format,
            args.top_k,
            args.route_cap,
            args.route_triad_cap,
            &group_by,
            args.normalize_paths,
        )?;
        let decision_ok = case.expected_decision.is_empty()
            || result["decision"].as_str() == Some(&case.expected_decision);
        let plain_ok = case.expected_plain_peak.is_empty()
            || result["plain"]["top_peak"].as_str() == Some(&case.expected_plain_peak);
        let negative_ok = case.expected_negative_peak.is_empty()
            || result["negative"]["top_peak"].as_str() == Some(&case.expected_negative_peak);
        let ok = decision_ok && plain_ok && negative_ok;
        if ok {
            passed += 1;
        }
        rows.push(json!({
            "id": if case.id.is_empty() { path.display().to_string() } else { case.id.clone() },
            "path": path.display().to_string(),
            "expected_decision": case.expected_decision,
            "actual_decision": result["decision"],
            "expected_plain_peak": case.expected_plain_peak,
            "actual_plain_peak": result["plain"]["top_peak"],
            "expected_negative_peak": case.expected_negative_peak,
            "actual_negative_peak": result["negative"]["top_peak"],
            "ok": ok,
            "delta": result["delta"],
            "recommended_action": result["recommended_action"]
        }));
    }
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "probe-suite",
        "name": suite.name,
        "passed": passed,
        "total": rows.len(),
        "accuracy": round4(passed as f64 / rows.len().max(1) as f64),
        "cases": rows
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_probe_suite_text(&out),
        OutputFormat::Md => print_probe_suite_md(&out),
    }
    if passed == out["total"].as_u64().unwrap_or(0) as usize {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

pub(crate) fn probe_report(plain: &Value, negative: &Value, negative_lanes: usize) -> Value {
    let plain_top = plain["top_peak"].as_str().unwrap_or("");
    let negative_top = negative["top_peak"].as_str().unwrap_or("");
    let plain_score = top_peak_score(plain);
    let negative_score = top_peak_score(negative);
    let plain_safe = plain["safe_to_answer"].as_bool().unwrap_or(false);
    let negative_safe = negative["safe_to_answer"].as_bool().unwrap_or(false);
    let suppression_count = negative["destructive_interference"]["suppressions"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let top_changed = plain_top != negative_top;
    let became_safer = !plain_safe && negative_safe;
    let used_negative_lane = suppression_count > 0;
    let (decision, recommended_action) =
        if used_negative_lane && (became_safer || (top_changed && negative_safe)) {
            ("IMPROVED", "USE_NEGATIVE_RESULT")
        } else if used_negative_lane && top_changed {
            (
                "SHIFTED_TO_REVIEW",
                "INSPECT_NEGATIVE_SUPPORT_BEFORE_ANSWER",
            )
        } else if used_negative_lane {
            ("SUPPRESSED_WITHOUT_TOP_CHANGE", "COMPARE_SUPPORT_AND_SCORE")
        } else if !used_negative_lane && top_changed {
            ("CHANGED_WITHOUT_SUPPRESSION", "CHECK_INPUTS_OR_ROUTE_FOCUS")
        } else if plain_safe && !negative_safe {
            ("REGRESSED", "DO_NOT_USE_NEGATIVE_RESULT")
        } else {
            ("UNCHANGED", "NO_PROVEN_NEGATIVE_LANE_BENEFIT")
        };
    let legacy_improved = if used_negative_lane && (top_changed || became_safer) {
        "IMPROVED"
    } else {
        "UNCHANGED"
    };
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "probe-report",
        "decision": decision,
        "legacy_decision": legacy_improved,
        "recommended_action": recommended_action,
        "negative_lanes": negative_lanes,
        "plain": probe_search_summary(plain),
        "negative": probe_search_summary(negative),
        "delta": {
            "top_changed": top_changed,
            "verdict_changed": plain["verdict"] != negative["verdict"],
            "field_state_changed": plain["field_state"] != negative["field_state"],
            "safe_to_answer_changed": plain["safe_to_answer"] != negative["safe_to_answer"],
            "score_delta": round4(negative_score - plain_score),
            "suppression_count": suppression_count,
            "suppressed_peaks": negative["destructive_interference"]["suppressions"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item["suppressed_peak"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        },
        "destructive_interference": negative["destructive_interference"].clone(),
        "read_as": "Probe compares the same search before and after negative lanes. Treat IMPROVED as evidence that destructive interference repaired a shortcut; treat UNCHANGED as no proof of benefit."
    })
}

pub(crate) fn load_feedback_negative_shortcuts(
    path: &Path,
) -> Result<Option<Vec<NegativeShortcut>>> {
    Ok(load_feedback_lanes(path)?.map(|(negative, _)| negative))
}

pub(crate) fn load_feedback_lanes(
    path: &Path,
) -> Result<Option<(Vec<NegativeShortcut>, Vec<PositiveShortcut>)>> {
    if !path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        return Ok(None);
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if value["mode"].as_str() != Some("feedback-memory") {
        return Ok(None);
    }
    let negative = value["negative_shortcuts"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<Vec<NegativeShortcut>, _>>()?;
    let positive = value["positive_shortcuts"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<Vec<PositiveShortcut>, _>>()?;
    Ok(Some((negative, positive)))
}

#[allow(clippy::type_complexity)]
pub(crate) fn merge_negative_shortcuts(shortcuts: Vec<NegativeShortcut>) -> Vec<NegativeShortcut> {
    let mut merged: BTreeMap<
        (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ),
        NegativeShortcut,
    > = BTreeMap::new();
    for mut shortcut in shortcuts {
        if shortcut.observations == 0 {
            shortcut.observations = 1;
        }
        if shortcut.rejected_count == 0 && shortcut.accepted_count == 0 {
            shortcut.rejected_count = shortcut.observations;
        }
        shortcut.terms = normalized_shortcut_terms(&shortcut.terms)
            .into_iter()
            .collect::<Vec<_>>();
        shortcut.support_terms = normalized_shortcut_terms(&shortcut.support_terms)
            .into_iter()
            .collect::<Vec<_>>();
        let key = (
            norm(&shortcut.suppress_peak),
            norm(&shortcut.suppress_route),
            norm(&shortcut.suppress_group),
            norm(&shortcut.prefer_peak),
            norm(&shortcut.prefer_route),
            norm(&shortcut.prefer_group),
            shortcut.terms.join("|"),
            shortcut.support_terms.join("|"),
        );
        merged
            .entry(key)
            .and_modify(|existing| {
                existing.observations += shortcut.observations;
                existing.rejected_count += shortcut.rejected_count;
                existing.accepted_count += shortcut.accepted_count;
                existing.penalty = existing.penalty.max(shortcut.penalty);
                if existing.reason.is_empty() {
                    existing.reason = shortcut.reason.clone();
                }
                if existing.suppress_route.is_empty() {
                    existing.suppress_route = shortcut.suppress_route.clone();
                }
                if existing.suppress_group.is_empty() {
                    existing.suppress_group = shortcut.suppress_group.clone();
                }
                if existing.prefer_route.is_empty() {
                    existing.prefer_route = shortcut.prefer_route.clone();
                }
                if existing.prefer_group.is_empty() {
                    existing.prefer_group = shortcut.prefer_group.clone();
                }
                if !shortcut.source_feedback.is_empty() {
                    if existing.source_feedback.is_empty() {
                        existing.source_feedback = shortcut.source_feedback.clone();
                    } else if !existing
                        .source_feedback
                        .split(';')
                        .any(|item| item == shortcut.source_feedback)
                    {
                        existing.source_feedback.push(';');
                        existing.source_feedback.push_str(&shortcut.source_feedback);
                    }
                }
            })
            .or_insert(shortcut);
    }
    merged.into_values().collect()
}

pub(crate) fn normalized_shortcut_terms(terms: &[String]) -> BTreeSet<String> {
    terms
        .iter()
        .flat_map(|term| {
            norm(term)
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|token| token.len() >= 2)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(crate) fn merge_positive_shortcuts(shortcuts: Vec<PositiveShortcut>) -> Vec<PositiveShortcut> {
    let mut merged: BTreeMap<(String, String, String, String, String), PositiveShortcut> =
        BTreeMap::new();
    for mut shortcut in shortcuts {
        if shortcut.observations == 0 {
            shortcut.observations = 1;
        }
        if shortcut.accepted_count == 0 && shortcut.rejected_count == 0 {
            shortcut.accepted_count = shortcut.observations;
        }
        shortcut.terms = normalized_shortcut_terms(&shortcut.terms)
            .into_iter()
            .collect::<Vec<_>>();
        shortcut.support_terms = normalized_shortcut_terms(&shortcut.support_terms)
            .into_iter()
            .collect::<Vec<_>>();
        let key = (
            norm(&shortcut.reinforce_peak),
            norm(&shortcut.reinforce_route),
            norm(&shortcut.reinforce_group),
            shortcut.terms.join("|"),
            shortcut.support_terms.join("|"),
        );
        merged
            .entry(key)
            .and_modify(|existing| {
                existing.observations += shortcut.observations;
                existing.accepted_count += shortcut.accepted_count;
                existing.rejected_count += shortcut.rejected_count;
                existing.boost = existing.boost.max(shortcut.boost);
                if existing.reason.is_empty() {
                    existing.reason = shortcut.reason.clone();
                }
                if existing.reinforce_route.is_empty() {
                    existing.reinforce_route = shortcut.reinforce_route.clone();
                }
                if existing.reinforce_group.is_empty() {
                    existing.reinforce_group = shortcut.reinforce_group.clone();
                }
                if !shortcut.source_feedback.is_empty() {
                    if existing.source_feedback.is_empty() {
                        existing.source_feedback = shortcut.source_feedback.clone();
                    } else if !existing
                        .source_feedback
                        .split(';')
                        .any(|item| item == shortcut.source_feedback)
                    {
                        existing.source_feedback.push(';');
                        existing.source_feedback.push_str(&shortcut.source_feedback);
                    }
                }
            })
            .or_insert(shortcut);
    }
    merged.into_values().collect()
}

pub(crate) fn feedback_cmd(args: FeedbackArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.input)
        .with_context(|| format!("read {}", args.input.display()))?;
    let search: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", args.input.display()))?;
    let top = search["peaks"].as_array().and_then(|peaks| peaks.first());
    let peak_name = top
        .and_then(|peak| peak["peak"].as_str())
        .unwrap_or("")
        .to_string();
    let support_ids = top
        .and_then(|peak| peak["supporting_triads"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["id"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let anti_ids = top
        .and_then(|peak| peak["anti_triads"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["id"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let negative_shortcuts = if matches!(args.decision, FeedbackDecision::Reject) {
        vec![negative_shortcut_from_search(
            &search,
            &peak_name,
            &args.note,
            args.input.display().to_string(),
        )]
    } else {
        vec![]
    };
    let positive_shortcuts = if matches!(args.decision, FeedbackDecision::Accept) {
        vec![positive_shortcut_from_search(
            &search,
            &peak_name,
            &args.note,
            args.input.display().to_string(),
        )]
    } else {
        vec![]
    };
    let decision = feedback_decision_label(&args.decision);
    let reinforcement = match args.decision {
        FeedbackDecision::Accept => json!({
            "reinforce_peak": peak_name,
            "reinforce_support": support_ids,
            "suppress_foreign": anti_ids,
            "positive_shortcuts": positive_shortcuts
        }),
        FeedbackDecision::Reject => json!({
            "reject_peak": peak_name,
            "suppress_support": support_ids,
            "inspect_alternatives": anti_ids,
            "negative_shortcuts": negative_shortcuts
        }),
        FeedbackDecision::Watch => json!({
            "watch_peak": peak_name,
            "needs_evidence": support_ids,
            "possible_foreign_pull": anti_ids
        }),
    };
    let feedback = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "feedback-memory",
        "source_search": args.input.display().to_string(),
        "decision": decision,
        "note": args.note,
        "peak": peak_name,
        "peak_score": top.and_then(|peak| peak["score"].as_f64()).unwrap_or(0.0),
        "peak_margin": search["peak_margin"].as_f64().unwrap_or(0.0),
        "peak_decision": search["peak_decision"].clone(),
        "lexical_baseline_top": search["lexical_baseline"]["top_peak"].as_str().unwrap_or(""),
        "wins_over_lexical_baseline": search["wins_over_lexical_baseline"].as_bool().unwrap_or(false),
        "support_ids": support_ids,
        "anti_ids": anti_ids,
        "negative_shortcuts": negative_shortcuts,
        "positive_shortcuts": positive_shortcuts,
        "memory_patch": reinforcement,
        "interpretation": "Feedback is a compact trace for later memory tuning. Reject feedback creates a negative shortcut; accept feedback creates a positive shortcut that can boost the same supported route in later search."
    });
    let output = serde_json::to_string_pretty(&feedback)? + "\n";
    write_or_print(args.out, args.stdout, output)?;
    Ok(EXIT_PASS)
}

pub(crate) fn feedback_decision_label(decision: &FeedbackDecision) -> &'static str {
    match decision {
        FeedbackDecision::Accept => "accept",
        FeedbackDecision::Reject => "reject",
        FeedbackDecision::Watch => "watch",
    }
}

pub(crate) fn apply_positive_lanes(
    peaks: &mut [Value],
    query: &[Triad],
    shortcuts: &[PositiveShortcut],
) -> Value {
    let query_tokens = query_token_set(query);
    let mut reinforcements = vec![];
    if shortcuts.is_empty() || peaks.is_empty() {
        return json!({
            "applied": false,
            "positive_lanes": shortcuts.len(),
            "reinforcements": reinforcements
        });
    }
    for shortcut in shortcuts {
        let query_ratio = positive_lane_match_ratio(&query_tokens, shortcut);
        if query_ratio <= 0.0 {
            continue;
        }
        let accepted_count = shortcut_accepted_count(shortcut);
        let learned_boost =
            (shortcut.boost.max(0.0) + (accepted_count.saturating_sub(1) as f64 * 0.025)).min(0.25);
        for peak in peaks.iter_mut() {
            let Some(peak_name) = peak["peak"].as_str().map(str::to_string) else {
                continue;
            };
            if !positive_lane_matches_reinforce(peak, shortcut) {
                continue;
            }
            let support_ratio = positive_lane_support_ratio(peak, shortcut);
            let lane_ratio = round4(query_ratio * support_ratio);
            if lane_ratio <= 0.0 {
                continue;
            }
            let boost = round4(learned_boost * lane_ratio);
            let old_score = peak["score"].as_f64().unwrap_or(0.0);
            let new_score = round4((old_score + boost).min(1.5));
            if let Some(object) = peak.as_object_mut() {
                object.insert("score".to_string(), json!(new_score));
                object.insert("raw_score".to_string(), json!(round4(old_score)));
                object.insert("positive_lane_boost".to_string(), json!(boost));
            }
            reinforcements.push(json!({
                "shortcut": shortcut.id,
                "reinforce_peak": peak_name,
                "reinforce_route": shortcut.reinforce_route,
                "reinforce_group": shortcut.reinforce_group,
                "boost": boost,
                "effective_boost": round4(learned_boost),
                "match_ratio": lane_ratio,
                "query_match_ratio": round4(query_ratio),
                "support_match_ratio": round4(support_ratio),
                "observations": shortcut.observations,
                "accepted_count": accepted_count,
                "reason": shortcut.reason
            }));
        }
    }
    json!({
        "applied": !reinforcements.is_empty(),
        "positive_lanes": shortcuts.len(),
        "reinforcements": reinforcements
    })
}

pub(crate) fn apply_negative_lanes(
    peaks: &mut [Value],
    query: &[Triad],
    shortcuts: &[NegativeShortcut],
) -> Value {
    let query_tokens = query_token_set(query);
    let mut suppressions = vec![];
    if shortcuts.is_empty() || peaks.is_empty() {
        return json!({
            "applied": false,
            "negative_lanes": shortcuts.len(),
            "suppressions": suppressions
        });
    }
    for shortcut in shortcuts {
        let query_ratio = negative_lane_match_ratio(&query_tokens, shortcut);
        if query_ratio <= 0.0 {
            continue;
        }
        let rejected_count = shortcut_rejected_count(shortcut);
        let learned_penalty = (shortcut.penalty.max(0.0)
            + (rejected_count.saturating_sub(1) as f64 * 0.04))
            .min(0.45);
        for peak in peaks.iter_mut() {
            let Some(peak_name) = peak["peak"].as_str().map(str::to_string) else {
                continue;
            };
            let support_ratio = negative_lane_support_ratio(peak, shortcut);
            let lane_ratio = round4(query_ratio * support_ratio);
            if lane_ratio <= 0.0 {
                continue;
            }
            let penalty = round4(learned_penalty * lane_ratio);
            let boost = round4(penalty * 0.35);
            if negative_lane_matches_suppress(peak, shortcut) {
                let old_score = peak["score"].as_f64().unwrap_or(0.0);
                let new_score = round4((old_score - penalty).max(0.0));
                if let Some(object) = peak.as_object_mut() {
                    object.insert("score".to_string(), json!(new_score));
                    object.insert("raw_score".to_string(), json!(round4(old_score)));
                    object.insert("negative_lane_penalty".to_string(), json!(penalty));
                }
                suppressions.push(json!({
                    "shortcut": shortcut.id,
                    "suppress_peak": peak_name,
                    "suppressed_peak": peak_name,
                    "suppress_route": shortcut.suppress_route,
                    "suppress_group": shortcut.suppress_group,
                    "penalty": penalty,
                    "effective_penalty": round4(learned_penalty),
                    "match_ratio": lane_ratio,
                    "query_match_ratio": round4(query_ratio),
                    "support_match_ratio": round4(support_ratio),
                    "observations": shortcut.observations,
                    "rejected_count": rejected_count,
                    "prefer_peak": shortcut.prefer_peak,
                    "prefer_route": shortcut.prefer_route,
                    "prefer_group": shortcut.prefer_group,
                    "reason": shortcut.reason
                }));
            } else if (!shortcut.prefer_peak.is_empty()
                || !shortcut.prefer_route.is_empty()
                || !shortcut.prefer_group.is_empty())
                && negative_lane_matches_prefer(peak, shortcut)
            {
                let old_score = peak["score"].as_f64().unwrap_or(0.0);
                let new_score = round4(old_score + boost);
                if let Some(object) = peak.as_object_mut() {
                    object.insert("score".to_string(), json!(new_score));
                    object.insert("raw_score".to_string(), json!(round4(old_score)));
                    object.insert("negative_lane_boost".to_string(), json!(boost));
                }
            }
        }
    }
    json!({
        "applied": !suppressions.is_empty(),
        "negative_lanes": shortcuts.len(),
        "suppressions": suppressions
    })
}

pub(crate) fn negative_lane_matches_suppress(peak: &Value, shortcut: &NegativeShortcut) -> bool {
    negative_lane_matches_labels(
        peak,
        &[
            shortcut.suppress_peak.as_str(),
            shortcut.suppress_route.as_str(),
            shortcut.suppress_group.as_str(),
        ],
    )
}

pub(crate) fn negative_lane_matches_prefer(peak: &Value, shortcut: &NegativeShortcut) -> bool {
    negative_lane_matches_labels(
        peak,
        &[
            shortcut.prefer_peak.as_str(),
            shortcut.prefer_route.as_str(),
            shortcut.prefer_group.as_str(),
        ],
    )
}

pub(crate) fn positive_lane_matches_reinforce(peak: &Value, shortcut: &PositiveShortcut) -> bool {
    negative_lane_matches_labels(
        peak,
        &[
            shortcut.reinforce_peak.as_str(),
            shortcut.reinforce_route.as_str(),
            shortcut.reinforce_group.as_str(),
        ],
    )
}

pub(crate) fn negative_lane_matches_labels(peak: &Value, labels: &[&str]) -> bool {
    let peak_name = peak["peak"].as_str().unwrap_or("");
    let route = peak["center"]["route"].as_str().unwrap_or("");
    let group = peak["center"]["group"].as_str().unwrap_or("");
    let composite = format!("{route}:{group}");
    labels.iter().any(|hint| {
        let hint = norm(hint);
        !hint.is_empty()
            && (hint == norm(peak_name)
                || hint == norm(route)
                || hint == norm(group)
                || hint == norm(&composite))
    })
}

pub(crate) fn positive_lane_support_ratio(peak: &Value, shortcut: &PositiveShortcut) -> f64 {
    let terms = normalized_shortcut_terms(&shortcut.support_terms);
    if terms.is_empty() {
        return 1.0;
    }
    let mut peak_terms = BTreeSet::new();
    if let Some(items) = peak["supporting_triads"].as_array() {
        for item in items.iter().take(8) {
            for key in ["subject", "relation", "object", "route", "group"] {
                if let Some(value) = item[key].as_str() {
                    peak_terms.extend(normalized_shortcut_terms(&[value.to_string()]));
                }
            }
        }
    }
    if peak_terms.is_empty() {
        return 0.0;
    }
    let hits = terms.intersection(&peak_terms).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.35 {
        ratio
    } else {
        0.0
    }
}

pub(crate) fn negative_lane_support_ratio(peak: &Value, shortcut: &NegativeShortcut) -> f64 {
    let terms = normalized_shortcut_terms(&shortcut.support_terms);
    if terms.is_empty() {
        return 1.0;
    }
    let mut peak_terms = BTreeSet::new();
    if let Some(items) = peak["supporting_triads"].as_array() {
        for item in items.iter().take(8) {
            for key in ["subject", "relation", "object", "route", "group"] {
                if let Some(value) = item[key].as_str() {
                    peak_terms.extend(normalized_shortcut_terms(&[value.to_string()]));
                }
            }
        }
    }
    if peak_terms.is_empty() {
        return 0.0;
    }
    let hits = terms.intersection(&peak_terms).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.35 {
        ratio
    } else {
        0.0
    }
}

pub(crate) fn shortcut_accepted_count(shortcut: &PositiveShortcut) -> usize {
    if shortcut.accepted_count > 0 {
        shortcut.accepted_count
    } else {
        shortcut.observations.max(1)
    }
}

pub(crate) fn shortcut_rejected_count(shortcut: &NegativeShortcut) -> usize {
    if shortcut.rejected_count > 0 {
        shortcut.rejected_count
    } else {
        shortcut.observations.max(1)
    }
}

pub(crate) fn positive_lane_match_ratio(
    query_tokens: &BTreeSet<String>,
    shortcut: &PositiveShortcut,
) -> f64 {
    if shortcut.reinforce_peak.trim().is_empty() {
        return 0.0;
    }
    if shortcut.terms.is_empty() {
        return 1.0;
    }
    if query_tokens.is_empty() {
        return 0.0;
    }
    let terms = normalized_shortcut_terms(&shortcut.terms);
    if terms.is_empty() {
        return 1.0;
    }
    let hits = terms.intersection(query_tokens).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.5 {
        ratio
    } else {
        0.0
    }
}

pub(crate) fn negative_lane_match_ratio(
    query_tokens: &BTreeSet<String>,
    shortcut: &NegativeShortcut,
) -> f64 {
    if shortcut.suppress_peak.trim().is_empty() {
        return 0.0;
    }
    if shortcut.terms.is_empty() {
        return 1.0;
    }
    if query_tokens.is_empty() {
        return 0.0;
    }
    let terms = normalized_shortcut_terms(&shortcut.terms);
    if terms.is_empty() {
        return 1.0;
    }
    let hits = terms.intersection(query_tokens).count();
    let ratio = hits as f64 / terms.len() as f64;
    if ratio >= 0.5 {
        ratio
    } else {
        0.0
    }
}

pub(crate) fn print_probe_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("decision: {}", out["decision"].as_str().unwrap_or(""));
    println!(
        "plain: top={} verdict={} field={} safe={} score={}",
        out["plain"]["top_peak"].as_str().unwrap_or(""),
        out["plain"]["verdict"].as_str().unwrap_or(""),
        out["plain"]["field_state"].as_str().unwrap_or(""),
        out["plain"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["plain"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "negative: top={} verdict={} field={} safe={} score={}",
        out["negative"]["top_peak"].as_str().unwrap_or(""),
        out["negative"]["verdict"].as_str().unwrap_or(""),
        out["negative"]["field_state"].as_str().unwrap_or(""),
        out["negative"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["negative"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "delta: top_changed={} score_delta={} suppression_count={}",
        out["delta"]["top_changed"].as_bool().unwrap_or(false),
        out["delta"]["score_delta"].as_f64().unwrap_or(0.0),
        out["delta"]["suppression_count"].as_u64().unwrap_or(0)
    );
}

pub(crate) fn print_probe_suite_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("mode: {}", out["mode"].as_str().unwrap_or("probe-suite"));
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {}: {} plain={} negative={}",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_plain_peak"].as_str().unwrap_or(""),
                case["actual_negative_peak"].as_str().unwrap_or("")
            );
        }
    }
}

pub(crate) fn print_probe_md(out: &Value) {
    println!("# NANDA Probe\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- decision: `{}`", out["decision"].as_str().unwrap_or(""));
    println!(
        "- plain: `{}` / `{}` / safe `{}` / score `{}`",
        out["plain"]["top_peak"].as_str().unwrap_or(""),
        out["plain"]["field_state"].as_str().unwrap_or(""),
        out["plain"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["plain"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "- negative: `{}` / `{}` / safe `{}` / score `{}`",
        out["negative"]["top_peak"].as_str().unwrap_or(""),
        out["negative"]["field_state"].as_str().unwrap_or(""),
        out["negative"]["safe_to_answer"].as_bool().unwrap_or(false),
        out["negative"]["top_score"].as_f64().unwrap_or(0.0)
    );
    println!(
        "- delta: top_changed `{}` / score_delta `{}` / suppressions `{}`",
        out["delta"]["top_changed"].as_bool().unwrap_or(false),
        out["delta"]["score_delta"].as_f64().unwrap_or(0.0),
        out["delta"]["suppression_count"].as_u64().unwrap_or(0)
    );
}

pub(crate) fn print_probe_suite_md(out: &Value) {
    println!("# NANDA Probe Suite\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- `{}`: `{}` plain `{}` negative `{}`",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_plain_peak"].as_str().unwrap_or(""),
                case["actual_negative_peak"].as_str().unwrap_or("")
            );
        }
    }
}
