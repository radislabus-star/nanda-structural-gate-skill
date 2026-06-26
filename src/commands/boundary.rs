use crate::*;
use clap::Parser;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub(crate) struct BoundaryEconomicsArgs {
    #[arg(default_value = ".")]
    pub(crate) input: PathBuf,
    #[arg(long)]
    pub(crate) atlas: Option<PathBuf>,
    #[arg(long)]
    pub(crate) route: Option<String>,
    #[arg(long)]
    pub(crate) owner: Option<String>,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Default)]
struct BoundaryFacts {
    files: Vec<String>,
    functions: Vec<String>,
    public_api: Vec<String>,
    call_edges: Vec<String>,
    shared_state: Vec<String>,
    runtime_side_effects: Vec<String>,
    tests: Vec<String>,
    routes: BTreeSet<String>,
    owners: BTreeSet<String>,
    thin_wrappers: Vec<String>,
    foreign_route_files: Vec<String>,
    foreign_route_details: Vec<Value>,
    runtime_checks: Vec<String>,
    route_scoped: bool,
    owner_filter_requested: bool,
    owner_filter_matched: bool,
    requested_owner: Option<String>,
    route_files_considered: usize,
}

pub(crate) fn cmd(args: BoundaryEconomicsArgs) -> Result<u8> {
    let out = report_with_atlas(
        &args.input,
        args.atlas.as_deref(),
        args.route.as_deref(),
        args.owner.as_deref(),
    )?;
    print_boundary_output(&out, &args.format)?;
    Ok(boundary_exit_code(&out))
}

pub(crate) fn report(input: &Path, route: Option<&str>, owner: Option<&str>) -> Result<Value> {
    report_with_atlas(input, None, route, owner)
}

