//! Linux-profile feedback packets for local anti-wave and positive-route memory.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    build_linux_reason_report, LinuxProfileBoundary, LinuxReasonReport, LinuxReasonRunConfig,
    LINUX_PROFILE_VERSION,
};

#[derive(Clone)]
pub(crate) struct LinuxFeedbackBuildConfig {
    pub residual_pack: PathBuf,
    pub text: String,
    pub decision: String,
    pub note: Option<String>,
    pub out: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct LinuxFeedbackApplyConfig {
    pub residual_pack: PathBuf,
    pub feedback: PathBuf,
    pub text: String,
    pub max_facts: usize,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxFeedbackPacket {
    pub version: String,
    pub source_prompt: String,
    pub decision: String,
    pub note: String,
    pub intent: String,
    pub negative_lanes: Vec<LinuxFeedbackLane>,
    pub positive_lanes: Vec<LinuxFeedbackLane>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinuxFeedbackLane {
    pub lane_id: String,
    pub intent: String,
    pub route: String,
    pub shortcut: String,
    pub polarity: String,
    pub strength: u8,
    pub evidence: Vec<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxFeedbackBuildReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub before: LinuxReasonReport,
    pub packet: LinuxFeedbackPacket,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxFeedbackApplyReport {
    pub mode: &'static str,
    pub version: &'static str,
    pub verdict: &'static str,
    pub before: LinuxReasonReport,
    pub feedback: LinuxFeedbackPacket,
    pub applied: LinuxFeedbackApplied,
    pub after: LinuxFeedbackAfter,
    pub claim_boundary: LinuxProfileBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxFeedbackApplied {
    pub negative_lanes_matched: usize,
    pub positive_lanes_matched: usize,
    pub matched_lane_ids: Vec<String>,
    pub anti_wave_strength_before: u32,
    pub anti_wave_strength_after: u32,
    pub positive_route_strength_before: u32,
    pub positive_route_strength_after: u32,
}

#[derive(Serialize, Clone)]
pub(crate) struct LinuxFeedbackAfter {
    pub state: String,
    pub answer_allowed: bool,
    pub answer: String,
    pub learned_negative_lanes_active: bool,
    pub learned_positive_lanes_active: bool,
}

pub(crate) fn build_linux_feedback_report(
    config: LinuxFeedbackBuildConfig,
) -> Result<LinuxFeedbackBuildReport> {
    let before = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: config.residual_pack,
        text: config.text.clone(),
        max_facts: 4,
    })?;
    let decision = normalize_decision(&config.decision);
    let packet = LinuxFeedbackPacket {
        version: LINUX_PROFILE_VERSION.to_string(),
        source_prompt: config.text,
        decision: decision.to_string(),
        note: config.note.unwrap_or_default(),
        intent: before.query_wave.intent.clone(),
        negative_lanes: if decision == "reject" {
            negative_lanes_from_reason(&before)
        } else {
            Vec::new()
        },
        positive_lanes: if decision == "accept" {
            positive_lanes_from_reason(&before)
        } else {
            Vec::new()
        },
    };
    let verdict = if packet.negative_lanes.is_empty() && packet.positive_lanes.is_empty() {
        "LINUX_FEEDBACK_OBSERVED_NO_MEMORY_LANES"
    } else {
        "LINUX_FEEDBACK_PACKET_READY_NOT_TRAINING"
    };
    let report = LinuxFeedbackBuildReport {
        mode: "llmwave-big-linux-feedback-build",
        version: LINUX_PROFILE_VERSION,
        verdict,
        before,
        packet,
        claim_boundary: boundary("Linux feedback packet is local profile memory only; it is not gradient training or general LLM readiness."),
    };
    write_json_if_requested(config.out.as_ref(), &report.packet)?;
    Ok(report)
}

pub(crate) fn build_linux_feedback_apply_report(
    config: LinuxFeedbackApplyConfig,
) -> Result<LinuxFeedbackApplyReport> {
    let packet: LinuxFeedbackPacket = serde_json::from_str(
        &fs::read_to_string(&config.feedback)
            .with_context(|| format!("read feedback packet {}", config.feedback.display()))?,
    )
    .with_context(|| format!("parse feedback packet {}", config.feedback.display()))?;
    let before = build_linux_reason_report(LinuxReasonRunConfig {
        residual_pack: config.residual_pack,
        text: config.text,
        max_facts: config.max_facts.max(1),
    })?;
    let applied = match_lanes(&packet, &before);
    let after = after_decision(&before, &applied);
    let report = LinuxFeedbackApplyReport {
        mode: "llmwave-big-linux-feedback-apply",
        version: LINUX_PROFILE_VERSION,
        verdict: if applied.negative_lanes_matched > 0 || applied.positive_lanes_matched > 0 {
            "LINUX_FEEDBACK_MEMORY_APPLIED_NOT_GENERAL_TRAINING"
        } else {
            "LINUX_FEEDBACK_MEMORY_NO_MATCH"
        },
        before,
        feedback: packet,
        applied,
        after,
        claim_boundary: boundary("Linux feedback changed only this local profile field pass; broad training and general LLM claims remain blocked."),
    };
    write_json_if_requested(config.out.as_ref(), &report)?;
    Ok(report)
}

fn negative_lanes_from_reason(reason: &LinuxReasonReport) -> Vec<LinuxFeedbackLane> {
    let mut lanes = Vec::new();
    for shortcut in &reason.query_wave.forbidden_shortcuts {
        lanes.push(LinuxFeedbackLane {
            lane_id: format!("neg:{}:{}", reason.query_wave.intent, shortcut),
            intent: reason.query_wave.intent.clone(),
            route: reason
                .query_wave
                .required_routes
                .first()
                .cloned()
                .unwrap_or_else(|| "linux.profile.unknown".to_string()),
            shortcut: shortcut.clone(),
            polarity: "negative".to_string(),
            strength: 16,
            evidence: reason
                .anti_wave_hits
                .iter()
                .filter(|hit| hit.shortcut == *shortcut)
                .map(|hit| hit.reason.clone())
                .collect(),
        });
    }
    lanes
}

fn positive_lanes_from_reason(reason: &LinuxReasonReport) -> Vec<LinuxFeedbackLane> {
    let mut lanes = Vec::new();
    for step in &reason.evidence_chain {
        if step.polarity == "positive" {
            lanes.push(LinuxFeedbackLane {
                lane_id: format!(
                    "pos:{}:{}:{}",
                    reason.query_wave.intent, step.route, step.role
                ),
                intent: reason.query_wave.intent.clone(),
                route: step.route.clone(),
                shortcut: "accepted_route_support".to_string(),
                polarity: "positive".to_string(),
                strength: 8,
                evidence: vec![format!(
                    "{} {} {}",
                    step.subject, step.relation, step.object
                )],
            });
        }
    }
    lanes.truncate(8);
    lanes
}

fn match_lanes(packet: &LinuxFeedbackPacket, reason: &LinuxReasonReport) -> LinuxFeedbackApplied {
    let mut matched_lane_ids = Vec::new();
    let mut negative_strength = 0u32;
    let mut positive_strength = 0u32;
    let negative_matches = packet
        .negative_lanes
        .iter()
        .filter(|lane| {
            lane.intent == reason.query_wave.intent
                && reason
                    .query_wave
                    .forbidden_shortcuts
                    .iter()
                    .any(|shortcut| shortcut == &lane.shortcut)
        })
        .inspect(|lane| {
            matched_lane_ids.push(lane.lane_id.clone());
            negative_strength += lane.strength as u32;
        })
        .count();
    let positive_matches = packet
        .positive_lanes
        .iter()
        .filter(|lane| {
            lane.intent == reason.query_wave.intent
                && reason
                    .evidence_chain
                    .iter()
                    .any(|step| step.route == lane.route)
        })
        .inspect(|lane| {
            matched_lane_ids.push(lane.lane_id.clone());
            positive_strength += lane.strength as u32;
        })
        .count();
    let anti_before = reason.anti_wave_hits.len() as u32 * 8;
    let positive_before = reason.evidence_chain.len() as u32 * 4;
    LinuxFeedbackApplied {
        negative_lanes_matched: negative_matches,
        positive_lanes_matched: positive_matches,
        matched_lane_ids,
        anti_wave_strength_before: anti_before,
        anti_wave_strength_after: anti_before + negative_strength,
        positive_route_strength_before: positive_before,
        positive_route_strength_after: positive_before + positive_strength,
    }
}

fn after_decision(
    reason: &LinuxReasonReport,
    applied: &LinuxFeedbackApplied,
) -> LinuxFeedbackAfter {
    let learned_negative = applied.negative_lanes_matched > 0;
    let learned_positive = applied.positive_lanes_matched > 0;
    let state = if learned_negative {
        format!("{}_LEARNED_ANTI_WAVE", reason.decision.state)
    } else if learned_positive {
        format!("{}_LEARNED_POSITIVE_ROUTE", reason.decision.state)
    } else {
        reason.decision.state.clone()
    };
    let answer = if learned_negative {
        format!(
            "{} Learned anti-wave lanes reinforce the shortcut refusal for this profile route.",
            reason.decision.answer
        )
    } else if learned_positive {
        format!(
            "{} Learned positive lanes reinforce the accepted evidence route.",
            reason.decision.answer
        )
    } else {
        reason.decision.answer.clone()
    };
    LinuxFeedbackAfter {
        state,
        answer_allowed: reason.decision.answer_allowed,
        answer,
        learned_negative_lanes_active: learned_negative,
        learned_positive_lanes_active: learned_positive,
    }
}

fn normalize_decision(decision: &str) -> &'static str {
    match decision.to_ascii_lowercase().as_str() {
        "accept" | "accepted" | "yes" => "accept",
        "reject" | "rejected" | "no" => "reject",
        _ => "watch",
    }
}

fn boundary(safe_claim: &str) -> LinuxProfileBoundary {
    LinuxProfileBoundary {
        linux_profile_query_wave_ready: true,
        linux_profile_reasoning_ready: true,
        linux_profile_broad_eval_ready: false,
        linux_profile_broad_chat_ready: false,
        linux_profile_nonlinear_memory_proven: false,
        general_llm_ready: false,
        open_domain_chat_ready: false,
        vulnerability_scanner_ready: false,
        network_scanner_ready: false,
        safe_claim: safe_claim.to_string(),
        blocked_claims: vec![
            "general_llm_ready".to_string(),
            "open_domain_chat_ready".to_string(),
            "vulnerability_scanner_ready".to_string(),
            "network_scanner_ready".to_string(),
        ],
    }
}

fn write_json_if_requested<T: Serialize>(out: Option<&PathBuf>, report: &T) -> Result<()> {
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output dir {}", parent.display()))?;
        }
        fs::write(path, serde_json::to_string_pretty(report)?)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}
