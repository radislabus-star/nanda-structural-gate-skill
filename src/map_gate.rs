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

    let route_field = route_field(source, candidates);
    let owner_gravity = owner_gravity(source, candidates);
    let negative_routes = negative_route_hits(source, candidates);
    let structural_energy = structural_energy(
        &route_field,
        &owner_gravity,
        &foreign_pull,
        &negative_routes,
    );
    let repair_queue = repair_queue(
        &repair_tasks,
        &foreign_pull,
        &owner_gravity,
        &negative_routes,
    );

    json!({
        "core_version": CORE_VERSION,
        "wave_dim": nanda_6m::WAVE_DIM,
        "route_field": route_field,
        "owner_gravity": owner_gravity,
        "negative_routes": negative_routes,
        "structural_energy": structural_energy,
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
        "repair_tasks": repair_tasks,
        "repair_queue": repair_queue
    })
}

fn route_field(source: &[Triad], candidates: &[Triad]) -> Value {
    let mut routes = BTreeMap::<String, Vec<&Triad>>::new();
    for triad in source.iter().chain(candidates) {
        routes
            .entry(route_name(triad, "unrouted"))
            .or_default()
            .push(triad);
    }
    let mut rows = serde_json::Map::new();
    for (route, triads) in routes {
        let mut layers = BTreeSet::new();
        let mut owners = BTreeSet::new();
        let mut entrypoints = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        let mut evidence_paths = BTreeSet::new();
        let mut scopes = BTreeSet::new();
        let mut source_count = 0usize;
        let mut candidate_count = 0usize;
        let mut confidence_sum = 0.0;
        for triad in &triads {
            if triad.id.starts_with('c') {
                candidate_count += 1;
            } else {
                source_count += 1;
            }
            confidence_sum += triad.confidence;
            insert_nonempty(&mut layers, &layer_name(triad));
            insert_nonempty(&mut owners, &owner_name(triad));
            insert_nonempty(&mut entrypoints, &triad.entrypoint);
            insert_nonempty(&mut outputs, &triad.output);
            insert_nonempty(&mut evidence_paths, &evidence_path(triad));
            insert_nonempty(&mut scopes, &scope_name(triad));
        }
        let avg_confidence = if triads.is_empty() {
            0.0
        } else {
            confidence_sum / triads.len() as f64
        };
        rows.insert(
            route,
            json!({
                "triads": triads.len(),
                "source_triads": source_count,
                "candidate_triads": candidate_count,
                "layers": layers.into_iter().collect::<Vec<_>>(),
                "owners": owners.into_iter().collect::<Vec<_>>(),
                "entrypoints": entrypoints.into_iter().collect::<Vec<_>>(),
                "outputs": outputs.into_iter().collect::<Vec<_>>(),
                "evidence_paths": evidence_paths.into_iter().collect::<Vec<_>>(),
                "scopes": scopes.into_iter().collect::<Vec<_>>(),
                "avg_confidence": round4(avg_confidence)
            }),
        );
    }
    json!({
        "routes": rows,
        "read_as": "Route field coordinates describe causal ownership: route, layer, owner, entrypoint, output, evidence_path, and scope."
    })
}

