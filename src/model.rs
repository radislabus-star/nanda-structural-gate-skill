use crate::*;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub(crate) fn default_negative_penalty() -> f64 {
    0.18
}

pub(crate) fn default_positive_boost() -> f64 {
    0.08
}

pub(crate) fn default_one_usize() -> usize {
    1
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_confidence() -> f64 {
    1.0
}

pub(crate) fn default_subject_role() -> String {
    "subject".to_string()
}

pub(crate) fn default_object_role() -> String {
    "object".to_string()
}

pub(crate) fn default_task_id() -> String {
    "task".to_string()
}

pub(crate) fn default_domain() -> String {
    "general".to_string()
}

pub(crate) fn normalize_ids(mut triads: Vec<Triad>, prefix: &str) -> Vec<Triad> {
    for (idx, triad) in triads.iter_mut().enumerate() {
        if triad.id.is_empty() {
            triad.id = format!("{prefix}{}", idx + 1);
        }
        if triad.subject_role.is_empty() {
            triad.subject_role = "subject".to_string();
        }
        if triad.object_role.is_empty() {
            triad.object_role = "object".to_string();
        }
        if triad.confidence == 0.0 {
            triad.confidence = 1.0;
        }
    }
    triads
}

pub(crate) fn norm(value: &str) -> String {
    value.trim().to_lowercase()
}

pub(crate) fn structural_key(triad: &Triad) -> (String, String, String) {
    (
        norm(&triad.subject),
        norm(&triad.relation),
        norm(&triad.object),
    )
}

pub(crate) fn reversed_structural_key(triad: &Triad) -> (String, String, String) {
    (
        norm(&triad.object),
        norm(&triad.relation),
        norm(&triad.subject),
    )
}

pub(crate) fn full_key(triad: &Triad) -> (String, String, String, String, String) {
    (
        norm(&triad.subject_role),
        norm(&triad.subject),
        norm(&triad.relation),
        norm(&triad.object_role),
        norm(&triad.object),
    )
}

pub(crate) fn vector(label: &str) -> Vec<i32> {
    let mut out = Vec::with_capacity(WAVE_DIM);
    let mut counter = 0u64;
    while out.len() < WAVE_DIM {
        let mut hasher = Sha256::new();
        hasher.update(format!("{label}|{counter}").as_bytes());
        let digest = hasher.finalize();
        for byte in digest {
            for bit in 0..8 {
                out.push(if ((byte >> bit) & 1) == 1 { 1 } else { -1 });
                if out.len() == WAVE_DIM {
                    break;
                }
            }
            if out.len() == WAVE_DIM {
                break;
            }
        }
        counter += 1;
    }
    out
}

pub(crate) fn bind(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().zip(b).map(|(x, y)| x * y).collect()
}

pub(crate) fn rotate(value: &[i32], amount: usize) -> Vec<i32> {
    if value.is_empty() {
        return vec![];
    }
    let amount = amount % value.len();
    value[amount..]
        .iter()
        .chain(value[..amount].iter())
        .copied()
        .collect()
}

pub(crate) fn add_into(acc: &mut [i32], value: &[i32]) {
    for (dst, src) in acc.iter_mut().zip(value) {
        *dst += src;
    }
}

pub(crate) fn cosine(a: &[i32], b: &[i32]) -> f64 {
    let dot: i64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| (*x as i64) * (*y as i64))
        .sum();
    let na: f64 = a
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    let nb: f64 = b
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot as f64 / (na * nb)
    }
}

pub(crate) fn triad_wave(triad: &Triad) -> Vec<i32> {
    let subject_binding = bind(
        &vector(&format!("role:{}", norm(&triad.subject_role))),
        &bind(
            &vector("position:subject"),
            &rotate(&vector(&format!("entity:{}", norm(&triad.subject))), 17),
        ),
    );
    let object_binding = bind(
        &vector(&format!("role:{}", norm(&triad.object_role))),
        &bind(
            &vector("position:object"),
            &rotate(&vector(&format!("entity:{}", norm(&triad.object))), 73),
        ),
    );
    let relation_mode = vector(&format!("relation:{}", norm(&triad.relation)));
    bind(&bind(&subject_binding, &relation_mode), &object_binding)
}

pub(crate) fn build_memory(source: &[Triad]) -> Vec<i32> {
    let mut memory = vec![0; WAVE_DIM];
    for triad in source {
        add_into(&mut memory, &triad_wave(triad));
    }
    memory
}

pub(crate) fn entity_set(triads: &[Triad]) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for triad in triads {
        if !triad.subject.is_empty() {
            values.insert(norm(&triad.subject));
        }
        if !triad.object.is_empty() {
            values.insert(norm(&triad.object));
        }
    }
    values
}

pub(crate) fn role_set(triads: &[Triad]) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for triad in triads {
        values.insert(norm(&triad.subject_role));
        values.insert(norm(&triad.object_role));
    }
    values
}

pub(crate) fn relation_set(triads: &[Triad]) -> BTreeSet<String> {
    triads
        .iter()
        .filter(|x| !x.relation.is_empty())
        .map(|x| norm(&x.relation))
        .collect()
}

pub(crate) fn evidence_set(triads: &[Triad]) -> BTreeSet<String> {
    triads
        .iter()
        .filter(|x| !x.evidence.is_empty())
        .map(|x| norm(&x.evidence))
        .collect()
}

pub(crate) fn count_conflicting_sources(triads: &[Triad]) -> i64 {
    let mut by_evidence: HashMap<String, HashSet<(String, String, String)>> = HashMap::new();
    for triad in triads {
        if triad.evidence.is_empty() {
            continue;
        }
        by_evidence
            .entry(norm(&triad.evidence))
            .or_default()
            .insert(structural_key(triad));
    }
    by_evidence.values().filter(|keys| keys.len() > 1).count() as i64
}

pub(crate) fn complexity_score(source: &[Triad], candidates: &[Triad]) -> i64 {
    let all: Vec<Triad> = source.iter().chain(candidates).cloned().collect();
    entity_set(&all).len() as i64
        + all.len() as i64
        + 2 * route_set(&all).len() as i64
        + 2 * count_conflicting_sources(&all)
        + 3 * high_risk_role_swaps(source, candidates) as i64
}

pub(crate) fn high_risk_role_swaps(source: &[Triad], candidates: &[Triad]) -> usize {
    let source_keys: HashSet<_> = source.iter().map(full_key).collect();
    candidates
        .iter()
        .filter(|candidate| {
            let swapped = (
                norm(&candidate.object_role),
                norm(&candidate.object),
                norm(&candidate.relation),
                norm(&candidate.subject_role),
                norm(&candidate.subject),
            );
            source_keys.contains(&swapped)
        })
        .count()
}

pub(crate) fn limit_warnings(
    source: &[Triad],
    candidates: &[Triad],
    packet: &Packet,
) -> Vec<String> {
    let all: Vec<Triad> = source.iter().chain(candidates).cloned().collect();
    let counts = [
        ("entities", entity_set(&all).len(), 16, 32),
        ("roles", role_set(&all).len(), 8, 16),
        ("relations", relation_set(&all).len(), 16, 32),
        ("triads", all.len(), 32, 64),
        ("routes", route_set(&all).len(), 4, 8),
        ("evidence_refs", evidence_set(&all).len(), 32, 64),
        (
            "candidate_answers",
            if packet.candidate_answer.is_empty() {
                0
            } else {
                1
            },
            2,
            4,
        ),
    ];
    let mut out = vec![];
    for (name, value, target, hard) in counts {
        if value > hard {
            out.push(format!("{name} hard limit exceeded: {value}>{hard}"));
        } else if value > target {
            out.push(format!("{name} target limit exceeded: {value}>{target}"));
        }
    }
    out
}

pub(crate) fn evidence_gaps(source: &[Triad], candidates: &[Triad]) -> Vec<String> {
    source
        .iter()
        .chain(candidates)
        .filter(|triad| triad.evidence.trim().is_empty())
        .map(|triad| triad.id.clone())
        .collect()
}

pub(crate) fn low_confidence(source: &[Triad], candidates: &[Triad]) -> Vec<String> {
    source
        .iter()
        .chain(candidates)
        .filter(|triad| triad.confidence < 0.7)
        .map(|triad| triad.id.clone())
        .collect()
}

pub(crate) fn symbolic_conflicts(source: &[Triad], candidates: &[Triad]) -> Vec<String> {
    let source_structural: HashSet<_> = source.iter().map(structural_key).collect();
    let source_reversed: HashSet<_> = source.iter().map(reversed_structural_key).collect();
    let mut conflicts = evidence_conflicts(source);
    for candidate in candidates {
        if source_reversed.contains(&structural_key(candidate))
            && !symmetric_relation(&candidate.relation)
        {
            conflicts.push(format!(
                "{} reverses a non-symmetric source relation",
                candidate.id
            ));
        }
        if functional_relation(&candidate.relation) {
            for source_tri in source {
                if norm(&source_tri.relation) == norm(&candidate.relation)
                    && norm(&source_tri.subject) == norm(&candidate.subject)
                    && norm(&source_tri.object) != norm(&candidate.object)
                {
                    conflicts.push(format!(
                        "{} changes functional object for {}",
                        candidate.id, candidate.subject
                    ));
                }
            }
        }
        if !source_structural.contains(&structural_key(candidate)) {
            for source_tri in source {
                if norm(&source_tri.subject) == norm(&candidate.object)
                    && norm(&source_tri.object) == norm(&candidate.subject)
                    && norm(&source_tri.relation) == norm(&candidate.relation)
                    && norm(&source_tri.subject_role) == norm(&candidate.object_role)
                    && norm(&source_tri.object_role) == norm(&candidate.subject_role)
                {
                    conflicts.push(format!(
                        "{} swaps roles for relation {}",
                        candidate.id, candidate.relation
                    ));
                }
            }
        }
    }
    conflicts.sort();
    conflicts.dedup();
    conflicts
}

