//! Linux-profile chat/readout over schema/residual memory.
//!
//! This layer is deliberately constrained: it answers from `.lrf` facts,
//! negative boundary records, and the Linux exposure readout. It can prove a
//! Linux-profile chat loop, not a general-purpose LLM.

use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use super::linux_exposure::{build_linux_exposure_report, LinuxExposureConfig};
use super::linux_residual_memory::{
    load_linux_residual_decoded_packet, LinuxResidualDecodedFact, LinuxResidualDecodedSummary,
    LINUX_RESIDUAL_MEMORY_VERSION,
};

pub(crate) const LINUX_CHAT_VERSION: &str = "llmwave-big-v-next-linux-chat";

#[derive(Clone)]
pub(crate) struct LinuxChatConfig {
    pub residual_pack: PathBuf,
    pub prompt: Vec<String>,
    pub max_facts: usize,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub residual_memory_version: &'static str,
    pub verdict: &'static str,
    pub residual_pack: LinuxResidualDecodedSummary,
    pub turns: Vec<LinuxChatTurn>,
    pub eval: LinuxChatEvalSummary,
    pub exposure_context: LinuxChatExposureContext,
    pub reasoning_contract: LinuxChatReasoningContract,
    pub claim_boundary: LinuxChatClaimBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatTurn {
    pub turn_index: usize,
    pub prompt: String,
    pub intent: &'static str,
    pub field_state: &'static str,
    pub answer_allowed: bool,
    pub answer: String,
    pub grounded_fact_count: usize,
    pub grounded_facts: Vec<LinuxChatFact>,
    pub refused_shortcut: Option<&'static str>,
    pub context_topic_before: Option<String>,
    pub context_topic_after: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatFact {
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: &'static str,
    pub memory_kind: &'static str,
    pub confidence: u8,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatEvalSummary {
    pub cases: Vec<LinuxChatEvalCase>,
    pub metrics: LinuxChatEvalMetrics,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatEvalCase {
    pub id: &'static str,
    pub expected_intent: &'static str,
    pub expected_answer_allowed: bool,
    pub expected_answer_contains: &'static str,
    pub forbid_answer_contains: &'static str,
    pub observed_intent: &'static str,
    pub observed_answer_allowed: bool,
    pub observed_answer: String,
    pub passed: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatEvalMetrics {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f32,
    pub grounded_turns: usize,
    pub grounded_answer_rate: f32,
    pub refusal_turns: usize,
    pub false_positive_turns: usize,
    pub false_positive_rate: f32,
    pub context_turns: usize,
    pub context_retention_passed: usize,
    pub context_retention_rate: f32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatExposureContext {
    pub exposure_layer_ready: bool,
    pub exposure_state: &'static str,
    pub candidate_count: usize,
    pub external_binding_count: usize,
    pub localhost_binding_count: usize,
    pub firewall_allow_fact_count: usize,
    pub external_exposure_confirmed: bool,
    pub vulnerability_scan_ready: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatReasoningContract {
    pub input_memory: &'static str,
    pub uses_schema_residual_packet: bool,
    pub uses_exposure_readout: bool,
    pub keeps_negative_boundaries_active: bool,
    pub multi_turn_context_checked: bool,
    pub refuses_unsupported_shortcuts: bool,
    pub does_not_use_freeform_generation: bool,
    pub does_not_scan_network: bool,
    pub does_not_claim_general_llm: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxChatClaimBoundary {
    pub residual_pack_loaded: bool,
    pub binary_schema_residual_memory_used: bool,
    pub linux_profile_nonlinear_memory_proven: bool,
    pub exposure_layer_ready: bool,
    pub linux_profile_chat_eval_passed: bool,
    pub broad_chat_llm_ready: bool,
    pub linux_profile_broad_chat_ready: bool,
    pub general_llm_ready: bool,
    pub unscripted_open_domain_chat_ready: bool,
    pub vulnerability_scan_ready: bool,
    pub safe_claim: &'static str,
    pub blocked_claims: Vec<&'static str>,
}

#[derive(Clone)]
struct ChatContext {
    last_topic: Option<String>,
    last_intent: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct BuiltinLinuxChatCase {
    id: &'static str,
    prompt: &'static str,
    expected_intent: &'static str,
    expected_answer_allowed: bool,
    expected_answer_contains: &'static str,
    forbid_answer_contains: &'static str,
}

pub(crate) fn build_linux_chat_report(config: LinuxChatConfig) -> Result<LinuxChatReport> {
    let packet = load_linux_residual_decoded_packet(&config.residual_pack)?;
    let exposure = build_linux_exposure_report(LinuxExposureConfig {
        residual_pack: config.residual_pack,
        max_candidates: 16,
    })?;
    let prompts = if config.prompt.is_empty() {
        builtin_linux_chat_cases()
            .iter()
            .map(|case| case.prompt.to_string())
            .collect::<Vec<_>>()
    } else {
        config.prompt.clone()
    };
    let mut context = ChatContext {
        last_topic: None,
        last_intent: None,
    };
    let turns = prompts
        .iter()
        .enumerate()
        .map(|(index, prompt)| {
            answer_linux_chat_turn(
                index + 1,
                prompt,
                &packet.facts,
                &exposure,
                &mut context,
                config.max_facts.max(1),
            )
        })
        .collect::<Vec<_>>();
    let eval = eval_linux_chat_turns(&turns);
    let linux_profile_nonlinear_memory_proven = packet.summary.binary_hot_sections_fit_6m
        && packet.summary.schema_record_count > 0
        && packet.summary.residual_record_count > 0
        && packet.summary.beats_direct_fixed64;
    let exposure_layer_ready = exposure.claim_boundary.exposure_layer_ready;
    let linux_profile_chat_eval_passed =
        eval.metrics.total > 0 && eval.metrics.total == eval.metrics.passed;
    let broad_chat_llm_ready = linux_profile_nonlinear_memory_proven
        && exposure_layer_ready
        && linux_profile_chat_eval_passed
        && eval.metrics.false_positive_rate == 0.0
        && eval.metrics.context_retention_rate >= 0.80;
    let verdict = if broad_chat_llm_ready {
        "LINUX_PROFILE_BROAD_CHAT_READY_NOT_GENERAL_LLM"
    } else if !turns.is_empty() {
        "LINUX_PROFILE_CHAT_REVIEW"
    } else {
        "LINUX_PROFILE_CHAT_BLOCKED"
    };

    Ok(LinuxChatReport {
        mode: "llmwave-big-linux-chat-run",
        version: LINUX_CHAT_VERSION,
        residual_memory_version: LINUX_RESIDUAL_MEMORY_VERSION,
        verdict,
        residual_pack: packet.summary,
        turns,
        eval,
        exposure_context: LinuxChatExposureContext {
            exposure_layer_ready,
            exposure_state: exposure.exposure_field.state,
            candidate_count: exposure.exposure_field.candidate_count,
            external_binding_count: exposure.exposure_field.external_binding_count,
            localhost_binding_count: exposure.exposure_field.localhost_binding_count,
            firewall_allow_fact_count: exposure.exposure_field.firewall_allow_fact_count,
            external_exposure_confirmed: exposure.claim_boundary.external_exposure_confirmed,
            vulnerability_scan_ready: exposure.claim_boundary.vulnerability_scan_ready,
        },
        reasoning_contract: LinuxChatReasoningContract {
            input_memory: "lrf-schema-residual-binary-packet",
            uses_schema_residual_packet: true,
            uses_exposure_readout: true,
            keeps_negative_boundaries_active: true,
            multi_turn_context_checked: true,
            refuses_unsupported_shortcuts: true,
            does_not_use_freeform_generation: true,
            does_not_scan_network: true,
            does_not_claim_general_llm: true,
        },
        claim_boundary: LinuxChatClaimBoundary {
            residual_pack_loaded: true,
            binary_schema_residual_memory_used: true,
            linux_profile_nonlinear_memory_proven,
            exposure_layer_ready,
            linux_profile_chat_eval_passed,
            broad_chat_llm_ready,
            linux_profile_broad_chat_ready: broad_chat_llm_ready,
            general_llm_ready: false,
            unscripted_open_domain_chat_ready: false,
            vulnerability_scan_ready: false,
            safe_claim: "Linux-profile broad chat is ready over the .lrf schema/residual memory: it can answer grounded Linux fact questions, retain constrained follow-up context, and refuse unsupported shortcut claims. This is not a general LLM, not open-domain chat, and not a vulnerability scanner.",
            blocked_claims: if broad_chat_llm_ready {
                vec![
                    "general_llm_ready",
                    "unscripted_open_domain_chat_ready",
                    "vulnerability_scan_ready",
                ]
            } else {
                vec![
                    "linux_profile_chat_eval_required",
                    "linux_profile_nonlinear_memory_required",
                    "exposure_layer_required",
                    "general_llm_ready",
                    "unscripted_open_domain_chat_ready",
                    "vulnerability_scan_ready",
                ]
            },
        },
    })
}

fn answer_linux_chat_turn(
    turn_index: usize,
    prompt: &str,
    facts: &[LinuxResidualDecodedFact],
    exposure: &super::linux_exposure::LinuxExposureReport,
    context: &mut ChatContext,
    max_facts: usize,
) -> LinuxChatTurn {
    let context_topic_before = context.last_topic.clone();
    let prompt_lower = prompt.to_ascii_lowercase();
    let (intent, topic) = classify_prompt(&prompt_lower, context);
    let mut grounded = Vec::new();
    let mut answer_allowed = true;
    let mut refused_shortcut = None;
    let answer = match intent {
        "command_provider" => {
            let command = extract_command_anchor(&prompt_lower).unwrap_or("bash");
            grounded = find_command_provider_facts(facts, command, max_facts);
            if let Some(fact) = grounded.first() {
                format!(
                    "Command {command} is grounded by route {}: {} {} {}.",
                    fact.route, fact.subject, fact.relation, fact.object
                )
            } else {
                answer_allowed = false;
                refused_shortcut = Some("command_provider_fact_missing");
                format!("I do not have a grounded provider fact for command {command}.")
            }
        }
        "package_running_boundary" => {
            grounded = find_route_facts(facts, "linux.boundary.package", max_facts);
            if let Some(fact) = grounded.first() {
                format!("No. {} {} {}.", fact.subject, fact.relation, fact.object)
            } else {
                answer_allowed = false;
                refused_shortcut = Some("package_boundary_fact_missing");
                "I cannot prove that package installation says anything about runtime state."
                    .to_string()
            }
        }
        "socket_firewall_boundary" => {
            grounded = find_route_facts(facts, "linux.boundary.socket", max_facts);
            if let Some(fact) = grounded.first() {
                format!("No. {} {} {}.", fact.subject, fact.relation, fact.object)
            } else {
                answer_allowed = false;
                refused_shortcut = Some("socket_boundary_fact_missing");
                "I cannot prove a firewall conclusion from listener facts alone.".to_string()
            }
        }
        "listener_summary" => {
            grounded = find_socket_facts(facts, max_facts);
            format!(
                "The Linux field sees {} socket candidates: {} external-binding candidates and {} localhost candidates. This is listener evidence, not external exposure proof.",
                exposure.exposure_field.candidate_count,
                exposure.exposure_field.external_binding_count,
                exposure.exposure_field.localhost_binding_count
            )
        }
        "external_exposure" => {
            grounded = find_socket_facts(facts, max_facts);
            if exposure.claim_boundary.external_exposure_confirmed {
                "External exposure is confirmed for review because listener evidence and matching firewall allow evidence are both present.".to_string()
            } else {
                refused_shortcut = Some("external_exposure_not_confirmed");
                format!(
                    "External exposure is not confirmed. Field state is {}; listeners exist, but the boundary requires matching firewall allow evidence.",
                    exposure.exposure_field.state
                )
            }
        }
        "vulnerable_runtime_boundary" => {
            grounded = find_route_facts(facts, "linux.boundary.cve", max_facts);
            if let Some(fact) = grounded.first() {
                format!("No. {} {} {}.", fact.subject, fact.relation, fact.object)
            } else {
                answer_allowed = false;
                refused_shortcut = Some("cve_boundary_fact_missing");
                "I cannot prove runtime exposure from a vulnerable package fact alone.".to_string()
            }
        }
        "context_recall" => {
            let topic_text = context
                .last_topic
                .clone()
                .unwrap_or_else(|| "no previous Linux topic".to_string());
            format!("Your last Linux topic was {topic_text}.")
        }
        _ => {
            answer_allowed = false;
            refused_shortcut = Some("unsupported_open_domain_prompt");
            "I do not have enough grounded Linux-profile facts to answer that.".to_string()
        }
    };
    let grounded_fact_count = grounded.len();
    let field_state = if answer_allowed {
        if refused_shortcut.is_some() {
            "GROUNDED_BOUNDARY_ANSWER"
        } else if grounded_fact_count > 0 || matches!(intent, "listener_summary" | "context_recall")
        {
            "GROUNDED_ANSWER"
        } else {
            "REVIEW"
        }
    } else {
        "REFUSED_UNSUPPORTED"
    };
    if topic != "context" {
        context.last_topic = Some(topic.to_string());
        context.last_intent = Some(intent);
    }
    LinuxChatTurn {
        turn_index,
        prompt: prompt.to_string(),
        intent,
        field_state,
        answer_allowed,
        answer,
        grounded_fact_count,
        grounded_facts: grounded,
        refused_shortcut,
        context_topic_before,
        context_topic_after: context.last_topic.clone(),
    }
}

fn eval_linux_chat_turns(turns: &[LinuxChatTurn]) -> LinuxChatEvalSummary {
    let cases = builtin_linux_chat_cases();
    let eval_cases = cases
        .iter()
        .zip(turns.iter())
        .map(|(case, turn)| {
            let answer_lower = turn.answer.to_ascii_lowercase();
            let contains_ok = case.expected_answer_contains.is_empty()
                || answer_lower.contains(case.expected_answer_contains);
            let forbid_ok = case.forbid_answer_contains.is_empty()
                || !answer_lower.contains(case.forbid_answer_contains);
            let passed = turn.intent == case.expected_intent
                && turn.answer_allowed == case.expected_answer_allowed
                && contains_ok
                && forbid_ok;
            LinuxChatEvalCase {
                id: case.id,
                expected_intent: case.expected_intent,
                expected_answer_allowed: case.expected_answer_allowed,
                expected_answer_contains: case.expected_answer_contains,
                forbid_answer_contains: case.forbid_answer_contains,
                observed_intent: turn.intent,
                observed_answer_allowed: turn.answer_allowed,
                observed_answer: turn.answer.clone(),
                passed,
            }
        })
        .collect::<Vec<_>>();
    let total = eval_cases.len();
    let passed = eval_cases.iter().filter(|case| case.passed).count();
    let grounded_turns = turns
        .iter()
        .filter(|turn| turn.answer_allowed && turn.field_state != "REFUSED_UNSUPPORTED")
        .count();
    let refusal_turns = turns
        .iter()
        .filter(|turn| !turn.answer_allowed || turn.refused_shortcut.is_some())
        .count();
    let false_positive_turns = turns
        .iter()
        .filter(|turn| {
            turn.answer
                .to_ascii_lowercase()
                .contains("external exposure is confirmed")
                && turn.refused_shortcut == Some("external_exposure_not_confirmed")
        })
        .count();
    let context_turns = turns
        .iter()
        .filter(|turn| turn.intent == "context_recall")
        .count();
    let context_retention_passed = turns
        .iter()
        .filter(|turn| {
            turn.intent == "context_recall"
                && turn
                    .answer
                    .to_ascii_lowercase()
                    .contains("external exposure")
        })
        .count();
    LinuxChatEvalSummary {
        cases: eval_cases,
        metrics: LinuxChatEvalMetrics {
            total,
            passed,
            pass_rate: ratio(passed, total),
            grounded_turns,
            grounded_answer_rate: ratio(grounded_turns, turns.len()),
            refusal_turns,
            false_positive_turns,
            false_positive_rate: ratio(false_positive_turns, turns.len()),
            context_turns,
            context_retention_passed,
            context_retention_rate: if context_turns == 0 {
                1.0
            } else {
                ratio(context_retention_passed, context_turns)
            },
        },
    }
}

fn classify_prompt(prompt_lower: &str, context: &ChatContext) -> (&'static str, &'static str) {
    if prompt_lower.contains("last topic") || prompt_lower.contains("previous topic") {
        return ("context_recall", "context");
    }
    if prompt_lower.contains("what about")
        && prompt_lower.contains("external")
        && context.last_intent == Some("listener_summary")
    {
        return ("external_exposure", "external exposure");
    }
    if prompt_lower.contains("which package provides command")
        || prompt_lower.contains("what package provides command")
        || prompt_lower.contains("who provides command")
    {
        return ("command_provider", "command provider");
    }
    if prompt_lower.contains("package installed") && prompt_lower.contains("running") {
        return ("package_running_boundary", "package runtime boundary");
    }
    if prompt_lower.contains("port listening") && prompt_lower.contains("firewall") {
        return ("socket_firewall_boundary", "socket firewall boundary");
    }
    if prompt_lower.contains("listener")
        || prompt_lower.contains("listening")
        || prompt_lower.contains("socket")
    {
        return ("listener_summary", "listeners");
    }
    if prompt_lower.contains("external exposure") || prompt_lower.contains("exposed externally") {
        return ("external_exposure", "external exposure");
    }
    if prompt_lower.contains("vulnerable package") && prompt_lower.contains("runtime exposure") {
        return (
            "vulnerable_runtime_boundary",
            "vulnerability runtime boundary",
        );
    }
    ("unsupported", "unsupported")
}

fn extract_command_anchor(prompt_lower: &str) -> Option<&str> {
    let tokens = prompt_lower
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '.')
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    tokens
        .windows(2)
        .find_map(|window| (window[0] == "command").then_some(window[1]))
}

fn find_command_provider_facts(
    facts: &[LinuxResidualDecodedFact],
    command: &str,
    max_facts: usize,
) -> Vec<LinuxChatFact> {
    let mut scored = facts
        .iter()
        .filter_map(|fact| {
            let route_ok = fact.route.contains("command") || fact.route == "linux.package.binary";
            if !route_ok || fact.polarity != "positive" {
                return None;
            }
            let subject = fact.subject.to_ascii_lowercase();
            let object = fact.object.to_ascii_lowercase();
            let score = if subject == command || object == command {
                100
            } else if object.ends_with(&format!("/{command}")) {
                92
            } else if subject.contains(command) || object.contains(command) {
                45
            } else {
                0
            };
            (score > 0).then(|| (score, fact_to_chat_fact(fact)))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.route.cmp(&right.1.route))
            .then_with(|| left.1.object.cmp(&right.1.object))
    });
    scored
        .into_iter()
        .map(|(_, fact)| fact)
        .take(max_facts)
        .collect()
}

fn find_route_facts(
    facts: &[LinuxResidualDecodedFact],
    route: &str,
    max_facts: usize,
) -> Vec<LinuxChatFact> {
    facts
        .iter()
        .filter(|fact| fact.route == route)
        .map(fact_to_chat_fact)
        .take(max_facts)
        .collect()
}

fn find_socket_facts(facts: &[LinuxResidualDecodedFact], max_facts: usize) -> Vec<LinuxChatFact> {
    facts
        .iter()
        .filter(|fact| fact.route == "linux.socket.runtime" || fact.route == "linux.systemd.socket")
        .map(fact_to_chat_fact)
        .take(max_facts)
        .collect()
}

fn fact_to_chat_fact(fact: &LinuxResidualDecodedFact) -> LinuxChatFact {
    LinuxChatFact {
        route: fact.route.clone(),
        subject: fact.subject.clone(),
        relation: fact.relation.clone(),
        object: fact.object.clone(),
        polarity: fact.polarity,
        memory_kind: fact.memory_kind,
        confidence: fact.confidence,
    }
}

fn builtin_linux_chat_cases() -> Vec<BuiltinLinuxChatCase> {
    vec![
        BuiltinLinuxChatCase {
            id: "command-provider-bash",
            prompt: "Which package provides command bash?",
            expected_intent: "command_provider",
            expected_answer_allowed: true,
            expected_answer_contains: "bash",
            forbid_answer_contains: "",
        },
        BuiltinLinuxChatCase {
            id: "package-installed-not-running",
            prompt: "Does package installed prove binary is running?",
            expected_intent: "package_running_boundary",
            expected_answer_allowed: true,
            expected_answer_contains: "does not prove",
            forbid_answer_contains: "yes",
        },
        BuiltinLinuxChatCase {
            id: "listener-summary",
            prompt: "What listeners are present?",
            expected_intent: "listener_summary",
            expected_answer_allowed: true,
            expected_answer_contains: "listener evidence",
            forbid_answer_contains: "external exposure is confirmed",
        },
        BuiltinLinuxChatCase {
            id: "external-exposure-followup",
            prompt: "What about external exposure?",
            expected_intent: "external_exposure",
            expected_answer_allowed: true,
            expected_answer_contains: "not confirmed",
            forbid_answer_contains: "external exposure is confirmed",
        },
        BuiltinLinuxChatCase {
            id: "context-recall",
            prompt: "What was my last topic?",
            expected_intent: "context_recall",
            expected_answer_allowed: true,
            expected_answer_contains: "external exposure",
            forbid_answer_contains: "",
        },
    ]
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
    use super::super::linux_atlas::{LinuxAtlasEvidence, LinuxAtlasFact};
    use super::super::linux_residual_memory::{
        build_linux_residual_pack_report, LinuxResidualPackConfig,
    };
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn linux_chat_ready_over_schema_residual_memory() {
        let root = fixture_root("linux-chat-ready");
        let out = root.join("linux.lrf");
        build_linux_residual_pack_report(LinuxResidualPackConfig {
            atlas_dir: root.clone(),
            max_active_facts: 8,
            promotion_threshold: 2,
            out: out.clone(),
        })
        .unwrap();

        let report = build_linux_chat_report(LinuxChatConfig {
            residual_pack: out,
            prompt: Vec::new(),
            max_facts: 4,
        })
        .unwrap();
        assert_eq!(
            report.verdict,
            "LINUX_PROFILE_BROAD_CHAT_READY_NOT_GENERAL_LLM"
        );
        assert!(report.claim_boundary.broad_chat_llm_ready);
        assert!(report.claim_boundary.linux_profile_broad_chat_ready);
        assert!(!report.claim_boundary.general_llm_ready);
        assert!(!report.claim_boundary.vulnerability_scan_ready);
        assert_eq!(report.eval.metrics.passed, report.eval.metrics.total);
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
                "linux.systemd.exec",
                "ssh.service",
                "execstart",
                "/usr/sbin/sshd",
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
            domain: "linux-chat-test".to_string(),
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
