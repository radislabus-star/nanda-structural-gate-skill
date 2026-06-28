use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Serialize;

use super::domain_pack::{
    candidate_facts_for_proposal, load_domain_proposal_seeds, select_domain_proposal_seed,
    DomainProposalFact,
};

const DEFAULT_PROPOSAL_REGISTRY: &str = "examples/domain-packs/proposal-seeds.json";

#[derive(Clone)]
pub(crate) struct ChatCoreDomainProposalConfig {
    pub text: String,
    pub context_file: Option<PathBuf>,
    pub proposal_registries: Vec<PathBuf>,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub(crate) struct ChatCoreDomainProposalReport {
    pub mode: &'static str,
    pub verdict: &'static str,
    pub decision_state: &'static str,
    pub safe_to_answer: bool,
    pub safe_to_learn_without_profile: bool,
    pub selected_evidence_count: usize,
    pub readout_source: &'static str,
    pub text: String,
    pub context_file: Option<String>,
    pub proposal_registries: Vec<String>,
    pub suggested_domain_id: Option<String>,
    pub candidate_routes: Vec<String>,
    pub candidate_facts: Vec<DomainProposalFact>,
    pub candidate_facts_are_memory: bool,
    pub cache_mutated: bool,
    pub overlay_written: bool,
    pub action_for_builder: &'static str,
    pub claim_boundary: ChatCoreDomainProposalBoundary,
}

#[derive(Serialize, Clone)]
pub(crate) struct ChatCoreDomainProposalBoundary {
    pub domain_profile_required: bool,
    pub proposal_is_read_only: bool,
    pub candidate_facts_have_no_authority: bool,
    pub general_llm_ready: bool,
    pub global_nonlinear_memory_proven: bool,
}

pub(crate) fn build_domain_proposal_report(
    config: ChatCoreDomainProposalConfig,
) -> Result<ChatCoreDomainProposalReport> {
    let context_text = if let Some(path) = &config.context_file {
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
    } else {
        String::new()
    };
    let source_label = config
        .context_file
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());
    let combined = if context_text.is_empty() {
        config.text.clone()
    } else {
        format!("{}\n{}", config.text, context_text)
    };
    let registry_paths = proposal_registry_paths(config.proposal_registries);
    let registry_strings = registry_paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let (proposal_seeds, _) =
        load_domain_proposal_seeds(&registry_strings, std::path::Path::new("."))?;
    let proposal_seed =
        select_domain_proposal_seed(&combined, source_label.as_deref(), &proposal_seeds);
    let suggested_domain_id = proposal_seed.map(|seed| seed.domain_id.clone());
    let candidate_routes = proposal_seed
        .map(|seed| seed.candidate_routes.clone())
        .unwrap_or_default();
    let candidate_facts =
        candidate_facts_for_proposal(&config.text, source_label.as_deref(), &candidate_routes);
    let report = ChatCoreDomainProposalReport {
        mode: "llmwave-big-chat-core-domain-proposal",
        verdict: "DOMAIN_PROFILE_REQUIRED",
        decision_state: "DOMAIN_UNSUPPORTED",
        safe_to_answer: false,
        safe_to_learn_without_profile: false,
        selected_evidence_count: 0,
        readout_source: "none_domain_unsupported",
        text: config.text,
        context_file: source_label,
        proposal_registries: registry_strings,
        suggested_domain_id,
        candidate_routes,
        candidate_facts,
        candidate_facts_are_memory: false,
        cache_mutated: false,
        overlay_written: false,
        action_for_builder: "CREATE_DOMAIN_PROFILE_OR_APPROVE_DOMAIN",
        claim_boundary: ChatCoreDomainProposalBoundary {
            domain_profile_required: true,
            proposal_is_read_only: true,
            candidate_facts_have_no_authority: true,
            general_llm_ready: false,
            global_nonlinear_memory_proven: false,
        },
    };
    if let Some(path) = config.out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&path, serde_json::to_vec_pretty(&report)?)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(report)
}

fn proposal_registry_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    if !paths.is_empty() {
        return paths;
    }
    let default = PathBuf::from(DEFAULT_PROPOSAL_REGISTRY);
    if default.exists() {
        vec![default]
    } else {
        Vec::new()
    }
}
