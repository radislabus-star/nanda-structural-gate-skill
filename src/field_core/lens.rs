use serde::{Deserialize, Serialize};

use super::FieldVector1024;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FieldLensSummary {
    pub lenses: Vec<String>,
    pub amplified: Vec<String>,
    pub suppressed: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FieldLensKind {
    Route,
    Group,
    Role,
    Polarity,
    Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FieldLensOperation {
    pub kind: FieldLensKind,
    pub label: String,
    pub strength: i32,
}

pub(crate) fn apply_field_lens(
    field: &FieldVector1024,
    operation: &FieldLensOperation,
) -> FieldVector1024 {
    let lens_vector =
        FieldVector1024::from_label(&format!("lens:{:?}:{}", operation.kind, operation.label));
    let mut focused = field.clone();
    focused.bundle_scaled(&lens_vector, operation.strength.max(1));
    focused
}

pub(crate) fn apply_lens_chain(
    field: &FieldVector1024,
    operations: &[FieldLensOperation],
) -> FieldVector1024 {
    operations.iter().fold(field.clone(), |current, operation| {
        apply_field_lens(&current, operation)
    })
}
