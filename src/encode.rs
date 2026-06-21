use crate::*;

pub(crate) fn encode_cmd(args: EncodeArgs) -> Result<u8> {
    let text = encode_input_text(&args)?;
    let tokens = tokenize_pattern(&text);
    if tokens.is_empty() {
        return Err(anyhow!(
            "nanda encode requires --text or --text-file with tokens"
        ));
    }
    let field = encode_tokens_to_field(&tokens);
    let query_triads = tokens_to_query_triads(&tokens, &args.task_id, &args.domain);
    let mut out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "wave-pattern-encoder",
        "encoder_version": "v33-token-pattern-encoder",
        "text": text,
        "token_count": tokens.len(),
        "tokens": tokens,
        "field": field_summary(&field),
        "preview_candidate_triads": query_triads,
        "interpretation": "Text is projected into deterministic token waves, position-rotated, superposed, and exposed as candidate structural patterns."
    });

    if let Some(input) = &args.input {
        let packet = load_packet_auto(
            input,
            &args.input_format,
            &args.task_id,
            &args.domain,
            "",
            args.normalize_paths,
        )?;
        out["packet_similarity"] = packet_similarity(&field, &packet, args.top_k);
    }
    if args.as_query_packet {
        out["query_packet"] = json!(Packet {
            task_id: args.task_id.clone(),
            domain: args.domain.clone(),
            query: out["text"].as_str().unwrap_or("").to_string(),
            triads: vec![],
            candidate_triads: serde_json::from_value(out["preview_candidate_triads"].clone())?,
            candidate_answer: String::new(),
            aliases: vec![],
            canonicalization: CanonicalizationReport::default(),
            negative_shortcuts: vec![],
            positive_shortcuts: vec![],
            resonance_memory: vec![],
            continuation_memory: vec![],
            failure_contract: Value::Null,
        });
    }

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_encode_text(&out),
        OutputFormat::Md => print_encode_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn encode_input_text(args: &EncodeArgs) -> Result<String> {
    if let Some(path) = &args.text_file {
        return fs::read_to_string(path).with_context(|| format!("read {}", path.display()));
    }
    Ok(args.text.clone())
}

