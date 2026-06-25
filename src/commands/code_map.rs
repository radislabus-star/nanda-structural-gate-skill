use anyhow::{Context, Result};
use clap::Parser;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{OutputFormat, EXIT_PASS};

#[derive(Parser)]
pub(crate) struct MapCodeArgs {
    #[arg(default_value = "src/main.rs")]
    input: PathBuf,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
    #[arg(long, default_value_t = 2)]
    min_cluster_functions: usize,
    #[arg(long, default_value_t = 20)]
    max_functions: usize,
}

#[derive(Clone)]
struct FunctionSymbol {
    name: String,
    kind: String,
    language: String,
    line_start: usize,
    line_end: usize,
    cluster: String,
    suggested_file: String,
    body: String,
}

#[derive(Default)]
struct ClusterStats {
    functions: Vec<FunctionSymbol>,
    deps: BTreeSet<String>,
}

pub(crate) fn cmd(args: MapCodeArgs) -> Result<u8> {
    let out = if args.input.is_dir() {
        repo_report(&args.input, args.min_cluster_functions, args.max_functions)?
    } else {
        report(&args.input, args.min_cluster_functions, args.max_functions)?
    };
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_text(&out),
        OutputFormat::Md => print_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn report(
    input: &Path,
    min_cluster_functions: usize,
    max_functions: usize,
) -> Result<Value> {
    let source = fs::read_to_string(input)
        .with_context(|| format!("read source for code map: {}", input.display()))?;
    let functions = parse_functions(&source, input);
    let mut by_name = BTreeMap::<String, String>::new();
    for function in &functions {
        by_name.insert(function.name.clone(), function.cluster.clone());
    }

    let mut clusters = BTreeMap::<String, ClusterStats>::new();
    for function in functions {
        clusters
            .entry(function.cluster.clone())
            .or_default()
            .functions
            .push(function);
    }

    for stats in clusters.values_mut() {
        for function in &stats.functions {
            for (name, cluster) in &by_name {
                if cluster == &function.cluster || name == &function.name {
                    continue;
                }
                if function.body.contains(&format!("{name}(")) {
                    stats.deps.insert(cluster.clone());
                }
            }
        }
    }

    let mut cluster_rows = vec![];
    for (cluster, stats) in clusters {
        if stats.functions.len() < min_cluster_functions {
            continue;
        }
        let line_count: usize = stats
            .functions
            .iter()
            .map(|function| function.line_end.saturating_sub(function.line_start) + 1)
            .sum();
        let suggested_file = stats
            .functions
            .first()
            .map(|function| function.suggested_file.as_str())
            .unwrap_or("src/main.rs");
        let route_critical = is_route_critical_file(input)
            || is_security_critical_file(input)
            || is_route_critical_cluster(&cluster);
        let risk = if route_critical {
            "HIGH"
        } else if stats.deps.len() <= 2 {
            "LOW"
        } else if stats.deps.len() <= 6 {
            "MEDIUM"
        } else {
            "HIGH"
        };
        let risk_reason = if route_critical {
            "ROUTE_CRITICAL state-machine, security hook, or kernel route boundary"
        } else if risk == "HIGH" {
            "many cross-cluster dependencies"
        } else if risk == "MEDIUM" {
            "some cross-cluster dependencies"
        } else {
            "bounded cross-cluster dependencies"
        };
        let functions = stats
            .functions
            .iter()
            .filter(|function| function.kind == "function")
            .chain(
                stats
                    .functions
                    .iter()
                    .filter(|function| function.kind != "function"),
            )
            .take(max_functions)
            .map(|function| {
                json!({
                    "name": function.name,
                    "kind": function.kind,
                    "language": function.language,
                    "line_start": function.line_start,
                    "line_end": function.line_end
                })
            })
            .collect::<Vec<_>>();
        let languages = stats
            .functions
            .iter()
            .map(|function| function.language.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let symbol_kinds = stats
            .functions
            .iter()
            .map(|function| function.kind.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        cluster_rows.push(json!({
            "cluster": cluster,
            "functions": stats.functions.len(),
            "line_count": line_count,
            "suggested_file": suggested_file,
            "risk": risk,
            "risk_reason": risk_reason,
            "languages": languages,
            "symbol_kinds": symbol_kinds,
            "external_deps": stats.deps.iter().cloned().collect::<Vec<_>>(),
            "sample_functions": functions
        }));
    }

    cluster_rows.sort_by(|left, right| {
        right["line_count"]
            .as_u64()
            .cmp(&left["line_count"].as_u64())
            .then_with(|| left["cluster"].as_str().cmp(&right["cluster"].as_str()))
    });

    let next_refactors = cluster_rows
        .iter()
        .filter(|row| {
            row["risk"].as_str() != Some("HIGH")
                && row["suggested_file"].as_str() != Some("src/main.rs")
        })
        .take(5)
        .map(|row| {
            json!({
                "cluster": row["cluster"],
                "suggested_file": row["suggested_file"],
                "risk": row["risk"],
                "why": "cohesive function cluster with bounded cross-cluster dependencies"
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "mode": "code-map",
        "input": input,
        "architecture": {
            "command_layer": "src/commands/*",
            "model_layer": "src/model.rs or domain modules",
            "cold_engine": "search/gate/feedback/report modules",
            "hot_core": "src/nanda_6m.rs",
            "cold_to_hot_bridge": "src/pack6m.rs"
        },
        "clusters": cluster_rows,
        "next_refactors": next_refactors,
        "read_as": "Use LOW/MEDIUM clusters as candidate extraction boundaries; HIGH means split the cluster or reduce dependencies first."
    }))
}

pub(crate) fn repo_report(
    repo_root: &Path,
    min_cluster_functions: usize,
    max_functions: usize,
) -> Result<Value> {
    let mut files = vec![];
    collect_code_files(repo_root, repo_root, &mut files)?;
    let total_files = files.len();
    let mut route_counts = BTreeMap::<String, usize>::new();
    for rel in &files {
        let route = auto_route_for_code_path(&rel.to_string_lossy());
        *route_counts.entry(route).or_insert(0) += 1;
    }
    let routes = route_counts.keys().cloned().collect::<Vec<_>>();
    let route_distribution = route_counts
        .iter()
        .map(|(route, count)| {
            json!({
                "route": route,
                "files": count
            })
        })
        .collect::<Vec<_>>();
    files = select_repo_files(&files, 12);
    let selected_files = files.len();

    let mut file_reports = vec![];
    let mut clusters = vec![];
    let mut next_refactors = vec![];
    for rel in files {
        let path = repo_root.join(&rel);
        let report = match report(&path, min_cluster_functions, max_functions) {
            Ok(report) => report,
            Err(_) => continue,
        };
        let cluster_count = report["clusters"].as_array().map(Vec::len).unwrap_or(0);
        let max_risk = max_cluster_risk(&report["clusters"]);
        if cluster_count > 0 {
            if let Some(items) = report["clusters"].as_array() {
                for item in items {
                    let mut item = item.clone();
                    item["input_file"] = json!(rel.display().to_string());
                    clusters.push(item);
                }
            }
            if let Some(items) = report["next_refactors"].as_array() {
                for item in items {
                    let mut item = item.clone();
                    item["input_file"] = json!(rel.display().to_string());
                    next_refactors.push(item);
                }
            }
        }
        file_reports.push(json!({
            "file": rel.display().to_string(),
            "clusters": cluster_count,
            "max_risk": max_risk,
            "rank": risk_file_rank(&rel)
        }));
    }

    clusters.sort_by(|left, right| {
        risk_sort_key(right["risk"].as_str().unwrap_or("LOW"))
            .cmp(&risk_sort_key(left["risk"].as_str().unwrap_or("LOW")))
            .then_with(|| {
                right["line_count"]
                    .as_u64()
                    .cmp(&left["line_count"].as_u64())
            })
    });
    next_refactors.truncate(8);

    Ok(json!({
        "mode": "repo-code-map",
        "input": repo_root.display().to_string(),
        "root": repo_root.display().to_string(),
        "total_files": total_files,
        "selected_files": selected_files,
        "routes": routes,
        "route_distribution": route_distribution,
        "files": file_reports,
        "clusters": clusters,
        "next_refactors": next_refactors,
        "risk_files": repo_risk_files(&clusters),
        "read_as": "Repo-level code map: inspect risk_files first, then run nanda-map-code on each target file before editing."
    }))
}

fn collect_code_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
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
            collect_code_files(root, &path, out)?;
        } else if is_code_file(&path) {
            out.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
    Ok(())
}

fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "c" | "h")
    )
}

