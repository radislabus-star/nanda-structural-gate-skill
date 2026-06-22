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
    let out = guard_action(&atlas, &args.symptom, &args.action_id, &runtime_snapshot);
    print_guard_output(&out, &args.format)?;
    Ok(guard_exit_code(&out))
}

pub(crate) fn guard_diff_cmd(args: GuardDiffArgs) -> Result<u8> {
    let atlas = load_atlas(&args.atlas)?;
    let diff = if args.diff.to_string_lossy() == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        fs::read_to_string(&args.diff)?
    };
    let out = guard_diff(&atlas, &args.action_id, &diff);
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
        "test": "test-flow"
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
    let route = route_for_action(atlas, action_id);
    let mut changed = diff_changed_files(diff);
    changed.sort();
    changed.dedup();
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
    let foreign_files = changed
        .iter()
        .filter(|file| !allowed.iter().any(|allowed| path_matches(file, allowed)))
        .cloned()
        .collect::<Vec<_>>();
    let foreign_routes = changed_routes
        .iter()
        .filter(|changed_route| route.as_ref() != Some(*changed_route))
        .cloned()
        .collect::<Vec<_>>();
    let verdict = if route.is_none() || !foreign_files.is_empty() || !foreign_routes.is_empty() {
        "VETO"
    } else {
        "PASS"
    };
    json!({
        "mode": "guard-diff",
        "verdict": verdict,
        "safe_to_edit": verdict == "PASS",
        "action_id": action_id,
        "route": route,
        "changed_files": changed,
        "changed_routes": changed_routes.into_iter().collect::<Vec<_>>(),
        "foreign_files": foreign_files,
        "foreign_routes": foreign_routes,
        "repair_queue": if verdict == "PASS" { json!([]) } else { json!([repair("side_effect_creep", "Split the diff or choose an action_id that owns all changed routes.")]) },
        "read_as": "Diff guard: changed files must stay inside the selected route atlas capsule."
    })
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
