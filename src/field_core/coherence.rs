use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldCoherenceSummary {
    pub coherence: f64,
    pub energy: f64,
    pub foreign_pull: usize,
    pub field_state: String,
    pub field_action: String,
}

impl Default for FieldCoherenceSummary {
    fn default() -> Self {
        Self {
            coherence: 0.0,
            energy: 0.0,
            foreign_pull: 0,
            field_state: "FIELD_UNKNOWN".to_string(),
            field_action: "review".to_string(),
        }
    }
}