fn auto_route_for_code_path(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if path_matches(&lower, "fs/namei.c")
        || path_matches(&lower, "include/linux/namei.h")
        || path_matches(&lower, "include/uapi/linux/openat2.h")
    {
        "kernel-vfs-path-flow".to_string()
    } else if path_matches(&lower, "fs/open.c") {
        "kernel-vfs-open-flow".to_string()
    } else if lower.contains("kernel/bpf") {
        "kernel-bpf-verifier-flow".to_string()
    } else if lower.contains("io_uring") {
        "kernel-io-uring-flow".to_string()
    } else if lower.contains("security/") || lower.contains("lsm") {
        "kernel-lsm-flow".to_string()
    } else if path_matches(&lower, "include/linux/nospec.h") {
        "kernel-nospec-flow".to_string()
    } else if lower.ends_with(".c") || lower.ends_with(".h") {
        "c-source-flow".to_string()
    } else {
        super::dogfood::auto_route_for_path(path)
    }
}

fn select_repo_files(files: &[PathBuf], max_files: usize) -> Vec<PathBuf> {
    let mut sorted = files.to_vec();
    sorted.sort_by(|left, right| {
        risk_file_rank(left)
            .cmp(&risk_file_rank(right))
            .then_with(|| left.cmp(right))
    });

    let mut selected = Vec::<PathBuf>::new();
    let mut selected_paths = BTreeSet::<String>::new();
    let mut seen_routes = BTreeSet::<String>::new();

    for rel in &sorted {
        let route = auto_route_for_code_path(&rel.to_string_lossy());
        let key = rel.to_string_lossy().to_string();
        if seen_routes.insert(route) && selected_paths.insert(key) {
            selected.push(rel.clone());
            if selected.len() >= max_files {
                return selected;
            }
        }
    }

    for rel in sorted {
        let key = rel.to_string_lossy().to_string();
        if selected_paths.insert(key) {
            selected.push(rel);
            if selected.len() >= max_files {
                break;
            }
        }
    }

    selected
}

