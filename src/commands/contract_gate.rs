use crate::*;
use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub(crate) struct ContractGateArgs {
    #[arg(long)]
    pub(crate) input: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "generic")]
    pub(crate) profile: ContractProfile,
    #[arg(long)]
    pub(crate) template: bool,
    #[arg(long = "audit-dir")]
    pub(crate) audit_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ContractProfile {
    Generic,
    Protocol,
    Edo,
}

#[derive(Deserialize, Serialize, Clone, Default)]
struct ContractPacket {
    #[serde(default)]
    task_id: String,
    #[serde(default)]
    base_document: String,
    #[serde(default)]
    counterparty_document: String,
    #[serde(default)]
    protocol_author: String,
    #[serde(default)]
    our_party: String,
    #[serde(default)]
    counterparty: String,
    #[serde(default)]
    document_type: String,
    #[serde(default)]
    same_clause_multi_effect_ok: bool,
    #[serde(default)]
    extracted_text_paths: Vec<String>,
    #[serde(default)]
    clauses: Vec<ContractClause>,
    #[serde(default)]
    edo_messages: Vec<EdoMessage>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
struct ContractClause {
    id: String,
    #[serde(default)]
    route: String,
    #[serde(default)]
    source_clause: String,
    #[serde(default)]
    original_obligation: String,
    #[serde(default)]
    protocol_change: String,
    #[serde(default)]
    protected_party: String,
    #[serde(default)]
    risk_removed: String,
    #[serde(default)]
    risk_tag: String,
    #[serde(default)]
    effects: Vec<String>,
    #[serde(default)]
    evidence: String,
}

#[derive(Deserialize, Serialize, Clone, Default)]
struct EdoMessage {
    message: String,
    #[serde(default)]
    sender: String,
    #[serde(default)]
    receiver: String,
    #[serde(default)]
    legal_effect: String,
    #[serde(default)]
    risk: String,
    #[serde(default)]
    fallback_route: String,
}

pub(crate) fn cmd(args: ContractGateArgs) -> Result<u8> {
    let out = if args.template {
        template(args.profile)
    } else {
        let input = args
            .input
            .as_ref()
            .ok_or_else(|| anyhow!("--input is required unless --template is used"))?;
        let raw = fs::read_to_string(input)
            .with_context(|| format!("read contract packet {}", input.display()))?;
        let packet: ContractPacket = serde_json::from_str(&raw)
            .with_context(|| format!("parse contract packet {}", input.display()))?;
        let mut report = build_report(&packet, args.profile, input);
        if let Some(audit_dir) = args.audit_dir.as_ref() {
            write_audit(audit_dir, &report)?;
            report["audit_trail"]["audit_dir"] = json!(audit_dir.display().to_string());
        }
        report
    };
    print(&out, &args.format)?;
    Ok(exit_for(&out))
}

fn build_report(packet: &ContractPacket, profile: ContractProfile, input: &Path) -> Value {
    let role = role_first_pass(packet);
    let protocol = protocol_direction(packet);
    let multi_effect = multi_effect_check(packet);
    let risk_map = risk_map(packet, profile);
    let split_plan = split_plan(packet, profile, &multi_effect);
    let negotiation = negotiation_position(packet, &risk_map);
    let watch_reasons = watch_reasons(&role, &protocol, &multi_effect, packet);
    let verdict = if watch_reasons.is_empty() {
        "STRUCTURAL_PASS_NOT_LEGAL_APPROVAL"
    } else {
        "WATCH"
    };

    json!({
        "mode": "nanda-contract-gate",
        "version": "contract-gate-v1-universal-document-flow",
        "verdict": verdict,
        "safe_to_sign": false,
        "profile": profile,
        "input": input.display().to_string(),
        "role_first_pass": role,
        "protocol_direction_check": protocol,
        "multi_effect_clause_check": multi_effect,
        "risk_map": risk_map,
        "split_plan": split_plan.clone(),
        "negotiation_position": negotiation,
        "audit_trail": {
            "input_packet": input.display().to_string(),
            "extracted_text_paths": &packet.extracted_text_paths,
            "triad_packet_paths": [],
            "gate_results": [],
            "veto_repair_steps": split_plan,
            "final_passed_routes": risk_map["routes"]
        },
        "claim_boundary": {
            "structural_pass_is_not_legal_approval": true,
            "final_signing_requires_legal_review": true,
            "final_signing_requires_accounting_review_when_payment_tax_or_edo_risk_present": true,
            "safe_claim": "NANDA contract-gate checks role, route, protocol direction, risk-tag, and split coherence. PASS means structural coherence only, not legal approval."
        },
        "watch_reasons": watch_reasons,
        "read_as": "Universal contract/document-flow gate. It does not hardcode a counterparty or project; domain behavior comes from packet fields and optional profile."
    })
}

fn role_first_pass(packet: &ContractPacket) -> Value {
    let missing = [
        ("our_party", packet.our_party.as_str()),
        ("counterparty", packet.counterparty.as_str()),
        ("protocol_author", packet.protocol_author.as_str()),
        ("document_type", packet.document_type.as_str()),
    ]
    .into_iter()
    .filter_map(|(name, value)| {
        if value.trim().is_empty() {
            Some(name)
        } else {
            None
        }
    })
    .collect::<Vec<_>>();
    let same_party = !packet.our_party.is_empty()
        && !packet.counterparty.is_empty()
        && eq_norm(&packet.our_party, &packet.counterparty);
    let verdict = if !missing.is_empty() || same_party {
        "WATCH"
    } else {
        "PASS"
    };
    json!({
        "verdict": verdict,
        "our_party": packet.our_party,
        "counterparty": packet.counterparty,
        "protocol_author": packet.protocol_author,
        "document_type": packet.document_type,
        "missing": missing,
        "same_party_error": same_party,
        "must_pass_before_deep_review": true
    })
}

fn protocol_direction(packet: &ContractPacket) -> Value {
    let author = packet.protocol_author.trim();
    let direction = if author.is_empty() {
        "unknown"
    } else if eq_norm(author, &packet.our_party) {
        "our_revision"
    } else if eq_norm(author, &packet.counterparty) {
        "counterparty_revision"
    } else {
        "third_party_or_unmatched_author"
    };
    let benefits = if direction == "our_revision" {
        "our_party"
    } else if direction == "counterparty_revision" {
        "counterparty"
    } else {
        "unknown"
    };
    json!({
        "verdict": if direction == "unknown" || direction == "third_party_or_unmatched_author" { "WATCH" } else { "PASS" },
        "direction": direction,
        "who_authored_protocol": author,
        "who_benefits_default": benefits,
        "source_vs_proposed_are_separate_routes": true
    })
}

fn multi_effect_check(packet: &ContractPacket) -> Value {
    let multi = packet
        .clauses
        .iter()
        .filter(|clause| clause.effects.len() > 1)
        .map(|clause| {
            json!({
                "clause_id": clause.id,
                "effect_count": clause.effects.len(),
                "effects": clause.effects
            })
        })
        .collect::<Vec<_>>();
    let verdict = if multi.is_empty() || packet.same_clause_multi_effect_ok {
        "PASS"
    } else {
        "WATCH"
    };
    json!({
        "verdict": verdict,
        "same_clause_multi_effect_ok": packet.same_clause_multi_effect_ok,
        "multi_effect_clauses": multi,
        "repair": if verdict == "WATCH" {
            json!("split_by_subclaim")
        } else {
            Value::Null
        }
    })
}

fn risk_map(packet: &ContractPacket, profile: ContractProfile) -> Value {
    let mut routes = BTreeMap::<String, Vec<Value>>::new();
    let mut tags = BTreeSet::<String>::new();
    for clause in &packet.clauses {
        let route = value_or(&clause.route, "unrouted-contract-clause");
        let risk_tag = value_or(&clause.risk_tag, "untagged_contract_risk");
        tags.insert(risk_tag.clone());
        routes.entry(route).or_default().push(json!({
            "clause_id": clause.id,
            "risk_tag": risk_tag,
            "source_clause": clause.source_clause,
            "original_obligation": clause.original_obligation,
            "protocol_change": clause.protocol_change,
            "protected_party": clause.protected_party,
            "risk_removed": clause.risk_removed,
            "effect_count": clause.effects.len(),
            "evidence": clause.evidence
        }));
    }
    let edo = if profile == ContractProfile::Edo || !packet.edo_messages.is_empty() {
        edo_map(packet)
    } else {
        Value::Null
    };
    json!({
        "routes": routes,
        "risk_tags": tags.into_iter().collect::<Vec<_>>(),
        "edo_route": edo,
        "known_risk_tag_examples": [
            "auto_acceptance",
            "unilateral_offset",
            "penalty_disproportion",
            "payment_delay",
            "evidence_irrebuttable",
            "supplier_liability_after_transfer",
            "edo_technical_failure",
            "tax_indemnity",
            "quality_claim_after_retail",
            "document_signature_block"
        ]
    })
}

fn edo_map(packet: &ContractPacket) -> Value {
    let messages = if packet.edo_messages.is_empty() {
        vec![
            json!({"message":"ORDERS","legal_effect":"request/order","fallback_route":"ORDRSP required"}),
            json!({"message":"ORDRSP","legal_effect":"confirmation","fallback_route":"partial confirmation safe"}),
            json!({"message":"DESADV","legal_effect":"shipment notice","fallback_route":"paper/email fallback"}),
            json!({"message":"RECADV","legal_effect":"receiving advice","fallback_route":"UPD/TTN/acts/logs"}),
            json!({"message":"UPD","legal_effect":"accounting document","fallback_route":"correction document"}),
        ]
    } else {
        packet
            .edo_messages
            .iter()
            .map(|message| {
                json!({
                    "message": message.message,
                    "sender": message.sender,
                    "receiver": message.receiver,
                    "legal_effect": message.legal_effect,
                    "risk": message.risk,
                    "fallback_route": message.fallback_route
                })
            })
            .collect::<Vec<_>>()
    };
    json!({
        "schema": "message | sender | receiver | legal_effect | risk | fallback_route",
        "messages": messages
    })
}

fn split_plan(
    packet: &ContractPacket,
    profile: ContractProfile,
    multi_effect: &Value,
) -> Vec<Value> {
    let mut routes = packet
        .clauses
        .iter()
        .map(|clause| value_or(&clause.route, "unrouted-contract-clause"))
        .collect::<BTreeSet<_>>();
    if profile == ContractProfile::Edo || !packet.edo_messages.is_empty() {
        routes.insert("edo-evidence".to_string());
    }
    let mut plan = vec![json!({
        "step": 1,
        "route": "roles-and-protocol-direction",
        "reason": "who is our side, who is counterparty, who authored protocol, who benefits from edits"
    })];
    for (idx, route) in routes.into_iter().enumerate() {
        plan.push(json!({
            "step": idx + 2,
            "route": route,
            "reason": "route-level contract risk check"
        }));
    }
    if multi_effect["verdict"] == "WATCH" {
        plan.push(json!({
            "step": plan.len() + 1,
            "route": "same-clause-subclaims",
            "reason": "same clause has multiple legal effects; split by atomic subclaim"
        }));
    }
    plan
}

fn negotiation_position(packet: &ContractPacket, risk_map: &Value) -> Value {
    let mut must_hold = Vec::new();
    let mut can_compromise = Vec::new();
    let mut legal_review = Vec::new();
    let mut accounting_review = Vec::new();
    let mut commercial_risk = Vec::new();

    for clause in &packet.clauses {
        let tag = clause.risk_tag.as_str();
        let item = json!({
            "clause_id": clause.id,
            "risk_tag": value_or(&clause.risk_tag, "untagged_contract_risk"),
            "protocol_change": clause.protocol_change,
            "protected_party": clause.protected_party
        });
        match tag {
            "auto_acceptance" | "unilateral_offset" | "evidence_irrebuttable" => {
                must_hold.push(item)
            }
            "tax_indemnity" | "payment_delay" => accounting_review.push(item),
            "penalty_disproportion" | "quality_claim_after_retail" => legal_review.push(item),
            "edo_technical_failure" | "document_signature_block" => can_compromise.push(item),
            _ => commercial_risk.push(item),
        }
    }

    json!({
        "must_hold": must_hold,
        "can_compromise": can_compromise,
        "legal_review": legal_review,
        "accounting_review": accounting_review,
        "commercial_risk": commercial_risk,
        "send_to_counterparty_summary": summarize_for_counterparty(packet, risk_map)
    })
}

fn summarize_for_counterparty(packet: &ContractPacket, risk_map: &Value) -> String {
    let route_count = risk_map["routes"]
        .as_object()
        .map(|routes| routes.len())
        .unwrap_or(0);
    format!(
        "We reviewed {} against {} and separated {} risk route(s). Proposed edits should be discussed by route; structural PASS is not legal approval.",
        value_or(&packet.counterparty_document, "counterparty document"),
        value_or(&packet.base_document, "base document"),
        route_count
    )
}

fn watch_reasons(
    role: &Value,
    protocol: &Value,
    multi_effect: &Value,
    packet: &ContractPacket,
) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if role["verdict"] != "PASS" {
        reasons.push("role_first_pass_not_passed");
    }
    if protocol["verdict"] != "PASS" {
        reasons.push("protocol_direction_unknown_or_unmatched");
    }
    if multi_effect["verdict"] != "PASS" {
        reasons.push("same_clause_multi_effect_requires_subclaim_split");
    }
    if packet.clauses.is_empty() {
        reasons.push("no_contract_clauses_provided");
    }
    if packet
        .clauses
        .iter()
        .any(|clause| clause.risk_tag.trim().is_empty())
    {
        reasons.push("risk_tags_missing");
    }
    reasons
}

