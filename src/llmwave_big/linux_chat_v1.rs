//! Linux Chat V1: bounded multi-turn surface over Linux-profile memory.
//!
//! This module is intentionally not a general LLM. It keeps short dialogue
//! state, rewrites follow-ups into grounded Linux-profile questions, delegates
//! evidence-chain decisions to `linux_profile`, and refuses unsupported routes.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::linux_profile::{build_linux_reason_report, LinuxReasonReport, LinuxReasonRunConfig};
use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
    LINUX_RESIDUAL_MEMORY_VERSION,
};

pub(crate) const LINUX_CHAT_V1_VERSION: &str = "llmwave-big-v-next-linux-chat-v1";

#[derive(Clone)]
pub(crate) struct LinuxChatV1Config {
    pub residual_pack: PathBuf,
    pub prompt: Vec<String>,
    pub script: Option<PathBuf>,
    pub max_facts: usize,
    pub runtime_snapshot: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1Report {
    pub mode: &'static str,
    pub version: &'static str,
    pub residual_memory_version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub turn_count: usize,
    pub turns: Vec<LinuxChatV1Turn>,
    pub eval: LinuxChatV1Eval,
    pub feedback_memory: LinuxChatV1FeedbackMemory,
    pub reasoning_contract: LinuxChatV1ReasoningContract,
    pub claim_boundary: LinuxChatV1ClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1Turn {
    pub turn_index: usize,
    pub user_prompt: String,
    pub resolved_prompt: String,
    pub intent: String,
    pub topic_before: Option<String>,
    pub topic_after: Option<String>,
    pub context_resolution: LinuxChatV1ContextResolution,
    pub answer_plan: LinuxChatV1AnswerPlan,
    pub verifier: LinuxChatV1Verifier,
    pub answer_allowed: bool,
    pub answer: String,
    pub evidence_count: usize,
    pub evidence: Vec<LinuxChatV1Evidence>,
    pub rejected_shortcuts: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1ContextResolution {
    pub used_previous_topic: bool,
    pub correction_applied: bool,
    pub corrected_anchor: Option<String>,
    pub resolved_from_intent: Option<String>,
    pub state: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1AnswerPlan {
    pub planner: &'static str,
    pub selected_route: String,
    pub answer_policy: String,
    pub required_routes: Vec<String>,
    pub forbidden_shortcuts: Vec<String>,
    pub surface: &'static str,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1Verifier {
    pub state: &'static str,
    pub grounded: bool,
    pub unsupported_open_domain: bool,
    pub overclaim_blocked: bool,
    pub reason_codes: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1Evidence {
    pub role: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub confidence: u8,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1Eval {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f32,
    pub cases: Vec<LinuxChatV1EvalCase>,
    pub context_retention_rate: f32,
    pub correction_pass_rate: f32,
    pub shortcut_rejection_rate: f32,
    pub unsupported_refusal_rate: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1EvalCase {
    pub id: &'static str,
    pub passed: bool,
    pub reason: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1FeedbackMemory {
    pub mode: &'static str,
    pub turns_observed: usize,
    pub context_rewrites: usize,
    pub corrections_applied: usize,
    pub negative_shortcuts_rejected: usize,
    pub unsupported_refusals: usize,
    pub next_turn_feedback_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1ReasoningContract {
    pub input_memory: &'static str,
    pub delegates_decision_to_linux_profile: bool,
    pub uses_schema_residual_packet: bool,
    pub keeps_dialogue_state: bool,
    pub supports_followup_resolution: bool,
    pub supports_correction_resolution: bool,
    pub keeps_negative_boundaries_active: bool,
    pub does_not_use_freeform_generation: bool,
    pub does_not_scan_network: bool,
    pub does_not_claim_general_llm: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatV1ClaimBoundary {
    pub linux_chat_v1_ready: bool,
    pub linux_profile_reasoning_used: bool,
    pub schema_residual_memory_used: bool,
    pub multi_turn_context_ready: bool,
    pub correction_ready: bool,
    pub shortcut_refusal_ready: bool,
    pub unsupported_refusal_ready: bool,
    pub broad_chat_llm_ready: bool,
    pub general_llm_ready: bool,
    pub open_domain_chat_ready: bool,
    pub vulnerability_scanner_ready: bool,
    pub network_scanner_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Default, Clone)]
struct DialogueState {
    last_topic: Option<String>,
    last_intent: Option<String>,
    last_anchor: Option<String>,
}

#[derive(Clone)]
struct ResolvedPrompt {
    text: String,
    context: LinuxChatV1ContextResolution,
}

pub(crate) fn build_linux_chat_v1_report(config: LinuxChatV1Config) -> Result<LinuxChatV1Report> {
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

    let eval = if using_builtin_eval {
        eval_builtin_script(&turns)
    } else {
        eval_observed_run(&turns)
    };
    let feedback_memory = feedback_memory(&turns);
    let ready = eval.total > 0
        && eval.total == eval.passed
        && eval.context_retention_rate == 1.0
        && eval.correction_pass_rate == 1.0
        && eval.shortcut_rejection_rate == 1.0
        && eval.unsupported_refusal_rate == 1.0;
    let verdict = if ready {
        "LINUX_CHAT_V1_READY_NOT_GENERAL_LLM"
    } else if turns.is_empty() {
        "LINUX_CHAT_V1_BLOCKED_NO_TURNS"
    } else {
        "LINUX_CHAT_V1_REVIEW"
    };
    let multi_turn_context_ready = eval.context_retention_rate == 1.0;
    let correction_ready = eval.correction_pass_rate == 1.0;
    let shortcut_refusal_ready = eval.shortcut_rejection_rate == 1.0;
    let unsupported_refusal_ready = eval.unsupported_refusal_rate == 1.0;

    Ok(LinuxChatV1Report {
        mode: "llmwave-big-linux-chat-v1",
        version: LINUX_CHAT_V1_VERSION,
        residual_memory_version: LINUX_RESIDUAL_MEMORY_VERSION,
        verdict,
        residual_pack: packet.summary,
        turn_count: turns.len(),
        turns,
        eval,
        feedback_memory,
        reasoning_contract: LinuxChatV1ReasoningContract {
            input_memory: "lrf-schema-residual-binary-packet",
            delegates_decision_to_linux_profile: true,
            uses_schema_residual_packet: true,
            keeps_dialogue_state: true,
            supports_followup_resolution: true,
            supports_correction_resolution: true,
            keeps_negative_boundaries_active: true,
            does_not_use_freeform_generation: true,
            does_not_scan_network: true,
            does_not_claim_general_llm: true,
        },
        claim_boundary: LinuxChatV1ClaimBoundary {
            linux_chat_v1_ready: ready,
            linux_profile_reasoning_used: true,
            schema_residual_memory_used: true,
            multi_turn_context_ready,
            correction_ready,
            shortcut_refusal_ready,
            unsupported_refusal_ready,
            broad_chat_llm_ready: false,
            general_llm_ready: false,
            open_domain_chat_ready: false,
            vulnerability_scanner_ready: false,
            network_scanner_ready: false,
            safe_claim: "Linux Chat V1 is a bounded Linux-profile chat loop over schema/residual memory. It can answer grounded Linux fact questions, resolve constrained follow-ups/corrections, and refuse unsupported shortcuts. It is not a general LLM.",
            blocked_claims: vec![
                "general_llm_ready",
                "open_domain_chat_ready",
                "broad_chat_llm_ready",
                "vulnerability_scanner_ready",
                "network_scanner_ready",
            ],
        },
    })
}

fn load_prompts(config: &LinuxChatV1Config) -> Result<Vec<String>> {
    let mut prompts = Vec::new();
    if let Some(script) = &config.script {
        let content = fs::read_to_string(script)
            .with_context(|| format!("read Linux chat script {}", script.display()))?;
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
    config: &LinuxChatV1Config,
    max_facts: usize,
) -> Result<LinuxChatV1Turn> {
    let topic_before = state.last_topic.clone();
    let resolved = resolve_prompt(prompt, state);
    let reason = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: config.residual_pack.clone(),
        text: resolved.text.clone(),
        max_facts,
        runtime_snapshot: config.runtime_snapshot.clone(),
    })?;
    let evidence = merge_evidence(facts, &reason, max_facts);
    let rejected_shortcuts = rejected_shortcuts(&reason);
    let answer_plan = answer_plan(&reason);
    let answer = surface_answer(&reason, &evidence, &rejected_shortcuts);
    let verifier = verify_answer(&reason, &answer, &evidence, &rejected_shortcuts);
    let answer_allowed = verifier.state != "REFUSED_UNSUPPORTED";

    if reason.query_wave.intent != "unknown" {
        state.last_intent = Some(reason.query_wave.intent.clone());
        state.last_topic = Some(topic_for_intent(&reason.query_wave.intent).to_string());
        if let Some(anchor) = reason.query_wave.anchors.first() {
            state.last_anchor = Some(anchor.clone());
        }
    }

    Ok(LinuxChatV1Turn {
        turn_index,
        user_prompt: prompt.to_string(),
        resolved_prompt: resolved.text,
        intent: reason.query_wave.intent,
        topic_before,
        topic_after: state.last_topic.clone(),
        context_resolution: resolved.context,
        answer_plan,
        verifier,
        answer_allowed,
        answer,
        evidence_count: evidence.len(),
        evidence,
        rejected_shortcuts,
    })
}

fn resolve_prompt(prompt: &str, state: &DialogueState) -> ResolvedPrompt {
    let lower = prompt.to_ascii_lowercase();
    if let Some(anchor) = correction_anchor(&lower) {
        if let Some(previous_intent) = &state.last_intent {
            let text = match previous_intent.as_str() {
                "command_provider" => format!("Which package provides command {anchor}?"),
                "external_exposure" => format!("Is {anchor} externally exposed?"),
                "listener_summary" => format!("What listeners are present for {anchor}?"),
                _ => prompt.to_string(),
            };
            return ResolvedPrompt {
                text,
                context: LinuxChatV1ContextResolution {
                    used_previous_topic: true,
                    correction_applied: true,
                    corrected_anchor: Some(anchor),
                    resolved_from_intent: Some(previous_intent.clone()),
                    state: "CORRECTION_APPLIED",
                },
            };
        }
    }

    if is_followup(&lower) {
        if let Some(previous_intent) = &state.last_intent {
            let anchor = state.last_anchor.as_deref().unwrap_or("this Linux route");
            let text = match previous_intent.as_str() {
                "listener_summary" => "Is this machine externally exposed?".to_string(),
                "command_provider" => {
                    format!("Does package installed prove command {anchor} is running?")
                }
                "external_exposure" => "Does that prove a vulnerability?".to_string(),
                _ => prompt.to_string(),
            };
            return ResolvedPrompt {
                text,
                context: LinuxChatV1ContextResolution {
                    used_previous_topic: true,
                    correction_applied: false,
                    corrected_anchor: None,
                    resolved_from_intent: Some(previous_intent.clone()),
                    state: "FOLLOWUP_RESOLVED",
                },
            };
        }
    }

    ResolvedPrompt {
        text: prompt.to_string(),
        context: LinuxChatV1ContextResolution {
            used_previous_topic: false,
            correction_applied: false,
            corrected_anchor: None,
            resolved_from_intent: None,
            state: "DIRECT_PROMPT",
        },
    }
}

fn merge_evidence(
    facts: &[LinuxResidualDecodedFact],
    reason: &LinuxReasonReport,
    max_facts: usize,
) -> Vec<LinuxChatV1Evidence> {
    let mut evidence = reason
        .evidence_chain
        .iter()
        .map(|step| LinuxChatV1Evidence {
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
) -> Vec<LinuxChatV1Evidence> {
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
                    LinuxChatV1Evidence {
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

fn rejected_shortcuts(reason: &LinuxReasonReport) -> Vec<String> {
    let mut rejected = reason
        .anti_wave_hits
        .iter()
        .map(|hit| hit.shortcut.clone())
        .collect::<Vec<_>>();
    if reason.query_wave.intent == "unknown" {
        rejected.push("unsupported_open_domain_prompt".to_string());
    }
    rejected.sort();
    rejected.dedup();
    rejected
}

fn answer_plan(reason: &LinuxReasonReport) -> LinuxChatV1AnswerPlan {
    LinuxChatV1AnswerPlan {
        planner: "linux-profile-reason-plus-chat-v1-surface",
        selected_route: reason
            .query_wave
            .route_priors
            .first()
            .cloned()
            .unwrap_or_else(|| "unsupported".to_string()),
        answer_policy: reason.query_wave.answer_policy.clone(),
        required_routes: reason.query_wave.required_routes.clone(),
        forbidden_shortcuts: reason.query_wave.forbidden_shortcuts.clone(),
        surface: match reason.query_wave.intent.as_str() {
            "command_provider" => "provider_fact_surface",
            "external_exposure" => "exposure_boundary_surface",
            "listener_summary" => "listener_summary_surface",
            "package_runtime_boundary" | "vulnerability_boundary" => "negative_boundary_surface",
            _ => "refusal_surface",
        },
    }
}

fn surface_answer(
    reason: &LinuxReasonReport,
    evidence: &[LinuxChatV1Evidence],
    rejected_shortcuts: &[String],
) -> String {
    match reason.query_wave.intent.as_str() {
        "command_provider" => {
            if let Some(fact) = evidence.iter().find(|fact| fact.polarity == "positive") {
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
            if reason.decision.state == "EXPOSURE_CONFIRMED_REVIEW" {
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
    answer: &str,
    evidence: &[LinuxChatV1Evidence],
    rejected_shortcuts: &[String],
) -> LinuxChatV1Verifier {
    let unsupported = reason.query_wave.intent == "unknown";
    let grounded = !evidence.is_empty()
        || matches!(
            reason.query_wave.intent.as_str(),
            "external_exposure" | "package_runtime_boundary" | "vulnerability_boundary"
        );
    let answer_lower = answer.to_ascii_lowercase();
    let overclaim_blocked = rejected_shortcuts.iter().any(|shortcut| {
        shortcut.contains("external_exposure")
            || shortcut.contains("vulnerable")
            || shortcut.contains("unsupported")
    }) && !answer_lower.contains("confirmed by vulnerability")
        && !answer_lower.contains("proves external exposure");

    let mut reason_codes = Vec::new();
    if unsupported {
        reason_codes.push("unsupported_open_domain_prompt".to_string());
    }
    if grounded {
        reason_codes.push("grounded_or_boundary_answer".to_string());
    } else {
        reason_codes.push("missing_grounded_evidence".to_string());
    }
    if overclaim_blocked {
        reason_codes.push("overclaim_blocked".to_string());
    }

    let state = if unsupported {
        "REFUSED_UNSUPPORTED"
    } else if grounded && overclaim_blocked {
        "GROUNDED_WITH_ANTI_WAVE"
    } else if grounded {
        "GROUNDED"
    } else {
        "REVIEW"
    };

    LinuxChatV1Verifier {
        state,
        grounded,
        unsupported_open_domain: unsupported,
        overclaim_blocked,
        reason_codes,
    }
}

fn feedback_memory(turns: &[LinuxChatV1Turn]) -> LinuxChatV1FeedbackMemory {
    let context_rewrites = turns
        .iter()
        .filter(|turn| turn.context_resolution.used_previous_topic)
        .count();
    let corrections_applied = turns
        .iter()
        .filter(|turn| turn.context_resolution.correction_applied)
        .count();
    let negative_shortcuts_rejected = turns
        .iter()
        .filter(|turn| !turn.rejected_shortcuts.is_empty())
        .count();
    let unsupported_refusals = turns
        .iter()
        .filter(|turn| turn.verifier.unsupported_open_domain)
        .count();
    LinuxChatV1FeedbackMemory {
        mode: "local-dialogue-feedback-summary",
        turns_observed: turns.len(),
        context_rewrites,
        corrections_applied,
        negative_shortcuts_rejected,
        unsupported_refusals,
        next_turn_feedback_ready: context_rewrites
            + corrections_applied
            + negative_shortcuts_rejected
            > 0,
    }
}

fn eval_observed_run(turns: &[LinuxChatV1Turn]) -> LinuxChatV1Eval {
    let cases = turns
        .iter()
        .map(|turn| LinuxChatV1EvalCase {
            id: "observed-turn-grounded-or-refused",
            passed: turn.verifier.grounded || turn.verifier.unsupported_open_domain,
            reason: turn.verifier.state.to_string(),
        })
        .collect::<Vec<_>>();
    eval_from_cases(turns, cases)
}

fn eval_builtin_script(turns: &[LinuxChatV1Turn]) -> LinuxChatV1Eval {
    let cases = vec![
        eval_case(
            "command-provider-grounded",
            turns.first().is_some_and(|turn| {
                turn.intent == "command_provider"
                    && turn.answer_allowed
                    && turn.answer.to_ascii_lowercase().contains("bash")
            }),
            turns.first(),
        ),
        eval_case(
            "correction-grounded",
            turns.get(1).is_some_and(|turn| {
                turn.context_resolution.correction_applied
                    && turn.intent == "command_provider"
                    && turn.answer.to_ascii_lowercase().contains("systemctl")
            }),
            turns.get(1),
        ),
        eval_case(
            "listener-summary-grounded",
            turns.get(2).is_some_and(|turn| {
                turn.intent == "listener_summary"
                    && turn.answer_allowed
                    && turn
                        .answer
                        .to_ascii_lowercase()
                        .contains("listener evidence")
            }),
            turns.get(2),
        ),
        eval_case(
            "followup-shortcut-rejected",
            turns.get(3).is_some_and(|turn| {
                turn.context_resolution.used_previous_topic
                    && turn.intent == "external_exposure"
                    && turn.answer.to_ascii_lowercase().contains("not confirmed")
                    && turn
                        .rejected_shortcuts
                        .iter()
                        .any(|shortcut| shortcut.contains("external_exposure"))
            }),
            turns.get(3),
        ),
        eval_case(
            "vulnerability-shortcut-refused",
            turns.get(4).is_some_and(|turn| {
                turn.intent == "vulnerability_boundary"
                    && turn.answer.to_ascii_lowercase().contains("does not prove")
            }),
            turns.get(4),
        ),
        eval_case(
            "unsupported-refused",
            turns.get(5).is_some_and(|turn| {
                turn.intent == "unknown" && turn.verifier.unsupported_open_domain
            }),
            turns.get(5),
        ),
    ];
    eval_from_cases(turns, cases)
}

fn eval_case(
    id: &'static str,
    passed: bool,
    turn: Option<&LinuxChatV1Turn>,
) -> LinuxChatV1EvalCase {
    LinuxChatV1EvalCase {
        id,
        passed,
        reason: turn
            .map(|turn| turn.verifier.state.to_string())
            .unwrap_or_else(|| "missing_turn".to_string()),
    }
}

fn eval_from_cases(turns: &[LinuxChatV1Turn], cases: Vec<LinuxChatV1EvalCase>) -> LinuxChatV1Eval {
    let total = cases.len();
    let passed = cases.iter().filter(|case| case.passed).count();
    let context_total = turns
        .iter()
        .filter(|turn| turn.context_resolution.used_previous_topic)
        .count();
    let context_passed = turns
        .iter()
        .filter(|turn| turn.context_resolution.used_previous_topic && turn.intent != "unknown")
        .count();
    let correction_total = turns
        .iter()
        .filter(|turn| turn.context_resolution.correction_applied)
        .count();
    let correction_passed = turns
        .iter()
        .filter(|turn| {
            turn.context_resolution.correction_applied
                && turn.verifier.grounded
                && !turn.verifier.unsupported_open_domain
        })
        .count();
    let shortcut_turns = turns
        .iter()
        .filter(|turn| {
            turn.intent == "external_exposure" || turn.intent == "vulnerability_boundary"
        })
        .count();
    let shortcut_rejected = turns
        .iter()
        .filter(|turn| {
            (turn.intent == "external_exposure" || turn.intent == "vulnerability_boundary")
                && (!turn.rejected_shortcuts.is_empty()
                    || turn.answer.to_ascii_lowercase().contains("does not prove")
                    || turn.answer.to_ascii_lowercase().contains("not confirmed"))
        })
        .count();
    let unsupported_total = turns.iter().filter(|turn| turn.intent == "unknown").count();
    let unsupported_refused = turns
        .iter()
        .filter(|turn| turn.intent == "unknown" && turn.verifier.unsupported_open_domain)
        .count();

    LinuxChatV1Eval {
        total,
        passed,
        pass_rate: ratio(passed, total),
        cases,
        context_retention_rate: ratio_or_one(context_passed, context_total),
        correction_pass_rate: ratio_or_one(correction_passed, correction_total),
        shortcut_rejection_rate: ratio_or_one(shortcut_rejected, shortcut_turns),
        unsupported_refusal_rate: ratio_or_one(unsupported_refused, unsupported_total),
    }
}

fn builtin_script() -> Vec<String> {
    vec![
        "Which package provides command bash?".to_string(),
        "I meant systemctl.".to_string(),
        "What listeners are present?".to_string(),
        "Does that mean external exposure?".to_string(),
        "Does a vulnerable package prove runtime exposure?".to_string(),
        "Explain a random non-Linux story.".to_string(),
    ]
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

fn topic_for_intent(intent: &str) -> &'static str {
    match intent {
        "command_provider" => "command provider",
        "listener_summary" => "listener evidence",
        "external_exposure" => "external exposure",
        "vulnerability_boundary" => "vulnerability boundary",
        "package_runtime_boundary" => "package runtime boundary",
        other if other.contains("service") => "service route",
        _ => "linux profile",
    }
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

fn ratio_or_one(part: usize, total: usize) -> f32 {
    if total == 0 {
        1.0
    } else {
        ratio(part, total)
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
    fn linux_chat_v1_resolves_followups_and_refuses_shortcuts() {
        let root = fixture_root("linux-chat-v1");
        let out = root.join("linux-chat-v1.lrf");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 8,
            promotion_threshold: 2,
            out: out.clone(),
        })
        .unwrap();

        let report = build_linux_chat_v1_report(LinuxChatV1Config {
            residual_pack: out,
            prompt: Vec::new(),
            script: None,
            max_facts: 4,
            runtime_snapshot: None,
        })
        .unwrap();

        assert_eq!(report.verdict, "LINUX_CHAT_V1_READY_NOT_GENERAL_LLM");
        assert!(report.claim_boundary.linux_chat_v1_ready);
        assert!(!report.claim_boundary.general_llm_ready);
        assert_eq!(report.eval.total, report.eval.passed);
        assert!(report
            .turns
            .iter()
            .any(|turn| turn.context_resolution.correction_applied
                && turn.answer.to_ascii_lowercase().contains("systemctl")));
        assert!(report.turns.iter().any(|turn| turn
            .rejected_shortcuts
            .iter()
            .any(|shortcut| shortcut.contains("external_exposure"))));
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_root(prefix: &str) -> PathBuf {
        let root = unique_tmp_dir(prefix);
        let facts_dir = root.join("facts");
        fs::create_dir_all(&facts_dir).unwrap();
        let facts_path = facts_dir.join("fixture.jsonl");
        let facts = vec![
            test_fact(
                "linux.apt.command.provider",
                "bash",
                "provided by package",
                "bash",
                "positive",
            ),
            test_fact(
                "linux.package.binary",
                "systemd",
                "provides binary",
                "/usr/bin/systemctl",
                "positive",
            ),
            test_fact(
                "linux.socket.runtime",
                "tcp",
                "listens on",
                "00000000:0016",
                "positive",
            ),
            test_fact(
                "linux.socket.runtime",
                "tcp",
                "listens on",
                "0100007F:1F90",
                "positive",
            ),
            test_fact(
                "linux.boundary.socket",
                "port listening",
                "does not prove",
                "firewall allows external packets",
                "negative",
            ),
            test_fact(
                "linux.boundary.package",
                "package installed",
                "does not prove",
                "binary is running",
                "negative",
            ),
            test_fact(
                "linux.boundary.cve",
                "vulnerable package",
                "does not prove",
                "runtime exposure",
                "negative",
            ),
        ];
        let mut file = fs::File::create(&facts_path).unwrap();
        for fact in facts {
            serde_json::to_writer(&mut file, &fact).unwrap();
            file.write_all(b"\n").unwrap();
        }
        root
    }

    fn test_fact(
        route: &str,
        subject: &str,
        relation: &str,
        object: &str,
        polarity: &str,
    ) -> LinuxAtlasFact {
        LinuxAtlasFact {
            fact_id: format!("test.{route}.{subject}.{object}"),
            layer: if polarity == "negative" {
                "negative-boundary".to_string()
            } else {
                "linux-knowledge".to_string()
            },
            domain: "linux-chat-v1-test".to_string(),
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
        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }
}
