use crate::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(crate) fn make_report(
    packet: &Packet,
    source: &[Triad],
    candidates: &[Triad],
) -> Result<Report> {
    let complexity = complexity_score(source, candidates);
    let gaps = evidence_gaps(source, candidates);
    let weak_conf = low_confidence(source, candidates);
    let conflicts = symbolic_conflicts(source, candidates);
    let limits = limit_warnings(source, candidates, packet);
    let wave = score_candidates(source, candidates);
    let routes = route_coherence(source, candidates);
    let structural_map = structural_map(source, candidates);
    let codex_failure_field = codex_failure_field(packet, source, candidates);
    let repair_queue = merge_repair_queues(&codex_failure_field, &structural_map["repair_queue"]);
    let baselines = baseline_summary(source, candidates);

    let has_foreign_pull = structural_map["foreign_pull"]
        .as_array()
        .is_some_and(|items| !items.is_empty());
    let has_owner_conflict = structural_map["owner_gravity"]["conflicts"]
        .as_array()
        .is_some_and(|items| !items.is_empty());
    let has_negative_routes = structural_map["negative_routes"]["hits"]
        .as_array()
        .is_some_and(|items| !items.is_empty());
    let failure_verdict = codex_failure_field["verdict"]
        .as_str()
        .unwrap_or("NOT_ENABLED");

    let alias_watch = packet.canonicalization.conflict_count > 0
        || packet
            .canonicalization
            .warnings
            .iter()
            .any(|item| item.contains("low confidence") || item.contains("empty canonical"));

    let has_weak_route = routes["weak"].as_array().is_some_and(|x| !x.is_empty());
    let has_weak_wave =
        !candidates.is_empty() && wave["weak"].as_array().is_some_and(|x| !x.is_empty());
    let has_candidate_watch = alias_watch || !gaps.is_empty() || !weak_conf.is_empty();

    let verdict = if failure_verdict == "HARD_STOP" || failure_verdict == "VETO" {
        "VETO"
    } else if failure_verdict == "ANALYSIS_INSUFFICIENT"
        || limits.iter().any(|x| x.contains("hard limit"))
    {
        "WATCH"
    } else if !conflicts.is_empty() || has_foreign_pull || has_owner_conflict || has_negative_routes
    {
        "VETO"
    } else if has_candidate_watch {
        "WATCH"
    } else if has_weak_route || has_weak_wave {
        "VETO"
    } else if (complexity < MANDATORY_COMPLEXITY && candidates.is_empty()) || source.is_empty() {
        "WATCH"
    } else {
        "PASS"
    }
    .to_string();

    let mut weak_ids: BTreeSet<String> = BTreeSet::new();
    for value in wave["weak"].as_array().into_iter().flatten() {
        if let Some(id) = value.as_str() {
            weak_ids.insert(id.to_string());
        }
    }
    for item in gaps.iter().chain(weak_conf.iter()) {
        weak_ids.insert(item.clone());
    }
    let stable: Vec<String> = if wave["stable"].as_array().is_some_and(|x| !x.is_empty()) {
        wave["stable"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect()
    } else {
        source
            .iter()
            .filter(|triad| !gaps.contains(&triad.id))
            .map(|triad| triad.id.clone())
            .collect()
    };

    let mut report = Report {
        verdict,
        engine: ENGINE_ID.to_string(),
        core_version: CORE_VERSION.to_string(),
        wave_dim: WAVE_DIM,
        task_id: packet.task_id.clone(),
        domain: packet.domain.clone(),
        complexity_score: complexity,
        mandatory_gate: complexity >= MANDATORY_COMPLEXITY,
        limits,
        stable_triads: stable,
        weak_triads: weak_ids.into_iter().collect(),
        conflicts,
        evidence_gaps: gaps,
        canonicalization: packet.canonicalization.clone(),
        baseline_summary: baselines,
        wave_summary: wave,
        route_coherence: routes,
        structural_map,
        codex_failure_field,
        repair_queue,
        agent_decision: Value::Null,
        explanation: vec![],
        repair_prompt: String::new(),
        trace_path: String::new(),
    };
    report.agent_decision = report_agent_decision(&report);
    report.explanation = build_explanation(&report);
    report.repair_prompt = build_repair_prompt(&report);
    report.trace_path = write_trace(&report)?;
    Ok(report)
}

pub(crate) fn print_report(report: &Report, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputFormat::Text => {
            println!("verdict: {}", report.verdict);
            println!("engine: {}", report.engine);
            println!("task_id: {}", report.task_id);
            println!("complexity_score: {}", report.complexity_score);
            println!("mandatory_gate: {}", report.mandatory_gate);
            println!(
                "agent_action: {}",
                report.agent_decision["action"]
                    .as_str()
                    .unwrap_or("REVIEW_REQUIRED")
            );
            println!(
                "codex_failure_field: {}",
                report.codex_failure_field["verdict"]
                    .as_str()
                    .unwrap_or("NOT_ENABLED")
            );
            if report.canonicalization.enabled {
                println!(
                    "canonicalization: applied={} conflicts={} warnings={}",
                    report.canonicalization.applied_count,
                    report.canonicalization.conflict_count,
                    report.canonicalization.watch_count
                );
                for item in &report.canonicalization.applied {
                    println!(
                        "  - {} {}: {} -> {}",
                        item.triad_id, item.field, item.from, item.to
                    );
                }
            }
            for (label, items) in [
                ("conflicts", &report.conflicts),
                ("evidence_gaps", &report.evidence_gaps),
                ("weak_triads", &report.weak_triads),
                ("explanation", &report.explanation),
            ] {
                if !items.is_empty() {
                    println!("{label}:");
                    for item in items {
                        println!("  - {item}");
                    }
                }
            }
            if report.verdict != "PASS" {
                print_repair_queue_text(&report.repair_queue);
                println!("repair:");
                for line in report.repair_prompt.lines() {
                    println!("  {line}");
                }
            }
            println!("trace_path: {}", report.trace_path);
        }
        OutputFormat::Md => {
            println!("# NANDA Report\n");
            println!("- verdict: `{}`", report.verdict);
            println!("- action: `{}`", action_for_report(report));
            println!(
                "- agent_action: `{}`",
                report.agent_decision["action"]
                    .as_str()
                    .unwrap_or("REVIEW_REQUIRED")
            );
            println!(
                "- codex_failure_field: `{}`",
                report.codex_failure_field["verdict"]
                    .as_str()
                    .unwrap_or("NOT_ENABLED")
            );
            println!("- complexity: `{}`", report.complexity_score);
            if report.canonicalization.enabled {
                println!(
                    "- canonicalization: `applied={} conflicts={} warnings={}`",
                    report.canonicalization.applied_count,
                    report.canonicalization.conflict_count,
                    report.canonicalization.watch_count
                );
            }
            println!("- trace: `{}`", report.trace_path);
        }
    }
    Ok(())
}

