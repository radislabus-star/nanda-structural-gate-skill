use serde::{Deserialize, Serialize};

use super::{FIELD_COMPUTE_VERSION, FIELD_CORE_VERSION, FIELD_PASS_VERSION, FIELD_RUNTIME_VERSION};

pub(crate) const FIELD_SOLE_ENGINE_VERSION: &str = "unified-field-sole-engine-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldOperationOwner {
    pub operation: &'static str,
    pub owner: &'static str,
    pub contract: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldPipelineConsumer {
    pub pipeline: &'static str,
    pub family: &'static str,
    pub route: &'static str,
    pub required: bool,
    pub big_pipeline: bool,
    pub entrypoint: &'static str,
    pub evidence: &'static str,
    pub uses_field_pass: bool,
    pub uses_field_runtime: bool,
    pub uses_field_engine_policy: bool,
    pub uses_shared_lens: bool,
    pub uses_shared_anti_wave: bool,
    pub local_physics_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldSoleEngineAudit {
    pub version: &'static str,
    pub field_core_version: &'static str,
    pub field_compute_version: &'static str,
    pub field_pass_version: &'static str,
    pub field_runtime_version: &'static str,
    pub policy_owner: &'static str,
    pub operation_owners: Vec<FieldOperationOwner>,
    pub pipeline_consumers: Vec<FieldPipelineConsumer>,
    pub blockers: Vec<&'static str>,
    pub big_pipelines: usize,
    pub field_core_backed_pipelines: usize,
    pub local_physics_copies_allowed: bool,
    pub field_core_as_sole_engine: bool,
    pub llm_ready: bool,
    pub nonlinear_memory_proven: bool,
    pub cache_only_execution_proven: bool,
    pub read_as: &'static str,
}

pub(crate) fn build_sole_engine_audit(structural_cutover_suite_pass: bool) -> FieldSoleEngineAudit {
    let pipeline_consumers = pipeline_consumers(structural_cutover_suite_pass);
    let blockers = sole_engine_blockers(&pipeline_consumers);
    let big_pipelines = pipeline_consumers
        .iter()
        .filter(|consumer| consumer.big_pipeline)
        .count();
    let field_core_backed_pipelines = pipeline_consumers
        .iter()
        .filter(|consumer| consumer.big_pipeline && consumer_is_field_core_backed(consumer))
        .count();
    let local_physics_copies_allowed = pipeline_consumers
        .iter()
        .any(|consumer| consumer.required && consumer.local_physics_allowed);
    let field_core_as_sole_engine = big_pipelines > 0
        && big_pipelines == field_core_backed_pipelines
        && blockers.is_empty()
        && !local_physics_copies_allowed;

    FieldSoleEngineAudit {
        version: FIELD_SOLE_ENGINE_VERSION,
        field_core_version: FIELD_CORE_VERSION,
        field_compute_version: FIELD_COMPUTE_VERSION,
        field_pass_version: FIELD_PASS_VERSION,
        field_runtime_version: FIELD_RUNTIME_VERSION,
        policy_owner: "field_core::engine::FieldEngineDecision",
        operation_owners: operation_owners(),
        pipeline_consumers,
        blockers,
        big_pipelines,
        field_core_backed_pipelines,
        local_physics_copies_allowed,
        field_core_as_sole_engine,
        llm_ready: false,
        nonlinear_memory_proven: false,
        cache_only_execution_proven: false,
        read_as:
            "Sole-engine here means one shared field physics owner for peak/coherence/lens/anti-wave/verdict/readout across registered big pipelines. It is not an LLM, nonlinear-memory, or hardware-cache proof.",
    }
}

fn operation_owners() -> Vec<FieldOperationOwner> {
    vec![
        FieldOperationOwner {
            operation: "vector_basis",
            owner: "field_core::vector::FieldVector1024",
            contract: "all field families project into one 1024-dimensional signed wave basis",
        },
        FieldOperationOwner {
            operation: "record_projection",
            owner: "field_core::pass::FieldRecord",
            contract: "domain objects enter the field as typed records before scoring",
        },
        FieldOperationOwner {
            operation: "lens_chain",
            owner: "field_core::lens::apply_lens_chain",
            contract: "route/group/role/polarity/evidence focusing uses one lens operation path",
        },
        FieldOperationOwner {
            operation: "anti_wave",
            owner: "field_core::anti_wave::apply_anti_wave",
            contract: "false shortcuts are suppressed through one anti-wave operation path",
        },
        FieldOperationOwner {
            operation: "peak_detection",
            owner: "field_core::peak::detect_field_peak",
            contract: "candidate selection uses one peak detector and margin contract",
        },
        FieldOperationOwner {
            operation: "coherence",
            owner: "field_core::coherence::summarize_field_coherence",
            contract: "focused/thin/contested/reversed states use one coherence state machine",
        },
        FieldOperationOwner {
            operation: "verdict",
            owner: "field_core::pass::run_field_pass",
            contract: "PASS/WATCH/VETO and safe_to_answer derive from FieldPassReport",
        },
        FieldOperationOwner {
            operation: "runtime_dual_run",
            owner: "field_core::runtime::FieldRuntimeDualRun",
            contract: "legacy/domain outputs are compared against field-core before cutover",
        },
    ]
}

fn pipeline_consumers(structural_cutover_suite_pass: bool) -> Vec<FieldPipelineConsumer> {
    vec![
        FieldPipelineConsumer {
            pipeline: "structural-search",
            family: "structural",
            route: "search-proof-gate",
            required: true,
            big_pipeline: true,
            entrypoint: "field_core::structural_dual_run_from_search",
            evidence: "nanda-field-cutover --suite structural-standard",
            uses_field_pass: structural_cutover_suite_pass,
            uses_field_runtime: true,
            uses_field_engine_policy: true,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
        FieldPipelineConsumer {
            pipeline: "packed-hot-runtime",
            family: "packed",
            route: "pack6m-active-runtime",
            required: true,
            big_pipeline: true,
            entrypoint: "field_core::packed_dual_run_from_pack",
            evidence: "packed field record view plus packed-field-engine-guard-v1",
            uses_field_pass: true,
            uses_field_runtime: true,
            uses_field_engine_policy: true,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
        FieldPipelineConsumer {
            pipeline: "llmwave-big-cognitive",
            family: "cognitive",
            route: "llmwave-big-report",
            required: true,
            big_pipeline: true,
            entrypoint: "field_core::cognitive_dual_run_from_report",
            evidence: "llmwave_big::report::with_unified_field",
            uses_field_pass: true,
            uses_field_runtime: true,
            uses_field_engine_policy: true,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
        FieldPipelineConsumer {
            pipeline: "pattern16-structural-capacity",
            family: "cognitive",
            route: "structural-capacity",
            required: true,
            big_pipeline: true,
            entrypoint: "structural_capacity::run_pattern16_field_pass",
            evidence: "lens_admission uses FieldPassInput and field_core::run_field_pass",
            uses_field_pass: true,
            uses_field_runtime: false,
            uses_field_engine_policy: false,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
        FieldPipelineConsumer {
            pipeline: "llmwave-lens-scan",
            family: "cognitive",
            route: "lens-scan",
            required: true,
            big_pipeline: true,
            entrypoint: "lens_scan::field_core_admission",
            evidence: "LensScanReport.field_core_admission is FieldPassReport",
            uses_field_pass: true,
            uses_field_runtime: false,
            uses_field_engine_policy: false,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
        FieldPipelineConsumer {
            pipeline: "llmwave-mature-anti-wave",
            family: "cognitive",
            route: "mature-anti-wave",
            required: true,
            big_pipeline: true,
            entrypoint: "mature_anti_wave::field_core_admission",
            evidence: "MatureAntiWaveReport.field_core_admission is FieldPassReport",
            uses_field_pass: true,
            uses_field_runtime: false,
            uses_field_engine_policy: false,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
        FieldPipelineConsumer {
            pipeline: "memory-feedback",
            family: "cognitive",
            route: "feedback-memory",
            required: true,
            big_pipeline: true,
            entrypoint: "field_core::feedback::apply_memory_deltas_to_pass",
            evidence: "UnifiedFieldReport.memory_delta replays feedback into the next FieldPassInput",
            uses_field_pass: true,
            uses_field_runtime: false,
            uses_field_engine_policy: false,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
        FieldPipelineConsumer {
            pipeline: "skill-repository-guard",
            family: "structural",
            route: "agent-guard",
            required: true,
            big_pipeline: true,
            entrypoint: "nanda-field-audit + nanda-dogfood route atlas",
            evidence: "agent-facing gate consumes field audit, dogfood, map-code, guard-action, and guard-diff",
            uses_field_pass: true,
            uses_field_runtime: true,
            uses_field_engine_policy: true,
            uses_shared_lens: true,
            uses_shared_anti_wave: true,
            local_physics_allowed: false,
        },
    ]
}

fn sole_engine_blockers(consumers: &[FieldPipelineConsumer]) -> Vec<&'static str> {
    let mut blockers = vec![];
    if consumers
        .iter()
        .any(|consumer| consumer.required && !consumer.uses_field_pass)
    {
        blockers.push("required_pipeline_without_field_pass");
    }
    if consumers.iter().any(|consumer| {
        consumer.required && (!consumer.uses_shared_lens || !consumer.uses_shared_anti_wave)
    }) {
        blockers.push("required_pipeline_without_shared_lens_or_anti_wave");
    }
    if consumers
        .iter()
        .any(|consumer| consumer.required && consumer.local_physics_allowed)
    {
        blockers.push("local_physics_copy_allowed");
    }
    blockers
}

fn consumer_is_field_core_backed(consumer: &FieldPipelineConsumer) -> bool {
    consumer.required
        && consumer.uses_field_pass
        && consumer.uses_shared_lens
        && consumer.uses_shared_anti_wave
        && !consumer.local_physics_allowed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sole_engine_audit_requires_every_big_pipeline_to_use_field_core() {
        let audit = build_sole_engine_audit(true);

        assert_eq!(audit.version, FIELD_SOLE_ENGINE_VERSION);
        assert_eq!(audit.big_pipelines, audit.field_core_backed_pipelines);
        assert!(audit.field_core_as_sole_engine);
        assert!(!audit.llm_ready);
        assert!(!audit.nonlinear_memory_proven);
        assert!(audit
            .pipeline_consumers
            .iter()
            .any(
                |consumer| consumer.pipeline == "pattern16-structural-capacity"
                    && consumer.uses_field_pass
            ));
        assert!(audit
            .pipeline_consumers
            .iter()
            .all(|consumer| !consumer.local_physics_allowed));
    }

    #[test]
    fn sole_engine_audit_blocks_when_structural_cutover_suite_is_not_passed() {
        let audit = build_sole_engine_audit(false);

        assert!(!audit.field_core_as_sole_engine);
        assert!(audit
            .blockers
            .contains(&"required_pipeline_without_field_pass"));
    }
}
