use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct AntiWaveSummary {
    pub active: bool,
    pub lanes: usize,
    pub suppressed_target: Option<String>,
    pub suppression_energy: f64,
    pub local_only: bool,
}