fn print_repair_queue_text(repair_queue: &Value) {
    let Some(items) = repair_queue.as_array() else {
        return;
    };
    if items.is_empty() {
        return;
    }
    println!("repair_queue:");
    for item in items {
        println!(
            "  - [{}] {}: {}",
            item["priority"].as_str().unwrap_or("medium"),
            item["kind"].as_str().unwrap_or("repair"),
            item["repair"]
                .as_str()
                .unwrap_or("repair route before retrying")
        );
    }
}

pub(crate) fn print_waw_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("suite: {}", out["suite"].as_str().unwrap_or(""));
    println!("passed: {}/{}", out["passed"], out["total"]);
    println!("waw_score: {}", out["waw_score"]);
    println!("structural_wins: {}", out["structural_wins"]);
    println!("lexical_traps: {}", out["lexical_traps"]);
    println!("explainable_drifts: {}", out["explainable_drifts"]);
}

pub(crate) fn print_waw_md(out: &Value) {
    println!("# NANDA WAW Benchmark\n");
    println!("- core: `{}`", out["core_version"].as_str().unwrap_or(""));
    println!("- suite: `{}`", out["suite"].as_str().unwrap_or(""));
    println!("- passed: `{}/{}`", out["passed"], out["total"]);
    println!("- waw_score: `{}`", out["waw_score"]);
    println!("- structural_wins: `{}`", out["structural_wins"]);
    println!("- lexical_traps: `{}`", out["lexical_traps"]);
    println!("- explainable_drifts: `{}`", out["explainable_drifts"]);
}

pub(crate) fn print_eval_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    println!("accuracy: {}", out["accuracy"].as_f64().unwrap_or(0.0));
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {} ok={} peak={}/{} state={}/{}",
                case["case"].as_str().unwrap_or(""),
                case["ok"].as_bool().unwrap_or(false),
                case["actual_peak"].as_str().unwrap_or(""),
                case["expected_peak"].as_str().unwrap_or(""),
                case["actual_state"].as_str().unwrap_or(""),
                case["expected_state"].as_str().unwrap_or("")
            );
        }
    }
}

