use serde::Serialize;

use super::nanda_6m;

#[derive(Serialize, Clone)]
pub(crate) struct BignessMetricsContract {
    pub contract_version: &'static str,
    pub required_metrics: Vec<MetricSpec>,
    pub measured_baseline: MeasuredBaseline,
    pub cognition_score_formula: CognitionScoreFormula,
    pub nonlinear_claim_gate: Vec<&'static str>,
}

#[derive(Serialize, Clone)]
pub(crate) struct MetricSpec {
    pub name: &'static str,
    pub unit: &'static str,
    pub direction: &'static str,
    pub required_for_claim: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct MeasuredBaseline {
    pub status: &'static str,
    pub active_core_bytes: usize,
    pub wave_dim: usize,
    pub runtime_focus_triad_capacity: usize,
    pub atlas_facts_total: Option<u64>,
    pub schemas_total: Option<u64>,
    pub operators_total: Option<u64>,
    pub active_schemas: Option<u64>,
    pub active_residuals: Option<u64>,
    pub bytes_per_useful_fact: Option<f64>,
    pub useful_inference_per_mb: Option<f64>,
    pub schema_reuse_ratio: Option<f64>,
    pub residual_saving_ratio: Option<f64>,
    pub role_error_rate: Option<f64>,
    pub false_positive_rate: Option<f64>,
    pub inference_score: Option<f64>,
    pub cognition_score: Option<f64>,
}

#[derive(Serialize, Clone)]
pub(crate) struct CognitionScoreFormula {
    pub positive_terms: Vec<&'static str>,
    pub negative_terms: Vec<&'static str>,
    pub expression: &'static str,
}

pub(crate) fn build_bigness_metrics() -> BignessMetricsContract {
    BignessMetricsContract {
        contract_version: "v159-bigness-metrics",
        required_metrics: vec![
            metric("atlas_facts_total", "facts", "higher_with_quality_gate", true),
            metric("schemas_total", "schemas", "higher_when_reused", true),
            metric("operators_total", "operators", "bounded", true),
            metric("active_core_bytes", "bytes", "bounded", true),
            metric("active_schemas", "schemas", "bounded_by_focus", true),
            metric("active_residuals", "residuals", "lower_after_consolidation", true),
            metric("bytes_per_useful_fact", "bytes_per_fact", "lower_is_better", true),
            metric(
                "useful_inference_per_mb",
                "inferences_per_mb",
                "higher_is_better",
                true,
            ),
            metric("schema_reuse_ratio", "ratio", "higher_is_better", true),
            metric("residual_saving_ratio", "ratio", "higher_is_better", true),
            metric("role_error_rate", "ratio", "lower_is_better", true),
            metric("false_positive_rate", "ratio", "lower_is_better", true),
            metric("inference_score", "score", "higher_is_better", true),
            metric("cognition_score", "score", "higher_is_better", true),
        ],
        measured_baseline: MeasuredBaseline {
            status: "CONTRACT_BASELINE_ONLY_UNMEASURED_CORPUS",
            active_core_bytes: nanda_6m::BUDGET_BYTES,
            wave_dim: nanda_6m::WAVE_DIM,
            runtime_focus_triad_capacity: nanda_6m::RUNTIME_FOCUS_TRIAD_CAPACITY,
            atlas_facts_total: None,
            schemas_total: None,
            operators_total: None,
            active_schemas: None,
            active_residuals: None,
            bytes_per_useful_fact: None,
            useful_inference_per_mb: None,
            schema_reuse_ratio: None,
            residual_saving_ratio: None,
            role_error_rate: None,
            false_positive_rate: None,
            inference_score: None,
            cognition_score: None,
        },
        cognition_score_formula: CognitionScoreFormula {
            positive_terms: vec![
                "recall",
                "inference",
                "role_safety",
                "contradiction_veto",
                "compression_gain",
            ],
            negative_terms: vec!["false_positive"],
            expression:
                "cognition_score=recall+inference+role_safety+contradiction_veto+compression_gain-false_positive",
        },
        nonlinear_claim_gate: vec![
            "bytes_per_useful_fact_must_improve_against_linear_baseline",
            "schema_reuse_ratio_must_rise_with_corpus_scale",
            "residual_saving_ratio_must_show_schema_consolidation",
            "role_error_rate_must_not_regress",
            "false_positive_rate_must_not_regress",
            "inference_score_must_improve_on_heldout_big_cognition_eval",
        ],
    }
}

fn metric(
    name: &'static str,
    unit: &'static str,
    direction: &'static str,
    required_for_claim: bool,
) -> MetricSpec {
    MetricSpec {
        name,
        unit,
        direction,
        required_for_claim,
    }
}
