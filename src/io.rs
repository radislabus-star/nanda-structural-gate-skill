use crate::*;

pub(crate) fn load_packet(path: Option<&Path>) -> Result<Packet> {
    match path {
        Some(path) => {
            let text =
                fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
            let packet: Packet =
                serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
            Ok(canonicalize_packet(packet))
        }
        None => Ok(Packet {
            task_id: "stdin-empty".to_string(),
            domain: "general".to_string(),
            query: String::new(),
            triads: vec![],
            candidate_triads: vec![],
            candidate_answer: String::new(),
            aliases: vec![],
            canonicalization: CanonicalizationReport::default(),
            negative_shortcuts: vec![],
            positive_shortcuts: vec![],
            resonance_memory: vec![],
            continuation_memory: vec![],
        }),
    }
}

pub(crate) fn load_packet_auto(
    path: &Path,
    input_format: &InputFormat,
    task_id: &str,
    domain: &str,
    query: &str,
    normalize_paths: bool,
) -> Result<Packet> {
    let is_json = match input_format {
        InputFormat::Json => true,
        InputFormat::Md => false,
        InputFormat::Auto => path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json")),
    };
    if is_json {
        load_packet(Some(path))
    } else {
        packet_from_markdown(path, task_id, domain, query, normalize_paths).map(canonicalize_packet)
    }
}

pub(crate) fn write_trace(report: &Report) -> Result<String> {
    let dir = std::env::temp_dir().join("nanda-structural-gate");
    fs::create_dir_all(&dir)?;
    let task_id = report.task_id.replace('/', "_");
    let path = dir.join(format!("{task_id}.trace.json"));
    fs::write(&path, serde_json::to_string_pretty(report)? + "\n")?;
    Ok(path.display().to_string())
}

pub(crate) fn init_task(args: InitTaskArgs) -> Result<u8> {
    let packet = Packet {
        task_id: args.task_id.clone(),
        domain: args.domain,
        query: args.query,
        triads: vec![Triad {
            id: "t1".to_string(),
            subject: String::new(),
            relation: String::new(),
            object: String::new(),
            evidence: String::new(),
            confidence: 0.9,
            subject_role: String::new(),
            object_role: String::new(),
            route: String::new(),
            group: String::new(),
        }],
        candidate_triads: vec![],
        candidate_answer: String::new(),
        aliases: vec![],
        canonicalization: CanonicalizationReport::default(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
        resonance_memory: vec![],
        continuation_memory: vec![],
    };
    let output = serde_json::to_string_pretty(&packet)? + "\n";
    write_or_print(
        args.out
            .unwrap_or_else(|| PathBuf::from(format!("nanda-task-{}.json", slug(&args.task_id)))),
        args.stdout,
        output,
    )?;
    Ok(EXIT_PASS)
}

pub(crate) fn init_md(args: InitMdArgs) -> Result<u8> {
    let text = template_text(&args.task_id, &args.domain, &args.query, &args.template);
    write_or_print(
        args.out
            .unwrap_or_else(|| PathBuf::from(format!("nanda-task-{}.md", slug(&args.task_id)))),
        args.stdout,
        text,
    )?;
    Ok(EXIT_PASS)
}

pub(crate) fn parse_arrow_triad(line: &str) -> Option<Triad> {
    let (body, attrs) = if let Some(start) = line.find('[') {
        let end = line.rfind(']').unwrap_or(line.len());
        (&line[..start], parse_attrs(&line[start + 1..end]))
    } else {
        (line, BTreeMap::new())
    };
    let parts = body
        .split("->")
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }
    Some(Triad {
        id: attrs.get("id").cloned().unwrap_or_default(),
        subject: parts[0].to_string(),
        relation: parts[1].to_string(),
        object: parts[2..].join(" -> "),
        evidence: attrs.get("evidence").cloned().unwrap_or_default(),
        confidence: attrs
            .get("confidence")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1.0),
        subject_role: attrs
            .get("subject_role")
            .cloned()
            .unwrap_or_else(|| "subject".to_string()),
        object_role: attrs
            .get("object_role")
            .cloned()
            .unwrap_or_else(|| "object".to_string()),
        route: attrs.get("route").cloned().unwrap_or_default(),
        group: attrs.get("group").cloned().unwrap_or_default(),
    })
}

pub(crate) fn parse_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

pub(crate) fn parse_markdown_tables(text: &str, normalize_paths: bool) -> (Vec<Triad>, Vec<Triad>) {
    let mut mode = "triads";
    let mut triads = vec![];
    let mut candidates = vec![];
    let mut header: Vec<String> = vec![];
    for raw in text.lines() {
        let line = raw.trim();
        let heading = line
            .trim_start_matches('#')
            .trim()
            .replace('-', "_")
            .to_lowercase();
        if matches!(heading.as_str(), "triads" | "source_triads" | "source") {
            mode = "triads";
            header.clear();
            continue;
        }
        if matches!(
            heading.as_str(),
            "candidate_triads" | "candidates" | "candidate"
        ) {
            mode = "candidate_triads";
            header.clear();
            continue;
        }
        let cells = parse_row(line);
        if cells.is_empty()
            || cells
                .iter()
                .all(|c| c.trim_matches(':').chars().all(|ch| ch == '-'))
        {
            continue;
        }
        if header.is_empty() {
            header = cells.iter().map(|c| normalize_header(c)).collect();
            continue;
        }
        let mut row = HashMap::new();
        for (key, value) in header.iter().zip(cells.iter()) {
            row.insert(key.as_str(), value.as_str());
        }
        let mut triad = Triad {
            id: row.get("id").unwrap_or(&"").to_string(),
            subject: row.get("subject").unwrap_or(&"").to_string(),
            relation: row.get("relation").unwrap_or(&"").to_string(),
            object: row.get("object").unwrap_or(&"").to_string(),
            evidence: row.get("evidence").unwrap_or(&"").to_string(),
            confidence: row
                .get("confidence")
                .and_then(|x| x.parse().ok())
                .unwrap_or(0.0),
            subject_role: row.get("subject_role").unwrap_or(&"subject").to_string(),
            object_role: row.get("object_role").unwrap_or(&"object").to_string(),
            route: row.get("route").unwrap_or(&"").to_string(),
            group: row.get("group").unwrap_or(&"").to_string(),
        };
        if normalize_paths {
            triad.subject = normalize_code_entity(&triad.subject);
            triad.object = normalize_code_entity(&triad.object);
        }
        if mode == "triads" {
            triads.push(triad);
        } else {
            candidates.push(triad);
        }
    }
    (normalize_ids(triads, "t"), normalize_ids(candidates, "c"))
}

pub(crate) fn parse_row(line: &str) -> Vec<String> {
    if !line.starts_with('|') || !line.ends_with('|') {
        return vec![];
    }
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().trim_matches('`').trim().to_string())
        .collect()
}
