use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FeedbackSummary {
    pub feedback_present: bool,
    pub accepted: usize,
    pub rejected: usize,
    pub watched: usize,
    pub replayable: bool,
}
