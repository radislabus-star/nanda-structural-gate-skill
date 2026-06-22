use crate::*;
use clap::Parser;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub(crate) struct BuildAtlasArgs {
    #[arg(default_value = ".")]
    pub(crate) repo: PathBuf,
    #[arg(long, default_value = ".nanda/route-atlas.json")]
    pub(crate) out: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Parser)]
pub(crate) struct GuardActionArgs {
    pub(crate) atlas: PathBuf,
    #[arg(long)]
    pub(crate) symptom: String,
    #[arg(long = "action-id")]
    pub(crate) action_id: String,
    #[arg(long = "runtime-snapshot")]
    pub(crate) runtime_snapshot: Option<PathBuf>,
    #[arg(long)]
    pub(crate) boundary_economics: bool,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Parser)]
pub(crate) struct GuardDiffArgs {
    pub(crate) atlas: PathBuf,
    #[arg(long = "action-id")]
    pub(crate) action_id: String,
    #[arg(long)]
    pub(crate) diff: PathBuf,
    #[arg(long)]
    pub(crate) boundary_economics: bool,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Parser)]
pub(crate) struct ReleaseGateArgs {
    pub(crate) atlas: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Default)]
struct RouteBuild {
    files: BTreeSet<String>,
    owners: BTreeSet<String>,
    layers: BTreeSet<String>,
    entrypoints: BTreeSet<String>,
    tests: BTreeSet<String>,
    runtime_checks: BTreeSet<String>,
}

pub(crate) fn build_atlas_cmd(args: BuildAtlasArgs) -> Result<u8> {
    let mut atlas = build_route_atlas(&args.repo)?;
    atlas["output"] = json!(args.out.display().to_string());
    atlas["written_to"] = json!(args.out.display().to_string());
    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, serde_json::to_string_pretty(&atlas)? + "\n")?;
    print_guard_output(&atlas, &args.format)?;
    Ok(EXIT_PASS)
}

pub(crate) fn guard_action_cmd(args: GuardActionArgs) -> Result<u8> {
    let atlas = load_atlas(&args.atlas)?;
    let runtime_snapshot = match args.runtime_snapshot {
        Some(path) => serde_json::from_str::<Value>(&fs::read_to_string(&path)?)?,
        None => Value::Null,
    };
    let mut out = guard_action(&atlas, &args.symptom, &args.action_id, &runtime_snapshot);
    if args.boundary_economics {
        let boundary_decision = commands::boundary::boundary_from_guard_action(&atlas, &out);
        out["boundary_decision"] = boundary_decision.clone();
        out["boundary_economics"] = json!({
            "mode": "boundary-economics",
            "scope": "route-atlas-action",
            "boundary_decision": boundary_decision
        });
    }
    print_guard_output(&out, &args.format)?;
    Ok(guard_exit_code(&out))
}

pub(crate) fn guard_diff_cmd(args: GuardDiffArgs) -> Result<u8> {
    let atlas = load_atlas(&args.atlas)?;
    let diff_source = diff_source_repo(&atlas, &args.diff);
    let diff_result = if args.diff.to_string_lossy() == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        fs::read_to_string(&args.diff)
    };
    let diff = match diff_result {
        Ok(diff) => diff,
        Err(err) => {
            let mut out = guard_diff_unreadable(&atlas, &args.action_id, diff_source, &err);
            if args.boundary_economics {
                let boundary_decision = commands::boundary::boundary_from_guard_diff(&atlas, &out);
                out["boundary_decision"] = boundary_decision.clone();
                out["boundary_economics"] = json!({
                    "mode": "boundary-economics",
                    "scope": "route-atlas-diff",
                    "boundary_decision": boundary_decision
                });
            }
            print_guard_output(&out, &args.format)?;
            return Ok(guard_exit_code(&out));
        }
    };
    let mut out = guard_diff_with_source(&atlas, &args.action_id, &diff, diff_source);
    if args.boundary_economics {
        let boundary_decision = commands::boundary::boundary_from_guard_diff(&atlas, &out);
        out["boundary_decision"] = boundary_decision.clone();
        out["boundary_economics"] = json!({
            "mode": "boundary-economics",
            "scope": "route-atlas-diff",
            "boundary_decision": boundary_decision
        });
    }
    print_guard_output(&out, &args.format)?;
    Ok(guard_exit_code(&out))
}