fn risk_file_rank(path: &Path) -> usize {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    if is_exact_kernel_security_path(&lower) {
        0
    } else if is_security_critical_path(&lower) {
        1
    } else if is_route_critical_path(&lower)
        || lower.contains("dogfood")
        || lower.contains("map_gate")
        || lower.contains("report")
    {
        2
    } else if lower.ends_with("main.rs") || lower.contains("/bin/") {
        3
    } else if lower.contains("commands") || lower.contains("runtime") || lower.contains("daemon") {
        4
    } else if lower.contains("model") {
        5
    } else {
        6
    }
}

fn is_route_critical_file(path: &Path) -> bool {
    is_route_critical_path(&path.to_string_lossy().to_ascii_lowercase())
}

fn is_route_critical_path(lower: &str) -> bool {
    lower.contains("event")
        || lower.contains("fsm")
        || lower.contains("manual_trigger")
        || lower.contains("double_shift")
        || lower.contains("hotkey")
        || lower.contains("key_event")
        || lower.contains("state_machine")
}

fn is_security_critical_file(path: &Path) -> bool {
    is_security_critical_path(&path.to_string_lossy().to_ascii_lowercase())
}

fn is_security_critical_path(lower: &str) -> bool {
    lower.contains("security")
        || lower.contains("lsm")
        || lower.contains("permission")
        || lower.contains("cred")
        || lower.contains("capable")
        || lower.contains("nospec")
        || lower.contains("speculat")
        || lower.contains("sanitize")
        || lower.contains("verifier")
        || lower.contains("io_uring")
        || lower.contains("namei")
        || lower.contains("openat2")
        || path_matches(lower, "fs/open.c")
        || path_matches(lower, "fs/namei.c")
}