fn owner_gravity(source: &[Triad], candidates: &[Triad]) -> Value {
    let mut by_route = BTreeMap::<String, BTreeMap<String, usize>>::new();
    let mut by_group = BTreeMap::<String, BTreeMap<String, usize>>::new();
    let mut decision_owner = BTreeMap::<(String, String), BTreeMap<String, Vec<String>>>::new();
    for triad in source.iter().chain(candidates) {
        let owner = owner_name(triad);
        by_route
            .entry(route_name(triad, "unrouted"))
            .or_default()
            .entry(owner.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        by_group
            .entry(group_name(triad, "ungrouped"))
            .or_default()
            .entry(owner.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        if is_decision_relation(triad) {
            decision_owner
                .entry((route_name(triad, "unrouted"), norm(&triad.object)))
                .or_default()
                .entry(owner)
                .or_default()
                .push(triad.id.clone());
        }
    }

    let mut route_centers = serde_json::Map::new();
    for (route, owners) in &by_route {
        route_centers.insert(route.clone(), owner_center(owners));
    }
    let mut group_centers = serde_json::Map::new();
    let mut conflicts = vec![];
    for (group, owners) in &by_group {
        let center = owner_center(owners);
        let distinct = owners.keys().filter(|owner| !owner.is_empty()).count();
        if distinct > 1 {
            conflicts.push(json!({
                "kind": "owner_conflict",
                "group": group,
                "owners": owners.keys().cloned().collect::<Vec<_>>(),
                "center": center["owner"],
                "risk": "helper or candidate group is attracted to more than one decision owner",
                "repair": "Split the group by owner or move decision logic behind one public entrypoint."
            }));
        }
        group_centers.insert(group.clone(), center);
    }
    for ((route, object), owners) in decision_owner {
        if owners.len() > 1 {
            conflicts.push(json!({
                "kind": "duplicate_decision_owner",
                "route": route,
                "decision_object": object,
                "owners": owners,
                "risk": "two owners appear to decide the same route object",
                "repair": "Pick one decision owner and demote the other path to adapter/helper."
            }));
        }
    }
    json!({
        "route_centers": route_centers,
        "group_centers": group_centers,
        "conflicts": conflicts
    })
}

fn negative_route_hits(source: &[Triad], candidates: &[Triad]) -> Value {
    let mut hits = vec![];
    for triad in source.iter().chain(candidates) {
        let layer = layer_name(triad);
        let relation = norm(&triad.relation);
        let subject_role = norm(&triad.subject_role);
        let object_role = norm(&triad.object_role);
        let scope = scope_name(triad);
        let hit = if matches!(layer.as_str(), "adapter" | "ui")
            && (is_decision_relation(triad) || relation.contains("source_of_truth"))
        {
            Some((
                "adapter_or_ui_must_not_decide",
                "Adapter/UI layer is taking a decision or source-of-truth role.",
                "Move decision to core owner and keep this layer as input/display adapter.",
            ))
        } else if layer == "test"
            && (relation.contains("executes") || relation.contains("calls"))
            && scope != "test"
        {
            Some((
                "test_helper_must_not_be_runtime",
                "Test layer appears to drive a non-test route.",
                "Mark the path test-only or move reusable behavior into a production owner.",
            ))
        } else if layer == "experiment"
            && (relation.contains("affects")
                || relation.contains("controls")
                || relation.contains("writes")
                || relation.contains("executes"))
        {
            Some((
                "experiment_must_not_affect_stable_path",
                "Experimental layer still affects a stable route.",
                "Isolate the experiment behind an explicit feature gate or remove runtime wiring.",
            ))
        } else if (subject_role.contains("helper") || object_role.contains("helper"))
            && is_decision_relation(triad)
        {
            Some((
                "helper_must_not_own_decision",
                "Helper role is taking an ownership/decision relation.",
                "Make the helper private and route decisions through the owner entrypoint.",
            ))
        } else {
            None
        };
        if let Some((rule, reason, repair)) = hit {
            hits.push(json!({
                "rule": rule,
                "triad": triad.id,
                "route": route_name(triad, "unrouted"),
                "group": group_name(triad, "ungrouped"),
                "layer": layer,
                "owner": owner_name(triad),
                "relation": triad.relation,
                "subject": triad.subject,
                "object": triad.object,
                "reason": reason,
                "repair": repair,
                "evidence": evidence_path(triad)
            }));
        }
    }
    json!({
        "anti_modes": [
            "adapter must not decide",
            "UI must not be source of truth",
            "test helper must not become runtime route",
            "experiment must not affect stable path",
            "one decision owner only"
        ],
        "hits": hits
    })
}

fn structural_energy(
    route_field: &Value,
    owner_gravity: &Value,
    foreign_pull: &[Value],
    negative_routes: &Value,
) -> Value {
    let route_count = route_field["routes"]
        .as_object()
        .map(|v| v.len())
        .unwrap_or(0);
    let mut route_confidence = 0.0;
    if let Some(routes) = route_field["routes"].as_object() {
        for route in routes.values() {
            route_confidence += route["avg_confidence"].as_f64().unwrap_or(0.0);
        }
    }
    let route_coherence = if route_count == 0 {
        0.0
    } else {
        route_confidence / route_count as f64
    };
    let owner_conflict = owner_gravity["conflicts"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let negative_hits = negative_routes["hits"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    json!({
        "route_coherence": round4(route_coherence),
        "owner_conflict": owner_conflict,
        "foreign_pull_energy": foreign_pull.len(),
        "duplicate_decision_risk": owner_gravity["conflicts"].as_array().map(|items| {
            items.iter().filter(|item| item["kind"].as_str() == Some("duplicate_decision_owner")).count()
        }).unwrap_or(0),
        "adapter_leak_risk": negative_routes["hits"].as_array().map(|items| {
            items.iter().filter(|item| item["rule"].as_str() == Some("adapter_or_ui_must_not_decide")).count()
        }).unwrap_or(0),
        "stale_experiment_risk": negative_routes["hits"].as_array().map(|items| {
            items.iter().filter(|item| item["rule"].as_str() == Some("experiment_must_not_affect_stable_path")).count()
        }).unwrap_or(0),
        "ui_runtime_mismatch_risk": negative_hits,
        "field_tension": owner_conflict + foreign_pull.len() + negative_hits
    })
}

fn repair_queue(
    repair_tasks: &[Value],
    foreign_pull: &[Value],
    owner_gravity: &Value,
    negative_routes: &Value,
) -> Value {
    let mut queue = vec![];
    for task in repair_tasks {
        queue.push(json!({
            "kind": "coherence_repair",
            "priority": "medium",
            "target": task["candidate_group"],
            "reason": task["reason"],
            "repair": task["suggested_fix"]
        }));
    }
    for pull in foreign_pull {
        queue.push(json!({
            "kind": "foreign_pull_repair",
            "priority": "high",
            "target": pull["candidate_triad"],
            "reason": "candidate triad pulls toward a foreign source route",
            "repair": pull["repair"],
            "owner_should_review": pull["triad_best_source_group"]
        }));
    }
    if let Some(conflicts) = owner_gravity["conflicts"].as_array() {
        for conflict in conflicts {
            queue.push(json!({
                "kind": conflict["kind"],
                "priority": "high",
                "target": conflict["group"].as_str().unwrap_or(conflict["route"].as_str().unwrap_or("owner-conflict")),
                "reason": conflict["risk"],
                "repair": conflict["repair"]
            }));
        }
    }
    if let Some(hits) = negative_routes["hits"].as_array() {
        for hit in hits {
            queue.push(json!({
                "kind": hit["rule"],
                "priority": "high",
                "target": hit["triad"],
                "reason": hit["reason"],
                "repair": hit["repair"]
            }));
        }
    }
    json!(queue)
}

fn owner_center(owners: &BTreeMap<String, usize>) -> Value {
    let mut best_owner = "";
    let mut best_count = 0usize;
    let total: usize = owners.values().sum();
    for (owner, count) in owners {
        if *count > best_count {
            best_owner = owner;
            best_count = *count;
        }
    }
    json!({
        "owner": best_owner,
        "support": best_count,
        "total": total,
        "gravity": if total == 0 { 0.0 } else { round4(best_count as f64 / total as f64) },
        "owners": owners
    })
}

fn insert_nonempty(values: &mut BTreeSet<String>, value: &str) {
    if !value.trim().is_empty() {
        values.insert(norm(value));
    }
}

fn layer_name(triad: &Triad) -> String {
    if !triad.layer.trim().is_empty() {
        return norm(&triad.layer);
    }
    let relation = norm(&triad.relation);
    let subject_role = norm(&triad.subject_role);
    let object_role = norm(&triad.object_role);
    if subject_role.contains("ui") || object_role.contains("ui") {
        "ui".to_string()
    } else if subject_role.contains("test") || object_role.contains("test") {
        "test".to_string()
    } else if subject_role.contains("wrapper") || relation.contains("executes") {
        "runtime".to_string()
    } else if subject_role.contains("adapter") || object_role.contains("adapter") {
        "adapter".to_string()
    } else {
        "core".to_string()
    }
}

fn owner_name(triad: &Triad) -> String {
    if !triad.owner.trim().is_empty() {
        return norm(&triad.owner);
    }
    if norm(&triad.subject_role).contains("owner") {
        return norm(&triad.subject);
    }
    if norm(&triad.object_role).contains("owner") {
        return norm(&triad.object);
    }
    group_name(triad, "unowned")
}

fn scope_name(triad: &Triad) -> String {
    if triad.scope.trim().is_empty() {
        route_name(triad, "unscoped")
    } else {
        norm(&triad.scope)
    }
}

fn evidence_path(triad: &Triad) -> String {
    if triad.evidence_path.trim().is_empty() {
        triad.evidence.clone()
    } else {
        triad.evidence_path.clone()
    }
}

fn is_decision_relation(triad: &Triad) -> bool {
    let relation = norm(&triad.relation);
    relation.contains("decides")
        || relation.contains("owns")
        || relation.contains("controls")
        || relation.contains("selects")
        || relation.contains("writes")
        || relation.contains("mutates")
        || relation.contains("source_of_truth")
}

pub(crate) fn codex_failure_field(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
) -> Value {
    let contract = &packet.failure_contract;
    if !contract["enabled"].as_bool().unwrap_or(false) {
        return json!({
            "enabled": false,
            "verdict": "NOT_ENABLED",
            "reason_codes": [],
            "read_as": "Add failure_contract.enabled=true to turn on Codex Failure Field."
        });
    }

    let symptom = contract["symptom"].as_str().unwrap_or(&packet.query);
    let action_id = contract["selected_action_id"].as_str().unwrap_or("");
    let runtime_snapshot = &contract["runtime_snapshot"];
    let actions = contract["actions"].as_array().cloned().unwrap_or_default();
    let verification = contract["verification"].as_object();
    let mut reason_codes = Vec::<String>::new();
    let mut blocked_operations = BTreeSet::<String>::new();
    let mut allowed_operations = BTreeSet::<String>::new();
    let mut diagnostics = vec![];

    if is_hard_stop(symptom)
        || contract["hard_stop"].as_bool().unwrap_or(false)
        || is_hard_stop(&packet.candidate_answer)
    {
        return json!({
            "enabled": true,
            "verdict": "HARD_STOP",
            "reason_codes": ["hard_stop_signal"],
            "allowed_operations": [],
            "blocked_operations": ["tools", "code", "restart", "install", "runtime_mutation"],
            "message": "No tools, no code, no restart"
        });
    }

    let selected_action = actions
        .iter()
        .find(|action| action["action_id"].as_str() == Some(action_id));
    if action_id.trim().is_empty() || action_id.split('.').count() < 2 {
        reason_codes.push("action_id_missing_or_too_generic".to_string());
        diagnostics.push(json!({
            "kind": "analysis_quality",
            "risk": "Codex is about to fix a blurred action instead of a named route action.",
            "repair": "Choose a concrete action_id such as domain.route.operation before editing."
        }));
    }
    if selected_action.is_none() {
        reason_codes.push("action_id_not_found".to_string());
    }

    if let Some(action) = selected_action {
        for route in string_array(&action["allowed_routes"]) {
            allowed_operations.insert(route);
        }
        for route in string_array(&action["forbidden_routes"]) {
            blocked_operations.insert(route);
        }
        let owner = action["owner"].as_str().unwrap_or("");
        if owner.trim().is_empty() {
            reason_codes.push("action_owner_missing".to_string());
        }
        if string_array(&action["input"]).is_empty() || string_array(&action["output"]).is_empty() {
            reason_codes.push("action_io_contract_missing".to_string());
        }
    }

    let evidence_items = contract["evidence"].as_array().cloned().unwrap_or_default();
    if evidence_items.is_empty() {
        reason_codes.push("evidence_missing".to_string());
    }
    let runtime_evidence =
        evidence_items.iter().any(is_runtime_evidence) || runtime_snapshot.is_object();
    let code_action = selected_action.is_some_and(is_code_change_action);
    if runtime_evidence && code_action {
        reason_codes.push("symptom_action_mismatch".to_string());
        diagnostics.push(json!({
            "kind": "symptom_action_mismatch",
            "risk": "Evidence points to runtime/config state, but selected action is a code-change route.",
            "repair": "Run or repair the runtime route first; do not edit candidate generation or unrelated code."
        }));
    }

    let namespace_terms = string_array(&contract["namespace_terms"]);
    let ambiguous_terms = namespace_confusions(source, candidates, &namespace_terms);
    if !ambiguous_terms.is_empty() {
        reason_codes.push("namespace_confusion".to_string());
        diagnostics.push(json!({
            "kind": "namespace_confusion",
            "terms": ambiguous_terms,
            "risk": "Same surface term is used without namespace across multiple routes.",
            "repair": "Use namespaced entities such as daemon.word_buffer.tail or ime.preedit_buffer."
        }));
    }

    let allowed_routes = selected_action
        .map(|action| string_array(&action["allowed_routes"]))
        .unwrap_or_default();
    let touched_routes = candidates
        .iter()
        .map(|triad| route_name(triad, "unrouted"))
        .collect::<BTreeSet<_>>();
    let side_effect_routes = touched_routes
        .iter()
        .filter(|route| !allowed_routes.is_empty() && !allowed_routes.contains(route))
        .cloned()
        .collect::<Vec<_>>();
    if !side_effect_routes.is_empty() {
        reason_codes.push("side_effect_creep".to_string());
        diagnostics.push(json!({
            "kind": "side_effect_creep",
            "touched_routes": side_effect_routes,
            "allowed_routes": allowed_routes,
            "risk": "Candidate diff touches routes outside the selected action.",
            "repair": "Split the patch or justify the extra route with explicit evidence."
        }));
    }

    if runtime_snapshot.is_object() && selected_action.is_some_and(is_code_change_action) {
        reason_codes.push("runtime_blindness".to_string());
    }
    if verification.is_none_or(|items| items.is_empty()) {
        reason_codes.push("verification_missing".to_string());
    } else if !verification_matches_routes(verification.unwrap(), &allowed_routes) {
        reason_codes.push("fake_verification".to_string());
    }

    if contract["checkpoint_before_risky_change"].as_bool() == Some(false) {
        reason_codes.push("no_checkpoint_before_risky_change".to_string());
    }
    if contract["hypothesis_proven"].as_bool() == Some(false) {
        reason_codes.push("unproven_hypothesis".to_string());
    }
    if contract["example_specific_patch"].as_bool() == Some(true) {
        reason_codes.push("example_specific_patch".to_string());
    }

    reason_codes.sort();
    reason_codes.dedup();
    let verdict = if reason_codes.iter().any(|code| {
        matches!(
            code.as_str(),
            "symptom_action_mismatch"
                | "side_effect_creep"
                | "runtime_blindness"
                | "fake_verification"
                | "no_checkpoint_before_risky_change"
        )
    }) {
        "VETO"
    } else if !reason_codes.is_empty() {
        "ANALYSIS_INSUFFICIENT"
    } else {
        "PASS"
    };

    json!({
        "enabled": true,
        "verdict": verdict,
        "symptom": symptom,
        "selected_action_id": action_id,
        "reason_codes": reason_codes,
        "allowed_operations": allowed_operations.into_iter().collect::<Vec<_>>(),
        "blocked_operations": blocked_operations.into_iter().collect::<Vec<_>>(),
        "diagnostics": diagnostics,
        "runtime_snapshot_present": runtime_snapshot.is_object(),
        "evidence_count": evidence_items.len(),
        "read_as": "Codex Failure Field checks whether the selected action is proven by evidence and confined to its allowed route before editing."
    })
}

fn is_hard_stop(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "stop",
        "стой",
        "остановись",
        "не трогай код",
        "ничего не делай",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(norm)
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn is_runtime_evidence(value: &Value) -> bool {
    let text = value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
        .to_lowercase();
    [
        "systemd",
        "process",
        "running",
        "not running",
        "config",
        "engine",
        "runtime",
        "version",
        "service",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn is_code_change_action(value: &Value) -> bool {
    let text = value.to_string().to_lowercase();
    [
        "edit",
        "code",
        "refactor",
        "patch",
        "change source",
        "candidate generation",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn namespace_confusions(
    source: &[Triad],
    candidates: &[Triad],
    namespace_terms: &[String],
) -> Vec<Value> {
    let mut by_term = BTreeMap::<String, BTreeSet<String>>::new();
    for triad in source.iter().chain(candidates) {
        for entity in [&triad.subject, &triad.object] {
            let entity_norm = norm(entity);
            let leaf = entity_norm
                .rsplit(['.', ':', '/'])
                .next()
                .unwrap_or(&entity_norm)
                .to_string();
            if namespace_terms.contains(&leaf)
                && !entity_norm.contains('.')
                && !entity_norm.contains(':')
            {
                by_term
                    .entry(leaf)
                    .or_default()
                    .insert(route_name(triad, "unrouted"));
            }
        }
    }
    by_term
        .into_iter()
        .filter(|(_, routes)| routes.len() > 1)
        .map(|(term, routes)| {
            json!({
                "term": term,
                "routes": routes.into_iter().collect::<Vec<_>>()
            })
        })
        .collect()
}

fn verification_matches_routes(
    verification: &serde_json::Map<String, Value>,
    allowed_routes: &[String],
) -> bool {
    if allowed_routes.is_empty() {
        return false;
    }
    allowed_routes.iter().all(|route| {
        verification.contains_key(route) || verification.contains_key(&route.replace('-', "_"))
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
                    failure_contract: packet.failure_contract.clone(),
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
    map["codex_failure_field"] = codex_failure_field(&packet, &source, &candidates);
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
    let global_failure_field = codex_failure_field(&packet, &source, &candidates);
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
            failure_contract: packet.failure_contract.clone(),
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
    let decision = hierarchical_decision(
        &global_report,
        &global_map,
        &branches,
        truncated,
        &global_failure_field,
    );
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
            "codex_failure_field": global_failure_field,
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
            "REPAIR_REQUIRED" | "HARD_STOP" => EXIT_VETO,
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
        failure_contract: Value::Null,
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
        failure_contract: packet.failure_contract.clone(),
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
