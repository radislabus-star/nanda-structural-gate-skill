use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FieldQuerySummary {
    pub source: String,
    pub text: Option<String>,
    pub requested_axes: Vec<String>,
    pub signature: Option<String>,
}
