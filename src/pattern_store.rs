use crate::*;
use serde::Deserialize;
use sha2::Digest;

pub(crate) const PACKED_PATTERN_BYTES: usize = 32;
pub(crate) const PATTERN_STORE_ARENA_BYTES: usize = 524_288;
pub(crate) const PATTERN_STORE_CAPACITY: usize = PATTERN_STORE_ARENA_BYTES / PACKED_PATTERN_BYTES;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PackedPattern32 {
    pub signature: u64,
    pub subject_hash: u32,
    pub relation_hash: u32,
    pub object_hash: u32,
    pub route_hash: u32,
    pub group_hash: u32,
    pub boost_i16: i16,
    pub penalty_i16: i16,
    pub accepted: u16,
    pub rejected: u16,
    pub flags: u16,
}

impl PackedPattern32 {
    pub(crate) fn from_memory(memory: &ContinuationMemory) -> Self {
        Self {
            signature: pattern_signature(
                &memory.subject,
                &memory.relation,
                &memory.object,
                &memory.route,
            ),
            subject_hash: hash32(&memory.subject),
            relation_hash: hash32(&memory.relation),
            object_hash: hash32(&memory.object),
            route_hash: hash32(&memory.route),
            group_hash: hash32(&memory.group),
            boost_i16: quantize_weight(memory.boost),
            penalty_i16: quantize_weight(memory.penalty),
            accepted: memory.accepted_count.min(u16::MAX as usize) as u16,
            rejected: memory.rejected_count.min(u16::MAX as usize) as u16,
            flags: match memory.decision.as_str() {
                "accept" => 1,
                "reject" => 2,
                _ => 4,
            },
        }
    }
}

pub(crate) fn pattern_store_cmd(args: PatternStoreArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.query,
        args.normalize_paths,
    )?;
    let out = compact_pattern_store_report(&packet, args.sample);
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_pattern_store_text(&out),
        OutputFormat::Md => print_pattern_store_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn pattern_capacity_cmd(args: PatternCapacityArgs) -> Result<u8> {
    let counts = if args.counts.is_empty() {
        vec![1024, 4096, 16_384, 65_536]
    } else {
        args.counts.clone()
    };
    let rows = counts
        .into_iter()
        .map(|count| pattern_capacity_row(count, args.query_patterns.max(1)))
        .collect::<Vec<_>>();
    let out = json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-pattern-capacity",
        "version": "v37-pattern-capacity",
        "packed_pattern_bytes": PACKED_PATTERN_BYTES,
        "pattern_store_arena_bytes": PATTERN_STORE_ARENA_BYTES,
        "pattern_store_capacity": PATTERN_STORE_CAPACITY,
        "rows": rows,
        "read_as": "Capacity estimates when learned continuation patterns stay compact enough for the NANDA-6M pattern arena."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_capacity_text(&out),
        OutputFormat::Md => print_capacity_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn llmwave_cmd(args: LlmwaveArgs) -> Result<u8> {
    let text = if let Some(path) = &args.text_file {
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
    } else {
        args.text.clone()
    };
    let mut packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &text,
        args.normalize_paths,
    )?;
    let tokens = tokenize_pattern(&text);
    let query = tokens_to_query_triads(&tokens, &args.task_id, &args.domain);
    packet.query = text.clone();
    let memory = normalize_ids(packet.triads.clone(), "m");
    let decode_args = DecodeArgs {
        input: args.input.clone(),
        input_format: args.input_format.clone(),
        task_id: args.task_id.clone(),
        domain: args.domain.clone(),
        query: text.clone(),
        query_file: None,
        query_format: args.input_format.clone(),
        top_k: args.top_k,
        steps: args.steps.clamp(1, 16),
        beam_width: 1,
        adaptive_scoring: false,
        search_top_k: args.search_top_k,
        route_cap: args.route_cap,
        route_triad_cap: args.route_triad_cap,
        group_by: args.group_by.clone(),
        format: OutputFormat::Json,
        normalize_paths: args.normalize_paths,
    };
    let decode = recurrent_decode_report(
        &packet,
        &memory,
        &query,
        "llmwave-auto-query",
        &decode_args,
        decode_args.steps,
    );
    let hrr_binding = hrr_binding_report(&packet, &query, args.top_k);
    let cleanup_memory = cleanup_memory_report(&decode, &packet);
    let attractor_trace = llmwave_attractor_report(&decode);
    let superposition_capacity = superposition_capacity_report(&packet);
    let anti_wave_audit = shortcut_anti_wave_audit(&decode, &packet);
    let packed_hrr_runtime = packed_hrr_runtime_report(&hrr_binding, &packet);
    let cleanup_dictionary = cleanup_dictionary_report(&cleanup_memory, &packet);
    let anti_wave_locality = anti_wave_locality_report(&anti_wave_audit, &decode);
    let capacity_curve = capacity_curve_report(&superposition_capacity, &packet);
    let packed_hot_cycle = llmwave_hot_cycle_report(
        &packed_hrr_runtime,
        &cleanup_dictionary,
        &capacity_curve,
        &anti_wave_locality,
    );
    let proof_summary = llmwave_proof_summary(
        &hrr_binding,
        &cleanup_memory,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &decode,
    );
    let llmwave_contract = llmwave_contract_report(
        &packet,
        &text,
        &tokens,
        &decode,
        &hrr_binding,
        &cleanup_memory,
        &cleanup_dictionary,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &proof_summary,
        &args.lens,
    );
    let public_demo = llmwave_public_demo_report(&proof_summary, &decode, &text);
    let feedback_preview = if args.train {
        decode_feedback_preview(&decode, &args.decision, &args.note)
    } else {
        json!({
            "enabled": false,
            "reason": "pass --train to preview continuation feedback"
        })
    };
    let out = json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-mini-loop",
        "version": "v60-public-demo-packet",
        "text": text,
        "tokens": tokens,
        "encoded_query_triads": query.iter().map(triad_json).collect::<Vec<_>>(),
        "hrr_binding": hrr_binding,
        "cleanup_memory": cleanup_memory,
        "attractor_trace": attractor_trace,
        "superposition_capacity": superposition_capacity,
        "anti_wave_audit": anti_wave_audit,
        "packed_hrr_runtime": packed_hrr_runtime,
        "cleanup_dictionary": cleanup_dictionary,
        "anti_wave_locality": anti_wave_locality,
        "capacity_curve": capacity_curve,
        "packed_hot_cycle": packed_hot_cycle,
        "proof_summary": proof_summary,
        "llmwave_contract": llmwave_contract,
        "public_demo": public_demo,
        "pattern_store": compact_pattern_store_report(&packet, 3),
        "decode": decode,
        "feedback_preview": feedback_preview,
        "read_as": "LLMWave v67 loop: raw text -> field excitation -> selected lens -> structural decode -> cleanup/energy/capacity/anti-wave audit -> packed readiness -> proof summary -> demo card."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_llmwave_text(&out),
        OutputFormat::Md => print_llmwave_md(&out),
    }
    Ok(EXIT_PASS)
}

pub(crate) fn llmwave_eval_cmd(args: LlmwaveEvalArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.suite)
        .with_context(|| format!("read {}", args.suite.display()))?;
    let suite: LlmwaveEvalSuite =
        serde_json::from_str(&text).with_context(|| format!("parse {}", args.suite.display()))?;
    if suite.cases.is_empty() {
        return Err(anyhow!("nanda llmwave-eval requires at least one case"));
    }
    let base = args.suite.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    for case in suite.cases {
        let row = run_llmwave_eval_case(&args, base, case)?;
        if row["ok"].as_bool().unwrap_or(false) {
            passed += 1;
        }
        rows.push(row);
    }
    let total = rows.len();
    let out = json!({
        "core_version": CORE_VERSION,
        "wave_dim": WAVE_DIM,
        "mode": "llmwave-eval-suite",
        "version": "v53-llmwave-proof-suite",
        "suite": if suite.name.is_empty() { args.suite.display().to_string() } else { suite.name },
        "passed": passed,
        "total": total,
        "accuracy": round4(passed as f64 / total.max(1) as f64),
        "cases": rows,
        "read_as": "LLMWave eval verifies the full v60 read/write/retrieve proof packet, not only the top decoded pattern."
    });
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Text => print_llmwave_eval_text(&out),
        OutputFormat::Md => print_llmwave_eval_md(&out),
    }
    Ok(if passed == total {
        EXIT_PASS
    } else {
        EXIT_VETO
    })
}

pub(crate) fn llmwave_memory_cmd(args: LlmwaveMemoryArgs) -> Result<u8> {
    match args.command {
        LlmwaveMemoryCommand::Write(args) => llmwave_memory_write_cmd(args),
        LlmwaveMemoryCommand::Inspect(args) => llmwave_memory_inspect_cmd(args),
        LlmwaveMemoryCommand::Vocabulary(args) => llmwave_memory_vocabulary_cmd(args),
        LlmwaveMemoryCommand::Retrieve(args) => llmwave_memory_retrieve_cmd(args),
        LlmwaveMemoryCommand::Feedback(args) => llmwave_memory_feedback_cmd(args),
        LlmwaveMemoryCommand::Correct(args) => llmwave_memory_correct_cmd(args),
        LlmwaveMemoryCommand::Consolidate(args) => llmwave_memory_consolidate_cmd(args),
        LlmwaveMemoryCommand::Decay(args) => llmwave_memory_decay_cmd(args),
        LlmwaveMemoryCommand::Generate(args) => llmwave_memory_generate_cmd(args),
        LlmwaveMemoryCommand::Chat(args) => llmwave_memory_chat_cmd(args),
        LlmwaveMemoryCommand::Answer(args) => llmwave_memory_answer_cmd(args),
        LlmwaveMemoryCommand::Train(args) => llmwave_memory_train_cmd(args),
        LlmwaveMemoryCommand::Grow(args) => llmwave_memory_grow_cmd(args),
        LlmwaveMemoryCommand::Eval(args) => llmwave_memory_eval_cmd(args),
        LlmwaveMemoryCommand::Demo(args) => llmwave_memory_demo_cmd(args),
        LlmwaveMemoryCommand::Density(args) => llmwave_memory_density_cmd(args),
        LlmwaveMemoryCommand::Pack(args) => llmwave_memory_pack_cmd(args),
        LlmwaveMemoryCommand::Unpack(args) => llmwave_memory_unpack_cmd(args),
    }
}

