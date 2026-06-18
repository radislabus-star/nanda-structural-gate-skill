use crate::*;

pub(crate) fn dogfood_cmd(args: DogfoodArgs) -> Result<u8> {
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
        let repo_root = input
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let main_rs = repo_root.join("src/main.rs");
        if main_rs.is_file() {
            Some(commands::code_map::report(&main_rs, 2, 12)?)
        } else {
            None
        }
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
    let decision = dogfood_decision(&comb_tree, &summary);
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
        "agent_decision": decision,
        "refactor_plan": refactor_plan
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
        "REPAIR_REQUIRED" => Ok(EXIT_VETO),
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
    }
    Err(anyhow!(
        "dogfood input not found: pass a triad packet or a repo containing examples/self-dogfood.nanda.json"
    ))
}

pub(crate) fn dogfood_decision(tree: &Value, summary: &Value) -> Value {
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

    let (action, next) = if foreign_pull > 0 || invariant_violation > 0 || local_veto > 0 {
        (
            "REPAIR_REQUIRED",
            "Repair foreign pull, invariant drift, or vetoed branch before editing.",
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
        "next": next
    })
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
        "STRUCTURE: foreign_pull={} invariant_violation={}",
        decision["foreign_pull"].as_u64().unwrap_or(0),
        decision["invariant_violation"].as_u64().unwrap_or(0)
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
        "- invariant_violation: `{}`",
        decision["invariant_violation"]
    );
    println!("- safe_to_edit: `{}`", decision["safe_to_edit"]);
}