pub(crate) fn print_eval_md(out: &Value) {
    println!("# NANDA Eval\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- passed: `{}/{}`", out["passed"], out["total"]);
    println!("- accuracy: `{}`", out["accuracy"]);
}

pub(crate) fn serve_cmd(args: ServeArgs) -> Result<u8> {
    match args.format {
        ServeFormat::Jsonl => serve_jsonl(),
    }
}

pub(crate) fn serve_jsonl() -> Result<u8> {
    let stdin = io::stdin();
    let mut state = ServeState::default();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let started = Instant::now();
        let response = match handle_serve_request(&line, &mut state) {
            Ok(result) => {
                json!({"ok": true, "elapsed_ms": started.elapsed().as_secs_f64() * 1000.0, "result": result})
            }
            Err(err) => json!({"ok": false, "error": format!("{err:#}")}),
        };
        println!("{}", serde_json::to_string(&response)?);
    }
    Ok(EXIT_PASS)
}

#[derive(Default)]
pub(crate) struct ServeState {
    focus_cache: HashMap<PathBuf, (focus::FocusBuild, Value)>,
    proof_cache: HashMap<ServeProofKey, Value>,
    token_cache: HashMap<ServeTokenKey, Value>,
    chat_cache: HashMap<ServeChatKey, Value>,
    answer_cache: HashMap<ServeAnswerKey, Value>,
    atlas_cache: HashMap<PathBuf, Value>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ServeProofKey {
    manifest: PathBuf,
    top_k: usize,
    group_by: String,
    sample: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ServeTokenKey {
    source: String,
    text: String,
    top_k: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ServeChatKey {
    source: String,
    prompt: String,
    steps: usize,
    top_k: usize,
    beam_width: usize,
    temperature_milli: i64,
    language: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ServeAnswerKey {
    source: String,
    prompt: String,
    facts: usize,
    top_k: usize,
    language: String,
}

pub(crate) fn handle_serve_request(line: &str, state: &mut ServeState) -> Result<Value> {
    let request: Value = serde_json::from_str(line).context("parse serve request JSON")?;
    let command = request["command"]
        .as_str()
        .ok_or_else(|| anyhow!("serve request requires string field command"))?;
    match command {
        "doctor" => Ok(doctor_value()),
        "search" => {
            let packet_value = request
                .get("packet")
                .ok_or_else(|| anyhow!("search request requires packet"))?;
            let packet: Packet = serde_json::from_value(packet_value.clone())?;
            let top_k = request["top_k"].as_u64().unwrap_or(5) as usize;
            let group_by = match request["group_by"].as_str().unwrap_or("route") {
                "group" => PeakGroupBy::Group,
                "route" => PeakGroupBy::Route,
                other => return Err(anyhow!("unsupported group_by: {other}")),
            };
            let (query, query_source) = search_query_triads(&packet, &packet.query);
            Ok(interference_search(
                &packet,
                &normalize_ids(packet.triads.clone(), "m"),
                &query,
                top_k,
                &group_by,
                query_source,
                no_focus_metadata(packet.triads.len()),
            ))
        }
        "check" => {
            let packet_value = request
                .get("packet")
                .ok_or_else(|| anyhow!("check request requires packet"))?;
            let packet: Packet = serde_json::from_value(packet_value.clone())?;
            let source = normalize_ids(packet.triads.clone(), "t");
            let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
            Ok(serde_json::to_value(make_report(
                &packet,
                &source,
                &candidates,
            )?)?)
        }
        "dataset-doctor" | "dataset_doctor" => {
            let packet_value = request
                .get("packet")
                .ok_or_else(|| anyhow!("dataset-doctor request requires packet"))?;
            let packet: Packet = serde_json::from_value(packet_value.clone())?;
            let route_cap = request["route_cap"].as_u64().unwrap_or(256) as usize;
            Ok(corpus_diagnostics(
                &normalize_ids(packet.triads.clone(), "m"),
                &normalize_ids(packet.candidate_triads.clone(), "q"),
                &packet.query,
                route_cap,
            ))
        }
        "guard_action" | "guard-action" => {
            let atlas_path = request["atlas"]
                .as_str()
                .ok_or_else(|| anyhow!("guard_action request requires atlas"))?;
            let symptom = request["symptom"].as_str().unwrap_or("");
            let action_id = request["action_id"]
                .as_str()
                .or_else(|| request["action-id"].as_str())
                .ok_or_else(|| anyhow!("guard_action request requires action_id"))?;
            let runtime_snapshot = request
                .get("runtime_snapshot")
                .or_else(|| request.get("runtime-snapshot"))
                .cloned()
                .unwrap_or(Value::Null);
            let (atlas, cache_hit) = load_serve_atlas(state, Path::new(atlas_path))?;
            let mut out =
                commands::guard::guard_action(atlas, symptom, action_id, &runtime_snapshot);
            out["serve_cache"] = json!({
                "enabled": true,
                "state": if cache_hit { "SERVE_ATLAS_HIT" } else { "SERVE_ATLAS_WARMED" }
            });
            Ok(out)
        }
        "guard_diff" | "guard-diff" => {
            let atlas_path = request["atlas"]
                .as_str()
                .ok_or_else(|| anyhow!("guard_diff request requires atlas"))?;
            let action_id = request["action_id"]
                .as_str()
                .or_else(|| request["action-id"].as_str())
                .ok_or_else(|| anyhow!("guard_diff request requires action_id"))?;
            let diff = if let Some(diff) = request["diff"].as_str() {
                diff.to_string()
            } else if let Some(path) = request["diff_path"].as_str() {
                std::fs::read_to_string(path)?
            } else {
                return Err(anyhow!("guard_diff request requires diff or diff_path"));
            };
            let (atlas, cache_hit) = load_serve_atlas(state, Path::new(atlas_path))?;
            let mut out = commands::guard::guard_diff(atlas, action_id, &diff);
            out["serve_cache"] = json!({
                "enabled": true,
                "state": if cache_hit { "SERVE_ATLAS_HIT" } else { "SERVE_ATLAS_WARMED" }
            });
            Ok(out)
        }
        "proof_cache_only" | "proof-cache-only" => {
            let manifest = request["manifest"]
                .as_str()
                .or_else(|| request["cache_only"].as_str())
                .ok_or_else(|| anyhow!("proof_cache_only request requires manifest"))?;
            let top_k = request["top_k"].as_u64().unwrap_or(5) as usize;
            let sample = request["sample"].as_u64().unwrap_or(8) as usize;
            let compact = request["compact"].as_bool().unwrap_or(false)
                || request["response"].as_str() == Some("compact")
                || request["format"].as_str() == Some("compact");
            let group_by = match request["group_by"].as_str().unwrap_or("route") {
                "group" => PeakGroupBy::Group,
                "route" => PeakGroupBy::Route,
                other => return Err(anyhow!("unsupported group_by: {other}")),
            };
            let group_by_name = match group_by {
                PeakGroupBy::Group => "group",
                PeakGroupBy::Route => "route",
            };
            let manifest_path = PathBuf::from(manifest);
            let manifest_key = serve_manifest_key(manifest_path.as_path())?;
            let proof_key = ServeProofKey {
                manifest: manifest_key.clone(),
                top_k,
                group_by: group_by_name.to_string(),
                sample,
            };
            if let Some(cached) = state.proof_cache.get(&proof_key) {
                let serve_cache = json!({
                    "enabled": true,
                    "state": "SERVE_PROOF_HIT"
                });
                if compact {
                    return Ok(serve_compact_proof_response(
                        cached,
                        serve_cache,
                        &proof_key,
                    ));
                }
                let mut proof = cached.clone();
                proof["serve_cache"] = serve_cache;
                return Ok(proof);
            }
            let ((focus_build, cache_report), cache_hit) =
                load_serve_focus_cache(state, &manifest_key, manifest_path.as_path())?;
            let mut proof = proof::run_cache_only_proof_from_focus(
                focus_build,
                cache_report.clone(),
                top_k,
                &group_by,
                sample,
                false,
            )?;
            state.proof_cache.insert(proof_key.clone(), proof.clone());
            proof["serve_cache"] = json!({
                "enabled": true,
                "state": if cache_hit { "SERVE_MEMORY_HIT" } else { "SERVE_MEMORY_WARMED" }
            });
            if compact {
                Ok(serve_compact_proof_response(
                    &proof,
                    proof["serve_cache"].clone(),
                    &proof_key,
                ))
            } else {
                Ok(proof)
            }
        }
        "llmwave_token" | "llmwave-token" => {
            let text = request["text"].as_str().unwrap_or("");
            let top_k = request["top_k"].as_u64().unwrap_or(5) as usize;
            let (packet, source) = if let Some(packet_value) = request.get("packet") {
                let packet: Packet = serde_json::from_value(packet_value.clone())?;
                (packet, serve_packet_hash(packet_value)?)
            } else {
                let input = request["input"]
                    .as_str()
                    .ok_or_else(|| anyhow!("llmwave_token request requires packet or input"))?;
                let task_id = request["task_id"].as_str().unwrap_or("serve-llmwave-token");
                let domain = request["domain"].as_str().unwrap_or("general");
                let normalize_paths = request["normalize_paths"].as_bool().unwrap_or(false);
                let packet = load_packet_auto(
                    Path::new(input),
                    &InputFormat::Auto,
                    task_id,
                    domain,
                    text,
                    normalize_paths,
                )?;
                (
                    packet,
                    serve_manifest_key(Path::new(input))?.display().to_string(),
                )
            };
            let key = ServeTokenKey {
                source,
                text: text.to_string(),
                top_k,
            };
            if let Some(cached) = state.token_cache.get(&key) {
                let mut out = cached.clone();
                out["serve_cache"] = json!({
                    "enabled": true,
                    "state": "SERVE_TOKEN_HIT"
                });
                return Ok(out);
            }
            let mut out = llmwave_token_compact_report(&packet, text, top_k);
            state.token_cache.insert(key, out.clone());
            out["serve_cache"] = json!({
                "enabled": true,
                "state": "SERVE_TOKEN_WARMED"
            });
            Ok(out)
        }
        "llmwave_chat" | "llmwave-chat" => {
            let prompt = request["prompt"].as_str().unwrap_or("");
            let steps = request["steps"].as_u64().unwrap_or(3) as usize;
            let top_k = request["top_k"].as_u64().unwrap_or(3) as usize;
            let beam_width = request["beam_width"].as_u64().unwrap_or(3) as usize;
            let temperature = request["temperature"].as_f64().unwrap_or(0.0);
            let language = request["language"].as_str().unwrap_or("en");
            let (memory, source) = if let Some(memory_value) = request.get("memory_packet") {
                (memory_value.clone(), serve_packet_hash(memory_value)?)
            } else if let Some(memory_value) =
                request.get("memory").filter(|value| value.is_object())
            {
                (memory_value.clone(), serve_packet_hash(memory_value)?)
            } else {
                let input = request["memory"]
                    .as_str()
                    .or_else(|| request["input"].as_str())
                    .ok_or_else(|| {
                        anyhow!("llmwave_chat request requires memory, memory_packet, or input")
                    })?;
                (
                    llmwave_memory_load(Path::new(input))?,
                    serve_manifest_key(Path::new(input))?.display().to_string(),
                )
            };
            let key = ServeChatKey {
                source,
                prompt: prompt.to_string(),
                steps,
                top_k,
                beam_width,
                temperature_milli: (temperature * 1000.0).round() as i64,
                language: language.to_string(),
            };
            if let Some(cached) = state.chat_cache.get(&key) {
                let mut out = cached.clone();
                out["serve_cache"] = json!({
                    "enabled": true,
                    "state": "SERVE_CHAT_HIT"
                });
                return Ok(out);
            }
            let mut out = llmwave_memory_chat_report(
                &memory,
                prompt,
                steps,
                top_k,
                beam_width,
                temperature,
                language,
            );
            state.chat_cache.insert(key, out.clone());
            out["serve_cache"] = json!({
                "enabled": true,
                "state": "SERVE_CHAT_WARMED"
            });
            Ok(out)
        }
        "llmwave_answer" | "llmwave-answer" => {
            let prompt = request["prompt"].as_str().unwrap_or("");
            let facts = request["facts"].as_u64().unwrap_or(5) as usize;
            let top_k = request["top_k"].as_u64().unwrap_or(3) as usize;
            let language = request["language"].as_str().unwrap_or("en");
            let (memory, source) = if let Some(memory_value) = request.get("memory_packet") {
                (memory_value.clone(), serve_packet_hash(memory_value)?)
            } else if let Some(memory_value) =
                request.get("memory").filter(|value| value.is_object())
            {
                (memory_value.clone(), serve_packet_hash(memory_value)?)
            } else {
                let input = request["memory"]
                    .as_str()
                    .or_else(|| request["input"].as_str())
                    .ok_or_else(|| {
                        anyhow!("llmwave_answer request requires memory, memory_packet, or input")
                    })?;
                (
                    llmwave_memory_load(Path::new(input))?,
                    serve_manifest_key(Path::new(input))?.display().to_string(),
                )
            };
            let key = ServeAnswerKey {
                source,
                prompt: prompt.to_string(),
                facts,
                top_k,
                language: language.to_string(),
            };
            if let Some(cached) = state.answer_cache.get(&key) {
                let mut out = cached.clone();
                out["serve_cache"] = json!({
                    "enabled": true,
                    "state": "SERVE_ANSWER_HIT"
                });
                return Ok(out);
            }
            let mut out = llmwave_memory_answer_report(&memory, prompt, facts, top_k, language);
            state.answer_cache.insert(key, out.clone());
            out["serve_cache"] = json!({
                "enabled": true,
                "state": "SERVE_ANSWER_WARMED"
            });
            Ok(out)
        }
        other => Err(anyhow!("unsupported serve command: {other}")),
    }
}

fn load_serve_atlas<'a>(state: &'a mut ServeState, path: &Path) -> Result<(&'a Value, bool)> {
    let key = serve_manifest_key(path)?;
    let cache_hit = state.atlas_cache.contains_key(&key);
    if !cache_hit {
        let atlas = commands::guard::load_atlas(&key)?;
        state.atlas_cache.insert(key.clone(), atlas);
    }
    let atlas = state
        .atlas_cache
        .get(&key)
        .ok_or_else(|| anyhow!("serve atlas cache did not retain {}", key.display()))?;
    Ok((atlas, cache_hit))
}

fn serve_packet_hash(packet: &Value) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_vec(packet)?);
    Ok(hasher
        .finalize()
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

fn serve_compact_proof_response(proof: &Value, serve_cache: Value, key: &ServeProofKey) -> Value {
    json!({
        "mode": "proof-cache-only-compact",
        "proof_id": serve_proof_id(key),
        "proof_state": proof["proof_state"],
        "verdict": proof["verdict"],
        "safe_to_answer": proof["safe_to_answer"],
        "answer_ready": proof["answer_ready"],
        "top_peak": proof["top_peak"],
        "field_state": proof["field_state"],
        "proof_confidence": proof["proof_confidence"],
        "reason_codes": proof["reason_codes"],
        "proof_mode": proof["proof_mode"],
        "proof_compare_state": proof["proof_compare"]["state"],
        "focus_cache": {
            "state": proof["focus_cache"]["state"],
            "key": proof["focus_cache"]["key"]
        },
        "serve_cache": serve_cache,
        "runtime_contract": {
            "state": proof["runtime_contract"]["state"],
            "focus_state": proof["runtime_contract"]["focus"]["state"]
        },
        "focused_memory_size": proof["focused_memory_size"],
        "focused_query_size": proof["focused_query_size"],
        "read_as": "Compact serve proof response. Use full response only when support, anti-support, or diagnostics are needed."
    })
}

fn serve_proof_id(key: &ServeProofKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.manifest.to_string_lossy().as_bytes());
    hasher.update(key.top_k.to_le_bytes());
    hasher.update(key.group_by.as_bytes());
    hasher.update(key.sample.to_le_bytes());
    hasher
        .finalize()
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn serve_manifest_key(manifest: &Path) -> Result<PathBuf> {
    if manifest.is_file() {
        manifest
            .canonicalize()
            .with_context(|| format!("canonicalize {}", manifest.display()))
    } else {
        Ok(manifest.to_path_buf())
    }
}

fn load_serve_focus_cache<'a>(
    state: &'a mut ServeState,
    key: &Path,
    manifest: &Path,
) -> Result<(&'a (focus::FocusBuild, Value), bool)> {
    let cache_hit = state.focus_cache.contains_key(key);
    if !cache_hit {
        let loaded = focus_cache::load_focus_cache_manifest(manifest)?;
        state.focus_cache.insert(key.to_path_buf(), loaded);
    }
    let record = state
        .focus_cache
        .get(key)
        .ok_or_else(|| anyhow!("serve focus cache did not retain {}", key.display()))?;
    Ok((record, cache_hit))
}

pub(crate) fn doctor_cmd(args: DoctorArgs) -> Result<u8> {
    let out = doctor_value();
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_doctor_text(&out),
        OutputFormat::Md => print_doctor_md(&out),
    }
    if out["healthy"].as_bool().unwrap_or(false) {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_VETO)
    }
}

