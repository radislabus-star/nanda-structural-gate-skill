//! Persistent wave-memory deltas for dialogue-time learning.
//!
//! This is deliberately not a transcript store. The hot learning unit is a
//! fixed 32-byte delta keyed by intent, route, relation, and subject/object
//! wave ids. The JSON wrapper is cold explain/debug data so we can prove what
//! was written and why.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub(crate) const PERSISTENT_WAVE_MEMORY_VERSION: &str = "llmwave-big-v-next-persistent-wave-memory";

pub(crate) const DELTA_NO_WRITE: &str = "NO_WRITE";
pub(crate) const DELTA_WATCH_TRACE: &str = "WATCH_TRACE";
pub(crate) const DELTA_POSITIVE: &str = "POSITIVE_DELTA";
pub(crate) const DELTA_NEGATIVE: &str = "NEGATIVE_DELTA";
pub(crate) const DELTA_CORRECTION: &str = "CORRECTION_DELTA";
pub(crate) const DELTA_CONFLICT: &str = "CONFLICT_DELTA";

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PersistentWaveDelta32 {
    pub source_id: u32,
    pub target_id: u32,
    pub route_id: u16,
    pub intent_id: u16,
    pub relation_id: u16,
    pub phase_delta: i16,
    pub reinforce_score: i16,
    pub suppress_score: i16,
    pub confidence_delta: i16,
    pub flags: u16,
    pub checksum: u32,
    pub packed_meta: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistentWaveDeltaRecord {
    pub delta_state: String,
    pub packed: PersistentWaveDelta32,
    pub source_prompt: String,
    pub intent: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistentWaveMemoryFile {
    pub version: String,
    pub record_bytes: usize,
    pub record_count: usize,
    pub records: Vec<PersistentWaveDeltaRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PersistentWaveMemorySummary {
    pub path: String,
    pub version: String,
    pub record_bytes: usize,
    pub record_count: usize,
    pub positive_delta_count: usize,
    pub negative_delta_count: usize,
    pub correction_delta_count: usize,
    pub watch_trace_count: usize,
    pub transcript_store: bool,
    pub fixed_wave_delta_records: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PersistentWaveDeltaSpec {
    pub delta_state: String,
    pub source_prompt: String,
    pub intent: String,
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub polarity: String,
    pub reason: String,
    pub strength: i16,
}

#[derive(Debug, Clone)]
pub(crate) struct PersistentWaveMemoryQuery {
    pub intent: String,
    pub anchors: Vec<String>,
    pub route_priors: Vec<String>,
    pub required_routes: Vec<String>,
    pub forbidden_shortcuts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PersistentWaveMemoryEffect {
    pub memory_record_count: usize,
    pub matched_record_count: usize,
    pub positive_matches: usize,
    pub negative_matches: usize,
    pub correction_matches: usize,
    pub reinforcement_delta: i32,
    pub suppression_delta: i32,
    pub learned_negative_lanes_active: bool,
    pub learned_positive_lanes_active: bool,
    pub answer_changed_due_to_wave_memory: bool,
    pub matched_lane_ids: Vec<String>,
    pub learned_answer: Option<PersistentWaveLearnedAnswer>,
    pub state: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PersistentWaveLearnedAnswer {
    pub route: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub confidence_delta: i16,
}

pub(crate) fn empty_memory() -> PersistentWaveMemoryFile {
    PersistentWaveMemoryFile {
        version: PERSISTENT_WAVE_MEMORY_VERSION.to_string(),
        record_bytes: core::mem::size_of::<PersistentWaveDelta32>(),
        record_count: 0,
        records: Vec::new(),
    }
}

pub(crate) fn load_memory(path: &Path) -> Result<PersistentWaveMemoryFile> {
    if !path.exists() {
        return Ok(empty_memory());
    }
    let mut memory: PersistentWaveMemoryFile = serde_json::from_str(
        &fs::read_to_string(path)
            .with_context(|| format!("read persistent wave memory {}", path.display()))?,
    )
    .with_context(|| format!("parse persistent wave memory {}", path.display()))?;
    memory.record_bytes = core::mem::size_of::<PersistentWaveDelta32>();
    memory.record_count = memory.records.len();
    Ok(memory)
}

pub(crate) fn write_memory(path: &Path, memory: &PersistentWaveMemoryFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create memory parent {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(memory).context("serialize persistent wave memory")?,
    )
    .with_context(|| format!("write persistent wave memory {}", path.display()))?;
    Ok(())
}

pub(crate) fn append_delta(
    path: &Path,
    record: PersistentWaveDeltaRecord,
) -> Result<PersistentWaveMemorySummary> {
    let mut memory = load_memory(path)?;
    memory.records.push(record);
    memory.record_count = memory.records.len();
    memory.record_bytes = core::mem::size_of::<PersistentWaveDelta32>();
    write_memory(path, &memory)?;
    Ok(summarize_memory(path, &memory))
}

pub(crate) fn summarize_memory(
    path: &Path,
    memory: &PersistentWaveMemoryFile,
) -> PersistentWaveMemorySummary {
    PersistentWaveMemorySummary {
        path: path.display().to_string(),
        version: memory.version.clone(),
        record_bytes: core::mem::size_of::<PersistentWaveDelta32>(),
        record_count: memory.records.len(),
        positive_delta_count: memory
            .records
            .iter()
            .filter(|record| record.delta_state == DELTA_POSITIVE)
            .count(),
        negative_delta_count: memory
            .records
            .iter()
            .filter(|record| record.delta_state == DELTA_NEGATIVE)
            .count(),
        correction_delta_count: memory
            .records
            .iter()
            .filter(|record| record.delta_state == DELTA_CORRECTION)
            .count(),
        watch_trace_count: memory
            .records
            .iter()
            .filter(|record| record.delta_state == DELTA_WATCH_TRACE)
            .count(),
        transcript_store: false,
        fixed_wave_delta_records: true,
    }
}

pub(crate) fn build_delta_record(spec: PersistentWaveDeltaSpec) -> PersistentWaveDeltaRecord {
    let source_id = stable_id(&spec.subject);
    let target_id = stable_id(&spec.object);
    let route_id = stable_id16(&spec.route);
    let intent_id = stable_id16(&spec.intent);
    let relation_id = stable_id16(&spec.relation);
    let polarity_code = polarity_code(&spec.polarity);
    let state_code = state_code(&spec.delta_state);
    let packed_meta = ((state_code as u32) << 24)
        | ((polarity_code as u32) << 16)
        | ((spec.strength.unsigned_abs() as u32).min(255) << 8)
        | 1;
    let (reinforce_score, suppress_score, phase_delta, confidence_delta) =
        scores_for_delta(&spec.delta_state, spec.strength);
    let flags = flags_for_delta(&spec.delta_state);
    let checksum = checksum([
        source_id,
        target_id,
        route_id as u32,
        intent_id as u32,
        relation_id as u32,
        packed_meta,
    ]);
    PersistentWaveDeltaRecord {
        delta_state: spec.delta_state,
        packed: PersistentWaveDelta32 {
            source_id,
            target_id,
            route_id,
            intent_id,
            relation_id,
            phase_delta,
            reinforce_score,
            suppress_score,
            confidence_delta,
            flags,
            checksum,
            packed_meta,
        },
        source_prompt: spec.source_prompt,
        intent: spec.intent,
        route: spec.route,
        subject: spec.subject,
        relation: spec.relation,
        object: spec.object,
        polarity: spec.polarity,
        reason: spec.reason,
    }
}

pub(crate) fn memory_effect(
    memory: &PersistentWaveMemoryFile,
    query: PersistentWaveMemoryQuery,
) -> PersistentWaveMemoryEffect {
    let mut positive_matches = 0;
    let mut negative_matches = 0;
    let mut correction_matches = 0;
    let mut reinforcement_delta = 0;
    let mut suppression_delta = 0;
    let mut matched_lane_ids = Vec::new();
    let mut learned_answer = None;

    for record in &memory.records {
        if !matches_query(record, &query) {
            continue;
        }
        matched_lane_ids.push(format!(
            "{}:{}:{}:{}",
            record.delta_state, record.intent, record.route, record.subject
        ));
        match record.delta_state.as_str() {
            DELTA_POSITIVE => {
                positive_matches += 1;
                reinforcement_delta += record.packed.reinforce_score as i32;
                if learned_answer.is_none() {
                    learned_answer = Some(PersistentWaveLearnedAnswer {
                        route: record.route.clone(),
                        subject: record.subject.clone(),
                        relation: record.relation.clone(),
                        object: record.object.clone(),
                        confidence_delta: record.packed.confidence_delta,
                    });
                }
            }
            DELTA_NEGATIVE => {
                negative_matches += 1;
                suppression_delta += record.packed.suppress_score as i32;
            }
            DELTA_CORRECTION => {
                correction_matches += 1;
                reinforcement_delta += record.packed.reinforce_score as i32 / 2;
            }
            _ => {}
        }
    }

    let matched_record_count = positive_matches + negative_matches + correction_matches;
    let state = if matched_record_count == 0 {
        "NO_MATCH"
    } else if negative_matches > 0 && positive_matches > 0 {
        "CONTESTED_MEMORY_MATCH"
    } else if negative_matches > 0 {
        "NEGATIVE_MEMORY_MATCH"
    } else {
        "POSITIVE_MEMORY_MATCH"
    };

    PersistentWaveMemoryEffect {
        memory_record_count: memory.records.len(),
        matched_record_count,
        positive_matches,
        negative_matches,
        correction_matches,
        reinforcement_delta,
        suppression_delta,
        learned_negative_lanes_active: negative_matches > 0,
        learned_positive_lanes_active: positive_matches > 0,
        answer_changed_due_to_wave_memory: learned_answer.is_some(),
        matched_lane_ids,
        learned_answer,
        state,
    }
}

pub(crate) fn stable_id(text: &str) -> u32 {
    let mut hash = 0x811c_9dc5_u32;
    for byte in text.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn stable_id16(text: &str) -> u16 {
    (stable_id(text) & 0xffff) as u16
}

fn matches_query(record: &PersistentWaveDeltaRecord, query: &PersistentWaveMemoryQuery) -> bool {
    if record.intent != query.intent {
        return false;
    }

    let route_match = query
        .route_priors
        .iter()
        .any(|route| route == &record.route)
        || query
            .required_routes
            .iter()
            .any(|route| route == &record.route)
        || query
            .forbidden_shortcuts
            .iter()
            .any(|shortcut| shortcut == &record.object || shortcut == &record.subject);
    let anchor_match = query.anchors.iter().any(|anchor| {
        let anchor = anchor.to_ascii_lowercase();
        let subject = record.subject.to_ascii_lowercase();
        let object = record.object.to_ascii_lowercase();
        subject == anchor
            || object == anchor
            || subject.contains(&anchor)
            || object.contains(&anchor)
            || object.ends_with(&format!("/{anchor}"))
    });
    let shortcut_match = query
        .forbidden_shortcuts
        .iter()
        .any(|shortcut| record.object.contains(shortcut) || record.subject.contains(shortcut));

    match record.delta_state.as_str() {
        DELTA_POSITIVE | DELTA_CORRECTION => anchor_match,
        DELTA_NEGATIVE => route_match || shortcut_match || anchor_match,
        _ => route_match || anchor_match || shortcut_match,
    }
}

fn scores_for_delta(delta_state: &str, strength: i16) -> (i16, i16, i16, i16) {
    let strength = strength.clamp(1, 255);
    match delta_state {
        DELTA_POSITIVE => (strength * 8, 0, strength, strength * 4),
        DELTA_NEGATIVE => (0, strength * 8, -strength, -(strength * 4)),
        DELTA_CORRECTION => (strength * 6, strength * 2, strength / 2, strength * 3),
        DELTA_CONFLICT => (0, strength * 4, 0, -strength),
        DELTA_WATCH_TRACE => (strength, strength, 0, 0),
        _ => (0, 0, 0, 0),
    }
}

fn flags_for_delta(delta_state: &str) -> u16 {
    match delta_state {
        DELTA_POSITIVE => 0b0001,
        DELTA_NEGATIVE => 0b0010,
        DELTA_CORRECTION => 0b0100,
        DELTA_CONFLICT => 0b1000,
        DELTA_WATCH_TRACE => 0b1_0000,
        _ => 0,
    }
}

fn state_code(delta_state: &str) -> u8 {
    match delta_state {
        DELTA_POSITIVE => 1,
        DELTA_NEGATIVE => 2,
        DELTA_CORRECTION => 3,
        DELTA_CONFLICT => 4,
        DELTA_WATCH_TRACE => 5,
        _ => 0,
    }
}

fn polarity_code(polarity: &str) -> u8 {
    match polarity {
        "positive" => 1,
        "negative" => 2,
        "correction" => 3,
        "watch" => 4,
        _ => 0,
    }
}

fn checksum(words: [u32; 6]) -> u32 {
    words.into_iter().fold(0x9e37_79b9_u32, |acc, word| {
        acc.rotate_left(5) ^ word.wrapping_mul(0x85eb_ca6b)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_wave_delta_is_32_bytes() {
        assert_eq!(core::mem::size_of::<PersistentWaveDelta32>(), 32);
    }

    #[test]
    fn positive_delta_matches_query_and_changes_answer() {
        let record = build_delta_record(PersistentWaveDeltaSpec {
            delta_state: DELTA_POSITIVE.to_string(),
            source_prompt: "learn accept".to_string(),
            intent: "command_provider".to_string(),
            route: "linux.apt.command.provider".to_string(),
            subject: "foocmd".to_string(),
            relation: "provided by package".to_string(),
            object: "foopkg".to_string(),
            polarity: "positive".to_string(),
            reason: "user accepted route".to_string(),
            strength: 16,
        });
        let memory = PersistentWaveMemoryFile {
            version: PERSISTENT_WAVE_MEMORY_VERSION.to_string(),
            record_bytes: core::mem::size_of::<PersistentWaveDelta32>(),
            record_count: 1,
            records: vec![record],
        };
        let effect = memory_effect(
            &memory,
            PersistentWaveMemoryQuery {
                intent: "command_provider".to_string(),
                anchors: vec!["foocmd".to_string()],
                route_priors: vec!["linux.apt.command.provider".to_string()],
                required_routes: Vec::new(),
                forbidden_shortcuts: Vec::new(),
            },
        );
        assert!(effect.learned_positive_lanes_active);
        assert!(effect.answer_changed_due_to_wave_memory);
        assert_eq!(effect.learned_answer.unwrap().object, "foopkg");
    }
}