pub(crate) fn report_with_atlas(
    input: &Path,
    atlas_path: Option<&Path>,
    route: Option<&str>,
    owner: Option<&str>,
) -> Result<Value> {
    let root = if input.is_dir() {
        input.to_path_buf()
    } else {
        input
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    let atlas = match atlas_path {
        Some(path) => Some(commands::guard::load_atlas(path)?),
        None => None,
    };
    let mut route_scoped = false;
    let mut runtime_checks = vec![];
    let mut files = vec![];
    if let (Some(atlas), Some(route)) = (atlas.as_ref(), route) {
        if let Some(route_node) = atlas["routes"][route].as_object() {
            route_scoped = true;
            files = route_node["allowed_files"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            runtime_checks = route_node["runtime_checks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
        }
    }
    if files.is_empty() && input.is_dir() {
        collect_boundary_files(&root, input, &mut files)?;
    } else if files.is_empty() && input.is_file() {
        files.push(input.to_string_lossy().to_string());
    }
    let route_files_considered = files.len();
    let mut owner_filter_requested = false;
    let mut owner_filter_matched = true;
    if route_scoped {
        if let Some(owner) = owner {
            owner_filter_requested = true;
            let filtered = filter_files_by_owner(&files, owner);
            if !filtered.is_empty() {
                files = filtered;
            } else {
                owner_filter_matched = false;
                files.clear();
            }
        }
    }
    files.sort();

    let target_route = route
        .map(str::to_string)
        .or_else(|| infer_single_route(&files));
    let target_owner = owner
        .map(str::to_string)
        .or_else(|| infer_single_owner(&files));
    let mut facts = collect_facts(&root, &files, target_route.as_deref());
    facts.runtime_checks = runtime_checks;
    facts.route_scoped = route_scoped;
    facts.owner_filter_requested = owner_filter_requested;
    facts.owner_filter_matched = owner_filter_matched;
    facts.requested_owner = owner.map(str::to_string);
    facts.route_files_considered = route_files_considered;
    let decision = boundary_decision(&facts, target_route.as_deref(), target_owner.as_deref());

    Ok(json!({
        "mode": "boundary-economics",
        "core_version": CORE_VERSION,
        "input": input.display().to_string(),
        "atlas": atlas_path.map(|path| path.display().to_string()),
        "scope": if route_scoped { "route-scoped" } else { "repo-wide" },
        "target_route": target_route,
        "target_owner": target_owner,
        "boundary_decision": decision,
        "read_as": "NANDA Boundary Economics: NO EVIDENCE => NO CUT. Split/merge decisions require route, owner, state, API, runtime, and test evidence."
    }))
}

pub(crate) fn boundary_from_guard_action(atlas: &Value, action_out: &Value) -> Value {
    let route = action_out["route"].as_str();
    let route_node = route.map(|route| &atlas["routes"][route]);
    let owner = route_node
        .and_then(|node| node["owners"].as_array())
        .and_then(|owners| owners.first())
        .and_then(Value::as_str);
    let allowed_files = route_node
        .map(|node| node["allowed_files"].clone())
        .unwrap_or_else(|| json!([]));
    let verdict = if action_out["verdict"].as_str() == Some("PASS") && route.is_some() {
        "KEEP"
    } else if action_out["verdict"].as_str() == Some("HARD_STOP") {
        "VETO"
    } else {
        "WATCH"
    };
    json!({
        "verdict": verdict,
        "route": route,
        "owner": owner,
        "reason": if verdict == "KEEP" { "action route is known; no boundary cut is requested" } else { "action route is not sufficiently proven for boundary edits" },
        "principle": "NO_EVIDENCE_NO_CUT",
        "score_components": empty_components(),
        "evidence": {
            "owner_edges": [],
            "foreign_pull": [],
            "shared_state": [],
            "runtime_side_effects": [],
            "tests": []
        },
        "allowed_files": allowed_files,
        "forbidden_routes": route_node.map(|node| node["forbidden_routes"].clone()).unwrap_or_else(|| json!([])),
        "must_not_change": [],
        "required_tests": route_node.map(|node| node["runtime_checks"].clone()).unwrap_or_else(|| json!([])),
        "repair": if verdict == "WATCH" { json!(["provide route, owner, evidence, and diff before split/merge"]) } else { json!([]) }
    })
}

pub(crate) fn boundary_from_guard_diff(atlas: &Value, diff_out: &Value) -> Value {
    let route = diff_out["route"].as_str();
    let route_node = route.map(|route| &atlas["routes"][route]);
    let foreign_routes = diff_out["foreign_routes"].clone();
    let foreign_files = diff_out["foreign_files"].clone();
    let has_foreign = foreign_routes
        .as_array()
        .is_some_and(|items| !items.is_empty())
        || foreign_files
            .as_array()
            .is_some_and(|items| !items.is_empty());
    let verdict = if has_foreign {
        "VETO"
    } else if diff_out["verdict"].as_str() == Some("PASS") {
        "KEEP"
    } else {
        "WATCH"
    };
    json!({
        "verdict": verdict,
        "route": route,
        "owner": route_node.and_then(|node| node["owners"].as_array()).and_then(|owners| owners.first()).and_then(Value::as_str),
        "reason": if verdict == "VETO" { "diff changes forbidden route or file; boundary refactor is unsafe" } else if verdict == "KEEP" { "diff stays inside selected route capsule; no boundary cut is requested" } else { "diff evidence is insufficient for boundary decision" },
        "principle": "NO_EVIDENCE_NO_CUT",
        "score_components": empty_components(),
        "evidence": {
            "owner_edges": [],
            "foreign_pull": foreign_routes,
            "shared_state": [],
            "runtime_side_effects": [],
            "tests": []
        },
        "allowed_files": route_node.map(|node| node["allowed_files"].clone()).unwrap_or_else(|| json!([])),
        "forbidden_routes": route_node.map(|node| node["forbidden_routes"].clone()).unwrap_or_else(|| json!([])),
        "must_not_change": ["foreign routes", "unowned runtime behavior"],
        "required_tests": route_node.map(|node| node["runtime_checks"].clone()).unwrap_or_else(|| json!([])),
        "repair": if verdict == "VETO" { json!(["split the diff by route", "choose an action_id that owns all changed files"]) } else { json!([]) }
    })
}

fn collect_boundary_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(".git")
            || name == "target"
            || name == "node_modules"
            || name == ".nanda"
            || name == "__pycache__"
        {
            continue;
        }
        if path.is_dir() {
            collect_boundary_files(root, &path, out)?;
            continue;
        }
        if !is_boundary_file(&path) {
            continue;
        }
        out.push(
            path.strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string(),
        );
    }
    Ok(())
}

fn is_boundary_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "rs" | "py" | "js" | "ts" | "tsx"))
}

