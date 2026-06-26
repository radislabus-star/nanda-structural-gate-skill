use crate::*;
use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const SKILL_READINESS_VERSION: &str = "skill-readiness-v1-public-gate";

#[derive(Parser)]
pub(crate) struct SkillReadinessArgs {
    #[arg(default_value = ".")]
    pub(crate) repo: PathBuf,
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: OutputFormat,
}

#[derive(Serialize)]
struct ReadinessCheck {
    check: &'static str,
    status: &'static str,
    evidence: Value,
}

pub(crate) fn cmd(args: SkillReadinessArgs) -> Result<u8> {
    let report = build_report(&args.repo);
    print(&report, &args.format)?;
    if report["public_v1_ready"].as_bool().unwrap_or(false) {
        Ok(EXIT_PASS)
    } else {
        Ok(EXIT_WATCH)
    }
}

fn build_report(repo: &Path) -> Value {
    let doctor = doctor_value();
    let field_audit = field_audit_summary();
    let structural_capacity = structural_capacity_summary();
    let claim_gates = claim_gate_summary();
    let packaging = packaging_summary(repo);
    let docs = docs_summary(repo);

    let checks = vec![
        check(
            "doctor",
            doctor["healthy"].as_bool().unwrap_or(false),
            json!({
                "healthy": doctor["healthy"],
                "route_trap": doctor["route_trap"],
                "noisy": doctor["noisy"]
            }),
        ),
        check(
            "field_core_sole_engine",
            field_audit["field_core_as_sole_engine"]
                .as_bool()
                .unwrap_or(false)
                && field_audit["local_physics_copies_allowed"].as_bool() == Some(false),
            field_audit,
        ),
        check(
            "pattern16_skill_admission",
            structural_capacity["final_gate_passed"]
                .as_bool()
                .unwrap_or(false)
                && structural_capacity["fixed_1024_only"]
                    .as_bool()
                    .unwrap_or(false),
            structural_capacity,
        ),
        check(
            "claim_boundaries",
            claim_gates["field_core_allowed"].as_bool().unwrap_or(false)
                && claim_gates["llm_ready_blocked"].as_bool().unwrap_or(false)
                && claim_gates["nonlinear_memory_blocked"]
                    .as_bool()
                    .unwrap_or(false),
            claim_gates,
        ),
        check(
            "packaging",
            packaging["has_skill_md"].as_bool().unwrap_or(false)
                && packaging["has_nanda_binary_wrapper"]
                    .as_bool()
                    .unwrap_or(false)
                && packaging["has_skill_readiness_wrapper"]
                    .as_bool()
                    .unwrap_or(false),
            packaging,
        ),
        check(
            "documentation",
            docs["readme_mentions_skill_readiness"]
                .as_bool()
                .unwrap_or(false)
                && docs["commands_mentions_skill_readiness"]
                    .as_bool()
                    .unwrap_or(false)
                && docs["runtime_skill_mentions_skill_readiness"]
                    .as_bool()
                    .unwrap_or(false),
            docs,
        ),
    ];

    let blockers = checks
        .iter()
        .filter(|item| item.status != "PASS")
        .map(|item| item.check)
        .collect::<Vec<_>>();
    let public_v1_ready = blockers.is_empty();
    json!({
        "mode": "nanda-skill-readiness",
        "version": SKILL_READINESS_VERSION,
        "repo": repo.display().to_string(),
        "verdict": if public_v1_ready { "PUBLIC_V1_READY" } else { "PUBLIC_V1_REVIEW" },
        "public_v1_ready": public_v1_ready,
        "checks": checks,
        "blockers": blockers,
        "claim_boundary": {
            "structural_gate_ready": public_v1_ready,
            "field_core_as_sole_engine": true,
            "broad_chat_llm_ready": false,
            "nonlinear_memory_proven": false,
            "hardware_cache_residency_counter_proven": false,
            "safe_claim": "nanda-structural-gate is ready as a public structural gate only when this readiness report is PUBLIC_V1_READY. It is not a broad chat LLM or global nonlinear-memory proof."
        },
        "recommended_next": if public_v1_ready {
            "Use nanda-skill-readiness before release and nanda dogfood/map-code before risky edits."
        } else {
            "Fix blockers before calling the skill public-v1 ready."
        }
    })
}

fn field_audit_summary() -> Value {
    let structural_cutover_suite = field_adapters::field_cutover_report(
        field_adapters::structural_standard_cutover_cases().unwrap_or_default(),
        "structural-standard",
    );
    let structural_cutover_suite_pass = structural_cutover_suite["acceptance"]
        ["structural_cutover_suite_pass"]
        .as_bool()
        .unwrap_or(false);
    let audit = field_core::build_sole_engine_audit(structural_cutover_suite_pass);
    json!({
        "version": audit.version,
        "field_core_as_sole_engine": audit.field_core_as_sole_engine,
        "big_pipelines": audit.big_pipelines,
        "field_core_backed_pipelines": audit.field_core_backed_pipelines,
        "local_physics_copies_allowed": audit.local_physics_copies_allowed,
        "blockers": audit.blockers
    })
}