fn is_exact_kernel_security_path(lower: &str) -> bool {
    path_matches(lower, "fs/namei.c")
        || path_matches(lower, "fs/open.c")
        || path_matches(lower, "kernel/bpf/verifier.c")
        || path_matches(lower, "io_uring/register.c")
        || path_matches(lower, "io_uring/io_uring.c")
        || path_matches(lower, "include/linux/nospec.h")
        || path_matches(lower, "include/linux/namei.h")
        || path_matches(lower, "include/uapi/linux/openat2.h")
}

fn path_matches(lower: &str, suffix: &str) -> bool {
    lower == suffix || lower.ends_with(&format!("/{suffix}"))
}

fn is_route_critical_cluster(cluster: &str) -> bool {
    matches!(
        cluster,
        "state-machine"
            | "manual-trigger"
            | "kernel-vfs-path"
            | "kernel-vfs-open"
            | "kernel-bpf-speculation"
            | "kernel-io-uring-restrictions"
            | "kernel-lsm-security"
            | "kernel-nospec"
    )
}

fn max_cluster_risk(clusters: &Value) -> &'static str {
    let mut max = 0usize;
    if let Some(items) = clusters.as_array() {
        for item in items {
            max = max.max(risk_sort_key(item["risk"].as_str().unwrap_or("LOW")));
        }
    }
    match max {
        2 => "HIGH",
        1 => "MEDIUM",
        _ => "LOW",
    }
}

fn risk_sort_key(risk: &str) -> usize {
    match risk {
        "HIGH" => 2,
        "MEDIUM" => 1,
        _ => 0,
    }
}