pub(crate) fn evidence_conflicts(triads: &[Triad]) -> Vec<String> {
    let mut by_evidence: HashMap<String, Vec<&Triad>> = HashMap::new();
    for triad in triads {
        if triad.evidence.trim().is_empty() {
            continue;
        }
        by_evidence
            .entry(norm(&triad.evidence))
            .or_default()
            .push(triad);
    }
    let mut conflicts = vec![];
    for (evidence, items) in by_evidence {
        let mut subject_slot: HashMap<(String, String, String), BTreeSet<String>> = HashMap::new();
        let mut object_slot: HashMap<(String, String, String), BTreeSet<String>> = HashMap::new();
        for triad in &items {
            subject_slot
                .entry((
                    norm(&triad.relation),
                    norm(&triad.object_role),
                    norm(&triad.object),
                ))
                .or_default()
                .insert(norm(&triad.subject));
            object_slot
                .entry((
                    norm(&triad.subject_role),
                    norm(&triad.subject),
                    norm(&triad.relation),
                ))
                .or_default()
                .insert(norm(&triad.object));
        }
        if subject_slot.values().any(|values| values.len() > 1)
            || object_slot.values().any(|values| values.len() > 1)
        {
            let ids = items
                .iter()
                .map(|triad| triad.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            conflicts.push(format!(
                "evidence {evidence} supports incompatible role fillers: {ids}"
            ));
        }
    }
    conflicts
}

pub(crate) fn symmetric_relation(relation: &str) -> bool {
    matches!(
        norm(relation).as_str(),
        "equals" | "same_as" | "related_to" | "matches" | "connected_to"
    )
}

pub(crate) fn functional_relation(relation: &str) -> bool {
    matches!(
        norm(relation).as_str(),
        "pays"
            | "pays_to"
            | "supplies"
            | "buys"
            | "owns"
            | "imports"
            | "exports"
            | "certifies"
            | "applies_for"
            | "manufactures"
            | "delivers_to"
            | "issued_by"
    )
}

pub(crate) fn centroid_summary(
    triads: &[Triad],
    memories: &BTreeMap<String, Vec<i32>>,
    axis: GroupAxis,
) -> Value {
    let mut out = serde_json::Map::new();
    for (name, memory) in memories {
        let members: Vec<&Triad> = triads
            .iter()
            .filter(|triad| match axis {
                GroupAxis::Group => {
                    group_name(triad, "source") == *name || group_name(triad, "candidate") == *name
                }
                GroupAxis::Route => {
                    route_name(triad, "source-route") == *name
                        || route_name(triad, "candidate-route") == *name
                }
            })
            .collect();
        let coherence = if members.is_empty() {
            0.0
        } else {
            members
                .iter()
                .map(|triad| cosine(memory, &triad_wave(triad)))
                .sum::<f64>()
                / members.len() as f64
        };
        out.insert(
            name.clone(),
            json!({
                "triads": members.len(),
                "norm": round4(norm2_i32(memory)),
                "self_coherence": round4(coherence)
            }),
        );
    }
    json!(out)
}

pub(crate) fn memory_summary(triads: &[Triad], memory: &[i32]) -> Value {
    let scores: Vec<f64> = triads
        .iter()
        .map(|triad| cosine(memory, &triad_wave(triad)))
        .collect();
    let avg = if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    };
    json!({
        "triads": triads.len(),
        "norm": round4(norm2_i32(memory)),
        "avg_triad_coherence": round4(avg)
    })
}

pub(crate) fn interference_matrix_for(
    source_memories: &BTreeMap<String, Vec<i32>>,
    candidate_memories: &BTreeMap<String, Vec<i32>>,
) -> Value {
    let mut out = serde_json::Map::new();
    for (candidate_name, candidate_memory) in candidate_memories {
        let mut row = serde_json::Map::new();
        for (source_name, source_memory) in source_memories {
            row.insert(
                source_name.clone(),
                json!(round4(cosine(source_memory, candidate_memory))),
            );
        }
        out.insert(candidate_name.clone(), json!(row));
    }
    json!(out)
}

pub(crate) fn best_memory_match(
    memories: &BTreeMap<String, Vec<i32>>,
    wave: &[i32],
) -> (String, f64) {
    let mut best_name = String::new();
    let mut best_score = -1.0;
    for (name, memory) in memories {
        let score = cosine(memory, wave);
        if score > best_score {
            best_score = score;
            best_name = name.clone();
        }
    }
    (best_name, best_score)
}

pub(crate) fn norm2_i32(a: &[i32]) -> f64 {
    a.iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt()
}

pub(crate) fn score_candidates(source: &[Triad], candidates: &[Triad]) -> Value {
    if source.is_empty() {
        return json!({"stable": [], "weak": [], "scores": {}, "weak_details": {}, "source_self_score": 0.0});
    }
    let memory = build_memory(source);
    let mut stable = vec![];
    let mut weak = vec![];
    let mut scores = serde_json::Map::new();
    let mut weak_details = serde_json::Map::new();
    for candidate in candidates {
        let candidate_wave = triad_wave(candidate);
        let score = cosine(&memory, &candidate_wave);
        scores.insert(candidate.id.clone(), json!(round4(score)));
        if score >= 0.28 {
            stable.push(candidate.id.clone());
        } else {
            weak.push(candidate.id.clone());
            let (nearest_id, nearest_score) = nearest_source(source, &candidate_wave);
            weak_details.insert(
                candidate.id.clone(),
                json!({
                    "score": round4(score),
                    "nearest_source": nearest_id,
                    "nearest_source_score": round4(nearest_score),
                    "why_weak": "candidate triad does not resonate with the composite source memory above the stability threshold",
                    "suggested_fix": "Check subject/object roles, relation name, group, route, and evidence for this candidate triad."
                }),
            );
        }
    }
    let source_scores: Vec<f64> = source
        .iter()
        .map(|triad| cosine(&memory, &triad_wave(triad)))
        .collect();
    let source_self_score = if source_scores.is_empty() {
        0.0
    } else {
        round4(source_scores.iter().sum::<f64>() / source_scores.len() as f64)
    };
    json!({"stable": stable, "weak": weak, "scores": scores, "weak_details": weak_details, "source_self_score": source_self_score})
}

pub(crate) fn nearest_source(source: &[Triad], candidate_wave: &[i32]) -> (String, f64) {
    let mut best_id = String::new();
    let mut best_score = -1.0;
    for triad in source {
        let score = cosine(&triad_wave(triad), candidate_wave);
        if score > best_score {
            best_score = score;
            best_id = triad.id.clone();
        }
    }
    (best_id, best_score)
}

pub(crate) fn baseline_summary(source: &[Triad], candidates: &[Triad]) -> Value {
    let source_tokens: HashSet<String> = source
        .iter()
        .flat_map(|t| [norm(&t.subject), norm(&t.relation), norm(&t.object)])
        .collect();
    let source_structural: HashSet<_> = source.iter().map(structural_key).collect();
    let source_reversed: HashSet<_> = source.iter().map(reversed_structural_key).collect();
    let mut exact = vec![];
    let mut reversed = vec![];
    let mut overlap = serde_json::Map::new();
    for candidate in candidates {
        if source_structural.contains(&structural_key(candidate)) {
            exact.push(candidate.id.clone());
        }
        if source_reversed.contains(&structural_key(candidate)) {
            reversed.push(candidate.id.clone());
        }
        let candidate_tokens: HashSet<String> = [
            norm(&candidate.subject),
            norm(&candidate.relation),
            norm(&candidate.object),
        ]
        .into_iter()
        .collect();
        let union = source_tokens.union(&candidate_tokens).count().max(1);
        let inter = source_tokens.intersection(&candidate_tokens).count();
        overlap.insert(
            candidate.id.clone(),
            json!(round4(inter as f64 / union as f64)),
        );
    }
    json!({"exact_matches": exact, "reversed_hits": reversed, "token_overlap": overlap})
}

pub(crate) fn build_explanation(report: &Report) -> Vec<String> {
    let mut notes = vec![];
    if report.verdict == "PASS" {
        notes.push("Candidate structure is coherent with source triads.".to_string());
    }
    if !report.conflicts.is_empty() {
        notes.push("Structural conflicts were detected.".to_string());
    }
    if report.route_coherence["weak"]
        .as_array()
        .is_some_and(|x| !x.is_empty())
    {
        notes.push("At least one candidate group has weak route coherence.".to_string());
    }
    if report.structural_map["foreign_pull"]
        .as_array()
        .is_some_and(|x| !x.is_empty())
    {
        notes.push("Route splice suspected: at least one candidate triad pulls toward a different source group.".to_string());
    }
    if report.wave_summary["weak"]
        .as_array()
        .is_some_and(|x| !x.is_empty())
    {
        notes.push("At least one candidate triad has weak composite-mode support.".to_string());
    }
    if !report.evidence_gaps.is_empty() {
        notes.push("Evidence is missing for one or more triads.".to_string());
    }
    if !report.limits.is_empty() {
        notes.push("Task exceeds target or hard limits and should be split.".to_string());
    }
    if report.canonicalization.conflict_count > 0 || report.canonicalization.watch_count > 0 {
        notes.push("Alias canonicalization needs review before structural acceptance.".to_string());
    }
    if notes.is_empty() {
        notes.push("No decisive structural signal was found.".to_string());
    }
    notes
}