pub(crate) fn release_gate_cmd(args: ReleaseGateArgs) -> Result<u8> {
    let atlas = load_atlas(&args.atlas)?;
    let out = release_gate(&atlas);
    print_guard_output(&out, &args.format)?;
    Ok(guard_exit_code(&out))
}

pub(crate) fn build_route_atlas(repo: &Path) -> Result<Value> {
    let mut files = vec![];
    collect_atlas_files(repo, repo, &mut files)?;
    files.sort();

    let mut routes = BTreeMap::<String, RouteBuild>::new();
    for rel in &files {
        let route = commands::dogfood::auto_route_for_path(rel);
        let layer = commands::dogfood::auto_layer_for_path(rel);
        let owner = commands::dogfood::auto_owner_for_path(rel);
        let entry = routes.entry(route.clone()).or_default();
        entry.files.insert(rel.clone());
        entry.owners.insert(owner);
        entry.layers.insert(layer);
        if is_entrypoint(rel) {
            entry.entrypoints.insert(rel.clone());
        }
        if route == "test-flow" || rel.contains("test") || rel.contains("spec") {
            entry.tests.insert(rel.clone());
        }
        for check in runtime_checks_for_route(&route) {
            entry.runtime_checks.insert(check.to_string());
        }
    }

    let route_values = routes
        .iter()
        .map(|(route, build)| {
            let files = sorted_values(&build.files);
            let forbidden_routes = routes
                .keys()
                .filter(|candidate| *candidate != route)
                .cloned()
                .collect::<Vec<_>>();
            (
                route.clone(),
                json!({
                    "route_id": route,
                    "owners": sorted_values(&build.owners),
                    "layers": sorted_values(&build.layers),
                    "files": files,
                    "allowed_files": files,
                    "forbidden_routes": forbidden_routes,
                    "entrypoints": sorted_values(&build.entrypoints),
                    "tests": sorted_values(&build.tests),
                    "runtime_checks": sorted_values(&build.runtime_checks),
                    "confidence": if !files.is_empty() { 0.62 } else { 0.0 },
                    "source": "repo-route-atlas"
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    json!({
        "mode": "route-atlas",
        "core_version": CORE_VERSION,
        "input": repo.display().to_string(),
        "repo": repo.display().to_string(),
        "total_files": files.len(),
        "routes": route_values,
        "shared_contracts": shared_contracts(),
        "action_prefixes": action_prefixes(),
        "read_as": "Atlas-first route memory: use guard-action before edits and guard-diff after edits; rebuild atlas after large refactors."
    })
    .pipe(Ok)
}

fn collect_atlas_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
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
            collect_atlas_files(root, &path, out)?;
            continue;
        }
        if !is_atlas_file(&path) {
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

fn is_atlas_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if matches!(
        name,
        "Cargo.toml" | "package.json" | "pyproject.toml" | "README.md" | "Makefile"
    ) {
        return true;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext,
                "rs" | "py" | "js" | "ts" | "tsx" | "json" | "toml" | "sh"
            )
        })
}

fn is_entrypoint(path: &str) -> bool {
    path.ends_with("main.rs")
        || path.contains("src/bin/")
        || path.contains("bin/")
        || path.contains("server")
        || path.contains("daemon")
        || path.contains("cli")
}

fn runtime_checks_for_route(route: &str) -> Vec<&'static str> {
    match route {
        "ime-display-flow" => vec![
            "ime engine selected",
            "ime process alive",
            "preedit/candidate visible",
        ],
        "manual-trigger-flow" => vec![
            "manual trigger FSM test",
            "hotkey trace",
            "no unrelated IME edit",
        ],
        "space-autocorrect-flow" => vec![
            "after-space regression",
            "word buffer sync",
            "no glue regression",
        ],
        "runtime-flow" => vec!["daemon/process active", "runtime smoke", "logs checked"],
        "ui-status-flow" => vec!["UI reads backend state", "setting matches runtime config"],
        "config-flow" => vec!["config source of truth", "runtime reload/parse test"],
        "nanda-field-flow" => vec!["field eval", "no route splice", "anti-wave regression"],
        _ => vec!["route-specific test or evidence required"],
    }
}

fn action_prefixes() -> Value {
    json!({
        "ime": "ime-display-flow",
        "ibus": "ime-display-flow",
        "manual": "manual-trigger-flow",
        "double_shift": "manual-trigger-flow",
        "space": "space-autocorrect-flow",
        "autocorrect": "space-autocorrect-flow",
        "nanda": "nanda-field-flow",
        "llmwave": "nanda-field-flow",
        "runtime": "runtime-flow",
        "daemon": "runtime-flow",
        "ui": "ui-status-flow",
        "tray": "ui-status-flow",
        "config": "config-flow",
        "install": "install-flow",
        "test": "test-flow",
        "shared": "shared-contract"
    })
}

fn shared_contracts() -> Value {
    json!({
        "shared.manual_toggle_contract": {
            "allowed_routes": ["source-flow", "manual-trigger-flow", "ime-display-flow", "runtime-flow", "test-flow"],
            "shared_candidates": ["src/manual_toggle.rs", "src/manual_toggle/", "manual_toggle"],
            "reason": "manual toggle is a shared contract between input/runtime and display adapters"
        },
        "shared.text_edit_contract": {
            "allowed_routes": ["source-flow", "ime-display-flow", "space-autocorrect-flow", "runtime-flow", "test-flow"],
            "shared_candidates": ["text_edit", "edit_contract", "composition_edit", "word_buffer"],
            "reason": "text edit contract may bridge visible edit state and correction/runtime state"
        },
        "shared.candidate_contract": {
            "allowed_routes": ["source-flow", "ime-display-flow", "nanda-field-flow", "space-autocorrect-flow", "test-flow"],
            "shared_candidates": ["candidate_contract", "candidate", "suggestion"],
            "reason": "candidate contract may bridge display, scoring, and correction candidates"
        },
        "shared.layout_sync_contract": {
            "allowed_routes": ["source-flow", "ime-display-flow", "runtime-flow", "config-flow", "test-flow"],
            "shared_candidates": ["layout_sync", "layout_controller", "backend_hint"],
            "reason": "layout sync contract may bridge runtime layout state and visible adapter state"
        }
    })
}

pub(crate) fn load_atlas(path: &Path) -> Result<Value> {
    let atlas: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    if atlas["mode"].as_str() != Some("route-atlas") {
        return Err(anyhow!("not a route atlas: {}", path.display()));
    }
    Ok(atlas)
}

pub(crate) fn guard_action(
    atlas: &Value,
    symptom: &str,
    action_id: &str,
    runtime_snapshot: &Value,
) -> Value {
    let route = route_for_action(atlas, action_id);
    let route_exists = route
        .as_ref()
        .is_some_and(|route| atlas["routes"][route].is_object());
    let symptom_route = route_for_symptom(atlas, symptom);
    let mut reason_codes = vec![];
    let mut repair_queue = vec![];

    if is_hard_stop_text(symptom) {
        return json!({
            "mode": "guard-action",
            "verdict": "HARD_STOP",
            "safe_to_edit": false,
            "action_id": action_id,
            "route": route,
            "reason_codes": ["hard_stop_signal"],
            "blocked_operations": ["tools", "code", "restart", "install", "runtime_mutation"],
            "repair_queue": [{
                "kind": "hard_stop",
                "priority": "critical",
                "repair": "Stop all tool, code, restart, install, and runtime mutation operations."
            }]
        });
    }

    if action_id.trim().is_empty() || action_id.split('.').count() < 2 {
        reason_codes.push("action_id_missing_or_too_generic");
        repair_queue.push(repair(
            "action_id_missing_or_too_generic",
            "Choose a concrete action_id before editing.",
        ));
    }
    if !route_exists {
        reason_codes.push("action_route_not_in_atlas");
        repair_queue.push(repair(
            "action_route_not_in_atlas",
            "Select an action_id whose route exists in the atlas, or rebuild/curate the atlas.",
        ));
    }
    if let (Some(symptom_route), Some(route)) = (symptom_route.as_ref(), route.as_ref()) {
        if symptom_route != route && !compatible_symptom_route(symptom_route, route) {
            reason_codes.push("symptom_action_mismatch");
            repair_queue.push(repair(
                "symptom_action_mismatch",
                "Choose an action route that matches the symptom evidence before editing.",
            ));
        }
    }
    if runtime_snapshot.is_object()
        && route
            .as_deref()
            .is_some_and(|route| !route.contains("runtime") && !route.contains("ime"))
    {
        reason_codes.push("runtime_blindness");
        repair_queue.push(repair(
            "runtime_blindness",
            "Runtime evidence is present; repair or verify runtime route before code changes.",
        ));
    }

    reason_codes.sort();
    reason_codes.dedup();
    let verdict = if reason_codes
        .iter()
        .any(|code| matches!(*code, "symptom_action_mismatch" | "runtime_blindness"))
    {
        "VETO"
    } else if !reason_codes.is_empty() {
        "ANALYSIS_INSUFFICIENT"
    } else {
        "PASS"
    };
    json!({
        "mode": "guard-action",
        "verdict": verdict,
        "safe_to_edit": verdict == "PASS",
        "action_id": action_id,
        "route": route,
        "symptom_route": symptom_route,
        "reason_codes": reason_codes,
        "allowed_files": route.as_ref().map(|route| atlas["routes"][route]["allowed_files"].clone()).unwrap_or(Value::Array(vec![])),
        "forbidden_routes": route.as_ref().map(|route| atlas["routes"][route]["forbidden_routes"].clone()).unwrap_or(Value::Array(vec![])),
        "runtime_snapshot_present": runtime_snapshot.is_object(),
        "repair_queue": repair_queue,
        "read_as": "Fast action guard: symptom -> action_id -> route atlas lookup before editing."
    })
}

pub(crate) fn guard_diff(atlas: &Value, action_id: &str, diff: &str) -> Value {
    guard_diff_with_source(atlas, action_id, diff, None)
}

fn guard_diff_unreadable(
    atlas: &Value,
    action_id: &str,
    diff_source: Option<Value>,
    err: &std::io::Error,
) -> Value {
    json!({
        "mode": "guard-diff",
        "verdict": "WATCH",
        "safe_to_edit": false,
        "action_id": action_id,
        "route": route_for_action(atlas, action_id),
        "reason": "empty_or_unreadable_diff",
        "diff_source": diff_source,
        "diff_error": err.to_string(),
        "changed_files": [],
        "changed_routes": [],
        "shared_candidates": [],
        "foreign_files": [],
        "foreign_routes": [],
        "route_crossing_report": {
            "changed_routes": [],
            "shared_candidates": [],
            "suggested_shared_actions": [],
            "decision": "diff unreadable"
        },
        "repair_queue": guard_diff_repairs("WATCH", "empty_or_unreadable_diff", &[]),
        "read_as": "Diff guard: changed files must stay inside the selected route atlas capsule."
    })
}

fn guard_diff_with_source(
    atlas: &Value,
    action_id: &str,
    diff: &str,
    diff_source: Option<Value>,
) -> Value {
    let route = route_for_action(atlas, action_id);
    let mut changed = diff_changed_files(diff);
    changed.sort();
    changed.dedup();
    let shared_contract = shared_contract_for_action(atlas, action_id);
    let shared_allowed_routes = shared_contract
        .and_then(|contract| contract["allowed_routes"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let shared_candidates = shared_candidate_files(shared_contract, &changed);
    let allowed = route
        .as_ref()
        .and_then(|route| atlas["routes"][route]["allowed_files"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let changed_routes = changed
        .iter()
        .map(|file| commands::dogfood::auto_route_for_path(file))
        .collect::<BTreeSet<_>>();
    let changed_route_list = changed_routes.iter().cloned().collect::<Vec<_>>();
    let foreign_files = if shared_contract.is_some() {
        changed
            .iter()
            .filter(|file| {
                let changed_route = commands::dogfood::auto_route_for_path(file);
                !shared_allowed_routes.contains(&changed_route)
            })
            .cloned()
            .collect::<Vec<_>>()
    } else {
        changed
            .iter()
            .filter(|file| !allowed.iter().any(|allowed| path_matches(file, allowed)))
            .cloned()
            .collect::<Vec<_>>()
    };
    let foreign_routes = if shared_contract.is_some() {
        changed_routes
            .iter()
            .filter(|changed_route| !shared_allowed_routes.contains(*changed_route))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        changed_routes
            .iter()
            .filter(|changed_route| route.as_ref() != Some(*changed_route))
            .cloned()
            .collect::<Vec<_>>()
    };
    let source_mismatch = diff_source
        .as_ref()
        .and_then(|source| source["mismatch"].as_bool())
        .unwrap_or(false);
    let empty_or_unreadable = diff.trim().is_empty() || changed.is_empty();
    let shared_allows_crossing = shared_contract.is_some()
        && !changed.is_empty()
        && changed_routes
            .iter()
            .all(|changed_route| shared_allowed_routes.contains(changed_route));
    let route_crossing =
        changed_routes.len() > 1 || !foreign_files.is_empty() || !foreign_routes.is_empty();
    let verdict = if empty_or_unreadable || source_mismatch {
        "WATCH"
    } else if (route.is_none() && shared_contract.is_none())
        || (route_crossing && !shared_allows_crossing)
    {
        "VETO"
    } else {
        "PASS"
    };
    let reason = if empty_or_unreadable {
        "empty_or_unreadable_diff"
    } else if source_mismatch {
        "diff_source_repo_mismatch"
    } else if route_crossing && shared_allows_crossing {
        "shared_contract_allows_route_crossing"
    } else if route_crossing {
        "route_crossing_requires_shared_contract"
    } else if route.is_none() && shared_contract.is_none() {
        "action_route_not_in_atlas"
    } else {
        "diff_stays_inside_allowed_routes"
    };
    let suggested_shared_actions = suggested_shared_actions(atlas, &changed_routes, &changed);
    let route_crossing_decision = if verdict == "PASS" && shared_contract.is_some() {
        format!("allowed by {action_id}")
    } else if route_crossing {
        "allowed only if action_id is an explicit shared contract for these routes".to_string()
    } else {
        "single-route diff".to_string()
    };
    json!({
        "mode": "guard-diff",
        "verdict": verdict,
        "safe_to_edit": verdict == "PASS",
        "action_id": action_id,
        "route": route,
        "shared_contract": shared_contract.map(|contract| json!({
            "action_id": action_id,
            "allowed_routes": contract["allowed_routes"].clone(),
            "reason": contract["reason"].clone()
        })),
        "reason": reason,
        "diff_source": diff_source,
        "changed_files": changed,
        "changed_routes": changed_route_list,
        "shared_candidates": shared_candidates.clone(),
        "foreign_files": foreign_files,
        "foreign_routes": foreign_routes,
        "route_crossing_report": {
            "changed_routes": changed_route_list,
            "shared_candidates": shared_candidates,
            "suggested_shared_actions": suggested_shared_actions.clone(),
            "decision": route_crossing_decision
        },
        "repair_queue": guard_diff_repairs(verdict, reason, &suggested_shared_actions),
        "read_as": "Diff guard: changed files must stay inside the selected route atlas capsule."
    })
}

fn shared_contract_for_action<'a>(atlas: &'a Value, action_id: &str) -> Option<&'a Value> {
    atlas["shared_contracts"][action_id].as_object()?;
    Some(&atlas["shared_contracts"][action_id])
}

fn shared_candidate_files(contract: Option<&Value>, changed: &[String]) -> Vec<String> {
    let patterns = contract
        .and_then(|contract| contract["shared_candidates"].as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|item| item.to_ascii_lowercase())
        .collect::<Vec<_>>();
    changed
        .iter()
        .filter(|file| {
            let lower = file.to_ascii_lowercase();
            patterns.iter().any(|pattern| lower.contains(pattern))
        })
        .cloned()
        .collect()
}

fn suggested_shared_actions(
    atlas: &Value,
    changed_routes: &BTreeSet<String>,
    changed: &[String],
) -> Vec<String> {
    atlas["shared_contracts"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(action, contract)| {
            let allowed_routes = contract["allowed_routes"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            let has_shared_candidate = !shared_candidate_files(Some(contract), changed).is_empty();
            (has_shared_candidate
                && changed_routes
                    .iter()
                    .all(|route| allowed_routes.contains(route)))
            .then(|| action.clone())
        })
        .collect()
}

fn guard_diff_repairs(verdict: &str, reason: &str, suggested_shared_actions: &[String]) -> Value {
    if verdict == "PASS" {
        return json!([]);
    }
    match reason {
        "empty_or_unreadable_diff" => {
            json!([repair(
                "empty_or_unreadable_diff",
                "Provide a non-empty git diff from the target repository before allowing edits."
            )])
        }
        "diff_source_repo_mismatch" => {
            json!([repair(
                "diff_source_repo_mismatch",
                "Regenerate the diff from the same repository as the route atlas."
            )])
        }
        "route_crossing_requires_shared_contract" if !suggested_shared_actions.is_empty() => {
            json!([repair(
                "route_crossing_requires_shared_contract",
                &format!(
                    "Use one of these shared action_id values if the crossing is intentional: {}",
                    suggested_shared_actions.join(", ")
                )
            )])
        }
        _ => json!([repair(
            "side_effect_creep",
            "Split the diff or choose an action_id that owns all changed routes.",
        )]),
    }
}

fn diff_source_repo(atlas: &Value, diff_path: &Path) -> Option<Value> {
    if diff_path.to_string_lossy() == "-" {
        return None;
    }
    let atlas_repo = atlas["repo"]
        .as_str()
        .or_else(|| atlas["input"].as_str())
        .and_then(|repo| fs::canonicalize(repo).ok());
    let diff_root = git_root_for_path(diff_path);
    let mismatch = atlas_repo
        .as_ref()
        .zip(diff_root.as_ref())
        .is_some_and(|(atlas_repo, diff_root)| atlas_repo != diff_root);
    Some(json!({
        "repo": atlas_repo.as_ref().map(|path| path.display().to_string()),
        "diff_source": diff_root.as_ref().map(|path| path.display().to_string()),
        "diff_path": diff_path.display().to_string(),
        "mismatch": mismatch
    }))
}

fn git_root_for_path(path: &Path) -> Option<PathBuf> {
    let canonical = fs::canonicalize(path).ok()?;
    let mut current = if canonical.is_dir() {
        canonical
    } else {
        canonical.parent()?.to_path_buf()
    };
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn release_gate(atlas: &Value) -> Value {
    let routes = atlas["routes"].as_object().cloned().unwrap_or_default();
    let routes_without_tests = routes
        .iter()
        .filter(|(_, route)| {
            route["tests"]
                .as_array()
                .is_none_or(|items| items.is_empty())
        })
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let route_critical = routes
        .keys()
        .filter(|route| {
            matches!(
                route.as_str(),
                "manual-trigger-flow"
                    | "ime-display-flow"
                    | "runtime-flow"
                    | "space-autocorrect-flow"
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let verdict = if routes.is_empty() || !routes_without_tests.is_empty() {
        "WATCH"
    } else {
        "PASS"
    };
    json!({
        "mode": "release-gate",
        "verdict": verdict,
        "safe_to_release": verdict == "PASS",
        "routes": routes.keys().cloned().collect::<Vec<_>>(),
        "route_critical": route_critical,
        "routes_without_tests": routes_without_tests,
        "required_external_checks": ["cargo test", "cargo clippy --all-targets --all-features -- -D warnings", "route-specific runtime smoke", "nanda dogfood --refactor-plan"],
        "read_as": "Release gate is a checklist summary over the atlas; external test commands still have to run."
    })
}

fn route_for_action(atlas: &Value, action_id: &str) -> Option<String> {
    let prefix = action_id.split('.').next().unwrap_or("");
    atlas["action_prefixes"][prefix]
        .as_str()
        .map(str::to_string)
        .or_else(|| {
            let normalized = format!("{}-flow", prefix.replace('_', "-"));
            atlas["routes"][&normalized]
                .is_object()
                .then_some(normalized)
        })
}

fn route_for_symptom(atlas: &Value, symptom: &str) -> Option<String> {
    let lower = symptom.to_ascii_lowercase();
    let prefixes = atlas["action_prefixes"].as_object()?;
    for (prefix, route) in prefixes {
        if lower.contains(prefix) {
            return route.as_str().map(str::to_string);
        }
    }
    if lower.contains("double shift") || lower.contains("hotkey") {
        return Some("manual-trigger-flow".to_string());
    }
    None
}

fn compatible_symptom_route(symptom_route: &str, action_route: &str) -> bool {
    matches!(
        (symptom_route, action_route),
        ("ime-display-flow", "runtime-flow") | ("runtime-flow", "ime-display-flow")
    )
}

fn diff_changed_files(diff: &str) -> Vec<String> {
    let mut out = vec![];
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            if rest != "/dev/null" {
                out.push(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            if rest != "/dev/null" {
                out.push(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("diff --git a/") {
            if let Some((left, _)) = rest.split_once(" b/") {
                out.push(left.to_string());
            }
        }
    }
    out
}

fn path_matches(changed: &str, allowed: &str) -> bool {
    changed == allowed || changed.starts_with(allowed) || allowed.starts_with(changed)
}

fn repair(kind: &str, repair: &str) -> Value {
    json!({
        "kind": kind,
        "priority": if kind == "side_effect_creep" || kind == "symptom_action_mismatch" { "high" } else { "medium" },
        "repair": repair
    })
}

fn is_hard_stop_text(value: &str) -> bool {
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

fn sorted_values(values: &BTreeSet<String>) -> Vec<String> {
    values.iter().cloned().collect()
}

fn print_guard_output(out: &Value, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(out)?),
        OutputFormat::Text => {
            println!("mode: {}", out["mode"].as_str().unwrap_or("guard"));
            println!("verdict: {}", out["verdict"].as_str().unwrap_or("PASS"));
            if let Some(safe) = out["safe_to_edit"].as_bool() {
                println!("safe_to_edit: {safe}");
            }
            if let Some(route) = out["route"].as_str() {
                println!("route: {route}");
            }
        }
        OutputFormat::Md => {
            println!("# NANDA Guard\n");
            println!("- mode: `{}`", out["mode"].as_str().unwrap_or("guard"));
            println!("- verdict: `{}`", out["verdict"].as_str().unwrap_or("PASS"));
        }
    }
    Ok(())
}

fn guard_exit_code(out: &Value) -> u8 {
    match out["verdict"].as_str().unwrap_or("WATCH") {
        "PASS" => EXIT_PASS,
        "VETO" | "HARD_STOP" => EXIT_VETO,
        _ => EXIT_WATCH,
    }
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}