fn template(profile: ContractProfile) -> Value {
    json!({
        "mode": "nanda-contract-gate-template",
        "version": "contract-gate-v1-universal-document-flow",
        "profile": profile,
        "packet": {
            "task_id": "contract-review",
            "base_document": "contract.pdf",
            "counterparty_document": "protocol-or-counterparty-redline.pdf",
            "protocol_author": "our_party_or_counterparty_or_unknown",
            "our_party": "our legal entity",
            "counterparty": "counterparty legal entity",
            "document_type": "contract | appendix | edo_agreement | protocol",
            "same_clause_multi_effect_ok": false,
            "extracted_text_paths": ["extracted/base.txt", "extracted/protocol.txt"],
            "clauses": [
                {
                    "id": "p1",
                    "route": "payment-or-offset",
                    "source_clause": "original clause id/text",
                    "original_obligation": "original obligation",
                    "protocol_change": "proposed change",
                    "protected_party": "our_party | counterparty | both | unknown",
                    "risk_removed": "risk removed by proposed edit",
                    "risk_tag": "unilateral_offset",
                    "effects": ["effect one", "effect two"],
                    "evidence": "page/paragraph/source"
                }
            ],
            "edo_messages": [
                {
                    "message": "ORDERS",
                    "sender": "buyer",
                    "receiver": "supplier",
                    "legal_effect": "request/order",
                    "risk": "may be treated as binding",
                    "fallback_route": "ORDRSP required"
                }
            ]
        },
        "read_as": "Fill this packet from any contract/protocol pair. The schema is universal; profiles only add route hints, not project-specific hardcode."
    })
}