pub(crate) fn build_repair_prompt(report: &Report) -> String {
    if report.verdict == "PASS" {
        return "No repair needed. Preserve the checked source/candidate bindings.".to_string();
    }
    let mut lines = vec![
        "Repair the candidate answer using only one coherent structural route.".to_string(),
        format!("NANDA verdict: {}.", report.verdict),
    ];
    if !report.conflicts.is_empty() {
        lines.push("Fix these conflicts:".to_string());
        for item in &report.conflicts {
            lines.push(format!("- {item}"));
        }
    }
    if !report.canonicalization.conflicts.is_empty() {
        lines.push("Fix alias conflicts before trusting the gate:".to_string());
        for item in &report.canonicalization.conflicts {
            lines.push(format!("- {item}"));
        }
    }
    if !report.canonicalization.warnings.is_empty() {
        lines.push("Review alias warnings before retrying:".to_string());
        for item in &report.canonicalization.warnings {
            lines.push(format!("- {item}"));
        }
    }
    if let Some(weak) = report.route_coherence["weak"].as_array() {
        if !weak.is_empty() {
            lines.push(
                "Do not splice triads from different source groups into one candidate group."
                    .to_string(),
            );
            for group in weak {
                if let Some(group) = group.as_str() {
                    let best = report.route_coherence["best_source_group"][group]
                        .as_str()
                        .unwrap_or("");
                    let score = &report.route_coherence["scores"][group];
                    lines.push(format!(
                        "- candidate group {group} best matches {best} only weakly: {score}"
                    ));
                }
            }
        }
    }
    if let Some(pulls) = report.structural_map["foreign_pull"].as_array() {
        if !pulls.is_empty() {
            lines.push("Route splice suspected from foreign_pull:".to_string());
            for pull in pulls {
                lines.push(format!(
                    "- {} in {} pulls from {} toward {}; repair: {}",
                    pull["candidate_triad"].as_str().unwrap_or(""),
                    pull["candidate_group"].as_str().unwrap_or(""),
                    pull["dominant_source_group"].as_str().unwrap_or(""),
                    pull["triad_best_source_group"].as_str().unwrap_or(""),
                    pull["repair"]
                        .as_str()
                        .unwrap_or("split or repair this candidate triad")
                ));
            }
        }
    }
    if !report.weak_triads.is_empty() {
        lines.push("Recheck or remove weak candidate triads:".to_string());
        for item in &report.weak_triads {
            if let Some(detail) = report.wave_summary["weak_details"].get(item) {
                lines.push(format!(
                    "- {item}: score={}, nearest_source={}, why={}",
                    detail["score"],
                    detail["nearest_source"].as_str().unwrap_or(""),
                    detail["why_weak"].as_str().unwrap_or("")
                ));
            } else {
                lines.push(format!("- {item}"));
            }
        }
    }
    if !report.evidence_gaps.is_empty() {
        lines.push("Add evidence before finalizing:".to_string());
        for item in &report.evidence_gaps {
            lines.push(format!("- {item}"));
        }
    }
    if !report.limits.is_empty() {
        lines.push("Split the task before retrying:".to_string());
        for item in &report.limits {
            lines.push(format!("- {item}"));
        }
    }
    lines.join("\n")
}

pub(crate) fn verdict_code(verdict: &str) -> u8 {
    match verdict {
        "PASS" => EXIT_PASS,
        "VETO" => EXIT_VETO,
        "WATCH" => EXIT_WATCH,
        _ => EXIT_ERROR,
    }
}

