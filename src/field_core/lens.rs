use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FieldLensSummary {
    pub lenses: Vec<String>,
    pub amplified: Vec<String>,
    pub suppressed: Vec<String>,
    pub explanation: String,
}