fn infer_single_route(files: &[String]) -> Option<String> {
    let routes = files
        .iter()
        .map(|file| commands::dogfood::auto_route_for_path(file))
        .collect::<BTreeSet<_>>();
    (routes.len() == 1)
        .then(|| routes.into_iter().next())
        .flatten()
}

fn infer_single_owner(files: &[String]) -> Option<String> {
    let owners = files
        .iter()
        .map(|file| commands::dogfood::auto_owner_for_path(file))
        .collect::<BTreeSet<_>>();
    (owners.len() == 1)
        .then(|| owners.into_iter().next())
        .flatten()
}

fn filter_files_by_owner(files: &[String], owner: &str) -> Vec<String> {
    let owner_norm = normalize_owner_for_match(owner);
    files
        .iter()
        .filter(|file| {
            let file_norm = normalize_owner_for_match(file);
            let auto_owner_norm =
                normalize_owner_for_match(&commands::dogfood::auto_owner_for_path(file));
            file_norm.contains(&owner_norm) || auto_owner_norm.contains(&owner_norm)
        })
        .cloned()
        .collect()
}

fn normalize_owner_for_match(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn collect_facts(root: &Path, files: &[String], target_route: Option<&str>) -> BoundaryFacts {
    let mut facts = BoundaryFacts {
        files: files.to_vec(),
        ..Default::default()
    };
    let mut function_owner = BTreeMap::<String, String>::new();

    for rel in files {
        let path = root.join(rel);
        let route = commands::dogfood::auto_route_for_path(rel);
        let owner = commands::dogfood::auto_owner_for_path(rel);
        facts.routes.insert(route.clone());
        facts.owners.insert(owner.clone());
        if target_route.is_some_and(|target| target != route) {
            facts.foreign_route_files.push(rel.clone());
            facts.foreign_route_details.push(json!({
                "file": rel,
                "route": route,
                "target_route": target_route
            }));
        }
        if rel.contains("test") || rel.contains("spec") {
            facts.tests.push(rel.clone());
        }
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let functions = extract_functions(&source);
        for function in &functions {
            function_owner.insert(function.clone(), rel.clone());
        }
        facts.functions.extend(
            functions
                .iter()
                .map(|function| format!("{rel}::{function}")),
        );
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if is_public_api_line(trimmed) {
                facts
                    .public_api
                    .push(format!("{rel}:{}:{trimmed}", line_no + 1));
            }
            if is_shared_state_line(trimmed) {
                facts
                    .shared_state
                    .push(format!("{rel}:{}:{trimmed}", line_no + 1));
            }
            if is_runtime_side_effect_line(trimmed) {
                facts
                    .runtime_side_effects
                    .push(format!("{rel}:{}:{trimmed}", line_no + 1));
            }
        }
        if is_thin_wrapper(&source, &functions) {
            facts.thin_wrappers.push(rel.clone());
        }
    }

    for rel in files {
        let source = match fs::read_to_string(root.join(rel)) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let route = commands::dogfood::auto_route_for_path(rel);
        for (name, owner_file) in &function_owner {
            if owner_file == rel {
                continue;
            }
            if source.contains(&format!("{name}(")) {
                let other_route = commands::dogfood::auto_route_for_path(owner_file);
                facts
                    .call_edges
                    .push(format!("{rel} -> {owner_file}::{name}"));
                if other_route != route {
                    facts.foreign_route_files.push(owner_file.clone());
                    facts.foreign_route_details.push(json!({
                        "file": owner_file,
                        "route": other_route,
                        "pulled_by": rel,
                        "call_edge": format!("{rel} -> {owner_file}::{name}")
                    }));
                }
            }
        }
    }
    facts.foreign_route_files.sort();
    facts.foreign_route_files.dedup();
    facts
        .foreign_route_details
        .sort_by_key(|item| item["file"].as_str().unwrap_or("").to_string());
    facts
        .foreign_route_details
        .dedup_by(|left, right| left["file"] == right["file"] && left["route"] == right["route"]);
    facts
}