pub(crate) fn pack_from_md(args: PackArgs) -> Result<u8> {
    let packet = packet_from_markdown(
        &args.input,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let output = serde_json::to_string_pretty(&packet)? + "\n";
    let out = args
        .out
        .unwrap_or_else(|| args.input.with_extension("nanda.json"));
    write_or_print(out, args.stdout, output)?;
    Ok(EXIT_PASS)
}

pub(crate) fn gate_md(args: GateMdArgs) -> Result<u8> {
    let packet = packet_from_markdown(
        &args.input,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let source = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let report = make_report(&packet, &source, &candidates)?;
    print_report(&report, &args.format)?;
    Ok(verdict_code(&report.verdict))
}

pub(crate) fn hierarchical_decision(
    global_report: &Report,
    global_map: &Value,
    branches: &[Value],
    truncated: usize,
) -> Value {
    let branch_count = branches.len();
    let local_pass = branches
        .iter()
        .filter(|branch| branch["verdict"].as_str() == Some("PASS"))
        .count();
    let local_watch = branches
        .iter()
        .filter(|branch| branch["verdict"].as_str() == Some("WATCH"))
        .count();
    let local_veto = branches
        .iter()
        .filter(|branch| branch["verdict"].as_str() == Some("VETO"))
        .count();
    let global_foreign_pull = global_map["foreign_pull"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let global_size_only = global_report.verdict == "WATCH"
        && !global_report.limits.is_empty()
        && global_report.conflicts.is_empty()
        && global_report.evidence_gaps.is_empty()
        && global_foreign_pull == 0;
    let all_local_pass = branch_count > 0 && local_pass == branch_count && truncated == 0;
    let (action, accepted, next) = if global_report.verdict == "VETO"
        || global_foreign_pull > 0
        || local_veto > 0
    {
        (
            "REPAIR_REQUIRED",
            false,
            "Repair global foreign pull, conflicts, or vetoed branches before accepting the structure.",
        )
    } else if all_local_pass && (global_size_only || global_report.verdict == "PASS") {
        (
            "STRUCTURALLY_ACCEPTED",
            true,
            "Use the local branch PASS results as the trusted acceptance surface.",
        )
    } else if local_watch > 0 || truncated > 0 || global_report.verdict == "WATCH" {
        (
            "SPLIT_REQUIRED",
            false,
            "Narrow unresolved WATCH branches or increase max branches before finalizing.",
        )
    } else {
        (
            "REVIEW_REQUIRED",
            false,
            "Review hierarchical gate output before trusting the structure.",
        )
    };
    json!({
        "action": action,
        "structurally_accepted": accepted,
        "global_verdict": global_report.verdict,
        "global_size_only": global_size_only,
        "branches": branch_count,
        "local_pass": local_pass,
        "local_watch": local_watch,
        "local_veto": local_veto,
        "global_foreign_pull": global_foreign_pull,
        "truncated_branches": truncated,
        "next": next
    })
}

pub(crate) fn resolve_suite_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

pub(crate) fn aliases_cmd(args: AliasesArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let out = json!({
        "mode": "canonical-aliases",
        "task_id": packet.task_id,
        "domain": packet.domain,
        "canonicalization": packet.canonicalization,
        "triads": packet.triads,
        "candidate_triads": packet.candidate_triads
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => aliases::print_text(&out),
        OutputFormat::Md => aliases::print_md(&out),
    }
    Ok(
        if out["canonicalization"]["conflict_count"]
            .as_u64()
            .unwrap_or(0)
            > 0
            || out["canonicalization"]["watch_count"].as_u64().unwrap_or(0) > 0
        {
            EXIT_WATCH
        } else {
            EXIT_PASS
        },
    )
}

pub(crate) fn index_cmd(args: IndexArgs) -> Result<u8> {
    if args.inputs.is_empty() {
        return Err(anyhow!(
            "nanda index requires at least one input packet or worksheet"
        ));
    }
    let mut triads = vec![];
    let mut negative_shortcuts = vec![];
    let mut positive_shortcuts = vec![];
    let mut resonance_memory = vec![];
    let mut continuation_memory = vec![];
    for input in &args.inputs {
        if let Some((negative, positive, resonance, continuation)) = load_feedback_lanes(input)? {
            negative_shortcuts.extend(negative);
            positive_shortcuts.extend(positive);
            resonance_memory.extend(resonance);
            continuation_memory.extend(continuation);
            continue;
        }
        let packet = load_packet_auto(
            input,
            &args.input_format,
            &args.task_id,
            &args.domain,
            &args.query,
            args.normalize_paths,
        )?;
        triads.extend(packet.triads);
        if args.include_candidates {
            triads.extend(packet.candidate_triads);
        }
        negative_shortcuts.extend(packet.negative_shortcuts);
        positive_shortcuts.extend(packet.positive_shortcuts);
        resonance_memory.extend(packet.resonance_memory);
        continuation_memory.extend(packet.continuation_memory);
    }
    let triads = dedup_triads(triads);
    let negative_shortcuts = merge_negative_shortcuts(negative_shortcuts);
    let positive_shortcuts = merge_positive_shortcuts(positive_shortcuts);
    let resonance_memory = merge_resonance_memory(resonance_memory);
    let continuation_memory = merge_continuation_memory(continuation_memory);
    let packet = json!({
        "task_id": args.task_id,
        "domain": args.domain,
        "query": args.query,
        "triads": triads,
        "candidate_triads": [],
        "candidate_answer": "",
        "negative_shortcuts": negative_shortcuts,
        "positive_shortcuts": positive_shortcuts,
        "resonance_memory": resonance_memory,
        "continuation_memory": continuation_memory,
        "index": {
            "core_version": CORE_VERSION,
            "wave_dim": WAVE_DIM,
            "source_files": args.inputs.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()
        }
    });
    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(&args.out, serde_json::to_string_pretty(&packet)? + "\n")?;
    println!("{}", args.out.display());
    Ok(EXIT_PASS)
}

pub(crate) fn extract_cmd(args: ExtractArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.input)
        .with_context(|| format!("read {}", args.input.display()))?;
    let packet = extract_packet_from_text(&text, &args.task_id, &args.domain, &args.query);
    let output = serde_json::to_string_pretty(&packet)? + "\n";
    if args.stdout {
        print!("{output}");
    } else {
        let out = args
            .out
            .unwrap_or_else(|| args.input.with_extension("nanda.json"));
        if let Some(parent) = out.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&out, output)?;
        println!("{}", out.display());
    }
    Ok(EXIT_PASS)
}

pub(crate) fn support_terms_from_items(items: &[Value]) -> Vec<String> {
    let mut terms = BTreeSet::new();
    for item in items.iter().take(3) {
        for key in ["subject", "relation", "object", "route", "group"] {
            if let Some(value) = item[key].as_str() {
                terms.extend(normalized_shortcut_terms(&[value.to_string()]));
            }
        }
    }
    terms.into_iter().take(24).collect()
}

pub(crate) fn extract_packet_from_text(
    text: &str,
    task_id: &str,
    domain: &str,
    query: &str,
) -> Packet {
    let mut triads = vec![];
    let mut candidates = vec![];
    let mut target_candidates = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') && !trimmed.starts_with("##") {
            continue;
        }
        let lower = norm(trimmed);
        if lower.starts_with("## candidate") || lower.starts_with("[candidate") {
            target_candidates = true;
            continue;
        }
        if lower.starts_with("## triads")
            || lower.starts_with("[triads")
            || lower.starts_with("## memory")
        {
            target_candidates = false;
            continue;
        }
        if let Some(mut triad) = parse_arrow_triad(trimmed) {
            let prefix = if target_candidates { "q" } else { "m" };
            let idx = if target_candidates {
                candidates.len() + 1
            } else {
                triads.len() + 1
            };
            if triad.id.is_empty() {
                triad.id = format!("{prefix}{idx}");
            }
            if target_candidates {
                candidates.push(triad);
            } else {
                triads.push(triad);
            }
        }
    }
    Packet {
        task_id: task_id.to_string(),
        domain: domain.to_string(),
        query: query.to_string(),
        triads,
        candidate_triads: candidates,
        candidate_answer: String::new(),
        aliases: vec![],
        canonicalization: CanonicalizationReport::default(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
        resonance_memory: vec![],
        continuation_memory: vec![],
    }
}

pub(crate) fn dedup_triads(triads: Vec<Triad>) -> Vec<Triad> {
    let mut seen = BTreeSet::new();
    let mut out = vec![];
    for triad in triads {
        let key = (
            norm(&triad.subject),
            norm(&triad.relation),
            norm(&triad.object),
            norm(&triad.subject_role),
            norm(&triad.object_role),
            norm(&triad.route),
            norm(&triad.group),
        );
        if seen.insert(key) {
            out.push(triad);
        }
    }
    out
}

pub(crate) fn source_weight(triad: &Triad) -> f64 {
    let text = norm(&format!(
        "{} {} {} {}",
        triad.evidence, triad.route, triad.group, triad.object_role
    ));
    let authority = if text.contains("archive_noise") || text.contains("noise") {
        0.35
    } else if text.contains("historical") || text.contains("archive") || text.contains("old") {
        0.65
    } else if text.contains("current")
        || text.contains("canon")
        || text.contains("canonical")
        || text.contains("source")
    {
        1.12
    } else if text.contains("latest")
        || text.contains("frontier")
        || text.contains("w-chain")
        || text.contains("w_")
        || text.contains("w-")
    {
        0.95
    } else {
        0.86
    };
    round4((triad.confidence.clamp(0.05, 1.0) * authority).clamp(0.05, 1.2))
}

pub(crate) fn no_focus_metadata(memory_size: usize) -> Value {
    json!({
        "enabled": false,
        "reason": "not_requested_by_internal_caller",
        "original_memory_size": memory_size,
        "focused_memory_size": memory_size
    })
}

pub(crate) fn field_interpretation(
    peaks: &[Value],
    margin: f64,
    lexical_baseline: &Value,
) -> Value {
    if peaks.is_empty() {
        return json!({
            "state": "NO_FIELD",
            "read_as": "No resonance field was produced."
        });
    }
    let top = &peaks[0];
    let second = peaks.get(1);
    let top_name = top["peak"].as_str().unwrap_or("");
    let second_name = second.and_then(|peak| peak["peak"].as_str()).unwrap_or("");
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let top_polarization = top["polarization"]["state"].as_str().unwrap_or("");
    let stability = if top_polarization == "REVERSED" {
        "polarity_reversed"
    } else if margin >= 0.055 && component_gap >= 0.12 {
        "stable"
    } else if margin < 0.04 {
        "contested"
    } else {
        "thin"
    };
    let top_center = &top["center"];
    let second_center = second.map(|peak| &peak["center"]);
    let centroid_drift = json!({
        "from_second_peak": second_name,
        "route": center_pair(second_center, top_center, "route"),
        "relation": center_pair(second_center, top_center, "relation"),
        "entity": center_pair(second_center, top_center, "entity"),
        "subject_role": center_pair(second_center, top_center, "subject_role"),
        "object_role": center_pair(second_center, top_center, "object_role")
    });
    let nearest_foreign_pull = top["anti_triads"]
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or(Value::Null);
    let lexical_trap = !lexical_peak.is_empty() && lexical_peak != top_name;
    json!({
        "state": stability,
        "top_peak": top_name,
        "second_peak": second_name,
        "margin": round4(margin),
        "component_gap": component_gap,
        "lexical_baseline_top": lexical_peak,
        "lexical_trap_detected": lexical_trap,
        "top_polarization": top_polarization,
        "centroid_drift": centroid_drift,
        "nearest_foreign_pull": nearest_foreign_pull,
        "read_as": if lexical_trap {
            "The structural field beats the lexical baseline; inspect support and anti-triads before final prose."
        } else if stability == "polarity_reversed" {
            "The top route has reversed role-direction polarity; do not use it as an answer route."
        } else if stability == "stable" {
            "The top route has a stable connected peak."
        } else {
            "The field is useful as retrieval context but is not a final answer skeleton."
        }
    })
}

pub(crate) fn center_pair(second_center: Option<&Value>, top_center: &Value, key: &str) -> Value {
    json!({
        "from": second_center
            .and_then(|center| center[key].as_str())
            .unwrap_or(""),
        "to": top_center[key].as_str().unwrap_or(""),
        "changed": second_center
            .and_then(|center| center[key].as_str())
            .unwrap_or("") != top_center[key].as_str().unwrap_or("")
    })
}

pub(crate) fn field_state_machine(
    peaks: &[Value],
    margin: f64,
    lexical_baseline: &Value,
    corpus: &Value,
    route_balanced_focus: &Value,
    coarse_to_fine: &Value,
) -> Value {
    if peaks.is_empty() {
        return json!({
            "state": "NO_FIELD",
            "safe_to_answer": false,
            "action": "NO_ANSWER",
            "blocking": ["no_peak"],
            "signals": {
                "margin": round4(margin),
                "component_gap": 0.0,
                "top_polarization": "",
                "corpus_verdict": corpus["verdict"].as_str().unwrap_or(""),
                "route_balanced": route_balanced_focus["enabled"].as_bool().unwrap_or(false),
                "coarse_to_fine": coarse_to_fine["state"].as_str().unwrap_or("")
            },
            "read_as": "No resonance field was produced."
        });
    }

    let top = &peaks[0];
    let second = peaks.get(1);
    let top_name = top["peak"].as_str().unwrap_or("");
    let lexical_peak = lexical_baseline["top_peak"].as_str().unwrap_or("");
    let top_polarization = top["polarization"]["state"].as_str().unwrap_or("");
    let top_component = top["propagation"]["component_score"]
        .as_f64()
        .unwrap_or(0.0);
    let second_component = second
        .and_then(|peak| peak["propagation"]["component_score"].as_f64())
        .unwrap_or(0.0);
    let component_gap = round4(top_component - second_component);
    let route_balanced = route_balanced_focus["enabled"].as_bool().unwrap_or(false);
    let ctf_state = coarse_to_fine["state"].as_str().unwrap_or("");
    let corpus_verdict = corpus["verdict"].as_str().unwrap_or("");
    let warnings = corpus["warnings"].as_array().cloned().unwrap_or_default();
    let warning_kinds = warnings
        .iter()
        .filter_map(|warning| warning["kind"].as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let noisy_warning_count = warning_kinds
        .iter()
        .filter(|kind| {
            matches!(
                kind.as_str(),
                "large_unbalanced_corpus"
                    | "route_imbalance"
                    | "hub_dominance"
                    | "duplicate_current"
            )
        })
        .count();
    let weak_text_query = warning_kinds.iter().any(|kind| kind == "weak_text_query");
    let corpus_noisy = corpus_verdict == "WATCH" && (noisy_warning_count > 0 || weak_text_query);
    let focused = margin >= 0.055 && component_gap >= 0.12 && ctf_state == "LOCALIZED";
    let lexical_trap = !lexical_peak.is_empty() && lexical_peak != top_name;

    let mut blocking: Vec<String> = vec![];
    let (state, safe_to_answer, action, read_as) = if top_polarization == "REVERSED" {
        blocking.push("polarity_reversed".to_string());
        (
            "FIELD_REVERSED",
            false,
            "STOP_REPAIR_POLARITY",
            "The top peak is role-direction reversed; do not read it as the answer route.",
        )
    } else if corpus_noisy && !route_balanced {
        blocking.extend(warning_kinds.iter().cloned());
        (
            "FIELD_NOISY",
            false,
            "FOCUS_CORPUS",
            "The corpus field is noisy; run dataset-doctor or route-balanced focus before trusting the peak.",
        )
    } else if margin < 0.04 {
        blocking.push("low_margin".to_string());
        (
            "FIELD_CONTESTED",
            false,
            "SPLIT_OR_QUERY",
            "The top peaks are too close; use the result as retrieval context and split or sharpen the query.",
        )
    } else if !focused {
        if component_gap < 0.12 {
            blocking.push("weak_component_gap".to_string());
        }
        if ctf_state != "LOCALIZED" {
            blocking.push("not_localized".to_string());
        }
        (
            "FIELD_THIN",
            false,
            "USE_AS_HINT",
            "The peak is plausible but not connected/localized enough to become an answer skeleton.",
        )
    } else if route_balanced {
        (
            "FIELD_ROUTE_BALANCED",
            true,
            "ANSWER_WITH_BALANCED_SUPPORT",
            "The peak is focused after route-balanced filtering; answer from support and mention the focused packet.",
        )
    } else if lexical_trap {
        (
            "FIELD_FOCUSED",
            true,
            "ANSWER_WITH_SUPPORT",
            "The structural field beats the lexical baseline and is focused enough to draft from support.",
        )
    } else {
        (
            "FIELD_SAFE",
            true,
            "ANSWER_WITH_SUPPORT",
            "The field is focused, localized, and not blocked by corpus or polarity warnings.",
        )
    };

    blocking.sort();
    blocking.dedup();

    json!({
        "state": state,
        "safe_to_answer": safe_to_answer,
        "action": action,
        "top_peak": top_name,
        "blocking": blocking,
        "signals": {
            "margin": round4(margin),
            "component_gap": component_gap,
            "top_polarization": top_polarization,
            "corpus_verdict": corpus_verdict,
            "corpus_warnings": warning_kinds,
            "route_balanced": route_balanced,
            "coarse_to_fine": ctf_state,
            "lexical_baseline_top": lexical_peak,
            "lexical_trap_detected": lexical_trap
        },
        "read_as": read_as
    })
}

pub(crate) fn lexical_baseline(
    scored: &[(f64, f64, f64, &Triad)],
    query: &[Triad],
    group_by: &PeakGroupBy,
) -> Value {
    let query_tokens = query_token_set(query);
    let mut by_group: BTreeMap<String, Vec<(f64, &Triad)>> = BTreeMap::new();
    for (_, _, _, triad) in scored {
        let key = match group_by {
            PeakGroupBy::Group => group_name(triad, "memory"),
            PeakGroupBy::Route => route_name(triad, "memory-route"),
        };
        by_group
            .entry(key)
            .or_default()
            .push((token_overlap(&query_tokens, triad), *triad));
    }
    let mut rows = vec![];
    for (key, mut items) in by_group {
        items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let max = items.first().map(|item| item.0).unwrap_or(0.0);
        let take = items.len().min(3);
        let avg_top3 = if take == 0 {
            0.0
        } else {
            items.iter().take(take).map(|item| item.0).sum::<f64>() / take as f64
        };
        rows.push(json!({
            "peak": key,
            "score": round4(max),
            "max_symbolic_overlap": round4(max),
            "avg_top3_symbolic_overlap": round4(avg_top3),
            "top_triads": items.iter().take(3).map(|(_, triad)| triad.id.clone()).collect::<Vec<_>>()
        }));
    }
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    json!({
        "top_peak": rows.first().and_then(|row| row["peak"].as_str()).unwrap_or(""),
        "scores": rows
    })
}

pub(crate) fn triad_tokens(triad: &Triad) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    for value in [
        &triad.subject,
        &triad.relation,
        &triad.object,
        &triad.subject_role,
        &triad.object_role,
        &triad.route,
        &triad.group,
    ] {
        for token in norm(value)
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|token| token.len() >= 2)
        {
            if token != "query" {
                tokens.insert(token.to_string());
            }
        }
    }
    tokens
}

