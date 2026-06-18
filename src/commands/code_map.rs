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
    let out = report(&args.input, args.min_cluster_functions, args.max_functions)?;
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
        .with_context(|| format!("read Rust source for code map: {}", input.display()))?;
    let functions = parse_functions(&source);
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
        let risk = if stats.deps.len() <= 2 {
            "LOW"
        } else if stats.deps.len() <= 6 {
            "MEDIUM"
        } else {
            "HIGH"
        };
        let functions = stats
            .functions
            .iter()
            .take(max_functions)
            .map(|function| {
                json!({
                    "name": function.name,
                    "line_start": function.line_start,
                    "line_end": function.line_end
                })
            })
            .collect::<Vec<_>>();
        cluster_rows.push(json!({
            "cluster": cluster,
            "functions": stats.functions.len(),
            "line_count": line_count,
            "suggested_file": suggested_file,
            "risk": risk,
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

fn parse_functions(source: &str) -> Vec<FunctionSymbol> {
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
        let cluster = classify_cluster(name);
        out.push(FunctionSymbol {
            name: name.clone(),
            line_start: idx + 1,
            line_end: next_start,
            suggested_file: suggested_file(&cluster).to_string(),
            cluster,
            body,
        });
    }
    out
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

fn classify_cluster(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    let cluster = if matches!(name, "main" | "run" | "run_check") {
        "cli-router"
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

fn suggested_file(cluster: &str) -> &str {
    match cluster {
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
