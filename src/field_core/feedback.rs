use serde::{Deserialize, Serialize};

use super::{
    FieldAntiWaveLane, FieldLensKind, FieldLensOperation, FieldPassInput, FieldRecord,
    FieldRecordKind,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FeedbackSummary {
    pub feedback_present: bool,
    pub accepted: usize,
    pub rejected: usize,
    pub watched: usize,
    pub replayable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FieldMemoryDeltaKind {
    ReinforcePeak,
    SuppressShortcut,
    WatchOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FieldMemoryDelta {
    pub id: String,
    pub kind: FieldMemoryDeltaKind,
    pub target: String,
    pub route: Option<String>,
    pub group: Option<String>,
    pub strength: i32,
    pub local_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct FieldMemoryDeltaSummary {
    pub accepted: usize,
    pub rejected: usize,
    pub watched: usize,
    pub deltas: Vec<FieldMemoryDelta>,
    pub replayable_into_next_pass: bool,
}

impl FieldMemoryDelta {
    pub(crate) fn reinforce(target: impl Into<String>, route: Option<String>) -> Self {
        let target = target.into();
        Self {
            id: format!("reinforce:{target}"),
            kind: FieldMemoryDeltaKind::ReinforcePeak,
            target,
            route,
            group: Some("feedback".to_string()),
            strength: 1,
            local_only: true,
        }
    }

    pub(crate) fn suppress(target: impl Into<String>, route: Option<String>) -> Self {
        let target = target.into();
        Self {
            id: format!("suppress:{target}"),
            kind: FieldMemoryDeltaKind::SuppressShortcut,
            target,
            route,
            group: Some("feedback".to_string()),
            strength: 2,
            local_only: true,
        }
    }

    pub(crate) fn watch(target: impl Into<String>, route: Option<String>) -> Self {
        let target = target.into();
        Self {
            id: format!("watch:{target}"),
            kind: FieldMemoryDeltaKind::WatchOnly,
            target,
            route,
            group: Some("feedback".to_string()),
            strength: 0,
            local_only: true,
        }
    }
}

pub(crate) fn summarize_memory_deltas(
    feedback: &FeedbackSummary,
    target: &str,
) -> FieldMemoryDeltaSummary {
    let mut deltas = vec![];
    for _ in 0..feedback.accepted {
        deltas.push(FieldMemoryDelta::reinforce(
            target,
            Some(target.to_string()),
        ));
    }
    for _ in 0..feedback.rejected {
        deltas.push(FieldMemoryDelta::suppress(target, Some(target.to_string())));
    }
    for _ in 0..feedback.watched {
        deltas.push(FieldMemoryDelta::watch(target, Some(target.to_string())));
    }
    FieldMemoryDeltaSummary {
        accepted: feedback.accepted,
        rejected: feedback.rejected,
        watched: feedback.watched,
        replayable_into_next_pass: feedback.replayable || !deltas.is_empty(),
        deltas,
    }
}

pub(crate) fn apply_memory_deltas_to_pass(
    input: &FieldPassInput,
    deltas: &[FieldMemoryDelta],
) -> FieldPassInput {
    let mut next = input.clone();
    for delta in deltas {
        match delta.kind {
            FieldMemoryDeltaKind::ReinforcePeak => {
                next.lenses.push(FieldLensOperation {
                    kind: FieldLensKind::Evidence,
                    label: delta.target.clone(),
                    strength: delta.strength.max(1),
                });
                next.records.push(FieldRecord::synthetic(
                    delta.id.clone(),
                    FieldRecordKind::FeedbackMemory,
                    "feedback",
                    "reinforces",
                    delta.target.clone(),
                    delta.route.clone(),
                    delta.group.clone(),
                ));
            }
            FieldMemoryDeltaKind::SuppressShortcut => {
                next.anti_waves.push(FieldAntiWaveLane {
                    id: delta.id.clone(),
                    target: delta.target.clone(),
                    subject: "feedback".to_string(),
                    relation: "suppresses".to_string(),
                    object: delta.target.clone(),
                    route: delta.route.clone(),
                    group: delta.group.clone(),
                    strength: delta.strength.max(1),
                });
            }
            FieldMemoryDeltaKind::WatchOnly => {}
        }
    }
    next
}