pub(crate) fn token_overlap(query_tokens: &BTreeSet<String>, triad: &Triad) -> f64 {
    if query_tokens.is_empty() {
        return 0.0;
    }
    let triad_tokens = triad_tokens(triad);
    query_tokens.intersection(&triad_tokens).count() as f64 / query_tokens.len() as f64
}

pub(crate) fn chain_coherence(items: &[(f64, f64, f64, &Triad)]) -> f64 {
    if items.len() < 2 {
        return 0.0;
    }
    let mut links = 0usize;
    let mut possible = 0usize;
    for (_, _, _, left) in items {
        for (_, _, _, right) in items {
            if left.id == right.id {
                continue;
            }
            possible += 1;
            let left_subject = norm(&left.subject);
            let left_object = norm(&left.object);
            let right_subject = norm(&right.subject);
            let right_object = norm(&right.object);
            if (!left_object.is_empty() && left_object == right_subject)
                || (!left_subject.is_empty() && left_subject == right_object)
            {
                links += 1;
            }
        }
    }
    if possible == 0 {
        0.0
    } else {
        (links as f64 / possible as f64).min(1.0)
    }
}

pub(crate) fn connected_components(size: usize, links: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::<usize>::new(); size];
    for (left, right) in links {
        adjacency[*left].push(*right);
        adjacency[*right].push(*left);
    }
    let mut seen = vec![false; size];
    let mut components = vec![];
    for start in 0..size {
        if seen[start] {
            continue;
        }
        let mut stack = vec![start];
        let mut component = vec![];
        seen[start] = true;
        while let Some(idx) = stack.pop() {
            component.push(idx);
            for next in &adjacency[idx] {
                if !seen[*next] {
                    seen[*next] = true;
                    stack.push(*next);
                }
            }
        }
        components.push(component);
    }
    components
}

pub(crate) fn shared_endpoint(left: &Triad, right: &Triad) -> String {
    let left_values = [norm(&left.subject), norm(&left.object)];
    let right_values = [norm(&right.subject), norm(&right.object)];
    for left_value in &left_values {
        if left_value.is_empty() {
            continue;
        }
        for right_value in &right_values {
            if left_value == right_value {
                return left_value.clone();
            }
        }
    }
    String::new()
}

pub(crate) fn role_family(role: &str) -> String {
    let role = norm(role);
    if role.contains("payer") || role.contains("buyer") || role.contains("customer") {
        "payer".to_string()
    } else if role.contains("supplier") || role.contains("seller") || role.contains("factory") {
        "supplier".to_string()
    } else if role.contains("document") || role.contains("certificate") || role.contains("doc") {
        "document".to_string()
    } else if role.contains("route") || role.contains("path") {
        "route".to_string()
    } else if role.contains("owner") || role.contains("holder") {
        "owner".to_string()
    } else if role.contains("asset") || role.contains("goods") || role.contains("product") {
        "asset".to_string()
    } else if role.is_empty() {
        "role".to_string()
    } else {
        role
    }
}

pub(crate) fn relation_family(relation: &str) -> String {
    let relation = norm(relation);
    if relation.contains("pay") || relation.contains("fund") || relation.contains("owe") {
        "payment".to_string()
    } else if relation.contains("own") || relation.contains("hold") {
        "ownership".to_string()
    } else if relation.contains("supply")
        || relation.contains("deliver")
        || relation.contains("ship")
    {
        "flow".to_string()
    } else if relation.contains("require")
        || relation.contains("confirm")
        || relation.contains("cert")
    {
        "evidence".to_string()
    } else if relation.is_empty() {
        "relation".to_string()
    } else {
        relation
    }
}

pub(crate) fn polarization_summary(query: &[Triad], items: &[(f64, f64, f64, &Triad)]) -> Value {
    let query_polarities = query
        .iter()
        .map(triad_polarity)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let query_reversed = query
        .iter()
        .map(reversed_polarity)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if query_polarities.is_empty() || items.is_empty() {
        return json!({
            "state": "NO_QUERY_POLARITY",
            "aligned": 0,
            "reversed": 0,
            "dominant": ""
        });
    }
    let mut counts = BTreeMap::<String, usize>::new();
    let mut aligned = 0usize;
    let mut reversed = 0usize;
    for (_, _, _, triad) in items {
        let polarity = triad_polarity(triad);
        if query_polarities.contains(&polarity) {
            aligned += 1;
        }
        if query_reversed.contains(&polarity) && !query_polarities.contains(&polarity) {
            reversed += 1;
        }
        *counts.entry(polarity).or_default() += 1;
    }
    let dominant = counts
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(polarity, _)| polarity.clone())
        .unwrap_or_default();
    let state = if aligned > 0 && reversed == 0 {
        "ALIGNED"
    } else if reversed > aligned {
        "REVERSED"
    } else if aligned > 0 {
        "MIXED"
    } else {
        "UNALIGNED"
    };
    json!({
        "state": state,
        "aligned": aligned,
        "reversed": reversed,
        "dominant": dominant,
        "query": query_polarities.into_iter().collect::<Vec<_>>()
    })
}