fn repo_risk_files(clusters: &[Value]) -> Value {
    let mut files = BTreeMap::<String, (usize, usize)>::new();
    for cluster in clusters {
        let file = cluster["input_file"].as_str().unwrap_or("").to_string();
        if file.is_empty() {
            continue;
        }
        let risk = risk_sort_key(cluster["risk"].as_str().unwrap_or("LOW"));
        let lines = cluster["line_count"].as_u64().unwrap_or(0) as usize;
        let entry = files.entry(file).or_insert((0, 0));
        entry.0 = entry.0.max(risk);
        entry.1 += lines;
    }
    let mut rows = files
        .into_iter()
        .map(|(file, (risk, lines))| {
            json!({
                "file": file,
                "max_risk": match risk {
                    2 => "HIGH",
                    1 => "MEDIUM",
                    _ => "LOW"
                },
                "cluster_lines": lines
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        risk_sort_key(right["max_risk"].as_str().unwrap_or("LOW"))
            .cmp(&risk_sort_key(left["max_risk"].as_str().unwrap_or("LOW")))
            .then_with(|| {
                right["cluster_lines"]
                    .as_u64()
                    .cmp(&left["cluster_lines"].as_u64())
            })
    });
    json!(rows)
}

fn parse_functions(source: &str, input: &Path) -> Vec<FunctionSymbol> {
    match source_language(input) {
        "c" | "header" => parse_c_symbols(source, input),
        _ => parse_rust_functions(source, input),
    }
}

fn parse_rust_functions(source: &str, input: &Path) -> Vec<FunctionSymbol> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut starts = vec![];
    for (idx, line) in lines.iter().enumerate() {
        if let Some(name) = function_name(line) {
            starts.push((idx, name));
        }
    }

    let mut out = vec![];
    for (pos, (idx, name)) in starts.iter().enumerate() {
        let next_start = starts
            .get(pos + 1)
            .map(|(next_idx, _)| *next_idx)
            .unwrap_or(lines.len());
        let body = lines[*idx..next_start].join("\n");
        let cluster = classify_cluster(name, input);
        out.push(FunctionSymbol {
            name: name.clone(),
            kind: "function".to_string(),
            language: "rust".to_string(),
            line_start: idx + 1,
            line_end: next_start,
            suggested_file: suggested_file(&cluster, input, "rust"),
            cluster,
            body,
        });
    }
    out
}

fn parse_c_symbols(source: &str, input: &Path) -> Vec<FunctionSymbol> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut out = vec![];

    for (idx, line) in lines.iter().enumerate() {
        if let Some(name) = c_macro_name(line) {
            let cluster = classify_cluster(&name, input);
            out.push(FunctionSymbol {
                name,
                kind: "macro".to_string(),
                language: source_language(input).to_string(),
                line_start: idx + 1,
                line_end: idx + 1,
                suggested_file: suggested_file(&cluster, input, source_language(input)),
                cluster,
                body: line.to_string(),
            });
        }
    }

    let mut idx = 0usize;
    while idx < lines.len() {
        if !looks_like_c_signature_start(lines[idx]) {
            idx += 1;
            continue;
        }

        let start_idx = idx;
        let mut signature = String::new();
        let mut found_body = None;
        let max_sig_end = (idx + 8).min(lines.len());
        let mut sig_idx = idx;
        while sig_idx < max_sig_end {
            let line = lines[sig_idx].trim();
            if line.starts_with('#') {
                break;
            }
            signature.push(' ');
            signature.push_str(line);
            if line.contains(';') && !line.contains('{') {
                break;
            }
            if line.contains('{') {
                found_body = Some(sig_idx);
                break;
            }
            sig_idx += 1;
        }

        let Some(body_start) = found_body else {
            idx += 1;
            continue;
        };
        let Some(name) = c_function_name(&signature) else {
            idx += 1;
            continue;
        };
        let end = c_block_end(&lines, body_start);
        let body = lines[start_idx..end].join("\n");
        let cluster = classify_cluster(&name, input);
        out.push(FunctionSymbol {
            name,
            kind: "function".to_string(),
            language: source_language(input).to_string(),
            line_start: start_idx + 1,
            line_end: end,
            suggested_file: suggested_file(&cluster, input, source_language(input)),
            cluster,
            body,
        });
        idx = end.max(idx + 1);
    }

    out
}

fn source_language(input: &Path) -> &'static str {
    match input.extension().and_then(|ext| ext.to_str()) {
        Some("c") => "c",
        Some("h") => "header",
        _ => "rust",
    }
}

fn function_name(line: &str) -> Option<String> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }
    let trimmed = line.trim_start();
    let marker = "fn ";
    let start = trimmed.find(marker)? + marker.len();
    let rest = &trimmed[start..];
    let name = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn c_macro_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("#define ")?;
    let name = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn looks_like_c_signature_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with('*')
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.ends_with(';')
    {
        return false;
    }
    trimmed.contains('(')
        || trimmed.starts_with("static ")
        || trimmed.starts_with("__")
        || trimmed.starts_with("int ")
        || trimmed.starts_with("void ")
        || trimmed.starts_with("bool ")
        || trimmed.starts_with("long ")
        || trimmed.starts_with("unsigned ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("const ")
}

fn c_function_name(signature: &str) -> Option<String> {
    let signature = signature.trim();
    if signature.contains(" = ") || signature.contains(" typedef ") {
        return None;
    }
    if let Some(name) = syscall_macro_name(signature) {
        return Some(name);
    }
    let open = signature.find('(')?;
    let before = signature[..open].trim();
    let name = before
        .split_whitespace()
        .last()?
        .trim_start_matches('*')
        .trim_start_matches('&')
        .to_string();
    if name.is_empty()
        || name.contains('(')
        || name.contains(')')
        || c_keyword(&name)
        || name
            .chars()
            .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
    {
        return None;
    }
    Some(name)
}

