use crate::*;

pub(crate) fn dogfood_cmd(args: DogfoodArgs) -> Result<u8> {
    if args.build_atlas {
        let out = args
            .atlas_out
            .clone()
            .unwrap_or_else(|| PathBuf::from(".nanda/route-atlas.json"));
        return commands::guard::build_atlas_cmd(commands::guard::BuildAtlasArgs {
            repo: args.input,
            out,
            format: args.format,
        });
    }
    let requested_input = args.input.clone();
    let input = resolve_dogfood_input(&args.input)?;
    let packet = load_packet_auto(
        &input,
        &args.input_format,
        "dogfood",
        "code",
        "Self-check repository structure.",
        args.normalize_paths,
    )?;
    let triads = normalize_ids(packet.triads.clone(), "t");
    let candidates = normalize_ids(packet.candidate_triads.clone(), "c");
    let branch_by = parse_csv(&args.branch_by);
    let stop_on = parse_csv(&args.stop_on);
    let topology = topology(&triads, &candidates);
    let refactor_plan = if args.refactor_plan {
        let repo_root = if requested_input.is_dir() {
            requested_input.clone()
        } else {
            input
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        };
        Some(commands::code_map::repo_report(&repo_root, 2, 12)?)
    } else {
        None
    };
    let boundary_economics = if args.boundary_economics {
        let repo_root = if requested_input.is_dir() {
            requested_input.clone()
        } else {
            input
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        };
        Some(commands::boundary::report(&repo_root, None, None)?)
    } else {
        None
    };
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
    let failure_field = codex_failure_field(&packet, &triads, &candidates);
    let mut decision = dogfood_decision(&comb_tree, &summary, &failure_field);
    if let Some(boundary_economics) = boundary_economics.as_ref() {
        apply_boundary_economics_to_decision(&mut decision, boundary_economics);
    }
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "dogfood",
        "input": input,
        "depth": args.depth,
        "branch_by": branch_by,
        "stop_on": stop_on,
        "topology": topology,
        "comb_tree": comb_tree,
        "summary": summary,
        "codex_failure_field": failure_field,
        "agent_decision": decision,
        "refactor_plan": refactor_plan,
        "boundary_economics": boundary_economics
    });
    if let Some(out_dir) = args.out_dir {
        fs::create_dir_all(&out_dir)?;
        fs::write(
            out_dir.join("dogfood.json"),
            serde_json::to_string_pretty(&out)? + "\n",
        )?;
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_dogfood_text(&out),
        OutputFormat::Md => print_dogfood_md(&out),
    }
    match out["agent_decision"]["action"].as_str().unwrap_or("") {
        "SAFE_TO_EDIT" => Ok(EXIT_PASS),
        "SPLIT_REQUIRED" | "REVIEW_REQUIRED" => Ok(EXIT_WATCH),
        "REPAIR_REQUIRED" | "HARD_STOP" => Ok(EXIT_VETO),
        _ => Ok(EXIT_ERROR),
    }
}