pub(crate) fn polarization_penalty(polarization: &Value) -> f64 {
    match polarization["state"].as_str().unwrap_or("") {
        "REVERSED" => 0.18,
        "MIXED" => 0.04,
        "UNALIGNED" => 0.02,
        _ => 0.0,
    }
}

pub(crate) fn triad_feature_wave(triad: &Triad) -> Vec<i32> {
    let mut wave = partial_triad_feature_wave(triad);
    add_feature(&mut wave, "group", &triad.group);
    add_feature(&mut wave, "route", &triad.route);
    add_feature(&mut wave, "subject_role", &triad.subject_role);
    add_feature(&mut wave, "object_role", &triad.object_role);
    add_feature(&mut wave, "polarity", &triad_polarity(triad));
    wave
}

pub(crate) fn partial_triad_feature_wave(triad: &Triad) -> Vec<i32> {
    let mut wave = vec![0; WAVE_DIM];
    add_feature(&mut wave, "subject", &triad.subject);
    add_feature(&mut wave, "relation", &triad.relation);
    add_feature(&mut wave, "object", &triad.object);
    add_feature(&mut wave, "subject_role", &triad.subject_role);
    add_feature(&mut wave, "object_role", &triad.object_role);
    add_feature(&mut wave, "route", &triad.route);
    add_feature(&mut wave, "group", &triad.group);
    add_feature(&mut wave, "polarity", &triad_polarity(triad));
    wave
}

pub(crate) fn add_feature(wave: &mut [i32], slot: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    let feature = bind(
        &vector(&format!("slot:{}", norm(slot))),
        &vector(&format!("value:{}", norm(value))),
    );
    add_into(wave, &feature);
}

pub(crate) fn triad_term_set(triad: &Triad) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for value in [
        &triad.subject,
        &triad.relation,
        &triad.object,
        &triad.subject_role,
        &triad.object_role,
        &triad.route,
        &triad.group,
    ] {
        let normalized = norm(value);
        if !normalized.is_empty() && normalized != "subject" && normalized != "object" {
            terms.insert(normalized);
        }
    }
    terms
}

pub(crate) fn missing_edges(
    query_terms: &BTreeSet<String>,
    items: &[(f64, f64, f64, &Triad)],
) -> Vec<Value> {
    let mut peak_terms = BTreeSet::new();
    for (_, _, _, triad) in items {
        peak_terms.extend(triad_term_set(triad));
    }
    query_terms
        .difference(&peak_terms)
        .map(|term| {
            json!({
                "term": term,
                "suggested_fix": "Add or retrieve evidence that binds this query term to the peak route."
            })
        })
        .collect()
}

pub(crate) fn answer_projection(center: &Value, items: &[(f64, f64, f64, &Triad)]) -> Value {
    let top = items
        .iter()
        .take(5)
        .map(|(_, _, _, triad)| {
            format!(
                "{} -> {} -> {}",
                triad.subject, triad.relation, triad.object
            )
        })
        .collect::<Vec<_>>();
    json!({
        "dominant_route": center["route"],
        "dominant_group": center["group"],
        "dominant_relation": center["relation"],
        "read_as": "Use this peak as a candidate structural route, not as proof by itself.",
        "top_structure": top
    })
}

pub(crate) fn triad_json(triad: &Triad) -> Value {
    json!({
        "id": triad.id,
        "subject": triad.subject,
        "relation": triad.relation,
        "object": triad.object,
        "confidence": triad.confidence,
        "subject_role": triad.subject_role,
        "object_role": triad.object_role,
        "route": triad.route,
        "group": triad.group,
        "evidence": triad.evidence
    })
}

pub(crate) fn invariant_scan(source: &[Triad], candidates: &[Triad]) -> Value {
    let mut by_key: BTreeMap<(String, String, String), Vec<&Triad>> = BTreeMap::new();
    let map = structural_map(source, candidates);
    let mut candidate_scope: BTreeMap<String, String> = BTreeMap::new();
    if let Some(group_links) = map["group_links"].as_object() {
        for (candidate_group, link) in group_links {
            if let Some(source_group) = link["dominant_source_group"].as_str() {
                if !source_group.is_empty() {
                    candidate_scope.insert(candidate_group.clone(), source_group.to_string());
                }
            }
        }
    }
    for triad in source {
        if !is_invariant_candidate(triad) {
            continue;
        }
        by_key
            .entry((
                group_name(triad, "source"),
                norm(&triad.subject),
                norm(&triad.relation),
            ))
            .or_default()
            .push(triad);
    }
    for triad in candidates {
        if !is_invariant_candidate(triad) {
            continue;
        }
        let candidate_group = group_name(triad, "candidate");
        let scope = candidate_scope
            .get(&candidate_group)
            .cloned()
            .unwrap_or(candidate_group);
        by_key
            .entry((scope, norm(&triad.subject), norm(&triad.relation)))
            .or_default()
            .push(triad);
    }
    let mut checked = vec![];
    let mut violations = vec![];
    for ((group, subject, relation), triads) in by_key {
        if triads.len() < 2 {
            continue;
        }
        let values: BTreeSet<String> = triads.iter().map(|triad| norm(&triad.object)).collect();
        checked.push(json!({
            "selector": {
                "group": group,
                "subject": subject,
                "relation": relation
            },
            "values": values.iter().cloned().collect::<Vec<_>>(),
            "count": triads.len()
        }));
        if values.len() > 1 {
            violations.push(json!({
                "kind": "same_value",
                "selector": {
                    "group": group,
                    "subject": subject,
                    "relation": relation
                },
                "values": values.iter().cloned().collect::<Vec<_>>(),
                "evidence": triads.iter().map(|triad| triad.evidence.clone()).collect::<Vec<_>>(),
                "triads": triads.iter().map(|triad| triad.id.clone()).collect::<Vec<_>>(),
                "message": "same group+subject+relation has multiple object values"
            }));
        }
    }
    json!({"checked": checked, "violations": violations})
}

pub(crate) fn is_invariant_candidate(triad: &Triad) -> bool {
    let relation = norm(&triad.relation);
    let object_role = norm(&triad.object_role);
    matches!(
        relation.as_str(),
        "default_value"
            | "value"
            | "type"
            | "schema"
            | "unit"
            | "formula"
            | "normalizes_to"
            | "version"
            | "rate"
            | "currency"
            | "owner"
            | "required"
    ) || matches!(
        object_role.as_str(),
        "value" | "type" | "schema" | "unit" | "formula" | "version" | "rate" | "currency"
    )
}