fn structural_capacity_summary() -> Value {
    let report = llmwave_big::structural_capacity::build_structural_capacity_report(
        llmwave_big::structural_capacity::StructuralCapacityConfig {
            seed: 13,
            seeds: 8,
            noise_edges: 16,
            hot_budget_bytes: 6 * 1024 * 1024,
            noise_profile:
                llmwave_big::structural_capacity::StructuralCapacityNoiseProfile::SkillAdmission,
        },
    );
    json!({
        "capacity_profile_checked": "skill-admission",
        "default_capacity_profile": "default",
        "default_profile_mismatch_is_not_blocker": true,
        "readiness_uses_skill_admission_profile": true,
        "admission_command": "nanda-llmwave-big structural-capacity --noise-profile skill-admission --format json",
        "default_command": "nanda-llmwave-big structural-capacity --format json",
        "verdict": report.verdict,
        "fixed_1024_only": report.gates.fixed_1024_only,
        "pattern16_macro_cell": report.gates.pattern16_macro_cell,
        "final_gate_passed": report.gates.final_gate_passed,
        "skill_admission_noise_profile": report.gates.skill_admission_noise_profile,
        "skill_admission_noise_pressure": report.gates.skill_admission_noise_pressure,
        "single_peak_under_noise": report.gates.single_peak_under_noise,
        "false_accept_rate": report.metrics.false_accept_rate,
        "false_negative_rate": report.metrics.false_negative_rate,
        "hot_bytes": report.memory.hot_bytes,
        "hot_budget_bytes": report.memory.hot_budget_bytes,
        "smaller_pattern_modes_available": report.claim_boundary.smaller_pattern_modes_available,
        "smaller_pattern_shapes_available": report.claim_boundary.smaller_pattern_shapes_available
    })
}

fn claim_gate_summary() -> Value {
    let field_core = llmwave_big::readiness::build_claim_gate_report(
        llmwave_big::readiness::ClaimGateKind::FieldCoreSoleEngine,
    );
    let llm = llmwave_big::readiness::build_claim_gate_report(
        llmwave_big::readiness::ClaimGateKind::LlmReady,
    );
    let nonlinear = llmwave_big::readiness::build_claim_gate_report(
        llmwave_big::readiness::ClaimGateKind::NonlinearMemory,
    );
    json!({
        "field_core_verdict": field_core.verdict,
        "field_core_allowed": field_core.allowed,
        "llm_ready_verdict": llm.verdict,
        "llm_ready_blocked": !llm.allowed,
        "nonlinear_memory_verdict": nonlinear.verdict,
        "nonlinear_memory_blocked": !nonlinear.allowed
    })
}

fn packaging_summary(repo: &Path) -> Value {
    json!({
        "has_skill_md": repo.join("nanda-structural-gate/SKILL.md").is_file(),
        "has_nanda_binary_wrapper": repo.join("nanda-structural-gate/scripts/nanda-check").is_file(),
        "has_skill_readiness_wrapper": repo.join("nanda-structural-gate/scripts/nanda-skill-readiness").is_file(),
        "install_local_script": repo.join("scripts/install-local.sh").is_file()
    })
}

fn docs_summary(repo: &Path) -> Value {
    json!({
        "readme_mentions_skill_readiness": contains(repo.join("README.md"), "nanda-skill-readiness"),
        "commands_mentions_skill_readiness": contains(repo.join("COMMANDS.md"), "nanda-skill-readiness"),
        "runtime_skill_mentions_skill_readiness": contains(repo.join("nanda-structural-gate/SKILL.md"), "nanda-skill-readiness")
    })
}

fn contains(path: PathBuf, needle: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|content| content.contains(needle))
        .unwrap_or(false)
}

fn check(check: &'static str, passed: bool, evidence: Value) -> ReadinessCheck {
    ReadinessCheck {
        check,
        status: if passed { "PASS" } else { "WATCH" },
        evidence,
    }
}

fn print(out: &Value, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(out)?),
        OutputFormat::Text => {
            println!("verdict: {}", out["verdict"].as_str().unwrap_or(""));
            println!(
                "public_v1_ready: {}",
                out["public_v1_ready"].as_bool().unwrap_or(false)
            );
            println!("blockers: {}", out["blockers"]);
        }
        OutputFormat::Md => {
            println!("# NANDA Skill Readiness\n");
            println!("- verdict: `{}`", out["verdict"].as_str().unwrap_or(""));
            println!(
                "- public_v1_ready: `{}`",
                out["public_v1_ready"].as_bool().unwrap_or(false)
            );
            println!("- blockers: `{}`", out["blockers"]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_report_keeps_llm_and_nonlinear_claims_closed() {
        let report = build_report(Path::new("."));

        assert_eq!(report["mode"], "nanda-skill-readiness");
        assert_eq!(report["claim_boundary"]["broad_chat_llm_ready"], false);
        assert_eq!(report["claim_boundary"]["nonlinear_memory_proven"], false);
        assert_eq!(report["checks"][2]["evidence"]["fixed_1024_only"], true);
    }
}