pub(crate) fn resolve_dogfood_input(input: &Path) -> Result<PathBuf> {
    if input.is_file() {
        return Ok(input.to_path_buf());
    }
    if input.is_dir() {
        for candidate in [
            input.join("examples/self-dogfood.nanda.json"),
            input.join("self-dogfood.nanda.json"),
            input.join(".nanda/self-dogfood.nanda.json"),
        ] {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        return write_auto_dogfood_packet(input);
    }
    Err(anyhow!(
        "dogfood input not found: pass a triad packet or a repo containing examples/self-dogfood.nanda.json"
    ))
}

fn write_auto_dogfood_packet(repo: &Path) -> Result<PathBuf> {
    let mut files = vec![];
    collect_repo_files(repo, repo, &mut files)?;
    files.sort();
    files = select_auto_field_files(files, 64);
    let mut triads = vec![];
    let mut candidates = vec![];
    for (idx, rel) in files.iter().enumerate() {
        let route = auto_route_for_path(rel);
        let layer = auto_layer_for_path(rel);
        let owner = auto_owner_for_path(rel);
        triads.push(Triad {
            id: format!("s{}", idx + 1),
            subject: rel.clone(),
            relation: "belongs_to".to_string(),
            object: route.clone(),
            evidence: rel.clone(),
            confidence: 0.55,
            subject_role: "file".to_string(),
            object_role: "route".to_string(),
            route: route.clone(),
            group: route.clone(),
            layer: layer.clone(),
            owner: owner.clone(),
            entrypoint: rel.clone(),
            output: route.clone(),
            evidence_path: rel.clone(),
            scope: "auto-scan-low-confidence".to_string(),
        });
        candidates.push(Triad {
            id: format!("c{}", idx + 1),
            subject: rel.clone(),
            relation: "belongs_to".to_string(),
            object: route.clone(),
            evidence: "repo-auto-field".to_string(),
            confidence: 0.55,
            subject_role: "file".to_string(),
            object_role: "route".to_string(),
            route: route.clone(),
            group: format!("candidate-{route}"),
            layer,
            owner,
            entrypoint: rel.clone(),
            output: route,
            evidence_path: rel.clone(),
            scope: "auto-scan-low-confidence".to_string(),
        });
    }
    let evidence = files
        .iter()
        .take(12)
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();
    let packet = Packet {
        task_id: "repo-auto-dogfood".to_string(),
        domain: "code".to_string(),
        query: "Auto-generated repository route field; review required before risky edits."
            .to_string(),
        triads,
        candidate_triads: candidates,
        candidate_answer: String::new(),
        aliases: vec![],
        canonicalization: CanonicalizationReport::default(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
        resonance_memory: vec![],
        continuation_memory: vec![],
        failure_contract: json!({
            "enabled": true,
            "symptom": "repo auto-field without curated self-dogfood packet",
            "selected_action_id": "",
            "evidence": evidence,
            "actions": [],
            "verification": {},
            "checkpoint_before_risky_change": true,
            "hypothesis_proven": false
        }),
    };
    let out_dir = std::env::temp_dir().join("nanda-structural-gate");
    fs::create_dir_all(&out_dir)?;
    let stem = slug(&repo.display().to_string());
    let out = out_dir.join(format!("repo-auto-dogfood-{stem}.json"));
    fs::write(&out, serde_json::to_string_pretty(&packet)? + "\n")?;
    Ok(out)
}

fn collect_repo_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
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
            collect_repo_files(root, &path, out)?;
            continue;
        }
        if !is_architecture_file(&path) {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        out.push(rel);
    }
    Ok(())
}

fn select_auto_field_files(files: Vec<String>, cap: usize) -> Vec<String> {
    let mut selected = vec![];
    let mut seen_routes = BTreeSet::<String>::new();
    for file in &files {
        let route = auto_route_for_path(file);
        if seen_routes.insert(route) {
            selected.push(file.clone());
            if selected.len() >= cap {
                return selected;
            }
        }
    }
    for file in files {
        if selected.iter().any(|item| item == &file) {
            continue;
        }
        selected.push(file);
        if selected.len() >= cap {
            break;
        }
    }
    selected.sort();
    selected
}

