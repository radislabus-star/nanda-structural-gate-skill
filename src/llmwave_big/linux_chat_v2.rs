//! Linux Chat V2: dialogue-time persistent wave learning.
//!
//! V1 keeps short dialogue state. V2 adds the missing write/read loop:
//! dialogue feedback compiles into fixed 32-byte wave deltas, writes them to a
//! persistent memory packet, reloads that packet on the next turn, and lets the
//! next Linux-profile field pass change because of memory rather than transcript
//! replay.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::linux_profile::{build_linux_reason_report, LinuxReasonReport, LinuxReasonRunConfig};
use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
    LINUX_RESIDUAL_MEMORY_VERSION,
};
use super::persistent_wave_memory::{
    append_delta, build_delta_record, load_memory, memory_effect, summarize_memory,
    PersistentWaveDeltaRecord, PersistentWaveDeltaSpec, PersistentWaveMemoryEffect,
    PersistentWaveMemoryQuery, PersistentWaveMemorySummary, DELTA_NEGATIVE, DELTA_NO_WRITE,
    DELTA_POSITIVE, PERSISTENT_WAVE_MEMORY_VERSION,
};

pub(crate) const LINUX_CHAT_V2_VERSION: &str = "llmwave-big-v-next-linux-chat-v2";

#[derive(Clone)]
pub(crate) struct LinuxChatV2Config {
    pub residual_pack: PathBuf,
    pub memory: PathBuf,
    pub prompt: Vec<String>,
    pub script: Option<PathBuf>,
    pub max_facts: usize,
    pub runtime_snapshot: Option<PathBuf>,
    pub reset_memory: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2Report {
    pub mode: &'static str,
    pub version: &'static str,
    pub residual_memory_version: &'static str,
    pub persistent_memory_version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub persistent_memory: PersistentWaveMemorySummary,
    pub turn_count: usize,
    pub turns: Vec<LinuxChatV2Turn>,
    pub eval: LinuxChatV2Eval,
    pub learning_contract: LinuxChatV2LearningContract,
    pub claim_boundary: LinuxChatV2ClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2Turn {
    pub turn_index: usize,
    pub operation: &'static str,
    pub user_prompt: String,
    pub resolved_prompt: Option<String>,
    pub intent: Option<String>,
    pub answer_allowed: bool,
    pub answer: String,
    pub verifier_state: &'static str,
    pub evidence_count: usize,
    pub evidence: Vec<LinuxChatV2Evidence>,
    pub rejected_shortcuts: Vec<String>,
    pub memory_effect: PersistentWaveMemoryEffect,
    pub pending_memory: LinuxChatV2PendingMemory,
    pub committed_delta: Option<PersistentWaveDeltaRecord>,
    pub memory_record_count_after: usize,
    pub answer_changed_due_to_wave_memory: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2Evidence {
    pub role: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub confidence: u8,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2PendingMemory {
    pub state: String,
    pub writable: bool,
    pub reason: String,
    pub preview: Option<PersistentWaveDeltaRecord>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2Eval {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f32,
    pub cases: Vec<LinuxChatV2EvalCase>,
    pub deltas_written: usize,
    pub memory_lift_observed: bool,
    pub answer_changed_due_to_wave_memory: bool,
    pub negative_lane_replay_observed: bool,
    pub unrelated_route_preserved: bool,
    pub memory_reloaded_after_write: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2EvalCase {
    pub id: &'static str,
    pub passed: bool,
    pub reason: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2LearningContract {
    pub input_memory: &'static str,
    pub persistent_memory: &'static str,
    pub writes_fixed_wave_deltas: bool,
    pub reloads_memory_before_each_question: bool,
    pub session_transcript_is_not_memory: bool,
    pub feedback_required_for_write: bool,
    pub explicit_learn_command_supported: bool,
    pub accept_reject_pending_supported: bool,
    pub does_not_use_gradient_training: bool,
    pub does_not_claim_general_llm: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV2ClaimBoundary {
    pub linux_chat_v2_ready: bool,
    pub dialogue_learning_ready: bool,
    pub persistent_wave_memory_ready: bool,
    pub fixed_wave_delta_records: bool,
    pub session_log_used_as_memory: bool,
    pub schema_residual_memory_used: bool,
    pub broad_chat_llm_ready: bool,
    pub general_llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub open_domain_chat_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Default, Clone)]
struct DialogueState {
    last_intent: Option<String>,
    last_anchor: Option<String>,
    pending: Option<PersistentWaveDeltaRecord>,
}

enum DialogueOp {
    Ask(String),
    Accept,
    Reject,
    Learn(Box<PersistentWaveDeltaRecord>),
}

pub(crate) fn build_linux_chat_v2_report(config: LinuxChatV2Config) -> Result<LinuxChatV2Report> {
    if config.reset_memory && config.memory.exists() {
        fs::remove_file(&config.memory)
            .with_context(|| format!("reset memory {}", config.memory.display()))?;
    }

    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let prompts = load_prompts(&config)?;
    let using_builtin_eval = config.prompt.is_empty() && config.script.is_none();
    let mut state = DialogueState::default();
    let mut turns = Vec::new();

    for (index, prompt) in prompts.iter().enumerate() {
        let turn = build_turn(
            index + 1,
            prompt,
            &packet.facts,
            &mut state,
            &config,
            config.max_facts.max(1),
        )?;
        turns.push(turn);
    }

    let memory = load_memory(&config.memory)?;
    let persistent_memory = summarize_memory(&config.memory, &memory);
    let eval = if using_builtin_eval {
        eval_builtin_script(&turns)
    } else {
        eval_observed_run(&turns)
    };
    let ready = eval.total > 0
        && eval.total == eval.passed
        && eval.deltas_written > 0
        && eval.memory_lift_observed
        && eval.negative_lane_replay_observed
        && eval.unrelated_route_preserved;
    let verdict = if ready {
        "LINUX_CHAT_V2_PERSISTENT_WAVE_LEARNING_READY_NOT_GENERAL_LLM"
    } else if turns.is_empty() {
        "LINUX_CHAT_V2_BLOCKED_NO_TURNS"
    } else {
        "LINUX_CHAT_V2_REVIEW"
    };
    let persistent_wave_memory_ready = persistent_memory.record_count > 0;
    let fixed_wave_delta_records = persistent_memory.fixed_wave_delta_records;

    Ok(LinuxChatV2Report {
        mode: "llmwave-big-linux-chat-v2",
        version: LINUX_CHAT_V2_VERSION,
        residual_memory_version: LINUX_RESIDUAL_MEMORY_VERSION,
        persistent_memory_version: PERSISTENT_WAVE_MEMORY_VERSION,
        verdict,
        residual_pack: packet.summary,
        persistent_memory,
        turn_count: turns.len(),
        turns,
        eval,
        learning_contract: LinuxChatV2LearningContract {
            input_memory: "lrf-schema-residual-binary-packet",
            persistent_memory: "fixed-wave-delta-memory-file",
            writes_fixed_wave_deltas: true,
            reloads_memory_before_each_question: true,
            session_transcript_is_not_memory: true,
            feedback_required_for_write: true,
            explicit_learn_command_supported: true,
            accept_reject_pending_supported: true,
            does_not_use_gradient_training: true,
            does_not_claim_general_llm: true,
        },
        claim_boundary: LinuxChatV2ClaimBoundary {
            linux_chat_v2_ready: ready,
            dialogue_learning_ready: ready,
            persistent_wave_memory_ready,
            fixed_wave_delta_records,
            session_log_used_as_memory: false,
            schema_residual_memory_used: true,
            broad_chat_llm_ready: false,
            general_llm_ready: false,
            nonlinear_memory_proven: false,
            open_domain_chat_ready: false,
            safe_claim: "Linux Chat V2 can learn local Linux-profile wave deltas from explicit dialogue feedback and apply them on the next pass. It is not a general LLM and not a nonlinear-memory proof.",
            blocked_claims: vec![
                "general_llm_ready",
                "open_domain_chat_ready",
                "broad_chat_llm_ready",
                "nonlinear_memory_proven",
                "gradient_training",
            ],
        },
    })
}

fn load_prompts(config: &LinuxChatV2Config) -> Result<Vec<String>> {
    let mut prompts = Vec::new();
    if let Some(script) = &config.script {
        let content = fs::read_to_string(script)
            .with_context(|| format!("read Linux chat v2 script {}", script.display()))?;
        prompts.extend(
            content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_string),
        );
    }
    prompts.extend(config.prompt.iter().cloned());
    if prompts.is_empty() {
        prompts = builtin_script();
    }
    Ok(prompts)
}

fn build_turn(
    turn_index: usize,
    prompt: &str,
    facts: &[LinuxResidualDecodedFact],
    state: &mut DialogueState,
    config: &LinuxChatV2Config,
    max_facts: usize,
) -> Result<LinuxChatV2Turn> {
    match parse_dialogue_op(prompt, state) {
        DialogueOp::Ask(text) => {
            build_ask_turn(turn_index, prompt, text, facts, state, config, max_facts)
        }
        DialogueOp::Accept => commit_pending_turn(turn_index, prompt, "accept", state, config),
        DialogueOp::Reject => reject_pending_turn(turn_index, prompt, state, config),
        DialogueOp::Learn(record) => commit_learn_turn(turn_index, prompt, *record, state, config),
    }
}

fn build_ask_turn(
    turn_index: usize,
    prompt: &str,
    resolved_prompt: String,
    facts: &[LinuxResidualDecodedFact],
    state: &mut DialogueState,
    config: &LinuxChatV2Config,
    max_facts: usize,
) -> Result<LinuxChatV2Turn> {
    let memory = load_memory(&config.memory)?;
    let reason = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: config.residual_pack.clone(),
        text: resolved_prompt.clone(),
        max_facts,
        runtime_snapshot: config.runtime_snapshot.clone(),
    })?;
    let memory_effect = memory_effect(&memory, query_from_reason(&reason));
    let evidence = merge_evidence(facts, &reason, max_facts);
    let rejected_shortcuts = rejected_shortcuts(&reason, &memory_effect);
    let answer = surface_answer(&reason, &evidence, &rejected_shortcuts, &memory_effect);
    let verifier_state = verify_answer(&reason, &evidence, &memory_effect, &rejected_shortcuts);
    let answer_allowed = matches!(
        verifier_state,
        "GROUNDED"
            | "GROUNDED_BY_WAVE_MEMORY"
            | "BOUNDARY_WITH_ANTI_WAVE"
            | "BOUNDARY_WITH_LEARNED_ANTI_WAVE"
    );
    let pending = pending_from_answer(
        prompt,
        &reason,
        &evidence,
        &memory_effect,
        &rejected_shortcuts,
    );

    if reason.query_wave.intent != "unknown" {
        state.last_intent = Some(reason.query_wave.intent.clone());
        if let Some(anchor) = reason.query_wave.anchors.first() {
            state.last_anchor = Some(anchor.clone());
        }
    }
    state.pending = pending.preview.clone();

    Ok(LinuxChatV2Turn {
        turn_index,
        operation: "ask",
        user_prompt: prompt.to_string(),
        resolved_prompt: Some(resolved_prompt),
        intent: Some(reason.query_wave.intent),
        answer_allowed,
        answer,
        verifier_state,
        evidence_count: evidence.len(),
        evidence,
        rejected_shortcuts,
        pending_memory: pending,
        committed_delta: None,
        memory_record_count_after: memory.records.len(),
        answer_changed_due_to_wave_memory: memory_effect.answer_changed_due_to_wave_memory,
        memory_effect,
    })
}

fn commit_pending_turn(
    turn_index: usize,
    prompt: &str,
    operation: &'static str,
    state: &mut DialogueState,
    config: &LinuxChatV2Config,
) -> Result<LinuxChatV2Turn> {
    let committed = state.pending.clone();
    let summary = if let Some(record) = committed.clone() {
        append_delta(&config.memory, record)?
    } else {
        summarize_memory(&config.memory, &load_memory(&config.memory)?)
    };
    state.pending = None;
    Ok(control_turn(
        turn_index,
        operation,
        prompt,
        committed,
        summary.record_count,
        if summary.record_count > 0 {
            "Memory feedback committed."
        } else {
            "No pending wave delta to commit."
        },
    ))
}

fn reject_pending_turn(
    turn_index: usize,
    prompt: &str,
    state: &mut DialogueState,
    config: &LinuxChatV2Config,
) -> Result<LinuxChatV2Turn> {
    let rejected = state.pending.clone().map(|pending| {
        build_delta_record(PersistentWaveDeltaSpec {
            delta_state: DELTA_NEGATIVE.to_string(),
            source_prompt: prompt.to_string(),
            intent: pending.intent,
            route: pending.route,
            subject: pending.subject,
            relation: "rejected by user".to_string(),
            object: pending.object,
            polarity: "negative".to_string(),
            reason: "user rejected previous pending answer".to_string(),
            strength: 16,
        })
    });
    let summary = if let Some(record) = rejected.clone() {
        append_delta(&config.memory, record)?
    } else {
        summarize_memory(&config.memory, &load_memory(&config.memory)?)
    };
    state.pending = None;
    Ok(control_turn(
        turn_index,
        "reject",
        prompt,
        rejected,
        summary.record_count,
        if summary.record_count > 0 {
            "Negative wave-memory feedback committed."
        } else {
            "No pending wave delta to reject."
        },
    ))
}

fn commit_learn_turn(
    turn_index: usize,
    prompt: &str,
    record: PersistentWaveDeltaRecord,
    state: &mut DialogueState,
    config: &LinuxChatV2Config,
) -> Result<LinuxChatV2Turn> {
    let summary = append_delta(&config.memory, record.clone())?;
    state.pending = None;
    Ok(control_turn(
        turn_index,
        "learn",
        prompt,
        Some(record),
        summary.record_count,
        "Explicit wave-memory delta committed.",
    ))
}

fn control_turn(
    turn_index: usize,
    operation: &'static str,
    prompt: &str,
    committed_delta: Option<PersistentWaveDeltaRecord>,
    memory_record_count_after: usize,
    answer: &'static str,
) -> LinuxChatV2Turn {
    LinuxChatV2Turn {
        turn_index,
        operation,
        user_prompt: prompt.to_string(),
        resolved_prompt: None,
        intent: committed_delta.as_ref().map(|record| record.intent.clone()),
        answer_allowed: false,
        answer: answer.to_string(),
        verifier_state: "CONTROL",
        evidence_count: 0,
        evidence: Vec::new(),
        rejected_shortcuts: Vec::new(),
        memory_effect: PersistentWaveMemoryEffect {
            memory_record_count: memory_record_count_after,
            matched_record_count: 0,
            positive_matches: 0,
            negative_matches: 0,
            correction_matches: 0,
            reinforcement_delta: 0,
            suppression_delta: 0,
            learned_negative_lanes_active: false,
            learned_positive_lanes_active: false,
            answer_changed_due_to_wave_memory: false,
            matched_lane_ids: Vec::new(),
            learned_answer: None,
            state: "CONTROL",
        },
        pending_memory: LinuxChatV2PendingMemory {
            state: DELTA_NO_WRITE.to_string(),
            writable: false,
            reason: "control turn already handled memory write".to_string(),
            preview: None,
        },
        committed_delta,
        memory_record_count_after,
        answer_changed_due_to_wave_memory: false,
    }
}

fn parse_dialogue_op(prompt: &str, state: &DialogueState) -> DialogueOp {
    let trimmed = prompt.trim();
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "accept" | "accepted" | "yes" | "да" => return DialogueOp::Accept,
        "reject" | "rejected" | "no" | "нет" => return DialogueOp::Reject,
        _ => {}
    }
    if let Some(record) = parse_explicit_learn(trimmed, state) {
        return DialogueOp::Learn(Box::new(record));
    }
    DialogueOp::Ask(resolve_prompt(trimmed, state))
}

fn parse_explicit_learn(prompt: &str, state: &DialogueState) -> Option<PersistentWaveDeltaRecord> {
    let lower = prompt.to_ascii_lowercase();
    let (delta_state, polarity, tail) = if let Some((_, tail)) = lower.split_once("learn accept:") {
        (DELTA_POSITIVE, "positive", tail)
    } else if let Some((_, tail)) = lower.split_once("learn reject:") {
        (DELTA_NEGATIVE, "negative", tail)
    } else {
        return None;
    };
    let original_tail = &prompt[prompt.len() - tail.len()..];
    let parts = original_tail
        .split('|')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }
    let subject = parts[0].to_string();
    let route = parts[1].to_string();
    let object = parts[2].to_string();
    let route_intent = intent_for_route(&route);
    let intent = if route_intent == "unknown" {
        state
            .last_intent
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        route_intent.to_string()
    };
    Some(build_delta_record(PersistentWaveDeltaSpec {
        delta_state: delta_state.to_string(),
        source_prompt: prompt.to_string(),
        intent,
        route: route.clone(),
        subject,
        relation: relation_for_route(&route, delta_state).to_string(),
        object,
        polarity: polarity.to_string(),
        reason: "explicit dialogue learn command".to_string(),
        strength: 16,
    }))
}

fn resolve_prompt(prompt: &str, state: &DialogueState) -> String {
    let lower = prompt.to_ascii_lowercase();
    if let Some(anchor) = correction_anchor(&lower) {
        if let Some(previous_intent) = &state.last_intent {
            return match previous_intent.as_str() {
                "command_provider" => format!("Which package provides command {anchor}?"),
                "external_exposure" => format!("Is {anchor} externally exposed?"),
                "listener_summary" => format!("What listeners are present for {anchor}?"),
                _ => prompt.to_string(),
            };
        }
    }
    if is_followup(&lower) {
        if let Some(previous_intent) = &state.last_intent {
            let anchor = state.last_anchor.as_deref().unwrap_or("this Linux route");
            return match previous_intent.as_str() {
                "listener_summary" => "Is this machine externally exposed?".to_string(),
                "command_provider" => {
                    format!("Does package installed prove command {anchor} is running?")
                }
                "external_exposure" => "Does that prove a vulnerability?".to_string(),
                _ => prompt.to_string(),
            };
        }
    }
    prompt.to_string()
}

fn merge_evidence(
    facts: &[LinuxResidualDecodedFact],
    reason: &LinuxReasonReport,
    max_facts: usize,
) -> Vec<LinuxChatV2Evidence> {
    let mut evidence = reason
        .evidence_chain
        .iter()
        .map(|step| LinuxChatV2Evidence {
            role: step.role.clone(),
            route: step.route.clone(),
            subject: step.subject.clone(),
            relation: step.relation.clone(),
            object: step.object.clone(),
            polarity: step.polarity.clone(),
            confidence: step.confidence,
        })
        .collect::<Vec<_>>();

    if reason.query_wave.intent == "command_provider" {
        for fact in command_provider_facts(facts, &reason.query_wave.anchors, max_facts) {
            if !evidence.iter().any(|existing| {
                existing.route == fact.route
                    && existing.subject == fact.subject
                    && existing.object == fact.object
            }) {
                evidence.push(fact);
            }
        }
    }

    evidence.truncate(max_facts);
    evidence
}

fn command_provider_facts(
    facts: &[LinuxResidualDecodedFact],
    anchors: &[String],
    max_facts: usize,
) -> Vec<LinuxChatV2Evidence> {
    let mut scored = facts
        .iter()
        .filter_map(|fact| {
            if fact.polarity != "positive" {
                return None;
            }
            let route_ok = fact.route.contains("command") || fact.route == "linux.package.binary";
            if !route_ok {
                return None;
            }
            let subject = fact.subject.to_ascii_lowercase();
            let object = fact.object.to_ascii_lowercase();
            let score = anchors.iter().fold(0, |best, anchor| {
                let anchor = anchor.to_ascii_lowercase();
                let candidate = if subject == anchor || object == anchor {
                    100
                } else if object.ends_with(&format!("/{anchor}")) {
                    96
                } else if subject.contains(&anchor) || object.contains(&anchor) {
                    50
                } else {
                    0
                };
                best.max(candidate)
            });
            (score > 0).then(|| {
                (
                    score,
                    LinuxChatV2Evidence {
                        role: "command-provider".to_string(),
                        route: fact.route.clone(),
                        subject: fact.subject.clone(),
                        relation: fact.relation.clone(),
                        object: fact.object.clone(),
                        polarity: fact.polarity.to_string(),
                        confidence: fact.confidence,
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    scored.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    scored
        .into_iter()
        .map(|(_, fact)| fact)
        .take(max_facts)
        .collect()
}

fn rejected_shortcuts(
    reason: &LinuxReasonReport,
    memory_effect: &PersistentWaveMemoryEffect,
) -> Vec<String> {
    let mut rejected = reason
        .anti_wave_hits
        .iter()
        .map(|hit| hit.shortcut.clone())
        .collect::<Vec<_>>();
    if memory_effect.learned_negative_lanes_active {
        rejected.extend(memory_effect.matched_lane_ids.iter().cloned());
    }
    if reason.query_wave.intent == "unknown" {
        rejected.push("unsupported_open_domain_prompt".to_string());
    }
    rejected.sort();
    rejected.dedup();
    rejected
}

fn surface_answer(
    reason: &LinuxReasonReport,
    evidence: &[LinuxChatV2Evidence],
    rejected_shortcuts: &[String],
    memory_effect: &PersistentWaveMemoryEffect,
) -> String {
    match reason.query_wave.intent.as_str() {
        "command_provider" => {
            if let Some(learned) = &memory_effect.learned_answer {
                format!(
                    "Learned wave-memory answer: {} {} {} on route {}.",
                    learned.subject, learned.relation, learned.object, learned.route
                )
            } else if let Some(fact) = evidence.iter().find(|fact| fact.polarity == "positive") {
                format!(
                    "Grounded answer: {} {} {} on route {}.",
                    fact.subject, fact.relation, fact.object, fact.route
                )
            } else {
                "I cannot answer: no package/provider fact matched the command.".to_string()
            }
        }
        "listener_summary" => {
            if evidence.is_empty() {
                "I do not have listener evidence in the active Linux profile.".to_string()
            } else {
                format!(
                    "Grounded listener evidence: {} fact(s). This is listener evidence only, not external exposure proof.",
                    evidence.len()
                )
            }
        }
        "external_exposure" => {
            if memory_effect.learned_negative_lanes_active {
                "External exposure is not confirmed. Learned anti-wave memory rejects the shortcut for this route.".to_string()
            } else if reason.decision.state == "EXPOSURE_CONFIRMED_REVIEW" {
                reason.decision.answer.clone()
            } else {
                let suffix = if rejected_shortcuts.is_empty() {
                    "The field requires listener evidence plus matching firewall allow evidence."
                } else {
                    "Rejected shortcut: listener evidence alone does not prove external exposure."
                };
                format!("External exposure is not confirmed. {suffix}")
            }
        }
        "package_runtime_boundary" | "vulnerability_boundary" => {
            if let Some(fact) = evidence.iter().find(|fact| fact.polarity == "negative") {
                format!("No. {} {} {}.", fact.subject, fact.relation, fact.object)
            } else {
                reason.decision.answer.clone()
            }
        }
        _ => "I cannot answer that from the bounded Linux-profile memory.".to_string(),
    }
}

fn verify_answer(
    reason: &LinuxReasonReport,
    evidence: &[LinuxChatV2Evidence],
    memory_effect: &PersistentWaveMemoryEffect,
    rejected_shortcuts: &[String],
) -> &'static str {
    if reason.query_wave.intent == "unknown" {
        return "REFUSED_UNSUPPORTED";
    }
    if memory_effect.learned_positive_lanes_active {
        return "GROUNDED_BY_WAVE_MEMORY";
    }
    if memory_effect.learned_negative_lanes_active {
        return "BOUNDARY_WITH_LEARNED_ANTI_WAVE";
    }
    if !evidence.is_empty()
        || matches!(
            reason.query_wave.intent.as_str(),
            "external_exposure" | "package_runtime_boundary" | "vulnerability_boundary"
        )
    {
        if rejected_shortcuts.is_empty() {
            "GROUNDED"
        } else {
            "BOUNDARY_WITH_ANTI_WAVE"
        }
    } else {
        "REVIEW_NO_WRITE"
    }
}

fn pending_from_answer(
    prompt: &str,
    reason: &LinuxReasonReport,
    evidence: &[LinuxChatV2Evidence],
    memory_effect: &PersistentWaveMemoryEffect,
    rejected_shortcuts: &[String],
) -> LinuxChatV2PendingMemory {
    if let Some(learned) = &memory_effect.learned_answer {
        let preview = build_delta_record(PersistentWaveDeltaSpec {
            delta_state: DELTA_POSITIVE.to_string(),
            source_prompt: prompt.to_string(),
            intent: reason.query_wave.intent.clone(),
            route: learned.route.clone(),
            subject: learned.subject.clone(),
            relation: learned.relation.clone(),
            object: learned.object.clone(),
            polarity: "positive".to_string(),
            reason: "pending reinforcement for learned wave-memory answer".to_string(),
            strength: 8,
        });
        return pending(
            DELTA_POSITIVE,
            true,
            "learned answer can be reinforced",
            Some(preview),
        );
    }

    if let Some(fact) = evidence.iter().find(|fact| fact.polarity == "positive") {
        let preview = build_delta_record(PersistentWaveDeltaSpec {
            delta_state: DELTA_POSITIVE.to_string(),
            source_prompt: prompt.to_string(),
            intent: reason.query_wave.intent.clone(),
            route: fact.route.clone(),
            subject: fact.subject.clone(),
            relation: fact.relation.clone(),
            object: fact.object.clone(),
            polarity: "positive".to_string(),
            reason: "pending user acceptance of grounded evidence".to_string(),
            strength: 8,
        });
        return pending(
            DELTA_POSITIVE,
            true,
            "grounded evidence can be accepted",
            Some(preview),
        );
    }

    if let Some(shortcut) = rejected_shortcuts.first() {
        let preview = build_delta_record(PersistentWaveDeltaSpec {
            delta_state: DELTA_NEGATIVE.to_string(),
            source_prompt: prompt.to_string(),
            intent: reason.query_wave.intent.clone(),
            route: reason
                .query_wave
                .negative_boundaries
                .first()
                .cloned()
                .or_else(|| reason.query_wave.required_routes.first().cloned())
                .unwrap_or_else(|| "linux.boundary.shortcut".to_string()),
            subject: "shortcut".to_string(),
            relation: "does not prove".to_string(),
            object: shortcut.clone(),
            polarity: "negative".to_string(),
            reason: "pending anti-wave feedback for rejected shortcut".to_string(),
            strength: 8,
        });
        return pending(
            DELTA_NEGATIVE,
            true,
            "rejected shortcut can be reinforced",
            Some(preview),
        );
    }

    pending(
        DELTA_NO_WRITE,
        false,
        "no grounded or boundary memory delta",
        None,
    )
}

fn pending(
    state: &str,
    writable: bool,
    reason: &str,
    preview: Option<PersistentWaveDeltaRecord>,
) -> LinuxChatV2PendingMemory {
    LinuxChatV2PendingMemory {
        state: state.to_string(),
        writable,
        reason: reason.to_string(),
        preview,
    }
}

fn query_from_reason(reason: &LinuxReasonReport) -> PersistentWaveMemoryQuery {
    PersistentWaveMemoryQuery {
        intent: reason.query_wave.intent.clone(),
        anchors: reason.query_wave.anchors.clone(),
        route_priors: reason.query_wave.route_priors.clone(),
        required_routes: reason.query_wave.required_routes.clone(),
        forbidden_shortcuts: reason.query_wave.forbidden_shortcuts.clone(),
    }
}

fn eval_observed_run(turns: &[LinuxChatV2Turn]) -> LinuxChatV2Eval {
    let cases = turns
        .iter()
        .map(|turn| LinuxChatV2EvalCase {
            id: "observed-turn-grounded-refused-or-control",
            passed: turn.operation != "ask"
                || matches!(
                    turn.verifier_state,
                    "GROUNDED"
                        | "GROUNDED_BY_WAVE_MEMORY"
                        | "BOUNDARY_WITH_ANTI_WAVE"
                        | "BOUNDARY_WITH_LEARNED_ANTI_WAVE"
                        | "REFUSED_UNSUPPORTED"
                        | "REVIEW_NO_WRITE"
                ),
            reason: turn.verifier_state.to_string(),
        })
        .collect::<Vec<_>>();
    eval_from_cases(turns, cases)
}

fn eval_builtin_script(turns: &[LinuxChatV2Turn]) -> LinuxChatV2Eval {
    let cases = vec![
        eval_case(
            "unknown-command-blocked-before-learning",
            turns.first().is_some_and(|turn| {
                turn.operation == "ask"
                    && turn.intent.as_deref() == Some("command_provider")
                    && turn.verifier_state == "REVIEW_NO_WRITE"
                    && !turn.answer_allowed
            }),
            turns.first(),
        ),
        eval_case(
            "positive-wave-delta-written",
            turns.get(1).is_some_and(|turn| {
                turn.operation == "learn"
                    && turn
                        .committed_delta
                        .as_ref()
                        .is_some_and(|delta| delta.delta_state == DELTA_POSITIVE)
            }),
            turns.get(1),
        ),
        eval_case(
            "learned-answer-after-reload",
            turns.get(2).is_some_and(|turn| {
                turn.operation == "ask"
                    && turn.verifier_state == "GROUNDED_BY_WAVE_MEMORY"
                    && turn.answer_changed_due_to_wave_memory
                    && turn.answer.to_ascii_lowercase().contains("foopkg")
            }),
            turns.get(2),
        ),
        eval_case(
            "unrelated-residual-route-preserved",
            turns.get(3).is_some_and(|turn| {
                turn.operation == "ask"
                    && turn.verifier_state == "GROUNDED"
                    && turn.answer.to_ascii_lowercase().contains("bash")
            }),
            turns.get(3),
        ),
        eval_case(
            "negative-wave-delta-written",
            turns.get(4).is_some_and(|turn| {
                turn.operation == "learn"
                    && turn
                        .committed_delta
                        .as_ref()
                        .is_some_and(|delta| delta.delta_state == DELTA_NEGATIVE)
            }),
            turns.get(4),
        ),
        eval_case(
            "learned-anti-wave-after-reload",
            turns.get(5).is_some_and(|turn| {
                turn.operation == "ask"
                    && turn.verifier_state == "BOUNDARY_WITH_LEARNED_ANTI_WAVE"
                    && turn.memory_effect.learned_negative_lanes_active
                    && turn
                        .answer
                        .to_ascii_lowercase()
                        .contains("learned anti-wave")
            }),
            turns.get(5),
        ),
    ];
    eval_from_cases(turns, cases)
}

fn eval_case(
    id: &'static str,
    passed: bool,
    turn: Option<&LinuxChatV2Turn>,
) -> LinuxChatV2EvalCase {
    LinuxChatV2EvalCase {
        id,
        passed,
        reason: turn
            .map(|turn| turn.verifier_state.to_string())
            .unwrap_or_else(|| "missing_turn".to_string()),
    }
}

fn eval_from_cases(turns: &[LinuxChatV2Turn], cases: Vec<LinuxChatV2EvalCase>) -> LinuxChatV2Eval {
    let total = cases.len();
    let passed = cases.iter().filter(|case| case.passed).count();
    let deltas_written = turns
        .iter()
        .filter(|turn| turn.committed_delta.is_some())
        .count();
    let answer_changed_due_to_wave_memory = turns
        .iter()
        .any(|turn| turn.answer_changed_due_to_wave_memory);
    let negative_lane_replay_observed = turns
        .iter()
        .any(|turn| turn.memory_effect.learned_negative_lanes_active);
    let memory_lift_observed = turns
        .iter()
        .any(|turn| turn.verifier_state == "GROUNDED_BY_WAVE_MEMORY");
    let unrelated_route_preserved = turns.iter().any(|turn| {
        turn.operation == "ask"
            && turn.verifier_state == "GROUNDED"
            && turn.answer.to_ascii_lowercase().contains("bash")
    });
    let memory_reloaded_after_write = turns.iter().any(|turn| {
        turn.operation == "ask"
            && turn.memory_effect.memory_record_count > 0
            && (turn.memory_effect.learned_positive_lanes_active
                || turn.memory_effect.learned_negative_lanes_active)
    });

    LinuxChatV2Eval {
        total,
        passed,
        pass_rate: ratio(passed, total),
        cases,
        deltas_written,
        memory_lift_observed,
        answer_changed_due_to_wave_memory,
        negative_lane_replay_observed,
        unrelated_route_preserved,
        memory_reloaded_after_write,
    }
}

fn builtin_script() -> Vec<String> {
    vec![
        "Which package provides command foocmd?".to_string(),
        "learn accept: foocmd | linux.apt.command.provider | foopkg".to_string(),
        "Which package provides command foocmd?".to_string(),
        "Which package provides command bash?".to_string(),
        "learn reject: listener | linux.boundary.socket | listener_implies_external_exposure"
            .to_string(),
        "Does listener mean external exposure?".to_string(),
    ]
}

fn intent_for_route(route: &str) -> &'static str {
    if route.contains("command") || route.contains("package.binary") {
        "command_provider"
    } else if route.contains("boundary.socket") || route.contains("exposure") {
        "external_exposure"
    } else {
        "unknown"
    }
}

fn relation_for_route(route: &str, delta_state: &str) -> &'static str {
    if delta_state == DELTA_NEGATIVE {
        "does not prove"
    } else if route.contains("command") || route.contains("package.binary") {
        "provided by package"
    } else {
        "supports route"
    }
}

fn is_followup(lower: &str) -> bool {
    lower.contains("that")
        || lower.contains("what about")
        || lower.contains("and this")
        || lower.contains("same question")
}

fn correction_anchor(lower: &str) -> Option<String> {
    for marker in ["i meant", "я имел в виду", "not "] {
        if let Some((_, tail)) = lower.split_once(marker) {
            return last_anchor_token(tail);
        }
    }
    None
}

fn last_anchor_token(text: &str) -> Option<String> {
    text.split(|ch: char| {
        !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' || ch == '/')
    })
    .rfind(|token| !token.is_empty() && !is_stop_token(token))
    .map(|token| token.trim_matches('.').to_string())
}

fn is_stop_token(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "and"
            | "are"
            | "command"
            | "does"
            | "for"
            | "i"
            | "is"
            | "it"
            | "meant"
            | "package"
            | "prove"
            | "that"
            | "the"
            | "this"
            | "what"
            | "which"
            | "with"
            | "yes"
    )
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        ((part as f64 / total as f64) * 10_000.0).round() as f32 / 10_000.0
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::linux_atlas::{LinuxAtlasEvidence, LinuxAtlasFact};
    use super::super::linux_residual_memory::{
        build_linux_residual_pack_report, LinuxResidualPackConfig,
    };
    use super::*;

    #[test]
    fn linux_chat_v2_writes_and_replays_persistent_wave_memory() {
        let root = fixture_root("linux-chat-v2");
        let residual = root.join("linux-chat-v2.lrf");
        let memory = root.join("linux-chat-v2.lwm");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 8,
            promotion_threshold: 2,
            out: residual.clone(),
        })
        .unwrap();

        let report = build_linux_chat_v2_report(LinuxChatV2Config {
            residual_pack: residual,
            memory,
            prompt: Vec::new(),
            script: None,
            max_facts: 4,
            runtime_snapshot: None,
            reset_memory: true,
        })
        .unwrap();

        assert_eq!(
            report.verdict,
            "LINUX_CHAT_V2_PERSISTENT_WAVE_LEARNING_READY_NOT_GENERAL_LLM"
        );
        assert!(report.claim_boundary.dialogue_learning_ready);
        assert!(report.claim_boundary.persistent_wave_memory_ready);
        assert!(!report.claim_boundary.general_llm_ready);
        assert!(!report.claim_boundary.nonlinear_memory_proven);
        assert_eq!(report.persistent_memory.record_bytes, 32);
        assert!(report.eval.memory_lift_observed);
        assert!(report.eval.answer_changed_due_to_wave_memory);
        assert!(report.eval.negative_lane_replay_observed);
        assert!(report.eval.unrelated_route_preserved);
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_root(prefix: &str) -> PathBuf {
        let root = unique_tmp_dir(prefix);
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let facts = vec![
            test_fact(
                "linux.chat.bash",
                "linux.apt.command.provider",
                "bash",
                "provided by package",
                "bash",
                "positive",
            ),
            test_fact(
                "linux.chat.systemctl",
                "linux.package.binary",
                "systemd",
                "provides binary",
                "/usr/bin/systemctl",
                "positive",
            ),
            test_fact(
                "linux.chat.socket.any",
                "linux.socket.runtime",
                "tcp",
                "listens on",
                "00000000:0016",
                "positive",
            ),
            test_fact(
                "linux.chat.boundary.socket",
                "linux.boundary.socket",
                "port listening",
                "does not prove",
                "firewall allows external packets",
                "negative",
            ),
            test_fact(
                "linux.chat.boundary.package",
                "linux.boundary.package",
                "package installed",
                "does not prove",
                "binary is running",
                "negative",
            ),
        ];
        let mut file = fs::File::create(facts_path).unwrap();
        for fact in facts {
            writeln!(file, "{}", serde_json::to_string(&fact).unwrap()).unwrap();
        }
        root
    }

    fn test_fact(
        fact_id: &str,
        route: &str,
        subject: &str,
        relation: &str,
        object: &str,
        polarity: &str,
    ) -> LinuxAtlasFact {
        LinuxAtlasFact {
            fact_id: fact_id.to_string(),
            layer: "fixture".to_string(),
            domain: "linux-chat-v2".to_string(),
            route: route.to_string(),
            subject: subject.to_string(),
            relation: relation.to_string(),
            object: object.to_string(),
            polarity: polarity.to_string(),
            confidence: 90,
            evidence: LinuxAtlasEvidence {
                source_kind: "fixture".to_string(),
                path: "fixture".to_string(),
                line: 1,
                extractor: "fixture".to_string(),
            },
        }
    }

    fn unique_tmp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
