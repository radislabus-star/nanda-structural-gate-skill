use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{NegativeShortcut, Packet, PositiveShortcut, Triad};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(super) struct AliasRule {
    #[serde(default)]
    pub(super) canonical: String,
    #[serde(default)]
    pub(super) variants: Vec<String>,
    #[serde(default = "default_alias_confidence")]
    pub(super) confidence: f64,
    #[serde(default)]
    pub(super) scope: String,
}

fn default_alias_confidence() -> f64 {
    1.0
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(super) struct CanonicalizationReport {
    pub(super) enabled: bool,
    pub(super) applied_count: usize,
    pub(super) conflict_count: usize,
    pub(super) watch_count: usize,
    #[serde(default)]
    pub(super) applied: Vec<CanonicalizationApplied>,
    #[serde(default)]
    pub(super) conflicts: Vec<String>,
    #[serde(default)]
    pub(super) warnings: Vec<String>,
}

impl CanonicalizationReport {
    pub(super) fn is_empty(&self) -> bool {
        !self.enabled
            && self.applied_count == 0
            && self.conflict_count == 0
            && self.watch_count == 0
            && self.applied.is_empty()
            && self.conflicts.is_empty()
            && self.warnings.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct CanonicalizationApplied {
    pub(super) triad_id: String,
    pub(super) field: String,
    pub(super) from: String,
    pub(super) to: String,
    pub(super) scope: String,
}

pub(super) fn canonicalize_packet(mut packet: Packet) -> Packet {
    let mut report = CanonicalizationReport {
        enabled: !packet.aliases.is_empty(),
        ..CanonicalizationReport::default()
    };
    if packet.aliases.is_empty() {
        packet.canonicalization = report;
        return packet;
    }

    let alias_map = build_alias_map(&packet.aliases, &mut report);
    canonicalize_triads(&mut packet.triads, &alias_map, &mut report);
    canonicalize_triads(&mut packet.candidate_triads, &alias_map, &mut report);
    canonicalize_shortcuts(&mut packet.negative_shortcuts, &alias_map, &mut report);
    canonicalize_positive_shortcuts(&mut packet.positive_shortcuts, &alias_map, &mut report);
    report.applied_count = report.applied.len();
    report.conflict_count = report.conflicts.len();
    report.watch_count = report.warnings.len() + report.conflicts.len();
    packet.canonicalization = report;
    packet
}

fn build_alias_map(
    rules: &[AliasRule],
    report: &mut CanonicalizationReport,
) -> BTreeMap<String, (String, String)> {
    let mut map = BTreeMap::<String, (String, String)>::new();
    for (idx, rule) in rules.iter().enumerate() {
        let canonical = rule.canonical.trim();
        if canonical.is_empty() {
            report
                .warnings
                .push(format!("alias rule {} has empty canonical", idx + 1));
            continue;
        }
        if rule.confidence < 0.85 {
            report.warnings.push(format!(
                "alias rule {} for {canonical} has low confidence {:.2}; variants were not applied",
                idx + 1,
                rule.confidence
            ));
            continue;
        }
        let scope = if rule.scope.trim().is_empty() {
            "global".to_string()
        } else {
            rule.scope.trim().to_string()
        };
        for raw in rule
            .variants
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(canonical))
        {
            let key = alias_key(raw);
            if key.is_empty() {
                continue;
            }
            if let Some((existing, existing_scope)) = map.get(&key) {
                if alias_key(existing) != alias_key(canonical) {
                    report.conflicts.push(format!(
                        "alias variant {raw} maps to both {existing} ({existing_scope}) and {canonical} ({scope})"
                    ));
                    continue;
                }
            }
            map.insert(key, (canonical.to_string(), scope.clone()));
        }
    }
    map
}

fn canonicalize_triads(
    triads: &mut [Triad],
    alias_map: &BTreeMap<String, (String, String)>,
    report: &mut CanonicalizationReport,
) {
    for triad in triads {
        canonicalize_plain_field(&mut triad.subject, &triad.id, "subject", alias_map, report);
        canonicalize_plain_field(&mut triad.object, &triad.id, "object", alias_map, report);
        canonicalize_plain_field(&mut triad.route, &triad.id, "route", alias_map, report);
        canonicalize_plain_field(&mut triad.group, &triad.id, "group", alias_map, report);
    }
}

fn canonicalize_shortcuts(
    shortcuts: &mut [NegativeShortcut],
    alias_map: &BTreeMap<String, (String, String)>,
    report: &mut CanonicalizationReport,
) {
    for item in shortcuts {
        canonicalize_plain_field(
            &mut item.suppress_peak,
            &item.id,
            "negative_shortcuts.suppress_peak",
            alias_map,
            report,
        );
        canonicalize_plain_field(
            &mut item.suppress_route,
            &item.id,
            "negative_shortcuts.suppress_route",
            alias_map,
            report,
        );
        canonicalize_plain_field(
            &mut item.suppress_group,
            &item.id,
            "negative_shortcuts.suppress_group",
            alias_map,
            report,
        );
        canonicalize_plain_field(
            &mut item.prefer_peak,
            &item.id,
            "negative_shortcuts.prefer_peak",
            alias_map,
            report,
        );
        canonicalize_plain_field(
            &mut item.prefer_route,
            &item.id,
            "negative_shortcuts.prefer_route",
            alias_map,
            report,
        );
        canonicalize_plain_field(
            &mut item.prefer_group,
            &item.id,
            "negative_shortcuts.prefer_group",
            alias_map,
            report,
        );
    }
}

fn canonicalize_positive_shortcuts(
    shortcuts: &mut [PositiveShortcut],
    alias_map: &BTreeMap<String, (String, String)>,
    report: &mut CanonicalizationReport,
) {
    for item in shortcuts {
        canonicalize_plain_field(
            &mut item.reinforce_peak,
            &item.id,
            "positive_shortcuts.reinforce_peak",
            alias_map,
            report,
        );
        canonicalize_plain_field(
            &mut item.reinforce_route,
            &item.id,
            "positive_shortcuts.reinforce_route",
            alias_map,
            report,
        );
        canonicalize_plain_field(
            &mut item.reinforce_group,
            &item.id,
            "positive_shortcuts.reinforce_group",
            alias_map,
            report,
        );
    }
}

fn canonicalize_plain_field(
    value: &mut String,
    triad_id: &str,
    field: &str,
    alias_map: &BTreeMap<String, (String, String)>,
    report: &mut CanonicalizationReport,
) {
    let key = alias_key(value);
    if key.is_empty() {
        return;
    }
    if let Some((canonical, scope)) = alias_map.get(&key) {
        if alias_key(canonical) != key {
            let from = value.clone();
            *value = canonical.clone();
            report.applied.push(CanonicalizationApplied {
                triad_id: triad_id.to_string(),
                field: field.to_string(),
                from,
                to: canonical.clone(),
                scope: scope.clone(),
            });
        }
    }
}

pub(super) fn inherit_aliases_if_needed(query_packet: &mut Packet, memory_packet: &Packet) {
    if query_packet.aliases.is_empty() && !memory_packet.aliases.is_empty() {
        query_packet.aliases = memory_packet.aliases.clone();
        let recanonicalized = canonicalize_packet(query_packet.clone());
        *query_packet = recanonicalized;
    }
}

pub(super) fn print_text(out: &Value) {
    let report = &out["canonicalization"];
    println!("mode: canonical-aliases");
    println!("enabled: {}", report["enabled"].as_bool().unwrap_or(false));
    println!("applied: {}", report["applied_count"].as_u64().unwrap_or(0));
    println!(
        "conflicts: {}",
        report["conflict_count"].as_u64().unwrap_or(0)
    );
    println!("warnings: {}", report["watch_count"].as_u64().unwrap_or(0));
    if let Some(items) = report["applied"].as_array() {
        for item in items {
            println!(
                "  - {} {}: {} -> {}",
                item["triad_id"].as_str().unwrap_or(""),
                item["field"].as_str().unwrap_or(""),
                item["from"].as_str().unwrap_or(""),
                item["to"].as_str().unwrap_or("")
            );
        }
    }
    if let Some(items) = report["conflicts"].as_array() {
        for item in items {
            println!("conflict: {}", item.as_str().unwrap_or(""));
        }
    }
    if let Some(items) = report["warnings"].as_array() {
        for item in items {
            println!("warning: {}", item.as_str().unwrap_or(""));
        }
    }
}

pub(super) fn print_md(out: &Value) {
    let report = &out["canonicalization"];
    println!("# NANDA Canonical Aliases\n");
    println!("- enabled: `{}`", report["enabled"]);
    println!("- applied: `{}`", report["applied_count"]);
    println!("- conflicts: `{}`", report["conflict_count"]);
    println!("- warnings: `{}`", report["watch_count"]);
}

fn alias_key(value: &str) -> String {
    value.trim().to_lowercase()
}