pub(crate) fn stop_reasons(
    report: &Report,
    map: &Value,
    invariants: &Value,
    stop_on: &[String],
) -> Vec<String> {
    let mut reasons = vec![];
    if stop_on.iter().any(|item| item == "foreign_pull")
        && map["foreign_pull"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    {
        reasons.push("foreign_pull".to_string());
    }
    if stop_on.iter().any(|item| item == "invariant_violation")
        && invariants["violations"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    {
        reasons.push("invariant_violation".to_string());
    }
    if stop_on.iter().any(|item| item == "size") && !report.limits.is_empty() {
        reasons.push("size".to_string());
    }
    reasons
}

pub(crate) fn worst_status(a: u8, b: u8) -> u8 {
    if a == EXIT_VETO || b == EXIT_VETO {
        EXIT_VETO
    } else if a == EXIT_WATCH || b == EXIT_WATCH {
        EXIT_WATCH
    } else {
        EXIT_PASS
    }
}

pub(crate) fn self_check() -> Result<u8> {
    let packet = example_packet(false);
    let report = make_report(
        &packet,
        &normalize_ids(packet.triads.clone(), "t"),
        &normalize_ids(packet.candidate_triads.clone(), "c"),
    )?;
    if report.verdict != "PASS" {
        println!("verdict: VETO");
        return Ok(EXIT_VETO);
    }
    println!("verdict: PASS");
    Ok(EXIT_PASS)
}

pub(crate) fn verdict_for(packet: &Packet) -> Result<String> {
    Ok(make_report(
        packet,
        &normalize_ids(packet.triads.clone(), "t"),
        &normalize_ids(packet.candidate_triads.clone(), "c"),
    )?
    .verdict)
}

pub(crate) fn exact_baseline_accepts(packet: &Packet) -> bool {
    let source: HashSet<_> = packet.triads.iter().map(structural_key).collect();
    packet
        .candidate_triads
        .iter()
        .any(|candidate| source.contains(&structural_key(candidate)))
}

pub(crate) fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

pub(crate) fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub(crate) fn packet_from_markdown(
    path: &Path,
    task_id: &str,
    domain: &str,
    query: &str,
    normalize_paths: bool,
) -> Result<Packet> {
    let text = fs::read_to_string(path)?;
    let (triads, candidate_triads) = parse_markdown_tables(&text, normalize_paths);
    Ok(Packet {
        task_id: task_id.to_string(),
        domain: domain.to_string(),
        query: query.to_string(),
        triads,
        candidate_triads,
        candidate_answer: String::new(),
        aliases: vec![],
        canonicalization: CanonicalizationReport::default(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
        resonance_memory: vec![],
        continuation_memory: vec![],
    })
}

pub(crate) fn normalize_header(value: &str) -> String {
    let key = value
        .trim()
        .to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");
    match key.trim_matches('_') {
        "subj" => "subject".to_string(),
        "rel" => "relation".to_string(),
        "obj" => "object".to_string(),
        "subj_role" => "subject_role".to_string(),
        "obj_role" => "object_role".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn normalize_code_entity(value: &str) -> String {
    let mut text = value.trim().trim_start_matches("./").to_string();
    if let Some(idx) = text.find("/projects/") {
        let rest = &text[idx + "/projects/".len()..];
        if let Some(pos) = rest.find('/') {
            text = rest[pos + 1..].to_string();
        }
    }
    if text.starts_with("src/bin/") && text.ends_with(".rs") {
        return format!(
            "bin::{}",
            Path::new(&text)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        );
    }
    if text.starts_with("src/") && text.ends_with(".rs") {
        let mut parts: Vec<&str> = text
            .trim_start_matches("src/")
            .trim_end_matches(".rs")
            .split('/')
            .collect();
        if parts.last() == Some(&"mod") {
            parts.pop();
        }
        return parts.join("::");
    }
    if text.ends_with(".rs") && text.contains('/') {
        return text
            .trim_end_matches(".rs")
            .split('/')
            .collect::<Vec<_>>()
            .join("::");
    }
    text
}

pub(crate) fn table(title: &str, rows: &[Triad]) -> String {
    let headers = [
        "id",
        "subject",
        "relation",
        "object",
        "evidence",
        "confidence",
        "subject_role",
        "object_role",
        "route",
        "group",
    ];
    let mut out = format!(
        "## {title}\n\n| {} |\n|{}|\n",
        headers.join(" | "),
        vec!["----"; headers.len()].join("|")
    );
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.3} | {} | {} | {} | {} |\n",
            row.id,
            row.subject,
            row.relation,
            row.object,
            row.evidence,
            row.confidence,
            row.subject_role,
            row.object_role,
            row.route,
            row.group
        ));
    }
    out.push('\n');
    out
}

pub(crate) fn template_text(
    task_id: &str,
    domain: &str,
    query: &str,
    kind: &TemplateKind,
) -> String {
    let rows = match kind {
        TemplateKind::Code => vec![
            (
                "t1",
                "source module",
                "installs_to",
                "runtime module",
                "README.md:1",
                "source",
                "runtime",
                "deploy",
                "source-runtime",
            ),
            (
                "t2",
                "runtime module",
                "exposes",
                "CLI command",
                "README.md:2",
                "runtime",
                "cli",
                "deploy",
                "source-runtime",
            ),
            (
                "t3",
                "CLI command",
                "calls",
                "checker core",
                "README.md:3",
                "cli",
                "core",
                "execution",
                "source-runtime",
            ),
        ],
        TemplateKind::Skill => vec![
            (
                "t1",
                "source skill",
                "syncs_to",
                "runtime skill",
                "scripts/install-local.sh:10",
                "source",
                "runtime",
                "deploy",
                "skill-flow",
            ),
            (
                "t2",
                "runtime skill",
                "provides",
                "trigger rule",
                "SKILL.md:2",
                "runtime",
                "trigger",
                "agent",
                "skill-flow",
            ),
            (
                "t3",
                "CLI command",
                "returns",
                "gate verdict",
                "scripts/nanda-check:1",
                "cli",
                "verdict",
                "execution",
                "skill-flow",
            ),
        ],
        TemplateKind::Project => vec![
            (
                "t1",
                "repository",
                "contains",
                "source files",
                "README.md:1",
                "repo",
                "source",
                "project",
                "project-flow",
            ),
            (
                "t2",
                "repository",
                "documents",
                "architecture",
                "ARCHITECTURE.md:1",
                "repo",
                "docs",
                "project",
                "project-flow",
            ),
            (
                "t3",
                "test suite",
                "validates",
                "runtime behavior",
                "scripts/test-local.sh:1",
                "tests",
                "runtime",
                "validation",
                "project-flow",
            ),
        ],
        TemplateKind::Generic => vec![("t1", "", "", "", "", "", "", "", "")],
    };
    let mut out = format!(
        "# NANDA Triad Worksheet\n\ntask_id: {task_id}\ndomain: {domain}\nquery: {query}\n\n"
    );
    out.push_str("## triads\n\n| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |\n|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|\n");
    for row in &rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | 0.9 | {} | {} | {} | {} |\n",
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8
        ));
    }
    out.push_str("\n## candidate_triads\n\n| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |\n|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|\n");
    for row in rows {
        out.push_str(&format!(
            "| c{} | {} | {} | {} | candidate_answer | 0.9 | {} | {} | {} | candidate-{} |\n",
            &row.0[1..],
            row.1,
            row.2,
            row.3,
            row.5,
            row.6,
            row.7,
            row.8
        ));
    }
    out.push_str("\n## notes\n\n- Fill `triads` from source evidence.\n- Fill `candidate_triads` from the answer being checked.\n- Keep one coherent `group` per route, case, or local structure.\n");
    out
}

