use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldPeakSummary {
    pub target: String,
    pub score: f64,
    pub margin: f64,
    pub state: String,
    pub safe_to_answer: bool,
    pub support_count: usize,
    pub anti_support_count: usize,
}

impl Default for FieldPeakSummary {
    fn default() -> Self {
        Self {
            target: String::new(),
            score: 0.0,
            margin: 0.0,
            state: "WATCH".to_string(),
            safe_to_answer: false,
            support_count: 0,
            anti_support_count: 0,
        }
    }
}