fn boundary_decision(facts: &BoundaryFacts, route: Option<&str>, owner: Option<&str>) -> Value {
    let evidence_count = facts.functions.len()
        + facts.public_api.len()
        + facts.call_edges.len()
        + facts.shared_state.len()
        + facts.runtime_side_effects.len()
        + facts.tests.len()
        + facts.foreign_route_files.len();
    let multi_route = facts.routes.len() > 1 || !facts.foreign_route_files.is_empty();
    let multi_owner = facts.owners.len() > 1;
    let has_owner = owner.is_some() || !facts.owners.is_empty();
    let has_route = route.is_some() || !facts.routes.is_empty();
    let has_tests = !facts.tests.is_empty();
    let high_public_api = facts.public_api.len() >= 6;
    let thin_wrapper = !facts.thin_wrappers.is_empty()
        && facts.shared_state.is_empty()
        && facts.tests.is_empty()
        && facts.runtime_side_effects.is_empty();

    let owner_clarity_gain = if multi_owner { 3 } else { 0 };
    let foreign_pull_reduction = if multi_route { 3 } else { 0 };
    let test_isolation_gain = if multi_route && has_tests { 1 } else { 0 };
    let state_locality_gain = if !facts.shared_state.is_empty() && !multi_route {
        1
    } else {
        0
    };
    let api_surface_growth = if high_public_api { 2 } else { 0 };
    let adapter_leak = if thin_wrapper { 2 } else { 0 };
    let runtime_risk = if !facts.runtime_side_effects.is_empty() && !has_tests {
        2
    } else {
        0
    };
    let migration_cost = if facts.files.len() > 8 || facts.functions.len() > 30 {
        2
    } else {
        0
    };
    let positive =
        owner_clarity_gain + foreign_pull_reduction + test_isolation_gain + state_locality_gain;
    let negative = api_surface_growth + adapter_leak + runtime_risk + migration_cost;
    let score = positive - negative;

    let owner_filter_failed = facts.owner_filter_requested && !facts.owner_filter_matched;
    let repo_wide_route_pressure = route.is_none() && !facts.foreign_route_files.is_empty();

    let verdict = if owner_filter_failed || !has_owner || !has_route || evidence_count == 0 {
        "WATCH"
    } else if !facts.foreign_route_files.is_empty() && route.is_some() {
        "VETO"
    } else if repo_wide_route_pressure && score <= 0 {
        "WATCH"
    } else if multi_route && score >= 3 {
        "SPLIT_STRONG"
    } else if multi_route && score > 0 {
        "SPLIT_WEAK"
    } else if thin_wrapper {
        "MERGE_CANDIDATE"
    } else {
        "KEEP"
    };

    let reason = match verdict {
        "WATCH" if owner_filter_failed => {
            "owner evidence not found in route atlas; explicit owner cannot fall back to whole route"
        }
        "WATCH" if repo_wide_route_pressure => {
            "repo-wide route pressure found, but evidence is not strong enough for autonomous split; rerun route-scoped with atlas, route, and owner"
        }
        "WATCH" => "insufficient route, owner, or evidence; NO EVIDENCE => NO CUT",
        "VETO" => "target route would cross foreign route files; split/merge is unsafe",
        "SPLIT_STRONG" => {
            "mixed routes/owners create enough evidence that a boundary should reduce confusion"
        }
        "SPLIT_WEAK" => {
            "boundary may help, but evidence is not strong enough for autonomous refactor"
        }
        "MERGE_CANDIDATE" => {
            "thin wrapper has no state, tests, runtime side effects, or independent owner evidence"
        }
        _ => "current boundary is acceptable; no cut is justified by available evidence",
    };

    json!({
        "verdict": verdict,
        "route": route,
        "owner": owner,
        "reason": reason,
        "principle": "NO_EVIDENCE_NO_CUT",
        "score": score,
        "safe_to_edit": matches!(verdict, "SPLIT_STRONG" | "KEEP"),
        "score_components": {
            "owner_clarity_gain": component(owner_clarity_gain, owner_evidence(facts)),
            "foreign_pull_reduction": component(foreign_pull_reduction, facts.foreign_route_files.clone()),
            "test_isolation_gain": component(test_isolation_gain, facts.tests.clone()),
            "state_locality_gain": component(state_locality_gain, facts.shared_state.clone()),
            "api_surface_growth": component(-api_surface_growth, facts.public_api.clone()),
            "adapter_leak": component(-adapter_leak, facts.thin_wrappers.clone()),
            "runtime_risk": component(-runtime_risk, facts.runtime_side_effects.clone()),
            "migration_cost": component(-migration_cost, facts.files.clone())
        },
        "evidence": {
            "files": facts.files,
            "functions": sample(&facts.functions, 24),
            "owner_edges": owner_evidence(facts),
            "call_edges": sample(&facts.call_edges, 24),
            "public_api_edges": sample(&facts.public_api, 24),
            "foreign_pull": facts.foreign_route_files,
            "foreign_pull_details": sample_values(&facts.foreign_route_details, 16),
            "shared_state": sample(&facts.shared_state, 24),
            "runtime_side_effects": sample(&facts.runtime_side_effects, 24),
            "tests": facts.tests,
            "route_ids": facts.routes.iter().cloned().collect::<Vec<_>>(),
            "owner_ids": facts.owners.iter().cloned().collect::<Vec<_>>(),
            "owner_filter": {
                "requested": facts.owner_filter_requested,
                "matched": facts.owner_filter_matched,
                "requested_owner": facts.requested_owner.clone(),
                "route_files_considered": facts.route_files_considered,
                "matched_files": facts.files.len()
            }
        },
        "allowed_files": if facts.route_scoped && matches!(verdict, "SPLIT_STRONG" | "SPLIT_WEAK" | "KEEP") { json!(facts.files) } else if matches!(verdict, "KEEP") { json!(sample(&facts.files, 12)) } else { json!([]) },
        "forbidden_routes": if let Some(route) = route { json!(facts.routes.iter().filter(|item| item.as_str() != route).cloned().collect::<Vec<_>>()) } else { json!([]) },
        "must_not_change": must_not_change(verdict),
        "required_tests": required_tests(facts),
        "repair": repair_tasks(verdict, owner_filter_failed, repo_wide_route_pressure)
    })
}