pub(crate) fn example_packet(swapped: bool) -> Packet {
    let triads = vec![
        triad(
            "t1", "supplier", "supplies", "buyer", "doc:1", "seller", "buyer", "route-a", "deal-a",
        ),
        triad(
            "t2", "buyer", "pays_to", "supplier", "doc:2", "payer", "payee", "route-a", "deal-a",
        ),
    ];
    let candidate_triads = if swapped {
        vec![triad(
            "c1", "buyer", "supplies", "supplier", "answer", "buyer", "seller", "route-a", "deal-a",
        )]
    } else {
        vec![
            triad(
                "c1", "supplier", "supplies", "buyer", "answer", "seller", "buyer", "route-a",
                "deal-a",
            ),
            triad(
                "c2", "buyer", "pays_to", "supplier", "answer", "payer", "payee", "route-a",
                "deal-a",
            ),
        ]
    };
    Packet {
        task_id: "example".to_string(),
        domain: "general".to_string(),
        query: String::new(),
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

pub(crate) fn synthetic_packet(idx: usize, kind: &str) -> Packet {
    let a = format!("supplier-{idx}");
    let b = format!("buyer-{idx}");
    let c = format!("carrier-{idx}");
    let d = format!("warehouse-{idx}");
    let triads = vec![
        triad(
            "t1",
            &a,
            "supplies",
            &b,
            "doc:invoice",
            "supplier",
            "buyer",
            "trade",
            "trade",
        ),
        triad(
            "t2",
            &b,
            "pays_to",
            &a,
            "doc:payment",
            "payer",
            "payee",
            "payment",
            "trade",
        ),
        triad(
            "t3",
            &a,
            "delivers_to",
            &c,
            "doc:logistics",
            "shipper",
            "carrier",
            "delivery",
            "logistics",
        ),
        triad(
            "t4",
            &c,
            "delivers_to",
            &d,
            "doc:warehouse",
            "carrier",
            "warehouse",
            "delivery",
            "logistics",
        ),
    ];
    let candidate_triads = match kind {
        "swap" => vec![triad(
            "c1",
            &b,
            "supplies",
            &a,
            "answer",
            "buyer",
            "supplier",
            "trade",
            "candidate",
        )],
        "splice" => vec![
            triad(
                "c1",
                &a,
                "supplies",
                &b,
                "answer",
                "supplier",
                "buyer",
                "trade",
                "candidate",
            ),
            triad(
                "c2",
                &c,
                "delivers_to",
                &d,
                "answer",
                "carrier",
                "warehouse",
                "delivery",
                "candidate",
            ),
        ],
        _ => vec![
            triad(
                "c1", &a, "supplies", &b, "answer", "supplier", "buyer", "trade", "trade",
            ),
            triad(
                "c2", &b, "pays_to", &a, "answer", "payer", "payee", "payment", "trade",
            ),
        ],
    };
    Packet {
        task_id: format!("synthetic-{idx}"),
        domain: "general".to_string(),
        query: String::new(),
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn triad(
    id: &str,
    subject: &str,
    relation: &str,
    object: &str,
    evidence: &str,
    subject_role: &str,
    object_role: &str,
    route: &str,
    group: &str,
) -> Triad {
    Triad {
        id: id.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        evidence: evidence.to_string(),
        confidence: 0.9,
        subject_role: subject_role.to_string(),
        object_role: object_role.to_string(),
        route: route.to_string(),
        group: group.to_string(),
    }
}

#[derive(Clone, ValueEnum)]
pub(crate) enum OutputFormat {
    Json,
    Text,
    Md,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum SplitOutputFormat {
    Json,
    Md,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum InputFormat {
    Auto,
    Json,
    Md,
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
pub(crate) enum TemplateKind {
    Generic,
    Code,
    Skill,
    Project,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum SplitBy {
    Group,
    Route,
    LinkedGroup,
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
pub(crate) enum PeakGroupBy {
    Group,
    Route,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum ServeFormat {
    Jsonl,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum FeedbackDecision {
    Accept,
    Reject,
    Watch,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum PatternBankMode {
    Build,
    Inspect,
    Apply,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum LlmwaveLensKind {
    Pattern,
    Polarity,
    Cleanup,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Triad {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) subject: String,
    #[serde(default)]
    pub(crate) relation: String,
    #[serde(default)]
    pub(crate) object: String,
    #[serde(default)]
    pub(crate) evidence: String,
    #[serde(default = "default_confidence")]
    pub(crate) confidence: f64,
    #[serde(default = "default_subject_role")]
    pub(crate) subject_role: String,
    #[serde(default = "default_object_role")]
    pub(crate) object_role: String,
    #[serde(default)]
    pub(crate) route: String,
    #[serde(default)]
    pub(crate) group: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Packet {
    #[serde(default = "default_task_id")]
    pub(crate) task_id: String,
    #[serde(default = "default_domain")]
    pub(crate) domain: String,
    #[serde(default)]
    pub(crate) query: String,
    #[serde(default)]
    pub(crate) triads: Vec<Triad>,
    #[serde(default)]
    pub(crate) candidate_triads: Vec<Triad>,
    #[serde(default)]
    pub(crate) candidate_answer: String,
    #[serde(default)]
    pub(crate) aliases: Vec<AliasRule>,
    #[serde(default, skip_serializing_if = "CanonicalizationReport::is_empty")]
    pub(crate) canonicalization: CanonicalizationReport,
    #[serde(default)]
    pub(crate) negative_shortcuts: Vec<NegativeShortcut>,
    #[serde(default)]
    pub(crate) positive_shortcuts: Vec<PositiveShortcut>,
    #[serde(default)]
    pub(crate) resonance_memory: Vec<ResonanceMemory>,
    #[serde(default)]
    pub(crate) continuation_memory: Vec<ContinuationMemory>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct NegativeShortcut {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) suppress_peak: String,
    #[serde(default)]
    pub(crate) suppress_route: String,
    #[serde(default)]
    pub(crate) suppress_group: String,
    #[serde(default)]
    pub(crate) prefer_peak: String,
    #[serde(default)]
    pub(crate) prefer_route: String,
    #[serde(default)]
    pub(crate) prefer_group: String,
    #[serde(default = "default_negative_penalty")]
    pub(crate) penalty: f64,
    #[serde(default)]
    pub(crate) terms: Vec<String>,
    #[serde(default)]
    pub(crate) support_terms: Vec<String>,
    #[serde(default)]
    pub(crate) reason: String,
    #[serde(default)]
    pub(crate) source_feedback: String,
    #[serde(default = "default_one_usize")]
    pub(crate) observations: usize,
    #[serde(default)]
    pub(crate) rejected_count: usize,
    #[serde(default)]
    pub(crate) accepted_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PositiveShortcut {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) reinforce_peak: String,
    #[serde(default)]
    pub(crate) reinforce_route: String,
    #[serde(default)]
    pub(crate) reinforce_group: String,
    #[serde(default = "default_positive_boost")]
    pub(crate) boost: f64,
    #[serde(default)]
    pub(crate) terms: Vec<String>,
    #[serde(default)]
    pub(crate) support_terms: Vec<String>,
    #[serde(default)]
    pub(crate) reason: String,
    #[serde(default)]
    pub(crate) source_feedback: String,
    #[serde(default = "default_one_usize")]
    pub(crate) observations: usize,
    #[serde(default)]
    pub(crate) accepted_count: usize,
    #[serde(default)]
    pub(crate) rejected_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ResonanceMemory {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) decision: String,
    #[serde(default)]
    pub(crate) peak: String,
    #[serde(default)]
    pub(crate) route: String,
    #[serde(default)]
    pub(crate) relation: String,
    #[serde(default)]
    pub(crate) role_mode: String,
    #[serde(default)]
    pub(crate) waw_status: String,
    #[serde(default)]
    pub(crate) field_state: String,
    #[serde(default)]
    pub(crate) phase_state: String,
    #[serde(default)]
    pub(crate) standing_state: String,
    #[serde(default)]
    pub(crate) energy_state: String,
    #[serde(default)]
    pub(crate) boundary_state: String,
    #[serde(default)]
    pub(crate) temporal_phase: String,
    #[serde(default)]
    pub(crate) support_terms: Vec<String>,
    #[serde(default)]
    pub(crate) anti_terms: Vec<String>,
    #[serde(default)]
    pub(crate) source_feedback: String,
    #[serde(default = "default_one_usize")]
    pub(crate) observations: usize,
    #[serde(default)]
    pub(crate) accepted_count: usize,
    #[serde(default)]
    pub(crate) rejected_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ContinuationMemory {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) decision: String,
    #[serde(default)]
    pub(crate) pattern_id: String,
    #[serde(default)]
    pub(crate) subject: String,
    #[serde(default)]
    pub(crate) relation: String,
    #[serde(default)]
    pub(crate) object: String,
    #[serde(default)]
    pub(crate) route: String,
    #[serde(default)]
    pub(crate) group: String,
    #[serde(default)]
    pub(crate) peak: String,
    #[serde(default = "default_positive_boost")]
    pub(crate) boost: f64,
    #[serde(default = "default_negative_penalty")]
    pub(crate) penalty: f64,
    #[serde(default)]
    pub(crate) terms: Vec<String>,
    #[serde(default)]
    pub(crate) support_terms: Vec<String>,
    #[serde(default)]
    pub(crate) reason: String,
    #[serde(default)]
    pub(crate) source_feedback: String,
    #[serde(default = "default_one_usize")]
    pub(crate) observations: usize,
    #[serde(default)]
    pub(crate) accepted_count: usize,
    #[serde(default)]
    pub(crate) rejected_count: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EvalSuite {
    #[serde(default)]
    pub(crate) cases: Vec<EvalSuiteCase>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EvalSuiteCase {
    pub(crate) path: PathBuf,
    pub(crate) expected_peak: String,
    pub(crate) expected_state: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WawSuite {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) cases: Vec<WawSuiteCase>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ProbeSuite {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) cases: Vec<ProbeSuiteCase>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ProbeSuiteCase {
    #[serde(default)]
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    #[serde(default)]
    pub(crate) negative: Vec<PathBuf>,
    #[serde(default)]
    pub(crate) expected_decision: String,
    #[serde(default)]
    pub(crate) expected_plain_peak: String,
    #[serde(default)]
    pub(crate) expected_negative_peak: String,
    #[serde(default)]
    pub(crate) group_by: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ProofSuite {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) cases: Vec<ProofSuiteCase>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ProofSuiteCase {
    #[serde(default)]
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    #[serde(default)]
    pub(crate) query_file: Option<PathBuf>,
    #[serde(default)]
    pub(crate) query: String,
    #[serde(default)]
    pub(crate) expected_proof_state: String,
    #[serde(default)]
    pub(crate) expected_top_peak: String,
    #[serde(default)]
    pub(crate) expected_field_state: String,
    #[serde(default)]
    pub(crate) expected_reason_codes: Vec<String>,
    #[serde(default)]
    pub(crate) group_by: String,
    #[serde(default)]
    pub(crate) max_triads: Option<usize>,
    #[serde(default)]
    pub(crate) route_cap: Option<usize>,
    #[serde(default)]
    pub(crate) route_triad_cap: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WawSuiteCase {
    #[serde(default)]
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    pub(crate) expected_peak: String,
    pub(crate) expected_state: String,
    #[serde(default)]
    pub(crate) expected_lexical_peak: String,
    #[serde(default = "default_true")]
    pub(crate) require_lexical_trap: bool,
    #[serde(default = "default_true")]
    pub(crate) require_safe_to_answer: bool,
}

#[derive(Serialize)]
pub(crate) struct Report {
    pub(crate) verdict: String,
    pub(crate) engine: String,
    pub(crate) core_version: String,
    pub(crate) wave_dim: usize,
    pub(crate) task_id: String,
    pub(crate) domain: String,
    pub(crate) complexity_score: i64,
    pub(crate) mandatory_gate: bool,
    pub(crate) limits: Vec<String>,
    pub(crate) stable_triads: Vec<String>,
    pub(crate) weak_triads: Vec<String>,
    pub(crate) conflicts: Vec<String>,
    pub(crate) evidence_gaps: Vec<String>,
    pub(crate) canonicalization: CanonicalizationReport,
    pub(crate) baseline_summary: Value,
    pub(crate) wave_summary: Value,
    pub(crate) route_coherence: Value,
    pub(crate) structural_map: Value,
    pub(crate) explanation: Vec<String>,
    pub(crate) repair_prompt: String,
    pub(crate) trace_path: String,
}

pub(crate) enum GroupAxis {
    Group,
    Route,
}

pub(crate) struct SplitItem {
    pub(crate) key: String,
    pub(crate) triads: Vec<Triad>,
    pub(crate) candidates: Vec<Triad>,
}

pub(crate) struct SplitResult {
    pub(crate) items: Vec<SplitItem>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn parse_attrs(value: &str) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    for item in value.split_whitespace() {
        if let Some((key, value)) = item.split_once('=') {
            attrs.insert(norm(key), value.trim_matches('"').to_string());
        }
    }
    attrs
}

pub(crate) struct FocusedMemory {
    pub(crate) memory: Vec<Triad>,
    pub(crate) metadata: Value,
}

pub(crate) fn count_by<I>(values: I) -> BTreeMap<String, usize>
where
    I: IntoIterator<Item = String>,
{
    let mut counts = BTreeMap::new();
    for value in values {
        if value.is_empty() {
            continue;
        }
        *counts.entry(value).or_default() += 1;
    }
    counts
}

pub(crate) fn weighted_center<I>(values: I) -> String
where
    I: IntoIterator<Item = (f64, String)>,
{
    let mut weights: BTreeMap<String, f64> = BTreeMap::new();
    for (score, value) in values {
        if value.is_empty() {
            continue;
        }
        *weights.entry(value).or_default() += score.max(0.0);
    }
    weights
        .into_iter()
        .max_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.cmp(&a.0))
        })
        .map(|(value, _)| value)
        .unwrap_or_default()
}