pub(crate) fn doctor_value() -> Value {
    let route_trap = builtin_route_trap_packet(false);
    let trap_result = interference_search(
        &route_trap,
        &normalize_ids(route_trap.triads.clone(), "m"),
        &normalize_ids(route_trap.candidate_triads.clone(), "q"),
        3,
        &PeakGroupBy::Route,
        "candidate_triads",
        no_focus_metadata(route_trap.triads.len()),
    );
    let noisy = builtin_route_trap_packet(true);
    let noisy_result = interference_search(
        &noisy,
        &normalize_ids(noisy.triads.clone(), "m"),
        &normalize_ids(noisy.candidate_triads.clone(), "q"),
        3,
        &PeakGroupBy::Route,
        "candidate_triads",
        no_focus_metadata(noisy.triads.len()),
    );
    let trap_field_state = trap_result["field_state_machine"]["state"]
        .as_str()
        .unwrap_or("");
    let noisy_field_state = noisy_result["field_state_machine"]["state"]
        .as_str()
        .unwrap_or("");
    let trap_ok = trap_result["peaks"]
        .as_array()
        .and_then(|peaks| peaks.first())
        .and_then(|peak| peak["peak"].as_str())
        == Some("certification")
        && trap_result["peak_decision"]["state"].as_str() == Some("FOCUSED")
        && trap_field_state == "FIELD_FOCUSED"
        && trap_result["wins_over_lexical_baseline"]
            .as_bool()
            .unwrap_or(false);
    let noisy_ok = noisy_result["peak_decision"]["state"].as_str() == Some("WATCH")
        && noisy_field_state == "FIELD_CONTESTED"
        && !noisy_result["peak_decision"]["safe_to_answer"]
            .as_bool()
            .unwrap_or(true);
    let healthy = trap_ok && noisy_ok;
    json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "doctor",
        "healthy": healthy,
        "checks": {
            "route_trap_focused": trap_ok,
            "noisy_query_watch": noisy_ok,
            "field_state_machine": trap_field_state == "FIELD_FOCUSED" && noisy_field_state == "FIELD_CONTESTED"
        },
        "route_trap": {
            "top": trap_result["peaks"].as_array().and_then(|peaks| peaks.first()).and_then(|peak| peak["peak"].as_str()).unwrap_or(""),
            "state": trap_result["peak_decision"]["state"],
            "field_state": trap_result["field_state_machine"]["state"],
            "field_action": trap_result["field_state_machine"]["action"],
            "safe_to_answer": trap_result["peak_decision"]["safe_to_answer"],
            "field_safe_to_answer": trap_result["field_state_machine"]["safe_to_answer"],
            "lexical_baseline_top": trap_result["lexical_baseline"]["top_peak"],
            "wins_over_lexical_baseline": trap_result["wins_over_lexical_baseline"],
            "peak_margin": trap_result["peak_margin"]
        },
        "noisy": {
            "top": noisy_result["peaks"].as_array().and_then(|peaks| peaks.first()).and_then(|peak| peak["peak"].as_str()).unwrap_or(""),
            "state": noisy_result["peak_decision"]["state"],
            "field_state": noisy_result["field_state_machine"]["state"],
            "field_action": noisy_result["field_state_machine"]["action"],
            "safe_to_answer": noisy_result["peak_decision"]["safe_to_answer"],
            "field_safe_to_answer": noisy_result["field_state_machine"]["safe_to_answer"],
            "peak_margin": noisy_result["peak_margin"]
        }
    })
}