fn extract_functions(source: &str) -> Vec<String> {
    let mut out = vec![];
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed
            .strip_prefix("fn ")
            .or_else(|| trimmed.strip_prefix("pub fn "))
            .or_else(|| trimmed.strip_prefix("pub(crate) fn "))
        {
            let name = rest
                .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .next()
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

fn is_public_api_line(line: &str) -> bool {
    line.starts_with("pub fn ")
        || line.starts_with("pub(crate) fn ")
        || line.starts_with("pub struct ")
        || line.starts_with("pub enum ")
        || line.starts_with("pub trait ")
}

fn is_shared_state_line(line: &str) -> bool {
    [
        "static ",
        "OnceLock",
        "Mutex",
        "RwLock",
        "Atomic",
        "thread_local!",
        "Arc<Mutex",
    ]
    .iter()
    .any(|needle| line.contains(needle))
}

fn is_runtime_side_effect_line(line: &str) -> bool {
    [
        "Command::new",
        "spawn(",
        "systemctl",
        "gsettings",
        "dbus",
        "DBus",
        "ibus",
        "fs::write",
        "std::process",
    ]
    .iter()
    .any(|needle| line.contains(needle))
}

fn is_thin_wrapper(source: &str, functions: &[String]) -> bool {
    let non_empty = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .count();
    !functions.is_empty()
        && non_empty <= 24
        && !source.contains("struct ")
        && !source.contains("enum ")
        && !source.contains("static ")
}

fn owner_evidence(facts: &BoundaryFacts) -> Vec<String> {
    facts
        .owners
        .iter()
        .map(|owner| format!("owner:{owner}"))
        .collect()
}

fn component(score: i32, evidence: Vec<String>) -> Value {
    json!({
        "score": score,
        "counted": !evidence.is_empty(),
        "evidence": sample(&evidence, 16)
    })
}

fn sample(items: &[String], cap: usize) -> Vec<String> {
    items.iter().take(cap).cloned().collect()
}

fn sample_values(items: &[Value], cap: usize) -> Vec<Value> {
    items.iter().take(cap).cloned().collect()
}

fn empty_components() -> Value {
    json!({
        "owner_clarity_gain": component(0, vec![]),
        "foreign_pull_reduction": component(0, vec![]),
        "test_isolation_gain": component(0, vec![]),
        "state_locality_gain": component(0, vec![]),
        "api_surface_growth": component(0, vec![]),
        "adapter_leak": component(0, vec![]),
        "runtime_risk": component(0, vec![]),
        "migration_cost": component(0, vec![])
    })
}

fn must_not_change(verdict: &str) -> Value {
    match verdict {
        "SPLIT_STRONG" | "SPLIT_WEAK" => json!([
            "foreign routes",
            "public behavior outside contract",
            "runtime side effects without tests"
        ]),
        "MERGE_CANDIDATE" => json!(["owner public API without human review", "runtime behavior"]),
        "KEEP" => json!(["module boundary"]),
        _ => json!(["code boundary"]),
    }
}

fn required_tests(facts: &BoundaryFacts) -> Value {
    let mut tests = vec![];
    tests.extend(facts.runtime_checks.iter().take(5).cloned());
    tests.extend(facts.tests.iter().take(5).cloned());
    tests.sort();
    tests.dedup();
    if !tests.is_empty() {
        json!(tests)
    } else if !facts.runtime_side_effects.is_empty() {
        json!(["add route-specific runtime smoke before refactor"])
    } else {
        json!(["add or identify route-specific test before changing boundary"])
    }
}

fn repair_tasks(verdict: &str, owner_filter_failed: bool, repo_wide_route_pressure: bool) -> Value {
    if owner_filter_failed {
        return json!([
            "use an owner that matches the selected route atlas",
            "rebuild atlas if the owner map is stale",
            "do not expand to the whole route after owner mismatch"
        ]);
    }
    if repo_wide_route_pressure && verdict == "WATCH" {
        return json!([
            "build or refresh the route atlas",
            "select the route with boundary pressure",
            "rerun boundary economics with --atlas, --route, and --owner before cutting"
        ]);
    }
    match verdict {
        "WATCH" => json!(["collect owner, route, public API, state, runtime, and test evidence before split/merge"]),
        "VETO" => json!(["do not cut across foreign route", "split the refactor by route owner"]),
        "SPLIT_STRONG" => json!(["extract behind owner-owned API", "keep forbidden routes out of diff", "run required tests"]),
        "SPLIT_WEAK" => json!(["prepare a small mechanical step only", "ask for human review before semantic changes"]),
        "MERGE_CANDIDATE" => json!(["merge wrapper into owner module", "or make helper private behind owner public method"]),
        _ => json!([]),
    }
}

fn print_boundary_output(out: &Value, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(out)?),
        OutputFormat::Text => {
            let decision = &out["boundary_decision"];
            println!("mode: boundary-economics");
            println!(
                "verdict: {}",
                decision["verdict"].as_str().unwrap_or("WATCH")
            );
            println!("score: {}", decision["score"].as_i64().unwrap_or(0));
            println!("reason: {}", decision["reason"].as_str().unwrap_or(""));
        }
        OutputFormat::Md => {
            let decision = &out["boundary_decision"];
            println!("# NANDA Boundary Economics\n");
            println!(
                "- verdict: `{}`",
                decision["verdict"].as_str().unwrap_or("WATCH")
            );
            println!("- reason: {}", decision["reason"].as_str().unwrap_or(""));
        }
    }
    Ok(())
}

fn boundary_exit_code(out: &Value) -> u8 {
    match out["boundary_decision"]["verdict"]
        .as_str()
        .unwrap_or("WATCH")
    {
        "SPLIT_STRONG" | "KEEP" => EXIT_PASS,
        "VETO" => EXIT_VETO,
        _ => EXIT_WATCH,
    }
}