fn is_architecture_file(path: &Path) -> bool {
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

pub(crate) fn auto_route_for_path(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.contains("test") || lower.contains("spec") {
        "test-flow".to_string()
    } else if lower.contains("install") || lower.contains("deploy") || lower.contains("script") {
        "install-flow".to_string()
    } else if lower.contains("double_shift")
        || lower.contains("manual_trigger")
        || lower.contains("hotkey")
        || lower.contains("fsm")
    {
        "manual-trigger-flow".to_string()
    } else if lower.contains("space")
        || lower.contains("autocorrect")
        || lower.contains("correction")
        || lower.contains("replacement")
        || lower.contains("word_buffer")
    {
        "space-autocorrect-flow".to_string()
    } else if lower.contains("nanda")
        || lower.contains("llmwave")
        || lower.contains("wave")
        || lower.contains("l2")
        || lower.contains("l3")
    {
        "nanda-field-flow".to_string()
    } else if lower.contains("runtime") || lower.contains("daemon") || lower.contains("server") {
        "runtime-flow".to_string()
    } else if lower.contains("ibus")
        || lower.contains("ime")
        || lower.contains("preedit")
        || lower.contains("composition")
        || lower.contains("candidate")
    {
        "ime-display-flow".to_string()
    } else if lower.contains("ui") || lower.contains("status") || lower.contains("tray") {
        "ui-status-flow".to_string()
    } else if lower.contains("config") || lower.ends_with(".toml") || lower.ends_with(".json") {
        "config-flow".to_string()
    } else if lower.contains("bin/") || lower.contains("main.") || lower.contains("cli") {
        "cli-flow".to_string()
    } else {
        "source-flow".to_string()
    }
}

pub(crate) fn auto_layer_for_path(path: &str) -> String {
    match auto_route_for_path(path).as_str() {
        "test-flow" => "test",
        "install-flow" => "install",
        "manual-trigger-flow" => "runtime/input/fsm",
        "ime-display-flow" => "adapter/display",
        "space-autocorrect-flow" => "runtime/correction",
        "nanda-field-flow" => "core/field",
        "ui-status-flow" => "ui",
        "config-flow" => "config",
        "cli-flow" => "adapter",
        "runtime-flow" => "runtime",
        _ => "source",
    }
    .to_string()
}

pub(crate) fn auto_owner_for_path(path: &str) -> String {
    let raw = path
        .split('/')
        .take(3)
        .collect::<Vec<_>>()
        .join("::")
        .trim_end_matches("::")
        .to_string();
    normalize_owner_label(&raw)
}

pub(crate) fn dogfood_decision(tree: &Value, summary: &Value, failure_field: &Value) -> Value {
    let root_verdict = tree["verdict"].as_str().unwrap_or("WATCH");
    let children = tree["children"].as_array().cloned().unwrap_or_default();
    let child_count = children.len();
    let local_pass = children
        .iter()
        .filter(|child| child["verdict"].as_str() == Some("PASS"))
        .count();
    let local_watch = children
        .iter()
        .filter(|child| child["verdict"].as_str() == Some("WATCH"))
        .count();
    let local_veto = children
        .iter()
        .filter(|child| child["verdict"].as_str() == Some("VETO"))
        .count();
    let foreign_pull = summary["foreign_pull"].as_u64().unwrap_or(0);
    let invariant_violation = summary["invariant_violation"].as_u64().unwrap_or(0);
    let field_tension = tree["map"]["structural_energy"]["field_tension"]
        .as_u64()
        .unwrap_or(0);
    let owner_conflict = tree["map"]["structural_energy"]["owner_conflict"]
        .as_u64()
        .unwrap_or(0);
    let negative_route_hits = tree["map"]["negative_routes"]["hits"]
        .as_array()
        .map(|items| items.len() as u64)
        .unwrap_or(0);
    let failure_verdict = failure_field["verdict"].as_str().unwrap_or("NOT_ENABLED");
    let root_stop = tree["stop_reasons"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let root_size_only = root_stop.iter().all(|item| item == "size")
        && !tree["limits"]
            .as_array()
            .map(|items| items.is_empty())
            .unwrap_or(true);

    let (action, next) = if failure_verdict == "HARD_STOP" {
        (
            "HARD_STOP",
            "User stop signal is active: no tools, no code, no restart.",
        )
    } else if failure_verdict == "VETO" {
        (
            "REPAIR_REQUIRED",
            "Codex Failure Field vetoed the selected action; repair action/evidence/route before editing.",
        )
    } else if failure_verdict == "ANALYSIS_INSUFFICIENT" {
        (
            "REVIEW_REQUIRED",
            "Codex Failure Field says analysis is insufficient; choose a precise action_id and evidence before editing.",
        )
    } else if foreign_pull > 0
        || invariant_violation > 0
        || local_veto > 0
        || owner_conflict > 0
        || negative_route_hits > 0
    {
        (
            "REPAIR_REQUIRED",
            "Repair foreign pull, owner conflict, anti-route hit, invariant drift, or vetoed branch before editing.",
        )
    } else if child_count > 0
        && local_pass == child_count
        && root_verdict == "WATCH"
        && root_size_only
    {
        (
            "SAFE_TO_EDIT",
            "Root is size-only WATCH; use linked branch PASS results as acceptance.",
        )
    } else if child_count > 0 && local_pass == child_count && root_verdict == "PASS" {
        (
            "SAFE_TO_EDIT",
            "Root and linked branches are structurally clean.",
        )
    } else if root_verdict == "WATCH" {
        (
            "SPLIT_REQUIRED",
            "Split or narrow unresolved WATCH branches before finalizing.",
        )
    } else {
        (
            "REVIEW_REQUIRED",
            "Review the comb tree before trusting the structure.",
        )
    };

    let repair_queue = merge_repair_queues(failure_field, &tree["map"]["repair_queue"]);

    json!({
        "action": action,
        "root_verdict": root_verdict,
        "root_size_only": root_size_only,
        "safe_to_edit": action == "SAFE_TO_EDIT",
        "local_branches": child_count,
        "local_pass": local_pass,
        "local_watch": local_watch,
        "local_veto": local_veto,
        "foreign_pull": foreign_pull,
        "invariant_violation": invariant_violation,
        "field_tension": field_tension,
        "owner_conflict": owner_conflict,
        "negative_route_hits": negative_route_hits,
        "repair_queue": repair_queue,
        "codex_failure_verdict": failure_verdict,
        "codex_failure_reasons": failure_field["reason_codes"],
        "next": next
    })
}

fn apply_boundary_economics_to_decision(decision: &mut Value, boundary_economics: &Value) {
    let verdict = boundary_economics["boundary_decision"]["verdict"]
        .as_str()
        .unwrap_or("WATCH");
    let boundary_safe = boundary_economics["boundary_decision"]["safe_to_edit"]
        .as_bool()
        .unwrap_or(false);
    let reason = boundary_economics["boundary_decision"]["reason"]
        .as_str()
        .unwrap_or("");
    decision["boundary_economics_verdict"] = json!(verdict);
    decision["boundary_economics_safe_to_edit"] = json!(boundary_safe);
    decision["boundary_economics_reason"] = json!(reason);

    if decision["safe_to_edit"].as_bool() == Some(true) && !boundary_safe {
        let action = if verdict == "VETO" {
            "REPAIR_REQUIRED"
        } else {
            "REVIEW_REQUIRED"
        };
        decision["action"] = json!(action);
        decision["safe_to_edit"] = json!(false);
        decision["next"] = json!(if verdict == "VETO" {
            "Boundary Economics vetoed the refactor boundary; split by route before editing."
        } else {
            "Boundary Economics is unresolved; rerun a route-scoped boundary pass before refactoring."
        });
    }
}

pub(crate) fn print_dogfood_text(out: &Value) {
    let decision = &out["agent_decision"];
    println!("NANDA DOGFOOD");
    println!(
        "core_version: {}",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "ACTION: {}",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "ROOT: {}{}",
        decision["root_verdict"].as_str().unwrap_or("WATCH"),
        if decision["root_size_only"].as_bool().unwrap_or(false) {
            " size-only"
        } else {
            ""
        }
    );
    println!(
        "STRUCTURE: foreign_pull={} owner_conflict={} negative_route_hits={} invariant_violation={}",
        decision["foreign_pull"].as_u64().unwrap_or(0),
        decision["owner_conflict"].as_u64().unwrap_or(0),
        decision["negative_route_hits"].as_u64().unwrap_or(0),
        decision["invariant_violation"].as_u64().unwrap_or(0)
    );
    println!(
        "FAILURE_FIELD: {}",
        decision["codex_failure_verdict"]
            .as_str()
            .unwrap_or("NOT_ENABLED")
    );
    println!(
        "BRANCHES: {}/{} PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["local_branches"].as_u64().unwrap_or(0)
    );
    println!(
        "SAFE_TO_EDIT: {}",
        decision["safe_to_edit"].as_bool().unwrap_or(false)
    );
    println!("NEXT: {}", decision["next"].as_str().unwrap_or(""));
}

pub(crate) fn print_dogfood_md(out: &Value) {
    let decision = &out["agent_decision"];
    println!("# NANDA Dogfood\n");
    println!(
        "- core_version: `{}`",
        out["core_version"].as_str().unwrap_or("")
    );
    println!(
        "- action: `{}`",
        decision["action"].as_str().unwrap_or("REVIEW_REQUIRED")
    );
    println!(
        "- root: `{}`",
        decision["root_verdict"].as_str().unwrap_or("WATCH")
    );
    println!("- root_size_only: `{}`", decision["root_size_only"]);
    println!(
        "- branches: `{}/{}` PASS",
        decision["local_pass"].as_u64().unwrap_or(0),
        decision["local_branches"].as_u64().unwrap_or(0)
    );
    println!("- foreign_pull: `{}`", decision["foreign_pull"]);
    println!(
        "- codex_failure_verdict: `{}`",
        decision["codex_failure_verdict"]
    );
    println!("- owner_conflict: `{}`", decision["owner_conflict"]);
    println!(
        "- negative_route_hits: `{}`",
        decision["negative_route_hits"]
    );
    println!(
        "- invariant_violation: `{}`",
        decision["invariant_violation"]
    );
    println!("- safe_to_edit: `{}`", decision["safe_to_edit"]);
}
