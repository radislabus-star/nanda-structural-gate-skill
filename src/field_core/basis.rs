use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldBasis {
    pub dimension: usize,
    pub basis_kind: String,
    pub axis_policy: String,
    pub quantization: String,
    pub memory_budget_class: String,
}

impl FieldBasis {
    pub(crate) fn dynamic_1024() -> Self {
        Self {
            dimension: 1024,
            basis_kind: "signed_wave_vector".to_string(),
            axis_policy: "route_group_relation_axes".to_string(),
            quantization: "dynamic_i32_json_boundary".to_string(),
            memory_budget_class: "cold_dynamic".to_string(),
        }
    }

    pub(crate) fn packed_1024() -> Self {
        Self {
            dimension: 1024,
            basis_kind: "packed_wave1024".to_string(),
            axis_policy: "numeric_route_group_axis_ids".to_string(),
            quantization: "fixed_packed_records".to_string(),
            memory_budget_class: "hot_cache_budgeted".to_string(),
        }
    }

    pub(crate) fn cognitive_1024() -> Self {
        Self {
            dimension: 1024,
            basis_kind: "llmwave_big_schema_surface_field".to_string(),
            axis_policy: "l2_surface_l3_schema_axes".to_string(),
            quantization: "mixed_experimental_records".to_string(),
            memory_budget_class: "cold_plus_active_core".to_string(),
        }
    }

    pub(crate) fn unknown() -> Self {
        Self {
            dimension: 0,
            basis_kind: "unknown".to_string(),
            axis_policy: "unknown".to_string(),
            quantization: "unknown".to_string(),
            memory_budget_class: "unknown".to_string(),
        }
    }
}