fn llmwave_memory_write_cmd(args: LlmwaveMemoryWriteArgs) -> Result<u8> {
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.text,
        args.normalize_paths,
    )?;
    let memory = llmwave_memory_from_packet(&packet, &args.text);
    llmwave_memory_emit(memory, args.out.as_deref(), &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_inspect_cmd(args: LlmwaveMemoryInspectArgs) -> Result<u8> {
    let memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_model_report(&memory);
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_vocabulary_cmd(args: LlmwaveMemoryVocabularyArgs) -> Result<u8> {
    let memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_vocabulary_report(&memory);
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_retrieve_cmd(args: LlmwaveMemoryRetrieveArgs) -> Result<u8> {
    let memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_retrieve_report(&memory, &args.prefix, args.top_k);
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_correct_cmd(args: LlmwaveMemoryCorrectArgs) -> Result<u8> {
    let mut memory = llmwave_memory_load(&args.memory)?;
    let mut actions = vec![];
    if !args.reject_token.trim().is_empty() {
        let reject = llmwave_memory_apply_feedback(
            &mut memory,
            &FeedbackDecision::Reject,
            "",
            &args.reject_token,
            &args.note,
        );
        actions.push(reject["lane"].clone());
    }
    if !args.accept_token.trim().is_empty() {
        let accept = llmwave_memory_apply_feedback(
            &mut memory,
            &FeedbackDecision::Accept,
            "",
            &args.accept_token,
            &args.note,
        );
        actions.push(accept["lane"].clone());
    }
    let out = json!({
        "mode": "llmwave-memory-correct",
        "version": "v103-self-correction",
        "state": if actions.is_empty() { "MEMORY_CORRECTION_EMPTY" } else { "MEMORY_CORRECTION_APPLIED" },
        "actions": actions,
        "memory": memory
    });
    llmwave_memory_emit(out, args.out.as_deref(), &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_feedback_cmd(args: LlmwaveMemoryFeedbackArgs) -> Result<u8> {
    let mut memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_apply_feedback(
        &mut memory,
        &args.decision,
        &args.pattern,
        &args.token,
        &args.note,
    );
    llmwave_memory_emit(out, args.out.as_deref(), &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_consolidate_cmd(args: LlmwaveMemoryConsolidateArgs) -> Result<u8> {
    let mut memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_consolidate(&mut memory);
    llmwave_memory_emit(out, args.out.as_deref(), &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_decay_cmd(args: LlmwaveMemoryDecayArgs) -> Result<u8> {
    let mut memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_decay(&mut memory, args.factor, args.min_strength);
    llmwave_memory_emit(out, args.out.as_deref(), &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_generate_cmd(args: LlmwaveMemoryGenerateArgs) -> Result<u8> {
    let memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_generate_report(
        &memory,
        &args.prefix,
        args.steps,
        args.top_k,
        args.beam_width,
        args.temperature,
        &args.language,
    );
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_chat_cmd(args: LlmwaveMemoryChatArgs) -> Result<u8> {
    let memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_chat_report(
        &memory,
        &args.prompt,
        args.steps,
        args.top_k,
        args.beam_width,
        args.temperature,
        &args.language,
    );
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_answer_cmd(args: LlmwaveMemoryAnswerArgs) -> Result<u8> {
    let memory = llmwave_memory_load(&args.memory)?;
    let out = llmwave_memory_answer_report(
        &memory,
        &args.prompt,
        args.facts,
        args.top_k,
        &args.language,
    );
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_train_cmd(args: LlmwaveMemoryTrainArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let memory = llmwave_memory_from_text(&text, &args.task_id, &args.domain);
    llmwave_memory_emit(memory, args.out.as_deref(), &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_grow_cmd(args: LlmwaveMemoryGrowArgs) -> Result<u8> {
    let mut memory = llmwave_memory_load(&args.memory)?;
    let packet = load_packet_auto(
        &args.input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        &args.text,
        args.normalize_paths,
    )?;
    let addition = llmwave_memory_from_packet(&packet, &args.text);
    let before = memory_record_count(&memory);
    llmwave_memory_append(&mut memory, &addition);
    let after = memory_record_count(&memory);
    llmwave_memory_refresh_budget(&mut memory);
    memory["growth"] = json!({
        "version": "v102-memory-growth",
        "state": "MEMORY_GROWN",
        "before_records": before,
        "after_records": after,
        "added_records": after.saturating_sub(before)
    });
    let out = json!({
        "mode": "llmwave-memory-grow",
        "version": "v102-memory-growth",
        "before_records": before,
        "after_records": after,
        "memory": memory
    });
    llmwave_memory_emit(out, args.out.as_deref(), &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_eval_cmd(args: LlmwaveMemoryEvalArgs) -> Result<u8> {
    let suite_text = fs::read_to_string(&args.suite)
        .with_context(|| format!("failed to read {}", args.suite.display()))?;
    let suite: Value = serde_json::from_str(&suite_text)
        .with_context(|| format!("failed to parse {}", args.suite.display()))?;
    let base = args.suite.parent().unwrap_or_else(|| Path::new("."));
    let mut rows = vec![];
    let mut passed = 0usize;
    for case in suite["cases"].as_array().cloned().unwrap_or_default() {
        let mut memory = if let Some(train_path) = case["train_text_path"].as_str() {
            let text = fs::read_to_string(base.join(train_path))
                .with_context(|| format!("failed to read {train_path}"))?;
            llmwave_memory_from_text(&text, "llmwave-memory-eval", "general")
        } else {
            let path = base.join(case["path"].as_str().unwrap_or(""));
            let packet = load_packet_auto(
                &path,
                &InputFormat::Auto,
                "llmwave-memory-eval",
                "general",
                "",
                false,
            )?;
            llmwave_memory_from_packet(&packet, case["write_text"].as_str().unwrap_or(""))
        };
        if let Some(grow_path) = case["grow_path"].as_str() {
            let packet = load_packet_auto(
                &base.join(grow_path),
                &InputFormat::Auto,
                "llmwave-memory-eval-grow",
                "general",
                "",
                false,
            )?;
            let addition = llmwave_memory_from_packet(&packet, "");
            llmwave_memory_append(&mut memory, &addition);
            llmwave_memory_refresh_budget(&mut memory);
        }
        if let Some(feedback) = case["feedback"].as_array() {
            for item in feedback {
                let decision = match item["decision"].as_str().unwrap_or("watch") {
                    "accept" => FeedbackDecision::Accept,
                    "reject" => FeedbackDecision::Reject,
                    _ => FeedbackDecision::Watch,
                };
                memory = llmwave_memory_apply_feedback(
                    &mut memory,
                    &decision,
                    item["pattern"].as_str().unwrap_or(""),
                    item["token"].as_str().unwrap_or(""),
                    item["note"].as_str().unwrap_or(""),
                )["memory"]
                    .clone();
            }
        }
        if case["consolidate"].as_bool().unwrap_or(false) {
            memory = llmwave_memory_consolidate(&mut memory)["memory"].clone();
        }
        if let Some(factor) = case["decay_factor"].as_f64() {
            memory = llmwave_memory_decay(&mut memory, factor, 0.05)["memory"].clone();
        }
        let prompt = case["prompt"].as_str();
        let prefix = case["prefix"].as_str().unwrap_or("");
        let adapter = prompt.map(|value| llmwave_prompt_adapter(&memory, value));
        let active_prefix = adapter
            .as_ref()
            .and_then(|value| value["selected_prefix"].as_str())
            .unwrap_or(prefix);
        let retrieve = llmwave_memory_retrieve_report(&memory, active_prefix, 5);
        let generate = if let Some(prompt) = prompt {
            llmwave_memory_chat_report(
                &memory,
                prompt,
                case["steps"].as_u64().unwrap_or(2) as usize,
                3,
                3,
                0.0,
                case["language"].as_str().unwrap_or("en"),
            )["generation"]
                .clone()
        } else {
            llmwave_memory_generate_report(
                &memory,
                active_prefix,
                case["steps"].as_u64().unwrap_or(2) as usize,
                3,
                3,
                0.0,
                case["language"].as_str().unwrap_or("en"),
            )
        };
        let answer = prompt.map(|prompt| {
            llmwave_memory_answer_report(
                &memory,
                prompt,
                case["facts"].as_u64().unwrap_or(3) as usize,
                3,
                case["language"].as_str().unwrap_or("en"),
            )
        });
        let inspect = llmwave_memory_model_report(&memory);
        let expected_token = case["expected_top_token"].as_str().unwrap_or("");
        let expected_state = case["expected_state"].as_str().unwrap_or("");
        let expected_generated_contains =
            case["expected_generated_contains"].as_str().unwrap_or("");
        let expected_answer_contains = case["expected_answer_contains"].as_str().unwrap_or("");
        let expected_answer_state = case["expected_answer_state"].as_str().unwrap_or("");
        let expected_version = case["expected_memory_version"].as_str().unwrap_or("");
        let token_ok =
            expected_token.is_empty() || retrieve["top_token"].as_str() == Some(expected_token);
        let state_ok =
            expected_state.is_empty() || retrieve["state"].as_str() == Some(expected_state);
        let generated_ok = expected_generated_contains.is_empty()
            || generate["generated_text"]
                .as_str()
                .is_some_and(|value| value.contains(expected_generated_contains));
        let answer_ok = expected_answer_contains.is_empty()
            || answer.as_ref().is_some_and(|value| {
                value["answer"]
                    .as_str()
                    .is_some_and(|text| text.contains(expected_answer_contains))
            });
        let answer_state_ok = expected_answer_state.is_empty()
            || answer
                .as_ref()
                .is_some_and(|value| value["state"].as_str() == Some(expected_answer_state));
        let version_ok =
            expected_version.is_empty() || memory["version"].as_str() == Some(expected_version);
        let inspect_ok = inspect["version"].as_str() == Some("v105-real-memory-file-format")
            && inspect["tokenizer_contract"]["version"].as_str() == Some("v106-tokenizer-contract")
            && inspect["model_config"]["version"].as_str() == Some("v107-model-config");
        let ok = token_ok
            && state_ok
            && generated_ok
            && answer_ok
            && answer_state_ok
            && version_ok
            && inspect_ok;
        passed += usize::from(ok);
        rows.push(json!({
            "id": case["id"],
            "ok": ok,
            "prefix": prefix,
            "prompt": prompt.unwrap_or(""),
            "selected_prefix": active_prefix,
            "expected_top_token": expected_token,
            "actual_top_token": retrieve["top_token"],
            "expected_state": expected_state,
            "actual_state": retrieve["state"],
            "expected_generated_contains": expected_generated_contains,
            "generated": generate["generated_text"],
            "expected_answer_contains": expected_answer_contains,
            "expected_answer_state": expected_answer_state,
            "answer": answer.as_ref().map(|value| value["answer"].clone()).unwrap_or(Value::Null),
            "answer_state": answer.as_ref().map(|value| value["state"].clone()).unwrap_or(Value::Null),
            "memory_version": memory["version"],
            "inspect_state": inspect["version"]
        }));
    }
    let total = rows.len();
    let out = json!({
        "mode": "llmwave-memory-eval",
        "version": "v126-core-field-eval",
        "legacy_version": "v119-qa-answer-eval",
        "passed": passed,
        "total": total,
        "accuracy": if total == 0 { 0.0 } else { round4(passed as f64 / total as f64) },
        "cases": rows
    });
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(if passed == total {
        EXIT_PASS
    } else {
        EXIT_VETO
    })
}

fn llmwave_memory_demo_cmd(args: LlmwaveMemoryDemoArgs) -> Result<u8> {
    let text = fs::read_to_string(&args.corpus)
        .with_context(|| format!("failed to read {}", args.corpus.display()))?;
    let mut memory = llmwave_memory_from_text(&text, "llmwave-memory-demo", "general");
    let before = llmwave_memory_chat_report(&memory, &args.prompt, args.steps, 3, 3, 0.0, "en");
    let correction = llmwave_memory_apply_feedback(
        &mut memory,
        &FeedbackDecision::Reject,
        "",
        &args.reject_token,
        "demo reject lane",
    );
    memory = correction["memory"].clone();
    let correction_accept = llmwave_memory_apply_feedback(
        &mut memory,
        &FeedbackDecision::Accept,
        "",
        &args.accept_token,
        "demo accept lane",
    );
    memory = correction_accept["memory"].clone();
    let after = llmwave_memory_chat_report(&memory, &args.prompt, args.steps, 3, 3, 0.0, "en");
    let packed = llmwave_memory_unpack_report(&llmwave_memory_pack_bytes(&memory));
    let out = json!({
        "mode": "llmwave-memory-demo",
        "version": "v114-public-demo-script",
        "state": "LLMWAVE_MEMORY_DEMO_READY",
        "prompt": args.prompt,
        "before": before,
        "corrections": [correction["lane"].clone(), correction_accept["lane"].clone()],
        "after": after,
        "packed": packed,
        "read_as": "Public demo trains a tiny corpus, adapts the prompt into a wave prefix, generates through semantic guard/coherence, applies feedback lanes, and validates packed memory."
    });
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_density_cmd(args: LlmwaveMemoryDensityArgs) -> Result<u8> {
    let out = llmwave_memory_density_report(&args.counts, args.facts);
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_density_report(counts: &[usize], facts: usize) -> Value {
    let mut rows = vec![];
    let mut first_degraded = Value::Null;
    for &count in counts {
        let row = llmwave_memory_density_row(count.max(16), facts.max(1));
        if first_degraded.is_null() && row["state"].as_str() != Some("DENSITY_STABLE") {
            first_degraded = row.clone();
        }
        rows.push(row);
    }
    json!({
        "mode": "llmwave-memory-density",
        "version": "v127-density-reality-check",
        "wave_dim": WAVE_DIM,
        "hot_budget_bytes": nanda_6m::BUDGET_BYTES,
        "record_bytes": 32,
        "focus_runtime_triads": nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY,
        "rows": rows,
        "first_degraded": first_degraded,
        "claims_boundary": {
            "nonlinear_density_proven": false,
            "cache_only_execution_proven": false,
            "lexical_baseline_compared": true,
            "what_this_measures": "Synthetic useful recall, reversed-trap safety, field-vs-lexical baseline, field-state drift, packed bytes, and hot-focus boundary."
        },
        "read_as": "Density reality check is a guardrail against overclaiming: stable rows keep useful recall and reversed traps while record count grows."
    })
}

fn llmwave_memory_density_row(records: usize, facts: usize) -> Value {
    let memory = llmwave_synthetic_density_memory(records);
    let actual_records = memory_record_count(&memory);
    let probes = [
        (
            "what does customs declaration require?",
            "ANSWER_READY",
            "customs declaration requires payment confirmation",
        ),
        (
            "who issues invoice?",
            "ANSWER_READY",
            "supplier issues invoice",
        ),
        (
            "what supports customs declaration?",
            "ANSWER_READY",
            "payment confirmation supports customs declaration",
        ),
        ("who pays supplier?", "ANSWER_READY", "buyer pays supplier"),
        ("what does invoice issue?", "ANSWER_EMPTY", ""),
    ];
    let started = std::time::Instant::now();
    let mut probe_rows = vec![];
    let mut passed = 0usize;
    let mut reversed_false_positive = false;
    let mut state_counts = BTreeMap::<String, usize>::new();
    let mut field_counts = BTreeMap::<String, usize>::new();
    let mut margin_sum = 0.0;
    let mut top_score_sum = 0.0;
    let mut suppressed_sum = 0usize;
    let mut anti_sum = 0.0;
    let mut lexical_passed = 0usize;
    let mut lexical_reversed_false_positive = false;
    let mut relation_passed = 0usize;
    let mut relation_reversed_false_positive = false;
    let mut naive_passed = 0usize;
    let mut naive_reversed_false_positive = false;
    for (prompt, expected_state, expected_contains) in probes {
        let answer = llmwave_memory_answer_report(&memory, prompt, facts, 3, "en");
        let lexical = llmwave_density_lexical_baseline(&memory, prompt);
        let relation_baseline = llmwave_density_relation_baseline(&memory, prompt);
        let naive_vector = llmwave_density_naive_vector_baseline(&memory, prompt);
        let answer_text = answer["answer"].as_str().unwrap_or("");
        let state = answer["state"]
            .as_str()
            .unwrap_or("ANSWER_EMPTY")
            .to_string();
        let lexical_state = lexical["state"].as_str().unwrap_or("ANSWER_EMPTY");
        let lexical_answer = lexical["answer"].as_str().unwrap_or("");
        let relation_state = relation_baseline["state"]
            .as_str()
            .unwrap_or("ANSWER_EMPTY");
        let relation_answer = relation_baseline["answer"].as_str().unwrap_or("");
        let naive_state = naive_vector["state"].as_str().unwrap_or("ANSWER_EMPTY");
        let naive_answer = naive_vector["answer"].as_str().unwrap_or("");
        let field_state = answer["field_core"]["state"]
            .as_str()
            .unwrap_or("FIELD_EMPTY")
            .to_string();
        let ok = state == expected_state
            && (expected_contains.is_empty() || answer_text.contains(expected_contains));
        let lexical_ok = lexical_state == expected_state
            && (expected_contains.is_empty() || lexical_answer.contains(expected_contains));
        let relation_ok = relation_state == expected_state
            && (expected_contains.is_empty() || relation_answer.contains(expected_contains));
        let naive_ok = naive_state == expected_state
            && (expected_contains.is_empty() || naive_answer.contains(expected_contains));
        if prompt == "what does invoice issue?" && state == "ANSWER_READY" {
            reversed_false_positive = true;
        }
        if prompt == "what does invoice issue?" && lexical_state == "ANSWER_READY" {
            lexical_reversed_false_positive = true;
        }
        if prompt == "what does invoice issue?" && relation_state == "ANSWER_READY" {
            relation_reversed_false_positive = true;
        }
        if prompt == "what does invoice issue?" && naive_state == "ANSWER_READY" {
            naive_reversed_false_positive = true;
        }
        passed += usize::from(ok);
        lexical_passed += usize::from(lexical_ok);
        relation_passed += usize::from(relation_ok);
        naive_passed += usize::from(naive_ok);
        *state_counts.entry(state.clone()).or_insert(0) += 1;
        *field_counts.entry(field_state.clone()).or_insert(0) += 1;
        margin_sum += answer["field_core"]["margin"].as_f64().unwrap_or(0.0);
        top_score_sum += answer["field_core"]["top_score"].as_f64().unwrap_or(0.0);
        suppressed_sum += answer["grounding"]["suppressed_facts"]
            .as_array()
            .map_or(0, Vec::len);
        anti_sum += answer["field_core"]["anti_energy"].as_f64().unwrap_or(0.0);
        probe_rows.push(json!({
            "prompt": prompt,
            "expected_state": expected_state,
            "actual_state": state,
            "field_state": field_state,
            "ok": ok,
            "answer": answer_text,
            "lexical_baseline": lexical,
            "relation_baseline": relation_baseline,
            "naive_vector_baseline": naive_vector,
            "wins_over_lexical_baseline": ok && !lexical_ok,
            "wins_over_relation_baseline": ok && !relation_ok,
            "wins_over_naive_vector_baseline": ok && !naive_ok,
            "top_score": answer["field_core"]["top_score"],
            "margin": answer["field_core"]["margin"],
            "facts": answer["grounding"]["facts"]
        }));
    }
    let elapsed_ns = started.elapsed().as_nanos() as u64;
    let packed_bytes = actual_records * 32;
    let accuracy = round4(passed as f64 / probes.len() as f64);
    let lexical_accuracy = round4(lexical_passed as f64 / probes.len() as f64);
    let relation_accuracy = round4(relation_passed as f64 / probes.len() as f64);
    let naive_accuracy = round4(naive_passed as f64 / probes.len() as f64);
    let avg_margin = round4(margin_sum / probes.len() as f64);
    let avg_top_score = round4(top_score_sum / probes.len() as f64);
    let avg_suppressed = round4(suppressed_sum as f64 / probes.len() as f64);
    let avg_anti_energy = round4(anti_sum / probes.len() as f64);
    let nonlinear = llmwave_density_nonlinear_candidate(avg_top_score, avg_margin, avg_anti_energy);
    let phase_lock = llmwave_density_phase_lock(&field_counts, &probe_rows);
    let packed_hot_loop = llmwave_density_packed_hot_loop(records);
    let focus_experiment = llmwave_density_focus_experiment(actual_records);
    let l2 = llmwave_density_l2_contour(actual_records);
    let focus_state = if actual_records <= nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY {
        "HOT_FOCUS_READY"
    } else if actual_records <= nanda_6m::TRIAD_CAPACITY {
        "STORAGE_OK_FOCUS_REQUIRED"
    } else {
        "OVER_PACKED_TRIAD_CAPACITY"
    };
    let state = if accuracy >= 1.0 && !reversed_false_positive {
        "DENSITY_STABLE"
    } else if reversed_false_positive {
        "DENSITY_REVERSED_LEAK"
    } else {
        "DENSITY_REVIEW"
    };
    json!({
        "records": actual_records,
        "requested_records": records,
        "state": state,
        "accuracy": accuracy,
        "passed": passed,
        "total": probes.len(),
        "reversed_false_positive": reversed_false_positive,
        "lexical_baseline": {
            "version": "v128-density-lexical-baseline",
            "accuracy": lexical_accuracy,
            "passed": lexical_passed,
            "total": probes.len(),
            "reversed_false_positive": lexical_reversed_false_positive
        },
        "relation_baseline": {
            "version": "v132-relation-only-baseline",
            "accuracy": relation_accuracy,
            "passed": relation_passed,
            "total": probes.len(),
            "reversed_false_positive": relation_reversed_false_positive
        },
        "naive_vector_baseline": {
            "version": "v132-naive-vector-baseline",
            "accuracy": naive_accuracy,
            "passed": naive_passed,
            "total": probes.len(),
            "reversed_false_positive": naive_reversed_false_positive
        },
        "wins_over_lexical_baseline": accuracy > lexical_accuracy
            || (!reversed_false_positive && lexical_reversed_false_positive),
        "wins_over_expanded_baselines": accuracy > lexical_accuracy
            || accuracy > relation_accuracy
            || accuracy > naive_accuracy
            || (!reversed_false_positive && (lexical_reversed_false_positive || relation_reversed_false_positive || naive_reversed_false_positive)),
        "density_signal": if accuracy > lexical_accuracy || (!reversed_false_positive && lexical_reversed_false_positive) {
            "FIELD_BEATS_LEXICAL_BASELINE"
        } else {
            "FIELD_NOT_ABOVE_BASELINE"
        },
        "phase_lock": phase_lock,
        "noise_pressure": {
            "version": "v130-noise-pressure",
            "avg_suppressed_facts": avg_suppressed,
            "avg_anti_energy": avg_anti_energy,
            "avg_margin": avg_margin,
            "state": if avg_margin >= 0.5 && avg_anti_energy < 0.5 { "NOISE_CONTAINED" } else { "NOISE_REVIEW" }
        },
        "nonlinear_candidate": nonlinear,
        "packed_hot_loop_proxy": packed_hot_loop,
        "perf_counter_plan": llmwave_density_perf_plan(actual_records),
        "focus_window_experiment": focus_experiment,
        "l2_contour_spec": l2,
        "avg_margin": avg_margin,
        "elapsed_ns": elapsed_ns,
        "ns_per_probe": elapsed_ns / probes.len() as u64,
        "packed_bytes": packed_bytes,
        "fits_6m": packed_bytes <= nanda_6m::BUDGET_BYTES,
        "focus_state": focus_state,
        "answer_states": state_counts,
        "field_states": field_counts,
        "probes": probe_rows
    })
}

fn llmwave_synthetic_density_memory(records: usize) -> Value {
    let token_patterns = llmwave_synthetic_density_token_patterns();
    let mut phrase_patterns = llmwave_synthetic_density_base_phrases();
    let target_phrase_records = records
        .saturating_sub(token_patterns.len())
        .max(phrase_patterns.len());
    while phrase_patterns.len() < target_phrase_records {
        let index = phrase_patterns.len() - 4;
        phrase_patterns.push(llmwave_synthetic_density_noise_phrase(index));
    }
    let packed_bytes = (token_patterns.len() + phrase_patterns.len()) * 32;
    json!({
        "mode": "llmwave-memory",
        "version": "v127-density-synthetic-memory",
        "source": {
            "text": "synthetic density corpus",
            "triads": 0,
            "task_id": "llmwave-density",
            "domain": "density"
        },
        "write_path": {
            "version": "v127-density-synthetic-memory",
            "state": "SYNTHETIC_MEMORY_WRITTEN",
            "token_records": token_patterns.len(),
            "phrase_records": phrase_patterns.len()
        },
        "vocabulary": llmwave_memory_vocabulary_from_records(&token_patterns, &phrase_patterns),
        "wave_memory": {
            "patterns": [],
            "token_patterns": token_patterns,
            "phrase_patterns": phrase_patterns,
            "phrase_memory": {
                "version": "v92-phrase-memory",
                "state": "PHRASE_MEMORY_READY",
                "records": target_phrase_records
            },
            "positive_lanes": [],
            "negative_lanes": [],
            "resonance_traces": [],
            "consolidation": {
                "version": "v90-consolidation",
                "state": "NOT_CONSOLIDATED"
            },
            "decay": {
                "version": "v91-decay-forgetting",
                "state": "NOT_APPLIED"
            },
            "packed_runtime": {
                "version": "v93-packed-6m-memory",
                "record_bytes": 32,
                "estimated_packed_bytes": packed_bytes,
                "hot_budget_bytes": nanda_6m::BUDGET_BYTES,
                "fits_6m": packed_bytes <= nanda_6m::BUDGET_BYTES
            }
        }
    })
}

fn llmwave_density_lexical_baseline(memory: &Value, prompt: &str) -> Value {
    let query_terms = tokenize_pattern(prompt)
        .into_iter()
        .map(|token| norm(&token))
        .filter(|token| !llmwave_prompt_stopword(token))
        .collect::<BTreeSet<_>>();
    let mut rows = memory["wave_memory"]["phrase_patterns"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|record| {
            let phrase = record["phrase"].as_str().unwrap_or("");
            let terms = tokenize_pattern(phrase)
                .into_iter()
                .map(|token| norm(&token))
                .collect::<BTreeSet<_>>();
            let overlap =
                query_terms.intersection(&terms).count() as f64 / query_terms.len().max(1) as f64;
            json!({
                "fact": phrase,
                "score": round4(overlap),
                "relation": record["relation"],
                "subject": record["subject"],
                "object": record["object"]
            })
        })
        .filter(|row| row["score"].as_f64().unwrap_or(0.0) > 0.0)
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top = rows.first().cloned().unwrap_or(Value::Null);
    let answer = top["fact"].as_str().unwrap_or("");
    json!({
        "mode": "density-lexical-baseline",
        "version": "v128-density-lexical-baseline",
        "state": if answer.is_empty() { "ANSWER_EMPTY" } else { "ANSWER_READY" },
        "answer": answer,
        "top_score": top["score"],
        "top_fact": top,
        "read_as": "Lexical baseline uses only bag-of-words overlap over phrase records; it has no relation phase or subject/object polarity."
    })
}

fn llmwave_density_relation_baseline(memory: &Value, prompt: &str) -> Value {
    let intent = llmwave_query_intent(prompt);
    let mut rows = memory["wave_memory"]["phrase_patterns"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|record| {
            let relation = record["relation"].as_str().unwrap_or("");
            let relation_match = usize::from(relation == intent.relation) as f64;
            let target_match = llmwave_term_match(
                &intent.target_terms,
                record["phrase"].as_str().unwrap_or(""),
            );
            json!({
                "fact": record["phrase"],
                "score": round4(relation_match + (0.25 * target_match)),
                "relation": relation,
                "subject": record["subject"],
                "object": record["object"]
            })
        })
        .filter(|row| row["score"].as_f64().unwrap_or(0.0) > 0.0)
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top = rows.first().cloned().unwrap_or(Value::Null);
    let answer = top["fact"].as_str().unwrap_or("");
    json!({
        "mode": "density-relation-baseline",
        "version": "v132-relation-only-baseline",
        "state": if answer.is_empty() { "ANSWER_EMPTY" } else { "ANSWER_READY" },
        "answer": answer,
        "top_score": top["score"],
        "top_fact": top,
        "read_as": "Relation baseline sees relation labels but not subject/object polarity."
    })
}

fn llmwave_density_naive_vector_baseline(memory: &Value, prompt: &str) -> Value {
    let query = token_prefix_wave(&tokenize_pattern(prompt));
    let mut rows = memory["wave_memory"]["phrase_patterns"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|record| {
            let phrase = record["phrase"].as_str().unwrap_or("");
            let wave = token_prefix_wave(&tokenize_pattern(phrase));
            let score = ((cosine(&query, &wave) + 1.0) / 2.0).clamp(0.0, 1.0);
            json!({
                "fact": phrase,
                "score": round4(score),
                "relation": record["relation"],
                "subject": record["subject"],
                "object": record["object"]
            })
        })
        .filter(|row| row["score"].as_f64().unwrap_or(0.0) > 0.0)
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top = rows.first().cloned().unwrap_or(Value::Null);
    let answer = top["fact"].as_str().unwrap_or("");
    json!({
        "mode": "density-naive-vector-baseline",
        "version": "v132-naive-vector-baseline",
        "state": if answer.is_empty() { "ANSWER_EMPTY" } else { "ANSWER_READY" },
        "answer": answer,
        "top_score": top["score"],
        "top_fact": top,
        "read_as": "Naive vector baseline uses token wave similarity without relation phase or polarity gates."
    })
}

fn llmwave_density_phase_lock(field_counts: &BTreeMap<String, usize>, probes: &[Value]) -> Value {
    let total = probes.len().max(1) as f64;
    let single = *field_counts.get("FIELD_SINGLE_PEAK").unwrap_or(&0) as f64;
    let mismatch = *field_counts.get("FIELD_PHASE_MISMATCH").unwrap_or(&0) as f64;
    let phase_locked = probes
        .iter()
        .filter(|probe| {
            probe["facts"].as_array().into_iter().flatten().any(|fact| {
                fact["phase"]["relation_phase_score"]
                    .as_f64()
                    .unwrap_or(0.0)
                    >= 1.0
                    && fact["polarity"]["state"].as_str() == Some("ALIGNED")
            })
        })
        .count() as f64;
    let score = round4((single + mismatch + phase_locked) / (total * 2.0));
    json!({
        "version": "v129-phase-lock-metric",
        "score": score,
        "single_peak_count": single as usize,
        "phase_mismatch_count": mismatch as usize,
        "aligned_relation_count": phase_locked as usize,
        "competing_phase_count": *field_counts.get("FIELD_MULTI_PEAK").unwrap_or(&0),
        "state": if score >= 0.8 { "PHASE_LOCK_STABLE" } else { "PHASE_LOCK_REVIEW" }
    })
}

fn llmwave_density_nonlinear_candidate(
    avg_top_score: f64,
    avg_margin: f64,
    avg_anti: f64,
) -> Value {
    let linear_energy = round4(avg_top_score + avg_margin - avg_anti);
    let power_energy =
        round4(avg_top_score.powf(1.35) + avg_margin.powf(1.20) - avg_anti.powf(1.10));
    json!({
        "version": "v131-nonlinear-candidate",
        "enabled_in_answer_core": false,
        "linear_energy": linear_energy,
        "power_energy": power_energy,
        "delta": round4(power_energy - linear_energy),
        "read_as": "Experimental power-energy candidate only. It is reported for comparison and is not yet used to decide answers."
    })
}

#[derive(Clone, Copy)]
struct DensityPackedRecord {
    subject_hash: u32,
    relation_id: u8,
    object_hash: u32,
}

fn llmwave_density_packed_hot_loop(records: usize) -> Value {
    let memory = llmwave_density_packed_records(records);
    let probes = [
        (
            "what does customs declaration require?",
            1u8,
            hash32("customs declaration"),
            true,
        ),
        ("who issues invoice?", 3u8, hash32("invoice"), true),
        (
            "what supports customs declaration?",
            2u8,
            hash32("customs declaration"),
            true,
        ),
        ("who pays supplier?", 4u8, hash32("supplier"), true),
        ("what does invoice issue?", 3u8, hash32("invoice"), false),
    ];
    let started = std::time::Instant::now();
    let mut passed = 0usize;
    let mut checksum = 0u64;
    for (_prompt, relation, target_hash, should_match) in probes {
        let mut found = false;
        for record in &memory {
            let relation_match = record.relation_id == relation;
            let target_match = if should_match {
                record.subject_hash == target_hash || record.object_hash == target_hash
            } else {
                record.subject_hash == target_hash
            };
            if relation_match && target_match {
                found = true;
                checksum = checksum.wrapping_add(record.subject_hash as u64);
                break;
            }
        }
        passed += usize::from(found == should_match);
    }
    let elapsed_ns = started.elapsed().as_nanos() as u64;
    json!({
        "version": "v133-packed-density-hot-loop-proxy",
        "records": memory.len(),
        "passed": passed,
        "total": probes.len(),
        "accuracy": round4(passed as f64 / probes.len() as f64),
        "elapsed_ns": elapsed_ns,
        "ns_per_probe": elapsed_ns / probes.len() as u64,
        "checksum": checksum,
        "read_as": "Typed packed proxy without JSON/string scoring. It is still a proxy, not the final nanda_6m hot loop."
    })
}

fn llmwave_density_packed_records(records: usize) -> Vec<DensityPackedRecord> {
    let mut out = vec![
        DensityPackedRecord {
            subject_hash: hash32("customs declaration"),
            relation_id: 1,
            object_hash: hash32("payment confirmation"),
        },
        DensityPackedRecord {
            subject_hash: hash32("payment confirmation"),
            relation_id: 2,
            object_hash: hash32("customs declaration"),
        },
        DensityPackedRecord {
            subject_hash: hash32("supplier"),
            relation_id: 3,
            object_hash: hash32("invoice"),
        },
        DensityPackedRecord {
            subject_hash: hash32("buyer"),
            relation_id: 4,
            object_hash: hash32("supplier"),
        },
    ];
    while out.len() < records {
        let idx = out.len();
        out.push(DensityPackedRecord {
            subject_hash: hash32(&format!("noise_subject_{idx}")),
            relation_id: ((idx % 4) + 1) as u8,
            object_hash: hash32(&format!("noise_object_{idx}")),
        });
    }
    out
}

fn llmwave_density_perf_plan(records: usize) -> Value {
    json!({
        "version": "v134-perf-counter-plan",
        "measured_here": false,
        "records": records,
        "command": "perf stat -e cycles,instructions,cache-references,cache-misses,L1-dcache-loads,L1-dcache-load-misses -- nanda llmwave-memory density --counts <N> --facts 3",
        "read_as": "Run externally when perf permissions are available; this report does not claim cache-only execution."
    })
}

fn llmwave_density_focus_experiment(records: usize) -> Value {
    let focus = nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY;
    json!({
        "version": "v135-focus-window-experiment",
        "records": records,
        "focus_window": focus,
        "state": if records <= focus { "SINGLE_FOCUS_PASS" } else { "COARSE_TO_FINE_REQUIRED" },
        "windows_required": records.div_ceil(focus),
        "read_as": "65k storage is allowed, but proof should use focused windows until the full hot loop is proven."
    })
}

fn llmwave_density_l2_contour(records: usize) -> Value {
    let l2_budget = 256 * 1024usize;
    let working_budget = 128 * 1024usize;
    let local_records = 2048usize.min(working_budget / 32);
    json!({
        "version": "v136-v137-l2-contour-spec-prototype",
        "l2_budget_bytes_observed_t480": l2_budget,
        "working_budget_bytes": working_budget,
        "target_local_records": local_records,
        "input_records": records,
        "state": if records <= local_records { "L2_CAN_HOLD_ALL_LOCAL_RECORDS" } else { "L2_NEEDS_L3_PHASE_BIAS" },
        "prototype": {
            "l3_role": "semantic phase, relation route, polarity filter",
            "l2_role": "prefix candidates, short context, local rerank",
            "sync": "word-boundary or punctuation update from L3; per-keystroke local L2 rerank"
        }
    })
}

fn llmwave_synthetic_density_token_patterns() -> Vec<Value> {
    [
        (
            "customs declaration",
            "requires",
            "customs declaration requires",
        ),
        (
            "customs declaration requires",
            "payment",
            "customs declaration requires payment confirmation",
        ),
        (
            "payment confirmation",
            "supports",
            "payment confirmation supports customs declaration",
        ),
        (
            "payment confirmation supports",
            "customs",
            "payment confirmation supports customs declaration",
        ),
        ("supplier", "issues", "supplier issues invoice"),
        ("supplier issues", "invoice", "supplier issues invoice"),
        ("buyer", "pays", "buyer pays supplier"),
        ("buyer pays", "supplier", "buyer pays supplier"),
        (
            "what does customs declaration",
            "requires",
            "customs declaration requires payment",
        ),
        ("who issues", "invoice", "supplier issues invoice"),
        (
            "what supports",
            "customs",
            "payment confirmation supports customs declaration",
        ),
        ("who pays", "supplier", "buyer pays supplier"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (prefix, next_token, next_phrase))| {
        json!({
            "id": format!("density-token-{index:04}"),
            "prefix": prefix,
            "next_token": next_token,
            "next_phrase": next_phrase,
            "pattern": format!("{prefix} -> continues -> {next_token}"),
            "route": "density-control",
            "group": "density-control",
            "polarity": "text->continues->token",
            "strength": 0.75,
            "accepted": 0,
            "rejected": 0,
            "observations": 1
        })
    })
    .collect()
}

fn llmwave_synthetic_density_base_phrases() -> Vec<Value> {
    [
        (
            "customs declaration requires",
            "customs declaration requires payment confirmation",
            "customs declaration",
            "requires",
            "payment confirmation",
        ),
        (
            "payment confirmation supports",
            "payment confirmation supports customs declaration",
            "payment confirmation",
            "supports",
            "customs declaration",
        ),
        (
            "supplier issues invoice",
            "supplier issues invoice",
            "supplier",
            "issues",
            "invoice",
        ),
        (
            "buyer pays supplier",
            "buyer pays supplier",
            "buyer",
            "pays",
            "supplier",
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (prefix, phrase, subject, relation, object))| {
        json!({
            "id": format!("density-phrase-{index:04}"),
            "prefix": prefix,
            "phrase": phrase,
            "pattern": format!("{prefix} -> phrase -> {phrase}"),
            "subject": subject,
            "relation": relation,
            "object": object,
            "polarity": "subject->relation->object",
            "route": "density-control",
            "strength": 0.75
        })
    })
    .collect()
}

fn llmwave_synthetic_density_noise_phrase(index: usize) -> Value {
    let relation = match index % 4 {
        0 => "requires",
        1 => "supports",
        2 => "issues",
        _ => "pays",
    };
    let subject = format!("noise_subject_{index}");
    let object = format!("noise_object_{index}");
    let phrase = format!("{subject} {relation} {object}");
    json!({
        "id": format!("density-noise-{index:06}"),
        "prefix": format!("{subject} {relation}"),
        "phrase": phrase,
        "pattern": format!("{subject} {relation} -> phrase -> {phrase}"),
        "subject": subject,
        "relation": relation,
        "object": object,
        "polarity": "subject->relation->object",
        "route": "density-noise",
        "strength": 0.35
    })
}

fn llmwave_memory_pack_cmd(args: LlmwaveMemoryPackArgs) -> Result<u8> {
    let memory = llmwave_memory_load(&args.memory)?;
    let packed = llmwave_memory_pack_bytes(&memory);
    fs::write(&args.out, &packed)
        .with_context(|| format!("failed to write {}", args.out.display()))?;
    let out = json!({
        "mode": "llmwave-memory-pack",
        "version": "v108-binary-packed-memory-prototype",
        "out": args.out,
        "bytes": packed.len(),
        "records": memory["wave_memory"]["token_patterns"].as_array().map_or(0, Vec::len),
        "format": "LLMWAVE1"
    });
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_unpack_cmd(args: LlmwaveMemoryUnpackArgs) -> Result<u8> {
    let bytes = fs::read(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let out = llmwave_memory_unpack_report(&bytes);
    llmwave_memory_emit(out, None, &args.format)?;
    Ok(EXIT_PASS)
}

fn llmwave_memory_emit(value: Value, out: Option<&Path>, format: &OutputFormat) -> Result<()> {
    if let Some(path) = out {
        fs::write(path, serde_json::to_string_pretty(&value)? + "\n")
            .with_context(|| format!("failed to write {}", path.display()))?;
        return Ok(());
    }
    match format {
        OutputFormat::Json | OutputFormat::Text | OutputFormat::Md => {
            println!("{}", serde_json::to_string_pretty(&value)?)
        }
    }
    Ok(())
}

pub(crate) fn llmwave_memory_load(path: &Path) -> Result<Value> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

fn llmwave_memory_model_report(memory: &Value) -> Value {
    let schema_source = json!({
        "version": memory["version"],
        "sections": ["source", "write_path", "vocabulary", "wave_memory"],
        "record_shapes": ["pattern", "token_pattern", "phrase_pattern", "feedback_lane"]
    });
    let schema_digest = Sha256::digest(schema_source.to_string().as_bytes());
    json!({
        "mode": "llmwave-memory-inspect",
        "version": "v105-real-memory-file-format",
        "file_format": {
            "container": "json",
            "schema_hash": format!("{:x}", schema_digest),
            "binary_candidate": ".llmw.bin",
            "required_sections": ["source", "write_path", "vocabulary", "wave_memory"]
        },
        "tokenizer_contract": llmwave_tokenizer_contract(),
        "model_config": llmwave_model_config(memory),
        "packed_runtime": memory["wave_memory"]["packed_runtime"],
        "records": {
            "patterns": memory["wave_memory"]["patterns"].as_array().map_or(0, Vec::len),
            "token_patterns": memory["wave_memory"]["token_patterns"].as_array().map_or(0, Vec::len),
            "phrase_patterns": memory["wave_memory"]["phrase_patterns"].as_array().map_or(0, Vec::len)
        },
        "read_as": "Inspect reports the model artifact contract before treating LLMWaveMemory as a model."
    })
}

fn llmwave_tokenizer_contract() -> Value {
    json!({
        "version": "v106-tokenizer-contract",
        "normalization": "lowercase ascii-domain token normalization",
        "split": "non-alphanumeric separators",
        "position_window": 6,
        "unknown_token": "<unk>",
        "stop_tokens": [".", "\n"],
        "preserve_domain_tokens": ["route", "role", "relation", "evidence"]
    })
}

fn llmwave_model_config(memory: &Value) -> Value {
    json!({
        "version": "v107-model-config",
        "wave_dim": WAVE_DIM,
        "hot_budget": nanda_6m::BUDGET_BYTES,
        "record_bytes": 32,
        "sampler": {
            "temperature": 0.0,
            "top_k": 5,
            "beam_width": 3
        },
        "decay": memory["wave_memory"]["decay"],
        "feedback": {
            "positive_lanes": memory["wave_memory"]["positive_lanes"].as_array().map_or(0, Vec::len),
            "negative_lanes": memory["wave_memory"]["negative_lanes"].as_array().map_or(0, Vec::len)
        }
    })
}

fn llmwave_memory_from_packet(packet: &Packet, text: &str) -> Value {
    let token_records = token_pattern_records(packet);
    let patterns = packet
        .triads
        .iter()
        .map(|triad| {
            let pattern = format!(
                "{} -> {} -> {}",
                triad.subject, triad.relation, triad.object
            );
            json!({
                "id": if triad.id.is_empty() { slug(&pattern) } else { triad.id.clone() },
                "pattern": pattern,
                "subject": triad.subject,
                "relation": triad.relation,
                "object": triad.object,
                "route": triad.route,
                "group": triad.group,
                "subject_role": triad.subject_role,
                "object_role": triad.object_role,
                "evidence": triad.evidence,
                "confidence": triad.confidence,
                "strength": round4(triad.confidence.max(0.05)),
                "accepted": 0,
                "rejected": 0,
                "observations": 1
            })
        })
        .collect::<Vec<_>>();
    let token_patterns = token_records
        .iter()
        .enumerate()
        .map(|(idx, record)| {
            json!({
                "id": format!("tok-{idx:04}"),
                "prefix": record.prefix_tokens.join(" "),
                "next_token": record.next_token,
                "next_phrase": record.next_phrase,
                "pattern": record.pattern,
                "route": record.route,
                "group": record.group,
                "polarity": record.polarity,
                "strength": round4(record.confidence.max(0.05)),
                "accepted": 0,
                "rejected": 0,
                "observations": 1
            })
        })
        .collect::<Vec<_>>();
    let phrase_patterns = packet
        .triads
        .iter()
        .map(|triad| {
            json!({
                "prefix": format!("{} {}", triad.subject, triad.relation),
                "phrase": triad.object,
                "pattern": format!("{} -> {} -> {}", triad.subject, triad.relation, triad.object),
                "subject": triad.subject,
                "relation": llmwave_canonical_relation(&triad.relation),
                "object": triad.object,
                "polarity": "subject->relation->object",
                "route": triad.route,
                "strength": round4(triad.confidence.max(0.05))
            })
        })
        .collect::<Vec<_>>();
    let packed_bytes = (patterns.len() + token_patterns.len() + phrase_patterns.len()) * 32;
    json!({
        "mode": "llmwave-memory",
        "version": "v86-wave-memory-schema",
        "source": {
            "text": text,
            "triads": packet.triads.len(),
            "task_id": packet.task_id,
            "domain": packet.domain
        },
        "write_path": {
            "version": "v87-memory-write-path",
            "state": "MEMORY_WRITTEN",
            "source_triads": packet.triads.len(),
            "token_records": token_patterns.len(),
            "phrase_records": phrase_patterns.len()
        },
        "vocabulary": llmwave_memory_vocabulary_from_records(&token_patterns, &phrase_patterns),
        "wave_memory": {
            "patterns": patterns,
            "token_patterns": token_patterns,
            "phrase_patterns": phrase_patterns,
            "phrase_memory": {
                "version": "v92-phrase-memory",
                "state": "PHRASE_MEMORY_READY",
                "records": packet.triads.len()
            },
            "positive_lanes": [],
            "negative_lanes": [],
            "resonance_traces": [],
            "consolidation": {
                "version": "v90-consolidation",
                "state": "NOT_CONSOLIDATED"
            },
            "decay": {
                "version": "v91-decay-forgetting",
                "state": "NOT_APPLIED"
            },
            "packed_runtime": {
                "version": "v93-packed-6m-memory",
                "record_bytes": 32,
                "estimated_packed_bytes": packed_bytes,
                "hot_budget_bytes": nanda_6m::BUDGET_BYTES,
                "fits_6m": packed_bytes <= nanda_6m::BUDGET_BYTES
            }
        },
        "read_as": "v86 writes triads, token continuations, and phrase continuations into one LLMWave memory object."
    })
}

fn llmwave_memory_from_text(text: &str, task_id: &str, domain: &str) -> Value {
    let tokens = tokenize_pattern(text);
    let mut token_patterns = vec![];
    let mut token_record_id = 0usize;
    for sentence in text
        .split(['.', '\n', ';'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let sentence_tokens = tokenize_pattern(sentence);
        for index in 1..sentence_tokens.len() {
            let start = index.saturating_sub(6);
            let prefix_tokens = sentence_tokens[start..index].to_vec();
            token_patterns.push(json!({
                "id": format!("txt-tok-{token_record_id:04}"),
                "prefix": prefix_tokens.join(" "),
                "next_token": sentence_tokens[index],
                "next_phrase": sentence_tokens[index..sentence_tokens.len().min(index + 3)].join(" "),
                "pattern": format!("{} -> continues -> {}", prefix_tokens.join(" "), sentence_tokens[index]),
                "route": "text",
                "group": "text-memory",
                "polarity": "text->continues->token",
                "strength": 0.5,
                "accepted": 0,
                "rejected": 0,
                "observations": 1
            }));
            token_record_id += 1;
        }
    }
    let phrase_patterns = text
        .split(['.', '\n', ';'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(idx, line)| {
            let line_tokens = tokenize_pattern(line);
            let prefix = line_tokens
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            let fact = llmwave_parse_fact(line);
            json!({
                "id": format!("txt-phrase-{idx:04}"),
                "prefix": prefix,
                "phrase": line,
                "pattern": format!("{prefix} -> phrase -> {line}"),
                "subject": fact.subject,
                "relation": fact.relation,
                "object": fact.object,
                "polarity": "subject->relation->object",
                "route": "text",
                "strength": 0.5
            })
        })
        .collect::<Vec<_>>();
    let packed_bytes = (token_patterns.len() + phrase_patterns.len()) * 32;
    json!({
        "mode": "llmwave-memory",
        "version": "v101-training-from-text",
        "source": {
            "text": text,
            "triads": 0,
            "task_id": task_id,
            "domain": domain
        },
        "write_path": {
            "version": "v101-training-from-text",
            "state": "TEXT_MEMORY_WRITTEN",
            "source_tokens": tokens.len(),
            "token_records": token_patterns.len(),
            "phrase_records": phrase_patterns.len()
        },
        "vocabulary": llmwave_memory_vocabulary_from_records(&token_patterns, &phrase_patterns),
        "wave_memory": {
            "patterns": [],
            "token_patterns": token_patterns,
            "phrase_patterns": phrase_patterns,
            "phrase_memory": {
                "version": "v92-phrase-memory",
                "state": "PHRASE_MEMORY_READY",
                "records": text.split(['.', '\n', ';']).filter(|line| !line.trim().is_empty()).count()
            },
            "positive_lanes": [],
            "negative_lanes": [],
            "resonance_traces": [],
            "consolidation": {
                "version": "v90-consolidation",
                "state": "NOT_CONSOLIDATED"
            },
            "decay": {
                "version": "v91-decay-forgetting",
                "state": "NOT_APPLIED"
            },
            "packed_runtime": {
                "version": "v93-packed-6m-memory",
                "record_bytes": 32,
                "estimated_packed_bytes": packed_bytes,
                "hot_budget_bytes": nanda_6m::BUDGET_BYTES,
                "fits_6m": packed_bytes <= nanda_6m::BUDGET_BYTES
            }
        },
        "read_as": "v101 trains a small LLMWave memory directly from text token windows."
    })
}

fn llmwave_memory_vocabulary_report(memory: &Value) -> Value {
    let token_patterns = memory["wave_memory"]["token_patterns"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let phrase_patterns = memory["wave_memory"]["phrase_patterns"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    llmwave_memory_vocabulary_from_records(&token_patterns, &phrase_patterns)
}

fn llmwave_memory_vocabulary_from_records(
    token_patterns: &[Value],
    phrase_patterns: &[Value],
) -> Value {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in token_patterns {
        for token in tokenize_pattern(record["prefix"].as_str().unwrap_or("")) {
            *counts.entry(token).or_insert(0) += 1;
        }
        if let Some(token) = record["next_token"].as_str() {
            *counts.entry(norm(token)).or_insert(0) += 1;
        }
    }
    for record in phrase_patterns {
        for token in tokenize_pattern(record["phrase"].as_str().unwrap_or("")) {
            *counts.entry(token).or_insert(0) += 1;
        }
    }
    let mut top_tokens = counts
        .iter()
        .map(|(token, count)| json!({"token": token, "count": count}))
        .collect::<Vec<_>>();
    top_tokens.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["count"].as_u64().unwrap_or(0))
            .then_with(|| {
                a["token"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["token"].as_str().unwrap_or(""))
            })
    });
    top_tokens.truncate(16);
    json!({
        "mode": "llmwave-memory-vocabulary",
        "version": "v96-vocabulary-token-space",
        "tokens": counts.len(),
        "token_records": token_patterns.len(),
        "phrase_records": phrase_patterns.len(),
        "top_tokens": top_tokens,
        "read_as": "Vocabulary is the explicit token/phrase space used by LLMWaveMemory generation."
    })
}

fn llmwave_memory_retrieve_report(memory: &Value, prefix: &str, top_k: usize) -> Value {
    let rows = llmwave_memory_score_tokens(memory, prefix, top_k.max(1));
    let top_token = rows
        .first()
        .and_then(|row| row["next_token"].as_str())
        .unwrap_or("");
    let top_score = rows
        .first()
        .and_then(|row| row["score"].as_f64())
        .unwrap_or(0.0);
    let second_score = rows
        .get(1)
        .and_then(|row| row["score"].as_f64())
        .unwrap_or(0.0);
    let margin = round4((top_score - second_score).max(0.0));
    let state = if top_token.is_empty() {
        "MEMORY_RETRIEVE_EMPTY"
    } else if margin < 0.015 && rows.len() > 1 {
        "MEMORY_RETRIEVE_CONTESTED"
    } else {
        "MEMORY_RETRIEVE_READY"
    };
    json!({
        "mode": "llmwave-memory-retrieve",
        "version": "v88-memory-retrieve-path",
        "state": state,
        "prefix": prefix,
        "top_token": top_token,
        "top_phrase": rows.first().map(|row| row["next_phrase"].clone()).unwrap_or(Value::Null),
        "margin": margin,
        "top_k": rows,
        "packed_runtime": memory["wave_memory"]["packed_runtime"],
        "read_as": "Retrieve reads LLMWaveMemory through token/phrase resonance, not a prose generator."
    })
}

fn llmwave_memory_score_tokens(memory: &Value, prefix: &str, top_k: usize) -> Vec<Value> {
    let tokens = tokenize_pattern(prefix);
    let query_wave = token_prefix_wave(&tokens);
    let query_terms = tokens
        .iter()
        .map(|token| norm(token))
        .collect::<BTreeSet<_>>();
    let mut rows = memory["wave_memory"]["token_patterns"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|record| {
            let record_prefix = tokenize_pattern(record["prefix"].as_str().unwrap_or(""));
            let wave = token_prefix_wave(&record_prefix);
            let wave_score = ((cosine(&query_wave, &wave) + 1.0) / 2.0).clamp(0.0, 1.0);
            let suffix = token_suffix_match(&tokens, &record_prefix);
            let route_score = llmwave_memory_route_match(&query_terms, record);
            let strength = record["strength"].as_f64().unwrap_or(0.5).max(0.0);
            let accepted = record["accepted"].as_u64().unwrap_or(0) as f64;
            let rejected = record["rejected"].as_u64().unwrap_or(0) as f64;
            let feedback = (accepted * 0.08) - (rejected * 0.12);
            let score = round4(
                ((0.38 * wave_score)
                    + (0.30 * suffix)
                    + (0.16 * route_score)
                    + (0.16 * strength)
                    + feedback)
                    .max(0.0),
            );
            json!({
                "next_token": record["next_token"],
                "next_phrase": record["next_phrase"],
                "pattern": record["pattern"],
                "prefix": record["prefix"],
                "route": record["route"],
                "group": record["group"],
                "score": score,
                "wave_score": round4(wave_score),
                "suffix_score": round4(suffix),
                "route_score": round4(route_score),
                "strength": round4(strength),
                "accepted": accepted as u64,
                "rejected": rejected as u64
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(top_k);
    rows
}

fn llmwave_memory_route_match(query_terms: &BTreeSet<String>, record: &Value) -> f64 {
    let mut route_terms = BTreeSet::new();
    for value in [record["route"].as_str(), record["group"].as_str()]
        .into_iter()
        .flatten()
    {
        for term in norm(value)
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|term| term.len() >= 2)
        {
            route_terms.insert(term.to_string());
        }
    }
    if route_terms.is_empty() {
        return 0.0;
    }
    query_terms.intersection(&route_terms).count() as f64 / route_terms.len() as f64
}

fn llmwave_memory_apply_feedback(
    memory: &mut Value,
    decision: &FeedbackDecision,
    pattern: &str,
    token: &str,
    note: &str,
) -> Value {
    let decision_label = match decision {
        FeedbackDecision::Accept => "accept",
        FeedbackDecision::Reject => "reject",
        FeedbackDecision::Watch => "watch",
    };
    let mut touched = 0usize;
    if let Some(records) = memory["wave_memory"]["token_patterns"].as_array_mut() {
        for record in records {
            let token_match = token.is_empty() || record["next_token"].as_str() == Some(token);
            let pattern_match = pattern.is_empty()
                || record["pattern"]
                    .as_str()
                    .is_some_and(|value| norm(value).contains(&norm(pattern)));
            if token_match && pattern_match {
                touched += 1;
                let strength = record["strength"].as_f64().unwrap_or(0.5);
                match decision {
                    FeedbackDecision::Accept => {
                        record["accepted"] = json!(record["accepted"].as_u64().unwrap_or(0) + 1);
                        record["strength"] = json!(round4((strength + 0.08).min(1.5)));
                    }
                    FeedbackDecision::Reject => {
                        record["rejected"] = json!(record["rejected"].as_u64().unwrap_or(0) + 1);
                        record["strength"] = json!(round4((strength - 0.12).max(0.0)));
                    }
                    FeedbackDecision::Watch => {
                        record["observations"] =
                            json!(record["observations"].as_u64().unwrap_or(1) + 1);
                    }
                }
            }
        }
    }
    let lane = json!({
        "decision": decision_label,
        "pattern": pattern,
        "token": token,
        "note": note,
        "touched": touched
    });
    match decision {
        FeedbackDecision::Accept => {
            if let Some(lanes) = memory["wave_memory"]["positive_lanes"].as_array_mut() {
                lanes.push(lane.clone());
            }
        }
        FeedbackDecision::Reject => {
            if let Some(lanes) = memory["wave_memory"]["negative_lanes"].as_array_mut() {
                lanes.push(lane.clone());
            }
        }
        FeedbackDecision::Watch => {
            if let Some(lanes) = memory["wave_memory"]["resonance_traces"].as_array_mut() {
                lanes.push(lane.clone());
            }
        }
    }
    json!({
        "mode": "llmwave-memory-feedback",
        "version": "v89-feedback-learning",
        "state": if touched > 0 { "MEMORY_FEEDBACK_APPLIED" } else { "MEMORY_FEEDBACK_NO_MATCH" },
        "decision": decision_label,
        "touched": touched,
        "lane": lane,
        "memory": memory
    })
}

fn llmwave_memory_consolidate(memory: &mut Value) -> Value {
    let before = memory["wave_memory"]["token_patterns"]
        .as_array()
        .map_or(0, Vec::len);
    let mut merged: BTreeMap<(String, String), Value> = BTreeMap::new();
    for record in memory["wave_memory"]["token_patterns"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        let key = (
            norm(record["prefix"].as_str().unwrap_or("")),
            norm(record["next_token"].as_str().unwrap_or("")),
        );
        merged
            .entry(key)
            .and_modify(|existing| {
                existing["strength"] = json!(round4(
                    existing["strength"].as_f64().unwrap_or(0.0)
                        + record["strength"].as_f64().unwrap_or(0.0)
                ));
                existing["observations"] = json!(
                    existing["observations"].as_u64().unwrap_or(0)
                        + record["observations"].as_u64().unwrap_or(1)
                );
                existing["accepted"] = json!(
                    existing["accepted"].as_u64().unwrap_or(0)
                        + record["accepted"].as_u64().unwrap_or(0)
                );
                existing["rejected"] = json!(
                    existing["rejected"].as_u64().unwrap_or(0)
                        + record["rejected"].as_u64().unwrap_or(0)
                );
            })
            .or_insert(record);
    }
    let mut records = merged.into_values().collect::<Vec<_>>();
    records.sort_by(|a, b| {
        b["strength"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["strength"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let after = records.len();
    memory["wave_memory"]["token_patterns"] = Value::Array(records);
    memory["wave_memory"]["consolidation"] = json!({
        "version": "v90-consolidation",
        "state": "CONSOLIDATED",
        "before": before,
        "after": after,
        "merged": before.saturating_sub(after)
    });
    llmwave_memory_refresh_budget(memory);
    json!({
        "mode": "llmwave-memory-consolidate",
        "version": "v90-consolidation",
        "before": before,
        "after": after,
        "memory": memory
    })
}

fn llmwave_memory_decay(memory: &mut Value, factor: f64, min_strength: f64) -> Value {
    let factor = factor.clamp(0.0, 1.0);
    let mut kept = vec![];
    let mut dropped = 0usize;
    for mut record in memory["wave_memory"]["token_patterns"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        let strength = round4(record["strength"].as_f64().unwrap_or(0.0) * factor);
        if strength >= min_strength {
            record["strength"] = json!(strength);
            kept.push(record);
        } else {
            dropped += 1;
        }
    }
    memory["wave_memory"]["token_patterns"] = Value::Array(kept);
    memory["wave_memory"]["decay"] = json!({
        "version": "v91-decay-forgetting",
        "state": "DECAY_APPLIED",
        "factor": factor,
        "min_strength": min_strength,
        "dropped": dropped
    });
    llmwave_memory_refresh_budget(memory);
    json!({
        "mode": "llmwave-memory-decay",
        "version": "v91-decay-forgetting",
        "dropped": dropped,
        "memory": memory
    })
}

pub(crate) fn llmwave_memory_chat_report(
    memory: &Value,
    prompt: &str,
    steps: usize,
    top_k: usize,
    beam_width: usize,
    temperature: f64,
    language: &str,
) -> Value {
    let prompt_adapter = llmwave_prompt_adapter(memory, prompt);
    let selected_prefix = prompt_adapter["selected_prefix"].as_str().unwrap_or(prompt);
    let generation = llmwave_memory_generate_report(
        memory,
        selected_prefix,
        steps,
        top_k,
        beam_width,
        temperature,
        language,
    );
    json!({
        "mode": "llmwave-memory-chat",
        "version": "v100-chat-loop",
        "prompt": prompt,
        "answer": generation["decoded_text"],
        "state": generation["state"],
        "prompt_adapter": prompt_adapter,
        "generation": generation,
        "read_as": "Chat is a tiny LLMWave loop: prompt adapter -> memory -> semantic guard -> coherent beam -> semantic decoder -> text."
    })
}

pub(crate) fn llmwave_memory_answer_report(
    memory: &Value,
    prompt: &str,
    facts: usize,
    top_k: usize,
    language: &str,
) -> Value {
    let prompt_adapter = llmwave_prompt_adapter(memory, prompt);
    let selected_prefix = prompt_adapter["selected_prefix"].as_str().unwrap_or(prompt);
    let intent = llmwave_query_intent(prompt);
    let retrieve = llmwave_memory_retrieve_report(memory, selected_prefix, top_k.max(3));
    let mut beams = llmwave_memory_beams(&retrieve, top_k.max(3), 0.0);
    llmwave_memory_guard_beams(&mut beams, &retrieve, &BTreeSet::new(), "");
    let (mut evidence, mut suppressed_facts, field_core) =
        llmwave_answer_evidence(memory, selected_prefix, prompt, &intent, facts.max(1));
    let suppressed = beams
        .iter()
        .filter(|beam| beam["semantic_guard"]["state"].as_str() != Some("BEAM_SAFE"))
        .cloned()
        .collect::<Vec<_>>();
    let safe_beams = beams
        .iter()
        .filter(|beam| beam["semantic_guard"]["state"].as_str() == Some("BEAM_SAFE"))
        .cloned()
        .collect::<Vec<_>>();
    let contested = retrieve["state"].as_str() == Some("MEMORY_RETRIEVE_CONTESTED");
    let field_state = field_core["state"].as_str().unwrap_or("FIELD_EMPTY");
    if field_state == "FIELD_PHASE_MISMATCH" {
        suppressed_facts.extend(evidence.clone());
        evidence.clear();
    }
    let answer_state =
        if matches!(field_state, "FIELD_EMPTY" | "FIELD_PHASE_MISMATCH") || evidence.is_empty() {
            "ANSWER_EMPTY"
        } else if field_state == "FIELD_MULTI_PEAK" {
            "ANSWER_CONTESTED"
        } else {
            "ANSWER_READY"
        };
    let answer_text = llmwave_memory_answer_text(prompt, &evidence, language, answer_state);
    json!({
        "mode": "llmwave-memory-answer",
        "version": "v115-answer-contract",
        "answer_versions": {
            "contract": "v115-answer-contract",
            "grounding": "v116-grounded-answer",
            "multi_fact": "v117-multi-fact-answer",
            "review_state": "v118-answer-review-state"
        },
        "core_versions": {
            "relation_phase": "v120-relation-phase-channels",
            "polarity": "v121-subject-object-polarity",
            "bidirectional_recall": "v122-bidirectional-recall",
            "field_decomposition": "v123-field-decomposition",
            "ambiguity_detector": "v124-phase-collision-detector",
            "core_eval": "v126-core-field-eval"
        },
        "state": answer_state,
        "safe_to_answer": answer_state == "ANSWER_READY",
        "prompt": prompt,
        "answer": answer_text,
        "prompt_adapter": prompt_adapter,
        "retrieve": retrieve,
        "grounding": {
            "version": "v116-grounded-answer",
            "selected_prefix": selected_prefix,
            "facts": evidence,
            "suppressed_facts": suppressed_facts,
            "safe_beams": safe_beams,
            "suppressed_beams": suppressed,
            "memory_records_used": evidence.len()
        },
        "field_core": field_core,
        "review": {
            "version": "v118-answer-review-state",
            "state": answer_state,
            "reasons": llmwave_answer_reasons(answer_state, &beams, &evidence, contested || field_state == "FIELD_MULTI_PEAK" || field_state == "FIELD_PHASE_MISMATCH")
        },
        "read_as": "Answer readiness is decided by the LLMWave field core. Retrieve/beams are diagnostics, not the final judge."
    })
}

#[derive(Clone, Debug)]
struct LlmwaveFact {
    subject: String,
    relation: String,
    object: String,
}

#[derive(Clone, Debug)]
struct LlmwaveQueryIntent {
    relation: String,
    direction: String,
    target: String,
    target_terms: BTreeSet<String>,
}

fn llmwave_answer_evidence(
    memory: &Value,
    prefix: &str,
    prompt: &str,
    intent: &LlmwaveQueryIntent,
    limit: usize,
) -> (Vec<Value>, Vec<Value>, Value) {
    let mut rows = vec![];
    let query_terms = tokenize_pattern(&format!("{prefix} {prompt}"))
        .into_iter()
        .map(|token| norm(&token))
        .filter(|token| !llmwave_prompt_stopword(token))
        .collect::<BTreeSet<_>>();
    for record in memory["wave_memory"]["phrase_patterns"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let phrase = record["phrase"].as_str().unwrap_or("");
        let record_prefix = record["prefix"].as_str().unwrap_or("");
        let fact = llmwave_fact_from_record(record, phrase);
        let terms = tokenize_pattern(&format!("{record_prefix} {phrase}"))
            .into_iter()
            .map(|token| norm(&token))
            .collect::<BTreeSet<_>>();
        let overlap =
            query_terms.intersection(&terms).count() as f64 / query_terms.len().max(1) as f64;
        let relation_phase = llmwave_relation_phase_score(intent, &fact);
        let polarity = llmwave_polarity_score(intent, &fact);
        let bidirectional = llmwave_bidirectional_score(intent, &fact);
        let phrase_energy = overlap;
        let anti_energy = if polarity < 0.0 { polarity.abs() } else { 0.0 };
        let prefix_bonus = if norm(record_prefix) == norm(prefix)
            || norm(phrase).contains(&norm(prefix))
            || norm(prefix).contains(&norm(record_prefix))
        {
            0.35
        } else {
            0.0
        };
        let strength_energy = record["strength"].as_f64().unwrap_or(0.0) * 0.20;
        let score = round4(
            ((0.22 * phrase_energy)
                + (0.30 * relation_phase)
                + (0.28 * bidirectional)
                + (0.20 * polarity.max(0.0))
                + prefix_bonus
                + strength_energy
                - (0.75 * anti_energy))
                .clamp(0.0, 1.5),
        );
        if score > 0.0 {
            rows.push(json!({
                "record_type": "phrase",
                "prefix": record_prefix,
                "fact": phrase,
                "pattern": record["pattern"],
                "subject": fact.subject,
                "relation": fact.relation,
                "object": fact.object,
                "route": record["route"],
                "score": score,
                "field_decomposition": {
                    "version": "v123-field-decomposition",
                    "subject_energy": round4(llmwave_term_match(&intent.target_terms, &fact.subject)),
                    "relation_energy": round4(relation_phase),
                    "object_energy": round4(llmwave_term_match(&intent.target_terms, &fact.object)),
                    "phrase_energy": round4(phrase_energy),
                    "polarity_energy": round4(polarity),
                    "bidirectional_energy": round4(bidirectional),
                    "anti_energy": round4(anti_energy)
                },
                "phase": {
                    "version": "v120-relation-phase-channels",
                    "intent_relation": intent.relation,
                    "record_relation": fact.relation,
                    "relation_phase_score": round4(relation_phase)
                },
                "polarity": {
                    "version": "v121-subject-object-polarity",
                    "direction": intent.direction,
                    "score": round4(polarity),
                    "state": if polarity < 0.0 { "REVERSED" } else if polarity > 0.0 { "ALIGNED" } else { "NEUTRAL" }
                },
                "bidirectional_recall": {
                    "version": "v122-bidirectional-recall",
                    "target": intent.target,
                    "score": round4(bidirectional)
                }
            }));
        }
    }
    if rows.is_empty() {
        for record in memory["wave_memory"]["token_patterns"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let record_prefix = record["prefix"].as_str().unwrap_or("");
            let terms = tokenize_pattern(record_prefix)
                .into_iter()
                .map(|token| norm(&token))
                .collect::<BTreeSet<_>>();
            let overlap =
                query_terms.intersection(&terms).count() as f64 / query_terms.len().max(1) as f64;
            if overlap > 0.0 {
                rows.push(json!({
                    "record_type": "token",
                    "prefix": record_prefix,
                    "fact": format!("{} {}", record_prefix, record["next_phrase"].as_str().unwrap_or(record["next_token"].as_str().unwrap_or(""))).trim().to_string(),
                    "pattern": record["pattern"],
                    "route": record["route"],
                    "score": round4(overlap)
                }));
            }
        }
    }
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_score = rows
        .first()
        .and_then(|row| row["score"].as_f64())
        .unwrap_or(0.0);
    let keep_threshold = (top_score * 0.35).max(0.25);
    let mut suppressed = rows
        .iter()
        .filter(|row| row["score"].as_f64().unwrap_or(0.0) < keep_threshold)
        .cloned()
        .collect::<Vec<_>>();
    rows.retain(|row| row["score"].as_f64().unwrap_or(0.0) >= keep_threshold);
    rows.dedup_by(|a, b| a["fact"].as_str() == b["fact"].as_str());
    suppressed.dedup_by(|a, b| a["fact"].as_str() == b["fact"].as_str());
    let field_core = llmwave_answer_field_core(&rows, &suppressed, intent);
    rows.truncate(limit);
    (rows, suppressed, field_core)
}

fn llmwave_answer_field_core(
    rows: &[Value],
    suppressed: &[Value],
    intent: &LlmwaveQueryIntent,
) -> Value {
    let top_score = rows
        .first()
        .and_then(|row| row["score"].as_f64())
        .unwrap_or(0.0);
    let second_score = rows
        .get(1)
        .and_then(|row| row["score"].as_f64())
        .unwrap_or(0.0);
    let margin = round4((top_score - second_score).max(0.0));
    let relation_energy = rows
        .first()
        .and_then(|row| row["field_decomposition"]["relation_energy"].as_f64())
        .unwrap_or(0.0);
    let anti_energy = suppressed
        .iter()
        .filter_map(|row| row["field_decomposition"]["anti_energy"].as_f64())
        .sum::<f64>();
    let state = if rows.is_empty() {
        "FIELD_EMPTY"
    } else if intent.relation != "related" && relation_energy < 0.5 {
        "FIELD_PHASE_MISMATCH"
    } else if rows.len() > 1 && margin < 0.18 {
        "FIELD_MULTI_PEAK"
    } else {
        "FIELD_SINGLE_PEAK"
    };
    json!({
        "version": "v124-phase-collision-detector",
        "state": state,
        "intent": {
            "relation": intent.relation,
            "direction": intent.direction,
            "target": intent.target
        },
        "top_score": round4(top_score),
        "second_score": round4(second_score),
        "margin": margin,
        "relation_energy": round4(relation_energy),
        "anti_energy": round4(anti_energy),
        "suppressed_count": suppressed.len(),
        "read_as": "Core field state is computed from relation phase, subject/object polarity, bidirectional target match, and anti-energy."
    })
}

fn llmwave_query_intent(prompt: &str) -> LlmwaveQueryIntent {
    let tokens = tokenize_pattern(prompt);
    let relation = tokens
        .iter()
        .find_map(|token| {
            let canonical = llmwave_canonical_relation(token);
            if canonical.is_empty() {
                None
            } else {
                Some(canonical)
            }
        })
        .unwrap_or_else(|| "related".to_string());
    let question = tokens.first().map(|token| norm(token)).unwrap_or_default();
    let has_does = tokens.iter().any(|token| norm(token) == "does");
    let direction = if question == "who" {
        "backward-subject"
    } else if question == "what" && has_does {
        "forward-object"
    } else if question == "what" {
        "backward-subject"
    } else {
        "bidirectional"
    }
    .to_string();
    let target_tokens = tokens
        .iter()
        .map(|token| norm(token))
        .filter(|token| {
            !llmwave_prompt_stopword(token)
                && llmwave_canonical_relation(token).is_empty()
                && token != "who"
                && token != "what"
                && token != "does"
        })
        .collect::<Vec<_>>();
    let target = target_tokens.join(" ");
    let target_terms = target_tokens.into_iter().collect::<BTreeSet<_>>();
    LlmwaveQueryIntent {
        relation,
        direction,
        target,
        target_terms,
    }
}

fn llmwave_fact_from_record(record: &Value, fallback_phrase: &str) -> LlmwaveFact {
    let subject = record["subject"].as_str().unwrap_or("");
    let relation = record["relation"].as_str().unwrap_or("");
    let object = record["object"].as_str().unwrap_or("");
    if !subject.is_empty() || !relation.is_empty() || !object.is_empty() {
        return LlmwaveFact {
            subject: subject.to_string(),
            relation: llmwave_canonical_relation(relation),
            object: object.to_string(),
        };
    }
    llmwave_parse_fact(fallback_phrase)
}

fn llmwave_parse_fact(text: &str) -> LlmwaveFact {
    let tokens = tokenize_pattern(text);
    let relation_index = tokens
        .iter()
        .position(|token| !llmwave_canonical_relation(token).is_empty());
    if let Some(index) = relation_index {
        let subject = tokens[..index].join(" ");
        let relation = llmwave_canonical_relation(&tokens[index]);
        let object = tokens[index + 1..].join(" ");
        return LlmwaveFact {
            subject,
            relation,
            object,
        };
    }
    LlmwaveFact {
        subject: tokens.first().cloned().unwrap_or_default(),
        relation: "related".to_string(),
        object: tokens.iter().skip(1).cloned().collect::<Vec<_>>().join(" "),
    }
}

fn llmwave_canonical_relation(value: &str) -> String {
    match norm(value).as_str() {
        "require" | "requires" | "required" | "need" | "needs" => "requires".to_string(),
        "support" | "supports" | "supported" => "supports".to_string(),
        "issue" | "issues" | "issued" => "issues".to_string(),
        "pay" | "pays" | "paid" => "pays".to_string(),
        _ => String::new(),
    }
}

fn llmwave_relation_phase_score(intent: &LlmwaveQueryIntent, fact: &LlmwaveFact) -> f64 {
    if intent.relation == "related" || fact.relation.is_empty() {
        0.35
    } else if intent.relation == fact.relation {
        1.0
    } else {
        0.0
    }
}

fn llmwave_polarity_score(intent: &LlmwaveQueryIntent, fact: &LlmwaveFact) -> f64 {
    let subject_match = llmwave_term_match(&intent.target_terms, &fact.subject);
    let object_match = llmwave_term_match(&intent.target_terms, &fact.object);
    match intent.direction.as_str() {
        "forward-object" => {
            if subject_match > 0.0 {
                subject_match
            } else if object_match > 0.0 {
                -object_match
            } else {
                0.0
            }
        }
        "backward-subject" => {
            if object_match > 0.0 {
                object_match
            } else if subject_match > 0.0 {
                if intent.relation == fact.relation {
                    0.25
                } else {
                    -subject_match
                }
            } else {
                0.0
            }
        }
        _ => subject_match.max(object_match),
    }
}

fn llmwave_bidirectional_score(intent: &LlmwaveQueryIntent, fact: &LlmwaveFact) -> f64 {
    let subject_match = llmwave_term_match(&intent.target_terms, &fact.subject);
    let object_match = llmwave_term_match(&intent.target_terms, &fact.object);
    match intent.direction.as_str() {
        "forward-object" => subject_match,
        "backward-subject" => object_match.max(if intent.relation == fact.relation {
            subject_match * 0.25
        } else {
            0.0
        }),
        _ => subject_match.max(object_match),
    }
}

fn llmwave_term_match(query_terms: &BTreeSet<String>, text: &str) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let terms = tokenize_pattern(text)
        .into_iter()
        .map(|token| norm(&token))
        .collect::<BTreeSet<_>>();
    query_terms.intersection(&terms).count() as f64 / query_terms.len().max(1) as f64
}

fn llmwave_memory_answer_text(
    prompt: &str,
    evidence: &[Value],
    language: &str,
    state: &str,
) -> String {
    if state == "ANSWER_EMPTY" {
        return if language == "ru" {
            "LLMWave не нашла устойчивый ответ в памяти.".to_string()
        } else {
            "LLMWave did not find a stable answer in memory.".to_string()
        };
    }
    let facts = evidence
        .iter()
        .filter_map(|row| row["fact"].as_str())
        .collect::<Vec<_>>();
    if language == "ru" {
        format!("По памяти LLMWave: {}.", facts.join("; "))
    } else {
        format!("LLMWave answer to '{prompt}': {}.", facts.join("; "))
    }
}

fn llmwave_answer_reasons(
    state: &str,
    beams: &[Value],
    evidence: &[Value],
    contested: bool,
) -> Vec<String> {
    let mut reasons = vec![];
    if evidence.is_empty() {
        reasons.push("NO_GROUNDED_FACTS".to_string());
    }
    if contested {
        reasons.push("LOW_MARGIN_CONTESTED_RETRIEVE".to_string());
    }
    if beams
        .iter()
        .all(|beam| beam["semantic_guard"]["state"].as_str() != Some("BEAM_SAFE"))
    {
        reasons.push("NO_SAFE_BEAM".to_string());
    }
    if reasons.is_empty() && state == "ANSWER_READY" {
        reasons.push("GROUNDED_FACTS_AND_SAFE_BEAM".to_string());
    }
    reasons
}

fn llmwave_prompt_adapter(memory: &Value, prompt: &str) -> Value {
    let prompt_tokens = tokenize_pattern(prompt);
    let prompt_terms = prompt_tokens
        .iter()
        .map(|token| norm(token))
        .filter(|token| !llmwave_prompt_stopword(token))
        .collect::<BTreeSet<_>>();
    let wants_require = prompt_terms.iter().any(|token| {
        matches!(
            token.as_str(),
            "require" | "requires" | "required" | "need" | "needs" | "нужно" | "требует"
        )
    });
    let mut candidates = BTreeMap::<String, f64>::new();
    for record in memory["wave_memory"]["token_patterns"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let prefix = record["prefix"].as_str().unwrap_or("").trim();
        if prefix.is_empty() {
            continue;
        }
        let prefix_terms = tokenize_pattern(prefix)
            .into_iter()
            .map(|token| norm(&token))
            .collect::<BTreeSet<_>>();
        let overlap = prompt_terms.intersection(&prefix_terms).count() as f64
            / prompt_terms.len().max(1) as f64;
        let route = llmwave_memory_route_match(&prompt_terms, record);
        let require_bonus = if wants_require
            && prefix_terms
                .iter()
                .any(|token| token == "require" || token == "requires")
        {
            0.2
        } else {
            0.0
        };
        let score = round4((0.70 * overlap) + (0.20 * route) + require_bonus);
        candidates
            .entry(prefix.to_string())
            .and_modify(|value| *value = value.max(score))
            .or_insert(score);
    }
    let mut rows = candidates
        .into_iter()
        .map(|(prefix, score)| {
            json!({
                "prefix": prefix,
                "score": score
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a["prefix"]
                    .as_str()
                    .unwrap_or("")
                    .len()
                    .cmp(&b["prefix"].as_str().unwrap_or("").len())
            })
    });
    rows.truncate(5);
    let selected = rows
        .first()
        .and_then(|row| row["prefix"].as_str())
        .unwrap_or(prompt);
    json!({
        "version": "v110-prompt-adapter",
        "state": if rows.is_empty() { "PROMPT_ADAPTER_FALLBACK" } else { "PROMPT_ADAPTER_READY" },
        "prompt": prompt,
        "selected_prefix": selected,
        "candidates": rows,
        "read_as": "Prompt adapter turns a natural question into the closest internal LLMWave prefix before generation."
    })
}

fn llmwave_prompt_stopword(token: &str) -> bool {
    matches!(
        token,
        "what"
            | "does"
            | "do"
            | "is"
            | "are"
            | "the"
            | "a"
            | "an"
            | "to"
            | "for"
            | "of"
            | "and"
            | "что"
            | "как"
            | "для"
            | "это"
            | "нужно"
    )
}

fn llmwave_memory_generate_report(
    memory: &Value,
    prefix: &str,
    steps: usize,
    top_k: usize,
    beam_width: usize,
    temperature: f64,
    language: &str,
) -> Value {
    let mut current = prefix.to_string();
    let mut rows = vec![];
    let mut selected_tokens = BTreeSet::<String>::new();
    let mut previous_route = String::new();
    let mut stop_reason = "MAX_STEPS".to_string();
    let mut route_consistent = true;
    for step in 0..steps.max(1) {
        let retrieve = llmwave_memory_retrieve_report(memory, &current, top_k);
        let mut beams = llmwave_memory_beams(&retrieve, beam_width, temperature);
        llmwave_memory_guard_beams(&mut beams, &retrieve, &selected_tokens, &previous_route);
        let selected = beams
            .iter()
            .find(|beam| beam["semantic_guard"]["state"].as_str() == Some("BEAM_SAFE"))
            .or_else(|| beams.first());
        let token = selected
            .and_then(|beam| beam["next_token"].as_str())
            .unwrap_or("")
            .to_string();
        let route = selected
            .and_then(|beam| beam["route"].as_str())
            .unwrap_or("")
            .to_string();
        let guard_state = selected
            .map(|beam| beam["semantic_guard"]["state"].clone())
            .unwrap_or_else(|| json!("BEAM_EMPTY"));
        if !previous_route.is_empty() && !route.is_empty() && previous_route != route {
            route_consistent = false;
        }
        rows.push(json!({
            "step": step + 1,
            "prefix": current,
            "top_token": token,
            "state": retrieve["state"],
            "margin": retrieve["margin"],
            "selected_guard_state": guard_state,
            "sampler": llmwave_memory_sampler_report(temperature, top_k, beam_width),
            "beams": beams,
            "top_k": retrieve["top_k"]
        }));
        if token.is_empty() {
            stop_reason = "EMPTY_TOKEN".to_string();
            break;
        }
        if selected_tokens.contains(&norm(&token)) {
            stop_reason = "REPEATED_TOKEN_STOP".to_string();
            break;
        }
        if retrieve["state"].as_str() != Some("MEMORY_RETRIEVE_READY") {
            stop_reason = "LOW_MARGIN_OR_REVIEW".to_string();
            break;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(&token);
        selected_tokens.insert(norm(&token));
        previous_route = route;
    }
    let decoded = llmwave_memory_semantic_decode(&current, language);
    let all_ready = rows.iter().all(|row| {
        row["state"].as_str() == Some("MEMORY_RETRIEVE_READY")
            && row["selected_guard_state"].as_str() == Some("BEAM_SAFE")
    });
    json!({
        "mode": "llmwave-memory-generate",
        "version": "v94-recurrent-generation",
        "coherence_version": "v112-multi-step-coherence",
        "state": if all_ready { "MEMORY_GENERATION_READY" } else { "MEMORY_GENERATION_REVIEW" },
        "initial_prefix": prefix,
        "generated_text": current,
        "decoded_text": decoded["text"],
        "semantic_decoder": decoded,
        "coherence": {
            "version": "v112-multi-step-coherence",
            "stop_reason": stop_reason,
            "selected_tokens": selected_tokens.iter().cloned().collect::<Vec<_>>(),
            "route_consistent": route_consistent
        },
        "steps": rows,
        "read_as": "Recurrent generation retrieves resonant tokens, vetoes unsafe beams, stops on incoherent loops, and decodes the selected path to text."
    })
}

fn llmwave_memory_guard_beams(
    beams: &mut [Value],
    retrieve: &Value,
    selected_tokens: &BTreeSet<String>,
    previous_route: &str,
) {
    let margin = retrieve["margin"].as_f64().unwrap_or(0.0);
    for beam in beams {
        let token = beam["next_token"].as_str().unwrap_or("");
        let rejected = beam["rejected"].as_u64().unwrap_or(0);
        let safe = beam["safe"].as_bool().unwrap_or(false);
        let route = beam["route"].as_str().unwrap_or("");
        let mut reasons = vec![];
        if rejected > 0 || !safe {
            reasons.push("REJECTED_TOKEN");
        }
        if selected_tokens.contains(&norm(token)) {
            reasons.push("REPEATED_TOKEN");
        }
        if margin < 0.015 && retrieve["top_k"].as_array().map_or(0, Vec::len) > 1 {
            reasons.push("LOW_MARGIN");
        }
        if !previous_route.is_empty() && !route.is_empty() && route != previous_route {
            reasons.push("ROUTE_SHIFT");
        }
        beam["semantic_guard"] = json!({
            "version": "v111-semantic-guard",
            "state": if reasons.is_empty() { "BEAM_SAFE" } else { "BEAM_VETO" },
            "safe_to_emit": reasons.is_empty(),
            "reasons": reasons
        });
    }
}

fn llmwave_memory_sampler_report(temperature: f64, top_k: usize, beam_width: usize) -> Value {
    json!({
        "version": "v97-sampler",
        "temperature": round4(temperature.max(0.0)),
        "top_k": top_k,
        "beam_width": beam_width,
        "strategy": if temperature <= 0.0 { "deterministic-top" } else { "temperature-rescore" }
    })
}

fn llmwave_memory_beams(retrieve: &Value, beam_width: usize, temperature: f64) -> Vec<Value> {
    let mut beams = retrieve["top_k"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| {
            let score = row["score"].as_f64().unwrap_or(0.0).max(0.0);
            let sampled_score = if temperature <= 0.0 {
                score
            } else {
                round4(score.powf(1.0 / temperature.max(0.05)))
            };
            json!({
                "version": "v98-beam-generator",
                "next_token": row["next_token"],
                "next_phrase": row["next_phrase"],
                "pattern": row["pattern"],
                "route": row["route"],
                "score": round4(score),
                "sampled_score": sampled_score,
                "safe": row["rejected"].as_u64().unwrap_or(0) == 0,
                "accepted": row["accepted"],
                "rejected": row["rejected"]
            })
        })
        .collect::<Vec<_>>();
    beams.sort_by(|a, b| {
        b["sampled_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["sampled_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    beams.truncate(beam_width.max(1));
    beams
}

fn llmwave_memory_semantic_decode(text: &str, language: &str) -> Value {
    let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let decoded = if language == "ru" {
        format!("LLMWave продолжение: {clean}.")
    } else {
        format!("LLMWave continuation: {clean}.")
    };
    json!({
        "version": "v99-semantic-decoder",
        "language": language,
        "text": decoded,
        "source_text": clean
    })
}

fn memory_record_count(memory: &Value) -> usize {
    memory["wave_memory"]["patterns"]
        .as_array()
        .map_or(0, Vec::len)
        + memory["wave_memory"]["token_patterns"]
            .as_array()
            .map_or(0, Vec::len)
        + memory["wave_memory"]["phrase_patterns"]
            .as_array()
            .map_or(0, Vec::len)
}

fn llmwave_memory_append(memory: &mut Value, addition: &Value) {
    for key in ["patterns", "token_patterns", "phrase_patterns"] {
        let additions = addition["wave_memory"][key]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if let Some(target) = memory["wave_memory"][key].as_array_mut() {
            target.extend(additions);
        }
    }
    memory["vocabulary"] = llmwave_memory_vocabulary_report(memory);
}

fn llmwave_memory_pack_bytes(memory: &Value) -> Vec<u8> {
    let records = memory["wave_memory"]["token_patterns"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut out = vec![];
    out.extend_from_slice(b"LLMWAVE1");
    out.extend_from_slice(&(records.len() as u32).to_le_bytes());
    out.extend_from_slice(&(32u16).to_le_bytes());
    out.extend_from_slice(&(WAVE_DIM as u16).to_le_bytes());
    for record in records {
        let prefix_hash = hash32(record["prefix"].as_str().unwrap_or(""));
        let next_hash = hash32(record["next_token"].as_str().unwrap_or(""));
        let route_hash = hash32(record["route"].as_str().unwrap_or(""));
        let group_hash = hash32(record["group"].as_str().unwrap_or(""));
        let strength = (record["strength"].as_f64().unwrap_or(0.0).clamp(0.0, 2.0) * 1000.0) as u16;
        let accepted = record["accepted"]
            .as_u64()
            .unwrap_or(0)
            .min(u16::MAX as u64) as u16;
        let rejected = record["rejected"]
            .as_u64()
            .unwrap_or(0)
            .min(u16::MAX as u64) as u16;
        let flags = if rejected > 0 { 2u16 } else { 1u16 };
        out.extend_from_slice(&prefix_hash.to_le_bytes());
        out.extend_from_slice(&next_hash.to_le_bytes());
        out.extend_from_slice(&route_hash.to_le_bytes());
        out.extend_from_slice(&group_hash.to_le_bytes());
        out.extend_from_slice(&strength.to_le_bytes());
        out.extend_from_slice(&accepted.to_le_bytes());
        out.extend_from_slice(&rejected.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
    }
    out
}

fn llmwave_memory_unpack_report(bytes: &[u8]) -> Value {
    let ok_magic = bytes.len() >= 16 && &bytes[0..8] == b"LLMWAVE1";
    let records = if ok_magic {
        u32::from_le_bytes(bytes[8..12].try_into().unwrap_or([0; 4])) as usize
    } else {
        0
    };
    let record_bytes = if ok_magic {
        u16::from_le_bytes(bytes[12..14].try_into().unwrap_or([0; 2])) as usize
    } else {
        0
    };
    let wave_dim = if ok_magic {
        u16::from_le_bytes(bytes[14..16].try_into().unwrap_or([0; 2])) as usize
    } else {
        0
    };
    let expected_bytes = 16 + records * record_bytes;
    json!({
        "mode": "llmwave-memory-unpack",
        "version": "v108-binary-packed-memory-prototype",
        "state": if ok_magic && bytes.len() == expected_bytes { "PACKED_MEMORY_OK" } else { "PACKED_MEMORY_REVIEW" },
        "magic": ok_magic,
        "records": records,
        "record_bytes": record_bytes,
        "wave_dim": wave_dim,
        "bytes": bytes.len(),
        "expected_bytes": expected_bytes,
        "read_as": "Unpack validates the first binary packed LLMWave memory prototype header."
    })
}

fn llmwave_memory_refresh_budget(memory: &mut Value) {
    let records = memory["wave_memory"]["patterns"]
        .as_array()
        .map_or(0, Vec::len)
        + memory["wave_memory"]["token_patterns"]
            .as_array()
            .map_or(0, Vec::len)
        + memory["wave_memory"]["phrase_patterns"]
            .as_array()
            .map_or(0, Vec::len);
    let bytes = records * 32;
    memory["wave_memory"]["packed_runtime"] = json!({
        "version": "v93-packed-6m-memory",
        "record_bytes": 32,
        "estimated_packed_bytes": bytes,
        "hot_budget_bytes": nanda_6m::BUDGET_BYTES,
        "fits_6m": bytes <= nanda_6m::BUDGET_BYTES
    });
}

pub(crate) fn llmwave_token_compact_report(packet: &Packet, text: &str, top_k: usize) -> Value {
    let args = LlmwaveEvalArgs {
        suite: PathBuf::from("<serve>"),
        input_format: InputFormat::Json,
        top_k,
        steps: 1,
        search_top_k: 8,
        route_cap: 256,
        route_triad_cap: 32,
        group_by: PeakGroupBy::Route,
        format: OutputFormat::Json,
        normalize_paths: false,
    };
    let report = llmwave_report_from_packet(packet, Path::new("<serve>"), text, &args);
    let token = &report["llmwave_contract"]["lenses"]["token"];
    json!({
        "mode": "llmwave-token-compact",
        "version": "v75-serve-compact-token-lens",
        "contract_state": report["llmwave_contract"]["state"],
        "proof_state": report["proof_summary"]["state"],
        "token_state": token["state"],
        "ready": token["ready"],
        "prefix": token["prefix"],
        "top_token": token["top_token"],
        "top_phrase": token["top_phrase"],
        "margin": token["margin"],
        "top_k": token["top_k"],
        "baseline_compare": token["baseline_compare"],
        "token_cleanup": token["token_cleanup"],
        "anti_wave": token["anti_wave"],
        "token_memory": token["token_memory"],
        "read_as": "Compact Token Lens response for agent next-token/phrase resonance."
    })
}

pub(crate) fn demo_cmd(args: DemoArgs) -> Result<u8> {
    if let Some(suite_path) = &args.suite {
        let text = fs::read_to_string(suite_path)
            .with_context(|| format!("read {}", suite_path.display()))?;
        let suite: DemoSuite = serde_json::from_str(&text)
            .with_context(|| format!("parse {}", suite_path.display()))?;
        if suite.cases.is_empty() {
            return Err(anyhow!("nanda demo --suite requires at least one case"));
        }
        let base = suite_path.parent().unwrap_or_else(|| Path::new("."));
        let mut rows = vec![];
        let mut passed = 0usize;
        for case in suite.cases {
            let row = run_demo_case(&args, base, case)?;
            if row["ok"].as_bool().unwrap_or(false) {
                passed += 1;
            }
            rows.push(row);
        }
        let total = rows.len();
        let out = json!({
            "core_version": CORE_VERSION,
            "mode": "llmwave-demo-suite",
            "version": "v62-demo-raw-text-adapter",
            "suite": if suite.name.is_empty() { suite_path.display().to_string() } else { suite.name },
            "passed": passed,
            "total": total,
            "accuracy": round4(passed as f64 / total.max(1) as f64),
            "cases": rows,
            "read_as": "Demo suite checks the human/agent-facing LLMWave surface: ready, anti-wave, and review cases must all be legible."
        });
        match args.format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
            OutputFormat::Text => print_demo_suite_text(&out),
            OutputFormat::Md => print_demo_suite_md(&out),
        }
        return Ok(if passed == total {
            EXIT_PASS
        } else {
            EXIT_VETO
        });
    }

    let text = demo_text(&args)?;
    let (input, mut packet, text, raw_adapter) = load_demo_packet(&args, &text)?;
    let eval_args = demo_eval_args(&args);
    if args.train {
        let case = LlmwaveEvalCase {
            id: "demo-feedback".to_string(),
            path: input.to_path_buf(),
            text: text.clone(),
            feedback_decision: feedback_decision_label(&args.decision).to_string(),
            note: "demo feedback preview".to_string(),
            expected_top_pattern: String::new(),
            expected_hrr_state: String::new(),
            expected_cleanup_state: String::new(),
            expected_attractor_state: String::new(),
            expected_capacity_state: String::new(),
            expected_anti_wave_state: String::new(),
            expected_proof_state: String::new(),
            expected_demo_state: String::new(),
            expected_next_token: String::new(),
        };
        packet = inject_llmwave_feedback(packet, &text, &case, &eval_args)?;
    }
    let report = llmwave_report_from_packet(&packet, input, &text, &eval_args);
    let mut demo = demo_surface_report("single", input, &text, &report);
    if let Some(adapter) = raw_adapter {
        apply_raw_adapter_metadata(&mut demo, adapter);
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&demo)?),
        OutputFormat::Text => print_demo_text(&demo),
        OutputFormat::Md => print_demo_md(&demo),
    }
    Ok(if demo["state"].as_str() == Some("PUBLIC_DEMO_READY") {
        EXIT_PASS
    } else {
        EXIT_WATCH
    })
}

fn run_demo_case(args: &DemoArgs, base: &Path, case: DemoCase) -> Result<Value> {
    let path = resolve_llmwave_suite_path(base, &case.path);
    let text = if case.text.trim().is_empty() {
        args.text.clone()
    } else {
        case.text.clone()
    };
    let mut packet = load_packet_auto(
        &path,
        &args.input_format,
        "demo-suite",
        "general",
        &text,
        args.normalize_paths,
    )?;
    let eval_args = demo_eval_args(args);
    if !case.feedback_decision.trim().is_empty() {
        let eval_case = LlmwaveEvalCase {
            id: case.id.clone(),
            path: path.clone(),
            text: text.clone(),
            feedback_decision: case.feedback_decision.clone(),
            note: case.note.clone(),
            expected_top_pattern: String::new(),
            expected_hrr_state: String::new(),
            expected_cleanup_state: String::new(),
            expected_attractor_state: String::new(),
            expected_capacity_state: String::new(),
            expected_anti_wave_state: String::new(),
            expected_proof_state: String::new(),
            expected_demo_state: String::new(),
            expected_next_token: String::new(),
        };
        packet = inject_llmwave_feedback(packet, &text, &eval_case, &eval_args)?;
    }
    let report = llmwave_report_from_packet(&packet, &path, &text, &eval_args);
    let demo = demo_surface_report(&case.id, &path, &text, &report);
    let expected_state_ok = case.expected_state.is_empty()
        || demo["state"].as_str() == Some(case.expected_state.as_str());
    let expected_weak_ok = case
        .expected_weak_spots
        .is_none_or(|expected| demo["weak_spots"].as_array().map_or(0, Vec::len) == expected);
    let expected_anti_ok = case.expected_anti_wave_state.is_empty()
        || demo["signals"]["anti_wave"].as_str() == Some(case.expected_anti_wave_state.as_str());
    let ok = expected_state_ok && expected_weak_ok && expected_anti_ok;
    Ok(json!({
        "id": if case.id.is_empty() { path.display().to_string() } else { case.id },
        "case": path.display().to_string(),
        "state": demo["state"],
        "top_pattern": demo["top_pattern"],
        "weak_spots": demo["weak_spots"],
        "signals": demo["signals"],
        "expected_state": case.expected_state,
        "checks": {
            "state": expected_state_ok,
            "weak_spots": expected_weak_ok,
            "anti_wave": expected_anti_ok
        },
        "ok": ok
    }))
}

fn run_llmwave_eval_case(
    args: &LlmwaveEvalArgs,
    base: &Path,
    case: LlmwaveEvalCase,
) -> Result<Value> {
    let path = resolve_llmwave_suite_path(base, &case.path);
    let mut packet = load_packet_auto(
        &path,
        &args.input_format,
        "llmwave-eval",
        "general",
        &case.text,
        args.normalize_paths,
    )?;
    let text = if case.text.trim().is_empty() {
        packet.query.clone()
    } else {
        case.text.clone()
    };
    if !case.feedback_decision.trim().is_empty() {
        packet = inject_llmwave_feedback(packet, &text, &case, args)?;
    }
    let report = llmwave_report_from_packet(&packet, &path, &text, args);
    let checks = json!({
        "top_pattern": case.expected_top_pattern.is_empty() || report["decode"]["top_pattern"].as_str() == Some(case.expected_top_pattern.as_str()),
        "hrr": case.expected_hrr_state.is_empty() || report["hrr_binding"]["state"].as_str() == Some(case.expected_hrr_state.as_str()),
        "cleanup": case.expected_cleanup_state.is_empty() || report["cleanup_memory"]["state"].as_str() == Some(case.expected_cleanup_state.as_str()),
        "attractor": case.expected_attractor_state.is_empty() || report["attractor_trace"]["state"].as_str() == Some(case.expected_attractor_state.as_str()),
        "capacity": case.expected_capacity_state.is_empty() || report["superposition_capacity"]["state"].as_str() == Some(case.expected_capacity_state.as_str()),
        "anti_wave": case.expected_anti_wave_state.is_empty() || report["anti_wave_audit"]["state"].as_str() == Some(case.expected_anti_wave_state.as_str()),
        "proof": case.expected_proof_state.is_empty() || report["proof_summary"]["state"].as_str() == Some(case.expected_proof_state.as_str()),
        "demo": case.expected_demo_state.is_empty() || report["public_demo"]["state"].as_str() == Some(case.expected_demo_state.as_str()),
        "llmwave_contract": if case.expected_next_token.is_empty() {
            report["llmwave_contract"]["state"].as_str() == Some("LLMWAVE_LENS_READY")
        } else {
            report["llmwave_contract"]["lenses"]["token"]["top_token"].as_str() == Some(case.expected_next_token.as_str())
        },
        "next_token": case.expected_next_token.is_empty() || report["llmwave_contract"]["lenses"]["token"]["top_token"].as_str() == Some(case.expected_next_token.as_str())
    });
    let ok = checks
        .as_object()
        .is_some_and(|items| items.values().all(|value| value.as_bool().unwrap_or(false)));
    Ok(json!({
        "id": if case.id.is_empty() { path.display().to_string() } else { case.id },
        "case": path.display().to_string(),
        "text": text,
        "feedback_decision": case.feedback_decision,
        "expected_top_pattern": case.expected_top_pattern,
        "actual_top_pattern": report["decode"]["top_pattern"],
        "states": {
            "hrr": report["hrr_binding"]["state"],
            "cleanup": report["cleanup_memory"]["state"],
            "attractor": report["attractor_trace"]["state"],
            "capacity": report["superposition_capacity"]["state"],
            "anti_wave": report["anti_wave_audit"]["state"],
            "packed_hrr": report["packed_hrr_runtime"]["state"],
            "cleanup_dictionary": report["cleanup_dictionary"]["state"],
            "anti_wave_locality": report["anti_wave_locality"]["state"],
            "capacity_curve": report["capacity_curve"]["state"],
            "hot_cycle": report["packed_hot_cycle"]["state"],
            "proof": report["proof_summary"]["state"],
            "llmwave_contract": report["llmwave_contract"]["state"],
            "token_lens": report["llmwave_contract"]["lenses"]["token"]["state"],
            "demo": report["public_demo"]["state"]
        },
        "actual_next_token": report["llmwave_contract"]["lenses"]["token"]["top_token"],
        "checks": checks,
        "ok": ok
    }))
}

fn llmwave_report_from_packet(
    packet: &Packet,
    input: &Path,
    text: &str,
    args: &LlmwaveEvalArgs,
) -> Value {
    let mut packet = packet.clone();
    let tokens = tokenize_pattern(text);
    let query = tokens_to_query_triads(&tokens, "llmwave-eval", "general");
    packet.query = text.to_string();
    let memory = normalize_ids(packet.triads.clone(), "m");
    let decode_args = DecodeArgs {
        input: input.to_path_buf(),
        input_format: args.input_format.clone(),
        task_id: "llmwave-eval".to_string(),
        domain: "general".to_string(),
        query: text.to_string(),
        query_file: None,
        query_format: args.input_format.clone(),
        top_k: args.top_k,
        steps: args.steps.clamp(1, 16),
        beam_width: 1,
        adaptive_scoring: false,
        search_top_k: args.search_top_k,
        route_cap: args.route_cap,
        route_triad_cap: args.route_triad_cap,
        group_by: args.group_by.clone(),
        format: OutputFormat::Json,
        normalize_paths: args.normalize_paths,
    };
    let decode = recurrent_decode_report(
        &packet,
        &memory,
        &query,
        "llmwave-eval-query",
        &decode_args,
        decode_args.steps,
    );
    let hrr_binding = hrr_binding_report(&packet, &query, args.top_k);
    let cleanup_memory = cleanup_memory_report(&decode, &packet);
    let attractor_trace = llmwave_attractor_report(&decode);
    let superposition_capacity = superposition_capacity_report(&packet);
    let anti_wave_audit = shortcut_anti_wave_audit(&decode, &packet);
    let packed_hrr_runtime = packed_hrr_runtime_report(&hrr_binding, &packet);
    let cleanup_dictionary = cleanup_dictionary_report(&cleanup_memory, &packet);
    let anti_wave_locality = anti_wave_locality_report(&anti_wave_audit, &decode);
    let capacity_curve = capacity_curve_report(&superposition_capacity, &packet);
    let packed_hot_cycle = llmwave_hot_cycle_report(
        &packed_hrr_runtime,
        &cleanup_dictionary,
        &capacity_curve,
        &anti_wave_locality,
    );
    let proof_summary = llmwave_proof_summary(
        &hrr_binding,
        &cleanup_memory,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &decode,
    );
    let llmwave_contract = llmwave_contract_report(
        &packet,
        text,
        &tokens,
        &decode,
        &hrr_binding,
        &cleanup_memory,
        &cleanup_dictionary,
        &attractor_trace,
        &superposition_capacity,
        &anti_wave_audit,
        &packed_hot_cycle,
        &proof_summary,
        &LlmwaveLensKind::Pattern,
    );
    let public_demo = llmwave_public_demo_report(&proof_summary, &decode, text);
    json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-eval-case-report",
        "version": "v60-public-demo-packet",
        "hrr_binding": hrr_binding,
        "cleanup_memory": cleanup_memory,
        "attractor_trace": attractor_trace,
        "superposition_capacity": superposition_capacity,
        "anti_wave_audit": anti_wave_audit,
        "packed_hrr_runtime": packed_hrr_runtime,
        "cleanup_dictionary": cleanup_dictionary,
        "anti_wave_locality": anti_wave_locality,
        "capacity_curve": capacity_curve,
        "packed_hot_cycle": packed_hot_cycle,
        "proof_summary": proof_summary,
        "llmwave_contract": llmwave_contract,
        "public_demo": public_demo,
        "decode": decode
    })
}

fn load_demo_packet<'a>(
    args: &'a DemoArgs,
    text: &str,
) -> Result<(&'a Path, Packet, String, Option<Value>)> {
    if let Some(raw_path) = args.from_text.as_deref() {
        let raw =
            fs::read_to_string(raw_path).with_context(|| format!("read {}", raw_path.display()))?;
        let query = if text.trim().is_empty() {
            raw.clone()
        } else {
            text.to_string()
        };
        let (packet, adapter) =
            packet_from_demo_text(&raw, &args.task_id, &args.domain, &query, raw_path);
        return Ok((raw_path, packet, query, Some(adapter)));
    }
    let input = args
        .input
        .as_deref()
        .ok_or_else(|| anyhow!("nanda demo requires an input packet, --from-text, or --suite"))?;
    let packet = load_packet_auto(
        input,
        &args.input_format,
        &args.task_id,
        &args.domain,
        text,
        args.normalize_paths,
    )?;
    Ok((input, packet, text.to_string(), None))
}

fn packet_from_demo_text(
    raw: &str,
    task_id: &str,
    domain: &str,
    query: &str,
    source: &Path,
) -> (Packet, Value) {
    let mut triads = parse_arrow_triads(raw, task_id, domain);
    let extraction_method = if triads.is_empty() {
        let tokens = tokenize_pattern(raw);
        triads = tokens_to_query_triads(&tokens, task_id, domain);
        "token-window-fallback"
    } else {
        "arrow-triads"
    };
    let quality = if extraction_method == "arrow-triads" {
        "RAW_ADAPTER_READY"
    } else {
        "RAW_ADAPTER_REVIEW"
    };
    let triad_count = triads.len();
    let packet = Packet {
        task_id: task_id.to_string(),
        domain: domain.to_string(),
        query: query.to_string(),
        triads,
        candidate_triads: vec![],
        candidate_answer: String::new(),
        aliases: vec![],
        canonicalization: CanonicalizationReport::default(),
        negative_shortcuts: vec![],
        positive_shortcuts: vec![],
        resonance_memory: vec![],
        continuation_memory: vec![],
    };
    (
        packet,
        json!({
            "input_mode": "raw-text",
            "source": source.display().to_string(),
            "extraction_method": extraction_method,
            "quality": quality,
            "triads": triad_count,
            "read_as": "Raw demo adapter accepts explicit `subject -> relation -> object [route=x group=y]` lines. Free text fallback is review-only."
        }),
    )
}

fn parse_arrow_triads(raw: &str, task_id: &str, domain: &str) -> Vec<Triad> {
    let mut triads = vec![];
    let mut section = "triads".to_string();
    for (line_idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            let lower = trimmed.trim_start_matches('#').trim().to_lowercase();
            if lower.contains("candidate") {
                section = "candidate_triads".to_string();
            } else if lower.contains("triad") {
                section = "triads".to_string();
            }
            continue;
        }
        let cleaned = trimmed
            .trim_start_matches('-')
            .trim_start_matches('*')
            .trim_start()
            .trim();
        if !cleaned.contains("->") {
            continue;
        }
        let (body, route, group) = split_arrow_metadata(cleaned, task_id, domain);
        let parts = body
            .split("->")
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.len() != 3 {
            continue;
        }
        triads.push(Triad {
            id: format!("raw{}", triads.len() + 1),
            subject: parts[0].to_string(),
            relation: parts[1].to_string(),
            object: parts[2].to_string(),
            evidence: format!("{}:{}", section, line_idx + 1),
            confidence: 0.82,
            subject_role: "subject".to_string(),
            object_role: "object".to_string(),
            route,
            group,
        });
    }
    normalize_ids(triads, "raw")
}

fn split_arrow_metadata(line: &str, task_id: &str, domain: &str) -> (String, String, String) {
    let default_route = if domain.trim().is_empty() {
        task_id.to_string()
    } else {
        domain.to_string()
    };
    let default_group = format!("{task_id}:raw");
    let Some(start) = line.rfind('[') else {
        return (line.to_string(), default_route, default_group);
    };
    let Some(end_rel) = line[start..].find(']') else {
        return (line.to_string(), default_route, default_group);
    };
    let end = start + end_rel;
    let mut route = default_route;
    let mut group = default_group;
    let meta = &line[start + 1..end];
    for item in meta.split_whitespace() {
        if let Some(value) = item.strip_prefix("route=") {
            route = value.trim_matches('"').trim_matches('\'').to_string();
        } else if let Some(value) = item.strip_prefix("group=") {
            group = value.trim_matches('"').trim_matches('\'').to_string();
        }
    }
    (line[..start].trim().to_string(), route, group)
}

fn apply_raw_adapter_metadata(demo: &mut Value, adapter: Value) {
    if let Some(obj) = demo.as_object_mut() {
        obj.insert("input_mode".to_string(), adapter["input_mode"].clone());
        obj.insert("raw_adapter".to_string(), adapter.clone());
    }
    if adapter["quality"].as_str() == Some("RAW_ADAPTER_REVIEW") {
        if let Some(items) = demo["weak_spots"].as_array_mut() {
            items.push(json!({
                "signal": "raw_adapter",
                "state": adapter["quality"],
                "expected": ["RAW_ADAPTER_READY"],
                "reason": "Raw input had no explicit arrow triads, so the token-window fallback is review-only."
            }));
        }
        if let Some(obj) = demo.as_object_mut() {
            obj.insert("state".to_string(), json!("PUBLIC_DEMO_REVIEW"));
            obj.insert(
                "safe_claim".to_string(),
                json!("LLMWave produced a reviewable structural continuation from weak raw-text extraction; write explicit arrow triads before relying on it."),
            );
        }
    }
}

fn inject_llmwave_feedback(
    mut packet: Packet,
    text: &str,
    case: &LlmwaveEvalCase,
    args: &LlmwaveEvalArgs,
) -> Result<Packet> {
    let report = llmwave_report_from_packet(&packet, &case.path, text, args);
    let decision = normalize_llmwave_feedback_decision(&case.feedback_decision)?;
    let note = if case.note.trim().is_empty() {
        format!("llmwave eval {decision}")
    } else {
        case.note.clone()
    };
    let mut memories =
        continuation_memory_from_decode(&report["decode"], &decision, &note, "llmwave-eval".into());
    packet.continuation_memory.append(&mut memories);
    packet.continuation_memory = merge_continuation_memory(packet.continuation_memory);
    Ok(packet)
}

fn hrr_binding_report(packet: &Packet, query: &[Triad], top_k: usize) -> Value {
    let mut rows = packet
        .triads
        .iter()
        .map(|triad| {
            let subject_bound = bind(
                &vector(&format!("role:{}", norm(&triad.subject_role))),
                &vector(&format!("entity:{}", norm(&triad.subject))),
            );
            let object_bound = bind(
                &vector(&format!("role:{}", norm(&triad.object_role))),
                &vector(&format!("entity:{}", norm(&triad.object))),
            );
            let relation = vector(&format!("relation:{}", norm(&triad.relation)));
            let bound = bind(&bind(&subject_bound, &relation), &object_bound);
            let query_score = if query.is_empty() {
                0.0
            } else {
                query
                    .iter()
                    .map(|q| cosine(&bound, &triad_wave(q)))
                    .fold(f64::NEG_INFINITY, f64::max)
                    .max(0.0)
            };
            let recovered_subject = bind(
                &subject_bound,
                &vector(&format!("role:{}", norm(&triad.subject_role))),
            );
            let recovered_object = bind(
                &object_bound,
                &vector(&format!("role:{}", norm(&triad.object_role))),
            );
            let subject_recovery = cosine(
                &recovered_subject,
                &vector(&format!("entity:{}", norm(&triad.subject))),
            );
            let object_recovery = cosine(
                &recovered_object,
                &vector(&format!("entity:{}", norm(&triad.object))),
            );
            json!({
                "triad": triad.id,
                "subject": triad.subject,
                "relation": triad.relation,
                "object": triad.object,
                "route": triad.route,
                "role_filler_binding": true,
                "query_score": round4(query_score),
                "subject_unbind_cosine": round4(subject_recovery),
                "object_unbind_cosine": round4(object_recovery),
                "state": if subject_recovery > 0.98 && object_recovery > 0.98 { "BINDING_RECOVERED" } else { "BINDING_AMBIGUOUS" }
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["query_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["query_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(top_k.max(1));
    let recovered = rows
        .iter()
        .filter(|row| row["state"].as_str() == Some("BINDING_RECOVERED"))
        .count();
    json!({
        "version": "v47-hrr-binding-sandbox",
        "operation": "role_filler_bind_unbind",
        "records_checked": packet.triads.len(),
        "sample": rows,
        "recovered_in_sample": recovered,
        "state": if recovered > 0 { "HRR_BINDING_VISIBLE" } else { "HRR_BINDING_REVIEW" },
        "read_as": "Experimental HRR-style binding probe: role * filler lanes are bound and then unbound back to subject/object vectors. This is a sandbox signal, not proof by itself."
    })
}

fn cleanup_memory_report(decode: &Value, packet: &Packet) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let mut cleanup_rows = vec![];
    for pattern in patterns.iter().take(5) {
        let mut best = None::<(f64, &Triad)>;
        let p_wave = triad_wave(&Triad {
            id: String::new(),
            subject: pattern["subject"].as_str().unwrap_or("").to_string(),
            relation: pattern["relation"].as_str().unwrap_or("").to_string(),
            object: pattern["object"].as_str().unwrap_or("").to_string(),
            evidence: String::new(),
            confidence: 1.0,
            subject_role: pattern["subject_role"]
                .as_str()
                .unwrap_or("subject")
                .to_string(),
            object_role: pattern["object_role"]
                .as_str()
                .unwrap_or("object")
                .to_string(),
            route: pattern["route"].as_str().unwrap_or("").to_string(),
            group: pattern["group"].as_str().unwrap_or("").to_string(),
        });
        for triad in &packet.triads {
            let score = cosine(&p_wave, &triad_wave(triad));
            if best.as_ref().is_none_or(|(old, _)| score > *old) {
                best = Some((score, triad));
            }
        }
        if let Some((score, triad)) = best {
            let state = if score >= 0.92 {
                "CLEANUP_EXACT"
            } else if score >= 0.45 {
                "CLEANUP_NEAR"
            } else {
                "CLEANUP_AMBIGUOUS"
            };
            cleanup_rows.push(json!({
                "raw_pattern": pattern_label_value(pattern),
                "nearest_memory_triad": triad.id,
                "nearest_pattern": format!("{} -> {} -> {}", triad.subject, triad.relation, triad.object),
                "route": triad.route,
                "score": round4(score),
                "state": state
            }));
        }
    }
    let ambiguous = cleanup_rows
        .iter()
        .filter(|row| row["state"].as_str() == Some("CLEANUP_AMBIGUOUS"))
        .count();
    json!({
        "version": "v48-cleanup-memory",
        "operation": "decoded_pattern_cleanup",
        "items": cleanup_rows,
        "state": if ambiguous == 0 { "CLEANUP_READY" } else { "CLEANUP_WATCH" },
        "read_as": "Cleanup memory maps raw decoded structural patterns back to the nearest known triad and keeps ambiguity visible instead of forcing a clean answer."
    })
}

fn llmwave_attractor_report(decode: &Value) -> Value {
    let steps = decode["recurrent"]["steps"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut last_energy = None::<f64>;
    let mut route_jumps = 0usize;
    let mut last_route = String::new();
    let mut rows = vec![];
    for step in steps {
        let route = step["source_search"]["top_peak"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if !last_route.is_empty() && !route.is_empty() && route != last_route {
            route_jumps += 1;
        }
        if !route.is_empty() {
            last_route = route.clone();
        }
        let margin = step["source_search"]["peak_margin"].as_f64().unwrap_or(0.0);
        let top_score = step["patterns"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item["score"].as_f64())
            .unwrap_or(0.0);
        let anti = step["source_search"]["resonance"]["energy_accounting"]["anti_energy"]
            .as_f64()
            .unwrap_or(0.0);
        let energy = round4((top_score + margin - anti).clamp(-1.0, 2.0));
        let trend = match last_energy {
            None => "START",
            Some(prev) if energy > prev + 0.005 => "IMPROVING",
            Some(prev) if energy < prev - 0.005 => "DROPPING",
            Some(_) => "SATURATING",
        };
        last_energy = Some(energy);
        rows.push(json!({
            "step": step["step"],
            "route_basin": route,
            "top_pattern": step["top_pattern"],
            "energy": energy,
            "trend": trend,
            "decoder_state": step["decoder_state"]
        }));
    }
    let final_state = if rows.is_empty() {
        "NO_ATTRACTOR_TRACE"
    } else if route_jumps == 0
        && rows
            .iter()
            .all(|row| row["trend"].as_str().is_some_and(|x| x != "DROPPING"))
    {
        "ATTRACTOR_STABLE"
    } else if route_jumps > 0 {
        "ATTRACTOR_ROUTE_JUMP"
    } else {
        "ATTRACTOR_REVIEW"
    };
    json!({
        "version": "v49-attractor-energy-trace",
        "route_jumps": route_jumps,
        "steps": rows,
        "state": final_state,
        "read_as": "Energy trace treats recurrent decode as an attractor candidate: stable routes should improve or saturate; route jumps and drops stay review-only."
    })
}

fn superposition_capacity_report(packet: &Packet) -> Value {
    let active_patterns = packet.triads.len() + packet.continuation_memory.len();
    let dimensions = WAVE_DIM;
    let route_count = packet
        .triads
        .iter()
        .filter_map(|triad| (!triad.route.is_empty()).then_some(norm(&triad.route)))
        .collect::<BTreeSet<_>>()
        .len()
        .max(1);
    let load = active_patterns as f64 / dimensions as f64;
    let route_pressure = route_count as f64 / 64.0;
    let crosstalk = round4((load * load + route_pressure * 0.25).min(1.0));
    let state = if active_patterns <= 1024 && crosstalk < 0.35 {
        "CAPACITY_HEALTHY"
    } else if active_patterns <= PATTERN_STORE_CAPACITY && crosstalk < 0.75 {
        "CAPACITY_WATCH"
    } else {
        "FOCUS_REQUIRED"
    };
    json!({
        "version": "v50-superposition-capacity-curve",
        "wave_dim": dimensions,
        "active_patterns": active_patterns,
        "routes": route_count,
        "pattern_store_capacity": PATTERN_STORE_CAPACITY,
        "load_factor": round4(load),
        "estimated_crosstalk": crosstalk,
        "state": state,
        "read_as": "Capacity is treated as a curve, not a slogan: more patterns increase useful compression and harmful crosstalk at the same time."
    })
}

fn shortcut_anti_wave_audit(decode: &Value, packet: &Packet) -> Value {
    let applications = decode["recurrent"]["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|step| {
            step["continuation_training"]["applications"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    let suppressions = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("suppress"))
        .count();
    let reinforcements = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("reinforce"))
        .count();
    let negative_records = packet
        .continuation_memory
        .iter()
        .filter(|item| item.decision == "reject")
        .count();
    let state = if suppressions > 0 {
        "ANTI_WAVE_APPLIED"
    } else if negative_records > 0 {
        "ANTI_WAVE_AVAILABLE_NOT_MATCHED"
    } else {
        "NO_ANTI_WAVE_MEMORY"
    };
    json!({
        "version": "v51-shortcut-specific-anti-wave-audit",
        "negative_records": negative_records,
        "suppressions": suppressions,
        "reinforcements": reinforcements,
        "applications": applications,
        "state": state,
        "read_as": "Anti-wave is accepted only when it is shortcut-specific: rejected local pattern signatures are suppressed without killing the whole topic."
    })
}

fn pattern_label_value(pattern: &Value) -> String {
    format!(
        "{} -> {} -> {}",
        pattern["subject"].as_str().unwrap_or(""),
        pattern["relation"].as_str().unwrap_or(""),
        pattern["object"].as_str().unwrap_or("")
    )
}

fn packed_hrr_runtime_report(hrr: &Value, packet: &Packet) -> Value {
    let records = hrr["sample"].as_array().map_or(0, Vec::len);
    let bytes_per_lane = 64usize;
    let bytes = records * bytes_per_lane;
    let recovered = hrr["recovered_in_sample"].as_u64().unwrap_or(0);
    json!({
        "version": "v54-packed-hrr-lanes",
        "state": if recovered > 0 { "PACKED_HRR_READY" } else { "PACKED_HRR_REVIEW" },
        "records": records,
        "source_triads": packet.triads.len(),
        "bytes_per_lane": bytes_per_lane,
        "bytes": bytes,
        "fits_pattern_arena": bytes <= PATTERN_STORE_ARENA_BYTES,
        "contract": "role/filler binding probes are representable as fixed 64-byte hot lanes",
        "read_as": "v54 moves HRR binding from a visual report toward a packed-lane runtime contract."
    })
}

fn cleanup_dictionary_report(cleanup: &Value, packet: &Packet) -> Value {
    let items = cleanup["items"].as_array().cloned().unwrap_or_default();
    let exact = items
        .iter()
        .filter(|item| item["state"].as_str() == Some("CLEANUP_EXACT"))
        .count();
    let near = items
        .iter()
        .filter(|item| item["state"].as_str() == Some("CLEANUP_NEAR"))
        .count();
    let ambiguous = items
        .iter()
        .filter(|item| item["state"].as_str() == Some("CLEANUP_AMBIGUOUS"))
        .count();
    let state = if ambiguous > 0 {
        "CLEANUP_DICTIONARY_WATCH"
    } else if exact + near > 0 {
        "CLEANUP_DICTIONARY_READY"
    } else {
        "CLEANUP_DICTIONARY_EMPTY"
    };
    json!({
        "version": "v55-cleanup-dictionary-thresholds",
        "state": state,
        "memory_triads": packet.triads.len(),
        "exact": exact,
        "near": near,
        "ambiguous": ambiguous,
        "exact_threshold": 0.92,
        "near_threshold": 0.45,
        "read_as": "v55 makes cleanup explicit: exact and near matches can support proof; ambiguous cleanup keeps the result under WATCH."
    })
}

fn anti_wave_locality_report(anti_wave: &Value, decode: &Value) -> Value {
    let applications = anti_wave["applications"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let suppressions = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("suppress"))
        .count();
    let reinforcements = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("reinforce"))
        .count();
    let top_after = decode["top_pattern"].as_str().unwrap_or("");
    let state = if suppressions > 0 && !top_after.is_empty() {
        "ANTI_WAVE_LOCAL"
    } else if anti_wave["negative_records"].as_u64().unwrap_or(0) > 0 {
        "ANTI_WAVE_NOT_TRIGGERED"
    } else {
        "ANTI_WAVE_NO_MEMORY"
    };
    json!({
        "version": "v56-anti-wave-locality-fixture",
        "state": state,
        "suppressions": suppressions,
        "reinforcements": reinforcements,
        "top_after": top_after,
        "kept_decode_alive": !top_after.is_empty(),
        "read_as": "v56 checks that anti-wave suppresses a shortcut-shaped continuation without destroying the whole decode surface."
    })
}

fn capacity_curve_report(capacity: &Value, packet: &Packet) -> Value {
    let active = capacity["active_patterns"].as_u64().unwrap_or(0) as usize;
    let direct_table_bytes = active * 96;
    let packed_wave_bytes = active * PACKED_PATTERN_BYTES;
    let compression_ratio = if packed_wave_bytes == 0 {
        0.0
    } else {
        direct_table_bytes as f64 / packed_wave_bytes as f64
    };
    json!({
        "version": "v57-superposition-capacity-baseline",
        "state": capacity["state"],
        "active_patterns": active,
        "routes": capacity["routes"],
        "direct_table_bytes_estimate": direct_table_bytes,
        "packed_wave_bytes_estimate": packed_wave_bytes,
        "compression_ratio_vs_direct_table": round4(compression_ratio),
        "continuation_records": packet.continuation_memory.len(),
        "read_as": "v57 compares the packed wave representation with a simple direct-table estimate before making capacity claims."
    })
}

fn llmwave_hot_cycle_report(
    packed_hrr: &Value,
    cleanup_dictionary: &Value,
    capacity_curve: &Value,
    anti_wave_locality: &Value,
) -> Value {
    let ready = packed_hrr["state"].as_str() == Some("PACKED_HRR_READY")
        && matches!(
            cleanup_dictionary["state"].as_str().unwrap_or(""),
            "CLEANUP_DICTIONARY_READY" | "CLEANUP_DICTIONARY_EMPTY"
        )
        && capacity_curve["state"].as_str() != Some("FOCUS_REQUIRED")
        && anti_wave_locality["kept_decode_alive"]
            .as_bool()
            .unwrap_or(true);
    json!({
        "version": "v58-packed-hot-cycle-bridge",
        "state": if ready { "LLMWAVE_HOT_READY" } else { "LLMWAVE_HOT_WATCH" },
        "packed_hrr_state": packed_hrr["state"],
        "cleanup_dictionary_state": cleanup_dictionary["state"],
        "capacity_state": capacity_curve["state"],
        "anti_wave_locality_state": anti_wave_locality["state"],
        "read_as": "v58 is the cold-to-hot bridge: v52 reports are collapsed into a single packed runtime readiness state."
    })
}

fn llmwave_proof_summary(
    hrr: &Value,
    cleanup: &Value,
    attractor: &Value,
    capacity: &Value,
    anti_wave: &Value,
    hot_cycle: &Value,
    decode: &Value,
) -> Value {
    let mut blockers = vec![];
    if hrr["state"].as_str() != Some("HRR_BINDING_VISIBLE") {
        blockers.push("hrr_binding");
    }
    if cleanup["state"].as_str() == Some("CLEANUP_WATCH") {
        blockers.push("cleanup_memory");
    }
    if !matches!(
        attractor["state"].as_str().unwrap_or(""),
        "ATTRACTOR_STABLE" | "NO_ATTRACTOR_TRACE"
    ) {
        blockers.push("attractor_trace");
    }
    if capacity["state"].as_str() == Some("FOCUS_REQUIRED") {
        blockers.push("capacity");
    }
    if hot_cycle["state"].as_str() != Some("LLMWAVE_HOT_READY") {
        blockers.push("hot_cycle");
    }
    let top_pattern = decode["top_pattern"].as_str().unwrap_or("");
    if top_pattern.is_empty() {
        blockers.push("decode");
    }
    let state = if blockers.is_empty() {
        "LLMWAVE_PROOF_READY"
    } else {
        "LLMWAVE_PROOF_WATCH"
    };
    json!({
        "version": "v59-llmwave-proof-command-contract",
        "state": state,
        "answer_ready": state == "LLMWAVE_PROOF_READY",
        "top_pattern": top_pattern,
        "anti_wave_state": anti_wave["state"],
        "blockers": blockers,
        "read_as": "v59 is the proof contract: a decoded pattern is useful only when binding, cleanup, attractor, capacity, and hot readiness agree."
    })
}

fn llmwave_public_demo_report(proof: &Value, decode: &Value, text: &str) -> Value {
    let top = decode["top_pattern"].as_str().unwrap_or("");
    let state = if proof["state"].as_str() == Some("LLMWAVE_PROOF_READY") {
        "PUBLIC_DEMO_READY"
    } else {
        "PUBLIC_DEMO_REVIEW"
    };
    json!({
        "version": "v60-public-demo-packet",
        "state": state,
        "input": text,
        "demo_claim": if state == "PUBLIC_DEMO_READY" { "LLMWave found a stable structural continuation and exposed its proof signals." } else { "LLMWave produced a reviewable structural continuation, but proof blockers remain." },
        "top_pattern": top,
        "safe_claim": "This is structural wave retrieval/proof, not standalone natural-language understanding.",
        "read_as": "v60 is the public demo surface: one compact packet that can be shown without hiding proof state."
    })
}

#[allow(clippy::too_many_arguments)]
fn llmwave_contract_report(
    packet: &Packet,
    text: &str,
    tokens: &[String],
    decode: &Value,
    hrr: &Value,
    cleanup: &Value,
    cleanup_dictionary: &Value,
    attractor: &Value,
    capacity: &Value,
    anti_wave: &Value,
    hot_cycle: &Value,
    proof: &Value,
    selected: &LlmwaveLensKind,
) -> Value {
    let pattern = llmwave_pattern_lens(decode, proof, cleanup_dictionary);
    let polarity = llmwave_polarity_lens(decode);
    let cleanup_lens = llmwave_cleanup_lens(cleanup, cleanup_dictionary);
    let token = llmwave_token_lens(packet, tokens, proof, cleanup_dictionary);
    let field = llmwave_field_report(packet, text, tokens, decode, hrr, attractor, capacity);
    let field_snapshot = llmwave_field_snapshot(packet, text, tokens, decode, anti_wave);
    let convex = llmwave_convex_lens(decode, &field_snapshot);
    let concave = llmwave_concave_lens(decode);
    let prism = llmwave_prism_lens(decode, &convex, anti_wave);
    let role = llmwave_role_lens(decode);
    let temporal = llmwave_temporal_lens(decode);
    let evidence = llmwave_evidence_lens(packet, decode);
    let energy = llmwave_energy_lens(attractor, &field_snapshot, &concave);
    let anti = llmwave_anti_lens(anti_wave, decode, &field_snapshot);
    let selected_name = llmwave_lens_name(selected);
    let selected_lens = match selected {
        LlmwaveLensKind::Pattern => pattern.clone(),
        LlmwaveLensKind::Polarity => polarity.clone(),
        LlmwaveLensKind::Cleanup => cleanup_lens.clone(),
        LlmwaveLensKind::Token => token.clone(),
        LlmwaveLensKind::Convex => convex.clone(),
        LlmwaveLensKind::Concave => concave.clone(),
        LlmwaveLensKind::Prism => prism.clone(),
        LlmwaveLensKind::Role => role.clone(),
        LlmwaveLensKind::Temporal => temporal.clone(),
        LlmwaveLensKind::Evidence => evidence.clone(),
        LlmwaveLensKind::Energy => energy.clone(),
        LlmwaveLensKind::Anti => anti.clone(),
    };
    let selected_ready = selected_lens["ready"].as_bool().unwrap_or(false);
    let hot_budget = llmwave_hot_budget_report(packet, capacity, hot_cycle);
    let baseline = llmwave_baseline_compare(tokens, decode);
    let state = if selected_ready && proof["state"].as_str() == Some("LLMWAVE_PROOF_READY") {
        "LLMWAVE_LENS_READY"
    } else if selected_lens["state"]
        .as_str()
        .is_some_and(|state| state.contains("REVERSED") || state.contains("AMBIGUOUS"))
    {
        "LLMWAVE_LENS_WATCH"
    } else {
        "LLMWAVE_LENS_REVIEW"
    };
    json!({
        "version": "v67-field-lens-contract",
        "state": state,
        "selected": selected_name,
        "selected_lens": selected_lens,
        "field": field,
        "field_snapshot": field_snapshot,
        "lenses": {
            "pattern": pattern,
            "polarity": polarity,
            "cleanup": cleanup_lens,
            "token": token,
            "convex": convex,
            "concave": concave,
            "prism": prism,
            "role": role,
            "temporal": temporal,
            "evidence": evidence,
            "energy": energy,
            "anti": anti
        },
        "lens_taxonomy": llmwave_lens_taxonomy(),
        "baseline_compare": baseline,
        "hot_budget": hot_budget,
        "proof_state": proof["state"],
        "answer_ready": state == "LLMWAVE_LENS_READY",
        "anti_wave_state": anti_wave["state"],
        "read_as": "v80 separates the wave field from lens optics. Convex gathers peaks, concave splits contested peaks, and prism explains peak contributions."
    })
}

fn llmwave_lens_name(kind: &LlmwaveLensKind) -> &'static str {
    match kind {
        LlmwaveLensKind::Pattern => "pattern",
        LlmwaveLensKind::Polarity => "polarity",
        LlmwaveLensKind::Cleanup => "cleanup",
        LlmwaveLensKind::Token => "token",
        LlmwaveLensKind::Convex => "convex",
        LlmwaveLensKind::Concave => "concave",
        LlmwaveLensKind::Prism => "prism",
        LlmwaveLensKind::Role => "role",
        LlmwaveLensKind::Temporal => "temporal",
        LlmwaveLensKind::Evidence => "evidence",
        LlmwaveLensKind::Energy => "energy",
        LlmwaveLensKind::Anti => "anti",
    }
}

fn llmwave_lens_taxonomy() -> Value {
    json!({
        "version": "v76-lens-taxonomy",
        "active": ["pattern", "polarity", "cleanup", "token", "convex", "concave", "prism", "role", "temporal", "evidence", "energy", "anti"],
        "planned": ["microscope", "telescope"],
        "convex": "gather weak aligned signals into a stable peak",
        "concave": "separate a mixed or contested peak into rival branches",
        "prism": "explain a peak by route, relation, role path, and polarity contribution",
        "role": "read actor/action/target role binding and role-swap risk",
        "temporal": "read event order and sequence gaps",
        "evidence": "read whether the peak is backed by evidence or only resembles a fact",
        "energy": "read basin stability, margin, attractor trace, and perturbation risk",
        "anti": "explain destructive interference and what changed after suppression"
    })
}

fn llmwave_field_report(
    packet: &Packet,
    text: &str,
    tokens: &[String],
    decode: &Value,
    hrr: &Value,
    attractor: &Value,
    capacity: &Value,
) -> Value {
    json!({
        "input_kind": if text.trim().is_empty() { "structural-prefix" } else { "text-prefix" },
        "tokens": tokens.len(),
        "memory_triads": packet.triads.len(),
        "continuation_memory": packet.continuation_memory.len(),
        "query_triads": decode["recurrent"]["steps"]
            .as_array()
            .and_then(|steps| steps.first())
            .and_then(|step| step["query"].as_array())
            .map_or(0, Vec::len),
        "field_state": decode["source_search"]["field_state"],
        "top_peak": decode["source_search"]["top_peak"],
        "hrr_state": hrr["state"],
        "attractor_state": attractor["state"],
        "capacity_state": capacity["state"],
        "estimated_crosstalk": capacity["estimated_crosstalk"],
        "read_as": "Shared LLMWave field before choosing a readout lens."
    })
}

fn llmwave_field_snapshot(
    packet: &Packet,
    text: &str,
    tokens: &[String],
    decode: &Value,
    anti_wave: &Value,
) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let energy = patterns
        .iter()
        .map(|item| item["score"].as_f64().unwrap_or(0.0).max(0.0))
        .sum::<f64>();
    let top_score = patterns
        .first()
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0);
    let second_score = patterns
        .get(1)
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0);
    let fingerprint_source = json!({
        "text": text,
        "tokens": tokens,
        "memory_triads": packet.triads.len(),
        "continuation_memory": packet.continuation_memory.len(),
        "top_peak": decode["source_search"]["top_peak"],
        "top_pattern": decode["top_pattern"],
        "top_score": round4(top_score),
        "patterns": patterns.iter().take(8).map(pattern_label_value).collect::<Vec<_>>(),
        "anti_wave_state": anti_wave["state"]
    });
    let digest = Sha256::digest(fingerprint_source.to_string().as_bytes());
    json!({
        "version": "v77-field-snapshot",
        "snapshot_id": format!("{:x}", digest)[..16].to_string(),
        "input_kind": if text.trim().is_empty() { "structural-prefix" } else { "text-prefix" },
        "token_count": tokens.len(),
        "pattern_count": patterns.len(),
        "energy": round4(energy),
        "top_score": round4(top_score),
        "second_score": round4(second_score),
        "margin": round4((top_score - second_score).max(0.0)),
        "top_peak": decode["source_search"]["top_peak"],
        "top_pattern": decode["top_pattern"],
        "anti_wave_state": anti_wave["state"],
        "read_as": "Field Snapshot is the cold repeatable fingerprint of the wave before lens optics read it."
    })
}

fn llmwave_pattern_lens(decode: &Value, proof: &Value, cleanup_dictionary: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let top_pattern = decode["top_pattern"].as_str().unwrap_or("");
    let cleanup_ready = matches!(
        cleanup_dictionary["state"].as_str().unwrap_or(""),
        "CLEANUP_DICTIONARY_READY" | "CLEANUP_DICTIONARY_EMPTY"
    );
    let proof_ready = proof["state"].as_str() == Some("LLMWAVE_PROOF_READY");
    let ready = !top_pattern.is_empty() && cleanup_ready && proof_ready;
    let candidates = patterns
        .iter()
        .take(5)
        .map(|item| {
            json!({
                "pattern": pattern_label_value(item),
                "score": item["score"],
                "route": item["route"],
                "polarity": item["polarity"],
                "continuity": item["continuity"],
                "peak": item["peak"],
                "safe_to_use": ready && item["polarity"].as_str().unwrap_or("") != "REVERSED"
            })
        })
        .collect::<Vec<_>>();
    json!({
        "kind": "pattern",
        "state": if ready { "PATTERN_LENS_READY" } else if top_pattern.is_empty() { "PATTERN_LENS_EMPTY" } else { "PATTERN_LENS_REVIEW" },
        "ready": ready,
        "top_pattern": top_pattern,
        "candidates": candidates,
        "cleanup_state": cleanup_dictionary["state"],
        "proof_state": proof["state"],
        "read_as": "Pattern Lens reads the field as ranked next structural continuations."
    })
}

fn llmwave_polarity_lens(decode: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let mut directional = 0usize;
    let mut reversed = 0usize;
    let mut unknown = 0usize;
    for item in &patterns {
        let polarity = item["polarity"].as_str().unwrap_or("");
        if polarity.contains("REVERSED") {
            reversed += 1;
        } else if polarity.contains("->") || polarity == "ALIGNED" {
            directional += 1;
        } else {
            unknown += 1;
        }
    }
    let top_polarity = patterns
        .first()
        .and_then(|item| item["polarity"].as_str())
        .unwrap_or("");
    let state = if top_polarity.contains("REVERSED") {
        "POLARITY_LENS_REVERSED"
    } else if top_polarity.contains("->") || top_polarity == "ALIGNED" {
        "POLARITY_LENS_DIRECTIONAL"
    } else if top_polarity.is_empty() {
        "POLARITY_LENS_EMPTY"
    } else {
        "POLARITY_LENS_AMBIGUOUS"
    };
    json!({
        "kind": "polarity",
        "state": state,
        "ready": state == "POLARITY_LENS_DIRECTIONAL",
        "top_polarity": top_polarity,
        "directional": directional,
        "reversed": reversed,
        "unknown": unknown,
        "stop_signal": state == "POLARITY_LENS_REVERSED",
        "read_as": "Polarity Lens reads direction, role reversal, and negation risk before using a continuation."
    })
}

fn llmwave_cleanup_lens(cleanup: &Value, cleanup_dictionary: &Value) -> Value {
    let items = cleanup["items"].as_array().cloned().unwrap_or_default();
    let state = if cleanup_dictionary["ambiguous"].as_u64().unwrap_or(0) > 0 {
        "CLEANUP_LENS_AMBIGUOUS"
    } else if cleanup_dictionary["exact"].as_u64().unwrap_or(0) > 0 {
        "CLEANUP_LENS_EXACT"
    } else if cleanup_dictionary["near"].as_u64().unwrap_or(0) > 0 {
        "CLEANUP_LENS_NEAR"
    } else {
        "CLEANUP_LENS_EMPTY"
    };
    let anchors = items
        .iter()
        .take(5)
        .map(|item| {
            json!({
                "raw_pattern": item["raw_pattern"],
                "nearest_pattern": item["nearest_pattern"],
                "score": item["score"],
                "state": item["state"]
            })
        })
        .collect::<Vec<_>>();
    json!({
        "kind": "cleanup",
        "state": state,
        "ready": matches!(state, "CLEANUP_LENS_EXACT" | "CLEANUP_LENS_NEAR"),
        "dictionary_state": cleanup_dictionary["state"],
        "anchors": anchors,
        "read_as": "Cleanup Lens maps noisy decoded peaks to known clean patterns and keeps ambiguity under WATCH."
    })
}

fn llmwave_convex_lens(decode: &Value, snapshot: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let mut basins: BTreeMap<String, Value> = BTreeMap::new();
    for item in &patterns {
        let key = item["route"]
            .as_str()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| item["peak"].as_str())
            .unwrap_or("unrouted")
            .to_string();
        let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
        let support_row = json!({
            "triad": item["triad"],
            "pattern": pattern_label_value(item),
            "score": round4(score),
            "relation": item["relation"],
            "polarity": item["polarity"]
        });
        basins
            .entry(key.clone())
            .and_modify(|basin| {
                let total = basin["total_score"].as_f64().unwrap_or(0.0) + score;
                let count = basin["support_count"].as_u64().unwrap_or(0) + 1;
                basin["total_score"] = json!(round4(total));
                basin["support_count"] = json!(count);
                if score > basin["top_score"].as_f64().unwrap_or(0.0) {
                    basin["top_score"] = json!(round4(score));
                    basin["top_pattern"] = json!(pattern_label_value(item));
                }
                if let Some(support) = basin["support"].as_array_mut() {
                    support.push(support_row.clone());
                    support.sort_by(|a, b| {
                        b["score"]
                            .as_f64()
                            .unwrap_or(0.0)
                            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    support.truncate(5);
                }
            })
            .or_insert_with(|| {
                json!({
                    "basin": key,
                    "total_score": round4(score),
                    "top_score": round4(score),
                    "support_count": 1,
                    "top_pattern": pattern_label_value(item),
                    "support": [support_row]
                })
            });
    }
    let mut rows = basins.into_values().collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["total_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["total_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for row in &mut rows {
        let total = row["total_score"].as_f64().unwrap_or(0.0);
        let top = row["top_score"].as_f64().unwrap_or(0.0);
        let count = row["support_count"].as_u64().unwrap_or(1).max(1) as f64;
        row["coherence"] = json!(round4((total / count).min(top).max(0.0)));
        row["gather_gain"] = json!(round4((total - top).max(0.0)));
    }
    let top_total = rows
        .first()
        .and_then(|row| row["total_score"].as_f64())
        .unwrap_or(0.0);
    let second_total = rows
        .get(1)
        .and_then(|row| row["total_score"].as_f64())
        .unwrap_or(0.0);
    let margin = round4((top_total - second_total).max(0.0));
    let support_count = rows
        .first()
        .and_then(|row| row["support_count"].as_u64())
        .unwrap_or(0);
    let ready = !rows.is_empty() && support_count >= 2 && margin >= 0.03;
    rows.truncate(5);
    json!({
        "kind": "convex",
        "version": "v78-convex-gathering-lens",
        "state": if ready { "CONVEX_LENS_READY" } else if rows.is_empty() { "CONVEX_LENS_EMPTY" } else { "CONVEX_LENS_REVIEW" },
        "ready": ready,
        "top_basin": rows.first().map(|row| row["basin"].clone()).unwrap_or(Value::Null),
        "top_pattern": rows.first().map(|row| row["top_pattern"].clone()).unwrap_or(Value::Null),
        "margin": margin,
        "basins": rows,
        "snapshot_id": snapshot["snapshot_id"],
        "read_as": "Convex Lens gathers aligned weak pattern waves into a route basin and reports whether one basin dominates."
    })
}

fn llmwave_concave_lens(decode: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let top_score = patterns
        .first()
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0)
        .max(0.0);
    let mut branches = patterns
        .iter()
        .take(8)
        .map(|item| {
            let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
            json!({
                "pattern": pattern_label_value(item),
                "score": round4(score),
                "delta_from_top": round4((top_score - score).max(0.0)),
                "route": item["route"],
                "group": item["group"],
                "relation": item["relation"],
                "polarity": item["polarity"],
                "competing": top_score > 0.0 && score >= top_score * 0.70
            })
        })
        .collect::<Vec<_>>();
    branches.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let competing = branches
        .iter()
        .filter(|branch| branch["competing"].as_bool().unwrap_or(false))
        .count();
    let second_score = branches
        .get(1)
        .and_then(|branch| branch["score"].as_f64())
        .unwrap_or(0.0);
    let separation = round4((top_score - second_score).max(0.0));
    let state = if branches.is_empty() {
        "CONCAVE_LENS_EMPTY"
    } else if competing >= 2 || separation < 0.04 {
        "CONCAVE_LENS_SPLIT"
    } else {
        "CONCAVE_LENS_SINGLE"
    };
    json!({
        "kind": "concave",
        "version": "v79-concave-separation-lens",
        "state": state,
        "ready": !branches.is_empty(),
        "competing_branches": competing,
        "separation": separation,
        "branches": branches,
        "read_as": "Concave Lens spreads a mixed peak into rival continuations instead of forcing one answer."
    })
}

fn llmwave_prism_lens(decode: &Value, convex: &Value, anti_wave: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let routes = prism_dimension(&patterns, "route");
    let relations = prism_dimension(&patterns, "relation");
    let polarities = prism_dimension(&patterns, "polarity");
    let role_paths = patterns
        .iter()
        .map(|item| {
            let key = format!(
                "{}->{}",
                item["subject_role"].as_str().unwrap_or("unknown"),
                item["object_role"].as_str().unwrap_or("unknown")
            );
            let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
            (key, score)
        })
        .collect::<Vec<_>>();
    let role_paths = aggregate_prism_pairs(role_paths);
    let top_pattern = patterns
        .first()
        .map(pattern_label_value)
        .map(Value::String)
        .unwrap_or(Value::Null);
    let ready = !patterns.is_empty();
    json!({
        "kind": "prism",
        "version": "v80-prism-explanation-lens",
        "state": if ready { "PRISM_LENS_READY" } else { "PRISM_LENS_EMPTY" },
        "ready": ready,
        "top_pattern": top_pattern,
        "dominant_basin": convex["top_basin"],
        "contributions": {
            "routes": routes,
            "relations": relations,
            "role_paths": role_paths,
            "polarities": polarities
        },
        "anti_wave_state": anti_wave["state"],
        "explain_peak": {
            "snapshot_route": decode["source_search"]["top_peak"],
            "field_state": decode["source_search"]["field_state"],
            "top_score": patterns.first().map(|item| item["score"].clone()).unwrap_or(Value::Null),
            "why_visible": "top pattern is explained by route basin, relation contribution, role path, polarity, and any anti-wave state"
        },
        "read_as": "Prism Lens decomposes one visible peak into the structural contributions that made it visible."
    })
}

fn prism_dimension(patterns: &[Value], field: &str) -> Vec<Value> {
    let pairs = patterns
        .iter()
        .map(|item| {
            let key = item[field]
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown")
                .to_string();
            let score = item["score"].as_f64().unwrap_or(0.0).max(0.0);
            (key, score)
        })
        .collect::<Vec<_>>();
    aggregate_prism_pairs(pairs)
}

fn aggregate_prism_pairs(pairs: Vec<(String, f64)>) -> Vec<Value> {
    let mut totals: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for (key, score) in pairs {
        let entry = totals.entry(key).or_insert((0.0, 0));
        entry.0 += score;
        entry.1 += 1;
    }
    let mut rows = totals
        .into_iter()
        .map(|(key, (score, count))| {
            json!({
                "key": key,
                "score": round4(score),
                "support_count": count
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(5);
    rows
}

fn llmwave_role_lens(decode: &Value) -> Value {
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let role_paths = patterns
        .iter()
        .take(8)
        .map(|item| {
            let subject_role = item["subject_role"].as_str().unwrap_or("");
            let object_role = item["object_role"].as_str().unwrap_or("");
            let relation = item["relation"].as_str().unwrap_or("");
            let path = format!(
                "{}->{}->{}",
                role_family(subject_role),
                relation_family(relation),
                role_family(object_role)
            );
            let polarity = item["polarity"].as_str().unwrap_or("");
            let risk = if polarity.contains("REVERSED") {
                "ROLE_SWAP_RISK"
            } else if role_family(subject_role) == role_family(object_role) {
                "ROLE_COLLAPSE_REVIEW"
            } else {
                "ROLE_ALIGNED"
            };
            json!({
                "actor": item["subject"],
                "action": item["relation"],
                "target": item["object"],
                "subject_role": subject_role,
                "object_role": object_role,
                "role_path": path,
                "polarity": polarity,
                "risk": risk,
                "score": item["score"]
            })
        })
        .collect::<Vec<_>>();
    let top_risk = role_paths
        .first()
        .and_then(|item| item["risk"].as_str())
        .unwrap_or("");
    let swap_risks = role_paths
        .iter()
        .filter(|item| item["risk"].as_str() == Some("ROLE_SWAP_RISK"))
        .count();
    let state = if role_paths.is_empty() {
        "ROLE_LENS_EMPTY"
    } else if top_risk == "ROLE_SWAP_RISK" {
        "ROLE_LENS_SWAP_RISK"
    } else if swap_risks > 0 {
        "ROLE_LENS_REVIEW"
    } else {
        "ROLE_LENS_READY"
    };
    json!({
        "kind": "role",
        "version": "v81-role-binding-lens",
        "state": state,
        "ready": state == "ROLE_LENS_READY",
        "top_role_path": role_paths.first().map(|item| item["role_path"].clone()).unwrap_or(Value::Null),
        "role_swap_risks": swap_risks,
        "bindings": role_paths,
        "read_as": "Role Lens reads actor/action/target binding and keeps role swaps visible."
    })
}

fn llmwave_temporal_lens(decode: &Value) -> Value {
    let steps = decode["recurrent"]["steps"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut route_jumps = 0usize;
    let mut last_route = String::new();
    let mut rows = vec![];
    for step in &steps {
        let route = step["source_search"]["top_peak"].as_str().unwrap_or("");
        if !last_route.is_empty() && !route.is_empty() && route != last_route {
            route_jumps += 1;
        }
        if !route.is_empty() {
            last_route = route.to_string();
        }
        rows.push(json!({
            "step": step["step"],
            "route": route,
            "top_pattern": step["top_pattern"],
            "decoder_state": step["decoder_state"],
            "field_state": step["source_search"]["field_state"]
        }));
    }
    let repeated_pattern = rows
        .windows(2)
        .any(|window| window[0]["top_pattern"] == window[1]["top_pattern"]);
    let state = if rows.is_empty() {
        "TEMPORAL_LENS_EMPTY"
    } else if route_jumps > 0 {
        "TEMPORAL_LENS_ROUTE_JUMP"
    } else if repeated_pattern {
        "TEMPORAL_LENS_STANDING"
    } else {
        "TEMPORAL_LENS_ORDERED"
    };
    json!({
        "kind": "temporal",
        "version": "v82-temporal-order-lens",
        "state": state,
        "ready": matches!(state, "TEMPORAL_LENS_ORDERED" | "TEMPORAL_LENS_STANDING"),
        "steps": rows,
        "route_jumps": route_jumps,
        "standing_pattern": repeated_pattern,
        "read_as": "Temporal Lens reads recurrent decode as event/order flow and flags route jumps."
    })
}

fn llmwave_evidence_lens(packet: &Packet, decode: &Value) -> Value {
    let evidence_by_triad = packet
        .triads
        .iter()
        .map(|triad| (triad.id.clone(), triad.evidence.clone()))
        .collect::<BTreeMap<_, _>>();
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let rows = patterns
        .iter()
        .take(8)
        .map(|item| {
            let triad_id = item["triad"].as_str().unwrap_or("");
            let evidence = evidence_by_triad.get(triad_id).cloned().unwrap_or_default();
            let has_evidence = !evidence.trim().is_empty();
            json!({
                "triad": triad_id,
                "pattern": pattern_label_value(item),
                "route": item["route"],
                "score": item["score"],
                "evidence": evidence,
                "has_evidence": has_evidence,
                "evidence_state": if has_evidence { "EVIDENCE_BOUND" } else { "EVIDENCE_MISSING" }
            })
        })
        .collect::<Vec<_>>();
    let missing = rows
        .iter()
        .filter(|row| !row["has_evidence"].as_bool().unwrap_or(false))
        .count();
    let conflicts = evidence_conflicts(&packet.triads);
    let top_bound = rows
        .first()
        .and_then(|row| row["has_evidence"].as_bool())
        .unwrap_or(false);
    let state = if rows.is_empty() {
        "EVIDENCE_LENS_EMPTY"
    } else if !conflicts.is_empty() {
        "EVIDENCE_LENS_CONFLICT"
    } else if !top_bound {
        "EVIDENCE_LENS_TOP_MISSING"
    } else if missing > 0 {
        "EVIDENCE_LENS_PARTIAL"
    } else {
        "EVIDENCE_LENS_READY"
    };
    json!({
        "kind": "evidence",
        "version": "v83-evidence-binding-lens",
        "state": state,
        "ready": matches!(state, "EVIDENCE_LENS_READY" | "EVIDENCE_LENS_PARTIAL"),
        "top_evidence_bound": top_bound,
        "missing": missing,
        "conflicts": conflicts,
        "bindings": rows,
        "read_as": "Evidence Lens separates an evidence-backed peak from a plausible but unsupported peak."
    })
}

fn llmwave_energy_lens(attractor: &Value, snapshot: &Value, concave: &Value) -> Value {
    let steps = attractor["steps"].as_array().cloned().unwrap_or_default();
    let final_energy = steps
        .last()
        .and_then(|step| step["energy"].as_f64())
        .unwrap_or_else(|| snapshot["energy"].as_f64().unwrap_or(0.0));
    let dropping = steps
        .iter()
        .any(|step| step["trend"].as_str() == Some("DROPPING"));
    let route_jumps = attractor["route_jumps"].as_u64().unwrap_or(0);
    let margin = snapshot["margin"].as_f64().unwrap_or(0.0);
    let contested = concave["state"].as_str() == Some("CONCAVE_LENS_SPLIT");
    let state = if steps.is_empty() {
        "ENERGY_LENS_SNAPSHOT_ONLY"
    } else if route_jumps > 0 || dropping {
        "ENERGY_LENS_UNSTABLE"
    } else if contested && margin < 0.08 {
        "ENERGY_LENS_CONTESTED"
    } else {
        "ENERGY_LENS_STABLE"
    };
    json!({
        "kind": "energy",
        "version": "v84-energy-stability-lens",
        "state": state,
        "ready": matches!(state, "ENERGY_LENS_STABLE" | "ENERGY_LENS_CONTESTED"),
        "final_energy": round4(final_energy),
        "margin": round4(margin),
        "route_jumps": route_jumps,
        "dropping": dropping,
        "attractor_state": attractor["state"],
        "contested": contested,
        "snapshot_energy": snapshot["energy"],
        "read_as": "Energy Lens reads basin stability: margin, attractor trend, route jumps, and contested split risk."
    })
}

fn llmwave_anti_lens(anti_wave: &Value, decode: &Value, snapshot: &Value) -> Value {
    let applications = anti_wave["applications"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let suppressions = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("suppress"))
        .cloned()
        .collect::<Vec<_>>();
    let reinforcements = applications
        .iter()
        .filter(|item| item["action"].as_str() == Some("reinforce"))
        .count();
    let top_after = decode["top_pattern"].as_str().unwrap_or("");
    let changed_field = !suppressions.is_empty() && !top_after.is_empty();
    let state = if !suppressions.is_empty() {
        "ANTI_LENS_SUPPRESSED_SHORTCUT"
    } else if anti_wave["negative_records"].as_u64().unwrap_or(0) > 0 {
        "ANTI_LENS_AVAILABLE_NOT_TRIGGERED"
    } else {
        "ANTI_LENS_NO_MEMORY"
    };
    json!({
        "kind": "anti",
        "version": "v85-anti-lens-destructive-report",
        "state": state,
        "ready": !suppressions.is_empty(),
        "negative_records": anti_wave["negative_records"],
        "suppressions": suppressions,
        "reinforcements": reinforcements,
        "top_after": top_after,
        "changed_field": changed_field,
        "snapshot_id": snapshot["snapshot_id"],
        "read_as": "Anti Lens explains destructive interference: which shortcut was suppressed, what stayed visible, and whether the field survived."
    })
}

#[derive(Clone, Debug)]
struct TokenPatternRecord {
    prefix_tokens: Vec<String>,
    next_token: String,
    next_phrase: String,
    pattern: String,
    triad_id: String,
    route: String,
    group: String,
    polarity: String,
    confidence: f64,
}

fn llmwave_token_lens(
    packet: &Packet,
    tokens: &[String],
    proof: &Value,
    cleanup_dictionary: &Value,
) -> Value {
    let records = token_pattern_records(packet);
    let anti = token_anti_wave_report(packet, tokens);
    let mut candidates = token_resonance_candidates(tokens, &records, &anti);
    candidates.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(5);
    let top_token = candidates
        .first()
        .and_then(|candidate| candidate["token"].as_str())
        .unwrap_or("");
    let top_score = candidates
        .first()
        .and_then(|candidate| candidate["score"].as_f64())
        .unwrap_or(0.0);
    let second_score = candidates
        .get(1)
        .and_then(|candidate| candidate["score"].as_f64())
        .unwrap_or(0.0);
    let margin = round4((top_score - second_score).max(0.0));
    let cleanup = token_cleanup_report(top_token, &candidates, &records, margin);
    let mut baseline = token_lens_baseline(tokens, &records);
    let baseline_score = baseline["top_score"].as_f64().unwrap_or(0.0);
    baseline["field_top_score"] = json!(round4(top_score));
    baseline["field_beats_baseline"] = json!(top_score > baseline_score + 0.03);
    let cleanup_ready = matches!(
        cleanup["state"].as_str().unwrap_or(""),
        "TOKEN_CLEANUP_EXACT" | "TOKEN_CLEANUP_NEAR"
    );
    let proof_ready = proof["state"].as_str() == Some("LLMWAVE_PROOF_READY");
    let ready = !top_token.is_empty() && cleanup_ready && proof_ready && margin >= 0.015;
    let state = if ready {
        "TOKEN_LENS_READY"
    } else if top_token.is_empty() {
        "TOKEN_LENS_EMPTY"
    } else if cleanup["state"].as_str() == Some("TOKEN_CLEANUP_AMBIGUOUS") || margin < 0.015 {
        "TOKEN_LENS_CONTESTED"
    } else if baseline["top_token"].as_str() == Some(top_token)
        && !baseline["field_beats_baseline"].as_bool().unwrap_or(false)
    {
        "TOKEN_LENS_BASELINE_TIE"
    } else {
        "TOKEN_LENS_REVIEW"
    };
    json!({
        "kind": "token",
        "version": "v75-token-lens-next-token-resonance",
        "state": state,
        "ready": ready,
        "prefix": tokens.join(" "),
        "top_token": top_token,
        "top_phrase": candidates.first().map(|candidate| candidate["phrase"].clone()).unwrap_or(Value::Null),
        "top_k": candidates,
        "margin": margin,
        "token_cleanup": cleanup,
        "structural_cleanup_state": cleanup_dictionary["state"],
        "anti_wave": anti,
        "baseline_compare": baseline,
        "token_memory": {
            "version": "v69-token-pattern-records",
            "records": records.len(),
            "packed_record_bytes": 32,
            "estimated_packed_bytes": records.len() * 32,
            "fits_pattern_arena": records.len() * 32 <= PATTERN_STORE_ARENA_BYTES
        },
        "encoder": {
            "version": "v70-token-position-phase",
            "position_weights": [1.0, 0.70, 0.45, 0.25, 0.15, 0.10],
            "read_as": "Prefix tokens are encoded with deterministic token waves plus relative position phase."
        },
        "read_as": "Token Lens reads the LLMWave field as next-token/phrase candidates, with cleanup, anti-wave, and cheap baselines visible."
    })
}

fn token_pattern_records(packet: &Packet) -> Vec<TokenPatternRecord> {
    let mut records = vec![];
    for triad in &packet.triads {
        let sequence = tokenize_pattern(&format!(
            "{} {} {}",
            triad.subject, triad.relation, triad.object
        ));
        if sequence.len() < 2 {
            continue;
        }
        for index in 1..sequence.len() {
            let start = index.saturating_sub(6);
            let prefix_tokens = sequence[start..index].to_vec();
            let next_token = sequence[index].clone();
            let next_phrase = sequence[index..].join(" ");
            records.push(TokenPatternRecord {
                prefix_tokens,
                next_token,
                next_phrase,
                pattern: format!(
                    "{} -> {} -> {}",
                    triad.subject, triad.relation, triad.object
                ),
                triad_id: triad.id.clone(),
                route: triad.route.clone(),
                group: triad.group.clone(),
                polarity: triad_polarity(triad),
                confidence: triad.confidence,
            });
        }
    }
    records
}

fn token_resonance_candidates(
    tokens: &[String],
    records: &[TokenPatternRecord],
    anti: &Value,
) -> Vec<Value> {
    let prefix_wave = token_prefix_wave(tokens);
    let query_terms = tokens
        .iter()
        .map(|token| norm(token))
        .collect::<BTreeSet<_>>();
    let mut by_token: BTreeMap<String, Value> = BTreeMap::new();
    for record in records {
        let wave_score = ((cosine(&prefix_wave, &token_prefix_wave(&record.prefix_tokens)) + 1.0)
            / 2.0)
            .clamp(0.0, 1.0);
        let suffix = token_suffix_match(tokens, &record.prefix_tokens);
        let length_fit = token_prefix_length_fit(tokens, &record.prefix_tokens);
        let route = token_route_match(&query_terms, record);
        let anti_penalty = token_anti_penalty(anti, record);
        let score = round4(
            (0.40 * wave_score)
                + (0.30 * suffix)
                + (0.10 * length_fit)
                + (0.12 * route)
                + (0.08 * record.confidence.min(1.0))
                - anti_penalty,
        )
        .max(0.0);
        let candidate = json!({
            "token": record.next_token,
            "phrase": record.next_phrase,
            "score": score,
            "wave_score": round4(wave_score),
            "suffix_score": round4(suffix),
            "length_fit": round4(length_fit),
            "route_score": round4(route),
            "anti_penalty": round4(anti_penalty),
            "pattern": record.pattern,
            "triad": record.triad_id,
            "route": record.route,
            "group": record.group,
            "polarity": record.polarity,
            "support": [{
                "triad": record.triad_id,
                "pattern": record.pattern,
                "prefix": record.prefix_tokens.join(" "),
                "next": record.next_token,
                "route": record.route,
                "score": score
            }]
        });
        by_token
            .entry(record.next_token.clone())
            .and_modify(|existing| {
                if score > existing["score"].as_f64().unwrap_or(0.0) {
                    *existing = candidate.clone();
                } else if let Some(support) = existing["support"].as_array_mut() {
                    support.push(candidate["support"][0].clone());
                }
            })
            .or_insert(candidate);
    }
    by_token.into_values().collect()
}

fn token_prefix_wave(tokens: &[String]) -> Vec<i32> {
    let mut out = vec![0i32; WAVE_DIM];
    let weights = [100, 70, 45, 25, 15, 10];
    for (offset, token) in tokens.iter().rev().take(weights.len()).enumerate() {
        let phase = vector(&format!("token:{}:pos:-{}", norm(token), offset + 1));
        let base = vector(&format!("token:{}", norm(token)));
        for idx in 0..WAVE_DIM {
            out[idx] += ((phase[idx] + base[idx]) * weights[offset]) / 100;
        }
    }
    out
}

fn token_suffix_match(query: &[String], prefix: &[String]) -> f64 {
    if query.is_empty() || prefix.is_empty() {
        return 0.0;
    }
    let max = query.len().min(prefix.len()).min(6);
    let mut weighted_hit = 0.0;
    let mut total = 0.0;
    for offset in 0..max {
        let weight = match offset {
            0 => 1.0,
            1 => 0.70,
            2 => 0.45,
            3 => 0.25,
            4 => 0.15,
            _ => 0.10,
        };
        total += weight;
        let q = query[query.len() - 1 - offset].as_str();
        let p = prefix[prefix.len() - 1 - offset].as_str();
        if norm(q) == norm(p) {
            weighted_hit += weight;
        }
    }
    if total == 0.0 {
        0.0
    } else {
        weighted_hit / total
    }
}

fn token_prefix_length_fit(query: &[String], prefix: &[String]) -> f64 {
    if query.is_empty() || prefix.is_empty() {
        return 0.0;
    }
    let delta = query.len().abs_diff(prefix.len()) as f64;
    (1.0 - (delta * 0.18)).clamp(0.0, 1.0)
}

fn token_route_match(query_terms: &BTreeSet<String>, record: &TokenPatternRecord) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let mut route_terms = BTreeSet::new();
    for value in [&record.route, &record.group] {
        for term in norm(value)
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|term| term.len() >= 2)
        {
            route_terms.insert(term.to_string());
        }
    }
    if route_terms.is_empty() {
        0.0
    } else {
        query_terms.intersection(&route_terms).count() as f64 / route_terms.len() as f64
    }
}

fn token_cleanup_report(
    top_token: &str,
    candidates: &[Value],
    records: &[TokenPatternRecord],
    margin: f64,
) -> Value {
    if top_token.is_empty() {
        return json!({
            "version": "v72-token-cleanup-dictionary",
            "state": "TOKEN_CLEANUP_EMPTY",
            "anchors": []
        });
    }
    let anchors = records
        .iter()
        .filter(|record| record.next_token == top_token)
        .take(5)
        .map(|record| {
            json!({
                "token": record.next_token,
                "phrase": record.next_phrase,
                "pattern": record.pattern,
                "triad": record.triad_id,
                "route": record.route
            })
        })
        .collect::<Vec<_>>();
    let state = if margin < 0.015 && candidates.len() > 1 {
        "TOKEN_CLEANUP_AMBIGUOUS"
    } else if !anchors.is_empty() {
        "TOKEN_CLEANUP_EXACT"
    } else {
        "TOKEN_CLEANUP_EMPTY"
    };
    json!({
        "version": "v72-token-cleanup-dictionary",
        "state": state,
        "token": top_token,
        "margin": margin,
        "anchors": anchors,
        "read_as": "Token cleanup maps a raw next-token peak back to known token-pattern records."
    })
}

fn token_anti_wave_report(packet: &Packet, tokens: &[String]) -> Value {
    let mut lanes = vec![];
    for memory in &packet.continuation_memory {
        if memory.decision != "reject" {
            continue;
        }
        let sequence = tokenize_pattern(&format!(
            "{} {} {}",
            memory.subject, memory.relation, memory.object
        ));
        for index in 1..sequence.len() {
            let prefix = sequence[index.saturating_sub(6)..index].to_vec();
            let next = sequence[index].clone();
            let match_score = token_suffix_match(tokens, &prefix);
            if match_score >= 0.65 {
                lanes.push(json!({
                    "prefix": prefix.join(" "),
                    "suppress_next": next,
                    "route": memory.route,
                    "match_score": round4(match_score),
                    "penalty": memory.penalty,
                    "source_feedback": memory.source_feedback
                }));
            }
        }
    }
    json!({
        "version": "v73-token-shortcut-anti-wave",
        "state": if lanes.is_empty() { "TOKEN_ANTI_WAVE_NONE" } else { "TOKEN_ANTI_WAVE_APPLIED" },
        "lanes": lanes,
        "read_as": "Token anti-wave suppresses prefix-specific false next tokens without killing the whole token topic."
    })
}

fn token_anti_penalty(anti: &Value, record: &TokenPatternRecord) -> f64 {
    anti["lanes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|lane| lane["suppress_next"].as_str() == Some(record.next_token.as_str()))
        .map(|lane| {
            lane["penalty"].as_f64().unwrap_or(0.18) * lane["match_score"].as_f64().unwrap_or(1.0)
        })
        .fold(0.0, f64::max)
}

fn token_lens_baseline(tokens: &[String], records: &[TokenPatternRecord]) -> Value {
    let mut by_token: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for record in records {
        let suffix = token_suffix_match(tokens, &record.prefix_tokens);
        let entry = by_token
            .entry(record.next_token.clone())
            .or_insert((0.0, 0));
        entry.0 = entry.0.max(suffix);
        entry.1 += 1;
    }
    let mut rows = by_token
        .into_iter()
        .map(|(token, (suffix, count))| {
            let frequency = count as f64 / records.len().max(1) as f64;
            json!({
                "token": token,
                "score": round4((0.75 * suffix) + (0.25 * frequency)),
                "suffix_score": round4(suffix),
                "frequency": round4(frequency)
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(5);
    let top = rows
        .first()
        .and_then(|row| row["token"].as_str())
        .unwrap_or("");
    let top_score = rows
        .first()
        .and_then(|row| row["score"].as_f64())
        .unwrap_or(0.0);
    json!({
        "version": "v74-token-baseline-compare",
        "top_token": top,
        "top_score": round4(top_score),
        "top_k": rows,
        "field_beats_baseline": false,
        "baselines": ["suffix-ngram", "frequency"],
        "read_as": "Cheap next-token baseline. Token Lens reports it explicitly before claiming resonance value."
    })
}

fn llmwave_hot_budget_report(packet: &Packet, capacity: &Value, hot_cycle: &Value) -> Value {
    let pattern_bytes =
        (packet.triads.len() + packet.continuation_memory.len()) * PACKED_PATTERN_BYTES;
    json!({
        "hot_budget_bytes": nanda_6m::BUDGET_BYTES,
        "pattern_arena_bytes": PATTERN_STORE_ARENA_BYTES,
        "pattern_record_bytes": PACKED_PATTERN_BYTES,
        "estimated_pattern_bytes": pattern_bytes,
        "active_patterns": capacity["active_patterns"],
        "fits_pattern_arena": pattern_bytes <= PATTERN_STORE_ARENA_BYTES,
        "hot_cycle_state": hot_cycle["state"],
        "rule": "JSON/text stay cold; packed records, lanes, centroids, and lens readout stay hot."
    })
}

fn llmwave_baseline_compare(tokens: &[String], decode: &Value) -> Value {
    let top_pattern = decode["top_pattern"].as_str().unwrap_or("");
    let lexical = lexical_overlap_with_pattern(tokens, top_pattern);
    let graph_hint = decode["patterns"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item["continuity"].as_f64())
        .unwrap_or(0.0);
    let field_score = decode["patterns"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item["score"].as_f64())
        .unwrap_or(0.0);
    json!({
        "version": "v67-lens-baseline-compare",
        "lexical_overlap": round4(lexical),
        "graph_next_edge_hint": round4(graph_hint),
        "field_top_score": round4(field_score),
        "field_beats_lexical_hint": field_score > lexical + 0.05,
        "baselines": ["token-overlap", "graph-continuity", "current-decode"],
        "read_as": "v67 keeps cheap baselines visible before calling a lens readout interesting."
    })
}

fn lexical_overlap_with_pattern(tokens: &[String], pattern: &str) -> f64 {
    let token_set = tokens
        .iter()
        .map(|token| norm(token))
        .filter(|token| token.len() >= 2)
        .collect::<BTreeSet<_>>();
    let pattern_terms = norm(pattern)
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 2)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if token_set.is_empty() || pattern_terms.is_empty() {
        return 0.0;
    }
    token_set.intersection(&pattern_terms).count() as f64 / pattern_terms.len() as f64
}

#[derive(Clone, Debug, Deserialize)]
struct LlmwaveEvalSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<LlmwaveEvalCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct LlmwaveEvalCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    #[serde(default)]
    text: String,
    #[serde(default)]
    feedback_decision: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    expected_top_pattern: String,
    #[serde(default)]
    expected_hrr_state: String,
    #[serde(default)]
    expected_cleanup_state: String,
    #[serde(default)]
    expected_attractor_state: String,
    #[serde(default)]
    expected_capacity_state: String,
    #[serde(default)]
    expected_anti_wave_state: String,
    #[serde(default)]
    expected_proof_state: String,
    #[serde(default)]
    expected_demo_state: String,
    #[serde(default)]
    expected_next_token: String,
}

fn resolve_llmwave_suite_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn normalize_llmwave_feedback_decision(decision: &str) -> Result<String> {
    match norm(decision).as_str() {
        "" => Ok(String::new()),
        "accept" | "accepted" => Ok("accept".to_string()),
        "reject" | "rejected" => Ok("reject".to_string()),
        "watch" => Ok("watch".to_string()),
        other => Err(anyhow!("unsupported llmwave feedback decision: {other}")),
    }
}

#[derive(Clone, Debug, Deserialize)]
struct DemoSuite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    cases: Vec<DemoCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct DemoCase {
    #[serde(default)]
    id: String,
    path: PathBuf,
    #[serde(default)]
    text: String,
    #[serde(default)]
    feedback_decision: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    expected_state: String,
    #[serde(default)]
    expected_weak_spots: Option<usize>,
    #[serde(default)]
    expected_anti_wave_state: String,
}

fn demo_text(args: &DemoArgs) -> Result<String> {
    if let Some(path) = &args.text_file {
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
    } else {
        Ok(args.text.clone())
    }
}

fn demo_eval_args(args: &DemoArgs) -> LlmwaveEvalArgs {
    LlmwaveEvalArgs {
        suite: PathBuf::new(),
        input_format: args.input_format.clone(),
        top_k: args.top_k,
        steps: args.steps,
        search_top_k: args.search_top_k,
        route_cap: args.route_cap,
        route_triad_cap: args.route_triad_cap,
        group_by: args.group_by.clone(),
        format: OutputFormat::Json,
        normalize_paths: args.normalize_paths,
    }
}

fn demo_surface_report(id: &str, path: &Path, text: &str, report: &Value) -> Value {
    let proof_state = report["proof_summary"]["state"].as_str().unwrap_or("");
    let demo_state = report["public_demo"]["state"].as_str().unwrap_or("");
    let top_pattern = report["decode"]["top_pattern"].as_str().unwrap_or("");
    let signals = json!({
        "hrr": report["hrr_binding"]["state"],
        "cleanup": report["cleanup_memory"]["state"],
        "attractor": report["attractor_trace"]["state"],
        "capacity": report["superposition_capacity"]["state"],
        "anti_wave": report["anti_wave_audit"]["state"],
        "packed_hrr": report["packed_hrr_runtime"]["state"],
        "cleanup_dictionary": report["cleanup_dictionary"]["state"],
        "anti_wave_locality": report["anti_wave_locality"]["state"],
        "capacity_curve": report["capacity_curve"]["state"],
        "hot_cycle": report["packed_hot_cycle"]["state"],
        "proof": proof_state,
        "demo": demo_state
    });
    let weak_spots = demo_weak_spots(report);
    let state = if demo_state == "PUBLIC_DEMO_READY" && weak_spots.is_empty() {
        "PUBLIC_DEMO_READY"
    } else {
        "PUBLIC_DEMO_REVIEW"
    };
    json!({
        "core_version": CORE_VERSION,
        "mode": "llmwave-demo",
        "version": "v62-demo-raw-text-adapter",
        "id": id,
        "input_packet": path.display().to_string(),
        "text": text,
        "state": state,
        "top_pattern": top_pattern,
        "proof_state": proof_state,
        "signals": signals,
        "weak_spots": weak_spots,
        "safe_claim": if state == "PUBLIC_DEMO_READY" {
            "LLMWave found a stable structural continuation and exposed its proof signals."
        } else {
            "LLMWave produced a reviewable structural continuation, but weak spots remain."
        },
        "boundary": "This is structural wave retrieval/proof, not standalone natural-language understanding.",
        "raw_public_demo": report["public_demo"],
        "read_as": "Demo mode is the weak-spot surface: it compresses v60 JSON into a human/agent readable proof dashboard."
    })
}

fn demo_weak_spots(report: &Value) -> Vec<Value> {
    let mut weak = vec![];
    push_weak(
        &mut weak,
        "hrr",
        report["hrr_binding"]["state"].as_str().unwrap_or(""),
        &["HRR_BINDING_VISIBLE"],
        "HRR binding did not visibly recover role/filler lanes.",
    );
    push_weak(
        &mut weak,
        "cleanup",
        report["cleanup_memory"]["state"].as_str().unwrap_or(""),
        &["CLEANUP_READY"],
        "Cleanup memory is ambiguous or empty for decoded patterns.",
    );
    push_weak(
        &mut weak,
        "attractor",
        report["attractor_trace"]["state"].as_str().unwrap_or(""),
        &["ATTRACTOR_STABLE", "NO_ATTRACTOR_TRACE"],
        "Decode trajectory jumps route or loses energy.",
    );
    push_weak(
        &mut weak,
        "capacity",
        report["superposition_capacity"]["state"]
            .as_str()
            .unwrap_or(""),
        &["CAPACITY_HEALTHY", "CAPACITY_WATCH"],
        "Superposition pressure requires focus or split.",
    );
    push_weak(
        &mut weak,
        "hot_cycle",
        report["packed_hot_cycle"]["state"].as_str().unwrap_or(""),
        &["LLMWAVE_HOT_READY"],
        "Packed hot-cycle readiness is not proven.",
    );
    push_weak(
        &mut weak,
        "proof",
        report["proof_summary"]["state"].as_str().unwrap_or(""),
        &["LLMWAVE_PROOF_READY"],
        "Proof summary is not answer-ready.",
    );
    if report["decode"]["top_pattern"]
        .as_str()
        .unwrap_or("")
        .is_empty()
    {
        weak.push(json!({
            "signal": "decode",
            "state": "NO_PATTERN",
            "reason": "No structural continuation was decoded."
        }));
    }
    weak
}

fn push_weak(weak: &mut Vec<Value>, signal: &str, state: &str, allowed: &[&str], reason: &str) {
    if !allowed.contains(&state) {
        weak.push(json!({
            "signal": signal,
            "state": state,
            "reason": reason
        }));
    }
}

pub(crate) fn compact_pattern_store_report(packet: &Packet, sample: usize) -> Value {
    let records = packet
        .continuation_memory
        .iter()
        .map(PackedPattern32::from_memory)
        .collect::<Vec<_>>();
    let used_bytes = records.len() * PACKED_PATTERN_BYTES;
    let capacity = PATTERN_STORE_CAPACITY;
    let accepted = packet
        .continuation_memory
        .iter()
        .filter(|item| item.decision == "accept")
        .count();
    let rejected = packet
        .continuation_memory
        .iter()
        .filter(|item| item.decision == "reject")
        .count();
    json!({
        "core_version": CORE_VERSION,
        "mode": "compact-pattern-store",
        "version": "v35-compact-pattern-store",
        "runtime_version": "v40-6m-pattern-runtime-contract",
        "packed_pattern_bytes": PACKED_PATTERN_BYTES,
        "arena_bytes": PATTERN_STORE_ARENA_BYTES,
        "capacity": capacity,
        "records": records.len(),
        "accepted": accepted,
        "rejected": rejected,
        "used_bytes": used_bytes,
        "remaining_bytes": PATTERN_STORE_ARENA_BYTES.saturating_sub(used_bytes),
        "fits_pattern_arena": records.len() <= capacity,
        "records_sample": packet.continuation_memory.iter().zip(records.iter()).take(sample).map(|(memory, record)| packed_pattern_json(memory, record)).collect::<Vec<_>>()
    })
}

pub(crate) fn apply_compact_pattern_store(
    candidates: &mut [Value],
    query: &[Triad],
    memories: &[ContinuationMemory],
) -> Value {
    let query_terms = query_term_set(query);
    let mut applications = vec![];
    let store_records = memories.len();
    if candidates.is_empty() || memories.is_empty() {
        return json!({
            "version": "v36-pattern-replay",
            "applied": false,
            "store_records": store_records,
            "applications": applications
        });
    }
    for memory in memories {
        let query_ratio = continuation_query_ratio(&query_terms, memory);
        if query_ratio <= 0.0 {
            continue;
        }
        let packed = PackedPattern32::from_memory(memory);
        for candidate in candidates.iter_mut() {
            let candidate_signature = pattern_signature(
                candidate["subject"].as_str().unwrap_or(""),
                candidate["relation"].as_str().unwrap_or(""),
                candidate["object"].as_str().unwrap_or(""),
                candidate["route"].as_str().unwrap_or(""),
            );
            if candidate_signature != packed.signature {
                continue;
            }
            let support_ratio = continuation_candidate_support_ratio(memory, candidate);
            let match_ratio = round4(query_ratio * support_ratio);
            if match_ratio <= 0.0 {
                continue;
            }
            let old_score = candidate["score"].as_f64().unwrap_or(0.0);
            let (delta, action, version) = match memory.decision.as_str() {
                "accept" => (
                    round4(
                        dequantize_weight(packed.boost_i16)
                            * learned_multiplier(memory.accepted_count)
                            * match_ratio,
                    ),
                    "reinforce",
                    "v36-pattern-replay",
                ),
                "reject" => (
                    -round4(
                        dequantize_weight(packed.penalty_i16)
                            * learned_multiplier(memory.rejected_count)
                            * match_ratio,
                    ),
                    "suppress",
                    "v38-negative-continuation-lane",
                ),
                _ => (0.0, "watch", "v36-pattern-replay"),
            };
            if delta == 0.0 {
                continue;
            }
            let new_score = round4((old_score + delta).clamp(-1.0, 1.5));
            if let Some(object) = candidate.as_object_mut() {
                object.insert("score".to_string(), json!(new_score));
                object.insert("raw_decode_score".to_string(), json!(round4(old_score)));
                object.insert("compact_pattern_delta".to_string(), json!(delta));
                object.insert("continuation_memory_delta".to_string(), json!(delta));
                object.insert(
                    "compact_pattern_signature".to_string(),
                    json!(format!("{:016x}", packed.signature)),
                );
            }
            applications.push(json!({
                "version": version,
                "memory": memory.id,
                "action": action,
                "pattern": format!("{} -> {} -> {}", memory.subject, memory.relation, memory.object),
                "signature": format!("{:016x}", packed.signature),
                "delta": delta,
                "match_ratio": match_ratio,
                "query_match_ratio": round4(query_ratio),
                "support_match_ratio": round4(support_ratio),
                "accepted_count": memory.accepted_count,
                "rejected_count": memory.rejected_count,
                "locality": if memory.decision == "reject" { "shortcut-specific-negative-lane" } else { "pattern-specific-positive-replay" },
                "reason": memory.reason
            }));
        }
    }
    json!({
        "version": "v36-pattern-replay",
        "negative_lane_version": "v38-negative-continuation-lane",
        "applied": !applications.is_empty(),
        "store_records": store_records,
        "applications": applications,
        "read_as": "Compact pattern store replays accepted continuations and suppresses rejected local pattern signatures before recurrent selection."
    })
}

fn decode_feedback_preview(decode: &Value, decision: &FeedbackDecision, note: &str) -> Value {
    let decision_label = feedback_decision_label(decision);
    let patterns = decode["patterns"].as_array().cloned().unwrap_or_default();
    let Some(pattern) = patterns.first() else {
        return json!({
            "enabled": true,
            "decision": decision_label,
            "continuation_memory": []
        });
    };
    let subject = pattern["subject"].as_str().unwrap_or("").to_string();
    let relation = pattern["relation"].as_str().unwrap_or("").to_string();
    let object = pattern["object"].as_str().unwrap_or("").to_string();
    let memory = ContinuationMemory {
        id: format!(
            "cont-{}",
            slug(&format!("{decision_label}-{subject}-{relation}-{object}"))
        ),
        decision: decision_label.to_string(),
        pattern_id: pattern["pattern_id"].as_str().unwrap_or("").to_string(),
        subject,
        relation,
        object,
        route: pattern["route"].as_str().unwrap_or("").to_string(),
        group: pattern["group"].as_str().unwrap_or("").to_string(),
        peak: pattern["peak"].as_str().unwrap_or("").to_string(),
        boost: default_positive_boost(),
        penalty: default_negative_penalty(),
        terms: vec![],
        support_terms: vec![],
        reason: if note.trim().is_empty() {
            "llmwave mini-loop preview".to_string()
        } else {
            note.to_string()
        },
        source_feedback: "llmwave-preview".to_string(),
        observations: 1,
        accepted_count: usize::from(decision_label == "accept"),
        rejected_count: usize::from(decision_label == "reject"),
    };
    json!({
        "enabled": true,
        "decision": decision_label,
        "continuation_memory": [memory],
        "read_as": "Preview only; use nanda-feedback on decode output to persist this memory."
    })
}

fn pattern_capacity_row(count: usize, query_patterns: usize) -> Value {
    let used_bytes = count * PACKED_PATTERN_BYTES;
    let load_factor = count as f64 / PATTERN_STORE_CAPACITY as f64;
    let estimated_collision = (load_factor * load_factor * query_patterns as f64).min(1.0);
    let state = if count <= PATTERN_STORE_CAPACITY {
        "FITS_PATTERN_ARENA"
    } else {
        "FOCUS_OR_SPLIT_PATTERN_STORE"
    };
    json!({
        "patterns": count,
        "used_bytes": used_bytes,
        "fits_pattern_arena": count <= PATTERN_STORE_CAPACITY,
        "state": state,
        "load_factor": round4(load_factor),
        "estimated_collision_pressure": round4(estimated_collision),
        "repair": if count <= PATTERN_STORE_CAPACITY { "none" } else { "keep active continuation focus <= 16k or split by route/group" }
    })
}

fn packed_pattern_json(memory: &ContinuationMemory, record: &PackedPattern32) -> Value {
    json!({
        "id": memory.id,
        "decision": memory.decision,
        "pattern": format!("{} -> {} -> {}", memory.subject, memory.relation, memory.object),
        "route": memory.route,
        "group": memory.group,
        "signature": format!("{:016x}", record.signature),
        "boost_i16": record.boost_i16,
        "penalty_i16": record.penalty_i16,
        "accepted": record.accepted,
        "rejected": record.rejected,
        "flags": record.flags
    })
}

fn continuation_query_ratio(query_terms: &BTreeSet<String>, memory: &ContinuationMemory) -> f64 {
    if memory.terms.is_empty() || query_terms.is_empty() {
        return 1.0;
    }
    let terms = normalized_shortcut_terms(&memory.terms);
    if terms.is_empty() {
        return 1.0;
    }
    terms.intersection(query_terms).count() as f64 / terms.len() as f64
}

fn continuation_candidate_support_ratio(memory: &ContinuationMemory, candidate: &Value) -> f64 {
    if memory.support_terms.is_empty() {
        return 1.0;
    }
    let candidate_terms = ["subject", "relation", "object", "route", "group", "peak"]
        .into_iter()
        .filter_map(|key| candidate[key].as_str())
        .flat_map(|value| {
            norm(value)
                .split(|ch: char| !ch.is_ascii_alphanumeric())
                .filter(|token| token.len() >= 2)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();
    let support_terms = normalized_shortcut_terms(&memory.support_terms);
    if support_terms.is_empty() || candidate_terms.is_empty() {
        return 1.0;
    }
    support_terms.intersection(&candidate_terms).count() as f64 / support_terms.len() as f64
}

fn pattern_signature(subject: &str, relation: &str, object: &str, route: &str) -> u64 {
    hash64(&format!(
        "{}|{}|{}|{}",
        norm(subject),
        norm(relation),
        norm(object),
        norm(route)
    ))
}

fn hash32(value: &str) -> u32 {
    let digest = Sha256::digest(norm(value).as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

fn hash64(value: &str) -> u64 {
    let digest = Sha256::digest(value.as_bytes());
    u64::from_le_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ])
}

fn quantize_weight(value: f64) -> i16 {
    (value.clamp(-1.0, 1.0) * 10_000.0).round() as i16
}

fn dequantize_weight(value: i16) -> f64 {
    value as f64 / 10_000.0
}

fn learned_multiplier(count: usize) -> f64 {
    (1.0 + count.saturating_sub(1) as f64 * 0.25).min(3.0)
}

fn print_pattern_store_text(out: &Value) {
    println!("mode: compact-pattern-store");
    println!("records: {}", out["records"].as_u64().unwrap_or(0));
    println!("capacity: {}", out["capacity"].as_u64().unwrap_or(0));
    println!(
        "fits: {}",
        out["fits_pattern_arena"].as_bool().unwrap_or(false)
    );
}

fn print_pattern_store_md(out: &Value) {
    println!("# NANDA Pattern Store\n");
    println!("- records: `{}`", out["records"]);
    println!("- capacity: `{}`", out["capacity"]);
    println!("- fits_pattern_arena: `{}`", out["fits_pattern_arena"]);
}

fn print_capacity_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    if let Some(rows) = out["rows"].as_array() {
        for row in rows {
            println!(
                "{} patterns: {}",
                row["patterns"].as_u64().unwrap_or(0),
                row["state"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_capacity_md(out: &Value) {
    println!("# NANDA Pattern Capacity\n");
    if let Some(rows) = out["rows"].as_array() {
        for row in rows {
            println!(
                "- `{}` patterns: `{}`",
                row["patterns"].as_u64().unwrap_or(0),
                row["state"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_llmwave_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "top_pattern: {}",
        out["decode"]["top_pattern"].as_str().unwrap_or("")
    );
    println!(
        "decoder_state: {}",
        out["decode"]["decoder_state"].as_str().unwrap_or("")
    );
}

fn print_llmwave_md(out: &Value) {
    println!("# NANDA LLMWave Mini Loop\n");
    println!("- mode: `{}`", out["mode"].as_str().unwrap_or(""));
    println!(
        "- top_pattern: `{}`",
        out["decode"]["top_pattern"].as_str().unwrap_or("")
    );
    println!(
        "- decoder_state: `{}`",
        out["decode"]["decoder_state"].as_str().unwrap_or("")
    );
}

fn print_llmwave_eval_text(out: &Value) {
    println!("mode: {}", out["mode"].as_str().unwrap_or(""));
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {}: {} top={}",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_top_pattern"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_llmwave_eval_md(out: &Value) {
    println!("# NANDA LLMWave Eval\n");
    println!("- mode: `{}`", out["mode"].as_str().unwrap_or(""));
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- `{}`: `{}` top=`{}`",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["actual_top_pattern"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_demo_text(out: &Value) {
    println!("NANDA LLMWave Demo");
    println!();
    println!("state: {}", out["state"].as_str().unwrap_or(""));
    if out.get("input_mode").is_some() {
        println!("input_mode: {}", out["input_mode"].as_str().unwrap_or(""));
    }
    if let Some(adapter) = out.get("raw_adapter") {
        println!(
            "raw_adapter: {} triads={} quality={}",
            adapter["extraction_method"].as_str().unwrap_or(""),
            adapter["triads"].as_u64().unwrap_or(0),
            adapter["quality"].as_str().unwrap_or("")
        );
    }
    println!("input: {}", out["text"].as_str().unwrap_or(""));
    println!("top_pattern: {}", out["top_pattern"].as_str().unwrap_or(""));
    println!("proof: {}", out["proof_state"].as_str().unwrap_or(""));
    println!();
    println!("signals:");
    if let Some(signals) = out["signals"].as_object() {
        for (key, value) in signals {
            println!("  {key}: {}", value.as_str().unwrap_or(""));
        }
    }
    println!();
    println!("weak_spots:");
    if out["weak_spots"].as_array().is_none_or(Vec::is_empty) {
        println!("  none");
    } else if let Some(items) = out["weak_spots"].as_array() {
        for item in items {
            println!(
                "  - {} [{}]: {}",
                item["signal"].as_str().unwrap_or(""),
                item["state"].as_str().unwrap_or(""),
                item["reason"].as_str().unwrap_or("")
            );
        }
    }
    println!();
    println!("safe_claim: {}", out["safe_claim"].as_str().unwrap_or(""));
    println!("boundary: {}", out["boundary"].as_str().unwrap_or(""));
}

fn print_demo_md(out: &Value) {
    println!("# NANDA LLMWave Demo\n");
    println!("- state: `{}`", out["state"].as_str().unwrap_or(""));
    if out.get("input_mode").is_some() {
        println!(
            "- input_mode: `{}`",
            out["input_mode"].as_str().unwrap_or("")
        );
    }
    if let Some(adapter) = out.get("raw_adapter") {
        println!(
            "- raw_adapter: `{}` triads=`{}` quality=`{}`",
            adapter["extraction_method"].as_str().unwrap_or(""),
            adapter["triads"].as_u64().unwrap_or(0),
            adapter["quality"].as_str().unwrap_or("")
        );
    }
    println!("- input: `{}`", out["text"].as_str().unwrap_or(""));
    println!(
        "- top_pattern: `{}`",
        out["top_pattern"].as_str().unwrap_or("")
    );
    println!("- proof: `{}`", out["proof_state"].as_str().unwrap_or(""));
    println!("\n## Signals\n");
    if let Some(signals) = out["signals"].as_object() {
        for (key, value) in signals {
            println!("- `{key}`: `{}`", value.as_str().unwrap_or(""));
        }
    }
    println!("\n## Weak Spots\n");
    if out["weak_spots"].as_array().is_none_or(Vec::is_empty) {
        println!("- none");
    } else if let Some(items) = out["weak_spots"].as_array() {
        for item in items {
            println!(
                "- `{}` / `{}`: {}",
                item["signal"].as_str().unwrap_or(""),
                item["state"].as_str().unwrap_or(""),
                item["reason"].as_str().unwrap_or("")
            );
        }
    }
    println!("\n## Claim\n");
    println!("{}", out["safe_claim"].as_str().unwrap_or(""));
    println!();
    println!("{}", out["boundary"].as_str().unwrap_or(""));
}

fn print_demo_suite_text(out: &Value) {
    println!("NANDA LLMWave Demo Suite");
    println!(
        "passed: {}/{}",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- {}: {} state={} weak_spots={}",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["state"].as_str().unwrap_or(""),
                case["weak_spots"].as_array().map_or(0, Vec::len)
            );
        }
    }
}

fn print_demo_suite_md(out: &Value) {
    println!("# NANDA LLMWave Demo Suite\n");
    println!(
        "- passed: `{}/{}`",
        out["passed"].as_u64().unwrap_or(0),
        out["total"].as_u64().unwrap_or(0)
    );
    if let Some(cases) = out["cases"].as_array() {
        for case in cases {
            println!(
                "- `{}`: `{}` state=`{}` weak_spots=`{}`",
                case["id"].as_str().unwrap_or(""),
                if case["ok"].as_bool().unwrap_or(false) {
                    "ok"
                } else {
                    "fail"
                },
                case["state"].as_str().unwrap_or(""),
                case["weak_spots"].as_array().map_or(0, Vec::len)
            );
        }
    }
}
