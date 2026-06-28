use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ChatCoreDomainPackSpec {
    pub domain_id: String,
    #[serde(default)]
    pub route_prefixes: Vec<String>,
    #[serde(default)]
    pub routes: Vec<String>,
    #[serde(default)]
    pub negative_routes: Vec<String>,
    #[serde(default)]
    pub slots: Vec<String>,
    #[serde(default)]
    pub intent_anchors: Vec<String>,
    #[serde(default)]
    pub packet_profiles: Vec<String>,
    #[serde(default)]
    pub overlay_ids: Vec<String>,
    #[serde(default)]
    pub action_scope: String,
    #[serde(default)]
    pub evidence_policy: Value,
    #[serde(default)]
    pub learning_policy: Value,
    #[serde(default)]
    pub safety_policy: Value,
    #[serde(default)]
    pub eval_suites: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct DomainPackArtifactDigest {
    pub domain_id: String,
    pub path: String,
    pub required: bool,
    pub present: bool,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ChatCoreDomainProposalSeed {
    pub domain_id: String,
    #[serde(default)]
    pub intent_anchors: Vec<String>,
    #[serde(default)]
    pub candidate_routes: Vec<String>,
}

#[derive(Deserialize)]
struct ChatCoreDomainProposalSeedFile {
    #[serde(default)]
    seeds: Vec<ChatCoreDomainProposalSeed>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) enum DomainSupportVerdict {
    Supported,
    Unsupported,
}

#[derive(Serialize, Clone, Debug)]
pub(crate) struct DomainSupportReport {
    pub verdict: DomainSupportVerdict,
    pub decision_state: String,
    pub domain_supported: bool,
    pub selected_domain_pack: Option<String>,
    pub supported_domains: Vec<String>,
    pub suggested_domain: Option<String>,
    pub candidate_routes: Vec<String>,
    pub candidate_facts: Vec<DomainProposalFact>,
    pub matched_intent_anchors: Vec<String>,
    pub matched_route_prefixes: Vec<String>,
    pub selected_evidence_count_must_be_zero: bool,
    pub safe_to_learn_without_profile: bool,
    pub action_for_builder: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct DomainProposalFact {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub source: String,
    pub memory_state: String,
}

pub(crate) fn load_domain_packs(
    pack_paths: &[String],
    profile_dir: &Path,
) -> Result<(Vec<ChatCoreDomainPackSpec>, Vec<DomainPackArtifactDigest>)> {
    let mut packs = Vec::new();
    let mut digests = Vec::new();
    for raw_path in pack_paths {
        let path = resolve_pack_path(raw_path, profile_dir);
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let pack: ChatCoreDomainPackSpec =
            serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
        digests.push(DomainPackArtifactDigest {
            domain_id: pack.domain_id.clone(),
            path: path_string(&path),
            required: true,
            present: true,
            bytes: bytes.len() as u64,
            sha256: hash_bytes(&bytes),
        });
        packs.push(pack);
    }
    Ok((packs, digests))
}

pub(crate) fn domain_pack_digests(
    pack_paths: &[String],
    profile_dir: &Path,
) -> Result<Vec<DomainPackArtifactDigest>> {
    load_domain_packs(pack_paths, profile_dir).map(|(_, digests)| digests)
}

pub(crate) fn load_domain_proposal_seeds(
    registry_paths: &[String],
    profile_dir: &Path,
) -> Result<(
    Vec<ChatCoreDomainProposalSeed>,
    Vec<DomainPackArtifactDigest>,
)> {
    let mut seeds = Vec::new();
    let mut digests = Vec::new();
    for raw_path in registry_paths {
        let path = resolve_pack_path(raw_path, profile_dir);
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let value: Value =
            serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
        let file_seeds = if value.is_array() {
            serde_json::from_value::<Vec<ChatCoreDomainProposalSeed>>(value)
                .with_context(|| format!("parse proposal seed array {}", path.display()))?
        } else {
            serde_json::from_value::<ChatCoreDomainProposalSeedFile>(value)
                .with_context(|| format!("parse proposal seed file {}", path.display()))?
                .seeds
        };
        let domain_id = file_seeds
            .iter()
            .map(|seed| seed.domain_id.as_str())
            .collect::<Vec<_>>()
            .join(",");
        digests.push(DomainPackArtifactDigest {
            domain_id,
            path: path_string(&path),
            required: true,
            present: true,
            bytes: bytes.len() as u64,
            sha256: hash_bytes(&bytes),
        });
        seeds.extend(file_seeds);
    }
    Ok((seeds, digests))
}

pub(crate) fn domain_proposal_seed_digests(
    registry_paths: &[String],
    profile_dir: &Path,
) -> Result<Vec<DomainPackArtifactDigest>> {
    load_domain_proposal_seeds(registry_paths, profile_dir).map(|(_, digests)| digests)
}

pub(crate) fn legacy_domain_pack_from_routes<T>(
    domain_id: &str,
    routes: impl IntoIterator<Item = T>,
    negative_routes: impl IntoIterator<Item = T>,
    overlay_ids: impl IntoIterator<Item = T>,
    action_scope: &str,
) -> ChatCoreDomainPackSpec
where
    T: Into<String>,
{
    ChatCoreDomainPackSpec {
        domain_id: domain_id.to_string(),
        route_prefixes: vec!["linux.".to_string()],
        routes: routes.into_iter().map(Into::into).collect(),
        negative_routes: negative_routes.into_iter().map(Into::into).collect(),
        slots: vec![
            "route".to_string(),
            "subject".to_string(),
            "relation".to_string(),
            "object".to_string(),
            "polarity".to_string(),
            "evidence_kind".to_string(),
        ],
        intent_anchors: default_linux_anchors(domain_id),
        packet_profiles: Vec::new(),
        overlay_ids: overlay_ids.into_iter().map(Into::into).collect(),
        action_scope: action_scope.to_string(),
        evidence_policy: Value::Null,
        learning_policy: Value::Null,
        safety_policy: Value::Null,
        eval_suites: Vec::new(),
    }
}

pub(crate) fn select_domain_support(
    packs: &[ChatCoreDomainPackSpec],
    proposal_seeds: &[ChatCoreDomainProposalSeed],
    text: &str,
    query_routes: &BTreeSet<String>,
    source_label: Option<&str>,
) -> DomainSupportReport {
    let normalized = normalize_text(text);
    let supported_domains = packs
        .iter()
        .map(|pack| pack.domain_id.clone())
        .collect::<Vec<_>>();
    let mut best: Option<(i32, &ChatCoreDomainPackSpec, Vec<String>, Vec<String>)> = None;
    for pack in packs {
        let matched_anchors = pack
            .intent_anchors
            .iter()
            .filter(|anchor| normalized.contains(&normalize_text(anchor)))
            .cloned()
            .collect::<Vec<_>>();
        let matched_prefixes = query_routes
            .iter()
            .filter(|route| {
                pack.route_prefixes
                    .iter()
                    .any(|prefix| route.starts_with(prefix))
            })
            .cloned()
            .collect::<Vec<_>>();
        let exact_route_hits = query_routes
            .iter()
            .filter(|route| {
                pack.routes.iter().any(|candidate| candidate == *route)
                    || pack
                        .negative_routes
                        .iter()
                        .any(|candidate| candidate == *route)
            })
            .count();
        let score = (matched_anchors.len() as i32 * 8)
            + (exact_route_hits as i32 * 4)
            + (matched_prefixes.len() as i32);
        if score > best.as_ref().map(|(score, _, _, _)| *score).unwrap_or(0) {
            best = Some((score, pack, matched_anchors, matched_prefixes));
        }
    }
    if let Some((score, pack, anchors, prefixes)) = best {
        if score >= 8 && (!anchors.is_empty() || query_routes.is_empty()) {
            return DomainSupportReport {
                verdict: DomainSupportVerdict::Supported,
                decision_state: "DOMAIN_SUPPORTED".to_string(),
                domain_supported: true,
                selected_domain_pack: Some(pack.domain_id.clone()),
                supported_domains,
                suggested_domain: None,
                candidate_routes: Vec::new(),
                candidate_facts: Vec::new(),
                matched_intent_anchors: anchors,
                matched_route_prefixes: prefixes,
                selected_evidence_count_must_be_zero: false,
                safe_to_learn_without_profile: true,
                action_for_builder: "USE_CONNECTED_DOMAIN_PACK".to_string(),
            };
        }
    }
    let proposal_seed = select_domain_proposal_seed(text, source_label, proposal_seeds);
    let suggested_domain = proposal_seed.as_ref().map(|seed| seed.domain_id.clone());
    let candidate_routes = proposal_seed
        .map(|seed| seed.candidate_routes.clone())
        .unwrap_or_default();
    DomainSupportReport {
        verdict: DomainSupportVerdict::Unsupported,
        decision_state: "DOMAIN_UNSUPPORTED".to_string(),
        domain_supported: false,
        selected_domain_pack: None,
        supported_domains,
        suggested_domain,
        candidate_routes: candidate_routes.clone(),
        candidate_facts: candidate_facts_for_proposal(text, source_label, &candidate_routes),
        matched_intent_anchors: Vec::new(),
        matched_route_prefixes: Vec::new(),
        selected_evidence_count_must_be_zero: true,
        safe_to_learn_without_profile: false,
        action_for_builder: "CREATE_DOMAIN_PROFILE_OR_APPROVE_DOMAIN".to_string(),
    }
}

pub(crate) fn route_supported_by_domain_packs(
    packs: &[ChatCoreDomainPackSpec],
    route: &str,
    requested_domain: Option<&str>,
) -> bool {
    packs.iter().any(|pack| {
        requested_domain
            .map(|domain_id| domain_id == pack.domain_id)
            .unwrap_or(true)
            && (pack.routes.iter().any(|candidate| candidate == route)
                || pack
                    .negative_routes
                    .iter()
                    .any(|candidate| candidate == route))
    })
}

pub(crate) fn candidate_facts_for_proposal(
    text: &str,
    source_label: Option<&str>,
    candidate_routes: &[String],
) -> Vec<DomainProposalFact> {
    candidate_routes
        .first()
        .map(|route| DomainProposalFact {
            subject: compact_subject_from_text(text),
            relation: "candidate_status".to_string(),
            object: "not_promoted_until_domain_profile_exists".to_string(),
            source: source_label.unwrap_or("prompt").to_string(),
            memory_state: format!("candidate_only_not_authority:{route}"),
        })
        .into_iter()
        .collect()
}

pub(crate) fn select_domain_proposal_seed<'a>(
    text: &str,
    source_label: Option<&str>,
    seeds: &'a [ChatCoreDomainProposalSeed],
) -> Option<&'a ChatCoreDomainProposalSeed> {
    let haystack = normalize_text(&format!("{} {}", text, source_label.unwrap_or("")));
    seeds
        .iter()
        .filter_map(|seed| {
            let score = seed
                .intent_anchors
                .iter()
                .filter(|anchor| haystack.contains(&normalize_text(anchor)))
                .count();
            (score > 0).then_some((score, seed))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, seed)| seed)
}

fn compact_subject_from_text(text: &str) -> String {
    text.split_whitespace()
        .find(|part| part.contains('_') || part.contains('-') || part.contains('.'))
        .unwrap_or("unsupported_domain_query")
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.')
        .to_string()
}

fn default_linux_anchors(domain_id: &str) -> Vec<String> {
    match domain_id {
        "packages" => vec!["package", "command", "binary", "apt", "dpkg"],
        "systemd" => vec!["systemd", "service", "unit", "systemctl"],
        "dns" => vec!["dns", "resolver", "resolvectl", "route"],
        "exposure" => vec!["socket", "firewall", "listener", "nftables", "iptables"],
        "vpn" => vec![
            "vpn",
            "wireguard",
            "wg-quick",
            "networkmanager",
            "tunnel",
            "private key",
        ],
        _ => vec!["linux"],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn resolve_pack_path(raw_path: &str, profile_dir: &Path) -> PathBuf {
    let path = PathBuf::from(raw_path);
    if path.is_absolute() {
        path
    } else {
        profile_dir.join(path)
    }
}

fn normalize_text(text: &str) -> String {
    text.to_ascii_lowercase()
        .replace(['_', '-', '/', ':'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_external_physics_filename_does_not_support_linux() {
        let pack = ChatCoreDomainPackSpec {
            domain_id: "linux".to_string(),
            route_prefixes: vec!["linux.".to_string()],
            routes: vec!["linux.boundary.socket".to_string()],
            negative_routes: Vec::new(),
            slots: Vec::new(),
            intent_anchors: vec!["socket".to_string(), "firewall".to_string()],
            packet_profiles: Vec::new(),
            overlay_ids: Vec::new(),
            action_scope: String::new(),
            evidence_policy: Value::Null,
            learning_policy: Value::Null,
            safety_policy: Value::Null,
            eval_suites: Vec::new(),
        };
        let routes = BTreeSet::from(["linux.boundary.socket".to_string()]);
        let seed = ChatCoreDomainProposalSeed {
            domain_id: "physics_material_layer".to_string(),
            intent_anchors: vec!["earth surface".to_string(), "external station".to_string()],
            candidate_routes: vec!["physics.material_layer.status".to_string()],
        };
        let report = select_domain_support(
            &[pack],
            &[seed],
            "what does current-earth-surface-mode-gate-v2-external-station-check.md say?",
            &routes,
            Some("current-earth-surface-mode-gate-v2-external-station-check.md"),
        );
        assert_eq!(report.verdict, DomainSupportVerdict::Unsupported);
        assert!(!report.domain_supported);
        assert!(report.selected_evidence_count_must_be_zero);
        assert_eq!(
            report.suggested_domain.as_deref(),
            Some("physics_material_layer")
        );
    }
}