fn write_audit(dir: &Path, report: &Value) -> Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(
        dir.join("contract-gate-report.json"),
        serde_json::to_string_pretty(report)? + "\n",
    )?;
    fs::write(
        dir.join("negotiation-position.json"),
        serde_json::to_string_pretty(&report["negotiation_position"])? + "\n",
    )?;
    Ok(())
}

fn print(value: &Value, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value)?),
        OutputFormat::Text => {
            println!("NANDA Contract Gate");
            println!(
                "verdict: {}",
                value["verdict"].as_str().unwrap_or("UNKNOWN")
            );
            println!(
                "protocol_direction: {}",
                value["protocol_direction_check"]["direction"]
                    .as_str()
                    .unwrap_or("unknown")
            );
            println!("safe_to_sign: false");
        }
        OutputFormat::Md => {
            println!("# NANDA Contract Gate");
            println!();
            println!(
                "- verdict: `{}`",
                value["verdict"].as_str().unwrap_or("UNKNOWN")
            );
            println!(
                "- protocol_direction: `{}`",
                value["protocol_direction_check"]["direction"]
                    .as_str()
                    .unwrap_or("unknown")
            );
            println!("- safe_to_sign: `false`");
        }
    }
    Ok(())
}

fn exit_for(value: &Value) -> u8 {
    match value["verdict"].as_str() {
        Some("STRUCTURAL_PASS_NOT_LEGAL_APPROVAL") => EXIT_PASS,
        Some("WATCH") => EXIT_WATCH,
        _ => EXIT_PASS,
    }
}

fn eq_norm(left: &str, right: &str) -> bool {
    normalize(left) == normalize(right)
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn value_or(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}