pub(crate) fn print_doctor_text(out: &Value) {
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("healthy: {}", out["healthy"].as_bool().unwrap_or(false));
    println!(
        "route_trap: top={} state={} field={} lexical={} wins={}",
        out["route_trap"]["top"].as_str().unwrap_or(""),
        out["route_trap"]["state"].as_str().unwrap_or(""),
        out["route_trap"]["field_state"].as_str().unwrap_or(""),
        out["route_trap"]["lexical_baseline_top"]
            .as_str()
            .unwrap_or(""),
        out["route_trap"]["wins_over_lexical_baseline"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "noisy: top={} state={} field={} safe={}",
        out["noisy"]["top"].as_str().unwrap_or(""),
        out["noisy"]["state"].as_str().unwrap_or(""),
        out["noisy"]["field_state"].as_str().unwrap_or(""),
        out["noisy"]["safe_to_answer"].as_bool().unwrap_or(false)
    );
}

pub(crate) fn print_doctor_md(out: &Value) {
    println!("# NANDA Doctor\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!("- healthy: `{}`", out["healthy"]);
    println!(
        "- route_trap: `{}` / `{}` / `{}`",
        out["route_trap"]["top"], out["route_trap"]["state"], out["route_trap"]["field_state"]
    );
    println!(
        "- noisy: `{}` / `{}` / `{}`",
        out["noisy"]["top"], out["noisy"]["state"], out["noisy"]["field_state"]
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn doctor_triad(
    id: &str,
    subject: &str,
    relation: &str,
    object: &str,
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
        evidence: "builtin-doctor".to_string(),
        confidence: 0.9,
        subject_role: subject_role.to_string(),
        object_role: object_role.to_string(),
        route: route.to_string(),
        group: group.to_string(),
        layer: "core".to_string(),
        owner: "builtin-doctor".to_string(),
        entrypoint: "nanda-doctor".to_string(),
        output: "doctor-report".to_string(),
        evidence_path: String::new(),
        scope: "self-check".to_string(),
    }
}

pub(crate) fn report_cmd(args: ReportArgs) -> Result<u8> {
    let overall_packet = packet_from_markdown(&args.overall, "overall", &args.domain, "", false)?;
    let overall = make_report(
        &overall_packet,
        &normalize_ids(overall_packet.triads.clone(), "t"),
        &normalize_ids(overall_packet.candidate_triads.clone(), "c"),
    )?;
    let mut route_reports = serde_json::Map::new();
    let mut worst = verdict_code(&overall.verdict);
    for route in args.routes {
        let (name, path) = route
            .split_once(':')
            .ok_or_else(|| anyhow!("--route must be name:path"))?;
        let packet = packet_from_markdown(Path::new(path), name, &args.domain, "", false)?;
        let report = make_report(
            &packet,
            &normalize_ids(packet.triads.clone(), "t"),
            &normalize_ids(packet.candidate_triads.clone(), "c"),
        )?;
        worst = worst_status(worst, verdict_code(&report.verdict));
        route_reports.insert(name.to_string(), serde_json::to_value(&report)?);
    }
    let action =
        if route_reports.values().any(|v| v["verdict"] == "VETO") || overall.verdict == "VETO" {
            "REPAIR_REQUIRED"
        } else if overall.verdict == "WATCH" {
            "DRAFT_OK_REVIEW_REQUIRED"
        } else {
            "SEND_OK"
        };
    let packet = json!({
        "title": args.title,
        "action": action,
        "safe_to_draft": action != "REPAIR_REQUIRED",
        "safe_to_send": action == "SEND_OK",
        "blocking": action == "REPAIR_REQUIRED",
        "review_required": action != "SEND_OK",
        "overall": overall,
        "routes": route_reports,
        "repair_prompts": [],
        "next_prompt": if action == "SEND_OK" { "Finalize with checked structure." } else { "Repair or split unresolved routes before final send." }
    });
    match args.format {
        OutputFormat::Json | OutputFormat::Text => {
            println!("{}", serde_json::to_string_pretty(&packet)?)
        }
        OutputFormat::Md => {
            println!("# {}\n", packet["title"].as_str().unwrap_or("NANDA Report"));
            println!("- action: `{}`", action);
            println!("- safe_to_draft: `{}`", packet["safe_to_draft"]);
            println!("- safe_to_send: `{}`", packet["safe_to_send"]);
        }
    }
    if action == "REPAIR_REQUIRED" {
        Ok(EXIT_VETO)
    } else if action == "DRAFT_OK_REVIEW_REQUIRED" {
        Ok(EXIT_WATCH)
    } else {
        Ok(worst)
    }
}

pub(crate) fn write_or_print(path: PathBuf, stdout: bool, output: String) -> Result<()> {
    if stdout {
        print!("{output}");
    } else {
        fs::write(&path, output)?;
        println!("{}", path.display());
    }
    Ok(())
}

pub(crate) fn action_for_report(report: &Report) -> &'static str {
    if report.agent_decision["action"].as_str() == Some("HARD_STOP") {
        return "HARD_STOP";
    }
    if report.agent_decision["action"].as_str() == Some("ANALYSIS_INSUFFICIENT") {
        return "ANALYSIS_INSUFFICIENT";
    }
    match report.verdict.as_str() {
        "PASS" => "SEND_OK",
        "WATCH" => "DRAFT_OK_REVIEW_REQUIRED",
        _ => "REPAIR_REQUIRED",
    }
}
