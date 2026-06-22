use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FieldRecordSummary {
    pub records: usize,
    pub routes: usize,
    pub groups: usize,
    pub schemas: usize,
    pub surfaces: usize,
    pub evidence_refs: usize,
}