pub(crate) fn tokenize_pattern(text: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == ':' || ch == '/' {
            for lower in ch.to_lowercase() {
                current.push(lower);
            }
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub(crate) fn encode_tokens_to_field(tokens: &[String]) -> Vec<i32> {
    let mut field = vec![0; WAVE_DIM];
    for (idx, token) in tokens.iter().enumerate() {
        let token_wave = vector(&format!("token:{token}"));
        let position_wave = vector(&format!("slot:{}", idx % 64));
        let order_wave = vector(&format!("order:{}", idx / 64));
        let bound = bind(&bind(&token_wave, &position_wave), &order_wave);
        add_into(
            &mut field,
            &rotate(&bound, (idx * 31 + token.len() * 7) % WAVE_DIM),
        );
    }
    field
}

pub(crate) fn tokens_to_query_triads(tokens: &[String], task_id: &str, domain: &str) -> Vec<Triad> {
    let mut triads = vec![];
    if tokens.len() == 1 {
        triads.push(encoded_triad(
            "q1",
            &tokens[0],
            "activates",
            domain,
            task_id,
            0,
        ));
        return triads;
    }
    for (idx, pair) in tokens.windows(2).enumerate().take(16) {
        triads.push(encoded_triad(
            &format!("q{}", idx + 1),
            &pair[0],
            "near",
            &pair[1],
            task_id,
            idx,
        ));
    }
    if tokens.len() >= 3 {
        for (idx, triple) in tokens.windows(3).enumerate().take(8) {
            triads.push(encoded_triad(
                &format!("q{}", triads.len() + 1),
                &triple[0],
                &format!("{}:{}", triple[1], "links"),
                &triple[2],
                task_id,
                idx,
            ));
        }
    }
    triads
}

fn encoded_triad(
    id: &str,
    subject: &str,
    relation: &str,
    object: &str,
    task_id: &str,
    idx: usize,
) -> Triad {
    Triad {
        id: id.to_string(),
        subject: subject.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
        evidence: "nanda-encode".to_string(),
        confidence: 0.72,
        subject_role: "token".to_string(),
        object_role: "token".to_string(),
        route: format!("{task_id}:encoded"),
        group: format!("pattern-window-{}", idx / 4),
        layer: "adapter".to_string(),
        owner: "nanda-encode".to_string(),
        entrypoint: "encode".to_string(),
        output: "query-packet".to_string(),
        evidence_path: String::new(),
        scope: "generated-query".to_string(),
    }
}

fn field_summary(field: &[i32]) -> Value {
    let energy: i64 = field
        .iter()
        .map(|value| (*value as i64) * (*value as i64))
        .sum();
    let nonzero = field.iter().filter(|value| **value != 0).count();
    let positive = field.iter().filter(|value| **value > 0).count();
    let negative = field.iter().filter(|value| **value < 0).count();
    json!({
        "energy": energy,
        "nonzero": nonzero,
        "sparsity": round4(1.0 - (nonzero as f64 / field.len().max(1) as f64)),
        "positive": positive,
        "negative": negative,
        "signature_hex": sign_signature_hex(field),
        "top_dimensions": top_dimensions(field, 12)
    })
}

fn sign_signature_hex(field: &[i32]) -> String {
    let mut out = String::with_capacity(field.len() / 4);
    for chunk in field.chunks(4) {
        let mut nibble = 0u8;
        for (idx, value) in chunk.iter().enumerate() {
            if *value >= 0 {
                nibble |= 1 << idx;
            }
        }
        out.push_str(&format!("{nibble:x}"));
    }
    out
}

fn top_dimensions(field: &[i32], limit: usize) -> Vec<Value> {
    let mut dims = field
        .iter()
        .enumerate()
        .map(|(idx, value)| (idx, *value, value.abs()))
        .collect::<Vec<_>>();
    dims.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    dims.into_iter()
        .take(limit)
        .map(|(index, value, magnitude)| {
            json!({
                "index": index,
                "value": value,
                "magnitude": magnitude
            })
        })
        .collect()
}

fn packet_similarity(field: &[i32], packet: &Packet, top_k: usize) -> Value {
    let mut rows = normalize_ids(packet.triads.clone(), "m")
        .into_iter()
        .map(|triad| {
            let score = cosine(field, &triad_wave(&triad));
            json!({
                "id": triad.id,
                "subject": triad.subject,
                "relation": triad.relation,
                "object": triad.object,
                "route": triad.route,
                "group": triad.group,
                "score": round4(score)
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["score"].as_f64().unwrap_or(0.0))
    });
    rows.truncate(top_k.max(1));
    json!({
        "input_triads": packet.triads.len(),
        "top_k": rows.len(),
        "top_similar_triads": rows
    })
}

fn print_encode_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!("encoder: {}", out["encoder_version"].as_str().unwrap_or(""));
    println!("tokens: {}", out["token_count"].as_u64().unwrap_or(0));
    println!("energy: {}", out["field"]["energy"].as_i64().unwrap_or(0));
    println!(
        "signature: {}",
        out["field"]["signature_hex"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(32)
            .collect::<String>()
    );
}

fn print_encode_md(out: &Value) {
    println!("# NANDA Encode");
    println!();
    println!("- mode: `{}`", out["mode"].as_str().unwrap_or(""));
    println!(
        "- encoder: `{}`",
        out["encoder_version"].as_str().unwrap_or("")
    );
    println!("- tokens: `{}`", out["token_count"].as_u64().unwrap_or(0));
    println!(
        "- energy: `{}`",
        out["field"]["energy"].as_i64().unwrap_or(0)
    );
}