fn syscall_macro_name(signature: &str) -> Option<String> {
    for prefix in [
        "SYSCALL_DEFINE",
        "COMPAT_SYSCALL_DEFINE",
        "BPF_CALL_",
        "TRACE_EVENT",
    ] {
        let Some(pos) = signature.find(prefix) else {
            continue;
        };
        let rest = &signature[pos..];
        let Some(open) = rest.find('(') else {
            continue;
        };
        let after = &rest[open + 1..];
        let name = after
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .collect::<String>();
        if !name.is_empty() {
            return Some(format!("{prefix}{name}"));
        }
    }
    None
}

fn c_keyword(name: &str) -> bool {
    matches!(
        name,
        "if" | "for" | "while" | "switch" | "return" | "sizeof" | "case"
    )
}

fn c_block_end(lines: &[&str], body_start: usize) -> usize {
    let mut depth = 0isize;
    let mut seen_open = false;
    for (idx, line) in lines.iter().enumerate().skip(body_start) {
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    seen_open = true;
                }
                '}' => {
                    depth -= 1;
                    if seen_open && depth <= 0 {
                        return idx + 1;
                    }
                }
                _ => {}
            }
        }
    }
    lines.len()
}

fn classify_cluster(name: &str, input: &Path) -> String {
    let lower = name.to_ascii_lowercase();
    let path = input.to_string_lossy().to_ascii_lowercase();
    let cluster = if path.contains("kernel/bpf")
        || lower.contains("sanitize")
        || lower.contains("speculative")
        || lower.contains("nospec")
        || lower.contains("bypass_spec")
        || lower.contains("verifier")
    {
        "kernel-bpf-speculation"
    } else if path.contains("io_uring")
        || lower.contains("io_")
        || lower.contains("uring")
        || lower.contains("restriction")
        || lower.contains("ioring")
    {
        "kernel-io-uring-restrictions"
    } else if path.contains("security/")
        || path.contains("lsm")
        || lower.starts_with("security_")
        || lower.contains("lsm")
    {
        "kernel-lsm-security"
    } else if path_matches(&path, "include/linux/nospec.h") || lower.contains("array_index_nospec")
    {
        "kernel-nospec"
    } else if path_matches(&path, "fs/namei.c")
        || path_matches(&path, "include/linux/namei.h")
        || path_matches(&path, "include/uapi/linux/openat2.h")
        || lower.contains("lookup")
        || lower.contains("dotdot")
        || lower.contains("path_connected")
        || lower.contains("jump_root")
        || lower.contains("resolve_")
        || lower.contains("lookup_")
    {
        "kernel-vfs-path"
    } else if path_matches(&path, "fs/open.c")
        || lower.contains("may_open")
        || lower.contains("vfs_open")
        || lower.contains("dentry_open")
        || lower.contains("inode_permission")
        || lower.contains("security_file_open")
    {
        "kernel-vfs-open"
    } else if matches!(name, "main" | "run" | "run_check") {
        "cli-router"
    } else if lower.contains("manual_trigger")
        || lower.contains("double_shift")
        || lower.contains("hotkey")
    {
        "manual-trigger"
    } else if lower.contains("event")
        || lower.contains("fsm")
        || lower.contains("state")
        || lower.contains("transition")
    {
        "state-machine"
    } else if lower.contains("dogfood") {
        "commands/dogfood"
    } else if lower.contains("pack6m")
        || lower.contains("packed")
        || lower.contains("budget")
        || lower.contains("triad6m")
    {
        "pack6m"
    } else if lower.contains("search")
        || lower.contains("peak")
        || lower.contains("query")
        || lower.contains("coarse")
        || lower.contains("propagation")
        || lower.contains("polarity")
    {
        "search"
    } else if lower.contains("probe")
        || lower.contains("shortcut")
        || lower.contains("feedback")
        || lower.contains("lane")
    {
        "feedback-probe"
    } else if lower.contains("dataset")
        || lower.contains("corpus")
        || lower.contains("hub")
        || lower.contains("duplicate")
        || lower.contains("distribution")
    {
        "dataset-doctor"
    } else if lower.contains("map")
        || lower.contains("topology")
        || lower.contains("hgate")
        || lower.contains("comb")
        || lower.contains("split")
        || lower.contains("route")
        || lower.contains("group")
    {
        "map-gate"
    } else if lower.contains("print")
        || lower.contains("report")
        || lower.contains("doctor")
        || lower.contains("serve")
    {
        "reporting"
    } else if lower.contains("eval") || lower.contains("waw") || lower.contains("benchmark") {
        "eval-benchmark"
    } else if lower.contains("load")
        || lower.contains("parse")
        || lower.contains("read")
        || lower.contains("write")
        || lower.contains("init")
    {
        "io"
    } else {
        "core-model"
    };
    cluster.to_string()
}

