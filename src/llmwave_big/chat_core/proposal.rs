use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::domain_pack::{
    candidate_facts_for_proposal, load_domain_packs, load_domain_proposal_seeds,
    select_domain_proposal_seed, ChatCoreDomainPackSpec, DomainProposalFact,
};

const DEFAULT_PROPOSAL_REGISTRY: &str = "examples/domain-packs/proposal-seeds.json";
const DEFAULT_PROFILE: &str = "examples/linux-chat-core.profile.json";

#[derive(Clone)]
pub(crate) struct ChatCoreDomainProposalConfig {
    pub text: String,
    pub profile: Option<PathBuf>,
    pub memory_root: Option<PathBuf>,
    pub context_file: Option<PathBuf>,
    pub proposal_registries: Vec<PathBuf>,
    pub out: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ChatCoreDomainProposalReport {
    pub mode: String,
    pub verdict: String,
    pub decision_state: String,
    pub safe_to_answer: bool,
    pub safe_to_learn_without_profile: bool,
    pub selected_evidence_count: usize,
    pub readout_source: String,
    pub text: String,
    pub profile: Option<String>,
    pub memory_root: Option<String>,
    pub context_file: Option<String>,
    pub proposal_registries: Vec<String>,
    pub connected_domains: Vec<String>,
    pub connected_domain_pack_count: usize,
    pub suggested_domain_id: Option<String>,
    pub candidate_routes: Vec<String>,
    pub candidate_facts: Vec<DomainProposalFact>,
    pub domain_pack_draft: Option<DomainPackDraft>,
    pub draft_gate: Option<DomainPackDraftGate>,
    pub candidate_facts_are_memory: bool,
    pub cache_mutated: bool,
    pub overlay_written: bool,
    pub action_for_builder: String,
    pub claim_boundary: ChatCoreDomainProposalBoundary,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ChatCoreDomainProposalBoundary {
    pub domain_profile_required: bool,
    pub proposal_is_read_only: bool,
    pub candidate_facts_have_no_authority: bool,
    pub automatic_structure_allowed: bool,
    pub automatic_authority_allowed: bool,
    pub unknown_domain_can_enter_hot_as_answerable_memory: bool,
    pub general_llm_ready: bool,
    pub global_nonlinear_memory_proven: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DomainPackDraft {
    pub domain_id: String,
    pub domain_family: String,
    pub domain_scope: String,
    pub parent_domain: String,
    pub maturity: String,
    pub authority_level: String,
    pub authority_rights: DomainPackAuthorityRights,
    pub routes: Vec<DomainPackDraftRoute>,
    pub claim_boundary: Vec<String>,
    pub evidence_required: Vec<String>,
    pub anti_wave: Vec<DomainPackAntiWaveSeed>,
    pub minimal_eval: DomainPackMinimalEval,
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DomainPackAuthorityRights {
    pub proposal_only: bool,
    pub answer_allowed: bool,
    pub learn_allowed: bool,
    pub cache_build_allowed: bool,
    pub overlay_write_allowed: bool,
    pub cache_mutation_allowed: bool,
    pub quarantine_write_allowed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DomainPackDraftRoute {
    pub route_id: String,
    pub route_family: String,
    pub relation_type: String,
    pub subject_role: String,
    pub object_role: String,
    pub allowed_subject_types: Vec<String>,
    pub forbidden_subject_types: Vec<String>,
    pub allowed_object_types: Vec<String>,
    pub evidence_kinds: Vec<String>,
    pub allowed_questions: Vec<String>,
    pub forbidden_questions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DomainPackAntiWaveSeed {
    pub anti_wave_id: String,
    pub blocks: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DomainPackMinimalEval {
    pub positive_cases: Vec<String>,
    pub negative_cases: Vec<String>,
    pub role_swap_cases: Vec<String>,
    pub route_collision_cases: Vec<String>,
    pub heldout_or_control_cases: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DomainPackDraftGate {
    pub verdict: String,
    pub structure_complete: bool,
    pub authority_granted: bool,
    pub promotion_allowed: bool,
    pub answer_allowed: bool,
    pub cache_mutation_allowed: bool,
    pub overlay_write_allowed: bool,
    pub route_collision: DomainPackGateCheck,
    pub role_collision: DomainPackGateCheck,
    pub relation_genericity: DomainPackGateCheck,
    pub foreign_pull: DomainPackGateCheck,
    pub boundary_economics: DomainPackGateCheck,
    pub claim_boundary: DomainPackGateCheck,
    pub evidence_contract: DomainPackGateCheck,
    pub anti_wave_seed: DomainPackGateCheck,
    pub minimal_eval: DomainPackGateCheck,
    pub required_next_action: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DomainPackGateCheck {
    pub status: String,
    pub reasons: Vec<String>,
    pub nearest_existing_routes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ChatCoreDomainGateReport {
    pub mode: String,
    pub verdict: String,
    pub decision_state: String,
    pub domain_id: Option<String>,
    pub maturity: Option<String>,
    pub authority_rights: Option<DomainPackAuthorityRights>,
    pub draft_gate: Option<DomainPackDraftGate>,
    pub safe_to_answer: bool,
    pub safe_to_learn_without_profile: bool,
    pub cache_mutated: bool,
    pub overlay_written: bool,
    pub claim_boundary: ChatCoreDomainProposalBoundary,
}

#[derive(Deserialize)]
struct ProposalProfileFile {
    #[serde(default)]
    domain_packs: Vec<String>,
    #[serde(default)]
    domain_proposals: Vec<String>,
}

pub(crate) fn build_domain_proposal_report(
    config: ChatCoreDomainProposalConfig,
) -> Result<ChatCoreDomainProposalReport> {
    build_domain_proposal_report_with_mode(config, "llmwave-big-chat-core-domain-proposal")
}

pub(crate) fn build_domain_builder_report(
    config: ChatCoreDomainProposalConfig,
) -> Result<ChatCoreDomainProposalReport> {
    build_domain_proposal_report_with_mode(config, "llmwave-big-chat-core-domain-build")
}

pub(crate) fn build_domain_gate_report(
    config: ChatCoreDomainProposalConfig,
    draft: Option<PathBuf>,
) -> Result<ChatCoreDomainGateReport> {
    let proposal = if let Some(path) = draft {
        serde_json::from_slice::<ChatCoreDomainProposalReport>(
            &fs::read(&path).with_context(|| format!("read {}", path.display()))?,
        )
        .with_context(|| format!("parse {}", path.display()))?
    } else {
        build_domain_builder_report(config)?
    };
    let draft = proposal.domain_pack_draft;
    let gate = proposal.draft_gate;
    Ok(ChatCoreDomainGateReport {
        mode: "llmwave-big-chat-core-domain-gate".to_string(),
        verdict: gate
            .as_ref()
            .map(|gate| gate.verdict.clone())
            .unwrap_or_else(|| "DOMAIN_PACK_DRAFT_MISSING".to_string()),
        decision_state: "DOMAIN_CANDIDATE_REVIEW".to_string(),
        domain_id: draft.as_ref().map(|draft| draft.domain_id.clone()),
        maturity: draft.as_ref().map(|draft| draft.maturity.clone()),
        authority_rights: draft.as_ref().map(|draft| draft.authority_rights.clone()),
        draft_gate: gate,
        safe_to_answer: false,
        safe_to_learn_without_profile: false,
        cache_mutated: false,
        overlay_written: false,
        claim_boundary: proposal.claim_boundary,
    })
}

fn build_domain_proposal_report_with_mode(
    config: ChatCoreDomainProposalConfig,
    mode: &'static str,
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
    let profile_context =
        proposal_profile_context(config.profile.as_ref(), config.memory_root.as_ref())?;
    let registry_paths = proposal_registry_paths(
        config.proposal_registries,
        profile_context
            .as_ref()
            .map(|context| context.proposal_paths.as_slice())
            .unwrap_or(&[]),
    );
    let registry_strings = registry_paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let (proposal_seeds, _) =
        load_domain_proposal_seeds(&registry_strings, std::path::Path::new("."))?;
    let connected_domain_packs = profile_context
        .as_ref()
        .map(|context| context.domain_packs.clone())
        .unwrap_or_default();
    let connected_domains = connected_domain_packs
        .iter()
        .map(|pack| pack.domain_id.clone())
        .collect::<Vec<_>>();
    let proposal_seed =
        select_domain_proposal_seed(&combined, source_label.as_deref(), &proposal_seeds);
    let suggested_domain_id = proposal_seed.map(|seed| seed.domain_id.clone());
    let candidate_routes = proposal_seed
        .map(|seed| seed.candidate_routes.clone())
        .unwrap_or_default();
    let candidate_facts =
        candidate_facts_for_proposal(&config.text, source_label.as_deref(), &candidate_routes);
    let domain_pack_draft = suggested_domain_id.as_ref().map(|domain_id| {
        build_domain_pack_draft(domain_id, &candidate_routes, source_label.as_deref())
    });
    let draft_gate = domain_pack_draft
        .as_ref()
        .map(|draft| gate_domain_pack_draft(draft, &connected_domain_packs));
    let report = ChatCoreDomainProposalReport {
        mode: mode.to_string(),
        verdict: "DOMAIN_PROFILE_REQUIRED".to_string(),
        decision_state: "DOMAIN_UNSUPPORTED".to_string(),
        safe_to_answer: false,
        safe_to_learn_without_profile: false,
        selected_evidence_count: 0,
        readout_source: "none_domain_unsupported".to_string(),
        text: config.text,
        profile: profile_context
            .as_ref()
            .map(|context| context.profile_path.to_string_lossy().into_owned()),
        memory_root: config
            .memory_root
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        context_file: source_label,
        proposal_registries: registry_strings,
        connected_domains,
        connected_domain_pack_count: connected_domain_packs.len(),
        suggested_domain_id,
        candidate_routes,
        candidate_facts,
        domain_pack_draft,
        draft_gate,
        candidate_facts_are_memory: false,
        cache_mutated: false,
        overlay_written: false,
        action_for_builder: "REVIEW_DOMAIN_PACK_DRAFT_AND_GATE_BEFORE_APPROVAL".to_string(),
        claim_boundary: ChatCoreDomainProposalBoundary {
            domain_profile_required: true,
            proposal_is_read_only: true,
            candidate_facts_have_no_authority: true,
            automatic_structure_allowed: true,
            automatic_authority_allowed: false,
            unknown_domain_can_enter_hot_as_answerable_memory: false,
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

#[derive(Clone)]
struct ProposalProfileContext {
    profile_path: PathBuf,
    proposal_paths: Vec<PathBuf>,
    domain_packs: Vec<ChatCoreDomainPackSpec>,
}

fn proposal_profile_context(
    profile: Option<&PathBuf>,
    memory_root: Option<&PathBuf>,
) -> Result<Option<ProposalProfileContext>> {
    let profile_path = profile
        .cloned()
        .or_else(|| {
            memory_root
                .map(|root| root.join("chat-core.profile.json"))
                .filter(|path| path.exists())
        })
        .or_else(|| {
            let default = PathBuf::from(DEFAULT_PROFILE);
            default.exists().then_some(default)
        });
    let Some(profile_path) = profile_path else {
        return Ok(None);
    };
    let profile_file: ProposalProfileFile = serde_json::from_slice(
        &fs::read(&profile_path).with_context(|| format!("read {}", profile_path.display()))?,
    )
    .with_context(|| format!("parse {}", profile_path.display()))?;
    let profile_dir = profile_path.parent().unwrap_or_else(|| Path::new("."));
    let proposal_paths = profile_file
        .domain_proposals
        .iter()
        .map(|path| resolve_relative(profile_dir, path))
        .collect::<Vec<_>>();
    let domain_pack_strings = profile_file
        .domain_packs
        .iter()
        .map(|path| {
            resolve_relative(profile_dir, path)
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    let domain_packs = if domain_pack_strings.is_empty() {
        Vec::new()
    } else {
        load_domain_packs(&domain_pack_strings, Path::new("."))?.0
    };
    Ok(Some(ProposalProfileContext {
        profile_path,
        proposal_paths,
        domain_packs,
    }))
}

fn proposal_registry_paths(paths: Vec<PathBuf>, profile_paths: &[PathBuf]) -> Vec<PathBuf> {
    if !paths.is_empty() {
        return paths;
    }
    if !profile_paths.is_empty() {
        return profile_paths.to_vec();
    }
    let default = PathBuf::from(DEFAULT_PROPOSAL_REGISTRY);
    if default.exists() {
        vec![default]
    } else {
        Vec::new()
    }
}

fn resolve_relative(base: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn build_domain_pack_draft(
    domain_id: &str,
    candidate_routes: &[String],
    source_label: Option<&str>,
) -> DomainPackDraft {
    let domain_family = domain_id
        .split(['_', '.'])
        .next()
        .unwrap_or("unknown")
        .to_string();
    DomainPackDraft {
        domain_id: domain_id.to_string(),
        domain_family: domain_family.clone(),
        domain_scope: if domain_id == "physics_material_layer" {
            "material_layer_operator".to_string()
        } else {
            format!("{domain_id}_scope")
        },
        parent_domain: domain_family,
        maturity: "candidate".to_string(),
        authority_level: "proposal_only".to_string(),
        authority_rights: DomainPackAuthorityRights {
            proposal_only: true,
            answer_allowed: false,
            learn_allowed: false,
            cache_build_allowed: false,
            overlay_write_allowed: false,
            cache_mutation_allowed: false,
            quarantine_write_allowed: false,
        },
        routes: candidate_routes
            .iter()
            .map(|route| draft_route(route))
            .collect(),
        claim_boundary: claim_boundaries(domain_id),
        evidence_required: evidence_required(domain_id),
        anti_wave: anti_wave_seeds(domain_id),
        minimal_eval: minimal_eval(domain_id),
        source: source_label.unwrap_or("prompt").to_string(),
    }
}

fn draft_route(route: &str) -> DomainPackDraftRoute {
    let relation_type = relation_type_for_route(route);
    DomainPackDraftRoute {
        route_id: route.to_string(),
        route_family: route_family_for_route(route),
        relation_type: relation_type.clone(),
        subject_role: subject_role_for_route(route),
        object_role: object_role_for_relation(&relation_type),
        allowed_subject_types: allowed_subject_types(route),
        forbidden_subject_types: vec![
            "business_deal".to_string(),
            "linux_service".to_string(),
            "contract_clause".to_string(),
        ],
        allowed_object_types: allowed_object_types(&relation_type),
        evidence_kinds: evidence_required("physics_material_layer"),
        allowed_questions: allowed_questions(route),
        forbidden_questions: vec![
            "is this final physical law?".to_string(),
            "does this prove all behavior?".to_string(),
            "can this answer another domain status?".to_string(),
        ],
    }
}

fn relation_type_for_route(route: &str) -> String {
    if route.ends_with(".dominant_band") {
        "spectral_band_classification".to_string()
    } else if route.ends_with(".mode_observation") {
        "mode_observation_binding".to_string()
    } else if route.ends_with(".residual_mode") {
        "residual_classification".to_string()
    } else if route.ends_with(".status") {
        "layer_mode_status".to_string()
    } else {
        "relation_candidate".to_string()
    }
}

fn route_family_for_route(route: &str) -> String {
    if route.contains("earth_surface") {
        "earth_surface_observation".to_string()
    } else if route.contains("water_layer") {
        "layer_trace".to_string()
    } else if route.contains("material_layer") {
        "material_layer".to_string()
    } else {
        route.split('.').take(2).collect::<Vec<_>>().join(".")
    }
}

fn subject_role_for_route(route: &str) -> String {
    if route.contains("earth_surface") {
        "station_window_or_surface_observation".to_string()
    } else if route.contains("water_layer") {
        "observed_layer_trace".to_string()
    } else {
        "layer_or_mode".to_string()
    }
}

fn object_role_for_relation(relation_type: &str) -> String {
    match relation_type {
        "spectral_band_classification" => "band_label",
        "mode_observation_binding" => "observed_mode_label",
        "residual_classification" => "residual_mode_label",
        "layer_mode_status" => "maturity_state",
        _ => "candidate_object",
    }
    .to_string()
}

fn allowed_subject_types(route: &str) -> Vec<String> {
    if route.starts_with("physics.") {
        vec![
            "material_layer".to_string(),
            "observed_mode".to_string(),
            "candidate_operator".to_string(),
            "station_window".to_string(),
        ]
    } else {
        vec!["domain_candidate_subject".to_string()]
    }
}

fn allowed_object_types(relation_type: &str) -> Vec<String> {
    match relation_type {
        "layer_mode_status" => vec!["candidate".to_string(), "provisional".to_string()],
        "spectral_band_classification" => vec!["dominant_band_label".to_string()],
        "mode_observation_binding" => vec!["mode_label".to_string()],
        "residual_classification" => vec!["residual_mode_label".to_string()],
        _ => vec!["candidate_object".to_string()],
    }
}

fn allowed_questions(route: &str) -> Vec<String> {
    if route.contains("status") {
        vec![
            "what candidate status does this artifact claim?".to_string(),
            "is this mode candidate still provisional?".to_string(),
        ]
    } else if route.contains("dominant_band") {
        vec!["what dominant band is claimed by this artifact?".to_string()]
    } else if route.contains("mode_observation") {
        vec!["what mode did this station-window show?".to_string()]
    } else {
        vec!["what residual mode is claimed by this artifact?".to_string()]
    }
}

fn claim_boundaries(domain_id: &str) -> Vec<String> {
    if domain_id == "physics_material_layer" {
        vec![
            "candidate_status != proven_physics".to_string(),
            "numeric_coverage != external_validation".to_string(),
            "active_mode != final_law".to_string(),
            "proxy_trace != full_causal_model".to_string(),
            "wind_repeat != wind_only_causality".to_string(),
        ]
    } else {
        vec![
            "candidate_domain != ready_domain".to_string(),
            "candidate_fact != answer_authority".to_string(),
        ]
    }
}

fn evidence_required(domain_id: &str) -> Vec<String> {
    if domain_id == "physics_material_layer" {
        vec![
            "source_ref".to_string(),
            "measurement_or_artifact_ref".to_string(),
            "eval_or_control_ref".to_string(),
            "local_project_artifact".to_string(),
            "numeric_table_or_trace".to_string(),
        ]
    } else {
        vec!["source_ref".to_string(), "eval_or_control_ref".to_string()]
    }
}

fn anti_wave_seeds(domain_id: &str) -> Vec<DomainPackAntiWaveSeed> {
    let mut seeds = vec![
        DomainPackAntiWaveSeed {
            anti_wave_id: "generic_status_relation".to_string(),
            blocks: "status words from unrelated domains leaking into this domain".to_string(),
        },
        DomainPackAntiWaveSeed {
            anti_wave_id: "cache_authority_jump".to_string(),
            blocks: "candidate facts entering hot cache as answerable memory".to_string(),
        },
    ];
    if domain_id == "physics_material_layer" {
        seeds.extend([
            DomainPackAntiWaveSeed {
                anti_wave_id: "validation_overclaim".to_string(),
                blocks: "candidate/repeat/numeric coverage being called proof".to_string(),
            },
            DomainPackAntiWaveSeed {
                anti_wave_id: "proxy_to_cause_jump".to_string(),
                blocks: "proxy trace being treated as full cause".to_string(),
            },
            DomainPackAntiWaveSeed {
                anti_wave_id: "route_by_token".to_string(),
                blocks: "domain creation from surface words instead of roles".to_string(),
            },
        ]);
    }
    seeds
}

fn minimal_eval(domain_id: &str) -> DomainPackMinimalEval {
    if domain_id == "physics_material_layer" {
        DomainPackMinimalEval {
            positive_cases: vec![
                "water layer tide band -> dominant_band".to_string(),
                "water layer residual -> residual_mode".to_string(),
                "wind_setup_compact_active -> candidate/provisional mode".to_string(),
                "material layer passport -> layer characteristics".to_string(),
            ],
            negative_cases: vec![
                "linux service status must not match material layer status".to_string(),
                "business deal status must not match material layer status".to_string(),
                "active station mode must not become proven law".to_string(),
                "numeric coverage must not become external validation".to_string(),
                "dominant band must not become full explanation".to_string(),
            ],
            role_swap_cases: vec![
                "mode label must not become observation source".to_string(),
                "proxy trace must not become causal model".to_string(),
            ],
            route_collision_cases: vec![
                "physics.material_layer.status != linux.service.status".to_string(),
                "physics.material_layer.status != business.deal_status".to_string(),
            ],
            heldout_or_control_cases: vec![
                "control shuffle keeps candidate route unsupported".to_string(),
                "heldout station-window does not become proof without evidence".to_string(),
            ],
        }
    } else {
        DomainPackMinimalEval {
            positive_cases: vec!["candidate route positive case required".to_string()],
            negative_cases: vec!["cross-domain collision negative case required".to_string()],
            role_swap_cases: vec!["role swap negative case required".to_string()],
            route_collision_cases: vec!["nearest route collision case required".to_string()],
            heldout_or_control_cases: vec!["heldout/control case required".to_string()],
        }
    }
}

fn gate_domain_pack_draft(
    draft: &DomainPackDraft,
    connected_packs: &[ChatCoreDomainPackSpec],
) -> DomainPackDraftGate {
    let existing_routes = connected_packs
        .iter()
        .flat_map(|pack| pack.routes.iter().chain(pack.negative_routes.iter()))
        .cloned()
        .collect::<Vec<_>>();
    let draft_routes = draft
        .routes
        .iter()
        .map(|route| route.route_id.clone())
        .collect::<Vec<_>>();
    let nearest = nearest_existing_routes(&draft_routes, &existing_routes);
    let exact_collision = draft_routes
        .iter()
        .any(|route| existing_routes.iter().any(|existing| existing == route));
    let relation_watch = draft
        .routes
        .iter()
        .filter(|route| {
            ["status", "active", "external", "mode"]
                .iter()
                .any(|generic| {
                    route.route_id.ends_with(&format!(".{generic}"))
                        || route.relation_type == *generic
                })
        })
        .map(|route| route.route_id.clone())
        .collect::<Vec<_>>();
    let route_collision = DomainPackGateCheck {
        status: if exact_collision { "VETO" } else { "PASS" }.to_string(),
        reasons: if exact_collision {
            vec!["exact_existing_route_collision".to_string()]
        } else {
            vec!["no_exact_route_collision".to_string()]
        },
        nearest_existing_routes: nearest.clone(),
    };
    let relation_genericity = DomainPackGateCheck {
        status: if relation_watch.is_empty() {
            "PASS"
        } else {
            "WATCH"
        }
        .to_string(),
        reasons: if relation_watch.is_empty() {
            vec!["relations_are_role_typed".to_string()]
        } else {
            vec![
                "generic_surface_relation_requires_role_typed_boundary".to_string(),
                relation_watch.join(","),
            ]
        },
        nearest_existing_routes: nearest.clone(),
    };
    let role_collision = DomainPackGateCheck {
        status: "PASS".to_string(),
        reasons: vec!["subject_and_object_roles_are_explicit".to_string()],
        nearest_existing_routes: nearest.clone(),
    };
    let evidence_contract = DomainPackGateCheck {
        status: if draft.evidence_required.is_empty() {
            "WATCH"
        } else {
            "PASS"
        }
        .to_string(),
        reasons: vec!["evidence_required_declared".to_string()],
        nearest_existing_routes: Vec::new(),
    };
    let claim_boundary = DomainPackGateCheck {
        status: if draft.claim_boundary.is_empty() {
            "WATCH"
        } else {
            "PASS"
        }
        .to_string(),
        reasons: vec!["claim_boundaries_declared".to_string()],
        nearest_existing_routes: Vec::new(),
    };
    let anti_wave_seed = DomainPackGateCheck {
        status: if draft.anti_wave.is_empty() {
            "WATCH"
        } else {
            "PASS"
        }
        .to_string(),
        reasons: vec!["anti_wave_seeds_declared".to_string()],
        nearest_existing_routes: Vec::new(),
    };
    let minimal_eval = DomainPackGateCheck {
        status: if draft.minimal_eval.positive_cases.is_empty()
            || draft.minimal_eval.negative_cases.is_empty()
        {
            "WATCH"
        } else {
            "PASS"
        }
        .to_string(),
        reasons: vec!["positive_and_negative_eval_cases_declared".to_string()],
        nearest_existing_routes: Vec::new(),
    };
    let foreign_pull = DomainPackGateCheck {
        status: "PASS".to_string(),
        reasons: vec!["candidate_has_no_cache_or_overlay_writes".to_string()],
        nearest_existing_routes: nearest.clone(),
    };
    let boundary_economics = DomainPackGateCheck {
        status: "WATCH".to_string(),
        reasons: vec!["candidate_domain_requires_builder_review_before_authority".to_string()],
        nearest_existing_routes: nearest,
    };
    let structure_complete = !draft.routes.is_empty()
        && !draft.claim_boundary.is_empty()
        && !draft.evidence_required.is_empty()
        && !draft.anti_wave.is_empty()
        && !draft.minimal_eval.positive_cases.is_empty()
        && !draft.minimal_eval.negative_cases.is_empty();
    let any_veto = route_collision.status == "VETO";
    let any_watch = [
        &relation_genericity,
        &evidence_contract,
        &claim_boundary,
        &anti_wave_seed,
        &minimal_eval,
        &boundary_economics,
    ]
    .iter()
    .any(|check| check.status == "WATCH");
    DomainPackDraftGate {
        verdict: if any_veto {
            "DOMAIN_PACK_DRAFT_VETO".to_string()
        } else if any_watch {
            "DOMAIN_PACK_DRAFT_WATCH_NO_AUTHORITY".to_string()
        } else {
            "DOMAIN_PACK_DRAFT_GATE_PASS_NO_AUTHORITY".to_string()
        },
        structure_complete,
        authority_granted: false,
        promotion_allowed: false,
        answer_allowed: false,
        cache_mutation_allowed: false,
        overlay_write_allowed: false,
        route_collision,
        role_collision,
        relation_genericity,
        foreign_pull,
        boundary_economics,
        claim_boundary,
        evidence_contract,
        anti_wave_seed,
        minimal_eval,
        required_next_action: "HUMAN_OR_BUILDER_APPROVAL_THEN_TRAIN_EVAL_BEFORE_READY".to_string(),
    }
}

fn nearest_existing_routes(draft_routes: &[String], existing_routes: &[String]) -> Vec<String> {
    let mut nearest = Vec::new();
    for route in draft_routes {
        let tail = route.rsplit('.').next().unwrap_or(route);
        for existing in existing_routes {
            let matches_route = existing.ends_with(tail)
                || existing
                    .split('.')
                    .any(|segment| route.split('.').any(|candidate| candidate == segment));
            if matches_route && !nearest.contains(existing) {
                nearest.push(existing.clone());
            }
            if nearest.len() >= 8 {
                return nearest;
            }
        }
    }
    nearest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_proposal_builds_candidate_draft_without_authority() {
        let report = build_domain_proposal_report(ChatCoreDomainProposalConfig {
            text: "what is wind_setup_compact_active in gravity-saturation-checks?".to_string(),
            profile: Some(PathBuf::from(DEFAULT_PROFILE)),
            memory_root: Some(PathBuf::from(".nanda/linux-active")),
            context_file: None,
            proposal_registries: Vec::new(),
            out: None,
        })
        .unwrap();

        assert_eq!(report.verdict, "DOMAIN_PROFILE_REQUIRED");
        assert_eq!(
            report.suggested_domain_id.as_deref(),
            Some("physics_material_layer")
        );
        assert!(!report.safe_to_answer);
        assert!(!report.cache_mutated);
        assert!(!report.overlay_written);
        assert_eq!(report.connected_domains, vec!["linux".to_string()]);
        let draft = report.domain_pack_draft.as_ref().unwrap();
        assert_eq!(draft.maturity, "candidate");
        assert_eq!(draft.authority_level, "proposal_only");
        assert!(draft.authority_rights.proposal_only);
        assert!(!draft.authority_rights.answer_allowed);
        assert!(!draft.authority_rights.learn_allowed);
        assert!(!draft.authority_rights.cache_build_allowed);
        assert!(!draft.authority_rights.overlay_write_allowed);
        assert!(!draft.authority_rights.cache_mutation_allowed);
        assert!(draft
            .claim_boundary
            .contains(&"candidate_status != proven_physics".to_string()));
        assert!(draft
            .anti_wave
            .iter()
            .any(|seed| seed.anti_wave_id == "cache_authority_jump"));
        let gate = report.draft_gate.as_ref().unwrap();
        assert_eq!(gate.verdict, "DOMAIN_PACK_DRAFT_WATCH_NO_AUTHORITY");
        assert!(gate.structure_complete);
        assert!(!gate.authority_granted);
        assert!(!gate.answer_allowed);
        assert_eq!(gate.relation_genericity.status, "WATCH");
    }

    #[test]
    fn domain_gate_never_promotes_candidate_authority() {
        let report = build_domain_gate_report(
            ChatCoreDomainProposalConfig {
                text: "what is wind_setup_compact_active in gravity-saturation-checks?".to_string(),
                profile: Some(PathBuf::from(DEFAULT_PROFILE)),
                memory_root: Some(PathBuf::from(".nanda/linux-active")),
                context_file: None,
                proposal_registries: Vec::new(),
                out: None,
            },
            None,
        )
        .unwrap();

        assert_eq!(report.verdict, "DOMAIN_PACK_DRAFT_WATCH_NO_AUTHORITY");
        assert_eq!(report.domain_id.as_deref(), Some("physics_material_layer"));
        assert_eq!(report.maturity.as_deref(), Some("candidate"));
        assert!(!report.safe_to_answer);
        assert!(!report.safe_to_learn_without_profile);
        assert!(!report.cache_mutated);
        assert!(!report.overlay_written);
        let rights = report.authority_rights.as_ref().unwrap();
        assert!(rights.proposal_only);
        assert!(!rights.answer_allowed);
        assert!(!rights.overlay_write_allowed);
        assert!(!rights.cache_mutation_allowed);
    }
}
