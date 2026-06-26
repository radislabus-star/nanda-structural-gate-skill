use anyhow::Result;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::{BoundaryFacts, BoundaryPathClassifier};

pub(super) fn collect_boundary_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
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

pub(super) fn infer_single_route(
    files: &[String],
    route_for_path: BoundaryPathClassifier,
) -> Option<String> {
    let routes = files
        .iter()
        .map(|file| route_for_path(file))
        .collect::<BTreeSet<_>>();
    (routes.len() == 1)
        .then(|| routes.into_iter().next())
        .flatten()
}

pub(super) fn infer_single_owner(
    files: &[String],
    owner_for_path: BoundaryPathClassifier,
) -> Option<String> {
    let owners = files
        .iter()
        .map(|file| owner_for_path(file))
        .collect::<BTreeSet<_>>();
    (owners.len() == 1)
        .then(|| owners.into_iter().next())
        .flatten()
}

pub(super) fn filter_files_by_owner(
    files: &[String],
    owner: &str,
    owner_for_path: BoundaryPathClassifier,
) -> Vec<String> {
    let owner_norm = normalize_owner_for_match(owner);
    files
        .iter()
        .filter(|file| {
            let file_norm = normalize_owner_for_match(file);
            let auto_owner_norm = normalize_owner_for_match(&owner_for_path(file));
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

pub(super) fn collect_facts(
    root: &Path,
    files: &[String],
    target_route: Option<&str>,
    route_for_path: BoundaryPathClassifier,
    owner_for_path: BoundaryPathClassifier,
) -> BoundaryFacts {
    let mut facts = BoundaryFacts {
        files: files.to_vec(),
        ..Default::default()
    };
    let mut function_owner = BTreeMap::<String, String>::new();

    for rel in files {
        let path = root.join(rel);
        let route = route_for_path(rel);
        let owner = owner_for_path(rel);
        facts.routes.insert(route.clone());
        facts.owners.insert(owner.clone());
        facts.file_routes.insert(rel.clone(), route.clone());
        facts.file_owners.insert(rel.clone(), owner.clone());
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
        let route = route_for_path(rel);
        for (name, owner_file) in &function_owner {
            if owner_file == rel {
                continue;
            }
            if source.contains(&format!("{name}(")) {
                let other_route = route_for_path(owner_file);
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