fn suggested_file(cluster: &str, input: &Path, language: &str) -> String {
    if matches!(language, "c" | "header") {
        return input.display().to_string();
    }
    match cluster {
        "manual-trigger" => "src/runtime/manual_trigger.rs",
        "state-machine" => "src/runtime/state_machine.rs",
        "commands/dogfood" => "src/commands/dogfood.rs",
        "pack6m" => "src/pack6m.rs",
        "search" => "src/search.rs",
        "feedback-probe" => "src/feedback.rs",
        "dataset-doctor" => "src/dataset_doctor.rs",
        "map-gate" => "src/map_gate.rs",
        "reporting" => "src/report.rs",
        "eval-benchmark" => "src/eval.rs",
        "io" => "src/io.rs",
        "core-model" => "src/model.rs",
        "cli-router" => "src/main.rs",
        _ => "src/main.rs",
    }
    .to_string()
}

fn print_text(out: &Value) {
    println!("NANDA CODE MAP");
    println!("input: {}", out["input"].as_str().unwrap_or(""));
    if let Some(clusters) = out["clusters"].as_array() {
        for cluster in clusters {
            println!(
                "- {}: functions {} / lines {} / risk {} / file {}",
                cluster["cluster"].as_str().unwrap_or(""),
                cluster["functions"].as_u64().unwrap_or(0),
                cluster["line_count"].as_u64().unwrap_or(0),
                cluster["risk"].as_str().unwrap_or(""),
                cluster["suggested_file"].as_str().unwrap_or("")
            );
            if let Some(deps) = cluster["external_deps"].as_array() {
                if !deps.is_empty() {
                    let deps = deps
                        .iter()
                        .filter_map(|dep| dep.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("  deps: {deps}");
                }
            }
        }
    }
    println!("next:");
    if let Some(next) = out["next_refactors"].as_array() {
        for item in next {
            println!(
                "- {} -> {} ({})",
                item["cluster"].as_str().unwrap_or(""),
                item["suggested_file"].as_str().unwrap_or(""),
                item["risk"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_md(out: &Value) {
    println!("# NANDA Code Map\n");
    println!("- input: `{}`", out["input"].as_str().unwrap_or(""));
    println!("\n## Clusters\n");
    if let Some(clusters) = out["clusters"].as_array() {
        for cluster in clusters {
            println!(
                "- `{}`: `{}` functions, `{}` lines, risk `{}`, file `{}`",
                cluster["cluster"].as_str().unwrap_or(""),
                cluster["functions"].as_u64().unwrap_or(0),
                cluster["line_count"].as_u64().unwrap_or(0),
                cluster["risk"].as_str().unwrap_or(""),
                cluster["suggested_file"].as_str().unwrap_or("")
            );
        }
    }
}
